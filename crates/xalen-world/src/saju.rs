//! Korean Saju (사주, "Four Pillars of Destiny")
//!
//! Korea's version of Chinese BaZi using the same Heavenly Stems (천간) and
//! Earthly Branches (지지) with Korean naming, plus distinct Daeun (대운,
//! major luck cycles) and Gunghap (궁합, compatibility) analysis.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Heavenly Stems (천간 Cheongan) — 10 stems
// ---------------------------------------------------------------------------

/// The 10 Heavenly Stems with Korean names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HeavenlyStem {
    Gap,    // 갑 (Wood Yang)
    Eul,    // 을 (Wood Yin)
    Byeong, // 병 (Fire Yang)
    Jeong,  // 정 (Fire Yin)
    Mu,     // 무 (Earth Yang)
    Gi,     // 기 (Earth Yin)
    Gyeong, // 경 (Metal Yang)
    Sin,    // 신 (Metal Yin)
    Im,     // 임 (Water Yang)
    Gye,    // 계 (Water Yin)
}

impl HeavenlyStem {
    pub const ALL: [HeavenlyStem; 10] = [
        HeavenlyStem::Gap,
        HeavenlyStem::Eul,
        HeavenlyStem::Byeong,
        HeavenlyStem::Jeong,
        HeavenlyStem::Mu,
        HeavenlyStem::Gi,
        HeavenlyStem::Gyeong,
        HeavenlyStem::Sin,
        HeavenlyStem::Im,
        HeavenlyStem::Gye,
    ];

    pub fn from_index(i: usize) -> Self {
        Self::ALL[i % 10]
    }

    /// Korean name in Hangul.
    pub fn korean(&self) -> &'static str {
        match self {
            HeavenlyStem::Gap => "갑",
            HeavenlyStem::Eul => "을",
            HeavenlyStem::Byeong => "병",
            HeavenlyStem::Jeong => "정",
            HeavenlyStem::Mu => "무",
            HeavenlyStem::Gi => "기",
            HeavenlyStem::Gyeong => "경",
            HeavenlyStem::Sin => "신",
            HeavenlyStem::Im => "임",
            HeavenlyStem::Gye => "계",
        }
    }

    /// Romanized Korean name.
    pub fn name(&self) -> &'static str {
        match self {
            HeavenlyStem::Gap => "Gap",
            HeavenlyStem::Eul => "Eul",
            HeavenlyStem::Byeong => "Byeong",
            HeavenlyStem::Jeong => "Jeong",
            HeavenlyStem::Mu => "Mu",
            HeavenlyStem::Gi => "Gi",
            HeavenlyStem::Gyeong => "Gyeong",
            HeavenlyStem::Sin => "Sin",
            HeavenlyStem::Im => "Im",
            HeavenlyStem::Gye => "Gye",
        }
    }

    /// Element associated with this stem.
    pub fn element(&self) -> Element {
        match self {
            HeavenlyStem::Gap | HeavenlyStem::Eul => Element::Wood,
            HeavenlyStem::Byeong | HeavenlyStem::Jeong => Element::Fire,
            HeavenlyStem::Mu | HeavenlyStem::Gi => Element::Earth,
            HeavenlyStem::Gyeong | HeavenlyStem::Sin => Element::Metal,
            HeavenlyStem::Im | HeavenlyStem::Gye => Element::Water,
        }
    }

    /// Whether this stem is Yang (양).
    pub fn is_yang(&self) -> bool {
        (*self as usize).is_multiple_of(2)
    }

    /// Index within the 10-stem cycle.
    pub fn index(&self) -> usize {
        *self as usize
    }
}

// ---------------------------------------------------------------------------
// Earthly Branches (지지 Jiji) — 12 branches
// ---------------------------------------------------------------------------

/// The 12 Earthly Branches with Korean animal associations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EarthlyBranch {
    Ja,   // 자 — Rat (쥐)
    Chuk, // 축 — Ox (소)
    In,   // 인 — Tiger (호랑이)
    Myo,  // 묘 — Rabbit (토끼)
    Jin,  // 진 — Dragon (용)
    Sa,   // 사 — Snake (뱀)
    O,    // 오 — Horse (말)
    Mi,   // 미 — Sheep (양)
    Sin_, // 신 — Monkey (원숭이)
    Yu,   // 유 — Rooster (닭)
    Sul,  // 술 — Dog (개)
    Hae,  // 해 — Pig (돼지)
}

