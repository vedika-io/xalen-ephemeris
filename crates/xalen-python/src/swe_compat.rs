//! `xalen.swe` — a `pyswisseph` drop-in compatibility submodule.
//!
//! The goal is that migrating an existing `pyswisseph` codebase to XALEN is a
//! search-and-replace:
//!
//! ```python
//! # before
//! import swisseph as swe
//! # after
//! import xalen.swe as swe
//! ```
//!
//! Every function here mirrors the **shape** of the corresponding `pyswisseph`
//! entry point (argument order, tuple layout, the `SE_*` / `SEFLG_*` constants)
//! and delegates the actual astronomy to the audited `xalen_ephem::compat`
//! layer, which itself is Swiss-cross-validated.
//!
//! ## Honesty notes (where XALEN and Swiss differ)
//! * Flags that Swiss assigns a *position-altering* meaning XALEN does not yet
//!   implement (`SEFLG_HELCTR`, `SEFLG_TOPOCTR`, `SEFLG_J2000`,
//!   `SEFLG_EQUATORIAL`, `SEFLG_BARYCTR`, `SEFLG_XYZ`, `SEFLG_RADIANS`) raise a
//!   `ValueError` rather than silently returning a geocentric ecliptic position
//!   mislabeled as something else. A loud error beats a silent-wrong drop-in.
//! * `calc_ut` returns `((lon, lat, dist, lon_speed, lat_speed, dist_speed),
//!   ret_flag)` exactly like `pyswisseph`. The speed slots are `0.0` unless
//!   `SEFLG_SPEED` is set (matching Swiss). `calc` takes a TT/ET epoch (like
//!   `pyswisseph`) and converts it to UT1 via ΔT before computing.
//! * `houses_ex` returns `(cusps_tuple_of_12, ascmc_tuple_of_8)`; all eight
//!   `ascmc` slots are populated — `[0..4]` are `asc, mc, armc, vertex` and
//!   `[4..8]` are the equatorial ascendant, the two co-ascendants (Koch,
//!   Munkasey) and the polar ascendant. `SEFLG_SIDEREAL` in `flags` returns the
//!   sidereal frame (active-mode ayanamsa subtracted from cusps and angles).
//! * Rise/set/transit (`rise_trans`, `rise_trans_true_hor`) and eclipse search
//!   (`sol_eclipse_when_glob`, `sol_eclipse_when_loc`, `lun_eclipse_when`,
//!   `lun_eclipse_when_loc`, `lun_eclipse_how`) return pyswisseph's tuple
//!   shapes and return codes; the astronomy is the `xalen_ephem` rise/set and
//!   eclipse engines. Slots those engines do not compute are `0.0` (listed in
//!   each docstring and in the README).
//!
//! ## Exceptions
//! `xalen.swe.Error` (a `RuntimeError`, like pyswisseph's `swe.Error`) is raised
//! for every calculation failure. `TypeError` and `ValueError` are raised only
//! for bad arguments: a wrong Python type, a non-finite date, an out-of-range
//! place, or a flag whose meaning XALEN does not implement.

use pyo3::create_exception;
use pyo3::exceptions::{PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyString, PyTuple};

use xalen_ephem::Almanac;
use xalen_ephem::Body;
use xalen_ephem::compat;
use xalen_ephem::eclipse::{self, LunarEclipse, LunarEclipseType, SolarEclipse, SolarEclipseType};
use xalen_ephem::local_eclipse::{self, LocalSolarType};
use xalen_ephem::rise_set::{self, DiscPoint, HorizonCrossing, Twilight};
use xalen_time::{DeltaTModel, JdUT1, JulianDay};

create_exception!(
    xalen.swe,
    Error,
    PyRuntimeError,
    "Calculation failure in ``xalen.swe`` (pyswisseph's ``swe.Error``).\n\n\
     A ``RuntimeError`` subclass, raised for every failed calculation: an\n\
     unknown body or star, an eclipse search that finds nothing, an ephemeris\n\
     error. Bad arguments raise ``TypeError`` or ``ValueError`` instead."
);

/// A `swe.Error` carrying `msg`.
fn calc_err(msg: impl std::fmt::Display) -> PyErr {
    Error::new_err(msg.to_string())
}

// ---------------------------------------------------------------------------
// calc_ut
// ---------------------------------------------------------------------------

/// Core of [`calc_ut`] / [`calc`] as a plain Rust helper (no `#[pyfunction]`),
/// so the two entry points can share it without one pyfunction calling another.
fn calc_ut_impl(jd_ut: f64, ipl: i32, flags: i32) -> PyResult<([f64; 6], i32)> {
    if !jd_ut.is_finite() {
        return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "jd_ut must be a finite number, got {jd_ut}"
        )));
    }
    // A flag whose meaning XALEN does not implement is a bad argument
    // (ValueError); anything the engine itself refuses is a calculation failure.
    if flags & compat::SEFLG_UNSUPPORTED_MASK != 0 {
        return Err(PyValueError::new_err(format!(
            "unsupported flag(s) set (flags={flags:#x}): HELCTR, TOPOCTR, J2000, \
             EQUATORIAL, BARYCTR, XYZ and RADIANS are not implemented"
        )));
    }
    let xx = compat::swe_calc_ut(jd_ut, ipl, flags).map_err(calc_err)?;
    // Swiss returns the flags it actually applied; SEFLG_SWIEPH is always the
    // active backend in XALEN, so report it OR'd into the caller's request.
    let ret_flag = flags | compat::SEFLG_SWIEPH;
    Ok((xx, ret_flag))
}

/// `swe.calc_ut(jd_ut, ipl, flags=SEFLG_SWIEPH)` — geocentric position.
///
/// Returns ``((lon, lat, dist, lon_speed, lat_speed, dist_speed), ret_flag)``
/// matching pyswisseph. Speeds are degrees/day (longitude/latitude) and AU/day
/// (distance); they are ``0.0`` unless ``SEFLG_SPEED`` is in ``flags``.
#[pyfunction]
#[pyo3(signature = (jd_ut, ipl, flags = compat::SEFLG_SWIEPH))]
fn calc_ut(jd_ut: f64, ipl: i32, flags: i32) -> PyResult<([f64; 6], i32)> {
    calc_ut_impl(jd_ut, ipl, flags)
}

/// `swe.calc(jd_et, ipl, flags)` — the ET/TT-input sibling of [`calc_ut`].
///
/// `pyswisseph`'s `swe.calc` takes Ephemeris Time (≈TT) as its time argument,
/// whereas `swe.calc_ut` takes UT1. This is a CORRECT drop-in: the input is
/// treated as TT and converted to UT1 (`jd_ut1 = jd_tt − ΔT`) before the
/// position is computed, so `swe.calc(jd_tt, ...)` and
/// `swe.calc_ut(jd_tt − deltat(jd_tt), ...)` agree. ΔT is evaluated at the input
/// epoch; the residual error from using `jd_et` rather than `jd_ut1` to look up
/// ΔT is second-order in ΔT (sub-milliarcsecond for any modern date).
#[pyfunction]
#[pyo3(signature = (jd_et, ipl, flags = compat::SEFLG_SWIEPH))]
fn calc(jd_et: f64, ipl: i32, flags: i32) -> PyResult<([f64; 6], i32)> {
    if !jd_et.is_finite() {
        return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "jd_et must be a finite number, got {jd_et}"
        )));
    }
    // Convert the TT/ET input to UT1. compat::swe_deltat returns ΔT in DAYS
    // (Swiss convention), so subtracting it from the TT JD yields the UT1 JD the
    // compat layer expects.
    let jd_ut1 = jd_et - compat::swe_deltat(jd_et);
    calc_ut_impl(jd_ut1, ipl, flags)
}

// ---------------------------------------------------------------------------
// houses_ex / houses
// ---------------------------------------------------------------------------

