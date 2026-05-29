//! Pushkara Navamsa and Pushkara Bhaga detection.
//!
//! ## Pushkara Navamsa
//!
//! Each sign has 9 navamsa divisions (3°20' each). Of these, exactly 2
//! per sign are considered "Pushkara" — navamsas ruled by benefic planets
//! (Moon, Mercury, Jupiter, Venus). A planet placed in a Pushkara Navamsa
//! is strengthened and gives auspicious results.
//!
//! The pattern repeats by element:
//! - Fire signs (Aries, Leo, Sagittarius): navamsas 7 & 9 (i.e. 20°00'–23°20' and 26°40'–30°00')
//! - Earth signs (Taurus, Virgo, Capricorn): navamsas 3 & 5 (i.e. 6°40'–10°00' and 13°20'–16°40')
//! - Air signs (Gemini, Libra, Aquarius): navamsas 6 & 8 (i.e. 16°40'–20°00' and 23°20'–26°40')
//! - Water signs (Cancer, Scorpio, Pisces): navamsas 1 & 3 (i.e. 0°00'–3°20' and 6°40'–10°00')
//!
//! 24 Pushkara navamsas total across the zodiac.
//!
//! ## Pushkara Bhaga
//!
//! One specific degree per sign is designated as "Pushkara Bhaga" —
//! an extremely auspicious point. A planet at that exact degree (within
//! a practical orb of ~1°) is considered highly fortunate.
//!
//! Per Jataka Parijata (the most widely used source):
//! Aries 21°, Taurus 14°, Gemini 18°, Cancer 8°, Leo 19°, Virgo 9°,
//! Libra 24°, Scorpio 11°, Sagittarius 23°, Capricorn 14°, Aquarius 19°,
//! Pisces 9°.
//!
//! Sources: Jataka Parijata, Komilla Sutton, Sanjay Rath.

use serde::{Deserialize, Serialize};

/// Navamsa span in degrees: 30°/9 = 3.3333...°
const NAVAMSA_SPAN: f64 = 30.0 / 9.0;

/// Pushkara Bhaga orb: a planet within this many degrees of the exact
/// Pushkara Bhaga point is considered to be at that bhaga.
const PUSHKARA_BHAGA_ORB: f64 = 1.0;

/// Pushkara Navamsa indices (1-based) by element.
/// Fire  → 7th and 9th navamsa
/// Earth → 3rd and 5th navamsa
/// Air   → 6th and 8th navamsa
/// Water → 1st and 3rd navamsa
const PUSHKARA_NAVAMSA_BY_ELEMENT: [[usize; 2]; 4] = [
    [7, 9], // Fire:  Aries(0), Leo(4), Sagittarius(8)
    [3, 5], // Earth: Taurus(1), Virgo(5), Capricorn(9)
    [6, 8], // Air:   Gemini(2), Libra(6), Aquarius(10)
    [1, 3], // Water: Cancer(3), Scorpio(7), Pisces(11)
];

/// Pushkara Bhaga degree for each sign (0=Aries through 11=Pisces).
/// Source: Jataka Parijata.
const PUSHKARA_BHAGA_DEGREES: [f64; 12] = [
    21.0, // Aries
    14.0, // Taurus
    18.0, // Gemini
    8.0,  // Cancer
    19.0, // Leo
    9.0,  // Virgo
    24.0, // Libra
    11.0, // Scorpio
    23.0, // Sagittarius
    14.0, // Capricorn
    19.0, // Aquarius
    9.0,  // Pisces
];

/// Result of Pushkara analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushkaraInfo {
    pub is_pushkara_navamsa: bool,
    pub is_pushkara_bhaga: bool,
    /// Which navamsa number (1-9) the planet occupies in its sign.
    pub navamsa_number: usize,
    /// Distance from the Pushkara Bhaga point (always positive).
    pub bhaga_distance_deg: f64,
}

/// Check whether a sidereal longitude falls in a Pushkara Navamsa.
///
/// Returns `true` if the position is in one of the 24 Pushkara Navamsa
/// divisions across the zodiac.
pub fn is_pushkara_navamsa(sidereal_lon_deg: f64) -> bool {
    let lon = sidereal_lon_deg.rem_euclid(360.0);
    let sign_idx = (lon / 30.0) as usize;
    let deg_in_sign = lon % 30.0;

    // Which of the 9 navamsas in this sign (1-based)
    let navamsa_num = (deg_in_sign / NAVAMSA_SPAN) as usize + 1;

    // Element: Fire=0, Earth=1, Air=2, Water=3
    let element = sign_idx % 4;
    let pushkara_pair = PUSHKARA_NAVAMSA_BY_ELEMENT[element];

    navamsa_num == pushkara_pair[0] || navamsa_num == pushkara_pair[1]
}

