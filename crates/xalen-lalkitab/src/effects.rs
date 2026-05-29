//! Planet effect classification per house.
//!
//! Each planet in each of the 12 houses gives Good, Neutral, or Troubled
//! results according to Lal Kitab principles. These tables are derived from
//! Pt. Roop Chand Joshi's classifications across the five volumes.

use crate::houses::Planet;
use serde::{Deserialize, Serialize};

/// Effect of a planet occupying a particular house.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Effect classification for a planet in a house (Good, Neutral, Troubled).
pub enum LalKitabEffect {
    /// Planet gives beneficial results in this house.
    Good,
    /// Planet gives mixed/moderate results.
    Neutral,
    /// Planet gives adverse results and needs remedies.
    Troubled,
}

impl std::fmt::Display for LalKitabEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Good => f.write_str("Good"),
            Self::Neutral => f.write_str("Neutral"),
            Self::Troubled => f.write_str("Troubled"),
        }
    }
}

/// Classify the effect of `planet` when placed in `house` (1-12).
///
/// The tables below encode the Lal Kitab good/troubled positions for each
/// planet. Houses not listed in either set are treated as Neutral.
pub fn planet_effect(planet: Planet, house: usize) -> LalKitabEffect {
    let house = house.clamp(1, 12);

    let (good, troubled): (&[usize], &[usize]) = match planet {
        Planet::Sun => (&[1, 3, 4, 10, 11], &[6, 7, 8, 12]),
        Planet::Moon => (&[1, 2, 3, 4, 7], &[6, 8, 12]),
        Planet::Mars => (&[1, 2, 3, 6, 10], &[4, 7, 8, 12]),
        Planet::Mercury => (&[1, 2, 4, 5, 6], &[3, 8, 9, 12]),
        Planet::Jupiter => (&[2, 5, 9, 11, 12], &[3, 6, 7, 10]),
        Planet::Venus => (&[2, 3, 4, 7, 12], &[1, 6, 8, 9]),
        Planet::Saturn => (&[2, 3, 7, 10, 11], &[1, 4, 5, 6]),
        Planet::Rahu => (&[3, 4, 6], &[1, 2, 5, 7, 8, 9]),
        Planet::Ketu => (&[3, 6, 11], &[4, 5, 7, 8]),
        _ => return LalKitabEffect::Neutral,
    };

    if good.contains(&house) {
        LalKitabEffect::Good
    } else if troubled.contains(&house) {
        LalKitabEffect::Troubled
    } else {
        LalKitabEffect::Neutral
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sun_good_houses() {
        for h in [1, 3, 4, 10, 11] {
            assert_eq!(
                planet_effect(Planet::Sun, h),
                LalKitabEffect::Good,
                "Sun in house {h} should be Good"
            );
        }
    }

    #[test]
    fn sun_troubled_houses() {
        for h in [6, 7, 8, 12] {
            assert_eq!(
                planet_effect(Planet::Sun, h),
                LalKitabEffect::Troubled,
                "Sun in house {h} should be Troubled"
            );
        }
    }

    #[test]
    fn sun_neutral_houses() {
        for h in [2, 5, 9] {
            assert_eq!(
                planet_effect(Planet::Sun, h),
                LalKitabEffect::Neutral,
                "Sun in house {h} should be Neutral"
            );
        }
    }

    #[test]
    fn all_planets_cover_all_houses() {
        // Every planet-house combo must return a valid variant (no panic).
        for p in Planet::VEDIC_NINE {
            for h in 1..=12 {
                let _ = planet_effect(p, h);
            }
        }
    }

    #[test]
    fn jupiter_good_includes_12() {
        // Lal Kitab uniquely considers Jupiter good in 12 (unlike BPHS).
        assert_eq!(planet_effect(Planet::Jupiter, 12), LalKitabEffect::Good);
    }

    #[test]
    fn rahu_troubled_majority() {
        // Rahu is troubled in more houses than it is good.
        let troubled_count = (1..=12)
            .filter(|&h| planet_effect(Planet::Rahu, h) == LalKitabEffect::Troubled)
            .count();
        let good_count = (1..=12)
            .filter(|&h| planet_effect(Planet::Rahu, h) == LalKitabEffect::Good)
            .count();
        assert!(troubled_count > good_count);
    }
}