impl EarthlyBranch {
    pub const ALL: [EarthlyBranch; 12] = [
        EarthlyBranch::Ja,
        EarthlyBranch::Chuk,
        EarthlyBranch::In,
        EarthlyBranch::Myo,
        EarthlyBranch::Jin,
        EarthlyBranch::Sa,
        EarthlyBranch::O,
        EarthlyBranch::Mi,
        EarthlyBranch::Sin_,
        EarthlyBranch::Yu,
        EarthlyBranch::Sul,
        EarthlyBranch::Hae,
    ];

    pub fn from_index(i: usize) -> Self {
        Self::ALL[i % 12]
    }

    /// Korean animal name in Hangul.
    pub fn animal_korean(&self) -> &'static str {
        match self {
            EarthlyBranch::Ja => "쥐",
            EarthlyBranch::Chuk => "소",
            EarthlyBranch::In => "호랑이",
            EarthlyBranch::Myo => "토끼",
            EarthlyBranch::Jin => "용",
            EarthlyBranch::Sa => "뱀",
            EarthlyBranch::O => "말",
            EarthlyBranch::Mi => "양",
            EarthlyBranch::Sin_ => "원숭이",
            EarthlyBranch::Yu => "닭",
            EarthlyBranch::Sul => "개",
            EarthlyBranch::Hae => "돼지",
        }
    }

    /// English animal name.
    pub fn animal(&self) -> &'static str {
        match self {
            EarthlyBranch::Ja => "Rat",
            EarthlyBranch::Chuk => "Ox",
            EarthlyBranch::In => "Tiger",
            EarthlyBranch::Myo => "Rabbit",
            EarthlyBranch::Jin => "Dragon",
            EarthlyBranch::Sa => "Snake",
            EarthlyBranch::O => "Horse",
            EarthlyBranch::Mi => "Sheep",
            EarthlyBranch::Sin_ => "Monkey",
            EarthlyBranch::Yu => "Rooster",
            EarthlyBranch::Sul => "Dog",
            EarthlyBranch::Hae => "Pig",
        }
    }

    /// Korean branch name in Hangul.
    pub fn korean(&self) -> &'static str {
        match self {
            EarthlyBranch::Ja => "자",
            EarthlyBranch::Chuk => "축",
            EarthlyBranch::In => "인",
            EarthlyBranch::Myo => "묘",
            EarthlyBranch::Jin => "진",
            EarthlyBranch::Sa => "사",
            EarthlyBranch::O => "오",
            EarthlyBranch::Mi => "미",
            EarthlyBranch::Sin_ => "신",
            EarthlyBranch::Yu => "유",
            EarthlyBranch::Sul => "술",
            EarthlyBranch::Hae => "해",
        }
    }

    /// Element of this branch.
    pub fn element(&self) -> Element {
        match self {
            EarthlyBranch::In | EarthlyBranch::Myo => Element::Wood,
            EarthlyBranch::Sa | EarthlyBranch::O => Element::Fire,
            EarthlyBranch::Chuk | EarthlyBranch::Jin | EarthlyBranch::Mi | EarthlyBranch::Sul => {
                Element::Earth
            }
            EarthlyBranch::Sin_ | EarthlyBranch::Yu => Element::Metal,
            EarthlyBranch::Hae | EarthlyBranch::Ja => Element::Water,
        }
    }

    /// Index within the 12-branch cycle.
    pub fn index(&self) -> usize {
        *self as usize
    }
}

// ---------------------------------------------------------------------------
// Five Elements (오행 Ohaeng)
// ---------------------------------------------------------------------------

/// The five elements in Korean astrology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Element {
    Wood,  // 목 (Mok)
    Fire,  // 화 (Hwa)
    Earth, // 토 (To)
    Metal, // 금 (Geum)
    Water, // 수 (Su)
}

