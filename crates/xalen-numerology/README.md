# xalen-numerology
> Pythagorean & Chaldean numerology — Life Path, Expression, Soul Urge, and cyclic forecast numbers.

Part of the [XALEN Ephemeris](https://github.com/vedika-io/xalen-ephemeris) suite — pure-Rust, thread-safe, Apache-2.0.

## Features
- Two letter-value systems: `System::Pythagorean` (A=1…I=9) and `System::Chaldean` (1–8, no letter ever maps to 9).
- Core name numbers: `expression_number`, `soul_urge_number` (vowels), `personality_number` (consonants), plus raw `name_value`, `vowel_value`, `consonant_value`.
- Context-aware Y-as-vowel variants (`*_smart`) that treat Y as a vowel by word position and adjacent letters (e.g. "Lynn", "Emily", "Kyle").
- Date numbers: `life_path`, `birthday_number`, `maturity_number`.
- Forecast cycles: `personal_year`, `personal_month`, `personal_day`.
- `challenge_numbers` and `pinnacle_numbers` (4-element arrays for life stages).
- Master-number preservation (11, 22, 33) via `reduce(n, preserve_master)`, plus `check_master_number` and `karmic_debt` (13/14/16/19) detection.
- `lo_shu_grid` magic-square placement from `birth_date_digits`.
- `full_profile` to compute a `NumerologyProfile` in one call. (`number_meaning` returns `Option<&'static str>` for each single-digit and master number; the descriptive text is not bundled in this open-source release, so the function currently returns `None` (genuinely absent) for every number rather than `Some("")` — callers can tell "no meaning text" from a real meaning. Supply your own keyword copy.)
- All public types derive `serde::Serialize` / `Deserialize`.

## Usage
```rust
use xalen_numerology::{full_profile, life_path, expression_number, System};

// One-call profile for a birth date + name
let profile = full_profile(1990, 7, 15, "Jane Doe", System::Pythagorean);
println!("Life Path:   {}", profile.life_path);
println!("Expression:  {}", profile.expression);
println!("Soul Urge:   {}", profile.soul_urge);

// Individual numbers
assert_eq!(life_path(1809, 2, 12), 5); // Feb 12, 1809
assert_eq!(expression_number("JOHN", System::Pythagorean), 2);
```

## Accuracy & sources
Numerology is a deterministic letter/digit reduction scheme, not an astronomical computation; the letter tables and reduction rules follow the standard Pythagorean and Chaldean systems (see the Chaldean note in `src/lib.rs`). For the suite's verification methodology and references see [ACCURACY.md](../../docs/ACCURACY.md) and [CREDITS.md](../../CREDITS.md).

## License
Licensed under Apache-2.0. See [LICENSE](../../LICENSE).
