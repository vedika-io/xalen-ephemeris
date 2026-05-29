use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Type of angular relationship between two celestial bodies.
pub enum AspectType {
    Conjunction,
    Opposition,
    Trine,
    Square,
    Sextile,
    SemiSextile,
    Quincunx,
    SemiSquare,
    Sesquiquadrate,
    Quintile,
    BiQuintile,
}

impl AspectType {
    /// Return the exact angle in degrees for this aspect type.
    pub fn angle_deg(&self) -> f64 {
        match self {
            AspectType::Conjunction => 0.0,
            AspectType::Opposition => 180.0,
            AspectType::Trine => 120.0,
            AspectType::Square => 90.0,
            AspectType::Sextile => 60.0,
            AspectType::SemiSextile => 30.0,
            AspectType::Quincunx => 150.0,
            AspectType::SemiSquare => 45.0,
            AspectType::Sesquiquadrate => 135.0,
            AspectType::Quintile => 72.0,
            AspectType::BiQuintile => 144.0,
        }
    }

    /// Return the default orb (tolerance) in degrees for this aspect.
    pub fn default_orb_deg(&self) -> f64 {
        match self {
            AspectType::Conjunction | AspectType::Opposition => 8.0,
            AspectType::Trine | AspectType::Square => 7.0,
            AspectType::Sextile => 5.0,
            AspectType::SemiSextile | AspectType::Quincunx => 2.0,
            AspectType::SemiSquare | AspectType::Sesquiquadrate => 2.0,
            AspectType::Quintile | AspectType::BiQuintile => 1.5,
        }
    }

    /// Returns `true` for Ptolemaic (major) aspects.
    pub fn is_major(&self) -> bool {
        matches!(
            self,
            AspectType::Conjunction
                | AspectType::Opposition
                | AspectType::Trine
                | AspectType::Square
                | AspectType::Sextile
        )
    }

    /// The five major (Ptolemaic) aspect types.
    pub const MAJOR: &[AspectType] = &[
        AspectType::Conjunction,
        AspectType::Sextile,
        AspectType::Square,
        AspectType::Trine,
        AspectType::Opposition,
    ];

