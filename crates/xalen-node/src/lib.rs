use napi::bindgen_prelude::*;
use napi_derive::napi;

use xalen_ayanamsa::Ayanamsa;
use xalen_ephem::{Body, EphemerisProvider, Vsop87Provider};
use xalen_houses::{GeoLocation, HouseSystem, compute_houses};
use xalen_time::{DeltaTModel, JulianDay};
use xalen_vedic::nakshatra::Nakshatra;
use xalen_vedic::panchang::compute_panchang;
use xalen_vedic::rashi::Rashi;

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn parse_body(name: &str) -> Result<Body> {
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
        _ => Err(Error::new(
            Status::InvalidArg,
            format!("unknown body: {name}"),
        )),
    }
}

fn parse_ayanamsa(name: &str) -> Result<Ayanamsa> {
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
        _ => Err(Error::new(
            Status::InvalidArg,
            format!("unknown ayanamsa: {name}"),
        )),
    }
}

fn parse_house_system(name: &str) -> Result<HouseSystem> {
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
        _ => Err(Error::new(
            Status::InvalidArg,
            format!("unknown house system: {name}"),
        )),
    }
}

fn parse_numerology_system(name: &str) -> Result<xalen_numerology::System> {
    match name.to_ascii_lowercase().as_str() {
        "pythagorean" => Ok(xalen_numerology::System::Pythagorean),
        "chaldean" => Ok(xalen_numerology::System::Chaldean),
        _ => Err(Error::new(
            Status::InvalidArg,
            format!("unknown numerology system: {name} (expected 'pythagorean' or 'chaldean')"),
        )),
    }
}

// ---------------------------------------------------------------------------
// Canonical integer-ID-to-enum converters (shared across all bindings)
// ---------------------------------------------------------------------------

/// Canonical body ID mapping.
/// 0=Sun, 1=Moon, 2=Mercury, 3=Venus, 4=Mars, 5=Jupiter, 6=Saturn,
/// 7=Uranus, 8=Neptune, 9=Rahu/MeanNode, 10=TrueNode, 11=Pluto,
/// 12=Chiron, 13=Ketu (computed as Rahu+180, handled by callers)
fn body_from_id(id: u8) -> Result<Body> {
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
        _ => Err(Error::new(
            Status::InvalidArg,
            format!(
                "invalid body id: {id}. Valid: 0=Sun, 1=Moon, 2=Mercury, 3=Venus, \
             4=Mars, 5=Jupiter, 6=Saturn, 7=Uranus, 8=Neptune, 9=MeanNode(Rahu), \
             10=TrueNode, 11=Pluto, 12=Chiron, 13=Ketu"
            ),
        )),
    }
}

/// Canonical ayanamsa ID mapping.
/// 0=Lahiri, 1=KP, 2=Raman, 3=FaganBradley, 4=TrueChitra, 5=TrueRevati,
/// 6=SuryaSiddhanta, 7=Yukteswar, 8=JNBhasin, 9=DeLuce, 10=Ushashashi,
/// 11=PushyaPaksha, 12=GalacticCenter, 13=LahiriICRC, 14=KPStraightLine,
/// 15=Hipparchos, 16=LahiriVP285
fn ayanamsa_from_id(id: u8) -> Result<Ayanamsa> {
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
        _ => Err(Error::new(
            Status::InvalidArg,
            format!(
                "invalid ayanamsa id: {id}. Valid: 0=Lahiri, 1=KP, 2=Raman, \
             3=FaganBradley, 4=TrueChitra, 5=TrueRevati, 6=SuryaSiddhanta, \
             7=Yukteswar, 8=JNBhasin, 9=DeLuce, 10=Ushashashi, 11=PushyaPaksha, \
             12=GalacticCenter, 13=LahiriICRC, 14=KPStraightLine, 15=Hipparchos, \
             16=LahiriVP285"
            ),
        )),
    }
}

/// Canonical house system ID mapping.
/// 0=WholeSign, 1=Equal, 2=Placidus, 3=Koch, 4=Porphyry, 5=Regiomontanus,
/// 6=Campanus, 7=Morinus, 8=Alcabitius, 9=Topocentric, 10=Sripati,
/// 11=Vehlow, 12=Meridian, 13=Krusinski
fn house_system_from_id(id: u8) -> Result<HouseSystem> {
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
        _ => Err(Error::new(
            Status::InvalidArg,
            format!(
                "invalid house system id: {id}. Valid: 0=WholeSign, 1=Equal, 2=Placidus, \
             3=Koch, 4=Porphyry, 5=Regiomontanus, 6=Campanus, 7=Morinus, \
             8=Alcabitius, 9=Topocentric, 10=Sripati, 11=Vehlow, 12=Meridian, \
             13=Krusinski"
            ),
        )),
    }
}

// ---------------------------------------------------------------------------
// exported functions (string-based)
// ---------------------------------------------------------------------------

