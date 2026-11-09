//! `xalen convert`: time scales, sidereal time and every ayanamsa.

use serde_json::json;
use xalen_ephemeris::ayanamsa::Ayanamsa;
use xalen_ephemeris::coords::{gast_deg, gmst_deg};
use xalen_ephemeris::time::delta_t;

use crate::cli::ConvertArgs;
use crate::common::ayanamsa_json;
use crate::error::Result;
use crate::input::delta_t_model;
use crate::output::{Report, num, table};

/// Hours as `HHh MMm SS.SSSs`.
fn hms(hours: f64) -> String {
    let total = hours.rem_euclid(24.0) * 3600.0;
    let h = (total / 3600.0).floor();
    let m = ((total - h * 3600.0) / 60.0).floor();
    let s = total - h * 3600.0 - m * 60.0;
    format!("{h:02.0}h {m:02.0}m {s:06.3}s")
}

/// Compute and format the conversions.
pub fn run(args: &ConvertArgs) -> Result<Report> {
    let inst = args.time.instant(&args.out)?;
    let delta_t_seconds = delta_t(inst.jd_ut, &delta_t_model());
    let t = (inst.jd_tt - 2_451_545.0) / 36_525.0;
    let gmst = gmst_deg(inst.jd_ut);
    let gast = gast_deg(inst.jd_ut, t);

    let mut rows = Vec::new();
    let mut items = Vec::new();
    for ayanamsa in Ayanamsa::all_named() {
        let value = ayanamsa.compute_deg(inst.jd_tt);
        rows.push(vec![
            format!("{ayanamsa:?}"),
            ayanamsa.to_string(),
            num(value, 6),
        ]);
        items.push(ayanamsa_json(*ayanamsa, value));
    }

    let mut text = format!("Conversions for {}\n\n", inst.stamp());
    text.push_str(&format!(
        "JD UT:                  {}\nJD TT:                  {}\nDelta T (TT - UT):      {} s\nGMST (mean sidereal):   {} deg = {}\nGAST (apparent):        {} deg = {}\n\nAyanamsa values at this date (degrees)\n\n",
        num(inst.jd_ut, 6),
        num(inst.jd_tt, 6),
        num(delta_t_seconds, 3),
        num(gmst, 6),
        hms(gmst / 15.0),
        num(gast, 6),
        hms(gast / 15.0),
    ));
    text.push_str(&table(&["Id", "Name", "Value"], &rows));

    let json = json!({
        "command": "convert",
        "date": inst.iso(),
        "jd_ut": inst.jd_ut,
        "jd_tt": inst.jd_tt,
        "delta_t_seconds": delta_t_seconds,
        "gmst_deg": gmst,
        "gmst_hours": gmst / 15.0,
        "gast_deg": gast,
        "gast_hours": gast / 15.0,
        "ayanamsas": items,
    });
    Ok(Report { json, text })
}
