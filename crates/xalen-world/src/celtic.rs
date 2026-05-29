//! Celtic Tree Calendar — 13 lunar months, each governed by a tree.
//!
//! The Beth-Luis-Nion tree alphabet (Ogham) assigns each lunar month a tree,
//! an Ogham letter, an element, and a totem animal. December 23 is the
//! "nameless day" — it falls outside all 13 months.
//!
//! Dates follow the popularised Robert Graves reconstruction (approximate
//! Gregorian boundaries, not true astronomical lunations).

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// CelticMonth
// ---------------------------------------------------------------------------

/// One of the 13 Celtic tree months.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CelticMonth {
    /// Month number, 1-13.
    pub number: u8,
    /// Name of the tree.
    pub tree_name: &'static str,
    /// Ogham letter (first letter of the Irish name).
    pub ogham_letter: char,
    /// Start month (Gregorian).
    pub start_month: u8,
    /// Start day (Gregorian).
    pub start_day: u8,
    /// End month (Gregorian).
    pub end_month: u8,
    /// End day (Gregorian).
    pub end_day: u8,
    /// Associated element.
    pub element: &'static str,
    /// Totem animal.
    pub animal: &'static str,
}

// ---------------------------------------------------------------------------
// Static table
// ---------------------------------------------------------------------------

static CELTIC_MONTHS: [CelticMonth; 13] = [
    CelticMonth {
        number: 1,
        tree_name: "Birch",
        ogham_letter: 'B',
        start_month: 12,
        start_day: 24,
        end_month: 1,
        end_day: 20,
        element: "Earth",
        animal: "White Stag",
    },
    CelticMonth {
        number: 2,
        tree_name: "Rowan",
        ogham_letter: 'L',
        start_month: 1,
        start_day: 21,
        end_month: 2,
        end_day: 17,
        element: "Fire",
        animal: "Dragon",
    },
    CelticMonth {
        number: 3,
        tree_name: "Ash",
        ogham_letter: 'N',
        start_month: 2,
        start_day: 18,
        end_month: 3,
        end_day: 17,
        element: "Water",
        animal: "Seahorse",
    },
    CelticMonth {
        number: 4,
        tree_name: "Alder",
        ogham_letter: 'F',
        start_month: 3,
        start_day: 18,
        end_month: 4,
        end_day: 14,
        element: "Fire",
        animal: "Fox",
    },
    CelticMonth {
        number: 5,
        tree_name: "Willow",
        ogham_letter: 'S',
        start_month: 4,
        start_day: 15,
        end_month: 5,
        end_day: 12,
        element: "Water",
        animal: "Hare",
    },
    CelticMonth {
        number: 6,
        tree_name: "Hawthorn",
        ogham_letter: 'H',
        start_month: 5,
        start_day: 13,
        end_month: 6,
        end_day: 9,
        element: "Fire",
        animal: "Owl",
    },
    CelticMonth {
        number: 7,
        tree_name: "Oak",
        ogham_letter: 'D',
        start_month: 6,
        start_day: 10,
        end_month: 7,
        end_day: 7,
        element: "Earth",
        animal: "Wren",
    },
    CelticMonth {
        number: 8,
        tree_name: "Holly",
        ogham_letter: 'T',
        start_month: 7,
        start_day: 8,
        end_month: 8,
        end_day: 4,
        element: "Fire",
        animal: "Horse",
    },
    CelticMonth {
        number: 9,
        tree_name: "Hazel",
        ogham_letter: 'C',
        start_month: 8,
        start_day: 5,
        end_month: 9,
        end_day: 1,
        element: "Air",
        animal: "Salmon",
    },
    CelticMonth {
        number: 10,
        tree_name: "Vine",
        ogham_letter: 'M',
        start_month: 9,
        start_day: 2,
        end_month: 9,
        end_day: 29,
        element: "Water",
        animal: "Swan",
    },
    CelticMonth {
        number: 11,
        tree_name: "Ivy",
        ogham_letter: 'G',
        start_month: 9,
        start_day: 30,
        end_month: 10,
        end_day: 27,
        element: "Earth",
        animal: "Butterfly",
    },
    CelticMonth {
        number: 12,
        tree_name: "Reed",
        ogham_letter: 'P',
        start_month: 10,
        start_day: 28,
        end_month: 11,
        end_day: 24,
        element: "Water",
        animal: "Hound",
    },
    CelticMonth {
        number: 13,
        tree_name: "Elder",
        ogham_letter: 'R',
        start_month: 11,
        start_day: 25,
        end_month: 12,
        end_day: 22,
        element: "Air",
        animal: "Raven",
    },
];

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Encode month + day into a single comparable integer (month * 100 + day).
const fn encode(month: u32, day: u32) -> u32 {
    month * 100 + day
}

