//! Swiss fixed-star oracle over the PUBLIC `find_by_name` API.
//!
//! WHAT THIS PROVES
//! ----------------
//! Every reference longitude/latitude below is a REAL value produced by the
//! Swiss Ephemeris (`pyswisseph` 2.10.3.2, `libswe` 2.10.03) — it is NOT
//! invented or hand-computed. Each row is the J2000.0 *mean* ecliptic place of
//! the star, queried with the astrometric flag set
//!
//!     SEFLG_SWIEPH | SEFLG_J2000 | SEFLG_NONUT | SEFLG_NOABERR | SEFLG_NOGDEFL
//!
//! at JD 2451545.0 (TT) using the official Swiss `sefstars.txt` star data file
//! (S. Moshier base, extended by Abramov & Koch). This is the SAME oracle, with
//! the SAME flags and epoch, that `tools/gen-star-catalog/validate_vs_swiss.py`
//! uses to validate the generated catalog — so these numbers are the catalog's
//! own ground truth.
//!
//! To regenerate (requires the Swiss star file on disk):
//!
//! ```text
//!   curl -sL https://raw.githubusercontent.com/aloistr/swisseph/master/ephe/sefstars.txt \
//!     -o /tmp/ephe/sefstars.txt
//!   python3 - <<'PY'
//!   import swisseph as swe
//!   swe.set_ephe_path("/tmp/ephe")
//!   jd = 2451545.0
//!   fl = swe.FLG_SWIEPH|swe.FLG_J2000|swe.FLG_NONUT|swe.FLG_NOABERR|swe.FLG_NOGDEFL
//!   for n in ["Polaris","Sirius","Aldebaran","Regulus","Spica","Antares","Vega",
//!             "Deneb","Rigel","Betelgeuse","Arcturus","Fomalhaut","Canopus",
//!             "Capella","Procyon","Altair","Castor","Pollux","Algol","Bellatrix",
//!             "Alnilam","Elnath","Alphard","Denebola","Hamal","Achernar",
//!             "Alpheratz","Mirach","Acrux","Shaula","Nunki","Markab","Scheat",
//!             "Algenib","Dubhe","Alioth","Mizar","Diphda"]:
//!       xx,_,_ = swe.fixstar2(n, jd, fl)
//!       print(f"{n} {xx[0]:.6f} {xx[1]:.6f}")
//!   PY
//! ```
//!
//! THE BUG THIS EXPOSES (orphaned validated catalog)
//! -------------------------------------------------
//! The public `find_by_name(...)` reads the hand-curated `CATALOG`. Several of
//! its entries carry GROSS J2000 ecliptic-longitude errors — Polaris is listed
//! at ~28.6 deg (true ~88.6 deg, ~60 deg error), Deneb at ~305.2 deg
//! (true ~335.3 deg, ~30 deg error), Dubhe at ~166.6 deg (true ~135.2 deg),
//! Mizar/Alioth ~18-20 deg off, plus many sub-degree errors.
//!
//! Meanwhile the validated 8,870-star `GENERATED_CATALOG` (Hipparcos-derived,
//! sub-arcsecond vs Swiss) carries the CORRECT positions for these same named
//! stars — e.g. `find_generated_by_name("Polaris")` returns 88.5676/66.1015,
//! matching Swiss to ~0.00001 deg. But `find_by_name` never consults it.
//!
//! So the data is right and orphaned; the public lookup is wrong. These tests
//! assert against the PUBLIC `find_by_name` surface and therefore FAIL on the
//! curated-catalog stars until `find_by_name` is routed through (or reconciled
//! with) the generated catalog. `oracle_generated_catalog_matches_swiss` proves
//! the correct data is already present, so the fix is a routing fix, not a data
//! collection effort.

use xalen_stars::{find_by_name, find_generated_by_name};

/// Match tolerance: 0.05 deg = 180 arcsec. Generous on purpose — the validated
/// catalog matches Swiss to sub-arcsecond, so anything failing this is a real
/// catalog defect, not numerical noise.
const TOL_DEG: f64 = 0.05;

/// A Swiss-Ephemeris oracle row: (star name, J2000 ecliptic longitude deg,
/// J2000 ecliptic latitude deg). Every value is LIVE `swe.fixstar2` output —
/// see the module header for the exact flags, epoch and star-file provenance.
struct Oracle {
    name: &'static str,
    swiss_lon: f64,
    swiss_lat: f64,
}

