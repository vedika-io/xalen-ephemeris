//! `xalen map`: astrocartography lines.

use serde_json::{Value, json};
use xalen_ephemeris::ephem::{Almanac, Body};
use xalen_ephemeris::maps::{
    AstroMap, BodyLines, HorizonCrossing, LineKind, MapOptions, astro_map, horizon_half_arc,
    mc_longitude_deg,
};

use crate::cli::MapArgs;
use crate::error::{CliError, Result};
use crate::input::{Point, parse_point};
use crate::output::{Report, num, table};
use crate::rel::{available_bodies, skipped_note, skipped_value};

fn kind_name(kind: LineKind) -> &'static str {
    match kind {
        LineKind::Mc => "MC",
        LineKind::Ic => "IC",
        LineKind::Asc => "ASC",
        LineKind::Dsc => "DSC",
    }
}

fn point_id(body: Body) -> &'static str {
    Point::Body(body).id()
}

/// The longitude of a body's MC or IC line (a straight meridian).
fn meridian_longitude(lines: &BodyLines, kind: LineKind) -> Option<f64> {
    let line = lines.lines.iter().find(|l| l.kind == kind)?;
    Some(line.segments.first()?.first()?.1)
}

/// Where a body rises or sets on one latitude.
enum Horizon {
    Crosses { asc: f64, dsc: f64 },
    AlwaysAbove,
    AlwaysBelow,
}

fn horizon_at(lines: &BodyLines, gast: f64, lat: f64, altitude: f64) -> Horizon {
    match horizon_half_arc(lat, lines.dec_deg, altitude) {
        HorizonCrossing::Crosses { half_arc_deg } => Horizon::Crosses {
            asc: mc_longitude_deg(lines.ra_deg, gast + half_arc_deg),
            dsc: mc_longitude_deg(lines.ra_deg, gast - half_arc_deg),
        },
        HorizonCrossing::AlwaysAbove => Horizon::AlwaysAbove,
        HorizonCrossing::AlwaysBelow => Horizon::AlwaysBelow,
    }
}

/// A GeoJSON FeatureCollection (RFC 7946): one LineString per line segment and
/// one Point per zenith, coordinates `[longitude, latitude]`.
fn geojson(map: &AstroMap) -> Value {
    let mut features = Vec::new();
    for body in &map.bodies {
        let id = point_id(body.body);
        for line in &body.lines {
            for segment in &line.segments {
                let coordinates: Vec<Value> = segment
                    .iter()
                    .map(|&(lat, lon)| json!([lon, lat]))
                    .collect();
                features.push(json!({
                    "type": "Feature",
                    "properties": { "body": id, "kind": kind_name(line.kind) },
                    "geometry": { "type": "LineString", "coordinates": coordinates },
                }));
            }
        }
        features.push(json!({
            "type": "Feature",
            "properties": { "body": id, "kind": "ZENITH" },
            "geometry": {
                "type": "Point",
                "coordinates": [body.zenith.1, body.zenith.0],
            },
        }));
    }
    json!({ "type": "FeatureCollection", "features": features })
}

