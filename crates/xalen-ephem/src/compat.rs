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
use std::cell::Cell;
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
/// Osculating (True) Apogee / Lilith (SE_OSCU_APOG = 13)
pub const SE_OSCU_APOG: i32 = 13;
/// Chiron (SE_CHIRON = 15)
pub const SE_CHIRON: i32 = 15;
/// Earth (SE_EARTH = 14) -- geocentric: returns 0,0,0
pub const SE_EARTH: i32 = 14;

// ── SE flag constants (accepted but most are no-ops) ────────────────────────

/// Use Swiss Ephemeris data (default; always active in XALEN).
pub const SEFLG_SWIEPH: i32 = 2;
/// Request speed computation. When set, `swe_calc_ut` fills the daily-motion
/// speed fields (`xx[3..6]`) via `Almanac::geocentric_speed`; when unset, those
/// fields are `0.0` (matching Swiss Ephemeris behaviour).
pub const SEFLG_SPEED: i32 = 256;
/// Sidereal mode. When set on [`swe_calc_ut`], the active sidereal-mode
/// ayanamsa (set via [`swe_set_sid_mode`], default Lahiri) is subtracted so the
/// returned longitude is sidereal — matching Swiss `swe_calc_ut(..., SEFLG_SIDEREAL)`.
pub const SEFLG_SIDEREAL: i32 = 64 * 1024;

// ── Unsupported-but-meaningful flags (rejected, never silently dropped) ─────
//
// Swiss Ephemeris assigns these flags specific meanings that materially change
// the result. XALEN's compat layer does not (yet) implement them, so rather
// than silently returning geocentric apparent-of-date positions — which would
// be wrong by degrees — `swe_calc_ut` HARD-ERRORS when any is set. A loud error
// is strictly safer than a silent-wrong drop-in.

/// Heliocentric position (SEFLG_HELCTR = 8).
pub const SEFLG_HELCTR: i32 = 8;
/// J2000 (mean equinox of 2000) frame instead of equinox-of-date (SEFLG_J2000 = 32).
pub const SEFLG_J2000: i32 = 32;
/// Equatorial coordinates (RA/Dec) instead of ecliptic (SEFLG_EQUATORIAL = 2048).
pub const SEFLG_EQUATORIAL: i32 = 2 * 1024;
/// Barycentric position (SEFLG_BARYCTR = 16384).
pub const SEFLG_BARYCTR: i32 = 16 * 1024;
/// Topocentric (observer-centered) position (SEFLG_TOPOCTR = 32768).
pub const SEFLG_TOPOCTR: i32 = 32 * 1024;
/// Cartesian x/y/z output instead of polar (SEFLG_XYZ = 4096).
pub const SEFLG_XYZ: i32 = 4 * 1024;
/// Output in radians instead of degrees (SEFLG_RADIANS = 8192).
pub const SEFLG_RADIANS: i32 = 8 * 1024;

/// Bit-mask of every flag [`swe_calc_ut`] cannot honour. Setting any of these
/// produces an error rather than a silently-wrong position.
const SEFLG_UNSUPPORTED_MASK: i32 = SEFLG_HELCTR
    | SEFLG_J2000
    | SEFLG_EQUATORIAL
    | SEFLG_BARYCTR
    | SEFLG_TOPOCTR
    | SEFLG_XYZ
    | SEFLG_RADIANS;

thread_local! {
    /// The active sidereal mode (SE_SIDM_* id), mirroring Swiss's process-global
    /// `swe_set_sid_mode`. Thread-local (not a global static) so concurrent
    /// callers on different threads cannot race on it. Defaults to Lahiri (1).
    static ACTIVE_SID_MODE: Cell<i32> = const { Cell::new(SE_SIDM_LAHIRI) };
}

/// Set the active sidereal mode used by [`swe_calc_ut`] when [`SEFLG_SIDEREAL`]
/// is requested. Drop-in shim for `swe_set_sid_mode(sidm, t0, ayan_t0)`.
///
/// Only `sidm` (an `SE_SIDM_*` constant) is honoured; the custom-ayanamsa
/// reference epoch/offset arguments (`t0`, `ayan_t0`) are accepted for source
/// compatibility but ignored, because XALEN's named ayanamsa models already
/// carry their own canonical reference epochs. The setting is per-thread.
///
/// # Examples
///
/// ```
/// use xalen_ephem::compat::*;
/// swe_set_sid_mode(SE_SIDM_KRISHNAMURTI, 0.0, 0.0);
/// let xx = swe_calc_ut(2451545.0, SE_SUN, SEFLG_SWIEPH | SEFLG_SIDEREAL).unwrap();
/// // Reset to the default so later calls aren't affected.
/// swe_set_sid_mode(SE_SIDM_LAHIRI, 0.0, 0.0);
/// assert!(xx[0] >= 0.0 && xx[0] < 360.0);
/// ```
pub fn swe_set_sid_mode(sidm: i32, _t0: f64, _ayan_t0: f64) {
    ACTIVE_SID_MODE.with(|m| m.set(sidm));
}

