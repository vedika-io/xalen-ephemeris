use serde::{Deserialize, Serialize};

/// Sign names for sidereal zodiac (same names, shifted positions).
const SIGN_NAMES: [&str; 12] = [
    "Aries",
    "Taurus",
    "Gemini",
    "Cancer",
    "Leo",
    "Virgo",
    "Libra",
    "Scorpio",
    "Sagittarius",
    "Capricorn",
    "Aquarius",
    "Pisces",
];

// ── Sidereal position types ────────────────────────────────────────

/// A single body's position in both tropical and sidereal frames.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiderealPosition {
    pub body: &'static str,
    pub tropical_lon: f64,
    pub sidereal_lon: f64,
    pub sign: &'static str,
    pub sign_degree: f64,
}

/// A full sidereal chart with positions and the ayanamsa used.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiderealChart {
    pub positions: Vec<SiderealPosition>,
    pub ayanamsa: f64,
    pub ayanamsa_name: &'static str,
}

// ── Conversion functions ───────────────────────────────────────────

/// Convert a tropical longitude to sidereal by subtracting the ayanamsa.
///
/// Result is normalized to [0, 360).
#[inline]
pub fn tropical_to_sidereal(tropical_lon: f64, ayanamsa: f64) -> f64 {
    (tropical_lon - ayanamsa).rem_euclid(360.0)
}

/// Extract the sign index (0=Aries..11=Pisces) from a sidereal longitude.
#[inline]
pub fn sidereal_sign_index(sidereal_lon: f64) -> usize {
    ((sidereal_lon.rem_euclid(360.0)) / 30.0) as usize
}

/// Extract the degree within the sign from a sidereal longitude.
#[inline]
pub fn sidereal_sign_degree(sidereal_lon: f64) -> f64 {
    sidereal_lon.rem_euclid(360.0) % 30.0
}

/// Sign name for a sidereal longitude.
pub fn sidereal_sign_name(sidereal_lon: f64) -> &'static str {
    SIGN_NAMES[sidereal_sign_index(sidereal_lon)]
}

// ── Ayanamsa functions ─────────────────────────────────────────────

/// Fagan-Bradley ayanamsa for a given Julian Day.
///
/// The Fagan-Bradley ayanamsa is the most widely used in Western sidereal
/// astrology. Cyril Fagan determined the Synetic Vernal Point (SVP) through
/// analysis of historical events and mundane astrology.
///
/// The Swiss Ephemeris SE_SIDM_FAGAN_BRADLEY value at J2000.0 (JD 2451545.0)
/// is 24.7405° (24°44'25.8"). NOTE: the often-quoted 24.042° is the *B1950.0*
/// value (= 360° − SVP 335°57'28.6"); precessing it forward ~50 tropical years
/// at the general precession rate (~50.29"/yr) adds +0.698° to reach the J2000
/// value. Anchoring directly at J2000 avoids that very confusion.
///
/// Reference: Cyril Fagan, "Zodiacs Old and New" (1951); Fagan & Bradley, AFA;
/// Swiss Ephemeris sweph.h ayanamsa[0]. Verified 2026-05-28.
pub fn fagan_bradley_ayanamsa(jd: f64) -> f64 {
    let years_from_j2000 = (jd - 2451545.0) / 365.25;
    24.7405 + years_from_j2000 * 50.29 / 3600.0
}

/// Cyril Fagan's original Synetic Vernal Point (SVP) ayanamsa.
///
/// Fagan's original SVP was defined by the Sun's position at the vernal
/// equinox of 221 AD, which he identified as the epoch when the tropical
/// and sidereal zodiacs were aligned in ancient Babylonian practice.
///
/// The SVP ayanamsa is very close to the Fagan-Bradley value but uses
/// a slightly different base, ~0.022° (~1.3') below Fagan-Bradley:
/// 24.7185° at J2000.0 (vs Fagan-Bradley 24.7405°).
///
/// Reference: Cyril Fagan, "Astrological Origins" (1971).
pub fn fagan_svp_ayanamsa(jd: f64) -> f64 {
    let years_from_j2000 = (jd - 2451545.0) / 365.25;
    24.7185 + years_from_j2000 * 50.29 / 3600.0
}

/// Lahiri ayanamsa (for comparison with Vedic systems).
///
/// Although primarily a Vedic (Hindu) ayanamsa, it is included here for
/// completeness — some Western sidereal astrologers compare Fagan-Bradley
/// with Lahiri when cross-referencing charts.
///
/// Base value: 23.85306° at J2000.0 (SE_SIDM_LAHIRI, 23°51'11"), precessing at
/// the mean Lahiri rate 50.27"/yr. Official Indian government ayanamsa
/// (Chitrapaksha). Verified 2026-05-28 against Swiss Ephemeris.
pub fn lahiri_ayanamsa(jd: f64) -> f64 {
    let years_from_j2000 = (jd - 2451545.0) / 365.25;
    23.85306 + years_from_j2000 * 50.27 / 3600.0
}

