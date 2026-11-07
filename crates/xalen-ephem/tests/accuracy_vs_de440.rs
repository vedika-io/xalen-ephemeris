// =============================================================================
// Reproducible Accuracy Harness — XALEN analytical engine vs bundled DE440 reader
// =============================================================================
//
// PURPOSE
// -------
// Make the "validated / reproducible" accuracy claim TRUE *from the public
// repository*, with no Swiss Ephemeris dependency. The 5,000,000-chart
// statistical run quoted in docs/ACCURACY.md lives in the monorepo because it
// needs a Swiss-Ephemeris FFI oracle that cannot ship in an open-source crate.
//
// This harness uses the OTHER ground truth that XALEN already bundles: the real
// NASA JPL DE440 SPK binary kernel, read by the in-crate `De440Reader`. It
// diffs the standalone analytical engine (VSOP87A + ELP2000-82 + IAU 2000B
// nutation) against the DE440 reader across N deterministically-sampled charts
// and prints per-body max / p99 / rms error in arcseconds — the same shape as
// the docs/ACCURACY.md table, but reproducible by anyone who downloads the
// public kernel.
//
// Both almanacs run the SAME apparent-place reduction chain (light-time +
// precession + IAU 2000B nutation + annual aberration), so the residual is the
// analytical *series* truncation error against JPL's numerically-integrated
// truth — exactly what the accuracy table is meant to characterise.
//
// REPRODUCING (no Swiss dependency)
// ---------------------------------
//   curl -o /tmp/de440s.bsp \
//     https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440s.bsp
//   cargo test -p xalen-ephem --test accuracy_vs_de440 -- --nocapture
//
// Without /tmp/de440s.bsp present every test SKIPs cleanly (prints SKIP, passes),
// so a plain `cargo test` stays green without the 32 MB kernel. The `de440`
// stage of scripts/gate.sh downloads the kernel and runs these tests with it.
//
// DETERMINISTIC SAMPLING
// ----------------------
// Charts are drawn with SplitMix64 (Steele, Lea & Flood, "Fast Splittable
// Pseudorandom Number Generators", OOPSLA 2014 — the same generator the JDK's
// `SplittableRandom` uses; golden-ratio increment 0x9E3779B97F4A7C15). Fixed
// seeds make the sample bit-for-bit reproducible. Dates span DE440s coverage
// (years ~1850–2150, well inside the kernel's 1550–2650 range).
// =============================================================================

use std::path::Path;
use std::sync::Arc;

use xalen_ephem::{Almanac, Body, De440Provider, De440Reader};
use xalen_time::JdUT1;

/// Path to the DE440s.bsp kernel (tests SKIP if absent; the gate's `de440` stage provides it).
const DE440S_PATH: &str = "/tmp/de440s.bsp";

/// Number of charts sampled per run. Kept modest so the harness stays a fast
/// `cargo test` (not the 5M monorepo run); enough for a meaningful p99/max.
const N_CHARTS: usize = 20_000;

/// Sampling window: Julian Days for 1850-01-01 .. 2150-01-01 (inside DE440s).
const JD_1850: f64 = 2_396_758.5;
const JD_2150: f64 = 2_506_322.5;

