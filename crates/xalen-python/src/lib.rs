use pyo3::prelude::*;
use pyo3::types::{PyDict, PyDictMethods, PyListMethods};

use xalen_ayanamsa::Ayanamsa;
use xalen_coords::obliquity::mean_obliquity;
use xalen_ephem::{Almanac, Body};
use xalen_houses::{GeoLocation, HouseSystem, compute_houses};
use xalen_time::{DeltaTModel, JdUT1, JulianDay};
use xalen_vedic::nakshatra::Nakshatra;
use xalen_vedic::panchang::compute_panchang;
use xalen_vedic::rashi::Rashi;

// ---------------------------------------------------------------------------
// ID-to-enum converters (mirrors xalen-wasm)
// ---------------------------------------------------------------------------

/// Canonical body ID mapping (shared across all bindings).
/// 0=Sun, 1=Moon, 2=Mercury, 3=Venus, 4=Mars, 5=Jupiter, 6=Saturn,
/// 7=Uranus, 8=Neptune, 9=Rahu/MeanNode, 10=TrueNode, 11=Pluto,
/// 12=Chiron, 13=Ketu (computed as Rahu+180, handled by callers)
fn body_from_id(id: u8) -> PyResult<Body> {
    match id {
        0 => Ok(Body::Sun),
        1 => Ok(Body::Moon),
        2 => Ok(Body::Mercury),
        3 => Ok(Body::Venus),
        4 => Ok(Body::Mars),
        5 => Ok(Body::Jupiter),
        6 => Ok(Body::Saturn),
        7 => Ok(Body::Uranus),
        8 => Ok(Body::Neptune),
        9 => Ok(Body::MeanNode),
        10 => Ok(Body::TrueNode),
        11 => Ok(Body::Pluto),
        12 => Ok(Body::Chiron),
        // 13 = Ketu handled specially by callers
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "invalid body id: {id}. Valid: 0=Sun, 1=Moon, 2=Mercury, 3=Venus, \
             4=Mars, 5=Jupiter, 6=Saturn, 7=Uranus, 8=Neptune, 9=MeanNode(Rahu), \
             10=TrueNode, 11=Pluto, 12=Chiron, 13=Ketu"
        ))),
    }
}

/// Canonical ayanamsa ID mapping (shared across all bindings).
/// 0=Lahiri, 1=KP, 2=Raman, 3=FaganBradley, 4=TrueChitra, 5=TrueRevati,
/// 6=SuryaSiddhanta, 7=Yukteswar, 8=JNBhasin, 9=DeLuce, 10=Ushashashi,
/// 11=PushyaPaksha, 12=GalacticCenter, 13=LahiriICRC, 14=KPStraightLine,
/// 15=Hipparchos, 16=LahiriVP285
fn ayanamsa_from_id(id: u8) -> PyResult<Ayanamsa> {
    match id {
        0 => Ok(Ayanamsa::Lahiri),
        1 => Ok(Ayanamsa::KPKrishnamurti),
        2 => Ok(Ayanamsa::Raman),
        3 => Ok(Ayanamsa::FaganBradley),
        4 => Ok(Ayanamsa::TrueChitra),
        5 => Ok(Ayanamsa::TrueRevati),
        6 => Ok(Ayanamsa::SuryaSiddhanta),
        7 => Ok(Ayanamsa::YukteswarSriSS),
        8 => Ok(Ayanamsa::JNBhasin),
        9 => Ok(Ayanamsa::DeLuce),
        10 => Ok(Ayanamsa::Ushashashi),
        11 => Ok(Ayanamsa::PushyaPaksha),
        12 => Ok(Ayanamsa::GalacticCenter0Sag),
        13 => Ok(Ayanamsa::LahiriICRC),
        14 => Ok(Ayanamsa::KPStraightLine),
        15 => Ok(Ayanamsa::Hipparchos),
        16 => Ok(Ayanamsa::LahiriVP285),
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "invalid ayanamsa id: {id}. Valid: 0=Lahiri, 1=KP, 2=Raman, \
             3=FaganBradley, 4=TrueChitra, 5=TrueRevati, 6=SuryaSiddhanta, \
             7=Yukteswar, 8=JNBhasin, 9=DeLuce, 10=Ushashashi, 11=PushyaPaksha, \
             12=GalacticCenter, 13=LahiriICRC, 14=KPStraightLine, 15=Hipparchos, \
             16=LahiriVP285"
        ))),
    }
}

