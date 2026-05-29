//! Bhrigu Nandi Nadi (BNN) predictive rules.
//!
//! Nadi astrology uses planet-in-sign combinations to derive predictions.
//! Each planet in a given sign carries specific indications about career,
//! relationships, health, finance, and spiritual life.
//!
//! This module provides 48 foundational rules (4 planets x 12 signs) covering
//! Sun, Moon, Jupiter, and Saturn -- the four karmic pillars.

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
    /// Concise indication text.
    pub indication: &'static str,
    /// Life domain this rule addresses.
    pub domain: NadiDomain,
}

// ---------------------------------------------------------------------------
// 48 foundational rules: Sun, Moon, Jupiter, Saturn x 12 signs
// ---------------------------------------------------------------------------

static SUN_RULES: [(&str, NadiDomain); 12] = [
    // Aries
    (
        "Leadership roles, government favor, father is influential",
        NadiDomain::Career,
    ),
    // Taurus
    (
        "Wealth through authority, inheritance from father",
        NadiDomain::Finance,
    ),
    // Gemini
    (
        "Intellectual career, writing, media, dual income sources",
        NadiDomain::Career,
    ),
    // Cancer
    (
        "Government service near water, emotional bond with father",
        NadiDomain::Career,
    ),
    // Leo
    (
        "Strong ego, political power, father figure dominant",
        NadiDomain::Career,
    ),
    // Virgo
    (
        "Analytical mind in career, health-sector work, modest father",
        NadiDomain::Career,
    ),
    // Libra (debilitated)
    (
        "Partnership struggles, father faces setbacks, diplomacy needed",
        NadiDomain::Relationship,
    ),
    // Scorpio
    (
        "Occult interests, research career, secretive father",
        NadiDomain::Career,
    ),
    // Sagittarius
    (
        "Dharmic career, teaching, father is philosophical",
        NadiDomain::Spiritual,
    ),
    // Capricorn
    (
        "Slow rise to authority, father is disciplined, government work",
        NadiDomain::Career,
    ),
    // Aquarius
    (
        "Unconventional career, social reform, distant father",
        NadiDomain::Career,
    ),
    // Pisces
    (
        "Spiritual vocation, father drawn to mysticism, foreign service",
        NadiDomain::Spiritual,
    ),
];

static MOON_RULES: [(&str, NadiDomain); 12] = [
    // Aries
    (
        "Quick mind, impulsive emotions, mother is independent",
        NadiDomain::Health,
    ),
    // Taurus (exalted)
    (
        "Peaceful mind, love of comfort, strong mother bond",
        NadiDomain::Relationship,
    ),
    // Gemini
    (
        "Restless mind, versatile thinking, mother is communicative",
        NadiDomain::Health,
    ),
    // Cancer (own sign)
    (
        "Deep emotions, nurturing nature, very close to mother",
        NadiDomain::Relationship,
    ),
    // Leo
    (
        "Proud mind, creative disposition, mother has regal bearing",
        NadiDomain::Relationship,
    ),
    // Virgo
    (
        "Analytical mind, worry-prone, mother focused on health",
        NadiDomain::Health,
    ),
    // Libra
    (
        "Balanced temperament, aesthetic sense, mother values harmony",
        NadiDomain::Relationship,
    ),
    // Scorpio (debilitated)
    (
        "Intense emotions, suspicion, mother faces hardships",
        NadiDomain::Health,
    ),
    // Sagittarius
    (
        "Optimistic mind, philosophical outlook, mother is religious",
        NadiDomain::Spiritual,
    ),
    // Capricorn
    (
        "Reserved emotions, practical mind, mother is hardworking",
        NadiDomain::Health,
    ),
    // Aquarius
    (
        "Detached mind, humanitarian thinking, mother is progressive",
        NadiDomain::Health,
    ),
    // Pisces
    (
        "Intuitive, dreamy disposition, mother is spiritual",
        NadiDomain::Spiritual,
    ),
];

