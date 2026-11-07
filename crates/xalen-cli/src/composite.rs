//! `xalen composite`: the midpoint chart of two births.

use serde_json::json;
use xalen_ephemeris::ephem::Almanac;
use xalen_ephemeris::houses::HouseSystem;
use xalen_ephemeris::relationship::{
    CompositeAscendant, CompositeOptions, NatalChart, composite_chart,
};

use crate::cli::{AscendantMethod, CompositeArgs};
use crate::common::Frame;
use crate::error::Result;
use crate::input::Instant;
use crate::output::{Report, num};
use crate::rel::{
    ChartView, available_bodies, birth_json, birth_line, chart_report, geo, house_system,
    select_bodies,
};

/// Compute and format the composite chart.
pub fn run(args: &CompositeArgs) -> Result<Report> {
    let a = args.a.resolve(&args.out)?;
    let b = args.b.resolve(&args.out)?;
    let ayanamsa = args.zodiac.resolve()?;
    let default_system = match (ayanamsa.is_some(), args.ascendant) {
        (true, _) => HouseSystem::WholeSign,
        (false, AscendantMethod::Midpoint) => HouseSystem::Equal,
        (false, AscendantMethod::Derived) => HouseSystem::Placidus,
    };
    let system = house_system(&args.house_system, default_system)?;
    let method = match args.ascendant {
        AscendantMethod::Derived => CompositeAscendant::DerivedFromMidpointMc,
        AscendantMethod::Midpoint => CompositeAscendant::MidpointOfAscendants,
    };
    let (bodies, explicit) = select_bodies(&args.bodies)?;
    let almanac = Almanac::default_vedic();
    let (bodies, skipped) =
        available_bodies(&almanac, &[a.inst.jd_ut, b.inst.jd_ut], bodies, explicit)?;

    let natal = |birth: &crate::cli::Birth| -> Result<NatalChart> {
        Ok(NatalChart::compute(
            &almanac,
            birth.inst.ut1(),
            &geo(&birth.place)?,
            system,
            &bodies,
        )?)
    };
    let chart_a = natal(&a)?;
    let chart_b = natal(&b)?;
    let composite = composite_chart(
        &chart_a,
        &chart_b,
        &CompositeOptions {
            house_system: system,
            ascendant: method,
        },
    )?;

    let inst = Instant::new(composite.mean_jd_ut1, args.out.zone()?)?;
    let frame = Frame::new(ayanamsa, inst.jd_tt);
    let method_name = match args.ascendant {
        AscendantMethod::Derived => "derived",
        AscendantMethod::Midpoint => "midpoint",
    };
    let reference = (
        composite.reference_location.lat_deg(),
        composite.reference_location.lon_deg(),
    );
    let notes = vec![
        "Composite chart: each body is the midpoint of the two births; the date above is their mean date".to_string(),
        birth_line("A", &a),
        birth_line("B", &b),
        format!(
            "Reference place (mean birthplace): lat {}, lon {}",
            num(reference.0, 4),
            num(reference.1, 4)
        ),
        format!(
            "Ascendant: {}",
            match args.ascendant {
                AscendantMethod::Derived => "derived from the midpoint Midheaven at the reference latitude",
                AscendantMethod::Midpoint => "midpoint of the two Ascendants",
            }
        ),
    ];
    Ok(chart_report(ChartView {
        command: "composite",
        title: "Composite chart",
        inst: &inst,
        frame: &frame,
        bodies: &composite.bodies,
        houses: &composite.houses,
        system,
        notes,
        extra: vec![
            ("a", birth_json(&a)),
            ("b", birth_json(&b)),
            ("ascendant_method", json!(method_name)),
            (
                "reference_place",
                json!({ "latitude": reference.0, "longitude": reference.1 }),
            ),
        ],
        skipped: &skipped,
    }))
}