/// Canonical house system ID mapping (shared across all bindings).
/// 0=WholeSign, 1=Equal, 2=Placidus, 3=Koch, 4=Porphyry, 5=Regiomontanus,
/// 6=Campanus, 7=Morinus, 8=Alcabitius, 9=Topocentric, 10=Sripati,
/// 11=Vehlow, 12=Meridian, 13=Krusinski
fn house_system_from_id(id: u8) -> PyResult<HouseSystem> {
    match id {
        0 => Ok(HouseSystem::WholeSign),
        1 => Ok(HouseSystem::Equal),
        2 => Ok(HouseSystem::Placidus),
        3 => Ok(HouseSystem::Koch),
        4 => Ok(HouseSystem::Porphyry),
        5 => Ok(HouseSystem::Regiomontanus),
        6 => Ok(HouseSystem::Campanus),
        7 => Ok(HouseSystem::Morinus),
        8 => Ok(HouseSystem::Alcabitius),
        9 => Ok(HouseSystem::Topocentric),
        10 => Ok(HouseSystem::Sripati),
        11 => Ok(HouseSystem::Vehlow),
        12 => Ok(HouseSystem::Meridian),
        13 => Ok(HouseSystem::KrusinskiPisa),
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "invalid house system id: {id}. Valid: 0=WholeSign, 1=Equal, 2=Placidus, \
             3=Koch, 4=Porphyry, 5=Regiomontanus, 6=Campanus, 7=Morinus, \
             8=Alcabitius, 9=Topocentric, 10=Sripati, 11=Vehlow, 12=Meridian, \
             13=Krusinski"
        ))),
    }
}

/// Compute the ayanamsa value in degrees for a given JD, converting UT1 to TT
/// internally via the SMH2016 delta-T model.
fn compute_ayanamsa_deg(jd_ut1: f64, aya: &Ayanamsa) -> f64 {
    let tt = JdUT1(jd_ut1).to_tt(&DeltaTModel::StephensonMorrisonHohenkerk2016);
    aya.compute_deg(tt.as_f64())
}

// ---------------------------------------------------------------------------
// String-based helpers (kept for ergonomic string overloads)
// ---------------------------------------------------------------------------

fn parse_body(name: &str) -> PyResult<Body> {
    match name.to_ascii_lowercase().as_str() {
        "sun" => Ok(Body::Sun),
        "moon" => Ok(Body::Moon),
        "mercury" => Ok(Body::Mercury),
        "venus" => Ok(Body::Venus),
        "earth" => Ok(Body::Earth),
        "mars" => Ok(Body::Mars),
        "jupiter" => Ok(Body::Jupiter),
        "saturn" => Ok(Body::Saturn),
        "uranus" => Ok(Body::Uranus),
        "neptune" => Ok(Body::Neptune),
        "pluto" => Ok(Body::Pluto),
        "rahu" | "meannode" | "mean_node" => Ok(Body::MeanNode),
        "truenode" | "true_node" => Ok(Body::TrueNode),
        "chiron" => Ok(Body::Chiron),
        "lilith" | "meanapogee" | "mean_apogee" => Ok(Body::MeanApogee),
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "unknown body: {name}"
        ))),
    }
}

fn parse_ayanamsa(name: &str) -> PyResult<Ayanamsa> {
    match name.to_ascii_lowercase().replace(['-', ' '], "").as_str() {
        "lahiri" => Ok(Ayanamsa::Lahiri),
        "kp" | "krishnamurti" | "kpkrishnamurti" => Ok(Ayanamsa::KPKrishnamurti),
        "raman" => Ok(Ayanamsa::Raman),
        "faganbradley" => Ok(Ayanamsa::FaganBradley),
        "truechitra" | "chitrapaksha" => Ok(Ayanamsa::TrueChitra),
        "truerevati" => Ok(Ayanamsa::TrueRevati),
        "suryasiddhanta" => Ok(Ayanamsa::SuryaSiddhanta),
        "yukteswar" => Ok(Ayanamsa::YukteswarSriSS),
        "jnbhasin" => Ok(Ayanamsa::JNBhasin),
        "deluce" => Ok(Ayanamsa::DeLuce),
        "ushashashi" => Ok(Ayanamsa::Ushashashi),
        "pushyapaksha" => Ok(Ayanamsa::PushyaPaksha),
        "galacticcenter" | "galacticcenter0sag" => Ok(Ayanamsa::GalacticCenter0Sag),
        "lahiriicrc" => Ok(Ayanamsa::LahiriICRC),
        "kpstraightline" => Ok(Ayanamsa::KPStraightLine),
        "hipparchos" => Ok(Ayanamsa::Hipparchos),
        "lahirivp285" => Ok(Ayanamsa::LahiriVP285),
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "unknown ayanamsa: {name}"
        ))),
    }
}

fn parse_house_system(name: &str) -> PyResult<HouseSystem> {
    match name.to_ascii_lowercase().replace(['-', ' '], "").as_str() {
        "wholesign" => Ok(HouseSystem::WholeSign),
        "equal" => Ok(HouseSystem::Equal),
        "placidus" => Ok(HouseSystem::Placidus),
        "koch" => Ok(HouseSystem::Koch),
        "porphyry" => Ok(HouseSystem::Porphyry),
        "regiomontanus" => Ok(HouseSystem::Regiomontanus),
        "campanus" => Ok(HouseSystem::Campanus),
        "morinus" => Ok(HouseSystem::Morinus),
        "alcabitius" => Ok(HouseSystem::Alcabitius),
        "topocentric" => Ok(HouseSystem::Topocentric),
        "meridian" => Ok(HouseSystem::Meridian),
        "vehlow" => Ok(HouseSystem::Vehlow),
        "sripati" => Ok(HouseSystem::Sripati),
        "krusinskipisa" => Ok(HouseSystem::KrusinskiPisa),
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "unknown house system: {name}"
        ))),
    }
}

