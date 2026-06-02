# xalen (Node.js)

**Pure-Rust astronomical & astrology ephemeris for Node.js — VSOP87 planets, Vedic/KP, 14 house systems, 17 ayanamsa, nakshatra/panchang, numerology. A native N-API addon (napi-rs), zero data files.**

[![npm](https://img.shields.io/npm/v/xalen.svg)](https://www.npmjs.com/package/xalen) <!-- coming soon: not yet published -->
[![CI](https://github.com/vedika-io/xalen-ephemeris/actions/workflows/ci.yml/badge.svg)](https://github.com/vedika-io/xalen-ephemeris/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://github.com/vedika-io/xalen-ephemeris/blob/main/LICENSE)

> Part of the [XALEN Ephemeris](https://github.com/vedika-io/xalen-ephemeris) suite — pure-Rust core, zero `unsafe` in the math, thread-safe, Apache-2.0. This package is a compiled **native addon** (fast, synchronous). For a pure-WebAssembly build that runs in the browser, use [`xalen-ephemeris`](https://github.com/vedika-io/xalen-ephemeris/tree/main/crates/xalen-wasm) (the WASM package) instead.

---

## Install

```bash
# FORTHCOMING — not on npm yet.
npm install xalen
```

> **Pre-publish note:** `xalen` is **not on npm yet** (the badge above is a placeholder). Until it ships, build the addon from the repo:

```bash
git clone https://github.com/vedika-io/xalen-ephemeris.git
cd xalen-ephemeris/crates/xalen-node
npm install
npm run build          # release; emits index.js + index.d.ts + xalen.<platform>.node
npm test               # node --test __test__/
```

Requires Node.js ≥ 16.

### Platform support

The published package ships **prebuilt** N-API addons as per-platform
`optionalDependencies` (npm installs only the one matching the host). npm
selects it automatically; no compiler is needed on the install host.

| Platform | npm triple | Built by |
|----------|------------|----------|
| macOS arm64 | `aarch64-apple-darwin` | `release.yml` (native) · `build-all-platforms.sh` (native) |
| macOS x86_64 | `x86_64-apple-darwin` | `release.yml` (native) · `build-all-platforms.sh` (on a macOS host) |
| Linux x86_64 (gnu) | `x86_64-unknown-linux-gnu` | `release.yml` · `build-all-platforms.sh` (zig) |
| Linux aarch64 (gnu) | `aarch64-unknown-linux-gnu` | `release.yml` · `build-all-platforms.sh` (zig) |
| Windows x86_64 | `x86_64-pc-windows-msvc` | `release.yml` (native windows runner) |

`scripts/build-all-platforms.sh` builds the addon matrix locally (the Windows
cross-build it produces targets `*-pc-windows-gnu` via `cargo-zigbuild` for
local verification; the **published** Windows addon is the `*-msvc` build from
[`.github/workflows/release.yml`](https://github.com/vedika-io/xalen-ephemeris/blob/main/.github/workflows/release.yml), which builds + tests + publishes all five
platforms above on official runners on every `v*` tag). To compile locally for
the host platform instead of using a prebuilt addon, build from the repo as
shown above.

```js
const xalen = require('xalen');
// or, with ESM:  import xalen from 'xalen';
```

---

## `planetPosition` — full 6-tuple + retrograde

The high-fidelity call: the pyswisseph `calc_ut(..., FLG_SPEED)` six components plus a retrograde flag.

```js
const xalen = require('xalen');
const J2000 = 2451545.0;

const p = xalen.planetPosition('sun', J2000);
// {
//   longitude: 280.37, latitude: 0.0, distance: 0.9833,   // AU
//   lonSpeed: 1.0194, latSpeed: ..., distSpeed: ...,       // per day
//   isRetrograde: false,
// }
```

`longitude`/`latitude`/speeds are in degrees (speeds per day); `distance`/`distSpeed` in AU. `longitude` is wrapped to `[0, 360)`. `isRetrograde` is taken from the tropical longitude rate.

### By integer ID — tropical or sidereal, with Ketu

```js
// body_id: 0=Sun 1=Moon 2=Mercury 3=Venus 4=Mars 5=Jupiter 6=Saturn 7=Uranus
//          8=Neptune 9=MeanNode(Rahu) 10=TrueNode 11=Pluto 12=Chiron 13=Ketu

const sunTrop = xalen.planetPositionById(0, J2000);        // tropical (ayanamsa omitted)
const sunSid  = xalen.planetPositionById(0, J2000, 0);     // sidereal, Lahiri (ayanamsa id 0)

// Mean node is always retrograde; Ketu (id 13) = Rahu + 180, sharing its state.
const rahu = xalen.planetPositionById(9, J2000);   // isRetrograde === true
const ketu = xalen.planetPositionById(13, J2000);  // longitude === (rahu.longitude + 180) % 360
```

When a valid `ayanamsaId` is supplied, the longitude is sidereal (tropical − ayanamsa) **and** the ayanamsa's own precession rate is removed from `lonSpeed`, matching Swiss `SEFLG_SIDEREAL | SEFLG_SPEED`.

---

## fullChart

Nine grahas (+ Ketu) with nakshatra/pada/rashi/lord, plus Whole-Sign ascendant, MC, ayanamsa, and the 12 cusps.

```js
const c = xalen.fullChart(J2000, 18.52, 73.85, 0); // lat, lon, ayanamsaId (0=Lahiri)
// {
//   planets: {
//     Sun:  { longitude, nakshatra, pada, rashi, lord },
//     Moon: { ... }, Mars: { ... }, ..., Rahu: { ... }, Ketu: { ... }
//   },
//   ascendant: <deg>, mc: <deg>, ayanamsaDeg: <deg>, cusps: [12 deg],
// }
console.log(c.planets.Sun.rashi, c.ascendant);
```

---

## nakshatra

```js
// Bare name from a sidereal longitude.
xalen.nakshatra(123.45);          // "Magha"  (string)

// Structured detail — the unified shape shared with the Python/WASM bindings.
const n = xalen.nakshatraInfo(0.0);
// { name: "Ashwini", pada: 1, lord: "Ketu", deity: "...", index: 0 }

xalen.rashi(123.45);              // sidereal sign name, e.g. "Simha (Leo)"
```

---

## More functions

```js
const J2000 = 2451545.0;

// Longitudes by name or id.
xalen.planetLongitude('moon', J2000);                  // tropical (geometric), deg
xalen.siderealLongitude('moon', J2000, 'lahiri');      // sidereal, named ayanamsa
xalen.planetLongitudeById(1, J2000);                   // tropical, by id
xalen.siderealLongitudeById(1, J2000, 0);              // sidereal, by ayanamsa id

// Houses. String name returns the full object (12 cusps + angles + fallbackUsed);
// the *ById variant returns just the 12 cusps as a number[].
const h = xalen.houses(J2000, 18.52, 73.85, 'placidus');
// { cusps: [12], ascendant, mc, ic, descendant, vertex, fallbackUsed }
xalen.housesById(J2000, 18.52, 73.85, 2);              // -> number[12]

// Ayanamsa (17 systems; id 0 = Lahiri), ΔT (SMH 2016 model).
xalen.ayanamsaById(J2000, 0);
xalen.deltaT(J2000);

// Panchang from sidereal Sun + Moon longitudes.
xalen.panchang(/*sunLon*/ 90.0, /*moonLon*/ 200.0, J2000);
// { tithi_number, tithi_name, paksha, nakshatra, yoga_number, yoga_name, karana_name, vara }

// Fixed stars near a longitude, and numerology.
xalen.fixedStarConjunctions(/*planetLon*/ 69.0, /*orb*/ 1.0, /*year*/ 2000.0);
xalen.lifePath(1990, 3, 15);                           // single digit / master number
xalen.expressionNumber('Ada Lovelace', 'pythagorean'); // or 'chaldean'
```

Accepted body names: `sun, moon, mercury, venus, earth, mars, jupiter, saturn, uranus, neptune, pluto, rahu`/`mean_node`, `true_node`, `chiron`, `lilith`/`mean_apogee`. Ayanamsa names: `lahiri, kp, raman, fagan-bradley, true-chitra, ...` (dashes/spaces ignored). House names: `whole-sign, equal, placidus, koch, porphyry, regiomontanus, campanus, morinus, alcabitius, topocentric, meridian, vehlow, sripati, krusinski-pisa`.

---

## Accuracy — honest bounds

Cross-validated against JPL Horizons (DE440), the real DE440 binary kernel, the official VSOP87 check file, and Swiss Ephemeris. XALEN **matches** these to the bounds below; DE440 *is* the reference (we don't "beat" it). The genuine differentiator is pure Rust, zero `unsafe` core, thread-safe, Apache-2.0, no data files.

- **Sun + Mercury–Saturn:** sub-arcsecond (Sun 0.21″, Mercury–Saturn ≤ 0.76″; 20k-chart bound) vs DE440 (analytical engine).
- **Uranus/Neptune:** Uranus 1.78″, Neptune 2.53″ (20k-chart bound). **Moon:** ~3″ RMS (max ~12″), ELP2000-82. **Pluto:** ~1 arcminute, 1885–2099.
- **vs Swiss Ephemeris:** 0 of 5,000,000 charts over 0.1° for any planet/node, worldwide 1850–2150.

Full report: [docs/ACCURACY.md](https://github.com/vedika-io/xalen-ephemeris/blob/main/docs/ACCURACY.md). Migrating from `sweph`/`swisseph` on npm: [docs/SWEPH_NPM_REPLACEMENT.md](https://github.com/vedika-io/xalen-ephemeris/blob/main/docs/SWEPH_NPM_REPLACEMENT.md).

---

## TypeScript

`index.d.ts` is generated by napi-rs at build time, so all functions, the `PlanetPosition` and `NakshatraInfo` object shapes, and parameter types are fully typed out of the box.

---

## License

Apache-2.0. See [LICENSE](https://github.com/vedika-io/xalen-ephemeris/blob/main/LICENSE) and [CREDITS.md](https://github.com/vedika-io/xalen-ephemeris/blob/main/CREDITS.md).
