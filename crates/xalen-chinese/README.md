# xalen-chinese
> Chinese astrology in pure Rust — BaZi Four Pillars, the 24 solar terms, Zi Wei Dou Shu, Qi Men Dun Jia, and Feng Shui.

Part of the [XALEN Ephemeris](https://github.com/vedika-io/xalen-ephemeris) suite — pure-Rust, thread-safe, Apache-2.0.

## Features
- **BaZi (Four Pillars / 八字)** — `compute_bazi` builds the year, month, day, and hour pillars plus the day master, with the BaZi year switching on the Li Chun (Start of Spring) boundary rather than Jan 1.
- **Heavenly Stems & Earthly Branches** — the 10 `HeavenlyStem` and 12 `EarthlyBranch` enums, with `element()` (Wu Xing), `animal()` (the 12 zodiac animals), and `is_yang()` helpers.
- **Wu Xing (Five Elements)** — `WuXing` with the generating (`generates`) and overcoming (`overcomes`) cycles.
- **Sexagenary cycle** — `sexagenary_year` and `sexagenary_day` resolve any year / Julian Day to its stem-branch pillar.
- **Solar terms (Jie Qi)** — `solar_term_sun_longitude`, `solar_longitude_approx` (Meeus Ch.25 low-accuracy formula), `solar_month_from_jd`, and `li_chun_jd` map dates to the 24-term / 12-month solar calendar.
- **Zi Wei Dou Shu (Purple Star Astrology)** — `ziwei` module for the 12-palace star chart.
- **Qi Men Dun Jia (奇门遁甲)** — `qimen` module: 9-Star / 8-Door / 8-Deity / San-Qi-Liu-Yi / Lo-Shu reference data plus `compute_qimen`, the **time chart (時家奇門) cast by the San Yuan placement school**. The Ju is set by the solar term + San Yuan Fu Tou yuan (`qimen_ju`), and Zhi Fu / Zhi Shi are anchored to the hour's Xun-head Yi. Qi Men has genuine school variation; this implements the San Yuan time-chart school consistently rather than claiming universality.
- **Feng Shui** — `fengshui` module with Flying Stars (Xuan Kong Fei Xing) and Eight Mansions (Ba Zhai).
- All public types derive `serde::{Serialize, Deserialize}`.

## Usage
```rust
use xalen_chinese::{compute_bazi, HeavenlyStem, EarthlyBranch};

// 1990-01-15 (Julian Day 2447908.0), 10:30 local hour.
// This falls before Li Chun, so the BaZi year is 1989 (Ji-Si).
let chart = compute_bazi(1990, 2447908.0, 10.5);

assert_eq!(chart.year.stem, HeavenlyStem::Ji);
assert_eq!(chart.year.branch, EarthlyBranch::Si);

// The day master is the stem of the day pillar.
println!("Day master: {:?}", chart.day_master);
println!("Year pillar: {}", chart.year); // e.g. "Ji-Si (Earth Snake)"
```

## Accuracy & sources
Solar-term boundaries use the Meeus low-accuracy solar-longitude formula (good to ~0.01°); see [ACCURACY.md](../../docs/ACCURACY.md) and [CREDITS.md](../../CREDITS.md) for methods, references, and limitations.

## License
Licensed under Apache-2.0. See [LICENSE](../../LICENSE).
