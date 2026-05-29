//! Varshaphal (Solar Return / Annual Horoscopy) timing computations.
//!
//! Covers:
//! - Solar return Julian Day approximation for a given year
//! - Year lord (weekday ruler of the solar return moment)
//! - Muntha lord (sign lord of the Muntha position)
//! - Tri-Pataki Chakra (triple transit check of Jupiter, Saturn, Sun over Muntha)
//!
//! Reference: Tajaka Neelakanthi, Varshaphal-Paddhati.

use serde::{Deserialize, Serialize};

/// Mean tropical year in days (IAU value).
pub const TROPICAL_YEAR_DAYS: f64 = 365.24219;

/// Weekday rulers in order: Sun=0(Sunday) .. Saturn=6(Saturday).
const WEEKDAY_RULERS: [&str; 7] = [
    "Sun",     // Sunday
    "Moon",    // Monday
    "Mars",    // Tuesday
    "Mercury", // Wednesday
    "Jupiter", // Thursday
    "Venus",   // Friday
    "Saturn",  // Saturday
];

/// Result of a Tri-Pataki (triple transit) check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriPatakiResult {
    /// Whether Jupiter transits over (is in the same sign as) the Muntha.
    pub jupiter_on_muntha: bool,
    /// Whether Saturn transits over the Muntha.
    pub saturn_on_muntha: bool,
    /// Whether the Sun transits over the Muntha.
    pub sun_on_muntha: bool,
    /// Number of planets (0..3) transiting the Muntha sign.
    pub count: u32,
    /// True when all three transit the Muntha sign — full Tri-Pataki.
    pub is_full_tripataki: bool,
}

/// Complete Varshaphal timing result for one year.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarshaphalResult {
    /// Which solar return year this represents (0 = birth year).
    pub year: u32,
    /// Approximate Julian Day of the solar return.
    pub solar_return_jd: f64,
    /// Weekday index (0=Sunday .. 6=Saturday) of the solar return.
    pub weekday_index: usize,
    /// Year lord — the planetary ruler of that weekday.
    pub year_lord: &'static str,
    /// 0-based sign index of the Muntha for this year.
    pub muntha_sign: usize,
    /// Lord of the Muntha sign.
    pub muntha_lord: &'static str,
    /// Tri-Pataki check (if planet positions were supplied).
    pub tri_pataki: Option<TriPatakiResult>,
}

/// Sign lords (Vedic rulership, same as `Rashi::lord`).
fn sign_lord(sign: usize) -> &'static str {
    match sign % 12 {
        0 | 7 => "Mars",     // Aries / Scorpio
        1 | 6 => "Venus",    // Taurus / Libra
        2 | 5 => "Mercury",  // Gemini / Virgo
        3 => "Moon",         // Cancer
        4 => "Sun",          // Leo
        8 | 11 => "Jupiter", // Sagittarius / Pisces
        9 | 10 => "Saturn",  // Capricorn / Aquarius
        _ => unreachable!(),
    }
}

/// Compute the approximate solar return Julian Day for a given year.
///
/// This is a simple linear approximation:
///   `return_jd = birth_jd + year * TROPICAL_YEAR_DAYS`
///
/// For precise work a true solar longitude search is needed, but this
/// approximation is standard for Tajaka timing and is accurate to within
/// a few hours for most practical purposes.
pub fn solar_return_jd(birth_jd: f64, year: u32) -> f64 {
    birth_jd + (year as f64) * TROPICAL_YEAR_DAYS
}

/// Determine the weekday index (0=Sunday..6=Saturday) from a Julian Day.
///
/// The Julian Day number modulo 7 gives the weekday, with JD 0 being a
/// Monday.  We use the standard astronomical convention.
pub fn weekday_from_jd(jd: f64) -> usize {
    // JD 0.0 = Monday (1). Add 1 so Sunday=0.
    let day_number = (jd + 0.5).floor() as i64;
    ((day_number + 1) % 7).rem_euclid(7) as usize
}

/// Get the year lord (weekday ruler) for a solar return moment.
pub fn year_lord(solar_return_jd: f64) -> &'static str {
    WEEKDAY_RULERS[weekday_from_jd(solar_return_jd)]
}