/// The currently active sidereal mode (SE_SIDM_* id) for this thread.
///
/// Public so language bindings (e.g. the `xalen.swe` Python submodule) can
/// implement `swe_get_ayanamsa_ut`-style calls that resolve "the ayanamsa of the
/// active sidereal mode" without re-implementing the thread-local state.
pub fn active_sid_mode() -> i32 {
    ACTIVE_SID_MODE.with(|m| m.get())
}

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
    /// Equatorial ascendant / East Point in degrees (ascmc[4], `SE_EQUASC`).
    pub equatorial_ascendant: f64,
    /// Co-ascendant after W. Koch in degrees (ascmc[5], `SE_COASC1`).
    pub co_ascendant_koch: f64,
    /// Co-ascendant after M. Munkasey in degrees (ascmc[6], `SE_COASC2`).
    pub co_ascendant_munkasey: f64,
    /// Polar ascendant after M. Munkasey in degrees (ascmc[7], `SE_POLASC`).
    pub polar_ascendant_munkasey: f64,
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
        SE_OSCU_APOG => Ok(Body::OsculatingApogee),
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
        // 'Z' (Zariel/axial-rotation) and 'b' (classic Alcabitius) are NOT real
        // Swiss Ephemeris `hsys` codes — Swiss has no distinct letter for either
        // (see `xalen_houses::HouseSystem::swiss_ephem_code`, which returns
        // `None` for both). Advertising them as distinct Swiss codes here was a
        // lie, so they now fall through to the error arm exactly like any other
        // unrecognized letter. Drive `HouseSystem::Zariel` / `AlcabitiusClassic`
        // through the typed `xalen_houses` API directly if you need them.
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
/// in degrees (longitude/latitude per day) and AU (distance, AU/day).
///
/// `iflag` honors the basic Swiss Ephemeris flags:
/// * [`SEFLG_SPEED`] — when set, the daily-motion speeds are computed and
///   returned in `xx[3..6]` (matching Swiss, which leaves them undefined when
///   the flag is absent); when unset they are `0.0`.
/// * [`SEFLG_SIDEREAL`] — when set, the active sidereal-mode ayanamsa (set via
///   [`swe_set_sid_mode`], default Lahiri) is subtracted so the returned
///   longitude (and longitude speed) are sidereal, matching Swiss exactly.
/// * [`SEFLG_SWIEPH`] is the implied default (XALEN always uses its embedded
///   VSOP87/DE440 data).
///
/// Flags that Swiss assigns a *position-altering* meaning XALEN does not yet
/// implement ([`SEFLG_HELCTR`], [`SEFLG_TOPOCTR`], [`SEFLG_J2000`],
/// [`SEFLG_EQUATORIAL`], [`SEFLG_BARYCTR`], [`SEFLG_XYZ`], [`SEFLG_RADIANS`])
/// cause an **error** rather than a silently-wrong geocentric/ecliptic result.
///
/// # Examples
///
/// ```
/// use xalen_ephem::compat::*;
/// let xx = swe_calc_ut(2451545.0, SE_SUN, SEFLG_SWIEPH | SEFLG_SPEED).unwrap();
/// assert!(xx[0] >= 0.0 && xx[0] < 360.0);
/// // The Sun advances ~1°/day in ecliptic longitude.
/// assert!(xx[3] > 0.9 && xx[3] < 1.1);
///
/// // Sidereal mode subtracts the active ayanamsa (Lahiri by default).
/// let trop = swe_calc_ut(2451545.0, SE_SUN, SEFLG_SWIEPH).unwrap();
/// let sid = swe_calc_ut(2451545.0, SE_SUN, SEFLG_SWIEPH | SEFLG_SIDEREAL).unwrap();
/// assert!(trop[0] - sid[0] > 23.0 && trop[0] - sid[0] < 25.0); // ~Lahiri ayanamsa
/// ```
pub fn swe_calc_ut(jd: f64, planet: i32, iflag: i32) -> Result<[f64; 6], String> {
    // Reject flags that would silently change the frame/center/coordinate type.
    // A drop-in that quietly ignores HELCTR/TOPOCTR/J2000/EQUATORIAL and returns
    // geocentric apparent-of-date ecliptic degrees is worse than one that errors.
    if iflag & SEFLG_UNSUPPORTED_MASK != 0 {
        return Err(format!(
            "swe_calc_ut: unsupported flag(s) set (iflag={iflag:#x}); \
             HELCTR/TOPOCTR/J2000/EQUATORIAL/BARYCTR/XYZ/RADIANS are not implemented \
             and must not be silently ignored"
        ));
    }

    let body = se_planet_to_body(planet)?;
    let almanac = Almanac::default_vedic();
    let pos = almanac
        .geocentric_ecliptic(body, JdUT1(jd))
        .map_err(|e| e.to_string())?;

    // Sidereal mode: subtract the active ayanamsa from the tropical longitude.
    // Ayanamsa models are evaluated in TT, so convert UT1→TT for the lookup,
    // exactly as `swe_get_ayanamsa_ut` does.
    let sidereal = iflag & SEFLG_SIDEREAL != 0;
    let ayanamsa_deg = if sidereal {
        let aya = se_sidm_to_ayanamsa(active_sid_mode())?;
        let jd_tt = JdUT1(jd)
            .to_tt(&xalen_time::DeltaTModel::StephensonMorrisonHohenkerk2016)
            .as_f64();
        aya.compute_deg(jd_tt)
    } else {
        0.0
    };

    let longitude = (pos.longitude.to_degrees() - ayanamsa_deg).rem_euclid(360.0);

    // Swiss Ephemeris only fills the speed fields when SEFLG_SPEED is set; the
    // engine computes apparent daily motion via finite difference, so wire it
    // through rather than returning zeros. In sidereal mode Swiss also subtracts
    // the ayanamsa's own rate (~0.145″/day) from the longitude speed; mirror that
    // via a one-day finite difference of the ayanamsa so the sidereal speed
    // matches Swiss to sub-arcsecond/day.
    let (lon_speed, lat_speed, dist_speed) = if iflag & SEFLG_SPEED != 0 {
        let speed = almanac
            .geocentric_speed(body, JdUT1(jd))
            .map_err(|e| e.to_string())?;
        let lon_speed = if sidereal {
            let aya = se_sidm_to_ayanamsa(active_sid_mode())?;
            let model = xalen_time::DeltaTModel::StephensonMorrisonHohenkerk2016;
            let jd_tt0 = JdUT1(jd - 0.5).to_tt(&model).as_f64();
            let jd_tt1 = JdUT1(jd + 0.5).to_tt(&model).as_f64();
            let ayanamsa_rate = aya.compute_deg(jd_tt1) - aya.compute_deg(jd_tt0); // deg/day
            speed.longitude.to_degrees() - ayanamsa_rate
        } else {
            speed.longitude.to_degrees()
        };
        (lon_speed, speed.latitude.to_degrees(), speed.distance)
    } else {
        (0.0, 0.0, 0.0)
    };

    Ok([
        longitude,
        pos.latitude.to_degrees(),
        pos.distance,
        lon_speed,
        lat_speed,
        dist_speed,
    ])
}

