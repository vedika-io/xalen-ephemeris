//! Swiss Ephemeris compatibility layer.
//!
//! Drop-in replacements for the most common `sweph` C functions.  Swap
//! `use sweph` for `use xalen_ephem::compat` and existing code compiles
//! with minimal changes.
//!
//! # Mapping
//!
//! | sweph C function | compat function |
//! |---|---|
//! | `swe_calc_ut` | [`swe_calc_ut`] |
//! | `swe_houses` | [`swe_houses`] |
//! | `swe_get_ayanamsa_ut` | [`swe_get_ayanamsa_ut`] |
//! | `swe_get_ayanamsa_ut` (with sid mode) | [`swe_get_ayanamsa_ut_ex`] |
//! | `swe_julday` | [`swe_julday`] |
//! | `swe_revjul` | [`swe_revjul`] |
//! | `swe_deltat_ex` | [`swe_deltat`] |
//! | `swe_fixstar_ut` | [`swe_fixstar_ut`] |
//! | `swe_set_ephe_path` | no-op (embedded data) |
//! | `swe_close` | no-op |
//!
//! # Example
//!
//! ```rust
//! use xalen_ephem::compat::*;
//!
//! let xx = swe_calc_ut(2451545.0, SE_SUN, SEFLG_SWIEPH).unwrap();
//! assert!(xx[0] >= 0.0 && xx[0] < 360.0);
//!
//! let cusps = swe_houses(2451545.0, 18.52, 73.85, 'P').unwrap();
//! assert_eq!(cusps.cusps.len(), 12);
//! ```

use crate::{Almanac, Body};
use xalen_time::{JdUT1, JulianDay};

// ── SE planet constants ─────────────────────────────────────────────────────

/// Sun (SE_SUN = 0)
pub const SE_SUN: i32 = 0;
/// Moon (SE_MOON = 1)
pub const SE_MOON: i32 = 1;
/// Mercury (SE_MERCURY = 2)
pub const SE_MERCURY: i32 = 2;
/// Venus (SE_VENUS = 3)
pub const SE_VENUS: i32 = 3;
/// Mars (SE_MARS = 4)
pub const SE_MARS: i32 = 4;
/// Jupiter (SE_JUPITER = 5)
pub const SE_JUPITER: i32 = 5;
/// Saturn (SE_SATURN = 6)
pub const SE_SATURN: i32 = 6;
/// Uranus (SE_URANUS = 7)
pub const SE_URANUS: i32 = 7;
/// Neptune (SE_NEPTUNE = 8)
pub const SE_NEPTUNE: i32 = 8;
/// Pluto (SE_PLUTO = 9)
pub const SE_PLUTO: i32 = 9;
/// Mean Node / Rahu (SE_MEAN_NODE = 10)
pub const SE_MEAN_NODE: i32 = 10;
/// True Node / Rahu osculating (SE_TRUE_NODE = 11)
pub const SE_TRUE_NODE: i32 = 11;
/// Mean Apogee / Lilith (SE_MEAN_APOG = 12)
pub const SE_MEAN_APOG: i32 = 12;
/// Chiron (SE_CHIRON = 15)
pub const SE_CHIRON: i32 = 15;
/// Earth (SE_EARTH = 14) -- geocentric: returns 0,0,0
pub const SE_EARTH: i32 = 14;

// ── SE flag constants (accepted but most are no-ops) ────────────────────────

/// Use Swiss Ephemeris data (default; always active in XALEN).
pub const SEFLG_SWIEPH: i32 = 2;
/// Request speed computation (accepted, speed fields will be 0.0).
pub const SEFLG_SPEED: i32 = 256;
/// Sidereal mode (use with [`swe_get_ayanamsa_ut_ex`] instead).
pub const SEFLG_SIDEREAL: i32 = 64 * 1024;

// ── SE ayanamsa constants (SE_SIDM_*) ──────────────────────────────────────

