//! Lal Kitab remedy catalog -- 108 planet-house combinations.
//!
//! Each of the 9 planets in each of the 12 houses has a remedy entry keyed by
//! (planet, house) as prescribed in the Lal Kitab tradition of
//! Pt. Roop Chand Joshi. Remedy and item text is omitted here.

use crate::houses::Planet;
use serde::{Deserialize, Serialize};

/// A Lal Kitab remedy for a specific planet-house combination.
///
/// Honesty contract: this crate ships **no** AI-generated or
/// copyright-encumbered remedy prose. The remedy and item text from
/// Pt. Roop Chand Joshi's *Lal Kitab* is still encumbered and has not been
/// backfilled, so [`remedies`](Self::remedies) and [`items`](Self::items) are
/// `None` (genuinely absent) rather than `Some(vec![])`. A `Some(_)` would
/// falsely advertise that real content is present; `None` lets a caller
/// distinguish "no remedy text bundled for this combination" from "a remedy
/// that happens to list zero items".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LalKitabRemedy {
    pub planet: Planet,
    /// House number (1-12).
    pub house: usize,
    /// Textual remedy descriptions, or `None` when no remedy text is bundled
    /// for this planet-house combination.
    pub remedies: Option<Vec<String>>,
    /// Specific items involved (donations, wearing, keeping at home), or `None`
    /// when no item data is bundled for this planet-house combination.
    pub items: Option<Vec<String>>,
}

/// Look up remedies for `planet` in `house` (1-12).
///
/// Always returns a `LalKitabRemedy` correctly keyed by `planet` and `house`
/// (the input house is clamped to 1-12). The `remedies` / `items` fields are
/// `None` whenever no real remedy text is bundled for the combination — see
/// the [`LalKitabRemedy`] honesty contract. They are `Some(_)` only once
/// genuine public-domain or licensed Lal Kitab text has been added.
pub fn remedy_lookup(planet: Planet, house: usize) -> LalKitabRemedy {
    let house = house.clamp(1, 12);

    let (remedies, items) = remedy_data(planet, house);
    // Map an empty list (stripped / not-yet-backfilled) to `None`, and a
    // populated list to `Some(_)`. This is the single point that enforces the
    // "absent ⇒ None, present ⇒ Some" honesty contract.
    let to_opt = |v: Vec<&'static str>| -> Option<Vec<String>> {
        if v.is_empty() {
            None
        } else {
            Some(v.into_iter().map(String::from).collect())
        }
    };
    LalKitabRemedy {
        planet,
        house,
        remedies: to_opt(remedies),
        items: to_opt(items),
    }
}

