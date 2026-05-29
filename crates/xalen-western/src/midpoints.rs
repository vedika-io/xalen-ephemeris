//! Midpoints (Ebertin / Cosmobiology school).
//!
//! A midpoint is the exact halfway point on the shorter arc between two
//! planetary positions.  Every midpoint also has an "opposite" point 180°
//! away, which is equally sensitive.
//!
//! Half-sum notation:  A/B = midpoint of A and B.
//! If planet C sits on A/B within orb, we write "C = A/B".
//!
//! Reference: Reinhold Ebertin, *The Combination of Stellar Influences* (1940).

use serde::{Deserialize, Serialize};

// ── Core types ────────────────────────────────────────────────────────

/// A single midpoint between two bodies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Midpoint {
    pub body_a: String,
    pub body_b: String,
    /// Shorter-arc midpoint position in degrees [0, 360).
    pub degree: f64,
    /// The opposite point, 180° away (also considered active).
    pub opposite: f64,
}

/// An entry in the midpoint tree: a planet sitting on one or more midpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidpointTree {
    pub planet: String,
    /// (body_pair label "A/B", orb in degrees) for each midpoint this planet
    /// activates (conjunct, opposite, or square within the given orb).
    pub midpoints_on_axis: Vec<(String, f64)>,
}

/// A midpoint activation: planet C = A/B within orb.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidpointActivation {
    /// The activating planet.
    pub planet: String,
    /// Half-sum notation, e.g. "SU/MO".
    pub pair: String,
    /// Hard-aspect type (conjunction=0, square=90, opposition=180, or 270).
    pub contact_angle: f64,
    /// Actual orb (always positive).
    pub orb: f64,
}

// ── Computation ───────────────────────────────────────────────────────

/// Compute the shorter-arc midpoint between two ecliptic longitudes (degrees).
///
/// Returns a value in [0, 360).
pub fn compute_midpoint(lon_a: f64, lon_b: f64) -> f64 {
    let diff = (lon_b - lon_a).rem_euclid(360.0);
    if diff <= 180.0 {
        (lon_a + diff / 2.0).rem_euclid(360.0)
    } else {
        (lon_a + (diff + 360.0) / 2.0).rem_euclid(360.0)
    }
}

/// Generate all n*(n-1)/2 midpoints from a set of positions.
///
/// Each position is `(body_name, ecliptic_longitude_degrees)`.
pub fn all_midpoints(positions: &[(&str, f64)]) -> Vec<Midpoint> {
    let mut result = Vec::with_capacity(positions.len() * (positions.len() - 1) / 2);
    for i in 0..positions.len() {
        for j in (i + 1)..positions.len() {
            let deg = compute_midpoint(positions[i].1, positions[j].1);
            result.push(Midpoint {
                body_a: positions[i].0.to_string(),
                body_b: positions[j].0.to_string(),
                degree: deg,
                opposite: (deg + 180.0).rem_euclid(360.0),
            });
        }
    }
    result
}

/// Angular distance on a circle, always in [0, 180].
fn arc_distance(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(360.0);
    if d > 180.0 { 360.0 - d } else { d }
}

/// Smallest distance to any of the four hard-aspect multiples (mod 90) of
/// the midpoint.  Returns `(contact_angle, orb)`.
fn hard_aspect_orb(planet_lon: f64, midpoint_lon: f64) -> (f64, f64) {
    let mut best_angle = 0.0_f64;
    let mut best_orb = f64::MAX;
    for &contact in &[0.0, 90.0, 180.0, 270.0] {
        let target = (midpoint_lon + contact).rem_euclid(360.0);
        let orb = arc_distance(planet_lon, target);
        if orb < best_orb {
            best_orb = orb;
            best_angle = contact;
        }
    }
    (best_angle, best_orb)
}

