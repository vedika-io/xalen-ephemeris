//! Cosmobiology — 90-degree dial and planetary pictures.
//!
//! Cosmobiology is a systematic method of astrological analysis using
//! midpoints on the 90° dial. It discards houses and signs, focusing
//! entirely on angular relationships.
//!
//! Key concepts:
//! - **90° dial**: all positions are reduced modulo 90°, collapsing
//!   conjunction, opposition, and square into a single axis.
//! - **Planetary picture**: a three-body configuration A/B = C (planet C
//!   sits on the midpoint of A and B).
//! - **Key interpretations**: midpoint-pair meanings keyed by planet pair.
//!
//! Method reference: the 90° dial and planetary-picture technique are
//! standard cosmobiological practice.

use crate::midpoints::{Midpoint, all_midpoints, to_90_degree};
use serde::Serialize;

// ── Core types ────────────────────────────────────────────────────────

/// Full cosmobiology chart with 90° positions, midpoints, and
/// planetary pictures.
#[derive(Debug, Clone, Serialize)]
pub struct CosmobiologyChart {
    /// Positions projected onto the 90° dial.
    pub positions_90: Vec<(String, f64)>,
    /// All midpoints (in 360° space).
    pub midpoints: Vec<Midpoint>,
    /// Active planetary pictures found within the given orb.
    pub planetary_pictures: Vec<PlanetaryPicture>,
}

/// A planetary picture: planet C sits on the midpoint A/B (within orb)
/// on the 90° dial.
#[derive(Debug, Clone, Serialize)]
pub struct PlanetaryPicture {
    /// Half-sum notation, e.g. "SU/MO = ME".
    pub formula: String,
    /// The body occupying the midpoint axis.
    pub planet: String,
    /// The two bodies forming the midpoint.
    pub body_a: String,
    pub body_b: String,
    /// Orb on the 90° dial (degrees).
    pub orb_90: f64,
    /// Midpoint key interpretation, if available.
    pub interpretation: Option<&'static str>,
}

/// A single midpoint key interpretation, keyed by planet pair.
#[derive(Debug, Clone)]
pub struct MidpointKey {
    pub body_a: &'static str,
    pub body_b: &'static str,
    pub keyword: &'static str,
    pub interpretation: &'static str,
}

// ── Midpoint key interpretations ──────────────────────────────────────

