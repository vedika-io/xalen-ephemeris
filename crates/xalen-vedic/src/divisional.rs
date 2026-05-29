use crate::rashi::Rashi;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VargaChart {
    D1,
    D2,
    D3,
    D4,
    D7,
    D9,
    D10,
    D12,
    D16,
    D20,
    D24,
    D27,
    D30,
    D40,
    D45,
    D60,
}

impl VargaChart {
    pub fn divisions(&self) -> u32 {
        match self {
            VargaChart::D1 => 1,
            VargaChart::D2 => 2,
            VargaChart::D3 => 3,
            VargaChart::D4 => 4,
            VargaChart::D7 => 7,
            VargaChart::D9 => 9,
            VargaChart::D10 => 10,
            VargaChart::D12 => 12,
            VargaChart::D16 => 16,
            VargaChart::D20 => 20,
            VargaChart::D24 => 24,
            VargaChart::D27 => 27,
            VargaChart::D30 => 30,
            VargaChart::D40 => 40,
            VargaChart::D45 => 45,
            VargaChart::D60 => 60,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            VargaChart::D1 => "Rashi",
            VargaChart::D2 => "Hora",
            VargaChart::D3 => "Drekkana",
            VargaChart::D4 => "Chaturthamsa",
            VargaChart::D7 => "Saptamsa",
            VargaChart::D9 => "Navamsa",
            VargaChart::D10 => "Dasamsa",
            VargaChart::D12 => "Dwadasamsa",
            VargaChart::D16 => "Shodasamsa",
            VargaChart::D20 => "Vimsamsa",
            VargaChart::D24 => "Chaturvimsamsa",
            VargaChart::D27 => "Bhamsa",
            VargaChart::D30 => "Trimsamsa",
            VargaChart::D40 => "Khavedamsa",
            VargaChart::D45 => "Akshavedamsa",
            VargaChart::D60 => "Shashtiamsa",
        }
    }
}

pub fn compute_varga_sign(lon_deg: f64, varga: VargaChart) -> Rashi {
    let lon_deg = lon_deg.rem_euclid(360.0);
    let sign_idx = (lon_deg / 30.0) as usize % 12;
    let deg_in_sign = lon_deg % 30.0;

    match varga {
        VargaChart::D1 => Rashi::from_index(sign_idx),
        VargaChart::D9 => navamsa(sign_idx, deg_in_sign),
        VargaChart::D2 => hora(sign_idx, deg_in_sign),
        VargaChart::D3 => drekkana(sign_idx, deg_in_sign),
        VargaChart::D7 => saptamsa(sign_idx, deg_in_sign),
        VargaChart::D10 => dasamsa(sign_idx, deg_in_sign),
        VargaChart::D12 => dwadasamsa(sign_idx, deg_in_sign),
        VargaChart::D16 => shodasamsa(sign_idx, deg_in_sign),
        VargaChart::D20 => vimsamsa(sign_idx, deg_in_sign),
        VargaChart::D24 => chaturvimsamsa(sign_idx, deg_in_sign),
        VargaChart::D27 => bhamsa(sign_idx, deg_in_sign),
        VargaChart::D30 => trimsamsa(sign_idx, deg_in_sign),
        VargaChart::D40 => khavedamsa(sign_idx, deg_in_sign),
        VargaChart::D45 => akshavedamsa(sign_idx, deg_in_sign),
        // D4 and D60 use generic sequential division (correct per BPHS)
        _ => generic_varga(sign_idx, deg_in_sign, varga.divisions()),
    }
}

fn navamsa(sign_idx: usize, deg: f64) -> Rashi {
    // Fire signs start from themselves, Earth from Capricorn, Air from Libra, Water from Cancer
    let element = sign_idx % 4;
    let start = match element {
        0 => sign_idx, // Fire: from self
        1 => 9,        // Earth: from Capricorn
        2 => 6,        // Air: from Libra
        _ => 3,        // Water: from Cancer
    };
    let navamsa_idx = (deg / (30.0 / 9.0)) as usize;
    Rashi::from_index((start + navamsa_idx) % 12)
}

fn hora(sign_idx: usize, deg: f64) -> Rashi {
    let is_odd = sign_idx.is_multiple_of(2); // 0-indexed: Aries=0=even index but odd sign
    let first_half = deg < 15.0;
    if is_odd {
        if first_half {
            Rashi::Simha
        } else {
            Rashi::Karka
        } // Sun then Moon
    } else {
        if first_half {
            Rashi::Karka
        } else {
            Rashi::Simha
        } // Moon then Sun
    }
}

fn drekkana(sign_idx: usize, deg: f64) -> Rashi {
    let drek = (deg / 10.0) as usize;
    let offset = drek * 4; // 0, 4, 8 = same sign, 5th sign, 9th sign
    Rashi::from_index((sign_idx + offset) % 12)
}

