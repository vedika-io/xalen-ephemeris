//! XALEN ephemeris-precision benchmark — planetary-longitude agreement vs JPL DE440.
//!
//! This harness measures how closely XALEN's planetary positions agree with an
//! independent reference ephemeris, sampled over a dense grid of timestamps, and
//! prints a per-body deviation table in **arcseconds** (max / mean / RMS, plus the
//! worst-offending epoch). It reuses the engine's OWN public API — nothing here
//! reimplements any astronomy:
//!
//!   * `Almanac::default_vedic()` — the analytic VSOP87/ELP baseline.
//!   * `Almanac::with_de440(path)` — swaps in the DE440 SPK provider (`De440Reader` +
//!     `De440Provider`).
//!   * `Almanac::geocentric_longitude_deg(body, jd)` — apparent geocentric ecliptic
//!     longitude (deg-of-date), the quantity a chart reads.
//!
//! ## What "reference" means here (read before quoting any number)
//!
//! This is **ephemeris / astronomical precision** — agreement of computed planetary
//! ecliptic longitudes — NOT "astrology accuracy". Two reference modes:
//!
//!   1. **DE440 mode** (a JPL DE440/DE441 `.bsp` SPK kernel is present): XALEN's
//!      DE440-backed apparent longitude is compared against XALEN's analytic VSOP87
//!      longitude across the grid. Both share the same apparent-place reduction
//!      (light-time, IAU 2006 precession, IAU 2000B nutation, aberration), so the
//!      residual isolates the underlying theory difference: numerically-integrated
//!      DE440 (the JPL reference) vs the analytic VSOP87/ELP series. The committed
//!      EXTERNAL check against independently-sourced JPL Horizons state vectors lives
//!      in `crates/xalen-ephem/tests/de440_real_crossval.rs`; this binary is the
//!      complementary statistical sweep at arbitrary epoch count.
//!
//!   2. **Self-baseline mode** (no kernel): the same Almanac is sampled against itself
//!      so the harness still runs and proves the wiring; every deviation is therefore
//!      ~0 by construction and is clearly labelled `SELF-BASELINE (no kernel)` in the
//!      output. This mode is NOT a validation — it only confirms the harness executes.
//!
//! JPL DE440 is a **public reference ephemeris** published by NASA's Jet Propulsion
//! Laboratory and used here purely for comparison. Nothing in this benchmark implies
//! any endorsement, partnership, or certification by NASA or JPL.
//!
//! ## Reproduce
//!
//! ```bash
//! # 1. (DE440 mode) fetch the public DE440s SPK kernel from NAIF/JPL (~32 MB):
//! curl -L -o /tmp/de440s.bsp \
//!   https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440s.bsp
//!
//! # 2. run the benchmark (defaults: kernel /tmp/de440s.bsp, ~3650 epochs over 1900–2050):
//! cargo run -p xalen-validation --release --bin de440_bench
//!
//! # options:
//! cargo run -p xalen-validation --release --bin de440_bench -- \
//!   --kernel /tmp/de440s.bsp --start-jd 2415021 --end-jd 2488070 --step-days 5
//! ```
//!
//! Without `/tmp/de440s.bsp` the harness runs in self-baseline mode (and says so).

use std::path::Path;

use xalen_ephem::{Almanac, Body};
use xalen_time::JdUT1;

/// Default DE440 SPK kernel path — the same one `de440_real_crossval.rs` uses,
/// and the path the `curl` line above writes to. CI does not fetch it, so the
/// harness degrades to self-baseline mode there.
const DEFAULT_KERNEL: &str = "/tmp/de440s.bsp";

/// Default sampling span: 1900-01-01 .. 2050-01-01 (JD), the modern era where the
/// analytic VSOP87/ELP series is tightest and which covers essentially every
/// real birth chart. Widen it on the command line to probe the far past/future.
const DEFAULT_START_JD: f64 = 2_415_021.0; // ~1900-01-01
const DEFAULT_END_JD: f64 = 2_469_807.0; // ~2050-01-01 (approx; exact value not load-bearing)
const DEFAULT_STEP_DAYS: f64 = 15.0;

/// Bodies whose longitude we benchmark. These are the bodies the DE440 SPK kernel
/// serves directly (Sun, Moon, the planets); the lunar nodes/apogees are abstract
/// derived points that fall back to the analytic model in BOTH almanacs, so a
/// DE440-vs-VSOP comparison of them would be meaningless and they are excluded.
const BENCH_BODIES: &[(Body, &str)] = &[
    (Body::Sun, "Sun"),
    (Body::Moon, "Moon"),
    (Body::Mercury, "Mercury"),
    (Body::Venus, "Venus"),
    (Body::Mars, "Mars"),
    (Body::Jupiter, "Jupiter"),
    (Body::Saturn, "Saturn"),
    (Body::Uranus, "Uranus"),
    (Body::Neptune, "Neptune"),
    (Body::Pluto, "Pluto"),
];

