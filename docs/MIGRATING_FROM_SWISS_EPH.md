# Migrating from Swiss Ephemeris to XALEN Ephemeris

This guide shows side-by-side code for every common Swiss Ephemeris operation
and its XALEN equivalent in Rust, Node.js, and Python.

**Why migrate?** XALEN is pure Rust -- no C dependencies, no `.se1` data files
to ship, and Apache-2.0 licensed (no copyleft, vs Swiss Ephemeris's AGPL-3.0 or
paid commercial license). It compiles to a single binary or `.wasm`.
The API is type-safe, thread-safe (`Send + Sync`), and the same crate powers
all language bindings.

---

## Quick start

> **Availability:** the Rust crates are on crates.io at the **0.3.1** line (pin
> `0.3` or depend on this repo for the newer 0.4.x+ work, which is not yet
> published). The **Node.js (`npm install xalen`) and Python (`pip install xalen`)
> packages are forthcoming — not yet published.** There is an unrelated `xalen`
> package on PyPI (a separate XALEN SDK); it is **not** these ephemeris bindings.
> Until the bindings publish, build them from source: `cd crates/xalen-python &&
> maturin develop` and `cd crates/xalen-node && napi build --release`.

```bash
# Rust (crates.io 0.3.1 line; pin 0.3 or use a git/path dep for newer work)
cargo add xalen-ephem xalen-ayanamsa xalen-houses xalen-time xalen-stars xalen-vedic

# Node.js — FORTHCOMING (not yet on npm); build from source for now
npm install xalen

# Python — FORTHCOMING (not yet on PyPI as these bindings); build from source for now
pip install xalen
```

---

## 1. Planet position (tropical longitude)

### Swiss Ephemeris (C)

```c
#include "swephexp.h"

double xx[6];
char serr[256];
int iflag = SEFLG_SWIEPH | SEFLG_SPEED;

swe_set_ephe_path("/path/to/ephe");
swe_calc_ut(jd, SE_SUN, iflag, xx, serr);
double longitude = xx[0];
double latitude  = xx[1];
double distance  = xx[2];
double lon_speed = xx[3];
```

### XALEN (Rust)

```rust
use xalen_ephem::{Almanac, Body};
use xalen_time::JdUT1;

let almanac = Almanac::default_vedic();
let lon = almanac.geocentric_longitude_deg(Body::Sun, JdUT1(jd))?;

// Full ecliptic position (lon, lat, distance):
let pos = almanac.geocentric_ecliptic(Body::Sun, JdUT1(jd))?;
let longitude = pos.longitude.to_degrees();
let latitude  = pos.latitude.to_degrees();
let distance  = pos.distance; // AU
```

### XALEN (Rust -- compat layer)

```rust
use xalen_ephem::compat::*;

// Drop-in replacement -- same signature as swe_calc_ut
let xx = swe_calc_ut(jd, SE_SUN, SEFLG_SWIEPH)?;
let longitude = xx[0];
let latitude  = xx[1];
```

### XALEN (Node.js)

```js
const xalen = require("xalen");

// By name
const lon = xalen.planetLongitude("Sun", jd);

// By integer ID (0=Sun, 1=Moon, 2=Mercury, ...)
const lon2 = xalen.planetLongitudeById(0, jd);
```

### XALEN (Python)

```python
import xalen

# By integer ID
lon = xalen.planet_longitude(jd, body=0)  # 0=Sun

# By name (convenience)
lon = xalen.planet_longitude_by_name("Sun", jd)
```

### Body ID mapping

| Swiss Eph constant | SE value | XALEN `Body` enum | XALEN int ID |
|---|---|---|---|
| `SE_SUN` | 0 | `Body::Sun` | 0 |
| `SE_MOON` | 1 | `Body::Moon` | 1 |
| `SE_MERCURY` | 2 | `Body::Mercury` | 2 |
| `SE_VENUS` | 3 | `Body::Venus` | 3 |
| `SE_MARS` | 4 | `Body::Mars` | 4 |
| `SE_JUPITER` | 5 | `Body::Jupiter` | 5 |
| `SE_SATURN` | 6 | `Body::Saturn` | 6 |
| `SE_URANUS` | 7 | `Body::Uranus` | 7 |
| `SE_NEPTUNE` | 8 | `Body::Neptune` | 8 |
| `SE_PLUTO` | 9 | `Body::Pluto` | 11 |
| `SE_MEAN_NODE` | 10 | `Body::MeanNode` | 9 |
| `SE_TRUE_NODE` | 11 | `Body::TrueNode` | 10 |
| `SE_CHIRON` | 15 | `Body::Chiron` | 12 |
| `SE_MEAN_APOG` | 12 | `Body::MeanApogee` | -- |
| -- (Ketu) | -- | Rahu + 180 | 13 |