/// Compute the Muntha sign for a given year.
///
/// Muntha advances one sign per year from the natal ascendant sign.
/// This is identical to `tajaka::compute_muntha` but provided here
/// for self-contained Varshaphal usage.
pub fn muntha_sign(natal_asc_sign: usize, year: u32) -> usize {
    (natal_asc_sign + year as usize) % 12
}

/// Check Tri-Pataki (triple transit) over the Muntha.
///
/// In Tajaka annual horoscopy, when Jupiter, Saturn, and Sun all transit
/// through the Muntha sign simultaneously, it forms a full Tri-Pataki
/// Chakra — considered highly significant for the year.
///
/// `jupiter_sign`, `saturn_sign`, `sun_sign` are 0-based sign indices
/// of the transiting planets at the time of the solar return.
pub fn check_tri_pataki(
    muntha: usize,
    jupiter_sign: usize,
    saturn_sign: usize,
    sun_sign: usize,
) -> TriPatakiResult {
    let j = jupiter_sign % 12 == muntha % 12;
    let s = saturn_sign % 12 == muntha % 12;
    let su = sun_sign % 12 == muntha % 12;
    let count = j as u32 + s as u32 + su as u32;
    TriPatakiResult {
        jupiter_on_muntha: j,
        saturn_on_muntha: s,
        sun_on_muntha: su,
        count,
        is_full_tripataki: count == 3,
    }
}

/// Compute full Varshaphal timing for one year WITHOUT transit positions.
///
/// Use [`compute_varshaphal_with_transits`] if you have planet sign positions
/// for the Tri-Pataki check.
pub fn compute_varshaphal(birth_jd: f64, natal_asc_sign: usize, year: u32) -> VarshaphalResult {
    let sr_jd = solar_return_jd(birth_jd, year);
    let wd = weekday_from_jd(sr_jd);
    let m_sign = muntha_sign(natal_asc_sign, year);

    VarshaphalResult {
        year,
        solar_return_jd: sr_jd,
        weekday_index: wd,
        year_lord: WEEKDAY_RULERS[wd],
        muntha_sign: m_sign,
        muntha_lord: sign_lord(m_sign),
        tri_pataki: None,
    }
}

