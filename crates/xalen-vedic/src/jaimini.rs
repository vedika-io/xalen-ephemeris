use crate::rashi::Rashi;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharaKaraka {
    Atmakaraka,    // AK — highest degree, soul significator
    Amatyakaraka,  // AmK — minister
    Bhratrikaraka, // BK — siblings
    Matrikaraka,   // MK — mother
    Putrakaraka,   // PK — children
    Gnatikaraka,   // GK — relatives/enemies
    Darakaraka,    // DK — spouse (lowest degree)
}

impl CharaKaraka {
    pub const ALL_7: [CharaKaraka; 7] = [
        CharaKaraka::Atmakaraka,
        CharaKaraka::Amatyakaraka,
        CharaKaraka::Bhratrikaraka,
        CharaKaraka::Matrikaraka,
        CharaKaraka::Putrakaraka,
        CharaKaraka::Gnatikaraka,
        CharaKaraka::Darakaraka,
    ];
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KarakaAssignment {
    pub planet: String,
    pub karaka: CharaKaraka,
    pub degree_in_sign: f64,
}

pub fn assign_chara_karakas(
    planet_degrees: &[(&str, f64)], // (planet_name, sidereal_longitude)
) -> Vec<KarakaAssignment> {
    // Sort by degree within sign (descending) — highest = AK, lowest = DK
    let planets: Vec<(&str, f64)> = planet_degrees
        .iter()
        .filter(|(name, _)| !matches!(*name, "Rahu" | "Ketu")) // exclude nodes
        .copied()
        .collect();

    // Get degree within sign for each planet
    let mut with_deg: Vec<(&str, f64, f64)> = planets
        .iter()
        .map(|(name, lon)| (*name, *lon, lon.rem_euclid(30.0)))
        .collect();

    // Sort descending by degree in sign
    with_deg.sort_by(|a, b| b.2.total_cmp(&a.2));

    with_deg
        .iter()
        .take(7)
        .enumerate()
        .map(|(i, (name, _lon, deg))| KarakaAssignment {
            planet: name.to_string(),
            karaka: CharaKaraka::ALL_7[i],
            degree_in_sign: *deg,
        })
        .collect()
}

pub fn jaimini_aspects(sign: Rashi) -> Vec<Rashi> {
    let idx = sign.index();
    let modality = idx % 3;
    match modality {
        0 => {
            // Movable: aspects all fixed EXCEPT adjacent
            (0..12)
                .filter(|&i| i % 3 == 1 && ((i as i32 - idx as i32).abs() % 12 != 1))
                .map(Rashi::from_index)
                .collect()
        }
        1 => {
            // Fixed: aspects all movable EXCEPT adjacent
            (0..12)
                .filter(|&i| i % 3 == 0 && ((i as i32 - idx as i32).abs() % 12 != 1))
                .map(Rashi::from_index)
                .collect()
        }
        _ => {
            // Dual: aspects all other dual signs
            (0..12)
                .filter(|&i| i % 3 == 2 && i != idx)
                .map(Rashi::from_index)
                .collect()
        }
    }
}

/// Arudha Pada of a bhava: count from the bhava to its lord, then the same
/// number again from the lord. Per BPHS/Jaimini, the Arudha can never fall in
/// the bhava itself (1st) or the 7th from it — "the 1st and 7th are truth, not
/// illusion". When the provisional pada lands in either, the **10th sign from
/// the provisional pada** is taken instead. Verified 2026-05-28.
///
/// Worked consequences (with `d` = lord's distance from the bhava):
///   d = 0 or 6  -> provisional = bhava       -> 10th from bhava
///   d = 3 or 9  -> provisional = 7th-from-bhava -> 4th from bhava (= 10th from 7th)
///   otherwise   -> provisional pada unchanged
pub fn arudha_pada(house_sign: usize, lord_sign: usize) -> usize {
    let distance = (lord_sign as i32 - house_sign as i32).rem_euclid(12) as usize;
    let provisional = (lord_sign + distance) % 12; // = (house + 2d) mod 12
    let from_house = (provisional as i32 - house_sign as i32).rem_euclid(12);
    if from_house == 0 || from_house == 6 {
        // Provisional pada is the 1st or 7th from the bhava: take the 10th from it.
        (provisional + 9) % 12
    } else {
        provisional
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chara_karakas_7_assigned() {
        let planets = vec![
            ("Sun", 10.5),
            ("Moon", 73.2),
            ("Mars", 298.8),
            ("Mercury", 265.1),
            ("Jupiter", 95.4),
            ("Venus", 327.9),
            ("Saturn", 200.3),
        ];
        let karakas = assign_chara_karakas(&planets);
        assert_eq!(karakas.len(), 7);
        assert_eq!(karakas[0].karaka, CharaKaraka::Atmakaraka);
        assert_eq!(karakas[6].karaka, CharaKaraka::Darakaraka);
    }

    #[test]
    fn ak_has_highest_degree_in_sign() {
        let planets = vec![
            ("Sun", 29.5),
            ("Moon", 15.0),
            ("Mars", 20.0),
            ("Mercury", 5.0),
            ("Jupiter", 10.0),
            ("Venus", 25.0),
            ("Saturn", 1.0),
        ];
        let karakas = assign_chara_karakas(&planets);
        assert_eq!(karakas[0].planet, "Sun"); // 29.5° in sign = highest
        assert_eq!(karakas[0].karaka, CharaKaraka::Atmakaraka);
    }

    #[test]
    fn nodes_excluded() {
        let planets = vec![
            ("Sun", 10.0),
            ("Moon", 15.0),
            ("Mars", 20.0),
            ("Mercury", 5.0),
            ("Jupiter", 25.0),
            ("Venus", 12.0),
            ("Saturn", 8.0),
            ("Rahu", 28.0),
            ("Ketu", 28.0),
        ];
        let karakas = assign_chara_karakas(&planets);
        assert_eq!(karakas.len(), 7); // Rahu/Ketu excluded
        assert!(!karakas.iter().any(|k| k.planet == "Rahu"));
    }

    #[test]
    fn jaimini_movable_aspects_fixed() {
        let aspects = jaimini_aspects(Rashi::Mesha); // Aries (movable)
        // Should aspect fixed signs except adjacent (Taurus)
        assert!(aspects.contains(&Rashi::Simha)); // Leo (fixed)
        assert!(aspects.contains(&Rashi::Vrishchika)); // Scorpio (fixed)
        assert!(aspects.contains(&Rashi::Kumbha)); // Aquarius (fixed)
        assert!(!aspects.contains(&Rashi::Vrishabha)); // Taurus excluded (adjacent)
    }

    #[test]
    fn jaimini_dual_aspects_dual() {
        let aspects = jaimini_aspects(Rashi::Mithuna); // Gemini (dual)
        assert!(aspects.contains(&Rashi::Kanya)); // Virgo
        assert!(aspects.contains(&Rashi::Dhanu)); // Sagittarius
        assert!(aspects.contains(&Rashi::Meena)); // Pisces
        assert!(!aspects.contains(&Rashi::Mithuna)); // Not self
    }

    #[test]
    fn arudha_pada_general() {
        // House sign=0 (Aries), Lord in sign=4 (Leo): distance=4
        // Arudha = 4 + 4 = 8 (Sagittarius)
        assert_eq!(arudha_pada(0, 4), 8);
    }

    #[test]
    fn arudha_pada_provisional_in_7th_takes_10th_from_it() {
        // House=0, lord in 4th (distance=3): provisional pada = house+6 = 7th
        // (sign 6). Exception -> 10th from provisional = (6+9)%12 = 3 (4th from
        // house, i.e. the 10th from the 7th). Per BPHS.
        assert_eq!(arudha_pada(0, 3), 3);
    }

    #[test]
    fn arudha_pada_provisional_in_bhava_takes_10th_from_it() {
        // House=0, lord in 7th (distance=6): provisional pada = house (sign 0).
        // Exception -> 10th from provisional = sign 9 (10th from house). Per BPHS.
        assert_eq!(arudha_pada(0, 6), 9);
    }

    #[test]
    fn arudha_pada_lord_in_lagna_takes_10th() {
        // Lord in the house itself (distance=0): provisional = house -> 10th from it.
        assert_eq!(arudha_pada(0, 0), 9);
    }

    #[test]
    fn arudha_pada_never_lands_in_1st_or_7th() {
        // The Arudha can never be the bhava (1st) or the 7th from it.
        for house in 0..12usize {
            for lord in 0..12usize {
                let pada = arudha_pada(house, lord);
                let from_house = (pada as i32 - house as i32).rem_euclid(12);
                assert!(
                    from_house != 0 && from_house != 6,
                    "house={house} lord={lord} gave pada in {from_house}th (forbidden)"
                );
            }
        }
    }
}
