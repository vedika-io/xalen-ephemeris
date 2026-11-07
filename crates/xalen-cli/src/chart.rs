//! `xalen chart`: ascendant, midheaven, house cusps and bodies in houses.

use serde_json::{Value, json};
use xalen_ephemeris::coords::{gast_deg, mean_obliquity, nutation_2000b};
use xalen_ephemeris::ephem::Almanac;
use xalen_ephemeris::houses::{GeoLocation, HouseSystem, compute_houses_from_ramc};

use crate::cli::ChartArgs;
use crate::common::{
    Frame, base_json, describe_bodies, gather, header, place_json, placement, skipped_json,
    skipped_text,
};
use crate::error::{CliError, Result};
use crate::input::{DEFAULT_POINTS, parse_house_system, select_points};
use crate::output::{Report, num, table};

fn angle_json(longitude: f64, sidereal: bool) -> Value {
    let place = placement(longitude, sidereal);
    json!({
        "longitude": longitude,
        "sign": place.sign,
        "degree_in_sign": place.degree_in_sign,
    })
}

/// Compute and format the chart.
pub fn run(args: &ChartArgs) -> Result<Report> {
    let inst = args.time.instant(&args.out)?;
    let place = args
        .loc
        .resolve(true)?
        .ok_or_else(|| CliError::usage("missing --lat and --lon (or use --place \"<city>\")"))?;
    let ayanamsa = args.zodiac.resolve()?;
    let system = match &args.house_system {
        Some(text) => parse_house_system(text)?,
        None if ayanamsa.is_some() => HouseSystem::WholeSign,
        None => HouseSystem::Placidus,
    };
    let (points, explicit) = select_points(&args.bodies, &DEFAULT_POINTS)?;
    let frame = Frame::new(ayanamsa, inst.jd_tt);
    let sidereal = frame.sidereal();
    let almanac = Almanac::default_vedic();

    let location = GeoLocation::try_new(place.lat, place.lon).ok_or_else(|| {
        CliError::usage("the place is outside the valid latitude/longitude range")
    })?;
    // Same recipe as the library's Swiss-compatible house call: apparent
    // sidereal time and the true obliquity of date.
    let t = (inst.jd_tt - 2_451_545.0) / 36_525.0;
    let epsilon = mean_obliquity(t) + nutation_2000b(t).delta_epsilon;
    let ramc = (gast_deg(inst.jd_ut, t) + place.lon)
        .rem_euclid(360.0)
        .to_radians();
    let mut houses = compute_houses_from_ramc(ramc, &location, epsilon, system);
    if sidereal {
        houses = houses.to_sidereal(frame.ayanamsa_deg.to_radians());
    }

    let degrees = |radians: f64| radians.to_degrees().rem_euclid(360.0);
    let angles = [
        ("Ascendant", "ascendant", degrees(houses.ascendant)),
        ("Midheaven", "mc", degrees(houses.mc)),
        ("Descendant", "descendant", degrees(houses.descendant)),
        ("Imum Coeli", "ic", degrees(houses.ic)),
        ("Vertex", "vertex", degrees(houses.vertex)),
    ];
    let cusps: Vec<f64> = houses.cusps.iter().map(|c| degrees(*c)).collect();

    let gathered = gather(&almanac, &inst, &points, explicit)?;
    let house_of = |longitude_deg: f64| houses.planet_in_house(longitude_deg.to_radians());
    let (bodies_table, items) = describe_bodies(&gathered.bodies, &frame, Some(&house_of));

    let mut angle_rows = Vec::new();
    for (label, _, longitude) in &angles {
        let p = placement(*longitude, sidereal);
        angle_rows.push(vec![
            (*label).to_string(),
            num(*longitude, 4),
            p.sign,
            num(p.degree_in_sign, 2),
        ]);
    }
    let mut cusp_rows = Vec::new();
    for (i, longitude) in cusps.iter().enumerate() {
        let p = placement(*longitude, sidereal);
        cusp_rows.push(vec![
            (i + 1).to_string(),
            num(*longitude, 4),
            p.sign,
            num(p.degree_in_sign, 2),
        ]);
    }

    let place_label = place
        .name
        .clone()
        .unwrap_or_else(|| format!("{}, {}", num(place.lat, 4), num(place.lon, 4)));
    let mut text = header("Chart", &inst, &frame);
    text.push_str(&format!(
        "Place: {place_label} (lat {}, lon {})\nHouse system: {system}{}\n\n",
        num(place.lat, 4),
        num(place.lon, 4),
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
    text.push_str(&bodies_table);
    text.push_str(&skipped_text(&gathered));

    let mut json = base_json("chart", &inst, &frame);
    json["place"] = place_json(&place);
    json["house_system"] = json!(format!("{system:?}"));
    json["house_system_name"] = json!(system.to_string());
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
    json["bodies"] = Value::Array(items);
    json["skipped"] = skipped_json(&gathered);
    Ok(Report { json, text })
}
