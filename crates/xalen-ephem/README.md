# xalen-ephem
> Geocentric and heliocentric planetary positions — VSOP87 analytical theory with optional JPL DE440 support.

Part of the [XALEN Ephemeris](https://github.com/vedika-io/xalen-ephemeris) suite — pure-Rust, thread-safe, Apache-2.0.

## Features
- `Almanac` facade: chains ephemeris providers with automatic fallback, converting UT1 to TT internally via a configurable delta-T model.
- Geocentric and heliocentric ecliptic positions for the Sun, Moon, and Mercury through Pluto via the `EphemerisProvider` trait (`Vsop87Provider` ships by default).
- Optional JPL DE440 binary-kernel reader (`De440Provider`, `De440Reader`, Chebyshev evaluation) for the highest-accuracy tier.
- Optional `kernel-autodownload` feature (off by default): `De440Provider::from_auto_cache()` fetches and caches the public NASA NAIF `de440s.bsp` kernel automatically on first use, giving a sub-arcsecond apparent Moon (and all kernel bodies) with no manual file handling. Without the feature the crate stays offline; the analytical Moon is RMS ~2.9″ / max ~12″ vs `pyswisseph` over AD 1600–2100.
- Tropical and sidereal longitude helpers — `sidereal_longitude_deg` subtracts any ayanamsa for Vedic work.
- Lunar nodes: mean and true (osculating) Rahu, with Ketu derived as the opposite point (`true_node` module).
- Solar and lunar eclipse detection and classification (`eclipse` module).
- Numerical event search: sign ingresses, longitude crossings, and planetary stations (`event_search` module).
- Black Moon Lilith / mean lunar apogee and Priapus (`lilith` module), plus Chiron and the four major asteroids.
- Swiss Ephemeris API compatibility layer (`compat` module) for drop-in migration.
- `Almanac` is `Send + Sync`, so a single instance can be shared across threads behind an `Arc`.

## Usage
```rust
use xalen_ephem::{Almanac, Body};
use xalen_time::{calendar_to_jd, CalendarSystem};
use xalen_ayanamsa::Ayanamsa;
use xalen_vedic::nakshatra::Nakshatra;

// 1990-03-15 12:00 local (UTC+5:30) → Julian Day
let jd = calendar_to_jd(1990, 3, 15, 12.0 - 5.5, CalendarSystem::default());

let almanac = Almanac::default_vedic();
let pos = almanac.geocentric_ecliptic(Body::Moon, jd).unwrap();

// Tropical → sidereal (Lahiri ayanamsa) → nakshatra
let sidereal = (pos.longitude.to_degrees()
    - Ayanamsa::Lahiri.compute_deg(jd.as_f64()))
    .rem_euclid(360.0);
let nakshatra = Nakshatra::from_longitude_deg(sidereal);

assert_eq!(format!("{nakshatra}"), "Swati");
```
This example is guarded by `tests/readme_example.rs`, so it stays in sync with the code.

## Accuracy & sources
Planetary positions use VSOP87A with IAU 2000B nutation and measure sub-arcsecond to roughly one arcsecond against JPL DE440 over 4000 BCE – 8000 CE; see [ACCURACY.md](../../docs/ACCURACY.md) for the full cross-validation report and [CREDITS.md](../../CREDITS.md) for upstream data sources.

## License
Licensed under Apache-2.0. See [LICENSE](../../LICENSE).
