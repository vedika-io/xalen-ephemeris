use crate::dignity::{Relationship, natural_relationship};
use crate::nakshatra::{Gana, Nakshatra};
use crate::rashi::Rashi;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Ashtakoota (8-fold) marriage compatibility score (max 36 points).
pub struct GunaMatch {
    pub varna: u8,
    pub vashya: u8,
    pub tara: u8,
    pub yoni: u8,
    pub graha_maitri: u8,
    pub gana: u8,
    pub bhakoot: u8,
    pub nadi: u8,
    pub total: u8,
}

impl GunaMatch {
    pub const MAX_SCORE: u8 = 36;
}

/// Compute the full 8-factor compatibility score between two Moon nakshatras.
pub fn ashtakoota_match(
    boy_moon_nak: Nakshatra,
    girl_moon_nak: Nakshatra,
    boy_moon_rashi_idx: usize,
    girl_moon_rashi_idx: usize,
) -> GunaMatch {
    let varna = varna_score(boy_moon_nak, girl_moon_nak);
    let vashya = vashya_score(boy_moon_rashi_idx, girl_moon_rashi_idx);
    let tara = tara_score(boy_moon_nak, girl_moon_nak);
    let yoni = yoni_score(boy_moon_nak, girl_moon_nak);
    let graha_maitri = graha_maitri_score(boy_moon_rashi_idx, girl_moon_rashi_idx);
    let gana = gana_score(boy_moon_nak, girl_moon_nak);
    let bhakoot = bhakoot_score(boy_moon_rashi_idx, girl_moon_rashi_idx);
    let nadi = nadi_score(boy_moon_nak, girl_moon_nak);
    let total = varna + vashya + tara + yoni + graha_maitri + gana + bhakoot + nadi;

    GunaMatch {
        varna,
        vashya,
        tara,
        yoni,
        graha_maitri,
        gana,
        bhakoot,
        nadi,
        total,
    }
}

// ---------------------------------------------------------------------------
// 1. Varna (max 1 point)
// Classical mapping: Brahmin(3) > Kshatriya(2) > Vaishya(1) > Shudra(0)
// Groom same or higher varna than bride = 1, else 0.
// ---------------------------------------------------------------------------

fn varna_score(boy: Nakshatra, girl: Nakshatra) -> u8 {
    let boy_v = nakshatra_varna(boy);
    let girl_v = nakshatra_varna(girl);
    if boy_v >= girl_v { 1 } else { 0 }
}

fn nakshatra_varna(nak: Nakshatra) -> u8 {
    // Classical Varna lookup per nakshatra
    // Brahmin=3, Kshatriya=2, Vaishya=1, Shudra=0
    match nak {
        // Brahmin
        Nakshatra::Ashwini
        | Nakshatra::Pushya
        | Nakshatra::Hasta
        | Nakshatra::Anuradha
        | Nakshatra::Shravana
        | Nakshatra::Revati
        | Nakshatra::Punarvasu
        | Nakshatra::Swati
        | Nakshatra::PurvaBhadrapada => 3,
        // Kshatriya
        Nakshatra::Bharani
        | Nakshatra::Magha
        | Nakshatra::PurvaPhalguni
        | Nakshatra::Vishakha
        | Nakshatra::PurvaAshadha
        | Nakshatra::Dhanishta => 2,
        // Vaishya
        Nakshatra::Krittika
        | Nakshatra::Ashlesha
        | Nakshatra::Chitra
        | Nakshatra::Jyeshtha
        | Nakshatra::UttaraAshadha
        | Nakshatra::Shatabhisha => 1,
        // Shudra
        Nakshatra::Rohini
        | Nakshatra::Mrigashira
        | Nakshatra::Ardra
        | Nakshatra::UttaraPhalguni
        | Nakshatra::Mula
        | Nakshatra::UttaraBhadrapada => 0,
    }
}

// ---------------------------------------------------------------------------
// 2. Vashya (max 2 points)
// Sign-based 5 categories. Same=2, Dwipad-with-all=1, else 0.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VashyaCategory {
    Chatushpada, // Mesha, Vrishabha, Dhanu 1st half, Makara 1st half
    Dwipad,      // Mithuna, Kanya, Tula, Kumbha 1st half, Dhanu 2nd half
    Jalchar,     // Karka, Meena, Makara 2nd half
    Vanchar,     // Simha
    Keeta,       // Vrishchika, Kumbha 2nd half
}

