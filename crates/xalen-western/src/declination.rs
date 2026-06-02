//! Declination aspects — parallel and contraparallel.
//!
//! A planet's **declination** is its angular distance north (+) or south (−)
//! of the celestial equator. Two bodies are in:
//!
//! - **Parallel** when their declinations are equal in sign and magnitude
//!   (both north or both south) within a small orb. A parallel behaves like a
//!   conjunction — it reinforces.
//! - **Contraparallel** when their declinations are equal in magnitude but
//!   opposite in sign (one north, one south) within orb. A contraparallel
//!   behaves like an opposition.
//!
//! Declination is derived from ecliptic longitude (λ), ecliptic latitude (β)
//! and the obliquity of the ecliptic (ε) by the standard spherical
//! transformation
//!
//! ```text
//! sin δ = sin β · cos ε + cos β · sin ε · sin λ
//! ```
//!
//! This is the declination half of the ecliptic→equatorial rotation
//! (Meeus, *Astronomical Algorithms*, 2nd ed., 1998, Ch. 13, eq. 13.4). The
//! transform itself is reused from [`xalen_coords::ecliptic_to_equatorial`] so
//! that the same well-tested rotation backs every declination figure.
//!
//! Typical declination orbs are tight — most practitioners use 1° for the
//! Sun/Moon and roughly 30′–1° for the rest (the orb is a caller parameter
//! here; 1° is a sensible default).

use serde::{Deserialize, Serialize};
use xalen_coords::transforms::{EclipticPosition, ecliptic_to_equatorial};

/// Default declination orb in degrees (1° is the common practitioner value).
pub const DEFAULT_DECLINATION_ORB: f64 = 1.0;

/// The kind of declination contact between two bodies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeclinationAspect {
    /// Declinations equal in sign and magnitude (both N or both S) — acts like
    /// a conjunction.
    Parallel,
    /// Declinations equal in magnitude but opposite in sign — acts like an
    /// opposition.
    Contraparallel,
}

/// A detected declination contact between two named bodies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclinationContact {
    pub aspect: DeclinationAspect,
    pub body1: String,
    pub body2: String,
    /// Declination of `body1` in degrees (north positive).
    pub dec1_deg: f64,
    /// Declination of `body2` in degrees (north positive).
    pub dec2_deg: f64,
    /// Orb of the contact in degrees (always ≥ 0).
    pub orb_deg: f64,
}

/// Compute the declination (degrees, north positive) of a body from its
/// ecliptic longitude, ecliptic latitude and the obliquity of the ecliptic.
///
/// All inputs are in **degrees**. Uses the standard ecliptic→equatorial
/// rotation: `sin δ = sin β · cos ε + cos β · sin ε · sin λ`
/// (Meeus 1998, eq. 13.4).
pub fn declination_deg(lon_deg: f64, lat_deg: f64, obliquity_deg: f64) -> f64 {
    let ecl = EclipticPosition {
        longitude: lon_deg.to_radians(),
        latitude: lat_deg.to_radians(),
        distance: 1.0,
    };
    let eq = ecliptic_to_equatorial(&ecl, obliquity_deg.to_radians());
    eq.declination.to_degrees()
}

/// Classify the declination contact between two declinations (in degrees).
///
/// Returns `None` if the bodies are outside `orb` on both the parallel and
/// contraparallel tests.
///
/// * **Parallel** — same sign, `|δ₁ − δ₂| ≤ orb`.
/// * **Contraparallel** — opposite sign, `|δ₁ + δ₂| ≤ orb` (i.e. magnitudes
///   match while signs differ).
///
/// When a body sits exactly on the equator (δ = 0) it has no sign, so it can
/// only form a parallel/contraparallel with another body that is itself within
/// `orb` of the equator; in that degenerate case the contact is reported as a
/// `Parallel` (both effectively on the equator).
pub fn classify_declination(
    dec1_deg: f64,
    dec2_deg: f64,
    orb_deg: f64,
) -> Option<(DeclinationAspect, f64)> {
    let parallel_orb = (dec1_deg - dec2_deg).abs();
    let contra_orb = (dec1_deg + dec2_deg).abs();

    // Prefer whichever contact is tighter when both happen to be in orb
    // (this only occurs near the equator where δ ≈ 0).
    let parallel_ok = parallel_orb <= orb_deg;
    let contra_ok = contra_orb <= orb_deg;

    match (parallel_ok, contra_ok) {
        (true, true) => {
            if parallel_orb <= contra_orb {
                Some((DeclinationAspect::Parallel, parallel_orb))
            } else {
                Some((DeclinationAspect::Contraparallel, contra_orb))
            }
        }
        (true, false) => Some((DeclinationAspect::Parallel, parallel_orb)),
        (false, true) => Some((DeclinationAspect::Contraparallel, contra_orb)),
        (false, false) => None,
    }
}