> **Note:** SE uses Pluto=9, MeanNode=10; XALEN uses MeanNode=9, Pluto=11.
> The compat layer in `xalen_ephem::compat` handles this mapping automatically.

---

## 2. All planets at once

### Swiss Ephemeris (C)

```c
for (int p = SE_SUN; p <= SE_PLUTO; p++) {
    swe_calc_ut(jd, p, iflag, xx, serr);
    printf("%d: %f\n", p, xx[0]);
}
```

### XALEN (Rust)

```rust
let positions = almanac.all_positions(Body::ALL_PLANETS, JdUT1(jd));
for (body, result) in &positions {
    if let Ok(pos) = result {
        println!("{}: {:.4}", body, pos.longitude.to_degrees());
    }
}
```

### XALEN (Python)

```python
positions = xalen.all_planets(jd)
# Returns: {"Sun": 280.5, "Moon": 120.3, "Mercury": 315.1, ...}
```

---

## 3. House cusps

### Swiss Ephemeris (C)

```c
double cusps[13], ascmc[10];
swe_houses(jd, lat, lon, 'P', cusps, ascmc);  // 'P' = Placidus

double asc = ascmc[0];
double mc  = ascmc[1];
for (int i = 1; i <= 12; i++)
    printf("House %d: %f\n", i, cusps[i]);
```

### XALEN (Rust)

```rust
use xalen_houses::{compute_houses, GeoLocation, HouseSystem};

let loc = GeoLocation::new(lat, lon);
let epsilon = 23.4393_f64.to_radians(); // mean obliquity
let h = compute_houses(jd, &loc, epsilon, HouseSystem::Placidus);

let asc = h.ascendant.to_degrees();
let mc  = h.mc.to_degrees();
for (i, cusp) in h.cusps.iter().enumerate() {
    println!("House {}: {:.4}", i + 1, cusp.to_degrees());
}
```

### XALEN (Rust -- compat layer)

```rust
use xalen_ephem::compat::*;

let result = swe_houses(jd, lat, lon, 'P')?;
let asc = result.ascendant;
let mc  = result.mc;
for (i, cusp) in result.cusps.iter().enumerate() {
    println!("House {}: {:.4}", i + 1, cusp);
}
```

### XALEN (Node.js)

```js
// By name
const cusps = xalen.houses(jd, lat, lon, "placidus");

// By ID (2=Placidus)
const cusps2 = xalen.housesById(jd, lat, lon, 2);
```

### XALEN (Python)

```python
result = xalen.houses(jd, lat, lon, system=2)  # 2=Placidus
# Returns: {"cusps": [float; 12], "ascendant": 123.4, "mc": 45.6, ...}
```

### House system mapping

| SE char code | SE name | XALEN `HouseSystem` | XALEN int ID |
|---|---|---|---|
| `'P'` | Placidus | `Placidus` | 2 |
| `'K'` | Koch | `Koch` | 3 |
| `'O'` | Porphyry | `Porphyry` | 4 |
| `'R'` | Regiomontanus | `Regiomontanus` | 5 |
| `'C'` | Campanus | `Campanus` | 6 |
| `'A'` or `'E'` | Equal | `Equal` | 1 |
| `'W'` | Whole Sign | `WholeSign` | 0 |
| `'M'` | Morinus | `Morinus` | 7 |
| `'B'` | Alcabitius | `Alcabitius` | 8 |
| `'T'` | Topocentric | `Topocentric` | 9 |
| `'X'` | Meridian | `Meridian` | 12 |
| `'V'` | Vehlow | `Vehlow` | 11 |
| `'U'` | Krusinski-Pisa | `KrusinskiPisa` | 13 |
| `'S'` | Sripati | `Sripati` | 10 |
| `'G'` | Gauquelin | `Gauquelin` | -- |