/// Parse a pyswisseph `hsys` argument into a single house-system char.
///
/// pyswisseph takes `hsys` as **bytes** (`b'P'`); existing code also passes a
/// `str` (`'P'`). Accept either here so the drop-in does not break on the
/// idiomatic bytes form. A 1-byte/1-char value is required, matching Swiss.
fn parse_hsys(hsys: &Bound<'_, PyAny>) -> PyResult<char> {
    // bytes (the pyswisseph-native form, e.g. b'P')
    if let Ok(b) = hsys.cast::<PyBytes>() {
        let bytes = b.as_bytes();
        return match bytes.first() {
            Some(&byte) => Ok(byte as char),
            None => Err(pyo3::exceptions::PyValueError::new_err(
                "empty house system code (expected a single byte, e.g. b'P')",
            )),
        };
    }
    // str (e.g. 'P')
    if let Ok(s) = hsys.cast::<PyString>() {
        let text: String = s.extract()?;
        return text.chars().next().ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(
                "empty house system code (expected a single character, e.g. 'P')",
            )
        });
    }
    Err(pyo3::exceptions::PyTypeError::new_err(
        "hsys must be bytes (e.g. b'P') or str (e.g. 'P')",
    ))
}

/// Core of [`houses_ex`] / [`houses`] as a plain Rust helper.
///
/// `sidereal` selects the tropical (`false`) vs sidereal (`true`,
/// `SEFLG_SIDEREAL`) frame, the latter subtracting the active-mode ayanamsa.
fn houses_impl(
    jd_ut: f64,
    lat: f64,
    lon: f64,
    hchar: char,
    sidereal: bool,
) -> PyResult<([f64; 12], [f64; 8])> {
    let h = compat::swe_houses_ex(jd_ut, lat, lon, hchar, sidereal).map_err(calc_err)?;
    let mut cusps = [0.0_f64; 12];
    for (i, c) in h.cusps.iter().enumerate().take(12) {
        cusps[i] = *c;
    }
    // ascmc layout (Swiss): [0]=Ascendant [1]=MC [2]=ARMC [3]=Vertex
    // [4]=equatorial ascendant [5]=co-ascendant(Koch) [6]=co-ascendant(Munkasey)
    // [7]=polar ascendant. All eight are now populated with real angles.
    let ascmc = [
        h.ascendant,
        h.mc,
        h.armc,
        h.vertex,
        h.equatorial_ascendant,
        h.co_ascendant_koch,
        h.co_ascendant_munkasey,
        h.polar_ascendant_munkasey,
    ];
    Ok((cusps, ascmc))
}

/// `swe.houses_ex(jd_ut, lat, lon, hsys=b'P', flags=0)`.
///
/// Returns ``(cusps, ascmc)`` where ``cusps`` is a 12-tuple of house cusps in
/// degrees and ``ascmc`` is an 8-tuple ``(asc, mc, armc, vertex,
/// equatorial_ascendant, co_ascendant_koch, co_ascendant_munkasey,
/// polar_ascendant)`` — all eight slots populated, matching Swiss.
/// ``hsys`` accepts a single-character house code, as bytes (``b'P'``) or str
/// (``'P'``), matching pyswisseph which takes bytes.
///
/// When ``flags`` includes ``SEFLG_SIDEREAL`` the cusps and angles are returned
/// in the sidereal frame (active-mode ayanamsa subtracted, default Lahiri; set
/// via [`set_sid_mode`]). Other position-altering flags are not modelled and
/// are ignored for houses (Swiss applies them only to `swe_calc`).
#[pyfunction]
#[pyo3(signature = (jd_ut, lat, lon, hsys = None, flags = 0))]
fn houses_ex(
    jd_ut: f64,
    lat: f64,
    lon: f64,
    hsys: Option<&Bound<'_, PyAny>>,
    flags: i32,
) -> PyResult<([f64; 12], [f64; 8])> {
    let hchar = match hsys {
        Some(h) => parse_hsys(h)?,
        None => 'P',
    };
    let sidereal = flags & compat::SEFLG_SIDEREAL != 0;
    houses_impl(jd_ut, lat, lon, hchar, sidereal)
}

/// `swe.houses(jd_ut, lat, lon, hsys=b'P')` — like [`houses_ex`] without the
/// trailing flags argument (tropical frame).
#[pyfunction]
#[pyo3(signature = (jd_ut, lat, lon, hsys = None))]
fn houses(
    jd_ut: f64,
    lat: f64,
    lon: f64,
    hsys: Option<&Bound<'_, PyAny>>,
) -> PyResult<([f64; 12], [f64; 8])> {
    let hchar = match hsys {
        Some(h) => parse_hsys(h)?,
        None => 'P',
    };
    houses_impl(jd_ut, lat, lon, hchar, false)
}

// ---------------------------------------------------------------------------
// ayanamsa
// ---------------------------------------------------------------------------

/// `swe.get_ayanamsa_ex_ut(jd_ut, flags)` — returns ``(ret_flag, ayanamsa_deg)``
/// for the currently active sidereal mode (set via [`set_sid_mode`]).
#[pyfunction]
#[pyo3(signature = (jd_ut, flags = compat::SEFLG_SWIEPH))]
fn get_ayanamsa_ex_ut(jd_ut: f64, flags: i32) -> PyResult<(i32, f64)> {
    let aya = compat::swe_get_ayanamsa_ut_ex(jd_ut, compat::active_sid_mode()).map_err(calc_err)?;
    Ok((flags | compat::SEFLG_SWIEPH, aya))
}

/// `swe.get_ayanamsa_ut(jd_ut)` — ayanamsa (degrees) for the active sidereal
/// mode. Returns a bare float, like pyswisseph.
#[pyfunction]
fn get_ayanamsa_ut(jd_ut: f64) -> PyResult<f64> {
    compat::swe_get_ayanamsa_ut_ex(jd_ut, compat::active_sid_mode()).map_err(calc_err)
}

/// `swe.set_sid_mode(sidmode, t0=0.0, ayan_t0=0.0)` — select the ayanamsa used by
/// sidereal `calc_ut` calls and `get_ayanamsa_*`. Only `sidmode` (an `SE_SIDM_*`
/// id) is honoured; `t0` / `ayan_t0` are accepted for source compatibility and
/// ignored (XALEN's named ayanamsa models carry their own reference epochs).
#[pyfunction]
#[pyo3(signature = (sidmode, t0 = 0.0, ayan_t0 = 0.0))]
fn set_sid_mode(sidmode: i32, t0: f64, ayan_t0: f64) {
    compat::swe_set_sid_mode(sidmode, t0, ayan_t0);
}

// ---------------------------------------------------------------------------
// calendar
// ---------------------------------------------------------------------------

/// `swe.julday(year, month, day, hour=12.0, cal=GREG_CAL)`.
#[pyfunction]
#[pyo3(signature = (year, month, day, hour = 12.0, cal = compat::SE_GREG_CAL))]
fn julday(year: i32, month: i32, day: i32, hour: f64, cal: i32) -> f64 {
    compat::swe_julday(year, month, day, hour, cal)
}

/// `swe.revjul(jd, cal=GREG_CAL)` — returns ``(year, month, day, hour)``.
#[pyfunction]
#[pyo3(signature = (jd, cal = compat::SE_GREG_CAL))]
fn revjul(jd: f64, cal: i32) -> (i32, i32, i32, f64) {
    compat::swe_revjul(jd, cal)
}

// ---------------------------------------------------------------------------
// delta-T
// ---------------------------------------------------------------------------

/// `swe.deltat(jd)` — ΔT (TT − UT1) in **days**, matching the Swiss convention.
#[pyfunction]
fn deltat(jd: f64) -> f64 {
    compat::swe_deltat(jd)
}

// ---------------------------------------------------------------------------
// fixed stars
// ---------------------------------------------------------------------------

/// Core of [`fixstar2_ut`] / [`fixstar_ut`] as a plain Rust helper.
fn fixstar_impl(star: String, jd_ut: f64, flags: i32) -> PyResult<([f64; 6], String, i32)> {
    let xx = compat::swe_fixstar_ut(&star, jd_ut).map_err(calc_err)?;
    Ok((xx, star, flags | compat::SEFLG_SWIEPH))
}

