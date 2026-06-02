//! Bhrigu Nandi Nadi (BNN) predictive rule slots.
//!
//! 4 planets x 12 signs = 48 rule slots covering Sun, Moon, Jupiter, and Saturn.
//!
//! Each slot carries a life-domain classification (career, health, etc.) but the
//! interpretive `indication` text is intentionally NOT bundled in this
//! open-source crate (the BNN source readings are copyrighted). The slots define
//! the planet/sign/domain scaffold; downstream callers supply the prose.

use serde::{Deserialize, Serialize};

use crate::prashna::Planet;

// ---------------------------------------------------------------------------
// Domain enum
// ---------------------------------------------------------------------------

/// The life domain a Nadi rule addresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NadiDomain {
    Career,
    Relationship,
    Health,
    Finance,
    Spiritual,
}

impl std::fmt::Display for NadiDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            NadiDomain::Career => "Career",
            NadiDomain::Relationship => "Relationship",
            NadiDomain::Health => "Health",
            NadiDomain::Finance => "Finance",
            NadiDomain::Spiritual => "Spiritual",
        };
        write!(f, "{name}")
    }
}

// ---------------------------------------------------------------------------
// NadiRule
// ---------------------------------------------------------------------------

/// A single Nadi prediction rule for a planet in a sign.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NadiRule {
    pub planet: Planet,
    /// Sign index 0-11 (0=Aries ... 11=Pisces).
    pub sign: usize,
    /// Concise indication text, or `None` when not bundled.
    ///
    /// The BNN interpretive readings are copyrighted and intentionally NOT
    /// shipped in this open-source crate, so this is `None` for every slot here
    /// (see module docs). It is `Option<&str>` — not `Some("")` — as a
    /// deliberate empty-API honesty contract: an absent reading is reported as
    /// genuinely absent, never as a misleading empty string that looks like
    /// data. When a caller bundles real readings, a non-empty string surfaces
    /// as `Some(...)`. The life-`domain` field below carries the only
    /// classification this crate provides.
    pub indication: Option<&'static str>,
    /// Life domain this rule addresses.
    pub domain: NadiDomain,
}

// ---------------------------------------------------------------------------
// 48 foundational rule slots: Sun, Moon, Jupiter, Saturn x 12 signs
// (planet + sign + life-domain scaffold; indication text intentionally
// unbundled — surfaced as `None`, see `nadi_indications`)
// ---------------------------------------------------------------------------

static SUN_RULES: [(&str, NadiDomain); 12] = [
    // Aries
    ("", NadiDomain::Career),
    // Taurus
    ("", NadiDomain::Finance),
    // Gemini
    ("", NadiDomain::Career),
    // Cancer
    ("", NadiDomain::Career),
    // Leo
    ("", NadiDomain::Career),
    // Virgo
    ("", NadiDomain::Career),
    // Libra (debilitated)
    ("", NadiDomain::Relationship),
    // Scorpio
    ("", NadiDomain::Career),
    // Sagittarius
    ("", NadiDomain::Spiritual),
    // Capricorn
    ("", NadiDomain::Career),
    // Aquarius
    ("", NadiDomain::Career),
    // Pisces
    ("", NadiDomain::Spiritual),
];

static MOON_RULES: [(&str, NadiDomain); 12] = [
    // Aries
    ("", NadiDomain::Health),
    // Taurus (exalted)
    ("", NadiDomain::Relationship),
    // Gemini
    ("", NadiDomain::Health),
    // Cancer (own sign)
    ("", NadiDomain::Relationship),
    // Leo
    ("", NadiDomain::Relationship),
    // Virgo
    ("", NadiDomain::Health),
    // Libra
    ("", NadiDomain::Relationship),
    // Scorpio (debilitated)
    ("", NadiDomain::Health),
    // Sagittarius
    ("", NadiDomain::Spiritual),
    // Capricorn
    ("", NadiDomain::Health),
    // Aquarius
    ("", NadiDomain::Health),
    // Pisces
    ("", NadiDomain::Spiritual),
];

static JUPITER_RULES: [(&str, NadiDomain); 12] = [
    // Aries
    ("", NadiDomain::Finance),
    // Taurus
    ("", NadiDomain::Finance),
    // Gemini
    ("", NadiDomain::Finance),
    // Cancer (exalted)
    ("", NadiDomain::Finance),
    // Leo
    ("", NadiDomain::Finance),
    // Virgo
    ("", NadiDomain::Career),
    // Libra
    ("", NadiDomain::Finance),
    // Scorpio
    ("", NadiDomain::Spiritual),
    // Sagittarius (own sign)
    ("", NadiDomain::Spiritual),
    // Capricorn (debilitated)
    ("", NadiDomain::Finance),
    // Aquarius
    ("", NadiDomain::Finance),
    // Pisces (own sign)
    ("", NadiDomain::Spiritual),
];