/// Compute and format the map.
pub fn run(args: &MapArgs) -> Result<Report> {
    let inst = args.time.instant(&args.out)?;
    if let Some(lat) = args.lat {
        if !lat.is_finite() || lat.abs() >= 90.0 {
            return Err(CliError::usage(
                "--lat must be strictly between -90 and 90 degrees",
            ));
        }
    }
    let options = MapOptions {
        lat_min_deg: args.lat_min,
        lat_max_deg: args.lat_max,
        lat_step_deg: args.lat_step,
        horizon_altitude_deg: args.altitude,
        ..MapOptions::default()
    };

    let almanac = Almanac::default_vedic();
    let (named, explicit) = if args.bodies.is_empty() {
        (Body::ALL_PLANETS.to_vec(), false)
    } else {
        let mut bodies = Vec::new();
        for name in &args.bodies {
            match parse_point(name)? {
                Point::Body(body) => {
                    if !bodies.contains(&body) {
                        bodies.push(body);
                    }
                }
                Point::Ketu => {
                    return Err(CliError::usage(
                        "ketu has no lines of its own; its MC line is the Rahu IC line",
                    ));
                }
            }
        }
        (bodies, true)
    };
    let (bodies, skipped) = available_bodies(&almanac, &[inst.jd_ut], named, explicit)?;
    let map = astro_map(&almanac, inst.ut1(), &bodies, &options)?;

    if args.geojson {
        return Ok(Report {
            json: geojson(&map),
            text: String::new(),
        });
    }

    let mut headers = vec![
        "Body",
        "RA",
        "Dec",
        "Zenith lat",
        "Zenith lon",
        "MC lon",
        "IC lon",
    ];
    if args.lat.is_some() {
        headers.extend(["ASC lon", "DSC lon"]);
    }
    let mut rows = Vec::new();
    let mut notes = Vec::new();
    let mut at_latitude = Vec::new();
    for lines in &map.bodies {
        let name = Point::Body(lines.body).name();
        let mc = meridian_longitude(lines, LineKind::Mc);
        let ic = meridian_longitude(lines, LineKind::Ic);
        let show = |value: Option<f64>| value.map_or_else(|| "-".to_string(), |v| num(v, 4));
        let mut row = vec![
            name.to_string(),
            num(lines.ra_deg, 4),
            num(lines.dec_deg, 4),
            num(lines.zenith.0, 4),
            num(lines.zenith.1, 4),
            show(mc),
            show(ic),
        ];
        if let Some(lat) = args.lat {
            let horizon = horizon_at(lines, map.gast_deg, lat, args.altitude);
            let (asc, dsc, status) = match horizon {
                Horizon::Crosses { asc, dsc } => (Some(asc), Some(dsc), "rises and sets"),
                Horizon::AlwaysAbove => (None, None, "always above the horizon"),
                Horizon::AlwaysBelow => (None, None, "never rises"),
            };
            if asc.is_none() {
                notes.push(format!("{name}: {status} at latitude {}", num(lat, 4)));
            }
            row.push(show(asc));
            row.push(show(dsc));
            at_latitude.push(json!({
                "body": point_id(lines.body),
                "status": status,
                "asc_longitude": asc,
                "dsc_longitude": dsc,
            }));
        }
        rows.push(row);
    }

    let mut text = format!(
        "Astrocartography for {}\nJD UT {}   JD TT {}   GAST {} deg\n\
         Latitudes {} to {}, step {}, horizon altitude {} deg\n\n",
        inst.stamp(),
        num(inst.jd_ut, 6),
        num(inst.jd_tt, 6),
        num(map.gast_deg, 4),
        num(options.lat_min_deg, 2),
        num(options.lat_max_deg, 2),
        num(options.lat_step_deg, 2),
        num(options.horizon_altitude_deg, 4),
    );
    text.push_str(&table(&headers, &rows));
    text.push_str(
        "\nLongitudes are east-positive in -180 to 180; RA, Dec and the zenith are in degrees.\n",
    );
    if let Some(lat) = args.lat {
        text.push_str(&format!(
            "ASC and DSC longitudes are where each body rises and sets on latitude {}.\n",
            num(lat, 4)
        ));
    }
    for note in &notes {
        text.push_str(note);
        text.push('\n');
    }
    text.push_str(&skipped_note(&skipped));

    let mut json = serde_json::to_value(&map)
        .map_err(|e| CliError::compute(format!("cannot encode the map: {e}")))?;
    json["command"] = json!("map");
    json["date"] = json!(inst.iso());
    json["jd_ut"] = json!(inst.jd_ut);
    json["jd_tt"] = json!(inst.jd_tt);
    json["options"] = serde_json::to_value(options)
        .map_err(|e| CliError::compute(format!("cannot encode the options: {e}")))?;
    if let Some(lat) = args.lat {
        json["at_latitude"] = json!({ "latitude": lat, "bodies": at_latitude });
    }
    json["skipped"] = skipped_value(&skipped);
    Ok(Report { json, text })
}
