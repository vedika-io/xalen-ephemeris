//! `xalen eclipses`: solar and lunar eclipses in a date range.

use serde_json::{Value, json};
use xalen_ephemeris::ephem::Almanac;
use xalen_ephemeris::ephem::eclipse::{find_lunar_eclipses, find_solar_eclipses};

use crate::cli::{EclipseKind, EclipsesArgs};
use crate::error::{CliError, Result};
use crate::input::parse_iso;
use crate::output::{Report, num, table};

/// The longest range accepted, in days (the search steps through every
/// New and Full Moon, so very long ranges take a while).
const MAX_RANGE_DAYS: f64 = 36_525.0;

struct Row {
    jd: f64,
    kind: &'static str,
    class: String,
    detail: String,
    extra: Value,
}

/// Find and format the eclipses.
pub fn run(args: &EclipsesArgs) -> Result<Report> {
    let tz = args.out.zone()?;
    let from = parse_iso(&args.from)?;
    let to = parse_iso(&args.to)?;
    if to <= from {
        return Err(CliError::usage("--to must be later than --from"));
    }
    if to - from > MAX_RANGE_DAYS {
        return Err(CliError::usage(
            "the range is longer than 100 years; search a shorter range",
        ));
    }
    let almanac = Almanac::default_vedic();
    let mut rows: Vec<Row> = Vec::new();
    if args.kind != EclipseKind::Lunar {
        for e in find_solar_eclipses(&almanac, from, to) {
            rows.push(Row {
                jd: e.jd_maximum,
                kind: "solar",
                class: format!("{:?}", e.eclipse_type),
                detail: format!("gamma {}", num(e.gamma, 4)),
                extra: json!({
                    "gamma": e.gamma,
                    "coverage_proxy": e.coverage_proxy,
                    "moon_latitude_deg": e.moon_latitude_deg,
                }),
            });
        }
    }
    if args.kind != EclipseKind::Solar {
        for e in find_lunar_eclipses(&almanac, from, to) {
            rows.push(Row {
                jd: e.jd_maximum,
                kind: "lunar",
                class: format!("{:?}", e.eclipse_type),
                detail: format!("umbral magnitude {}", num(e.umbral_magnitude, 3)),
                extra: json!({
                    "umbral_magnitude": e.umbral_magnitude,
                    "penumbral_magnitude": e.penumbral_magnitude,
                    "moon_latitude_deg": e.moon_latitude_deg,
                }),
            });
        }
    }
    rows.sort_by(|a, b| a.jd.total_cmp(&b.jd));

    let cells: Vec<Vec<String>> = rows
        .iter()
        .map(|r| {
            vec![
                tz.stamp(r.jd),
                r.kind.to_string(),
                r.class.clone(),
                r.detail.clone(),
            ]
        })
        .collect();
    let mut text = format!(
        "Eclipses from {} to {}: {} found\n\n",
        tz.stamp(from),
        tz.stamp(to),
        rows.len()
    );
    if !cells.is_empty() {
        text.push_str(&table(&["Maximum", "Kind", "Type", "Detail"], &cells));
    }

    let items: Vec<Value> = rows
        .iter()
        .map(|r| {
            let mut item = json!({
                "kind": r.kind,
                "type": r.class,
                "maximum": tz.iso(r.jd),
                "jd_ut": r.jd,
            });
            if let (Some(base), Some(extra)) = (item.as_object_mut(), r.extra.as_object()) {
                for (key, value) in extra {
                    base.insert(key.clone(), value.clone());
                }
            }
            item
        })
        .collect();
    let json = json!({
        "command": "eclipses",
        "from": tz.iso(from),
        "to": tz.iso(to),
        "count": items.len(),
        "eclipses": items,
    });
    Ok(Report { json, text })
}
