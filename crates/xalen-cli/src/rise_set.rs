//! `xalen rise-set`: rise, transit and set times of a body.

use serde_json::json;
use xalen_ephemeris::ephem::{Almanac, Body};

use crate::cli::RiseSetArgs;
use crate::common::place_json;
use crate::error::{CliError, Result};
use crate::input::{Point, parse_point};
use crate::output::{Report, num};

/// Compute and format the rise, transit and set of a body over the 24 hours
/// that start at the given instant.
pub fn run(args: &RiseSetArgs) -> Result<Report> {
    let inst = args.time.instant(&args.out)?;
    let place = args
        .loc
        .resolve(true)?
        .ok_or_else(|| CliError::usage("missing --lat and --lon (or use --place \"<city>\")"))?;
    let point = parse_point(&args.body)?;
    let body = match point {
        Point::Body(
            body @ (Body::Sun
            | Body::Moon
            | Body::Mercury
            | Body::Venus
            | Body::Mars
            | Body::Jupiter
            | Body::Saturn
            | Body::Uranus
            | Body::Neptune
            | Body::Pluto),
        ) => body,
        _ => {
            return Err(CliError::usage(format!(
                "rise-set works for the Sun, the Moon and the planets, not '{}'",
                args.body.trim()
            )));
        }
    };
    let almanac = Almanac::default_vedic();
    let times = almanac.rise_transit_set(body, inst.ut1(), place.lat, place.lon, place.alt)?;
    let (rise, transit, set) = (
        times.rise.map(|j| j.0),
        times.transit.map(|j| j.0),
        times.set.map(|j| j.0),
    );
    let tz = &inst.tz;

    let place_label = place
        .name
        .clone()
        .unwrap_or_else(|| format!("{}, {}", num(place.lat, 4), num(place.lon, 4)));
    let mut text = format!(
        "{} rise, transit and set at {place_label}\nWindow: 24 hours from {}\n\n",
        point.name(),
        inst.stamp()
    );
    text.push_str(&format!(
        "Rise:     {}\nTransit:  {}\nSet:      {}\n",
        tz.stamp_opt(rise),
        tz.stamp_opt(transit),
        tz.stamp_opt(set)
    ));
    if times.transit_altitude_deg.is_finite() {
        text.push_str(&format!(
            "Altitude at transit: {} deg\n",
            num(times.transit_altitude_deg, 2)
        ));
    }
    if times.always_above {
        text.push_str("The body stays above the horizon for the whole window.\n");
    }
    if times.always_below {
        text.push_str("The body stays below the horizon for the whole window.\n");
    }

    let json = json!({
        "command": "rise-set",
        "date": inst.iso(),
        "jd_ut": inst.jd_ut,
        "place": place_json(&place),
        "body": point.id(),
        "rise": tz.iso_opt(rise),
        "transit": tz.iso_opt(transit),
        "set": tz.iso_opt(set),
        "rise_jd_ut": rise,
        "transit_jd_ut": transit,
        "set_jd_ut": set,
        "transit_altitude_deg": times.transit_altitude_deg,
        "always_above": times.always_above,
        "always_below": times.always_below,
    });
    Ok(Report { json, text })
}
