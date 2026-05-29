use serde::{Deserialize, Serialize};

/// GMT correlation constant (Goodman-Martinez-Thompson).
/// Maps Julian Day 584283 to the Mayan creation date 0.0.0.0.0 (13.0.0.0.0).
pub const GMT_CORRELATION: f64 = 584_283.0;

// ---------------------------------------------------------------------------
// Tzolkin day names (20 named days in the sacred 260-day cycle)
// ---------------------------------------------------------------------------

/// The 20 Tzolkin day names (also called day signs or *k'in* names).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TzolkinDayName {
    Imix,
    Ik,
    Akbal,
    Kan,
    Chicchan,
    Cimi,
    Manik,
    Lamat,
    Muluc,
    Oc,
    Chuen,
    Eb,
    Ben,
    Ix,
    Men,
    Cib,
    Caban,
    Etznab,
    Cauac,
    Ahau,
}

impl TzolkinDayName {
    /// All 20 day names in order.
    pub const ALL: [TzolkinDayName; 20] = [
        TzolkinDayName::Imix,
        TzolkinDayName::Ik,
        TzolkinDayName::Akbal,
        TzolkinDayName::Kan,
        TzolkinDayName::Chicchan,
        TzolkinDayName::Cimi,
        TzolkinDayName::Manik,
        TzolkinDayName::Lamat,
        TzolkinDayName::Muluc,
        TzolkinDayName::Oc,
        TzolkinDayName::Chuen,
        TzolkinDayName::Eb,
        TzolkinDayName::Ben,
        TzolkinDayName::Ix,
        TzolkinDayName::Men,
        TzolkinDayName::Cib,
        TzolkinDayName::Caban,
        TzolkinDayName::Etznab,
        TzolkinDayName::Cauac,
        TzolkinDayName::Ahau,
    ];

    /// Return the day name from a 0-based index (wraps at 20).
    pub fn from_index(i: usize) -> Self {
        Self::ALL[i % 20]
    }

    /// Human-readable English name.
    pub fn name(&self) -> &'static str {
        match self {
            TzolkinDayName::Imix => "Imix",
            TzolkinDayName::Ik => "Ik",
            TzolkinDayName::Akbal => "Akbal",
            TzolkinDayName::Kan => "Kan",
            TzolkinDayName::Chicchan => "Chicchan",
            TzolkinDayName::Cimi => "Cimi",
            TzolkinDayName::Manik => "Manik",
            TzolkinDayName::Lamat => "Lamat",
            TzolkinDayName::Muluc => "Muluc",
            TzolkinDayName::Oc => "Oc",
            TzolkinDayName::Chuen => "Chuen",
            TzolkinDayName::Eb => "Eb",
            TzolkinDayName::Ben => "Ben",
            TzolkinDayName::Ix => "Ix",
            TzolkinDayName::Men => "Men",
            TzolkinDayName::Cib => "Cib",
            TzolkinDayName::Caban => "Caban",
            TzolkinDayName::Etznab => "Etznab",
            TzolkinDayName::Cauac => "Cauac",
            TzolkinDayName::Ahau => "Ahau",
        }
    }
}

// ---------------------------------------------------------------------------
// Haab month names (18 months of 20 days + 5-day Wayeb)
// ---------------------------------------------------------------------------

/// The 19 Haab month names (18 regular + Wayeb).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HaabMonthName {
    Pop,
    Uo,
    Zip,
    Zotz,
    Tzec,
    Xul,
    Yaxkin,
    Mol,
    Chen,
    Yax,
    Zac,
    Ceh,
    Mac,
    Kankin,
    Muan,
    Pax,
    Kayab,
    Cumku,
    Wayeb,
}

impl HaabMonthName {
    /// All 19 month names in order (18 regular + Wayeb).
    pub const ALL: [HaabMonthName; 19] = [
        HaabMonthName::Pop,
        HaabMonthName::Uo,
        HaabMonthName::Zip,
        HaabMonthName::Zotz,
        HaabMonthName::Tzec,
        HaabMonthName::Xul,
        HaabMonthName::Yaxkin,
        HaabMonthName::Mol,
        HaabMonthName::Chen,
        HaabMonthName::Yax,
        HaabMonthName::Zac,
        HaabMonthName::Ceh,
        HaabMonthName::Mac,
        HaabMonthName::Kankin,
        HaabMonthName::Muan,
        HaabMonthName::Pax,
        HaabMonthName::Kayab,
        HaabMonthName::Cumku,
        HaabMonthName::Wayeb,
    ];

