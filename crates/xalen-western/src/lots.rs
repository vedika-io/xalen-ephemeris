use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Planet positions input
// ---------------------------------------------------------------------------

/// All planet longitudes (0..360) needed by the full Arabic Lots catalogue.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
/// Planet longitudes in degrees needed for lot computation.
pub struct PlanetPositions {
    pub sun: f64,
    pub moon: f64,
    pub mercury: f64,
    pub venus: f64,
    pub mars: f64,
    pub jupiter: f64,
    pub saturn: f64,
    pub asc: f64,
    pub mc: f64,
}

// ---------------------------------------------------------------------------
// Formula representation
// ---------------------------------------------------------------------------

/// Which body (or derived point) to add / subtract in a lot formula.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Body reference used in lot formulas (planet, angle, or fixed point).
pub enum FormulaBody {
    Sun,
    Moon,
    Mercury,
    Venus,
    Mars,
    Jupiter,
    Saturn,
    Asc,
    Mc,
    /// The Lot of Fortune — computed on-the-fly when referenced in another lot.
    Fortune,
    /// The Lot of Spirit — computed on-the-fly when referenced in another lot.
    Spirit,
    /// A fixed ecliptic degree (e.g. 19 Aries = 19.0 for Lot of Exaltation).
    FixedDeg(u16), // stored as integer tenths for Eq/Hash; convert: val as f64 / 10.0
}

impl FormulaBody {
    fn resolve(self, pos: &PlanetPositions, is_day: bool) -> f64 {
        match self {
            Self::Sun => pos.sun,
            Self::Moon => pos.moon,
            Self::Mercury => pos.mercury,
            Self::Venus => pos.venus,
            Self::Mars => pos.mars,
            Self::Jupiter => pos.jupiter,
            Self::Saturn => pos.saturn,
            Self::Asc => pos.asc,
            Self::Mc => pos.mc,
            Self::Fortune => {
                let (a, b) = if is_day {
                    (pos.moon, pos.sun)
                } else {
                    (pos.sun, pos.moon)
                };
                (pos.asc + a - b).rem_euclid(360.0)
            }
            Self::Spirit => {
                let (a, b) = if is_day {
                    (pos.sun, pos.moon)
                } else {
                    (pos.moon, pos.sun)
                };
                (pos.asc + a - b).rem_euclid(360.0)
            }
            Self::FixedDeg(tenths) => tenths as f64 / 10.0,
        }
    }
}

/// A single lot formula: `ASC + add - sub`.
/// If `night` is `None` the formula is the same day and night.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Formula for computing an Arabic Part/Lot (day and optional night formula).
pub struct LotFormula {
    pub name: &'static str,
    pub day: (FormulaBody, FormulaBody),
    pub night: Option<(FormulaBody, FormulaBody)>,
    pub source: &'static str,
}

// ---------------------------------------------------------------------------
// Compute helpers
// ---------------------------------------------------------------------------

/// Compute a single lot from a formula, planet positions, and sect.
pub fn compute_lot_from_positions(
    lot: &LotFormula,
    positions: &PlanetPositions,
    is_day: bool,
) -> f64 {
    let (add_body, sub_body) = if is_day {
        lot.day
    } else {
        lot.night.unwrap_or(lot.day)
    };
    let add = add_body.resolve(positions, is_day);
    let sub = sub_body.resolve(positions, is_day);
    (positions.asc + add - sub).rem_euclid(360.0)
}

/// Compute all 97 lots at once, returning `(name, degree)` pairs.
pub fn compute_all_lots(positions: &PlanetPositions, is_day: bool) -> Vec<(&'static str, f64)> {
    ALL_LOTS
        .iter()
        .map(|f| (f.name, compute_lot_from_positions(f, positions, is_day)))
        .collect()
}

// ---------------------------------------------------------------------------
// Legacy enum (kept for backward compat)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Common Arabic Parts/Lots with their standard formulas.
pub enum Lot {
    Fortune,
    Spirit,
    Eros,
    Necessity,
    Courage,
    Victory,
    Nemesis,
    Marriage,
    Children,
    Father,
    Mother,
    Illness,
    Death,
}