fn parse_numerology_system(name: &str) -> PyResult<xalen_numerology::System> {
    match name.to_ascii_lowercase().as_str() {
        "pythagorean" => Ok(xalen_numerology::System::Pythagorean),
        "chaldean" => Ok(xalen_numerology::System::Chaldean),
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "unknown numerology system: {name} (expected 'pythagorean' or 'chaldean')"
        ))),
    }
}

// ---------------------------------------------------------------------------
// Core ephemeris functions
// ---------------------------------------------------------------------------

/// Compute the ecliptic longitude of a celestial body in degrees [0, 360).
///
/// Parameters
/// ----------
/// jd : float
///     Julian Day (UT1).
/// body : int
///     Body ID: 0=Sun, 1=Moon, 2=Mercury, 3=Venus, 4=Mars, 5=Jupiter,
///     6=Saturn, 7=Uranus, 8=Neptune, 9=MeanNode(Rahu), 10=TrueNode,
///     11=Pluto, 12=Chiron, 13=Ketu (Rahu+180).
/// sidereal : bool, optional
///     If True, return sidereal longitude (default: False = tropical).
/// ayanamsa : int, optional
///     Ayanamsa system ID (0=Lahiri, 1=KP, ...). Only used when sidereal=True.
///
/// Returns
/// -------
/// float
///     Longitude in degrees [0, 360).
#[pyfunction]
#[pyo3(signature = (jd, body, sidereal = false, ayanamsa = 0))]
fn planet_longitude(jd: f64, body: u8, sidereal: bool, ayanamsa: u8) -> PyResult<f64> {
    let almanac = Almanac::default_vedic();
    let jd_ut1 = JdUT1(jd);

    // Handle Ketu (id 13) as Rahu + 180
    if body == 13 {
        let rahu_lon = if sidereal {
            let aya = ayanamsa_from_id(ayanamsa)?;
            let aya_deg = compute_ayanamsa_deg(jd, &aya);
            almanac.sidereal_longitude_deg(Body::MeanNode, jd_ut1, aya_deg)
        } else {
            almanac.geocentric_longitude_deg(Body::MeanNode, jd_ut1)
        }
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        return Ok((rahu_lon + 180.0).rem_euclid(360.0));
    }

    let b = body_from_id(body)?;

    if sidereal {
        let aya = ayanamsa_from_id(ayanamsa)?;
        let aya_deg = compute_ayanamsa_deg(jd, &aya);
        almanac
            .sidereal_longitude_deg(b, jd_ut1, aya_deg)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    } else {
        almanac
            .geocentric_longitude_deg(b, jd_ut1)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }
}

/// Compute longitudes for all major planets at once.
///
/// Returns a dict mapping body name to longitude in degrees.
/// Bodies computed: Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn,
/// Uranus, Neptune, Pluto, Rahu (Mean Node).
///
/// Parameters
/// ----------
/// jd : float
///     Julian Day (UT1).
/// sidereal : bool, optional
///     If True, return sidereal longitudes (default: False = tropical).
/// ayanamsa : int, optional
///     Ayanamsa system ID. Only used when sidereal=True.
///
/// Returns
/// -------
/// dict
///     ``{"Sun": 280.5, "Moon": 120.3, ...}``
#[pyfunction]
#[pyo3(signature = (jd, sidereal = false, ayanamsa = 0))]
fn all_planets(py: Python<'_>, jd: f64, sidereal: bool, ayanamsa: u8) -> PyResult<Py<PyAny>> {
    let almanac = Almanac::default_vedic();
    let jd_ut1 = JdUT1(jd);
    let aya_deg = if sidereal {
        let aya = ayanamsa_from_id(ayanamsa)?;
        compute_ayanamsa_deg(jd, &aya)
    } else {
        0.0
    };

    let dict = PyDict::new(py);
    // All major bodies including outer planets and nodes
    let bodies: &[(Body, &str)] = &[
        (Body::Sun, "Sun"),
        (Body::Moon, "Moon"),
        (Body::Mercury, "Mercury"),
        (Body::Venus, "Venus"),
        (Body::Mars, "Mars"),
        (Body::Jupiter, "Jupiter"),
        (Body::Saturn, "Saturn"),
        (Body::Uranus, "Uranus"),
        (Body::Neptune, "Neptune"),
        (Body::Pluto, "Pluto"),
        (Body::MeanNode, "Rahu"),
    ];

    for &(body, name) in bodies {
        let lon = if sidereal {
            almanac.sidereal_longitude_deg(body, jd_ut1, aya_deg)
        } else {
            almanac.geocentric_longitude_deg(body, jd_ut1)
        };
        match lon {
            Ok(deg) => {
                dict.set_item(name, deg)?;
            }
            Err(e) => {
                dict.set_item(name, format!("error: {e}"))?;
            }
        }
    }

    // Add Ketu (opposite of Rahu)
    if let Ok(rahu_lon) = if sidereal {
        almanac.sidereal_longitude_deg(Body::MeanNode, jd_ut1, aya_deg)
    } else {
        almanac.geocentric_longitude_deg(Body::MeanNode, jd_ut1)
    } {
        dict.set_item("Ketu", (rahu_lon + 180.0).rem_euclid(360.0))?;
    }

    Ok(dict.into())
}

