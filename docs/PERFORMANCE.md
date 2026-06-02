# XALEN Ephemeris -- Performance Benchmarks

**Last updated:** 2026-05-25
**Engine version:** 0.5.0
**Platform:** Apple Silicon (ARM64), macOS, Rust 2024 edition
**Benchmark tool:** Criterion 0.5 with HTML reports

---

## Benchmark Results

All benchmarks measured using Criterion's statistical sampling (100 samples per
benchmark, 3-second warmup, confidence intervals reported).

| Benchmark | Median | Range (95% CI) | Iterations |
|-----------|--------|-----------------|------------|
| **Sun longitude** (VSOP87A geocentric) | 20.8 us | 19.9 -- 22.1 us | 242K |
| **Moon longitude** (ELP2000-82 geocentric) | 12.1 us | 11.3 -- 13.2 us | 308K |
| **Full Vedic chart** (9 planets + houses + ayanamsa + nakshatra) | 379.6 us | 364.8 -- 395.4 us | 15K |
| **Placidus house cusps** (12 cusps, Pune lat 18.52) | 2.8 us | 2.5 -- 3.2 us | 2.8M |
| **Ayanamsa** (Lahiri, includes delta-T conversion) | 6.9 ns | 6.2 -- 7.9 ns | 810M |
| **Nakshatra** (from longitude: nakshatra + pada + lord + deity) | 271 ps | 267 -- 276 ps | 18B |
| **Panchang** (5 limbs: tithi + nakshatra + yoga + karana + vara) | 35.2 us | 33.4 -- 37.3 us | 86K |
| **Vimshottari Dasha** (120-year cycle to Antardasha level) | 1.2 us | 1.2 -- 1.3 us | 3.6M |

### What Each Benchmark Measures

- **Sun longitude:** Single `geocentric_longitude_deg(Body::Sun, jd)` call. Includes
  VSOP87A heliocentric Earth computation, geocentric inversion, precession to
  equinox-of-date, and aberration correction. This is the fundamental atomic
  operation for the most precisely computed body.

- **Moon longitude:** Single `geocentric_longitude_deg(Body::Moon, jd)` call. Evaluates
  all 60 longitude terms from ELP2000-82 (Meeus Table 47.A) plus fundamental arguments.

- **Full Vedic chart:** The complete computation a production astrology application
  performs for one chart: all 9 Vedic grahas (Sun, Moon, Mercury, Venus, Mars,
  Jupiter, Saturn, Rahu, Ketu) in sidereal coordinates with nakshatra and pada
  for each, plus Whole Sign house cusps.

- **Placidus house cusps:** A single `compute_houses()` call for the most
  computationally expensive house system (requires iterative trisection of
  semi-arcs).

- **Ayanamsa:** Lahiri ayanamsa computation including the UT1-to-TT delta-T
  conversion. The ayanamsa polynomial itself is sub-nanosecond; the 6.9 ns
  includes the Stephenson-Morrison-Hohenkerk 2016 delta-T evaluation.

- **Nakshatra:** Pure arithmetic lookup from a sidereal longitude: 27-fold division,
  pada (quarter), ruling lord, and deity. Sub-nanosecond because it is pure integer
  and floating-point arithmetic with no table lookups or allocations.

- **Panchang:** All five limbs of the Hindu calendar: Tithi (lunar day), Nakshatra
  (lunar mansion), Yoga (Sun+Moon sum), Karana (half-tithi), and Vara (weekday).
  Requires computing both Sun and Moon sidereal positions, so the time is dominated
  by the two planetary position calls.

- **Vimshottari Dasha:** Computes the full 120-year Vimshottari dasha tree down to
  the Antardasha (sub-period) level from a given Moon longitude and birth date.
  9 Mahadasha periods x 9 Antardasha each = 81 sub-periods.

---

## Context: Swiss Ephemeris Comparison

> **Caveat — preliminary, unbenchmarked comparison.** The XALEN column below is
> the measured Criterion median on the platform in the header (Apple Silicon
> ARM64, macOS, release build). The "Swiss Eph" column is a rough **order-of-
> magnitude estimate** from the Swiss Ephemeris documentation / community reports
> — it was **not** benchmarked here, on this machine, or against any pinned Swiss
> Ephemeris version. Hardware, build flags, Moshier-vs-`.se1`, and warm/cold caches
> all move these numbers substantially. Treat any "faster"/"slower" conclusion as
> indicative only; a head-to-head on identical hardware is future work.

