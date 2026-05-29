//! Prashna Shastra (Vedic Horary Astrology)
//!
//! Implements the Prasna Marga tradition of horary analysis.
//! The querent asks a question, and the chart of the moment of asking is analyzed.
//!
//! Key concepts:
//! - **Arudha Lagna**: number (1-12) thought of by the querent, maps to a sign
//! - **Mook Number**: 1-108, maps to nakshatra pada for event timing
//! - **Ashtamangala Prashna**: Kerala tradition using 8 auspicious items
//! - **Timing**: sign modality of Prashna Lagna determines speed of outcome

use serde::{Deserialize, Serialize};

use crate::nakshatra::Nakshatra;
use crate::rashi::{Modality, Rashi};

pub use xalen_coords::Planet;

// ---------------------------------------------------------------------------
// Core structs
// ---------------------------------------------------------------------------

/// Chart erected at the moment a question is asked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrashnaChart {
    /// Julian Day of the query moment.
    pub query_jd: f64,
    /// Sidereal longitude of the ascendant (degrees).
    pub asc_sidereal: f64,
    /// Sidereal longitude of the Moon (degrees).
    pub moon_sidereal: f64,
    /// Sidereal positions of all relevant planets.
    pub planet_positions: Vec<(Planet, f64)>,
}

/// Speed at which the queried event is expected to manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimingIndicator {
    /// Cardinal / Movable signs -- outcome arrives quickly.
    Quick,
    /// Fixed signs -- outcome is delayed.
    Delayed,
    /// Mutable / Dual signs -- medium timeframe.
    Medium,
}

/// Result of a full Prashna analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrashnaAnalysis {
    /// Sign number (1-12) chosen or derived for the querent.
    pub arudha_lagna: u8,
    /// 1-108, maps to a nakshatra pada for timing.
    pub mook_number: u8,
    /// Planet ruling the Prashna Lagna sign.
    pub prashna_lagna_lord: Planet,
    /// Nakshatra index (0-26) of the Moon at query time.
    pub moon_nakshatra: u8,
    /// Whether the query outcome is favorable based on Moon's house from Arudha.
    pub favorable: bool,
    /// Timing based on sign modality of the Prashna Lagna.
    pub timing_indicator: TimingIndicator,
}

// ---------------------------------------------------------------------------
// Ashtamangala (Kerala 8-item tradition)
// ---------------------------------------------------------------------------

/// The eight auspicious items used in the Ashtamangala Prashna of Kerala.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AshtamangalaItem {
    Gold,
    Mirror,
    Turmeric,
    Book,
    Betel,
    Fruit,
    Lamp,
    FullVessel,
}

impl AshtamangalaItem {
    pub const ALL: [AshtamangalaItem; 8] = [
        AshtamangalaItem::Gold,
        AshtamangalaItem::Mirror,
        AshtamangalaItem::Turmeric,
        AshtamangalaItem::Book,
        AshtamangalaItem::Betel,
        AshtamangalaItem::Fruit,
        AshtamangalaItem::Lamp,
        AshtamangalaItem::FullVessel,
    ];
}

impl std::fmt::Display for AshtamangalaItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            AshtamangalaItem::Gold => "Gold",
            AshtamangalaItem::Mirror => "Mirror",
            AshtamangalaItem::Turmeric => "Turmeric",
            AshtamangalaItem::Book => "Book",
            AshtamangalaItem::Betel => "Betel",
            AshtamangalaItem::Fruit => "Fruit",
            AshtamangalaItem::Lamp => "Lamp",
            AshtamangalaItem::FullVessel => "Full Vessel",
        };
        write!(f, "{name}")
    }
}

/// Interpretation of a chosen Ashtamangala item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AshtamangalaReading {
    pub item: AshtamangalaItem,
    pub ruling_planet: Planet,
    pub outcome_nature: &'static str,
    pub recommendation: &'static str,
}

// ---------------------------------------------------------------------------
// Sign lord mapping (for Prashna Lagna lord)
// ---------------------------------------------------------------------------

