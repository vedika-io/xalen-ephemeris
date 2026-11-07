//! Pieces shared by the relationship commands (`synastry`, `composite`,
//! `davison`, `relocate`): body selection, house systems, and the chart-style
//! report.

use serde_json::{Value, json};
use xalen_ephemeris::ephem::{Almanac, Body, EphemerisError};
use xalen_ephemeris::houses::{GeoLocation, HouseCusps, HouseSystem};
use xalen_ephemeris::relationship::{ChartBody, default_bodies};
use xalen_ephemeris::time::JdUT1;

use crate::cli::Birth;
use crate::common::{Frame, base_json, header, place_json, placement};
use crate::error::{CliError, Result};
use crate::input::{Instant, Place, Point, parse_house_system, parse_point};
use crate::output::{Report, num, table};

/// Resolve `--bodies`. The flag is true when the user named the bodies.
pub fn select_bodies(names: &[String]) -> Result<(Vec<Body>, bool)> {
    if names.is_empty() {
        return Ok((default_bodies(), false));
    }
    let mut bodies = Vec::new();
    for name in names {
        match parse_point(name)? {
            Point::Body(body) => {
                if !bodies.contains(&body) {
                    bodies.push(body);
                }
            }
            Point::Ketu => {
                return Err(CliError::usage(
                    "ketu is not a separate body here; use rahu, Ketu is 180 degrees opposite",
                ));
            }
        }
    }
    Ok((bodies, true))
}

/// Bodies left out of a default listing, with the reason.
pub type Skipped = Vec<(Body, String)>;

/// In a default listing, drop the bodies the ephemeris cannot reach at one of
/// the instants (Pluto outside 1885-2099) and report them. A body the user
/// named is kept, so the library's error reaches them.
pub fn available_bodies(
    almanac: &Almanac,
    jds_ut: &[f64],
    bodies: Vec<Body>,
    explicit: bool,
) -> Result<(Vec<Body>, Skipped)> {
    if explicit {
        return Ok((bodies, Vec::new()));
    }
    let mut kept = Vec::new();
    let mut skipped = Vec::new();
    for body in bodies {
        let mut reason = None;
        for &jd in jds_ut {
            match almanac.geocentric_ecliptic(body, JdUT1(jd)) {
                Ok(_) => {}
                Err(
                    e @ (EphemerisError::EpochOutOfRange(_) | EphemerisError::BodyNotAvailable(_)),
                ) => {
                    reason = Some(e.to_string());
                    break;
                }
                Err(e) => {
                    return Err(CliError::compute(format!(
                        "{}: {e}",
                        Point::Body(body).name()
                    )));
                }
            }
        }
        match reason {
            Some(why) => skipped.push((body, why)),
            None => kept.push(body),
        }
    }
    if kept.is_empty() {
        return Err(CliError::compute(
            "no body could be computed for these instants",
        ));
    }
    Ok((kept, skipped))
}

/// The library location for a place.
pub fn geo(place: &Place) -> Result<GeoLocation> {
    GeoLocation::try_new(place.lat, place.lon)
        .ok_or_else(|| CliError::usage("the place is outside the valid latitude/longitude range"))
}

/// `--house-system`, or the default.
pub fn house_system(text: &Option<String>, default: HouseSystem) -> Result<HouseSystem> {
    match text {
        Some(text) => parse_house_system(text),
        None => Ok(default),
    }
}

/// A place as `lat, lon` text, or its city name.
pub fn place_label(place: &Place) -> String {
    place
        .name
        .clone()
        .unwrap_or_else(|| format!("{}, {}", num(place.lat, 4), num(place.lon, 4)))
}

/// A birth as JSON.
pub fn birth_json(birth: &Birth) -> Value {
    json!({
        "date": birth.inst.iso(),
        "jd_ut": birth.inst.jd_ut,
        "place": place_json(&birth.place),
    })
}

/// A birth as one line of text.
pub fn birth_line(label: &str, birth: &Birth) -> String {
    format!(
        "{label}: {}  {} (lat {}, lon {})",
        birth.inst.stamp(),
        place_label(&birth.place),
        num(birth.place.lat, 4),
        num(birth.place.lon, 4)
    )
}

/// The `Not available at this date` note.
pub fn skipped_note(skipped: &[(Body, String)]) -> String {
    if skipped.is_empty() {
        return String::new();
    }
    let names: Vec<String> = skipped
        .iter()
        .map(|(body, why)| format!("{} ({why})", Point::Body(*body).name()))
        .collect();
    format!("\nNot available at this date: {}\n", names.join("; "))
}

/// The `skipped` JSON array.
pub fn skipped_value(skipped: &[(Body, String)]) -> Value {
    Value::Array(
        skipped
            .iter()
            .map(|(body, why)| {
                let point = Point::Body(*body);
                json!({ "id": point.id(), "name": point.name(), "reason": why })
            })
            .collect(),
    )
}

pub fn angle_json(longitude: f64, sidereal: bool) -> Value {
    let place = placement(longitude, sidereal);
    json!({
        "longitude": longitude,
        "sign": place.sign,
        "degree_in_sign": place.degree_in_sign,
    })
}

