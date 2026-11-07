# Features in detail

The complete feature list, the accuracy table with its notes, and further code
examples. The [README](../README.md) gives the overview; the measured figures and
how to reproduce them are in [ACCURACY.md](ACCURACY.md).

## What you get

- **50 ayanamsa systems** -- Lahiri, KP Krishnamurti, Raman, Fagan-Bradley, True Chitrapaksha, True Revati, Surya Siddhanta, all Galactic Center variants, Babylonian (Kugler), all Swiss Ephemeris IDs (0-46), plus a fully custom variant with user-defined epoch, value, and precession rate

- **23 house systems** -- Placidus, Koch, Campanus, Regiomontanus, Whole Sign, Equal, Porphyry, Morinus, Alcabitius, Topocentric (Polich-Page), Meridian, Vehlow, Sripati, Krusinski-Pisa, Sunshine (Makransky & Treindl), Pullen Sinusoidal (Delta & Ratio), Carter Poli-Equatorial, APC, Zariel, Alcabitius Classic -- with automatic Porphyry fallback at polar latitudes. (Gauquelin sectors is an experimental placeholder that currently returns Placidus cusps, not a true Gauquelin division.)

- **12+ astrology traditions** -- Vedic/Jyotish (dasha, shadbala, KP, Jaimini, Tajaka, panchang with tithi/nakshatra/yoga/karana transition times, compatibility, 16 divisional charts, yoga, dosha), Western (aspects, dignities, 97 Arabic Lots, Hellenistic, Uranian, Cosmobiology, progressions, exact solar/lunar returns by iterative refinement, declination aspects + antiscia, harmonics, horary), Chinese (BaZi, Zi Wei Dou Shu, Feng Shui Flying Stars, Qi Men Dun Jia [experimental]), Lal Kitab, I Ching (64 hexagrams + all 384 line texts, verbatim public-domain Legge SBE XVI), Numerology, Korean Saju, Japanese Nine Star Ki, Burmese Mahabote (day-sign profile only), Mayan, Aztec, Tibetan, Persian/Zoroastrian, Egyptian, Celtic

- **Relationship charts and astrocartography** -- `xalen-relationship` gives synastry (aspect grid and house overlays), composite charts (Hand) and Davison charts, plus a relocated-chart helper; `xalen-maps` draws astrocartography lines (MC, IC, Ascendant, Descendant and zenith points for each body, Lewis) with circumpolar breaks and antimeridian splitting, and parans (the crossings of two bodies' lines, Brady). Both are geocentric

- **9,096 built-in fixed stars** (`xalen-stars`: every star of the Yale Bright Star Catalogue, 5th rev. ed., with IAU-CSN names and positions where a star has an IAU name) plus a 108-star curated core with Ptolemaic natures, and **503 named fixed stars** in `xalen-western` (all mag < 3.0, Behenian, Royal, Nakshatra yogatara, IAU-named to mag ~5). Proper motion and IAU 2006/P03 precession to any epoch, zero data files; the data carries no non-commercial restriction. External catalogs can be loaded from CSV at runtime

- **Solar and lunar eclipse engine** -- rigorous Besselian elements (Explanatory Supplement §11; Meeus Ch. 54) for global type / γ / greatest-eclipse, plus a per-observer local-circumstances layer (magnitude, obscuration, C1–C4 contact times); lunar eclipses from Earth's shadow cones (Danjon) with umbral / penumbral magnitudes and P1–P4 contacts, matching Swiss Ephemeris for every eclipse of 1900–2100. Global type validated against NASA for 2017-08-21 and 2024-04-08

- **Black Moon Lilith** -- both **mean** lunar apogee and **True (osculating)** Lilith (Swiss `SE_OSCU_APOG`), Chiron, True Node, Mean Node

- **Velocity / daily motion** for every body, with `is_retrograde()`,
  **topocentric positions** (diurnal parallax) for an observer's exact location,
  and **equatorial RA/Dec, heliocentric, and rectangular XYZ** output alongside
  the geocentric ecliptic place

- **Time systems** -- UT1 / TT / TDB Julian Day types, UTC↔TAI with the full
  leap-second table, and a genuine Stephenson–Morrison–Hohenkerk 2016 ΔT spline
  with a per-epoch σ envelope built from NASA and USNO public-domain sources

- **15 asteroids** -- the big 4 (Ceres, Pallas, Juno, Vesta), Hygeia, Astraea, Psyche, Eros, Lilith (1181), centaurs (Pholus, Nessus), and TNOs (Eris, Sedna, Makemake, Haumea), plus an external element loader for any asteroid with known orbital elements

- **SVG chart rendering** -- North Indian diamond, South Indian box, and Western wheel charts, zero external dependencies

- **City geocoding** for 130+ cities with latitude, longitude, and timezone

- **NAIF DE440 SPK reader** -- reads real JPL `.bsp` binary files for extended validity range and JPL-grade accuracy; the analytical engine (VSOP87A + ELP2000-82 + IAU 2000B nutation + full IAU 2006/P03 rotation) works standalone with zero data files: **Sun + Mercury–Saturn sub-arcsecond** vs JPL DE440 (Sun 0.21", ≤0.76"), Uranus/Neptune ~1.8–2.5", **Moon RMS ~2.8" / max ~12"** vs pyswisseph (AD 1600–2100; an earlier build mis-applied the planet annual-aberration term to the geocentric Moon for a ~11–31" residual — that bug is fixed, see [ACCURACY.md](ACCURACY.md)), and Pluto arcminute-class in-window (published Goffin-fit bound ~1′; the committed golden test asserts a 600″/10′ regression ceiling vs DE440 — no test asserts a tighter figure). **Load the DE440 kernel for JPL-grade sub-arcsecond on the Sun, planets and Moon, plus full-range Pluto.** Both engines apply the same apparent-place chain (full-rotation precession + nutation; planets/Sun get 2-pass annual aberration, the geocentric Moon gets geocentric light-time but NOT annual aberration). Numbers reproducible from this repo via `cargo test -p xalen-ephem --test accuracy_vs_de440` (with a `de440s.bsp` kernel present)