impl Element {
    /// Korean name in Hangul.
    pub fn korean(&self) -> &'static str {
        match self {
            Element::Wood => "목",
            Element::Fire => "화",
            Element::Earth => "토",
            Element::Metal => "금",
            Element::Water => "수",
        }
    }

    /// English name.
    pub fn name(&self) -> &'static str {
        match self {
            Element::Wood => "Wood",
            Element::Fire => "Fire",
            Element::Earth => "Earth",
            Element::Metal => "Metal",
            Element::Water => "Water",
        }
    }

    /// Element that this element generates (상생 Sangsaeng — generating cycle).
    pub fn generates(&self) -> Element {
        match self {
            Element::Wood => Element::Fire,
            Element::Fire => Element::Earth,
            Element::Earth => Element::Metal,
            Element::Metal => Element::Water,
            Element::Water => Element::Wood,
        }
    }

    /// Element that this element overcomes (상극 Sanggeuk — overcoming cycle).
    pub fn overcomes(&self) -> Element {
        match self {
            Element::Wood => Element::Earth,
            Element::Fire => Element::Metal,
            Element::Earth => Element::Water,
            Element::Metal => Element::Wood,
            Element::Water => Element::Fire,
        }
    }
}

// ---------------------------------------------------------------------------
// Gender
// ---------------------------------------------------------------------------

/// Gender for Daeun calculation (direction of luck cycle flow).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
}

// ---------------------------------------------------------------------------
// SajuChart
// ---------------------------------------------------------------------------

/// A Korean Saju (Four Pillars) chart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SajuChart {
    pub year_pillar: (HeavenlyStem, EarthlyBranch),
    pub month_pillar: (HeavenlyStem, EarthlyBranch),
    pub day_pillar: (HeavenlyStem, EarthlyBranch),
    pub hour_pillar: (HeavenlyStem, EarthlyBranch),
    /// Day Master (일간 Ilgan) — the stem of the day pillar; primary identity.
    pub day_master: HeavenlyStem,
    /// Element of the Day Master.
    pub day_master_element: Element,
    /// Birth instant as a Julian Day (midnight JD of the civil date + hour
    /// fraction). Retained so [`daeun`] can recover the astronomically correct
    /// distance to the adjacent solar term (the real Daeun start age) without a
    /// signature change. Not part of the four pillars themselves.
    pub birth_jd: f64,
}

impl std::fmt::Display for SajuChart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Year: {}-{} | Month: {}-{} | Day: {}-{} | Hour: {}-{} | Day Master: {} ({})",
            self.year_pillar.0.name(),
            self.year_pillar.1.animal(),
            self.month_pillar.0.name(),
            self.month_pillar.1.animal(),
            self.day_pillar.0.name(),
            self.day_pillar.1.animal(),
            self.hour_pillar.0.name(),
            self.hour_pillar.1.animal(),
            self.day_master.korean(),
            self.day_master_element.korean(),
        )
    }
}

// ---------------------------------------------------------------------------
// Daeun (대운, Major Luck Cycles)
// ---------------------------------------------------------------------------

/// A single 10-year Daeun (major luck) period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaeunPeriod {
    /// Starting age for this period.
    pub start_age: u32,
    /// Ending age (exclusive) for this period.
    pub end_age: u32,
    /// The stem governing this period.
    pub stem: HeavenlyStem,
    /// The branch governing this period.
    pub branch: EarthlyBranch,
}

impl std::fmt::Display for DaeunPeriod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Age {}-{}: {}{} ({} {})",
            self.start_age,
            self.end_age,
            self.stem.korean(),
            self.branch.korean(),
            self.stem.name(),
            self.branch.animal(),
        )
    }
}

// ---------------------------------------------------------------------------
// Gunghap (궁합, Compatibility)
// ---------------------------------------------------------------------------

/// Result of a Gunghap (궁합) compatibility analysis between two charts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GunghapResult {
    /// Overall harmony score (0-100).
    pub score: u32,
    /// Whether the day-master elements are in a generating relationship.
    pub day_master_harmony: bool,
    /// Number of matching branches across all pillars (0-4).
    pub branch_matches: u32,
    /// Number of branch clashes (opposing branches) across pillars.
    pub branch_clashes: u32,
    /// Textual summary.
    pub summary: &'static str,
}

// ---------------------------------------------------------------------------
// Core computations
// ---------------------------------------------------------------------------

/// Compute the year pillar for a given year using the sexagenary cycle.
/// Year 4 CE = Gap-Ja (cycle start), same as Chinese BaZi.
fn year_pillar(year: i32) -> (HeavenlyStem, EarthlyBranch) {
    let cycle = ((year - 4).rem_euclid(60)) as usize;
    (
        HeavenlyStem::from_index(cycle),
        EarthlyBranch::from_index(cycle),
    )
}

