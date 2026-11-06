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
//! The 38 stars of `SWISS_ORACLE` and the 16 of `SWISS_J2000_FULL` are named
//! stars, so their positions come from the IAU Catalog of Star Names (6
//! decimals of a degree). Measured worst separation 0.245" (Sirius), median
//! 0.03".
//!
//! `SWISS_ORACLE_ALIASED` holds six reference values (recorded by the original
//! oracle test with a 0.05 deg tolerance) that disagree with this list by 2" to
//! 27". The Hipparcos-derived data of `xalen-stars-hip-data` disagrees with the
//! same six values by the same amounts (see that crate's
//! `tests/bright_vs_hipparcos.rs`), so the disagreement is in the reference
//! values, not in the new list. They are listed here by name with their
//! measured separations instead of being folded into the sub-arcsecond bound.

mod common;

use common::{SWISS_J2000_FULL, SWISS_ORACLE, SWISS_ORACLE_ALIASED, sep_arcsec};
use xalen_stars::{find_bright_by_name, find_by_name};

/// Bound for the IAU-positioned named stars. Measured worst case 0.245"
/// (Sirius); the bound sits a little above it.
const IAU_BOUND_ARCSEC: f64 = 0.30;
/// Bound for the median separation (measured 0.03").
const IAU_MEDIAN_BOUND_ARCSEC: f64 = 0.06;

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

/// The six aliased/curated stars whose reference values disagree with this list
/// (and with the Hipparcos-derived data). Measured separations are pinned so any
/// change of the underlying data is noticed; they are NOT sub-arcsecond and are
/// reported as such. `Al Jabhah` is the only one positioned from the Bright
/// Star Catalogue itself (RA to 0.1 s, Dec to 1") because it has no IAU name.
#[test]
fn aliased_stars_known_reference_disagreements_are_pinned() {
    // (name, measured separation in arcsec, tolerance in arcsec)
    let pinned: &[(&str, f64, f64)] = &[
        ("Zuben Eschamali", 26.97, 0.2),
        ("Zuben Elgenubi", 25.75, 0.2),
        ("Lambda Orionis", 6.58, 0.2),
        ("Hyadum II", 3.56, 0.2),
        ("Bharani 41", 3.12, 0.2),
        ("Al Jabhah", 1.95, 0.2),
    ];
    assert_eq!(pinned.len(), SWISS_ORACLE_ALIASED.len());
    for o in SWISS_ORACLE_ALIASED {
        let star = find_by_name(o.name).unwrap_or_else(|| panic!("{} missing", o.name));
        let sep = sep_arcsec(
            star.longitude_j2000,
            star.latitude_j2000,
            o.swiss_lon,
            o.swiss_lat,
        );
        let &(_, expected, tol) = pinned
            .iter()
            .find(|(n, _, _)| *n == o.name)
            .unwrap_or_else(|| panic!("{} not pinned", o.name));
        println!("MEASURED {} vs Swiss reference: {sep:.3}\"", o.name);
        assert!(
            (sep - expected).abs() < tol,
            "{}: separation {sep:.3}\" differs from pinned {expected}\" +/- {tol}\"",
            o.name
        );
    }
}

/// The BSC-only star among the references (Al Jabhah, no IAU row). Its source
/// position is rounded to RA 0.1 s and Dec 1" (up to about 0.9" of rounding);
/// the measured separation vs Swiss is 1.95".
#[test]
fn bsc_only_star_within_two_arcsec_of_swiss() {
    let o = SWISS_ORACLE_ALIASED
        .iter()
        .find(|o| o.name == "Al Jabhah")
        .unwrap();
    let star = find_by_name("Al Jabhah").unwrap();
    let sep = sep_arcsec(
        star.longitude_j2000,
        star.latitude_j2000,
        o.swiss_lon,
        o.swiss_lat,
    );
    assert!(sep < 2.5, "Al Jabhah is {sep:.3}\" from Swiss J2000");
}