/// Everything a chart-style report needs.
pub struct ChartView<'a> {
    /// The `command` key of the JSON.
    pub command: &'a str,
    /// The first words of the text report.
    pub title: &'a str,
    /// The instant the chart is for.
    pub inst: &'a Instant,
    /// The zodiac to print in.
    pub frame: &'a Frame,
    /// The bodies, tropical.
    pub bodies: &'a [ChartBody],
    /// The houses, tropical.
    pub houses: &'a HouseCusps,
    /// The house system that was asked for.
    pub system: HouseSystem,
    /// Lines printed under the header.
    pub notes: Vec<String>,
    /// Extra JSON keys.
    pub extra: Vec<(&'static str, Value)>,
    /// Bodies left out of a default listing.
    pub skipped: &'a [(Body, String)],
}

/// Print a chart the way `xalen chart` does: angles, cusps, bodies in houses.
pub fn chart_report(view: ChartView<'_>) -> Report {
    let frame = view.frame;
    let sidereal = frame.sidereal();
    let houses = if sidereal {
        view.houses.to_sidereal(frame.ayanamsa_deg.to_radians())
    } else {
        view.houses.clone()
    };
    let degrees = |radians: f64| radians.to_degrees().rem_euclid(360.0);
    let angles = [
        ("Ascendant", "ascendant", degrees(houses.ascendant)),
        ("Midheaven", "mc", degrees(houses.mc)),
        ("Descendant", "descendant", degrees(houses.descendant)),
        ("Imum Coeli", "ic", degrees(houses.ic)),
        ("Vertex", "vertex", degrees(houses.vertex)),
    ];
    let cusps: Vec<f64> = houses.cusps.iter().map(|c| degrees(*c)).collect();

    let mut headers = vec!["Body", "Longitude", "Sign", "Deg"];
    if sidereal {
        headers.extend(["Nakshatra", "Pada"]);
    }
    headers.extend(["Latitude", "Speed/day", "R", "House"]);
    let mut body_rows = Vec::new();
    let mut body_items = Vec::new();
    for body in view.bodies {
        let point = Point::Body(body.body);
        let longitude = frame.apply(body.longitude_deg);
        let place = placement(longitude, sidereal);
        let house = houses.planet_in_house(longitude.to_radians());
        let mut row = vec![
            point.name().to_string(),
            num(longitude, 4),
            place.sign.clone(),
            num(place.degree_in_sign, 2),
        ];
        let mut item = json!({
            "id": point.id(),
            "name": point.name(),
            "longitude": longitude,
            "latitude": body.latitude_deg,
            "speed_deg_per_day": body.speed_deg_per_day,
            "retrograde": body.speed_deg_per_day < 0.0,
            "sign": place.sign,
            "degree_in_sign": place.degree_in_sign,
            "house": house,
        });
        if let Some((name, pada)) = &place.nakshatra {
            row.push(name.clone());
            row.push(pada.to_string());
            item["nakshatra"] = json!(name);
            item["pada"] = json!(pada);
        }
        row.push(num(body.latitude_deg, 4));
        row.push(num(body.speed_deg_per_day, 4));
        row.push(
            if body.speed_deg_per_day < 0.0 {
                "R"
            } else {
                ""
            }
            .to_string(),
        );
        row.push(house.to_string());
        body_rows.push(row);
        body_items.push(item);
    }

    let angle_rows: Vec<Vec<String>> = angles
        .iter()
        .map(|(label, _, longitude)| {
            let p = placement(*longitude, sidereal);
            vec![
                (*label).to_string(),
                num(*longitude, 4),
                p.sign,
                num(p.degree_in_sign, 2),
            ]
        })
        .collect();
    let cusp_rows: Vec<Vec<String>> = cusps
        .iter()
        .enumerate()
        .map(|(i, longitude)| {
            let p = placement(*longitude, sidereal);
            vec![
                (i + 1).to_string(),
                num(*longitude, 4),
                p.sign,
                num(p.degree_in_sign, 2),
            ]
        })
        .collect();

    let mut text = header(view.title, view.inst, frame);
    for note in &view.notes {
        text.push_str(note);
        text.push('\n');
    }
    text.push_str(&format!(
        "House system: {}{}\n\n",
        view.system,
        if houses.fallback_used {
            " (fell back to Porphyry at this latitude)"
        } else {
            ""
        }
    ));
    text.push_str(&table(&["Angle", "Longitude", "Sign", "Deg"], &angle_rows));
    text.push('\n');
    text.push_str(&table(&["House", "Cusp", "Sign", "Deg"], &cusp_rows));
    text.push('\n');
    text.push_str(&table(&headers, &body_rows));
    text.push_str(&skipped_note(view.skipped));

    let mut json = base_json(view.command, view.inst, frame);
    json["house_system"] = json!(format!("{:?}", view.system));
    json["house_system_name"] = json!(view.system.to_string());
    json["fallback_used"] = json!(houses.fallback_used);
    for (_, key, longitude) in &angles {
        json[*key] = angle_json(*longitude, sidereal);
    }
    json["cusps"] = Value::Array(
        cusps
            .iter()
            .enumerate()
            .map(|(i, longitude)| {
                let mut item = angle_json(*longitude, sidereal);
                item["house"] = json!(i + 1);
                item
            })
            .collect(),
    );
    json["bodies"] = Value::Array(body_items);
    json["skipped"] = skipped_value(view.skipped);
    for (key, value) in view.extra {
        json[key] = value;
    }
    Report { json, text }
}