/// `swe.fixstar2_ut(star, jd_ut, flags=SEFLG_SWIEPH)`.
///
/// Returns ``((lon, lat, dist, lon_speed, lat_speed, dist_speed), star_name,
/// ret_flag)`` matching pyswisseph. The catalog carries no parallax/proper-motion
/// velocity, so distance and the three speed slots are ``0.0`` and the returned
/// name echoes the requested name.
#[pyfunction]
#[pyo3(signature = (star, jd_ut, flags = compat::SEFLG_SWIEPH))]
fn fixstar2_ut(star: String, jd_ut: f64, flags: i32) -> PyResult<([f64; 6], String, i32)> {
    fixstar_impl(star, jd_ut, flags)
}

/// `swe.fixstar_ut(star, jd_ut, flags=SEFLG_SWIEPH)` — alias kept for the older
/// (non-`2`) pyswisseph entry point; identical return shape to [`fixstar2_ut`].
#[pyfunction]
#[pyo3(signature = (star, jd_ut, flags = compat::SEFLG_SWIEPH))]
fn fixstar_ut(star: String, jd_ut: f64, flags: i32) -> PyResult<([f64; 6], String, i32)> {
    fixstar_impl(star, jd_ut, flags)
}

/// `swe.fixstar2_mag(star)` / `swe.fixstar_mag(star)` — visual magnitude.
/// pyswisseph returns ``(mag, name)``; we match that.
#[pyfunction]
fn fixstar2_mag(star: String) -> PyResult<(f64, String)> {
    let mag = compat::swe_fixstar_mag(&star).map_err(calc_err)?;
    Ok((mag, star))
}

// ---------------------------------------------------------------------------
// no-ops (source compatibility)
// ---------------------------------------------------------------------------

/// `swe.set_ephe_path(path)` — no-op. XALEN embeds all data at compile time.
#[pyfunction]
#[pyo3(signature = (path = None))]
fn set_ephe_path(path: Option<String>) {
    let _ = path;
    compat::swe_set_ephe_path("");
}

/// `swe.close()` — no-op. XALEN holds no resources to release.
#[pyfunction]
fn close() {
    compat::swe_close();
}

// ---------------------------------------------------------------------------
// shared argument handling for rise/set and eclipses
// ---------------------------------------------------------------------------

/// `rsmi` event selectors and `BIT_*` modifiers of `swe_rise_trans`
/// (values from the Swiss Ephemeris header).
const CALC_RISE: i32 = 1;
const CALC_SET: i32 = 2;
const CALC_MTRANSIT: i32 = 4;
const CALC_ITRANSIT: i32 = 8;
const BIT_GEOCTR_NO_ECL_LAT: i32 = 128;
const BIT_DISC_CENTER: i32 = 256;
const BIT_NO_REFRACTION: i32 = 512;
const BIT_CIVIL_TWILIGHT: i32 = 1024;
const BIT_NAUTIC_TWILIGHT: i32 = 2048;
const BIT_ASTRO_TWILIGHT: i32 = 4096;
const BIT_DISC_BOTTOM: i32 = 8192;
const BIT_FIXED_DISC_SIZE: i32 = 16384;
const BIT_FORCE_SLOW_METHOD: i32 = 32768;
const BIT_HINDU_RISING: i32 = BIT_DISC_CENTER | BIT_NO_REFRACTION | BIT_GEOCTR_NO_ECL_LAT;

/// Eclipse type and visibility bits (`SE_ECL_*`).
const ECL_CENTRAL: i32 = 1;
const ECL_NONCENTRAL: i32 = 2;
const ECL_TOTAL: i32 = 4;
const ECL_ANNULAR: i32 = 8;
const ECL_PARTIAL: i32 = 16;
const ECL_ANNULAR_TOTAL: i32 = 32;
const ECL_HYBRID: i32 = ECL_ANNULAR_TOTAL;
const ECL_PENUMBRAL: i32 = 64;
const ECL_ALLTYPES_SOLAR: i32 =
    ECL_CENTRAL | ECL_NONCENTRAL | ECL_TOTAL | ECL_ANNULAR | ECL_PARTIAL | ECL_ANNULAR_TOTAL;
const ECL_ALLTYPES_LUNAR: i32 = ECL_TOTAL | ECL_PARTIAL | ECL_PENUMBRAL;
const ECL_VISIBLE: i32 = 128;
const ECL_MAX_VISIBLE: i32 = 256;
const ECL_1ST_VISIBLE: i32 = 512;
const ECL_2ND_VISIBLE: i32 = 1024;
const ECL_3RD_VISIBLE: i32 = 2048;
const ECL_4TH_VISIBLE: i32 = 4096;
const ECL_PARTBEG_VISIBLE: i32 = ECL_1ST_VISIBLE;
const ECL_TOTBEG_VISIBLE: i32 = ECL_2ND_VISIBLE;
const ECL_TOTEND_VISIBLE: i32 = ECL_3RD_VISIBLE;
const ECL_PARTEND_VISIBLE: i32 = ECL_4TH_VISIBLE;
const ECL_PENUMBBEG_VISIBLE: i32 = 8192;
const ECL_PENUMBEND_VISIBLE: i32 = 16384;
const ECL_OCC_BEG_DAYLIGHT: i32 = 8192;
const ECL_OCC_END_DAYLIGHT: i32 = 16384;
const ECL_ONE_TRY: i32 = 32768;

/// Ephemeris-selection flags. XALEN always computes with its own engine, so
/// these are accepted and change nothing.
const FLG_JPLEPH: i32 = 1;
const FLG_MOSEPH: i32 = 4;

/// pyswisseph's value for an unknown Saros series or member number.
const SAROS_UNKNOWN: f64 = -99_999_999.0;

/// An eclipse search gives up after this many years without a match.
const ECLIPSE_SEARCH_YEARS: f64 = 100.0;
/// Each search step covers this many days; consecutive steps overlap a little
/// so an eclipse at a step boundary is never missed.
const ECLIPSE_STEP_DAYS: f64 = 400.0;
const ECLIPSE_STEP_OVERLAP_DAYS: f64 = 2.0;

fn finite_jd(name: &str, jd: f64) -> PyResult<()> {
    if jd.is_finite() {
        Ok(())
    } else {
        Err(PyValueError::new_err(format!(
            "{name} must be a finite number, got {jd}"
        )))
    }
}

/// Parse pyswisseph's `geopos` (a sequence `(longitude, latitude, altitude in
/// metres)`, east and north positive) into `(lon, lat, alt)`.
fn parse_geopos(geopos: &Bound<'_, PyAny>) -> PyResult<(f64, f64, f64)> {
    let bad = || {
        PyTypeError::new_err(
            "geopos must be a sequence of 3 numbers: (longitude, latitude, altitude in metres)",
        )
    };
    if geopos.is_instance_of::<PyString>() || geopos.is_instance_of::<PyBytes>() {
        return Err(bad());
    }
    let v: Vec<f64> = geopos.extract().map_err(|_| bad())?;
    if v.len() < 3 {
        return Err(bad());
    }
    let (lon, lat, alt) = (v[0], v[1], v[2]);
    if !(lon.is_finite() && alt.is_finite() && (-90.0..=90.0).contains(&lat)) {
        return Err(PyValueError::new_err(format!(
            "geopos out of range: longitude={lon}, latitude={lat}, altitude={alt}"
        )));
    }
    Ok((lon, lat, alt))
}

/// pyswisseph parses `backwards` as an int; accept any truthy value.
fn is_backwards(backwards: Option<&Bound<'_, PyAny>>) -> PyResult<bool> {
    backwards.map_or(Ok(false), |b| b.is_truthy())
}

fn tuple10<'py>(py: Python<'py>, values: [f64; 10]) -> PyResult<Bound<'py, PyTuple>> {
    PyTuple::new(py, values)
}

fn tuple20<'py>(py: Python<'py>, values: [f64; 20]) -> PyResult<Bound<'py, PyTuple>> {
    PyTuple::new(py, values)
}

