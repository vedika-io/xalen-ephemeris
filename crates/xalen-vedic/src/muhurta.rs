use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChoghadiyaType {
    Amrit, // Best
    Shubh, // Good
    Labh,  // Profit
    Char,  // Neutral/moving
    Rog,   // Disease (inauspicious)
    Kaal,  // Death (inauspicious)
    Udveg, // Anxiety (inauspicious)
}

impl ChoghadiyaType {
    pub fn is_auspicious(&self) -> bool {
        matches!(
            self,
            ChoghadiyaType::Amrit | ChoghadiyaType::Shubh | ChoghadiyaType::Labh
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoghadiyaPeriod {
    pub choghadiya: ChoghadiyaType,
    pub start_jd: f64,
    pub end_jd: f64,
    pub is_day: bool,
}

// Day choghadiya sequences by weekday (0=Sun, 1=Mon, ...)
const DAY_FIRST: [ChoghadiyaType; 7] = [
    ChoghadiyaType::Udveg, // Sunday
    ChoghadiyaType::Amrit, // Monday
    ChoghadiyaType::Rog,   // Tuesday
    ChoghadiyaType::Labh,  // Wednesday
    ChoghadiyaType::Shubh, // Thursday
    ChoghadiyaType::Char,  // Friday
    ChoghadiyaType::Kaal,  // Saturday
];

const CHOGHADIYA_SEQUENCE: [ChoghadiyaType; 7] = [
    ChoghadiyaType::Udveg,
    ChoghadiyaType::Char,
    ChoghadiyaType::Labh,
    ChoghadiyaType::Amrit,
    ChoghadiyaType::Kaal,
    ChoghadiyaType::Shubh,
    ChoghadiyaType::Rog,
];

pub fn compute_choghadiya(
    sunrise_jd: f64,
    sunset_jd: f64,
    next_sunrise_jd: f64,
    weekday: usize, // 0=Sun ... 6=Sat
) -> Vec<ChoghadiyaPeriod> {
    let mut periods = Vec::new();

    let day_duration = sunset_jd - sunrise_jd;
    let night_duration = next_sunrise_jd - sunset_jd;
    let day_period = day_duration / 8.0;
    let night_period = night_duration / 8.0;

    let start_idx = CHOGHADIYA_SEQUENCE
        .iter()
        .position(|&c| c == DAY_FIRST[weekday % 7])
        .unwrap_or(0);

    // 8 day periods
    for i in 0..8 {
        let idx = (start_idx + i) % 7;
        periods.push(ChoghadiyaPeriod {
            choghadiya: CHOGHADIYA_SEQUENCE[idx],
            start_jd: sunrise_jd + i as f64 * day_period,
            end_jd: sunrise_jd + (i + 1) as f64 * day_period,
            is_day: true,
        });
    }

    // Night first choghadiya is different from day sequence
    let night_start_idx = (start_idx + 4) % 7;
    for i in 0..8 {
        let idx = (night_start_idx + i) % 7;
        periods.push(ChoghadiyaPeriod {
            choghadiya: CHOGHADIYA_SEQUENCE[idx],
            start_jd: sunset_jd + i as f64 * night_period,
            end_jd: sunset_jd + (i + 1) as f64 * night_period,
            is_day: false,
        });
    }

    periods
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RahuKalam {
    pub start_jd: f64,
    pub end_jd: f64,
}

// Rahu Kalam position in 1/8th of day: Sun=8, Mon=2, Tue=7, Wed=5, Thu=6, Fri=4, Sat=3
const RAHU_KALAM_SLOT: [usize; 7] = [8, 2, 7, 5, 6, 4, 3];
const YAMAGANDAM_SLOT: [usize; 7] = [5, 4, 3, 2, 1, 7, 6];
const GULIKA_SLOT: [usize; 7] = [7, 6, 5, 4, 3, 2, 1];

pub fn rahu_kalam(sunrise_jd: f64, sunset_jd: f64, weekday: usize) -> RahuKalam {
    let period = (sunset_jd - sunrise_jd) / 8.0;
    let slot = RAHU_KALAM_SLOT[weekday % 7] - 1;
    RahuKalam {
        start_jd: sunrise_jd + slot as f64 * period,
        end_jd: sunrise_jd + (slot + 1) as f64 * period,
    }
}

pub fn yamagandam(sunrise_jd: f64, sunset_jd: f64, weekday: usize) -> RahuKalam {
    let period = (sunset_jd - sunrise_jd) / 8.0;
    let slot = YAMAGANDAM_SLOT[weekday % 7] - 1;
    RahuKalam {
        start_jd: sunrise_jd + slot as f64 * period,
        end_jd: sunrise_jd + (slot + 1) as f64 * period,
    }
}

pub fn gulika_kalam(sunrise_jd: f64, sunset_jd: f64, weekday: usize) -> RahuKalam {
    let period = (sunset_jd - sunrise_jd) / 8.0;
    let slot = GULIKA_SLOT[weekday % 7] - 1;
    RahuKalam {
        start_jd: sunrise_jd + slot as f64 * period,
        end_jd: sunrise_jd + (slot + 1) as f64 * period,
    }
}

pub fn abhijit_muhurta(sunrise_jd: f64, sunset_jd: f64) -> (f64, f64) {
    // 8th muhurta of the day (15 muhurtas per day, each ~48 min)
    let muhurta_duration = (sunset_jd - sunrise_jd) / 15.0;
    let start = sunrise_jd + 7.0 * muhurta_duration;
    let end = start + muhurta_duration;
    (start, end)
}

pub fn brahma_muhurta(sunrise_jd: f64) -> (f64, f64) {
    let muhurta_duration = 48.0 / (24.0 * 60.0); // ~48 minutes in JD
    let start = sunrise_jd - 2.0 * muhurta_duration;
    let end = sunrise_jd - muhurta_duration;
    (start, end)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DishaShool {
    East,
    West,
    North,
    South,
}

pub fn disha_shool(weekday: usize) -> DishaShool {
    match weekday % 7 {
        0 => DishaShool::West,  // Sunday
        1 => DishaShool::East,  // Monday
        2 => DishaShool::North, // Tuesday
        3 => DishaShool::North, // Wednesday
        4 => DishaShool::South, // Thursday
        5 => DishaShool::West,  // Friday
        _ => DishaShool::East,  // Saturday
    }
}

// ---------------------------------------------------------------------------
// Panchaka (5-fold inauspiciousness check)
// ---------------------------------------------------------------------------

/// Five types of Panchaka -- inauspicious combinations of weekday + nakshatra.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PanchakaType {
    /// Death-related inauspiciousness.
    MrityuPanchaka,
    /// Fire-related danger.
    AgniPanchaka,
    /// Trouble from rulers / government.
    RajaPanchaka,
    /// Theft, robbery.
    ChoraPanchaka,
    /// Disease.
    RogaPanchaka,
}

/// Check whether a weekday + nakshatra combination produces Panchaka.
///
/// Formula: `(weekday_num + nakshatra_num) % 9`.
/// Values 1-5 each map to a type; 0 and 6-8 mean no Panchaka.
///
/// - `weekday`: 0=Sunday ... 6=Saturday
/// - `nakshatra`: 0=Ashwini ... 26=Revati
pub fn panchaka(weekday: usize, nakshatra: usize) -> Option<PanchakaType> {
    let val = (weekday + nakshatra) % 9;
    match val {
        1 => Some(PanchakaType::MrityuPanchaka),
        2 => Some(PanchakaType::AgniPanchaka),
        4 => Some(PanchakaType::RajaPanchaka),
        6 => Some(PanchakaType::ChoraPanchaka),
        8 => Some(PanchakaType::RogaPanchaka),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Hora (Planetary hours -- Vedic style)
// ---------------------------------------------------------------------------

/// Information about the current planetary hora.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoraInfo {
    /// Ruling planet name for the current hora.
    pub ruler: &'static str,
    /// 1-based hora index within the day (1-12) or night (13-24).
    pub hora_number: usize,
    /// Whether this hora falls during the daytime half.
    pub is_day: bool,
    /// JD of hora start.
    pub start_jd: f64,
    /// JD of hora end.
    pub end_jd: f64,
}

/// Chaldean order of planets for hora sequence.
const CHALDEAN_ORDER: [&str; 7] = [
    "Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon",
];

/// Index into `CHALDEAN_ORDER` for the first hora of each weekday.
/// Sunday=Sun, Monday=Moon, Tuesday=Mars, Wednesday=Mercury,
/// Thursday=Jupiter, Friday=Venus, Saturday=Saturn.
const FIRST_HORA_CHALDEAN_IDX: [usize; 7] = [
    3, // Sunday -> Sun
    6, // Monday -> Moon
    2, // Tuesday -> Mars
    5, // Wednesday -> Mercury
    1, // Thursday -> Jupiter
    4, // Friday -> Venus
    0, // Saturday -> Saturn
];

/// Compute which planetary hora is active at `query_jd`.
///
/// Day has 12 horas from sunrise to sunset, night has 12 from sunset to
/// next sunrise. The first hora of each day is ruled by the day-lord, and
/// subsequent horas follow the Chaldean descent order.
///
/// - `weekday`: 0=Sunday ... 6=Saturday
pub fn planetary_hora(
    sunrise_jd: f64,
    sunset_jd: f64,
    next_sunrise_jd: f64,
    weekday: usize,
    query_jd: f64,
) -> HoraInfo {
    let day_len = (sunset_jd - sunrise_jd).max(0.001); // guard polar edge case
    let night_len = (next_sunrise_jd - sunset_jd).max(0.001);

    let first_idx = FIRST_HORA_CHALDEAN_IDX[weekday % 7];

    if query_jd < sunset_jd {
        // Daytime
        let hora_dur = day_len / 12.0;
        let hora_num = ((query_jd - sunrise_jd) / hora_dur) as usize;
        let hora_num = hora_num.min(11);
        let ruler_idx = (first_idx + hora_num) % 7;
        HoraInfo {
            ruler: CHALDEAN_ORDER[ruler_idx],
            hora_number: hora_num + 1,
            is_day: true,
            start_jd: sunrise_jd + hora_num as f64 * hora_dur,
            end_jd: sunrise_jd + (hora_num + 1) as f64 * hora_dur,
        }
    } else {
        // Nighttime
        let hora_dur = night_len / 12.0;
        let hora_num = ((query_jd - sunset_jd) / hora_dur) as usize;
        let hora_num = hora_num.min(11);
        // Night horas continue from where day left off (12 horas in)
        let ruler_idx = (first_idx + 12 + hora_num) % 7;
        HoraInfo {
            ruler: CHALDEAN_ORDER[ruler_idx],
            hora_number: hora_num + 13,
            is_day: false,
            start_jd: sunset_jd + hora_num as f64 * hora_dur,
            end_jd: sunset_jd + (hora_num + 1) as f64 * hora_dur,
        }
    }
}

// ---------------------------------------------------------------------------
// Simple planetary hour lookup by weekday + hour-of-day
// ---------------------------------------------------------------------------

/// Return the ruling planet for the current planetary hour.
///
/// This is a simplified Chaldean planetary-hour function that divides the
/// 24-hour day into 24 equal hours (no sunrise/sunset awareness).  For
/// unequal (temporal) hours tied to sunrise/sunset, use [`planetary_hora`].
///
/// * `weekday` — 0=Sunday, 1=Monday, ..., 6=Saturday.
/// * `hour_of_day` — 0.0 to 24.0 (fractional hours from midnight).
///
/// Hour 0 starts at midnight.  The first hour of the day (sunrise onwards)
/// is traditionally hour 1 in many systems, but since callers may not have
/// sunrise data, this function counts from midnight and the first hour
/// (0.0..1.0) is still ruled by the day-lord's Chaldean predecessor.
///
/// For the traditional sunrise-based planetary hora, prefer [`planetary_hora`].
pub fn planetary_hour(weekday: usize, hour_of_day: f64) -> &'static str {
    // Chaldean descending order (same constant as planetary_hora above).
    const CHALDEAN: [&str; 7] = [
        "Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon",
    ];
    // Chaldean index of each day-lord.
    // Sunday=Sun(3), Monday=Moon(6), Tuesday=Mars(2), Wednesday=Mercury(5),
    // Thursday=Jupiter(1), Friday=Venus(4), Saturday=Saturn(0).
    const DAY_LORD_IDX: [usize; 7] = [3, 6, 2, 5, 1, 4, 0];

    let day_lord_idx = DAY_LORD_IDX[weekday % 7];
    let hour_num = (hour_of_day.max(0.0) as usize).min(23);
    let idx = (day_lord_idx + hour_num) % 7;
    CHALDEAN[idx]
}

// ---------------------------------------------------------------------------
// Nakshatra Muhurta (quality classification + activity suitability)
// ---------------------------------------------------------------------------

/// Quality classification of a nakshatra for muhurta purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NakshatraQuality {
    /// Good for permanent, lasting things (foundation, marriage).
    Fixed,
    /// Good for travel, moving, starting journeys.
    Movable,
    /// Good for destruction, surgery, demolition.
    Sharp,
    /// Good for friendships, romance, gentle activities.
    Tender,
    /// Mixed: can be used for both gentle and harsh purposes.
    Mixed,
}

/// Activity categories for nakshatra suitability checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Activity {
    Marriage,
    Travel,
    Surgery,
    Business,
    Construction,
    Learning,
    Friendship,
    MovingHouse,
}

/// Classify a nakshatra's quality for muhurta purposes.
///
/// `nakshatra`: 0=Ashwini ... 26=Revati.
pub fn nakshatra_quality(nakshatra: usize) -> NakshatraQuality {
    match nakshatra % 27 {
        // Fixed: Rohini(3), Uttara Phalguni(11), Uttara Ashadha(20), Uttara Bhadrapada(25)
        3 | 11 | 20 | 25 => NakshatraQuality::Fixed,
        // Sharp: Ardra(5), Ashlesha(8), Jyeshtha(17), Mula(18)
        5 | 8 | 17 | 18 => NakshatraQuality::Sharp,
        // Tender: Mrigashira(4), Chitra(13), Anuradha(16), Revati(26)
        4 | 13 | 16 | 26 => NakshatraQuality::Tender,
        // Movable: Ashwini(0), Pushya(7), Hasta(12), Swati(14),
        //          Shravana(21), Dhanishta(22), Shatabhisha(23), Punarvasu(6)
        0 | 6 | 7 | 12 | 14 | 21 | 22 | 23 => NakshatraQuality::Movable,
        // Everything else: Mixed
        _ => NakshatraQuality::Mixed,
    }
}

/// Check whether a nakshatra is suitable for a given activity.
///
/// `nakshatra`: 0=Ashwini ... 26=Revati.
pub fn activity_suitable(nakshatra: usize, activity: Activity) -> bool {
    let quality = nakshatra_quality(nakshatra);
    match activity {
        Activity::Marriage => matches!(quality, NakshatraQuality::Fixed | NakshatraQuality::Tender),
        Activity::Travel => matches!(quality, NakshatraQuality::Movable),
        Activity::Surgery => matches!(quality, NakshatraQuality::Sharp),
        Activity::Business => {
            matches!(quality, NakshatraQuality::Movable | NakshatraQuality::Fixed)
        }
        Activity::Construction => matches!(quality, NakshatraQuality::Fixed),
        Activity::Learning => matches!(
            quality,
            NakshatraQuality::Fixed | NakshatraQuality::Movable | NakshatraQuality::Tender
        ),
        Activity::Friendship => matches!(
            quality,
            NakshatraQuality::Tender | NakshatraQuality::Movable
        ),
        Activity::MovingHouse => matches!(quality, NakshatraQuality::Movable),
    }
}

// ---------------------------------------------------------------------------
// Tarabala (star strength from birth nakshatra)
// ---------------------------------------------------------------------------

/// The nine Tarabala categories (repeating cycle of 9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TarabalaCategory {
    /// 1st star -- birth star itself (unfavorable).
    Janma,
    /// 2nd -- wealth (favorable).
    Sampat,
    /// 3rd -- danger (unfavorable).
    Vipat,
    /// 4th -- prosperity (favorable).
    Kshema,
    /// 5th -- obstacles (unfavorable).
    Pratyari,
    /// 6th -- accomplishment (favorable).
    Sadhaka,
    /// 7th -- death/harm (unfavorable).
    Vadha,
    /// 8th -- friendship (favorable).
    Mitra,
    /// 9th -- great friendship (favorable).
    Atimitra,
}

/// Result of a Tarabala computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TarabalaResult {
    /// The category for this transit nakshatra.
    pub category: TarabalaCategory,
    /// Whether this is favorable (true) or unfavorable (false).
    pub is_favorable: bool,
    /// Count from birth nakshatra (1-27).
    pub count: usize,
}

