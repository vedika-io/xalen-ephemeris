//! Japanese Nine Star Ki (九星気学, Kigaku)
//!
//! Maps birth year and month to one of 9 numbers derived from the Lo Shu
//! magic square. Each star has an associated element, direction, and trigram.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Star metadata
// ---------------------------------------------------------------------------

/// Element, direction, and trigram for each of the 9 stars.
struct StarInfo {
    element: &'static str,
    direction: &'static str,
    trigram: &'static str,
}

const STAR_INFO: [StarInfo; 9] = [
    StarInfo {
        element: "Water",
        direction: "North",
        trigram: "Kan",
    }, // 1
    StarInfo {
        element: "Earth",
        direction: "Southwest",
        trigram: "Kun",
    }, // 2
    StarInfo {
        element: "Tree",
        direction: "East",
        trigram: "Zhen",
    }, // 3
    StarInfo {
        element: "Tree",
        direction: "Southeast",
        trigram: "Xun",
    }, // 4
    StarInfo {
        element: "Earth",
        direction: "Center",
        trigram: "None",
    }, // 5
    StarInfo {
        element: "Metal",
        direction: "Northwest",
        trigram: "Qian",
    }, // 6
    StarInfo {
        element: "Metal",
        direction: "West",
        trigram: "Dui",
    }, // 7
    StarInfo {
        element: "Earth",
        direction: "Northeast",
        trigram: "Gen",
    }, // 8
    StarInfo {
        element: "Fire",
        direction: "South",
        trigram: "Li",
    }, // 9
];

// ---------------------------------------------------------------------------
// Month star tables (Feb = index 0, Jan = index 11)
//
// Group A: year stars 1, 4, 7
// Group B: year stars 2, 5, 8
// Group C: year stars 3, 6, 9
// ---------------------------------------------------------------------------

const MONTH_STARS_A: [u8; 12] = [8, 7, 6, 5, 4, 3, 2, 1, 9, 8, 7, 6];
const MONTH_STARS_B: [u8; 12] = [2, 1, 9, 8, 7, 6, 5, 4, 3, 2, 1, 9];
const MONTH_STARS_C: [u8; 12] = [5, 4, 3, 2, 1, 9, 8, 7, 6, 5, 4, 3];

// ---------------------------------------------------------------------------
// NineStarKiProfile
// ---------------------------------------------------------------------------

/// A Nine Star Ki profile for a given birth year and month.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NineStarKiProfile {
    /// Year star (1-9).
    pub year_star: u8,
    /// Month star (1-9).
    pub month_star: u8,
    /// Element of the year star.
    pub element: &'static str,
    /// Direction associated with the year star.
    pub direction: &'static str,
    /// I Ching trigram of the year star.
    pub trigram: &'static str,
}

impl std::fmt::Display for NineStarKiProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}-{} ({}, {}, {})",
            self.year_star, self.month_star, self.element, self.direction, self.trigram
        )
    }
}

// ---------------------------------------------------------------------------
// Computation
// ---------------------------------------------------------------------------

/// Compute the year star (1-9) for a given year.
///
/// Formula: `(11 - (year % 9)) % 9`; if result is 0, use 9.
/// Equivalent to the traditional method: add year digits to a single digit,
/// subtract from 11 (subtract 9 more if result > 9).
///
/// Verified against: <https://kimimihealingarts.com/2024/05/01/nine-star-ki-astrology-2024-year-of-the-wood-dragon/>
/// and <https://the-art-of-wind-and-water.com/3-yang-wood-house-2024/>
/// Known values: 2020=7 (Metal), 2024=3 (Tree/Wood), 2025=2 (Earth).
///
/// Note: the Nine Star Ki year begins in February (Chinese Solar New Year).
/// For dates in January, use the previous year.
pub fn year_star(year: i32) -> u8 {
    let rem = (year.rem_euclid(9)) as u8;
    let raw = (11 - rem) % 9;
    if raw == 0 { 9 } else { raw }
}

/// Compute the month star (1-9) for a given year and month.
///
/// The month star depends on which group the year star falls into:
/// - Group A (year star 1, 4, 7): starts at 8 in February, descending
/// - Group B (year star 2, 5, 8): starts at 2 in February, descending
/// - Group C (year star 3, 6, 9): starts at 5 in February, descending
///
/// Month 1 = February, month 12 = January (of next year).
/// Input `month` is the Gregorian month (1-12). February = 2, January = 1.
pub fn month_star(year: i32, month: u32) -> u8 {
    // Adjust: Nine Star Ki months start in February.
    // Gregorian month 2 -> index 0, month 3 -> index 1, ..., month 1 -> index 11.
    let ki_year = if month == 1 { year - 1 } else { year };
    let month_idx = if month == 1 { 11 } else { (month - 2) as usize };

    let ys = year_star(ki_year);
    let group = (ys - 1) % 3; // 0 = A (1,4,7), 1 = B (2,5,8), 2 = C (3,6,9)

    let table = match group {
        0 => &MONTH_STARS_A,
        1 => &MONTH_STARS_B,
        _ => &MONTH_STARS_C,
    };

    table[month_idx]
}