// ---------------------------------------------------------------------------
// rise_trans / rise_trans_true_hor
// ---------------------------------------------------------------------------

/// The body for a rise/set/transit search: a planet number of the Sun, Moon,
/// a planet or Chiron.
fn rise_body(fname: &str, body: &Bound<'_, PyAny>) -> PyResult<Body> {
    if body.is_instance_of::<PyString>() || body.is_instance_of::<PyBytes>() {
        return Err(calc_err(format!(
            "{fname}: fixed-star rise and set are not supported; pass a planet number"
        )));
    }
    let ipl: i32 = body
        .extract()
        .map_err(|_| PyTypeError::new_err("body must be a planet number (int) or a star name"))?;
    let b = compat::se_planet_to_body(ipl).map_err(|e| calc_err(format!("{fname}: {e}")))?;
    match b {
        Body::Sun
        | Body::Moon
        | Body::Mercury
        | Body::Venus
        | Body::Mars
        | Body::Jupiter
        | Body::Saturn
        | Body::Uranus
        | Body::Neptune
        | Body::Pluto
        | Body::Chiron => Ok(b),
        _ => Err(calc_err(format!(
            "{fname}: rise, set and transit are not available for body {ipl}"
        ))),
    }
}

/// Shared core of [`rise_trans`] and [`rise_trans_true_hor`].
fn rise_trans_impl<'py>(
    py: Python<'py>,
    fname: &str,
    tjdut: f64,
    body: &Bound<'py, PyAny>,
    rsmi: i32,
    geopos: &Bound<'py, PyAny>,
    atpress: f64,
    attemp: f64,
    horhgt: f64,
) -> PyResult<(i32, Bound<'py, PyTuple>)> {
    finite_jd("tjdut", tjdut)?;
    let (lon, lat, alt) = parse_geopos(geopos)?;
    let body = rise_body(fname, body)?;
    if !(atpress.is_finite() && atpress >= 0.0 && attemp.is_finite() && attemp > -273.15) {
        return Err(PyValueError::new_err(format!(
            "{fname}: atpress must be >= 0 hPa and attemp above -273.15 C \
             (got {atpress}, {attemp})"
        )));
    }
    if !horhgt.is_finite() {
        return Err(PyValueError::new_err(format!(
            "{fname}: horhgt must be a finite number of degrees, got {horhgt}"
        )));
    }
    let almanac = Almanac::default_vedic();
    let jd = JdUT1(tjdut);
    let found = if rsmi & (CALC_MTRANSIT | CALC_ITRANSIT) != 0 {
        let upper = rsmi & CALC_MTRANSIT != 0;
        rise_set::next_transit(&almanac, body, jd, lat, lon, alt, upper)
    } else {
        let twilight = if rsmi & BIT_CIVIL_TWILIGHT != 0 {
            Some(Twilight::Civil)
        } else if rsmi & BIT_NAUTIC_TWILIGHT != 0 {
            Some(Twilight::Nautical)
        } else if rsmi & BIT_ASTRO_TWILIGHT != 0 {
            Some(Twilight::Astronomical)
        } else {
            None
        };
        let disc = if rsmi & BIT_DISC_BOTTOM != 0 {
            DiscPoint::LowerLimb
        } else if rsmi & BIT_DISC_CENTER != 0 {
            DiscPoint::Center
        } else {
            DiscPoint::UpperLimb
        };
        let crossing = HorizonCrossing {
            disc,
            refraction: rsmi & BIT_NO_REFRACTION == 0,
            // pyswisseph: a pressure of 0 means "estimate it from the altitude".
            pressure_hpa: if atpress == 0.0 {
                rise_set::standard_pressure_hpa(alt)
            } else {
                atpress
            },
            temperature_c: attemp,
            horizon_height_deg: horhgt,
            fixed_disc_size: rsmi & BIT_FIXED_DISC_SIZE != 0,
            twilight,
            geocentric_no_latitude: rsmi & BIT_GEOCTR_NO_ECL_LAT != 0,
        };
        // CALC_RISE wins when both bits are set; neither bit means a rise.
        let rising = rsmi & CALC_RISE != 0 || rsmi & CALC_SET == 0;
        rise_set::next_horizon_crossing(&almanac, body, jd, lat, lon, alt, rising, &crossing)
    }
    .map_err(|e| calc_err(format!("{fname}: {e}")))?;
    let mut tret = [0.0_f64; 10];
    let res = match found {
        Some(t) => {
            tret[0] = t.as_f64();
            0
        }
        None => -2,
    };
    Ok((res, tuple10(py, tret)?))
}

/// ``swe.rise_trans(tjdut, body, rsmi, geopos, atpress=0.0, attemp=0.0,
/// flags=FLG_SWIEPH)`` -> ``(res, tret)``.
///
/// Next rise, set or meridian transit of ``body`` after ``tjdut`` (UT) for
/// ``geopos = (longitude, latitude, altitude_m)``. ``rsmi`` picks one event:
/// ``CALC_RISE`` (also when no event bit is set), ``CALC_SET``,
/// ``CALC_MTRANSIT`` (upper) or ``CALC_ITRANSIT`` (lower), combined with
/// ``BIT_DISC_CENTER``, ``BIT_DISC_BOTTOM``, ``BIT_NO_REFRACTION``,
/// ``BIT_FIXED_DISC_SIZE``, ``BIT_HINDU_RISING`` or a twilight bit
/// (``BIT_CIVIL_TWILIGHT`` -6 deg, ``BIT_NAUTIC_TWILIGHT`` -12 deg,
/// ``BIT_ASTRO_TWILIGHT`` -18 deg, Sun centre). The default is the upper limb
/// with refraction. ``atpress`` (hPa; 0 = standard atmosphere at the altitude)
/// and ``attemp`` (deg C) scale the refraction. ``flags`` selects the
/// ephemeris in pyswisseph and is accepted and ignored here.
///
/// ``res`` is 0 when the event was found (``tret[0]`` is its JD UT) and -2
/// when the body does not rise or set within about a day (circumpolar);
/// ``tret`` is a 10-tuple whose other slots are 0.0. Bodies: the Sun, Moon,
/// planets and Chiron; anything else, including a star name, raises
/// ``swe.Error``.
#[pyfunction]
#[pyo3(signature = (tjdut, body, rsmi, geopos, atpress = 0.0, attemp = 0.0, flags = compat::SEFLG_SWIEPH))]
fn rise_trans<'py>(
    py: Python<'py>,
    tjdut: f64,
    body: &Bound<'py, PyAny>,
    rsmi: i32,
    geopos: &Bound<'py, PyAny>,
    atpress: f64,
    attemp: f64,
    flags: i32,
) -> PyResult<(i32, Bound<'py, PyTuple>)> {
    let _ = flags;
    rise_trans_impl(
        py,
        "rise_trans",
        tjdut,
        body,
        rsmi,
        geopos,
        atpress,
        attemp,
        0.0,
    )
}

/// ``swe.rise_trans_true_hor(tjdut, body, rsmi, geopos, atpress=0.0,
/// attemp=0.0, horhgt=0.0, flags=FLG_SWIEPH)`` -> ``(res, tret)``.
///
/// Like ``rise_trans`` with the local horizon at an apparent altitude of
/// ``horhgt`` degrees (a mountain ridge, a raised skyline). The refraction is
/// taken at that altitude. ``horhgt=0`` gives the same result as
/// ``rise_trans``.
#[pyfunction]
#[pyo3(signature = (tjdut, body, rsmi, geopos, atpress = 0.0, attemp = 0.0, horhgt = 0.0, flags = compat::SEFLG_SWIEPH))]
fn rise_trans_true_hor<'py>(
    py: Python<'py>,
    tjdut: f64,
    body: &Bound<'py, PyAny>,
    rsmi: i32,
    geopos: &Bound<'py, PyAny>,
    atpress: f64,
    attemp: f64,
    horhgt: f64,
    flags: i32,
) -> PyResult<(i32, Bound<'py, PyTuple>)> {
    let _ = flags;
    rise_trans_impl(
        py,
        "rise_trans_true_hor",
        tjdut,
        body,
        rsmi,
        geopos,
        atpress,
        attemp,
        horhgt,
    )
}

