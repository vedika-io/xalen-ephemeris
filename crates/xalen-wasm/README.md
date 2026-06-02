# xalen-ephemeris (WASM)

**Pure-Rust astronomical ephemeris for astrology, compiled to WebAssembly — Vedic charts, panchang, houses, dasha, and compatibility in any browser or Node.js. Zero data files, zero native dependencies.**

[![npm](https://img.shields.io/npm/v/xalen-ephemeris.svg)](https://www.npmjs.com/package/xalen-ephemeris) <!-- coming soon: not yet published -->
[![CI](https://github.com/vedika-io/xalen-ephemeris/actions/workflows/ci.yml/badge.svg)](https://github.com/vedika-io/xalen-ephemeris/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://github.com/vedika-io/xalen-ephemeris/blob/main/LICENSE)

> Part of the [XALEN Ephemeris](https://github.com/vedika-io/xalen-ephemeris) suite — pure-Rust core, zero `unsafe` in the math, thread-safe, Apache-2.0. This package is the **WebAssembly** build (runs in the browser and in Node). For a faster, synchronous **native** Node addon, use the [`xalen`](https://github.com/vedika-io/xalen-ephemeris/tree/main/crates/xalen-node) package instead.

This crate wraps the XALEN core crates behind a single `XalenWasm` handle exported to JavaScript via `wasm-bindgen`. The same handle also compiles to native Rust, so it can be exercised in ordinary `cargo test`.

---

## Install

```bash
# FORTHCOMING — not on npm yet.
npm install xalen-ephemeris
```

> **Pre-publish note:** `xalen-ephemeris` is **not on npm yet** (the badge above is a placeholder). Until it ships, build the package from the repo:

```bash
git clone https://github.com/vedika-io/xalen-ephemeris.git
cd xalen-ephemeris/crates/xalen-wasm
wasm-pack build --target web        # browser; or --target nodejs / --target bundler
# emits pkg/ with xalen_wasm_bg.wasm, xalen_wasm.js, and xalen_wasm.d.ts
```

### Platform support

WebAssembly is **platform-independent**: a single `.wasm` artifact runs on every
CPU and OS — any modern browser and any Node.js ≥ 16. There is no per-platform
build to select.

`scripts/build-all-platforms.sh wasm` produces the package locally; the durable
build runs on every `v*` tag in
[`.github/workflows/release.yml`](https://github.com/vedika-io/xalen-ephemeris/blob/main/.github/workflows/release.yml) (`wasm-pack build --target web` →
`wasm-pack publish`). The crate compiles `wasm32-unknown-unknown` only; the same
`XalenWasm` handle also builds natively for `cargo test`.

---

## Usage

In the browser (`--target web`):

```js
import init, { XalenWasm } from "./pkg/xalen_wasm.js";
await init();

const w = new XalenWasm();
const J2000 = 2451545.0;

// Full Vedic chart for Pune (18.52 N, 73.85 E), Lahiri ayanamsa (id 0).
const chart = JSON.parse(w.fullChartJson(J2000, 18.52, 73.85, 0));
console.log(chart.planets.Sun, chart.ascendant_deg);
```

In Node (`--target nodejs`):

```js
const { XalenWasm } = require("xalen-ephemeris");
const w = new XalenWasm();
const aya = XalenWasm.ayanamsaDeg(2451545.0, 0); // Lahiri at J2000 ≈ 23.85°
```

---

## API

The `XalenWasm` handle is constructed once (`new XalenWasm()`) and serves the instance methods; the time/ayanamsa helpers are static.

### Planetary positions

- **`tropicalLongitude(jdUt1, bodyId)`** / **`siderealLongitude(jdUt1, bodyId, ayanamsaId)`** — a single longitude in degrees. Ketu (body 13) is derived as Rahu + 180°.
- **`planetPositionJson(jdUt1, bodyId, sidereal, ayanamsaId)`** — the full state pyswisseph returns from `swe.calc_ut(..., FLG_SPEED)` plus a retrograde flag: `{ longitude, latitude, distance, lon_speed, lat_speed, dist_speed, is_retrograde }`. Angles/speeds in degrees (speeds per day); distance in AU. When `sidereal` is true, the longitude is made sidereal and the ayanamsa's own precession rate is removed from `lon_speed` (matching Swiss `SEFLG_SIDEREAL | SEFLG_SPEED`); `is_retrograde` is taken from the frame-independent tropical rate.

Body IDs: `0`=Sun `1`=Moon `2`=Mercury `3`=Venus `4`=Mars `5`=Jupiter `6`=Saturn `7`=Uranus `8`=Neptune `9`=MeanNode(Rahu) `10`=TrueNode `11`=Pluto `12`=Chiron `13`=Ketu.

### Charts & Vedic

- **`fullChartJson(jdUt1, lat, lon, ayanamsaId)`** — the nine grahas (each `{ longitude, nakshatra, pada, rashi }`) plus Whole-Sign `ascendant_deg`, `mc_deg`, and `ayanamsa_deg`.
- **`panchangJson(jdUt1, ayanamsaId)`** — the five limbs (tithi, vara, nakshatra, yoga, karana).
- **`housesJson(jdUt1, lat, lon, systemId)`** — house cusps across 14 systems (`0`=WholeSign `1`=Equal `2`=Placidus `3`=Koch … `13`=Krusinski).
- **`getNakshatra(moonSiderealDeg)`** — nakshatra name (string).
- **`nakshatraInfoJson(siderealDeg)`** — structured `{ name, pada, lord, deity, index }`, the one-contract shape shared with the Python/Node bindings.
- **`getRashi(siderealDeg)`** — rashi name (string).
- **`vimshottariDasha(moonDeg, birthJd)`** — all 9 Mahadashas, each with Antardasha sub-periods (JSON array).
- **`divisionalChart(lonDeg, varga)`** — the divisional (Varga) sign for D1, D2, D3, D4, D7, D9, D10, D12, D16, D20, D24, D27, D30, D40, D45, D60.

### Compatibility

- **`compatibility(boyMoonDeg, girlMoonDeg)`** — Ashta Koota (8-fold) match from sidereal Moon longitudes in degrees (0–360); returns the 8 koota scores and total (out of 36). Both nakshatra and rashi are resolved from the longitude, so a Moon near a sign boundary maps to the correct rashi.

### Time & ayanamsa helpers (static)

- **`XalenWasm.julianDay(year, month, day, hour)`** — calendar (proleptic Gregorian, UT) to Julian Day.
- **`XalenWasm.ayanamsaDeg(jdUt1, ayanamsaId)`** — ayanamsa value (17 systems).
- **`XalenWasm.deltaT(jd)`** — ΔT (TT − UT1) in seconds (SMH 2016 model).
- **`XalenWasm.bodyName(bodyId)`** — body name string.

> Functions that take a longitude or JD return an error string (thrown value on the JS side) for non-finite or out-of-range input rather than silently returning `NaN`.

```rust
// The handle also runs natively, so it is testable without a browser:
use xalen_wasm::XalenWasm;
let w = XalenWasm::new();
let jd = XalenWasm::julian_day(2000, 1, 1, 12.0);
let aya = XalenWasm::ayanamsa_deg(jd, 0).unwrap();   // Lahiri ≈ 23.85°
assert!(aya > 23.0 && aya < 25.0);
let chart = w.full_chart_json(jd, 18.52, 73.85, 0).unwrap();
assert!(chart.contains("Sun") && chart.contains("Moon"));
```

---

## Accuracy — honest bounds

Positions and derived quantities come from the XALEN core crates (VSOP87A + IAU 2000B nutation; ELP2000-82 for the Moon), cross-validated against JPL Horizons (DE440), the official VSOP87 check file, and Swiss Ephemeris. XALEN **matches** these references — DE440 *is* the reference, we do not "beat" it. Measured bounds (20k-chart bound vs DE440): **Sun 0.21″, Mercury–Saturn ≤ 0.76″ (sub-arcsecond)**; Uranus 1.78″, Neptune 2.53″; Moon ~3″ RMS analytical; Pluto ~1 arcminute in-window. vs Swiss: 0 of 5,000,000 charts over 0.1° for any planet/node, 1850–2150. The genuine differentiator is pure Rust, zero `unsafe` core, thread-safe, Apache-2.0, WebAssembly-ready. Full report → [docs/ACCURACY.md](https://github.com/vedika-io/xalen-ephemeris/blob/main/docs/ACCURACY.md) and [CREDITS.md](https://github.com/vedika-io/xalen-ephemeris/blob/main/CREDITS.md).

---

## TypeScript

`wasm-pack` emits `xalen_wasm.d.ts` alongside the `.wasm`, typing the `XalenWasm` class and every method, so TypeScript consumers get full autocomplete.

---

## License

Apache-2.0. See [LICENSE](https://github.com/vedika-io/xalen-ephemeris/blob/main/LICENSE).