/// Convert a calendar date to a Julian Day number.
///
/// Parameters
/// ----------
/// year : int
///     Calendar year (negative for BCE).
/// month : int
///     Month (1-12).
/// day : int
///     Day of month (1-31).
/// hour : float, optional
///     Fractional hour of day, UT (default: 12.0 = noon).
///
/// Returns
/// -------
/// float
///     Julian Day number (UT1).
#[pyfunction]
#[pyo3(signature = (year, month, day, hour = 12.0))]
fn julian_day(year: i32, month: u32, day: u32, hour: f64) -> f64 {
    xalen_time::calendar_to_jd(
        year,
        month,
        day,
        hour,
        xalen_time::CalendarSystem::ProlepticGregorian,
    )
    .as_f64()
}

// ---------------------------------------------------------------------------
// Vedic functions
// ---------------------------------------------------------------------------

/// Determine the Nakshatra (lunar mansion) for a given sidereal Moon longitude.
///
/// Parameters
/// ----------
/// moon_sidereal_deg : float
///     Sidereal longitude of the Moon in degrees.
///
/// Returns
/// -------
/// dict
///     ``{"name": "Ashwini", "pada": 1, "lord": "Ketu",
///        "deity": "Ashwini Kumaras", "index": 0}``
#[pyfunction]
fn nakshatra(py: Python<'_>, moon_sidereal_deg: f64) -> PyResult<Py<PyAny>> {
    let nak = Nakshatra::from_longitude_deg(moon_sidereal_deg);
    let pada = Nakshatra::pada(moon_sidereal_deg);
    let lord = nak.lord();

    let dict = PyDict::new(py);
    dict.set_item("name", nak.to_string())?;
    dict.set_item("pada", pada)?;
    dict.set_item("lord", lord.to_string())?;
    dict.set_item("deity", nak.deity())?;
    dict.set_item("index", nak.index())?;
    Ok(dict.into())
}

/// Compute the five Panchang elements for a given Julian Day.
///
/// Internally computes sidereal Sun and Moon longitudes using the specified
/// ayanamsa, then derives Tithi, Nakshatra, Yoga, Karana, and Vara.
///
/// Parameters
/// ----------
/// jd : float
///     Julian Day (UT1).
/// ayanamsa : int, optional
///     Ayanamsa system ID (default: 0 = Lahiri).
///
/// Returns
/// -------
/// dict
///     ``{"tithi": {"number": 5, "name": "Panchami", "paksha": "Shukla"},
///        "nakshatra": "Rohini", "yoga": {"number": 3, "name": "..."},
///        "karana": "...", "vara": "..."}``
#[pyfunction]
#[pyo3(signature = (jd, ayanamsa = 0))]
fn panchang(py: Python<'_>, jd: f64, ayanamsa: u8) -> PyResult<Py<PyAny>> {
    let almanac = Almanac::default_vedic();
    let jd_ut1 = JdUT1(jd);
    let aya = ayanamsa_from_id(ayanamsa)?;
    let aya_deg = compute_ayanamsa_deg(jd, &aya);

    let sun_deg = almanac
        .sidereal_longitude_deg(Body::Sun, jd_ut1, aya_deg)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let moon_deg = almanac
        .sidereal_longitude_deg(Body::Moon, jd_ut1, aya_deg)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

    let p = compute_panchang(sun_deg, moon_deg, jd);

    let dict = PyDict::new(py);

    // Tithi sub-dict
    let tithi_dict = PyDict::new(py);
    tithi_dict.set_item("number", p.tithi.number)?;
    tithi_dict.set_item("name", p.tithi.name())?;
    tithi_dict.set_item(
        "paksha",
        match p.tithi.paksha {
            xalen_vedic::panchang::Paksha::Shukla => "Shukla",
            xalen_vedic::panchang::Paksha::Krishna => "Krishna",
        },
    )?;
    dict.set_item("tithi", tithi_dict)?;

    dict.set_item("nakshatra", p.nakshatra.to_string())?;

    // Yoga sub-dict
    let yoga_dict = PyDict::new(py);
    yoga_dict.set_item("number", p.yoga.number)?;
    yoga_dict.set_item("name", p.yoga.name())?;
    dict.set_item("yoga", yoga_dict)?;

    dict.set_item("karana", p.karana.name())?;
    dict.set_item("vara", p.vara.name())?;

    Ok(dict.into())
}

/// Determine the Rashi (sidereal zodiac sign) for a given sidereal longitude.
///
/// Parameters
/// ----------
/// sidereal_deg : float
///     Sidereal longitude in degrees.
///
/// Returns
/// -------
/// str
///     Rashi name, e.g. ``"Mesha (Aries)"``.
#[pyfunction]
fn rashi(sidereal_deg: f64) -> String {
    Rashi::from_longitude_deg(sidereal_deg).to_string()
}

// ---------------------------------------------------------------------------
// Houses
// ---------------------------------------------------------------------------

