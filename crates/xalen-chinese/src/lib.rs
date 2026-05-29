/// Feng Shui compass analysis and Flying Stars.
pub mod fengshui;
/// Qi Men Dun Jia (Mystical Door Hiding Technique).
pub mod qimen;
/// Zi Wei Dou Shu (Purple Star Astrology) chart computation.
pub mod ziwei;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HeavenlyStem {
    Jia = 0,
    Yi,
    Bing,
    Ding,
    Wu,
    Ji,
    Geng,
    Xin,
    Ren,
    Gui,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EarthlyBranch {
    Zi = 0,
    Chou,
    Yin,
    Mao,
    Chen,
    Si,
    Wu,
    Wei,
    Shen,
    You,
    Xu,
    Hai,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WuXing {
    Wood,
    Fire,
    Earth,
    Metal,
    Water,
}

impl HeavenlyStem {
    pub const ALL: [HeavenlyStem; 10] = [
        HeavenlyStem::Jia,
        HeavenlyStem::Yi,
        HeavenlyStem::Bing,
        HeavenlyStem::Ding,
        HeavenlyStem::Wu,
        HeavenlyStem::Ji,
        HeavenlyStem::Geng,
        HeavenlyStem::Xin,
        HeavenlyStem::Ren,
        HeavenlyStem::Gui,
    ];

    pub fn from_index(i: usize) -> Self {
        Self::ALL[i % 10]
    }

    pub fn element(&self) -> WuXing {
        match self {
            HeavenlyStem::Jia | HeavenlyStem::Yi => WuXing::Wood,
            HeavenlyStem::Bing | HeavenlyStem::Ding => WuXing::Fire,
            HeavenlyStem::Wu | HeavenlyStem::Ji => WuXing::Earth,
            HeavenlyStem::Geng | HeavenlyStem::Xin => WuXing::Metal,
            HeavenlyStem::Ren | HeavenlyStem::Gui => WuXing::Water,
        }
    }

    pub fn is_yang(&self) -> bool {
        (*self as usize).is_multiple_of(2)
    }
}

impl EarthlyBranch {
    pub const ALL: [EarthlyBranch; 12] = [
        EarthlyBranch::Zi,
        EarthlyBranch::Chou,
        EarthlyBranch::Yin,
        EarthlyBranch::Mao,
        EarthlyBranch::Chen,
        EarthlyBranch::Si,
        EarthlyBranch::Wu,
        EarthlyBranch::Wei,
        EarthlyBranch::Shen,
        EarthlyBranch::You,
        EarthlyBranch::Xu,
        EarthlyBranch::Hai,
    ];

    pub fn from_index(i: usize) -> Self {
        Self::ALL[i % 12]
    }

    pub fn animal(&self) -> &'static str {
        match self {
            EarthlyBranch::Zi => "Rat",
            EarthlyBranch::Chou => "Ox",
            EarthlyBranch::Yin => "Tiger",
            EarthlyBranch::Mao => "Rabbit",
            EarthlyBranch::Chen => "Dragon",
            EarthlyBranch::Si => "Snake",
            EarthlyBranch::Wu => "Horse",
            EarthlyBranch::Wei => "Goat",
            EarthlyBranch::Shen => "Monkey",
            EarthlyBranch::You => "Rooster",
            EarthlyBranch::Xu => "Dog",
            EarthlyBranch::Hai => "Pig",
        }
    }

    pub fn element(&self) -> WuXing {
        match self {
            EarthlyBranch::Yin | EarthlyBranch::Mao => WuXing::Wood,
            EarthlyBranch::Si | EarthlyBranch::Wu => WuXing::Fire,
            EarthlyBranch::Chen | EarthlyBranch::Xu | EarthlyBranch::Chou | EarthlyBranch::Wei => {
                WuXing::Earth
            }
            EarthlyBranch::Shen | EarthlyBranch::You => WuXing::Metal,
            EarthlyBranch::Hai | EarthlyBranch::Zi => WuXing::Water,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Pillar {
    pub stem: HeavenlyStem,
    pub branch: EarthlyBranch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaZiChart {
    pub year: Pillar,
    pub month: Pillar,
    pub day: Pillar,
    pub hour: Pillar,
    pub day_master: HeavenlyStem,
}

pub fn sexagenary_year(year: i32) -> Pillar {
    // Sexagenary cycle: year 4 CE = Jia-Zi (cycle start)
    // Verified: 2024 = Jia-Chen (Wood Dragon), 2000 = Geng-Chen (Metal Dragon)
    // Reference: https://en.wikipedia.org/wiki/Sexagenary_cycle
    let cycle = ((year - 4).rem_euclid(60)) as usize;
    Pillar {
        stem: HeavenlyStem::from_index(cycle),
        branch: EarthlyBranch::from_index(cycle),
    }
}

pub fn sexagenary_day(jd: f64) -> Pillar {
    // Day pillar from JD using the formula from ytliu0.github.io/ChineseCalendar:
    // T = 1 + (JD_noon - 1) % 10, B = 1 + (JD_noon + 1) % 12
    // Verified: 2000-01-01 (JD 2451545) = Wu-Wu (戊午), 1970-01-01 (JD 2440588) = Xin-Si (辛巳)
    // For .5-offset JDs: anchor at JD 2451544.5 = midnight Jan 1 2000 = Wu-Wu
    let day_count = ((jd - 2451544.5).floor() as i64).rem_euclid(60) as usize;
    let stem_offset = 4; // Wu = index 4 (2000-01-01 = Wu-Wu per web-verified formula)
    let branch_offset = 6; // Wu = index 6
    Pillar {
        stem: HeavenlyStem::from_index((day_count + stem_offset) % 10),
        branch: EarthlyBranch::from_index((day_count + branch_offset) % 12),
    }
}

pub fn hour_branch(hour: f64) -> EarthlyBranch {
    // Chinese hours: 2-hour blocks starting at 23:00 (Zi hour)
    let h = ((hour + 1.0).rem_euclid(24.0) / 2.0) as usize;
    EarthlyBranch::from_index(h)
}

pub fn hour_stem(day_stem: HeavenlyStem, hour_branch: EarthlyBranch) -> HeavenlyStem {
    // Hour stem derived from day stem
    let day_idx = day_stem as usize;
    let base = (day_idx % 5) * 2;
    HeavenlyStem::from_index(base + hour_branch as usize)
}

pub fn solar_term_sun_longitude(term_index: usize) -> f64 {
    // 24 solar terms: each at 15° intervals of Sun's ecliptic longitude
    // Li Chun (Start of Spring) = Sun at 315°
    (315.0 + term_index as f64 * 15.0).rem_euclid(360.0)
}

/// Approximate Sun ecliptic longitude for a given Julian Day.
/// Uses Meeus Ch.25 low-accuracy formula (good to ~0.01 degree).
/// Returns degrees in 0..360.
pub fn solar_longitude_approx(jd: f64) -> f64 {
    let t = (jd - 2451545.0) / 36525.0; // Julian centuries from J2000.0

    // Geometric mean longitude of the Sun (degrees)
    let l0 = 280.46646 + 36000.76983 * t + 0.0003032 * t * t;

    // Mean anomaly of the Sun (degrees)
    let m_deg = 357.52911 + 35999.05029 * t - 0.0001537 * t * t;
    let m = m_deg.to_radians();

    // Equation of center
    let c = (1.914602 - 0.004817 * t - 0.000014 * t * t) * m.sin()
        + (0.019993 - 0.000101 * t) * (2.0 * m).sin()
        + 0.000289 * (3.0 * m).sin();

    // Sun's true longitude
    let sun_lon = l0 + c;

    // Apparent longitude (nutation + aberration, simplified)
    let omega = (125.04 - 1934.136 * t).to_radians();
    let apparent = sun_lon - 0.00569 - 0.00478 * omega.sin();

    apparent.rem_euclid(360.0)
}

/// Jie Qi boundary longitudes for BaZi months (reference table).
/// Each entry is the Sun longitude at which the month BEGINS.
/// Used by `solar_month_from_jd` via offset arithmetic from 315 deg (Li Chun).
///
/// Month 1 (Yin):  Li Chun  = 315 deg   Month 7 (Shen):  Li Qiu  = 135 deg
/// Month 2 (Mao):  Jing Zhe = 345 deg   Month 8 (You):   Bai Lu  = 165 deg
/// Month 3 (Chen): Qing Ming=  15 deg   Month 9 (Xu):    Han Lu  = 195 deg
/// Month 4 (Si):   Li Xia   =  45 deg   Month 10 (Hai):  Li Dong = 225 deg
/// Month 5 (Wu):   Mang Zhong= 75 deg   Month 11 (Zi):   Da Xue  = 255 deg
/// Month 6 (Wei):  Xiao Shu = 105 deg   Month 12 (Chou): Xiao Han= 285 deg
#[allow(dead_code)]
const MONTH_JIE_QI: [f64; 12] = [
    315.0, 345.0, 15.0, 45.0, 75.0, 105.0, 135.0, 165.0, 195.0, 225.0, 255.0, 285.0,
];

/// Determine the BaZi month (1-12) from a Julian Day based on the Sun's
/// ecliptic longitude and the Jie Qi boundaries.
/// Returns (month_number 1..=12, sun_longitude_degrees).
pub fn solar_month_from_jd(jd: f64) -> (usize, f64) {
    let sun_lon = solar_longitude_approx(jd);

    // Normalize longitude to an offset from Li Chun (315 deg).
    // This puts the start of month 1 at 0, and wraps through 360.
    let offset = (sun_lon - 315.0).rem_euclid(360.0);

    // Each month spans 30 degrees of longitude in this offset space.
    let month_zero = (offset / 30.0).floor() as usize;
    // month_zero is 0..11 corresponding to months 1..12.
    let month = month_zero + 1;

    (month, sun_lon)
}

/// Find the approximate Julian Day of Li Chun (Sun at 315 deg) in a given
/// Gregorian year. Li Chun typically falls around Feb 3-5.
/// Uses Newton-Raphson iteration on the solar longitude.
pub fn li_chun_jd(year: i32) -> f64 {
    // Initial guess: Feb 4 of the given year.
    // JD of 2000-01-01 12:00 UT = 2451545.0
    // Approximate JD of year-Feb-04: base + (year-2000)*365.25 + 34 days
    let jd_guess = 2451545.0 + (year - 2000) as f64 * 365.25 + 34.0;

    // Iterate to find when sun longitude = 315 deg.
    let mut jd = jd_guess;
    for _ in 0..20 {
        let lon = solar_longitude_approx(jd);
        // Difference from target 315, wrapped to -180..180
        let mut diff = (lon - 315.0).rem_euclid(360.0);
        if diff > 180.0 {
            diff -= 360.0;
        }
        if diff.abs() < 0.0001 {
            break;
        }
        // Sun moves ~1 degree/day
        jd -= diff;
    }
    jd
}

pub fn compute_bazi(year: i32, jd: f64, hour: f64) -> BaZiChart {
    // --- Year pillar: use Li Chun boundary, not Jan 1 ---
    // If date is before Li Chun of the Gregorian year, BaZi year = previous year.
    let li_chun = li_chun_jd(year);
    let bazi_year = if jd < li_chun { year - 1 } else { year };
    let year_pillar = sexagenary_year(bazi_year);

    let day_pillar = sexagenary_day(jd);
    let h_branch = hour_branch(hour);
    let h_stem = hour_stem(day_pillar.stem, h_branch);

    // --- Month pillar: based on solar term (Jie Qi) boundaries ---
    let (month_num, _sun_lon) = solar_month_from_jd(jd);
    // month_num is 1..=12, with 1 = Yin month starting at Li Chun

    // Month branch: Yin (index 2) for month 1, Mao (index 3) for month 2, etc.
    // month 11 = Zi (index 0), month 12 = Chou (index 1)
    let month_branch_idx = (month_num + 1) % 12; // month 1 -> 2 (Yin), month 11 -> 0 (Zi)
    let month_branch = EarthlyBranch::from_index(month_branch_idx);

    // Month stem: derived from year stem using the Five Tigers Escape (五虎遁).
    // Standard table:  Jia/Ji→Bing(2), Yi/Geng→Wu(4), Bing/Xin→Geng(6),
    //                  Ding/Ren→Ren(8), Wu/Gui→Jia(0)  for month 1 (Yin).
    // Formula: base = (year_stem_index % 5) * 2  (gives 0,2,4,6,8)
    //          month 1 stem = base + 2  (shift to Bing for Jia year)
    //          month N stem = base + 2 + (N-1) = base + N + 1
    let year_stem_idx = year_pillar.stem as usize;
    let month_stem_base = (year_stem_idx % 5) * 2;
    let month_stem = HeavenlyStem::from_index((month_stem_base + month_num + 1) % 10);

    BaZiChart {
        year: year_pillar,
        month: Pillar {
            stem: month_stem,
            branch: month_branch,
        },
        day: day_pillar,
        hour: Pillar {
            stem: h_stem,
            branch: h_branch,
        },
        day_master: day_pillar.stem,
    }
}

impl std::fmt::Display for Pillar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}-{:?} ({} {})",
            self.stem,
            self.branch,
            self.stem.element().name(),
            self.branch.animal()
        )
    }
}

impl WuXing {
    pub fn name(&self) -> &'static str {
        match self {
            WuXing::Wood => "Wood",
            WuXing::Fire => "Fire",
            WuXing::Earth => "Earth",
            WuXing::Metal => "Metal",
            WuXing::Water => "Water",
        }
    }