/// Determine the Saju solar year for a birth instant, honoring the Ipchun
/// (입춘 / 立春 Li Chun, ≈ 4 Feb) boundary that opens the solar year.
///
/// Saju, like Chinese BaZi, anchors the year pillar to the Sun reaching 315°
/// (Start of Spring), NOT to the civil 1 January. A birth between 1 January and
/// Ipchun therefore belongs to the *previous* solar year. We reuse the published
/// `xalen_chinese::li_chun_jd` so Saju and BaZi share one solar-term model.
fn saju_solar_year(year: i32, birth_jd: f64) -> i32 {
    let li_chun = xalen_chinese::li_chun_jd(year);
    if birth_jd < li_chun { year - 1 } else { year }
}

/// Signed day-distance from `birth_jd` to the adjacent **Jie Qi** (節氣, the
/// 12 major month-opening solar terms at 30° intervals starting from Li Chun =
/// 315°). Positive when `forward` (distance to the NEXT term), negative when
/// backward (distance to the PREVIOUS term). Used by [`daeun`] for the start age.
///
/// The Sun's ecliptic longitude is the published `xalen_chinese` approximation
/// (Meeus low-accuracy, ~0.01°), so Saju and BaZi share one solar model. The
/// term longitudes are the multiples-of-30° grid phased at 315°
/// (315, 345, 15, 45, …, 285) — exactly the BaZi month boundaries.
fn days_to_adjacent_jie_qi(birth_jd: f64, forward: bool) -> f64 {
    // Mean solar motion ≈ 360°/365.2422 ≈ 0.9856°/day — used for the Newton step.
    const DEG_PER_DAY: f64 = 0.985_647;

    let lon0 = xalen_chinese::solar_longitude_approx(birth_jd);
    // How far (0..30) the Sun has advanced past the most recent Jie Qi boundary.
    let past = (lon0 - 315.0).rem_euclid(30.0);

    // Target longitude of the adjacent boundary, expressed relative to lon0 so
    // the sign of the longitude delta carries the search direction.
    let target_delta_deg = if forward {
        30.0 - past // ahead to the next boundary (0, 30]
    } else if past < 1e-9 {
        -30.0 // exactly on a boundary: the previous one is a full term back
    } else {
        -past // behind to the previous boundary [-30, 0)
    };
    let target_lon = (lon0 + target_delta_deg).rem_euclid(360.0);

    // Newton/secant refinement on solar longitude. Initial guess by mean motion.
    let mut jd = birth_jd + target_delta_deg / DEG_PER_DAY;
    for _ in 0..40 {
        let cur = xalen_chinese::solar_longitude_approx(jd);
        // Signed angular error wrapped to (-180, 180].
        let mut diff = (cur - target_lon).rem_euclid(360.0);
        if diff > 180.0 {
            diff -= 360.0;
        }
        if diff.abs() < 1e-6 {
            break;
        }
        jd -= diff / DEG_PER_DAY;
    }
    jd - birth_jd
}

/// Compute the month pillar from the year stem and month (1-12).
///
/// The month stem is derived from the year stem following the same
/// five-tiger-escape formula (오호둔갑 Oho-Dungap) as BaZi:
/// Year stem Gap/Gi -> month 1 starts at Byeong-In
/// Year stem Eul/Gyeong -> month 1 starts at Mu-In
/// Year stem Byeong/Sin -> month 1 starts at Gyeong-In
/// Year stem Jeong/Im -> month 1 starts at Im-In
/// Year stem Mu/Gye -> month 1 starts at Gap-In
fn month_pillar(year_stem: HeavenlyStem, month: u32) -> (HeavenlyStem, EarthlyBranch) {
    // Clamp untrusted / deserialized `month` into the valid 1..=12 range
    // BEFORE the `- 1` subtraction. A `month` of 0 (e.g. from a malformed
    // deserialized payload) would otherwise underflow `usize`, panicking in
    // debug builds (overflow-checks) and silently wrapping in release.
    let m = month.clamp(1, 12) as usize;

    // Month branches: month 1 (February) = In (Tiger), month 2 = Myo, etc.
    let branch_idx = ((m - 1) + 2) % 12; // In=2
    let branch = EarthlyBranch::from_index(branch_idx);

    // Month stem base from year stem (five-tiger-escape)
    let stem_base = match year_stem {
        HeavenlyStem::Gap | HeavenlyStem::Gi => 2,     // Byeong
        HeavenlyStem::Eul | HeavenlyStem::Gyeong => 4, // Mu
        HeavenlyStem::Byeong | HeavenlyStem::Sin => 6, // Gyeong
        HeavenlyStem::Jeong | HeavenlyStem::Im => 8,   // Im
        HeavenlyStem::Mu | HeavenlyStem::Gye => 0,     // Gap
    };
    let stem = HeavenlyStem::from_index(stem_base + (m - 1));

    (stem, branch)
}