static SATURN_RULES: [(&str, NadiDomain); 12] = [
    // Aries (debilitated)
    ("", NadiDomain::Health),
    // Taurus
    ("", NadiDomain::Finance),
    // Gemini
    ("", NadiDomain::Career),
    // Cancer
    ("", NadiDomain::Health),
    // Leo
    ("", NadiDomain::Career),
    // Virgo
    ("", NadiDomain::Health),
    // Libra (exalted)
    ("", NadiDomain::Career),
    // Scorpio
    ("", NadiDomain::Spiritual),
    // Sagittarius
    ("", NadiDomain::Spiritual),
    // Capricorn (own sign)
    ("", NadiDomain::Career),
    // Aquarius (own sign)
    ("", NadiDomain::Career),
    // Pisces
    ("", NadiDomain::Spiritual),
];

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Look up Nadi rule slots for a given planet in a given sign.
///
/// Returns foundational rule slots for Sun, Moon, Jupiter, and Saturn. Each slot
/// carries the planet/sign/life-domain scaffold; the `indication` text is
/// `None` (the BNN interpretive readings are not bundled in this crate — see
/// module docs). Other planets return an empty vec (future expansion).
///
/// `sign` is 0-based (0 = Aries ... 11 = Pisces). Values > 11 are wrapped.
pub fn nadi_indications(planet: Planet, sign: usize) -> Vec<NadiRule> {
    let s = sign % 12;
    let table: Option<&[(&str, NadiDomain); 12]> = match planet {
        Planet::Sun => Some(&SUN_RULES),
        Planet::Moon => Some(&MOON_RULES),
        Planet::Jupiter => Some(&JUPITER_RULES),
        Planet::Saturn => Some(&SATURN_RULES),
        _ => None,
    };
    match table {
        Some(rules) => {
            let (indication, domain) = rules[s];
            vec![NadiRule {
                planet,
                sign: s,
                // Empty-API honesty: an unbundled (empty) reading becomes
                // `None`, never `Some("")`. A real reading (non-empty) surfaces
                // as `Some(text)`.
                indication: indication_text(indication),
                domain,
            }]
        }
        None => vec![],
    }
}

/// Map a raw indication string to `Some(text)` when non-empty, `None` when
/// empty. Centralises the empty-API honesty contract so every construction path
/// (current and future) reports an absent reading identically.
#[inline]
fn indication_text(raw: &'static str) -> Option<&'static str> {
    if raw.is_empty() { None } else { Some(raw) }
}

/// Total number of foundational rule slots in this module (interpretive text
/// not bundled — see module docs).
pub const TOTAL_RULES: usize = 48;

/// Returns all 48 foundational Nadi rule slots (planet/sign/domain scaffold;
/// `indication` text intentionally empty — see module docs).
pub fn all_rules() -> Vec<NadiRule> {
    let mut rules = Vec::with_capacity(TOTAL_RULES);
    for planet in [Planet::Sun, Planet::Moon, Planet::Jupiter, Planet::Saturn] {
        for sign in 0..12 {
            rules.extend(nadi_indications(planet, sign));
        }
    }
    rules
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sun_aries_is_career() {
        let rules = nadi_indications(Planet::Sun, 0);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].domain, NadiDomain::Career);
        assert_eq!(rules[0].planet, Planet::Sun);
        assert_eq!(rules[0].sign, 0);
    }

    #[test]
    fn unbundled_indication_is_none_not_empty_string() {
        // Empty-API honesty: the BNN readings are not bundled, so every slot's
        // `indication` must be `None` — never `Some("")`, which would falsely
        // look like (empty) data.
        for rule in all_rules() {
            assert_eq!(
                rule.indication, None,
                "{:?} sign {} should report None (unbundled), got {:?}",
                rule.planet, rule.sign, rule.indication
            );
        }
    }

    #[test]
    fn indication_text_maps_empty_to_none() {
        assert_eq!(indication_text(""), None);
        assert_eq!(
            indication_text("Career rise after 32"),
            Some("Career rise after 32")
        );
    }

    #[test]
    fn moon_taurus_exalted() {
        let rules = nadi_indications(Planet::Moon, 1);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].domain, NadiDomain::Relationship);
        assert_eq!(rules[0].sign, 1);
    }

    #[test]
    fn jupiter_cancer_exalted() {
        let rules = nadi_indications(Planet::Jupiter, 3);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].domain, NadiDomain::Finance);
        assert_eq!(rules[0].sign, 3);
    }

    #[test]
    fn saturn_libra_exalted() {
        let rules = nadi_indications(Planet::Saturn, 6);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].domain, NadiDomain::Career);
        assert_eq!(rules[0].sign, 6);
    }

    #[test]
    fn unsupported_planet_returns_empty() {
        let rules = nadi_indications(Planet::Mars, 0);
        assert!(rules.is_empty());
        let rules = nadi_indications(Planet::Rahu, 5);
        assert!(rules.is_empty());
    }

    #[test]
    fn sign_wraps_around() {
        let r1 = nadi_indications(Planet::Sun, 0);
        let r2 = nadi_indications(Planet::Sun, 12);
        assert_eq!(r1[0].sign, r2[0].sign);
        assert_eq!(r1[0].sign, 0);
        assert_eq!(r1[0].domain, r2[0].domain);
    }

    #[test]
    fn all_rules_count() {
        let rules = all_rules();
        assert_eq!(rules.len(), TOTAL_RULES);
    }

    #[test]
    fn all_rules_have_valid_structure() {
        for rule in all_rules() {
            assert!(
                rule.sign < 12,
                "Rule for {:?} has out-of-range sign {}",
                rule.planet,
                rule.sign
            );
        }
    }

    #[test]
    fn each_sign_covered_for_supported_planets() {
        for planet in [Planet::Sun, Planet::Moon, Planet::Jupiter, Planet::Saturn] {
            for sign in 0..12 {
                let rules = nadi_indications(planet, sign);
                assert_eq!(
                    rules.len(),
                    1,
                    "{:?} in sign {} should have exactly 1 rule",
                    planet,
                    sign
                );
            }
        }
    }

    #[test]
    fn domain_display() {
        assert_eq!(format!("{}", NadiDomain::Career), "Career");
        assert_eq!(format!("{}", NadiDomain::Spiritual), "Spiritual");
    }
}