---

## 4. Ayanamsa (sidereal mode)

### Swiss Ephemeris (C)

```c
swe_set_sid_mode(SE_SIDM_LAHIRI, 0, 0);
double ayanamsa = swe_get_ayanamsa_ut(jd);

// Sidereal planet position:
int iflag_sid = SEFLG_SWIEPH | SEFLG_SIDEREAL;
swe_calc_ut(jd, SE_SUN, iflag_sid, xx, serr);
double sid_lon = xx[0];
```

### XALEN (Rust)

```rust
use xalen_ayanamsa::Ayanamsa;

// Get ayanamsa value
let aya = Ayanamsa::Lahiri.compute_deg(jd_tt);

// Sidereal longitude in one call
let sid_lon = almanac.sidereal_longitude_deg(Body::Sun, JdUT1(jd), aya)?;

// Or convert existing tropical longitude
use xalen_ayanamsa::tropical_to_sidereal;
let sid_rad = tropical_to_sidereal(tropical_rad, &Ayanamsa::Lahiri, jd_tt);
```

### XALEN (Rust -- compat layer)

```rust
use xalen_ephem::compat::*;

let aya = swe_get_ayanamsa_ut(jd);                   // Lahiri default
let aya_kp = swe_get_ayanamsa_ut_ex(jd, SE_SIDM_KRISHNAMURTI)?;
```

### XALEN (Node.js)

```js
// By name
const lon = xalen.siderealLongitude("Sun", jd, "lahiri");

// By ID
const lon2 = xalen.siderealLongitudeById(0, jd, 0);  // body=Sun, aya=Lahiri

// Ayanamsa value alone
const aya = xalen.ayanamsaById(jd, 0);  // 0=Lahiri
```

### XALEN (Python)

```python
# Sidereal longitude
lon = xalen.planet_longitude(jd, body=0, sidereal=True, ayanamsa=0)

# Ayanamsa value
aya = xalen.ayanamsa(jd, system=0)  # 0=Lahiri
```

### Ayanamsa system mapping

XALEN covers all 47 Swiss Ephemeris ayanamsa systems (SE IDs 0--46).
Use `Ayanamsa::from_swiss_ephem_id(id)` or `Ayanamsa::swiss_ephem_id()`
to convert.

| SE ID | SE constant | XALEN `Ayanamsa` |
|---|---|---|
| 0 | `SE_SIDM_FAGAN_BRADLEY` | `FaganBradley` |
| 1 | `SE_SIDM_LAHIRI` | `Lahiri` |
| 3 | `SE_SIDM_RAMAN` | `Raman` |
| 5 | `SE_SIDM_KRISHNAMURTI` | `KPKrishnamurti` |
| 27 | `SE_SIDM_TRUE_CITRA` | `TrueChitra` |
| 28 | `SE_SIDM_TRUE_REVATI` | `TrueRevati` |

Full table: all 47 SE IDs are mapped. Use `Ayanamsa::from_swiss_ephem_id(n)`.

---

## 5. Sidereal conversion (tropical to sidereal and back)

### Swiss Ephemeris (C)

```c
swe_set_sid_mode(SE_SIDM_LAHIRI, 0, 0);
double aya = swe_get_ayanamsa_ut(jd);
double sidereal = fmod(tropical - aya + 360.0, 360.0);
```

### XALEN (Rust)

```rust
use xalen_ayanamsa::{Ayanamsa, tropical_to_sidereal, sidereal_to_tropical};

let sid = tropical_to_sidereal(tropical_rad, &Ayanamsa::Lahiri, jd_tt);
let trop = sidereal_to_tropical(sidereal_rad, &Ayanamsa::Lahiri, jd_tt);
```

---

## 6. Fixed stars

### Swiss Ephemeris (C)

```c
double xx[6];
char serr[256], star_name[256];
strcpy(star_name, "Spica");
swe_fixstar_ut(star_name, jd, SEFLG_SWIEPH, xx, serr);
double star_lon = xx[0];
```