/// Internal lookup returning (remedies, items) for each of the 108 combos.
fn remedy_data(planet: Planet, house: usize) -> (Vec<&'static str>, Vec<&'static str>) {
    match (planet, house) {
        // =====================================================================
        // SUN in houses 1-12
        // =====================================================================
        (Planet::Sun, 1) => (vec![], vec![]),
        (Planet::Sun, 2) => (vec![], vec![]),
        (Planet::Sun, 3) => (vec![], vec![]),
        (Planet::Sun, 4) => (vec![], vec![]),
        (Planet::Sun, 5) => (vec![], vec![]),
        (Planet::Sun, 6) => (vec![], vec![]),
        (Planet::Sun, 7) => (vec![], vec![]),
        (Planet::Sun, 8) => (vec![], vec![]),
        (Planet::Sun, 9) => (vec![], vec![]),
        (Planet::Sun, 10) => (vec![], vec![]),
        (Planet::Sun, 11) => (vec![], vec![]),
        (Planet::Sun, 12) => (vec![], vec![]),

        // =====================================================================
        // MOON in houses 1-12
        // =====================================================================
        (Planet::Moon, 1) => (vec![], vec![]),
        (Planet::Moon, 2) => (vec![], vec![]),
        (Planet::Moon, 3) => (vec![], vec![]),
        (Planet::Moon, 4) => (vec![], vec![]),
        (Planet::Moon, 5) => (vec![], vec![]),
        (Planet::Moon, 6) => (vec![], vec![]),
        (Planet::Moon, 7) => (vec![], vec![]),
        (Planet::Moon, 8) => (vec![], vec![]),
        (Planet::Moon, 9) => (vec![], vec![]),
        (Planet::Moon, 10) => (vec![], vec![]),
        (Planet::Moon, 11) => (vec![], vec![]),
        (Planet::Moon, 12) => (vec![], vec![]),

        // =====================================================================
        // MARS in houses 1-12
        // =====================================================================
        (Planet::Mars, 1) => (vec![], vec![]),
        (Planet::Mars, 2) => (vec![], vec![]),
        (Planet::Mars, 3) => (vec![], vec![]),
        (Planet::Mars, 4) => (vec![], vec![]),
        (Planet::Mars, 5) => (vec![], vec![]),
        (Planet::Mars, 6) => (vec![], vec![]),
        (Planet::Mars, 7) => (vec![], vec![]),
        (Planet::Mars, 8) => (vec![], vec![]),
        (Planet::Mars, 9) => (vec![], vec![]),
        (Planet::Mars, 10) => (vec![], vec![]),
        (Planet::Mars, 11) => (vec![], vec![]),
        (Planet::Mars, 12) => (vec![], vec![]),

        // =====================================================================
        // MERCURY in houses 1-12
        // =====================================================================
        (Planet::Mercury, 1) => (vec![], vec![]),
        (Planet::Mercury, 2) => (vec![], vec![]),
        (Planet::Mercury, 3) => (vec![], vec![]),
        (Planet::Mercury, 4) => (vec![], vec![]),
        (Planet::Mercury, 5) => (vec![], vec![]),
        (Planet::Mercury, 6) => (vec![], vec![]),
        (Planet::Mercury, 7) => (vec![], vec![]),
        (Planet::Mercury, 8) => (vec![], vec![]),
        (Planet::Mercury, 9) => (vec![], vec![]),
        (Planet::Mercury, 10) => (vec![], vec![]),
        (Planet::Mercury, 11) => (vec![], vec![]),
        (Planet::Mercury, 12) => (vec![], vec![]),

        // =====================================================================
        // JUPITER in houses 1-12
        // =====================================================================
        (Planet::Jupiter, 1) => (vec![], vec![]),
        (Planet::Jupiter, 2) => (vec![], vec![]),
        (Planet::Jupiter, 3) => (vec![], vec![]),
        (Planet::Jupiter, 4) => (vec![], vec![]),
        (Planet::Jupiter, 5) => (vec![], vec![]),
        (Planet::Jupiter, 6) => (vec![], vec![]),
        (Planet::Jupiter, 7) => (vec![], vec![]),
        (Planet::Jupiter, 8) => (vec![], vec![]),
        (Planet::Jupiter, 9) => (vec![], vec![]),
        (Planet::Jupiter, 10) => (vec![], vec![]),
        (Planet::Jupiter, 11) => (vec![], vec![]),
        (Planet::Jupiter, 12) => (vec![], vec![]),

        // =====================================================================
        // VENUS in houses 1-12
        // =====================================================================
        (Planet::Venus, 1) => (vec![], vec![]),
        (Planet::Venus, 2) => (vec![], vec![]),
        (Planet::Venus, 3) => (vec![], vec![]),
        (Planet::Venus, 4) => (vec![], vec![]),
        (Planet::Venus, 5) => (vec![], vec![]),
        (Planet::Venus, 6) => (vec![], vec![]),
        (Planet::Venus, 7) => (vec![], vec![]),
        (Planet::Venus, 8) => (vec![], vec![]),
        (Planet::Venus, 9) => (vec![], vec![]),
        (Planet::Venus, 10) => (vec![], vec![]),
        (Planet::Venus, 11) => (vec![], vec![]),
        (Planet::Venus, 12) => (vec![], vec![]),

        // =====================================================================
        // SATURN in houses 1-12
        // =====================================================================
        (Planet::Saturn, 1) => (vec![], vec![]),
        (Planet::Saturn, 2) => (vec![], vec![]),
        (Planet::Saturn, 3) => (vec![], vec![]),
        (Planet::Saturn, 4) => (vec![], vec![]),
        (Planet::Saturn, 5) => (vec![], vec![]),
        (Planet::Saturn, 6) => (vec![], vec![]),
        (Planet::Saturn, 7) => (vec![], vec![]),
        (Planet::Saturn, 8) => (vec![], vec![]),
        (Planet::Saturn, 9) => (vec![], vec![]),
        (Planet::Saturn, 10) => (vec![], vec![]),
        (Planet::Saturn, 11) => (vec![], vec![]),
        (Planet::Saturn, 12) => (vec![], vec![]),

        // =====================================================================
        // RAHU in houses 1-12
        // =====================================================================
        (Planet::Rahu, 1) => (vec![], vec![]),
        (Planet::Rahu, 2) => (vec![], vec![]),
        (Planet::Rahu, 3) => (vec![], vec![]),
        (Planet::Rahu, 4) => (vec![], vec![]),
        (Planet::Rahu, 5) => (vec![], vec![]),
        (Planet::Rahu, 6) => (vec![], vec![]),
        (Planet::Rahu, 7) => (vec![], vec![]),
        (Planet::Rahu, 8) => (vec![], vec![]),
        (Planet::Rahu, 9) => (vec![], vec![]),
        (Planet::Rahu, 10) => (vec![], vec![]),
        (Planet::Rahu, 11) => (vec![], vec![]),
        (Planet::Rahu, 12) => (vec![], vec![]),

        // =====================================================================
        // KETU in houses 1-12
        // =====================================================================
        (Planet::Ketu, 1) => (vec![], vec![]),
        (Planet::Ketu, 2) => (vec![], vec![]),
        (Planet::Ketu, 3) => (vec![], vec![]),
        (Planet::Ketu, 4) => (vec![], vec![]),
        (Planet::Ketu, 5) => (vec![], vec![]),
        (Planet::Ketu, 6) => (vec![], vec![]),
        (Planet::Ketu, 7) => (vec![], vec![]),
        (Planet::Ketu, 8) => (vec![], vec![]),
        (Planet::Ketu, 9) => (vec![], vec![]),
        (Planet::Ketu, 10) => (vec![], vec![]),
        (Planet::Ketu, 11) => (vec![], vec![]),
        (Planet::Ketu, 12) => (vec![], vec![]),

        _ => (vec![], vec![]),
    }
}