// ---------------------------------------------------------------------------
// SplitMix64 — public-domain splittable PRNG (Steele/Lea/Flood, OOPSLA 2014).
// ---------------------------------------------------------------------------
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        SplitMix64 { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        // Golden-ratio increment, as published.
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform f64 in [0, 1).
    fn next_unit(&mut self) -> f64 {
        // 53-bit mantissa, standard construction.
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform f64 in [lo, hi).
    fn next_range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.next_unit()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------
fn de440s_exists() -> bool {
    Path::new(DE440S_PATH).exists()
}

/// Shortest angular separation between two longitudes, in arcseconds.
fn angle_delta_arcsec(a: f64, b: f64) -> f64 {
    let mut d = (a - b).abs() % 360.0;
    if d > 180.0 {
        d = 360.0 - d;
    }
    d * 3600.0
}

/// Bodies measured against DE440. The lunar nodes are excluded: they are derived
/// (mean polynomial / osculating) and reflect node *algorithm* differences, not
/// ephemeris error — exactly the carve-out docs/ACCURACY.md makes.
const BODIES: &[(Body, &str)] = &[
    (Body::Sun, "Sun"),
    (Body::Moon, "Moon"),
    (Body::Mercury, "Mercury"),
    (Body::Venus, "Venus"),
    (Body::Mars, "Mars"),
    (Body::Jupiter, "Jupiter"),
    (Body::Saturn, "Saturn"),
    (Body::Uranus, "Uranus"),
    (Body::Neptune, "Neptune"),
    // Pluto omitted: the analytical Meeus Ch.37 fit is only valid 1885–2099, so a
    // 1850–2150 sample would mix in-window and out-of-window dates and produce a
    // misleading aggregate. A dedicated in-window Pluto check lives below.
];

struct Stats {
    max: f64,
    p99: f64,
    rms: f64,
    n: usize,
}

fn summarize(mut errs: Vec<f64>) -> Stats {
    let n = errs.len();
    errs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let max = *errs.last().unwrap_or(&0.0);
    let p99_idx = ((n as f64) * 0.99).floor() as usize;
    let p99 = errs
        .get(p99_idx.min(n.saturating_sub(1)))
        .copied()
        .unwrap_or(0.0);
    let sum_sq: f64 = errs.iter().map(|e| e * e).sum();
    let rms = if n > 0 {
        (sum_sq / n as f64).sqrt()
    } else {
        0.0
    };
    Stats { max, p99, rms, n }
}

fn build_de440_almanac() -> Almanac {
    let reader = De440Reader::from_file(Path::new(DE440S_PATH)).expect("failed to load de440s.bsp");
    let provider = De440Provider::with_reader(reader);
    Almanac::default_vedic().with_provider(Arc::new(provider))
}

// ===========================================================================
// MAIN HARNESS: per-body analytical-vs-DE440 statistics over N sampled charts.
// ===========================================================================
#[test]
fn analytical_vs_de440_per_body_statistics() {
    if !de440s_exists() {
        println!(
            "SKIP: {} not found — download the public NASA DE440s kernel to run \
             the reproducible accuracy harness (see file header).",
            DE440S_PATH
        );
        return;
    }

    let analytical = Almanac::default_vedic(); // VSOP87A + ELP2000-82, zero data files
    let de440 = build_de440_almanac(); // real NASA JPL DE440 SPK kernel

    // One error vector per body, parallel to BODIES.
    let mut errs: Vec<Vec<f64>> = vec![Vec::with_capacity(N_CHARTS); BODIES.len()];

    // Single deterministic seed for the JD stream (the monorepo run uses ten
    // 500k shards seeded 1..10; here one shard of N is enough for a fast harness).
    let mut rng = SplitMix64::new(0x5A1E_2026_0601);

    for _ in 0..N_CHARTS {
        let jd = JdUT1(rng.next_range(JD_1850, JD_2150));
        for (i, (body, _)) in BODIES.iter().enumerate() {
            // Identical JD into both engines — the only difference is the model.
            let a = analytical.geocentric_longitude_deg(*body, jd);
            let d = de440.geocentric_longitude_deg(*body, jd);
            if let (Ok(av), Ok(dv)) = (a, d) {
                errs[i].push(angle_delta_arcsec(av, dv));
            }
        }
    }

    println!(
        "\n=== XALEN analytical (VSOP87A/ELP2000-82) vs bundled JPL DE440 ===\n\
         {} charts, dates 1850–2150, deterministic SplitMix64 seed 0x5A1E20260601\n\
         (ecliptic-of-date apparent longitude; errors in arcseconds)\n",
        N_CHARTS
    );
    println!(
        "  {:>9}  {:>8}  {:>8}  {:>8}  {:>7}",
        "body", "max", "p99", "rms", "n"
    );
    println!(
        "  {:->9}  {:->8}  {:->8}  {:->8}  {:->7}",
        "", "", "", "", ""
    );

    // Loose ceilings: a regression alarm, NOT the published figure. These are
    // generous bounds derived from the body's documented series-truncation error
    // (Moon ELP is the soft spot at tens of arcsec; planets are sub-few-arcsec).
    // The PRINTED numbers are the measured truth; this only fails on a blow-up.
    let ceiling_arcsec = |body: Body| -> f64 {
        match body {
            Body::Moon => 120.0, // truncated ELP/Meeus series — documented soft spot
            Body::Neptune => 30.0,
            Body::Uranus => 30.0,
            _ => 20.0,
        }
    };

    let mut alarms = Vec::new();
    for (i, (body, label)) in BODIES.iter().enumerate() {
        let s = summarize(std::mem::take(&mut errs[i]));
        println!(
            "  {:>9}  {:>8.2}  {:>8.2}  {:>8.2}  {:>7}",
            label, s.max, s.p99, s.rms, s.n
        );
        let ceil = ceiling_arcsec(*body);
        if s.max > ceil {
            alarms.push(format!(
                "  {}: max {:.2}\" exceeds regression ceiling {:.0}\" — investigate \
                 (the published figure is whatever this harness MEASURES, not this ceiling)",
                label, s.max, ceil
            ));
        }
    }

    assert!(
        alarms.is_empty(),
        "\nAnalytical-vs-DE440 regression alarm:\n{}\n",
        alarms.join("\n")
    );
}

// ===========================================================================
// Pluto — measured ONLY inside the analytical-fit validity window (1885–2099),
// matching the honest scope in docs/ACCURACY.md.
// ===========================================================================
#[test]
fn analytical_vs_de440_pluto_in_window() {
    if !de440s_exists() {
        println!("SKIP: {} not found", DE440S_PATH);
        return;
    }

    let analytical = Almanac::default_vedic();
    let de440 = build_de440_almanac();

    // Meeus Ch.37 Pluto fit validity window.
    const JD_1885: f64 = 2_409_768.5;
    const JD_2099: f64 = 2_488_069.5;

    let mut errs = Vec::with_capacity(N_CHARTS);
    let mut rng = SplitMix64::new(0x9111_2099_1885);
    for _ in 0..N_CHARTS {
        let jd = JdUT1(rng.next_range(JD_1885, JD_2099));
        if let (Ok(av), Ok(dv)) = (
            analytical.geocentric_longitude_deg(Body::Pluto, jd),
            de440.geocentric_longitude_deg(Body::Pluto, jd),
        ) {
            errs.push(angle_delta_arcsec(av, dv));
        }
    }

    let s = summarize(errs);
    println!(
        "\n=== Pluto (Meeus Ch.37 analytical fit) vs DE440, 1885–2099 window ===\n\
         {} charts: max {:.2}\"  p99 {:.2}\"  rms {:.2}\"  (n={})\n",
        N_CHARTS, s.max, s.p99, s.rms, s.n
    );

    // Every sampled JD lies inside the Meeus Pluto validity window (1885–2099)
    // AND inside DE440s, so EVERY call must succeed. Asserting the sample count
    // prevents the harness from silently under-sampling: without this, a window
    // where every call errored would leave `errs` empty, and `summarize` reports
    // max = 0.0, making the regression ceiling below pass on ZERO data.
    assert_eq!(
        s.n, N_CHARTS,
        "Pluto DE440 harness under-sampled: {} of {} calls succeeded — a skipped \
         sample must fail the test, not be silently dropped",
        s.n, N_CHARTS
    );

    // The Goffin/Steyaert fit is an arcminute-class model; allow generous headroom.
    assert!(
        s.max < 600.0,
        "Pluto analytical-vs-DE440 max {:.2}\" exceeds 600\" (10') regression ceiling \
         in its own 1885–2099 validity window",
        s.max
    );
}

// ===========================================================================
// Sanity: at J2000 the analytical and DE440 inner planets must agree closely —
// a fast smoke test that the two engines are wired to the SAME reduction chain.
// ===========================================================================
#[test]
fn analytical_vs_de440_inner_planets_agree_at_j2000() {
    if !de440s_exists() {
        println!("SKIP: {} not found", DE440S_PATH);
        return;
    }

    const JD_J2000: f64 = 2_451_545.0;
    let analytical = Almanac::default_vedic();
    let de440 = build_de440_almanac();

    for body in [Body::Sun, Body::Mercury, Body::Venus, Body::Mars] {
        let a = analytical
            .geocentric_longitude_deg(body, JdUT1(JD_J2000))
            .expect("analytical longitude");
        let d = de440
            .geocentric_longitude_deg(body, JdUT1(JD_J2000))
            .expect("de440 longitude");
        let delta = angle_delta_arcsec(a, d);
        println!("  {:?} @ J2000: analytical-vs-DE440 = {:.2}\"", body, delta);
        // Inner planets at J2000 are VSOP87's best epoch; a few arcsec ceiling.
        assert!(
            delta < 5.0,
            "{:?} analytical vs DE440 at J2000 = {:.2}\" (> 5\" smoke ceiling)",
            body,
            delta
        );
    }
}
