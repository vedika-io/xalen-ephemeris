//! `xalen positions`: longitudes, latitudes, distances and daily speeds.

use serde_json::Value;
use xalen_ephemeris::ephem::Almanac;

use crate::cli::PositionsArgs;
use crate::common::{
    Frame, base_json, describe_bodies, gather, header, skipped_json, skipped_text,
};
use crate::error::Result;
use crate::input::{DEFAULT_POINTS, select_points};
use crate::output::Report;

/// Compute and format the positions.
pub fn run(args: &PositionsArgs) -> Result<Report> {
    let inst = args.time.instant(&args.out)?;
    let ayanamsa = args.zodiac.resolve()?;
    let (points, explicit) = select_points(&args.bodies, &DEFAULT_POINTS)?;
    let frame = Frame::new(ayanamsa, inst.jd_tt);
    let almanac = Almanac::default_vedic();
    let gathered = gather(&almanac, &inst, &points, explicit)?;
    let (table, items) = describe_bodies(&gathered.bodies, &frame, None);

    let mut text = header("Positions", &inst, &frame);
    text.push_str(&table);
    text.push_str(&skipped_text(&gathered));

    let mut json = base_json("positions", &inst, &frame);
    json["bodies"] = Value::Array(items);
    json["skipped"] = skipped_json(&gathered);
    Ok(Report { json, text })
}
