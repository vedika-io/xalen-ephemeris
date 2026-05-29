//! Aztec Tonalpohualli — the 260-day sacred calendar.
//!
//! The Tonalpohualli interlocks 20 day-signs with 13 numbers. Each day has a
//! unique combination; after 260 days (20 x 13) the cycle repeats. Days are
//! also grouped into 20 *trecenas* (13-day periods), each with its own
//! ruling day-sign.
//!
//! **Correlation** (Alfonso Caso): August 13, 1521 (Julian calendar) =
//! JD 2276827.5 = Tonalpohualli day **1 Coatl (Serpent)**, day-sign index 4,
//! number 1. This is the fall of Tenochtitlan — the single most consistently
//! documented Aztec date (recorded by Sahagún, corroborated by Native and
//! Spanish sources). Verified 2026-05-28 against azteccalendar.com and an
//! independent Meeus Julian-calendar JD computation.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Julian Day of the correlation anchor: August 13, 1521 (Julian) = 1 Coatl
/// (the fall of Tenochtitlan; Caso correlation).
const ANCHOR_JD: f64 = 2_276_827.5;

/// Day-sign index of the anchor date (Coatl = index 4).
const ANCHOR_SIGN_INDEX: i64 = 4;

/// Day-number of the anchor date (1).
const ANCHOR_NUMBER: i64 = 1;

/// The 20 Nahuatl day-sign names with English translations.
const DAY_SIGNS: [(&str, &str); 20] = [
    ("Cipactli", "Crocodile"),
    ("Ehecatl", "Wind"),
    ("Calli", "House"),
    ("Cuetzpalin", "Lizard"),
    ("Coatl", "Serpent"),
    ("Miquiztli", "Death"),
    ("Mazatl", "Deer"),
    ("Tochtli", "Rabbit"),
    ("Atl", "Water"),
    ("Itzcuintli", "Dog"),
    ("Ozomatli", "Monkey"),
    ("Malinalli", "Grass"),
    ("Acatl", "Reed"),
    ("Ocelotl", "Jaguar"),
    ("Cuauhtli", "Eagle"),
    ("Cozcacuauhtli", "Vulture"),
    ("Ollin", "Movement"),
    ("Tecpatl", "Flint"),
    ("Quiahuitl", "Rain"),
    ("Xochitl", "Flower"),
];

// ---------------------------------------------------------------------------
// TonalpohualliFate
// ---------------------------------------------------------------------------

/// A Tonalpohualli date: the combination of a day-sign and a number.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TonalpohualliFate {
    /// Nahuatl name of the day-sign.
    pub day_sign: &'static str,
    /// English translation of the day-sign.
    pub day_sign_english: &'static str,
    /// Day number within the trecena (1-13).
    pub day_number: u8,
    /// The day-sign that starts the current 13-day trecena.
    pub trecena_ruler: &'static str,
    /// Zero-based index of the day-sign (0-19).
    pub day_sign_index: u8,
}

impl std::fmt::Display for TonalpohualliFate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} ({}), trecena of {}",
            self.day_number, self.day_sign, self.day_sign_english, self.trecena_ruler
        )
    }
}

// ---------------------------------------------------------------------------
// Core computation
// ---------------------------------------------------------------------------

/// Compute the Tonalpohualli date for a given Julian Day number.
pub fn tonalpohualli_from_jd(jd: f64) -> TonalpohualliFate {
    let day_diff = (jd.floor() as i64) - (ANCHOR_JD.floor() as i64);

    // Day-sign index: cycles through 20
    let sign_idx = (ANCHOR_SIGN_INDEX + day_diff).rem_euclid(20) as usize;

    // Day number: cycles through 13 (1-based)
    let number_raw = (ANCHOR_NUMBER - 1 + day_diff).rem_euclid(13) + 1;
    let day_number = number_raw as u8;

    // Trecena ruler: the day-sign at the start of the current 13-day period.
    // Within the 260-day cycle, each trecena starts when day_number == 1.
    // We need to find which day-sign was active `day_number - 1` days ago.
    let trecena_sign_idx = (sign_idx as i64 - (day_number as i64 - 1)).rem_euclid(20) as usize;

    let (day_sign, day_sign_english) = DAY_SIGNS[sign_idx];
    let (trecena_ruler, _) = DAY_SIGNS[trecena_sign_idx];

    TonalpohualliFate {
        day_sign,
        day_sign_english,
        day_number,
        trecena_ruler,
        day_sign_index: sign_idx as u8,
    }
}