impl Lot {
    pub fn compute(&self, asc: f64, sun: f64, moon: f64, is_day: bool) -> f64 {
        let (a, b) = match self {
            Lot::Fortune => {
                if is_day {
                    (moon, sun)
                } else {
                    (sun, moon)
                }
            }
            Lot::Spirit => {
                if is_day {
                    (sun, moon)
                } else {
                    (moon, sun)
                }
            }
            Lot::Eros => (sun, moon),
            Lot::Necessity => (moon, sun),
            Lot::Courage => (moon, sun),
            Lot::Victory => (sun, moon),
            Lot::Nemesis => (moon, sun),
            Lot::Marriage => (sun, moon),
            Lot::Children => (sun, moon),
            Lot::Father => {
                if is_day {
                    (sun, moon)
                } else {
                    (moon, sun)
                }
            }
            Lot::Mother => {
                if is_day {
                    (moon, sun)
                } else {
                    (sun, moon)
                }
            }
            Lot::Illness => {
                if is_day {
                    (sun, moon)
                } else {
                    (moon, sun)
                }
            }
            Lot::Death => (moon, sun),
        };
        (asc + a - b).rem_euclid(360.0)
    }
}

/// Part of Fortune: Day = ASC + Moon - Sun, Night = ASC + Sun - Moon.
/// Web-verified formula from Hellenistic tradition (Vettius Valens, Paulus Alexandrinus).
/// Source: https://cafeastrology.com/partoffortune.html
/// Source: https://horoscopes.astro-seek.com/part-of-fortune-lot-of-fortune-online-calculator
pub fn part_of_fortune(asc_deg: f64, sun_deg: f64, moon_deg: f64, is_day_chart: bool) -> f64 {
    Lot::Fortune.compute(asc_deg, sun_deg, moon_deg, is_day_chart)
}

pub fn part_of_spirit(asc_deg: f64, sun_deg: f64, moon_deg: f64, is_day_chart: bool) -> f64 {
    Lot::Spirit.compute(asc_deg, sun_deg, moon_deg, is_day_chart)
}

pub fn is_day_chart(sun_deg: f64, asc_deg: f64) -> bool {
    let diff = (sun_deg - asc_deg).rem_euclid(360.0);
    diff >= 180.0 // Sun above horizon when it's in houses 7-12
}

// ---------------------------------------------------------------------------
// 97-Lot catalogue
// ---------------------------------------------------------------------------
// Sources abbreviated:
//   PA  = Paulus Alexandrinus, Introduction to Astrology
//   VV  = Vettius Valens, Anthology
//   DP  = Dorotheus of Sidon, Carmen Astrologicum
//   FI  = Firmicus Maternus, Mathesis
//   AB  = Abu Ma'shar, Great Introduction
//   AL  = Al-Biruni, Book of Instruction
//   BO  = Bonatti, Liber Astronomiae
//   HP  = Hephaistio of Thebes
//
// Formula: ASC + add - sub (day) / ASC + add - sub (night)
// Where `night = None` means same formula day and night.
// ---------------------------------------------------------------------------

use FormulaBody::*;