// ---------------------------------------------------------------------------
// eclipse search
// ---------------------------------------------------------------------------

/// Walk the eclipses `find(start, end)` returns, in time order away from
/// `tjd` (forward, or backward when `backward`), and return the first one for
/// which `visit` gives `Some`. Only eclipses whose maximum lies strictly after
/// (before) `tjd` are visited. Raises `swe.Error` after
/// [`ECLIPSE_SEARCH_YEARS`] without a match.
fn search_eclipses<E, T>(
    fname: &str,
    tjd: f64,
    backward: bool,
    find: impl Fn(f64, f64) -> Vec<E>,
    jd_max: impl Fn(&E) -> f64,
    mut visit: impl FnMut(&E) -> PyResult<Option<T>>,
) -> PyResult<T> {
    let steps = (ECLIPSE_SEARCH_YEARS * 365.25 / ECLIPSE_STEP_DAYS).ceil() as usize;
    let mut edge = tjd;
    for _ in 0..steps {
        let (start, end) = if backward {
            (edge - ECLIPSE_STEP_DAYS, edge + ECLIPSE_STEP_OVERLAP_DAYS)
        } else {
            (edge - ECLIPSE_STEP_OVERLAP_DAYS, edge + ECLIPSE_STEP_DAYS)
        };
        let mut found: Vec<E> = find(start, end)
            .into_iter()
            .filter(|e| {
                let t = jd_max(e);
                if backward { t < tjd } else { t > tjd }
            })
            .collect();
        found.sort_by(|a, b| jd_max(a).total_cmp(&jd_max(b)));
        if backward {
            found.reverse();
        }
        for e in &found {
            if let Some(hit) = visit(e)? {
                return Ok(hit);
            }
        }
        edge = if backward { start } else { end };
    }
    Err(calc_err(format!(
        "{fname}: no matching eclipse within {ECLIPSE_SEARCH_YEARS} years"
    )))
}

/// pyswisseph `retflag` bits of a global solar eclipse: its type, plus
/// `ECL_CENTRAL` or `ECL_NONCENTRAL` for a total, annular or hybrid one.
fn solar_flags(e: &SolarEclipse) -> i32 {
    let kind = match e.eclipse_type {
        SolarEclipseType::Partial => return ECL_PARTIAL,
        SolarEclipseType::Total => ECL_TOTAL,
        SolarEclipseType::Annular => ECL_ANNULAR,
        SolarEclipseType::Hybrid => ECL_ANNULAR_TOTAL,
    };
    let centrality = if e.is_central() {
        ECL_CENTRAL
    } else {
        ECL_NONCENTRAL
    };
    kind | centrality
}

/// Whether solar `retflag` bits satisfy pyswisseph's `ecltype` filter
/// (0 = any; type bits and central bits each narrow it when present).
fn solar_matches(retflag: i32, ecltype: i32) -> bool {
    let types = ecltype & (ECL_TOTAL | ECL_ANNULAR | ECL_PARTIAL | ECL_ANNULAR_TOTAL);
    let centrality = ecltype & (ECL_CENTRAL | ECL_NONCENTRAL);
    let type_ok = types == 0 || retflag & types != 0;
    // A partial eclipse has no central path, so it counts as non-central.
    let partial_noncentral = retflag & ECL_PARTIAL != 0 && centrality & ECL_NONCENTRAL != 0;
    let central_ok = centrality == 0 || retflag & centrality != 0 || partial_noncentral;
    type_ok && central_ok
}

fn lunar_flag(kind: LunarEclipseType) -> i32 {
    match kind {
        LunarEclipseType::Total => ECL_TOTAL,
        LunarEclipseType::Partial => ECL_PARTIAL,
        LunarEclipseType::Penumbral => ECL_PENUMBRAL,
    }
}

/// pyswisseph's lunar `tret`: maximum, (unused), partial begin/end, total
/// begin/end, penumbral begin/end, then two slots for the caller. Phases the
/// eclipse does not reach are 0.0.
fn lunar_tret(e: &LunarEclipse) -> [f64; 10] {
    let c = &e.contacts;
    let z = |t: Option<f64>| t.unwrap_or(0.0);
    [
        e.jd_maximum,
        0.0,
        z(c.partial_begin),
        z(c.partial_end),
        z(c.total_begin),
        z(c.total_end),
        z(c.penumbral_begin),
        z(c.penumbral_end),
        0.0,
        0.0,
    ]
}

/// `(retflag, attr)` of `lun_eclipse_how` at `jd` for an observer.
fn lunar_how(
    almanac: &Almanac,
    fname: &str,
    jd: f64,
    lat: f64,
    lon: f64,
    alt: f64,
) -> PyResult<(i32, [f64; 20])> {
    let err = |e: xalen_ephem::EphemerisError| calc_err(format!("{fname}: {e}"));
    let s = eclipse::lunar_shadow_state(almanac, jd).map_err(err)?;
    let moon_alt = almanac
        .topocentric_altitude_deg(Body::Moon, JdUT1(jd), lat, lon, alt)
        .map_err(err)?;
    let mut attr = [0.0_f64; 20];
    attr[0] = s.umbral_magnitude;
    attr[1] = s.penumbral_magnitude;
    attr[5] = moon_alt;
    attr[7] = s.separation_deg;
    attr[8] = s.umbral_magnitude;
    attr[9] = SAROS_UNKNOWN;
    attr[10] = SAROS_UNKNOWN;
    Ok((s.eclipse_type().map_or(0, lunar_flag), attr))
}

/// The next sunrise/moonrise (`rising`) or set of `body` after `from`, if it
/// comes before `until` (standard upper-limb, refracted horizon).
fn crossing_between(
    almanac: &Almanac,
    fname: &str,
    body: Body,
    (lat, lon, alt): (f64, f64, f64),
    from: f64,
    until: f64,
    rising: bool,
) -> PyResult<Option<f64>> {
    let t = rise_set::next_horizon_crossing(
        almanac,
        body,
        JdUT1(from),
        lat,
        lon,
        alt,
        rising,
        &HorizonCrossing::default(),
    )
    .map_err(|e| calc_err(format!("{fname}: {e}")))?;
    Ok(t.map(|t| t.as_f64()).filter(|&t| t < until))
}

/// ``swe.sol_eclipse_when_glob(tjdut, flags=FLG_SWIEPH, ecltype=0,
/// backwards=False)`` -> ``(retflag, tret)``.
///
/// Next (or, with ``backwards``, previous) solar eclipse anywhere on Earth
/// after ``tjdut`` (UT). ``ecltype`` filters by ``ECL_TOTAL``, ``ECL_ANNULAR``,
/// ``ECL_PARTIAL``, ``ECL_ANNULAR_TOTAL`` and ``ECL_CENTRAL`` /
/// ``ECL_NONCENTRAL`` (0 = any). ``retflag`` is the eclipse type, plus
/// ``ECL_CENTRAL`` or ``ECL_NONCENTRAL`` for a total, annular or hybrid
/// eclipse. ``tret`` is a 10-tuple; ``tret[0]`` is the time of greatest
/// eclipse (JD UT). The global contact times ``tret[1:10]`` (local noon,
/// begin, end, totality and centre-line limits) are not computed and are 0.0;
/// use ``sol_eclipse_when_loc`` for contacts at a place. Raises ``swe.Error``
/// if nothing matches within 100 years.
#[pyfunction]
#[pyo3(signature = (tjdut, flags = compat::SEFLG_SWIEPH, ecltype = 0, backwards = None))]
fn sol_eclipse_when_glob<'py>(
    py: Python<'py>,
    tjdut: f64,
    flags: i32,
    ecltype: i32,
    backwards: Option<&Bound<'py, PyAny>>,
) -> PyResult<(i32, Bound<'py, PyTuple>)> {
    let _ = flags;
    finite_jd("tjdut", tjdut)?;
    let backward = is_backwards(backwards)?;
    let almanac = Almanac::default_vedic();
    let (retflag, jd) = search_eclipses(
        "sol_eclipse_when_glob",
        tjdut,
        backward,
        |a, b| eclipse::find_solar_eclipses(&almanac, a, b),
        |e: &SolarEclipse| e.jd_maximum,
        |e| {
            let f = solar_flags(e);
            Ok(solar_matches(f, ecltype).then_some((f, e.jd_maximum)))
        },
    )?;
    let mut tret = [0.0_f64; 10];
    tret[0] = jd;
    Ok((retflag, tuple10(py, tret)?))
}

