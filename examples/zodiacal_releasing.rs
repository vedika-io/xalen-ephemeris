//! Zodiacal Releasing (Hellenistic timing technique, Vettius Valens).
//!
//! Run with: cargo run --example zodiacal_releasing

use xalen_time::{CalendarSystem, calendar_to_jd, jd_to_calendar};
use xalen_western::hellenistic::zodiacal_releasing;

const SIGNS: [&str; 12] = [
    "Aries",
    "Taurus",
    "Gemini",
    "Cancer",
    "Leo",
    "Virgo",
    "Libra",
    "Scorpio",
    "Sagittarius",
    "Capricorn",
    "Aquarius",
    "Pisces",
];

fn main() {
    let birth_jd = calendar_to_jd(1947, 8, 15, 0.0 - 5.5, CalendarSystem::default());

    // Lot of Fortune assumed in Cancer (sign index 3)
    let lot_sign = 3;
    let periods = zodiacal_releasing(lot_sign, birth_jd.0, 1);

    println!("=== Zodiacal Releasing from {} ===\n", SIGNS[lot_sign]);

    let l1_periods: Vec<_> = periods.iter().filter(|p| p.level == 1).collect();
    for p in &l1_periods {
        let (sy, sm, sd, _) = jd_to_calendar(p.start_jd, CalendarSystem::default());
        let (ey, em, ed, _) = jd_to_calendar(p.end_jd, CalendarSystem::default());
        println!(
            "{:<14} {:04}-{:02}-{:02} to {:04}-{:02}-{:02}  ({:.0} yrs)",
            SIGNS[p.sign_index], sy, sm, sd, ey, em, ed, p.years
        );
    }
}
