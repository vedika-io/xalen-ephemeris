# xalen-stars
> Fixed stars with rigorously precessed, proper-motion-corrected ecliptic positions: the 9,096 stars of the Yale Bright Star Catalogue plus a 108-star curated core with Ptolemaic natures.

Part of the [XALEN Ephemeris](https://github.com/vedika-io/xalen-ephemeris) suite — pure-Rust, thread-safe, Apache-2.0.

## Catalog layers
- **Curated `CATALOG`** — 108 traditional fixed stars, each with a `'static` name, Ptolemaic planetary nature, and constellation.
- **`BRIGHT_CATALOG`** (feature `bright-catalog`, default-on) — all 9,096 stars of the Yale Bright Star Catalogue, 5th rev. ed. (Hoffleit & Warren 1991, CDS V/50), as `BrightStar { hr, hd, designation, name, constellation, longitude_j2000, latitude_j2000, magnitude, nature, pm_lon_mas_per_year, pm_lat_mas_per_year }`, sorted by HR. 338 stars that are also in the IAU Catalog of Star Names (IAU-CSN) carry the IAU proper name, constellation and the IAU 6-decimal J2000 position; every other star uses the Bright Star Catalogue position (RA to 0.1 s, Dec to 1″). Proper motion and magnitude always come from the Bright Star Catalogue. 106 of the 108 curated names are joined to their star (through the IAU-CSN, or the Bayer designation for Al Jabhah); the two without a single star are the open clusters Pleiades and Praesepe. `constellation` is filled for stars that have an IAU name or a Bayer/Flamsteed designation (3,154 stars) and is `""` for the rest.
- The public `find_by_name` / `find_conjunctions` surface reads a **reconciled catalog**: for every curated name that joins to a bright star it returns the catalogue position, and it falls back to the curated coordinates only for the two clusters. Without the `bright-catalog` feature (a size switch with no licence meaning) only the 108 curated stars are available and `find_by_name` serves them verbatim.

## Features
- `FixedStar::longitude_at_epoch` / `latitude_at_epoch` / `longitude_at_jd` and the same methods on `BrightStar` — positions at any decimal year or Julian Date. Precession uses the **rigorous IAU 2006/P03 rotation** (`precessed_ecliptic_of_date`), which couples ecliptic longitude **and** latitude. Checked against `pyswisseph` `swe.fixstar2` at 1000 CE and 3000 CE (worst total separation 0.34′ with the Bright Star Catalogue proper motions).
- `find_bright_by_hr` / `find_bright_by_hd` / `find_bright_by_name` (case-insensitive; matches the IAU or curated name, an IAU spelling that differs from the curated one, or the designation) — direct access to the Bright Star Catalogue list.
- `find_by_name` (case-insensitive lookup) and `find_conjunctions` / `find_conjunctions_at_epoch` — stars within a given orb of a planetary longitude, plus `find_conjunctions_expanded` over all 9,096 bright stars.
- `nakshatra_yogatara` — maps each of the 27 nakshatra indices (0 = Ashwini … 26 = Revati) to its primary reference star.
- `catalog` module — load and merge external catalogs from CSV at runtime via `load_catalog_from_csv` / `load_catalog_from_str`, `merge_catalogs`, `find_in_catalog`, and `find_conjunctions_in_catalog`.
- `serde`-serializable `FixedStar` and `BrightStar`; no `unsafe`, no global mutable state.

## Usage
```rust
use xalen_stars::{find_bright_by_hr, find_by_name, find_conjunctions, nakshatra_yogatara};

// Look up a star and compute its precessed longitude in the year 2100.
let spica = find_by_name("Spica").unwrap();
let lon_2100 = spica.longitude_at_epoch(2100.0);
println!("Spica longitude in 2100: {lon_2100:.3}°");

// The reference star (yogatara) for Chitra nakshatra (index 13) is Spica.
let chitra = nakshatra_yogatara(13).unwrap();
assert_eq!(chitra.name, "Spica");

// Any Bright Star Catalogue star by its HR number (HR 2491 = Sirius).
let sirius = find_bright_by_hr(2491).unwrap();
assert_eq!(sirius.name, Some("Sirius"));

// Find catalog stars within a 2° orb of ecliptic longitude 70°.
for (star, dist) in find_conjunctions(70.0, 2.0) {
    println!("{} at {:.2}° away", star.name, dist);
}
```

## Accuracy & sources
Positions are equinox J2000, epoch 2000.0, rotated to the J2000 ecliptic at the IAU 2006 mean obliquity. Measured against live Swiss Ephemeris `swe_fixstar2` J2000 places recorded in `tests/common/mod.rs`: the 43 IAU-named oracle stars are within 0.245″ (median 0.026″); Al Jabhah, which has no IAU name, is 0.79″ away (Bright Star Catalogue rounding). Stars without an IAU name carry the Bright Star Catalogue rounding: against the Hipparcos-derived positions the median is 0.64″, 95% are within 2.3″, and the tail (close binaries, high proper motion) reaches tens of arcseconds. Proper motions are the FK5 values of the Bright Star Catalogue, so positions propagated by ±1000 years differ from Swiss by up to 21″ (49 live Swiss places, seven stars, 1000 to 3000: worst 20.6″, median 0.72″). The list is rebuilt by `tools/gen-star-catalog/gen_bright_catalog.py`. See [ACCURACY.md](../../docs/ACCURACY.md) and [CREDITS.md](../../CREDITS.md).

## License

The crate **code** is licensed under Apache-2.0. See [LICENSE](../../LICENSE). The star data is the Yale Bright Star Catalogue (Hoffleit & Warren 1991) and the IAU Catalog of Star Names (Creative Commons Attribution); see [NOTICE](../../NOTICE).

The Hipparcos-derived catalogue, which is **CC BY-NC 3.0 IGO** (non-commercial), is not part of this crate. It lives in the optional, unpublished [`xalen-stars-hip-data`](../xalen-stars-hip-data) crate, which depends on `xalen-stars`; no published crate depends on it.
