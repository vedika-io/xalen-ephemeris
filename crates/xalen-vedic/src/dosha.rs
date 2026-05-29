use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dosha {
    pub name: &'static str,
    pub present: bool,
    pub severity: DoshaSeverity,
    pub from_chart: &'static str,
    pub cancellations: Vec<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DoshaSeverity {
    None,
    Mild,
    Moderate,
    Severe,
}

pub fn detect_mangal_dosha(
    mars_house_from_lagna: usize,
    mars_house_from_moon: usize,
    mars_house_from_venus: usize,
    mars_rashi_index: usize,
    jupiter_aspects_mars: bool,
    mars_in_own_sign: bool,
    mars_in_exaltation: bool,
) -> Dosha {
    // Web-verified: Mars in houses 1, 2, 4, 7, 8, 12 from Lagna/Moon/Venus
    // Source: https://ishvaram.com/dosha/mangal-dosha/
    // Source: https://astronidan.com/calculator/manglik-dosha-calculator/
    // Note: house 2 is included per South Indian tradition; some texts use only 1, 4, 7, 8, 12
    let dosha_houses = [1, 2, 4, 7, 8, 12];

    let from_lagna = dosha_houses.contains(&mars_house_from_lagna);
    let from_moon = dosha_houses.contains(&mars_house_from_moon);
    let from_venus = dosha_houses.contains(&mars_house_from_venus);

    let count = [from_lagna, from_moon, from_venus]
        .iter()
        .filter(|&&x| x)
        .count();

    if count == 0 {
        return Dosha {
            name: "Mangal Dosha",
            present: false,
            severity: DoshaSeverity::None,
            from_chart: "None",
            cancellations: vec![],
        };
    }

    let mut cancellations = Vec::new();

    if mars_in_own_sign {
        cancellations.push("Mars in own sign (Aries/Scorpio)");
    }
    if mars_in_exaltation {
        cancellations.push("Mars in exaltation (Capricorn)");
    }
    if jupiter_aspects_mars {
        cancellations.push("Jupiter aspects Mars");
    }
    // Mrigashira=4, Chitra=13, Dhanishta=22 nakshatras — check by rashi approximation
    if matches!(mars_rashi_index, 1 | 5 | 9) {
        cancellations.push("Mars in Mrigashira/Chitra/Dhanishta nakshatra region");
    }

    let severity = if !cancellations.is_empty() {
        DoshaSeverity::Mild
    } else {
        match count {
            3 => DoshaSeverity::Severe,
            2 => DoshaSeverity::Moderate,
            _ => DoshaSeverity::Moderate,
        }
    };

    let from_chart = match (from_lagna, from_moon, from_venus) {
        (true, true, true) => "Lagna + Moon + Venus",
        (true, true, false) => "Lagna + Moon",
        (true, false, true) => "Lagna + Venus",
        (false, true, true) => "Moon + Venus",
        (true, false, false) => "Lagna",
        (false, true, false) => "Moon",
        (false, false, true) => "Venus",
        _ => "None",
    };

    Dosha {
        name: "Mangal Dosha",
        present: true,
        severity,
        from_chart,
        cancellations,
    }
}

/// Detect Kaal Sarpa Dosha / Kaal Amrit Yoga.
///
/// Web-verified conditions:
/// - All 7 visible planets (Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn)
///   must be hemmed between Rahu and Ketu on one side of the axis.
/// - If even one planet lies outside the Rahu-Ketu axis, no dosha forms.
/// - Rahu-to-Ketu direction = Kaal Sarpa Dosha (ascending hemisphere).
/// - Ketu-to-Rahu direction = Kaal Amrit Yoga (descending hemisphere).
/// - 12 types named by Rahu's house: Anant, Kulik, Vasuki, Shankhpal,
///   Padma, Mahapadma, Takshak, Karkotak, Shankhachud, Ghatak, Vishdhar, Sheshnag.
/// - A planet conjunct Rahu/Ketu partially breaks the axis (reduced severity).
///
/// Sources:
/// - https://astronidan.com/doshas/kaal-sarp-dosh/
/// - https://blog.pocketpandit.com/kaal-sarp-dosha-in-vedic-astrology/
/// - https://kaalavedicastro.com/tool/kaal-sarp-dosha-calculator
pub fn detect_kaal_sarpa(
    rahu_house: usize,
    ketu_house: usize,
    planet_houses: &[usize], // houses of Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn
) -> Dosha {
    // All 7 planets must be between Rahu and Ketu (within one hemisphere)
    let rahu = rahu_house;
    let ketu = ketu_house;

    let all_between = planet_houses.iter().all(|&h| {
        if rahu < ketu {
            h >= rahu && h <= ketu
        } else {
            h >= rahu || h <= ketu
        }
    });

    let all_between_reverse = planet_houses.iter().all(|&h| {
        if ketu < rahu {
            h >= ketu && h <= rahu
        } else {
            h >= ketu || h <= rahu
        }
    });

    let is_kaal_sarpa = all_between;
    let is_kaal_amrit = all_between_reverse && !is_kaal_sarpa;

    if is_kaal_sarpa || is_kaal_amrit {
        let ksd_type = match rahu_house {
            1 => "Anant",
            2 => "Kulik",
            3 => "Vasuki",
            4 => "Shankhpal",
            5 => "Padma",
            6 => "Mahapadma",
            7 => "Takshak",
            8 => "Karkotak",
            9 => "Shankhachud",
            10 => "Ghatak",
            11 => "Vishdhar",
            12 => "Sheshnag",
            _ => "Unknown",
        };

        let any_conjunct_node = planet_houses.iter().any(|&h| h == rahu || h == ketu);

        Dosha {
            name: if is_kaal_sarpa {
                "Kaal Sarpa Dosha"
            } else {
                "Kaal Amrit Yoga"
            },
            present: true,
            severity: if any_conjunct_node {
                DoshaSeverity::Moderate
            } else {
                DoshaSeverity::Severe
            },
            from_chart: ksd_type,
            cancellations: if any_conjunct_node {
                vec!["Planet conjunct Rahu/Ketu breaks the axis (partial KSD)"]
            } else {
                vec![]
            },
        }
    } else {
        Dosha {
            name: "Kaal Sarpa Dosha",
            present: false,
            severity: DoshaSeverity::None,
            from_chart: "None",
            cancellations: vec![],
        }
    }
}

pub fn detect_pitra_dosha(
    sun_house: usize,
    saturn_house: usize,
    rahu_house: usize,
    _ninth_lord_house: usize,
    ninth_lord_debilitated: bool,
) -> Dosha {
    let mut triggers = Vec::new();

    if sun_house == 9 && saturn_house == 9 {
        triggers.push("Sun-Saturn conjunction in 9th house");
    }
    if rahu_house == 9 {
        triggers.push("Rahu in 9th house");
    }
    if ninth_lord_debilitated {
        triggers.push("9th lord debilitated");
    }
    if saturn_house == 9 {
        triggers.push("Saturn in 9th house");
    }

    Dosha {
        name: "Pitra Dosha",
        present: !triggers.is_empty(),
        severity: if triggers.len() >= 2 {
            DoshaSeverity::Severe
        } else if !triggers.is_empty() {
            DoshaSeverity::Moderate
        } else {
            DoshaSeverity::None
        },
        from_chart: if !triggers.is_empty() {
            "9th house affliction"
        } else {
            "None"
        },
        cancellations: triggers.into_iter().collect(),
    }
}

// ---------------------------------------------------------------------------
// Combustion (Asta / Moudhya) detection — Vedic orbs
// ---------------------------------------------------------------------------

/// Check whether a planet is combust (Asta/Moudhya) per Vedic orbs.
///
/// When a planet's ecliptic longitude is within the combustion orb of the
/// Sun, the planet is considered combust and loses strength.
///
/// Orbs follow the standard Vedic (Surya Siddhanta / BPHS) convention:
///
/// | Planet  | Direct orb | Retrograde orb |
/// |---------|-----------|---------------|
/// | Moon    | 12°       | 12°           |
/// | Mars    | 17°       | 17°           |
/// | Mercury | 14°       | 12°           |
/// | Jupiter | 11°       | 11°           |
/// | Venus   | 10°       |  8°           |
/// | Saturn  | 15°       | 15°           |
///
/// * `planet` — planet name (Sun/Rahu/Ketu always return `false`).
/// * `planet_lon` — planet's ecliptic longitude in degrees.
/// * `sun_lon` — Sun's ecliptic longitude in degrees.
///
/// For planets that have different retrograde orbs (Mercury, Venus), pass
/// `is_retrograde = true` to use the narrower orb.
pub fn is_combust(planet: &str, planet_lon: f64, sun_lon: f64) -> bool {
    is_combust_with_retrograde(planet, planet_lon, sun_lon, false)
}

/// Like [`is_combust`] but with explicit retrograde flag for Mercury/Venus.
pub fn is_combust_with_retrograde(
    planet: &str,
    planet_lon: f64,
    sun_lon: f64,
    is_retrograde: bool,
) -> bool {
    let orb = match (planet, is_retrograde) {
        ("Moon", _) => 12.0,
        ("Mars", _) => 17.0,
        ("Mercury", false) => 14.0,
        ("Mercury", true) => 12.0,
        ("Jupiter", _) => 11.0,
        ("Venus", false) => 10.0,
        ("Venus", true) => 8.0,
        ("Saturn", _) => 15.0,
        // Sun, Rahu, Ketu, and unknown planets cannot be combust
        _ => return false,
    };

    let diff = (planet_lon - sun_lon).rem_euclid(360.0);
    let separation = if diff > 180.0 { 360.0 - diff } else { diff };
    separation < orb
}

/// Detailed combustion status for a planet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombustionStatus {
    /// Planet is not combust — free from Sun's influence.
    Free,
    /// Planet is combust within the standard orb.
    Combust,
    /// Planet is deeply combust (within half the standard orb).
    DeeplyCombust,
}

