# Crates and bindings

XALEN Ephemeris is a Cargo workspace. The umbrella crate
[`xalen-ephemeris`](https://crates.io/crates/xalen-ephemeris) re-exports the
sub-crates (`xalen_ephemeris::ephem`, `::vedic`, `::time`, …); depend on
individual crates instead for a smaller footprint.

```toml
[dependencies]
xalen-ephem = "0.7"         # planetary engine
xalen-vedic = "0.7"         # Vedic astrology
xalen-houses = "0.7"        # house systems
xalen-ayanamsa = "0.7"      # ayanamsa / sidereal conversion
xalen-time = "0.7"          # Julian Day, delta-T, calendars
```

| Crate | Purpose |
|-------|---------|
| [`xalen-ephem`](../crates/xalen-ephem) | Planetary engine: VSOP87A, ELP2000-82, DE440 reader, Pluto, Chiron, lunar nodes, asteroids, eclipses |
| [`xalen-time`](../crates/xalen-time) | Julian Day types (TT/UT1/TDB), delta-T models, calendar conversions |
| [`xalen-coords`](../crates/xalen-coords) | Coordinate transforms, IAU 2006 precession, IAU 2000B nutation, obliquity |
| [`xalen-houses`](../crates/xalen-houses) | 23 house systems with Ascendant, MC, Vertex, polar fallback, city geocoding |
| [`xalen-ayanamsa`](../crates/xalen-ayanamsa) | 50 ayanamsa systems for tropical-to-sidereal conversion |
| [`xalen-stars`](../crates/xalen-stars) | 9,096 compiled-in Bright Star Catalogue stars (feature `bright-catalog`, default-on) + 108-star curated core + runtime CSV loader, proper motion, conjunction search (the 503-star astrology catalog lives in `xalen-western`) |
| [`xalen-vedic`](../crates/xalen-vedic) | Vedic astrology: dasha, shadbala, KP, Jaimini, Tajaka, ashtakavarga, panchang, compatibility, yoga, dosha, upagraha, transit |
| [`xalen-western`](../crates/xalen-western) | Western astrology: aspects, dignities, Arabic Lots, Hellenistic, Uranian, Cosmobiology, returns, progressions, harmonics, horary |
| [`xalen-chinese`](../crates/xalen-chinese) | BaZi, Zi Wei Dou Shu, Feng Shui (Flying Stars, Ba Zhai); Qi Men Dun Jia (experimental — authoritative reference data + a simplified chart assembly, not an authoritative reading) |
| [`xalen-lalkitab`](../crates/xalen-lalkitab) | Lal Kitab: planet-house effects, debts, dormancy, remedies |
| [`xalen-iching`](../crates/xalen-iching) | I Ching: 64 hexagrams, 8 trigrams, all 384 Legge line texts, date casting |
| [`xalen-numerology`](../crates/xalen-numerology) | Pythagorean and Chaldean numerology |
| [`xalen-world`](../crates/xalen-world) | Mayan, Aztec, Tibetan, Persian, Egyptian, Celtic, Korean Saju, Nine Star Ki; Burmese Mahabote (day-sign / ruling-planet profile only — not the full 7-house square) |
| [`xalen-chart`](../crates/xalen-chart) | SVG chart rendering: North Indian, South Indian, Western wheel |
| [`xalen-relationship`](../crates/xalen-relationship) | Relationship astrology: synastry, composite and Davison charts, relocated charts |
| [`xalen-maps`](../crates/xalen-maps) | Astrocartography: MC, IC, Ascendant, Descendant and zenith lines, and parans |
| [`xalen-ffi`](../crates/xalen-ffi) | C FFI exports (`extern "C"` with `repr(C)` structs) |
| [`xalen-wasm`](../crates/xalen-wasm) | WASM bindings via wasm-bindgen |
| [`xalen-python`](../crates/xalen-python) | Python bindings via PyO3 |
| [`xalen-node`](../crates/xalen-node) | Node.js bindings via napi-rs |
| [`xalen-cli`](../crates/xalen-cli) | The `xalen` command line: planetary positions, charts, panchang, dasha, eclipses and more |
| [`xalen-cloud`](../crates/xalen-cloud) | Local-first interpretation shim: the chart is computed on-device; only the optional interpretation step is delegated to a pluggable provider |
| [`xalen-star-anchors`](../crates/xalen-star-anchors) | Reference star anchors (single factual J2000 coordinates) for sidereal ayanamsa computation, Apache-2.0 |
| [`xalen-stars-hip-data`](../crates/xalen-stars-hip-data) | Optional Hipparcos-derived star catalog data (CC BY-NC, non-commercial); unpublished add-on, not used by any published crate |

The `validation/` directory holds `xalen-validation`, the unpublished runner
that compares XALEN against a `pyswisseph` oracle at any chart count (see
[ACCURACY.md](ACCURACY.md)).

## Language bindings

| Language | Crate | Mechanism | Status |
|----------|-------|-----------|--------|
| **Rust** | `xalen-ephemeris` (umbrella) | Native | On crates.io |
| **C / C++** | `xalen-ffi` | `extern "C"` + `repr(C)` structs | Source-stable; on crates.io |
| **Python** | `xalen-python` | PyO3 | Alpha; on PyPI as `xalen` (`pip install xalen`) |
| **Node.js** | `xalen-node` | napi-rs native addon, prebuilt for Linux x64 and arm64, macOS x64 and arm64, and Windows x64 | Alpha; on npm as `@xalen/ephemeris` with each release from 0.8, build from source until then |
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
[BINDING_IDS.md](BINDING_IDS.md).

## Building

```bash
# Build the full workspace
cargo build --release

# Run all tests. Plain `cargo test` runs the workspace default-members (which
# exclude the PyO3 extension-module crate xalen-python, tested separately via
# maturin — see below).
cargo test
# The Python bindings are tested via maturin:  cd crates/xalen-python && maturin develop && pytest

# Build WASM
cd crates/xalen-wasm && wasm-pack build --target web

# Run benchmarks
cargo bench
```

The full pre-merge gate (formatting, clippy, tests, docs, the release-config
tests and the accuracy harness) is `bash scripts/gate.sh`; see
[CONTRIBUTING.md](../CONTRIBUTING.md).
