# xalen-stars
> Astrologically significant fixed stars with rigorously precessed, proper-motion-corrected ecliptic positions, backed by the Hipparcos catalogue.

Part of the [XALEN Ephemeris](https://github.com/vedika-io/xalen-ephemeris) suite — pure-Rust, thread-safe, Apache-2.0.

## Catalog layers
- **Curated `CATALOG`** — 108 traditional fixed stars, each with a `'static` name, Ptolemaic planetary nature, and constellation.
- **`GENERATED_CATALOG`** — every Hipparcos Main Catalogue star with Vmag ≤ 6.5 (8,870 stars), each derived from a real `hip_main.dat` record (no value invented or interpolated). 106 of the 108 traditional names are joined to their exact Hipparcos row via the IAU Catalog of Star Names (IAU-CSN); the only two without a single HIP star are the open clusters Pleiades and Praesepe.
- The public `find_by_name` / `find_conjunctions` surface reads a **reconciled catalog**: for every named star it returns the validated Hipparcos position (sub-arcsecond agreement with the source), and falls back to the curated coordinates only for those two clusters.

## Features
- `FixedStar::longitude_at_epoch` / `latitude_at_epoch` / `longitude_at_jd` — positions at any decimal year or Julian Date. Precession uses the **rigorous IAU 2006/P03 rotation** (`precessed_ecliptic_of_date`), which couples ecliptic longitude **and** latitude — not a linear-longitude model with frozen latitude. The 50.28796″/yr (IAU 2006 J2000) figure remains the leading-order longitude term but is no longer the propagation method. Validated against `pyswisseph` `swe.fixstar2` at 1000 CE and 3000 CE (worst total separation ≈ 4.4″).
- `find_by_name` (case-insensitive lookup) and `find_conjunctions` / `find_conjunctions_at_epoch` — stars within a given orb of a planetary longitude, plus `find_conjunctions_expanded` over all 8,870 generated stars.
- `find_generated_by_hip` / `find_generated_by_name` — direct access to the Hipparcos-derived catalog.
- `nakshatra_yogatara` — maps each of the 27 nakshatra indices (0 = Ashwini … 26 = Revati) to its primary reference star.
- `catalog` module — load and merge external catalogs from CSV at runtime via `load_catalog_from_csv` / `load_catalog_from_str`, `merge_catalogs`, `find_in_catalog`, and `find_conjunctions_in_catalog`.
- `serde`-serializable `FixedStar`; no `unsafe`, no global mutable state.

## Usage
```rust
use xalen_stars::{find_by_name, find_conjunctions, nakshatra_yogatara};

// Look up a star and compute its precessed longitude in the year 2100.
let spica = find_by_name("Spica").unwrap();
let lon_2100 = spica.longitude_at_epoch(2100.0);
println!("Spica longitude in 2100: {lon_2100:.3}°");

// The reference star (yogatara) for Chitra nakshatra (index 13) is Spica.
let chitra = nakshatra_yogatara(13).unwrap();
assert_eq!(chitra.name, "Spica");

// Find catalog stars within a 2° orb of ecliptic longitude 70°.
for (star, dist) in find_conjunctions(70.0, 2.0) {
    println!("{} at {:.2}° away", star.name, dist);
}
```

## Accuracy & sources
J2000.0 positions, magnitudes and proper motions are derived from the Hipparcos Main Catalogue (CDS I/239, ESA 1997): each `hip_main.dat` record is propagated J1991.25 → J2000.0 by its measured proper motion, then rotated to the J2000 ecliptic at the IAU 2006 mean obliquity. Traditional names are joined to HIP numbers through the IAU Catalog of Star Names (IAU-CSN). Epoch propagation to other years uses the rigorous IAU 2006/P03 precession rotation plus per-star proper motion (CSV imports apply an approximate equatorial-to-ecliptic PM conversion). The generated catalog is rebuilt by `tools/gen-star-catalog/gen_catalog.py`. See [ACCURACY.md](../../docs/ACCURACY.md) and [CREDITS.md](../../CREDITS.md).

## License
Licensed under Apache-2.0. See [LICENSE](../../LICENSE).