### XALEN (Rust)

```rust
use xalen_stars::{find_by_name, find_conjunctions_at_epoch};

// Look up a specific star
let spica = find_by_name("Spica").unwrap();
let lon = spica.longitude_at_jd(jd);

// Find all stars within 2 degrees of a planet
let hits = find_conjunctions_at_epoch(planet_lon_deg, 2.0, 2026.0);
for (star, distance) in &hits {
    println!("{}: {:.2} deg away", star.name, distance);
}
```

### XALEN (Node.js)

```js
const hits = xalen.fixedStarConjunctions(planetLon, 2.0, 2026.0);
// Returns: [{ name: "Spica", distance: 0.5, constellation: "Virgo", ... }]
```

### XALEN (Python)

```python
hits = xalen.fixed_star_conjunctions(planet_lon, orb=2.0, year=2026.0)
```

XALEN ships a 108-star catalog with proper motion. No external data files needed.

---

## 7. Eclipse search

### Swiss Ephemeris (C)

```c
double tret[10], attr[20];
char serr[256];

// Next solar eclipse
swe_sol_eclipse_when_glob(jd_start, SEFLG_SWIEPH, SE_ECL_TOTAL, tret, 0, serr);
double jd_max = tret[0];
```

### XALEN (Rust)

```rust
use xalen_ephem::{Almanac, find_solar_eclipses, find_lunar_eclipses};

let almanac = Almanac::default_vedic();
let solar = find_solar_eclipses(&almanac, jd_start, jd_end);
for eclipse in &solar {
    // `coverage_proxy` is a diameter-ratio / overlap PROXY, not the
    // astronomical magnitude; `gamma` is the authoritative Besselian quantity.
    println!("JD {:.2}: {:?}, gamma {:.3}, coverage~{:.3}",
        eclipse.jd_maximum, eclipse.eclipse_type, eclipse.gamma, eclipse.coverage_proxy);
}

let lunar = find_lunar_eclipses(&almanac, jd_start, jd_end);
```

XALEN returns typed results (`SolarEclipse`, `LunarEclipse`) with classification
(`Total`, `Annular`, `Partial`, `Penumbral`), the Besselian `gamma` (solar), and
honestly-named proxies (`coverage_proxy` for solar, `shadow_depth_proxy` for
lunar) rather than a true astronomical magnitude. Global eclipse type verified
against the NASA Eclipse Catalog for 2024--2025. For a real per-observer
magnitude / obscuration / contact times, use the local-circumstances layer.

---

## 8. Dasha periods (not in Swiss Ephemeris)

Swiss Ephemeris has no dasha support. XALEN includes Vimshottari, Yogini,
Ashtottari, Chara, and Nadi dasha systems out of the box.

### XALEN (Rust)

```rust
use xalen_vedic::dasha::{vimshottari_dasha, DashaEntry};
use xalen_vedic::nakshatra::Nakshatra;

let moon_nak = Nakshatra::from_longitude_deg(moon_sidereal_deg);
let birth_jd = 2451545.0; // J2000

let dashas = vimshottari_dasha(moon_nak, birth_jd, moon_sidereal_deg);
for d in &dashas {
    println!("{}: {:.0} days", d.lord, d.duration_days);
}
```

### XALEN (Python)

```python
chart = xalen.full_chart(jd, lat, lon)
# Dasha: use the Moon's nakshatra from chart["planets"]["Moon"]["nakshatra"]
```

---

## 9. Julian Day conversion

### Swiss Ephemeris (C)

```c
double jd = swe_julday(1990, 6, 15, 10.5, SE_GREG_CAL);
```

### XALEN (Rust)

```rust
use xalen_time::{calendar_to_jd, CalendarSystem};

let jd = calendar_to_jd(1990, 6, 15, 10.5, CalendarSystem::ProlepticGregorian);
```

### XALEN (Node.js / Python)

```js
// Node.js
const dt = xalen.deltaT(jd);
```

```python
# Python
jd = xalen.julian_day(1990, 6, 15, hour=10.5)
dt = xalen.delta_t(jd)
```

---

## 10. Delta-T

### Swiss Ephemeris (C)

```c
double dt = swe_deltat_ex(jd, SEFLG_SWIEPH, serr);
```

