# XALEN Ephemeris Playground

Interactive browser-based Vedic birth chart calculator powered by the XALEN Ephemeris WASM module.

All computation runs client-side -- zero server calls.

## Quick Start (mock mode)

The playground works immediately without compiling WASM. It will show mock data
for a sample chart (Pune, 1990-01-15, 06:30, Lahiri).

```bash
# From the repository root:
cd playground
python3 -m http.server 8000
# Open http://localhost:8000
```

Or with Node.js:

```bash
npx serve playground
```

## Building WASM for Live Computation

To enable real ephemeris computation in the browser:

### Prerequisites

```bash
# Install wasm-pack (one-time)
cargo install wasm-pack
```

### Build

```bash
# From the repository root:
cd crates/xalen-wasm
wasm-pack build --target web --out-dir ../../playground/pkg
```

This produces `playground/pkg/` containing:
- `xalen_wasm.js` -- ES module glue code
- `xalen_wasm_bg.wasm` -- compiled WASM binary
- `xalen_wasm.d.ts` -- TypeScript definitions

### Serve and Open

```bash
cd ../../playground
python3 -m http.server 8000
```

Open `http://localhost:8000`. The status badge will show "Live WASM Computation"
when the module loads successfully.

## What It Does

The playground provides a form with:

- **Date**: Year, month, day
- **Time**: Hour, minute (24h format)
- **City**: 70+ cities from the built-in geocoding table (from `xalen-houses`)
- **Ayanamsa**: 17 systems (Lahiri, KP, Raman, Fagan-Bradley, etc.)
- **House System**: 14 systems (Whole Sign, Placidus, Koch, etc.)

On submit, it computes and displays:

- **Planet positions** (sidereal longitude, rashi, nakshatra, pada, house)
- **North Indian diamond chart** (SVG, matching `xalen-chart` layout)
- **House cusps** (12 houses in the selected system)
- **Panchang** (tithi, nakshatra, yoga, karana)
- **Metadata** (Julian Day, ayanamsa value, ascendant, MC)

## WASM API Used

The playground calls these `XalenWasm` methods (defined in `crates/xalen-wasm/src/lib.rs`):

| Method | Purpose |
|--------|---------|
| `julianDay(year, month, day, hour)` | Convert calendar date to Julian Day |
| `fullChartJson(jd, lat, lon, ayanamsa_id)` | Full chart: planets + ascendant + MC |
| `housesJson(jd, lat, lon, system_id)` | House cusps for any system |
| `ayanamsaDeg(jd, ayanamsa_id)` | Ayanamsa value in degrees |
| `panchangJson(jd, ayanamsa_id)` | Panchang (tithi, nakshatra, yoga, karana) |

## Architecture

```
playground/
  index.html    -- Single self-contained file (HTML + CSS + JS)
  pkg/          -- WASM output (created by wasm-pack build)
    xalen_wasm.js
    xalen_wasm_bg.wasm
    xalen_wasm.d.ts
```

No build tools, no bundler, no npm install. The HTML file uses native ES module
`import()` to load the WASM glue code. If the import fails (no `pkg/` directory),
it falls back to mock data.

## Data Sources

- **City coordinates**: Hardcoded from `crates/xalen-houses/src/geocoding.rs`
- **Ayanamsa IDs**: From `crates/xalen-wasm/src/lib.rs` `ayanamsa_from_id()`
- **House system IDs**: From `crates/xalen-wasm/src/lib.rs` `house_system_from_id()`
- **SVG layout**: Mirrors `crates/xalen-chart/src/north_indian.rs`
