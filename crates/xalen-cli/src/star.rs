//! `xalen star`: the position of a fixed star on a date.

use serde_json::json;
use xalen_ephemeris::coords::{EclipticPosition, ecliptic_to_equatorial, mean_obliquity};
use xalen_ephemeris::stars::find_by_name;

use crate::cli::StarArgs;
use crate::common::placement;
use crate::error::{CliError, Result};
use crate::output::{Report, num};

/// Look the star up and format its position.
pub fn run(args: &StarArgs) -> Result<Report> {
    let inst = args.time.instant(&args.out)?;
    let name = args.name.join(" ");
    let wanted = name.trim();
    let star = find_by_name(wanted).ok_or_else(|| {
        CliError::usage(format!(
            "unknown star '{wanted}': give an IAU proper name or a traditional name, spelled \
             in full and quoted if it has spaces (for example Spica, Regulus, Antares, \
             Aldebaran, \"Alpha Centauri\"); the matching ignores case"
        ))
    })?;

    // Same epoch convention as the library's fixed-star calls.
    let year = 2000.0 + (inst.jd_ut - 2_451_545.0) / 365.25;
    let (longitude, latitude) = star.ecliptic_at_epoch(year);
    let t = (inst.jd_tt - 2_451_545.0) / 36_525.0;
    let equatorial = ecliptic_to_equatorial(
        &EclipticPosition {
            longitude: longitude.to_radians(),
            latitude: latitude.to_radians(),
            distance: 1.0,
        },
        mean_obliquity(t),
    );
    let ra_deg = equatorial.right_ascension.to_degrees().rem_euclid(360.0);
    let dec_deg = equatorial.dec_deg();
    let sign = placement(longitude, false);

    let mut text = format!("Fixed star {} for {}\n\n", star.name, inst.stamp());
    text.push_str(&format!(
        "Constellation: {}\nMagnitude:     {}\nEcliptic lon:  {} deg ({} {} deg)\nEcliptic lat:  {} deg\nRight asc.:    {} deg ({} h)\nDeclination:   {} deg\n",
        if star.constellation.is_empty() { "-" } else { star.constellation },
        num(star.magnitude, 2),
        num(longitude, 4),
        sign.sign,
        num(sign.degree_in_sign, 2),
        num(latitude, 4),
        num(ra_deg, 4),
        num(ra_deg / 15.0, 4),
        num(dec_deg, 4),
    ));
    if !star.nature.is_empty() {
        text.push_str(&format!("Nature:        {}\n", star.nature));
    }
    text.push_str("Mean ecliptic and equator of date, proper motion included.\n");

    let json = json!({
        "command": "star",
        "date": inst.iso(),
        "jd_ut": inst.jd_ut,
        "jd_tt": inst.jd_tt,
        "name": star.name,
        "constellation": star.constellation,
        "magnitude": star.magnitude,
        "nature": star.nature,
        "longitude": longitude,
        "latitude": latitude,
        "sign": sign.sign,
        "degree_in_sign": sign.degree_in_sign,
        "ra_deg": ra_deg,
        "ra_hours": ra_deg / 15.0,
        "dec_deg": dec_deg,
        "epoch_year": year,
    });
    Ok(Report { json, text })
}