static JUPITER_RULES: [(&str, NadiDomain); 12] = [
    // Aries
    (
        "Bold wisdom, early prosperity, self-made wealth",
        NadiDomain::Finance,
    ),
    // Taurus
    (
        "Steady wealth accumulation, banking, traditional values",
        NadiDomain::Finance,
    ),
    // Gemini
    (
        "Wealth through intellect, teaching, publishing",
        NadiDomain::Finance,
    ),
    // Cancer (exalted)
    (
        "Great fortune, spiritual wisdom, abundant prosperity",
        NadiDomain::Finance,
    ),
    // Leo
    (
        "Royal favor, generous nature, wealth through government",
        NadiDomain::Finance,
    ),
    // Virgo
    (
        "Analytical wisdom, moderate wealth, teaching profession",
        NadiDomain::Career,
    ),
    // Libra
    (
        "Wealth through partnerships, legal profession, fairness",
        NadiDomain::Finance,
    ),
    // Scorpio
    (
        "Occult wisdom, inheritance, deep research grants insight",
        NadiDomain::Spiritual,
    ),
    // Sagittarius (own sign)
    (
        "Natural philosopher, religious leader, wealth through dharma",
        NadiDomain::Spiritual,
    ),
    // Capricorn (debilitated)
    (
        "Delayed wisdom, struggles with faith, wealth comes late",
        NadiDomain::Finance,
    ),
    // Aquarius
    (
        "Unconventional wisdom, humanitarian wealth, group earnings",
        NadiDomain::Finance,
    ),
    // Pisces (own sign)
    (
        "Spiritual wealth, guru status, prosperity through devotion",
        NadiDomain::Spiritual,
    ),
];

static SATURN_RULES: [(&str, NadiDomain); 12] = [
    // Aries (debilitated)
    (
        "Impulsive karma, obstacles in early career, headaches",
        NadiDomain::Health,
    ),
    // Taurus
    (
        "Slow material gains, persistent effort, throat issues",
        NadiDomain::Finance,
    ),
    // Gemini
    (
        "Mental discipline through hardship, delayed communication success",
        NadiDomain::Career,
    ),
    // Cancer
    (
        "Emotional restrictions, mother faces difficulties, stomach issues",
        NadiDomain::Health,
    ),
    // Leo
    (
        "Authority through struggle, heart issues, conflict with father",
        NadiDomain::Career,
    ),
    // Virgo
    (
        "Methodical worker, health-related career, intestinal issues",
        NadiDomain::Health,
    ),
    // Libra (exalted)
    (
        "Justice through karma, excellent partnerships, balanced authority",
        NadiDomain::Career,
    ),
    // Scorpio
    (
        "Deep karmic debts, transformation through suffering, chronic issues",
        NadiDomain::Spiritual,
    ),
    // Sagittarius
    (
        "Disciplined philosophy, delayed spiritual progress, hip problems",
        NadiDomain::Spiritual,
    ),
    // Capricorn (own sign)
    (
        "Strong discipline, lasting authority, knee or bone issues",
        NadiDomain::Career,
    ),
    // Aquarius (own sign)
    (
        "Social service karma, technology career, circulatory issues",
        NadiDomain::Career,
    ),
    // Pisces
    (
        "Spiritual renunciation, isolation, feet-related health issues",
        NadiDomain::Spiritual,
    ),
];

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Look up Nadi indications for a given planet in a given sign.
///
/// Returns foundational rules for Sun, Moon, Jupiter, and Saturn.
/// Other planets return an empty vec (future expansion).
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
                indication,
                domain,
            }]
        }
        None => vec![],
    }
}

/// Total number of foundational rules in this module.
pub const TOTAL_RULES: usize = 48;

/// Returns all 48 foundational Nadi rules.
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
        assert!(rules[0].indication.contains("Leadership"));
    }

    #[test]
    fn moon_taurus_exalted() {
        let rules = nadi_indications(Planet::Moon, 1);
        assert_eq!(rules.len(), 1);
        assert!(rules[0].indication.contains("Peaceful"));
    }

    #[test]
    fn jupiter_cancer_exalted() {
        let rules = nadi_indications(Planet::Jupiter, 3);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].domain, NadiDomain::Finance);
        assert!(rules[0].indication.contains("fortune"));
    }

    #[test]
    fn saturn_libra_exalted() {
        let rules = nadi_indications(Planet::Saturn, 6);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].domain, NadiDomain::Career);
        assert!(rules[0].indication.contains("Justice"));
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
        assert_eq!(r1[0].indication, r2[0].indication);
    }

    #[test]
    fn all_rules_count() {
        let rules = all_rules();
        assert_eq!(rules.len(), TOTAL_RULES);
    }

    #[test]
    fn all_rules_have_nonempty_indication() {
        for rule in all_rules() {
            assert!(
                !rule.indication.is_empty(),
                "Rule for {:?} in sign {} has empty indication",
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
