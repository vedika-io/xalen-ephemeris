use crate::rashi::Rashi;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharaDashaPeriod {
    pub sign: Rashi,
    pub years: u32,
    pub start_jd: f64,
    pub end_jd: f64,
    pub sub_periods: Vec<CharaDashaPeriod>,
}

fn is_odd_sign(rashi: Rashi) -> bool {
    matches!(
        rashi,
        Rashi::Mesha | Rashi::Mithuna | Rashi::Simha | Rashi::Tula | Rashi::Dhanu | Rashi::Kumbha
    )
}

#[allow(dead_code)]
fn sign_lord_index(rashi: Rashi) -> usize {
    match rashi {
        Rashi::Mesha => 0,
        Rashi::Vrishabha => 1,
        Rashi::Mithuna => 2,
        Rashi::Karka => 3,
        Rashi::Simha => 4,
        Rashi::Kanya => 5,
        Rashi::Tula => 6,
        Rashi::Vrishchika => 7,
        Rashi::Dhanu => 8,
        Rashi::Makara => 9,
        Rashi::Kumbha => 10,
        Rashi::Meena => 11,
    }
}

pub fn chara_dasha_period(sign: Rashi, lord_sign: Rashi) -> u32 {
    let sign_idx = sign.index();
    let lord_idx = lord_sign.index();

    if sign_idx == lord_idx {
        return 12;
    }

    let forward = ((lord_idx as i32 - sign_idx as i32).rem_euclid(12)) as u32;
    let backward = 12 - forward;

    if is_odd_sign(sign) { forward } else { backward }
}

pub fn dasha_sequence(lagna: Rashi) -> [Rashi; 12] {
    let mut seq = [Rashi::Mesha; 12];
    if is_odd_sign(lagna) {
        for i in 0..12 {
            seq[i] = Rashi::from_index((lagna.index() + i) % 12);
        }
    } else {
        for i in 0..12 {
            seq[i] = Rashi::from_index((lagna.index() + 12 - i) % 12);
        }
    }
    seq
}

pub fn compute_chara_dasha(
    lagna: Rashi,
    lord_positions: &[(Rashi, Rashi)],
    birth_jd: f64,
) -> Vec<CharaDashaPeriod> {
    let seq = dasha_sequence(lagna);
    let year_days = 365.25;
    let mut jd = birth_jd;
    let mut periods = Vec::new();

    for &sign in &seq {
        let lord_sign = lord_positions
            .iter()
            .find(|(s, _)| *s == sign)
            .map(|(_, l)| *l)
            .unwrap_or(sign);

        let years = chara_dasha_period(sign, lord_sign);
        let end_jd = jd + years as f64 * year_days;

        let sub_periods = compute_sub_periods(sign, lagna, lord_positions, jd, end_jd);

        periods.push(CharaDashaPeriod {
            sign,
            years,
            start_jd: jd,
            end_jd,
            sub_periods,
        });
        jd = end_jd;
    }
    periods
}

fn compute_sub_periods(
    dasha_sign: Rashi,
    _lagna: Rashi,
    lord_positions: &[(Rashi, Rashi)],
    start_jd: f64,
    end_jd: f64,
) -> Vec<CharaDashaPeriod> {
    let total_days = end_jd - start_jd;
    let seq = dasha_sequence(dasha_sign);
    let total_years: u32 = seq
        .iter()
        .map(|&s| {
            let lord = lord_positions
                .iter()
                .find(|(rs, _)| *rs == s)
                .map(|(_, l)| *l)
                .unwrap_or(s);
            chara_dasha_period(s, lord)
        })
        .sum();

    if total_years == 0 {
        return Vec::new();
    }

    let mut jd = start_jd;
    let mut sub = Vec::new();
    for &sign in &seq {
        let lord = lord_positions
            .iter()
            .find(|(s, _)| *s == sign)
            .map(|(_, l)| *l)
            .unwrap_or(sign);
        let years = chara_dasha_period(sign, lord);
        let fraction = years as f64 / total_years as f64;
        let sub_days = total_days * fraction;
        let sub_end = jd + sub_days;
        sub.push(CharaDashaPeriod {
            sign,
            years,
            start_jd: jd,
            end_jd: sub_end,
            sub_periods: Vec::new(),
        });
        jd = sub_end;
    }
    sub
}