/// 38 well-known named stars with their LIVE Swiss J2000 mean-ecliptic places.
const SWISS_ORACLE: &[Oracle] = &[
    Oracle {
        name: "Polaris",
        swiss_lon: 88.567582,
        swiss_lat: 66.101473,
    },
    Oracle {
        name: "Sirius",
        swiss_lon: 104.081678,
        swiss_lat: -39.605304,
    },
    Oracle {
        name: "Aldebaran",
        swiss_lon: 69.789184,
        swiss_lat: -5.467321,
    },
    Oracle {
        name: "Regulus",
        swiss_lon: 149.829145,
        swiss_lat: 0.464849,
    },
    Oracle {
        name: "Spica",
        swiss_lon: 203.841363,
        swiss_lat: -2.054487,
    },
    Oracle {
        name: "Antares",
        swiss_lon: 249.762295,
        swiss_lat: -4.569952,
    },
    Oracle {
        name: "Vega",
        swiss_lon: 285.316389,
        swiss_lat: 61.732822,
    },
    Oracle {
        name: "Deneb",
        swiss_lon: 335.329214,
        swiss_lat: 59.906140,
    },
    Oracle {
        name: "Rigel",
        swiss_lon: 76.829594,
        swiss_lat: -31.122760,
    },
    Oracle {
        name: "Betelgeuse",
        swiss_lon: 88.754602,
        swiss_lat: -16.027000,
    },
    Oracle {
        name: "Arcturus",
        swiss_lon: 204.233651,
        swiss_lat: 30.736231,
    },
    Oracle {
        name: "Fomalhaut",
        swiss_lon: 333.860228,
        swiss_lat: -21.135617,
    },
    Oracle {
        name: "Canopus",
        swiss_lon: 104.960545,
        swiss_lat: -75.823875,
    },
    Oracle {
        name: "Capella",
        swiss_lon: 81.857911,
        swiss_lat: 22.864282,
    },
    Oracle {
        name: "Procyon",
        swiss_lon: 115.785561,
        swiss_lat: -16.019629,
    },
    Oracle {
        name: "Altair",
        swiss_lon: 301.776395,
        swiss_lat: 29.303426,
    },
    Oracle {
        name: "Castor",
        swiss_lon: 110.240210,
        swiss_lat: 10.095768,
    },
    Oracle {
        name: "Pollux",
        swiss_lon: 113.215648,
        swiss_lat: 6.684202,
    },
    Oracle {
        name: "Algol",
        swiss_lon: 56.167575,
        swiss_lat: 22.428536,
    },
    Oracle {
        name: "Bellatrix",
        swiss_lon: 80.946467,
        swiss_lat: -16.816035,
    },
    Oracle {
        name: "Alnilam",
        swiss_lon: 83.463651,
        swiss_lat: -24.506375,
    },
    Oracle {
        name: "Elnath",
        swiss_lon: 82.574936,
        swiss_lat: 5.385117,
    },
    Oracle {
        name: "Alphard",
        swiss_lon: 147.279215,
        swiss_lat: -22.382383,
    },
    Oracle {
        name: "Denebola",
        swiss_lon: 171.617559,
        swiss_lat: 12.266884,
    },
    Oracle {
        name: "Hamal",
        swiss_lon: 37.662469,
        swiss_lat: 9.965113,
    },
    Oracle {
        name: "Achernar",
        swiss_lon: 345.311280,
        swiss_lat: -59.378150,
    },
    Oracle {
        name: "Alpheratz",
        swiss_lon: 14.308460,
        swiss_lat: 25.680439,
    },
    Oracle {
        name: "Mirach",
        swiss_lon: 30.405224,
        swiss_lat: 25.943362,
    },
    Oracle {
        name: "Acrux",
        swiss_lon: 221.869909,
        swiss_lat: -52.878864,
    },
    Oracle {
        name: "Shaula",
        swiss_lon: 264.585713,
        swiss_lat: -13.788463,
    },
    Oracle {
        name: "Nunki",
        swiss_lon: 282.385331,
        swiss_lat: -3.449530,
    },
    Oracle {
        name: "Markab",
        swiss_lon: 353.485597,
        swiss_lat: 19.406022,
    },
    Oracle {
        name: "Scheat",
        swiss_lon: 359.374173,
        swiss_lat: 31.140508,
    },
    Oracle {
        name: "Algenib",
        swiss_lon: 9.156034,
        swiss_lat: 12.599925,
    },
    Oracle {
        name: "Dubhe",
        swiss_lon: 135.197397,
        swiss_lat: 49.680317,
    },
    Oracle {
        name: "Alioth",
        swiss_lon: 158.933501,
        swiss_lat: 54.318808,
    },
    Oracle {
        name: "Mizar",
        swiss_lon: 165.700165,
        swiss_lat: 56.378929,
    },
    Oracle {
        name: "Diphda",
        swiss_lon: 2.583502,
        swiss_lat: -20.783547,
    },
    // Formerly-fallback stars: curated names that did not exact-match the
    // IAU-WGSN name in the generated catalog, so `find_by_name` used to return
    // the coarse curated coordinates. After the curated->IAU alias fix they
    // resolve to the validated Hipparcos row and match Swiss within TOL_DEG.
    Oracle {
        name: "Bharani 41",
        swiss_lon: 48.203266,
        swiss_lat: 10.450526,
    },
    Oracle {
        name: "Lambda Orionis",
        swiss_lon: 83.708661,
        swiss_lat: -13.369833,
    },
    Oracle {
        name: "Hyadum II",
        swiss_lon: 66.871684,
        swiss_lat: -3.969725,
    },
    Oracle {
        name: "Zuben Elgenubi",
        swiss_lon: 225.075512,
        swiss_lat: 0.333043,
    },
    Oracle {
        name: "Zuben Eschamali",
        swiss_lon: 229.364142,
        swiss_lat: 8.495271,
    },
    Oracle {
        name: "Al Jabhah",
        swiss_lon: 147.905168,
        swiss_lat: 4.865355,
    },
];