Swiss Ephemeris (C implementation) is reported to compute a single planet position
in roughly the same order of magnitude (low-tens of microseconds with its Moshier
analytical engine, faster with pre-loaded binary ephemeris files). XALEN targets
the same performance class in pure Rust:

| Operation | XALEN (measured) | Swiss Eph (estimate, unverified) | Notes |
|-----------|------------------|----------------------------------|-------|
| Sun position | 20.8 us | ~10 us | XALEN uses full VSOP87A with precession + aberration |
| Moon position | 12.1 us | ~10 us | XALEN evaluates all 60 ELP2000-82 terms |
| House cusps (Placidus) | 2.8 us | ~5 us | XALEN's Placidus uses an optimized trisection (estimate not benchmarked) |
| Ayanamsa | 6.9 ns | ~1 us | XALEN evaluates a pure polynomial (estimate not benchmarked) |
| Full chart (9 bodies + houses) | 380 us | ~100-150 us | XALEN: no caching between body calls yet |

**Key differences (XALEN-internal facts; cross-engine deltas are estimates):**

1. XALEN uses the full VSOP87A series for each planet call (no caching of intermediate
   Earth position across bodies), which accounts for the per-body cost on individual
   planet calls. A production optimization to cache the Earth heliocentric position per
   JD would bring Sun/planet times down.

2. House cusp and ayanamsa computation are very cheap in absolute terms (microsecond /
   nanosecond class). Whether they are faster than a given Swiss Ephemeris build is
   not something this document has measured — the SE figures are estimates.

3. The full-chart benchmark (380 us) is the measured real-world cost of computing
   one complete natal chart on the header platform. At this speed, a single thread
   computes roughly **2,600 full natal charts per second** on that machine.

---

## Throughput Estimates

Based on the benchmark medians:

| Workload | Throughput (single thread) |
|----------|---------------------------|
| Individual planet positions (Sun) | ~48,000 / second |
| Individual planet positions (Moon) | ~82,000 / second |
| Full Vedic natal charts (9 planets + houses) | ~2,600 / second |
| Panchang computations | ~28,000 / second |
| Vimshottari Dasha trees | ~830,000 / second |
| Nakshatra lookups | ~3.7 billion / second |
| Ayanamsa computations | ~145 million / second |

All computation is `Send + Sync` (thread-safe by construction), so throughput
scales linearly with CPU cores. On an 8-core machine, the full-chart throughput
exceeds **20,000 charts per second** with zero locking overhead.

---

## Running Benchmarks

```bash
# Full benchmark suite with HTML reports
cargo bench --bench core_bench -p xalen-ephemeris

# Quick benchmark (text output only)
cargo bench --bench core_bench -p xalen-ephemeris 2>&1 | grep "time:"

# Individual benchmark
cargo bench --bench core_bench -p xalen-ephemeris -- planet_longitude_sun
```

Criterion generates detailed HTML reports in `target/criterion/`. Open
`target/criterion/report/index.html` for interactive charts showing distributions,
regression analysis, and comparison across runs.

---

## Memory Profile

| Component | Allocation | Notes |
|-----------|-----------|-------|
| `Almanac` (analytical) | ~1 KB | Provider references only; no data files loaded |
| `Almanac` (with DE440) | Depends on loaded segments | Segments loaded lazily from `.bsp` file |
| Planet position call | Zero heap allocation | All computation on stack |
| House cusps call | Zero heap allocation | 12-element array on stack |
| Nakshatra/Rashi lookup | Zero allocation | Pure arithmetic |
| Dasha tree (Antardasha) | ~81 entries (~6 KB) | Allocated once, returned as Vec |
| Full chart (9 planets) | < 1 KB total | Positions stored as f64 tuples |

The analytical engine requires zero data files and performs zero heap allocations
for individual position queries. This makes it suitable for embedded, WASM, and
high-throughput server workloads.

---

## Build Optimization

The benchmarks above were measured with Rust release-mode optimizations:

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

Debug builds are approximately 10-50x slower due to unoptimized trigonometric
functions and missing inlining. Always benchmark with `--release` (which Criterion
uses by default).
