use crate::rashi::Rashi;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedYoga {
    pub name: &'static str,
    pub category: YogaCategory,
    pub planets_involved: Vec<String>,
    pub strength: YogaStrength,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum YogaCategory {
    PanchaMahapurusha,
    Raja,
    Dhana,
    Chandra,
    Solar,
    Parivartana,
    Vipreeta,
    Nabhasa,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum YogaStrength {
    Strong,
    Moderate,
    Weak,
}

/// Detect Pancha Mahapurusha yoga for a single planet.
///
/// Conditions (web-verified):
/// - Ruchaka: Mars in own sign (Aries/Scorpio) or exalted (Capricorn) in kendra (1,4,7,10)
/// - Bhadra:  Mercury in own sign (Gemini/Virgo) in kendra — Virgo is both own sign AND
///   exaltation, so Mercury's exaltation is already covered by the own-sign check
/// - Hamsa:   Jupiter in own sign (Sagittarius/Pisces) or exalted (Cancer) in kendra
/// - Malavya: Venus in own sign (Taurus/Libra) or exalted (Pisces) in kendra
/// - Shasha:  Saturn in own sign (Capricorn/Aquarius) or exalted (Libra) in kendra
///
/// Sources:
/// - https://blog.cosmicinsights.net/pancha-mahapurusha-yogas/
/// - https://panchangbodh.com/panch-mahapurusha-yoga
/// - https://www.indastro.com/learn-astrology/yoga-dasa/pancha-maha-purush-yoga.html
pub fn detect_pancha_mahapurusha(
    planet: &str,
    planet_rashi: Rashi,
    planet_house: usize, // 1-12
) -> Option<DetectedYoga> {
    let is_kendra = matches!(planet_house, 1 | 4 | 7 | 10);
    if !is_kendra {
        return None;
    }

    let (yoga_name, is_own_or_exalted) = match planet {
        // Mars: own = Aries(Mesha), Scorpio(Vrishchika); exalted = Capricorn(Makara)
        "Mars" => (
            "Ruchaka",
            matches!(
                planet_rashi,
                Rashi::Mesha | Rashi::Vrishchika | Rashi::Makara
            ),
        ),
        // Mercury: own = Gemini(Mithuna), Virgo(Kanya); exalted = Virgo(Kanya) — already covered
        "Mercury" => (
            "Bhadra",
            matches!(planet_rashi, Rashi::Mithuna | Rashi::Kanya),
        ),
        // Jupiter: own = Sagittarius(Dhanu), Pisces(Meena); exalted = Cancer(Karka)
        "Jupiter" => (
            "Hamsa",
            matches!(planet_rashi, Rashi::Dhanu | Rashi::Meena | Rashi::Karka),
        ),
        // Venus: own = Taurus(Vrishabha), Libra(Tula); exalted = Pisces(Meena)
        "Venus" => (
            "Malavya",
            matches!(planet_rashi, Rashi::Vrishabha | Rashi::Tula | Rashi::Meena),
        ),
        // Saturn: own = Capricorn(Makara), Aquarius(Kumbha); exalted = Libra(Tula)
        "Saturn" => (
            "Shasha",
            matches!(planet_rashi, Rashi::Makara | Rashi::Kumbha | Rashi::Tula),
        ),
        _ => return None,
    };

    if is_own_or_exalted {
        Some(DetectedYoga {
            name: yoga_name,
            category: YogaCategory::PanchaMahapurusha,
            planets_involved: vec![planet.to_string()],
            strength: YogaStrength::Strong,
        })
    } else {
        None
    }
}

/// Detect Gajakesari Yoga: Jupiter in kendra (1,4,7,10) from Moon.
///
/// Web-verified: Jupiter must occupy the 1st, 4th, 7th, or 10th house
/// counted from the Moon's position.
///
/// Sources:
/// - https://www.ganeshaspeaks.com/learn-astrology/yogas/gaj-kesari-yoga/
/// - https://www.indastro.com/learn-astrology/yoga-dasa/gaj-kesari-yoga.html
/// - https://www.anytimeastro.com/blog/astrology/gajakesari-yoga-in-kundali/
pub fn detect_gajakesari(jupiter_house: usize, moon_house: usize) -> Option<DetectedYoga> {
    // diff 0 = same house (1st), 3 = 4th, 6 = 7th, 9 = 10th from Moon
    let diff = ((jupiter_house as i32 - moon_house as i32).rem_euclid(12)) as usize;
    if matches!(diff, 0 | 3 | 6 | 9) {
        Some(DetectedYoga {
            name: "Gajakesari",
            category: YogaCategory::Chandra,
            planets_involved: vec!["Jupiter".into(), "Moon".into()],
            strength: YogaStrength::Strong,
        })
    } else {
        None
    }
}

/// Detect Budhaditya Yoga: Sun and Mercury in the same house (conjunction).
///
/// Web-verified conditions (BPHS, Saravali):
/// - Sun and Mercury must occupy the same house (rashi) in the birth chart.
/// - Mercury must NOT be combust (within ~3 degrees of Sun) for full strength,
///   though the basic yoga still forms. Combustion check is left to the caller.
/// - Strongest when formed in Kendra (1,4,7,10) or Trikona (1,5,9) houses.
///
/// Sources:
/// - https://astrosight.ai/yogas/budhaditya-yoga-kundli-meaning-benefits
/// - https://www.indastro.com/learn-astrology/yoga-dasa/budh-aditya-yoga.html
/// - https://www.mpanchang.com/articles/astrology/budhaditya-yoga/
/// - https://omegaastro.com/how-budhaditya-yoga-shapes-communication-leadership/
pub fn detect_budhaditya(sun_house: usize, mercury_house: usize) -> Option<DetectedYoga> {
    if sun_house == mercury_house {
        Some(DetectedYoga {
            name: "Budhaditya",
            category: YogaCategory::Solar,
            planets_involved: vec!["Sun".into(), "Mercury".into()],
            strength: YogaStrength::Moderate,
        })
    } else {
        None
    }
}

/// Detect Vipreeta Raja Yoga (three sub-types).
///
/// Web-verified conditions:
/// - **Harsha Yoga**: 6th lord placed in 8th or 12th house (NOT own 6th).
/// - **Sarala Yoga**: 8th lord placed in 6th or 12th house (NOT own 8th).
/// - **Vimala Yoga**: 12th lord placed in 6th or 8th house (NOT own 12th).
///
/// The principle: a trik lord (6/8/12) going to ANOTHER trik house destroys
/// both negativities — "two negatives giving a positive."
///
/// Sources:
/// - https://www.astrosage.com/magazine/vipreetrajyoga.asp
/// - https://vedicmarga.com/vipareet-rajayoga/
/// - https://www.mypandit.com/kundli/yoga/vipreet-raj/
/// - https://www.sanatanveda.com/astrology/vipareeta-raja-yoga-in-vedic-astrology/
pub fn detect_vipreeta_raja(
    lord_6_house: usize,
    lord_8_house: usize,
    lord_12_house: usize,
) -> Vec<DetectedYoga> {
    let mut yogas = Vec::new();
    let trik = [6, 8, 12];

    // Harsha Yoga: 6th lord in another trik house (8 or 12), not own 6th
    if trik.contains(&lord_6_house) && lord_6_house != 6 {
        yogas.push(DetectedYoga {
            name: "Vipreeta Raja (Harsha - 6th lord)",
            category: YogaCategory::Vipreeta,
            planets_involved: vec![format!("6th lord in house {}", lord_6_house)],
            strength: YogaStrength::Moderate,
        });
    }
    // Sarala Yoga: 8th lord in another trik house (6 or 12), not own 8th
    if trik.contains(&lord_8_house) && lord_8_house != 8 {
        yogas.push(DetectedYoga {
            name: "Vipreeta Raja (Sarala - 8th lord)",
            category: YogaCategory::Vipreeta,
            planets_involved: vec![format!("8th lord in house {}", lord_8_house)],
            strength: YogaStrength::Moderate,
        });
    }
    // Vimala Yoga: 12th lord in another trik house (6 or 8), not own 12th
    if trik.contains(&lord_12_house) && lord_12_house != 12 {
        yogas.push(DetectedYoga {
            name: "Vipreeta Raja (Vimala - 12th lord)",
            category: YogaCategory::Vipreeta,
            planets_involved: vec![format!("12th lord in house {}", lord_12_house)],
            strength: YogaStrength::Moderate,
        });
    }
    yogas
}

/// Detect Kemadruma Yoga: no planets in the 2nd or 12th house from Moon.
///
/// Web-verified conditions:
/// - No planets (excluding Sun, Rahu, Ketu) in the 2nd house from Moon.
/// - No planets (excluding Sun, Rahu, Ketu) in the 12th house from Moon.
/// - The caller should exclude Sun, Rahu, and Ketu from `planets_in_houses`.
///
/// Cancellation conditions (not checked here, left to higher-level logic):
/// - Benefic aspects on Moon (Jupiter/Venus/Mercury)
/// - Benefic planet in kendra from Moon
/// - Moon in kendra from Lagna
/// - Moon in own sign (Cancer) or exaltation (Taurus)
///
/// Sources:
/// - https://vedicmarga.com/kemadruma-yoga/
/// - https://blog.pocketpandit.com/kemadruma-yoga/
/// - https://www.sanatanveda.com/astrology/kemadruma-yoga-and-remedies-in-vedic-astrology/
/// - https://astroparasar.com/kemadruma-yoga/
pub fn detect_kemadruma(
    moon_house: usize,
    planets_in_houses: &[usize], // house numbers of all planets except Sun, Rahu, Ketu
) -> Option<DetectedYoga> {
    let mh = if moon_house == 0 { 1 } else { moon_house }; // guard zero input
    let house_2_from_moon = (mh % 12) + 1;
    let house_12_from_moon = if mh <= 1 { 12 } else { mh - 1 };

    let has_support = planets_in_houses
        .iter()
        .any(|&h| h == house_2_from_moon || h == house_12_from_moon);

    if !has_support {
        Some(DetectedYoga {
            name: "Kemadruma",
            category: YogaCategory::Chandra,
            planets_involved: vec!["Moon".into()],
            strength: YogaStrength::Weak,
        })
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Graha Yuddha (Planetary War)
// ---------------------------------------------------------------------------

/// Result of a Graha Yuddha (planetary war) detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrahaYuddha {
    /// The planet that wins the war (brighter/more northern latitude).
    pub winner: String,
    /// The planet that loses the war.
    pub loser: String,
    /// Angular separation in degrees between the two combatants.
    pub separation_deg: f64,
}

/// Detect Graha Yuddha (planetary war) between two planets.
///
/// Graha Yuddha occurs when two of the five tara grahas (Mars, Mercury,
/// Jupiter, Venus, Saturn) are within 1 degree of each other in ecliptic
/// longitude.  Sun and Moon do not participate in graha yuddha.
///
/// The winner is determined by apparent brightness.  Since we do not have
/// magnitude data here, we use the classical hierarchy where a planet
/// closer to the beginning of the natural brightness order wins:
///   Venus > Jupiter > Mars > Saturn > Mercury
/// (Venus is always brightest; Mercury always faintest among tara grahas.)
///
/// This simplified ordering matches the BPHS/Surya Siddhanta convention
/// used by most Vedic software.
///
/// Sources:
/// - BPHS Ch. 18 (Graha Yuddha section)
/// - Surya Siddhanta (planetary war chapter)
pub fn detect_graha_yuddha(
    planet1: &str,
    lon1: f64,
    planet2: &str,
    lon2: f64,
) -> Option<GrahaYuddha> {
    // Only the five tara grahas participate
    const TARA_GRAHAS: [&str; 5] = ["Mars", "Mercury", "Jupiter", "Venus", "Saturn"];
    if !TARA_GRAHAS.contains(&planet1) || !TARA_GRAHAS.contains(&planet2) {
        return None;
    }
    if planet1 == planet2 {
        return None;
    }

    let diff = (lon1 - lon2).rem_euclid(360.0);
    let separation = if diff > 180.0 { 360.0 - diff } else { diff };

    if separation > 1.0 {
        return None;
    }

    // Brightness order: Venus(0) > Jupiter(1) > Mars(2) > Saturn(3) > Mercury(4)
    // Lower index = brighter = wins.
    fn brightness_rank(planet: &str) -> usize {
        match planet {
            "Venus" => 0,
            "Jupiter" => 1,
            "Mars" => 2,
            "Saturn" => 3,
            "Mercury" => 4,
            _ => 5,
        }
    }

    let rank1 = brightness_rank(planet1);
    let rank2 = brightness_rank(planet2);
    let (winner, loser) = if rank1 <= rank2 {
        (planet1, planet2)
    } else {
        (planet2, planet1)
    };

    Some(GrahaYuddha {
        winner: winner.to_string(),
        loser: loser.to_string(),
        separation_deg: separation,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ruchaka_yoga() {
        let y = detect_pancha_mahapurusha("Mars", Rashi::Mesha, 1);
        assert!(y.is_some());
        assert_eq!(y.unwrap().name, "Ruchaka");
    }

    #[test]
    fn hamsa_yoga_in_kendra() {
        let y = detect_pancha_mahapurusha("Jupiter", Rashi::Karka, 4);
        assert!(y.is_some());
        assert_eq!(y.unwrap().name, "Hamsa");
    }

    #[test]
    fn no_yoga_wrong_sign() {
        let y = detect_pancha_mahapurusha("Mars", Rashi::Mithuna, 1);
        assert!(y.is_none());
    }

    #[test]
    fn no_yoga_not_kendra() {
        let y = detect_pancha_mahapurusha("Mars", Rashi::Mesha, 3);
        assert!(y.is_none());
    }

    #[test]
    fn gajakesari_conjunction() {
        let y = detect_gajakesari(4, 4);
        assert!(y.is_some());
    }

    #[test]
    fn gajakesari_trine() {
        let y = detect_gajakesari(1, 10);
        assert!(y.is_some());
    }

    #[test]
    fn no_gajakesari() {
        let y = detect_gajakesari(1, 3);
        assert!(y.is_none());
    }

    #[test]
    fn budhaditya() {
        let y = detect_budhaditya(5, 5);
        assert!(y.is_some());
    }

    #[test]
    fn vipreeta_raja() {
        let yogas = detect_vipreeta_raja(8, 12, 6);
        assert_eq!(yogas.len(), 3);
    }

    #[test]
    fn kemadruma_no_support() {
        let y = detect_kemadruma(5, &[1, 10, 8]);
        assert!(y.is_some());
    }

    #[test]
    fn no_kemadruma_with_support() {
        let y = detect_kemadruma(5, &[4, 6, 10]); // planet in 4th (12th from Moon) and 6th (2nd from Moon)
        assert!(y.is_none());
    }

    // -----------------------------------------------------------------------
    // Graha Yuddha tests
    // -----------------------------------------------------------------------

    #[test]
    fn graha_yuddha_mars_mercury_within_1_deg() {
        let gy = detect_graha_yuddha("Mars", 100.0, "Mercury", 100.5);
        assert!(gy.is_some());
        let gy = gy.unwrap();
        assert_eq!(gy.winner, "Mars"); // Mars brighter than Mercury
        assert_eq!(gy.loser, "Mercury");
        assert!((gy.separation_deg - 0.5).abs() < 0.01);
    }

    #[test]
    fn graha_yuddha_venus_wins_over_jupiter() {
        let gy = detect_graha_yuddha("Jupiter", 200.0, "Venus", 200.3);
        assert!(gy.is_some());
        assert_eq!(gy.unwrap().winner, "Venus");
    }

    #[test]
    fn graha_yuddha_none_beyond_1_deg() {
        let gy = detect_graha_yuddha("Mars", 100.0, "Mercury", 101.5);
        assert!(gy.is_none());
    }

    #[test]
    fn graha_yuddha_none_for_sun() {
        let gy = detect_graha_yuddha("Sun", 100.0, "Mars", 100.5);
        assert!(gy.is_none());
    }

    #[test]
    fn graha_yuddha_none_for_moon() {
        let gy = detect_graha_yuddha("Moon", 100.0, "Venus", 100.3);
        assert!(gy.is_none());
    }

    #[test]
    fn graha_yuddha_none_same_planet() {
        let gy = detect_graha_yuddha("Mars", 100.0, "Mars", 100.5);
        assert!(gy.is_none());
    }

    #[test]
    fn graha_yuddha_wrap_around_360() {
        // Mars at 359.5°, Mercury at 0.3° → separation = 0.8°
        let gy = detect_graha_yuddha("Mars", 359.5, "Mercury", 0.3);
        assert!(gy.is_some());
        let gy = gy.unwrap();
        assert!((gy.separation_deg - 0.8).abs() < 0.01);
    }
}