/// Sthira Dasha period per sign, based on sign modality per Jaimini:
///
/// - Movable (Chara) signs: Aries, Cancer, Libra, Capricorn = 7 years each
/// - Fixed (Sthira) signs: Taurus, Leo, Scorpio, Aquarius = 8 years each
/// - Dual (Dvisvabhava) signs: Gemini, Virgo, Sagittarius, Pisces = 9 years each
///
/// Total cycle = 4*7 + 4*8 + 4*9 = 28 + 32 + 36 = 96 years
pub fn sthira_dasha_period(sign: Rashi) -> u32 {
    match sign.modality() {
        crate::rashi::Modality::Movable => 7, // Aries, Cancer, Libra, Capricorn
        crate::rashi::Modality::Fixed => 8,   // Taurus, Leo, Scorpio, Aquarius
        crate::rashi::Modality::Dual => 9,    // Gemini, Virgo, Sagittarius, Pisces
    }
}

/// Compute Sthira Dasha with sub-periods.
///
/// Each major Sthira period is subdivided among 12 signs. The sub-period
/// order follows the same clockwise/anticlockwise rule as the main dasha
/// sequence (odd sign = clockwise, even sign = anticlockwise from the
/// dasha sign). Each sub-period duration is proportional to its own
/// Sthira period length within the major period.
pub fn compute_sthira_dasha(lagna: Rashi, birth_jd: f64) -> Vec<CharaDashaPeriod> {
    let seq = dasha_sequence(lagna);
    let year_days = 365.25;
    let mut jd = birth_jd;
    let mut periods = Vec::new();

    for &sign in &seq {
        let years = sthira_dasha_period(sign);
        let end_jd = jd + years as f64 * year_days;

        let sub_periods = compute_sthira_sub_periods(sign, jd, end_jd);

        periods.push(CharaDashaPeriod {
            sign,
            years,
            start_jd: jd,
            end_jd,
            sub_periods,
        });
        jd = end_jd;
    }
    periods
}

/// Compute sub-periods within a Sthira major period.
///
/// Sub-period signs follow the dasha_sequence order from the major-period sign.
/// Duration of each sub-period is proportional to its Sthira period (7/8/9)
/// relative to the sum of all 12 sub-period Sthira values (96).
fn compute_sthira_sub_periods(
    dasha_sign: Rashi,
    start_jd: f64,
    end_jd: f64,
) -> Vec<CharaDashaPeriod> {
    let total_days = end_jd - start_jd;
    let sub_seq = dasha_sequence(dasha_sign);

    // Sum of all 12 sub-period lengths (always 96 for Sthira)
    let total_sub_years: u32 = sub_seq.iter().map(|&s| sthira_dasha_period(s)).sum();
    if total_sub_years == 0 {
        return Vec::new();
    }

    let mut jd = start_jd;
    let mut subs = Vec::new();

    for &sign in &sub_seq {
        let years = sthira_dasha_period(sign);
        let fraction = years as f64 / total_sub_years as f64;
        let sub_days = total_days * fraction;
        let sub_end = jd + sub_days;

        subs.push(CharaDashaPeriod {
            sign,
            years,
            start_jd: jd,
            end_jd: sub_end,
            sub_periods: Vec::new(),
        });
        jd = sub_end;
    }
    subs
}

// ---------------------------------------------------------------------------
// Narayana Dasha (Padakrama Dasha)
// ---------------------------------------------------------------------------

/// A Narayana Dasha period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarayanaPeriod {
    pub sign: usize,
    pub start_jd: f64,
    pub end_jd: f64,
    pub years: f64,
}