/// Signed shortest angular separation in longitude (handles the 0/360 wrap).
fn lon_delta(a: f64, b: f64) -> f64 {
    ((a - b + 180.0).rem_euclid(360.0)) - 180.0
}

/// PUBLIC-API ORACLE.
///
/// For each well-known star, assert that the public `find_by_name(...)` J2000
/// ecliptic longitude/latitude match the LIVE Swiss oracle within `TOL_DEG`.
///
/// EXPECTED TO FAIL against the current code: `find_by_name` reads the curated
/// `CATALOG`, whose Polaris/Deneb/Dubhe/Mizar/Alioth/Rigel (and ~20 sub-degree)
/// entries are wrong. The single panic lists every offender with its delta.
#[test]
fn oracle_find_by_name_matches_swiss() {
    let mut failures: Vec<String> = Vec::new();

    for o in SWISS_ORACLE {
        let star = match find_by_name(o.name) {
            Some(s) => s,
            None => {
                failures.push(format!("{:<11} MISSING from public CATALOG", o.name));
                continue;
            }
        };

        let dlon = lon_delta(star.longitude_j2000, o.swiss_lon).abs();
        let dlat = (star.latitude_j2000 - o.swiss_lat).abs();

        if dlon > TOL_DEG || dlat > TOL_DEG {
            failures.push(format!(
                "{:<11} dLon={:+8.4} dLat={:+8.4}  (catalog {:.4}/{:.4} vs Swiss {:.4}/{:.4})",
                o.name,
                lon_delta(star.longitude_j2000, o.swiss_lon),
                star.latitude_j2000 - o.swiss_lat,
                star.longitude_j2000,
                star.latitude_j2000,
                o.swiss_lon,
                o.swiss_lat,
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} named stars disagree with the Swiss J2000 oracle by > {} deg \
         (public find_by_name reads the orphaned curated CATALOG, not the \
         validated GENERATED_CATALOG):\n  {}",
        failures.len(),
        SWISS_ORACLE.len(),
        TOL_DEG,
        failures.join("\n  ")
    );
}

/// CONTROL: prove the CORRECT data already exists, orphaned, in the validated
/// `GENERATED_CATALOG`.
///
/// For every oracle star that ALSO carries a name in the generated catalog,
/// `find_generated_by_name(...)` must match Swiss within `TOL_DEG`. This is the
/// near-zero-residual control: it demonstrates the bug in
/// `oracle_find_by_name_matches_swiss` is a *lookup routing* defect, not a data
/// problem. (Stars not present as named entries in the generated catalog are
/// skipped — they are not asserted, only counted.)
#[test]
fn oracle_generated_catalog_matches_swiss() {
    let mut failures: Vec<String> = Vec::new();
    let mut checked = 0usize;

    for o in SWISS_ORACLE {
        let Some(star) = find_generated_by_name(o.name) else {
            continue; // not a named HIP join — not part of this control set
        };
        checked += 1;

        let dlon = lon_delta(star.longitude_j2000, o.swiss_lon).abs();
        let dlat = (star.latitude_j2000 - o.swiss_lat).abs();

        if dlon > TOL_DEG || dlat > TOL_DEG {
            failures.push(format!(
                "{:<11} dLon={:+8.4} dLat={:+8.4}",
                o.name,
                lon_delta(star.longitude_j2000, o.swiss_lon),
                star.latitude_j2000 - o.swiss_lat,
            ));
        }
    }

    assert!(checked > 0, "no named stars resolved in GENERATED_CATALOG");
    assert!(
        failures.is_empty(),
        "{} named generated stars disagree with Swiss (the validated catalog \
         should match sub-arcsecond):\n  {}",
        failures.len(),
        failures.join("\n  ")
    );
}