/// Compute full Varshaphal timing WITH Tri-Pataki transit check.
///
/// `jupiter_sign`, `saturn_sign`, `sun_sign` are the 0-based sign indices
/// of the transiting planets at the solar return moment.
pub fn compute_varshaphal_with_transits(
    birth_jd: f64,
    natal_asc_sign: usize,
    year: u32,
    jupiter_sign: usize,
    saturn_sign: usize,
    sun_sign: usize,
) -> VarshaphalResult {
    let mut result = compute_varshaphal(birth_jd, natal_asc_sign, year);
    result.tri_pataki = Some(check_tri_pataki(
        result.muntha_sign,
        jupiter_sign,
        saturn_sign,
        sun_sign,
    ));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // Known reference: J2000.0 epoch = JD 2451545.0 = 2000-Jan-1.5 = Saturday
    const J2000_JD: f64 = 2451545.0;

    // --- Test 1: Solar return JD at year 0 equals birth JD ---
    #[test]
    fn solar_return_year_zero() {
        let jd = solar_return_jd(J2000_JD, 0);
        assert!((jd - J2000_JD).abs() < 1e-10);
    }

    // --- Test 2: Solar return JD advances by ~365.24219 per year ---
    #[test]
    fn solar_return_advances_one_year() {
        let jd1 = solar_return_jd(J2000_JD, 1);
        let diff = jd1 - J2000_JD;
        assert!(
            (diff - TROPICAL_YEAR_DAYS).abs() < 1e-6,
            "Expected ~{TROPICAL_YEAR_DAYS}, got {diff}"
        );
    }

    // --- Test 3: Weekday of J2000.0 (2000-Jan-1.5 = Saturday) ---
    #[test]
    fn weekday_j2000_is_saturday() {
        let wd = weekday_from_jd(J2000_JD);
        assert_eq!(wd, 6, "J2000.0 should be Saturday (6), got {wd}");
        assert_eq!(WEEKDAY_RULERS[wd], "Saturn");
    }

    // --- Test 4: Year lord at J2000.0 is Saturn ---
    #[test]
    fn year_lord_j2000() {
        assert_eq!(year_lord(J2000_JD), "Saturn");
    }

    // --- Test 5: Muntha cycles every 12 years ---
    #[test]
    fn muntha_sign_cycles() {
        let asc = 3; // Cancer
        assert_eq!(muntha_sign(asc, 0), 3);
        assert_eq!(muntha_sign(asc, 1), 4);
        assert_eq!(muntha_sign(asc, 12), 3); // back to start
        assert_eq!(muntha_sign(asc, 13), 4);
    }

    // --- Test 6: Muntha lord is correct ---
    #[test]
    fn muntha_lord_is_sign_lord() {
        // Muntha in Aries (0) → Mars
        assert_eq!(sign_lord(0), "Mars");
        // Muntha in Leo (4) → Sun
        assert_eq!(sign_lord(4), "Sun");
        // Muntha in Pisces (11) → Jupiter
        assert_eq!(sign_lord(11), "Jupiter");
    }

    // --- Test 7: Full Tri-Pataki when all three match Muntha ---
    #[test]
    fn full_tripataki() {
        let tp = check_tri_pataki(5, 5, 5, 5);
        assert!(tp.is_full_tripataki);
        assert_eq!(tp.count, 3);
        assert!(tp.jupiter_on_muntha);
        assert!(tp.saturn_on_muntha);
        assert!(tp.sun_on_muntha);
    }

    // --- Test 8: Partial Tri-Pataki (only Jupiter matches) ---
    #[test]
    fn partial_tripataki_one() {
        let tp = check_tri_pataki(5, 5, 3, 10);
        assert!(!tp.is_full_tripataki);
        assert_eq!(tp.count, 1);
        assert!(tp.jupiter_on_muntha);
        assert!(!tp.saturn_on_muntha);
        assert!(!tp.sun_on_muntha);
    }

    // --- Test 9: No Tri-Pataki when none match ---
    #[test]
    fn no_tripataki() {
        let tp = check_tri_pataki(5, 0, 3, 10);
        assert!(!tp.is_full_tripataki);
        assert_eq!(tp.count, 0);
    }

    // --- Test 10: compute_varshaphal without transits has None tri_pataki ---
    #[test]
    fn varshaphal_without_transits() {
        let r = compute_varshaphal(J2000_JD, 0, 5);
        assert_eq!(r.year, 5);
        assert_eq!(r.muntha_sign, 5); // Aries(0) + 5 = Virgo
        assert_eq!(r.muntha_lord, "Mercury");
        assert!(r.tri_pataki.is_none());
    }

    // --- Test 11: compute_varshaphal_with_transits populates tri_pataki ---
    #[test]
    fn varshaphal_with_transits() {
        let muntha = muntha_sign(0, 5); // 5 = Virgo
        let r = compute_varshaphal_with_transits(
            J2000_JD, 0, 5, muntha, // Jupiter in Virgo → match
            3,      // Saturn in Cancer → no match
            muntha, // Sun in Virgo → match
        );
        let tp = r.tri_pataki.as_ref().unwrap();
        assert!(tp.jupiter_on_muntha);
        assert!(!tp.saturn_on_muntha);
        assert!(tp.sun_on_muntha);
        assert_eq!(tp.count, 2);
        assert!(!tp.is_full_tripataki);
    }

    // --- Test 12: Weekday rotates correctly across 7 consecutive days ---
    #[test]
    fn weekday_rotates_seven_days() {
        let base_wd = weekday_from_jd(J2000_JD);
        for offset in 0..7 {
            let wd = weekday_from_jd(J2000_JD + offset as f64);
            assert_eq!(
                wd,
                (base_wd + offset) % 7,
                "Day offset {offset}: expected {}, got {wd}",
                (base_wd + offset) % 7
            );
        }
    }
}