/// Detect all parallel and contraparallel contacts in a chart.
///
/// Each input is `(body_name, declination_degrees)` with north positive. Use
/// [`declination_deg`] to compute declinations from ecliptic coordinates first
/// if you only have longitude/latitude.
///
/// Returns one [`DeclinationContact`] per pair that is in orb, sorted tightest
/// orb first.
pub fn detect_declination_aspects(
    declinations: &[(&str, f64)],
    orb_deg: f64,
) -> Vec<DeclinationContact> {
    let mut out = Vec::new();
    let n = declinations.len();
    for i in 0..n {
        for j in (i + 1)..n {
            let (name1, d1) = declinations[i];
            let (name2, d2) = declinations[j];
            if let Some((aspect, orb)) = classify_declination(d1, d2, orb_deg) {
                out.push(DeclinationContact {
                    aspect,
                    body1: name1.to_string(),
                    body2: name2.to_string(),
                    dec1_deg: d1,
                    dec2_deg: d2,
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

/// Convenience: detect declination aspects directly from ecliptic positions.
///
/// Each input is `(body_name, longitude_deg, latitude_deg)`. The obliquity is
/// supplied in degrees (use [`xalen_coords::mean_obliquity`] for the mean
/// obliquity at a given epoch). Internally computes each body's declination
/// then delegates to [`detect_declination_aspects`].
pub fn detect_declination_aspects_from_ecliptic(
    positions: &[(&str, f64, f64)],
    obliquity_deg: f64,
    orb_deg: f64,
) -> Vec<DeclinationContact> {
    let decs: Vec<(&str, f64)> = positions
        .iter()
        .map(|&(name, lon, lat)| (name, declination_deg(lon, lat, obliquity_deg)))
        .collect();
    detect_declination_aspects(&decs, orb_deg)
}

// ── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Mean obliquity at J2000.0, degrees (IAU). Used for fixture geometry.
    const EPS_J2000: f64 = 23.439_291_11;

    #[test]
    fn declination_zero_on_equator_points() {
        // 0° Aries (λ=0, β=0) and 0° Libra (λ=180, β=0) lie on the equator.
        assert!(declination_deg(0.0, 0.0, EPS_J2000).abs() < 1e-9);
        assert!(declination_deg(180.0, 0.0, EPS_J2000).abs() < 1e-9);
    }

    #[test]
    fn declination_at_solstices_equals_obliquity() {
        // 0° Cancer (λ=90, β=0) → max north declination = +ε.
        let dec_cancer = declination_deg(90.0, 0.0, EPS_J2000);
        assert!(
            (dec_cancer - EPS_J2000).abs() < 1e-6,
            "0 Cancer declination should equal +obliquity, got {dec_cancer}"
        );
        // 0° Capricorn (λ=270, β=0) → max south declination = −ε.
        let dec_cap = declination_deg(270.0, 0.0, EPS_J2000);
        assert!(
            (dec_cap + EPS_J2000).abs() < 1e-6,
            "0 Capricorn declination should equal −obliquity, got {dec_cap}"
        );
    }

    #[test]
    fn declination_matches_meeus_spherical_formula() {
        // Independent re-derivation of Meeus eq. 13.4 to guard the rotation reuse.
        let (lon, lat) = (137.5_f64, 4.3_f64);
        let eps = EPS_J2000;
        let manual = (lat.to_radians().sin() * eps.to_radians().cos()
            + lat.to_radians().cos() * eps.to_radians().sin() * lon.to_radians().sin())
        .asin()
        .to_degrees();
        let got = declination_deg(lon, lat, eps);
        assert!(
            (manual - got).abs() < 1e-9,
            "declination_deg should match Meeus 13.4: {manual} vs {got}"
        );
    }

    #[test]
    fn parallel_same_sign_in_orb() {
        // +20.0 and +20.5 → parallel, orb 0.5.
        let (asp, orb) = classify_declination(20.0, 20.5, 1.0).expect("should be parallel");
        assert_eq!(asp, DeclinationAspect::Parallel);
        assert!((orb - 0.5).abs() < 1e-9, "orb should be 0.5, got {orb}");
    }

    #[test]
    fn parallel_out_of_orb() {
        // +20.0 and +22.0, orb 1.0 → no contact.
        assert!(classify_declination(20.0, 22.0, 1.0).is_none());
    }

    #[test]
    fn contraparallel_opposite_sign_in_orb() {
        // +18.0 and −18.4 → contraparallel, orb 0.4.
        let r = classify_declination(18.0, -18.4, 1.0);
        assert!(r.is_some());
        let (asp, orb) = r.unwrap();
        assert_eq!(asp, DeclinationAspect::Contraparallel);
        assert!((orb - 0.4).abs() < 1e-9, "orb should be 0.4, got {orb}");
    }

    #[test]
    fn opposite_sign_large_magnitude_is_not_contraparallel() {
        // +20 and −5: magnitudes differ by 15 → neither parallel nor contraparallel.
        assert!(classify_declination(20.0, -5.0, 1.0).is_none());
    }

    #[test]
    fn detect_finds_both_kinds() {
        let decs = vec![
            ("Sun", 23.0),
            ("Venus", 22.7), // parallel to Sun (orb 0.3)
            ("Mars", -23.2), // contraparallel to Sun (orb 0.2), to Venus (orb 0.5)
        ];
        let contacts = detect_declination_aspects(&decs, 1.0);
        assert!(
            contacts
                .iter()
                .any(|c| c.aspect == DeclinationAspect::Parallel
                    && ((c.body1 == "Sun" && c.body2 == "Venus")
                        || (c.body1 == "Venus" && c.body2 == "Sun"))),
            "Sun/Venus parallel expected: {contacts:#?}"
        );
        assert!(
            contacts
                .iter()
                .any(|c| c.aspect == DeclinationAspect::Contraparallel
                    && ((c.body1 == "Sun" && c.body2 == "Mars")
                        || (c.body1 == "Mars" && c.body2 == "Sun"))),
            "Sun/Mars contraparallel expected: {contacts:#?}"
        );
    }

    #[test]
    fn detect_sorted_by_orb() {
        let decs = vec![("A", 10.0), ("B", 10.8), ("C", 10.1)];
        let contacts = detect_declination_aspects(&decs, 1.0);
        for w in contacts.windows(2) {
            assert!(w[0].orb_deg <= w[1].orb_deg + 1e-12);
        }
    }

    #[test]
    fn from_ecliptic_solstice_pair_is_parallel() {
        // Two bodies near 0° Cancer share the same (max north) declination →
        // parallel. λ=88° and λ=92° (β=0) are symmetric about the solstice and
        // have nearly identical declination.
        let positions = vec![("X", 88.0, 0.0), ("Y", 92.0, 0.0)];
        let contacts = detect_declination_aspects_from_ecliptic(&positions, EPS_J2000, 1.0);
        assert!(
            contacts
                .iter()
                .any(|c| c.aspect == DeclinationAspect::Parallel),
            "λ=88 and λ=92 should be parallel near the solstice: {contacts:#?}"
        );
    }

    #[test]
    fn from_ecliptic_antiscia_share_declination() {
        // A longitude and its antiscion (180−λ) have identical declination →
        // they form a parallel. λ=30 ↔ antiscion 150.
        let positions = vec![("P", 30.0, 0.0), ("Q", 150.0, 0.0)];
        let contacts = detect_declination_aspects_from_ecliptic(&positions, EPS_J2000, 0.01);
        assert!(
            contacts
                .iter()
                .any(|c| c.aspect == DeclinationAspect::Parallel && c.orb_deg < 0.01),
            "antiscion pair must be exactly parallel in declination: {contacts:#?}"
        );
    }
}