/// Compute house cusps and chart angles.
///
/// Parameters
/// ----------
/// jd : float
///     Julian Day (UT1).
/// lat : float
///     Geographic latitude in degrees (positive = North).
/// lon : float
///     Geographic longitude in degrees (positive = East).
/// system : int, optional
///     House system ID (default: 0 = Whole Sign).
///     0=WholeSign, 1=Equal, 2=Placidus, 3=Koch, 4=Porphyry, 5=Regiomontanus,
///     6=Campanus, 7=Morinus, 8=Alcabitius, 9=Topocentric, 10=Sripati,
///     11=Vehlow, 12=Meridian, 13=KrusinskiPisa.
///
/// Returns
/// -------
/// dict
///     ``{"cusps": [float; 12], "ascendant": float, "mc": float,
///        "ic": float, "descendant": float, "vertex": float}``
///     All values in degrees [0, 360).
#[pyfunction]
#[pyo3(signature = (jd, lat, lon, system = 0))]
fn houses(py: Python<'_>, jd: f64, lat: f64, lon: f64, system: u8) -> PyResult<Py<PyAny>> {
    let sys = house_system_from_id(system)?;
    let loc = GeoLocation::new(lat, lon);
    let t = (jd - 2_451_545.0) / 36525.0;
    let epsilon = mean_obliquity(t);
    let h = compute_houses(jd, &loc, epsilon, sys);

    let dict = PyDict::new(py);
    let cusps: Vec<f64> = h
        .cusps
        .iter()
        .map(|c| c.to_degrees().rem_euclid(360.0))
        .collect();
    dict.set_item("cusps", cusps)?;
    dict.set_item("ascendant", h.ascendant.to_degrees().rem_euclid(360.0))?;
    dict.set_item("mc", h.mc.to_degrees().rem_euclid(360.0))?;
    dict.set_item("ic", h.ic.to_degrees().rem_euclid(360.0))?;
    dict.set_item("descendant", h.descendant.to_degrees().rem_euclid(360.0))?;
    dict.set_item("vertex", h.vertex.to_degrees().rem_euclid(360.0))?;
    Ok(dict.into())
}

// ---------------------------------------------------------------------------
// Ayanamsa
// ---------------------------------------------------------------------------

/// Compute the ayanamsa (precession correction) value in degrees.
///
/// Parameters
/// ----------
/// jd : float
///     Julian Day (UT1).
/// system : int, optional
///     Ayanamsa system ID (default: 0 = Lahiri).
///     0=Lahiri, 1=KP/Krishnamurti, 2=Raman, 3=FaganBradley,
///     4=TrueChitra, 5=TrueRevati, 6=SuryaSiddhanta, 7=Yukteswar,
///     8=JNBhasin, 9=DeLuce, 10=Ushashashi, 11=PushyaPaksha,
///     12=GalacticCenter, 13=LahiriICRC, 14=KPStraightLine,
///     15=Hipparchos, 16=LahiriVP285.
///
/// Returns
/// -------
/// float
///     Ayanamsa value in degrees.
#[pyfunction]
#[pyo3(signature = (jd, system = 0))]
fn ayanamsa(jd: f64, system: u8) -> PyResult<f64> {
    let aya = ayanamsa_from_id(system)?;
    Ok(compute_ayanamsa_deg(jd, &aya))
}

// ---------------------------------------------------------------------------
// Full chart
// ---------------------------------------------------------------------------