/// ``swe.sol_eclipse_when_loc(tjdut, geopos, flags=FLG_SWIEPH,
/// backwards=False)`` -> ``(retflag, tret, attr)``.
///
/// Next (or previous) solar eclipse visible from ``geopos = (longitude,
/// latitude, altitude_m)``. ``retflag``: ``ECL_TOTAL``, ``ECL_ANNULAR`` or
/// ``ECL_PARTIAL`` as seen there, plus ``ECL_VISIBLE``, ``ECL_MAX_VISIBLE`` and
/// ``ECL_1ST_VISIBLE`` .. ``ECL_4TH_VISIBLE`` for the phases with the Sun above
/// the horizon. ``tret`` (10): maximum, first to fourth contact (0.0 when
/// absent), sunrise and sunset between first and fourth contact (0.0 when
/// none), then 0.0. ``attr`` (20): ``[0]`` magnitude (fraction of the solar
/// diameter covered), ``[2]`` obscuration, ``[5]`` true altitude of the Sun at
/// maximum, ``[8]`` magnitude, ``[9]``/``[10]`` Saros series/member (-99999999,
/// unknown); the other slots are not computed and are 0.0.
#[pyfunction]
#[pyo3(signature = (tjdut, geopos, flags = compat::SEFLG_SWIEPH, backwards = None))]
fn sol_eclipse_when_loc<'py>(
    py: Python<'py>,
    tjdut: f64,
    geopos: &Bound<'py, PyAny>,
    flags: i32,
    backwards: Option<&Bound<'py, PyAny>>,
) -> PyResult<(i32, Bound<'py, PyTuple>, Bound<'py, PyTuple>)> {
    const FNAME: &str = "sol_eclipse_when_loc";
    let _ = flags;
    finite_jd("tjdut", tjdut)?;
    let place = parse_geopos(geopos)?;
    let (lon, lat, alt) = place;
    let observer = (lat, lon, alt);
    let backward = is_backwards(backwards)?;
    let almanac = Almanac::default_vedic();
    let model = DeltaTModel::StephensonMorrisonHohenkerk2016;
    let sun_up = |jd: f64| {
        rise_set::is_above_horizon(&almanac, Body::Sun, JdUT1(jd), lat, lon, alt)
            .map_err(|e| calc_err(format!("{FNAME}: {e}")))
    };
    let (retflag, tret, attr) = search_eclipses(
        FNAME,
        tjdut,
        backward,
        |a, b| eclipse::find_solar_eclipses(&almanac, a, b),
        |e: &SolarEclipse| e.jd_maximum,
        |e| {
            let seed = JdUT1(e.jd_maximum).to_tt(&model);
            let Some(lc) = local_eclipse::local_circumstances(&almanac, seed, lat, lon, alt) else {
                return Ok(None);
            };
            let after = if backward {
                lc.jd_maximum_ut < tjdut
            } else {
                lc.jd_maximum_ut > tjdut
            };
            let kind = match lc.eclipse_type {
                LocalSolarType::Total => ECL_TOTAL,
                LocalSolarType::Annular => ECL_ANNULAR,
                LocalSolarType::Partial => ECL_PARTIAL,
                LocalSolarType::None => return Ok(None),
            };
            if !after {
                return Ok(None);
            }
            let mut retflag = kind;
            if sun_up(lc.jd_maximum_ut)? {
                retflag |= ECL_MAX_VISIBLE;
            }
            let contacts = [
                (lc.c1, ECL_1ST_VISIBLE),
                (lc.c2, ECL_2ND_VISIBLE),
                (lc.c3, ECL_3RD_VISIBLE),
                (lc.c4, ECL_4TH_VISIBLE),
            ];
            for (jd, bit) in contacts
                .iter()
                .filter_map(|(c, bit)| c.map(|c| (c.jd_ut, *bit)))
            {
                if sun_up(jd)? {
                    retflag |= bit;
                }
            }
            let first = lc.c1.map_or(lc.jd_maximum_ut, |c| c.jd_ut);
            let last = lc.c4.map_or(lc.jd_maximum_ut, |c| c.jd_ut);
            let sunrise =
                crossing_between(&almanac, FNAME, Body::Sun, observer, first, last, true)?;
            let sunset =
                crossing_between(&almanac, FNAME, Body::Sun, observer, first, last, false)?;
            if retflag == kind && sunrise.is_none() && sunset.is_none() {
                return Ok(None); // the whole eclipse happens below the horizon
            }
            retflag |= ECL_VISIBLE;
            let z = |c: Option<local_eclipse::Contact>| c.map_or(0.0, |c| c.jd_ut);
            let tret = [
                lc.jd_maximum_ut,
                z(lc.c1),
                z(lc.c2),
                z(lc.c3),
                z(lc.c4),
                sunrise.unwrap_or(0.0),
                sunset.unwrap_or(0.0),
                0.0,
                0.0,
                0.0,
            ];
            let mut attr = [0.0_f64; 20];
            attr[0] = lc.magnitude;
            attr[2] = lc.obscuration;
            attr[5] = lc.altitude_at_max_deg;
            attr[8] = lc.magnitude;
            attr[9] = SAROS_UNKNOWN;
            attr[10] = SAROS_UNKNOWN;
            Ok(Some((retflag, tret, attr)))
        },
    )?;
    Ok((retflag, tuple10(py, tret)?, tuple20(py, attr)?))
}

/// ``swe.lun_eclipse_when(tjdut, flags=FLG_SWIEPH, ecltype=0,
/// backwards=False)`` -> ``(retflag, tret)``.
///
/// Next (or previous) lunar eclipse after ``tjdut`` (UT). ``ecltype`` filters
/// by ``ECL_TOTAL``, ``ECL_PARTIAL`` and ``ECL_PENUMBRAL`` (0 = any).
/// ``retflag`` is the eclipse type. ``tret`` (10): ``[0]`` greatest eclipse,
/// ``[2]``/``[3]`` partial phase begin/end, ``[4]``/``[5]`` totality
/// begin/end, ``[6]``/``[7]`` penumbral phase begin/end (JD UT; 0.0 for a
/// phase the eclipse does not reach); ``[1]``, ``[8]``, ``[9]`` are 0.0.
#[pyfunction]
#[pyo3(signature = (tjdut, flags = compat::SEFLG_SWIEPH, ecltype = 0, backwards = None))]
fn lun_eclipse_when<'py>(
    py: Python<'py>,
    tjdut: f64,
    flags: i32,
    ecltype: i32,
    backwards: Option<&Bound<'py, PyAny>>,
) -> PyResult<(i32, Bound<'py, PyTuple>)> {
    let _ = flags;
    finite_jd("tjdut", tjdut)?;
    let backward = is_backwards(backwards)?;
    let wanted = ecltype & ECL_ALLTYPES_LUNAR;
    let almanac = Almanac::default_vedic();
    let (retflag, tret) = search_eclipses(
        "lun_eclipse_when",
        tjdut,
        backward,
        |a, b| eclipse::find_lunar_eclipses(&almanac, a, b),
        |e: &LunarEclipse| e.jd_maximum,
        |e| {
            let f = lunar_flag(e.eclipse_type);
            Ok((wanted == 0 || f & wanted != 0).then(|| (f, lunar_tret(e))))
        },
    )?;
    Ok((retflag, tuple10(py, tret)?))
}