/// Return all 108 remedy entries.
pub fn all_remedies() -> Vec<LalKitabRemedy> {
    let mut out = Vec::with_capacity(108);
    for p in Planet::VEDIC_NINE {
        for h in 1..=12 {
            out.push(remedy_lookup(p, h));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_108_entries_exist() {
        let all = all_remedies();
        assert_eq!(all.len(), 108);
    }

    #[test]
    fn every_entry_has_planet_and_house() {
        for r in all_remedies() {
            assert!(r.house >= 1 && r.house <= 12);
        }
    }

    #[test]
    fn empty_entries_report_none_not_some_empty() {
        // Honesty contract: no remedy prose is bundled, so every one of the 108
        // planet-house combinations must report absent content as `None`,
        // never `Some(vec![])`. This lets callers tell "real remedy text" from
        // "stripped / not yet backfilled".
        for r in all_remedies() {
            assert_eq!(
                r.remedies, None,
                "{:?} in house {} must report remedies as None, not Some(vec![])",
                r.planet, r.house
            );
            assert_eq!(
                r.items, None,
                "{:?} in house {} must report items as None, not Some(vec![])",
                r.planet, r.house
            );
        }
    }

    #[test]
    fn lookup_does_not_panic_for_all_combos() {
        for p in Planet::VEDIC_NINE {
            for h in 1..=12 {
                let r = remedy_lookup(p, h);
                assert_eq!(r.planet, p);
                assert_eq!(r.house, h);
            }
        }
    }

    #[test]
    fn sun_in_1_keyed_correctly() {
        let r = remedy_lookup(Planet::Sun, 1);
        assert_eq!(r.planet, Planet::Sun);
        assert_eq!(r.house, 1);
    }

    #[test]
    fn saturn_in_10_keyed_correctly() {
        let r = remedy_lookup(Planet::Saturn, 10);
        assert_eq!(r.planet, Planet::Saturn);
        assert_eq!(r.house, 10);
    }

    #[test]
    fn ketu_in_6_keyed_correctly() {
        let r = remedy_lookup(Planet::Ketu, 6);
        assert_eq!(r.planet, Planet::Ketu);
        assert_eq!(r.house, 6);
    }
}
