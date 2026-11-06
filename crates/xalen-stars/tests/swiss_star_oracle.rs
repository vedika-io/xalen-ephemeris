//! Swiss fixed-star oracle over the PUBLIC `find_by_name` API.
//!
//! Requires the `bright-catalog` feature (default-on); the table is absent
//! without it.
#![cfg(feature = "bright-catalog")]
//!
//! WHAT THIS PROVES
//! ----------------
//! The reference longitudes/latitudes in `tests/common/mod.rs` are real
//! `pyswisseph` 2.10.3.2 (`libswe` 2.10.03) `swe.fixstar2` output at J2000.0
//! with the astrometric flag set (`SEFLG_SWIEPH | SEFLG_J2000 | SEFLG_NONUT |
//! SEFLG_NOABERR | SEFLG_NOGDEFL`). The tests compare them with the in-house
//! Bright Star Catalogue list, through the public lookups, in arcseconds.
//!
//! The 43 stars of `SWISS_ORACLE` and the 16 of `SWISS_J2000_FULL` are named
//! stars, so their positions come from the IAU Catalog of Star Names (6
//! decimals of a degree). `Al Jabhah` (eta Leonis, `SWISS_BSC_ONLY`) has no IAU
//! name; its position comes from the Bright Star Catalogue (RA to 0.1 s, Dec to
//! 1") and has its own, looser bound.
//!
//! `SWISS_EPOCH_ORACLE` holds live Swiss places of seven stars at seven epochs
//! (1000 to 3000 CE), checked against `BrightStar::ecliptic_at_epoch`.
//!
//! The measured separations are printed with a `MEASURED` prefix (run the test
//! binary with `--nocapture`).

mod common;

use common::{SWISS_BSC_ONLY, SWISS_EPOCH_ORACLE, SWISS_J2000_FULL, SWISS_ORACLE, sep_arcsec};
use xalen_stars::{find_bright_by_name, find_by_name};

/// Bound for the IAU-positioned named stars. Measured worst case 0.245"
/// (Sirius); the bound sits a little above it.
const IAU_BOUND_ARCSEC: f64 = 0.30;
/// Bound for the median separation (measured 0.03").
const IAU_MEDIAN_BOUND_ARCSEC: f64 = 0.06;
/// Bound for the BSC-positioned star. Measured 0.79"; its source position is
/// rounded to RA 0.1 s and Dec 1".
const BSC_ONLY_BOUND_ARCSEC: f64 = 1.0;
/// Bounds for the epoch propagation. Measured worst 20.6" (Sirius at 3000 CE:
/// the Bright Star Catalogue proper motion differs from Swiss's by 18 mas/yr),
/// median 0.72".
const EPOCH_BOUND_ARCSEC: f64 = 22.0;
const EPOCH_MEDIAN_BOUND_ARCSEC: f64 = 1.0;

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

/// The public `find_by_name` place of each of the 38 oracle stars is within
/// `IAU_BOUND_ARCSEC` of live Swiss.
#[test]
fn oracle_find_by_name_matches_swiss() {
    let mut seps = Vec::new();
    let mut worst = (0.0_f64, "");
    for o in SWISS_ORACLE {
        let star = find_by_name(o.name).unwrap_or_else(|| panic!("{} missing", o.name));
        let sep = sep_arcsec(
            star.longitude_j2000,
            star.latitude_j2000,
            o.swiss_lon,
            o.swiss_lat,
        );
        if sep > worst.0 {
            worst = (sep, o.name);
        }
        assert!(
            sep < IAU_BOUND_ARCSEC,
            "{} is {sep:.3}\" from Swiss J2000 (bound {IAU_BOUND_ARCSEC}\")",
            o.name
        );
        seps.push(sep);
    }
    let med = median(&mut seps);
    println!(
        "MEASURED find_by_name vs Swiss J2000 ({} IAU-positioned stars): worst {:.3}\" ({}), median {:.3}\"",
        SWISS_ORACLE.len(),
        worst.0,
        worst.1,
        med
    );
    assert!(med < IAU_MEDIAN_BOUND_ARCSEC, "median {med:.3}\"");
}