fn saptamsa(sign_idx: usize, deg: f64) -> Rashi {
    let part = (deg / (30.0 / 7.0)) as usize;
    let is_odd = sign_idx.is_multiple_of(2);
    let start = if is_odd { sign_idx } else { sign_idx + 6 };
    Rashi::from_index((start + part) % 12)
}

fn dasamsa(sign_idx: usize, deg: f64) -> Rashi {
    let part = (deg / 3.0) as usize;
    let is_odd = sign_idx.is_multiple_of(2);
    let start = if is_odd { sign_idx } else { sign_idx + 8 };
    Rashi::from_index((start + part) % 12)
}

fn dwadasamsa(sign_idx: usize, deg: f64) -> Rashi {
    let part = (deg / 2.5) as usize;
    Rashi::from_index((sign_idx + part) % 12)
}

fn trimsamsa(sign_idx: usize, deg: f64) -> Rashi {
    // BPHS Trimsamsa per Parashara (web-verified against srath.com + jagannathhora.com):
    // Odd signs: Mars 0-5→Aries, Saturn 5-10→Aquarius, Jupiter 10-18→Sagittarius,
    //            Mercury 18-25→Gemini, Venus 25-30→Libra
    // Even signs: Venus 0-5→Taurus, Mercury 5-12→Virgo, Jupiter 12-20→Pisces,
    //             Saturn 20-25→Capricorn, Mars 25-30→Scorpio
    let is_odd = sign_idx.is_multiple_of(2); // 0-indexed: Aries=0=even index=odd sign
    if is_odd {
        match deg {
            d if d < 5.0 => Rashi::Mesha,    // Mars → Aries
            d if d < 10.0 => Rashi::Kumbha,  // Saturn → Aquarius
            d if d < 18.0 => Rashi::Dhanu,   // Jupiter → Sagittarius
            d if d < 25.0 => Rashi::Mithuna, // Mercury → Gemini
            _ => Rashi::Tula,                // Venus → Libra
        }
    } else {
        match deg {
            d if d < 5.0 => Rashi::Vrishabha, // Venus → Taurus
            d if d < 12.0 => Rashi::Kanya,    // Mercury → Virgo
            d if d < 20.0 => Rashi::Meena,    // Jupiter → Pisces
            d if d < 25.0 => Rashi::Makara,   // Saturn → Capricorn
            _ => Rashi::Vrishchika,           // Mars → Scorpio
        }
    }
}

/// D16 (Shodasamsa) per BPHS:
/// Movable signs: sequential from Aries (0)
/// Fixed signs: sequential from Leo (4)
/// Dual signs: sequential from Sagittarius (8)
/// Each part = 30/16 = 1.875°
fn shodasamsa(sign_idx: usize, deg: f64) -> Rashi {
    let modality = sign_idx % 3;
    let start = match modality {
        0 => 0, // Movable: from Aries
        1 => 4, // Fixed: from Leo
        _ => 8, // Dual: from Sagittarius
    };
    let part = (deg / (30.0 / 16.0)) as usize;
    Rashi::from_index((start + part) % 12)
}

/// D20 (Vimsamsa) per BPHS:
/// Movable signs: sequential from Aries (0)
/// Fixed signs: sequential from Sagittarius (8)
/// Dual signs: sequential from Leo (4)
/// Each part = 30/20 = 1.5°
fn vimsamsa(sign_idx: usize, deg: f64) -> Rashi {
    let modality = sign_idx % 3;
    let start = match modality {
        0 => 0, // Movable: from Aries
        1 => 8, // Fixed: from Sagittarius
        _ => 4, // Dual: from Leo
    };
    let part = (deg / 1.5) as usize;
    Rashi::from_index((start + part) % 12)
}

/// D24 (Chaturvimsamsa / Siddhamsa) per BPHS:
/// Odd signs: sequential from Leo (4)
/// Even signs: reverse from Cancer (3) — Cancer, Gemini, Taurus, Aries, Pisces...
/// Each part = 30/24 = 1.25°
fn chaturvimsamsa(sign_idx: usize, deg: f64) -> Rashi {
    let is_odd_sign = sign_idx.is_multiple_of(2); // 0-indexed: Aries=0=even idx=odd sign
    let part = (deg / 1.25) as usize;
    if is_odd_sign {
        // Odd signs: forward from Leo
        Rashi::from_index((4 + part) % 12)
    } else {
        // Even signs: backward from Cancer
        // Cancer=3, going backward. Use modular arithmetic to avoid overflow:
        // (3 - part) mod 12, implemented as (3 + 12 - (part % 12)) % 12
        Rashi::from_index((3 + 12 - (part % 12)) % 12)
    }
}