/// Compute a full Nine Star Ki profile for a given year and month.
pub fn nine_star_ki(year: i32, month: u32) -> NineStarKiProfile {
    let ys = year_star(if month == 1 { year - 1 } else { year });
    let ms = month_star(year, month);
    let info = &STAR_INFO[(ys - 1) as usize];

    NineStarKiProfile {
        year_star: ys,
        month_star: ms,
        element: info.element,
        direction: info.direction,
        trigram: info.trigram,
    }
}

/// Return the stars that are compatible with the given star.
///
/// Compatibility is based on the generating (productive) cycle of elements:
/// - Water (1) generates Tree (3, 4) and is generated by Metal (6, 7)
/// - Tree (3, 4) generates Fire (9) and is generated by Water (1)
/// - Fire (9) generates Earth (2, 5, 8) and is generated by Tree (3, 4)
/// - Earth (2, 5, 8) generates Metal (6, 7) and is generated by Fire (9)
/// - Metal (6, 7) generates Water (1) and is generated by Earth (2, 5, 8)
/// - Same element stars are also compatible
pub fn compatible_stars(star: u8) -> Vec<u8> {
    match star {
        1 => vec![3, 4, 6, 7],    // Water: generated by Metal, generates Tree
        2 => vec![5, 8, 6, 7, 9], // Earth: same-element + generates Metal + generated by Fire
        3 => vec![1, 4, 9],       // Tree: same-element + generated by Water + generates Fire
        4 => vec![1, 3, 9],       // Tree: same-element + generated by Water + generates Fire
        5 => vec![2, 8, 6, 7, 9], // Earth: same-element + generates Metal + generated by Fire
        6 => vec![1, 2, 5, 7, 8], // Metal: same-element + generates Water + generated by Earth
        7 => vec![1, 2, 5, 6, 8], // Metal: same-element + generates Water + generated by Earth
        8 => vec![2, 5, 6, 7, 9], // Earth: same-element + generates Metal + generated by Fire
        9 => vec![2, 3, 4, 5, 8], // Fire: generates Earth + generated by Tree
        _ => vec![],
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn year_star_known_values() {
        // Web-verified: 2024 = 3 Tree/Wood
        // Source: https://the-art-of-wind-and-water.com/3-yang-wood-house-2024/
        assert_eq!(year_star(2024), 3);
        // 2023 = 4 Tree
        assert_eq!(year_star(2023), 4);
        // 2020 = 7 Metal
        // Source: multiple 9 Star Ki forecasts confirm 2020 as 7 Metal
        assert_eq!(year_star(2020), 7);
        // 2025 = 2 Earth
        // Source: https://mindfuldesignschool.com/blog/nine-star-ki-forecast-for-2025
        assert_eq!(year_star(2025), 2);
    }

    #[test]
    fn year_star_cycle_repeats_at_9() {
        let s = year_star(2024);
        assert_eq!(year_star(2024 + 9), s);
        assert_eq!(year_star(2024 + 18), s);
    }

    #[test]
    fn year_star_range_1_to_9() {
        for y in 1900..2100 {
            let s = year_star(y);
            assert!(s >= 1 && s <= 9, "year {y} gave star {s}");
        }
    }

    #[test]
    fn month_star_february_group_a() {
        // For year star 7 (group A: 1,4,7), February should be 8
        // 2020 has year_star = 7 (Metal), which is in group A
        assert_eq!(year_star(2020), 7);
        assert_eq!(month_star(2020, 2), 8);
    }

    #[test]
    fn month_star_range_1_to_9() {
        for y in 2020..2030 {
            for m in 1..=12 {
                let s = month_star(y, m);
                assert!(s >= 1 && s <= 9, "year {y} month {m} gave star {s}");
            }
        }
    }

    #[test]
    fn profile_has_correct_element() {
        // 2020 year star = 7 = Metal (West, Dui)
        let p = nine_star_ki(2020, 6);
        assert_eq!(p.element, "Metal");
    }

    #[test]
    fn profile_display_format() {
        let p = nine_star_ki(2024, 5);
        let s = format!("{p}");
        assert!(s.contains('-')); // "year-month" format
    }

    #[test]
    fn compatible_stars_water() {
        let compat = compatible_stars(1);
        assert!(compat.contains(&3)); // Tree
        assert!(compat.contains(&6)); // Metal
        assert!(!compat.contains(&9)); // Fire clashes with Water
    }

    #[test]
    fn compatible_stars_all_valid() {
        for star in 1..=9 {
            let compat = compatible_stars(star);
            assert!(
                !compat.is_empty(),
                "star {star} should have compatible stars"
            );
            for c in &compat {
                assert!(*c >= 1 && *c <= 9);
            }
        }
    }

    #[test]
    fn january_uses_previous_year() {
        // January 2024 should use 2023's year star for month calculation
        let p = nine_star_ki(2024, 1);
        // Year star for 2023 = 4 (Tree)
        assert_eq!(year_star(2023), 4);
        assert_eq!(p.year_star, year_star(2023));
    }

    #[test]
    fn star_info_complete() {
        // Verify all 9 stars have metadata
        for i in 0..9 {
            assert!(!STAR_INFO[i].element.is_empty());
            assert!(!STAR_INFO[i].direction.is_empty());
            assert!(!STAR_INFO[i].trigram.is_empty());
        }
    }
}
