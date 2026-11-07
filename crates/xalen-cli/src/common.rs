//! Pieces shared by several commands: the zodiac frame, body positions,
//! sign and nakshatra placement, and the common JSON and text headers.

use serde_json::{Value, json};
use xalen_ephemeris::ayanamsa::Ayanamsa;
use xalen_ephemeris::ephem::{Almanac, Body, EphemerisError};
use xalen_ephemeris::vedic::nakshatra::Nakshatra;
use xalen_ephemeris::vedic::rashi::Rashi;

use crate::error::{CliError, Result};
use crate::input::{Instant, Place, Point};
use crate::output::{num, table};

/// The zodiac the longitudes are printed in.
#[derive(Debug, Clone)]
pub struct Frame {
    /// The ayanamsa, or `None` for the tropical zodiac.
    pub ayanamsa: Option<Ayanamsa>,
    /// Its value in degrees at the instant (0 when tropical).
    pub ayanamsa_deg: f64,
}

impl Frame {
    /// Build the frame for an instant (the ayanamsa is evaluated in TT).
    pub fn new(ayanamsa: Option<Ayanamsa>, jd_tt: f64) -> Self {
        Frame {
            ayanamsa,
            ayanamsa_deg: ayanamsa.map_or(0.0, |a| a.compute_deg(jd_tt)),
        }
    }

    /// True for a sidereal zodiac.
    pub fn sidereal(&self) -> bool {
        self.ayanamsa.is_some()
    }

    /// Move a tropical longitude (degrees) into this zodiac.
    pub fn apply(&self, tropical_deg: f64) -> f64 {
        (tropical_deg - self.ayanamsa_deg).rem_euclid(360.0)
    }
}

/// A computed body in the tropical zodiac.
#[derive(Debug, Clone)]
pub struct BodyPos {
    /// Which body.
    pub point: Point,
    /// Ecliptic longitude of date in degrees, 0 to 360.
    pub tropical_lon: f64,
    /// Ecliptic latitude in degrees.
    pub lat: f64,
    /// Distance in AU (0 for the geometric points).
    pub dist_au: f64,
    /// Daily motion in longitude, degrees per day (negative: retrograde).
    pub speed: f64,
}

/// The result of computing a list of bodies.
#[derive(Debug, Default)]
pub struct Gathered {
    /// The bodies that were computed.
    pub bodies: Vec<BodyPos>,
    /// Bodies left out of a default listing, with the reason.
    pub skipped: Vec<(Point, String)>,
}

fn compute_one(
    almanac: &Almanac,
    inst: &Instant,
    point: Point,
) -> std::result::Result<BodyPos, EphemerisError> {
    let body = match point {
        Point::Body(body) => body,
        Point::Ketu => Body::MeanNode,
    };
    let position = almanac.geocentric_ecliptic(body, inst.ut1())?;
    let speed = almanac.geocentric_speed(body, inst.ut1())?;
    let mut longitude = position.longitude;
    let mut latitude = position.latitude;
    if point == Point::Ketu {
        longitude = Body::ketu_longitude(longitude);
        latitude = -latitude;
    }
    Ok(BodyPos {
        point,
        tropical_lon: longitude.to_degrees().rem_euclid(360.0),
        lat: latitude.to_degrees(),
        dist_au: position.distance,
        speed: speed.longitude_deg_per_day(),
    })
}

/// Compute `points`. In a default listing (`explicit` false) a body the
/// ephemeris cannot reach at this date is reported in `skipped`; a body the
/// user asked for by name is an error instead.
pub fn gather(
    almanac: &Almanac,
    inst: &Instant,
    points: &[Point],
    explicit: bool,
) -> Result<Gathered> {
    let mut gathered = Gathered::default();
    for &point in points {
        match compute_one(almanac, inst, point) {
            Ok(pos) => gathered.bodies.push(pos),
            Err(e @ (EphemerisError::EpochOutOfRange(_) | EphemerisError::BodyNotAvailable(_)))
                if !explicit =>
            {
                gathered.skipped.push((point, e.to_string()))
            }
            Err(e) => {
                return Err(CliError::compute(format!("{}: {e}", point.name())));
            }
        }
    }
    if gathered.bodies.is_empty() {
        return Err(CliError::compute(
            "no body could be computed for this instant",
        ));
    }
    Ok(gathered)
}

/// Where a longitude sits in the zodiac.
#[derive(Debug, Clone)]
pub struct Placement {
    /// Sign name (Western for tropical, rashi with Western name for sidereal).
    pub sign: String,
    /// Degrees since the start of the sign.
    pub degree_in_sign: f64,
    /// Nakshatra name and pada, for sidereal longitudes.
    pub nakshatra: Option<(String, u8)>,
}