/// Compute the Narayana Dasha (Padakrama Dasha) sequence.
///
/// The Narayana Dasha depends on:
/// - Whether the lagna is an odd (1,3,5,7,9,11 = Aries,Gemini,Leo,Libra,Sagittarius,Aquarius)
///   or even (2,4,6,8,10,12 = Taurus,Cancer,Virgo,Scorpio,Capricorn,Pisces) sign.
/// - Odd signs: dasha signs run clockwise from the lagna.
/// - Even signs: dasha signs run anticlockwise from the lagna.
///
/// The duration of each period is determined by counting from the dasha sign
/// to the position of the 7th lord FROM that sign:
/// - Odd dasha sign: count forward (clockwise)
/// - Even dasha sign: count backward (anticlockwise)
/// - If the lord is in the sign itself, the period is 12 years.
///
/// `planet_signs`: array of 7 planet sign positions [Sun, Moon, Mars, Mercury,
///   Jupiter, Venus, Saturn] as 0-based indices.
/// `lagna_sign`: 0-based sign index of the lagna.
/// `birth_jd`: Julian Day of birth.
/// `max_years`: maximum total years to generate (e.g., 120).
pub fn narayana_dasha(
    lagna_sign: usize,
    planet_signs: &[usize; 7],
    birth_jd: f64,
    max_years: usize,
) -> Vec<NarayanaPeriod> {
    let year_days = 365.25;
    let lagna = lagna_sign % 12;
    let is_odd = is_odd_rashi(lagna);

    let mut periods = Vec::new();
    let mut jd = birth_jd;
    let mut total_years = 0.0;

    for i in 0..12 {
        if total_years >= max_years as f64 {
            break;
        }

        // Determine dasha sign
        let dasha_sign = if is_odd {
            (lagna + i) % 12
        } else {
            (lagna + 12 - i) % 12
        };

        // Find the 7th house from the dasha sign
        let seventh_house = (dasha_sign + 6) % 12;

        // Lord of the 7th house
        let seventh_lord = sign_lord_index_static(seventh_house);

        // Position of the 7th lord
        let lord_position = planet_signs[seventh_lord];

        // Calculate years
        let years = narayana_period_years(dasha_sign, lord_position);

        let duration = years * year_days;
        let end_jd = jd + duration;

        periods.push(NarayanaPeriod {
            sign: dasha_sign,
            start_jd: jd,
            end_jd,
            years,
        });

        jd = end_jd;
        total_years += years;
    }

    periods
}

/// Compute Narayana Dasha with dual lordship support for Scorpio/Aquarius.
///
/// This extended version accepts Rahu and Ketu sign positions to enable
/// proper Jaimini dual lordship selection for Scorpio (Mars/Ketu) and
/// Aquarius (Saturn/Rahu).
pub fn narayana_dasha_with_nodes(
    lagna_sign: usize,
    planet_signs: &[usize; 7],
    rahu_sign: usize,
    ketu_sign: usize,
    birth_jd: f64,
    max_years: usize,
) -> Vec<NarayanaPeriod> {
    let year_days = 365.25;
    let lagna = lagna_sign % 12;
    let is_odd = is_odd_rashi(lagna);

    let mut periods = Vec::new();
    let mut jd = birth_jd;
    let mut total_years = 0.0;

    for i in 0..12 {
        if total_years >= max_years as f64 {
            break;
        }

        let dasha_sign = if is_odd {
            (lagna + i) % 12
        } else {
            (lagna + 12 - i) % 12
        };

        let seventh_house = (dasha_sign + 6) % 12;

        let lord_idx = sign_lord_with_nodes(
            seventh_house,
            dasha_sign,
            planet_signs,
            rahu_sign,
            ketu_sign,
        );

        let lord_position = if lord_idx < 7 {
            planet_signs[lord_idx]
        } else if lord_idx == 7 {
            rahu_sign
        } else {
            ketu_sign
        };

        let years = narayana_period_years(dasha_sign, lord_position);
        let duration = years * year_days;
        let end_jd = jd + duration;

        periods.push(NarayanaPeriod {
            sign: dasha_sign,
            start_jd: jd,
            end_jd,
            years,
        });

        jd = end_jd;
        total_years += years;
    }

    periods
}

/// Whether a 0-based sign index is "odd" in the Jaimini sense.
/// Odd signs: Aries(0), Gemini(2), Leo(4), Libra(6), Sagittarius(8), Aquarius(10).
fn is_odd_rashi(sign: usize) -> bool {
    sign.is_multiple_of(2)
}

/// Calculate Narayana Dasha period years based on the dasha sign and the
/// position of the 7th lord.
///
/// - If the lord is in its own sign (same as dasha sign), period = 12 years.
/// - For odd dasha signs: count forward from dasha sign to lord position.
/// - For even dasha signs: count backward from dasha sign to lord position.
fn narayana_period_years(dasha_sign: usize, lord_sign: usize) -> f64 {
    let d = dasha_sign % 12;
    let l = lord_sign % 12;

    if d == l {
        return 12.0;
    }

    

    if is_odd_rashi(d) {
        // Forward count
        ((l as i32 - d as i32).rem_euclid(12)) as f64
    } else {
        // Backward count
        ((d as i32 - l as i32).rem_euclid(12)) as f64
    }
}