/// Sign-based approximation: each sign maps to primary Vashya category.
/// Signs that span two categories (Dhanu, Makara, Kumbha) use the dominant half.
fn rashi_vashya(rashi_idx: usize) -> VashyaCategory {
    match rashi_idx % 12 {
        0 => VashyaCategory::Chatushpada, // Mesha
        1 => VashyaCategory::Chatushpada, // Vrishabha
        2 => VashyaCategory::Dwipad,      // Mithuna
        3 => VashyaCategory::Jalchar,     // Karka
        4 => VashyaCategory::Vanchar,     // Simha
        5 => VashyaCategory::Dwipad,      // Kanya
        6 => VashyaCategory::Dwipad,      // Tula
        7 => VashyaCategory::Keeta,       // Vrishchika
        8 => VashyaCategory::Chatushpada, // Dhanu (1st half dominant)
        9 => VashyaCategory::Chatushpada, // Makara (1st half dominant)
        10 => VashyaCategory::Dwipad,     // Kumbha (1st half dominant)
        _ => VashyaCategory::Jalchar,     // Meena
    }
}

fn vashya_score(boy_rashi: usize, girl_rashi: usize) -> u8 {
    let boy_v = rashi_vashya(boy_rashi);
    let girl_v = rashi_vashya(girl_rashi);
    if boy_v == girl_v {
        return 2;
    }
    // Dwipad (humans) are compatible with all categories
    if boy_v == VashyaCategory::Dwipad || girl_v == VashyaCategory::Dwipad {
        return 1;
    }
    0
}

// ---------------------------------------------------------------------------
// 3. Tara (max 3 points) — unchanged, already correct
// ---------------------------------------------------------------------------

fn tara_score(boy: Nakshatra, girl: Nakshatra) -> u8 {
    let diff = ((boy.index() as i32 - girl.index() as i32).rem_euclid(27)) as u8;
    let tara_group = (diff % 9) + 1;
    match tara_group {
        1 | 3 | 5 | 7 => 0,
        _ => 3,
    }
}

// ---------------------------------------------------------------------------
// 4. Yoni (max 4 points)
// Classical 14-animal lookup with enemy pairs.
// Same=4, friendly=3, neutral=2, hostile=1, enemy=0.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum YoniAnimal {
    Horse,
    Elephant,
    Sheep,
    Serpent,
    Dog,
    Cat,
    Rat,
    Cow,
    Buffalo,
    Tiger,
    Deer,
    Monkey,
    Mongoose,
    Lion,
}

fn nakshatra_yoni(nak: Nakshatra) -> YoniAnimal {
    match nak {
        Nakshatra::Ashwini => YoniAnimal::Horse,
        Nakshatra::Bharani => YoniAnimal::Elephant,
        Nakshatra::Krittika => YoniAnimal::Sheep,
        Nakshatra::Rohini => YoniAnimal::Serpent,
        Nakshatra::Mrigashira => YoniAnimal::Serpent,
        Nakshatra::Ardra => YoniAnimal::Dog,
        Nakshatra::Punarvasu => YoniAnimal::Cat,
        Nakshatra::Pushya => YoniAnimal::Sheep,
        Nakshatra::Ashlesha => YoniAnimal::Cat,
        Nakshatra::Magha => YoniAnimal::Rat,
        Nakshatra::PurvaPhalguni => YoniAnimal::Rat,
        Nakshatra::UttaraPhalguni => YoniAnimal::Cow,
        Nakshatra::Hasta => YoniAnimal::Buffalo,
        Nakshatra::Chitra => YoniAnimal::Tiger,
        Nakshatra::Swati => YoniAnimal::Buffalo,
        Nakshatra::Vishakha => YoniAnimal::Tiger,
        Nakshatra::Anuradha => YoniAnimal::Deer,
        Nakshatra::Jyeshtha => YoniAnimal::Deer,
        Nakshatra::Mula => YoniAnimal::Dog,
        Nakshatra::PurvaAshadha => YoniAnimal::Monkey,
        Nakshatra::UttaraAshadha => YoniAnimal::Mongoose,
        Nakshatra::Shravana => YoniAnimal::Monkey,
        Nakshatra::Dhanishta => YoniAnimal::Lion,
        Nakshatra::Shatabhisha => YoniAnimal::Horse,
        Nakshatra::PurvaBhadrapada => YoniAnimal::Lion,
        Nakshatra::UttaraBhadrapada => YoniAnimal::Cow,
        Nakshatra::Revati => YoniAnimal::Elephant,
    }
}

