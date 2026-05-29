# Replacing the `sweph` npm Package with XALEN

The [`sweph`](https://www.npmjs.com/package/sweph) npm package wraps the Swiss
Ephemeris C library via native addons. It requires C compilation at install time,
ships 100+ MB of `.se1` ephemeris data files, and uses process-global state that
is not thread-safe.

`@xalen/ephemeris` is a pure-Rust native addon (via napi-rs) with zero C
dependencies, no data files, and a thread-safe API. This guide covers the 10
most-used `sweph` functions and shows the exact XALEN replacement.

---

## Installation

```bash
# Remove sweph
npm uninstall sweph

# Install XALEN
npm install @xalen/ephemeris
```

No `node-gyp`, no Python, no C compiler needed. The npm package ships
prebuilt binaries for Linux x64/arm64, macOS x64/arm64, and Windows x64.

---

## Import change

```diff
- const sweph = require('sweph');
+ const xalen = require('@xalen/ephemeris');
```

---

## 1. `swe_calc_ut` -- Planet position

### sweph

```js
const sweph = require('sweph');
sweph.swe_set_ephe_path('/path/to/ephe');

const result = sweph.swe_calc_ut(jd, sweph.SE_SUN, sweph.SEFLG_SWIEPH | sweph.SEFLG_SPEED);
const longitude = result.longitude;
const latitude  = result.latitude;
const distance  = result.distance;
```

### XALEN

```js
const xalen = require('@xalen/ephemeris');

// By name (returns tropical longitude in degrees)
const longitude = xalen.planetLongitude('Sun', jd);

// By integer ID (0=Sun, 1=Moon, 2=Mercury, ...)
const longitude2 = xalen.planetLongitudeById(0, jd);
```

### Body ID mapping

| sweph constant | value | XALEN name | XALEN ID |
|---|---|---|---|
| `SE_SUN` | 0 | `"Sun"` | 0 |
| `SE_MOON` | 1 | `"Moon"` | 1 |
| `SE_MERCURY` | 2 | `"Mercury"` | 2 |
| `SE_VENUS` | 3 | `"Venus"` | 3 |
| `SE_MARS` | 4 | `"Mars"` | 4 |
| `SE_JUPITER` | 5 | `"Jupiter"` | 5 |
| `SE_SATURN` | 6 | `"Saturn"` | 6 |
| `SE_URANUS` | 7 | `"Uranus"` | 7 |
| `SE_NEPTUNE` | 8 | `"Neptune"` | 8 |
| `SE_PLUTO` | 9 | `"Pluto"` | 11 |
| `SE_MEAN_NODE` | 10 | `"MeanNode"` or `"Rahu"` | 9 |
| `SE_TRUE_NODE` | 11 | `"TrueNode"` | 10 |
| `SE_CHIRON` | 15 | `"Chiron"` | 12 |
| -- (Ketu) | -- | -- | 13 |

> **Important:** Pluto is ID 11 (not 9) in XALEN. Mean Node is 9 (not 10).

---

## 2. `swe_houses` -- House cusps

### sweph

```js
const result = sweph.swe_houses(jd, lat, lon, 'P'); // 'P' = Placidus
const cusps = result.house;     // array of 12+1 (index 1-12)
const asc   = result.ascendant; // ascmc[0]
const mc    = result.mc;        // ascmc[1]
```

### XALEN

```js
// By name (returns array of 12 cusp values in degrees)
const cusps = xalen.houses(jd, lat, lon, 'placidus');

// By ID (2=Placidus)
const cusps2 = xalen.housesById(jd, lat, lon, 2);
```

**Note:** XALEN returns a 0-indexed array of 12 elements.
sweph returns a 1-indexed array where index 0 is unused.

### House system mapping

| sweph char | XALEN name | XALEN ID |
|---|---|---|
| `'P'` | `"placidus"` | 2 |
| `'K'` | `"koch"` | 3 |
| `'O'` | `"porphyry"` | 4 |
| `'R'` | `"regiomontanus"` | 5 |
| `'C'` | `"campanus"` | 6 |
| `'A'` | `"equal"` | 1 |
| `'W'` | `"wholesign"` | 0 |
| `'M'` | `"morinus"` | 7 |
| `'B'` | `"alcabitius"` | 8 |
| `'T'` | `"topocentric"` | 9 |
| `'S'` | `"sripati"` | 10 |
| `'V'` | `"vehlow"` | 11 |
| `'X'` | `"meridian"` | 12 |
| `'U'` | `"krusinskipisa"` | 13 |

---

## 3. `swe_get_ayanamsa_ut` -- Ayanamsa

### sweph

```js
sweph.swe_set_sid_mode(sweph.SE_SIDM_LAHIRI, 0, 0);
const ayanamsa = sweph.swe_get_ayanamsa_ut(jd);
```

### XALEN

```js
// By ID (0=Lahiri, 1=KP, 2=Raman, ...)
const ayanamsa = xalen.ayanamsaById(jd, 0);
```

No global `set_sid_mode` call needed. The ayanamsa system is a parameter to
each call, so there is no risk of cross-contamination between callers.

---

## 4. `swe_calc_ut` with `SEFLG_SIDEREAL` -- Sidereal longitude

### sweph

```js
sweph.swe_set_sid_mode(sweph.SE_SIDM_LAHIRI, 0, 0);
const result = sweph.swe_calc_ut(jd, sweph.SE_SUN,
    sweph.SEFLG_SWIEPH | sweph.SEFLG_SIDEREAL);
const sidLon = result.longitude;
```

### XALEN

```js
// By name
const sidLon = xalen.siderealLongitude('Sun', jd, 'lahiri');

// By ID (body=0, ayanamsa=0)
const sidLon2 = xalen.siderealLongitudeById(0, jd, 0);
```

---

## 5. `swe_set_ephe_path` -- Ephemeris path

### sweph

```js
sweph.swe_set_ephe_path('/usr/local/share/sweph/ephe');
```

### XALEN

```js
// Not needed -- all data is embedded in the binary.
// Simply remove this line.
```

---

## 6. `swe_close` -- Cleanup

### sweph

```js
sweph.swe_close();
```

### XALEN

```js
// Not needed -- no global state to clean up.
// Simply remove this line.
```

---

## 7. `swe_julday` -- Calendar to Julian Day

### sweph

```js
const jd = sweph.swe_julday(1990, 6, 15, 10.5, sweph.SE_GREG_CAL);
```

### XALEN

```js
// Use xalen.deltaT(jd) for delta-T; for JD conversion, use standard formulas
// or compute in your application layer. XALEN's Rust API exposes calendar_to_jd
// directly; the Node binding does not yet include it. For now:
//
//   function julday(y, m, d, h) {
//     const a = Math.floor((14 - m) / 12);
//     const Y = y + 4800 - a;
//     const M = m + 12 * a - 3;
//     return d + Math.floor((153 * M + 2) / 5) + 365 * Y
//          + Math.floor(Y / 4) - Math.floor(Y / 100) + Math.floor(Y / 400)
//          - 32045 + (h - 12) / 24;
//   }
```

This is a standard formula identical across all ephemeris libraries.

---

## 8. `swe_deltat_ex` -- Delta-T

### sweph

```js
const dt = sweph.swe_deltat_ex(jd, sweph.SEFLG_SWIEPH);
// dt is in fraction of a day
```

### XALEN

```js
const dtSeconds = xalen.deltaT(jd);
// XALEN returns seconds (not days). Divide by 86400 if you need days.
```

---

## 9. `swe_fixstar_ut` -- Fixed stars

### sweph

```js
const result = sweph.swe_fixstar_ut('Spica', jd, sweph.SEFLG_SWIEPH);
const starLon = result.longitude;
```

### XALEN

```js
// Find all stars within an orb of a planet longitude
const hits = xalen.fixedStarConjunctions(planetLon, 2.0, 2026.0);
// Returns: [{ name: "Spica", distance: 0.5, constellation: "Virgo",
//             magnitude: 0.98, nature: "Venus-Mars" }]
```

XALEN's star API is conjunction-oriented: you give it a longitude and orb,
and it returns matching stars. This is what most astrology software actually
needs. The catalog has 108 stars with proper motion.

---

## 10. `swe_set_sid_mode` -- Set sidereal mode

### sweph

```js
sweph.swe_set_sid_mode(sweph.SE_SIDM_KRISHNAMURTI, 0, 0);
// Now all swe_calc_ut calls with SEFLG_SIDEREAL use KP
```

### XALEN

```js
// No global mode -- pass ayanamsa to each call:
const kpLon = xalen.siderealLongitude('Moon', jd, 'kp');
const lahiriLon = xalen.siderealLongitude('Moon', jd, 'lahiri');
// Both work independently in the same process, even concurrently.
```

### Ayanamsa name mapping

| sweph constant | XALEN name | XALEN ID |
|---|---|---|
| `SE_SIDM_LAHIRI` | `"lahiri"` | 0 |
| `SE_SIDM_KRISHNAMURTI` | `"kp"` | 1 |
| `SE_SIDM_RAMAN` | `"raman"` | 2 |
| `SE_SIDM_FAGAN_BRADLEY` | `"faganbradley"` | 3 |
| `SE_SIDM_TRUE_CITRA` | `"truechitra"` | 4 |
| `SE_SIDM_TRUE_REVATI` | `"truerevati"` | 5 |

---

## Complete migration example

### Before (sweph)

```js
const sweph = require('sweph');

sweph.swe_set_ephe_path('/usr/share/sweph/ephe');
sweph.swe_set_sid_mode(sweph.SE_SIDM_LAHIRI, 0, 0);

const jd = sweph.swe_julday(1990, 6, 15, 10.5, sweph.SE_GREG_CAL);
const sun = sweph.swe_calc_ut(jd, sweph.SE_SUN,
    sweph.SEFLG_SWIEPH | sweph.SEFLG_SIDEREAL);
const moon = sweph.swe_calc_ut(jd, sweph.SE_MOON,
    sweph.SEFLG_SWIEPH | sweph.SEFLG_SIDEREAL);
const cusps = sweph.swe_houses(jd, 18.52, 73.85, 'P');
const aya = sweph.swe_get_ayanamsa_ut(jd);

console.log(`Sun: ${sun.longitude}, Moon: ${moon.longitude}`);
console.log(`ASC: ${cusps.ascendant}, MC: ${cusps.mc}`);
console.log(`Ayanamsa: ${aya}`);

sweph.swe_close();
```

### After (XALEN)

```js
const xalen = require('@xalen/ephemeris');

// No init needed -- no ephe path, no sid mode, no close

const jd = 2448078.9375; // 1990-06-15 10:30 UT (or compute from your date lib)
const sunLon  = xalen.siderealLongitude('Sun', jd, 'lahiri');
const moonLon = xalen.siderealLongitude('Moon', jd, 'lahiri');
const cusps   = xalen.houses(jd, 18.52, 73.85, 'placidus');
const aya     = xalen.ayanamsaById(jd, 0);

console.log(`Sun: ${sunLon}, Moon: ${moonLon}`);
// ASC and MC: use houses() result (array of 12 cusps)
console.log(`Ayanamsa: ${aya}`);

// No cleanup needed
```

### What you gain

- **No native compilation.** `npm install` just works, even on CI without
  build tools.
- **No data files.** No 100+ MB of `.se1` files to ship or configure paths for.
- **Thread-safe.** Use in worker threads without fear of global-state
  corruption from `swe_set_sid_mode`.
- **Smaller footprint.** The XALEN binary is ~2 MB vs ~100 MB for sweph + data.
- **MIT licensed.** No GPL concerns for commercial products.
- **Vedic features built in.** Nakshatra, Rashi, Panchang, Dasha -- all in the
  same package, no additional libraries needed.

```js
// Bonus: Vedic features that sweph doesn't have
const nak = xalen.nakshatra(moonLon);           // "Rohini"
const rashi = xalen.rashi(moonLon);             // "Vrishabha (Taurus)"
const panch = xalen.panchang(sunLon, moonLon, jd); // {tithi, nakshatra, yoga, karana, vara}
```