/// Compute Tarabala (star strength) from birth nakshatra to transit nakshatra.
///
/// Count from birth to transit (inclusive), then take modulo 9 to find the
/// repeating category. Janma(1), Vipat(3), Pratyari(5), Vadha(7) are
/// unfavorable; the rest are favorable.
///
/// - `birth_nakshatra`: 0-26
/// - `transit_nakshatra`: 0-26
pub fn tarabala(birth_nakshatra: usize, transit_nakshatra: usize) -> TarabalaResult {
    let b = birth_nakshatra % 27;
    let t = transit_nakshatra % 27;
    let count = (t + 27 - b) % 27 + 1; // 1-27
    let position = ((count - 1) % 9) + 1; // 1-9

    let category = match position {
        1 => TarabalaCategory::Janma,
        2 => TarabalaCategory::Sampat,
        3 => TarabalaCategory::Vipat,
        4 => TarabalaCategory::Kshema,
        5 => TarabalaCategory::Pratyari,
        6 => TarabalaCategory::Sadhaka,
        7 => TarabalaCategory::Vadha,
        8 => TarabalaCategory::Mitra,
        _ => TarabalaCategory::Atimitra,
    };

    let is_favorable = !matches!(
        category,
        TarabalaCategory::Janma
            | TarabalaCategory::Vipat
            | TarabalaCategory::Pratyari
            | TarabalaCategory::Vadha
    );

    TarabalaResult {
        category,
        is_favorable,
        count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_sunrise_sunset() -> (f64, f64, f64) {
        // Approximate Pune sunrise/sunset for a typical day
        let sunrise = 2451545.0 + 0.25; // 6:00 AM UT
        let sunset = sunrise + 0.5; // 6:00 PM UT (12h day)
        let next_sunrise = sunrise + 1.0; // next day
        (sunrise, sunset, next_sunrise)
    }

    #[test]
    fn choghadiya_16_periods() {
        let (sr, ss, nsr) = test_sunrise_sunset();
        let periods = compute_choghadiya(sr, ss, nsr, 0); // Sunday
        assert_eq!(
            periods.len(),
            16,
            "Should have 8 day + 8 night = 16 periods"
        );
    }

    #[test]
    fn choghadiya_covers_full_day() {
        let (sr, ss, nsr) = test_sunrise_sunset();
        let periods = compute_choghadiya(sr, ss, nsr, 0);
        let first_start = periods[0].start_jd;
        let last_end = periods[15].end_jd;
        assert!((first_start - sr).abs() < 1e-10);
        assert!((last_end - nsr).abs() < 1e-10);
    }

    #[test]
    fn rahu_kalam_sunday() {
        let (sr, ss, _) = test_sunrise_sunset();
        let rk = rahu_kalam(sr, ss, 0); // Sunday: slot 8 (last 1/8th)
        let period = (ss - sr) / 8.0;
        assert!((rk.start_jd - (sr + 7.0 * period)).abs() < 1e-10);
    }

    #[test]
    fn rahu_kalam_within_day() {
        let (sr, ss, _) = test_sunrise_sunset();
        for day in 0..7 {
            let rk = rahu_kalam(sr, ss, day);
            assert!(
                rk.start_jd >= sr && rk.end_jd <= ss,
                "Rahu Kalam day {day} should be within day bounds"
            );
        }
    }

    #[test]
    fn abhijit_near_noon() {
        let (sr, ss, _) = test_sunrise_sunset();
        let (start, _end) = abhijit_muhurta(sr, ss);
        let noon = (sr + ss) / 2.0;
        assert!((start - noon).abs() < 0.05, "Abhijit should be near noon");
    }

    #[test]
    fn brahma_muhurta_before_sunrise() {
        let (sr, _, _) = test_sunrise_sunset();
        let (start, end) = brahma_muhurta(sr);
        assert!(end < sr, "Brahma muhurta should end before sunrise");
        assert!(start < end, "Start should be before end");
    }

    #[test]
    fn disha_shool_values() {
        assert_eq!(disha_shool(0), DishaShool::West); // Sunday
        assert_eq!(disha_shool(1), DishaShool::East); // Monday
        assert_eq!(disha_shool(4), DishaShool::South); // Thursday
    }

    #[test]
    fn auspicious_choghadiya() {
        assert!(ChoghadiyaType::Amrit.is_auspicious());
        assert!(ChoghadiyaType::Shubh.is_auspicious());
        assert!(!ChoghadiyaType::Kaal.is_auspicious());
        assert!(!ChoghadiyaType::Rog.is_auspicious());
    }

    #[test]
    fn choghadiya_weekday_7_no_panic() {
        let (sr, ss, nsr) = test_sunrise_sunset();
        // weekday=7 should wrap to Sunday (0) via % 7, not panic
        let periods = compute_choghadiya(sr, ss, nsr, 7);
        assert_eq!(periods.len(), 16);
        // Should produce same result as weekday=0
        let sunday = compute_choghadiya(sr, ss, nsr, 0);
        assert_eq!(periods[0].choghadiya, sunday[0].choghadiya);
    }

    // -----------------------------------------------------------------------
    // Panchaka tests
    // -----------------------------------------------------------------------

    #[test]
    fn panchaka_mrityu() {
        // (weekday + nakshatra) % 9 == 1
        // weekday=0, nakshatra=1 -> (0+1) % 9 = 1
        assert_eq!(panchaka(0, 1), Some(PanchakaType::MrityuPanchaka));
    }

    #[test]
    fn panchaka_agni() {
        // (weekday + nakshatra) % 9 == 2
        assert_eq!(panchaka(0, 2), Some(PanchakaType::AgniPanchaka));
    }

    #[test]
    fn panchaka_raja() {
        // (weekday + nakshatra) % 9 == 4
        assert_eq!(panchaka(0, 4), Some(PanchakaType::RajaPanchaka));
    }

    #[test]
    fn panchaka_chora() {
        // (weekday + nakshatra) % 9 == 6
        assert_eq!(panchaka(0, 6), Some(PanchakaType::ChoraPanchaka));
    }

    #[test]
    fn panchaka_roga() {
        // (weekday + nakshatra) % 9 == 8
        assert_eq!(panchaka(0, 8), Some(PanchakaType::RogaPanchaka));
    }

    #[test]
    fn panchaka_none() {
        // (0 + 0) % 9 = 0 -> no panchaka
        assert_eq!(panchaka(0, 0), None);
        // (0 + 3) % 9 = 3 -> no panchaka
        assert_eq!(panchaka(0, 3), None);
    }

    // -----------------------------------------------------------------------
    // Hora tests
    // -----------------------------------------------------------------------

    #[test]
    fn hora_sunday_first_is_sun() {
        let (sr, ss, nsr) = test_sunrise_sunset();
        let hora = planetary_hora(sr, ss, nsr, 0, sr + 0.001);
        assert_eq!(hora.ruler, "Sun");
        assert_eq!(hora.hora_number, 1);
        assert!(hora.is_day);
    }

    #[test]
    fn hora_monday_first_is_moon() {
        let (sr, ss, nsr) = test_sunrise_sunset();
        let hora = planetary_hora(sr, ss, nsr, 1, sr + 0.001);
        assert_eq!(hora.ruler, "Moon");
    }

    #[test]
    fn hora_saturday_first_is_saturn() {
        let (sr, ss, nsr) = test_sunrise_sunset();
        let hora = planetary_hora(sr, ss, nsr, 6, sr + 0.001);
        assert_eq!(hora.ruler, "Saturn");
    }

    #[test]
    fn hora_night_not_day() {
        let (sr, ss, nsr) = test_sunrise_sunset();
        let hora = planetary_hora(sr, ss, nsr, 0, ss + 0.001);
        assert!(!hora.is_day);
        assert!(hora.hora_number >= 13);
    }

    #[test]
    fn hora_24_total() {
        let (sr, ss, nsr) = test_sunrise_sunset();
        let day_dur = (ss - sr) / 12.0;
        let night_dur = (nsr - ss) / 12.0;
        // Check all 12 day horas + 12 night horas exist
        for i in 0..12 {
            let t = sr + i as f64 * day_dur + 0.001;
            let h = planetary_hora(sr, ss, nsr, 0, t);
            assert_eq!(h.hora_number, i + 1);
        }
        for i in 0..12 {
            let t = ss + i as f64 * night_dur + 0.001;
            let h = planetary_hora(sr, ss, nsr, 0, t);
            assert_eq!(h.hora_number, i + 13);
        }
    }

    // -----------------------------------------------------------------------
    // Nakshatra quality tests
    // -----------------------------------------------------------------------

    #[test]
    fn nakshatra_fixed_quality() {
        assert_eq!(nakshatra_quality(3), NakshatraQuality::Fixed); // Rohini
        assert_eq!(nakshatra_quality(11), NakshatraQuality::Fixed); // Uttara Phalguni
        assert_eq!(nakshatra_quality(20), NakshatraQuality::Fixed); // Uttara Ashadha
        assert_eq!(nakshatra_quality(25), NakshatraQuality::Fixed); // Uttara Bhadrapada
    }

    #[test]
    fn nakshatra_sharp_quality() {
        assert_eq!(nakshatra_quality(5), NakshatraQuality::Sharp); // Ardra
        assert_eq!(nakshatra_quality(8), NakshatraQuality::Sharp); // Ashlesha
        assert_eq!(nakshatra_quality(17), NakshatraQuality::Sharp); // Jyeshtha
        assert_eq!(nakshatra_quality(18), NakshatraQuality::Sharp); // Mula
    }

    #[test]
    fn nakshatra_tender_quality() {
        assert_eq!(nakshatra_quality(4), NakshatraQuality::Tender); // Mrigashira
        assert_eq!(nakshatra_quality(13), NakshatraQuality::Tender); // Chitra
        assert_eq!(nakshatra_quality(26), NakshatraQuality::Tender); // Revati
    }

    #[test]
    fn nakshatra_movable_quality() {
        assert_eq!(nakshatra_quality(0), NakshatraQuality::Movable); // Ashwini
        assert_eq!(nakshatra_quality(7), NakshatraQuality::Movable); // Pushya
        assert_eq!(nakshatra_quality(21), NakshatraQuality::Movable); // Shravana
    }

    #[test]
    fn activity_marriage_needs_fixed_or_tender() {
        assert!(activity_suitable(3, Activity::Marriage)); // Rohini (Fixed)
        assert!(activity_suitable(4, Activity::Marriage)); // Mrigashira (Tender)
        assert!(!activity_suitable(5, Activity::Marriage)); // Ardra (Sharp)
        assert!(!activity_suitable(0, Activity::Marriage)); // Ashwini (Movable)
    }

    #[test]
    fn activity_surgery_needs_sharp() {
        assert!(activity_suitable(5, Activity::Surgery)); // Ardra
        assert!(!activity_suitable(3, Activity::Surgery)); // Rohini
    }

    #[test]
    fn activity_travel_needs_movable() {
        assert!(activity_suitable(0, Activity::Travel)); // Ashwini (Movable)
        assert!(!activity_suitable(3, Activity::Travel)); // Rohini (Fixed)
    }

    // -----------------------------------------------------------------------
    // Tarabala tests
    // -----------------------------------------------------------------------

    #[test]
    fn tarabala_same_nakshatra_is_janma() {
        let result = tarabala(0, 0);
        assert_eq!(result.category, TarabalaCategory::Janma);
        assert!(!result.is_favorable);
        assert_eq!(result.count, 1);
    }

    #[test]
    fn tarabala_second_is_sampat() {
        let result = tarabala(0, 1);
        assert_eq!(result.category, TarabalaCategory::Sampat);
        assert!(result.is_favorable);
        assert_eq!(result.count, 2);
    }

    #[test]
    fn tarabala_third_is_vipat() {
        let result = tarabala(0, 2);
        assert_eq!(result.category, TarabalaCategory::Vipat);
        assert!(!result.is_favorable);
    }

    #[test]
    fn tarabala_cycle_repeats() {
        // Position 10 -> same as position 1 (Janma)
        let result = tarabala(0, 9);
        assert_eq!(result.category, TarabalaCategory::Janma);
        assert_eq!(result.count, 10);
    }

    #[test]
    fn tarabala_wrap_around() {
        // birth=26 (Revati), transit=0 (Ashwini) -> count=2 -> Sampat
        let result = tarabala(26, 0);
        assert_eq!(result.count, 2);
        assert_eq!(result.category, TarabalaCategory::Sampat);
    }

    #[test]
    fn tarabala_unfavorable_set() {
        // 1=Janma, 3=Vipat, 5=Pratyari, 7=Vadha
        let r1 = tarabala(0, 0);
        assert!(!r1.is_favorable);
        let r3 = tarabala(0, 2);
        assert!(!r3.is_favorable);
        let r5 = tarabala(0, 4);
        assert!(!r5.is_favorable);
        let r7 = tarabala(0, 6);
        assert!(!r7.is_favorable);
    }

    #[test]
    fn tarabala_favorable_set() {
        // 2=Sampat, 4=Kshema, 6=Sadhaka, 8=Mitra, 9=Atimitra
        let r2 = tarabala(0, 1);
        assert!(r2.is_favorable);
        let r4 = tarabala(0, 3);
        assert!(r4.is_favorable);
        let r6 = tarabala(0, 5);
        assert!(r6.is_favorable);
        let r8 = tarabala(0, 7);
        assert!(r8.is_favorable);
        let r9 = tarabala(0, 8);
        assert!(r9.is_favorable);
    }

    // -----------------------------------------------------------------------
    // planetary_hour tests (simplified Chaldean, equal hours from midnight)
    // -----------------------------------------------------------------------

    #[test]
    fn planetary_hour_sunday_hour0_is_sun() {
        // Hour 0 (midnight) on Sunday: day-lord is Sun (Chaldean idx 3),
        // offset by 0 → Sun.
        assert_eq!(planetary_hour(0, 0.0), "Sun");
    }

    #[test]
    fn planetary_hour_monday_hour0_is_moon() {
        assert_eq!(planetary_hour(1, 0.0), "Moon");
    }

    #[test]
    fn planetary_hour_saturday_hour0_is_saturn() {
        assert_eq!(planetary_hour(6, 0.0), "Saturn");
    }

    #[test]
    fn planetary_hour_always_valid_planet() {
        let valid = ["Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon"];
        for weekday in 0..7 {
            for hour in 0..24 {
                let p = planetary_hour(weekday, hour as f64);
                assert!(
                    valid.contains(&p),
                    "Invalid planet '{}' for weekday={}, hour={}",
                    p,
                    weekday,
                    hour
                );
            }
        }
    }

    #[test]
    fn planetary_hour_clamps_out_of_range() {
        // Negative hour clamps to 0
        let p = planetary_hour(0, -1.0);
        assert_eq!(p, planetary_hour(0, 0.0));
        // Hour >= 24 clamps to 23
        let p2 = planetary_hour(0, 25.5);
        assert_eq!(p2, planetary_hour(0, 23.0));
    }
}