/// Approximate the day pillar from a Gregorian date.
///
/// Uses the same sexagenary day-count as Chinese BaZi, with a continuous
/// day count from a known reference point.
///
/// Verified against ytliu0.github.io/ChineseCalendar/sexagenary.html:
/// 1970-01-01 (JD 2440588 noon) = Xin-Si = Sin(7)-Sa(5) in Korean.
/// 2000-01-01 (JD 2451545 noon) = Wu-Wu  = Mu(4)-O(6)   in Korean.
fn day_pillar(year: i32, month: u32, day: u32) -> (HeavenlyStem, EarthlyBranch) {
    let jd = gregorian_to_jd(year, month, day);
    // JD 2440587.5 = midnight 1970-01-01 = Sin(7)-Sa(5) in sexagenary cycle
    // (Xin-Si in Chinese; verified via ytliu0 formula: T=1+(JD_noon-1)%10, B=1+(JD_noon+1)%12)
    let day_count = ((jd - 2_440_587.5).floor() as i64).rem_euclid(60) as usize;
    let stem_offset = 7; // Sin = index 7 (Xin in Chinese)
    let branch_offset = 5; // Sa = index 5 (Si in Chinese)
    (
        HeavenlyStem::from_index((day_count + stem_offset) % 10),
        EarthlyBranch::from_index((day_count + branch_offset) % 12),
    )
}

/// Compute the hour pillar from the day stem and birth hour (0-23).
///
/// Hours map to 2-hour blocks starting from Ja (23:00-01:00).
fn hour_pillar(day_stem: HeavenlyStem, hour: u32) -> (HeavenlyStem, EarthlyBranch) {
    let branch_idx = (((hour + 1) % 24) / 2) as usize;
    let branch = EarthlyBranch::from_index(branch_idx);

    // Hour stem from day stem (오서둔갑 Oseo-Dungap)
    let day_idx = day_stem.index();
    let base = (day_idx % 5) * 2;
    let stem = HeavenlyStem::from_index(base + branch_idx);

    (stem, branch)
}