/// Compute a complete astrological chart (planets + houses + angles).
///
/// Returns sidereal planet positions (with nakshatra, rashi, pada for each),
/// house cusps (Whole Sign), ascendant, MC, and the ayanamsa used.
///
/// Parameters
/// ----------
/// jd : float
///     Julian Day (UT1).
/// lat : float
///     Geographic latitude in degrees.
/// lon : float
///     Geographic longitude in degrees.
/// ayanamsa : int, optional
///     Ayanamsa system ID (default: 0 = Lahiri).
///
/// Returns
/// -------
/// dict
///     ``{"planets": {"Sun": {"longitude": 280.5, "nakshatra": "Shravana",
///        "pada": 2, "rashi": "Makara (Capricorn)"}, ...},
///        "ascendant": 123.4, "mc": 45.6, "ayanamsa_deg": 24.1,
///        "cusps": [float; 12]}``
#[pyfunction]
#[pyo3(signature = (jd, lat, lon, ayanamsa = 0))]
fn full_chart(py: Python<'_>, jd: f64, lat: f64, lon: f64, ayanamsa: u8) -> PyResult<Py<PyAny>> {
    let almanac = Almanac::default_vedic();
    let jd_ut1 = JdUT1(jd);
    let aya = ayanamsa_from_id(ayanamsa)?;
    let aya_deg = compute_ayanamsa_deg(jd, &aya);

    // Planets
    let planets_dict = PyDict::new(py);
    for &body in Body::VEDIC_GRAHAS {
        match almanac.sidereal_longitude_deg(body, jd_ut1, aya_deg) {
            Ok(sid_lon) => {
                let info = PyDict::new(py);
                info.set_item("longitude", sid_lon)?;
                info.set_item(
                    "nakshatra",
                    Nakshatra::from_longitude_deg(sid_lon).to_string(),
                )?;
                info.set_item("pada", Nakshatra::pada(sid_lon))?;
                info.set_item("rashi", Rashi::from_longitude_deg(sid_lon).to_string())?;
                info.set_item(
                    "lord",
                    Nakshatra::from_longitude_deg(sid_lon).lord().to_string(),
                )?;
                planets_dict.set_item(body.to_string(), info)?;
            }
            Err(e) => {
                planets_dict.set_item(body.to_string(), format!("error: {e}"))?;
            }
        }
    }

    // Add Ketu (opposite of Rahu/MeanNode)
    if let Ok(rahu_lon) = almanac.sidereal_longitude_deg(Body::MeanNode, jd_ut1, aya_deg) {
        let ketu_lon = (rahu_lon + 180.0).rem_euclid(360.0);
        let info = PyDict::new(py);
        info.set_item("longitude", ketu_lon)?;
        info.set_item(
            "nakshatra",
            Nakshatra::from_longitude_deg(ketu_lon).to_string(),
        )?;
        info.set_item("pada", Nakshatra::pada(ketu_lon))?;
        info.set_item("rashi", Rashi::from_longitude_deg(ketu_lon).to_string())?;
        info.set_item(
            "lord",
            Nakshatra::from_longitude_deg(ketu_lon).lord().to_string(),
        )?;
        planets_dict.set_item("Ketu", info)?;
    }

    // Houses (Whole Sign default)
    let loc = GeoLocation::new(lat, lon);
    let t = (jd - 2_451_545.0) / 36525.0;
    let epsilon = mean_obliquity(t);
    let h = compute_houses(jd, &loc, epsilon, HouseSystem::WholeSign);

    let cusps: Vec<f64> = h
        .cusps
        .iter()
        .map(|c| c.to_degrees().rem_euclid(360.0))
        .collect();

    let chart = PyDict::new(py);
    chart.set_item("planets", planets_dict)?;
    chart.set_item("ascendant", h.ascendant.to_degrees().rem_euclid(360.0))?;
    chart.set_item("mc", h.mc.to_degrees().rem_euclid(360.0))?;
    chart.set_item("ayanamsa_deg", aya_deg)?;
    chart.set_item("cusps", cusps)?;

    Ok(chart.into())
}

// ---------------------------------------------------------------------------
// Supplementary: delta-T, fixed stars, numerology
// ---------------------------------------------------------------------------

/// Delta-T in seconds (TT - UT1) at the given Julian Day.
///
/// Uses the Stephenson-Morrison-Hohenkerk 2016 model.
#[pyfunction]
fn delta_t(jd: f64) -> f64 {
    xalen_time::delta_t(jd, &DeltaTModel::StephensonMorrisonHohenkerk2016)
}

/// Find fixed stars within `orb` degrees of a planet longitude, precessed to
/// a given year.
///
/// Parameters
/// ----------
/// planet_lon : float
///     Planet longitude in degrees.
/// orb : float
///     Orb in degrees for conjunction matching.
/// year : float
///     Decimal year for precession (e.g. 2026.0).
///
/// Returns
/// -------
/// list[dict]
///     Each dict: ``{"name": str, "distance": float, "constellation": str,
///     "magnitude": float, "nature": str}``.
#[pyfunction]
fn fixed_star_conjunctions(
    py: Python<'_>,
    planet_lon: f64,
    orb: f64,
    year: f64,
) -> PyResult<Py<PyAny>> {
    let matches = xalen_stars::find_conjunctions_at_epoch(planet_lon, orb, year);
    let list = pyo3::types::PyList::empty(py);
    for (star, dist) in &matches {
        let d = PyDict::new(py);
        d.set_item("name", star.name)?;
        d.set_item("distance", *dist)?;
        d.set_item("constellation", star.constellation)?;
        d.set_item("magnitude", star.magnitude)?;
        d.set_item("nature", star.nature)?;
        list.append(d)?;
    }
    Ok(list.into())
}

/// Numerology life-path number from birth date.
#[pyfunction]
fn life_path(year: u32, month: u32, day: u32) -> u32 {
    xalen_numerology::life_path(year, month, day)
}

/// Numerology expression number from full name.
///
/// Parameters
/// ----------
/// name : str
///     Full name.
/// system : str
///     ``"pythagorean"`` or ``"chaldean"``.
#[pyfunction]
fn expression_number(name: &str, system: &str) -> PyResult<u32> {
    let sys = parse_numerology_system(system)?;
    Ok(xalen_numerology::expression_number(name, sys))
}

// ---------------------------------------------------------------------------
// String-based convenience wrappers (ergonomic for interactive Python use)
// ---------------------------------------------------------------------------