/// Fagan-Bradley (SE_SIDM_FAGAN_BRADLEY = 0)
pub const SE_SIDM_FAGAN_BRADLEY: i32 = 0;
/// Lahiri (SE_SIDM_LAHIRI = 1)
pub const SE_SIDM_LAHIRI: i32 = 1;
/// De Luce (SE_SIDM_DELUCE = 2)
pub const SE_SIDM_DELUCE: i32 = 2;
/// Raman (SE_SIDM_RAMAN = 3)
pub const SE_SIDM_RAMAN: i32 = 3;
/// Usha/Shashi (SE_SIDM_USHASHASHI = 4)
pub const SE_SIDM_USHASHASHI: i32 = 4;
/// Krishnamurti (SE_SIDM_KRISHNAMURTI = 5)
pub const SE_SIDM_KRISHNAMURTI: i32 = 5;
/// Djwhal Khul (SE_SIDM_DJWHAL_KHUL = 6)
pub const SE_SIDM_DJWHAL_KHUL: i32 = 6;
/// Yukteswar (SE_SIDM_YUKTESWAR = 7)
pub const SE_SIDM_YUKTESWAR: i32 = 7;
/// J.N. Bhasin (SE_SIDM_JN_BHASIN = 8)
pub const SE_SIDM_JN_BHASIN: i32 = 8;
/// True Chitra (SE_SIDM_TRUE_CITRA = 27)
pub const SE_SIDM_TRUE_CITRA: i32 = 27;
/// True Revati (SE_SIDM_TRUE_REVATI = 28)
pub const SE_SIDM_TRUE_REVATI: i32 = 28;

// ── SE house system codes ───────────────────────────────────────────────────

// These are just chars, but documented here for reference.
// 'P' Placidus, 'K' Koch, 'O' Porphyry, 'R' Regiomontanus,
// 'C' Campanus, 'A'/'E' Equal, 'W' Whole Sign, 'M' Morinus,
// 'B' Alcabitius, 'T' Topocentric, 'X' Meridian, 'V' Vehlow,
// 'U' Krusinski-Pisa, 'S' Sripati, 'G' Gauquelin

// ── SE calendar type ────────────────────────────────────────────────────────

/// Gregorian calendar (SE_GREG_CAL = 1)
pub const SE_GREG_CAL: i32 = 1;
/// Julian calendar (SE_JUL_CAL = 0)
pub const SE_JUL_CAL: i32 = 0;

// ── Result types ────────────────────────────────────────────────────────────

/// House cusp results matching the SE `swe_houses` output layout.
#[derive(Debug, Clone)]
pub struct HouseCusps {
    /// 12 house cusps in degrees [0, 360).
    pub cusps: Vec<f64>,
    /// Ascendant in degrees (ascmc[0]).
    pub ascendant: f64,
    /// MC (Midheaven) in degrees (ascmc[1]).
    pub mc: f64,
    /// ARMC (Sidereal time in degrees) (ascmc[2]).
    pub armc: f64,
    /// Vertex in degrees (ascmc[3]).
    pub vertex: f64,
}

// ── Planet mapping ──────────────────────────────────────────────────────────

/// Convert an SE planet number to our [`Body`] enum.
fn se_planet_to_body(planet: i32) -> Result<Body, String> {
    match planet {
        SE_SUN => Ok(Body::Sun),
        SE_MOON => Ok(Body::Moon),
        SE_MERCURY => Ok(Body::Mercury),
        SE_VENUS => Ok(Body::Venus),
        SE_MARS => Ok(Body::Mars),
        SE_JUPITER => Ok(Body::Jupiter),
        SE_SATURN => Ok(Body::Saturn),
        SE_URANUS => Ok(Body::Uranus),
        SE_NEPTUNE => Ok(Body::Neptune),
        SE_PLUTO => Ok(Body::Pluto),
        SE_MEAN_NODE => Ok(Body::MeanNode),
        SE_TRUE_NODE => Ok(Body::TrueNode),
        SE_MEAN_APOG => Ok(Body::MeanApogee),
        SE_EARTH => Ok(Body::Earth),
        SE_CHIRON => Ok(Body::Chiron),
        _ => Err(format!("unsupported SE planet number: {planet}")),
    }
}

