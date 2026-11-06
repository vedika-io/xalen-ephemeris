//! Hipparcos-derived catalogue vs live Swiss `fixstar2` at J2000.0 (moved from
//! `xalen-stars` and the workspace root tests).
//!
//! Reference values: `crates/xalen-stars/tests/common/mod.rs` (pyswisseph
//! 2.10.3.2, flags `SEFLG_SWIEPH | SEFLG_J2000 | SEFLG_NONUT | SEFLG_NOABERR |
//! SEFLG_NOGDEFL`). Reproduce with
//! `SE_EPHE_PATH=<dir with sefstars.txt> python3 tools/gen-star-catalog/validate_vs_swiss.py`.

#[path = "../../xalen-stars/tests/common/mod.rs"]
mod common;

use common::{SWISS_J2000_FULL, SWISS_ORACLE, SWISS_ORACLE_ALIASED, sep_arcsec};
use xalen_stars_hip_data::find_generated_by_name;

#[test]
fn named_stars_match_swiss_subarcsecond() {
    let mut worst = (0.0_f64, "");
    for &(name, slon, slat) in SWISS_J2000_FULL {
        let star = find_generated_by_name(name).unwrap_or_else(|| panic!("{name} missing"));
        let sep = sep_arcsec(star.longitude_j2000, star.latitude_j2000, slon, slat);
        if sep > worst.0 {
            worst = (sep, name);
        }
        assert!(
            sep < 1.0,
            "{name} (HIP {}) is {sep:.3}\" from Swiss J2000",
            star.hip
        );
    }
    println!(
        "MEASURED Hipparcos-derived vs Swiss full precision: worst {:.3}\" ({})",
        worst.0, worst.1
    );
    assert!(
        worst.0 < 0.5,
        "worst anchor separation {:.3}\" ({})",
        worst.0,
        worst.1
    );
}

#[test]
fn oracle_generated_catalog_matches_swiss() {
    let mut worst = (0.0_f64, "");
    for o in SWISS_ORACLE {
        let Some(star) = find_generated_by_name(o.name) else {
            panic!("{} not a named generated star", o.name);
        };
        let sep = sep_arcsec(
            star.longitude_j2000,
            star.latitude_j2000,
            o.swiss_lon,
            o.swiss_lat,
        );
        if sep > worst.0 {
            worst = (sep, o.name);
        }
        assert!(sep < 1.0, "{} is {sep:.3}\" from Swiss", o.name);
    }
    println!(
        "MEASURED Hipparcos-derived vs Swiss (38 stars): worst {:.3}\" ({})",
        worst.0, worst.1
    );
}

/// The six aliased stars: the Hipparcos-derived data disagrees with the same
/// reference values as the Bright Star Catalogue list does (2" to 27"), which
/// shows the disagreement is in the reference values.
#[test]
fn aliased_reference_disagreements_are_shared_with_hipparcos() {
    let pinned: &[(&str, f64)] = &[
        ("Zuben Eschamali", 26.98),
        ("Zuben Elgenubi", 25.75),
        ("Lambda Orionis", 6.57),
        ("Hyadum II", 3.55),
        ("Bharani 41", 3.11),
        ("Al Jabhah", 1.32),
    ];
    for o in SWISS_ORACLE_ALIASED {
        let star = find_generated_by_name(o.name).unwrap_or_else(|| panic!("{}", o.name));
        let sep = sep_arcsec(
            star.longitude_j2000,
            star.latitude_j2000,
            o.swiss_lon,
            o.swiss_lat,
        );
        let &(_, expected) = pinned.iter().find(|(n, _)| *n == o.name).unwrap();
        println!(
            "MEASURED Hipparcos-derived {} vs Swiss reference: {sep:.3}\"",
            o.name
        );
        assert!(
            (sep - expected).abs() < 0.3,
            "{}: {sep:.3}\" vs pinned {expected}\"",
            o.name
        );
    }
}
