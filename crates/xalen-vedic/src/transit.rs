use crate::rashi::Rashi;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SadeSatiPhase {
    NotActive,
    Rising,  // Saturn in 12th from Moon sign
    Peak,    // Saturn in Moon sign (over Moon)
    Setting, // Saturn in 2nd from Moon sign
}

pub fn sade_sati_phase(saturn_rashi: Rashi, moon_rashi: Rashi) -> SadeSatiPhase {
    let sat_idx = saturn_rashi.index();
    let moon_idx = moon_rashi.index();
    let diff = ((sat_idx as i32 - moon_idx as i32).rem_euclid(12)) as usize;

    match diff {
        11 => SadeSatiPhase::Rising, // 12th from Moon (0-indexed: 11)
        0 => SadeSatiPhase::Peak,    // Same sign as Moon
        1 => SadeSatiPhase::Setting, // 2nd from Moon
        _ => SadeSatiPhase::NotActive,
    }
}

pub fn is_sade_sati(saturn_rashi: Rashi, moon_rashi: Rashi) -> bool {
    sade_sati_phase(saturn_rashi, moon_rashi) != SadeSatiPhase::NotActive
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitStrength {
    Favorable,
    Unfavorable,
    Neutral,
}

/// Planet identifier for transit-strength lookup.
/// Favorable houses from Moon per Brihat Parashara Hora Shastra.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitPlanet {
    Sun,
    Moon,
    Mars,
    Mercury,
    Jupiter,
    Venus,
    Saturn,
    Rahu,
    Ketu,
}

/// Returns the set of favorable house-numbers (1-based, from Moon) for a given planet.
/// Source: BPHS transit-gochar tables.
fn favorable_houses(planet: TransitPlanet) -> &'static [usize] {
    match planet {
        TransitPlanet::Sun => &[3, 6, 10, 11],
        TransitPlanet::Moon => &[1, 3, 6, 7, 10, 11],
        TransitPlanet::Mars => &[3, 6, 11],
        TransitPlanet::Mercury => &[2, 4, 6, 8, 10, 11],
        TransitPlanet::Jupiter => &[2, 5, 7, 9, 11],
        TransitPlanet::Venus => &[1, 2, 3, 4, 5, 8, 9, 11, 12],
        TransitPlanet::Saturn => &[3, 6, 11],
        TransitPlanet::Rahu => &[3, 6, 11],
        TransitPlanet::Ketu => &[3, 6, 11],
    }
}

/// Evaluate transit strength of a specific planet from Moon using
/// planet-specific favorable house tables (BPHS).
pub fn transit_from_moon(
    planet: TransitPlanet,
    planet_rashi: Rashi,
    moon_rashi: Rashi,
) -> TransitStrength {
    let house =
        ((planet_rashi.index() as i32 - moon_rashi.index() as i32).rem_euclid(12)) as usize + 1;
    if favorable_houses(planet).contains(&house) {
        TransitStrength::Favorable
    } else {
        TransitStrength::Unfavorable
    }
}

