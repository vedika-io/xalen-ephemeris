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

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyString};

use xalen_ephem::compat;

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
    let xx =
        compat::swe_calc_ut(jd_ut, ipl, flags).map_err(pyo3::exceptions::PyValueError::new_err)?;
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
    if let Ok(b) = hsys.downcast::<PyBytes>() {
        let bytes = b.as_bytes();
        return match bytes.first() {
            Some(&byte) => Ok(byte as char),
            None => Err(pyo3::exceptions::PyValueError::new_err(
                "empty house system code (expected a single byte, e.g. b'P')",
            )),
        };
    }
    // str (e.g. 'P')
    if let Ok(s) = hsys.downcast::<PyString>() {
        let text: String = s.extract()?;
        return text.chars().next().ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(
                "empty house system code (expected a single character, e.g. 'P')",
            )
        });
    }
    Err(pyo3::exceptions::PyValueError::new_err(
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
    let h = compat::swe_houses_ex(jd_ut, lat, lon, hchar, sidereal)
        .map_err(pyo3::exceptions::PyValueError::new_err)?;
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
    let aya = compat::swe_get_ayanamsa_ut_ex(jd_ut, compat::active_sid_mode())
        .map_err(pyo3::exceptions::PyValueError::new_err)?;
    Ok((flags | compat::SEFLG_SWIEPH, aya))
}

/// `swe.get_ayanamsa_ut(jd_ut)` — ayanamsa (degrees) for the active sidereal
/// mode. Returns a bare float, like pyswisseph.
#[pyfunction]
fn get_ayanamsa_ut(jd_ut: f64) -> PyResult<f64> {
    compat::swe_get_ayanamsa_ut_ex(jd_ut, compat::active_sid_mode())
        .map_err(pyo3::exceptions::PyValueError::new_err)
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
    let xx =
        compat::swe_fixstar_ut(&star, jd_ut).map_err(pyo3::exceptions::PyValueError::new_err)?;
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
    let mag = compat::swe_fixstar_mag(&star).map_err(pyo3::exceptions::PyValueError::new_err)?;
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

    Ok(())
}