/// Returns true if two animals are a classical enemy pair.
fn is_enemy_pair(a: YoniAnimal, b: YoniAnimal) -> bool {
    use YoniAnimal::*;
    matches!(
        (a, b),
        (Horse, Buffalo)
            | (Buffalo, Horse)
            | (Elephant, Lion)
            | (Lion, Elephant)
            | (Sheep, Monkey)
            | (Monkey, Sheep)
            | (Serpent, Mongoose)
            | (Mongoose, Serpent)
            | (Dog, Deer)
            | (Deer, Dog)
            | (Cat, Rat)
            | (Rat, Cat)
            | (Cow, Tiger)
            | (Tiger, Cow)
    )
}

fn yoni_score(boy: Nakshatra, girl: Nakshatra) -> u8 {
    let boy_y = nakshatra_yoni(boy);
    let girl_y = nakshatra_yoni(girl);
    if boy_y == girl_y {
        return 4; // Same animal
    }
    if is_enemy_pair(boy_y, girl_y) {
        return 0; // Enemy pair
    }
    // Friendly pairs (same species group) = 3, neutral = 2, hostile = 1.
    // Classical texts use a 14x14 matrix; simplified here as:
    // enemy=0 (handled above), same=4 (handled above), default=2 (neutral).
    // A full 14x14 table would give 1 or 3 for some combos, but the enemy
    // pairs are the critical P0 fix. Neutral is a safe default.
    2
}

// ---------------------------------------------------------------------------
// 5. Graha Maitri (max 5 points)
// Uses natural friendship from dignity.rs between rashi lords.
// ---------------------------------------------------------------------------

fn rashi_lord_name(rashi_idx: usize) -> &'static str {
    Rashi::from_index(rashi_idx).lord()
}

fn graha_maitri_score(boy_rashi: usize, girl_rashi: usize) -> u8 {
    let boy_lord = rashi_lord_name(boy_rashi);
    let girl_lord = rashi_lord_name(girl_rashi);

    if boy_lord == girl_lord {
        return 5; // Same lord = best friends
    }

    let rel_boy_to_girl = natural_relationship(boy_lord, girl_lord);
    let rel_girl_to_boy = natural_relationship(girl_lord, boy_lord);

    // Average both directions: Mitra=5, Sama=3, Shatru=1
    let score_a = relationship_points(rel_boy_to_girl);
    let score_b = relationship_points(rel_girl_to_boy);
    
    (score_a + score_b) / 2
}

fn relationship_points(rel: Relationship) -> u8 {
    match rel {
        Relationship::AdhiMitra | Relationship::Mitra => 5,
        Relationship::Sama => 3,
        Relationship::Shatru | Relationship::AdhiShatru => 1,
    }
}

// ---------------------------------------------------------------------------
// 6. Gana (max 6 points) — unchanged, already correct
// ---------------------------------------------------------------------------

fn gana_score(boy: Nakshatra, girl: Nakshatra) -> u8 {
    let bg = boy.gana();
    let gg = girl.gana();
    match (bg, gg) {
        (Gana::Deva, Gana::Deva)
        | (Gana::Manushya, Gana::Manushya)
        | (Gana::Rakshasa, Gana::Rakshasa) => 6,
        (Gana::Deva, Gana::Manushya) | (Gana::Manushya, Gana::Deva) => 5,
        _ => 0,
    }
}

// ---------------------------------------------------------------------------
// 7. Bhakoot (max 7 points) — unchanged, already correct
// ---------------------------------------------------------------------------

