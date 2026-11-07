//! `xalen aspects`: tropical aspects between the bodies at an instant.

use serde_json::{Value, json};
use xalen_ephemeris::ephem::Almanac;
use xalen_ephemeris::western::aspects::{Aspect, AspectType, find_all_aspects, find_aspect};

use crate::cli::AspectsArgs;
use crate::common::{Frame, base_json, gather, header, skipped_json, skipped_text};
use crate::error::{CliError, Result};
use crate::input::{ASPECT_POINTS, select_points};
use crate::output::{Report, num, table};

/// Compute and format the aspects.
pub fn run(args: &AspectsArgs) -> Result<Report> {
    let inst = args.time.instant(&args.out)?;
    if let Some(orb) = args.orb {
        if !orb.is_finite() || orb <= 0.0 || orb > 30.0 {
            return Err(CliError::usage(
                "--orb must be above 0 and at most 30 degrees",
            ));
        }
    }
    let (points, explicit) = select_points(&args.bodies, &ASPECT_POINTS)?;
    let frame = Frame::new(None, inst.jd_tt);
    let almanac = Almanac::default_vedic();
    let gathered = gather(&almanac, &inst, &points, explicit)?;
    let kinds = if args.minor {
        AspectType::ALL
    } else {
        AspectType::MAJOR
    };

    let positions: Vec<(String, f64, f64)> = gathered
        .bodies
        .iter()
        .map(|b| (b.point.name().to_string(), b.tropical_lon, b.speed))
        .collect();

    let mut found: Vec<Aspect> = match args.orb {
        None => find_all_aspects(&positions, kinds),
        Some(orb) => {
            // The library's orb argument is a multiplier of each aspect's own
            // default orb, so a flat orb is a per-aspect multiplier.
            let mut out = Vec::new();
            for i in 0..positions.len() {
                for j in (i + 1)..positions.len() {
                    for &kind in kinds {
                        let multiplier = orb / kind.default_orb_deg();
                        if let Some(mut aspect) = find_aspect(
                            positions[i].1,
                            positions[j].1,
                            positions[i].2,
                            positions[j].2,
                            &[kind],
                            multiplier,
                        ) {
                            aspect.body1 = positions[i].0.clone();
                            aspect.body2 = positions[j].0.clone();
                            out.push(aspect);
                            break;
                        }
                    }
                }
            }
            out
        }
    };
    found.sort_by(|a, b| a.orb_deg.total_cmp(&b.orb_deg));

    let rows: Vec<Vec<String>> = found
        .iter()
        .map(|a| {
            vec![
                a.body1.clone(),
                format!("{:?}", a.aspect_type),
                a.body2.clone(),
                num(a.aspect_type.angle_deg(), 0),
                num(a.orb_deg, 3),
                format!("{:?}", a.direction),
            ]
        })
        .collect();
    let mut text = header("Aspects", &inst, &frame);
    let orb_note = match args.orb {
        Some(orb) => format!("flat orb {} deg", num(orb, 2)),
        None => "default orbs of the aspects module".to_string(),
    };
    text.push_str(&format!("{} aspects found ({orb_note})\n\n", found.len()));
    if !rows.is_empty() {
        text.push_str(&table(
            &["Body", "Aspect", "Body", "Angle", "Orb", "Motion"],
            &rows,
        ));
    }
    text.push_str(&skipped_text(&gathered));

    let items: Vec<Value> = found
        .iter()
        .map(|a| {
            json!({
                "body1": a.body1,
                "body2": a.body2,
                "aspect": format!("{:?}", a.aspect_type),
                "angle_deg": a.aspect_type.angle_deg(),
                "default_orb_deg": a.aspect_type.default_orb_deg(),
                "separation_deg": a.exact_deg,
                "orb_deg": a.orb_deg,
                "direction": format!("{:?}", a.direction),
            })
        })
        .collect();
    let mut json = base_json("aspects", &inst, &frame);
    json["orb_override_deg"] = json!(args.orb);
    json["minor"] = json!(args.minor);
    json["aspects"] = Value::Array(items);
    json["skipped"] = skipped_json(&gathered);
    Ok(Report { json, text })
}