/// Check whether a sidereal longitude is at a Pushkara Bhaga degree.
///
/// Uses a 1° orb on either side of the exact Pushkara Bhaga degree.
/// For example, if the Pushkara Bhaga for Aries is 21°, any planet
/// between 20° and 22° Aries qualifies.
pub fn is_pushkara_bhaga(sidereal_lon_deg: f64) -> bool {
    let lon = sidereal_lon_deg.rem_euclid(360.0);
    let sign_idx = (lon / 30.0) as usize % 12;
    let deg_in_sign = lon % 30.0;
    let bhaga = PUSHKARA_BHAGA_DEGREES[sign_idx];
    (deg_in_sign - bhaga).abs() <= PUSHKARA_BHAGA_ORB
}

/// Return the Pushkara Bhaga degree for a given sign index (0=Aries).
pub fn pushkara_bhaga_degree(sign_idx: usize) -> f64 {
    PUSHKARA_BHAGA_DEGREES[sign_idx % 12]
}

/// Full Pushkara analysis for a given sidereal longitude.
pub fn pushkara_info(sidereal_lon_deg: f64) -> PushkaraInfo {
    let lon = sidereal_lon_deg.rem_euclid(360.0);
    let sign_idx = (lon / 30.0) as usize % 12;
    let deg_in_sign = lon % 30.0;
    let navamsa_num = (deg_in_sign / NAVAMSA_SPAN) as usize + 1;
    let bhaga = PUSHKARA_BHAGA_DEGREES[sign_idx];

    PushkaraInfo {
        is_pushkara_navamsa: is_pushkara_navamsa(sidereal_lon_deg),
        is_pushkara_bhaga: (deg_in_sign - bhaga).abs() <= PUSHKARA_BHAGA_ORB,
        navamsa_number: navamsa_num,
        bhaga_distance_deg: (deg_in_sign - bhaga).abs(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Pushkara Navamsa: Fire signs (navamsas 7 & 9) ----

    #[test]
    fn fire_sign_aries_navamsa_7() {
        // Navamsa 7 = 20°00'–23°20' in sign
        // 21° Aries = absolute 21°
        assert!(is_pushkara_navamsa(21.0));
    }

    #[test]
    fn fire_sign_aries_navamsa_9() {
        // Navamsa 9 = 26°40'–30°00'
        // 28° Aries = absolute 28°
        assert!(is_pushkara_navamsa(28.0));
    }

    #[test]
    fn fire_sign_aries_navamsa_1_not_pushkara() {
        // Navamsa 1 = 0°–3°20'
        assert!(!is_pushkara_navamsa(1.0));
    }

    #[test]
    fn fire_sign_leo_navamsa_7() {
        // Leo starts at 120°, navamsa 7 = 120° + 20° = 140°
        assert!(is_pushkara_navamsa(141.0));
    }

    #[test]
    fn fire_sign_sagittarius_navamsa_9() {
        // Sagittarius starts at 240°, navamsa 9 = 240° + 26.667° = 266.667°
        assert!(is_pushkara_navamsa(268.0));
    }

    // ---- Pushkara Navamsa: Earth signs (navamsas 3 & 5) ----

    #[test]
    fn earth_sign_taurus_navamsa_3() {
        // Taurus starts at 30°, navamsa 3 = 30° + 6.667° = 36.667°
        // 38° = 8° Taurus → navamsa 3 (6.667°–10°)
        assert!(is_pushkara_navamsa(38.0));
    }

    #[test]
    fn earth_sign_taurus_navamsa_5() {
        // Navamsa 5 = 13.333°–16.667°
        // 45° = 15° Taurus → navamsa 5
        assert!(is_pushkara_navamsa(45.0));
    }

    #[test]
    fn earth_sign_virgo_navamsa_3() {
        // Virgo starts at 150°, navamsa 3 = 150° + 6.667° = 156.667°
        assert!(is_pushkara_navamsa(158.0));
    }

    // ---- Pushkara Navamsa: Air signs (navamsas 6 & 8) ----

    #[test]
    fn air_sign_gemini_navamsa_6() {
        // Gemini starts at 60°, navamsa 6 = 60° + 16.667° = 76.667°
        // 78° = 18° Gemini → navamsa 6
        assert!(is_pushkara_navamsa(78.0));
    }

    #[test]
    fn air_sign_libra_navamsa_8() {
        // Libra starts at 180°, navamsa 8 = 180° + 23.333° = 203.333°
        // 205° = 25° Libra → navamsa 8
        assert!(is_pushkara_navamsa(205.0));
    }

    // ---- Pushkara Navamsa: Water signs (navamsas 1 & 3) ----

    #[test]
    fn water_sign_cancer_navamsa_1() {
        // Cancer starts at 90°, navamsa 1 = 90° + 0° = 90°
        assert!(is_pushkara_navamsa(91.0));
    }

    #[test]
    fn water_sign_cancer_navamsa_3() {
        // Cancer navamsa 3 = 90° + 6.667° = 96.667°
        assert!(is_pushkara_navamsa(98.0));
    }

    #[test]
    fn water_sign_scorpio_navamsa_1() {
        // Scorpio starts at 210°
        assert!(is_pushkara_navamsa(211.0));
    }

    #[test]
    fn water_sign_pisces_navamsa_1() {
        // Pisces starts at 330°
        assert!(is_pushkara_navamsa(331.0));
    }

    // ---- Non-Pushkara navamsa spots ----

    #[test]
    fn fire_sign_middle_navamsas_not_pushkara() {
        // Aries navamsas 2,3,4,5,6,8 should NOT be pushkara
        // Navamsa 4 = 10°–13.333° → test 11° Aries
        assert!(!is_pushkara_navamsa(11.0));
        // Navamsa 5 = 13.333°–16.667° → test 15° Aries
        assert!(!is_pushkara_navamsa(15.0));
    }

    // ---- Pushkara Bhaga ----

    #[test]
    fn pushkara_bhaga_aries_21() {
        // Exact hit
        assert!(is_pushkara_bhaga(21.0));
        // Within 1° orb
        assert!(is_pushkara_bhaga(20.5));
        assert!(is_pushkara_bhaga(21.9));
        // Outside orb
        assert!(!is_pushkara_bhaga(19.0));
        assert!(!is_pushkara_bhaga(23.0));
    }

    #[test]
    fn pushkara_bhaga_taurus_14() {
        // 14° Taurus = 30° + 14° = 44°
        assert!(is_pushkara_bhaga(44.0));
        assert!(is_pushkara_bhaga(43.5));
        assert!(!is_pushkara_bhaga(42.0));
    }

    #[test]
    fn pushkara_bhaga_cancer_8() {
        // 8° Cancer = 90° + 8° = 98°
        assert!(is_pushkara_bhaga(98.0));
    }

    #[test]
    fn pushkara_bhaga_libra_24() {
        // 24° Libra = 180° + 24° = 204°
        assert!(is_pushkara_bhaga(204.0));
    }

    #[test]
    fn pushkara_bhaga_pisces_9() {
        // 9° Pisces = 330° + 9° = 339°
        assert!(is_pushkara_bhaga(339.0));
    }

    #[test]
    fn pushkara_bhaga_all_signs_have_one() {
        for sign_idx in 0..12 {
            let bhaga = pushkara_bhaga_degree(sign_idx);
            let abs_lon = sign_idx as f64 * 30.0 + bhaga;
            assert!(
                is_pushkara_bhaga(abs_lon),
                "Sign {sign_idx} pushkara bhaga at {bhaga}° should be detected at {abs_lon}°"
            );
        }
    }

    // ---- PushkaraInfo ----

    #[test]
    fn pushkara_info_both_hits() {
        // 21° Aries is BOTH pushkara navamsa (navamsa 7) AND pushkara bhaga
        let info = pushkara_info(21.0);
        assert!(info.is_pushkara_navamsa);
        assert!(info.is_pushkara_bhaga);
        assert_eq!(info.navamsa_number, 7);
        assert!(info.bhaga_distance_deg < 0.001);
    }

    #[test]
    fn pushkara_info_navamsa_only() {
        // 28° Aries: pushkara navamsa (9th) but not pushkara bhaga (21°)
        let info = pushkara_info(28.0);
        assert!(info.is_pushkara_navamsa);
        assert!(!info.is_pushkara_bhaga);
    }

    #[test]
    fn pushkara_info_neither() {
        // 10° Aries: navamsa 4 (not pushkara), bhaga distance = 11°
        let info = pushkara_info(10.0);
        assert!(!info.is_pushkara_navamsa);
        assert!(!info.is_pushkara_bhaga);
    }

    // ---- Normalization ----

    #[test]
    fn negative_longitude() {
        // -1° = 359° = 29° Pisces
        // Pisces pushkara navamsas are 1 & 3 → 29° = navamsa 9 → not pushkara
        assert!(!is_pushkara_navamsa(-1.0));
    }

    #[test]
    fn over_360() {
        // 381° = 21° Aries
        assert!(is_pushkara_navamsa(381.0));
        assert!(is_pushkara_bhaga(381.0));
    }

    // ---- Count total Pushkara Navamsas ----

    #[test]
    fn total_pushkara_navamsas_is_24() {
        let mut count = 0;
        // Test at the midpoint of each of the 108 navamsas (12 signs * 9 navamsas)
        for sign in 0..12 {
            for nav in 0..9 {
                let sign_start = sign as f64 * 30.0;
                let nav_mid = sign_start + (nav as f64 + 0.5) * NAVAMSA_SPAN;
                if is_pushkara_navamsa(nav_mid) {
                    count += 1;
                }
            }
        }
        assert_eq!(count, 24, "There should be exactly 24 Pushkara Navamsas");
    }
}
