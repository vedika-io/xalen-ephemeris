//! Antiscia and contra-antiscia — reflections across the solstitial axis.
//!
//! The **antiscion** of a point is its mirror image across the 0° Cancer /
//! 0° Capricorn axis (the solstitial axis, at ecliptic longitude 90° and
//! 270°). Two points that are antiscia of each other are equidistant from the
//! solstice points and therefore share the **same solar declination** — they
//! are "equal in light." The reflection across the line through 90° and 270°
//! is `x' = 2·90° − x`, i.e.
//!
//! ```text
//! antiscion(λ)        = (180° − λ)  mod 360°
//! contra-antiscion(λ) = (antiscion(λ) + 180°) mod 360° = (360° − λ) mod 360°
//! ```
//!
//! The contra-antiscion is the mirror across the **0° Aries / 0° Libra**
//! equinoctial axis (the point opposite the antiscion). Contacts are formed
//! when a body sits on another body's antiscion (a conjunction-like, harmonious
//! contact) or contra-antiscion (an opposition-like, tense contact).
//!
//! Canonical sign pairs produced by this reflection (Brennan, *Hellenistic
//! Astrology*, 2017; Lilly, *Christian Astrology*, 1647):
//! Aries↔Virgo, Taurus↔Leo, Gemini↔Cancer, Libra↔Pisces, Scorpio↔Aquarius,
//! Sagittarius↔Capricorn.
//!
//! This is pure longitude geometry — no latitude, declination, or epoch is
//! involved.

use crate::aspects::angular_distance;
use serde::{Deserialize, Serialize};

/// Default orb (degrees) for antiscia contacts. Antiscia are traditionally
/// worked tight; 1° is a common practitioner value.
pub const DEFAULT_ANTISCIA_ORB: f64 = 1.0;

/// Compute the antiscion of an ecliptic longitude (degrees).
///
/// Reflection across the 0° Cancer / 0° Capricorn solstitial axis:
/// `antiscion = (180° − λ) mod 360°`.
pub fn antiscion(lon_deg: f64) -> f64 {
    (180.0 - lon_deg).rem_euclid(360.0)
}

/// Compute the contra-antiscion of an ecliptic longitude (degrees).
///
/// The point opposite the antiscion — reflection across the 0° Aries /
/// 0° Libra equinoctial axis: `contra = (360° − λ) mod 360°` (equivalently the
/// negation, `−λ mod 360°`).
pub fn contra_antiscion(lon_deg: f64) -> f64 {
    (360.0 - lon_deg).rem_euclid(360.0)
}

/// The kind of antiscia contact between two bodies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AntisciaContactKind {
    /// One body sits on the other's antiscion (solstitial mirror) — a
    /// conjunction-like, supportive contact.
    Antiscion,
    /// One body sits on the other's contra-antiscion (equinoctial mirror) — an
    /// opposition-like, challenging contact.
    ContraAntiscion,
}

/// A detected antiscia contact between two named bodies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntisciaContact {
    pub kind: AntisciaContactKind,
    pub body1: String,
    pub body2: String,
    /// Longitude of `body1` (degrees).
    pub lon1_deg: f64,
    /// Longitude of `body2` (degrees).
    pub lon2_deg: f64,
    /// The reflected point of `body1` that `body2` falls on (degrees): the
    /// antiscion for an [`AntisciaContactKind::Antiscion`] contact, or the
    /// contra-antiscion for a [`AntisciaContactKind::ContraAntiscion`] contact.
    pub axis_point_deg: f64,
    /// Orb of the contact (degrees, always ≥ 0).
    pub orb_deg: f64,
}

/// Classify the antiscia contact between two longitudes (degrees), if any.
///
/// Returns the tighter of the antiscion / contra-antiscion contact when both
/// are within `orb` (this only happens when both bodies sit on the solstitial
/// or equinoctial axis itself). The reflection is applied to `lon1`; because
/// the relation is symmetric, the same contact is found regardless of which
/// body is reflected.
pub fn classify_antiscia(
    lon1_deg: f64,
    lon2_deg: f64,
    orb_deg: f64,
) -> Option<(AntisciaContactKind, f64, f64)> {
    let ant = antiscion(lon1_deg);
    let contra = contra_antiscion(lon1_deg);
    let ant_orb = angular_distance(ant, lon2_deg);
    let contra_orb = angular_distance(contra, lon2_deg);

    let ant_ok = ant_orb <= orb_deg;
    let contra_ok = contra_orb <= orb_deg;

    match (ant_ok, contra_ok) {
        (true, true) => {
            if ant_orb <= contra_orb {
                Some((AntisciaContactKind::Antiscion, ant, ant_orb))
            } else {
                Some((AntisciaContactKind::ContraAntiscion, contra, contra_orb))
            }
        }
        (true, false) => Some((AntisciaContactKind::Antiscion, ant, ant_orb)),
        (false, true) => Some((AntisciaContactKind::ContraAntiscion, contra, contra_orb)),
        (false, false) => None,
    }
}

