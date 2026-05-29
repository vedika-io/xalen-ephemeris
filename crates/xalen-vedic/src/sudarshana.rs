//! Sudarshana Chakra — three concentric wheels (Lagna, Moon, Sun).
//!
//! Each wheel is a 12-house chart starting from the respective reference
//! sign.  Year progression advances one house per year, cycling every 12
//! years.  When the same sign is active in two or more wheels simultaneously,
//! classical texts consider that year especially potent.
//!
//! Web-verified:
//! - Reference: BPHS Ch. 29 (Brihat Parashara Hora Shastra).
//! - https://modernastro.com/astrology/vedic/sudarshan-chakra
//! - https://vedicastrology.wikidot.com/sudarshana-chakra
//! - https://www.vedic-astrology.net/kala/sudarshana-chakra.asp
//! - https://medium.com/thoughts-on-jyotish/secrets-of-sudarshana-chakra-dasa-part-2-84214579b0d7
//!
//! Structure: Three charts erected concentrically — outer from Sun, middle
//! from Moon, inner from Lagna. The 1st year (age 0) activates house 1 in
//! all three wheels. Each subsequent year advances to the next house,
//! cycling every 12 years. The basis is that all three tripods of life
//! (Sun, Moon, Lagna) are taken into consideration simultaneously.

use serde::{Deserialize, Serialize};

/// Which wheel of the Sudarshana Chakra.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Wheel {
    Lagna,
    Moon,
    Sun,
}

/// The active house in a single wheel for a given year.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WheelActivation {
    pub wheel: Wheel,
    /// 1-based house number (1..=12) active in this wheel.
    pub active_house: u32,
    /// 0-based sign index (0 = Aries .. 11 = Pisces) that the active house
    /// falls in.
    pub active_sign: usize,
}

/// Combined result for all three wheels at a given age.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SudarshanaResult {
    pub age: u32,
    pub lagna: WheelActivation,
    pub moon: WheelActivation,
    pub sun: WheelActivation,
    /// Signs that are active in more than one wheel simultaneously.
    /// Empty when no overlap; presence indicates a stronger year for
    /// that sign's significations.
    pub overlapping_signs: Vec<usize>,
    /// Number of wheels in which the most-activated sign appears (2 or 3).
    /// 1 means no overlap.
    pub overlap_strength: u32,
}

/// Compute the active house in a single wheel.
///
/// `reference_sign` is the 0-based sign of the wheel's anchor point
/// (lagna sign, moon sign, or sun sign).  `age` is the native's completed
/// years.  House 1 is active at age 0 (birth year), house 2 at age 1, etc.
fn active_house_for_wheel(reference_sign: usize, age: u32) -> WheelActivation {
    let house_0 = (age as usize) % 12; // 0-based offset
    let active_sign = (reference_sign + house_0) % 12;
    let wheel = Wheel::Lagna;
    WheelActivation {
        wheel,
        active_house: (house_0 as u32) + 1,
        active_sign,
    }
}

/// Compute the full Sudarshana Chakra for a given age.
///
/// # Arguments
///
/// * `lagna_sign` — 0-based sign index (0 = Aries) of the natal ascendant.
/// * `moon_sign`  — 0-based sign index of the natal Moon.
/// * `sun_sign`   — 0-based sign index of the natal Sun.
/// * `age`        — completed years of the native (0 = birth year).
///
/// # Returns
///
/// A [`SudarshanaResult`] showing which house/sign is active in each wheel
/// and whether any overlapping activation exists.
pub fn compute_sudarshana(
    lagna_sign: usize,
    moon_sign: usize,
    sun_sign: usize,
    age: u32,
) -> SudarshanaResult {
    let mut lagna = active_house_for_wheel(lagna_sign, age);
    lagna.wheel = Wheel::Lagna;

    let mut moon = active_house_for_wheel(moon_sign, age);
    moon.wheel = Wheel::Moon;

    let mut sun = active_house_for_wheel(sun_sign, age);
    sun.wheel = Wheel::Sun;

    // Count how many wheels activate each sign.
    let mut sign_counts = [0u32; 12];
    sign_counts[lagna.active_sign] += 1;
    sign_counts[moon.active_sign] += 1;
    sign_counts[sun.active_sign] += 1;

    let overlapping_signs: Vec<usize> = sign_counts
        .iter()
        .enumerate()
        .filter(|(_, c)| **c >= 2)
        .map(|(i, _)| i)
        .collect();

    let overlap_strength = sign_counts.iter().copied().max().unwrap_or(1);

    SudarshanaResult {
        age,
        lagna,
        moon,
        sun,
        overlapping_signs,
        overlap_strength,
    }
}