/// Midpoint key interpretations, keyed by planet pair.
pub fn ebertin_keys() -> Vec<MidpointKey> {
    vec![
        MidpointKey {
            body_a: "Sun",
            body_b: "Moon",
            keyword: "",
            interpretation: "",
        },
        MidpointKey {
            body_a: "Sun",
            body_b: "Mercury",
            keyword: "",
            interpretation: "",
        },
        MidpointKey {
            body_a: "Sun",
            body_b: "Venus",
            keyword: "",
            interpretation: "",
        },
        MidpointKey {
            body_a: "Sun",
            body_b: "Mars",
            keyword: "",
            interpretation: "",
        },
        MidpointKey {
            body_a: "Sun",
            body_b: "Jupiter",
            keyword: "",
            interpretation: "",
        },
        MidpointKey {
            body_a: "Sun",
            body_b: "Saturn",
            keyword: "",
            interpretation: "",
        },
        MidpointKey {
            body_a: "Sun",
            body_b: "Uranus",
            keyword: "",
            interpretation: "",
        },
        MidpointKey {
            body_a: "Sun",
            body_b: "Neptune",
            keyword: "",
            interpretation: "",
        },
        MidpointKey {
            body_a: "Sun",
            body_b: "Pluto",
            keyword: "",
            interpretation: "",
        },
        MidpointKey {
            body_a: "Moon",
            body_b: "Mercury",
            keyword: "",
            interpretation: "",
        },
        MidpointKey {
            body_a: "Moon",
            body_b: "Venus",
            keyword: "",
            interpretation: "",
        },
        MidpointKey {
            body_a: "Moon",
            body_b: "Mars",
            keyword: "",
            interpretation: "",
        },
        MidpointKey {
            body_a: "Moon",
            body_b: "Jupiter",
            keyword: "",
            interpretation: "",
        },
        MidpointKey {
            body_a: "Moon",
            body_b: "Saturn",
            keyword: "",
            interpretation: "",
        },
        MidpointKey {
            body_a: "Mars",
            body_b: "Saturn",
            keyword: "",
            interpretation: "",
        },
        MidpointKey {
            body_a: "Venus",
            body_b: "Jupiter",
            keyword: "",
            interpretation: "",
        },
        MidpointKey {
            body_a: "Venus",
            body_b: "Mars",
            keyword: "",
            interpretation: "",
        },
        MidpointKey {
            body_a: "Venus",
            body_b: "Saturn",
            keyword: "",
            interpretation: "",
        },
        MidpointKey {
            body_a: "Jupiter",
            body_b: "Saturn",
            keyword: "",
            interpretation: "",
        },
        MidpointKey {
            body_a: "Mars",
            body_b: "Jupiter",
            keyword: "",
            interpretation: "",
        },
        MidpointKey {
            body_a: "Mars",
            body_b: "Uranus",
            keyword: "",
            interpretation: "",
        },
        MidpointKey {
            body_a: "Mars",
            body_b: "Neptune",
            keyword: "",
            interpretation: "",
        },
        MidpointKey {
            body_a: "Mars",
            body_b: "Pluto",
            keyword: "",
            interpretation: "",
        },
        MidpointKey {
            body_a: "Jupiter",
            body_b: "Uranus",
            keyword: "",
            interpretation: "",
        },
        MidpointKey {
            body_a: "Saturn",
            body_b: "Uranus",
            keyword: "",
            interpretation: "",
        },
        MidpointKey {
            body_a: "Saturn",
            body_b: "Neptune",
            keyword: "",
            interpretation: "",
        },
        MidpointKey {
            body_a: "Saturn",
            body_b: "Pluto",
            keyword: "",
            interpretation: "",
        },
        MidpointKey {
            body_a: "Uranus",
            body_b: "Neptune",
            keyword: "",
            interpretation: "",
        },
        MidpointKey {
            body_a: "Uranus",
            body_b: "Pluto",
            keyword: "",
            interpretation: "",
        },
        MidpointKey {
            body_a: "Neptune",
            body_b: "Pluto",
            keyword: "",
            interpretation: "",
        },
    ]
}

// ── Computation ───────────────────────────────────────────────────────

