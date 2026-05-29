//! Mrityu Bhaga (Death Degrees) detection.
//!
//! Mrityu Bhaga ("death portion") marks specific degrees in each sign
//! where a planet's energy is considered depleted or obstructed.  A
//! planet placed at or very near its Mrityu Bhaga degree struggles to
//! deliver its significations.
//!
//! The table below is per Jataka Parijata / Sarvartha Chintamani (the
//! most widely used classical source).  Each planet has a different
//! Mrityu Bhaga degree in each of the 12 signs.
//!
//! Orb: traditionally 1° on either side is considered afflicted.
//! Benefic aspects (especially Jupiter's) can mitigate the effect.
//!
//! Sources: Jataka Parijata, Sarvartha Chintamani, Kausika Hora,
//! web-verified against astrojyoti.com and lunarastro.org.

use serde::{Deserialize, Serialize};

/// Practical orb for Mrityu Bhaga: within this many degrees of the
/// exact point, the planet is considered afflicted.
const MRITYU_BHAGA_ORB: f64 = 1.0;

/// Mrityu Bhaga degrees by planet, indexed [sign_idx] where 0=Aries.
///
/// Columns: Aries, Taurus, Gemini, Cancer, Leo, Virgo,
///          Libra, Scorpio, Sagittarius, Capricorn, Aquarius, Pisces.
///
/// Source: Jataka Parijata / Sarvartha Chintamani.
const SUN_MB: [f64; 12] = [20.0, 9.0, 12.0, 6.0, 8.0, 24.0, 16.0, 17.0, 22.0, 2.0, 3.0, 23.0];
const MOON_MB: [f64; 12] =
    [8.0, 25.0, 22.0, 22.0, 21.0, 1.0, 4.0, 23.0, 18.0, 20.0, 20.0, 10.0];
const MARS_MB: [f64; 12] =
    [19.0, 28.0, 25.0, 23.0, 29.0, 28.0, 14.0, 21.0, 2.0, 15.0, 11.0, 6.0];
const MERCURY_MB: [f64; 12] =
    [15.0, 14.0, 13.0, 12.0, 8.0, 18.0, 20.0, 10.0, 21.0, 22.0, 7.0, 5.0];
const JUPITER_MB: [f64; 12] =
    [19.0, 29.0, 12.0, 27.0, 6.0, 4.0, 13.0, 10.0, 17.0, 11.0, 15.0, 28.0];
const VENUS_MB: [f64; 12] =
    [28.0, 15.0, 11.0, 17.0, 10.0, 13.0, 4.0, 6.0, 27.0, 12.0, 29.0, 19.0];
const SATURN_MB: [f64; 12] =
    [10.0, 4.0, 7.0, 9.0, 12.0, 16.0, 3.0, 18.0, 28.0, 14.0, 13.0, 15.0];

/// Lagna (Ascendant) Mrityu Bhaga degrees — sometimes used for the
/// Ascendant degree itself, per Kausika Hora.
const LAGNA_MB: [f64; 12] =
    [1.0, 9.0, 21.0, 22.0, 25.0, 2.0, 4.0, 23.0, 18.0, 20.0, 24.0, 10.0];

/// Mrityu Bhaga analysis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MrityuBhagaInfo {
    /// Whether the planet is within orb of its Mrityu Bhaga.
    pub is_mrityu_bhaga: bool,
    /// The exact Mrityu Bhaga degree for this planet in this sign.
    pub mb_degree: f64,
    /// Distance from the exact Mrityu Bhaga point.
    pub distance_deg: f64,
}

/// Look up the Mrityu Bhaga table for a planet.
fn mb_table(planet: &str) -> Option<&'static [f64; 12]> {
    match planet {
        "Sun" => Some(&SUN_MB),
        "Moon" => Some(&MOON_MB),
        "Mars" => Some(&MARS_MB),
        "Mercury" => Some(&MERCURY_MB),
        "Jupiter" => Some(&JUPITER_MB),
        "Venus" => Some(&VENUS_MB),
        "Saturn" => Some(&SATURN_MB),
        "Lagna" | "Ascendant" => Some(&LAGNA_MB),
        _ => None,
    }
}