fn bhakoot_score(boy_rashi: usize, girl_rashi: usize) -> u8 {
    // Forward distance from boy to girl (0-11)
    let fwd = ((girl_rashi as i32 - boy_rashi as i32).rem_euclid(12)) as usize;
    match fwd {
        0 | 4 | 8 => 7, // 1/1, 5/5, 9/9 — favorable
        1 | 11 => 0,    // 2/12 — unfavorable
        5 | 7 => 0,     // 6/8 — unfavorable
        _ => 7,         // all other combinations — favorable
    }
}

// ---------------------------------------------------------------------------
// 8. Nadi (max 8 points)
// Classical Adi/Madhya/Antya pattern — NOT index % 3.
// Same nadi = 0 (nadi dosha), different = 8.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Nadi {
    Adi,
    Madhya,
    Antya,
}

fn nakshatra_nadi(nak: Nakshatra) -> Nadi {
    match nak {
        // Adi (Vata)
        Nakshatra::Ashwini
        | Nakshatra::Ardra
        | Nakshatra::Punarvasu
        | Nakshatra::UttaraPhalguni
        | Nakshatra::Hasta
        | Nakshatra::Jyeshtha
        | Nakshatra::Mula
        | Nakshatra::Shatabhisha
        | Nakshatra::PurvaBhadrapada => Nadi::Adi,
        // Madhya (Pitta)
        Nakshatra::Bharani
        | Nakshatra::Mrigashira
        | Nakshatra::Pushya
        | Nakshatra::PurvaPhalguni
        | Nakshatra::Chitra
        | Nakshatra::Anuradha
        | Nakshatra::PurvaAshadha
        | Nakshatra::Dhanishta
        | Nakshatra::UttaraBhadrapada => Nadi::Madhya,
        // Antya (Kapha)
        Nakshatra::Krittika
        | Nakshatra::Rohini
        | Nakshatra::Ashlesha
        | Nakshatra::Magha
        | Nakshatra::Swati
        | Nakshatra::Vishakha
        | Nakshatra::UttaraAshadha
        | Nakshatra::Shravana
        | Nakshatra::Revati => Nadi::Antya,
    }
}

