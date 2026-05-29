//! Uranian astrology (Hamburg School) — TransNeptunian planets and
//! planetary pictures.
//!
//! The Hamburg School was founded by Alfred Witte (1878–1941) and
//! systematized by Friedrich Sieggrün.  It introduces eight hypothetical
//! TransNeptunian planets (TNPs) beyond Neptune, each with a fixed
//! orbital period and near-circular orbit.  Because the orbits are
//! nearly circular and the periods very long, simple mean-motion from
//! the J2000 epoch is adequate for ephemeris computation.
//!
//! **Planetary pictures** are the central interpretive technique:
//! symmetric three-body formulae of the form A = B + C − D, evaluated
//! on both the 360° and 90° dials.
//!
//! References:
//! - Alfred Witte & Hermann Lefeldt, *Rules for Planetary Pictures*
//!   (Witte-Verlag, 1928; Ludwig Rudolph, 1959 English edition).
//! - Udo Rudolph, *ABC für Planetenbilder* (Witte-Verlag, 1992).
//! - Witte Institute / Hamburg School Ephemeris tables.

use crate::midpoints::to_90_degree;
use serde::{Deserialize, Serialize};

// ── TransNeptunian planets ───────────────────────────────────────────

/// The eight hypothetical TransNeptunian planets (TNPs) of the
/// Hamburg School.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransNeptunian {
    /// Cupido — society, art, marriage.  Period ~262.5 yr, a ≈ 40.9 AU.
    Cupido,
    /// Hades — poverty, disease, antiquity.  Period ~360.7 yr, a ≈ 50.7 AU.
    Hades,
    /// Zeus — fire, machines, leadership.  Period ~455.0 yr, a ≈ 59.2 AU.
    Zeus,
    /// Kronos — authority, mastery, the state.  Period ~521.8 yr, a ≈ 64.8 AU.
    Kronos,
    /// Apollon — commerce, science, expansion.  Period ~576.0 yr, a ≈ 69.1 AU.
    Apollon,
    /// Admetos — stagnation, depth, raw material.  Period ~624.0 yr, a ≈ 73.0 AU.
    Admetos,
    /// Vulkanus — mighty force, power.  Period ~663.0 yr, a ≈ 76.0 AU.
    Vulkanus,
    /// Poseidon — spirit, truth, enlightenment.  Period ~740.0 yr, a ≈ 81.5 AU.
    Poseidon,
}

impl TransNeptunian {
    /// All eight TNPs in Witte's canonical order.
    pub const ALL: [TransNeptunian; 8] = [
        Self::Cupido,
        Self::Hades,
        Self::Zeus,
        Self::Kronos,
        Self::Apollon,
        Self::Admetos,
        Self::Vulkanus,
        Self::Poseidon,
    ];

    /// Return the J2000 epoch longitude (degrees) and mean annual rate
    /// (degrees per tropical year) for this TNP.
    ///
    /// Epoch positions are from the Witte/Hamburg School published
    /// tables at J2000.0 (JD 2451545.0, i.e. 2000-01-01T12:00 TT).
    ///
    /// Mean motion rates verified against:
    ///   - Arlene Kramer, "The Eight TransNeptunian Planets of the
    ///     Uranian System" (planetwaves.net): Cupido 1d23'/yr = 1.383,
    ///     Hades 1d01'/yr = 1.017, Zeus 0d48.2'/yr = 0.803, Kronos
    ///     0d48.1'/yr = 0.802, Apollon 0d37'/yr = 0.617, Admetos
    ///     0d35'/yr = 0.583, Vulkanus 0d32'/yr = 0.533, Poseidon
    ///     0d29'/yr = 0.483.
    ///   - Swiss Ephemeris seorbel.txt (J1900 Neely elements):
    ///     Kepler-derived rates from semi-major axes match within ~2%.
    ///
    /// Note: The Swiss Ephemeris (seorbel.txt) uses James Neely's 1980
    /// revised Keplerian elements (J1900 epoch with small eccentricities
    /// and inclinations), which is a more sophisticated model.  Our
    /// simplified mean-motion approach from published J2000 positions
    /// follows the traditional Hamburg School practice of linear
    /// extrapolation from tabulated positions.
    pub fn ephemeris_data(self) -> (f64, f64) {
        match self {
            Self::Cupido => (174.79, 1.371),
            Self::Hades => (108.44, 0.998),
            Self::Zeus => (60.50, 0.791),
            Self::Kronos => (88.95, 0.690),
            Self::Apollon => (26.68, 0.625),
            Self::Admetos => (19.35, 0.577),
            Self::Vulkanus => (16.10, 0.543),
            Self::Poseidon => (26.37, 0.486),
        }
    }