/// Build the midpoint tree: for every planet, find which midpoints it
/// activates via hard aspects (0/90/180/270) within `orb` degrees.
pub fn midpoint_tree(positions: &[(&str, f64)], orb: f64) -> Vec<MidpointTree> {
    let mps = all_midpoints(positions);

    positions
        .iter()
        .map(|&(planet, lon)| {
            let mut on_axis = Vec::new();
            for mp in &mps {
                // Skip midpoints that include this planet itself.
                if mp.body_a == planet || mp.body_b == planet {
                    continue;
                }
                let (_contact, actual_orb) = hard_aspect_orb(lon, mp.degree);
                if actual_orb <= orb {
                    let pair = format!("{}/{}", mp.body_a, mp.body_b);
                    on_axis.push((pair, actual_orb));
                }
            }
            on_axis.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            MidpointTree {
                planet: planet.to_string(),
                midpoints_on_axis: on_axis,
            }
        })
        .collect()
}

/// Find all midpoint activations across the chart.
///
/// Returns every case where planet C is within `orb` of A/B via a hard
/// aspect (0/90/180/270).
pub fn find_activations(positions: &[(&str, f64)], orb: f64) -> Vec<MidpointActivation> {
    let mps = all_midpoints(positions);
    let mut activations = Vec::new();

    for &(planet, lon) in positions {
        for mp in &mps {
            if mp.body_a == planet || mp.body_b == planet {
                continue;
            }
            let (contact_angle, actual_orb) = hard_aspect_orb(lon, mp.degree);
            if actual_orb <= orb {
                activations.push(MidpointActivation {
                    planet: planet.to_string(),
                    pair: format!("{}/{}", mp.body_a, mp.body_b),
                    contact_angle,
                    orb: actual_orb,
                });
            }
        }
    }
    activations.sort_by(|a, b| {
        a.orb
            .partial_cmp(&b.orb)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    activations
}

/// Format a midpoint activation in Ebertin half-sum notation.
///
/// e.g. `"ME = SU/MO (0°00' orb)"`.
pub fn format_halfsums(activations: &[MidpointActivation]) -> Vec<String> {
    activations
        .iter()
        .map(|a| {
            let deg = a.orb.floor() as u32;
            let min = ((a.orb - deg as f64) * 60.0).round() as u32;
            format!("{} = {} ({:>3}°{:02}' orb)", a.planet, a.pair, deg, min)
        })
        .collect()
}

/// Convert a longitude to the 90° sort-wheel position used in Cosmobiology.
pub fn to_90_degree(lon: f64) -> f64 {
    lon.rem_euclid(90.0)
}

/// Sort all positions onto the 90° dial.
pub fn sort_wheel_90(positions: &[(&str, f64)]) -> Vec<(String, f64)> {
    let mut sorted: Vec<(String, f64)> = positions
        .iter()
        .map(|&(name, lon)| (name.to_string(), to_90_degree(lon)))
        .collect();
    sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    sorted
}

// ── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn midpoint_simple_average() {
        // Sun 10°, Moon 30° => midpoint 20°
        let mp = compute_midpoint(10.0, 30.0);
        assert!((mp - 20.0).abs() < 1e-10, "Expected 20°, got {mp}°");
    }

    #[test]
    fn midpoint_shorter_arc_wrapping() {
        // 350° and 10° -> shorter arc is 20° across 0° -> midpoint = 0°
        let mp = compute_midpoint(350.0, 10.0);
        assert!(
            mp.abs() < 1e-10 || (mp - 360.0).abs() < 1e-10,
            "Expected ~0°, got {mp}°"
        );
    }

    #[test]
    fn midpoint_opposite_is_180_away() {
        let mps = all_midpoints(&[("Sun", 10.0), ("Moon", 30.0)]);
        assert_eq!(mps.len(), 1);
        let diff = (mps[0].opposite - mps[0].degree).abs();
        assert!((diff - 180.0).abs() < 1e-10);
    }

    #[test]
    fn midpoint_symmetric() {
        let ab = compute_midpoint(45.0, 135.0);
        let ba = compute_midpoint(135.0, 45.0);
        assert!(
            (ab - ba).abs() < 1e-10,
            "Midpoint should be symmetric: {ab} vs {ba}"
        );
    }

    #[test]
    fn midpoint_opposition_case() {
        // Bodies exactly 180° apart: midpoints at 90° offsets from each body.
        let mp = compute_midpoint(0.0, 180.0);
        assert!(
            (mp - 90.0).abs() < 1e-10 || (mp - 270.0).abs() < 1e-10,
            "Midpoint of 0° and 180° should be 90° or 270°, got {mp}°"
        );
    }

    #[test]
    fn all_midpoints_count() {
        let positions: Vec<(&str, f64)> = vec![
            ("Sun", 0.0),
            ("Moon", 90.0),
            ("Mars", 180.0),
            ("Jupiter", 270.0),
        ];
        let mps = all_midpoints(&positions);
        // 4 choose 2 = 6
        assert_eq!(mps.len(), 6);
    }

    #[test]
    fn midpoint_tree_activation() {
        // Mars at 45° is exactly on the Sun(0°)/Moon(90°) midpoint.
        let positions: Vec<(&str, f64)> = vec![("Sun", 0.0), ("Moon", 90.0), ("Mars", 45.0)];
        let tree = midpoint_tree(&positions, 2.0);
        let mars_tree = tree.iter().find(|t| t.planet == "Mars").unwrap();
        assert!(
            mars_tree
                .midpoints_on_axis
                .iter()
                .any(|(pair, orb)| pair == "Sun/Moon" && *orb < 0.01),
            "Mars should activate Sun/Moon midpoint: {:#?}",
            mars_tree
        );
    }

    #[test]
    fn midpoint_tree_square_activation() {
        // Mercury at 135° is square (90°) to the Sun(0°)/Moon(90°) midpoint at 45°.
        let positions: Vec<(&str, f64)> = vec![("Sun", 0.0), ("Moon", 90.0), ("Mercury", 135.0)];
        let tree = midpoint_tree(&positions, 2.0);
        let merc_tree = tree.iter().find(|t| t.planet == "Mercury").unwrap();
        assert!(
            !merc_tree.midpoints_on_axis.is_empty(),
            "Mercury at 135° should activate Sun/Moon midpoint (45°) via square"
        );
    }

    #[test]
    fn find_activations_basic() {
        let positions: Vec<(&str, f64)> = vec![("Sun", 0.0), ("Moon", 90.0), ("Mars", 45.5)];
        let acts = find_activations(&positions, 1.0);
        assert!(
            acts.iter()
                .any(|a| a.planet == "Mars" && a.pair == "Sun/Moon" && a.orb < 1.0),
            "Mars at 45.5° should activate Sun/Moon midpoint at 45°"
        );
    }

    #[test]
    fn format_halfsums_notation() {
        let act = MidpointActivation {
            planet: "ME".to_string(),
            pair: "SU/MO".to_string(),
            contact_angle: 0.0,
            orb: 0.5,
        };
        let formatted = format_halfsums(&[act]);
        assert_eq!(formatted.len(), 1);
        assert!(formatted[0].contains("ME = SU/MO"));
    }

    #[test]
    fn to_90_degree_values() {
        assert!((to_90_degree(45.0) - 45.0).abs() < 1e-10);
        assert!((to_90_degree(135.0) - 45.0).abs() < 1e-10);
        assert!((to_90_degree(225.0) - 45.0).abs() < 1e-10);
        assert!((to_90_degree(315.0) - 45.0).abs() < 1e-10);
        assert!((to_90_degree(360.0) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn sort_wheel_90_ordering() {
        let positions: Vec<(&str, f64)> = vec![
            ("Sun", 300.0),  // 300 mod 90 = 30
            ("Moon", 135.0), // 135 mod 90 = 45
            ("Mars", 10.0),  // 10 mod 90 = 10
        ];
        let sorted = sort_wheel_90(&positions);
        assert_eq!(sorted[0].0, "Mars");
        assert_eq!(sorted[1].0, "Sun");
        assert_eq!(sorted[2].0, "Moon");
    }

    #[test]
    fn no_self_activation() {
        // A planet should never activate a midpoint involving itself.
        let positions: Vec<(&str, f64)> = vec![("Sun", 0.0), ("Moon", 90.0), ("Mars", 45.0)];
        let acts = find_activations(&positions, 5.0);
        for a in &acts {
            assert!(
                !a.pair.contains(&a.planet),
                "Planet {} should not activate midpoint {} involving itself",
                a.planet,
                a.pair
            );
        }
    }
}
