//! `xalen davison`: the chart for the midpoint in time and place of two births.

use serde_json::json;
use xalen_ephemeris::ephem::Almanac;
use xalen_ephemeris::houses::HouseSystem;
use xalen_ephemeris::relationship::{BirthMoment, DavisonOptions, SpaceMidpoint, davison_chart};

use crate::cli::{Birth, DavisonArgs, SpaceKind};
use crate::common::Frame;
use crate::error::Result;
use crate::input::Instant;
use crate::output::{Report, num};
use crate::rel::{
    ChartView, available_bodies, birth_json, birth_line, chart_report, geo, house_system,
    select_bodies,
};

/// Compute and format the Davison chart.
pub fn run(args: &DavisonArgs) -> Result<Report> {
    let a = args.a.resolve(&args.out)?;
    let b = args.b.resolve(&args.out)?;
    let ayanamsa = args.zodiac.resolve()?;
    let default_system = if ayanamsa.is_some() {
        HouseSystem::WholeSign
    } else {
        HouseSystem::Placidus
    };
    let system = house_system(&args.house_system, default_system)?;
    let space = match args.space {
        SpaceKind::Arithmetic => SpaceMidpoint::Arithmetic,
        SpaceKind::GreatCircle => SpaceMidpoint::GreatCircle,
    };
    let (bodies, explicit) = select_bodies(&args.bodies)?;
    let almanac = Almanac::default_vedic();
    // The mean date lies between the two births, so a body that is reachable
    // at both births is reachable there.
    let (bodies, skipped) =
        available_bodies(&almanac, &[a.inst.jd_ut, b.inst.jd_ut], bodies, explicit)?;

    let moment = |birth: &Birth| -> Result<BirthMoment> {
        Ok(BirthMoment {
            jd_ut1: birth.inst.jd_ut,
            location: geo(&birth.place)?,
        })
    };
    let chart = davison_chart(
        &almanac,
        &moment(&a)?,
        &moment(&b)?,
        &DavisonOptions {
            bodies,
            house_system: system,
            space,
        },
    )?;

    let inst = Instant::new(chart.jd_ut1, args.out.zone()?)?;
    let frame = Frame::new(ayanamsa, inst.jd_tt);
    let (lat, lon) = (chart.location.lat_deg(), chart.location.lon_deg());
    let space_name = match args.space {
        SpaceKind::Arithmetic => "arithmetic",
        SpaceKind::GreatCircle => "great-circle",
    };
    let notes = vec![
        "Davison chart: the sky at the midpoint in time and place of the two births".to_string(),
        birth_line("A", &a),
        birth_line("B", &b),
        format!(
            "Mean place ({space_name}): lat {}, lon {}",
            num(lat, 4),
            num(lon, 4)
        ),
    ];
    Ok(chart_report(ChartView {
        command: "davison",
        title: "Davison chart",
        inst: &inst,
        frame: &frame,
        bodies: &chart.bodies,
        houses: &chart.houses,
        system,
        notes,
        extra: vec![
            ("a", birth_json(&a)),
            ("b", birth_json(&b)),
            ("space", json!(space_name)),
            ("place", json!({ "latitude": lat, "longitude": lon })),
        ],
        skipped: &skipped,
    }))
}