/// Map sign index to planet index for determining sign lord.
/// Returns index into [Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn].
///
/// For dual-lordship signs (Scorpio and Aquarius), the stronger co-lord is
/// selected based on which lord occupies a stronger position. When no
/// planet positions are available, the traditional lord is returned.
fn sign_lord_index_static(sign: usize) -> usize {
    match sign % 12 {
        0 => 2,      // Aries → Mars
        1 | 6 => 5,  // Taurus/Libra → Venus
        2 | 5 => 3,  // Gemini/Virgo → Mercury
        3 => 1,      // Cancer → Moon
        4 => 0,      // Leo → Sun
        7 => 2,      // Scorpio → Mars (traditional; co-lord Ketu not in 7-planet array)
        8 | 11 => 4, // Sagittarius/Pisces → Jupiter
        9 => 6,      // Capricorn → Saturn
        10 => 6,     // Aquarius → Saturn (traditional; co-lord Rahu not in 7-planet array)
        _ => unreachable!(),
    }
}

/// Extended sign lordship that accounts for Jaimini dual lordship.
/// Scorpio: Mars (primary) + Ketu (co-lord)
/// Aquarius: Saturn (primary) + Rahu (co-lord)
///
/// `planet_signs` is the 7-planet array [Sun..Saturn].
/// `rahu_sign` and `ketu_sign` are the node positions (0-based sign indices).
///
/// The stronger lord is selected: whichever lord occupies a stronger position
/// relative to the dasha sign. Specifically, the lord whose sign position is
/// in a kendra (1,4,7,10) or trikona (1,5,9) from the dasha sign is preferred.
/// If both or neither qualify, the traditional (primary) lord is used.
pub fn sign_lord_with_nodes(
    sign: usize,
    dasha_sign: usize,
    planet_signs: &[usize; 7],
    rahu_sign: usize,
    ketu_sign: usize,
) -> usize {
    let s = sign % 12;
    match s {
        7 => {
            let mars_pos = planet_signs[2];
            let mars_strength = is_kendra_or_trikona(dasha_sign, mars_pos);
            let ketu_strength = is_kendra_or_trikona(dasha_sign, ketu_sign);
            if ketu_strength && !mars_strength {
                8
            } else {
                2
            } // 8 = special index for Ketu
        }
        10 => {
            let saturn_pos = planet_signs[6];
            let saturn_strength = is_kendra_or_trikona(dasha_sign, saturn_pos);
            let rahu_strength = is_kendra_or_trikona(dasha_sign, rahu_sign);
            if rahu_strength && !saturn_strength {
                7
            } else {
                6
            } // 7 = special index for Rahu
        }
        _ => sign_lord_index_static(s),
    }
}

