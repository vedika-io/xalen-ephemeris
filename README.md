# XALEN Ephemeris

**A pure-Rust astronomical ephemeris for astrology — JPL-class planet accuracy, zero `unsafe` core, Apache-2.0.**

[![Crates.io](https://img.shields.io/crates/v/xalen-ephemeris.svg)](https://crates.io/crates/xalen-ephemeris)
[![docs.rs](https://img.shields.io/docsrs/xalen-ephemeris)](https://docs.rs/xalen-ephemeris)
[![CI](https://github.com/vedika-io/xalen-ephemeris/actions/workflows/ci.yml/badge.svg)](https://github.com/vedika-io/xalen-ephemeris/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

> **Packages:** Rust crates on [crates.io](https://crates.io/crates/xalen-ephemeris), Python on PyPI as [`xalen`](https://pypi.org/project/xalen/), and browser / WASM on npm as [`@xalen/wasm`](https://www.npmjs.com/package/@xalen/wasm). The native Node.js addon (`crates/xalen-node`) is published to npm as [`@xalen/ephemeris`](https://www.npmjs.com/package/@xalen/ephemeris) with each release from 0.8, with prebuilt binaries for Linux and macOS (x64 and arm64); until then build it from source.

> **Validated against the references that matter** — JPL Horizons **DE440** (NASA's
> definitive ephemeris), the real DE440 binary kernel, **Swiss Ephemeris**, and the
> public calculators (astro.com, AstroSage, Drik Panchang, Prokerala, Jagannatha
> Hora). **Sun + Mercury–Saturn sub-arcsecond** vs DE440 (Sun 0.21″, Mercury–Saturn
> ≤ 0.76″; Uranus/Neptune ~1.8–2.5″, Pluto arcminute-class in-window). Analytical
> **Moon RMS ~2.8″ / max ~12″** vs pyswisseph (AD 1600–2100); the DE440 kernel takes
> the Moon **sub-arcsecond**. Pure Rust, zero `unsafe` in the core, thread-safe,
> Apache-2.0. Full report → [docs/ACCURACY.md](docs/ACCURACY.md).

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

- **23 house systems** -- Placidus, Koch, Campanus, Regiomontanus, Whole Sign, Equal, Porphyry, Morinus, Alcabitius, Topocentric (Polich-Page), Meridian, Vehlow, Sripati, Krusinski-Pisa, Sunshine (Makransky & Treindl), Pullen Sinusoidal (Delta & Ratio), Carter Poli-Equatorial, APC, Zariel, Alcabitius Classic -- with automatic Porphyry fallback at polar latitudes. (Gauquelin sectors is an experimental placeholder that currently returns Placidus cusps, not a true Gauquelin division.)

- **12+ astrology traditions** -- Vedic/Jyotish (dasha, shadbala, KP, Jaimini, Tajaka, panchang with tithi/nakshatra/yoga/karana transition times, compatibility, 16 divisional charts, yoga, dosha), Western (aspects, dignities, 97 Arabic Lots, Hellenistic, Uranian, Cosmobiology, progressions, exact solar/lunar returns by iterative refinement, declination aspects + antiscia, harmonics, horary), Chinese (BaZi, Zi Wei Dou Shu, Feng Shui Flying Stars, Qi Men Dun Jia [experimental]), Lal Kitab, I Ching (64 hexagrams + all 384 line texts, verbatim public-domain Legge SBE XVI), Numerology, Korean Saju, Japanese Nine Star Ki, Burmese Mahabote (day-sign profile only), Mayan, Aztec, Tibetan, Persian/Zoroastrian, Egyptian, Celtic

- **Relationship charts and astrocartography** -- `xalen-relationship` gives synastry (aspect grid and house overlays), composite charts (Hand) and Davison charts, plus a relocated-chart helper; `xalen-maps` draws astrocartography lines (MC, IC, Ascendant, Descendant and zenith points for each body, Lewis) with circumpolar breaks and antimeridian splitting. Both are geocentric; parans are not yet included

- **9,096 built-in fixed stars** (`xalen-stars`: every star of the Yale Bright Star Catalogue, 5th rev. ed., with IAU-CSN names and positions where a star has an IAU name) plus a 108-star curated core with Ptolemaic natures, and **503 named fixed stars** in `xalen-western` (all mag < 3.0, Behenian, Royal, Nakshatra yogatara, IAU-named to mag ~5). Proper motion and IAU 2006/P03 precession to any epoch, zero data files; the data carries no non-commercial restriction. External catalogs can be loaded from CSV at runtime

- **Solar and lunar eclipse engine** -- rigorous Besselian elements (Explanatory Supplement §11; Meeus Ch. 54) for global type / γ / greatest-eclipse, plus a per-observer local-circumstances layer (magnitude, obscuration, C1–C4 contact times); lunar eclipses from Earth's shadow cones (Danjon) with umbral / penumbral magnitudes and P1–P4 contacts, matching Swiss Ephemeris for every eclipse of 1900–2100. Global type validated against NASA for 2017-08-21 and 2024-04-08

- **Black Moon Lilith** -- both **mean** lunar apogee and **True (osculating)** Lilith (Swiss `SE_OSCU_APOG`), Chiron, True Node, Mean Node

- **Velocity / daily motion** for every body, with `is_retrograde()`,
  **topocentric positions** (diurnal parallax) for an observer's exact location,
  and **equatorial RA/Dec, heliocentric, and rectangular XYZ** output alongside
  the geocentric ecliptic place

- **Time systems** -- UT1 / TT / TDB Julian Day types, UTC↔TAI with the full
  leap-second table, and a genuine Stephenson–Morrison–Hohenkerk 2016 ΔT spline
  with a per-epoch σ envelope built from NASA and USNO public-domain sources

- **15 asteroids** -- the big 4 (Ceres, Pallas, Juno, Vesta), Hygeia, Astraea, Psyche, Eros, Lilith (1181), centaurs (Pholus, Nessus), and TNOs (Eris, Sedna, Makemake, Haumea), plus an external element loader for any asteroid with known orbital elements

- **SVG chart rendering** -- North Indian diamond, South Indian box, and Western wheel charts, zero external dependencies

- **City geocoding** for 130+ cities with latitude, longitude, and timezone

- **NAIF DE440 SPK reader** -- reads real JPL `.bsp` binary files for extended validity range and JPL-grade accuracy; the analytical engine (VSOP87A + ELP2000-82 + IAU 2000B nutation + full IAU 2006/P03 rotation) works standalone with zero data files: **Sun + Mercury–Saturn sub-arcsecond** vs JPL DE440 (Sun 0.21", ≤0.76"), Uranus/Neptune ~1.8–2.5", **Moon RMS ~2.8" / max ~12"** vs pyswisseph (AD 1600–2100; an earlier build mis-applied the planet annual-aberration term to the geocentric Moon for a ~11–31" residual — that bug is fixed, see ACCURACY.md), and Pluto arcminute-class in-window (published Goffin-fit bound ~1′; the committed golden test asserts a 600″/10′ regression ceiling vs DE440 — no test asserts a tighter figure). **Load the DE440 kernel for JPL-grade sub-arcsecond on the Sun, planets and Moon, plus full-range Pluto.** Both engines apply the same apparent-place chain (full-rotation precession + nutation; planets/Sun get 2-pass annual aberration, the geocentric Moon gets geocentric light-time but NOT annual aberration). Numbers reproducible from this repo via `cargo test -p xalen-ephem --test accuracy_vs_de440` (with a `de440s.bsp` kernel present)

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
xalen-ephem = "0.7"         # planetary engine
xalen-vedic = "0.7"         # Vedic astrology
xalen-houses = "0.7"        # house systems
xalen-ayanamsa = "0.7"      # ayanamsa / sidereal conversion
xalen-time = "0.7"          # Julian Day, delta-T, calendars
```

Python and browser / WASM:

```bash
pip install xalen
npm install @xalen/wasm
```

## Quick Start

### Vedic chart: Sun and Moon with nakshatra and rashi

With only `xalen-ephemeris` added, every sub-crate is reachable through the
umbrella re-exports (`xalen_ephemeris::ephem`, `::vedic`, `::time`, …):

```rust,no_run
use xalen_ephemeris::ayanamsa::Ayanamsa;
use xalen_ephemeris::coords::RAD_TO_DEG;
use xalen_ephemeris::ephem::{Almanac, Body};
use xalen_ephemeris::time::{calendar_to_jd, CalendarSystem, JulianDay};
use xalen_ephemeris::vedic::nakshatra::Nakshatra;
use xalen_ephemeris::vedic::rashi::Rashi;

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
use xalen_ephemeris::western::aspects::{find_all_aspects, AspectType};
use xalen_ephemeris::western::dignity::essential_dignity_score;
```

### More examples

```bash
cargo run --example basic_chart      # Sun/Moon + nakshatra + rashi
cargo run --example vedic_chart      # Full Vedic chart with dasha, shadbala, panchang
cargo run --example western_chart    # Aspects, dignities, Arabic Lots
cargo run --example chinese_bazi     # BaZi Four Pillars with Wu Xing
```

---

## Command line

The `xalen` tool puts the library in the terminal: positions, charts, panchang,
dasha, fixed stars, eclipses, rise and set times, aspects, time conversions,
astrocartography maps, synastry, composite, Davison and relocated charts,
as a table or with `--json`. It works offline and has no data files.

```bash
cargo install xalen-cli
```

```bash
# Sidereal (Lahiri) positions with nakshatra and pada
xalen positions --date 2026-11-07T00:00:00Z --sidereal --bodies sun,moon

# Whole-sign Vedic chart for a place in the built-in city table
xalen chart --date 1990-03-15T06:30:00Z --place delhi --sidereal

# Solar eclipses of 2024 as JSON (the 8 April total eclipse is the first)
xalen eclipses --from 2024-01-01 --to 2024-12-31 --type solar --json

# Astrocartography lines for a moment, as GeoJSON for any GIS tool or geojson.io
xalen map --date 2024-06-21T00:00:00Z --bodies sun,moon --geojson

# Aspects and house overlays between two births
xalen synastry --a-date 1990-03-15T06:30:00Z --a-place delhi --b-date 1992-08-02T14:10:00Z --b-place mumbai

# Composite and Davison charts of the same two births
xalen composite --a-date 1990-03-15T06:30:00Z --a-place delhi --b-date 1992-08-02T14:10:00Z --b-place mumbai
xalen davison --a-date 1990-03-15T06:30:00Z --a-place delhi --b-date 1992-08-02T14:10:00Z --b-place mumbai

# The same birth with the houses moved to another place
xalen relocate --date 1990-03-15T06:30:00Z --place delhi --to-place london
```

See [`crates/xalen-cli`](crates/xalen-cli/README.md) for every command.

---

## Accuracy

| Theory | Bodies | Accuracy | Data Files Needed? |
|--------|--------|----------|--------------------|
| VSOP87A + IAU 2000B nutation | Sun | **0.21"** (vs JPL DE440) | No (analytical) |
| VSOP87A + IAU 2000B nutation | Mercury -- Saturn | **0.21--0.76"** (vs JPL DE440, 20k charts) | No (analytical) |
| VSOP87A + IAU 2000B nutation | Uranus / Neptune | 1.78" / 2.53" (vs JPL DE440) | No (analytical) |
| ELP2000-82 (Chapront-Touzé & Chapront; abridged 60+60 terms per Meeus Ch.47) + Δψ + geocentric light-time | Moon | **RMS ~2.8" / max ~12"** (vs pyswisseph 2.10.03, AD 1600--2100; truncation-limited, not aberration) | No (analytical) |
| Goffin (1989) DE200 fit (43 terms per Meeus Ch.37) | Pluto | arcminute-class in-window; golden test asserts ≤600″/10′ vs DE440 (no tighter figure asserted) | No (valid 1885--2099; DE440 for full range) |
| JPL DE440 | Sun + planets / Pluto | sub-arcsecond apparent (raw geometry sub-mas; 2-pass body light-time) | Yes (binary `.bsp`) |
| JPL DE440 | Moon | sub-arcsecond apparent (raw geometry sub-mas; geocentric light-time, no annual aberration) | Yes (binary `.bsp`) |
| IAU 2006 | Precession | ~0.3 mas/century | No |
| IAU 2000B | Nutation | ~1 mas | No |

For most astrological applications, the analytical theories (VSOP87A, ELP2000-82)
provide more than enough precision -- they require zero external data files and
work entirely from compiled-in polynomial series; the analytical Moon now lands
within RMS ~2.8" of Swiss Ephemeris across four centuries. Load DE440 when you
need JPL-grade sub-arcsecond precision on the Moon and outer bodies, full-range
Pluto, or primary directions to the minute.

**Sub-arcsecond Moon with zero manual kernel handling.** Enable the optional
`kernel-autodownload` feature and call `De440Provider::from_auto_cache()`; the
public NASA NAIF `de440s.bsp` kernel (~32 MB) is fetched once into the OS cache
directory, verified, and reused thereafter. The feature is off by default so the
base crate stays offline and data-file-free:

```toml
[dependencies]
xalen-ephem = { version = "0.6", features = ["kernel-autodownload"] }
```

```rust
use xalen_ephem::De440Provider;
// First call fetches + caches de440s.bsp; later calls reuse it (no network).
let provider = De440Provider::from_auto_cache()?;
// Apparent Moon (and all kernel bodies) now sub-arcsecond.
```

Eclipse output now includes both a
global-circumstances classifier (type / Besselian γ / greatest eclipse; the
global figure is a diameter-ratio coverage proxy, not a true magnitude) and a
Besselian local-circumstances engine that does provide a real per-observer
magnitude, obscuration, and C1–C4 contact times.

---

## Crate Map

| Crate | Purpose |
|-------|---------|
| [`xalen-ephem`](crates/xalen-ephem) | Planetary engine: VSOP87A, ELP2000-82, DE440 reader, Pluto, Chiron, lunar nodes, asteroids, eclipses |
| [`xalen-time`](crates/xalen-time) | Julian Day types (TT/UT1/TDB), delta-T models, calendar conversions |
| [`xalen-coords`](crates/xalen-coords) | Coordinate transforms, IAU 2006 precession, IAU 2000B nutation, obliquity |
| [`xalen-houses`](crates/xalen-houses) | 23 house systems with Ascendant, MC, Vertex, polar fallback, city geocoding |
| [`xalen-ayanamsa`](crates/xalen-ayanamsa) | 50 ayanamsa systems for tropical-to-sidereal conversion |
| [`xalen-stars`](crates/xalen-stars) | 9,096 compiled-in Bright Star Catalogue stars (feature `bright-catalog`, default-on) + 108-star curated core + runtime CSV loader, proper motion, conjunction search (the 503-star astrology catalog lives in `xalen-western`) |
| [`xalen-vedic`](crates/xalen-vedic) | Vedic astrology: dasha, shadbala, KP, Jaimini, Tajaka, ashtakavarga, panchang, compatibility, yoga, dosha, upagraha, transit |
| [`xalen-western`](crates/xalen-western) | Western astrology: aspects, dignities, Arabic Lots, Hellenistic, Uranian, Cosmobiology, returns, progressions, harmonics, horary |
| [`xalen-chinese`](crates/xalen-chinese) | BaZi, Zi Wei Dou Shu, Feng Shui (Flying Stars, Ba Zhai); Qi Men Dun Jia (experimental — authoritative reference data + a simplified chart assembly, not an authoritative reading) |
| [`xalen-lalkitab`](crates/xalen-lalkitab) | Lal Kitab: planet-house effects, debts, dormancy, remedies |
| [`xalen-iching`](crates/xalen-iching) | I Ching: 64 hexagrams, 8 trigrams, all 384 Legge line texts, date casting |
| [`xalen-numerology`](crates/xalen-numerology) | Pythagorean and Chaldean numerology |
| [`xalen-world`](crates/xalen-world) | Mayan, Aztec, Tibetan, Persian, Egyptian, Celtic, Korean Saju, Nine Star Ki; Burmese Mahabote (day-sign / ruling-planet profile only — not the full 7-house square) |
| [`xalen-chart`](crates/xalen-chart) | SVG chart rendering: North Indian, South Indian, Western wheel |
| [`xalen-relationship`](crates/xalen-relationship) | Relationship astrology: synastry, composite and Davison charts, relocated charts |
| [`xalen-maps`](crates/xalen-maps) | Astrocartography: MC, IC, Ascendant, Descendant and zenith lines |
| [`xalen-ffi`](crates/xalen-ffi) | C FFI exports (`extern "C"` with `repr(C)` structs) |
| [`xalen-wasm`](crates/xalen-wasm) | WASM bindings via wasm-bindgen |
| [`xalen-python`](crates/xalen-python) | Python bindings via PyO3 |
| [`xalen-node`](crates/xalen-node) | Node.js bindings via napi-rs |

## Language Bindings

| Language | Crate | Mechanism | Status |
|----------|-------|-----------|--------|
| **Rust** | `xalen-ephemeris` (umbrella) | Native | On crates.io |
| **C / C++** | `xalen-ffi` | `extern "C"` + `repr(C)` structs | Source-stable; on crates.io |
| **Python** | `xalen-python` | PyO3 | Alpha; on PyPI as `xalen` (`pip install xalen`) |
| **Node.js** | `xalen-node` | napi-rs native addon, prebuilt for Linux, macOS and Windows x64 | Alpha; on npm as `@xalen/ephemeris` with each release from 0.8, build from source until then |
| **Browser / WASM** | `xalen-wasm` | wasm-bindgen | Alpha; on npm as `@xalen/wasm` |

> The native Node.js addon is published to npm as `@xalen/ephemeris` with each
> release from 0.8, so the `npm install @xalen/ephemeris` command in the migration
> guides works from that release. Until then, build the addon from this
> repository with `napi build`.

Core computation crates compile to `wasm32-unknown-unknown` without modification.

All four bindings expose the full `(lon, lat, dist, lon_speed, lat_speed,
dist_speed)` 6-tuple plus a retrograde flag, matching what
`swe.calc_ut(..., FLG_SPEED)` returns. The Python binding additionally ships an
`xalen.swe` submodule (`import xalen.swe as swe`) intended as a near-drop-in
substitute for `import swisseph as swe` for the common `calc_ut` / `fixstar2`
calls — a search-and-replace migration path, not a 1:1 reimplementation of every
pyswisseph entry point.

The integer IDs the bindings take for bodies, ayanamsas and house systems, and
the Swiss Ephemeris numbers next to them, are listed in
[docs/BINDING_IDS.md](docs/BINDING_IDS.md).

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

# Run all tests. `cargo test --workspace` reports 2,199 passing across the
# library and integration suites, 0 failures. Plain `cargo test` runs the
# workspace default-members (which exclude the PyO3 extension-module crate
# xalen-python, tested separately via maturin — see below).
cargo test
# The Python bindings are tested via maturin:  cd crates/xalen-python && maturin develop && pytest

# Build WASM
cd crates/xalen-wasm && wasm-pack build --target web

# Run benchmarks
cargo bench
```

## Project Stats

- over 70,000 lines of Rust
- 18 crates (14 core + 4 binding/rendering crates)
- 2,199 tests passing (`cargo test --workspace`, library + integration suites), 0 failures
- 150+ source files
- Zero `unsafe` in core crates

---

## License

Licensed under the [Apache License, Version 2.0](LICENSE).

```
Copyright 2024-2026 XALEN Technology Pvt Ltd.
```

For commercial licensing inquiries, contact [hello@xalen.io](mailto:hello@xalen.io).

---

## Enterprise

XALEN Ephemeris is free and open source (Apache-2.0) — and always will be. For
production use that needs an **SLA, IP indemnification, a managed/hosted API,
certified-accuracy reports, white-label, or on-premise support**, see
**[ENTERPRISE.md](ENTERPRISE.md)** or reach us at **hello@xalen.io**. The
open-source core stays Apache-2.0; Enterprise only adds services and optional
proprietary modules on top.

---

## Contributing

Contributions are welcome. Please open an issue before starting work on
significant changes.
The workflow from issue to merged pull request (branch names, commit style,
checks, the contributor agreement) is in [CONTRIBUTING.md](CONTRIBUTING.md).

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
