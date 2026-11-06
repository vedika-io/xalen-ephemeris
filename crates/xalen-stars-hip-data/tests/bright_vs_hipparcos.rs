//! The in-house Bright Star Catalogue list (`xalen-stars`) against the
//! Hipparcos-derived data of this crate. Hipparcos values are used here as a
//! cross-check only; none of them is copied into `BRIGHT_CATALOG`.
//!
//! * Position statistics for every bright star that has a Hipparcos counterpart
//!   (nearest star within 60" and 0.5 mag), split into named stars (IAU-CSN
//!   positions, 6 decimals of a degree) and unnamed stars (Bright Star
//!   Catalogue positions: RA to 0.1 s, Dec to 1").
//! * Multi-epoch agreement of the epoch propagation at 1600, 1900, 2000, 2100
//!   and 2400 for the seven stars of the Swiss multi-epoch test.

use xalen_stars::{BRIGHT_CATALOG, find_bright_by_name};
use xalen_stars_hip_data::{GENERATED_CATALOG, GeneratedStarExt, find_generated_by_name};

#[path = "../../xalen-stars/tests/common/mod.rs"]
mod common;

use common::sep_arcsec;

fn percentile(sorted: &[f64], p: f64) -> f64 {
    let i = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[i]
}

struct Stats {
    matched: usize,
    unmatched: usize,
    median: f64,
    p95: f64,
    p99: f64,
    max: f64,
    worst_hr: u32,
}

fn compare(named: bool) -> Stats {
    let mut seps: Vec<f64> = Vec::new();
    let mut unmatched = 0;
    let mut worst = (0.0_f64, 0_u32);
    for b in BRIGHT_CATALOG.iter().filter(|b| b.name.is_some() == named) {
        let mut best: Option<f64> = None;
        for h in GENERATED_CATALOG {
            if (h.magnitude - b.magnitude).abs() > 0.5 {
                continue;
            }
            if (h.latitude_j2000 - b.latitude_j2000).abs() > 60.0 / 3600.0 {
                continue;
            }
            let s = sep_arcsec(
                b.longitude_j2000,
                b.latitude_j2000,
                h.longitude_j2000,
                h.latitude_j2000,
            );
            if s <= 60.0 && best.is_none_or(|x| s < x) {
                best = Some(s);
            }
        }
        match best {
            Some(s) => {
                if s > worst.0 {
                    worst = (s, b.hr);
                }
                seps.push(s);
            }
            None => unmatched += 1,
        }
    }
    seps.sort_by(|a, b| a.partial_cmp(b).unwrap());
    Stats {
        matched: seps.len(),
        unmatched,
        median: percentile(&seps, 0.5),
        p95: percentile(&seps, 0.95),
        p99: percentile(&seps, 0.99),
        max: worst.0,
        worst_hr: worst.1,
    }
}

#[test]
fn named_bright_stars_agree_with_hipparcos() {
    let s = compare(true);
    println!(
        "MEASURED named bright stars vs Hipparcos-derived: matched {} unmatched {} median {:.3}\" p95 {:.3}\" p99 {:.3}\" max {:.3}\" (HR {})",
        s.matched, s.unmatched, s.median, s.p95, s.p99, s.max, s.worst_hr
    );
    // Measured: 326 matched, median 0.008", p95 0.045", p99 0.263", max 1.344"
    // (HR 5460, Toliman, a resolved component of alpha Centauri).
    assert!(s.matched >= 320, "matched {}", s.matched);
    assert!(s.median < 0.02, "median {:.3}\"", s.median);
    assert!(s.p95 < 0.08, "p95 {:.3}\"", s.p95);
    assert!(s.p99 < 0.4, "p99 {:.3}\"", s.p99);
    assert!(s.max < 1.6, "max {:.3}\" (HR {})", s.max, s.worst_hr);
}

#[test]
fn unnamed_bright_stars_agree_with_hipparcos() {
    let s = compare(false);
    println!(
        "MEASURED unnamed (BSC-positioned) bright stars vs Hipparcos-derived: matched {} unmatched {} median {:.3}\" p95 {:.3}\" p99 {:.3}\" max {:.3}\" (HR {})",
        s.matched, s.unmatched, s.median, s.p95, s.p99, s.max, s.worst_hr
    );
    // Measured: 7,952 matched (805 have no counterpart within 60" and 0.5 mag),
    // median 0.643", p95 2.264", p99 3.418", max 38.7" (HR 1754). The tail is
    // close binaries and high-proper-motion stars, where the Bright Star
    // Catalogue and Hipparcos refer to different photocentres.
    assert!(s.matched >= 7900, "matched {}", s.matched);
    assert!(s.median < 0.75, "median {:.3}\"", s.median);
    assert!(s.p95 < 2.6, "p95 {:.3}\"", s.p95);
    assert!(s.p99 < 4.0, "p99 {:.3}\"", s.p99);
}

/// Epoch propagation of the same seven stars as the Swiss multi-epoch test
/// (Polaris, Aldebaran, Spica, Antares, Regulus, Sirius, Pollux), at 1600,
/// 1900, 2000, 2100 and 2400, bright list vs Hipparcos-derived data.
#[test]
fn multi_epoch_bright_vs_hipparcos() {
    let names = [
        "Polaris",
        "Aldebaran",
        "Spica",
        "Antares",
        "Regulus",
        "Sirius",
        "Pollux",
    ];
    let epochs = [1600.0, 1900.0, 2000.0, 2100.0, 2400.0];
    let mut worst = (0.0_f64, "", 0.0_f64);
    let mut all = Vec::new();
    for n in names {
        let b = find_bright_by_name(n).unwrap();
        let h = find_generated_by_name(n).unwrap();
        for y in epochs {
            let (bl, bb) = b.ecliptic_at_epoch(y);
            let (hl, hb) = h.ecliptic_at_epoch(y);
            let s = sep_arcsec(bl, bb, hl, hb);
            if s > worst.0 {
                worst = (s, n, y);
            }
            all.push(s);
        }
    }
    all.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!(
        "MEASURED multi-epoch (7 stars x 5 epochs) bright vs Hipparcos-derived: worst {:.3}\" ({} at {}), median {:.3}\"",
        worst.0,
        worst.1,
        worst.2,
        percentile(&all, 0.5)
    );
    // Measured: worst 7.757" (Sirius at 1600), median 0.391".
    assert!(
        worst.0 < 9.0,
        "worst {:.3}\" ({} at {})",
        worst.0,
        worst.1,
        worst.2
    );
}