/// Shortest angular separation between two longitudes, in degrees [0, 180].
/// Handles the 0/360 wrap so a body near 0° vs 359.99° reads as ~0.01°, not ~360°.
fn angle_sep_deg(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(360.0);
    if d > 180.0 { 360.0 - d } else { d }
}

/// Per-body running deviation statistics, all in arcseconds.
#[derive(Default)]
struct Deviation {
    count: u64,
    sum_arcsec: f64,
    sum_sq_arcsec: f64,
    max_arcsec: f64,
    worst_jd: f64,
}

impl Deviation {
    fn record(&mut self, delta_deg: f64, jd: f64) {
        let arcsec = delta_deg * 3600.0;
        self.count += 1;
        self.sum_arcsec += arcsec;
        self.sum_sq_arcsec += arcsec * arcsec;
        if arcsec > self.max_arcsec {
            self.max_arcsec = arcsec;
            self.worst_jd = jd;
        }
    }

    fn mean_arcsec(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum_arcsec / self.count as f64
        }
    }

    fn rms_arcsec(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            (self.sum_sq_arcsec / self.count as f64).sqrt()
        }
    }
}

struct Config {
    kernel: String,
    start_jd: f64,
    end_jd: f64,
    step_days: f64,
}

fn parse_args() -> Config {
    let mut cfg = Config {
        kernel: DEFAULT_KERNEL.to_string(),
        start_jd: DEFAULT_START_JD,
        end_jd: DEFAULT_END_JD,
        step_days: DEFAULT_STEP_DAYS,
    };
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--kernel" => {
                if let Some(v) = args.next() {
                    cfg.kernel = v;
                }
            }
            "--start-jd" => {
                if let Some(v) = args.next().and_then(|v| v.parse().ok()) {
                    cfg.start_jd = v;
                }
            }
            "--end-jd" => {
                if let Some(v) = args.next().and_then(|v| v.parse().ok()) {
                    cfg.end_jd = v;
                }
            }
            "--step-days" => {
                if let Some(v) = args.next().and_then(|v| v.parse::<f64>().ok())
                    && v > 0.0
                {
                    cfg.step_days = v;
                }
            }
            "--help" | "-h" => {
                eprintln!(
                    "usage: de440_bench [--kernel PATH] [--start-jd JD] [--end-jd JD] [--step-days N]\n\
                     \n\
                     Compares XALEN's DE440-backed apparent longitudes against the analytic\n\
                     VSOP87 baseline over a grid of epochs and prints a per-body deviation\n\
                     table in arcseconds. Fetch the public kernel first:\n  \
                     curl -L -o {DEFAULT_KERNEL} \\\n    \
                     https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440s.bsp"
                );
                std::process::exit(0);
            }
            other => {
                eprintln!("de440_bench: ignoring unrecognised argument '{other}' (try --help)");
            }
        }
    }
    cfg
}