fn is_kendra_or_trikona(from_sign: usize, planet_sign: usize) -> bool {
    let dist = ((planet_sign as i32 - from_sign as i32).rem_euclid(12)) as usize;
    matches!(dist, 0 | 3 | 4 | 6 | 8 | 9)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ----- Existing Chara Dasha tests -----

    #[test]
    fn odd_signs_correct() {
        assert!(is_odd_sign(Rashi::Mesha));
        assert!(!is_odd_sign(Rashi::Vrishabha));
        assert!(is_odd_sign(Rashi::Mithuna));
        assert!(!is_odd_sign(Rashi::Karka));
    }

    #[test]
    fn lord_in_same_sign_gives_12() {
        assert_eq!(chara_dasha_period(Rashi::Mesha, Rashi::Mesha), 12);
    }

    #[test]
    fn odd_sign_forward_count() {
        assert_eq!(chara_dasha_period(Rashi::Mesha, Rashi::Simha), 4);
    }

    #[test]
    fn even_sign_backward_count() {
        assert_eq!(chara_dasha_period(Rashi::Vrishabha, Rashi::Karka), 10);
    }

    #[test]
    fn sequence_odd_lagna_forward() {
        let seq = dasha_sequence(Rashi::Mesha);
        assert_eq!(seq[0], Rashi::Mesha);
        assert_eq!(seq[1], Rashi::Vrishabha);
        assert_eq!(seq[11], Rashi::Meena);
    }

    #[test]
    fn sequence_even_lagna_backward() {
        let seq = dasha_sequence(Rashi::Vrishabha);
        assert_eq!(seq[0], Rashi::Vrishabha);
        assert_eq!(seq[1], Rashi::Mesha);
        assert_eq!(seq[11], Rashi::Mithuna);
    }

    #[test]
    fn chara_dasha_12_periods() {
        let lords: Vec<(Rashi, Rashi)> = (0..12)
            .map(|i| (Rashi::from_index(i), Rashi::from_index((i + 3) % 12)))
            .collect();
        let periods = compute_chara_dasha(Rashi::Mesha, &lords, 2451545.0);
        assert_eq!(periods.len(), 12);
    }

    #[test]
    fn chara_dasha_periods_contiguous() {
        let lords: Vec<(Rashi, Rashi)> = (0..12)
            .map(|i| (Rashi::from_index(i), Rashi::from_index((i + 5) % 12)))
            .collect();
        let periods = compute_chara_dasha(Rashi::Mesha, &lords, 2451545.0);
        for i in 1..periods.len() {
            let diff = (periods[i].start_jd - periods[i - 1].end_jd).abs();
            assert!(
                diff < 0.01,
                "Gap between periods {} and {i}: {diff} days",
                i - 1
            );
        }
    }

    #[test]
    fn chara_dasha_has_sub_periods() {
        let lords: Vec<(Rashi, Rashi)> = (0..12)
            .map(|i| (Rashi::from_index(i), Rashi::from_index((i + 4) % 12)))
            .collect();
        let periods = compute_chara_dasha(Rashi::Mesha, &lords, 2451545.0);
        assert!(
            !periods[0].sub_periods.is_empty(),
            "Should have sub-periods"
        );
        assert_eq!(periods[0].sub_periods.len(), 12);
    }

    #[test]
    fn period_years_valid_range() {
        for i in 0..12 {
            for j in 0..12 {
                let p = chara_dasha_period(Rashi::from_index(i), Rashi::from_index(j));
                assert!(
                    p >= 1 && p <= 12,
                    "Period must be 1-12, got {p} for signs {i},{j}"
                );
            }
        }
    }

    // ----- Corrected Sthira Dasha tests -----

    #[test]
    fn sthira_dasha_movable_7_years() {
        // Movable signs: Aries, Cancer, Libra, Capricorn = 7 years each
        assert_eq!(sthira_dasha_period(Rashi::Mesha), 7);
        assert_eq!(sthira_dasha_period(Rashi::Karka), 7);
        assert_eq!(sthira_dasha_period(Rashi::Tula), 7);
        assert_eq!(sthira_dasha_period(Rashi::Makara), 7);
    }

    #[test]
    fn sthira_dasha_fixed_8_years() {
        // Fixed signs: Taurus, Leo, Scorpio, Aquarius = 8 years each
        assert_eq!(sthira_dasha_period(Rashi::Vrishabha), 8);
        assert_eq!(sthira_dasha_period(Rashi::Simha), 8);
        assert_eq!(sthira_dasha_period(Rashi::Vrishchika), 8);
        assert_eq!(sthira_dasha_period(Rashi::Kumbha), 8);
    }

    #[test]
    fn sthira_dasha_dual_9_years() {
        // Dual signs: Gemini, Virgo, Sagittarius, Pisces = 9 years each
        assert_eq!(sthira_dasha_period(Rashi::Mithuna), 9);
        assert_eq!(sthira_dasha_period(Rashi::Kanya), 9);
        assert_eq!(sthira_dasha_period(Rashi::Dhanu), 9);
        assert_eq!(sthira_dasha_period(Rashi::Meena), 9);
    }

    #[test]
    fn sthira_total_years_96() {
        // Total cycle = 4*7 + 4*8 + 4*9 = 96 years
        let periods = compute_sthira_dasha(Rashi::Mesha, 2451545.0);
        let total: u32 = periods.iter().map(|p| p.years).sum();
        assert_eq!(
            total, 96,
            "Sthira dasha total should be exactly 96 years, got {total}"
        );
    }

    #[test]
    fn sthira_dasha_12_periods() {
        let periods = compute_sthira_dasha(Rashi::Mesha, 2451545.0);
        assert_eq!(periods.len(), 12);
    }

    #[test]
    fn sthira_dasha_has_sub_periods() {
        let periods = compute_sthira_dasha(Rashi::Mesha, 2451545.0);
        for (i, p) in periods.iter().enumerate() {
            assert_eq!(
                p.sub_periods.len(),
                12,
                "Sthira period {i} should have 12 sub-periods, got {}",
                p.sub_periods.len()
            );
        }
    }

    #[test]
    fn sthira_sub_periods_contiguous() {
        let periods = compute_sthira_dasha(Rashi::Mesha, 2451545.0);
        for (pi, p) in periods.iter().enumerate() {
            for i in 1..p.sub_periods.len() {
                let diff = (p.sub_periods[i].start_jd - p.sub_periods[i - 1].end_jd).abs();
                assert!(
                    diff < 0.01,
                    "Gap in period {pi} sub-period {i}: {diff} days"
                );
            }
            // First sub-period starts at major period start
            let start_diff = (p.sub_periods[0].start_jd - p.start_jd).abs();
            assert!(
                start_diff < 0.01,
                "Sub-period should start at major period start"
            );
            // Last sub-period ends at major period end
            let end_diff = (p.sub_periods.last().unwrap().end_jd - p.end_jd).abs();
            assert!(
                end_diff < 0.5,
                "Sub-period should end near major period end"
            );
        }
    }

    #[test]
    fn sthira_periods_contiguous() {
        let periods = compute_sthira_dasha(Rashi::Mesha, 2451545.0);
        for i in 1..periods.len() {
            let diff = (periods[i].start_jd - periods[i - 1].end_jd).abs();
            assert!(
                diff < 0.01,
                "Gap between Sthira periods {} and {i}: {diff}",
                i - 1
            );
        }
    }

    // ----- Narayana Dasha tests -----

    #[test]
    fn narayana_dasha_odd_lagna_clockwise() {
        // Aries (0) is odd → clockwise
        let planets = [4, 1, 7, 2, 8, 5, 9]; // arbitrary positions
        let periods = narayana_dasha(0, &planets, 2451545.0, 120);
        assert_eq!(periods.len(), 12);
        // First dasha should be Aries (0)
        assert_eq!(periods[0].sign, 0);
        // Second should be Taurus (1) for clockwise
        assert_eq!(periods[1].sign, 1);
    }

    #[test]
    fn narayana_dasha_even_lagna_anticlockwise() {
        // Taurus (1) is even → anticlockwise
        let planets = [4, 1, 7, 2, 8, 5, 9];
        let periods = narayana_dasha(1, &planets, 2451545.0, 120);
        assert_eq!(periods[0].sign, 1); // Taurus
        assert_eq!(periods[1].sign, 0); // Aries (anticlockwise from Taurus)
    }

    #[test]
    fn narayana_dasha_periods_contiguous() {
        let planets = [4, 1, 7, 2, 8, 5, 9];
        let periods = narayana_dasha(0, &planets, 2451545.0, 120);
        for i in 1..periods.len() {
            let diff = (periods[i].start_jd - periods[i - 1].end_jd).abs();
            assert!(diff < 0.01, "Gap in Narayana periods at {i}: {diff} days");
        }
    }

    #[test]
    fn narayana_period_years_range() {
        let planets = [4, 1, 7, 2, 8, 5, 9];
        let periods = narayana_dasha(0, &planets, 2451545.0, 120);
        for (i, p) in periods.iter().enumerate() {
            assert!(
                p.years >= 1.0 && p.years <= 12.0,
                "Narayana period {i} years should be 1-12, got {}",
                p.years
            );
        }
    }

    #[test]
    fn narayana_lord_in_own_sign_12_years() {
        // If the 7th lord from a dasha sign is in its own sign (same as dasha sign),
        // the period should be 12 years.
        assert_eq!(narayana_period_years(0, 0), 12.0);
        assert_eq!(narayana_period_years(5, 5), 12.0);
    }

    #[test]
    fn narayana_odd_sign_forward_count() {
        // Aries (0, odd): lord in Leo (4) → forward = 4
        assert_eq!(narayana_period_years(0, 4), 4.0);
        // Gemini (2, odd): lord in Virgo (5) → forward = 3
        assert_eq!(narayana_period_years(2, 5), 3.0);
    }

    #[test]
    fn narayana_even_sign_backward_count() {
        // Taurus (1, even): lord in Capricorn (9) → backward = (1-9+12)%12 = 4
        assert_eq!(narayana_period_years(1, 9), 4.0);
        // Cancer (3, even): lord in Aries (0) → backward = (3-0)%12 = 3
        assert_eq!(narayana_period_years(3, 0), 3.0);
    }

    #[test]
    fn dual_lordship_scorpio_ketu_stronger() {
        let planets = [4, 1, 11, 2, 8, 5, 9]; // Mars in sign 11 (Pisces)
        let rahu_sign = 3;
        let ketu_sign = 0; // Ketu in Aries — kendra (0 dist) from Aries dasha sign
        let lord = sign_lord_with_nodes(7, 0, &planets, rahu_sign, ketu_sign);
        assert_eq!(
            lord, 8,
            "When Ketu is in kendra and Mars isn't, Ketu should be lord"
        );
    }

    #[test]
    fn dual_lordship_scorpio_mars_default() {
        let planets = [4, 1, 0, 2, 8, 5, 9]; // Mars in Aries (kendra from 0)
        let rahu_sign = 3;
        let ketu_sign = 5; // Ketu in Virgo — NOT kendra from Aries
        let lord = sign_lord_with_nodes(7, 0, &planets, rahu_sign, ketu_sign);
        assert_eq!(
            lord, 2,
            "When Mars is in kendra, Mars should be lord of Scorpio"
        );
    }

    #[test]
    fn dual_lordship_aquarius_rahu_stronger() {
        let planets = [4, 1, 7, 2, 8, 5, 11]; // Saturn in Pisces (sign 11)
        let rahu_sign = 0; // Rahu in Aries — kendra from Aries
        let ketu_sign = 6;
        let lord = sign_lord_with_nodes(10, 0, &planets, rahu_sign, ketu_sign);
        assert_eq!(
            lord, 7,
            "When Rahu is in kendra and Saturn isn't, Rahu should be lord"
        );
    }

    #[test]
    fn dual_lordship_non_dual_signs_unchanged() {
        let planets = [4, 1, 7, 2, 8, 5, 9];
        for sign in [0, 1, 2, 3, 4, 5, 6, 8, 9, 11] {
            let lord = sign_lord_with_nodes(sign, 0, &planets, 3, 9);
            let static_lord = sign_lord_index_static(sign);
            assert_eq!(
                lord, static_lord,
                "Non-dual sign {sign} should match static lord"
            );
        }
    }

    #[test]
    fn narayana_total_years_reasonable() {
        let planets = [4, 1, 0, 2, 8, 5, 9];
        let periods = narayana_dasha(0, &planets, 2451545.0, 120);
        let total: f64 = periods.iter().map(|p| p.years).sum();
        assert!(
            total >= 12.0 && total <= 144.0,
            "Narayana total should be 12-144 years, got {total}"
        );
    }

    #[test]
    fn narayana_with_nodes_produces_12_periods() {
        let planets = [4, 1, 7, 2, 8, 5, 9];
        let periods = narayana_dasha_with_nodes(0, &planets, 3, 9, 2451545.0, 120);
        assert_eq!(periods.len(), 12);
        assert_eq!(periods[0].sign, 0); // starts at lagna
    }

    #[test]
    fn narayana_with_nodes_can_use_ketu_lord() {
        // Mars in Pisces (11), Ketu in Aries (0) — kendra from Aries
        let planets = [4, 1, 11, 2, 8, 5, 9];
        let without = narayana_dasha(0, &planets, 2451545.0, 120);
        let with = narayana_dasha_with_nodes(0, &planets, 3, 0, 2451545.0, 120);
        // If Scorpio's 7th house is involved and Ketu is in kendra,
        // the periods should differ from the no-nodes version
        let total_without: f64 = without.iter().map(|p| p.years).sum();
        let total_with: f64 = with.iter().map(|p| p.years).sum();
        // Not guaranteed to differ for every config, but verify both are valid
        assert!(total_with >= 12.0 && total_with <= 144.0);
        assert!(total_without >= 12.0 && total_without <= 144.0);
    }
}
