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
///   120 = trine, etc.). Angles outside [0, 180] are folded into that range.
/// * `jd_start` / `jd_end` — search window (Julian Dates).
///
/// Returns the JDs in `[jd_start, jd_end]` where the aspect is exact, in
/// ascending order, refined to the f64 resolution of a Julian Date (~5e-10
/// day, roughly 40 µs).
///
/// A body that reaches the exact degree and turns retrograde on it without
/// crossing (a tangent touch) is not reported, and the body must move less
/// than 90° per 0.5-day scan step.
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

    // The aspect is exact when the transiting body sits on natal_lon + aspect
    // or natal_lon - aspect (the same point for conjunction and opposition).
    // For each of those target longitudes the signed difference
    // wrap_delta(transit_lon - target) crosses zero with a sign change, which
    // the unsigned angular_distance never does at 0° and 180°.
    let aspect = angular_distance(0.0, aspect);
    let mut targets = vec![natal_lon + aspect];
    if aspect > 0.0 && aspect < 180.0 {
        targets.push(natal_lon - aspect);
    }

    // Step size: 0.5 day catches even lunar transits (~13 deg/day).
    let step = 0.5_f64;
    let mut results = Vec::new();
    let (mut jd, mut lon) = (jd_start, transit_fn(jd_start));

    while jd < jd_end {
        let next_jd = (jd + step).min(jd_end);
        let next_lon = transit_fn(next_jd);

        for &target in &targets {
            let prev = wrap_delta(lon - target);
            let curr = wrap_delta(next_lon - target);

            if prev == 0.0 {
                results.push(jd);
            } else if prev * curr < 0.0 && prev.abs() < 90.0 && curr.abs() < 90.0 {
                // The magnitude check rejects the ±180° wrap opposite the
                // target, where the signed difference also changes sign.
                let f = |t: f64| wrap_delta(transit_fn(t) - target);
                results.push(bisect_aspect_crossing(&f, jd, next_jd, prev, 60));
            }
        }
        (jd, lon) = (next_jd, next_lon);
    }

    // A pass exactly on the last sample has no following interval to report it.
    for &target in &targets {
        if wrap_delta(lon - target) == 0.0 {
            results.push(jd_end);
        }
    }

    results.sort_by(f64::total_cmp);
    results
}

/// Normalize an angle difference to [-180, +180].
fn wrap_delta(d: f64) -> f64 {
    let d = d.rem_euclid(360.0);
    if d > 180.0 { d - 360.0 } else { d }
}

