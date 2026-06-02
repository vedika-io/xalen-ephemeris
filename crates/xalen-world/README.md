# xalen-world
> World astrology and calendar systems: Mayan, Tibetan, Korean Saju, Japanese Nine Star Ki, Burmese Mahabote, Persian, Egyptian decans, Celtic tree, and Aztec Tonalpohualli.

Part of the [XALEN Ephemeris](https://github.com/vedika-io/xalen-ephemeris) suite — pure-Rust, thread-safe, Apache-2.0.

## Features
- **Mayan** — Long Count, Tzolkin, Haab, and Calendar Round from a Julian Day (GMT correlation 584283).
- **Tibetan** — Rabjung 60-year cycle: animal, element, gender, and approximate Losar JD for a Western year.
- **Korean Saju** (사주) — Four Pillars chart (year/month/day/hour), Daeun major-luck cycles, and Gunghap compatibility.
- **Japanese Nine Star Ki** (九星気学) — year and month stars from the Lo Shu square, plus elemental compatibility.
- **Burmese Mahabote** (မဟာဘုတ်) — weekday-based profiles (incl. the Wednesday AM/PM split) and compatibility, plus the deterministic 7-house square (`mahabote_house_square`): the birth-lord seated in Binga with the seven planet-lords laid out in the Burmese weekday sequence around Binga..Puti.
- **Persian/Arabic** — Jarbakhtar chronocrator periods (129-year cycle) and an ecliptic-longitude Tasyir arc (`ecliptic_tasyir_arc`, a first-order approximation — not a true right-ascension primary direction).
- **Egyptian** — the 36 decans with Chaldean planetary rulers, looked up by ecliptic degree.
- **Celtic** — 13-month Beth-Luis-Nion tree calendar and birth-year tree.
- **Aztec** — Tonalpohualli 260-day sacred calendar (Caso correlation) from a Gregorian date or Julian Day.

All public types derive `serde::Serialize`. No `unsafe`, no external runtime dependencies beyond `serde`.

## Usage
```rust
use xalen_world::tibetan::tibetan_year;

// The Rabjung 60-year cycle began in 1027 CE.
let year = tibetan_year(2024);
println!("{year}"); // Male Wood Dragon (Rabjung 17, year 38)
assert_eq!(year.animal.name(), "Dragon");
assert_eq!(year.element.name(), "Wood");
```

## Accuracy & sources
Correlations and formulas are documented and verified per module (e.g. the Mayan GMT correlation, the Aztec Caso anchor, and the Nine Star Ki year-star formula); see [ACCURACY.md](../../docs/ACCURACY.md) and [CREDITS.md](../../CREDITS.md) for methodology and references.

## License
Licensed under Apache-2.0. See [LICENSE](../../LICENSE).
