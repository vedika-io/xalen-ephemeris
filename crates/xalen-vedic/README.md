# xalen-vedic
> Vedic (Jyotish) astrology computations on sidereal positions — nakshatras, dashas, panchang, yogas, doshas, divisional charts, and compatibility.

Part of the [XALEN Ephemeris](https://github.com/vedika-io/xalen-ephemeris) suite — pure-Rust, thread-safe, Apache-2.0.

This crate turns sidereal longitudes (produced by `xalen-coords` + `xalen-ayanamsa`) into the classical building blocks of a Vedic chart. Computations are deterministic and allocation-light; all public types implement `serde` `Serialize`/`Deserialize`.

## Features
- **Nakshatra & rashi** — 27 lunar mansions and 12 signs from sidereal longitude, with pada, lord, deity, gana, element, and modality.
- **Dasha systems** — Vimshottari (`dasha`), Ashtottari, Yogini, Jaimini Chara, and Sudarshana Chakra.
- **Panchang** — the five limbs (tithi, vara, nakshatra, yoga, karana).
- **Yogas & doshas** — Pancha Mahapurusha / Raja yogas, Mangal (Kuja) Dosha, Kaal Sarp, and Gandanta detection.
- **Divisional (varga) charts** — D-2 through D-60.
- **Strength & dignity** — Shadbala, Ashtakavarga, planetary dignity, and Mrityu Bhaga death-degrees.
- **Specialized branches** — KP (Krishnamurti Paddhati), Jaimini karakas/arudha, Nadi (Bhrigu Nandi Nadi — 48 planet/sign rule slots tagged by life-domain; the interpretive readings are **not bundled** in this open-source release, so each slot's `indication` field returns `None` rather than an empty string), Tajaka / Varshaphal annual charts, Prashna (horary), and Muhurta (electional) timing.
- **Transits & motion** — Gochara transit analysis, retrograde/motion status, and Upagraha sub-bodies (Gulika, Mandi, etc.).
- **i18n** — Hindi, Sanskrit, Tamil, and Telugu names for planets, rashis, nakshatras, tithis, yogas, karanas, and varas.

## Usage
```rust
use xalen_vedic::nakshatra::Nakshatra;
use xalen_vedic::rashi::Rashi;

// A sidereal longitude in degrees (e.g. the Moon's position).
let sidereal_lon = 47.5_f64;

let nak = Nakshatra::from_longitude_deg(sidereal_lon);
let pada = Nakshatra::pada(sidereal_lon);
let rashi = Rashi::from_longitude_deg(sidereal_lon);

println!("Nakshatra: {nak} (pada {pada}), lord {}", nak.lord());
println!("Rashi: {rashi}, lord {}", rashi.lord());
// Nakshatra: Rohini (pada 3), lord Moon
// Rashi: Vrishabha (Taurus), lord Venus
```

## Accuracy & sources
Vedic results depend on the sidereal longitudes fed in (ayanamsa choice and ephemeris accuracy); see [ACCURACY.md](../../docs/ACCURACY.md) for the underlying astronomical accuracy and [CREDITS.md](../../CREDITS.md) for the classical references the algorithms follow.

## License
Licensed under Apache-2.0. See [LICENSE](../../LICENSE).