/// Bisection helper for transit aspect search. `f(lo)` and `f(hi)` must have
/// opposite signs; `f_lo` is `f(lo)`.
fn bisect_aspect_crossing(
    f: &impl Fn(f64) -> f64,
    mut lo: f64,
    mut hi: f64,
    mut f_lo: f64,
    max_iter: u32,
) -> f64 {
    for _ in 0..max_iter {
        let mid = (lo + hi) / 2.0;
        let f_mid = f(mid);

        if f_mid * f_lo <= 0.0 {
            hi = mid;
        } else {
            (lo, f_lo) = (mid, f_mid);
        }
    }
    (lo + hi) / 2.0
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
        assert!(!results.is_empty(), "Should find at least one conjunction");
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
        let results = find_transit_aspect(5.0, |jd| (350.0 + jd).rem_euclid(360.0), 0.0, 0.0, 30.0);
        assert!(
            !results.is_empty(),
            "Should find conjunction across 360/0 wrap"
        );
        assert!(
            (results[0] - 15.0).abs() < 1.0,
            "Conjunction should be near jd=15, got {}",
            results[0]
        );
    }

    // -----------------------------------------------------------------------
    // find_transit_aspect: exact moments that fall between the scan samples
    // -----------------------------------------------------------------------

    const J2000: f64 = 2451545.0;

    /// Run `find_transit_aspect` for a body moving linearly at `rate` deg/day
    /// from `start_lon` at J2000, and compare with the analytically known
    /// passes (`expected_days` after J2000). Every returned JD must also be a
    /// real root of g(jd) = angular_distance(transit, natal) - aspect.
    fn check_linear_passes(
        natal_lon: f64,
        start_lon: f64,
        rate: f64,
        aspect: f64,
        days: f64,
        expected_days: &[f64],
    ) {
        let transit = |jd: f64| (start_lon + rate * (jd - J2000)).rem_euclid(360.0);
        let results = find_transit_aspect(natal_lon, transit, aspect, J2000, J2000 + days);
        let got_days: Vec<f64> = results.iter().map(|jd| jd - J2000).collect();

        for &jd in &results {
            let residual = (angular_distance(transit(jd), natal_lon) - aspect).abs();
            assert!(
                residual < 1e-6,
                "returned day {} is not exact: |g| = {residual} deg (all returned days: {got_days:?})",
                jd - J2000
            );
        }
        assert_eq!(
            got_days.len(),
            expected_days.len(),
            "expected passes at days {expected_days:?}, got {got_days:?}"
        );
        for (got, want) in got_days.iter().zip(expected_days) {
            assert!(
                (got - want).abs() < 1e-6,
                "expected passes at days {expected_days:?}, got {got_days:?}"
            );
        }
    }

    #[test]
    fn transit_conjunction_between_samples() {
        // 100° + 1°/day reaches natal 110.3° at day 10.3.
        check_linear_passes(110.3, 100.0, 1.0, 0.0, 30.0, &[10.3]);
    }

    #[test]
    fn transit_opposition_between_samples() {
        // Natal 290.3°: the opposition point is 110.3°, reached at day 10.3.
        check_linear_passes(290.3, 100.0, 1.0, 180.0, 30.0, &[10.3]);
    }

    #[test]
    fn transit_conjunction_fast_body() {
        // 13°/day: 110.3° is reached at 10.3/13 and (10.3 + 360)/13 days.
        check_linear_passes(110.3, 100.0, 13.0, 0.0, 30.0, &[10.3 / 13.0, 370.3 / 13.0]);
    }

    #[test]
    fn transit_opposition_fast_body() {
        check_linear_passes(
            290.3,
            100.0,
            13.0,
            180.0,
            30.0,
            &[10.3 / 13.0, 370.3 / 13.0],
        );
    }

    #[test]
    fn transit_conjunction_retrograde_body() {
        // 120° − 1°/day reaches natal 110.3° at day 9.7.
        check_linear_passes(110.3, 120.0, -1.0, 0.0, 30.0, &[9.7]);
    }

    #[test]
    fn transit_conjunction_slow_body_is_not_reported_early() {
        // 0.03°/day: exact at day 10.33, but the samples at days 10.0 and 10.5
        // are already within 0.01° of the natal longitude.
        check_linear_passes(100.3099, 100.0, 0.03, 0.0, 30.0, &[10.33]);
    }

    #[test]
    fn transit_retrograde_loop_gives_three_passes() {
        // Mean motion 0.5°/day plus a 10° oscillation with a 100-day period:
        // the body is retrograde between days ~39.7 and ~60.3, sweeping back
        // from ~125.88° to ~124.12°, so 125.3° is crossed direct, retrograde
        // and direct again.
        let transit = |jd: f64| {
            let t = jd - J2000;
            (100.0 + 0.5 * t + 10.0 * (std::f64::consts::TAU * t / 100.0).sin()).rem_euclid(360.0)
        };
        for (natal_lon, aspect) in [(125.3, 0.0), (305.3, 180.0)] {
            let results = find_transit_aspect(natal_lon, transit, aspect, J2000, J2000 + 120.0);
            assert_eq!(results.len(), 3, "aspect {aspect}: got {results:?}");
            assert!(results.windows(2).all(|w| w[0] < w[1]));
            for jd in results {
                let residual = (angular_distance(transit(jd), natal_lon) - aspect).abs();
                assert!(residual < 1e-6, "aspect {aspect}: |g| = {residual} at {jd}");
            }
        }
    }

    #[test]
    fn transit_pass_on_window_edge_is_reported_once() {
        // Natal 30°, transit longitude = jd: exact at jd = 30.
        assert_eq!(find_transit_aspect(30.0, |jd| jd, 0.0, 30.0, 40.0), [30.0]);
        assert_eq!(find_transit_aspect(30.0, |jd| jd, 0.0, 20.0, 30.0), [30.0]);
        assert_eq!(find_transit_aspect(30.0, |jd| jd, 0.0, 20.0, 40.0), [30.0]);
        // The scan must not run past jd_end.
        assert!(find_transit_aspect(30.0, |jd| jd, 0.0, 20.0, 29.9).is_empty());
    }

    #[test]
    fn transit_square_between_samples() {
        // Natal 20.3°: square points are 110.3° (day 10.3) and 290.3° (not reached).
        check_linear_passes(20.3, 100.0, 1.0, 90.0, 30.0, &[10.3]);
    }

    #[test]
    fn transit_trine_between_samples() {
        // Natal 350.3°: trine points are 110.3° (day 10.3) and 230.3° (not reached).
        check_linear_passes(350.3, 100.0, 1.0, 120.0, 30.0, &[10.3]);
    }

    #[test]
    fn transit_square_fast_body_both_sides() {
        // Natal 110.3°, 13°/day: 200.3° at 100.3/13 days, 20.3° at 280.3/13 days.
        check_linear_passes(
            110.3,
            100.0,
            13.0,
            90.0,
            30.0,
            &[100.3 / 13.0, 280.3 / 13.0],
        );
    }
}