/// ``swe.lun_eclipse_when_loc(tjdut, geopos, flags=FLG_SWIEPH,
/// backwards=False)`` -> ``(retflag, tret, attr)``.
///
/// Next (or previous) lunar eclipse visible from ``geopos``. ``retflag``: the
/// eclipse type plus ``ECL_VISIBLE``, ``ECL_MAX_VISIBLE`` and the
/// ``ECL_PENUMBBEG_VISIBLE``, ``ECL_PARTBEG_VISIBLE``, ``ECL_TOTBEG_VISIBLE``,
/// ``ECL_TOTEND_VISIBLE``, ``ECL_PARTEND_VISIBLE``, ``ECL_PENUMBEND_VISIBLE``
/// bits for the contacts with the Moon above the horizon. ``tret`` is laid out
/// as in ``lun_eclipse_when`` (all contacts are given, seen or not), with
/// ``[8]`` moonrise and ``[9]`` moonset when they fall inside the eclipse
/// (0.0 otherwise). ``attr`` is ``lun_eclipse_how`` at greatest eclipse.
#[pyfunction]
#[pyo3(signature = (tjdut, geopos, flags = compat::SEFLG_SWIEPH, backwards = None))]
fn lun_eclipse_when_loc<'py>(
    py: Python<'py>,
    tjdut: f64,
    geopos: &Bound<'py, PyAny>,
    flags: i32,
    backwards: Option<&Bound<'py, PyAny>>,
) -> PyResult<(i32, Bound<'py, PyTuple>, Bound<'py, PyTuple>)> {
    const FNAME: &str = "lun_eclipse_when_loc";
    let _ = flags;
    finite_jd("tjdut", tjdut)?;
    let (lon, lat, alt) = parse_geopos(geopos)?;
    let observer = (lat, lon, alt);
    let backward = is_backwards(backwards)?;
    let almanac = Almanac::default_vedic();
    let moon_up = |jd: f64| {
        rise_set::is_above_horizon(&almanac, Body::Moon, JdUT1(jd), lat, lon, alt)
            .map_err(|e| calc_err(format!("{FNAME}: {e}")))
    };
    let (retflag, tret, attr) = search_eclipses(
        FNAME,
        tjdut,
        backward,
        |a, b| eclipse::find_lunar_eclipses(&almanac, a, b),
        |e: &LunarEclipse| e.jd_maximum,
        |e| {
            let kind = lunar_flag(e.eclipse_type);
            let c = &e.contacts;
            let mut retflag = kind;
            let points = [
                (Some(e.jd_maximum), ECL_MAX_VISIBLE),
                (c.penumbral_begin, ECL_PENUMBBEG_VISIBLE),
                (c.partial_begin, ECL_PARTBEG_VISIBLE),
                (c.total_begin, ECL_TOTBEG_VISIBLE),
                (c.total_end, ECL_TOTEND_VISIBLE),
                (c.partial_end, ECL_PARTEND_VISIBLE),
                (c.penumbral_end, ECL_PENUMBEND_VISIBLE),
            ];
            for (jd, bit) in points.iter().filter_map(|(t, bit)| t.map(|t| (t, *bit))) {
                if moon_up(jd)? {
                    retflag |= bit;
                }
            }
            let first = c.penumbral_begin.unwrap_or(e.jd_maximum);
            let last = c.penumbral_end.unwrap_or(e.jd_maximum);
            let moonrise =
                crossing_between(&almanac, FNAME, Body::Moon, observer, first, last, true)?;
            let moonset =
                crossing_between(&almanac, FNAME, Body::Moon, observer, first, last, false)?;
            if retflag == kind && moonrise.is_none() && moonset.is_none() {
                return Ok(None); // the Moon is down for the whole eclipse
            }
            retflag |= ECL_VISIBLE;
            let mut tret = lunar_tret(e);
            tret[8] = moonrise.unwrap_or(0.0);
            tret[9] = moonset.unwrap_or(0.0);
            let (_, attr) = lunar_how(&almanac, FNAME, e.jd_maximum, lat, lon, alt)?;
            Ok(Some((retflag, tret, attr)))
        },
    )?;
    Ok((retflag, tuple10(py, tret)?, tuple20(py, attr)?))
}

/// ``swe.lun_eclipse_how(tjdut, geopos, flags=FLG_SWIEPH)`` ->
/// ``(retflag, attr)``.
///
/// Lunar eclipse circumstances at the instant ``tjdut`` (UT). ``retflag`` is
/// ``ECL_TOTAL``, ``ECL_PARTIAL`` or ``ECL_PENUMBRAL``, or 0 when the Moon is
/// outside Earth's penumbra. ``attr`` (20): ``[0]`` umbral magnitude, ``[1]``
/// penumbral magnitude, ``[5]`` true (topocentric) altitude of the Moon at
/// ``geopos``, ``[7]`` distance of the Moon from the shadow axis in degrees,
/// ``[8]`` umbral magnitude again, ``[9]``/``[10]`` Saros series/member
/// (-99999999, unknown); the azimuth ``[4]``, apparent altitude ``[6]`` and the
/// remaining slots are not computed and are 0.0.
#[pyfunction]
#[pyo3(signature = (tjdut, geopos, flags = compat::SEFLG_SWIEPH))]
fn lun_eclipse_how<'py>(
    py: Python<'py>,
    tjdut: f64,
    geopos: &Bound<'py, PyAny>,
    flags: i32,
) -> PyResult<(i32, Bound<'py, PyTuple>)> {
    let _ = flags;
    finite_jd("tjdut", tjdut)?;
    let (lon, lat, alt) = parse_geopos(geopos)?;
    let almanac = Almanac::default_vedic();
    let (retflag, attr) = lunar_how(&almanac, "lun_eclipse_how", tjdut, lat, lon, alt)?;
    Ok((retflag, tuple20(py, attr)?))
}

// ---------------------------------------------------------------------------
// module registration
// ---------------------------------------------------------------------------

