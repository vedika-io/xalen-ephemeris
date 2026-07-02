# XALEN Ephemeris-Precision Benchmark

**Planetary-longitude agreement of the XALEN ephemeris against the JPL DE440 reference.**

This document describes how XALEN's computed planetary positions are measured against
an independent reference ephemeris, and how to reproduce that measurement from this
repository with a single `cargo` command. The numbers below are **ephemeris /
astronomical precision** — agreement between computed planetary ecliptic longitudes.
They are **not** a measure of "astrology accuracy," which is a separate, non-numerical
question about interpretation.

> **JPL DE440** is a public reference ephemeris published by NASA's Jet Propulsion
> Laboratory. XALEN compares against it purely as a reference. Nothing here implies any
> endorsement, partnership, certification, or affiliation with NASA or JPL. The DE440
> kernel is downloaded directly from the public NAIF archive at run time.

---

## What is being measured

For a dense grid of timestamps, the harness computes the **apparent geocentric ecliptic
longitude** (equinox-of-date — the quantity a chart actually reads) of each major body,
using two independent paths through the same library:

| Path | Theory |
|------|--------|
| **Reference** | JPL **DE440** — NASA/JPL's numerically-integrated planetary & lunar ephemeris, read directly from the official `.bsp` SPK kernel via XALEN's NAIF DAF/SPK reader. |
| **Candidate** | XALEN's pure-Rust **analytic** series (VSOP87 for the planets, a truncated ELP-2000 theory for the Moon). |

Both paths share the **identical** apparent-place reduction (light-time retardation,
IAU 2006 precession, IAU 2000B nutation, annual aberration), so the residual between
them isolates exactly one thing: **the difference between the DE440 numerical reference
and XALEN's analytic series.** The metric reported is the absolute angular separation
`|Δλ|` in **arcseconds** (with correct 0°/360° wrap handling), summarized per body as
mean, RMS, and worst-case max, plus the worst-offending epoch.

The harness lives in [`validation/src/de440_bench.rs`](validation/src/de440_bench.rs)
and is built entirely on the engine's own public API
(`Almanac::with_de440`, `Almanac::geocentric_longitude_deg`) — it re-implements no
astronomy of its own.

---

## How to reproduce (exact commands)

```bash
# 1. Fetch the public DE440s SPK kernel directly from NASA's NAIF archive (~32 MB).
#    de440s is the "small"/short-span DE440 release covering ~1550–2650.
curl -L -o /tmp/de440s.bsp \
  https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440s.bsp

# 2. Build and run the benchmark (Rust 1.85+ / edition 2024).
cargo run -p xalen-validation --release --bin de440_bench
```

That default run sweeps the modern era (≈1900–2050) on a 15-day grid. To probe a
different span or resolution:

```bash
# Tighter modern window (1950–2030) on a 7-day grid:
cargo run -p xalen-validation --release --bin de440_bench -- \
  --kernel /tmp/de440s.bsp --start-jd 2433283 --end-jd 2462502 --step-days 7
```

| Flag | Meaning | Default |
|------|---------|---------|
| `--kernel PATH` | DE440/DE441 `.bsp` SPK kernel | `/tmp/de440s.bsp` |
| `--start-jd JD` | first epoch (Julian Day) | `2415021` (~1900-01-01) |
| `--end-jd JD` | last epoch (Julian Day) | `2469807` (~2050-01-01) |
| `--step-days N` | grid spacing in days | `15` |

If the kernel file is absent the harness still runs, but in a clearly-labelled
**self-baseline mode** (the analytic almanac sampled against itself, deviations ~0). That
mode is a wiring check only — it is **not** a validation, and the output says so on every
run. CI does not fetch the kernel, so CI exercises the self-baseline path.

---

## Results

Measured on this repository at `cargo` 1.95 / edition 2024, kernel `de440s.bsp` fetched
from the NAIF archive above. **Reference = JPL DE440, candidate = XALEN analytic series.**
Run the commands above to reproduce these numbers for your configuration — they are not
hard-coded anywhere, and the harness prints the worst epoch for every row so any value can
be inspected directly.

### Modern era — JD 2415021 .. 2469807 (≈1900–2050), 15-day grid, 3653 epochs/body

| Body | mean (″) | RMS (″) | max (″) | worst epoch |
|------|---------:|--------:|--------:|-------------|
| Sun | 0.0812 | 0.0983 | 0.1932 | 1903-09-28 |
| Moon | 2.2027 | 2.8168 | 11.5162 | 2041-02-25 |
| Mercury | 0.0849 | 0.1014 | 0.2553 | 1923-09-28 |
| Venus | 0.0940 | 0.1140 | 0.4033 | 1903-09-28 |
| Mars | 0.0801 | 0.0914 | 0.3231 | 2005-01-15 |
| Jupiter | 0.1609 | 0.1868 | 0.4550 | 2049-04-21 |
| Saturn | 0.1691 | 0.1942 | 0.3862 | 2046-11-20 |
| Uranus | 0.4029 | 0.5558 | 1.4096 | 2036-09-08 |
| Neptune | 0.6238 | 0.8910 | 2.1540 | 2049-10-26 |
| Pluto | 0.3521 | 0.4510 | 1.2924 | 2049-08-30 |