/// Check whether `(month, day)` falls within a Celtic month's date range,
/// handling the year-wrap for Birch (Dec 24 - Jan 20).
fn date_in_range(month: u32, day: u32, cm: &CelticMonth) -> bool {
    let md = encode(month, day);
    let start = encode(cm.start_month as u32, cm.start_day as u32);
    let end = encode(cm.end_month as u32, cm.end_day as u32);

    if start <= end {
        // Normal range within one calendar year
        md >= start && md <= end
    } else {
        // Wraps around year boundary (Birch: Dec 24 - Jan 20)
        md >= start || md <= end
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Return the Celtic tree month for a given Gregorian month (1-12) and day.
///
/// Returns `None` for December 23 (the "nameless day").
pub fn celtic_tree(month: u32, day: u32) -> Option<&'static CelticMonth> {
    CELTIC_MONTHS
        .iter()
        .find(|cm| date_in_range(month, day, cm))
}

/// Return the Celtic tree for a given year in the 13-year cycle.
///
/// Uses `year % 13` to index into the 13 trees. This is a simplified
/// "birth year tree" used in some popular Celtic astrology systems.
pub fn celtic_year_tree(year: i32) -> &'static CelticMonth {
    let idx = year.rem_euclid(13) as usize;
    &CELTIC_MONTHS[idx]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_13_months_present() {
        assert_eq!(CELTIC_MONTHS.len(), 13);
        for (i, cm) in CELTIC_MONTHS.iter().enumerate() {
            assert_eq!(cm.number, (i + 1) as u8);
        }
    }

    #[test]
    fn birch_dec_24_to_jan_20() {
        let d1 = celtic_tree(12, 24).expect("Dec 24 should be Birch");
        assert_eq!(d1.tree_name, "Birch");

        let d2 = celtic_tree(1, 1).expect("Jan 1 should be Birch");
        assert_eq!(d2.tree_name, "Birch");

        let d3 = celtic_tree(1, 20).expect("Jan 20 should be Birch");
        assert_eq!(d3.tree_name, "Birch");
    }

    #[test]
    fn rowan_jan_21() {
        let cm = celtic_tree(1, 21).expect("Jan 21 should be Rowan");
        assert_eq!(cm.tree_name, "Rowan");
    }

    #[test]
    fn oak_summer_solstice() {
        // Jun 21 falls in Oak (Jun 10 - Jul 7)
        let cm = celtic_tree(6, 21).expect("Jun 21 should be Oak");
        assert_eq!(cm.tree_name, "Oak");
    }

    #[test]
    fn elder_last_day() {
        let cm = celtic_tree(12, 22).expect("Dec 22 should be Elder");
        assert_eq!(cm.tree_name, "Elder");
    }

    #[test]
    fn nameless_day_dec_23() {
        assert!(celtic_tree(12, 23).is_none(), "Dec 23 is the nameless day");
    }

    #[test]
    fn year_tree_cycles_at_13() {
        let a = celtic_year_tree(2024);
        let b = celtic_year_tree(2024 + 13);
        assert_eq!(a.tree_name, b.tree_name);
    }

    #[test]
    fn year_tree_negative_year() {
        // Should not panic
        let cm = celtic_year_tree(-500);
        assert!(!cm.tree_name.is_empty());
    }

    #[test]
    fn every_day_covered_except_dec_23() {
        // Check that every day of the year maps to a tree, except Dec 23
        let days_in_month = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        let mut uncovered = Vec::new();
        for m in 1..=12u32 {
            for d in 1..=days_in_month[(m - 1) as usize] {
                if celtic_tree(m, d).is_none() {
                    uncovered.push((m, d));
                }
            }
        }
        assert_eq!(uncovered, vec![(12, 23)], "only Dec 23 should be uncovered");
    }
}