/// Register every `swe_*` function and `SE_*` / `SEFLG_*` / `SIDM_*` constant on
/// the `xalen.swe` submodule. Constants are exposed under BOTH the bare
/// pyswisseph names (`SUN`, `FLG_SWIEPH`, `SIDM_LAHIRI`) and the C-style
/// `SE_`-prefixed names, so either spelling from existing code keeps working.
pub(crate) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(calc_ut, m)?)?;
    m.add_function(wrap_pyfunction!(calc, m)?)?;
    m.add_function(wrap_pyfunction!(houses_ex, m)?)?;
    m.add_function(wrap_pyfunction!(houses, m)?)?;
    m.add_function(wrap_pyfunction!(get_ayanamsa_ex_ut, m)?)?;
    m.add_function(wrap_pyfunction!(get_ayanamsa_ut, m)?)?;
    m.add_function(wrap_pyfunction!(set_sid_mode, m)?)?;
    m.add_function(wrap_pyfunction!(julday, m)?)?;
    m.add_function(wrap_pyfunction!(revjul, m)?)?;
    m.add_function(wrap_pyfunction!(deltat, m)?)?;
    m.add_function(wrap_pyfunction!(fixstar2_ut, m)?)?;
    m.add_function(wrap_pyfunction!(fixstar_ut, m)?)?;
    m.add_function(wrap_pyfunction!(fixstar2_mag, m)?)?;
    m.add_function(wrap_pyfunction!(set_ephe_path, m)?)?;
    m.add_function(wrap_pyfunction!(close, m)?)?;
    m.add_function(wrap_pyfunction!(rise_trans, m)?)?;
    m.add_function(wrap_pyfunction!(rise_trans_true_hor, m)?)?;
    m.add_function(wrap_pyfunction!(sol_eclipse_when_glob, m)?)?;
    m.add_function(wrap_pyfunction!(sol_eclipse_when_loc, m)?)?;
    m.add_function(wrap_pyfunction!(lun_eclipse_when, m)?)?;
    m.add_function(wrap_pyfunction!(lun_eclipse_when_loc, m)?)?;
    m.add_function(wrap_pyfunction!(lun_eclipse_how, m)?)?;

    // `swe.Error`, raised for every calculation failure.
    m.add("Error", m.py().get_type::<Error>())?;

    // Planet constants — both `SUN` and `SE_SUN` spellings.
    let planets: &[(&str, i32)] = &[
        ("SUN", compat::SE_SUN),
        ("MOON", compat::SE_MOON),
        ("MERCURY", compat::SE_MERCURY),
        ("VENUS", compat::SE_VENUS),
        ("MARS", compat::SE_MARS),
        ("JUPITER", compat::SE_JUPITER),
        ("SATURN", compat::SE_SATURN),
        ("URANUS", compat::SE_URANUS),
        ("NEPTUNE", compat::SE_NEPTUNE),
        ("PLUTO", compat::SE_PLUTO),
        ("MEAN_NODE", compat::SE_MEAN_NODE),
        ("TRUE_NODE", compat::SE_TRUE_NODE),
        ("MEAN_APOG", compat::SE_MEAN_APOG),
        ("OSCU_APOG", compat::SE_OSCU_APOG),
        ("CHIRON", compat::SE_CHIRON),
        ("EARTH", compat::SE_EARTH),
    ];
    for (name, val) in planets {
        m.add(*name, *val)?;
        m.add(format!("SE_{name}").as_str(), *val)?;
    }

    // SEFLG_* flags — both `FLG_*` (pyswisseph) and `SEFLG_*` spellings.
    let flags: &[(&str, i32)] = &[
        ("FLG_SWIEPH", compat::SEFLG_SWIEPH),
        ("FLG_SPEED", compat::SEFLG_SPEED),
        ("FLG_SIDEREAL", compat::SEFLG_SIDEREAL),
        ("FLG_HELCTR", compat::SEFLG_HELCTR),
        ("FLG_J2000", compat::SEFLG_J2000),
        ("FLG_EQUATORIAL", compat::SEFLG_EQUATORIAL),
        ("FLG_BARYCTR", compat::SEFLG_BARYCTR),
        ("FLG_TOPOCTR", compat::SEFLG_TOPOCTR),
        ("FLG_XYZ", compat::SEFLG_XYZ),
        ("FLG_RADIANS", compat::SEFLG_RADIANS),
    ];
    for (name, val) in flags {
        m.add(*name, *val)?;
        m.add(format!("SE{name}").as_str(), *val)?; // SE + FLG_* = SEFLG_*
    }

    // SIDM_* sidereal-mode constants — both `SIDM_*` and `SE_SIDM_*` spellings.
    let sidm: &[(&str, i32)] = &[
        ("SIDM_FAGAN_BRADLEY", compat::SE_SIDM_FAGAN_BRADLEY),
        ("SIDM_LAHIRI", compat::SE_SIDM_LAHIRI),
        ("SIDM_DELUCE", compat::SE_SIDM_DELUCE),
        ("SIDM_RAMAN", compat::SE_SIDM_RAMAN),
        ("SIDM_USHASHASHI", compat::SE_SIDM_USHASHASHI),
        ("SIDM_KRISHNAMURTI", compat::SE_SIDM_KRISHNAMURTI),
        ("SIDM_DJWHAL_KHUL", compat::SE_SIDM_DJWHAL_KHUL),
        ("SIDM_YUKTESWAR", compat::SE_SIDM_YUKTESWAR),
        ("SIDM_JN_BHASIN", compat::SE_SIDM_JN_BHASIN),
        ("SIDM_TRUE_CITRA", compat::SE_SIDM_TRUE_CITRA),
        ("SIDM_TRUE_REVATI", compat::SE_SIDM_TRUE_REVATI),
    ];
    for (name, val) in sidm {
        m.add(*name, *val)?;
        m.add(format!("SE_{name}").as_str(), *val)?;
    }

    // Calendar constants.
    m.add("GREG_CAL", compat::SE_GREG_CAL)?;
    m.add("SE_GREG_CAL", compat::SE_GREG_CAL)?;
    m.add("JUL_CAL", compat::SE_JUL_CAL)?;
    m.add("SE_JUL_CAL", compat::SE_JUL_CAL)?;

    // Ephemeris-selection flags — both `FLG_*` and `SEFLG_*` spellings.
    for (name, val) in [("FLG_JPLEPH", FLG_JPLEPH), ("FLG_MOSEPH", FLG_MOSEPH)] {
        m.add(name, val)?;
        m.add(format!("SE{name}").as_str(), val)?;
    }

    // rise_trans event selectors and modifiers, and eclipse bits — both the
    // bare pyswisseph names and the `SE_` spellings.
    let rise_and_eclipse: &[(&str, i32)] = &[
        ("CALC_RISE", CALC_RISE),
        ("CALC_SET", CALC_SET),
        ("CALC_MTRANSIT", CALC_MTRANSIT),
        ("CALC_ITRANSIT", CALC_ITRANSIT),
        ("BIT_DISC_CENTER", BIT_DISC_CENTER),
        ("BIT_DISC_BOTTOM", BIT_DISC_BOTTOM),
        ("BIT_NO_REFRACTION", BIT_NO_REFRACTION),
        ("BIT_CIVIL_TWILIGHT", BIT_CIVIL_TWILIGHT),
        ("BIT_NAUTIC_TWILIGHT", BIT_NAUTIC_TWILIGHT),
        ("BIT_ASTRO_TWILIGHT", BIT_ASTRO_TWILIGHT),
        ("BIT_FIXED_DISC_SIZE", BIT_FIXED_DISC_SIZE),
        ("BIT_FORCE_SLOW_METHOD", BIT_FORCE_SLOW_METHOD),
        ("BIT_HINDU_RISING", BIT_HINDU_RISING),
        ("ECL_NUT", compat::SE_ECL_NUT),
        ("ECL_CENTRAL", ECL_CENTRAL),
        ("ECL_NONCENTRAL", ECL_NONCENTRAL),
        ("ECL_TOTAL", ECL_TOTAL),
        ("ECL_ANNULAR", ECL_ANNULAR),
        ("ECL_PARTIAL", ECL_PARTIAL),
        ("ECL_ANNULAR_TOTAL", ECL_ANNULAR_TOTAL),
        ("ECL_HYBRID", ECL_HYBRID),
        ("ECL_PENUMBRAL", ECL_PENUMBRAL),
        ("ECL_ALLTYPES_SOLAR", ECL_ALLTYPES_SOLAR),
        ("ECL_ALLTYPES_LUNAR", ECL_ALLTYPES_LUNAR),
        ("ECL_VISIBLE", ECL_VISIBLE),
        ("ECL_MAX_VISIBLE", ECL_MAX_VISIBLE),
        ("ECL_1ST_VISIBLE", ECL_1ST_VISIBLE),
        ("ECL_PARTBEG_VISIBLE", ECL_PARTBEG_VISIBLE),
        ("ECL_2ND_VISIBLE", ECL_2ND_VISIBLE),
        ("ECL_TOTBEG_VISIBLE", ECL_TOTBEG_VISIBLE),
        ("ECL_3RD_VISIBLE", ECL_3RD_VISIBLE),
        ("ECL_TOTEND_VISIBLE", ECL_TOTEND_VISIBLE),
        ("ECL_4TH_VISIBLE", ECL_4TH_VISIBLE),
        ("ECL_PARTEND_VISIBLE", ECL_PARTEND_VISIBLE),
        ("ECL_PENUMBBEG_VISIBLE", ECL_PENUMBBEG_VISIBLE),
        ("ECL_PENUMBEND_VISIBLE", ECL_PENUMBEND_VISIBLE),
        ("ECL_OCC_BEG_DAYLIGHT", ECL_OCC_BEG_DAYLIGHT),
        ("ECL_OCC_END_DAYLIGHT", ECL_OCC_END_DAYLIGHT),
        ("ECL_ONE_TRY", ECL_ONE_TRY),
    ];
    for (name, val) in rise_and_eclipse {
        m.add(*name, *val)?;
        m.add(format!("SE_{name}").as_str(), *val)?;
    }

    Ok(())
}