/// Describe a zodiac longitude in degrees.
pub fn placement(longitude: f64, sidereal: bool) -> Placement {
    let rashi = Rashi::from_longitude_deg(longitude);
    let sign = if sidereal {
        rashi.to_string()
    } else {
        rashi.western_name().to_string()
    };
    let nakshatra = sidereal.then(|| {
        (
            Nakshatra::from_longitude_deg(longitude).to_string(),
            Nakshatra::pada(longitude),
        )
    });
    Placement {
        sign,
        degree_in_sign: longitude.rem_euclid(30.0),
        nakshatra,
    }
}

/// Table and JSON rows for a list of bodies. With `house_of`, a house column
/// is added (used by `chart`).
pub fn describe_bodies(
    bodies: &[BodyPos],
    frame: &Frame,
    house_of: Option<&dyn Fn(f64) -> usize>,
) -> (String, Vec<Value>) {
    let sidereal = frame.sidereal();
    let mut headers = vec!["Body", "Longitude", "Sign", "Deg"];
    if sidereal {
        headers.extend(["Nakshatra", "Pada"]);
    }
    headers.extend(["Latitude", "Dist (AU)", "Speed/day", "R"]);
    if house_of.is_some() {
        headers.push("House");
    }

    let mut rows = Vec::new();
    let mut items = Vec::new();
    for body in bodies {
        let longitude = frame.apply(body.tropical_lon);
        let place = placement(longitude, sidereal);
        let mut row = vec![
            body.point.name().to_string(),
            num(longitude, 4),
            place.sign.clone(),
            num(place.degree_in_sign, 2),
        ];
        let mut item = json!({
            "id": body.point.id(),
            "name": body.point.name(),
            "longitude": longitude,
            "latitude": body.lat,
            "distance_au": body.dist_au,
            "speed_deg_per_day": body.speed,
            "retrograde": body.speed < 0.0,
            "sign": place.sign,
            "degree_in_sign": place.degree_in_sign,
        });
        if let Some((name, pada)) = &place.nakshatra {
            row.push(name.clone());
            row.push(pada.to_string());
            item["nakshatra"] = json!(name);
            item["pada"] = json!(pada);
        }
        row.push(num(body.lat, 4));
        row.push(num(body.dist_au, 6));
        row.push(num(body.speed, 4));
        let retro = if body.speed < 0.0 { "R" } else { "" };
        row.push(retro.to_string());
        if let Some(house_of) = house_of {
            let house = house_of(longitude);
            row.push(house.to_string());
            item["house"] = json!(house);
        }
        rows.push(row);
        items.push(item);
    }
    (table(&headers, &rows), items)
}

/// The note listing bodies that were left out of a default listing.
pub fn skipped_text(gathered: &Gathered) -> String {
    if gathered.skipped.is_empty() {
        return String::new();
    }
    let names: Vec<String> = gathered
        .skipped
        .iter()
        .map(|(point, why)| format!("{} ({why})", point.name()))
        .collect();
    format!("\nNot available at this date: {}\n", names.join("; "))
}

/// The `skipped` JSON array.
pub fn skipped_json(gathered: &Gathered) -> Value {
    Value::Array(
        gathered
            .skipped
            .iter()
            .map(|(point, why)| json!({ "id": point.id(), "name": point.name(), "reason": why }))
            .collect(),
    )
}

/// The first lines of every text report.
pub fn header(title: &str, inst: &Instant, frame: &Frame) -> String {
    let zodiac = match frame.ayanamsa {
        Some(a) => format!("sidereal, {a} ayanamsa {} deg", num(frame.ayanamsa_deg, 6)),
        None => "tropical".to_string(),
    };
    format!(
        "{title} for {}\nJD UT {}   JD TT {}\nZodiac: {zodiac}\n\n",
        inst.stamp(),
        num(inst.jd_ut, 6),
        num(inst.jd_tt, 6),
    )
}

/// The JSON object every command starts from.
pub fn base_json(command: &str, inst: &Instant, frame: &Frame) -> Value {
    let zodiac = if frame.sidereal() {
        "sidereal"
    } else {
        "tropical"
    };
    let mut value = json!({
        "command": command,
        "date": inst.iso(),
        "jd_ut": inst.jd_ut,
        "jd_tt": inst.jd_tt,
        "zodiac": zodiac,
        "ayanamsa": Value::Null,
    });
    if let Some(a) = frame.ayanamsa {
        value["ayanamsa"] = ayanamsa_json(a, frame.ayanamsa_deg);
    }
    value
}

/// An ayanamsa and its value as JSON.
pub fn ayanamsa_json(ayanamsa: Ayanamsa, value_deg: f64) -> Value {
    json!({
        "name": ayanamsa.to_string(),
        "id": format!("{ayanamsa:?}"),
        "value_deg": value_deg,
    })
}

/// An observer as JSON.
pub fn place_json(place: &Place) -> Value {
    json!({
        "name": place.name,
        "latitude": place.lat,
        "longitude": place.lon,
        "altitude_m": place.alt,
    })
}