/// Return detailed combustion status for a planet.
pub fn combustion_status(planet: &str, planet_lon: f64, sun_lon: f64, is_retrograde: bool) -> CombustionStatus {
    let orb = match (planet, is_retrograde) {
        ("Moon", _) => 12.0,
        ("Mars", _) => 17.0,
        ("Mercury", false) => 14.0,
        ("Mercury", true) => 12.0,
        ("Jupiter", _) => 11.0,
        ("Venus", false) => 10.0,
        ("Venus", true) => 8.0,
        ("Saturn", _) => 15.0,
        _ => return CombustionStatus::Free,
    };

    let diff = (planet_lon - sun_lon).rem_euclid(360.0);
    let separation = if diff > 180.0 { 360.0 - diff } else { diff };

    if separation < orb / 2.0 {
        CombustionStatus::DeeplyCombust
    } else if separation < orb {
        CombustionStatus::Combust
    } else {
        CombustionStatus::Free
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mangal_dosha_from_7th() {
        let d = detect_mangal_dosha(7, 3, 5, 0, false, false, false);
        assert!(d.present);
        assert_eq!(d.severity, DoshaSeverity::Moderate);
    }

    #[test]
    fn no_mangal_dosha() {
        let d = detect_mangal_dosha(5, 5, 5, 0, false, false, false);
        assert!(!d.present);
    }

    #[test]
    fn mangal_dosha_cancelled_by_own_sign() {
        let d = detect_mangal_dosha(7, 3, 5, 0, false, true, false);
        assert!(d.present);
        assert_eq!(d.severity, DoshaSeverity::Mild);
        assert!(!d.cancellations.is_empty());
    }

    #[test]
    fn severe_mangal_dosha() {
        let d = detect_mangal_dosha(7, 8, 12, 3, false, false, false);
        assert!(d.present);
        assert_eq!(d.severity, DoshaSeverity::Severe);
    }

    #[test]
    fn kaal_sarpa_present() {
        // Rahu in 1, Ketu in 7, all planets in houses 1-7
        let d = detect_kaal_sarpa(1, 7, &[2, 3, 4, 5, 6, 3, 4]);
        assert!(d.present);
        assert_eq!(d.name, "Kaal Sarpa Dosha");
    }

    #[test]
    fn kaal_sarpa_absent() {
        // Rahu in 1, Ketu in 7, one planet in house 10 (outside axis)
        let d = detect_kaal_sarpa(1, 7, &[2, 3, 10, 5, 6, 3, 4]);
        assert!(!d.present);
    }

    #[test]
    fn pitra_dosha_rahu_in_9th() {
        let d = detect_pitra_dosha(1, 4, 9, 5, false);
        assert!(d.present);
    }

    #[test]
    fn no_pitra_dosha() {
        let d = detect_pitra_dosha(1, 4, 3, 5, false);
        assert!(!d.present);
    }

    // -----------------------------------------------------------------------
    // Combustion tests
    // -----------------------------------------------------------------------

    #[test]
    fn moon_combust_within_12_deg() {
        assert!(is_combust("Moon", 100.0, 110.0)); // 10° < 12°
        assert!(!is_combust("Moon", 100.0, 115.0)); // 15° > 12°
    }

    #[test]
    fn mars_combust_within_17_deg() {
        assert!(is_combust("Mars", 50.0, 65.0)); // 15° < 17°
        assert!(!is_combust("Mars", 50.0, 70.0)); // 20° > 17°
    }

    #[test]
    fn mercury_combust_14_direct_12_retrograde() {
        // Direct: orb = 14°
        assert!(is_combust_with_retrograde("Mercury", 100.0, 113.0, false)); // 13° < 14°
        // Retrograde: orb = 12°
        assert!(!is_combust_with_retrograde("Mercury", 100.0, 113.0, true)); // 13° > 12°
        assert!(is_combust_with_retrograde("Mercury", 100.0, 111.0, true)); // 11° < 12°
    }

    #[test]
    fn venus_combust_10_direct_8_retrograde() {
        // Direct: orb = 10°
        assert!(is_combust_with_retrograde("Venus", 200.0, 209.0, false)); // 9° < 10°
        // Retrograde: orb = 8°
        assert!(!is_combust_with_retrograde("Venus", 200.0, 209.0, true)); // 9° > 8°
        assert!(is_combust_with_retrograde("Venus", 200.0, 207.0, true)); // 7° < 8°
    }

    #[test]
    fn jupiter_combust_within_11_deg() {
        assert!(is_combust("Jupiter", 300.0, 310.0)); // 10° < 11°
        assert!(!is_combust("Jupiter", 300.0, 315.0)); // 15° > 11°
    }

    #[test]
    fn saturn_combust_within_15_deg() {
        assert!(is_combust("Saturn", 0.0, 14.0)); // 14° < 15°
        assert!(!is_combust("Saturn", 0.0, 16.0)); // 16° > 15°
    }

    #[test]
    fn sun_never_combust() {
        assert!(!is_combust("Sun", 100.0, 100.0));
    }

    #[test]
    fn rahu_ketu_never_combust() {
        assert!(!is_combust("Rahu", 100.0, 100.0));
        assert!(!is_combust("Ketu", 100.0, 100.0));
    }

    #[test]
    fn combustion_wraps_around_360() {
        // Mars at 5°, Sun at 355° → separation = 10° < 17°
        assert!(is_combust("Mars", 5.0, 355.0));
    }

    #[test]
    fn combustion_status_levels() {
        // Saturn orb = 15°, deeply combust < 7.5°
        assert_eq!(
            combustion_status("Saturn", 100.0, 105.0, false),
            CombustionStatus::DeeplyCombust
        );
        assert_eq!(
            combustion_status("Saturn", 100.0, 112.0, false),
            CombustionStatus::Combust
        );
        assert_eq!(
            combustion_status("Saturn", 100.0, 120.0, false),
            CombustionStatus::Free
        );
    }

    #[test]
    fn is_combust_default_is_direct() {
        // is_combust uses direct orb (14° for Mercury)
        assert!(is_combust("Mercury", 100.0, 113.0)); // 13° < 14° (direct)
    }
}