/// Convert a Gregorian date to a Julian Day number (noon-based).
fn gregorian_to_jd(year: i32, month: u32, day: u32) -> f64 {
    let y = if month <= 2 { year - 1 } else { year } as f64;
    let m = if month <= 2 { month + 12 } else { month } as f64;
    let d = day as f64;
    let a = (y / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();
    (365.25 * (y + 4716.0)).floor() + (30.6001 * (m + 1.0)).floor() + d + b - 1524.5
}

/// Compute the Tonalpohualli date for a given Gregorian date.
pub fn tonalpohualli_from_date(year: i32, month: u32, day: u32) -> TonalpohualliFate {
    let jd = gregorian_to_jd(year, month, day);
    tonalpohualli_from_jd(jd)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_date_is_1_coatl() {
        // August 13, 1521 (Julian) = JD 2276827.5 = 1 Coatl (fall of Tenochtitlan).
        let fate = tonalpohualli_from_jd(ANCHOR_JD);
        assert_eq!(fate.day_sign, "Coatl");
        assert_eq!(fate.day_sign_english, "Serpent");
        assert_eq!(fate.day_number, 1);
        assert_eq!(fate.day_sign_index, 4);
    }

    #[test]
    fn day_after_anchor_is_2_miquiztli() {
        let fate = tonalpohualli_from_jd(ANCHOR_JD + 1.0);
        assert_eq!(fate.day_sign, "Miquiztli");
        assert_eq!(fate.day_number, 2);
        assert_eq!(fate.day_sign_index, 5);
    }

    #[test]
    fn cycle_repeats_at_260() {
        let a = tonalpohualli_from_jd(ANCHOR_JD);
        let b = tonalpohualli_from_jd(ANCHOR_JD + 260.0);
        assert_eq!(a.day_sign, b.day_sign);
        assert_eq!(a.day_number, b.day_number);
        assert_eq!(a.trecena_ruler, b.trecena_ruler);
    }

    #[test]
    fn all_20_day_signs_appear_in_sequence() {
        for i in 0..20u8 {
            let fate = tonalpohualli_from_jd(ANCHOR_JD + i as f64);
            assert_eq!(fate.day_sign_index, (4 + i) % 20);
        }
    }

    #[test]
    fn day_number_cycles_through_13() {
        for i in 0..13u8 {
            let fate = tonalpohualli_from_jd(ANCHOR_JD + i as f64);
            assert_eq!(fate.day_number, i + 1);
        }
        // Day 14 should be number 1 again
        let fate = tonalpohualli_from_jd(ANCHOR_JD + 13.0);
        assert_eq!(fate.day_number, 1);
    }

    #[test]
    fn trecena_ruler_is_correct_at_start() {
        // At the anchor, day_number is 1, so trecena_ruler == day_sign
        let fate = tonalpohualli_from_jd(ANCHOR_JD);
        assert_eq!(fate.trecena_ruler, fate.day_sign);
    }

    #[test]
    fn trecena_ruler_stable_within_trecena() {
        // All 13 days of a trecena should share the same ruler
        let ruler = tonalpohualli_from_jd(ANCHOR_JD).trecena_ruler;
        for i in 0..13 {
            let fate = tonalpohualli_from_jd(ANCHOR_JD + i as f64);
            assert_eq!(
                fate.trecena_ruler, ruler,
                "day {} has wrong trecena ruler",
                i
            );
        }
    }

    #[test]
    fn from_date_matches_from_jd() {
        // 2024-01-01 Gregorian
        let jd = gregorian_to_jd(2024, 1, 1);
        let a = tonalpohualli_from_jd(jd);
        let b = tonalpohualli_from_date(2024, 1, 1);
        assert_eq!(a, b);
    }

    #[test]
    fn display_format() {
        let fate = tonalpohualli_from_jd(ANCHOR_JD);
        let s = format!("{fate}");
        assert!(s.contains("Coatl"));
        assert!(s.contains("Serpent"));
        assert!(s.contains("1"));
    }
}