/// Convert an SE house system character to our [`HouseSystem`](xalen_houses::HouseSystem).
fn se_hsys_to_system(hsys: char) -> Result<xalen_houses::HouseSystem, String> {
    use xalen_houses::HouseSystem;
    match hsys {
        'P' => Ok(HouseSystem::Placidus),
        'K' => Ok(HouseSystem::Koch),
        'O' => Ok(HouseSystem::Porphyry),
        'R' => Ok(HouseSystem::Regiomontanus),
        'C' => Ok(HouseSystem::Campanus),
        'A' | 'E' => Ok(HouseSystem::Equal),
        'W' => Ok(HouseSystem::WholeSign),
        'M' => Ok(HouseSystem::Morinus),
        'B' => Ok(HouseSystem::Alcabitius),
        'T' => Ok(HouseSystem::Topocentric),
        'X' => Ok(HouseSystem::Meridian),
        'V' => Ok(HouseSystem::Vehlow),
        'U' => Ok(HouseSystem::KrusinskiPisa),
        'S' => Ok(HouseSystem::Sripati),
        'G' => Ok(HouseSystem::Gauquelin),
        'i' => Ok(HouseSystem::SunshineMakransky),
        'I' => Ok(HouseSystem::SunshineTreindl),
        'L' => Ok(HouseSystem::PullenSinusoidalDelta),
        'Q' => Ok(HouseSystem::PullenSinusoidalRatio),
        'F' => Ok(HouseSystem::CarterPoliEquatorial),
        'Y' => Ok(HouseSystem::APC),
        'Z' => Ok(HouseSystem::Zariel),
        'b' => Ok(HouseSystem::AlcabitiusClassic),
        _ => Err(format!("unsupported SE house system char: '{hsys}'")),
    }
}

/// Convert an SE ayanamsa ID (SE_SIDM_*) to our [`Ayanamsa`](xalen_ayanamsa::Ayanamsa).
fn se_sidm_to_ayanamsa(sidm: i32) -> Result<xalen_ayanamsa::Ayanamsa, String> {
    xalen_ayanamsa::Ayanamsa::from_swiss_ephem_id(sidm as u32)
        .ok_or_else(|| format!("unsupported SE ayanamsa ID: {sidm}"))
}

// ── Public compat functions ─────────────────────────────────────────────────

/// Compute geocentric ecliptic position of a planet at Julian Day UT1.
///
/// Drop-in replacement for `swe_calc_ut(jd, planet, iflag, xx, serr)`.
///
/// Returns `[longitude, latitude, distance, lon_speed, lat_speed, dist_speed]`
/// in degrees (longitude/latitude) and AU (distance). Speed fields are
/// currently set to 0.0.
///
/// The `iflag` parameter is accepted for signature compatibility but has no
/// effect -- XALEN always uses its built-in VSOP87 data.
///
/// # Examples
///
/// ```
/// use xalen_ephem::compat::*;
/// let xx = swe_calc_ut(2451545.0, SE_SUN, SEFLG_SWIEPH).unwrap();
/// assert!(xx[0] >= 0.0 && xx[0] < 360.0);
/// ```
pub fn swe_calc_ut(jd: f64, planet: i32, _iflag: i32) -> Result<[f64; 6], String> {
    let body = se_planet_to_body(planet)?;
    let almanac = Almanac::default_vedic();
    let pos = almanac
        .geocentric_ecliptic(body, JdUT1(jd))
        .map_err(|e| e.to_string())?;

    Ok([
        pos.longitude.to_degrees().rem_euclid(360.0),
        pos.latitude.to_degrees(),
        pos.distance,
        0.0, // lon speed (not yet computed)
        0.0, // lat speed
        0.0, // distance speed
    ])
}

/// Compute house cusps and chart angles.
///
/// Drop-in replacement for `swe_houses(jd, lat, lon, hsys, cusps, ascmc)`.
///
/// `hsys` is the single-character house system code from Swiss Ephemeris:
/// `'P'` Placidus, `'K'` Koch, `'O'` Porphyry, `'R'` Regiomontanus,
/// `'C'` Campanus, `'A'` Equal, `'W'` Whole Sign, etc.
///
/// Returns a [`HouseCusps`] struct with 12 cusps and angles, all in degrees.
///
/// # Examples
///
/// ```
/// use xalen_ephem::compat::*;
/// let h = swe_houses(2451545.0, 18.52, 73.85, 'P').unwrap();
/// assert_eq!(h.cusps.len(), 12);
/// assert!(h.ascendant >= 0.0 && h.ascendant < 360.0);
/// ```
pub fn swe_houses(jd: f64, lat: f64, lon: f64, hsys: char) -> Result<HouseCusps, String> {
    let system = se_hsys_to_system(hsys)?;
    let loc = xalen_houses::GeoLocation::new(lat, lon);
    // Mean obliquity at epoch (Lieske 1979, matches SE default).
    let t = (jd - 2_451_545.0) / 36525.0;
    let epsilon = (23.439291 - 0.013004 * t).to_radians();
    let h = xalen_houses::compute_houses(jd, &loc, epsilon, system);

    let cusps: Vec<f64> = h
        .cusps
        .iter()
        .map(|c| c.to_degrees().rem_euclid(360.0))
        .collect();

    Ok(HouseCusps {
        cusps,
        ascendant: h.ascendant.to_degrees().rem_euclid(360.0),
        mc: h.mc.to_degrees().rem_euclid(360.0),
        armc: 0.0, // RAMC not yet exposed
        vertex: h.vertex.to_degrees().rem_euclid(360.0),
    })
}