    /// Human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            HaabMonthName::Pop => "Pop",
            HaabMonthName::Uo => "Uo",
            HaabMonthName::Zip => "Zip",
            HaabMonthName::Zotz => "Zotz",
            HaabMonthName::Tzec => "Tzec",
            HaabMonthName::Xul => "Xul",
            HaabMonthName::Yaxkin => "Yaxkin",
            HaabMonthName::Mol => "Mol",
            HaabMonthName::Chen => "Chen",
            HaabMonthName::Yax => "Yax",
            HaabMonthName::Zac => "Zac",
            HaabMonthName::Ceh => "Ceh",
            HaabMonthName::Mac => "Mac",
            HaabMonthName::Kankin => "Kankin",
            HaabMonthName::Muan => "Muan",
            HaabMonthName::Pax => "Pax",
            HaabMonthName::Kayab => "Kayab",
            HaabMonthName::Cumku => "Cumku",
            HaabMonthName::Wayeb => "Wayeb",
        }
    }
}

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

/// Mayan Long Count date: Baktun.Katun.Tun.Uinal.Kin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LongCount {
    pub baktun: u32,
    pub katun: u32,
    pub tun: u32,
    pub uinal: u32,
    pub kin: u32,
}

/// Tzolkin (sacred 260-day calendar) date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TzolkinDate {
    /// Day number 1..=13.
    pub number: u32,
    /// Day name (one of 20).
    pub day_name: TzolkinDayName,
}

/// Haab (civil 365-day calendar) date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HaabDate {
    /// Day within the month: 0..=19 for regular months, 0..=4 for Wayeb.
    pub day: u32,
    /// Month name.
    pub month: HaabMonthName,
}

/// Calendar Round: combined Tzolkin + Haab (repeats every 18,980 days = 52 Haab years).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalendarRound {
    pub tzolkin: TzolkinDate,
    pub haab: HaabDate,
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl std::fmt::Display for LongCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}.{}",
            self.baktun, self.katun, self.tun, self.uinal, self.kin
        )
    }
}

impl std::fmt::Display for TzolkinDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.number, self.day_name.name())
    }
}

impl std::fmt::Display for HaabDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.day, self.month.name())
    }
}

impl std::fmt::Display for CalendarRound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.tzolkin, self.haab)
    }
}

// ---------------------------------------------------------------------------
// Conversion functions
// ---------------------------------------------------------------------------

/// Number of days since the Mayan creation date for the given Julian Day.
///
/// Converts the JD to a Julian Day Number (integer at noon) before subtracting
/// the GMT correlation.  Civil-convention JDs end in .5 (midnight), so we round
/// to the nearest integer with `floor(jd + 0.5)` to obtain the correct calendar
/// day.  Without this, midnight JDs (e.g. 2456282.5 for Dec 21 2012) would be
/// off by one day.
///
/// Verified: Dec 21 2012 (JD 2456282.5) -> day 1872000 = 13.0.0.0.0
/// Reference: https://en.wikipedia.org/wiki/Mesoamerican_Long_Count_calendar
fn day_count(jd: f64) -> i64 {
    ((jd + 0.5).floor() as i64) - (GMT_CORRELATION as i64)
}

/// Convert a Julian Day number to a Mayan Long Count date.
///
/// Uses the GMT correlation constant (584283). The Long Count counts days
/// from the Mayan creation date 0.0.0.0.0 = 13.0.0.0.0 (August 11, 3114 BCE
/// proleptic Gregorian).
pub fn long_count_from_jd(jd: f64) -> LongCount {
    let mut d = day_count(jd);
    // Handle dates before the creation epoch
    if d < 0 {
        // Wrap into the current Grand Cycle (13 baktuns = 1,872,000 days)
        d = d.rem_euclid(13 * 144_000);
    }
    let d = d as u32;

    let baktun = d / 144_000;
    let rem = d % 144_000;
    let katun = rem / 7_200;
    let rem = rem % 7_200;
    let tun = rem / 360;
    let rem = rem % 360;
    let uinal = rem / 20;
    let kin = rem % 20;

    LongCount {
        baktun,
        katun,
        tun,
        uinal,
        kin,
    }
}

/// Convert a Long Count date back to a Julian Day number.
pub fn jd_from_long_count(lc: &LongCount) -> f64 {
    let days = lc.baktun as f64 * 144_000.0
        + lc.katun as f64 * 7_200.0
        + lc.tun as f64 * 360.0
        + lc.uinal as f64 * 20.0
        + lc.kin as f64;
    days + GMT_CORRELATION
}