pub fn chandrashtama(moon_transit_rashi: Rashi, natal_moon_rashi: Rashi) -> bool {
    let diff = ((moon_transit_rashi.index() as i32 - natal_moon_rashi.index() as i32)
        .rem_euclid(12)) as usize
        + 1;
    diff == 8 // Moon transiting 8th from natal Moon
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sade_sati_peak() {
        assert_eq!(
            sade_sati_phase(Rashi::Mesha, Rashi::Mesha),
            SadeSatiPhase::Peak
        );
    }

    #[test]
    fn sade_sati_rising() {
        // Saturn in 12th from Moon: if Moon=Aries(0), Saturn=Pisces(11) → 12th
        assert_eq!(
            sade_sati_phase(Rashi::Meena, Rashi::Mesha),
            SadeSatiPhase::Rising
        );
    }

    #[test]
    fn sade_sati_setting() {
        // Saturn in 2nd from Moon: if Moon=Aries(0), Saturn=Taurus(1)
        assert_eq!(
            sade_sati_phase(Rashi::Vrishabha, Rashi::Mesha),
            SadeSatiPhase::Setting
        );
    }

    #[test]
    fn sade_sati_not_active() {
        assert_eq!(
            sade_sati_phase(Rashi::Karka, Rashi::Mesha),
            SadeSatiPhase::NotActive
        );
    }

    #[test]
    fn transit_jupiter_favorable_2nd() {
        // Jupiter in 2nd from Moon = favorable per BPHS (2,5,7,9,11)
        assert_eq!(
            transit_from_moon(TransitPlanet::Jupiter, Rashi::Vrishabha, Rashi::Mesha),
            TransitStrength::Favorable
        );
    }

    #[test]
    fn transit_jupiter_unfavorable_3rd() {
        // Jupiter in 3rd from Moon = unfavorable (3 is NOT in Jupiter's list)
        assert_eq!(
            transit_from_moon(TransitPlanet::Jupiter, Rashi::Mithuna, Rashi::Mesha),
            TransitStrength::Unfavorable
        );
    }

    #[test]
    fn transit_sun_favorable_houses() {
        // Sun favorable from Moon: 3,6,10,11
        // Moon in Aries(0). Sun in Gemini(2) = 3rd house → favorable
        assert_eq!(
            transit_from_moon(TransitPlanet::Sun, Rashi::Mithuna, Rashi::Mesha),
            TransitStrength::Favorable
        );
        // Sun in Taurus(1) = 2nd house → unfavorable
        assert_eq!(
            transit_from_moon(TransitPlanet::Sun, Rashi::Vrishabha, Rashi::Mesha),
            TransitStrength::Unfavorable
        );
    }

    #[test]
    fn transit_saturn_favorable_houses() {
        // Saturn favorable from Moon: 3,6,11
        // Moon in Aries(0). Saturn in Gemini(2) = 3rd → favorable
        assert_eq!(
            transit_from_moon(TransitPlanet::Saturn, Rashi::Mithuna, Rashi::Mesha),
            TransitStrength::Favorable
        );
        // Saturn in Taurus(1) = 2nd → unfavorable
        assert_eq!(
            transit_from_moon(TransitPlanet::Saturn, Rashi::Vrishabha, Rashi::Mesha),
            TransitStrength::Unfavorable
        );
    }

    #[test]
    fn transit_venus_generous_favorable() {
        // Venus favorable: 1,2,3,4,5,8,9,11,12 — only 6,7,10 are unfavorable
        // Moon in Aries(0). Venus in Aries(0) = 1st → favorable
        assert_eq!(
            transit_from_moon(TransitPlanet::Venus, Rashi::Mesha, Rashi::Mesha),
            TransitStrength::Favorable
        );
        // Venus in Virgo(5) = 6th → unfavorable
        assert_eq!(
            transit_from_moon(TransitPlanet::Venus, Rashi::Kanya, Rashi::Mesha),
            TransitStrength::Unfavorable
        );
    }

    #[test]
    fn transit_mars_matches_saturn_pattern() {
        // Mars and Saturn share the same favorable houses: 3,6,11
        for sign_idx in 0..12 {
            let planet_rashi = Rashi::from_index(sign_idx);
            assert_eq!(
                transit_from_moon(TransitPlanet::Mars, planet_rashi, Rashi::Mesha),
                transit_from_moon(TransitPlanet::Saturn, planet_rashi, Rashi::Mesha),
                "Mars and Saturn should agree for house {}",
                sign_idx + 1
            );
        }
    }

    #[test]
    fn transit_rahu_ketu_match_mars() {
        // Rahu/Ketu favorable: 3,6,11 — same as Mars/Saturn
        for sign_idx in 0..12 {
            let planet_rashi = Rashi::from_index(sign_idx);
            let mars = transit_from_moon(TransitPlanet::Mars, planet_rashi, Rashi::Mesha);
            assert_eq!(
                transit_from_moon(TransitPlanet::Rahu, planet_rashi, Rashi::Mesha),
                mars
            );
            assert_eq!(
                transit_from_moon(TransitPlanet::Ketu, planet_rashi, Rashi::Mesha),
                mars
            );
        }
    }

    #[test]
    fn chandrashtama_detection() {
        // Moon transiting 8th from natal Moon
        // Natal Moon in Aries(0), transit Moon in Scorpio(7) → 8th house
        assert!(chandrashtama(Rashi::Vrishchika, Rashi::Mesha));
        assert!(!chandrashtama(Rashi::Karka, Rashi::Mesha));
    }

    #[test]
    fn sade_sati_all_signs_consistent() {
        for moon in 0..12 {
            let moon_rashi = Rashi::from_index(moon);
            let mut phases = 0;
            for sat in 0..12 {
                let sat_rashi = Rashi::from_index(sat);
                if is_sade_sati(sat_rashi, moon_rashi) {
                    phases += 1;
                }
            }
            assert_eq!(
                phases, 3,
                "Sade Sati should cover exactly 3 signs for Moon in {moon_rashi}"
            );
        }
    }
}