/// Tropical (geometric) ecliptic longitude in degrees for `body` at `jd_ut1`.
#[napi]
pub fn planet_longitude(body: String, jd_ut1: f64) -> Result<f64> {
    let b = parse_body(&body)?;
    let jd_tt = xalen_time::JdUT1(jd_ut1).to_tt(&DeltaTModel::StephensonMorrisonHohenkerk2016);
    let provider = Vsop87Provider::new();
    let pos = provider
        .geocentric_ecliptic(b, jd_tt)
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
    Ok(pos.longitude.to_degrees().rem_euclid(360.0))
}

/// Sidereal ecliptic longitude in degrees for `body` at `jd_ut1` using the
/// named `ayanamsa` system (e.g. "lahiri", "kp", "raman").
#[napi]
pub fn sidereal_longitude(body: String, jd_ut1: f64, ayanamsa: String) -> Result<f64> {
    let b = parse_body(&body)?;
    let aya = parse_ayanamsa(&ayanamsa)?;
    let jd_tt = xalen_time::JdUT1(jd_ut1).to_tt(&DeltaTModel::StephensonMorrisonHohenkerk2016);
    let provider = Vsop87Provider::new();
    let pos = provider
        .geocentric_ecliptic(b, jd_tt)
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
    let sid = xalen_ayanamsa::tropical_to_sidereal(pos.longitude, &aya, jd_tt.as_f64());
    Ok(sid.to_degrees().rem_euclid(360.0))
}

/// Nakshatra name for a given sidereal longitude in degrees.
#[napi]
pub fn nakshatra(sidereal_lon: f64) -> String {
    Nakshatra::from_longitude_deg(sidereal_lon).to_string()
}

/// Rashi (sidereal zodiac sign) name for a given sidereal longitude in degrees.
#[napi]
pub fn rashi(sidereal_lon: f64) -> String {
    Rashi::from_longitude_deg(sidereal_lon).to_string()
}

/// Panchang (tithi, nakshatra, yoga, karana, vara) as a JSON object.
///
/// `sun_lon` and `moon_lon` are sidereal longitudes in degrees.
/// `jd` is the Julian Day (UT1).
#[napi]
pub fn panchang(sun_lon: f64, moon_lon: f64, jd: f64) -> serde_json::Value {
    let p = compute_panchang(sun_lon, moon_lon, jd);
    serde_json::json!({
        "tithi_number": p.tithi.number,
        "tithi_name": p.tithi.name(),
        "paksha": match p.tithi.paksha {
            xalen_vedic::panchang::Paksha::Shukla => "Shukla",
            xalen_vedic::panchang::Paksha::Krishna => "Krishna",
        },
        "nakshatra": p.nakshatra.to_string(),
        "yoga_number": p.yoga.number,
        "yoga_name": p.yoga.name(),
        "karana_name": p.karana.name(),
        "vara": p.vara.name(),
    })
}

/// Compute houses for a given Julian Day (UT1), geographic latitude/longitude in
/// degrees, and house system name. Returns an object with the 12 cusps plus the
/// Ascendant, MC, IC, Descendant and Vertex (all degrees), and `fallbackUsed`
/// (true when a polar latitude forced the Porphyry fallback). This mirrors the
/// Python binding — earlier versions returned only the 12 cusps and silently
/// dropped the angles.
#[napi]
pub fn houses(jd: f64, lat: f64, lon: f64, system: String) -> Result<serde_json::Value> {
    let sys = parse_house_system(&system)?;
    let loc = GeoLocation::new(lat, lon);
    let t = (jd - 2_451_545.0) / 36525.0;
    let epsilon = xalen_coords::obliquity::mean_obliquity(t);
    let h = compute_houses(jd, &loc, epsilon, sys);
    let deg = |r: f64| r.to_degrees().rem_euclid(360.0);
    Ok(serde_json::json!({
        "cusps": h.cusps.iter().map(|c| deg(*c)).collect::<Vec<f64>>(),
        "ascendant": deg(h.ascendant),
        "mc": deg(h.mc),
        "ic": deg(h.ic),
        "descendant": deg(h.descendant),
        "vertex": deg(h.vertex),
        "fallbackUsed": h.fallback_used,
    }))
}

/// Numerology life-path number from birth date.
#[napi]
pub fn life_path(year: u32, month: u32, day: u32) -> u32 {
    xalen_numerology::life_path(year, month, day)
}

/// Numerology expression number from full name.
/// `system` is "pythagorean" or "chaldean".
#[napi]
pub fn expression_number(name: String, system: String) -> Result<u32> {
    let sys = parse_numerology_system(&system)?;
    Ok(xalen_numerology::expression_number(&name, sys))
}

