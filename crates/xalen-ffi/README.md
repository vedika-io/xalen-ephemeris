# xalen-ffi
> C FFI export layer for XALEN Ephemeris — call planetary positions, sidereal longitudes, ayanamsa, and house cusps from any language with a C ABI.

Part of the [XALEN Ephemeris](https://github.com/vedika-io/xalen-ephemeris) suite — pure-Rust, thread-safe, Apache-2.0.

Built as both `cdylib` and `staticlib`, this crate exposes a small, stable `extern "C"` surface over the rest of the suite (`xalen-time`, `xalen-coords`, `xalen-ephem`, `xalen-houses`, `xalen-ayanamsa`, `xalen-vedic`). Link the resulting shared/static library from C, C++, Python (ctypes/cffi), Node (N-API/ffi), Go (cgo), or any FFI host.

## Features
- **`xalen_init() -> int`** — thread-safe, idempotent initialization of the shared almanac (lazy `OnceLock`); returns `0`.
- **`xalen_planet_position(jd_ut1, body_id, *out XalenPosition) -> int`** — geocentric tropical ecliptic longitude, latitude (deg), and distance (AU). Output is zero-initialized before use; null `out` returns `-1`.
- **`xalen_planet_position_full(jd_ut1, body_id, *out XalenPositionFull) -> int`** — the FULL six-component state Swiss Ephemeris returns from `swe_calc_ut(..., SEFLG_SPEED)`: longitude, latitude (deg), distance (AU), plus daily-motion speeds (`lon_speed_deg`, `lat_speed_deg` in deg/day, `dist_speed_au` in AU/day) and an `is_retrograde` flag (`1`/`0`, from the tropical longitude rate). Additive — the `xalen_planet_position` / `XalenPosition` ABI is unchanged. Ketu (id 13) inherits Rahu's speed and retrograde state.
- **`xalen_sidereal_longitude(jd_ut1, body_id, ayanamsa_id) -> double`** — sidereal longitude (deg) for the chosen ayanamsa.
- **`xalen_ayanamsa(jd_ut1, ayanamsa_id) -> double`** — ayanamsa offset in degrees at the given epoch.
- **`xalen_houses(jd_ut1, lat, lon, system_id, *out XalenHouses) -> int`** — 12 house cusps, Ascendant, and MC (deg).
- **`xalen_version() -> const char*`** — static version string.
- **Canonical integer IDs** (documented inline in `lib.rs`): bodies `0=Sun … 12=Chiron`, `9=Rahu/MeanNode`, `10=TrueNode`, `13=Ketu` (Rahu + 180°); 17 ayanamsa systems (`0=Lahiri … 16=LahiriVP285`); 14 house systems (`0=WholeSign … 13=Krusinski`).
- **Negative status codes** on error: `-1` null pointer, `-2` invalid body/system/ayanamsa ID **or non-finite/out-of-range numeric input** (NaN/Inf `jd`/`lat`/`lon`, latitude outside ±90°), `-3` computation failure.

## Usage
```rust
use xalen_ffi::{xalen_init, xalen_planet_position, xalen_houses, XalenPosition, XalenHouses};

xalen_init();

// Geocentric tropical Sun at J2000.0 (JD UT1 = 2451545.0).
let mut pos = XalenPosition { longitude_deg: 0.0, latitude_deg: 0.0, distance_au: 0.0, status: -1 };
let ret = unsafe { xalen_planet_position(2_451_545.0, 0, &mut pos) };
assert_eq!(ret, 0);
println!("Sun longitude: {:.4} deg", pos.longitude_deg);

// Whole-Sign house cusps for Pune (18.52 N, 73.85 E).
let mut houses = XalenHouses { cusps: [0.0; 12], ascendant_deg: 0.0, mc_deg: 0.0, status: -1 };
let ret = unsafe { xalen_houses(2_451_545.0, 18.52, 73.85, 0, &mut houses) };
assert_eq!(ret, 0);
println!("Ascendant: {:.4} deg", houses.ascendant_deg);
```

From C, link the library and declare the matching prototypes and `XalenPosition` / `XalenHouses` `#[repr(C)]` structs.

## Accuracy & sources
This layer performs no astronomy of its own — it forwards to the suite's VSOP87/ELP-based ephemeris and IAU house algorithms, returning their values unchanged. See [ACCURACY.md](../../docs/ACCURACY.md) and [CREDITS.md](../../CREDITS.md).

## License
Licensed under Apache-2.0. See [LICENSE](../../LICENSE).