/// Compute the Lahiri ayanamsa in degrees at the given Julian Day UT1.
///
/// Drop-in replacement for `swe_get_ayanamsa_ut(jd)`.
///
/// This uses the default Lahiri (Chitrapaksha) system, matching the behavior
/// of Swiss Ephemeris when `swe_set_sid_mode(SE_SIDM_LAHIRI, 0, 0)` is active.
///
/// For other ayanamsa systems, use [`swe_get_ayanamsa_ut_ex`].
pub fn swe_get_ayanamsa_ut(jd: f64) -> f64 {
    let jd_tt = JdUT1(jd)
        .to_tt(&xalen_time::DeltaTModel::StephensonMorrisonHohenkerk2016)
        .as_f64();
    xalen_ayanamsa::Ayanamsa::Lahiri.compute_deg(jd_tt)
}

/// Compute any ayanamsa in degrees at the given Julian Day UT1.
///
/// `sidm` is the SE_SIDM_* constant (0--46).
///
/// # Examples
///
/// ```
/// use xalen_ephem::compat::*;
/// let aya = swe_get_ayanamsa_ut_ex(2451545.0, SE_SIDM_KRISHNAMURTI).unwrap();
/// assert!(aya > 20.0 && aya < 30.0);
/// ```
pub fn swe_get_ayanamsa_ut_ex(jd: f64, sidm: i32) -> Result<f64, String> {
    let aya = se_sidm_to_ayanamsa(sidm)?;
    let jd_tt = JdUT1(jd)
        .to_tt(&xalen_time::DeltaTModel::StephensonMorrisonHohenkerk2016)
        .as_f64();
    Ok(aya.compute_deg(jd_tt))
}

/// Convert a calendar date to Julian Day.
///
/// Drop-in replacement for `swe_julday(year, month, day, hour, gregflag)`.
///
/// `gregflag`: `SE_GREG_CAL` (1) for Gregorian, `SE_JUL_CAL` (0) for Julian.
pub fn swe_julday(year: i32, month: i32, day: i32, hour: f64, gregflag: i32) -> f64 {
    let cal = if gregflag == SE_JUL_CAL {
        xalen_time::CalendarSystem::ProlepticJulian
    } else {
        xalen_time::CalendarSystem::ProlepticGregorian
    };
    xalen_time::calendar_to_jd(year, month as u32, day as u32, hour, cal).as_f64()
}

/// Convert a Julian Day to calendar date.
///
/// Drop-in replacement for `swe_revjul(jd, gregflag, &year, &month, &day, &hour)`.
///
/// Returns `(year, month, day, hour)`.
pub fn swe_revjul(jd: f64, gregflag: i32) -> (i32, i32, i32, f64) {
    let cal = if gregflag == SE_JUL_CAL {
        xalen_time::CalendarSystem::ProlepticJulian
    } else {
        xalen_time::CalendarSystem::ProlepticGregorian
    };
    let (year, month, day, hour) = xalen_time::jd_to_calendar(jd, cal);
    (year, month as i32, day as i32, hour)
}

/// Compute delta-T (TT - UT1) in days at the given Julian Day.
///
/// Drop-in replacement for `swe_deltat_ex(jd, iflag, serr)`.
///
/// Note: Swiss Ephemeris returns delta-T in **days**. This function matches
/// that convention. Multiply by 86400 to get seconds.
pub fn swe_deltat(jd: f64) -> f64 {
    let dt_seconds = xalen_time::delta_t(
        jd,
        &xalen_time::DeltaTModel::StephensonMorrisonHohenkerk2016,
    );
    dt_seconds / 86400.0 // convert seconds to days, matching SE convention
}