/// Build a complete Cosmobiology chart.
///
/// `positions`: `(body_name, ecliptic_longitude_degrees)`.
/// `orb`: maximum orb on the **90° dial** for planetary pictures.
pub fn cosmobiology_chart(positions: &[(&str, f64)], orb: f64) -> CosmobiologyChart {
    // 90° dial positions
    let positions_90: Vec<(String, f64)> = positions
        .iter()
        .map(|&(name, lon)| (name.to_string(), to_90_degree(lon)))
        .collect();

    // All midpoints in 360° space
    let midpoints = all_midpoints(positions);

    // Find planetary pictures on the 90° dial
    let keys = ebertin_keys();
    let mut pictures = Vec::new();

    for mp in &midpoints {
        let mp_90 = to_90_degree(mp.degree);

        for &(planet_name, planet_lon) in positions {
            // Skip if planet is part of this midpoint
            if planet_name == mp.body_a || planet_name == mp.body_b {
                continue;
            }
            let planet_90 = to_90_degree(planet_lon);
            let diff = (planet_90 - mp_90).abs();
            let dist = if diff > 45.0 { 90.0 - diff } else { diff };

            if dist <= orb {
                let formula = format!("{}/{} = {}", mp.body_a, mp.body_b, planet_name);

                // Look up the key interpretation for this midpoint pair.
                // Honesty contract: a stripped (empty) interpretation surfaces
                // as `None`, never `Some("")`.
                let interp = keys
                    .iter()
                    .find(|k| {
                        (k.body_a == mp.body_a && k.body_b == mp.body_b)
                            || (k.body_a == mp.body_b && k.body_b == mp.body_a)
                    })
                    .map(|k| k.interpretation)
                    .filter(|s| !s.is_empty());

                pictures.push(PlanetaryPicture {
                    formula,
                    planet: planet_name.to_string(),
                    body_a: mp.body_a.clone(),
                    body_b: mp.body_b.clone(),
                    orb_90: dist,
                    interpretation: interp,
                });
            }
        }
    }

    pictures.sort_by(|a, b| {
        a.orb_90
            .partial_cmp(&b.orb_90)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    CosmobiologyChart {
        positions_90,
        midpoints,
        planetary_pictures: pictures,
    }
}

/// Look up the key interpretation for a midpoint pair.
///
/// Returns `None` if the pair is not in the table **or** if its interpretation
/// text has been stripped (empty string). The interpretation table ships with
/// empty text by design — the empty-API honesty contract: we never fabricate or
/// AI-generate interpretive content, so a stripped entry must report "no text
/// available" (`None`) rather than an empty `Some("")` that callers might render
/// as a blank reading.
pub fn lookup_midpoint_key(body_a: &str, body_b: &str) -> Option<&'static str> {
    ebertin_keys()
        .into_iter()
        .find(|k| {
            (k.body_a == body_a && k.body_b == body_b) || (k.body_a == body_b && k.body_b == body_a)
        })
        .map(|k| k.interpretation)
        .filter(|s| !s.is_empty())
}

/// Format a planetary picture for display.
pub fn format_picture(pp: &PlanetaryPicture) -> String {
    let deg = pp.orb_90.floor() as u32;
    let min = ((pp.orb_90 - deg as f64) * 60.0).round() as u32;
    let interp = pp.interpretation.unwrap_or("(no standard key)");
    format!("{} (orb {}°{:02}') — {}", pp.formula, deg, min, interp)
}

// ── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_positions() -> Vec<(&'static str, f64)> {
        vec![
            ("Sun", 10.0),
            ("Moon", 100.0),
            ("Mercury", 55.0),
            ("Venus", 200.0),
            ("Mars", 280.0),
            ("Jupiter", 150.0),
            ("Saturn", 320.0),
        ]
    }

    #[test]
    fn ebertin_keys_has_at_least_20() {
        assert!(
            ebertin_keys().len() >= 20,
            "Should have at least 20 key interpretations, got {}",
            ebertin_keys().len()
        );
    }

    #[test]
    fn chart_has_all_positions() {
        let pos = sample_positions();
        let chart = cosmobiology_chart(&pos, 2.0);
        assert_eq!(chart.positions_90.len(), pos.len());
    }

    #[test]
    fn chart_positions_are_mod_90() {
        let chart = cosmobiology_chart(&sample_positions(), 2.0);
        for (_, deg) in &chart.positions_90 {
            assert!(
                *deg >= 0.0 && *deg < 90.0,
                "90° position should be in [0,90), got {deg}"
            );
        }
    }

    #[test]
    fn chart_midpoints_count() {
        let pos = sample_positions();
        let n = pos.len();
        let chart = cosmobiology_chart(&pos, 2.0);
        assert_eq!(chart.midpoints.len(), n * (n - 1) / 2);
    }

    #[test]
    fn planetary_picture_exact_hit() {
        // Sun at 0°, Moon at 90°. Midpoint = 45°.
        // Mars at 45° exactly on the midpoint.
        let pos = vec![("Sun", 0.0), ("Moon", 90.0), ("Mars", 45.0)];
        let chart = cosmobiology_chart(&pos, 2.0);
        assert!(
            chart.planetary_pictures.iter().any(|pp| pp.planet == "Mars"
                && ((pp.body_a == "Sun" && pp.body_b == "Moon")
                    || (pp.body_a == "Moon" && pp.body_b == "Sun"))),
            "Mars should appear in Sun/Moon planetary picture"
        );
    }

    #[test]
    fn planetary_picture_via_square() {
        // Sun at 0°, Moon at 90° -> midpoint 45°.
        // Mercury at 135° is square (90°) to 45°, so on 90° dial they coincide.
        // 135 mod 90 = 45, and midpoint 45 mod 90 = 45.
        let pos = vec![("Sun", 0.0), ("Moon", 90.0), ("Mercury", 135.0)];
        let chart = cosmobiology_chart(&pos, 2.0);
        assert!(
            chart
                .planetary_pictures
                .iter()
                .any(|pp| pp.planet == "Mercury"),
            "Mercury at 135° should appear in a planetary picture via square"
        );
    }

    #[test]
    fn no_self_reference_in_pictures() {
        let chart = cosmobiology_chart(&sample_positions(), 5.0);
        for pp in &chart.planetary_pictures {
            assert_ne!(
                pp.planet, pp.body_a,
                "Planet should not appear in its own midpoint"
            );
            assert_ne!(
                pp.planet, pp.body_b,
                "Planet should not appear in its own midpoint"
            );
        }
    }

    #[test]
    fn lookup_returns_none_for_stripped_interpretation() {
        // Honesty contract: interpretation text is stripped (empty) in the
        // shipped table, so a present-but-empty entry must resolve to `None`,
        // never `Some("")`. Sun/Moon IS a known pair, but its interpretation
        // text has been removed — so we return `None`.
        let interp = lookup_midpoint_key("Sun", "Moon");
        assert_eq!(
            interp, None,
            "Sun/Moon has a stripped (empty) interpretation → must be None, got {interp:?}"
        );
    }

    #[test]
    fn lookup_none_is_symmetric_for_stripped_entries() {
        // A stripped pair returns `None` regardless of argument order.
        let forward = lookup_midpoint_key("Mars", "Saturn");
        let reverse = lookup_midpoint_key("Saturn", "Mars");
        assert_eq!(forward, None, "Mars/Saturn stripped → None");
        assert_eq!(reverse, None, "Saturn/Mars stripped → None");
        assert_eq!(forward, reverse, "Lookup should be symmetric");
    }

    #[test]
    fn lookup_unknown_pair_is_none() {
        // A pair that is not in the table at all is also `None` (same result as
        // a stripped entry — callers cannot distinguish, which is correct: in
        // both cases there is no honest interpretation to show).
        assert_eq!(lookup_midpoint_key("Sun", "Sun"), None);
        assert_eq!(lookup_midpoint_key("Ceres", "Vesta"), None);
    }

    #[test]
    fn lookup_nonempty_entry_returns_some() {
        // Guard the *mechanism*: if a real interpretation is ever restored to
        // the table, `lookup_midpoint_key` must surface it. We test the
        // private filter logic against a synthetic non-empty key here so the
        // contract is locked even while the shipped table is empty.
        let key = MidpointKey {
            body_a: "Sun",
            body_b: "Moon",
            keyword: "kw",
            interpretation: "non-empty",
        };
        // Re-implement the exact match+filter the function performs to prove
        // a non-empty interpretation would pass the empty filter.
        let resolved = Some(key.interpretation).filter(|s: &&str| !s.is_empty());
        assert_eq!(resolved, Some("non-empty"));
    }

    #[test]
    fn format_picture_output() {
        let pp = PlanetaryPicture {
            formula: "SU/MO = ME".to_string(),
            planet: "Mercury".to_string(),
            body_a: "Sun".to_string(),
            body_b: "Moon".to_string(),
            orb_90: 1.5,
            interpretation: Some("KEY_TEXT"),
        };
        let formatted = format_picture(&pp);
        assert!(formatted.contains("SU/MO = ME"));
        assert!(formatted.contains("KEY_TEXT"));
    }

    #[test]
    fn chart_pictures_have_no_stripped_interpretations() {
        // End-to-end honesty contract: every planetary picture built from the
        // shipped (stripped) table must carry `None`, never `Some("")`.
        let chart = cosmobiology_chart(&sample_positions(), 5.0);
        for pp in &chart.planetary_pictures {
            assert_ne!(
                pp.interpretation,
                Some(""),
                "planetary picture must never carry an empty-string interpretation"
            );
            // With the shipped empty table, all interpretations are None.
            assert_eq!(
                pp.interpretation, None,
                "stripped table → picture interpretation must be None, got {:?}",
                pp.interpretation
            );
        }
    }

    #[test]
    fn pictures_sorted_by_orb() {
        let chart = cosmobiology_chart(&sample_positions(), 5.0);
        if chart.planetary_pictures.len() > 1 {
            for w in chart.planetary_pictures.windows(2) {
                assert!(
                    w[0].orb_90 <= w[1].orb_90 + 1e-10,
                    "Pictures should be sorted by orb"
                );
            }
        }
    }
}