/// Returns the ruling planet of a sign given its 0-based index.
fn sign_lord(sign_index: usize) -> Planet {
    match sign_index % 12 {
        0 => Planet::Mars,    // Aries
        1 => Planet::Venus,   // Taurus
        2 => Planet::Mercury, // Gemini
        3 => Planet::Moon,    // Cancer
        4 => Planet::Sun,     // Leo
        5 => Planet::Mercury, // Virgo
        6 => Planet::Venus,   // Libra
        7 => Planet::Mars,    // Scorpio (traditional ruler)
        8 => Planet::Jupiter, // Sagittarius
        9 => Planet::Saturn,  // Capricorn
        10 => Planet::Saturn, // Aquarius
        _ => Planet::Jupiter, // Pisces
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Houses from Arudha Lagna where Moon gives favorable results.
const FAVORABLE_MOON_HOUSES: [u8; 8] = [1, 2, 3, 5, 7, 9, 10, 11];

/// Analyze a Prashna chart given the querent's arudha number (1-12).
///
/// The arudha is the number the querent thinks of or the sign derived from the
/// first syllable of the question. Moon's position relative to this sign
/// determines favorability, and the sign's modality determines timing.
pub fn analyze_prashna(chart: &PrashnaChart, arudha: u8) -> PrashnaAnalysis {
    let arudha_clamped = (arudha.clamp(1, 12) - 1) as usize; // 0-based sign index

    // Prashna Lagna lord
    let prashna_lagna_lord = sign_lord(arudha_clamped);

    // Moon's nakshatra
    let moon_nak = Nakshatra::from_longitude_deg(chart.moon_sidereal);
    let moon_nakshatra = moon_nak.index() as u8;

    // Mook number: nakshatra pada (1-108)
    let nak_index = moon_nak.index() as u8;
    let pada = Nakshatra::pada(chart.moon_sidereal); // 1-4
    let mook_number = nak_index * 4 + pada; // 1-108 range

    // Moon's house from Arudha Lagna
    let moon_sign = (chart.moon_sidereal.rem_euclid(360.0) / 30.0) as usize;
    let house_from_arudha = ((moon_sign + 12 - arudha_clamped) % 12) as u8 + 1;
    let favorable = FAVORABLE_MOON_HOUSES.contains(&house_from_arudha);

    // Timing from lagna modality
    let timing_indicator = timing_from_lagna(chart.asc_sidereal);

    PrashnaAnalysis {
        arudha_lagna: arudha_clamped as u8 + 1,
        mook_number,
        prashna_lagna_lord,
        moon_nakshatra,
        favorable,
        timing_indicator,
    }
}

/// Determine timing speed from the Prashna Lagna's sign modality.
///
/// - Cardinal (Movable) signs: quick outcome
/// - Fixed signs: delayed outcome
/// - Mutable (Dual) signs: medium-speed outcome
pub fn timing_from_lagna(lagna_deg: f64) -> TimingIndicator {
    let rashi = Rashi::from_longitude_deg(lagna_deg);
    match rashi.modality() {
        Modality::Movable => TimingIndicator::Quick,
        Modality::Fixed => TimingIndicator::Delayed,
        Modality::Dual => TimingIndicator::Medium,
    }
}

/// Signification of a chosen Ashtamangala item (Kerala tradition).
///
/// Each of the eight auspicious items maps to a planet, an outcome nature,
/// and a recommendation. The querent picks one item at random; the reading
/// reveals the karmic flavor of the answer.
pub fn ashtamangala_item_signification(item: AshtamangalaItem) -> AshtamangalaReading {
    match item {
        AshtamangalaItem::Gold => AshtamangalaReading {
            item,
            ruling_planet: Planet::Sun,
            outcome_nature: "Success through authority, government or father figures",
            recommendation: "Propitiate Sun; wear ruby or offer wheat",
        },
        AshtamangalaItem::Mirror => AshtamangalaReading {
            item,
            ruling_planet: Planet::Moon,
            outcome_nature: "Emotional resolution; matters related to mother or mind",
            recommendation: "Propitiate Moon; wear pearl or offer rice",
        },
        AshtamangalaItem::Turmeric => AshtamangalaReading {
            item,
            ruling_planet: Planet::Jupiter,
            outcome_nature: "Wisdom, expansion, blessings from guru or teacher",
            recommendation: "Propitiate Jupiter; wear yellow sapphire or offer turmeric",
        },
        AshtamangalaItem::Book => AshtamangalaReading {
            item,
            ruling_planet: Planet::Mercury,
            outcome_nature: "Success through intellect, communication or commerce",
            recommendation: "Propitiate Mercury; wear emerald or offer green lentils",
        },
        AshtamangalaItem::Betel => AshtamangalaReading {
            item,
            ruling_planet: Planet::Venus,
            outcome_nature: "Favorable for love, luxury, partnerships and arts",
            recommendation: "Propitiate Venus; wear diamond or offer white items",
        },
        AshtamangalaItem::Fruit => AshtamangalaReading {
            item,
            ruling_planet: Planet::Mars,
            outcome_nature: "Action-oriented results; energy, siblings, land matters",
            recommendation: "Propitiate Mars; wear red coral or offer red lentils",
        },
        AshtamangalaItem::Lamp => AshtamangalaReading {
            item,
            ruling_planet: Planet::Ketu,
            outcome_nature: "Spiritual insight; detachment or sudden liberation",
            recommendation: "Propitiate Ketu; offer sesame or give to ascetics",
        },
        AshtamangalaItem::FullVessel => AshtamangalaReading {
            item,
            ruling_planet: Planet::Saturn,
            outcome_nature: "Delayed but lasting results; karmic debt resolution",
            recommendation: "Propitiate Saturn; wear blue sapphire or offer sesame oil",
        },
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_chart() -> PrashnaChart {
        PrashnaChart {
            query_jd: 2460000.5,
            asc_sidereal: 15.0,  // Aries lagna
            moon_sidereal: 50.0, // Taurus, Rohini nakshatra
            planet_positions: vec![
                (Planet::Sun, 280.0),
                (Planet::Moon, 50.0),
                (Planet::Mars, 120.0),
                (Planet::Mercury, 300.0),
                (Planet::Jupiter, 45.0),
                (Planet::Venus, 200.0),
                (Planet::Saturn, 330.0),
            ],
        }
    }

    #[test]
    fn arudha_clamped_to_range() {
        let chart = sample_chart();
        let result = analyze_prashna(&chart, 1);
        assert_eq!(result.arudha_lagna, 1);
        let result = analyze_prashna(&chart, 12);
        assert_eq!(result.arudha_lagna, 12);
        // Out of range clamped
        let result = analyze_prashna(&chart, 0);
        assert_eq!(result.arudha_lagna, 1);
        let result = analyze_prashna(&chart, 255);
        assert_eq!(result.arudha_lagna, 12);
    }

    #[test]
    fn prashna_lagna_lord_aries() {
        let chart = sample_chart();
        let result = analyze_prashna(&chart, 1); // Aries
        assert_eq!(result.prashna_lagna_lord, Planet::Mars);
    }

    #[test]
    fn prashna_lagna_lord_cancer() {
        let chart = sample_chart();
        let result = analyze_prashna(&chart, 4); // Cancer
        assert_eq!(result.prashna_lagna_lord, Planet::Moon);
    }

    #[test]
    fn prashna_lagna_lord_all_signs() {
        let chart = sample_chart();
        let expected = [
            Planet::Mars,    // 1 Aries
            Planet::Venus,   // 2 Taurus
            Planet::Mercury, // 3 Gemini
            Planet::Moon,    // 4 Cancer
            Planet::Sun,     // 5 Leo
            Planet::Mercury, // 6 Virgo
            Planet::Venus,   // 7 Libra
            Planet::Mars,    // 8 Scorpio
            Planet::Jupiter, // 9 Sagittarius
            Planet::Saturn,  // 10 Capricorn
            Planet::Saturn,  // 11 Aquarius
            Planet::Jupiter, // 12 Pisces
        ];
        for (i, exp) in expected.iter().enumerate() {
            let result = analyze_prashna(&chart, (i + 1) as u8);
            assert_eq!(
                result.prashna_lagna_lord,
                *exp,
                "Sign {} should have lord {:?}",
                i + 1,
                exp
            );
        }
    }

    #[test]
    fn moon_nakshatra_computed() {
        let chart = sample_chart();
        let result = analyze_prashna(&chart, 1);
        // Moon at 50 degrees -> Rohini (index 3)
        assert_eq!(result.moon_nakshatra, 3);
    }

    #[test]
    fn mook_number_in_range() {
        let chart = sample_chart();
        let result = analyze_prashna(&chart, 1);
        assert!(result.mook_number >= 1 && result.mook_number <= 108);
    }

    #[test]
    fn favorable_moon_houses() {
        // Moon in Taurus (sign 1), Arudha=1 (Aries, sign 0) -> house 2 from arudha
        let chart = sample_chart();
        let result = analyze_prashna(&chart, 1);
        // house 2 is in the favorable list
        assert!(result.favorable);
    }

    #[test]
    fn unfavorable_moon_house() {
        // Set Moon so it lands in house 4, 6, 8, or 12 from arudha
        // Arudha=2 (Taurus, index 1), Moon at 50 deg = Taurus (index 1)
        // house = (1 - 1 + 12) % 12 + 1 = 1 -> favorable
        // Arudha=3 (Gemini, index 2), Moon at 50 deg = Taurus (index 1)
        // house = (1 - 2 + 12) % 12 + 1 = 12 -> NOT in favorable list
        let chart = sample_chart();
        let result = analyze_prashna(&chart, 3);
        assert!(
            !result.favorable,
            "House 12 from arudha should be unfavorable"
        );
    }

    #[test]
    fn timing_movable_sign() {
        // Aries (0-30 deg) is Movable
        assert_eq!(timing_from_lagna(15.0), TimingIndicator::Quick);
    }

    #[test]
    fn timing_fixed_sign() {
        // Taurus (30-60 deg) is Fixed
        assert_eq!(timing_from_lagna(45.0), TimingIndicator::Delayed);
    }

    #[test]
    fn timing_dual_sign() {
        // Gemini (60-90 deg) is Dual
        assert_eq!(timing_from_lagna(75.0), TimingIndicator::Medium);
    }

    #[test]
    fn timing_all_modalities_cycle() {
        // Aries=Movable, Taurus=Fixed, Gemini=Dual, Cancer=Movable, ...
        let expected = [
            TimingIndicator::Quick,   // Aries
            TimingIndicator::Delayed, // Taurus
            TimingIndicator::Medium,  // Gemini
            TimingIndicator::Quick,   // Cancer
            TimingIndicator::Delayed, // Leo
            TimingIndicator::Medium,  // Virgo
            TimingIndicator::Quick,   // Libra
            TimingIndicator::Delayed, // Scorpio
            TimingIndicator::Medium,  // Sagittarius
            TimingIndicator::Quick,   // Capricorn
            TimingIndicator::Delayed, // Aquarius
            TimingIndicator::Medium,  // Pisces
        ];
        for (i, exp) in expected.iter().enumerate() {
            let deg = i as f64 * 30.0 + 15.0;
            assert_eq!(
                timing_from_lagna(deg),
                *exp,
                "Sign index {} should have timing {:?}",
                i,
                exp
            );
        }
    }

    #[test]
    fn ashtamangala_all_items() {
        for item in AshtamangalaItem::ALL {
            let reading = ashtamangala_item_signification(item);
            assert_eq!(reading.item, item);
            assert!(!reading.outcome_nature.is_empty());
            assert!(!reading.recommendation.is_empty());
        }
    }

    #[test]
    fn ashtamangala_gold_is_sun() {
        let r = ashtamangala_item_signification(AshtamangalaItem::Gold);
        assert_eq!(r.ruling_planet, Planet::Sun);
    }

    #[test]
    fn ashtamangala_mirror_is_moon() {
        let r = ashtamangala_item_signification(AshtamangalaItem::Mirror);
        assert_eq!(r.ruling_planet, Planet::Moon);
    }

    #[test]
    fn ashtamangala_turmeric_is_jupiter() {
        let r = ashtamangala_item_signification(AshtamangalaItem::Turmeric);
        assert_eq!(r.ruling_planet, Planet::Jupiter);
    }

    #[test]
    fn planet_display() {
        assert_eq!(format!("{}", Planet::Sun), "Sun");
        assert_eq!(format!("{}", Planet::Rahu), "Rahu");
    }
}