/// Compute house cusps and chart angles (tropical).
///
/// Drop-in replacement for `swe_houses(jd, lat, lon, hsys, cusps, ascmc)`.
///
/// `hsys` is the single-character house system code from Swiss Ephemeris:
/// `'P'` Placidus, `'K'` Koch, `'O'` Porphyry, `'R'` Regiomontanus,
/// `'C'` Campanus, `'A'` Equal, `'W'` Whole Sign, etc.
///
/// Returns a [`HouseCusps`] struct with 12 cusps and angles, all in degrees,
/// including the four auxiliary ascendants Swiss exposes in `ascmc[4..8]`.
///
/// For a sidereal house frame (`swe_houses_ex(..., SEFLG_SIDEREAL)`), use
/// [`swe_houses_ex`].
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
    swe_houses_ex(jd, lat, lon, hsys, false)
}

/// Compute house cusps and chart angles, optionally in the sidereal frame.
///
/// Drop-in replacement for `swe_houses_ex(jd, iflag, lat, lon, hsys, cusps, ascmc)`
/// for the `SEFLG_SIDEREAL` case: when `sidereal` is `true`, the active
/// sidereal-mode ayanamsa (set via [`swe_set_sid_mode`], default Lahiri) is
/// subtracted from every cusp and every angle — matching Swiss, which only
/// offsets the resulting longitudes by the ayanamsa and does not re-tilt the
/// ecliptic.
///
/// Returns a [`HouseCusps`] with 12 cusps and angles, all in degrees, including
/// the four auxiliary ascendants Swiss exposes in `ascmc[4..8]`.
///
/// # Examples
///
/// ```
/// use xalen_ephem::compat::*;
/// let trop = swe_houses_ex(2451545.0, 18.52, 73.85, 'P', false).unwrap();
/// let sid = swe_houses_ex(2451545.0, 18.52, 73.85, 'P', true).unwrap();
/// // Sidereal Ascendant is tropical − ayanamsa (Lahiri ~23.85° at J2000).
/// let offset = (trop.ascendant - sid.ascendant).rem_euclid(360.0);
/// assert!(offset > 23.0 && offset < 25.0);
/// ```
pub fn swe_houses_ex(
    jd: f64,
    lat: f64,
    lon: f64,
    hsys: char,
    sidereal: bool,
) -> Result<HouseCusps, String> {
    let system = se_hsys_to_system(hsys)?;
    // Validate geographic coordinates before doing any trigonometry. Swiss
    // Ephemeris silently produces garbage for out-of-range input; reject it.
    if !lat.is_finite() || !lon.is_finite() {
        return Err(format!("non-finite coordinates: lat={lat}, lon={lon}"));
    }
    if !(-90.0..=90.0).contains(&lat) {
        return Err(format!("latitude out of range [-90, 90]: {lat}"));
    }
    if !(-180.0..=360.0).contains(&lon) {
        return Err(format!("longitude out of range [-180, 360]: {lon}"));
    }
    let loc = xalen_houses::GeoLocation::new(lat, lon);

    // Swiss `swe_houses` works on the TRUE equinox of date: it uses the TRUE
    // obliquity (mean + nutation in obliquity) and APPARENT sidereal time (GAST),
    // so MC/ASC carry nutation in longitude. Match that exactly here — the old
    // code used a mean-obliquity polynomial and GMST, which biased ARMC by the
    // equation of the equinoxes (~12.8″ at J2000) and ASC by ~1″, verified
    // against pyswisseph.
    //
    // Obliquity/nutation are functions of TT; convert UT1→TT for the argument
    // (ΔT ≈ 69 s today — tiny here, but correct).
    let jd_tt = JdUT1(jd)
        .to_tt(&xalen_time::DeltaTModel::StephensonMorrisonHohenkerk2016)
        .as_f64();
    let t_tt = (jd_tt - 2_451_545.0) / 36525.0;
    let nut = xalen_coords::nutation_2000b(t_tt);
    let epsilon = xalen_coords::mean_obliquity(t_tt) + nut.delta_epsilon; // true obliquity of date

    // RAMC from apparent sidereal time at the meridian (GAST + observer longitude).
    // Swiss returns this same value in ascmc[2]. The ARMC is a sidereal-time
    // angle (right ascension) and is NOT shifted by the ayanamsa even in the
    // sidereal frame — Swiss reports the same ARMC for both.
    let armc_deg = (xalen_coords::gast_deg(jd, t_tt) + lon).rem_euclid(360.0);
    let ramc = armc_deg.to_radians();
    let mut h = xalen_houses::compute_houses_from_ramc(ramc, &loc, epsilon, system);

    // Sidereal frame: subtract the active-mode ayanamsa from every cusp and
    // angle (Swiss leaves the obliquity/ARMC untouched and only offsets the
    // ecliptic longitudes). Evaluate the ayanamsa in TT, as Swiss does.
    if sidereal {
        let aya = se_sidm_to_ayanamsa(active_sid_mode())?;
        let ayanamsa_rad = aya.compute_deg(jd_tt).to_radians();
        h = h.to_sidereal(ayanamsa_rad);
    }

    let cusps: Vec<f64> = h
        .cusps
        .iter()
        .map(|c| c.to_degrees().rem_euclid(360.0))
        .collect();

    Ok(HouseCusps {
        cusps,
        ascendant: h.ascendant.to_degrees().rem_euclid(360.0),
        mc: h.mc.to_degrees().rem_euclid(360.0),
        armc: armc_deg,
        vertex: h.vertex.to_degrees().rem_euclid(360.0),
        equatorial_ascendant: h.equatorial_ascendant.to_degrees().rem_euclid(360.0),
        co_ascendant_koch: h.co_ascendant_koch.to_degrees().rem_euclid(360.0),
        co_ascendant_munkasey: h.co_ascendant_munkasey.to_degrees().rem_euclid(360.0),
        polar_ascendant_munkasey: h.polar_ascendant_munkasey.to_degrees().rem_euclid(360.0),
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
/// Drop-in replacement for `swe_fixstar_ut(star, jd, iflag, xx, serr)`.
///
/// Returns the Swiss Ephemeris `xx` layout
/// `[longitude, latitude, distance, lon_speed, lat_speed, dist_speed]` —
/// longitude/latitude in degrees, distance in AU. **Magnitude is NOT placed in
/// `xx`**: Swiss returns it via a separate `swe_fixstar_mag` call, so use
/// [`swe_fixstar_mag`] for the visual magnitude. (The earlier compat layer put
/// the magnitude in `xx[2]`, the distance slot — that broke drop-in parity for
/// callers reading distance.)
///
/// The catalog carries no parallax, so distance is reported as `0.0`.
/// If the star is not found, returns an error.
pub fn swe_fixstar_ut(star_name: &str, jd: f64) -> Result<[f64; 6], String> {
    #[cfg(not(feature = "hip-catalog"))]
    {
        let _ = (star_name, jd);
        return Err(
            "swe_fixstar_ut requires the `hip-catalog` feature (the Hipparcos \
             fixed-star catalog is non-commercial and is not linked in this build)"
                .to_string(),
        );
    }
    #[cfg(feature = "hip-catalog")]
    {
        let star = xalen_stars::find_by_name(star_name)
            .ok_or_else(|| format!("star not found: {star_name}"))?;
        let year = 2000.0 + (jd - 2_451_545.0) / 365.25;
        Ok([
            star.longitude_at_epoch(year),
            star.latitude_at_epoch(year),
            0.0, // distance (AU): no parallax data in the catalog
            0.0, // lon speed
            0.0, // lat speed
            0.0, // dist speed
        ])
    }
}

/// Return the visual magnitude of a fixed star.
///
/// Drop-in replacement for `swe_fixstar_mag(star, &mag, serr)` — Swiss
/// Ephemeris reports magnitude through this separate call, not via the `xx`
/// position array.
///
/// # Examples
///
/// ```
/// use xalen_ephem::compat::*;
/// let mag = swe_fixstar_mag("Spica").unwrap();
/// assert!((mag - 0.98).abs() < 0.5);
/// ```
pub fn swe_fixstar_mag(star_name: &str) -> Result<f64, String> {
    #[cfg(not(feature = "hip-catalog"))]
    {
        let _ = star_name;
        return Err(
            "swe_fixstar_mag requires the `hip-catalog` feature (the Hipparcos \
             fixed-star catalog is non-commercial and is not linked in this build)"
                .to_string(),
        );
    }
    #[cfg(feature = "hip-catalog")]
    {
        let star = xalen_stars::find_by_name(star_name)
            .ok_or_else(|| format!("star not found: {star_name}"))?;
        Ok(star.magnitude)
    }
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
    fn calc_ut_osculating_apogee_matches_pyswisseph() {
        // SE_OSCU_APOG (13) = True (osculating) Black Moon Lilith. pyswisseph
        // 2.10.03 reports 252.979401° at J2000; the osculating apogee is
        // intrinsically model-sensitive, so 0.5° is a meaningful bound.
        let xx = swe_calc_ut(J2000, SE_OSCU_APOG, SEFLG_SWIEPH).unwrap();
        let mut diff = (xx[0] - 252.979401).abs() % 360.0;
        diff = diff.min(360.0 - diff);
        assert!(
            diff < 0.5,
            "SE_OSCU_APOG at J2000 ~252.98°, got {}° (diff {diff}°)",
            xx[0]
        );
        // It must differ from the mean apogee (SE_MEAN_APOG) — that is the point.
        let mean = swe_calc_ut(J2000, SE_MEAN_APOG, SEFLG_SWIEPH).unwrap();
        assert!(
            (xx[0] - mean[0]).abs() > 1.0,
            "osculating {} should differ from mean {} apogee",
            xx[0],
            mean[0]
        );
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
    fn houses_armc_is_real() {
        // ARMC must be the APPARENT sidereal time at the meridian (Swiss uses
        // GAST, not GMST), not a placeholder 0.
        let h = swe_houses(J2000, 18.52, 73.85, 'P').unwrap();
        assert!(
            h.armc > 0.0 && h.armc < 360.0,
            "armc should be a real angle in (0,360): {}",
            h.armc
        );
        // Independent re-derivation with apparent sidereal time must match.
        let jd_tt = JdUT1(J2000)
            .to_tt(&xalen_time::DeltaTModel::StephensonMorrisonHohenkerk2016)
            .as_f64();
        let t_tt = (jd_tt - 2_451_545.0) / 36525.0;
        let expected = (xalen_coords::gast_deg(J2000, t_tt) + 73.85).rem_euclid(360.0);
        assert!(
            (h.armc - expected).abs() < 1e-9,
            "armc {} != expected {}",
            h.armc,
            expected
        );
    }

    #[test]
    fn houses_armc_uses_gast_not_gmst() {
        // Regression guard for the obliquity/sidereal-time fix: the ARMC must be
        // referred to the APPARENT (GAST) equinox, so it differs from a naive
        // GMST-based ARMC by the equation of the equinoxes (~12.8″ ≈ 0.0035° at
        // J2000), and matches the GAST value to machine precision.
        let h = swe_houses(J2000, 18.52, 73.85, 'P').unwrap();
        let gmst_armc = (xalen_houses::gmst(J2000) * 15.0 + 73.85).rem_euclid(360.0);
        let delta = (h.armc - gmst_armc).abs();
        assert!(
            delta > 0.002 && delta < 0.006,
            "ARMC should differ from GMST-ARMC by ~equation of equinoxes (0.0035°), got {delta}°"
        );
    }

    #[test]
    fn houses_rejects_bad_coordinates() {
        assert!(swe_houses(J2000, 91.0, 0.0, 'P').is_err(), "lat > 90");
        assert!(swe_houses(J2000, -91.0, 0.0, 'P').is_err(), "lat < -90");
        assert!(swe_houses(J2000, 0.0, 400.0, 'P').is_err(), "lon > 360");
        assert!(swe_houses(J2000, f64::NAN, 0.0, 'P').is_err(), "lat NaN");
    }

    #[test]
    fn calc_ut_speed_flag_populates_motion() {
        // Without SEFLG_SPEED the speed fields are zero (matches Swiss).
        let no_speed = swe_calc_ut(J2000, SE_SUN, SEFLG_SWIEPH).unwrap();
        assert_eq!((no_speed[3], no_speed[4], no_speed[5]), (0.0, 0.0, 0.0));

        // With SEFLG_SPEED the Sun's daily motion (~1°/day) must be filled in.
        let with_speed = swe_calc_ut(J2000, SE_SUN, SEFLG_SWIEPH | SEFLG_SPEED).unwrap();
        assert!(
            with_speed[3] > 0.9 && with_speed[3] < 1.1,
            "Sun lon speed ~1°/day, got {}",
            with_speed[3]
        );

        // The Moon moves ~13°/day; sanity-check a fast body too.
        let moon = swe_calc_ut(J2000, SE_MOON, SEFLG_SWIEPH | SEFLG_SPEED).unwrap();
        assert!(
            moon[3] > 11.0 && moon[3] < 15.0,
            "Moon lon speed ~13°/day, got {}",
            moon[3]
        );
    }

    #[test]
    fn calc_ut_sidereal_subtracts_ayanamsa() {
        // SEFLG_SIDEREAL must return tropical − ayanamsa (Lahiri by default),
        // not silently fall back to the tropical longitude.
        swe_set_sid_mode(SE_SIDM_LAHIRI, 0.0, 0.0); // defensive: isolate from sibling tests
        let trop = swe_calc_ut(J2000, SE_SUN, SEFLG_SWIEPH).unwrap();
        let sid = swe_calc_ut(J2000, SE_SUN, SEFLG_SWIEPH | SEFLG_SIDEREAL).unwrap();
        let aya = swe_get_ayanamsa_ut(J2000);
        let expected = (trop[0] - aya).rem_euclid(360.0);
        assert!(
            (sid[0] - expected).abs() < 1e-9,
            "sidereal {} != tropical−ayanamsa {} (aya={})",
            sid[0],
            expected,
            aya
        );
        // And the offset is the Lahiri ayanamsa (~23.85° at J2000), proving the
        // flag is no longer ignored.
        let offset = (trop[0] - sid[0]).rem_euclid(360.0);
        assert!(offset > 23.0 && offset < 25.0, "ayanamsa offset = {offset}");
    }

    #[test]
    fn calc_ut_sidereal_honors_active_sid_mode() {
        // swe_set_sid_mode must change which ayanamsa SEFLG_SIDEREAL subtracts.
        swe_set_sid_mode(SE_SIDM_KRISHNAMURTI, 0.0, 0.0);
        let sid_kp = swe_calc_ut(J2000, SE_SUN, SEFLG_SWIEPH | SEFLG_SIDEREAL).unwrap();
        let trop = swe_calc_ut(J2000, SE_SUN, SEFLG_SWIEPH).unwrap();
        let kp_aya = swe_get_ayanamsa_ut_ex(J2000, SE_SIDM_KRISHNAMURTI).unwrap();
        let expected = (trop[0] - kp_aya).rem_euclid(360.0);
        assert!(
            (sid_kp[0] - expected).abs() < 1e-9,
            "KP-mode sidereal {} != tropical−KP-ayanamsa {}",
            sid_kp[0],
            expected
        );
        // Restore default so other tests on this thread are unaffected.
        swe_set_sid_mode(SE_SIDM_LAHIRI, 0.0, 0.0);
        let sid_lahiri = swe_calc_ut(J2000, SE_SUN, SEFLG_SWIEPH | SEFLG_SIDEREAL).unwrap();
        assert!(
            (sid_kp[0] - sid_lahiri[0]).abs() > 1e-4,
            "KP and Lahiri sidereal longitudes must differ"
        );
    }

    #[test]
    fn calc_ut_sidereal_speed_subtracts_ayanamsa_rate() {
        // In sidereal mode the longitude speed is reduced by the ayanamsa's own
        // precession rate (~0.145″/day); it must stay positive and very close to
        // the tropical speed.
        swe_set_sid_mode(SE_SIDM_LAHIRI, 0.0, 0.0);
        let trop = swe_calc_ut(J2000, SE_SUN, SEFLG_SWIEPH | SEFLG_SPEED).unwrap();
        let sid = swe_calc_ut(J2000, SE_SUN, SEFLG_SWIEPH | SEFLG_SIDEREAL | SEFLG_SPEED).unwrap();
        let delta_arcsec = (trop[3] - sid[3]) * 3600.0;
        assert!(
            delta_arcsec > 0.05 && delta_arcsec < 0.30,
            "sidereal speed should be ~0.145″/day slower than tropical, got {delta_arcsec}″/day"
        );
        assert!(
            sid[3] > 0.9 && sid[3] < 1.1,
            "sidereal Sun speed ~1°/day: {}",
            sid[3]
        );
    }

    #[test]
    fn calc_ut_rejects_unsupported_flags() {
        // HELCTR/TOPOCTR/J2000/EQUATORIAL/BARYCTR/XYZ/RADIANS must ERROR, never
        // be silently dropped (that would return a geocentric ecliptic position
        // mislabeled as something else).
        for &flag in &[
            SEFLG_HELCTR,
            SEFLG_TOPOCTR,
            SEFLG_J2000,
            SEFLG_EQUATORIAL,
            SEFLG_BARYCTR,
            SEFLG_XYZ,
            SEFLG_RADIANS,
        ] {
            let r = swe_calc_ut(J2000, SE_SUN, SEFLG_SWIEPH | flag);
            assert!(r.is_err(), "flag {flag:#x} should be rejected, got {r:?}");
        }
        // The supported flags together still succeed.
        assert!(swe_calc_ut(J2000, SE_SUN, SEFLG_SWIEPH | SEFLG_SPEED | SEFLG_SIDEREAL).is_ok());
    }

    #[test]
    fn houses_match_true_obliquity_apparent_sidereal() {
        // After the fix, ASC/MC for Pune match Swiss-grade values closely. We
        // assert the angles are finite and in-range; precise oracle comparison
        // lives in the cross-validation suite. This guards against the angles
        // collapsing or wrapping incorrectly when fed a GAST-derived RAMC.
        let h = swe_houses(J2000, 18.52, 73.85, 'P').unwrap();
        assert!(h.ascendant >= 0.0 && h.ascendant < 360.0);
        assert!(h.mc >= 0.0 && h.mc < 360.0);
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
        // xx[2] is distance (Swiss layout), not magnitude.
        assert_eq!(xx[2], 0.0, "no parallax data -> distance 0");
        // Magnitude comes from the dedicated call, matching Swiss.
        let mag = swe_fixstar_mag("Spica").unwrap();
        assert!(mag < 2.0, "Spica magnitude={mag}"); // mag ~0.98
    }

    #[test]
    fn fixstar_not_found() {
        assert!(swe_fixstar_ut("NonexistentStar", J2000).is_err());
        assert!(swe_fixstar_mag("NonexistentStar").is_err());
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
        // Only the letters that are GENUINE Swiss Ephemeris `hsys` codes map.
        // 'Z' and 'b' are deliberately excluded — see `house_codes_z_and_b_refused`.
        let codes = [
            'P', 'K', 'O', 'R', 'C', 'A', 'E', 'W', 'M', 'B', 'T', 'X', 'V', 'U', 'S', 'G', 'i',
            'I', 'L', 'Q', 'F', 'Y',
        ];
        for &c in &codes {
            assert!(
                se_hsys_to_system(c).is_ok(),
                "House system char '{}' should be mapped",
                c
            );
        }
    }

    #[test]
    fn house_codes_z_and_b_refused() {
        // Swiss Ephemeris has no distinct `hsys` letter for Zariel ('Z') or
        // classic Alcabitius ('b') — `swiss_ephem_code()` returns None for both.
        // The compat layer must NOT advertise a fabricated distinct code; both
        // are rejected like any other unrecognized letter, matching systems.rs.
        assert!(
            swe_houses(J2000, 18.52, 73.85, 'Z').is_err(),
            "'Z' is not a Swiss code"
        );
        assert!(
            swe_houses(J2000, 18.52, 73.85, 'b').is_err(),
            "'b' is not a Swiss code"
        );
        // 'B' (real Alcabitius) and 'X' (Meridian/axial-rotation) still work —
        // those are the genuine Swiss codes Zariel and classic-Alcabitius alias.
        assert!(swe_houses(J2000, 18.52, 73.85, 'B').is_ok());
        assert!(swe_houses(J2000, 18.52, 73.85, 'X').is_ok());
    }

    #[test]
    fn houses_populates_auxiliary_ascmc() {
        // ascmc[4..8] — equatorial ascendant, the two co-ascendants, and the
        // polar ascendant — must be real angles in [0, 360), not placeholder 0s.
        let h = swe_houses(J2000, 18.52, 73.85, 'P').unwrap();
        for (name, v) in [
            ("equatorial_ascendant", h.equatorial_ascendant),
            ("co_ascendant_koch", h.co_ascendant_koch),
            ("co_ascendant_munkasey", h.co_ascendant_munkasey),
            ("polar_ascendant_munkasey", h.polar_ascendant_munkasey),
        ] {
            assert!(
                (0.0..360.0).contains(&v),
                "ascmc auxiliary '{name}' out of range: {v}"
            );
        }
        // The Koch co-ascendant and the Munkasey polar ascendant are exactly
        // 180° apart (Swiss property), proving the values are computed, not 0.
        let diff = (h.co_ascendant_koch - h.polar_ascendant_munkasey).rem_euclid(360.0);
        assert!(
            (diff - 180.0).abs() < 1e-6,
            "Koch co-asc and polar-asc should be 180° apart, got {diff}"
        );
    }

    #[test]
    fn houses_ex_sidereal_subtracts_ayanamsa() {
        // swe_houses_ex(..., sidereal=true) must shift every cusp and angle by
        // −ayanamsa (Lahiri by default), exactly like swe_houses_ex(SEFLG_SIDEREAL).
        swe_set_sid_mode(SE_SIDM_LAHIRI, 0.0, 0.0);
        let trop = swe_houses_ex(J2000, 18.52, 73.85, 'P', false).unwrap();
        let sid = swe_houses_ex(J2000, 18.52, 73.85, 'P', true).unwrap();
        let aya = swe_get_ayanamsa_ut(J2000);

        let ascendant_offset = (trop.ascendant - sid.ascendant).rem_euclid(360.0);
        assert!(
            (ascendant_offset - aya).abs() < 1e-6,
            "sidereal ASC offset {ascendant_offset} != ayanamsa {aya}"
        );
        // Every cusp shifts by the same ayanamsa.
        for i in 0..12 {
            let off = (trop.cusps[i] - sid.cusps[i]).rem_euclid(360.0);
            assert!(
                (off - aya).abs() < 1e-6,
                "cusp {} sidereal offset {off} != ayanamsa {aya}",
                i + 1
            );
        }
        // The ARMC is a sidereal-time (RA) angle and is NOT shifted by ayanamsa.
        assert!(
            (trop.armc - sid.armc).abs() < 1e-9,
            "ARMC must be identical in tropical and sidereal frames"
        );
    }
}