/// Batch-compute Sudarshana Chakra for a range of ages.
///
/// Useful for generating a timeline view (e.g. ages 0..=60).
pub fn compute_sudarshana_range(
    lagna_sign: usize,
    moon_sign: usize,
    sun_sign: usize,
    start_age: u32,
    end_age: u32,
) -> Vec<SudarshanaResult> {
    (start_age..=end_age)
        .map(|age| compute_sudarshana(lagna_sign, moon_sign, sun_sign, age))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Test 1: Birth year (age 0) returns house 1, signs match anchors ---
    #[test]
    fn birth_year_returns_house_one() {
        let r = compute_sudarshana(0, 4, 9, 0); // Aries lagna, Leo Moon, Capricorn Sun
        assert_eq!(r.lagna.active_house, 1);
        assert_eq!(r.moon.active_house, 1);
        assert_eq!(r.sun.active_house, 1);
        assert_eq!(r.lagna.active_sign, 0); // Aries
        assert_eq!(r.moon.active_sign, 4); // Leo
        assert_eq!(r.sun.active_sign, 9); // Capricorn
    }

    // --- Test 2: Age 1 advances to house 2 ---
    #[test]
    fn age_one_advances_to_house_two() {
        let r = compute_sudarshana(0, 4, 9, 1);
        assert_eq!(r.lagna.active_house, 2);
        assert_eq!(r.lagna.active_sign, 1); // Taurus
        assert_eq!(r.moon.active_house, 2);
        assert_eq!(r.moon.active_sign, 5); // Virgo
    }

    // --- Test 3: 12-year cycle returns to the same house ---
    #[test]
    fn twelve_year_cycle() {
        let r0 = compute_sudarshana(3, 7, 11, 0);
        let r12 = compute_sudarshana(3, 7, 11, 12);
        let r24 = compute_sudarshana(3, 7, 11, 24);
        assert_eq!(r0.lagna.active_sign, r12.lagna.active_sign);
        assert_eq!(r0.moon.active_sign, r12.moon.active_sign);
        assert_eq!(r0.sun.active_sign, r12.sun.active_sign);
        assert_eq!(r0.lagna.active_sign, r24.lagna.active_sign);
    }

    // --- Test 4: Triple overlap when all anchors are the same sign ---
    #[test]
    fn triple_overlap_same_anchor() {
        // Lagna, Moon, Sun all in Aries (0) → every year is triple overlap
        let r = compute_sudarshana(0, 0, 0, 5);
        assert_eq!(r.overlap_strength, 3);
        assert_eq!(r.overlapping_signs.len(), 1);
        assert_eq!(r.overlapping_signs[0], 5); // sign at age 5 offset
    }

    // --- Test 5: No overlap when anchors are 4 signs apart ---
    #[test]
    fn no_overlap_distinct_anchors() {
        // 0, 4, 8 are each 4 apart → the 3 active signs never collide
        let r = compute_sudarshana(0, 4, 8, 0);
        assert!(r.overlapping_signs.is_empty());
        assert_eq!(r.overlap_strength, 1);
    }

    // --- Test 6: Double overlap detection ---
    #[test]
    fn double_overlap_two_same_anchor() {
        // Lagna=0 Moon=0 Sun=6 → lagna & moon always overlap, sun does not
        let r = compute_sudarshana(0, 0, 6, 3);
        assert_eq!(r.overlap_strength, 2);
        assert_eq!(r.overlapping_signs.len(), 1);
        assert_eq!(r.overlapping_signs[0], 3); // both lagna & moon land on sign 3
    }

    // --- Test 7: Wheel labels are correct ---
    #[test]
    fn wheel_labels() {
        let r = compute_sudarshana(2, 5, 10, 7);
        assert_eq!(r.lagna.wheel, Wheel::Lagna);
        assert_eq!(r.moon.wheel, Wheel::Moon);
        assert_eq!(r.sun.wheel, Wheel::Sun);
    }

    // --- Test 8: Range computation produces correct count and consistency ---
    #[test]
    fn range_computation() {
        let range = compute_sudarshana_range(1, 6, 11, 0, 11);
        assert_eq!(range.len(), 12);
        // First entry matches direct call
        let single = compute_sudarshana(1, 6, 11, 0);
        assert_eq!(range[0].lagna.active_sign, single.lagna.active_sign);
        assert_eq!(range[0].moon.active_sign, single.moon.active_sign);
        assert_eq!(range[0].sun.active_sign, single.sun.active_sign);
        // Last entry is age 11
        assert_eq!(range[11].age, 11);
        assert_eq!(range[11].lagna.active_house, 12);
    }

    // --- Test 9: Large age wraps correctly ---
    #[test]
    fn large_age_wraps() {
        let r = compute_sudarshana(5, 5, 5, 120);
        // 120 % 12 == 0, so back to house 1
        assert_eq!(r.lagna.active_house, 1);
        assert_eq!(r.lagna.active_sign, 5);
    }

    // --- Test 10: Sign arithmetic does not overflow past 11 ---
    #[test]
    fn sign_wraps_past_pisces() {
        // Pisces (11) + age 3 → sign (11+3)%12 = 2 (Gemini)
        let r = compute_sudarshana(11, 11, 11, 3);
        assert_eq!(r.lagna.active_sign, 2);
        assert_eq!(r.lagna.active_house, 4);
    }
}
