# xalen-lalkitab
> Lal Kitab remedial astrology — house-based planet analysis, planetary debts, dormancy, a 108-slot remedy lookup, and annual (Varshphal) charts.

Part of the [XALEN Ephemeris](https://github.com/vedika-io/xalen-ephemeris) suite — pure-Rust, thread-safe, Apache-2.0.

Implements the house-based system from Pt. Roop Chand Joshi's five Lal Kitab volumes (1939–1952). Unlike classical BPHS Jyotish, Lal Kitab judges planets by **house occupation** rather than sign placement.

## Features
- **House-based charts** — `LalKitabChart` places planets by house (1–12); natural house rulers via `natural_rulers` (fixed, ascendant-independent).
- **Effect classification** — `planet_effect` returns `Good`, `Neutral`, or `Troubled` for any planet-house combination.
- **Planetary debts (Rin)** — `detect_debts` reports the five karmic debts: Ancestral (Pitri), Maternal (Matri), Spousal (Stri), Self (Atma), and Duty (Dharma).
- **Dormant planets** — `is_dormant` detects Soya Hua Graha (planet in an enemy's house or conjunct an enemy).
- **108-slot remedy lookup** — `remedy_lookup` and `all_remedies` enumerate the full 9-planet × 12-house remedy scaffold. The prescribed remedy and material-item text is **not bundled** in this open-source release (the readings are sourced from copyrighted Lal Kitab volumes), so the `remedies` / `items` fields are `Option<Vec<String>>` and are currently `None` (genuinely absent) rather than `Some(vec![])` — callers can therefore tell "no text bundled" from a populated list. Supply your own remedy copy keyed by planet-house.
- **Varshphal** — `build_annual_chart` derives the Lal Kitab annual chart by the standard 12-year house rotation.
- `serde` `Serialize`/`Deserialize` on all public data types.

## Usage
```rust
use xalen_lalkitab::{
    LalKitabChart, Planet, planet_effect, detect_debts, remedy_lookup,
};

let mut chart = LalKitabChart::empty();
chart.place(Planet::Sun, 6);
chart.place(Planet::Saturn, 6);

// Classify a placement.
assert_eq!(
    planet_effect(Planet::Sun, 6).to_string(),
    "Troubled"
);

// Detect karmic debts (Sun in house 6 triggers Pitri Rin).
let debts = detect_debts(&chart);
assert!(!debts.is_empty());
println!("{}", debts[0].kind); // "Pitri Rin (Ancestral Debt)"

// Look up remedies for the troubled Sun. The remedy / item text is not
// bundled in this open-source release, so `remedies` is `None` (absent),
// never `Some(vec![])`.
let remedy = remedy_lookup(Planet::Sun, 6);
match &remedy.remedies {
    Some(steps) => {
        for step in steps {
            println!("- {step}");
        }
    }
    None => println!("(no remedy text bundled for this combination)"),
}
```

## Accuracy & sources
House-effect, debt, enmity, and remedy tables are encoded from Pt. Roop Chand Joshi's Lal Kitab volumes; see [ACCURACY.md](../../docs/ACCURACY.md) and [CREDITS.md](../../CREDITS.md) for sourcing and verification notes.

## License
Licensed under Apache-2.0. See [LICENSE](../../LICENSE).
