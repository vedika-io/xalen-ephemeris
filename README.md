# XALEN Ephemeris

**The most comprehensive astronomical ephemeris for astrology. Pure Rust.**

[![Crates.io](https://img.shields.io/crates/v/xalen-ephemeris.svg)](https://crates.io/crates/xalen-ephemeris)
[![docs.rs](https://img.shields.io/docsrs/xalen-ephemeris)](https://docs.rs/xalen-ephemeris)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

```rust
use xalen_ephem::{Almanac, Body};
use xalen_time::{calendar_to_jd, CalendarSystem, JulianDay};
use xalen_ayanamsa::Ayanamsa;
use xalen_vedic::nakshatra::Nakshatra;

let jd = calendar_to_jd(1990, 3, 15, 12.0 - 5.5, CalendarSystem::default());
let almanac = Almanac::default_vedic();
let pos = almanac.geocentric_ecliptic(Body::Moon, jd).unwrap();
let sid = (pos.longitude.to_degrees() - Ayanamsa::Lahiri.compute_deg(jd.as_f64())).rem_euclid(360.0);
println!("{}", Nakshatra::from_longitude_deg(sid)); // Swati
```

---

## What You Get

- **50 ayanamsa systems** -- Lahiri, KP Krishnamurti, Raman, Fagan-Bradley, True Chitrapaksha, True Revati, Surya Siddhanta, all Galactic Center variants, Babylonian (Kugler), all Swiss Ephemeris IDs (0-46), plus a fully custom variant with user-defined epoch, value, and precession rate

- **23 house systems** -- Placidus, Koch, Campanus, Regiomontanus, Whole Sign, Equal, Porphyry, Morinus, Alcabitius, Topocentric (Polich-Page), Meridian, Vehlow, Sripati, Krusinski-Pisa, Gauquelin sectors, Sunshine (Makransky & Treindl), Pullen Sinusoidal (Delta & Ratio), Carter Poli-Equatorial, APC, Zariel, Alcabitius Classic -- with automatic Porphyry fallback at polar latitudes

- **12+ astrology traditions** -- Vedic/Jyotish (dasha, shadbala, KP, Jaimini, Tajaka, panchang, compatibility, 16 divisional charts, yoga, dosha), Western (aspects, dignities, 97 Arabic Lots, Hellenistic, Uranian, Cosmobiology, progressions, returns, harmonics, horary), Chinese (BaZi, Zi Wei Dou Shu, Feng Shui Flying Stars, Qi Men Dun Jia), Lal Kitab, I Ching, Numerology, Korean Saju, Japanese Nine Star Ki, Burmese Mahabote, Mayan, Aztec, Tibetan, Persian/Zoroastrian, Egyptian, Celtic

- **506 built-in fixed stars** (in `xalen-western`) with proper motion and precession correction (all mag < 3.0, Behenian, Royal, Nakshatra yogatara, IAU-named to mag ~5). A separate lightweight `xalen-stars` crate carries a 108-star core catalog plus a runtime loader for the full Hipparcos catalog (118,218 stars)

- **Solar and lunar eclipse detection** -- Meeus Ch. 54/55, with latitude thresholds and classification (total, annular, partial, penumbral)

- **Black Moon Lilith** (mean lunar apogee), Chiron, True Node, Mean Node

- **15 asteroids** -- the big 4 (Ceres, Pallas, Juno, Vesta), Hygeia, Astraea, Psyche, Eros, Lilith (1181), centaurs (Pholus, Nessus), and TNOs (Eris, Sedna, Makemake, Haumea), plus an external element loader for any asteroid with known orbital elements

- **SVG chart rendering** -- North Indian diamond, South Indian box, and Western wheel charts, zero external dependencies

- **City geocoding** for 130+ cities with latitude, longitude, and timezone

- **NAIF DE440 SPK reader** -- reads real JPL `.bsp` binary files for extended validity range and high raw positional accuracy; the analytical engine (VSOP87A + ELP2000-82 + IAU 2000B nutation) works standalone with zero data files at **sub-arcsecond accuracy** (JPL Horizons DE440 cross-validated: Sun 0.5", Mercury-Neptune < 1.1", Moon < 18"). Both engines apply the same apparent-place chain (precession + nutation + annual aberration)

- **19 languages** -- English, Hindi, Sanskrit, Tamil, Telugu, Kannada, Malayalam, Bengali, Gujarati, Marathi, Punjabi, Odia, Spanish, Portuguese, French, German, Japanese, Thai, Indonesian -- for planet names, signs, nakshatras, weekdays

- **Language bindings** -- Node.js (napi-rs), Python (PyO3), WASM (wasm-bindgen), and C FFI (`extern "C"`)

---

## Installation

```bash
cargo add xalen-ephemeris
```

Or pick individual crates for a smaller footprint:

```toml
[dependencies]
xalen-ephem = "0.1"       # planetary engine
xalen-vedic = "0.1"       # Vedic astrology
xalen-houses = "0.1"      # house systems
xalen-ayanamsa = "0.1"    # ayanamsa / sidereal conversion
xalen-time = "0.1"        # Julian Day, delta-T, calendars
```

## Quick Start

### Vedic chart: Sun and Moon with nakshatra and rashi

```rust
use xalen_ayanamsa::Ayanamsa;
use xalen_coords::RAD_TO_DEG;
use xalen_ephem::{Almanac, Body};
use xalen_time::{calendar_to_jd, CalendarSystem, JulianDay};
use xalen_vedic::nakshatra::Nakshatra;
use xalen_vedic::rashi::Rashi;

fn main() {
    // 15 August 1947, 00:00 IST (UTC+5:30) -- Indian Independence
    let jd = calendar_to_jd(1947, 8, 15, 0.0 - 5.5, CalendarSystem::default());

    let almanac = Almanac::default_vedic();
    let aya_deg = Ayanamsa::Lahiri.compute_deg(jd.as_f64());

    for &body in &[Body::Sun, Body::Moon] {
        let pos = almanac.geocentric_ecliptic(body, jd).unwrap();
        let sid = (pos.longitude * RAD_TO_DEG - aya_deg).rem_euclid(360.0);

        let rashi = Rashi::from_longitude_deg(sid);
        let nak = Nakshatra::from_longitude_deg(sid);
        let pada = Nakshatra::pada(sid);

        println!("{body}: {sid:.2} deg -- {rashi}, {nak} (pada {pada})");
    }
}
```

### Western chart: aspects and dignities

```rust
use xalen_western::aspects::{find_aspects, AspectType};
use xalen_western::dignities::essential_dignity;
```

### More examples

```bash
cargo run --example basic_chart      # Sun/Moon + nakshatra + rashi
cargo run --example vedic_chart      # Full Vedic chart with dasha, shadbala, panchang
cargo run --example western_chart    # Aspects, dignities, Arabic Lots
cargo run --example chinese_bazi     # BaZi Four Pillars with Wu Xing
```

---

## Accuracy

| Theory | Bodies | Accuracy | Data Files Needed? |
|--------|--------|----------|--------------------|
| VSOP87A + IAU 2000B nutation | Mercury -- Neptune | **0.1--1.1"** (JPL DE440 cross-validated) | No (analytical) |
| ELP2000-82 | Moon | ~2" | No (analytical) |
| Meeus Ch.37 | Pluto | ~15" | No (valid 1885--2099) |
| JPL DE440 | All planets | ~1" apparent (raw geometry sub-mas; body light-time not iterated) | Yes (binary `.bsp`) |
| IAU 2006 | Precession | ~0.3 mas/century | No |
| IAU 2000B | Nutation | ~1 mas | No |

For most astrological applications, the analytical theories (VSOP87A, ELP2000-82)
provide more than enough precision -- they require zero external data files and
work entirely from compiled-in polynomial series. Load DE440 only when you need
sub-arcsecond precision (e.g., occultation timing, primary directions to the
minute).

---

## Crate Map

| Crate | Purpose |
|-------|---------|
| [`xalen-ephem`](crates/xalen-ephem) | Planetary engine: VSOP87A, ELP2000-82, DE440 reader, Pluto, Chiron, lunar nodes, asteroids, eclipses |
| [`xalen-time`](crates/xalen-time) | Julian Day types (TT/UT1/TDB), delta-T models, calendar conversions |
| [`xalen-coords`](crates/xalen-coords) | Coordinate transforms, IAU 2006 precession, IAU 2000B nutation, obliquity |
| [`xalen-houses`](crates/xalen-houses) | 23 house systems with Ascendant, MC, Vertex, polar fallback, city geocoding |
| [`xalen-ayanamsa`](crates/xalen-ayanamsa) | 50 ayanamsa systems for tropical-to-sidereal conversion |
| [`xalen-stars`](crates/xalen-stars) | 108-star core catalog + Hipparcos catalog loader, proper motion, conjunction search (the 506-star astrology catalog lives in `xalen-western`) |
| [`xalen-vedic`](crates/xalen-vedic) | Vedic astrology: dasha, shadbala, KP, Jaimini, Tajaka, ashtakavarga, panchang, compatibility, yoga, dosha, upagraha, transit |
| [`xalen-western`](crates/xalen-western) | Western astrology: aspects, dignities, Arabic Lots, Hellenistic, Uranian, Cosmobiology, returns, progressions, harmonics, horary |
| [`xalen-chinese`](crates/xalen-chinese) | BaZi, Zi Wei Dou Shu, Feng Shui (Flying Stars, Ba Zhai), Qi Men Dun Jia |
| [`xalen-lalkitab`](crates/xalen-lalkitab) | Lal Kitab: planet-house effects, debts, dormancy, remedies |
| [`xalen-iching`](crates/xalen-iching) | I Ching: 64 hexagrams, 8 trigrams, date casting |
| [`xalen-numerology`](crates/xalen-numerology) | Pythagorean and Chaldean numerology |
| [`xalen-world`](crates/xalen-world) | Mayan, Aztec, Tibetan, Persian, Egyptian, Celtic, Korean Saju, Nine Star Ki, Burmese Mahabote |
| [`xalen-chart`](crates/xalen-chart) | SVG chart rendering: North Indian, South Indian, Western wheel |
| [`xalen-ffi`](crates/xalen-ffi) | C FFI exports (`extern "C"` with `repr(C)` structs) |
| [`xalen-wasm`](crates/xalen-wasm) | WASM bindings via wasm-bindgen |
| [`xalen-python`](crates/xalen-python) | Python bindings via PyO3 |
| [`xalen-node`](crates/xalen-node) | Node.js bindings via napi-rs |

## Language Bindings

| Language | Crate | Mechanism | Status |
|----------|-------|-----------|--------|
| **Rust** | `xalen-ephemeris` (umbrella) | Native | Stable |
| **C / C++** | `xalen-ffi` | `extern "C"` + `repr(C)` structs | Stable |
| **Python** | `xalen-python` | PyO3 (`pip install xalen`, planned) | Alpha |
| **Node.js** | `xalen-node` | napi-rs native addon | Alpha |
| **Browser / WASM** | `xalen-wasm` | wasm-bindgen, build with `wasm-pack` | Alpha |

Core computation crates compile to `wasm32-unknown-unknown` without modification.

---

## Architecture

- **Pure Rust** -- no C FFI dependencies in any core crate
- **No `unsafe` in core crates** -- only `xalen-ffi` uses `unsafe` (required for `extern "C"`)
- **Zero global state** -- all computation through owned or `Arc<T>` references
- **Thread-safe** -- `Almanac` and all position types are `Send + Sync`
- **WASM-compatible** -- core crates target `wasm32-unknown-unknown`
- **Provider-layered** -- stack DE440 on top of VSOP87 with automatic body/epoch fallback
- **Serde-ready** -- all data types derive `Serialize` / `Deserialize`

---

## Building

```bash
# Build the full workspace
cargo build --release

# Run all tests (1,847 pass, 0 fail as of v0.1.0).
# Use plain `cargo test` (the workspace default-members), NOT
# `cargo test --workspace`: the latter pulls in the PyO3 extension-module
# crate (xalen-python), which cannot link as a test binary.
cargo test
# The Python bindings are tested via maturin:  cd crates/xalen-python && maturin develop && pytest

# Build WASM
cd crates/xalen-wasm && wasm-pack build --target web

# Run benchmarks
cargo bench
```

## Project Stats

- ~50,000 lines of Rust
- 18 crates (14 core + 4 binding/rendering crates)
- 1,847 tests, 0 failures
- 116 source files
- Zero `unsafe` in core crates

---

## License

Licensed under the [Apache License, Version 2.0](LICENSE).

```
Copyright 2024-2026 XALEN Technology Pvt Ltd.
```

For commercial licensing inquiries, contact [hello@xalen.ai](mailto:hello@xalen.ai).

---

## Contributing

Contributions are welcome. Please open an issue before starting work on
significant changes.

**Before submitting a pull request:**

1. Run `cargo test` and `cargo clippy --workspace --exclude xalen-python -- -D warnings` -- both must pass
2. Add tests for new computations -- every astrological formula needs at least
   one known-answer test case against a published reference
3. Follow `rustfmt` defaults
4. Cite sources in code comments: textbook name, chapter, verse/page number.
   Wikipedia and blog posts are not acceptable sources for astrological
   algorithms

**Areas where contributions are especially welcome:**

- Expanded fixed star catalog
- DE441 reader support
- Additional world tradition systems
- Documentation and examples
- Cross-validation test vectors against Swiss Ephemeris or JPL Horizons