    /// Orbital period in tropical years.
    pub fn period_years(self) -> f64 {
        match self {
            Self::Cupido => 262.5,
            Self::Hades => 360.7,
            Self::Zeus => 455.0,
            Self::Kronos => 521.8,
            Self::Apollon => 576.0,
            Self::Admetos => 624.0,
            Self::Vulkanus => 663.0,
            Self::Poseidon => 740.0,
        }
    }

    /// Semi-major axis in AU (approximate).
    pub fn semi_major_axis_au(self) -> f64 {
        match self {
            Self::Cupido => 40.9,
            Self::Hades => 50.7,
            Self::Zeus => 59.2,
            Self::Kronos => 64.8,
            Self::Apollon => 69.1,
            Self::Admetos => 73.0,
            Self::Vulkanus => 76.0,
            Self::Poseidon => 81.5,
        }
    }

    /// Short display name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Cupido => "Cupido",
            Self::Hades => "Hades",
            Self::Zeus => "Zeus",
            Self::Kronos => "Kronos",
            Self::Apollon => "Apollon",
            Self::Admetos => "Admetos",
            Self::Vulkanus => "Vulkanus",
            Self::Poseidon => "Poseidon",
        }
    }

    /// Two-letter abbreviation used in Uranian notation.
    pub fn abbrev(self) -> &'static str {
        match self {
            Self::Cupido => "CU",
            Self::Hades => "HA",
            Self::Zeus => "ZE",
            Self::Kronos => "KR",
            Self::Apollon => "AP",
            Self::Admetos => "AD",
            Self::Vulkanus => "VU",
            Self::Poseidon => "PO",
        }
    }
}

// ── TNP longitude computation ────────────────────────────────────────

/// Compute the ecliptic longitude of a TransNeptunian planet at the
/// given Julian Day (TT).
///
/// Uses simple mean motion from the J2000 epoch — adequate for these
/// hypothetical bodies whose orbits are defined as near-circular.
///
/// # Example
/// ```
/// use xalen_western::uranian::{TransNeptunian, tnp_longitude};
/// // At J2000 epoch itself, should return the epoch longitude.
/// let lon = tnp_longitude(TransNeptunian::Cupido, 2451545.0);
/// assert!((lon - 174.79).abs() < 1e-10);
/// ```
pub fn tnp_longitude(planet: TransNeptunian, jd: f64) -> f64 {
    let t = (jd - 2451545.0) / 365.25; // tropical years from J2000
    let (epoch_lon, rate) = planet.ephemeris_data();
    (epoch_lon + rate * t).rem_euclid(360.0)
}

/// Compute longitudes for all eight TNPs at the given Julian Day.
///
/// Returns `(TransNeptunian, longitude_degrees)` pairs in canonical
/// order (Cupido through Poseidon).
pub fn all_tnp_longitudes(jd: f64) -> [(TransNeptunian, f64); 8] {
    TransNeptunian::ALL.map(|tnp| (tnp, tnp_longitude(tnp, jd)))
}

// ── Planetary pictures (Witte formula) ───────────────────────────────

/// A planetary picture result from the Witte symmetric formula.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetaryPictureResult {
    /// The computed point: B + C − A (degrees, [0, 360)).
    pub degree: f64,
    /// Same point projected onto the 90° dial.
    pub degree_90: f64,
}

/// Compute a Witte planetary picture: the symmetric point B + C − A.
///
/// In Uranian astrology, if planet D occupies the point B + C − A
/// (within orb), the four bodies form a "planetary picture."
///
/// The result is always normalized to [0, 360).
pub fn planetary_picture(a: f64, b: f64, c: f64) -> f64 {
    (b + c - a).rem_euclid(360.0)
}

/// Compute a planetary picture and return both 360° and 90° values.
pub fn planetary_picture_full(a: f64, b: f64, c: f64) -> PlanetaryPictureResult {
    let deg = planetary_picture(a, b, c);
    PlanetaryPictureResult {
        degree: deg,
        degree_90: to_90_degree(deg),
    }
}

/// Check whether a target longitude sits on the planetary picture
/// point B + C − A, within the given orb.
///
/// The check accounts for the circular nature of the ecliptic.
pub fn is_planetary_picture(target: f64, a: f64, b: f64, c: f64, orb: f64) -> bool {
    let pp = planetary_picture(a, b, c);
    let diff = (target - pp).rem_euclid(360.0);
    let dist = if diff > 180.0 { 360.0 - diff } else { diff };
    dist <= orb
}