fn main() {
    let cfg = parse_args();

    // Reference almanac: DE440 SPK if the kernel is present, else the analytic
    // baseline itself (self-baseline mode). The candidate (always-XALEN-analytic)
    // almanac is the VSOP87 default.
    let kernel_present = Path::new(&cfg.kernel).exists();
    let baseline = Almanac::default_vedic(); // VSOP87/ELP analytic — the candidate
    let reference = if kernel_present {
        Almanac::default_vedic().with_de440(Path::new(&cfg.kernel))
    } else {
        // No kernel: compare the analytic almanac against itself so the harness
        // still runs end-to-end. Every delta is ~0 and the report says so.
        Almanac::default_vedic()
    };

    println!("XALEN ephemeris-precision benchmark — planetary-longitude agreement");
    println!("===================================================================");
    if kernel_present {
        println!("reference   : JPL DE440 SPK kernel ({})", cfg.kernel);
        println!("candidate   : XALEN analytic VSOP87/ELP series");
        println!(
            "metric      : |Δ apparent geocentric ecliptic longitude| (DE440 − VSOP87), arcsec"
        );
    } else {
        println!(
            "reference   : SELF-BASELINE (no kernel found at {})",
            cfg.kernel
        );
        println!("candidate   : XALEN analytic VSOP87/ELP series (vs itself)");
        println!(
            "metric      : |Δ longitude| — ~0 by construction; this is a wiring check, NOT a validation"
        );
        println!(
            "NOTE        : fetch the public kernel for a real DE440 comparison:\n              \
             curl -L -o {} \\\n                \
             https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440s.bsp",
            cfg.kernel
        );
    }

    // Build the epoch grid.
    let mut epochs: Vec<f64> = Vec::new();
    let mut jd = cfg.start_jd;
    while jd <= cfg.end_jd {
        epochs.push(jd);
        jd += cfg.step_days;
    }
    println!(
        "span        : JD {:.1} .. {:.1}  ({} epochs, step {} days)",
        cfg.start_jd,
        cfg.end_jd,
        epochs.len(),
        cfg.step_days
    );
    println!();

    // Sweep.
    let mut devs: Vec<Deviation> = (0..BENCH_BODIES.len())
        .map(|_| Deviation::default())
        .collect();
    let mut skipped: u64 = 0;
    for &jd in &epochs {
        let jd_ut1 = JdUT1(jd);
        for (i, (body, _)) in BENCH_BODIES.iter().enumerate() {
            let r = reference.geocentric_longitude_deg(*body, jd_ut1);
            let c = baseline.geocentric_longitude_deg(*body, jd_ut1);
            match (r, c) {
                (Ok(ref_lon), Ok(cand_lon)) => {
                    devs[i].record(angle_sep_deg(ref_lon, cand_lon), jd);
                }
                _ => skipped += 1,
            }
        }
    }

    // Report.
    println!(
        "{:<10} {:>10} {:>14} {:>14} {:>14} {:>16}",
        "body", "samples", "mean(\")", "rms(\")", "max(\")", "worst@JD"
    );
    println!("{}", "-".repeat(82));
    let mut worst_overall = 0.0_f64;
    let mut worst_body = "";
    for (i, (_, name)) in BENCH_BODIES.iter().enumerate() {
        let d = &devs[i];
        println!(
            "{:<10} {:>10} {:>14.4} {:>14.4} {:>14.4} {:>16.1}",
            name,
            d.count,
            d.mean_arcsec(),
            d.rms_arcsec(),
            d.max_arcsec,
            d.worst_jd
        );
        if d.max_arcsec > worst_overall {
            worst_overall = d.max_arcsec;
            worst_body = name;
        }
    }
    println!("{}", "-".repeat(82));
    println!(
        "worst body  : {worst_body} at {worst_overall:.4}\" ({:.6}°)",
        worst_overall / 3600.0
    );
    if skipped > 0 {
        println!("skipped     : {skipped} (body,epoch) pairs (out-of-coverage / unsupported body)");
    }
    if !kernel_present {
        println!();
        println!(
            "SELF-BASELINE run: numbers above are ~0 by construction and do NOT measure DE440 agreement."
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angle_sep_handles_wrap() {
        assert!((angle_sep_deg(0.0, 360.0) - 0.0).abs() < 1e-12);
        assert!((angle_sep_deg(1.0, 359.0) - 2.0).abs() < 1e-9);
        assert!((angle_sep_deg(10.0, 350.0) - 20.0).abs() < 1e-9);
        assert!((angle_sep_deg(100.0, 100.0)).abs() < 1e-12);
        // The separation is always the SHORTEST arc, never > 180°.
        assert!(angle_sep_deg(0.0, 270.0) <= 180.0 + 1e-9);
    }

    #[test]
    fn deviation_accumulates_arcseconds() {
        let mut d = Deviation::default();
        // 1° == 3600". Record two samples: 1° and 2°.
        d.record(1.0, 2_451_545.0);
        d.record(2.0, 2_451_560.0);
        assert_eq!(d.count, 2);
        // mean = (3600 + 7200)/2 = 5400"
        assert!((d.mean_arcsec() - 5400.0).abs() < 1e-6);
        // max = 7200" at the second epoch
        assert!((d.max_arcsec - 7200.0).abs() < 1e-6);
        assert_eq!(d.worst_jd, 2_451_560.0);
        // rms = sqrt((3600^2 + 7200^2)/2)
        let expected_rms = ((3600.0_f64.powi(2) + 7200.0_f64.powi(2)) / 2.0).sqrt();
        assert!((d.rms_arcsec() - expected_rms).abs() < 1e-6);
    }

    #[test]
    fn self_baseline_longitudes_are_identical() {
        // In self-baseline mode the same almanac is sampled twice, so the
        // longitude delta for any body/epoch must be exactly zero — this is the
        // wiring sanity the no-kernel run reports.
        let a = Almanac::default_vedic();
        let jd = JdUT1(2_451_545.0);
        for (body, _) in BENCH_BODIES {
            if let (Ok(x), Ok(y)) = (
                a.geocentric_longitude_deg(*body, jd),
                a.geocentric_longitude_deg(*body, jd),
            ) {
                assert!(
                    angle_sep_deg(x, y) < 1e-12,
                    "self-baseline longitude must be identical"
                );
            }
        }
    }

    /// Sanity that the candidate almanac computes real, in-range longitudes for
    /// every benchmarked body at J2000 — i.e. the sweep operates on genuine
    /// positions, not stubs or errors.
    #[test]
    fn baseline_computes_real_positions() {
        let a = Almanac::default_vedic();
        let jd = JdUT1(2_451_545.0);
        for (body, name) in BENCH_BODIES {
            let lon = a
                .geocentric_longitude_deg(*body, jd)
                .unwrap_or_else(|e| panic!("{name}: {e}"));
            assert!(
                (0.0..360.0).contains(&lon),
                "{name} longitude out of range: {lon}"
            );
        }
    }
}