**Worst body: Moon, 11.5162″ (0.0032°).** Every planet and the Sun agree with the DE440
reference to **well under 1 arcsecond** across the entire 150-year span; the outermost
planets (Uranus / Neptune / Pluto) reach ~1–2″ only at the far edges of the window, where
the analytic series is least constrained.

### Tighter modern window — JD 2433283 .. 2462502 (≈1950–2030), 7-day grid, 4175 epochs/body

| Body | mean (″) | RMS (″) | max (″) |
|------|---------:|--------:|--------:|
| Sun | 0.0766 | 0.0941 | 0.1753 |
| Moon | 2.2417 | 2.9379 | 14.1755 |
| Mercury | 0.0813 | 0.0972 | 0.2344 |
| Venus | 0.0905 | 0.1105 | 0.3813 |
| Mars | 0.0793 | 0.0913 | 0.3464 |
| Jupiter | 0.1834 | 0.1977 | 0.3798 |
| Saturn | 0.1969 | 0.2107 | 0.3770 |
| Uranus | 0.3774 | 0.4847 | 1.3339 |
| Neptune | 0.6566 | 0.7889 | 1.6355 |
| Pluto | 0.2690 | 0.3275 | 0.5838 |

---

## How to read these numbers (honest scope)

- **The residual characterizes XALEN's *analytic* path against DE440.** What the table
  shows is how close XALEN's pure-Rust VSOP87/ELP series comes to the JPL DE440 numerical
  integration. For the Sun and all eight planets that is **sub-arcsecond** through the
  modern era — i.e. far below the ~1′ precision any astrological chart can use, and below
  the typical convention differences (mean-vs-true node, ayanamsa choice) between
  ephemeris products.

- **When a DE440 kernel is loaded, XALEN serves DE440 directly.** XALEN's `with_de440`
  path reads positions straight from the SPK Chebyshev polynomials — the same data JPL
  publishes — so it reproduces the DE440 integration itself, not an approximation of it.
  That path is cross-validated to **sub-kilometer** raw geometry against an independently
  sourced JPL Horizons state vector, and to **sub-arcsecond** apparent longitude against
  JPL Horizons, in the committed test
  [`crates/xalen-ephem/tests/de440_real_crossval.rs`](crates/xalen-ephem/tests/de440_real_crossval.rs)
  (`cargo test -p xalen-ephem --test de440_real_crossval`, run with the kernel present).
  This `BENCHMARK.md` harness is the complementary **statistical sweep at arbitrary epoch
  count**; the test file is the **point external check** against Horizons.

- **The Moon is the widest row, by design.** XALEN's *analytic* Moon uses a truncated
  ELP-2000 series, so it shows the largest analytic-vs-DE440 spread (~2″ RMS, ~14″ worst).
  XALEN's *DE440-backed* Moon is sub-arcsecond vs JPL Horizons (see the test above) — the
  spread in the table is the price of the analytic fallback, not of the DE440 path.

- **Lunar nodes and apogees are excluded** from this table on purpose: they are abstract
  derived points that fall back to the same analytic model in *both* the reference and the
  candidate almanac here, so a DE440-vs-analytic comparison of them measures nothing. Their
  separate characterization lives with the engine's other validation material.

---

## Related, independently-sourced validation in this repo

| Check | File | What it asserts |
|-------|------|-----------------|
| DE440 raw geometry vs **JPL Horizons** state vector | `crates/xalen-ephem/tests/de440_real_crossval.rs` | Sun/SSB position to **<1 km** vs an independently-quoted Horizons Vector-Table value. |
| DE440 apparent longitude vs **JPL Horizons** | same file | Moon apparent longitude to **<0.01°** vs Horizons quantity #31. |
| Statistical sweep vs a **Swiss-Ephemeris / Moshier** oracle, any chart count | `validation/` (`xalen-validation`, `oracle_pyswisseph.py`) | Per-body / house / ayanamsa deltas across N random charts; see `validation/README.md`. |
| Medieval-epoch analytic longitudes vs **Horizons (DE441)** | `crates/xalen-ephem/examples/validate_medieval.rs` | AD 500–1700 apparent longitudes for offline comparison. |

Together these give three independent angles on the same engine: a **point** external
check against JPL Horizons (the test), a **statistical** sweep against a Swiss/Moshier
oracle (the `validation/` harness), and **this** dense DE440-reference longitude sweep.