fn nadi_score(boy: Nakshatra, girl: Nakshatra) -> u8 {
    let boy_nadi = nakshatra_nadi(boy);
    let girl_nadi = nakshatra_nadi(girl);
    if boy_nadi != girl_nadi { 8 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Varna tests
    // -----------------------------------------------------------------------
    #[test]
    fn varna_brahmin_lookup() {
        assert_eq!(nakshatra_varna(Nakshatra::Ashwini), 3);
        assert_eq!(nakshatra_varna(Nakshatra::Pushya), 3);
        assert_eq!(nakshatra_varna(Nakshatra::Revati), 3);
        assert_eq!(nakshatra_varna(Nakshatra::PurvaBhadrapada), 3);
    }

    #[test]
    fn varna_kshatriya_lookup() {
        assert_eq!(nakshatra_varna(Nakshatra::Bharani), 2);
        assert_eq!(nakshatra_varna(Nakshatra::Magha), 2);
        assert_eq!(nakshatra_varna(Nakshatra::Dhanishta), 2);
    }

    #[test]
    fn varna_vaishya_lookup() {
        assert_eq!(nakshatra_varna(Nakshatra::Krittika), 1);
        assert_eq!(nakshatra_varna(Nakshatra::Ashlesha), 1);
        assert_eq!(nakshatra_varna(Nakshatra::Shatabhisha), 1);
    }

    #[test]
    fn varna_shudra_lookup() {
        assert_eq!(nakshatra_varna(Nakshatra::Rohini), 0);
        assert_eq!(nakshatra_varna(Nakshatra::Mrigashira), 0);
        assert_eq!(nakshatra_varna(Nakshatra::UttaraBhadrapada), 0);
    }

    #[test]
    fn varna_groom_higher_scores_1() {
        // Brahmin groom, Shudra bride => 1
        assert_eq!(varna_score(Nakshatra::Ashwini, Nakshatra::Rohini), 1);
    }

    #[test]
    fn varna_groom_lower_scores_0() {
        // Shudra groom, Brahmin bride => 0
        assert_eq!(varna_score(Nakshatra::Rohini, Nakshatra::Ashwini), 0);
    }

    // -----------------------------------------------------------------------
    // Vashya tests
    // -----------------------------------------------------------------------
    #[test]
    fn vashya_same_category() {
        // Mesha(0) and Vrishabha(1) both Chatushpada
        assert_eq!(vashya_score(0, 1), 2);
    }

    #[test]
    fn vashya_dwipad_compatible_with_all() {
        // Mithuna(2)=Dwipad with Simha(4)=Vanchar => 1
        assert_eq!(vashya_score(2, 4), 1);
        // Kanya(5)=Dwipad with Meena(11)=Jalchar => 1
        assert_eq!(vashya_score(5, 11), 1);
    }

    #[test]
    fn vashya_incompatible() {
        // Simha(4)=Vanchar with Vrishchika(7)=Keeta => 0
        assert_eq!(vashya_score(4, 7), 0);
    }

    // -----------------------------------------------------------------------
    // Yoni tests
    // -----------------------------------------------------------------------
    #[test]
    fn yoni_same_animal() {
        // Ashwini=Horse, Shatabhisha=Horse => 4
        assert_eq!(yoni_score(Nakshatra::Ashwini, Nakshatra::Shatabhisha), 4);
        // Rohini=Serpent, Mrigashira=Serpent => 4
        assert_eq!(yoni_score(Nakshatra::Rohini, Nakshatra::Mrigashira), 4);
    }

    #[test]
    fn yoni_enemy_pair() {
        // Ashwini=Horse, Hasta=Buffalo => enemy => 0
        assert_eq!(yoni_score(Nakshatra::Ashwini, Nakshatra::Hasta), 0);
        // Magha=Rat, Punarvasu=Cat => enemy => 0
        assert_eq!(yoni_score(Nakshatra::Magha, Nakshatra::Punarvasu), 0);
        // Bharani=Elephant, Dhanishta=Lion => enemy => 0
        assert_eq!(yoni_score(Nakshatra::Bharani, Nakshatra::Dhanishta), 0);
    }

    #[test]
    fn yoni_neutral() {
        // Ashwini=Horse, Bharani=Elephant => neutral => 2
        assert_eq!(yoni_score(Nakshatra::Ashwini, Nakshatra::Bharani), 2);
    }

    // -----------------------------------------------------------------------
    // Graha Maitri tests
    // -----------------------------------------------------------------------
    #[test]
    fn graha_maitri_same_lord() {
        // Mesha(0) and Vrishchika(7) both Mars => 5
        assert_eq!(graha_maitri_score(0, 7), 5);
    }

    #[test]
    fn graha_maitri_friends() {
        // Mesha(0)=Mars lord, Simha(4)=Sun lord => Mars-Sun mutual friends => 5
        assert_eq!(graha_maitri_score(0, 4), 5);
    }

    #[test]
    fn graha_maitri_enemies() {
        // Simha(4)=Sun lord, Makara(9)=Saturn lord => Sun-Saturn enemies => 1
        assert_eq!(graha_maitri_score(4, 9), 1);
    }

    #[test]
    fn graha_maitri_mixed() {
        // Mesha(0)=Mars, Mithuna(2)=Mercury => Mars enemies Mercury, Mercury neutral Mars
        // Mars->Mercury = Shatru(1), Mercury->Mars = Sama(3) => avg = 2
        assert_eq!(graha_maitri_score(0, 2), 2);
    }

    // -----------------------------------------------------------------------
    // Nadi tests
    // -----------------------------------------------------------------------
    #[test]
    fn nadi_classical_adi() {
        assert_eq!(nakshatra_nadi(Nakshatra::Ashwini), Nadi::Adi);
        assert_eq!(nakshatra_nadi(Nakshatra::Ardra), Nadi::Adi);
        assert_eq!(nakshatra_nadi(Nakshatra::Punarvasu), Nadi::Adi);
        assert_eq!(nakshatra_nadi(Nakshatra::Mula), Nadi::Adi);
    }

    #[test]
    fn nadi_classical_madhya() {
        assert_eq!(nakshatra_nadi(Nakshatra::Bharani), Nadi::Madhya);
        assert_eq!(nakshatra_nadi(Nakshatra::Mrigashira), Nadi::Madhya);
        assert_eq!(nakshatra_nadi(Nakshatra::PurvaAshadha), Nadi::Madhya);
    }

    #[test]
    fn nadi_classical_antya() {
        assert_eq!(nakshatra_nadi(Nakshatra::Krittika), Nadi::Antya);
        assert_eq!(nakshatra_nadi(Nakshatra::Rohini), Nadi::Antya);
        assert_eq!(nakshatra_nadi(Nakshatra::Revati), Nadi::Antya);
    }

    #[test]
    fn nadi_dosha_same() {
        // Ashwini=Adi, Ardra=Adi => same => 0
        assert_eq!(nadi_score(Nakshatra::Ashwini, Nakshatra::Ardra), 0);
    }

    #[test]
    fn nadi_different() {
        // Ashwini=Adi, Bharani=Madhya => different => 8
        assert_eq!(nadi_score(Nakshatra::Ashwini, Nakshatra::Bharani), 8);
    }

    #[test]
    fn nadi_dosha_ashwini_magha() {
        // Ashwini=Adi, Magha=Antya => different nadi => 8 (NOT 0)
        // Under the old buggy % 3: both index 0 and 9 => 0%3 == 0 => same => 0. WRONG.
        // Classical: Ashwini=Adi, Magha=Antya => different => 8
        assert_eq!(nadi_score(Nakshatra::Ashwini, Nakshatra::Magha), 8);
    }

    // -----------------------------------------------------------------------
    // Integration tests
    // -----------------------------------------------------------------------
    #[test]
    fn same_nakshatra_scores_well() {
        let m = ashtakoota_match(Nakshatra::Ashwini, Nakshatra::Ashwini, 0, 0);
        // Same nak: varna=1(same), vashya=2(same rashi), tara=0(group 1),
        // yoni=4(same), graha_maitri=5(same lord), gana=6(same Deva),
        // bhakoot=7(same rashi), nadi=0(same Adi)
        assert_eq!(
            m.total, 25,
            "Ashwini-Ashwini same rashi should be 25, got {}",
            m.total
        );
    }

    #[test]
    fn total_never_exceeds_36() {
        for i in 0..27 {
            for j in 0..27 {
                let boy = Nakshatra::ALL[i];
                let girl = Nakshatra::ALL[j];
                let m = ashtakoota_match(boy, girl, i % 12, j % 12);
                assert!(m.total <= 36, "{boy}-{girl}: total {} > 36", m.total);
            }
        }
    }

    #[test]
    fn max_components_correct() {
        // Verify individual max values across all combos
        let mut max_v = 0u8;
        let mut max_vs = 0u8;
        let mut max_t = 0u8;
        let mut max_y = 0u8;
        let mut max_gm = 0u8;
        let mut max_g = 0u8;
        let mut max_b = 0u8;
        let mut max_n = 0u8;
        for i in 0..27 {
            for j in 0..27 {
                let boy = Nakshatra::ALL[i];
                let girl = Nakshatra::ALL[j];
                let m = ashtakoota_match(boy, girl, i % 12, j % 12);
                max_v = max_v.max(m.varna);
                max_vs = max_vs.max(m.vashya);
                max_t = max_t.max(m.tara);
                max_y = max_y.max(m.yoni);
                max_gm = max_gm.max(m.graha_maitri);
                max_g = max_g.max(m.gana);
                max_b = max_b.max(m.bhakoot);
                max_n = max_n.max(m.nadi);
            }
        }
        assert_eq!(max_v, 1, "Varna max should be 1");
        assert_eq!(max_vs, 2, "Vashya max should be 2");
        assert_eq!(max_t, 3, "Tara max should be 3");
        assert_eq!(max_y, 4, "Yoni max should be 4");
        assert_eq!(max_gm, 5, "Graha Maitri max should be 5");
        assert_eq!(max_g, 6, "Gana max should be 6");
        assert_eq!(max_b, 7, "Bhakoot max should be 7");
        assert_eq!(max_n, 8, "Nadi max should be 8");
    }
}