/// D27 (Bhamsa / Nakshatramsa) per BPHS:
/// Fire signs (Aries=0, Leo=4, Sagittarius=8): start from Aries (0)
/// Earth signs (Taurus=1, Virgo=5, Capricorn=9): start from Cancer (3)
/// Air signs (Gemini=2, Libra=6, Aquarius=10): start from Libra (6)
/// Water signs (Cancer=3, Scorpio=7, Pisces=11): start from Capricorn (9)
/// Each part = 30/27 ≈ 1.111°
fn bhamsa(sign_idx: usize, deg: f64) -> Rashi {
    let element = sign_idx % 4;
    let start = match element {
        0 => 0, // Fire: from Aries
        1 => 3, // Earth: from Cancer
        2 => 6, // Air: from Libra
        _ => 9, // Water: from Capricorn
    };
    let part = (deg / (30.0 / 27.0)) as usize;
    Rashi::from_index((start + part) % 12)
}

/// D40 (Khavedamsa) per BPHS:
/// Odd signs: sequential from Aries (0)
/// Even signs: sequential from Libra (6)
/// Each part = 30/40 = 0.75°
fn khavedamsa(sign_idx: usize, deg: f64) -> Rashi {
    let is_odd_sign = sign_idx.is_multiple_of(2); // 0-indexed: Aries=0=even idx=odd sign
    let start = if is_odd_sign { 0 } else { 6 }; // Odd: Aries, Even: Libra
    let part = (deg / 0.75) as usize;
    Rashi::from_index((start + part) % 12)
}

/// D45 (Akshavedamsa) per BPHS:
/// Movable signs: sequential from Aries (0)
/// Fixed signs: sequential from Leo (4)
/// Dual signs: sequential from Sagittarius (8)
/// Each part = 30/45 = 0.6667°
fn akshavedamsa(sign_idx: usize, deg: f64) -> Rashi {
    let modality = sign_idx % 3;
    let start = match modality {
        0 => 0, // Movable: from Aries
        1 => 4, // Fixed: from Leo
        _ => 8, // Dual: from Sagittarius
    };
    let part = (deg / (30.0 / 45.0)) as usize;
    Rashi::from_index((start + part) % 12)
}

fn generic_varga(sign_idx: usize, deg: f64, divisions: u32) -> Rashi {
    let span = 30.0 / divisions as f64;
    let part = (deg / span) as usize;
    Rashi::from_index((sign_idx + part) % 12)
}

pub fn is_vargottama(d1_rashi: Rashi, d9_rashi: Rashi) -> bool {
    d1_rashi == d9_rashi
}

/// Check whether a planet at the given sidereal longitude is Vargottama
/// (same Rashi in D1 and D9) by computing both charts from position.
pub fn is_vargottama_by_longitude(sidereal_lon_deg: f64) -> bool {
    let d1 = compute_varga_sign(sidereal_lon_deg, VargaChart::D1);
    let d9 = compute_varga_sign(sidereal_lon_deg, VargaChart::D9);
    d1 == d9
}