/// Detect all antiscia and contra-antiscia contacts in a chart.
///
/// Each input is `(body_name, longitude_degrees)`. Returns one
/// [`AntisciaContact`] per pair in orb, sorted tightest orb first.
pub fn detect_antiscia(positions: &[(&str, f64)], orb_deg: f64) -> Vec<AntisciaContact> {
    let mut out = Vec::new();
    let n = positions.len();
    for i in 0..n {
        for j in (i + 1)..n {
            let (name1, lon1) = positions[i];
            let (name2, lon2) = positions[j];
            if let Some((kind, axis_point, orb)) = classify_antiscia(lon1, lon2, orb_deg) {
                out.push(AntisciaContact {
                    kind,
                    body1: name1.to_string(),
                    body2: name2.to_string(),
                    lon1_deg: lon1,
                    lon2_deg: lon2,
                    axis_point_deg: axis_point,
                    orb_deg: orb,
                });
            }
        }
    }
    out.sort_by(|a, b| {
        a.orb_deg
            .partial_cmp(&b.orb_deg)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out
}

// ── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn antiscion_solstice_points_fixed() {
        // The solstice points are their own antiscia (on the mirror axis).
        assert!(
            (antiscion(90.0) - 90.0).abs() < 1e-9,
            "0 Cancer is self-antiscion"
        );
        assert!(
            (antiscion(270.0) - 270.0).abs() < 1e-9,
            "0 Cap is self-antiscion"
        );
    }

    #[test]
    fn antiscion_canonical_sign_pairs() {
        // Mid-sign degree -> mid of the partner sign (Brennan / Lilly tables).
        // Taurus 15° (=45°) ↔ Leo 15° (=135°).
        assert!((antiscion(45.0) - 135.0).abs() < 1e-9);
        // Gemini 15° (=75°) ↔ Cancer 15° (=105°).
        assert!((antiscion(75.0) - 105.0).abs() < 1e-9);
        // Aries 15° (=15°) ↔ Virgo 15° (=165°).
        assert!((antiscion(15.0) - 165.0).abs() < 1e-9);
        // Scorpio 15° (=225°) ↔ Aquarius 15° (=315°).
        assert!((antiscion(225.0) - 315.0).abs() < 1e-9);
    }

    #[test]
    fn antiscion_is_involution() {
        // Reflecting twice returns the original longitude.
        for lon in [0.0, 13.3, 90.0, 200.7, 359.9] {
            let back = antiscion(antiscion(lon));
            assert!(
                (angular_distance(back, lon)).abs() < 1e-9,
                "antiscion∘antiscion = id for {lon}"
            );
        }
    }

    #[test]
    fn contra_antiscion_is_opposite_of_antiscion() {
        for lon in [0.0, 30.0, 137.5, 280.0] {
            let diff = angular_distance(antiscion(lon), contra_antiscion(lon));
            assert!(
                (diff - 180.0).abs() < 1e-9,
                "contra should be 180° from antiscion for {lon}"
            );
        }
    }

    #[test]
    fn contra_antiscion_equinox_points_fixed() {
        // 0 Aries and 0 Libra lie on the equinoctial axis → self-contra-antiscia.
        // contra_antiscion(0) normalizes to 0° (the same point as 0 Aries).
        assert!(
            angular_distance(contra_antiscion(0.0), 0.0) < 1e-9,
            "0 Aries is self contra-antiscion"
        );
        assert!(
            angular_distance(contra_antiscion(180.0), 180.0) < 1e-9,
            "0 Libra is self contra-antiscion"
        );
    }

    #[test]
    fn detect_antiscion_contact() {
        // Sun at 30° (Taurus 0°); Moon at 150.4° (near Virgo 0° = antiscion 150°).
        let positions = vec![("Sun", 30.0), ("Moon", 150.4)];
        let contacts = detect_antiscia(&positions, 1.0);
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].kind, AntisciaContactKind::Antiscion);
        assert!(
            (contacts[0].orb_deg - 0.4).abs() < 1e-9,
            "orb should be 0.4°"
        );
    }

    #[test]
    fn detect_contra_antiscion_contact() {
        // Sun at 30°; antiscion 150°, contra-antiscion 330°.
        // Mars at 329.5° → contra-antiscion contact, orb 0.5°.
        let positions = vec![("Sun", 30.0), ("Mars", 329.5)];
        let contacts = detect_antiscia(&positions, 1.0);
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].kind, AntisciaContactKind::ContraAntiscion);
        assert!(
            (contacts[0].orb_deg - 0.5).abs() < 1e-9,
            "orb should be 0.5°"
        );
        assert!((contacts[0].axis_point_deg - 330.0).abs() < 1e-9);
    }

    #[test]
    fn no_contact_when_out_of_orb() {
        // Sun at 30° (antiscion 150, contra 330); Venus at 200° is far from both.
        let positions = vec![("Sun", 30.0), ("Venus", 200.0)];
        assert!(detect_antiscia(&positions, 1.0).is_empty());
    }

    #[test]
    fn contact_is_symmetric() {
        // The relation must be found regardless of input order.
        let ab = detect_antiscia(&[("A", 30.0), ("B", 150.0)], 1.0);
        let ba = detect_antiscia(&[("B", 150.0), ("A", 30.0)], 1.0);
        assert_eq!(ab.len(), 1);
        assert_eq!(ba.len(), 1);
        assert_eq!(ab[0].kind, ba[0].kind);
        assert!((ab[0].orb_deg - ba[0].orb_deg).abs() < 1e-9);
    }

    #[test]
    fn detect_sorted_by_orb() {
        // Sun at 0° → antiscion 180°, contra 0°. Several partners at varying orb.
        let positions = vec![
            ("Sun", 0.0),
            ("A", 181.0), // antiscion orb 1.0
            ("B", 180.3), // antiscion orb 0.3
            ("C", 179.6), // antiscion orb 0.4
        ];
        let contacts = detect_antiscia(&positions, 2.0);
        for w in contacts.windows(2) {
            assert!(w[0].orb_deg <= w[1].orb_deg + 1e-12, "must be orb-sorted");
        }
    }
}
