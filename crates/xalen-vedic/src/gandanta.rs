//! Gandanta detection — the dreaded water-fire sign junctions.
//!
//! Gandanta (Sanskrit: "knot at the end") marks the boundary between a
//! water sign and the fire sign that follows it. These are the three
//! junctions in the zodiac:
//!
//! | Water sign end | Fire sign start | Boundary degree |
//! |----------------|-----------------|-----------------|
//! | Cancer (120°)  | Leo (120°)      | 120°            |
//! | Scorpio (240°) | Sagittarius     | 240°            |
//! | Pisces (360°)  | Aries (0°)      | 0° / 360°       |
//!
//! A planet within 3°20' (= 200 arc-minutes = one nakshatra pada) on
//! either side of these boundaries is considered to be in Gandanta.
//! Severity increases as the planet approaches the exact junction point.
//!
//! Source: Brihat Parashara Hora Shastra; standard 3°20' orb is the
//! most widely used convention (one pada = 13°20'/4 = 3°20').

use serde::{Deserialize, Serialize};

/// The standard Gandanta orb: 3°20' = 3.333... degrees = one nakshatra
/// pada on each side of the boundary.
const GANDANTA_ORB_DEG: f64 = 10.0 / 3.0; // 3.33333...

/// The three zodiacal boundary degrees where water meets fire.
const GANDANTA_BOUNDARIES: [f64; 3] = [0.0, 120.0, 240.0];

/// Severity levels for Gandanta placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GandantaSeverity {
    /// Within the inner 1° of the boundary — most afflicted.
    Severe,
    /// Between 1° and 2° from the boundary.
    Moderate,
    /// Between 2° and 3°20' from the boundary — mildly afflicted.
    Mild,
}

impl GandantaSeverity {
    pub fn label(&self) -> &'static str {
        match self {
            GandantaSeverity::Severe => "Severe",
            GandantaSeverity::Moderate => "Moderate",
            GandantaSeverity::Mild => "Mild",
        }
    }
}

/// Detailed result of a Gandanta check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GandantaInfo {
    /// Whether the planet is in Gandanta at all.
    pub in_gandanta: bool,
    /// The severity level, if in Gandanta.
    pub severity: Option<GandantaSeverity>,
    /// The nearest Gandanta boundary degree (0, 120, or 240).
    pub nearest_boundary: f64,
    /// Angular distance from the nearest boundary (always positive).
    pub distance_deg: f64,
}

/// Returns the shortest angular distance from `lon` to the nearest
/// Gandanta boundary, and which boundary that is.
fn nearest_gandanta_boundary(lon: f64) -> (f64, f64) {
    let lon = lon.rem_euclid(360.0);
    let mut best_dist = f64::MAX;
    let mut best_boundary = 0.0;
    for &b in &GANDANTA_BOUNDARIES {
        let diff = (lon - b).rem_euclid(360.0);
        let dist = diff.min(360.0 - diff);
        if dist < best_dist {
            best_dist = dist;
            best_boundary = b;
        }
    }
    (best_dist, best_boundary)
}

/// Check whether a sidereal longitude falls in a Gandanta zone.
///
/// Returns `true` if the longitude is within 3°20' of a water-fire
/// sign boundary.
pub fn is_gandanta(sidereal_lon_deg: f64) -> bool {
    let (dist, _) = nearest_gandanta_boundary(sidereal_lon_deg);
    dist <= GANDANTA_ORB_DEG
}

/// Determine the severity of Gandanta affliction, if any.
///
/// Returns `None` if the longitude is not in a Gandanta zone.
pub fn gandanta_severity(sidereal_lon_deg: f64) -> Option<GandantaSeverity> {
    let (dist, _) = nearest_gandanta_boundary(sidereal_lon_deg);
    if dist > GANDANTA_ORB_DEG {
        None
    } else if dist <= 1.0 {
        Some(GandantaSeverity::Severe)
    } else if dist <= 2.0 {
        Some(GandantaSeverity::Moderate)
    } else {
        Some(GandantaSeverity::Mild)
    }
}