/// Given a list of (planet_name, sidereal_longitude) pairs, return
/// the names of all planets that are Vargottama (same D1 and D9 rashi).
pub fn find_vargottama_planets(planet_positions: &[(String, f64)]) -> Vec<String> {
    planet_positions
        .iter()
        .filter(|(_, lon)| is_vargottama_by_longitude(*lon))
        .map(|(name, _)| name.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn d1_identity() {
        assert_eq!(compute_varga_sign(45.0, VargaChart::D1), Rashi::Vrishabha);
    }

    #[test]
    fn navamsa_aries_0_deg() {
        // 0° Aries: Fire sign, starts from self (Aries), first navamsa = Aries
        let d9 = compute_varga_sign(0.0, VargaChart::D9);
        assert_eq!(d9, Rashi::Mesha, "0° Aries navamsa should be Aries");
    }

    #[test]
    fn navamsa_aries_end() {
        // ~29° Aries: last navamsa of Aries = Sagittarius (9th from Aries)
        let d9 = compute_varga_sign(29.9, VargaChart::D9);
        assert_eq!(d9, Rashi::Dhanu, "29° Aries navamsa should be Sagittarius");
    }

    #[test]
    fn hora_odd_sign() {
        // First half of Aries (odd sign) = Leo (Sun's hora)
        assert_eq!(compute_varga_sign(10.0, VargaChart::D2), Rashi::Simha);
        // Second half = Cancer (Moon's hora)
        assert_eq!(compute_varga_sign(20.0, VargaChart::D2), Rashi::Karka);
    }

    #[test]
    fn drekkana() {
        // 0-10° Aries = Aries (1st decanate = same sign)
        assert_eq!(compute_varga_sign(5.0, VargaChart::D3), Rashi::Mesha);
        // 10-20° Aries = Leo (5th from Aries)
        assert_eq!(compute_varga_sign(15.0, VargaChart::D3), Rashi::Simha);
        // 20-30° Aries = Sagittarius (9th from Aries)
        assert_eq!(compute_varga_sign(25.0, VargaChart::D3), Rashi::Dhanu);
    }

    #[test]
    fn dwadasamsa() {
        // 0-2.5° Aries = Aries itself
        assert_eq!(compute_varga_sign(1.0, VargaChart::D12), Rashi::Mesha);
        // 2.5-5° Aries = Taurus
        assert_eq!(compute_varga_sign(3.0, VargaChart::D12), Rashi::Vrishabha);
    }

    #[test]
    fn vargottama_detection() {
        assert!(is_vargottama(Rashi::Mesha, Rashi::Mesha));
        assert!(!is_vargottama(Rashi::Mesha, Rashi::Vrishabha));
    }

    #[test]
    fn negative_longitude_normalized() {
        let r1 = compute_varga_sign(-30.0, VargaChart::D1);
        let r2 = compute_varga_sign(330.0, VargaChart::D1);
        assert_eq!(r1, r2, "-30° should equal 330°");
        assert_eq!(r1, Rashi::Meena, "-30° = 330° = Pisces");
    }

    #[test]
    fn longitude_over_360_normalized() {
        let r1 = compute_varga_sign(400.0, VargaChart::D9);
        let r2 = compute_varga_sign(40.0, VargaChart::D9);
        assert_eq!(r1, r2, "400° navamsa should equal 40° navamsa");
    }

    #[test]
    fn trimsamsa_odd_sign() {
        // Web-verified: srath.com + jagannathhora.com
        // 3° Aries (odd sign) → Mars → Aries
        assert_eq!(compute_varga_sign(3.0, VargaChart::D30), Rashi::Mesha);
        // 7° Aries → Saturn → Aquarius
        assert_eq!(compute_varga_sign(7.0, VargaChart::D30), Rashi::Kumbha);
        // 15° Aries → Jupiter → Sagittarius
        assert_eq!(compute_varga_sign(15.0, VargaChart::D30), Rashi::Dhanu);
        // 22° Aries → Mercury → Gemini
        assert_eq!(compute_varga_sign(22.0, VargaChart::D30), Rashi::Mithuna);
        // 28° Aries → Venus → Libra
        assert_eq!(compute_varga_sign(28.0, VargaChart::D30), Rashi::Tula);
    }

    #[test]
    fn trimsamsa_even_sign() {
        // Web-verified: srath.com + jagannathhora.com
        // 3° Taurus (even sign) → Venus → Taurus
        assert_eq!(compute_varga_sign(33.0, VargaChart::D30), Rashi::Vrishabha);
        // 8° Taurus → Mercury → Virgo
        assert_eq!(compute_varga_sign(38.0, VargaChart::D30), Rashi::Kanya);
        // 16° Taurus → Jupiter → Pisces
        assert_eq!(compute_varga_sign(46.0, VargaChart::D30), Rashi::Meena);
        // 22° Taurus → Saturn → Capricorn
        assert_eq!(compute_varga_sign(52.0, VargaChart::D30), Rashi::Makara);
        // 28° Taurus → Mars → Scorpio
        assert_eq!(compute_varga_sign(58.0, VargaChart::D30), Rashi::Vrishchika);
    }

    #[test]
    fn all_vargas_produce_valid_rashi() {
        let charts = [
            VargaChart::D1,
            VargaChart::D2,
            VargaChart::D3,
            VargaChart::D4,
            VargaChart::D7,
            VargaChart::D9,
            VargaChart::D10,
            VargaChart::D12,
            VargaChart::D16,
            VargaChart::D20,
            VargaChart::D24,
            VargaChart::D27,
            VargaChart::D30,
            VargaChart::D40,
            VargaChart::D45,
            VargaChart::D60,
        ];
        for lon in (0..360).step_by(5) {
            for chart in &charts {
                let rashi = compute_varga_sign(lon as f64, *chart);
                assert!(
                    rashi.index() < 12,
                    "{chart:?} at {lon}° produced invalid rashi"
                );
            }
        }
    }

    // ---- D16 (Shodasamsa) tests ----

    #[test]
    fn d16_movable_sign_starts_aries() {
        // 0° Aries (movable, idx=0): part 0 → start=0(Aries) + 0 = Aries
        assert_eq!(compute_varga_sign(0.0, VargaChart::D16), Rashi::Mesha);
        // 0° Cancer (movable, idx=3): part 0 → start=0(Aries) + 0 = Aries
        assert_eq!(compute_varga_sign(90.0, VargaChart::D16), Rashi::Mesha);
        // 0° Libra (movable, idx=6): part 0 → start=0(Aries) + 0 = Aries
        assert_eq!(compute_varga_sign(180.0, VargaChart::D16), Rashi::Mesha);
        // 0° Capricorn (movable, idx=9): part 0 → start=0(Aries) + 0 = Aries
        assert_eq!(compute_varga_sign(270.0, VargaChart::D16), Rashi::Mesha);
    }

    #[test]
    fn d16_fixed_sign_starts_leo() {
        // 0° Taurus (fixed, idx=1): part 0 → start=4(Leo) + 0 = Leo
        assert_eq!(compute_varga_sign(30.0, VargaChart::D16), Rashi::Simha);
        // 0° Leo (fixed, idx=4): part 0 → start=4(Leo) + 0 = Leo
        assert_eq!(compute_varga_sign(120.0, VargaChart::D16), Rashi::Simha);
        // 0° Scorpio (fixed, idx=7): part 0 → start=4(Leo) + 0 = Leo
        assert_eq!(compute_varga_sign(210.0, VargaChart::D16), Rashi::Simha);
    }

    #[test]
    fn d16_dual_sign_starts_sagittarius() {
        // 0° Gemini (dual, idx=2): part 0 → start=8(Sagittarius) + 0 = Sagittarius
        assert_eq!(compute_varga_sign(60.0, VargaChart::D16), Rashi::Dhanu);
        // 0° Virgo (dual, idx=5): part 0 → start=8(Sagittarius) + 0 = Sagittarius
        assert_eq!(compute_varga_sign(150.0, VargaChart::D16), Rashi::Dhanu);
        // 0° Pisces (dual, idx=11): part 0 → start=8(Sagittarius) + 0 = Sagittarius
        assert_eq!(compute_varga_sign(330.0, VargaChart::D16), Rashi::Dhanu);
    }

    #[test]
    fn d16_last_part_movable() {
        // 29.9° Aries: part = floor(29.9 / 1.875) = floor(15.946) = 15
        // start=0 + 15 = 15 % 12 = 3 = Cancer (wraps around: 16 parts cycle through all 12 + 4)
        assert_eq!(compute_varga_sign(29.9, VargaChart::D16), Rashi::Karka);
    }

    #[test]
    fn d16_sequential_check() {
        // In Aries (movable): 1.875° per part, start from Aries
        // Part 0 (0-1.875°) = Aries, Part 1 = Taurus, Part 2 = Gemini, Part 3 = Cancer
        assert_eq!(compute_varga_sign(0.5, VargaChart::D16), Rashi::Mesha);
        assert_eq!(compute_varga_sign(2.0, VargaChart::D16), Rashi::Vrishabha);
        assert_eq!(compute_varga_sign(4.0, VargaChart::D16), Rashi::Mithuna);
        assert_eq!(compute_varga_sign(6.0, VargaChart::D16), Rashi::Karka);
    }

    // ---- D20 (Vimsamsa) tests ----

    #[test]
    fn d20_movable_sign_starts_aries() {
        // 0° Aries (movable): start=0(Aries)
        assert_eq!(compute_varga_sign(0.0, VargaChart::D20), Rashi::Mesha);
        // 0° Cancer (movable): start=0(Aries)
        assert_eq!(compute_varga_sign(90.0, VargaChart::D20), Rashi::Mesha);
    }

    #[test]
    fn d20_fixed_sign_starts_sagittarius() {
        // 0° Taurus (fixed, idx=1): start=8(Sagittarius)
        assert_eq!(compute_varga_sign(30.0, VargaChart::D20), Rashi::Dhanu);
        // 0° Leo (fixed, idx=4): start=8(Sagittarius)
        assert_eq!(compute_varga_sign(120.0, VargaChart::D20), Rashi::Dhanu);
    }

    #[test]
    fn d20_dual_sign_starts_leo() {
        // 0° Gemini (dual, idx=2): start=4(Leo)
        assert_eq!(compute_varga_sign(60.0, VargaChart::D20), Rashi::Simha);
        // 0° Virgo (dual, idx=5): start=4(Leo)
        assert_eq!(compute_varga_sign(150.0, VargaChart::D20), Rashi::Simha);
    }

    #[test]
    fn d20_sequential_check() {
        // In Aries (movable): 1.5° per part, start from Aries
        // Part 0 (0-1.5°) = Aries, Part 1 = Taurus, Part 2 = Gemini
        assert_eq!(compute_varga_sign(0.5, VargaChart::D20), Rashi::Mesha);
        assert_eq!(compute_varga_sign(1.6, VargaChart::D20), Rashi::Vrishabha);
        assert_eq!(compute_varga_sign(3.1, VargaChart::D20), Rashi::Mithuna);
    }

    #[test]
    fn d20_last_part_movable() {
        // 29.9° Aries: part = floor(29.9 / 1.5) = floor(19.933) = 19
        // start=0 + 19 = 19 % 12 = 7 = Scorpio
        assert_eq!(compute_varga_sign(29.9, VargaChart::D20), Rashi::Vrishchika);
    }

    // ---- D24 (Chaturvimsamsa) tests ----

    #[test]
    fn d24_odd_sign_starts_leo() {
        // 0° Aries (odd sign, idx=0): start=4(Leo)
        assert_eq!(compute_varga_sign(0.0, VargaChart::D24), Rashi::Simha);
        // 0° Gemini (odd sign, idx=2): start=4(Leo)
        assert_eq!(compute_varga_sign(60.0, VargaChart::D24), Rashi::Simha);
        // 0° Leo (odd sign, idx=4): start=4(Leo)
        assert_eq!(compute_varga_sign(120.0, VargaChart::D24), Rashi::Simha);
    }

    #[test]
    fn d24_even_sign_starts_cancer_backward() {
        // 0° Taurus (even sign, idx=1): part 0 → (3 + 12 - 0) % 12 = 3 = Cancer
        assert_eq!(compute_varga_sign(30.0, VargaChart::D24), Rashi::Karka);
        // 0° Cancer (even sign, idx=3): part 0 → Cancer
        assert_eq!(compute_varga_sign(90.0, VargaChart::D24), Rashi::Karka);
    }

    #[test]
    fn d24_even_sign_backward_sequence() {
        // In Taurus (even sign): 1.25° per part, backward from Cancer
        // Part 0 = Cancer, Part 1 = Gemini, Part 2 = Taurus, Part 3 = Aries
        assert_eq!(compute_varga_sign(30.0, VargaChart::D24), Rashi::Karka); // Part 0
        assert_eq!(compute_varga_sign(31.3, VargaChart::D24), Rashi::Mithuna); // Part 1
        assert_eq!(compute_varga_sign(32.6, VargaChart::D24), Rashi::Vrishabha); // Part 2
        assert_eq!(compute_varga_sign(33.9, VargaChart::D24), Rashi::Mesha); // Part 3
        assert_eq!(compute_varga_sign(35.1, VargaChart::D24), Rashi::Meena); // Part 4 (wraps)
    }

    #[test]
    fn d24_odd_sign_forward_sequence() {
        // In Aries (odd sign): 1.25° per part, forward from Leo
        // Part 0 = Leo, Part 1 = Virgo, Part 2 = Libra
        assert_eq!(compute_varga_sign(0.0, VargaChart::D24), Rashi::Simha);
        assert_eq!(compute_varga_sign(1.3, VargaChart::D24), Rashi::Kanya);
        assert_eq!(compute_varga_sign(2.6, VargaChart::D24), Rashi::Tula);
    }

    #[test]
    fn d24_last_part_odd_sign() {
        // 29.9° Aries: part = floor(29.9 / 1.25) = floor(23.92) = 23
        // start=4 + 23 = 27 % 12 = 3 = Cancer
        assert_eq!(compute_varga_sign(29.9, VargaChart::D24), Rashi::Karka);
    }

    // ---- D27 (Bhamsa) tests ----

    #[test]
    fn d27_fire_sign_starts_aries() {
        // 0° Aries (fire, idx=0): start=0(Aries)
        assert_eq!(compute_varga_sign(0.0, VargaChart::D27), Rashi::Mesha);
        // 0° Leo (fire, idx=4): start=0(Aries)
        assert_eq!(compute_varga_sign(120.0, VargaChart::D27), Rashi::Mesha);
        // 0° Sagittarius (fire, idx=8): start=0(Aries)
        assert_eq!(compute_varga_sign(240.0, VargaChart::D27), Rashi::Mesha);
    }

    #[test]
    fn d27_earth_sign_starts_cancer() {
        // 0° Taurus (earth, idx=1): start=3(Cancer)
        assert_eq!(compute_varga_sign(30.0, VargaChart::D27), Rashi::Karka);
        // 0° Virgo (earth, idx=5): start=3(Cancer)
        assert_eq!(compute_varga_sign(150.0, VargaChart::D27), Rashi::Karka);
        // 0° Capricorn (earth, idx=9): start=3(Cancer)
        assert_eq!(compute_varga_sign(270.0, VargaChart::D27), Rashi::Karka);
    }

    #[test]
    fn d27_air_sign_starts_libra() {
        // 0° Gemini (air, idx=2): start=6(Libra)
        assert_eq!(compute_varga_sign(60.0, VargaChart::D27), Rashi::Tula);
        // 0° Libra (air, idx=6): start=6(Libra)
        assert_eq!(compute_varga_sign(180.0, VargaChart::D27), Rashi::Tula);
        // 0° Aquarius (air, idx=10): start=6(Libra)
        assert_eq!(compute_varga_sign(300.0, VargaChart::D27), Rashi::Tula);
    }

    #[test]
    fn d27_water_sign_starts_capricorn() {
        // 0° Cancer (water, idx=3): start=9(Capricorn)
        assert_eq!(compute_varga_sign(90.0, VargaChart::D27), Rashi::Makara);
        // 0° Scorpio (water, idx=7): start=9(Capricorn)
        assert_eq!(compute_varga_sign(210.0, VargaChart::D27), Rashi::Makara);
        // 0° Pisces (water, idx=11): start=9(Capricorn)
        assert_eq!(compute_varga_sign(330.0, VargaChart::D27), Rashi::Makara);
    }

    #[test]
    fn d27_sequential_in_fire() {
        // In Aries (fire): 30/27 ≈ 1.111° per part, start from Aries
        // Part 0 = Aries, Part 1 = Taurus, Part 2 = Gemini
        assert_eq!(compute_varga_sign(0.5, VargaChart::D27), Rashi::Mesha);
        assert_eq!(compute_varga_sign(1.2, VargaChart::D27), Rashi::Vrishabha);
        assert_eq!(compute_varga_sign(2.3, VargaChart::D27), Rashi::Mithuna);
    }

    #[test]
    fn d27_last_part_fire() {
        // 29.9° Aries: part = floor(29.9 / (30/27)) = floor(29.9 * 27/30) = floor(26.91) = 26
        // start=0 + 26 = 26 % 12 = 2 = Gemini
        assert_eq!(compute_varga_sign(29.9, VargaChart::D27), Rashi::Mithuna);
    }

    // ---- D40 (Khavedamsa) tests ----

    #[test]
    fn d40_odd_sign_starts_aries() {
        // 0° Aries (odd, idx=0): start=0(Aries)
        assert_eq!(compute_varga_sign(0.0, VargaChart::D40), Rashi::Mesha);
        // 0° Gemini (odd, idx=2): start=0(Aries)
        assert_eq!(compute_varga_sign(60.0, VargaChart::D40), Rashi::Mesha);
        // 0° Leo (odd, idx=4): start=0(Aries)
        assert_eq!(compute_varga_sign(120.0, VargaChart::D40), Rashi::Mesha);
    }

    #[test]
    fn d40_even_sign_starts_libra() {
        // 0° Taurus (even, idx=1): start=6(Libra)
        assert_eq!(compute_varga_sign(30.0, VargaChart::D40), Rashi::Tula);
        // 0° Cancer (even, idx=3): start=6(Libra)
        assert_eq!(compute_varga_sign(90.0, VargaChart::D40), Rashi::Tula);
        // 0° Virgo (even, idx=5): start=6(Libra)
        assert_eq!(compute_varga_sign(150.0, VargaChart::D40), Rashi::Tula);
    }

    #[test]
    fn d40_sequential_odd() {
        // In Aries (odd): 0.75° per part, start from Aries
        // Part 0 = Aries, Part 1 = Taurus, Part 2 = Gemini, Part 3 = Cancer
        assert_eq!(compute_varga_sign(0.3, VargaChart::D40), Rashi::Mesha);
        assert_eq!(compute_varga_sign(0.8, VargaChart::D40), Rashi::Vrishabha);
        assert_eq!(compute_varga_sign(1.6, VargaChart::D40), Rashi::Mithuna);
        assert_eq!(compute_varga_sign(2.3, VargaChart::D40), Rashi::Karka);
    }

    #[test]
    fn d40_last_part_odd() {
        // 29.9° Aries: part = floor(29.9 / 0.75) = floor(39.866) = 39
        // start=0 + 39 = 39 % 12 = 3 = Cancer
        assert_eq!(compute_varga_sign(29.9, VargaChart::D40), Rashi::Karka);
    }

    #[test]
    fn d40_last_part_even() {
        // 29.9° Taurus: lon=59.9°, deg_in_sign=29.9°
        // part = floor(29.9 / 0.75) = 39
        // start=6 + 39 = 45 % 12 = 9 = Capricorn
        assert_eq!(compute_varga_sign(59.9, VargaChart::D40), Rashi::Makara);
    }

    // ---- D45 (Akshavedamsa) tests ----

    #[test]
    fn d45_movable_sign_starts_aries() {
        // 0° Aries (movable, idx=0): start=0(Aries)
        assert_eq!(compute_varga_sign(0.0, VargaChart::D45), Rashi::Mesha);
        // 0° Cancer (movable, idx=3): start=0(Aries)
        assert_eq!(compute_varga_sign(90.0, VargaChart::D45), Rashi::Mesha);
        // 0° Libra (movable, idx=6): start=0(Aries)
        assert_eq!(compute_varga_sign(180.0, VargaChart::D45), Rashi::Mesha);
    }

    #[test]
    fn d45_fixed_sign_starts_leo() {
        // 0° Taurus (fixed, idx=1): start=4(Leo)
        assert_eq!(compute_varga_sign(30.0, VargaChart::D45), Rashi::Simha);
        // 0° Leo (fixed, idx=4): start=4(Leo)
        assert_eq!(compute_varga_sign(120.0, VargaChart::D45), Rashi::Simha);
    }

    #[test]
    fn d45_dual_sign_starts_sagittarius() {
        // 0° Gemini (dual, idx=2): start=8(Sagittarius)
        assert_eq!(compute_varga_sign(60.0, VargaChart::D45), Rashi::Dhanu);
        // 0° Virgo (dual, idx=5): start=8(Sagittarius)
        assert_eq!(compute_varga_sign(150.0, VargaChart::D45), Rashi::Dhanu);
    }

    #[test]
    fn d45_sequential_check() {
        // In Aries (movable): 30/45 = 0.6667° per part, start from Aries
        // Part 0 = Aries, Part 1 = Taurus, Part 2 = Gemini
        assert_eq!(compute_varga_sign(0.3, VargaChart::D45), Rashi::Mesha);
        assert_eq!(compute_varga_sign(0.7, VargaChart::D45), Rashi::Vrishabha);
        assert_eq!(compute_varga_sign(1.4, VargaChart::D45), Rashi::Mithuna);
    }

    #[test]
    fn d45_last_part_movable() {
        // 29.9° Aries: part = floor(29.9 / (30/45)) = floor(29.9 * 45/30) = floor(44.85) = 44
        // start=0 + 44 = 44 % 12 = 8 = Sagittarius
        assert_eq!(compute_varga_sign(29.9, VargaChart::D45), Rashi::Dhanu);
    }

    // ---- Cross-chart boundary consistency ----

    #[test]
    fn all_new_vargas_produce_valid_rashi_fine_grained() {
        let charts = [
            VargaChart::D16,
            VargaChart::D20,
            VargaChart::D24,
            VargaChart::D27,
            VargaChart::D40,
            VargaChart::D45,
        ];
        // Test every degree with 0.1° resolution
        let mut lon = 0.0_f64;
        while lon < 360.0 {
            for chart in &charts {
                let rashi = compute_varga_sign(lon, *chart);
                assert!(
                    rashi.index() < 12,
                    "{chart:?} at {lon:.1}° produced invalid rashi"
                );
            }
            lon += 0.1;
        }
    }

    // ---- Vargottama by longitude ----

    #[test]
    fn vargottama_by_longitude_0_aries() {
        // 0° Aries: D1=Aries, D9 for fire sign starts from self → navamsa 1 = Aries
        assert!(is_vargottama_by_longitude(0.0));
    }

    #[test]
    fn vargottama_by_longitude_mid_aries_not() {
        // 10° Aries: D1=Aries, D9 = navamsa 4 (10/3.333=3 → 4th) = Cancer
        assert!(!is_vargottama_by_longitude(10.0));
    }

    // ---- find_vargottama_planets batch ----

    #[test]
    fn find_vargottama_planets_basic() {
        let positions = vec![
            ("Sun".to_string(), 0.0),   // 0° Aries → vargottama
            ("Moon".to_string(), 10.0),  // 10° Aries → not vargottama
            ("Mars".to_string(), 30.0),  // 0° Taurus: D1=Taurus, D9 earth start=Cap, nav1=Cap → not
        ];
        let result = find_vargottama_planets(&positions);
        assert_eq!(result, vec!["Sun".to_string()]);
    }

    #[test]
    fn find_vargottama_planets_empty_input() {
        let result = find_vargottama_planets(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn find_vargottama_planets_none_match() {
        // 10° Aries: D1=Aries, D9=Cancer (fire sign, nav 4, 0+3=3=Cancer)
        // 40° = 10° Taurus: D1=Taurus, D9=Capricorn (earth, nav 4, 9+3=12%12=0=Aries)
        let positions = vec![
            ("Sun".to_string(), 10.0),
            ("Moon".to_string(), 40.0),
        ];
        let result = find_vargottama_planets(&positions);
        assert!(result.is_empty());
    }

    #[test]
    fn find_vargottama_planets_multiple_match() {
        // Both 0° Aries and 0° Cancer should be vargottama
        // 0° Cancer: water sign, D9 water starts from Cancer, nav1=Cancer → vargottama
        let positions = vec![
            ("Sun".to_string(), 0.0),    // 0° Aries → vargottama
            ("Moon".to_string(), 90.0),  // 0° Cancer → vargottama
            ("Mars".to_string(), 15.0),  // 15° Aries → not
        ];
        let result = find_vargottama_planets(&positions);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"Sun".to_string()));
        assert!(result.contains(&"Moon".to_string()));
    }
}