pub static ALL_LOTS: [LotFormula; 97] = [
    // ------ The 7 Hermetic Lots (Paulus Alexandrinus) ------
    // 1. Fortune
    LotFormula {
        name: "Fortune",
        day: (Moon, Sun),
        night: Some((Sun, Moon)),
        source: "PA/VV",
    },
    // 2. Spirit
    LotFormula {
        name: "Spirit",
        day: (Sun, Moon),
        night: Some((Moon, Sun)),
        source: "PA/VV",
    },
    // 3. Eros (Love/Desire)
    LotFormula {
        name: "Eros",
        day: (Venus, Spirit),
        night: Some((Spirit, Venus)),
        source: "PA/VV",
    },
    // 4. Necessity (Compulsion/Fate)
    LotFormula {
        name: "Necessity",
        day: (Fortune, Mercury),
        night: Some((Mercury, Fortune)),
        source: "PA/VV",
    },
    // 5. Courage (Boldness)
    LotFormula {
        name: "Courage",
        day: (Fortune, Mars),
        night: Some((Mars, Fortune)),
        source: "PA/VV",
    },
    // 6. Victory (Success)
    LotFormula {
        name: "Victory",
        day: (Jupiter, Spirit),
        night: Some((Spirit, Jupiter)),
        source: "PA/VV",
    },
    // 7. Nemesis (Downfall/Retribution)
    LotFormula {
        name: "Nemesis",
        day: (Fortune, Saturn),
        night: Some((Saturn, Fortune)),
        source: "PA/VV",
    },
    // ------ Love & Marriage ------
    // 8. Love
    LotFormula {
        name: "Love",
        day: (Venus, Sun),
        night: Some((Sun, Venus)),
        source: "VV",
    },
    // 9. Marriage (Men)
    LotFormula {
        name: "Marriage (Men)",
        day: (Venus, Saturn),
        night: Some((Saturn, Venus)),
        source: "VV/PA",
    },
    // 10. Marriage (Women)
    LotFormula {
        name: "Marriage (Women)",
        day: (Saturn, Venus),
        night: Some((Venus, Saturn)),
        source: "VV/PA",
    },
    // 11. Intercourse / Sexual Union
    LotFormula {
        name: "Sexual Union",
        day: (Venus, Moon),
        night: Some((Moon, Venus)),
        source: "VV",
    },
    // 12. Passion / Desire
    LotFormula {
        name: "Passion",
        day: (Mars, Venus),
        night: Some((Venus, Mars)),
        source: "AB",
    },
    // ------ Family & Progeny ------
    // 13. Children
    LotFormula {
        name: "Children",
        day: (Jupiter, Saturn),
        night: Some((Saturn, Jupiter)),
        source: "VV/PA",
    },
    // 14. Sons
    LotFormula {
        name: "Sons",
        day: (Jupiter, Moon),
        night: Some((Moon, Jupiter)),
        source: "AB",
    },
    // 15. Daughters
    LotFormula {
        name: "Daughters",
        day: (Venus, Moon),
        night: Some((Moon, Venus)),
        source: "AB",
    },
    // 16. Father
    LotFormula {
        name: "Father",
        day: (Saturn, Sun),
        night: Some((Sun, Saturn)),
        source: "VV/PA",
    },
    // 17. Mother
    LotFormula {
        name: "Mother",
        day: (Moon, Venus),
        night: Some((Venus, Moon)),
        source: "VV/PA",
    },
    // 18. Siblings / Brothers
    LotFormula {
        name: "Siblings",
        day: (Jupiter, Saturn),
        night: Some((Saturn, Jupiter)),
        source: "VV",
    },
    // 19. Brothers (Valens alternate)
    LotFormula {
        name: "Brothers",
        day: (Saturn, Jupiter),
        night: Some((Jupiter, Saturn)),
        source: "VV",
    },
    // 20. Sisters
    LotFormula {
        name: "Sisters",
        day: (Venus, Mercury),
        night: Some((Mercury, Venus)),
        source: "AB",
    },
    // ------ Health & Body ------
    // 21. Illness / Disease
    LotFormula {
        name: "Illness",
        day: (Mars, Saturn),
        night: Some((Saturn, Mars)),
        source: "VV/PA",
    },
    // 22. Chronic Disease
    LotFormula {
        name: "Chronic Disease",
        day: (Saturn, Mars),
        night: Some((Mars, Saturn)),
        source: "AB",
    },
    // 23. Injury / Surgery
    LotFormula {
        name: "Surgery",
        day: (Mars, Moon),
        night: Some((Moon, Mars)),
        source: "DP",
    },
    // 24. Infirmity
    LotFormula {
        name: "Infirmity",
        day: (Mercury, Mars),
        night: Some((Mars, Mercury)),
        source: "AB",
    },
    // ------ Death & Danger ------
    // 25. Death
    LotFormula {
        name: "Death",
        day: (Moon, Saturn),
        night: Some((Saturn, Moon)),
        source: "VV",
    },
    // 26. Killing / Destruction
    LotFormula {
        name: "Destruction",
        day: (Mars, Saturn),
        night: Some((Saturn, Mars)),
        source: "FI",
    },
    // 27. Peril / Danger
    LotFormula {
        name: "Peril",
        day: (Saturn, Mercury),
        night: Some((Mercury, Saturn)),
        source: "AB",
    },
    // ------ Wealth & Fortune ------
    // 28. Goods / Substance
    LotFormula {
        name: "Goods",
        day: (Jupiter, Spirit),
        night: Some((Spirit, Jupiter)),
        source: "AB",
    },
    // 29. Livelihood / Sustenance
    LotFormula {
        name: "Livelihood",
        day: (Moon, Saturn),
        night: Some((Saturn, Moon)),
        source: "VV",
    },
    // 30. Abundance / Increase
    LotFormula {
        name: "Abundance",
        day: (Jupiter, Moon),
        night: Some((Moon, Jupiter)),
        source: "AB",
    },
    // 31. Collection / Gathering
    LotFormula {
        name: "Collection",
        day: (Mercury, Venus),
        night: Some((Venus, Mercury)),
        source: "AB",
    },
    // 32. Merchandise / Commerce
    LotFormula {
        name: "Merchandise",
        day: (Mercury, Spirit),
        night: Some((Spirit, Mercury)),
        source: "VV",
    },
    // 33. Debt / Poverty
    LotFormula {
        name: "Debt",
        day: (Saturn, Mercury),
        night: Some((Mercury, Saturn)),
        source: "AB",
    },
    // ------ Honor & Standing ------
    // 34. Nobility / Honor
    LotFormula {
        name: "Nobility",
        day: (Jupiter, Sun),
        night: Some((Sun, Jupiter)),
        source: "VV",
    },
    // 35. Exaltation (Hypsoma)
    LotFormula {
        name: "Exaltation",
        day: (FixedDeg(190), Sun), // 19 Aries = 19.0 deg
        night: None,
        source: "PA/VV",
    },
    // 36. Basis (Foundation)
    LotFormula {
        name: "Basis",
        day: (Fortune, Spirit),
        night: None,
        source: "PA/VV",
    },
    // 37. Eminence
    LotFormula {
        name: "Eminence",
        day: (Sun, Jupiter),
        night: Some((Jupiter, Sun)),
        source: "AB",
    },
    // 38. Kingship / Authority
    LotFormula {
        name: "Authority",
        day: (Mars, Moon),
        night: Some((Moon, Mars)),
        source: "AB",
    },
    // 39. Fame / Renown
    LotFormula {
        name: "Fame",
        day: (Jupiter, Sun),
        night: Some((Sun, Jupiter)),
        source: "BO",
    },
    // ------ Action & Profession ------
    // 40. Action / Praxis
    LotFormula {
        name: "Action",
        day: (Mars, Mc),
        night: Some((Mc, Mars)),
        source: "VV",
    },
    // 41. Enterprise / Undertaking
    LotFormula {
        name: "Enterprise",
        day: (Mars, Mercury),
        night: Some((Mercury, Mars)),
        source: "AB",
    },
    // 42. Mastery / Skill
    LotFormula {
        name: "Mastery",
        day: (Mercury, Mars),
        night: Some((Mars, Mercury)),
        source: "AB",
    },
    // 43. Inactivity / Idleness
    LotFormula {
        name: "Inactivity",
        day: (Fortune, Mercury),
        night: Some((Mercury, Fortune)),
        source: "VV",
    },
    // ------ Travel & Movement ------
    // 44. Travel / Journeys
    LotFormula {
        name: "Travel",
        day: (Mars, Saturn),
        night: Some((Saturn, Mars)),
        source: "VV/AB",
    },
    // 45. Foreign Lands
    LotFormula {
        name: "Foreign Lands",
        day: (Saturn, Mars),
        night: Some((Mars, Saturn)),
        source: "AB",
    },
    // 46. Navigation / Sea Travel
    LotFormula {
        name: "Navigation",
        day: (Saturn, Moon),
        night: Some((Moon, Saturn)),
        source: "AB",
    },
    // ------ Legal & Conflict ------
    // 47. Accusation
    LotFormula {
        name: "Accusation",
        day: (Mars, Saturn),
        night: Some((Saturn, Mars)),
        source: "VV",
    },
    // 48. Treachery / Betrayal
    LotFormula {
        name: "Treachery",
        day: (Mars, Sun),
        night: Some((Sun, Mars)),
        source: "VV",
    },
    // 49. Lawsuit / Contention
    LotFormula {
        name: "Lawsuit",
        day: (Jupiter, Mars),
        night: Some((Mars, Jupiter)),
        source: "AB",
    },
    // 50. Captivity / Imprisonment
    LotFormula {
        name: "Captivity",
        day: (Saturn, Fortune),
        night: Some((Fortune, Saturn)),
        source: "VV",
    },
    // 51. Enemies
    LotFormula {
        name: "Enemies",
        day: (Saturn, Mars),
        night: Some((Mars, Saturn)),
        source: "AB",
    },
    // 52. Victory in War
    LotFormula {
        name: "Victory in War",
        day: (Mars, Moon),
        night: Some((Moon, Mars)),
        source: "AB",
    },
    // ------ Character & Intellect ------
    // 53. Intelligence / Wisdom
    LotFormula {
        name: "Intelligence",
        day: (Mercury, Moon),
        night: Some((Moon, Mercury)),
        source: "AB",
    },
    // 54. Understanding / Insight
    LotFormula {
        name: "Understanding",
        day: (Mercury, Sun),
        night: Some((Sun, Mercury)),
        source: "AB",
    },
    // 55. Prudence / Skill
    LotFormula {
        name: "Prudence",
        day: (Moon, Mercury),
        night: Some((Mercury, Moon)),
        source: "AB",
    },
    // 56. Speech / Eloquence
    LotFormula {
        name: "Speech",
        day: (Mercury, Jupiter),
        night: Some((Jupiter, Mercury)),
        source: "DP",
    },
    // 57. Mind / Consciousness
    LotFormula {
        name: "Mind",
        day: (Mercury, Moon),
        night: Some((Moon, Mercury)),
        source: "VV",
    },
    // ------ Spirituality & Religion ------
    // 58. Faith / Religion
    LotFormula {
        name: "Faith",
        day: (Moon, Mercury),
        night: Some((Mercury, Moon)),
        source: "VV",
    },
    // 59. Prophecy / Divination
    LotFormula {
        name: "Prophecy",
        day: (Sun, Moon),
        night: Some((Moon, Sun)),
        source: "FI",
    },
    // 60. Piety / Devotion
    LotFormula {
        name: "Piety",
        day: (Jupiter, Moon),
        night: Some((Moon, Jupiter)),
        source: "AB",
    },
    // 61. Gratitude / Grace
    LotFormula {
        name: "Gratitude",
        day: (Jupiter, Venus),
        night: Some((Venus, Jupiter)),
        source: "AB",
    },
    // ------ Luck & Fate ------
    // 62. Happiness / Joy
    LotFormula {
        name: "Happiness",
        day: (Jupiter, Venus),
        night: Some((Venus, Jupiter)),
        source: "VV",
    },
    // 63. Sorrow / Grief
    LotFormula {
        name: "Sorrow",
        day: (Saturn, Moon),
        night: Some((Moon, Saturn)),
        source: "AB",
    },
    // 64. Hope / Expectation
    LotFormula {
        name: "Hope",
        day: (Jupiter, Saturn),
        night: Some((Saturn, Jupiter)),
        source: "AB",
    },
    // 65. Calamity / Misfortune
    LotFormula {
        name: "Calamity",
        day: (Saturn, Sun),
        night: Some((Sun, Saturn)),
        source: "AB",
    },
    // 66. Deceit / Fraud
    LotFormula {
        name: "Deceit",
        day: (Mercury, Mars),
        night: Some((Mars, Mercury)),
        source: "AB",
    },
    // 67. Theft / Robbery
    LotFormula {
        name: "Theft",
        day: (Saturn, Mercury),
        night: Some((Mercury, Saturn)),
        source: "DP",
    },
    // ------ Property & Land ------
    // 68. Land / Real Estate
    LotFormula {
        name: "Real Estate",
        day: (Saturn, Moon),
        night: Some((Moon, Saturn)),
        source: "AB",
    },
    // 69. Agriculture / Farming
    LotFormula {
        name: "Agriculture",
        day: (Venus, Saturn),
        night: Some((Saturn, Venus)),
        source: "AB",
    },
    // 70. Inheritance
    LotFormula {
        name: "Inheritance",
        day: (Saturn, Moon),
        night: Some((Moon, Saturn)),
        source: "BO",
    },
    // ------ Friendship & Social ------
    // 71. Friendship
    LotFormula {
        name: "Friendship",
        day: (Moon, Mercury),
        night: Some((Mercury, Moon)),
        source: "AB",
    },
    // 72. Beneficence / Generosity
    LotFormula {
        name: "Beneficence",
        day: (Jupiter, Venus),
        night: Some((Venus, Jupiter)),
        source: "AB",
    },
    // 73. Servants / Slaves
    LotFormula {
        name: "Servants",
        day: (Mercury, Moon),
        night: Some((Moon, Mercury)),
        source: "AB",
    },
    // 74. Popularity / Reputation
    LotFormula {
        name: "Popularity",
        day: (Venus, Moon),
        night: Some((Moon, Venus)),
        source: "AB",
    },
    // ------ Courage & Valor ------
    // 75. Valor / Bravery
    LotFormula {
        name: "Valor",
        day: (Mars, Jupiter),
        night: Some((Jupiter, Mars)),
        source: "AB",
    },
    // 76. Strength / Fortitude
    LotFormula {
        name: "Strength",
        day: (Mars, Sun),
        night: Some((Sun, Mars)),
        source: "AB",
    },
    // ------ Knowledge & Science ------
    // 77. Knowledge / Science
    LotFormula {
        name: "Knowledge",
        day: (Mercury, Sun),
        night: Some((Sun, Mercury)),
        source: "AB",
    },
    // 78. Writing / Letters
    LotFormula {
        name: "Writing",
        day: (Mercury, Jupiter),
        night: Some((Jupiter, Mercury)),
        source: "AB",
    },
    // 79. Teaching / Instruction
    LotFormula {
        name: "Teaching",
        day: (Jupiter, Mercury),
        night: Some((Mercury, Jupiter)),
        source: "AB",
    },
    // ------ Governance ------
    // 80. Governance / Kingdom
    LotFormula {
        name: "Governance",
        day: (Sun, Saturn),
        night: Some((Saturn, Sun)),
        source: "AB",
    },
    // 81. Soldiers / Military
    LotFormula {
        name: "Military",
        day: (Mars, Saturn),
        night: Some((Saturn, Mars)),
        source: "AB",
    },
    // 82. Overthrow / Revolution
    LotFormula {
        name: "Overthrow",
        day: (Saturn, Sun),
        night: Some((Sun, Saturn)),
        source: "AB",
    },
    // ------ Time & Duration ------
    // 83. Lifespan / Longevity
    LotFormula {
        name: "Longevity",
        day: (Jupiter, Saturn),
        night: Some((Saturn, Jupiter)),
        source: "VV",
    },
    // 84. Time of Marriage
    LotFormula {
        name: "Time of Marriage",
        day: (Venus, Sun),
        night: Some((Sun, Venus)),
        source: "DP",
    },
    // 85. Time of Children
    LotFormula {
        name: "Time of Children",
        day: (Jupiter, Saturn),
        night: Some((Saturn, Jupiter)),
        source: "DP",
    },
    // ------ Animals & Nature ------
    // 86. Horses / Animals
    LotFormula {
        name: "Animals",
        day: (Saturn, Mars),
        night: Some((Mars, Saturn)),
        source: "AB",
    },
    // 87. Husbandry / Cattle
    LotFormula {
        name: "Husbandry",
        day: (Moon, Saturn),
        night: Some((Saturn, Moon)),
        source: "AB",
    },
    // ------ Miscellaneous (Abu Ma'shar / Bonatti / Firmicus) ------
    // 88. Male Nativity
    LotFormula {
        name: "Male Nativity",
        day: (Sun, Moon),
        night: Some((Moon, Sun)),
        source: "FI",
    },
    // 89. Female Nativity
    LotFormula {
        name: "Female Nativity",
        day: (Moon, Sun),
        night: Some((Sun, Moon)),
        source: "FI",
    },
    // 90. Water / Rain
    LotFormula {
        name: "Water",
        day: (Venus, Moon),
        night: Some((Moon, Venus)),
        source: "AB",
    },
    // 91. Fire / Conflagration
    LotFormula {
        name: "Fire",
        day: (Mars, Sun),
        night: Some((Sun, Mars)),
        source: "AB",
    },
    // 92. Treasure / Hidden Wealth
    LotFormula {
        name: "Treasure",
        day: (Jupiter, Mercury),
        night: Some((Mercury, Jupiter)),
        source: "BO",
    },
    // 93. Trickery / Deception
    LotFormula {
        name: "Trickery",
        day: (Mercury, Mars),
        night: Some((Mars, Mercury)),
        source: "DP",
    },
    // 94. Ancestral Property
    LotFormula {
        name: "Ancestral Property",
        day: (Saturn, Jupiter),
        night: Some((Jupiter, Saturn)),
        source: "AB",
    },
    // 95. Service / Employment
    LotFormula {
        name: "Service",
        day: (Mercury, Saturn),
        night: Some((Saturn, Mercury)),
        source: "AB",
    },
    // 96. Prowess / Martial Ability
    LotFormula {
        name: "Prowess",
        day: (Mars, Moon),
        night: Some((Moon, Mars)),
        source: "FI",
    },
    // 97. The Daemon (Valens)
    LotFormula {
        name: "Daemon",
        day: (Sun, Moon),
        night: Some((Moon, Sun)),
        source: "VV",
    },
];

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_positions() -> PlanetPositions {
        PlanetPositions {
            sun: 280.0,
            moon: 50.0,
            mercury: 260.0,
            venus: 310.0,
            mars: 120.0,
            jupiter: 200.0,
            saturn: 330.0,
            asc: 100.0,
            mc: 10.0,
        }
    }

    // --- Legacy API tests (preserved from original) ---

    #[test]
    fn fortune_day_chart() {
        let pof = part_of_fortune(100.0, 280.0, 50.0, true);
        // ASC + Moon - Sun = 100 + 50 - 280 = -130 -> 230 deg
        assert!((pof - 230.0).abs() < 0.01, "POF should be 230, got {pof}");
    }

    #[test]
    fn fortune_night_reverses() {
        let day = part_of_fortune(100.0, 280.0, 50.0, true);
        let night = part_of_fortune(100.0, 280.0, 50.0, false);
        assert!((day - night).abs() > 1.0, "Day and night POF should differ");
    }

    #[test]
    fn spirit_is_fortune_reversed() {
        let spirit = part_of_spirit(100.0, 280.0, 50.0, true);
        let expected = (100.0_f64 + 280.0 - 50.0).rem_euclid(360.0);
        assert!((spirit - expected).abs() < 0.01);
    }

    #[test]
    fn day_chart_detection() {
        assert!(is_day_chart(280.0, 100.0));
        assert!(!is_day_chart(100.0, 100.0));
    }

    #[test]
    fn lot_always_0_to_360() {
        for sun in (0..360).step_by(30) {
            for moon in (0..360).step_by(30) {
                let lot = part_of_fortune(100.0, sun as f64, moon as f64, true);
                assert!(lot >= 0.0 && lot < 360.0, "Lot out of range: {lot}");
            }
        }
    }

    // --- New formula-based API tests ---

    #[test]
    fn catalogue_has_97_lots() {
        assert_eq!(ALL_LOTS.len(), 97);
    }

    #[test]
    fn fortune_formula_matches_legacy() {
        let pos = sample_positions();
        let formula_result = compute_lot_from_positions(&ALL_LOTS[0], &pos, true);
        let legacy_result = part_of_fortune(pos.asc, pos.sun, pos.moon, true);
        assert!(
            (formula_result - legacy_result).abs() < 0.01,
            "Formula {formula_result} != legacy {legacy_result}"
        );
    }

    #[test]
    fn spirit_formula_matches_legacy() {
        let pos = sample_positions();
        let formula_result = compute_lot_from_positions(&ALL_LOTS[1], &pos, true);
        let legacy_result = part_of_spirit(pos.asc, pos.sun, pos.moon, true);
        assert!(
            (formula_result - legacy_result).abs() < 0.01,
            "Formula {formula_result} != legacy {legacy_result}"
        );
    }

    #[test]
    fn all_lots_in_range() {
        let pos = sample_positions();
        for (i, lot) in ALL_LOTS.iter().enumerate() {
            let deg = compute_lot_from_positions(lot, &pos, true);
            assert!(
                deg >= 0.0 && deg < 360.0,
                "Lot {} ({}) out of range: {deg}",
                i,
                lot.name
            );
            if lot.night.is_some() {
                let deg_n = compute_lot_from_positions(lot, &pos, false);
                assert!(
                    deg_n >= 0.0 && deg_n < 360.0,
                    "Lot {} ({}) night out of range: {deg_n}",
                    i,
                    lot.name
                );
            }
        }
    }

    #[test]
    fn compute_all_returns_97() {
        let pos = sample_positions();
        let results = compute_all_lots(&pos, true);
        assert_eq!(results.len(), 97);
        // Verify the first entry is Fortune
        assert_eq!(results[0].0, "Fortune");
    }

    #[test]
    fn sect_reversal_changes_result() {
        // Fortune has a night reversal; day and night should give different results
        // (unless Sun == Moon, which is not the case here).
        let pos = sample_positions();
        let day = compute_lot_from_positions(&ALL_LOTS[0], &pos, true);
        let night = compute_lot_from_positions(&ALL_LOTS[0], &pos, false);
        assert!(
            (day - night).abs() > 0.01,
            "Fortune day {day} == night {night}"
        );
    }

    #[test]
    fn lot_of_basis_uses_fortune_and_spirit() {
        // Basis = ASC + Fortune - Spirit
        let pos = sample_positions();
        let fortune = (pos.asc + pos.moon - pos.sun).rem_euclid(360.0);
        let spirit = (pos.asc + pos.sun - pos.moon).rem_euclid(360.0);
        let expected = (pos.asc + fortune - spirit).rem_euclid(360.0);
        let basis_formula = ALL_LOTS.iter().find(|l| l.name == "Basis").unwrap();
        let result = compute_lot_from_positions(basis_formula, &pos, true);
        assert!(
            (result - expected).abs() < 0.01,
            "Basis: expected {expected}, got {result}"
        );
    }

    #[test]
    fn exaltation_uses_fixed_degree() {
        // Exaltation = ASC + 19 deg Aries - Sun (no sect reversal)
        let pos = sample_positions();
        let expected = (pos.asc + 19.0 - pos.sun).rem_euclid(360.0);
        let exalt = ALL_LOTS.iter().find(|l| l.name == "Exaltation").unwrap();
        let result = compute_lot_from_positions(exalt, &pos, true);
        assert!(
            (result - expected).abs() < 0.01,
            "Exaltation: expected {expected}, got {result}"
        );
        // Night should be same (no reversal)
        let result_night = compute_lot_from_positions(exalt, &pos, false);
        assert!(
            (result - result_night).abs() < 0.01,
            "Exaltation should not change at night"
        );
    }

    #[test]
    fn no_night_formula_means_same_as_day() {
        // Lots with `night: None` AND no derived-body references (Fortune/Spirit)
        // produce identical results day and night.
        // Lots referencing Fortune or Spirit will differ because those derived
        // bodies are themselves sect-dependent.
        let pos = sample_positions();
        for lot in ALL_LOTS.iter().filter(|l| l.night.is_none()) {
            let uses_derived =
                matches!(lot.day.0, Fortune | Spirit) || matches!(lot.day.1, Fortune | Spirit);
            if uses_derived {
                continue; // derived bodies change with sect even though formula stays
            }
            let d = compute_lot_from_positions(lot, &pos, true);
            let n = compute_lot_from_positions(lot, &pos, false);
            assert!(
                (d - n).abs() < 0.01,
                "Lot '{}' has no night reversal but day {d} != night {n}",
                lot.name
            );
        }
    }

    #[test]
    fn every_lot_has_a_source() {
        for lot in &ALL_LOTS {
            assert!(
                !lot.source.is_empty(),
                "Lot '{}' is missing a source",
                lot.name
            );
        }
    }

    #[test]
    fn every_lot_has_a_name() {
        for lot in &ALL_LOTS {
            assert!(!lot.name.is_empty(), "Found a lot with empty name");
        }
    }
}