/// Same check at full reference precision, through the catalogue lookup.
#[test]
fn full_precision_anchors_match_swiss() {
    let mut seps = Vec::new();
    let mut worst = (0.0_f64, "");
    for &(name, slon, slat) in SWISS_J2000_FULL {
        let star =
            find_bright_by_name(name).unwrap_or_else(|| panic!("{name} missing from catalogue"));
        assert_eq!(star.name, Some(name));
        let sep = sep_arcsec(star.longitude_j2000, star.latitude_j2000, slon, slat);
        if sep > worst.0 {
            worst = (sep, name);
        }
        assert!(
            sep < IAU_BOUND_ARCSEC,
            "{name} (HR {}) is {sep:.3}\" from Swiss J2000 (bound {IAU_BOUND_ARCSEC}\")",
            star.hr
        );
        seps.push(sep);
    }
    let med = median(&mut seps);
    println!(
        "MEASURED catalogue vs Swiss J2000 full precision ({} stars): worst {:.3}\" ({}), median {:.3}\"",
        SWISS_J2000_FULL.len(),
        worst.0,
        worst.1,
        med
    );
    assert!(med < IAU_MEDIAN_BOUND_ARCSEC);
}

/// The BSC-positioned star among the references (Al Jabhah, no IAU row). Its
/// source position is rounded to RA 0.1 s and Dec 1" (up to about 0.9" of
/// rounding in each axis).
#[test]
fn bsc_only_star_within_bound_of_swiss() {
    for o in SWISS_BSC_ONLY {
        let star = find_by_name(o.name).unwrap_or_else(|| panic!("{} missing", o.name));
        let sep = sep_arcsec(
            star.longitude_j2000,
            star.latitude_j2000,
            o.swiss_lon,
            o.swiss_lat,
        );
        println!(
            "MEASURED {} (BSC-positioned) vs Swiss J2000: {sep:.3}\"",
            o.name
        );
        assert!(
            sep < BSC_ONLY_BOUND_ARCSEC,
            "{} is {sep:.3}\" from Swiss J2000 (bound {BSC_ONLY_BOUND_ARCSEC}\")",
            o.name
        );
    }
}

/// Epoch propagation (IAU 2006/P03 precession plus the catalogue proper
/// motion) against live Swiss at 1000, 1600, 1900, 2000, 2100, 2400 and 3000.
#[test]
fn epoch_propagation_matches_swiss() {
    let mut seps: Vec<f64> = Vec::new();
    let mut worst = (0.0_f64, String::new());
    let mut per_epoch: std::collections::BTreeMap<i64, f64> = std::collections::BTreeMap::new();
    for &(name, year, slon, slat) in SWISS_EPOCH_ORACLE {
        let star = find_bright_by_name(name).unwrap_or_else(|| panic!("{name} missing"));
        let (lon, lat) = star.ecliptic_at_epoch(year);
        let sep = sep_arcsec(lon, lat, slon, slat);
        if sep > worst.0 {
            worst = (sep, format!("{name} {year:.0}"));
        }
        let e = per_epoch.entry(year as i64).or_insert(0.0);
        *e = e.max(sep);
        assert!(
            sep < EPOCH_BOUND_ARCSEC,
            "{name} at {year:.0}: {sep:.2}\" from Swiss (bound {EPOCH_BOUND_ARCSEC}\")"
        );
        seps.push(sep);
    }
    let med = median(&mut seps);
    println!(
        "MEASURED epoch propagation vs Swiss ({} star/epoch pairs): worst {:.3}\" ({}), median {:.3}\"",
        SWISS_EPOCH_ORACLE.len(),
        worst.0,
        worst.1,
        med
    );
    for (y, w) in &per_epoch {
        println!("MEASURED   epoch {y}: worst {w:.3}\"");
    }
    assert!(med < EPOCH_MEDIAN_BOUND_ARCSEC, "median {med:.3}\"");
}