// ── Chart conversion ───────────────────────────────────────────────

/// Convert a set of tropical planetary positions into a sidereal Western chart.
///
/// * `tropical_positions` — slice of (body_name, tropical_longitude) pairs.
/// * `jd` — Julian Day for which to compute the ayanamsa.
/// * `ayanamsa_fn` — function returning the ayanamsa for a given JD.
///
/// The body names should be &'static str matching standard planet names.
pub fn sidereal_western_chart(
    tropical_positions: &[(&'static str, f64)],
    jd: f64,
    ayanamsa_fn: fn(f64) -> f64,
) -> SiderealChart {
    let ayanamsa = ayanamsa_fn(jd);

    // Determine ayanamsa name by comparing function pointer
    let ayanamsa_name = identify_ayanamsa_name(ayanamsa_fn);

    let positions = tropical_positions
        .iter()
        .map(|&(body, tropical_lon)| {
            let sidereal_lon = tropical_to_sidereal(tropical_lon, ayanamsa);
            let sign_idx = sidereal_sign_index(sidereal_lon);
            SiderealPosition {
                body,
                tropical_lon,
                sidereal_lon,
                sign: SIGN_NAMES[sign_idx],
                sign_degree: sidereal_sign_degree(sidereal_lon),
            }
        })
        .collect();

    SiderealChart {
        positions,
        ayanamsa,
        ayanamsa_name,
    }
}

/// Identify the ayanamsa by its computed value at J2000.0.
/// Function pointer comparison is unreliable in release mode (inlining).
fn identify_ayanamsa_name(f: fn(f64) -> f64) -> &'static str {
    let val = f(2451545.0);
    let fb = fagan_bradley_ayanamsa(2451545.0);
    let svp = fagan_svp_ayanamsa(2451545.0);
    let lah = lahiri_ayanamsa(2451545.0);
    if (val - fb).abs() < 1e-10 {
        "Fagan-Bradley"
    } else if (val - svp).abs() < 1e-10 {
        "Fagan SVP"
    } else if (val - lah).abs() < 1e-10 {
        "Lahiri"
    } else {
        "Custom"
    }
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// J2000.0 epoch for test convenience.
    const J2000: f64 = 2451545.0;

    #[test]
    fn tropical_to_sidereal_basic() {
        // At J2000, Fagan-Bradley ayanamsa = 24.042
        // Tropical 30 deg -> sidereal = 30 - 24.042 = 5.958
        let sid = tropical_to_sidereal(30.0, 24.042);
        assert!((sid - 5.958).abs() < 0.001);
    }

    #[test]
    fn tropical_to_sidereal_wrap_around() {
        // Tropical 10 deg with ayanamsa 24 deg -> sidereal = 10 - 24 = -14 -> 346
        let sid = tropical_to_sidereal(10.0, 24.0);
        assert!((sid - 346.0).abs() < 0.001);
    }

    #[test]
    fn sidereal_sign_extraction() {
        // 0 deg sidereal = Aries
        assert_eq!(sidereal_sign_index(0.0), 0);
        assert_eq!(sidereal_sign_name(0.0), "Aries");

        // 45 deg sidereal = Taurus (30-60)
        assert_eq!(sidereal_sign_index(45.0), 1);
        assert_eq!(sidereal_sign_name(45.0), "Taurus");
        assert!((sidereal_sign_degree(45.0) - 15.0).abs() < 0.001);

        // 359 deg = Pisces
        assert_eq!(sidereal_sign_index(359.0), 11);
        assert_eq!(sidereal_sign_name(359.0), "Pisces");
    }

    #[test]
    fn fagan_bradley_at_j2000() {
        let aya = fagan_bradley_ayanamsa(J2000);
        assert!(
            (aya - 24.7405).abs() < 0.001,
            "Fagan-Bradley at J2000 should be ~24.7405 (SE_SIDM_FAGAN_BRADLEY), got {}",
            aya
        );
    }

    #[test]
    fn fagan_bradley_increases_over_time() {
        // Ayanamsa increases due to precession
        let aya_2000 = fagan_bradley_ayanamsa(J2000);
        let aya_2100 = fagan_bradley_ayanamsa(J2000 + 100.0 * 365.25);
        assert!(
            aya_2100 > aya_2000,
            "Ayanamsa should increase: 2000={}, 2100={}",
            aya_2000,
            aya_2100
        );
        // 100 years * 50.29"/yr / 3600 = ~1.397 degrees
        let diff = aya_2100 - aya_2000;
        assert!(
            (diff - 1.397).abs() < 0.01,
            "100-year difference should be ~1.397 deg, got {}",
            diff
        );
    }

    #[test]
    fn fagan_svp_close_to_fagan_bradley() {
        let fb = fagan_bradley_ayanamsa(J2000);
        let svp = fagan_svp_ayanamsa(J2000);
        let diff = (fb - svp).abs();
        assert!(
            diff < 0.1,
            "Fagan SVP and Fagan-Bradley should differ by < 0.1 deg, got {}",
            diff
        );
        assert!(
            (svp - 24.7185).abs() < 0.001,
            "Fagan SVP at J2000 should be ~24.7185 (~0.022 below Fagan-Bradley), got {}",
            svp
        );
    }

    #[test]
    fn lahiri_differs_from_fagan_bradley() {
        let fb = fagan_bradley_ayanamsa(J2000);
        let la = lahiri_ayanamsa(J2000);
        // Lahiri is typically ~0.2 deg less than Fagan-Bradley
        assert!(fb > la, "Fagan-Bradley should be greater than Lahiri");
        assert!(
            (la - 23.85306).abs() < 0.001,
            "Lahiri at J2000 should be ~23.85306 (SE_SIDM_LAHIRI), got {}",
            la
        );
    }

    #[test]
    fn sidereal_chart_construction() {
        let positions: Vec<(&'static str, f64)> = vec![
            ("Sun", 45.0),      // ~15 Taurus tropical
            ("Moon", 120.0),    // ~0 Leo tropical
            ("Mars", 270.0),    // ~0 Capricorn tropical
            ("Jupiter", 350.0), // ~20 Pisces tropical
        ];

        let chart = sidereal_western_chart(&positions, J2000, fagan_bradley_ayanamsa);

        assert_eq!(chart.positions.len(), 4);
        assert_eq!(chart.ayanamsa_name, "Fagan-Bradley");
        assert!((chart.ayanamsa - 24.7405).abs() < 0.001);

        // Sun at 45 - 24.7405 = 20.2595 sidereal = Aries (0-30)
        let sun = &chart.positions[0];
        assert_eq!(sun.body, "Sun");
        assert_eq!(sun.sign, "Aries");
        assert!((sun.sidereal_lon - 20.2595).abs() < 0.01);
        assert!((sun.sign_degree - 20.2595).abs() < 0.01);

        // Moon at 120 - 24.7405 = 95.2595 sidereal = Cancer (90-120)
        let moon = &chart.positions[1];
        assert_eq!(moon.body, "Moon");
        assert_eq!(moon.sign, "Cancer");
    }

    #[test]
    fn sidereal_chart_with_svp() {
        let positions: Vec<(&'static str, f64)> = vec![("Sun", 100.0)];
        let chart = sidereal_western_chart(&positions, J2000, fagan_svp_ayanamsa);
        assert_eq!(chart.ayanamsa_name, "Fagan SVP");
        // Sun at 100 - 24.02 = 75.98 = Gemini
        assert_eq!(chart.positions[0].sign, "Gemini");
    }

    #[test]
    fn sidereal_chart_with_lahiri() {
        let positions: Vec<(&'static str, f64)> = vec![("Venus", 200.0)];
        let chart = sidereal_western_chart(&positions, J2000, lahiri_ayanamsa);
        assert_eq!(chart.ayanamsa_name, "Lahiri");
        // Venus at 200 - 23.856 = 176.144 = Virgo (150-180)
        assert_eq!(chart.positions[0].sign, "Virgo");
    }

    #[test]
    fn sidereal_chart_wrap_around_pisces_aries() {
        // Planet at tropical 10 deg (Aries), ayanamsa ~24.74 deg
        // Sidereal = 10 - 24.7405 = -14.7405 -> 345.2595 = Pisces
        let positions: Vec<(&'static str, f64)> = vec![("Mercury", 10.0)];
        let chart = sidereal_western_chart(&positions, J2000, fagan_bradley_ayanamsa);
        assert_eq!(chart.positions[0].sign, "Pisces");
        assert!((chart.positions[0].sidereal_lon - 345.2595).abs() < 0.01);
    }

    #[test]
    fn identify_ayanamsa_names() {
        assert_eq!(
            identify_ayanamsa_name(fagan_bradley_ayanamsa),
            "Fagan-Bradley"
        );
        assert_eq!(identify_ayanamsa_name(fagan_svp_ayanamsa), "Fagan SVP");
        assert_eq!(identify_ayanamsa_name(lahiri_ayanamsa), "Lahiri");
    }
}