    /// All supported aspect types including minor aspects.
    pub const ALL: &[AspectType] = &[
        AspectType::Conjunction,
        AspectType::SemiSextile,
        AspectType::SemiSquare,
        AspectType::Sextile,
        AspectType::Quintile,
        AspectType::Square,
        AspectType::Trine,
        AspectType::Sesquiquadrate,
        AspectType::BiQuintile,
        AspectType::Quincunx,
        AspectType::Opposition,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Whether an aspect is applying (tightening), separating, or exact.
pub enum AspectDirection {
    Applying,
    Separating,
    Exact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A detected aspect between two bodies with orb and direction.
pub struct Aspect {
    pub aspect_type: AspectType,
    pub body1: String,
    pub body2: String,
    pub orb_deg: f64,
    pub direction: AspectDirection,
    pub exact_deg: f64,
}

/// Compute the shortest angular distance between two longitudes in degrees.
pub fn angular_distance(lon1_deg: f64, lon2_deg: f64) -> f64 {
    let diff = (lon2_deg - lon1_deg).rem_euclid(360.0);
    if diff > 180.0 { 360.0 - diff } else { diff }
}

/// Check if two bodies form a specific aspect within the given orb.
pub fn find_aspect(
    lon1_deg: f64,
    lon2_deg: f64,
    speed1: f64,
    speed2: f64,
    aspects_to_check: &[AspectType],
    orb_multiplier: f64,
) -> Option<Aspect> {
    let dist = angular_distance(lon1_deg, lon2_deg);

    for &aspect_type in aspects_to_check {
        let target = aspect_type.angle_deg();
        let orb = aspect_type.default_orb_deg() * orb_multiplier;
        let diff = (dist - target).abs();

        if diff <= orb {
            let direction = if diff < 0.01 {
                AspectDirection::Exact
            } else {
                let _relative_speed = speed2 - speed1;
                let future_dist =
                    angular_distance(lon1_deg + speed1 * 0.1, lon2_deg + speed2 * 0.1);
                let future_diff = (future_dist - target).abs();
                if future_diff < diff {
                    AspectDirection::Applying
                } else {
                    AspectDirection::Separating
                }
            };

            return Some(Aspect {
                aspect_type,
                body1: String::new(),
                body2: String::new(),
                orb_deg: diff,
                direction,
                exact_deg: dist,
            });
        }
    }
    None
}

/// Find all aspects between two bodies across all aspect types.
pub fn find_all_aspects(
    positions: &[(String, f64, f64)], // (name, longitude_deg, speed_deg_day)
    aspects_to_check: &[AspectType],
) -> Vec<Aspect> {
    let mut results = Vec::new();
    for i in 0..positions.len() {
        for j in (i + 1)..positions.len() {
            if let Some(mut asp) = find_aspect(
                positions[i].1,
                positions[j].1,
                positions[i].2,
                positions[j].2,
                aspects_to_check,
                1.0,
            ) {
                asp.body1 = positions[i].0.clone();
                asp.body2 = positions[j].0.clone();
                results.push(asp);
            }
        }
    }
    results
}

/// Find all Julian Dates when a transiting planet forms an exact aspect
/// to a fixed natal longitude.
///
/// This uses a coarse scan + bisection refinement strategy (same approach as
/// `xalen_ephem::event_search::find_crossing`).
///
/// * `natal_lon` — the natal planet's ecliptic longitude in degrees (fixed).
/// * `transit_fn` — a closure that returns the transiting planet's longitude
///   at a given JD: `transit_fn(jd) -> longitude_deg`.
/// * `aspect` — the aspect angle in degrees (0 = conjunction, 180 = opposition,
///   120 = trine, etc.).
/// * `jd_start` / `jd_end` — search window (Julian Dates).
///
/// Returns a `Vec<f64>` of JDs where the aspect is exact (to ~1e-8 day
/// precision, roughly 1 ms).
pub fn find_transit_aspect(
    natal_lon: f64,
    transit_fn: impl Fn(f64) -> f64,
    aspect: f64,
    jd_start: f64,
    jd_end: f64,
) -> Vec<f64> {
    if jd_start >= jd_end {
        return Vec::new();
    }

    // The aspect is exact when angular_distance(transit_lon, natal_lon) == aspect.
    // We define g(jd) = angular_distance(transit_fn(jd), natal_lon) - aspect.
    // Exact aspect happens at g(jd) == 0.
    //
    // However angular_distance is always [0, 180], so sign-change detection works
    // for most aspects.  For conjunction (aspect=0) and opposition (aspect=180)
    // there is an additional complication because g() can touch zero without
    // crossing it, but the bisection still converges because we check both
    // sign-changes and close approaches.
    //
    // Step size: 0.5 day catches even lunar transits (~13 deg/day).
    let step = 0.5_f64;
    let mut results = Vec::new();
    let mut jd = jd_start;

    let g = |t: f64| -> f64 { angular_distance(transit_fn(t), natal_lon) - aspect };

    let mut prev = g(jd);

    while jd < jd_end {
        jd += step;
        let curr = g(jd);

        // Sign change or very close approach
        if prev * curr < 0.0 || curr.abs() < 0.01 {
            // Bisect to refine
            if let Some(exact_jd) = bisect_aspect_crossing(&g, jd - step, jd, 60) {
                // De-duplicate: skip if we already found a crossing within 0.5 day
                if results.last().is_none_or(|&last: &f64| (exact_jd - last).abs() > 0.5) {
                    results.push(exact_jd);
                }
            }
        }
        prev = curr;
    }
    results
}

/// Bisection helper for transit aspect search.
fn bisect_aspect_crossing(g: &impl Fn(f64) -> f64, mut lo: f64, mut hi: f64, max_iter: u32) -> Option<f64> {
    let g_lo = g(lo);
    let g_hi = g(hi);

    // If both endpoints have the same sign and neither is very close to zero,
    // this might be a touch-without-crossing near conjunction/opposition.
    // Still try: if one endpoint is close, bisect toward it.
    if g_lo * g_hi > 0.0 && g_lo.abs() > 0.5 && g_hi.abs() > 0.5 {
        return None;
    }

    for _ in 0..max_iter {
        let mid = (lo + hi) / 2.0;
        let g_mid = g(mid);

        if g_mid.abs() < 1e-8 || (hi - lo) < 1e-10 {
            return Some(mid);
        }

        // Prefer the interval containing the sign change
        if g(lo) * g_mid <= 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    Some((lo + hi) / 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angular_distance_basic() {
        assert!((angular_distance(10.0, 130.0) - 120.0).abs() < 0.001);
        assert!((angular_distance(350.0, 10.0) - 20.0).abs() < 0.001);
        assert!((angular_distance(0.0, 180.0) - 180.0).abs() < 0.001);
    }

    #[test]
    fn conjunction_detected() {
        let asp = find_aspect(100.0, 103.0, 1.0, 0.5, AspectType::MAJOR, 1.0);
        assert!(asp.is_some());
        assert_eq!(asp.unwrap().aspect_type, AspectType::Conjunction);
    }

    #[test]
    fn opposition_detected() {
        let asp = find_aspect(10.0, 192.0, 1.0, 0.5, AspectType::MAJOR, 1.0);
        assert!(asp.is_some());
        assert_eq!(asp.unwrap().aspect_type, AspectType::Opposition);
    }

    #[test]
    fn trine_detected() {
        let asp = find_aspect(30.0, 151.0, 0.9, 0.5, AspectType::MAJOR, 1.0);
        assert!(asp.is_some());
        assert_eq!(asp.unwrap().aspect_type, AspectType::Trine);
    }

    #[test]
    fn no_aspect_when_out_of_orb() {
        let asp = find_aspect(10.0, 55.0, 1.0, 0.5, AspectType::MAJOR, 1.0);
        assert!(asp.is_none());
    }

    #[test]
    fn applying_vs_separating() {
        // Faster planet approaching exact conjunction
        let asp = find_aspect(100.0, 105.0, 1.5, 0.5, AspectType::MAJOR, 1.0).unwrap();
        assert_eq!(asp.direction, AspectDirection::Applying);

        // Faster planet moving away from conjunction
        let asp2 = find_aspect(100.0, 105.0, 0.5, 1.5, AspectType::MAJOR, 1.0).unwrap();
        assert_eq!(asp2.direction, AspectDirection::Separating);
    }

    #[test]
    fn find_all_aspects_multi() {
        let positions = vec![
            ("Sun".into(), 10.0, 1.0),
            ("Moon".into(), 130.0, 13.0),
            ("Mars".into(), 100.0, 0.5),
        ];
        let aspects = find_all_aspects(&positions, AspectType::MAJOR);
        assert!(!aspects.is_empty());
    }

    #[test]
    fn minor_aspects() {
        let asp = find_aspect(10.0, 41.0, 1.0, 0.5, AspectType::ALL, 1.0);
        assert!(asp.is_some());
        assert_eq!(asp.unwrap().aspect_type, AspectType::SemiSextile);
    }

    // -----------------------------------------------------------------------
    // find_transit_aspect tests
    // -----------------------------------------------------------------------

    #[test]
    fn transit_conjunction_linear() {
        // Transit planet moves 1 deg/day from 0°, natal planet at 30°.
        // Conjunction (aspect=0) at jd=30.
        let results = find_transit_aspect(30.0, |jd| jd * 1.0, 0.0, 0.0, 60.0);
        assert!(
            !results.is_empty(),
            "Should find at least one conjunction"
        );
        assert!(
            (results[0] - 30.0).abs() < 0.01,
            "Conjunction should be near jd=30, got {}",
            results[0]
        );
    }

    #[test]
    fn transit_opposition_linear() {
        // Transit planet at (jd * 1.0) deg, natal at 30°.
        // Opposition (180°) when transit_lon = 210 → jd=210.
        let results = find_transit_aspect(30.0, |jd| jd * 1.0, 180.0, 200.0, 220.0);
        assert!(!results.is_empty(), "Should find opposition");
        assert!(
            (results[0] - 210.0).abs() < 0.01,
            "Opposition should be near jd=210, got {}",
            results[0]
        );
    }

    #[test]
    fn transit_trine_linear() {
        // Trine (120°) from natal at 0°: when transit = 120 or 240.
        let results = find_transit_aspect(0.0, |jd| jd * 1.0, 120.0, 100.0, 260.0);
        assert!(
            results.len() >= 2,
            "Should find at least 2 trines, got {}",
            results.len()
        );
    }

    #[test]
    fn transit_no_results_empty_window() {
        let results = find_transit_aspect(30.0, |jd| jd, 0.0, 50.0, 50.0);
        assert!(results.is_empty(), "Empty window should return no results");
    }

    #[test]
    fn transit_wrapping_conjunction() {
        // Transit crosses 360/0 boundary: natal at 5°, transit starts at 350°
        // and gains 1 deg/day.  Conjunction near jd=15 (350+15=365→5°).
        let results = find_transit_aspect(
            5.0,
            |jd| (350.0 + jd).rem_euclid(360.0),
            0.0,
            0.0,
            30.0,
        );
        assert!(!results.is_empty(), "Should find conjunction across 360/0 wrap");
        assert!(
            (results[0] - 15.0).abs() < 1.0,
            "Conjunction should be near jd=15, got {}",
            results[0]
        );
    }
}