### XALEN (Rust)

```rust
use xalen_time::{delta_t, DeltaTModel};

let dt = delta_t(jd, &DeltaTModel::StephensonMorrisonHohenkerk2016);
```

---

## 11. Panchang (Vedic calendar -- not in Swiss Ephemeris)

```rust
use xalen_vedic::panchang::compute_panchang;
let p = compute_panchang(sun_sidereal_deg, moon_sidereal_deg, jd);
// p.tithi, p.nakshatra, p.yoga, p.karana, p.vara
```

```python
p = xalen.panchang(jd, ayanamsa=0)
# {"tithi": {"number": 5, "name": "Panchami", "paksha": "Shukla"}, ...}
```

---

## 12. Initialization and cleanup

### Swiss Ephemeris (C)

```c
swe_set_ephe_path("/usr/share/sweph/ephe");
// ... use library ...
swe_close();
```

### XALEN

```rust
// No initialization needed. No data files. No cleanup.
let almanac = Almanac::default_vedic();
// Thread-safe, can be shared via Arc<Almanac>.
```

XALEN embeds all astronomical data at compile time. There is no runtime
file I/O, no `ephe_path`, and no `close()` call.

---

## Key differences from Swiss Ephemeris

| Feature | Swiss Ephemeris | XALEN |
|---|---|---|
| Language | C (with bindings) | Rust (with Node/Python/WASM) |
| License | AGPL-3.0 or paid commercial | Apache-2.0 (permissive) |
| Data files | `.se1` files required at runtime | Embedded at compile time |
| Thread safety | Not thread-safe (`swe_set_*` is global) | `Send + Sync`, no global state |
| Initialization | `swe_set_ephe_path()` required | None |
| Cleanup | `swe_close()` required | None |
| Ayanamsa | 47 systems (global `swe_set_sid_mode`) | 47 systems (per-call parameter) |
| Houses | 23 systems | 23 systems |
| Fixed stars | File-based catalog | 108-star embedded catalog |
| Eclipses | Yes (comprehensive) | Yes (latitude-based, NASA-verified) |
| Dasha | No | Vimshottari, Yogini, Ashtottari, Chara, Nadi |
| Panchang | No | Tithi, Nakshatra, Yoga, Karana, Vara |
| Vedic astrology | No | Full support (nakshatras, rashis, yogas, doshas) |
| WASM | No | Yes (`xalen-wasm` crate) |
| Accuracy | Sub-arcsecond (DE431) | ~1 arcsec (VSOP87), sub-arcsec with DE440 |

---

## Gotchas when migrating

1. **Planet IDs differ.** SE uses Pluto=9, MeanNode=10. XALEN uses
   MeanNode=9, Pluto=11. The compat layer handles this; raw integer-based
   APIs use the XALEN numbering.

2. **No global state.** SE uses `swe_set_sid_mode()` globally. In XALEN you
   pass the ayanamsa as a parameter to each call. This means no accidental
   cross-contamination between threads or callers.

3. **Angles are in radians internally.** The Rust API returns `EclipticPosition`
   with radians. Use `.to_degrees()` or the `*_deg()` convenience methods.
   The Node.js and Python bindings always return degrees.

4. **Ketu is computed, not a body.** XALEN treats Ketu as Rahu + 180 degrees.
   Use body ID 13 in the bindings, or compute it yourself:
   `Body::ketu_longitude(rahu_lon_rad)`.

5. **Speed is computed; retrograde is the sign of the longitude speed.**
   SE returns velocity in `xx[3..5]`. XALEN computes daily motion directly via
   `Almanac::geocentric_speed` (in the `swe_calc_ut` compat shim, pass
   `SEFLG_SPEED` to fill `xx[3..6]`). A body is retrograde when its geocentric
   ecliptic-longitude speed is negative:

   ```rust
   use xalen_ephemeris::ephem::{Almanac, Body};
   use xalen_ephemeris::time::JdUT1;

   let almanac = Almanac::default_vedic();
   let speed = almanac.geocentric_speed(Body::Mercury, JdUT1(jd)).unwrap();
   let is_retrograde = speed.longitude < 0.0; // negative = retrograde
   ```