/// Delta-T in seconds (TT - UT1) at the given Julian Day using the
/// Stephenson-Morrison-Hohenkerk 2016 model.
#[napi]
pub fn delta_t(jd: f64) -> f64 {
    xalen_time::delta_t(jd, &DeltaTModel::StephensonMorrisonHohenkerk2016)
}

/// Fixed-star conjunctions within `orb` degrees of `planet_lon` (degrees),
/// precessed to `year`. Returns a JSON array of { name, distance, constellation, magnitude }.
#[napi]
pub fn fixed_star_conjunctions(planet_lon: f64, orb: f64, year: f64) -> serde_json::Value {
    let matches = xalen_stars::find_conjunctions_at_epoch(planet_lon, orb, year);
    let arr: Vec<serde_json::Value> = matches
        .iter()
        .map(|(star, dist)| {
            serde_json::json!({
                "name": star.name,
                "distance": dist,
                "constellation": star.constellation,
                "magnitude": star.magnitude,
                "nature": star.nature,
            })
        })
        .collect();
    serde_json::Value::Array(arr)
}

// ---------------------------------------------------------------------------
// Integer-ID based API (canonical mapping, matches FFI/Python/WASM)
// ---------------------------------------------------------------------------

/// Tropical ecliptic longitude by integer body ID.
/// body_id: 0=Sun, 1=Moon, 2=Mercury, 3=Venus, 4=Mars, 5=Jupiter, 6=Saturn,
///          7=Uranus, 8=Neptune, 9=Rahu, 10=TrueNode, 11=Pluto, 12=Chiron,
///          13=Ketu (Rahu+180).
#[napi]
pub fn planet_longitude_by_id(body_id: u8, jd_ut1: f64) -> Result<f64> {
    let jd_tt = xalen_time::JdUT1(jd_ut1).to_tt(&DeltaTModel::StephensonMorrisonHohenkerk2016);
    let provider = Vsop87Provider::new();

    // Handle Ketu (id 13) as Rahu + 180
    if body_id == 13 {
        let pos = provider
            .geocentric_ecliptic(Body::MeanNode, jd_tt)
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
        return Ok((pos.longitude.to_degrees() + 180.0).rem_euclid(360.0));
    }

    let b = body_from_id(body_id)?;
    let pos = provider
        .geocentric_ecliptic(b, jd_tt)
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
    Ok(pos.longitude.to_degrees().rem_euclid(360.0))
}

/// Sidereal ecliptic longitude by integer body and ayanamsa IDs.
#[napi]
pub fn sidereal_longitude_by_id(body_id: u8, jd_ut1: f64, ayanamsa_id: u8) -> Result<f64> {
    let aya = ayanamsa_from_id(ayanamsa_id)?;
    let jd_tt = xalen_time::JdUT1(jd_ut1).to_tt(&DeltaTModel::StephensonMorrisonHohenkerk2016);
    let provider = Vsop87Provider::new();

    // Handle Ketu (id 13) as Rahu + 180
    if body_id == 13 {
        let pos = provider
            .geocentric_ecliptic(Body::MeanNode, jd_tt)
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
        let sid = xalen_ayanamsa::tropical_to_sidereal(pos.longitude, &aya, jd_tt.as_f64());
        return Ok((sid.to_degrees() + 180.0).rem_euclid(360.0));
    }

    let b = body_from_id(body_id)?;
    let pos = provider
        .geocentric_ecliptic(b, jd_tt)
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
    let sid = xalen_ayanamsa::tropical_to_sidereal(pos.longitude, &aya, jd_tt.as_f64());
    Ok(sid.to_degrees().rem_euclid(360.0))
}

/// Compute house cusps by integer house system ID.
/// system_id: 0=WholeSign, 1=Equal, 2=Placidus, 3=Koch, ..., 13=Krusinski.
#[napi]
pub fn houses_by_id(jd: f64, lat: f64, lon: f64, system_id: u8) -> Result<Vec<f64>> {
    let sys = house_system_from_id(system_id)?;
    let loc = GeoLocation::new(lat, lon);
    let t = (jd - 2_451_545.0) / 36525.0; let epsilon = xalen_coords::obliquity::mean_obliquity(t);
    let h = compute_houses(jd, &loc, epsilon, sys);
    Ok(h.cusps
        .iter()
        .map(|c| c.to_degrees().rem_euclid(360.0))
        .collect())
}

/// Ayanamsa value in degrees by integer ID.
/// ayanamsa_id: 0=Lahiri, 1=KP, ..., 16=LahiriVP285.
#[napi]
pub fn ayanamsa_by_id(jd_ut1: f64, ayanamsa_id: u8) -> Result<f64> {
    let aya = ayanamsa_from_id(ayanamsa_id)?;
    let jd_tt = xalen_time::JdUT1(jd_ut1).to_tt(&DeltaTModel::StephensonMorrisonHohenkerk2016);
    Ok(aya.compute_deg(jd_tt.as_f64()))
}