    pub fn generates(&self) -> WuXing {
        match self {
            WuXing::Wood => WuXing::Fire,
            WuXing::Fire => WuXing::Earth,
            WuXing::Earth => WuXing::Metal,
            WuXing::Metal => WuXing::Water,
            WuXing::Water => WuXing::Wood,
        }
    }

    pub fn overcomes(&self) -> WuXing {
        match self {
            WuXing::Wood => WuXing::Earth,
            WuXing::Fire => WuXing::Metal,
            WuXing::Earth => WuXing::Water,
            WuXing::Metal => WuXing::Wood,
            WuXing::Water => WuXing::Fire,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn year_2024_is_jia_chen() {
        let p = sexagenary_year(2024);
        assert_eq!(p.stem, HeavenlyStem::Jia);
        assert_eq!(p.branch, EarthlyBranch::Chen); // Dragon
    }

    #[test]
    fn year_2000_cycle() {
        let p = sexagenary_year(2000);
        assert_eq!(p.branch.animal(), "Dragon"); // 2000 = Geng-Chen
    }

    #[test]
    fn sexagenary_60_year_cycle() {
        let p1 = sexagenary_year(2024);
        let p2 = sexagenary_year(2024 + 60);
        assert_eq!(p1.stem, p2.stem);
        assert_eq!(p1.branch, p2.branch);
    }

    #[test]
    fn hour_branches() {
        assert_eq!(hour_branch(0.0), EarthlyBranch::Zi); // midnight = Zi
        assert_eq!(hour_branch(12.0), EarthlyBranch::Wu); // noon = Wu (Horse)
        assert_eq!(hour_branch(23.5), EarthlyBranch::Zi); // 23:30 = Zi
    }

    #[test]
    fn solar_terms_start_at_315() {
        assert_eq!(solar_term_sun_longitude(0), 315.0); // Li Chun
        assert_eq!(solar_term_sun_longitude(6), 45.0); // Li Xia (Start of Summer)
        assert_eq!(solar_term_sun_longitude(12), 135.0); // Li Qiu (Start of Autumn)
    }

    #[test]
    fn wu_xing_generating_cycle() {
        assert_eq!(WuXing::Wood.generates(), WuXing::Fire);
        assert_eq!(WuXing::Fire.generates(), WuXing::Earth);
        assert_eq!(WuXing::Earth.generates(), WuXing::Metal);
        assert_eq!(WuXing::Metal.generates(), WuXing::Water);
        assert_eq!(WuXing::Water.generates(), WuXing::Wood);
    }

    #[test]
    fn wu_xing_overcoming_cycle() {
        assert_eq!(WuXing::Wood.overcomes(), WuXing::Earth);
        assert_eq!(WuXing::Fire.overcomes(), WuXing::Metal);
    }

    #[test]
    fn heavenly_stem_elements() {
        assert_eq!(HeavenlyStem::Jia.element(), WuXing::Wood);
        assert_eq!(HeavenlyStem::Bing.element(), WuXing::Fire);
        assert!(HeavenlyStem::Jia.is_yang());
        assert!(!HeavenlyStem::Yi.is_yang());
    }

    #[test]
    fn bazi_chart_produces_4_pillars() {
        // 1990-01-15 12:00 UT (JD 2447908.0) — after Li Chun 1989, before Li Chun 1990
        // BaZi year should be 1989 (Ji-Si) since Jan 15 is before Li Chun (~Feb 4).
        let jd = 2447908.0; // 1990-01-15
        let chart = compute_bazi(1990, jd, 10.5);
        // Year should be previous year (1989 = Ji-Si) since before Li Chun
        assert_eq!(chart.year.stem, HeavenlyStem::Ji);
        assert_eq!(chart.year.branch, EarthlyBranch::Si);
        assert!(format!("{}", chart.day).len() > 0);
    }

    #[test]
    fn all_12_animals() {
        let animals: Vec<&str> = EarthlyBranch::ALL.iter().map(|b| b.animal()).collect();
        assert_eq!(animals.len(), 12);
        assert!(animals.contains(&"Dragon"));
        assert!(animals.contains(&"Rat"));
    }

    // ---- Solar longitude tests ----

    #[test]
    fn solar_longitude_at_j2000() {
        // J2000.0 = 2000-01-01 12:00 UT, Sun longitude ~ 280.5 deg
        let lon = solar_longitude_approx(2451545.0);
        assert!(
            (lon - 280.5).abs() < 1.0,
            "J2000 sun lon={lon}, expected ~280.5"
        );
    }

    #[test]
    fn solar_longitude_at_vernal_equinox_2024() {
        // 2024 vernal equinox: ~Mar 20, 03:06 UT → JD 2460388.629
        // Sun should be near 0 deg (Aries ingress)
        let lon = solar_longitude_approx(2460388.629);
        assert!(
            lon < 1.0 || lon > 359.0,
            "Vernal equinox 2024 sun lon={lon}, expected ~0 deg"
        );
    }

    #[test]
    fn solar_longitude_at_summer_solstice_2024() {
        // 2024 summer solstice: ~Jun 20, 20:51 UT → JD 2460481.369
        // Sun should be near 90 deg
        let lon = solar_longitude_approx(2460481.369);
        assert!(
            (lon - 90.0).abs() < 1.0,
            "Summer solstice 2024 sun lon={lon}, expected ~90 deg"
        );
    }

    // ---- Solar month determination tests ----

    #[test]
    fn solar_month_mid_march_is_month2() {
        // Mar 15 2024 → JD ~2460384.5
        // Sun ~355 deg → between Jing Zhe (345) and Qing Ming (15) → month 2 (Mao)
        let (month, lon) = solar_month_from_jd(2460384.5);
        assert_eq!(
            month, 2,
            "Mar 15 should be month 2 (Mao), got {month}, sun lon={lon}"
        );
    }

    #[test]
    fn solar_month_mid_july_is_month6() {
        // Jul 15 2024 → JD ~2460506.5
        // Sun ~113 deg → between Xiao Shu (105) and Li Qiu (135) → month 6 (Wei)
        let (month, lon) = solar_month_from_jd(2460506.5);
        assert_eq!(
            month, 6,
            "Jul 15 should be month 6 (Wei), got {month}, sun lon={lon}"
        );
    }

    #[test]
    fn solar_month_mid_january_is_month12() {
        // Jan 15 2024 → JD ~2460324.5
        // Sun ~295 deg → between Xiao Han (285) and Li Chun (315) → month 12 (Chou)
        let (month, lon) = solar_month_from_jd(2460324.5);
        assert_eq!(
            month, 12,
            "Jan 15 should be month 12 (Chou), got {month}, sun lon={lon}"
        );
    }

    #[test]
    fn solar_month_mid_december_is_month11() {
        // Dec 15 2023 → JD ~2460293.5
        // Sun ~263 deg → between Da Xue (255) and Xiao Han (285) → month 11 (Zi)
        let (month, lon) = solar_month_from_jd(2460293.5);
        assert_eq!(
            month, 11,
            "Dec 15 should be month 11 (Zi), got {month}, sun lon={lon}"
        );
    }

    #[test]
    fn all_12_months_reachable() {
        // Walk through the year 2024 month by month, verify each BaZi month appears
        let mut seen = [false; 12];
        // Start at Feb 15 2024 (JD ~2460355.5) and step by ~31 days
        let mut jd = 2460355.5;
        for _ in 0..12 {
            let (month, _) = solar_month_from_jd(jd);
            assert!(month >= 1 && month <= 12);
            seen[month - 1] = true;
            jd += 31.0;
        }
        for (i, &s) in seen.iter().enumerate() {
            assert!(s, "Month {} never seen during year walk", i + 1);
        }
    }

    // ---- Li Chun boundary tests ----

    #[test]
    fn li_chun_2024_around_feb4() {
        // Li Chun 2024 = Feb 4, 2024 ~04:27 UT → JD ~2460344.685
        let lc = li_chun_jd(2024);
        // Should be within 1 day of Feb 4
        assert!(
            (lc - 2460344.685).abs() < 1.0,
            "Li Chun 2024 JD={lc}, expected ~2460344.685"
        );
    }

    #[test]
    fn li_chun_2000_around_feb4() {
        // Li Chun 2000 = Feb 4, 2000 ~14:48 UT → JD ~2451579.117
        let lc = li_chun_jd(2000);
        assert!(
            (lc - 2451579.117).abs() < 1.0,
            "Li Chun 2000 JD={lc}, expected ~2451579.117"
        );
    }

    // ---- Year pillar Li Chun boundary tests ----

    #[test]
    fn year_pillar_before_li_chun_uses_previous_year() {
        // Jan 20 2024 (JD ~2460329.5) → before Li Chun 2024 (~Feb 4)
        // BaZi year = 2023 = Gui-Mao (Water Rabbit)
        let chart = compute_bazi(2024, 2460329.5, 12.0);
        assert_eq!(
            chart.year.stem,
            HeavenlyStem::Gui,
            "Before Li Chun: stem should be Gui (2023)"
        );
        assert_eq!(
            chart.year.branch,
            EarthlyBranch::Mao,
            "Before Li Chun: branch should be Mao (2023)"
        );
    }

    #[test]
    fn year_pillar_after_li_chun_uses_current_year() {
        // Mar 1 2024 (JD ~2460370.5) → after Li Chun 2024
        // BaZi year = 2024 = Jia-Chen (Wood Dragon)
        let chart = compute_bazi(2024, 2460370.5, 12.0);
        assert_eq!(
            chart.year.stem,
            HeavenlyStem::Jia,
            "After Li Chun: stem should be Jia (2024)"
        );
        assert_eq!(
            chart.year.branch,
            EarthlyBranch::Chen,
            "After Li Chun: branch should be Chen (2024)"
        );
    }

    // ---- Month branch mapping tests ----

    #[test]
    fn month_branch_yin_for_month1() {
        // Feb 15 2024 → after Li Chun (~Feb 4), before Jing Zhe (~Mar 5) → month 1
        // Month 1 branch = Yin (Tiger)
        let chart = compute_bazi(2024, 2460355.5, 12.0);
        assert_eq!(
            chart.month.branch,
            EarthlyBranch::Yin,
            "Month 1 branch should be Yin (Tiger)"
        );
    }

    #[test]
    fn month_branch_wu_for_month5() {
        // Jun 15 2024 → JD ~2460476.5
        // Sun ~84 deg → between Mang Zhong (75) and Xiao Shu (105) → month 5
        // Month 5 branch = Wu (Horse)
        let chart = compute_bazi(2024, 2460476.5, 12.0);
        assert_eq!(
            chart.month.branch,
            EarthlyBranch::Wu,
            "Month 5 branch should be Wu (Horse)"
        );
    }

    #[test]
    fn month_branch_zi_for_month11() {
        // Dec 15 2023 → JD ~2460293.5 → month 11
        // Month 11 branch = Zi (Rat)
        let chart = compute_bazi(2023, 2460293.5, 12.0);
        assert_eq!(
            chart.month.branch,
            EarthlyBranch::Zi,
            "Month 11 branch should be Zi (Rat)"
        );
    }

    // ---- Month stem tests ----

    #[test]
    fn month_stem_jia_year_month1_is_bing() {
        // Five Tigers Escape: Jia/Ji year → month 1 (Yin) stem = Bing.
        // Feb 15 2024: Jia year, month 1, should be Bing-Yin.
        let chart = compute_bazi(2024, 2460355.5, 12.0);
        assert_eq!(
            chart.month.stem,
            HeavenlyStem::Bing,
            "Jia year, month 1 stem should be Bing"
        );
        assert_eq!(
            chart.month.branch,
            EarthlyBranch::Yin,
            "Jia year, month 1 branch should be Yin"
        );
    }

    #[test]
    fn month_stem_yi_year_month1_is_wu() {
        // Five Tigers Escape: Yi/Geng year → month 1 stem = Wu (戊).
        // Feb 15 2025 → JD ~2460720.5 (approx). Yi year, month 1 = Wu-Yin.
        // 2025 = Yi-Si year. After Li Chun 2025 (~Feb 3).
        let chart = compute_bazi(2025, 2460720.5, 12.0);
        assert_eq!(
            chart.month.stem,
            HeavenlyStem::Wu,
            "Yi year, month 1 stem should be Wu"
        );
    }

    #[test]
    fn month_stem_all_five_year_groups() {
        // Verify month 1 stems for all five year-stem groups.
        // Jia(0)/Ji(5) → Bing(2), Yi(1)/Geng(6) → Wu(4),
        // Bing(2)/Xin(7) → Geng(6), Ding(3)/Ren(8) → Ren(8), Wu(4)/Gui(9) → Jia(0)
        let expected_month1_stems = [
            (HeavenlyStem::Jia, HeavenlyStem::Bing),
            (HeavenlyStem::Ji, HeavenlyStem::Bing),
            (HeavenlyStem::Yi, HeavenlyStem::Wu),
            (HeavenlyStem::Geng, HeavenlyStem::Wu),
            (HeavenlyStem::Bing, HeavenlyStem::Geng),
            (HeavenlyStem::Xin, HeavenlyStem::Geng),
            (HeavenlyStem::Ding, HeavenlyStem::Ren),
            (HeavenlyStem::Ren, HeavenlyStem::Ren),
            (HeavenlyStem::Wu, HeavenlyStem::Jia),
            (HeavenlyStem::Gui, HeavenlyStem::Jia),
        ];
        for (year_stem, expected_m1_stem) in expected_month1_stems {
            let year_stem_idx = year_stem as usize;
            let base = (year_stem_idx % 5) * 2;
            let m1_stem = HeavenlyStem::from_index((base + 1 + 1) % 10);
            assert_eq!(
                m1_stem, expected_m1_stem,
                "Year stem {:?}: month 1 stem should be {:?}, got {:?}",
                year_stem, expected_m1_stem, m1_stem
            );
        }
    }

    // ---- Known BaZi chart validation ----

    #[test]
    fn known_chart_2000_jun15() {
        // Jun 15 2000, 12:00 UT → JD 2451710.0
        // BaZi year 2000 = Geng-Chen (after Li Chun).
        // Sun ~84 deg → month 5 (Wu horse). Geng year → month 5 stem = Ren.
        // Five Tigers: Geng (idx 6), base = (6%5)*2 = 2, month 5: (2+5+1)%10 = 8 → Ren.
        let chart = compute_bazi(2000, 2451710.0, 12.0);
        assert_eq!(chart.year.stem, HeavenlyStem::Geng);
        assert_eq!(chart.year.branch, EarthlyBranch::Chen);
        assert_eq!(
            chart.month.stem,
            HeavenlyStem::Ren,
            "Geng year, month 5 stem should be Ren"
        );
        assert_eq!(
            chart.month.branch,
            EarthlyBranch::Wu,
            "Jun 15 2000 should be month 5, branch Wu (Horse)"
        );
    }
}