/// Tropical longitude by body name (string). See ``planet_longitude`` for the
/// primary integer-based API.
#[pyfunction]
fn planet_longitude_by_name(body: &str, jd_ut1: f64) -> PyResult<f64> {
    let b = parse_body(body)?;
    let almanac = Almanac::default_vedic();
    almanac
        .geocentric_longitude_deg(b, JdUT1(jd_ut1))
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

/// Sidereal longitude by body and ayanamsa name (strings).
#[pyfunction]
fn sidereal_longitude(body: &str, jd_ut1: f64, ayanamsa: &str) -> PyResult<f64> {
    let b = parse_body(body)?;
    let aya = parse_ayanamsa(ayanamsa)?;
    let almanac = Almanac::default_vedic();
    let aya_deg = compute_ayanamsa_deg(jd_ut1, &aya);
    almanac
        .sidereal_longitude_deg(b, JdUT1(jd_ut1), aya_deg)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

/// Compute house cusps by house system name (string). See ``houses`` for the
/// primary integer-based API.
#[pyfunction]
fn houses_by_name(
    py: Python<'_>,
    jd: f64,
    lat: f64,
    lon: f64,
    system: &str,
) -> PyResult<Py<PyAny>> {
    let sys = parse_house_system(system)?;
    let loc = GeoLocation::new(lat, lon);
    let t = (jd - 2_451_545.0) / 36525.0;
    let epsilon = mean_obliquity(t);
    let h = compute_houses(jd, &loc, epsilon, sys);

    let dict = PyDict::new(py);
    let cusps: Vec<f64> = h
        .cusps
        .iter()
        .map(|c| c.to_degrees().rem_euclid(360.0))
        .collect();
    dict.set_item("cusps", cusps)?;
    dict.set_item("ascendant", h.ascendant.to_degrees().rem_euclid(360.0))?;
    dict.set_item("mc", h.mc.to_degrees().rem_euclid(360.0))?;
    dict.set_item("ic", h.ic.to_degrees().rem_euclid(360.0))?;
    dict.set_item("descendant", h.descendant.to_degrees().rem_euclid(360.0))?;
    dict.set_item("vertex", h.vertex.to_degrees().rem_euclid(360.0))?;
    Ok(dict.into())
}

/// Ayanamsa by system name (string). See ``ayanamsa`` for the primary
/// integer-based API.
#[pyfunction]
fn ayanamsa_by_name(jd: f64, system: &str) -> PyResult<f64> {
    let aya = parse_ayanamsa(system)?;
    Ok(compute_ayanamsa_deg(jd, &aya))
}

// ---------------------------------------------------------------------------
// Python module
// ---------------------------------------------------------------------------

/// XALEN Ephemeris --- pure-Rust astronomical library for Python.
///
/// Provides high-precision planetary positions (VSOP87), Vedic astrology
/// computations (nakshatra, panchang, rashi), house cusps (14 systems),
/// ayanamsa (8+ systems), fixed stars, and numerology.
///
/// Quick start::
///
///     import xalen
///     jd = xalen.julian_day(1990, 6, 15, 10.5)
///     chart = xalen.full_chart(jd, 18.52, 73.85)
///     print(chart["planets"]["Sun"])
#[pymodule]
fn xalen(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Core ephemeris
    m.add_function(wrap_pyfunction!(planet_longitude, m)?)?;
    m.add_function(wrap_pyfunction!(all_planets, m)?)?;
    m.add_function(wrap_pyfunction!(julian_day, m)?)?;

    // Vedic
    m.add_function(wrap_pyfunction!(nakshatra, m)?)?;
    m.add_function(wrap_pyfunction!(panchang, m)?)?;
    m.add_function(wrap_pyfunction!(rashi, m)?)?;

    // Houses
    m.add_function(wrap_pyfunction!(houses, m)?)?;

    // Ayanamsa
    m.add_function(wrap_pyfunction!(ayanamsa, m)?)?;

    // Full chart
    m.add_function(wrap_pyfunction!(full_chart, m)?)?;

    // Supplementary
    m.add_function(wrap_pyfunction!(delta_t, m)?)?;
    m.add_function(wrap_pyfunction!(fixed_star_conjunctions, m)?)?;
    m.add_function(wrap_pyfunction!(life_path, m)?)?;
    m.add_function(wrap_pyfunction!(expression_number, m)?)?;

    // String-based convenience wrappers
    m.add_function(wrap_pyfunction!(planet_longitude_by_name, m)?)?;
    m.add_function(wrap_pyfunction!(sidereal_longitude, m)?)?;
    m.add_function(wrap_pyfunction!(houses_by_name, m)?)?;
    m.add_function(wrap_pyfunction!(ayanamsa_by_name, m)?)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_body_from_id_valid() {
        assert!(body_from_id(0).is_ok()); // Sun
        assert!(body_from_id(1).is_ok()); // Moon
        assert!(body_from_id(9).is_ok()); // MeanNode/Rahu
        assert!(body_from_id(10).is_ok()); // TrueNode
        assert!(body_from_id(11).is_ok()); // Pluto
        assert!(body_from_id(12).is_ok()); // Chiron
    }

    #[test]
    fn test_body_from_id_canonical_order() {
        // Verify canonical mapping matches spec
        assert_eq!(body_from_id(0).unwrap(), Body::Sun);
        assert_eq!(body_from_id(9).unwrap(), Body::MeanNode);
        assert_eq!(body_from_id(10).unwrap(), Body::TrueNode);
        assert_eq!(body_from_id(11).unwrap(), Body::Pluto);
        assert_eq!(body_from_id(12).unwrap(), Body::Chiron);
    }

    #[test]
    fn test_body_from_id_invalid() {
        assert!(body_from_id(99).is_err());
        // 13 is Ketu (handled by callers, not body_from_id)
        assert!(body_from_id(14).is_err());
    }

    #[test]
    fn test_ayanamsa_from_id_errors_on_invalid() {
        assert!(ayanamsa_from_id(255).is_err());
        assert!(ayanamsa_from_id(17).is_err());
    }

    #[test]
    fn test_ayanamsa_from_id_all_valid() {
        for id in 0..=16u8 {
            assert!(
                ayanamsa_from_id(id).is_ok(),
                "Ayanamsa ID {id} should be valid"
            );
        }
    }

    #[test]
    fn test_house_system_from_id_errors_on_invalid() {
        assert!(house_system_from_id(255).is_err());
        assert!(house_system_from_id(14).is_err());
    }

    #[test]
    fn test_house_system_from_id_canonical_order() {
        assert_eq!(house_system_from_id(0).unwrap(), HouseSystem::WholeSign);
        assert_eq!(house_system_from_id(3).unwrap(), HouseSystem::Koch);
        assert_eq!(house_system_from_id(4).unwrap(), HouseSystem::Porphyry);
        assert_eq!(house_system_from_id(9).unwrap(), HouseSystem::Topocentric);
        assert_eq!(
            house_system_from_id(13).unwrap(),
            HouseSystem::KrusinskiPisa
        );
    }

    #[test]
    fn test_house_system_from_id_all_valid() {
        for id in 0..=13u8 {
            assert!(
                house_system_from_id(id).is_ok(),
                "House system ID {id} should be valid"
            );
        }
    }

    #[test]
    fn test_julian_day_j2000() {
        let jd = julian_day(2000, 1, 1, 12.0);
        assert!(
            (jd - 2451545.0).abs() < 0.01,
            "J2000 should be 2451545.0, got {jd}"
        );
    }

    #[test]
    fn test_rashi_aries() {
        let r = rashi(5.0);
        assert!(r.contains("Mesha"), "5 deg should be Mesha, got: {r}");
    }

    #[test]
    fn test_rashi_pisces() {
        let r = rashi(355.0);
        assert!(r.contains("Meena"), "355 deg should be Meena, got: {r}");
    }

    #[test]
    fn test_delta_t_positive() {
        let dt = delta_t(2451545.0);
        assert!(dt > 0.0, "Delta-T at J2000 should be positive, got {dt}");
    }

    #[test]
    fn test_parse_body_string() {
        assert!(parse_body("sun").is_ok());
        assert!(parse_body("Moon").is_ok());
        assert!(parse_body("rahu").is_ok());
        assert!(parse_body("truenode").is_ok());
        assert!(parse_body("invalid").is_err());
    }

    #[test]
    fn test_parse_ayanamsa_string() {
        assert!(parse_ayanamsa("lahiri").is_ok());
        assert!(parse_ayanamsa("kp").is_ok());
        assert!(parse_ayanamsa("KP Krishnamurti").is_ok());
        assert!(parse_ayanamsa("bogus").is_err());
    }

    #[test]
    fn test_parse_house_system_string() {
        assert!(parse_house_system("placidus").is_ok());
        assert!(parse_house_system("Whole Sign").is_ok());
        assert!(parse_house_system("fake").is_err());
    }

    #[test]
    fn test_compute_ayanamsa_deg_lahiri_j2000() {
        let deg = compute_ayanamsa_deg(2451545.0, &Ayanamsa::Lahiri);
        // Lahiri at J2000 is around 23.85 degrees
        assert!(
            deg > 23.0 && deg < 25.0,
            "Lahiri at J2000 should be ~23.85, got {deg}"
        );
    }

    #[test]
    fn test_life_path_basic() {
        // 1990-06-15 => 1+9+9+0+6+1+5 = 31 => 3+1 = 4
        let lp = life_path(1990, 6, 15);
        assert!(
            lp >= 1 && lp <= 9 || lp == 11 || lp == 22 || lp == 33,
            "Life path should be a valid number, got {lp}"
        );
    }

    #[test]
    fn test_expression_number_basic() {
        let en = expression_number("John Doe", "pythagorean").unwrap();
        assert!(
            en >= 1 && en <= 9 || en == 11 || en == 22 || en == 33,
            "Expression number should be valid, got {en}"
        );
    }

    #[test]
    fn test_expression_number_invalid_system() {
        assert!(expression_number("Test", "invalid").is_err());
    }

    #[test]
    fn test_all_ayanamsa_ids_finite() {
        for id in 0..=16u8 {
            let aya = ayanamsa_from_id(id).unwrap();
            let deg = compute_ayanamsa_deg(2451545.0, &aya);
            assert!(deg.is_finite(), "Ayanamsa {id} should be finite, got {deg}");
        }
    }

    #[test]
    fn test_all_body_ids_computable() {
        let almanac = Almanac::default_vedic();
        let jd = JdUT1(2451545.0);
        for id in 0..=12u8 {
            let body = body_from_id(id).unwrap();
            let result = almanac.geocentric_longitude_deg(body, jd);
            assert!(
                result.is_ok(),
                "Body id {id} ({body}) failed: {:?}",
                result.err()
            );
            let lon = result.unwrap();
            assert!(
                lon >= 0.0 && lon < 360.0,
                "Body {id} lon out of range: {lon}"
            );
        }
    }
}