/// Look up a fixed star by name and return its ecliptic position at JD.
///
/// Simplified replacement for `swe_fixstar_ut(star, jd, iflag, xx, serr)`.
///
/// Returns `[longitude, latitude, magnitude, 0, 0, 0]` in degrees.
/// If the star is not found, returns an error.
pub fn swe_fixstar_ut(star_name: &str, jd: f64) -> Result<[f64; 6], String> {
    let star = xalen_stars::find_by_name(star_name)
        .ok_or_else(|| format!("star not found: {star_name}"))?;
    let year = 2000.0 + (jd - 2_451_545.0) / 365.25;
    Ok([
        star.longitude_at_epoch(year),
        star.latitude_at_epoch(year),
        star.magnitude,
        0.0,
        0.0,
        0.0,
    ])
}

/// No-op. XALEN embeds all data at compile time.
///
/// Provided for source compatibility with `swe_set_ephe_path(path)`.
pub fn swe_set_ephe_path(_path: &str) {
    // No-op: XALEN has no external data files.
}

/// No-op. XALEN has no resources to release.
///
/// Provided for source compatibility with `swe_close()`.
pub fn swe_close() {
    // No-op: nothing to clean up.
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const J2000: f64 = 2_451_545.0;

    #[test]
    fn calc_ut_sun_at_j2000() {
        let xx = swe_calc_ut(J2000, SE_SUN, SEFLG_SWIEPH).unwrap();
        // Sun at J2000 should be around 280 degrees (Capricorn)
        assert!(xx[0] > 270.0 && xx[0] < 290.0, "Sun lon={}", xx[0]);
        assert!(xx[1].abs() < 1.0, "Sun lat should be near 0: {}", xx[1]);
        assert!(xx[2] > 0.9 && xx[2] < 1.1, "Sun dist ~1 AU: {}", xx[2]);
    }

    #[test]
    fn calc_ut_moon_at_j2000() {
        let xx = swe_calc_ut(J2000, SE_MOON, SEFLG_SWIEPH).unwrap();
        assert!(xx[0] >= 0.0 && xx[0] < 360.0, "Moon lon={}", xx[0]);
    }

    #[test]
    fn calc_ut_all_se_planets() {
        let planets = [
            SE_SUN,
            SE_MOON,
            SE_MERCURY,
            SE_VENUS,
            SE_MARS,
            SE_JUPITER,
            SE_SATURN,
            SE_URANUS,
            SE_NEPTUNE,
            SE_PLUTO,
            SE_MEAN_NODE,
            SE_TRUE_NODE,
            SE_CHIRON,
        ];
        for &p in &planets {
            let xx = swe_calc_ut(J2000, p, SEFLG_SWIEPH);
            assert!(xx.is_ok(), "SE planet {} failed: {:?}", p, xx.err());
            let arr = xx.unwrap();
            assert!(
                arr[0] >= 0.0 && arr[0] < 360.0,
                "Planet {} lon={}",
                p,
                arr[0]
            );
        }
    }

    #[test]
    fn calc_ut_invalid_planet() {
        assert!(swe_calc_ut(J2000, 99, SEFLG_SWIEPH).is_err());
    }

    #[test]
    fn houses_placidus() {
        let h = swe_houses(J2000, 18.52, 73.85, 'P').unwrap();
        assert_eq!(h.cusps.len(), 12);
        assert!(h.ascendant >= 0.0 && h.ascendant < 360.0);
        assert!(h.mc >= 0.0 && h.mc < 360.0);
        // Cusps should be in [0, 360)
        for (i, c) in h.cusps.iter().enumerate() {
            assert!(
                *c >= 0.0 && *c < 360.0,
                "Cusp {} out of range: {}",
                i + 1,
                c
            );
        }
    }

    #[test]
    fn houses_all_systems() {
        let systems = [
            'P', 'K', 'O', 'R', 'C', 'A', 'W', 'M', 'B', 'T', 'X', 'V', 'U', 'S',
        ];
        for &sys in &systems {
            let h = swe_houses(J2000, 40.0, -74.0, sys);
            assert!(h.is_ok(), "House system '{}' failed: {:?}", sys, h.err());
        }
    }

    #[test]
    fn houses_invalid_system() {
        assert!(swe_houses(J2000, 0.0, 0.0, '?').is_err());
    }

    #[test]
    fn ayanamsa_lahiri_at_j2000() {
        let aya = swe_get_ayanamsa_ut(J2000);
        assert!(aya > 23.0 && aya < 25.0, "Lahiri aya={}", aya);
    }

    #[test]
    fn ayanamsa_ex_all_47_systems() {
        for id in 0..=46_i32 {
            let result = swe_get_ayanamsa_ut_ex(J2000, id);
            assert!(result.is_ok(), "SE ayanamsa ID {} failed", id);
            let aya = result.unwrap();
            assert!(aya.is_finite(), "SE aya ID {} non-finite: {}", id, aya);
        }
    }

    #[test]
    fn ayanamsa_ex_invalid() {
        assert!(swe_get_ayanamsa_ut_ex(J2000, 99).is_err());
    }

    #[test]
    fn julday_j2000() {
        let jd = swe_julday(2000, 1, 1, 12.0, SE_GREG_CAL);
        assert!((jd - 2_451_545.0).abs() < 0.01, "J2000 jd={}", jd);
    }

    #[test]
    fn revjul_roundtrip() {
        let jd = swe_julday(1990, 6, 15, 10.5, SE_GREG_CAL);
        let (y, m, d, h) = swe_revjul(jd, SE_GREG_CAL);
        assert_eq!(y, 1990);
        assert_eq!(m, 6);
        assert_eq!(d, 15);
        assert!((h - 10.5).abs() < 0.001, "hour={}", h);
    }

    #[test]
    fn deltat_positive_at_j2000() {
        let dt = swe_deltat(J2000);
        assert!(dt > 0.0, "delta-T should be positive at J2000: {}", dt);
        // Delta-T at J2000 is ~63.8 seconds = ~0.000738 days
        assert!(dt < 0.01, "delta-T should be small in days: {}", dt);
    }

    #[test]
    fn fixstar_spica() {
        let xx = swe_fixstar_ut("Spica", J2000).unwrap();
        // Spica at J2000 is ~203.8 degrees ecliptic longitude
        assert!(xx[0] > 200.0 && xx[0] < 210.0, "Spica lon={}", xx[0]);
        assert!(xx[2] < 2.0, "Spica magnitude={}", xx[2]); // mag ~0.98
    }

    #[test]
    fn fixstar_not_found() {
        assert!(swe_fixstar_ut("NonexistentStar", J2000).is_err());
    }

    #[test]
    fn set_ephe_path_is_noop() {
        swe_set_ephe_path("/nonexistent/path"); // should not panic
    }

    #[test]
    fn close_is_noop() {
        swe_close(); // should not panic
    }

    #[test]
    fn se_planet_mapping_complete() {
        // Verify the key SE planet constants map correctly
        assert_eq!(se_planet_to_body(SE_SUN).unwrap(), Body::Sun);
        assert_eq!(se_planet_to_body(SE_MOON).unwrap(), Body::Moon);
        assert_eq!(se_planet_to_body(SE_MERCURY).unwrap(), Body::Mercury);
        assert_eq!(se_planet_to_body(SE_VENUS).unwrap(), Body::Venus);
        assert_eq!(se_planet_to_body(SE_MARS).unwrap(), Body::Mars);
        assert_eq!(se_planet_to_body(SE_JUPITER).unwrap(), Body::Jupiter);
        assert_eq!(se_planet_to_body(SE_SATURN).unwrap(), Body::Saturn);
        assert_eq!(se_planet_to_body(SE_URANUS).unwrap(), Body::Uranus);
        assert_eq!(se_planet_to_body(SE_NEPTUNE).unwrap(), Body::Neptune);
        assert_eq!(se_planet_to_body(SE_PLUTO).unwrap(), Body::Pluto);
        assert_eq!(se_planet_to_body(SE_MEAN_NODE).unwrap(), Body::MeanNode);
        assert_eq!(se_planet_to_body(SE_TRUE_NODE).unwrap(), Body::TrueNode);
        assert_eq!(se_planet_to_body(SE_MEAN_APOG).unwrap(), Body::MeanApogee);
        assert_eq!(se_planet_to_body(SE_CHIRON).unwrap(), Body::Chiron);
    }

    #[test]
    fn se_house_system_mapping_complete() {
        let codes = [
            'P', 'K', 'O', 'R', 'C', 'A', 'E', 'W', 'M', 'B', 'T', 'X', 'V', 'U', 'S', 'G', 'i',
            'I', 'L', 'Q', 'F', 'Y', 'Z', 'b',
        ];
        for &c in &codes {
            assert!(
                se_hsys_to_system(c).is_ok(),
                "House system char '{}' should be mapped",
                c
            );
        }
    }
}
