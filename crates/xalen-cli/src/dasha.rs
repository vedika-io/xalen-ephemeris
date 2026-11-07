//! `xalen dasha`: Vimshottari mahadasha and antardasha periods.

use serde_json::{Value, json};
use xalen_ephemeris::ayanamsa::Ayanamsa;
use xalen_ephemeris::ephem::{Almanac, Body};
use xalen_ephemeris::vedic::dasha::{DashaLevel, DashaPeriod, vimshottari_dasha};
use xalen_ephemeris::vedic::nakshatra::Nakshatra;

use crate::cli::DashaArgs;
use crate::common::{Frame, base_json, header, placement};
use crate::error::Result;
use crate::input::{Tz, parse_ayanamsa};
use crate::output::{Report, num, table};

fn period_json(period: &DashaPeriod, tz: &Tz) -> Value {
    json!({
        "level": format!("{:?}", period.level),
        "lord": period.lord.to_string(),
        "start": tz.iso(period.start_jd),
        "end": tz.iso(period.end_jd),
        "start_jd_ut": period.start_jd,
        "end_jd_ut": period.end_jd,
        "sub_periods": period
            .sub_periods
            .iter()
            .map(|p| period_json(p, tz))
            .collect::<Vec<_>>(),
    })
}

fn push_rows(rows: &mut Vec<Vec<String>>, period: &DashaPeriod, tz: &Tz, depth: usize) {
    rows.push(vec![
        format!("{}{}", "  ".repeat(depth), period.lord),
        format!("{:?}", period.level),
        tz.date(period.start_jd),
        tz.date(period.end_jd),
        num((period.end_jd - period.start_jd) / 365.25, 2),
    ]);
    for sub in &period.sub_periods {
        push_rows(rows, sub, tz, depth + 1);
    }
}

/// Compute and format the dasha periods, starting at the given instant as birth.
pub fn run(args: &DashaArgs) -> Result<Report> {
    let inst = args.time.instant(&args.out)?;
    let ayanamsa = match &args.ayanamsa {
        Some(text) => parse_ayanamsa(text)?,
        None => Ayanamsa::Lahiri,
    };
    let frame = Frame::new(Some(ayanamsa), inst.jd_tt);
    let almanac = Almanac::default_vedic();
    let moon = almanac.sidereal_longitude_deg(Body::Moon, inst.ut1(), frame.ayanamsa_deg)?;
    let depth = if args.levels == 1 {
        DashaLevel::Mahadasha
    } else {
        DashaLevel::Antardasha
    };
    let periods = vimshottari_dasha(moon, inst.jd_ut, depth);

    let nakshatra = Nakshatra::from_longitude_deg(moon);
    let pada = Nakshatra::pada(moon);
    let sign = placement(moon, true).sign;

    let mut rows = Vec::new();
    for period in &periods {
        push_rows(&mut rows, period, &inst.tz, 0);
    }
    let mut text = header("Vimshottari dasha", &inst, &frame);
    text.push_str(&format!(
        "Moon {} deg in {sign}, {nakshatra} pada {pada} (lord {})\n\n",
        num(moon, 4),
        nakshatra.lord()
    ));
    text.push_str(&table(&["Lord", "Level", "Start", "End", "Years"], &rows));

    let mut json = base_json("dasha", &inst, &frame);
    json["levels"] = json!(args.levels);
    json["moon"] = json!({
        "longitude": moon,
        "sign": sign,
        "nakshatra": nakshatra.to_string(),
        "pada": pada,
        "lord": nakshatra.lord().to_string(),
    });
    json["periods"] = Value::Array(periods.iter().map(|p| period_json(p, &inst.tz)).collect());
    Ok(Report { json, text })
}