/// Convert a Gregorian civil date to its Julian Day at 00:00 UT (midnight) — the
/// trailing `- 1524.5` (not `- 1524.0`) makes this midnight-based. Callers add an
/// hour fraction when they need the exact birth instant; `day_pillar` floors it.
fn gregorian_to_jd(year: i32, month: u32, day: u32) -> f64 {
    let y = if month <= 2 { year - 1 } else { year } as f64;
    let m = if month <= 2 { month + 12 } else { month } as f64;
    let d = day as f64;
    let a = (y / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();
    (365.25 * (y + 4716.0)).floor() + (30.6001 * (m + 1.0)).floor() + d + b - 1524.5
}

/// Compute a full Saju chart for a given birth date and hour.
///
/// - `year`: Gregorian year (e.g., 1990)
/// - `month`: month 1-12 (out-of-range values are clamped)
/// - `day`: day of month 1-31 (out-of-range values are clamped)
/// - `hour`: hour 0-23 (out-of-range values are clamped)
///
/// Out-of-range inputs (e.g. from a malformed deserialized payload) are clamped
/// ONCE here so every pillar uses the same values. Previously `month_pillar`
/// clamped its month internally while `day_pillar` received the raw month, which
/// could desync the month and day pillars for an out-of-range month.
pub fn compute_saju(year: i32, month: u32, day: u32, hour: u32) -> SajuChart {
    let month = month.clamp(1, 12);
    let day = day.clamp(1, 31);
    let hour = hour.min(23);

    // Birth instant as a Julian Day (midnight JD of the civil date + the hour
    // fraction). Used both to resolve the Ipchun solar-year boundary for the
    // year pillar and the Jie Qi solar-term distance for Daeun.
    let birth_jd = gregorian_to_jd(year, month, day) + hour as f64 / 24.0;

    // Year pillar uses the Saju SOLAR year (Ipchun boundary), not the raw
    // Gregorian year — a Jan..~Feb-4 birth rolls back to the previous year.
    let solar_year = saju_solar_year(year, birth_jd);
    let yp = year_pillar(solar_year);
    let mp = month_pillar(yp.0, month);
    let dp = day_pillar(year, month, day);
    let hp = hour_pillar(dp.0, hour);

    SajuChart {
        year_pillar: yp,
        month_pillar: mp,
        day_pillar: dp,
        hour_pillar: hp,
        day_master: dp.0,
        day_master_element: dp.0.element(),
        birth_jd,
    }
}

/// Compute Daeun (대운, major luck cycles) — 10-year periods that shape
/// a person's fate. The direction of traversal through the stems/branches
/// depends on gender and whether the year stem is Yang or Yin.
///
/// - Male + Yang year OR Female + Yin year: forward through the cycle
/// - Male + Yin year OR Female + Yang year: backward through the cycle
///
/// Returns 8 periods covering ages roughly 1-80.
pub fn daeun(chart: &SajuChart, gender: Gender, _birth_year: i32) -> Vec<DaeunPeriod> {
    let year_stem_yang = chart.year_pillar.0.is_yang();
    let forward = match gender {
        Gender::Male => year_stem_yang,
        Gender::Female => !year_stem_yang,
    };

    let month_stem_idx = chart.month_pillar.0.index();
    let month_branch_idx = chart.month_pillar.1.index();

    // Starting age for the first Daeun = distance (in days) from birth to the
    // ADJACENT solar term, divided by three (the classical "3 days = 1 year"
    // rule). The adjacent term is the NEXT Jie Qi when the cycle runs forward
    // (yang-year male / yin-year female) and the PREVIOUS one when it runs
    // backward — i.e. the same direction that drives the stem/branch traversal.
    // `_birth_year` is retained for API compatibility but is no longer used: the
    // distance comes from the stored birth instant, which is astronomically
    // meaningful, unlike the old `birth_year % 3` placeholder.
    let days = days_to_adjacent_jie_qi(chart.birth_jd, forward).abs();
    // 3 days ≈ 1 year of life; round to the nearest whole year, floored at 1
    // (a real Daeun never starts at age 0). Real start ages run ~1-10.
    let start_age = ((days / 3.0).round() as u32).clamp(1, 10);

    let mut periods = Vec::with_capacity(8);
    for i in 0..8u32 {
        let offset = (i + 1) as usize;
        let (stem_idx, branch_idx) = if forward {
            (
                (month_stem_idx + offset) % 10,
                (month_branch_idx + offset) % 12,
            )
        } else {
            (
                (month_stem_idx + 10 - (offset % 10)) % 10,
                (month_branch_idx + 12 - (offset % 12)) % 12,
            )
        };

        let period_start = start_age + i * 10;
        periods.push(DaeunPeriod {
            start_age: period_start,
            end_age: period_start + 10,
            stem: HeavenlyStem::from_index(stem_idx),
            branch: EarthlyBranch::from_index(branch_idx),
        });
    }

    periods
}

/// Compute Gunghap (궁합, compatibility) between two Saju charts.
///
/// Evaluates:
/// 1. Day-master element harmony (generating cycle = good)
/// 2. Branch matching across all 4 pillars
/// 3. Branch clashing (directly opposite branches are clashes)
pub fn gunghap(chart_a: &SajuChart, chart_b: &SajuChart) -> GunghapResult {
    // Day master harmony: check if elements are in a generating relationship
    let elem_a = chart_a.day_master_element;
    let elem_b = chart_b.day_master_element;
    let day_master_harmony =
        elem_a.generates() == elem_b || elem_b.generates() == elem_a || elem_a == elem_b;

    // Branch matching and clashing across all 4 pillars
    let pillars_a = [
        chart_a.year_pillar.1,
        chart_a.month_pillar.1,
        chart_a.day_pillar.1,
        chart_a.hour_pillar.1,
    ];
    let pillars_b = [
        chart_b.year_pillar.1,
        chart_b.month_pillar.1,
        chart_b.day_pillar.1,
        chart_b.hour_pillar.1,
    ];

    let mut branch_matches = 0u32;
    let mut branch_clashes = 0u32;

    for i in 0..4 {
        if pillars_a[i] == pillars_b[i] {
            branch_matches += 1;
        }
        // Clash: branches 6 apart in the 12-branch cycle
        let diff = (pillars_a[i].index() as i32 - pillars_b[i].index() as i32).unsigned_abs();
        if diff == 6 {
            branch_clashes += 1;
        }
    }

    // Score: base 50, +15 for day master harmony, +10 per match, -15 per clash
    let raw_score = 50i32 + if day_master_harmony { 15 } else { 0 } + (branch_matches as i32 * 10)
        - (branch_clashes as i32 * 15);
    let score = raw_score.clamp(0, 100) as u32;

    let summary = match score {
        80..=100 => "Excellent compatibility — strong natural harmony",
        60..=79 => "Good compatibility — complementary energies",
        40..=59 => "Average compatibility — some tension, some support",
        20..=39 => "Challenging compatibility — significant friction",
        _ => "Difficult compatibility — opposing energies",
    };

    GunghapResult {
        score,
        day_master_harmony,
        branch_matches,
        branch_clashes,
        summary,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn year_2024_is_gap_jin() {
        // 2024 = Jia-Chen in Chinese = Gap-Jin in Korean (Wood Dragon)
        let chart = compute_saju(2024, 6, 15, 12);
        assert_eq!(chart.year_pillar.0, HeavenlyStem::Gap);
        assert_eq!(chart.year_pillar.1, EarthlyBranch::Jin); // Dragon
    }

    #[test]
    fn day_master_matches_day_stem() {
        let chart = compute_saju(1990, 3, 15, 10);
        assert_eq!(chart.day_master, chart.day_pillar.0);
        assert_eq!(chart.day_master_element, chart.day_pillar.0.element());
    }

    #[test]
    fn sexagenary_cycle_repeats_at_60() {
        let a = compute_saju(2024, 1, 1, 0);
        let b = compute_saju(2084, 1, 1, 0);
        assert_eq!(a.year_pillar, b.year_pillar);
    }

    #[test]
    fn all_10_stems_korean() {
        let names: Vec<&str> = HeavenlyStem::ALL.iter().map(|s| s.korean()).collect();
        assert_eq!(
            names,
            ["갑", "을", "병", "정", "무", "기", "경", "신", "임", "계"]
        );
    }

    #[test]
    fn all_12_animals_korean() {
        let animals: Vec<&str> = EarthlyBranch::ALL
            .iter()
            .map(|b| b.animal_korean())
            .collect();
        assert_eq!(
            animals,
            [
                "쥐",
                "소",
                "호랑이",
                "토끼",
                "용",
                "뱀",
                "말",
                "양",
                "원숭이",
                "닭",
                "개",
                "돼지"
            ]
        );
    }

    #[test]
    fn stem_elements_correct() {
        assert_eq!(HeavenlyStem::Gap.element(), Element::Wood);
        assert_eq!(HeavenlyStem::Byeong.element(), Element::Fire);
        assert_eq!(HeavenlyStem::Mu.element(), Element::Earth);
        assert_eq!(HeavenlyStem::Gyeong.element(), Element::Metal);
        assert_eq!(HeavenlyStem::Im.element(), Element::Water);
    }

    #[test]
    fn yang_yin_alternates() {
        for (i, stem) in HeavenlyStem::ALL.iter().enumerate() {
            assert_eq!(stem.is_yang(), i % 2 == 0);
        }
    }

    #[test]
    fn daeun_returns_8_periods() {
        let chart = compute_saju(1990, 5, 20, 14);
        let periods = daeun(&chart, Gender::Male, 1990);
        assert_eq!(periods.len(), 8);
    }

    #[test]
    fn daeun_periods_are_consecutive() {
        let chart = compute_saju(1985, 8, 10, 6);
        let periods = daeun(&chart, Gender::Female, 1985);
        for i in 1..periods.len() {
            assert_eq!(periods[i].start_age, periods[i - 1].end_age);
        }
    }

    #[test]
    fn daeun_male_yang_goes_forward() {
        // 2024 year stem = Gap (Yang) + Male -> forward.
        // NOTE: the date must be AFTER Ipchun (Li Chun ≈ 4 Feb) so the Saju solar
        // year is 2024 (Gap, Yang). A January date would roll back to 2023 (Gui,
        // Yin) under the Ipchun boundary and flip the direction — see
        // `ipchun_year_boundary_january_birth`. We use mid-March 2024.
        let chart = compute_saju(2024, 3, 15, 8);
        assert!(chart.year_pillar.0.is_yang());
        let periods = daeun(&chart, Gender::Male, 2024);
        // First period stem should be the NEXT stem after the month stem
        let expected_stem_idx = (chart.month_pillar.0.index() + 1) % 10;
        assert_eq!(periods[0].stem, HeavenlyStem::from_index(expected_stem_idx));
    }

    #[test]
    fn gunghap_identical_charts_high_score() {
        let chart = compute_saju(1990, 5, 20, 14);
        let result = gunghap(&chart, &chart);
        assert!(result.score >= 80);
        assert!(result.day_master_harmony);
        assert_eq!(result.branch_matches, 4);
        assert_eq!(result.branch_clashes, 0);
    }

    #[test]
    fn gunghap_score_in_range() {
        let a = compute_saju(1988, 3, 15, 10);
        let b = compute_saju(1991, 7, 22, 16);
        let result = gunghap(&a, &b);
        assert!(result.score <= 100);
    }

    #[test]
    fn element_generating_cycle() {
        assert_eq!(Element::Wood.generates(), Element::Fire);
        assert_eq!(Element::Fire.generates(), Element::Earth);
        assert_eq!(Element::Earth.generates(), Element::Metal);
        assert_eq!(Element::Metal.generates(), Element::Water);
        assert_eq!(Element::Water.generates(), Element::Wood);
    }

    #[test]
    fn element_overcoming_cycle() {
        assert_eq!(Element::Wood.overcomes(), Element::Earth);
        assert_eq!(Element::Fire.overcomes(), Element::Metal);
    }

    #[test]
    fn display_format_readable() {
        let chart = compute_saju(1990, 5, 20, 14);
        let s = format!("{chart}");
        assert!(s.contains("Day Master"));
        assert!(s.contains("Year:"));
    }

    #[test]
    fn month_branch_starts_at_tiger() {
        // Month 1 (February) should have branch In (Tiger)
        let chart = compute_saju(2000, 1, 15, 12);
        assert_eq!(chart.month_pillar.1, EarthlyBranch::In);
    }

    // --- P1-27 regression: out-of-range `month` underflow + pillar desync ------

    #[test]
    fn month_out_of_range_clamps_consistently() {
        // A malformed / deserialized `month` of 0 used to underflow `usize` in
        // `month_pillar` (`month as usize - 1`), panicking under overflow-checks.
        // After clamping to 1..=12 ONCE in `compute_saju`, month 0 must behave
        // exactly like month 1 — for BOTH the month pillar AND the day pillar.
        // (The day pillar previously received the raw, unclamped month, so an
        // out-of-range month desynced it from the clamped month pillar.)
        let lo = compute_saju(1990, 0, 15, 10);
        let m1 = compute_saju(1990, 1, 15, 10);
        assert_eq!(lo.month_pillar, m1.month_pillar);
        assert_eq!(
            lo.day_pillar, m1.day_pillar,
            "day pillar must use the clamped month (1), not the raw 0"
        );

        let hi = compute_saju(1990, 99, 15, 10);
        let m12 = compute_saju(1990, 12, 15, 10);
        assert_eq!(hi.month_pillar, m12.month_pillar);
        assert_eq!(
            hi.day_pillar, m12.day_pillar,
            "day pillar must use the clamped month (12), not the raw 99"
        );
    }

    #[test]
    fn day_pillar_anchors_match_reference() {
        // Verified against ytliu0.github.io/ChineseCalendar/sexagenary.html:
        //   1970-01-01 -> Sin(7)-Sa(5)   (Xin-Si in Chinese)
        //   2000-01-01 -> Mu(4)-O(6)     (Wu-Wu  in Chinese)
        let a = compute_saju(1970, 1, 1, 12);
        assert_eq!(a.day_pillar.0, HeavenlyStem::Sin); // stem index 7
        assert_eq!(a.day_pillar.1, EarthlyBranch::Sa); // branch index 5

        let b = compute_saju(2000, 1, 1, 12);
        assert_eq!(b.day_pillar.0, HeavenlyStem::Mu); // stem index 4
        assert_eq!(b.day_pillar.1, EarthlyBranch::O); // branch index 6
    }
}