/// Check whether a planet is at its Mrityu Bhaga in a given sidereal
/// longitude.
///
/// Returns `true` if the planet's position (degree within its sign) is
/// within 1° of the Mrityu Bhaga point for that planet and sign.
///
/// Returns `false` for Rahu, Ketu, or unrecognized planet names (they
/// have no standard Mrityu Bhaga assignment in Jataka Parijata).
pub fn is_mrityu_bhaga(planet: &str, sidereal_lon_deg: f64) -> bool {
    let table = match mb_table(planet) {
        Some(t) => t,
        None => return false,
    };
    let lon = sidereal_lon_deg.rem_euclid(360.0);
    let sign_idx = (lon / 30.0) as usize % 12;
    let deg_in_sign = lon % 30.0;
    let mb_deg = table[sign_idx];
    (deg_in_sign - mb_deg).abs() <= MRITYU_BHAGA_ORB
}

/// Return the Mrityu Bhaga degree for a planet in a given sign index.
///
/// Returns `None` for unrecognized planet names.
pub fn mrityu_bhaga_degree(planet: &str, sign_idx: usize) -> Option<f64> {
    mb_table(planet).map(|t| t[sign_idx % 12])
}

/// Full Mrityu Bhaga analysis.
pub fn mrityu_bhaga_info(planet: &str, sidereal_lon_deg: f64) -> Option<MrityuBhagaInfo> {
    let table = mb_table(planet)?;
    let lon = sidereal_lon_deg.rem_euclid(360.0);
    let sign_idx = (lon / 30.0) as usize % 12;
    let deg_in_sign = lon % 30.0;
    let mb_deg = table[sign_idx];
    let distance = (deg_in_sign - mb_deg).abs();
    Some(MrityuBhagaInfo {
        is_mrityu_bhaga: distance <= MRITYU_BHAGA_ORB,
        mb_degree: mb_deg,
        distance_deg: distance,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Sun ----

    #[test]
    fn sun_mb_aries_20() {
        // Sun's MB in Aries is 20°
        assert!(is_mrityu_bhaga("Sun", 20.0)); // exact
        assert!(is_mrityu_bhaga("Sun", 19.5)); // within orb
        assert!(is_mrityu_bhaga("Sun", 20.9)); // within orb
        assert!(!is_mrityu_bhaga("Sun", 18.0)); // outside
        assert!(!is_mrityu_bhaga("Sun", 22.0)); // outside
    }

    #[test]
    fn sun_mb_capricorn_2() {
        // Sun's MB in Capricorn is 2° → absolute lon = 270° + 2° = 272°
        assert!(is_mrityu_bhaga("Sun", 272.0));
        assert!(!is_mrityu_bhaga("Sun", 275.0));
    }

    // ---- Moon ----

    #[test]
    fn moon_mb_aries_8() {
        assert!(is_mrityu_bhaga("Moon", 8.0));
        assert!(!is_mrityu_bhaga("Moon", 10.0));
    }

    #[test]
    fn moon_mb_taurus_25() {
        // 25° Taurus = 30° + 25° = 55°
        assert!(is_mrityu_bhaga("Moon", 55.0));
    }

    #[test]
    fn moon_mb_virgo_1() {
        // Moon MB in Virgo is 1° → absolute 150° + 1° = 151°
        assert!(is_mrityu_bhaga("Moon", 151.0));
        assert!(!is_mrityu_bhaga("Moon", 155.0));
    }

    // ---- Mars ----

    #[test]
    fn mars_mb_aries_19() {
        assert!(is_mrityu_bhaga("Mars", 19.0));
    }

    #[test]
    fn mars_mb_sagittarius_2() {
        // Mars MB in Sagittarius = 2° → 240° + 2° = 242°
        assert!(is_mrityu_bhaga("Mars", 242.0));
    }

    // ---- Mercury ----

    #[test]
    fn mercury_mb_aries_15() {
        assert!(is_mrityu_bhaga("Mercury", 15.0));
    }

    #[test]
    fn mercury_mb_pisces_5() {
        // 5° Pisces = 330° + 5° = 335°
        assert!(is_mrityu_bhaga("Mercury", 335.0));
    }

    // ---- Jupiter ----

    #[test]
    fn jupiter_mb_cancer_27() {
        // Jupiter MB in Cancer = 27° → 90° + 27° = 117°
        assert!(is_mrityu_bhaga("Jupiter", 117.0));
    }

    #[test]
    fn jupiter_mb_leo_6() {
        // Jupiter MB in Leo = 6° → 120° + 6° = 126°
        assert!(is_mrityu_bhaga("Jupiter", 126.0));
    }

    // ---- Venus ----

    #[test]
    fn venus_mb_aries_28() {
        assert!(is_mrityu_bhaga("Venus", 28.0));
    }

    #[test]
    fn venus_mb_aquarius_29() {
        // Venus MB in Aquarius = 29° → 300° + 29° = 329°
        assert!(is_mrityu_bhaga("Venus", 329.0));
    }

    // ---- Saturn ----

    #[test]
    fn saturn_mb_aries_10() {
        assert!(is_mrityu_bhaga("Saturn", 10.0));
    }

    #[test]
    fn saturn_mb_cancer_9() {
        // Saturn MB in Cancer = 9° → 90° + 9° = 99°
        assert!(is_mrityu_bhaga("Saturn", 99.0));
    }

    // ---- Lagna ----

    #[test]
    fn lagna_mb_aries_1() {
        assert!(is_mrityu_bhaga("Lagna", 1.0));
        assert!(is_mrityu_bhaga("Ascendant", 1.0));
    }

    // ---- Unknown planets ----

    #[test]
    fn rahu_has_no_mb() {
        assert!(!is_mrityu_bhaga("Rahu", 100.0));
    }

    #[test]
    fn ketu_has_no_mb() {
        assert!(!is_mrityu_bhaga("Ketu", 200.0));
    }

    #[test]
    fn unknown_planet() {
        assert!(!is_mrityu_bhaga("Pluto", 50.0));
    }

    // ---- Degree lookup ----

    #[test]
    fn mrityu_bhaga_degree_lookup() {
        assert_eq!(mrityu_bhaga_degree("Sun", 0), Some(20.0)); // Aries
        assert_eq!(mrityu_bhaga_degree("Moon", 3), Some(22.0)); // Cancer
        assert_eq!(mrityu_bhaga_degree("Jupiter", 3), Some(27.0)); // Cancer
        assert_eq!(mrityu_bhaga_degree("Saturn", 3), Some(9.0)); // Cancer
        assert_eq!(mrityu_bhaga_degree("Rahu", 0), None);
    }

    // ---- MrityuBhagaInfo ----

    #[test]
    fn info_struct_hit() {
        let info = mrityu_bhaga_info("Sun", 20.0).unwrap();
        assert!(info.is_mrityu_bhaga);
        assert!((info.mb_degree - 20.0).abs() < 0.001);
        assert!(info.distance_deg < 0.001);
    }

    #[test]
    fn info_struct_miss() {
        let info = mrityu_bhaga_info("Sun", 10.0).unwrap();
        assert!(!info.is_mrityu_bhaga);
        assert!((info.mb_degree - 20.0).abs() < 0.001);
        assert!((info.distance_deg - 10.0).abs() < 0.001);
    }

    #[test]
    fn info_unknown_planet() {
        assert!(mrityu_bhaga_info("Rahu", 100.0).is_none());
    }

    // ---- Normalization ----

    #[test]
    fn negative_longitude() {
        // -10° = 350° = 20° Pisces. Sun MB Pisces = 23°. Distance = 3° → not MB
        assert!(!is_mrityu_bhaga("Sun", -10.0));
    }

    #[test]
    fn over_360() {
        // 380° = 20° Aries. Sun MB Aries = 20° → hit
        assert!(is_mrityu_bhaga("Sun", 380.0));
    }

    // ---- Cross-check: each planet has distinct MB per sign ----

    #[test]
    fn all_seven_planets_have_all_12_entries() {
        let planets = ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"];
        for planet in &planets {
            for sign in 0..12 {
                let deg = mrityu_bhaga_degree(planet, sign);
                assert!(
                    deg.is_some(),
                    "{planet} should have MB for sign {sign}"
                );
                let d = deg.unwrap();
                assert!(
                    (0.0..30.0).contains(&d),
                    "{planet} MB in sign {sign} = {d}° is out of range"
                );
            }
        }
    }

    // ---- Venus table is mirror of Jupiter ----
    // Per Jataka Parijata, Venus MB = reverse of Jupiter MB.
    // Venus: 28,15,11,17,10,13, 4, 6,27,12,29,19
    // Jupiter: 19,29,12,27, 6, 4,13,10,17,11,15,28
    // Venus[i] == Jupiter[11-i]

    #[test]
    fn venus_jupiter_mirror_symmetry() {
        for i in 0..12 {
            let v = mrityu_bhaga_degree("Venus", i).unwrap();
            let j = mrityu_bhaga_degree("Jupiter", 11 - i).unwrap();
            assert!(
                (v - j).abs() < 0.001,
                "Venus[{i}]={v} should equal Jupiter[{}]={j}",
                11 - i
            );
        }
    }
}
