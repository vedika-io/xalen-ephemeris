//! `xalen relocate`: a birth chart with the houses recomputed for another place.

use xalen_ephemeris::ephem::Almanac;
use xalen_ephemeris::houses::HouseSystem;
use xalen_ephemeris::relationship::NatalChart;

use crate::cli::RelocateArgs;
use crate::common::{Frame, place_json};
use crate::error::Result;
use crate::output::{Report, num};
use crate::rel::{
    ChartView, angle_json, available_bodies, chart_report, geo, house_system, place_label,
    select_bodies,
};

/// Compute and format the relocated chart.
pub fn run(args: &RelocateArgs) -> Result<Report> {
    let inst = args.time.instant(&args.out)?;
    let birth = args.birth_place()?;
    let target = args.target_place()?;
    let ayanamsa = args.zodiac.resolve()?;
    let default_system = if ayanamsa.is_some() {
        HouseSystem::WholeSign
    } else {
        HouseSystem::Placidus
    };
    let system = house_system(&args.house_system, default_system)?;
    let (bodies, explicit) = select_bodies(&args.bodies)?;
    let almanac = Almanac::default_vedic();
    let (bodies, skipped) = available_bodies(&almanac, &[inst.jd_ut], bodies, explicit)?;

    let natal = NatalChart::compute(&almanac, inst.ut1(), &geo(&birth)?, system, &bodies)?;
    let moved = natal.relocated(&geo(&target)?, system)?;

    let frame = Frame::new(ayanamsa, inst.jd_tt);
    let natal_asc = frame.apply(natal.houses.ascendant.to_degrees().rem_euclid(360.0));
    let notes = vec![
        format!(
            "Born at: {} (lat {}, lon {})",
            place_label(&birth),
            num(birth.lat, 4),
            num(birth.lon, 4)
        ),
        format!(
            "Relocated to: {} (lat {}, lon {}); the bodies are unchanged, the houses are recomputed",
            place_label(&target),
            num(target.lat, 4),
            num(target.lon, 4)
        ),
    ];
    Ok(chart_report(ChartView {
        command: "relocate",
        title: "Relocated chart",
        inst: &inst,
        frame: &frame,
        bodies: &moved.bodies,
        houses: &moved.houses,
        system,
        notes,
        extra: vec![
            ("birth_place", place_json(&birth)),
            ("relocated_to", place_json(&target)),
            ("natal_ascendant", angle_json(natal_asc, frame.sidereal())),
        ],
        skipped: &skipped,
    }))
}