/// Convert a Julian Day number to a Tzolkin date (260-day sacred calendar).
///
/// The Tzolkin combines a number (1-13) with a day name (20 names) in an
/// interlocking cycle of 260 days.
pub fn tzolkin_from_jd(jd: f64) -> TzolkinDate {
    let d = day_count(jd);
    // On the creation date (Long Count 0.0.0.0.0 = JD 584283), the Tzolkin
    // was 4 Ahau. Ahau = index 19, number = 4.
    let number = ((d + 3).rem_euclid(13) + 1) as u32; // +3 because creation = 4 (4-1=3)
    let name_idx = (d + 19).rem_euclid(20) as usize; // +19 because creation = Ahau (index 19)
    TzolkinDate {
        number,
        day_name: TzolkinDayName::from_index(name_idx),
    }
}

/// Convert a Julian Day number to a Haab date (365-day civil calendar).
///
/// The Haab has 18 months of 20 days (numbered 0-19) plus 5 "unlucky" days
/// called Wayeb (numbered 0-4).
pub fn haab_from_jd(jd: f64) -> HaabDate {
    let d = day_count(jd);
    // On the creation date, the Haab was 8 Cumku.
    // Cumku is the 18th month (0-indexed = 17). Day 8.
    // 17 * 20 + 8 = 348 days into the Haab year.
    let haab_day = (d + 348).rem_euclid(365) as u32;
    let month_idx = (haab_day / 20) as usize;
    let day = haab_day % 20;

    if month_idx >= 18 {
        // Wayeb (5 extra days)
        HaabDate {
            day: haab_day - 360,
            month: HaabMonthName::Wayeb,
        }
    } else {
        HaabDate {
            day,
            month: HaabMonthName::ALL[month_idx],
        }
    }
}

