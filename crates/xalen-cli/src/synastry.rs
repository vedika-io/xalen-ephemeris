//! `xalen synastry`: the aspects between two births and the house overlays.

use serde_json::json;
use xalen_ephemeris::ephem::Almanac;
use xalen_ephemeris::houses::HouseSystem;
use xalen_ephemeris::relationship::{HouseOverlay, NatalChart, PointId, SynastryOptions, synastry};
use xalen_ephemeris::western::aspects::AspectType;

use crate::cli::{Birth, SynastryArgs};
use crate::common::placement;
use crate::error::{CliError, Result};
use crate::input::{Point, normalize};
use crate::output::{Report, num, table};
use crate::rel::{
    available_bodies, birth_json, birth_line, geo, house_system, select_bodies, skipped_note,
    skipped_value,
};

/// Three-letter label of an aspect, used in the grid.
fn aspect_abbreviation(kind: AspectType) -> &'static str {
    match kind {
        AspectType::Conjunction => "Cnj",
        AspectType::Opposition => "Opp",
        AspectType::Trine => "Tri",
        AspectType::Square => "Sqr",
        AspectType::Sextile => "Sxt",
        AspectType::SemiSextile => "Ssx",
        AspectType::Quincunx => "Qnx",
        AspectType::SemiSquare => "Ssq",
        AspectType::Sesquiquadrate => "Ses",
        AspectType::Quintile => "Qnt",
        AspectType::BiQuintile => "Bqn",
    }
}

/// Full name of a point of a chart.
fn point_name(point: PointId) -> &'static str {
    match point {
        PointId::Body(body) => Point::Body(body).name(),
        PointId::Ascendant => "Ascendant",
        PointId::Midheaven => "Midheaven",
        PointId::Descendant => "Descendant",
        PointId::Imum => "Imum Coeli",
    }
}

/// Parse `--aspects`: `major`, `all`, or aspect names.
fn parse_aspects(names: &[String]) -> Result<Vec<AspectType>> {
    if names.is_empty() {
        return Ok(AspectType::MAJOR.to_vec());
    }
    let mut kinds: Vec<AspectType> = Vec::new();
    let mut add = |found: &[AspectType]| {
        for kind in found {
            if !kinds.contains(kind) {
                kinds.push(*kind);
            }
        }
    };
    for name in names {
        match normalize(name).as_str() {
            "major" => add(AspectType::MAJOR),
            "all" => add(AspectType::ALL),
            "conjunction" | "conj" => add(&[AspectType::Conjunction]),
            "opposition" | "opp" => add(&[AspectType::Opposition]),
            "trine" => add(&[AspectType::Trine]),
            "square" => add(&[AspectType::Square]),
            "sextile" => add(&[AspectType::Sextile]),
            "semisextile" => add(&[AspectType::SemiSextile]),
            "quincunx" | "inconjunct" => add(&[AspectType::Quincunx]),
            "semisquare" => add(&[AspectType::SemiSquare]),
            "sesquiquadrate" | "sesquisquare" => add(&[AspectType::Sesquiquadrate]),
            "quintile" => add(&[AspectType::Quintile]),
            "biquintile" => add(&[AspectType::BiQuintile]),
            _ => {
                return Err(CliError::usage(format!(
                    "unknown aspect '{}'; use major, all, or names such as conjunction, \
                     opposition, trine, square, sextile, semi-sextile, quincunx, \
                     semi-square, sesquiquadrate, quintile, biquintile",
                    name.trim()
                )));
            }
        }
    }
    Ok(kinds)
}

fn overlay_table(items: &[HouseOverlay]) -> String {
    let rows: Vec<Vec<String>> = items
        .iter()
        .map(|item| {
            let p = placement(item.longitude_deg, false);
            vec![
                item.point.abbreviation().to_string(),
                point_name(item.point).to_string(),
                num(item.longitude_deg, 4),
                p.sign,
                num(p.degree_in_sign, 2),
                item.house.to_string(),
            ]
        })
        .collect();
    table(&["Pt", "Name", "Longitude", "Sign", "Deg", "House"], &rows)
}

/// Compute and format the synastry.
pub fn run(args: &SynastryArgs) -> Result<Report> {
    let a = args.a.resolve(&args.out)?;
    let b = args.b.resolve(&args.out)?;
    let system = house_system(&args.house_system, HouseSystem::Placidus)?;
    let kinds = parse_aspects(&args.aspects)?;
    if !args.orb_mult.is_finite() || args.orb_mult <= 0.0 {
        return Err(CliError::usage("--orb-mult must be a number above 0"));
    }
    let (bodies, explicit) = select_bodies(&args.bodies)?;
    let almanac = Almanac::default_vedic();
    let (bodies, skipped) =
        available_bodies(&almanac, &[a.inst.jd_ut, b.inst.jd_ut], bodies, explicit)?;

    let natal = |birth: &Birth| -> Result<NatalChart> {
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
    let result = synastry(
        &chart_a,
        &chart_b,
        &SynastryOptions {
            aspects: kinds.clone(),
            orb_multiplier: args.orb_mult,
            include_angles: args.angles,
        },
    )?;

    // The aspect grid: rows are A's points, columns B's.
    let col_labels: Vec<&str> = result.cols.iter().map(|p| p.abbreviation()).collect();
    let mut headers = vec!["A \\ B"];
    headers.extend(col_labels.iter().copied());
    let mut grid_rows = Vec::new();
    for (i, row) in result.rows.iter().enumerate() {
        let mut cells = vec![row.abbreviation().to_string()];
        for j in 0..result.cols.len() {
            cells.push(match result.cell(i, j) {
                Some(hit) => format!(
                    "{} {}",
                    aspect_abbreviation(hit.aspect_type),
                    num(hit.orb_deg, 2)
                ),
                None => "-".to_string(),
            });
        }
        grid_rows.push(cells);
    }
    let legend: Vec<String> = kinds
        .iter()
        .map(|k| format!("{} {:?}", aspect_abbreviation(*k), k))
        .collect();

    let mut text = String::from("Synastry (tropical zodiac)\n");
    text.push_str(&birth_line("A", &a));
    text.push('\n');
    text.push_str(&birth_line("B", &b));
    text.push('\n');
    text.push_str(&format!(
        "House system: {system}   orb multiplier: {}   {} aspects found\n\n",
        num(args.orb_mult, 2),
        result.aspects().count()
    ));
    text.push_str(
        "Aspects (rows are A, columns are B; each cell is the aspect and its orb in degrees)\n",
    );
    text.push_str(&table(&headers, &grid_rows));
    text.push_str(&format!("Key: {}\n\n", legend.join(", ")));
    text.push_str("B's points in A's houses\n");
    text.push_str(&overlay_table(&result.b_in_a));
    text.push_str("\nA's points in B's houses\n");
    text.push_str(&overlay_table(&result.a_in_b));
    text.push_str(&skipped_note(&skipped));

    let mut json = serde_json::to_value(&result)
        .map_err(|e| CliError::compute(format!("cannot encode the synastry: {e}")))?;
    json["command"] = json!("synastry");
    json["zodiac"] = json!("tropical");
    json["a"] = birth_json(&a);
    json["b"] = birth_json(&b);
    json["house_system"] = json!(format!("{system:?}"));
    json["house_system_name"] = json!(system.to_string());
    json["aspect_types"] = json!(kinds.iter().map(|k| format!("{k:?}")).collect::<Vec<_>>());
    json["orb_multiplier"] = json!(args.orb_mult);
    json["include_angles"] = json!(args.angles);
    json["skipped"] = skipped_value(&skipped);
    Ok(Report { json, text })
}