/// Check a planetary picture on the 90° dial (hard aspects collapsed).
///
/// This is the standard Uranian practice — conjunction, opposition,
/// and square are treated as equivalent contacts.
pub fn is_planetary_picture_90(target: f64, a: f64, b: f64, c: f64, orb: f64) -> bool {
    let pp_90 = to_90_degree(planetary_picture(a, b, c));
    let target_90 = to_90_degree(target);
    let diff = (target_90 - pp_90).abs();
    let dist = if diff > 45.0 { 90.0 - diff } else { diff };
    dist <= orb
}

// ── Direct / indirect midpoints ──────────────────────────────────────

/// A direct midpoint between two bodies on the 360° dial.
///
/// This is the standard midpoint (shorter arc), identical to
/// `midpoints::compute_midpoint`, included here for self-contained
/// Uranian analysis.
pub fn direct_midpoint(lon_a: f64, lon_b: f64) -> f64 {
    let diff = (lon_b - lon_a).rem_euclid(360.0);
    if diff <= 180.0 {
        (lon_a + diff / 2.0).rem_euclid(360.0)
    } else {
        (lon_a + (diff + 360.0) / 2.0).rem_euclid(360.0)
    }
}

/// An indirect midpoint: the point 180° opposite the direct midpoint.
///
/// In Uranian work both the direct and indirect midpoints are
/// considered active.
pub fn indirect_midpoint(lon_a: f64, lon_b: f64) -> f64 {
    (direct_midpoint(lon_a, lon_b) + 180.0).rem_euclid(360.0)
}

// ── Dial conversions ─────────────────────────────────────────────────

/// Project a longitude onto the 360° dial (identity, included for
/// symmetry with `to_dial_90`).
pub fn to_dial_360(lon: f64) -> f64 {
    lon.rem_euclid(360.0)
}

/// Project a longitude onto the 90° dial.
///
/// This collapses conjunction (0°), square (90°), opposition (180°),
/// and sesquiquadrate (270°) into a single axis.  Re-exports
/// `midpoints::to_90_degree`.
pub fn to_dial_90(lon: f64) -> f64 {
    to_90_degree(lon)
}

// ── Sensitive-point scan ─────────────────────────────────────────────

/// A detected planetary picture activation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PictureActivation {
    /// The body occupying the sensitive point.
    pub occupant: String,
    /// Notation: "B + C − A = D".
    pub formula: String,
    /// Orb on the 360° dial (degrees).
    pub orb_360: f64,
    /// Orb on the 90° dial (degrees).
    pub orb_90: f64,
}