/// Compute the full Calendar Round (Tzolkin + Haab) for a Julian Day.
///
/// The Calendar Round repeats every 18,980 days (52 Haab years = 73 Tzolkin
/// cycles), which is the LCM of 260 and 365.
pub fn calendar_round(jd: f64) -> CalendarRound {
    CalendarRound {
        tzolkin: tzolkin_from_jd(jd),
        haab: haab_from_jd(jd),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Known reference: 2012-12-21 = JD 2456282.5 = 13.0.0.0.0 (end of 13th Baktun)
    const JD_2012_DEC_21: f64 = 2_456_282.5;
    // Known reference: 2024-01-01 = JD 2460310.5
    const JD_2024_JAN_01: f64 = 2_460_310.5;
    // Creation date: August 11, 3114 BCE = JD 584283
    const JD_CREATION: f64 = 584_283.0;

    #[test]
    fn long_count_creation_date() {
        let lc = long_count_from_jd(JD_CREATION);
        assert_eq!(
            lc,
            LongCount {
                baktun: 0,
                katun: 0,
                tun: 0,
                uinal: 0,
                kin: 0
            }
        );
    }

    #[test]
    fn long_count_2012_end_of_cycle() {
        // 2012-12-21 = 13.0.0.0.0 (end of the 13th Baktun)
        // JD 2456282.5 (midnight) -> day_count = floor(2456282.5+0.5) - 584283
        //   = 2456283 - 584283 = 1,872,000 = 13 * 144,000
        // Verified: https://en.wikipedia.org/wiki/Mesoamerican_Long_Count_calendar
        let lc = long_count_from_jd(JD_2012_DEC_21);
        assert_eq!(lc.baktun, 13);
        assert_eq!(lc.katun, 0);
        assert_eq!(lc.tun, 0);
        assert_eq!(lc.uinal, 0);
        assert_eq!(lc.kin, 0);
    }

    #[test]
    fn long_count_day_after_2012() {
        // 2012-12-22 = JD 2456283.5 -> 13.0.0.0.1
        let lc = long_count_from_jd(JD_2012_DEC_21 + 1.0);
        assert_eq!(lc.baktun, 13);
        assert_eq!(lc.katun, 0);
        assert_eq!(lc.tun, 0);
        assert_eq!(lc.uinal, 0);
        assert_eq!(lc.kin, 1);
    }

    #[test]
    fn long_count_roundtrip() {
        let lc = long_count_from_jd(JD_2024_JAN_01);
        let jd_back = jd_from_long_count(&lc);
        // jd_from_long_count returns a noon-epoch JD (integer).
        // Our day_count normalizes via floor(jd+0.5), so the roundtrip target
        // is the Julian Day Number = floor(jd + 0.5).
        let jd_normalized = (JD_2024_JAN_01 + 0.5).floor();
        assert!((jd_back - jd_normalized).abs() < 0.01);
    }

    #[test]
    fn long_count_display() {
        let lc = LongCount {
            baktun: 13,
            katun: 0,
            tun: 11,
            uinal: 5,
            kin: 10,
        };
        assert_eq!(format!("{lc}"), "13.0.11.5.10");
    }

    #[test]
    fn tzolkin_creation_date_is_4_ahau() {
        let tz = tzolkin_from_jd(JD_CREATION);
        assert_eq!(tz.number, 4);
        assert_eq!(tz.day_name, TzolkinDayName::Ahau);
    }

    #[test]
    fn tzolkin_260_day_cycle() {
        let tz1 = tzolkin_from_jd(JD_2024_JAN_01);
        let tz2 = tzolkin_from_jd(JD_2024_JAN_01 + 260.0);
        assert_eq!(tz1, tz2);
    }

    #[test]
    fn tzolkin_day_after_creation() {
        // Day after 4 Ahau should be 5 Imix
        let tz = tzolkin_from_jd(JD_CREATION + 1.0);
        assert_eq!(tz.number, 5);
        assert_eq!(tz.day_name, TzolkinDayName::Imix);
    }

    #[test]
    fn tzolkin_number_wraps_at_13() {
        // 13 days after creation: 4+13=17 -> wraps: number should cycle
        let tz = tzolkin_from_jd(JD_CREATION + 13.0);
        assert_eq!(tz.number, 4); // 4 + 13 mod 13 = 4
    }

    #[test]
    fn tzolkin_20_day_names() {
        assert_eq!(TzolkinDayName::ALL.len(), 20);
        assert_eq!(TzolkinDayName::ALL[0], TzolkinDayName::Imix);
        assert_eq!(TzolkinDayName::ALL[19], TzolkinDayName::Ahau);
    }

    #[test]
    fn haab_creation_date_is_8_cumku() {
        let h = haab_from_jd(JD_CREATION);
        assert_eq!(h.day, 8);
        assert_eq!(h.month, HaabMonthName::Cumku);
    }

    #[test]
    fn haab_365_day_cycle() {
        let h1 = haab_from_jd(JD_2024_JAN_01);
        let h2 = haab_from_jd(JD_2024_JAN_01 + 365.0);
        assert_eq!(h1, h2);
    }

    #[test]
    fn haab_19_month_names() {
        assert_eq!(HaabMonthName::ALL.len(), 19);
        assert_eq!(HaabMonthName::ALL[0], HaabMonthName::Pop);
        assert_eq!(HaabMonthName::ALL[17], HaabMonthName::Cumku);
        assert_eq!(HaabMonthName::ALL[18], HaabMonthName::Wayeb);
    }

    #[test]
    fn haab_wayeb_at_end_of_year() {
        // Wayeb is the last 5 days of the Haab year.
        // 348 (Cumku day 8 at creation) + X where month_idx >= 18
        // From creation, Cumku starts at day 340 of the Haab year.
        // Wayeb starts at day 360 of the Haab year.
        // So 360 - 348 = 12 days after creation should be Wayeb 0.
        let h = haab_from_jd(JD_CREATION + 12.0);
        assert_eq!(h.month, HaabMonthName::Wayeb);
        assert_eq!(h.day, 0);
    }

    #[test]
    fn calendar_round_repeats_every_18980_days() {
        let cr1 = calendar_round(JD_2024_JAN_01);
        let cr2 = calendar_round(JD_2024_JAN_01 + 18_980.0);
        assert_eq!(cr1, cr2);
    }

    #[test]
    fn calendar_round_creation_date() {
        let cr = calendar_round(JD_CREATION);
        assert_eq!(cr.tzolkin.number, 4);
        assert_eq!(cr.tzolkin.day_name, TzolkinDayName::Ahau);
        assert_eq!(cr.haab.day, 8);
        assert_eq!(cr.haab.month, HaabMonthName::Cumku);
    }

    #[test]
    fn calendar_round_display() {
        let cr = calendar_round(JD_CREATION);
        assert_eq!(format!("{cr}"), "4 Ahau 8 Cumku");
    }

    #[test]
    fn gmt_correlation_constant() {
        assert_eq!(GMT_CORRELATION, 584_283.0);
    }
}