- **19 languages** -- English, Hindi, Sanskrit, Tamil, Telugu, Kannada, Malayalam, Bengali, Gujarati, Marathi, Punjabi, Odia, Spanish, Portuguese, French, German, Japanese, Thai, Indonesian -- for planet names, signs, nakshatras, weekdays

- **Language bindings** -- Node.js (napi-rs), Python (PyO3), WASM (wasm-bindgen), and C FFI (`extern "C"`)

## Accuracy by theory

| Theory | Bodies | Accuracy | Data Files Needed? |
|--------|--------|----------|--------------------|
| VSOP87A + IAU 2000B nutation | Sun | **0.21"** (vs JPL DE440) | No (analytical) |
| VSOP87A + IAU 2000B nutation | Mercury -- Saturn | **0.21--0.76"** (vs JPL DE440, 20k charts) | No (analytical) |
| VSOP87A + IAU 2000B nutation | Uranus / Neptune | 1.78" / 2.53" (vs JPL DE440) | No (analytical) |
| ELP2000-82 (Chapront-Touzé & Chapront; abridged 60+60 terms per Meeus Ch.47) + Δψ + geocentric light-time | Moon | **RMS ~2.8" / max ~12"** (vs pyswisseph 2.10.03, AD 1600--2100; truncation-limited, not aberration) | No (analytical) |
| Goffin (1989) DE200 fit (43 terms per Meeus Ch.37) | Pluto | arcminute-class in-window; golden test asserts ≤600″/10′ vs DE440 (no tighter figure asserted) | No (valid 1885--2099; DE440 for full range) |
| JPL DE440 | Sun + planets / Pluto | sub-arcsecond apparent (raw geometry sub-mas; 2-pass body light-time) | Yes (binary `.bsp`) |
| JPL DE440 | Moon | sub-arcsecond apparent (raw geometry sub-mas; geocentric light-time, no annual aberration) | Yes (binary `.bsp`) |
| IAU 2006 | Precession | ~0.3 mas/century | No |
| IAU 2000B | Nutation | ~1 mas | No |

For most astrological applications, the analytical theories (VSOP87A, ELP2000-82)
provide more than enough precision -- they require zero external data files and
work entirely from compiled-in polynomial series; the analytical Moon now lands
within RMS ~2.8" of Swiss Ephemeris across four centuries. Load DE440 when you
need JPL-grade sub-arcsecond precision on the Moon and outer bodies, full-range
Pluto, or primary directions to the minute.

**Sub-arcsecond Moon with zero manual kernel handling.** Enable the optional
`kernel-autodownload` feature and call `De440Provider::from_auto_cache()`; the
public NASA NAIF `de440s.bsp` kernel (~32 MB) is fetched once into the OS cache
directory, verified, and reused thereafter. The feature is off by default so the
base crate stays offline and data-file-free:

```toml
[dependencies]
xalen-ephem = { version = "0.6", features = ["kernel-autodownload"] }
```

```rust
use xalen_ephem::De440Provider;
// First call fetches + caches de440s.bsp; later calls reuse it (no network).
let provider = De440Provider::from_auto_cache()?;
// Apparent Moon (and all kernel bodies) now sub-arcsecond.
```

Eclipse output now includes both a
global-circumstances classifier (type / Besselian γ / greatest eclipse; the
global figure is a diameter-ratio coverage proxy, not a true magnitude) and a
Besselian local-circumstances engine that does provide a real per-observer
magnitude, obscuration, and C1–C4 contact times.

## More code

### Vedic chart: Sun and Moon with nakshatra and rashi

With only `xalen-ephemeris` added, every sub-crate is reachable through the
umbrella re-exports (`xalen_ephemeris::ephem`, `::vedic`, `::time`, …):

```rust,no_run
use xalen_ephemeris::ayanamsa::Ayanamsa;
use xalen_ephemeris::coords::RAD_TO_DEG;
use xalen_ephemeris::ephem::{Almanac, Body};
use xalen_ephemeris::time::{calendar_to_jd, CalendarSystem, JulianDay};
use xalen_ephemeris::vedic::nakshatra::Nakshatra;
use xalen_ephemeris::vedic::rashi::Rashi;

fn main() {
    // 15 August 1947, 00:00 IST (UTC+5:30) -- Indian Independence
    let jd = calendar_to_jd(1947, 8, 15, 0.0 - 5.5, CalendarSystem::default());

    let almanac = Almanac::default_vedic();
    let aya_deg = Ayanamsa::Lahiri.compute_deg(jd.as_f64());

    for &body in &[Body::Sun, Body::Moon] {
        let pos = almanac.geocentric_ecliptic(body, jd).unwrap();
        let sid = (pos.longitude * RAD_TO_DEG - aya_deg).rem_euclid(360.0);

        let rashi = Rashi::from_longitude_deg(sid);
        let nak = Nakshatra::from_longitude_deg(sid);
        let pada = Nakshatra::pada(sid);

        println!("{body}: {sid:.2} deg -- {rashi}, {nak} (pada {pada})");
    }
}
```

### Western chart: aspects and dignities

```rust
use xalen_ephemeris::western::aspects::{find_all_aspects, AspectType};
use xalen_ephemeris::western::dignity::essential_dignity_score;
```

### More examples

```bash
cargo run --example basic_chart      # Sun/Moon + nakshatra + rashi
cargo run --example vedic_chart      # Full Vedic chart with dasha, shadbala, panchang
cargo run --example western_chart    # Aspects, dignities, Arabic Lots
cargo run --example chinese_bazi     # BaZi Four Pillars with Wu Xing
```

## Command line examples

The `xalen` tool puts the library in the terminal: positions, charts, panchang,
dasha, fixed stars, eclipses, rise and set times, aspects, time conversions,
astrocartography maps, synastry, composite, Davison and relocated charts,
as a table or with `--json`. It works offline and has no data files.

```bash
cargo install xalen-cli
```

```bash
# Sidereal (Lahiri) positions with nakshatra and pada
xalen positions --date 2026-11-07T00:00:00Z --sidereal --bodies sun,moon

# Whole-sign Vedic chart for a place in the built-in city table
xalen chart --date 1990-03-15T06:30:00Z --place delhi --sidereal

# Solar eclipses of 2024 as JSON (the 8 April total eclipse is the first)
xalen eclipses --from 2024-01-01 --to 2024-12-31 --type solar --json

# Astrocartography lines for a moment, as GeoJSON for any GIS tool or geojson.io
xalen map --date 2024-06-21T00:00:00Z --bodies sun,moon --geojson

# Aspects and house overlays between two births
xalen synastry --a-date 1990-03-15T06:30:00Z --a-place delhi --b-date 1992-08-02T14:10:00Z --b-place mumbai

# Composite and Davison charts of the same two births
xalen composite --a-date 1990-03-15T06:30:00Z --a-place delhi --b-date 1992-08-02T14:10:00Z --b-place mumbai
xalen davison --a-date 1990-03-15T06:30:00Z --a-place delhi --b-date 1992-08-02T14:10:00Z --b-place mumbai

# The same birth with the houses moved to another place
xalen relocate --date 1990-03-15T06:30:00Z --place delhi --to-place london
```

See [`crates/xalen-cli`](../crates/xalen-cli/README.md) for every command.