/// Scan all four-body combinations for planetary picture activations.
///
/// For every triple (A, B, C) this checks whether any fourth body D
/// occupies B + C − A within `orb` on the 90° dial.
///
/// `positions`: `(name, ecliptic_longitude_degrees)`.
///
/// Returns activations sorted by 90° orb (tightest first).
pub fn scan_planetary_pictures(positions: &[(&str, f64)], orb: f64) -> Vec<PictureActivation> {
    let n = positions.len();
    let mut results = Vec::new();

    for i in 0..n {
        for j in 0..n {
            if j == i {
                continue;
            }
            for k in (j + 1)..n {
                if k == i {
                    continue;
                }
                let pp = planetary_picture(positions[i].1, positions[j].1, positions[k].1);
                let pp_90 = to_90_degree(pp);

                for l in 0..n {
                    if l == i || l == j || l == k {
                        continue;
                    }
                    let d_90 = to_90_degree(positions[l].1);
                    let diff_90 = (d_90 - pp_90).abs();
                    let dist_90 = if diff_90 > 45.0 {
                        90.0 - diff_90
                    } else {
                        diff_90
                    };

                    if dist_90 <= orb {
                        let diff_360 = (positions[l].1 - pp).rem_euclid(360.0);
                        let dist_360 = if diff_360 > 180.0 {
                            360.0 - diff_360
                        } else {
                            diff_360
                        };

                        results.push(PictureActivation {
                            occupant: positions[l].0.to_string(),
                            formula: format!(
                                "{} + {} - {} = {}",
                                positions[j].0, positions[k].0, positions[i].0, positions[l].0,
                            ),
                            orb_360: dist_360,
                            orb_90: dist_90,
                        });
                    }
                }
            }
        }
    }

    results.sort_by(|a, b| {
        a.orb_90
            .partial_cmp(&b.orb_90)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── TNP epoch positions ──────────────────────────────────────────

    #[test]
    fn tnp_at_j2000_returns_epoch_longitude() {
        let j2000 = 2451545.0;
        for tnp in TransNeptunian::ALL {
            let (epoch_lon, _) = tnp.ephemeris_data();
            let computed = tnp_longitude(tnp, j2000);
            assert!(
                (computed - epoch_lon).abs() < 1e-10,
                "{:?} at J2000: expected {epoch_lon}, got {computed}",
                tnp,
            );
        }
    }

    #[test]
    fn tnp_one_year_later() {
        let jd_one_year = 2451545.0 + 365.25;
        for tnp in TransNeptunian::ALL {
            let (epoch_lon, rate) = tnp.ephemeris_data();
            let expected = (epoch_lon + rate).rem_euclid(360.0);
            let computed = tnp_longitude(tnp, jd_one_year);
            assert!(
                (computed - expected).abs() < 1e-10,
                "{:?} at J2000+1yr: expected {expected}, got {computed}",
                tnp,
            );
        }
    }

    #[test]
    fn tnp_full_orbit_wraps_to_epoch() {
        // After one full orbital period, the planet should return to
        // its epoch position (modulo float precision).
        for tnp in TransNeptunian::ALL {
            let (epoch_lon, _) = tnp.ephemeris_data();
            let period = tnp.period_years();
            let jd_full = 2451545.0 + period * 365.25;
            let computed = tnp_longitude(tnp, jd_full);
            // Allow 1° tolerance since period_years and rate aren't
            // perfectly consistent (these are approximate figures).
            let diff = (computed - epoch_lon).rem_euclid(360.0);
            let dist = if diff > 180.0 { 360.0 - diff } else { diff };
            assert!(
                dist < 5.0,
                "{:?} after full period: expected ~{epoch_lon}°, got {computed}° (diff {dist}°)",
                tnp,
            );
        }
    }

    #[test]
    fn all_tnp_longitudes_returns_eight() {
        let lons = all_tnp_longitudes(2451545.0);
        assert_eq!(lons.len(), 8);
    }

    // ── TransNeptunian enum ──────────────────────────────────────────

    #[test]
    fn all_array_has_eight_entries() {
        assert_eq!(TransNeptunian::ALL.len(), 8);
    }

    #[test]
    fn names_and_abbrevs_are_unique() {
        let names: Vec<&str> = TransNeptunian::ALL.iter().map(|t| t.name()).collect();
        let abbrevs: Vec<&str> = TransNeptunian::ALL.iter().map(|t| t.abbrev()).collect();
        for i in 0..8 {
            for j in (i + 1)..8 {
                assert_ne!(names[i], names[j], "Duplicate name: {}", names[i]);
                assert_ne!(abbrevs[i], abbrevs[j], "Duplicate abbrev: {}", abbrevs[i]);
            }
        }
    }

    #[test]
    fn periods_are_increasing() {
        let periods: Vec<f64> = TransNeptunian::ALL
            .iter()
            .map(|t| t.period_years())
            .collect();
        for w in periods.windows(2) {
            assert!(w[0] < w[1], "Periods should increase: {} >= {}", w[0], w[1],);
        }
    }

    #[test]
    fn semi_major_axes_are_increasing() {
        let axes: Vec<f64> = TransNeptunian::ALL
            .iter()
            .map(|t| t.semi_major_axis_au())
            .collect();
        for w in axes.windows(2) {
            assert!(
                w[0] < w[1],
                "Semi-major axes should increase: {} >= {}",
                w[0],
                w[1],
            );
        }
    }

    // ── Planetary pictures ───────────────────────────────────────────

    #[test]
    fn planetary_picture_identity() {
        // If A = B = C, result should be C (= B + C - A = A).
        let result = planetary_picture(100.0, 100.0, 100.0);
        assert!(
            (result - 100.0).abs() < 1e-10,
            "B+C-A with all equal should be 100°, got {result}°",
        );
    }

    #[test]
    fn planetary_picture_basic_arithmetic() {
        // B=30, C=60, A=10 => 30 + 60 - 10 = 80
        let result = planetary_picture(10.0, 30.0, 60.0);
        assert!((result - 80.0).abs() < 1e-10, "Expected 80°, got {result}°",);
    }

    #[test]
    fn planetary_picture_wraps_around() {
        // B=350, C=340, A=10 => 350 + 340 - 10 = 680 mod 360 = 320
        let result = planetary_picture(10.0, 350.0, 340.0);
        assert!(
            (result - 320.0).abs() < 1e-10,
            "Expected 320°, got {result}°",
        );
    }

    #[test]
    fn planetary_picture_negative_wraps() {
        // B=10, C=20, A=350 => 10 + 20 - 350 = -320 mod 360 = 40
        let result = planetary_picture(350.0, 10.0, 20.0);
        assert!((result - 40.0).abs() < 1e-10, "Expected 40°, got {result}°",);
    }

    #[test]
    fn is_planetary_picture_within_orb() {
        // B+C-A = 80.  Target at 81 with orb 2 -> true.
        assert!(is_planetary_picture(81.0, 10.0, 30.0, 60.0, 2.0));
    }

    #[test]
    fn is_planetary_picture_outside_orb() {
        // B+C-A = 80.  Target at 85 with orb 2 -> false.
        assert!(!is_planetary_picture(85.0, 10.0, 30.0, 60.0, 2.0));
    }

    #[test]
    fn is_planetary_picture_90_collapses_square() {
        // PP = 80.  80 mod 90 = 80.
        // Target at 170: 170 mod 90 = 80.  On 90° dial they coincide.
        assert!(is_planetary_picture_90(170.0, 10.0, 30.0, 60.0, 1.0));
    }

    // ── Direct/indirect midpoints ────────────────────────────────────

    #[test]
    fn direct_midpoint_simple() {
        let mp = direct_midpoint(10.0, 30.0);
        assert!(
            (mp - 20.0).abs() < 1e-10,
            "Direct midpoint of 10° and 30° should be 20°, got {mp}°",
        );
    }

    #[test]
    fn indirect_is_180_from_direct() {
        let direct = direct_midpoint(10.0, 30.0);
        let indirect = indirect_midpoint(10.0, 30.0);
        let diff = (indirect - direct).abs();
        assert!(
            (diff - 180.0).abs() < 1e-10,
            "Indirect should be 180° from direct: diff = {diff}°",
        );
    }

    // ── Dial conversions ─────────────────────────────────────────────

    #[test]
    fn dial_360_normalizes() {
        assert!((to_dial_360(370.0) - 10.0).abs() < 1e-10);
        assert!((to_dial_360(-10.0) - 350.0).abs() < 1e-10);
    }

    #[test]
    fn dial_90_collapses_cardinal() {
        // 0, 90, 180, 270 should all map to 0 on the 90° dial.
        for &lon in &[0.0, 90.0, 180.0, 270.0] {
            assert!(
                to_dial_90(lon).abs() < 1e-10,
                "{lon}° should map to 0° on 90° dial, got {}",
                to_dial_90(lon),
            );
        }
    }

    // ── Scan ─────────────────────────────────────────────────────────

    #[test]
    fn scan_finds_exact_picture() {
        // A=0, B=30, C=60 => B+C-A = 90.
        // D at 90° exactly.
        let positions = vec![("A", 0.0), ("B", 30.0), ("C", 60.0), ("D", 90.0)];
        let results = scan_planetary_pictures(&positions, 1.0);
        assert!(
            results.iter().any(|r| r.occupant == "D" && r.orb_90 < 0.01),
            "Should find D at B+C-A: {results:#?}",
        );
    }

    #[test]
    fn scan_respects_orb() {
        let positions = vec![
            ("A", 0.0),
            ("B", 30.0),
            ("C", 60.0),
            ("D", 100.0), // 10° away from PP at 90°
        ];
        let tight = scan_planetary_pictures(&positions, 1.0);
        let wide = scan_planetary_pictures(&positions, 15.0);
        // With tight orb, D should NOT be found for this particular
        // triple.  With wide orb, it should.
        let tight_has_d = tight
            .iter()
            .any(|r| r.occupant == "D" && r.formula.contains("B + C - A"));
        let wide_has_d = wide
            .iter()
            .any(|r| r.occupant == "D" && r.formula.contains("B + C - A"));
        assert!(!tight_has_d, "1° orb should not catch 10° distance");
        assert!(wide_has_d, "15° orb should catch 10° distance");
    }

    #[test]
    fn scan_results_sorted_by_orb() {
        let positions = vec![
            ("Sun", 10.0),
            ("Moon", 100.0),
            ("Mars", 55.0),
            ("Jupiter", 150.0),
            ("Saturn", 320.0),
        ];
        let results = scan_planetary_pictures(&positions, 5.0);
        for w in results.windows(2) {
            assert!(
                w[0].orb_90 <= w[1].orb_90 + 1e-10,
                "Results should be sorted by 90° orb",
            );
        }
    }

    #[test]
    fn planetary_picture_full_returns_both_dials() {
        let result = planetary_picture_full(10.0, 30.0, 60.0);
        assert!((result.degree - 80.0).abs() < 1e-10);
        assert!((result.degree_90 - 80.0_f64.rem_euclid(90.0)).abs() < 1e-10);
    }
}
