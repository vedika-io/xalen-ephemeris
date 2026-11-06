//! Hipparcos-derived catalogue vs live Swiss `fixstar2` at J2000.0 (moved from
//! `xalen-stars` and the workspace root tests).
//!
//! Reference values: `crates/xalen-stars/tests/common/mod.rs` (pyswisseph
//! 2.10.3.2, flags `SEFLG_SWIEPH | SEFLG_J2000 | SEFLG_NONUT | SEFLG_NOABERR |
//! SEFLG_NOGDEFL`). Reproduce with
//! `SE_EPHE_PATH=<dir with sefstars.txt> python3 tools/gen-star-catalog/validate_vs_swiss.py`.

#[path = "../../xalen-stars/tests/common/mod.rs"]
mod common;

use common::{SWISS_BSC_ONLY, SWISS_J2000_FULL, SWISS_ORACLE, sep_arcsec};
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
        "MEASURED Hipparcos-derived vs Swiss (43 stars): worst {:.3}\" ({})",
        worst.0, worst.1
    );
}

/// The star whose place the default list takes from the Bright Star Catalogue
/// (Al Jabhah): the Hipparcos-derived place against live Swiss.
#[test]
fn bsc_only_reference_star_vs_hipparcos() {
    for o in SWISS_BSC_ONLY {
        let star = find_generated_by_name(o.name).unwrap_or_else(|| panic!("{}", o.name));
        let sep = sep_arcsec(
            star.longitude_j2000,
            star.latitude_j2000,
            o.swiss_lon,
            o.swiss_lat,
        );
        println!(
            "MEASURED Hipparcos-derived {} vs Swiss reference: {sep:.3}\"",
            o.name
        );
        assert!(sep < 0.05, "{}: {sep:.3}\"", o.name);
    }
}