/// Full Gandanta analysis returning all details.
pub fn gandanta_info(sidereal_lon_deg: f64) -> GandantaInfo {
    let (dist, boundary) = nearest_gandanta_boundary(sidereal_lon_deg);
    let in_gandanta = dist <= GANDANTA_ORB_DEG;
    GandantaInfo {
        in_gandanta,
        severity: if in_gandanta {
            gandanta_severity(sidereal_lon_deg)
        } else {
            None
        },
        nearest_boundary: boundary,
        distance_deg: dist,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Pisces-Aries boundary (0°/360°) ----

    #[test]
    fn exactly_at_0_degrees() {
        assert!(is_gandanta(0.0));
        assert_eq!(gandanta_severity(0.0), Some(GandantaSeverity::Severe));
    }

    #[test]
    fn just_inside_aries_side() {
        // 2° Aries = 2° from boundary → Moderate
        assert!(is_gandanta(2.0));
        assert_eq!(gandanta_severity(2.0), Some(GandantaSeverity::Moderate));
    }

    #[test]
    fn just_inside_pisces_side() {
        // 358° = 28° Pisces = 2° before 360°/0° boundary → Moderate
        assert!(is_gandanta(358.0));
        assert_eq!(gandanta_severity(358.0), Some(GandantaSeverity::Moderate));
    }

    #[test]
    fn mild_gandanta_pisces() {
        // 357° = 27° Pisces = 3° from boundary → Mild
        assert!(is_gandanta(357.0));
        assert_eq!(gandanta_severity(357.0), Some(GandantaSeverity::Mild));
    }

    #[test]
    fn outside_gandanta_aries() {
        // 5° Aries = well outside 3°20' orb
        assert!(!is_gandanta(5.0));
        assert_eq!(gandanta_severity(5.0), None);
    }

    #[test]
    fn outside_gandanta_pisces() {
        // 355° = 25° Pisces = 5° from boundary
        assert!(!is_gandanta(355.0));
    }

    // ---- Cancer-Leo boundary (120°) ----

    #[test]
    fn exactly_at_120() {
        assert!(is_gandanta(120.0));
        assert_eq!(gandanta_severity(120.0), Some(GandantaSeverity::Severe));
    }

    #[test]
    fn cancer_side_of_120() {
        // 119° = 29° Cancer = 1° from boundary → Severe
        assert!(is_gandanta(119.0));
        assert_eq!(gandanta_severity(119.0), Some(GandantaSeverity::Severe));
    }

    #[test]
    fn leo_side_of_120() {
        // 121.5° = 1.5° Leo = 1.5° from boundary → Moderate
        assert!(is_gandanta(121.5));
        assert_eq!(gandanta_severity(121.5), Some(GandantaSeverity::Moderate));
    }

    #[test]
    fn edge_of_gandanta_120() {
        // 3°20' = 3.333... degrees
        // 123.3° is just within the orb
        assert!(is_gandanta(123.3));
        // 123.4° is at the ragged edge — let's check
        let dist = (123.4_f64 - 120.0).abs();
        assert!(dist > GANDANTA_ORB_DEG); // 3.4 > 3.333
        assert!(!is_gandanta(123.4));
    }

    #[test]
    fn outside_cancer_leo_boundary() {
        // 15° Leo = 135° → 15° from 120°
        assert!(!is_gandanta(135.0));
        // 15° Cancer = 105° → 15° from 120°
        assert!(!is_gandanta(105.0));
    }

    // ---- Scorpio-Sagittarius boundary (240°) ----

    #[test]
    fn exactly_at_240() {
        assert!(is_gandanta(240.0));
        assert_eq!(gandanta_severity(240.0), Some(GandantaSeverity::Severe));
    }

    #[test]
    fn scorpio_side_of_240() {
        // 238.5° = 28°30' Scorpio = 1.5° from boundary → Moderate
        assert!(is_gandanta(238.5));
        assert_eq!(gandanta_severity(238.5), Some(GandantaSeverity::Moderate));
    }

    #[test]
    fn sagittarius_side_of_240() {
        // 240.5° = 0°30' Sagittarius = 0.5° from boundary → Severe
        assert!(is_gandanta(240.5));
        assert_eq!(gandanta_severity(240.5), Some(GandantaSeverity::Severe));
    }

    // ---- Wrap-around and normalization ----

    #[test]
    fn negative_longitude() {
        // -1° should normalize to 359° = 1° from 0° boundary → Severe
        assert!(is_gandanta(-1.0));
        assert_eq!(gandanta_severity(-1.0), Some(GandantaSeverity::Severe));
    }

    #[test]
    fn over_360_longitude() {
        // 361° = 1° = near 0° boundary
        assert!(is_gandanta(361.0));
    }

    // ---- Middle of signs (definitely not Gandanta) ----

    #[test]
    fn middle_of_each_sign_not_gandanta() {
        // Test 15° within each sign (i.e., the midpoint)
        for sign_start in (0..12).map(|i| i as f64 * 30.0 + 15.0) {
            assert!(
                !is_gandanta(sign_start),
                "Middle of sign at {sign_start}° should not be Gandanta"
            );
        }
    }

    // ---- GandantaInfo struct ----

    #[test]
    fn gandanta_info_in_zone() {
        let info = gandanta_info(119.5);
        assert!(info.in_gandanta);
        assert_eq!(info.severity, Some(GandantaSeverity::Severe));
        assert!((info.nearest_boundary - 120.0).abs() < 0.001);
        assert!((info.distance_deg - 0.5).abs() < 0.001);
    }

    #[test]
    fn gandanta_info_outside_zone() {
        let info = gandanta_info(180.0);
        assert!(!info.in_gandanta);
        assert_eq!(info.severity, None);
    }

    // ---- Non-gandanta boundaries (fire→earth, earth→air, air→water) ----

    #[test]
    fn aries_taurus_boundary_not_gandanta() {
        // 30° = Aries/Taurus boundary (fire→earth) — NOT gandanta
        assert!(!is_gandanta(30.0));
        assert!(!is_gandanta(29.0));
        assert!(!is_gandanta(31.0));
    }

    #[test]
    fn taurus_gemini_boundary_not_gandanta() {
        // 60° = Taurus/Gemini (earth→air)
        assert!(!is_gandanta(60.0));
    }

    #[test]
    fn gemini_cancer_boundary_not_gandanta() {
        // 90° = Gemini/Cancer (air→water)
        assert!(!is_gandanta(90.0));
    }

    #[test]
    fn leo_virgo_boundary_not_gandanta() {
        // 150° = Leo/Virgo (fire→earth)
        assert!(!is_gandanta(150.0));
    }
}
