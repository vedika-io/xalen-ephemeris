//! # Qi Men Dun Jia (奇门遁甲) — Strategic Divination
//!
//! Qi Men Dun Jia ("Mysterious Gates Escaping Technique") is one of the
//! three supreme arts of Chinese metaphysics. It uses a 9-palace grid based
//! on the Lo Shu magic square, combined with 8 Doors, 8 Deities, 3 Qi
//! (Heavenly Stems Yi/Bing/Ding), and 6 Yi (Wu/Ji/Geng/Xin/Ren/Gui).
//!
//! This module provides the foundational structures (9 Stars, 8 Doors,
//! 8 Deities, San Qi / Liu Yi, the Lo Shu magic square) and a time-based
//! Qi Men Dun Jia chart computation.
//!
//! # Which method this implements
//!
//! [`compute_qimen`] casts the **time chart (時家奇門) by the San Yuan (三元)
//! placement school** — the standard time-based method in which:
//!
//! * **Ju (局) selection** is set by the **solar term** (二十四节气) the instant
//!   falls in, together with the **San Yuan upper/middle/lower-yuan rule** keyed
//!   off the day pillar's Fu Tou (符头) — see [`qimen_ju`]. This is the
//!   canonical determination, not a civil-month approximation.
//! * **Zhi Fu (值符) and Zhi Shi (值使)** are anchored to the palace where the
//!   hour's Xun-head Yi (旬首, the Six-Yi stem hiding the leading Jia) sits on
//!   the Earth Plate: the Zhi Fu star and Zhi Shi door of that palace lead the
//!   Heaven-Plate rotation along the Yang (forward) / Yin (reverse) path — see
//!   [`compute_qimen`].
//!
//! # Honest scope
//!
//! Qi Men Dun Jia has genuine school-to-school variation (拆補 chai-bu vs
//! 置閏 zhi-run reconciliation of the calendar, differing Heaven/Human-plate
//! conventions). This module implements the **San Yuan time-chart school
//! consistently**; it is faithful to that school, not a claim that every Qi Men
//! lineage assembles the chart identically. The Stars/Doors/Deities/Lo-Shu
//! reference data and the day/hour stem pillars are anchored to the same
//! sexagenary calendar as BaZi.

use super::HeavenlyStem;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// 9 Stars (九星 Jiu Xing)
// ---------------------------------------------------------------------------

/// The 9 Stars of Qi Men Dun Jia, mapped to the Lo Shu palaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Star {
    /// 天蓬 — Palace 1 (Water)
    TianPeng,
    /// 天芮 — Palace 2 (Earth)
    TianRui,
    /// 天冲 — Palace 3 (Wood)
    TianChong,
    /// 天辅 — Palace 4 (Wood)
    TianFu,
    /// 天禽 — Palace 5 (Earth, center)
    TianQin,
    /// 天心 — Palace 6 (Metal)
    TianXin,
    /// 天柱 — Palace 7 (Metal)
    TianZhu,
    /// 天任 — Palace 8 (Earth)
    TianRen,
    /// 天英 — Palace 9 (Fire)
    TianYing,
}

impl Star {
    pub const ALL: [Star; 9] = [
        Star::TianPeng,
        Star::TianRui,
        Star::TianChong,
        Star::TianFu,
        Star::TianQin,
        Star::TianXin,
        Star::TianZhu,
        Star::TianRen,
        Star::TianYing,
    ];

    /// Chinese name of the star.
    pub fn chinese_name(self) -> &'static str {
        match self {
            Star::TianPeng => "天蓬",
            Star::TianRui => "天芮",
            Star::TianChong => "天冲",
            Star::TianFu => "天辅",
            Star::TianQin => "天禽",
            Star::TianXin => "天心",
            Star::TianZhu => "天柱",
            Star::TianRen => "天任",
            Star::TianYing => "天英",
        }
    }

    /// English description.
    pub fn english_name(self) -> &'static str {
        match self {
            Star::TianPeng => "Heavenly Canopy",
            Star::TianRui => "Heavenly Stamen",
            Star::TianChong => "Heavenly Charge",
            Star::TianFu => "Heavenly Assistant",
            Star::TianQin => "Heavenly Bird",
            Star::TianXin => "Heavenly Heart",
            Star::TianZhu => "Heavenly Pillar",
            Star::TianRen => "Heavenly Appointment",
            Star::TianYing => "Heavenly Hero",
        }
    }

    /// Home Lo Shu palace number (1-9).
    pub fn home_palace(self) -> u8 {
        match self {
            Star::TianPeng => 1,
            Star::TianRui => 2,
            Star::TianChong => 3,
            Star::TianFu => 4,
            Star::TianQin => 5,
            Star::TianXin => 6,
            Star::TianZhu => 7,
            Star::TianRen => 8,
            Star::TianYing => 9,
        }
    }
}

// ---------------------------------------------------------------------------
// 8 Doors (八门 Ba Men)
// ---------------------------------------------------------------------------

/// The 8 Doors (八门) of Qi Men Dun Jia.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Door {
    /// 休门 — Rest, recuperation (Palace 1)
    Rest,
    /// 生门 — Life, growth (Palace 8)
    Life,
    /// 伤门 — Injury, harm (Palace 3)
    Injury,
    /// 杜门 — Obstruction, closing (Palace 4)
    Obstruction,
    /// 景门 — Scenery, vision (Palace 9)
    Scenery,
    /// 死门 — Death, ending (Palace 2)
    Death,
    /// 惊门 — Surprise, shock (Palace 7)
    Surprise,
    /// 开门 — Opening, beginning (Palace 6)
    Opening,
}

impl Door {
    pub const ALL: [Door; 8] = [
        Door::Rest,
        Door::Life,
        Door::Injury,
        Door::Obstruction,
        Door::Scenery,
        Door::Death,
        Door::Surprise,
        Door::Opening,
    ];

    /// Chinese name of the door.
    pub fn chinese_name(self) -> &'static str {
        match self {
            Door::Rest => "休门",
            Door::Life => "生门",
            Door::Injury => "伤门",
            Door::Obstruction => "杜门",
            Door::Scenery => "景门",
            Door::Death => "死门",
            Door::Surprise => "惊门",
            Door::Opening => "开门",
        }
    }

    /// Whether this door is generally auspicious.
    pub fn is_auspicious(self) -> bool {
        matches!(self, Door::Rest | Door::Life | Door::Opening)
    }

    /// Home Lo Shu palace number (1-9, excluding center 5).
    pub fn home_palace(self) -> u8 {
        match self {
            Door::Rest => 1,
            Door::Death => 2,
            Door::Injury => 3,
            Door::Obstruction => 4,
            Door::Scenery => 9,
            Door::Opening => 6,
            Door::Surprise => 7,
            Door::Life => 8,
        }
    }
}

// ---------------------------------------------------------------------------
// 8 Deities (八神 Ba Shen)
// ---------------------------------------------------------------------------

/// The 8 Deities (八神) of Qi Men Dun Jia.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Deity {
    /// 值符 — Chief (the presiding deity)
    ZhiFu,
    /// 腾蛇 — Soaring Serpent
    TengShe,
    /// 太阴 — Great Yin
    TaiYin,
    /// 六合 — Six Harmony
    LiuHe,
    /// 勾陈 / 白虎 — Hook Formation (Yang) / White Tiger (Yin)
    GouChen,
    /// 朱雀 / 玄武 — Vermillion Bird (Yang) / Dark Warrior (Yin)
    ZhuQue,
    /// 九地 — Nine Earth
    JiuDi,
    /// 九天 — Nine Heaven
    JiuTian,
}

impl Deity {
    pub const ALL: [Deity; 8] = [
        Deity::ZhiFu,
        Deity::TengShe,
        Deity::TaiYin,
        Deity::LiuHe,
        Deity::GouChen,
        Deity::ZhuQue,
        Deity::JiuDi,
        Deity::JiuTian,
    ];

    /// Chinese name of the deity.
    pub fn chinese_name(self) -> &'static str {
        match self {
            Deity::ZhiFu => "值符",
            Deity::TengShe => "腾蛇",
            Deity::TaiYin => "太阴",
            Deity::LiuHe => "六合",
            Deity::GouChen => "勾陈",
            Deity::ZhuQue => "朱雀",
            Deity::JiuDi => "九地",
            Deity::JiuTian => "九天",
        }
    }

    /// English description.
    pub fn english_name(self) -> &'static str {
        match self {
            Deity::ZhiFu => "Chief",
            Deity::TengShe => "Soaring Serpent",
            Deity::TaiYin => "Great Yin",
            Deity::LiuHe => "Six Harmony",
            Deity::GouChen => "Hook Formation",
            Deity::ZhuQue => "Vermillion Bird",
            Deity::JiuDi => "Nine Earth",
            Deity::JiuTian => "Nine Heaven",
        }
    }
}

// ---------------------------------------------------------------------------
// San Qi (三奇) and Liu Yi (六仪)
// ---------------------------------------------------------------------------

/// The 3 Qi (三奇 — the "strange doors") are the three Heavenly Stems
/// Yi (乙), Bing (丙), Ding (丁).
pub const SAN_QI: [HeavenlyStem; 3] = [HeavenlyStem::Yi, HeavenlyStem::Bing, HeavenlyStem::Ding];

/// The 6 Yi (六仪) are the six Heavenly Stems Wu through Gui.
pub const LIU_YI: [HeavenlyStem; 6] = [
    HeavenlyStem::Wu,
    HeavenlyStem::Ji,
    HeavenlyStem::Geng,
    HeavenlyStem::Xin,
    HeavenlyStem::Ren,
    HeavenlyStem::Gui,
];

// ---------------------------------------------------------------------------
// Lo Shu Magic Square
// ---------------------------------------------------------------------------

/// The Lo Shu magic square arrangement.
/// `LO_SHU[row][col]` gives the palace number.
///
/// ```text
///   4  9  2
///   3  5  7
///   8  1  6
/// ```
pub const LO_SHU: [[u8; 3]; 3] = [[4, 9, 2], [3, 5, 7], [8, 1, 6]];

/// The Lo Shu flight path (the order palaces are traversed):
/// 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9
/// This is the natural number order used for rotating stars/doors.
pub const LO_SHU_FLIGHT: [u8; 9] = [1, 2, 3, 4, 5, 6, 7, 8, 9];

// ---------------------------------------------------------------------------
// Palace
// ---------------------------------------------------------------------------

/// A single palace in the Qi Men chart.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Palace {
    /// Lo Shu palace number (1-9).
    pub number: u8,
    /// The star currently in this palace.
    pub star: Star,
    /// The door currently in this palace.
    pub door: Door,
    /// The deity currently in this palace.
    pub deity: Deity,
    /// The Heavenly Stem assigned to this palace (from the 3 Qi + 6 Yi).
    pub stem: HeavenlyStem,
}

// ---------------------------------------------------------------------------
// QiMenChart
// ---------------------------------------------------------------------------

/// A Qi Men Dun Jia time chart produced by [`compute_qimen`] (San Yuan
/// time-chart school — see the module documentation for scope).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QiMenChart {
    /// The 9 palaces, indexed 0-8 corresponding to palace numbers 1-9.
    pub palaces: [Palace; 9],
    /// The Heavenly Stem of the hour.
    pub hour_stem: HeavenlyStem,
    /// The Heavenly Stem of the day.
    pub day_stem: HeavenlyStem,
    /// The Ju (局) number, 1-18. 1-9 = Yang Dun, 10-18 = Yin Dun.
    pub ju_number: u8,
    /// Whether this is a Yang Dun (true) or Yin Dun (false) chart.
    pub is_yang_dun: bool,
}

// ---------------------------------------------------------------------------
// Computation
// ---------------------------------------------------------------------------

/// The 24 solar terms in calendar order **starting from Dong Zhi (Winter
/// Solstice, Sun at 270°)**, each with its San Yuan triplet
/// `(upper_yuan_ju, middle_yuan_ju, lower_yuan_ju)`.
///
/// Terms 0..12 (Dong Zhi → Mang Zhong) are **Yang Dun**; terms 12..24
/// (Xia Zhi → Da Xue) are **Yin Dun**. The Ju values are the canonical San Yuan
/// table used by the time-based school. This is reference data, not a
/// computation.
const SAN_YUAN_JU: [(u8, u8, u8); 24] = [
    (1, 7, 4), // 0  Dong Zhi (Winter Solstice)   — Yang
    (2, 8, 5), // 1  Xiao Han (Minor Cold)
    (3, 9, 6), // 2  Da Han (Major Cold)
    (8, 5, 2), // 3  Li Chun (Spring Begins)
    (9, 6, 3), // 4  Yu Shui (Rain Water)
    (1, 7, 4), // 5  Jing Zhe (Insects Awaken)
    (3, 9, 6), // 6  Chun Fen (Spring Equinox)
    (4, 1, 7), // 7  Qing Ming (Clear & Bright)
    (5, 2, 8), // 8  Gu Yu (Grain Rain)
    (4, 1, 7), // 9  Li Xia (Summer Begins)
    (5, 2, 8), // 10 Xiao Man (Grain Full)
    (6, 3, 9), // 11 Mang Zhong (Grain in Ear)
    (9, 3, 6), // 12 Xia Zhi (Summer Solstice)    — Yin
    (8, 2, 5), // 13 Xiao Shu (Minor Heat)
    (7, 1, 4), // 14 Da Shu (Major Heat)
    (2, 5, 8), // 15 Li Qiu (Autumn Begins)
    (1, 4, 7), // 16 Chu Shu (Heat Stops)
    (9, 3, 6), // 17 Bai Lu (White Dew)
    (7, 1, 4), // 18 Qiu Fen (Autumn Equinox)
    (6, 9, 3), // 19 Han Lu (Cold Dew)
    (5, 8, 2), // 20 Shuang Jiang (Frost Descends)
    (6, 9, 3), // 21 Li Dong (Winter Begins)
    (5, 8, 2), // 22 Xiao Xue (Minor Snow)
    (4, 7, 1), // 23 Da Xue (Major Snow)
];

/// Solar-term index `0..24` (Dong Zhi = 0) for a Julian Day, from the Sun's
/// apparent ecliptic longitude. Each term spans 15°; term 0 begins at 270°.
fn solar_term_index_dongzhi(jd: f64) -> usize {
    let lon = crate::solar_longitude_approx(jd);
    // Offset so Dong Zhi (270°) maps to index 0, wrapping through the year.
    let offset = (lon - 270.0).rem_euclid(360.0);
    ((offset / 15.0).floor() as usize) % 24
}

/// The San Yuan **yuan** (0 = upper 上元, 1 = middle 中元, 2 = lower 下元) for a
/// day pillar, via the Fu Tou (符头) rule.
///
/// Each 5-day "yuan" run begins on a day whose stem is Jia (甲) or Ji (己). The
/// branch of that run-head day chooses the yuan:
/// * Zi/Wu/Mao/You (子午卯酉) → upper,
/// * Yin/Shen/Si/Hai (寅申巳亥) → middle,
/// * Chen/Xu/Chou/Wei (辰戌丑未) → lower.
fn san_yuan(day_stem_idx: usize, day_branch_idx: usize) -> usize {
    // Days since the run head (the head has stem Jia=0 or Ji=5).
    let head_offset = day_stem_idx % 5;
    let head_branch = (day_branch_idx + 12 - (head_offset % 12)) % 12;
    // Branch groups by their index 0..11 (Zi=0 .. Hai=11).
    const UPPER: [usize; 4] = [0, 6, 3, 9]; // Zi, Wu, Mao, You
    const MIDDLE: [usize; 4] = [2, 8, 5, 11]; // Yin, Shen, Si, Hai
    if UPPER.contains(&head_branch) {
        0
    } else if MIDDLE.contains(&head_branch) {
        1
    } else {
        2 // Chen, Xu, Chou, Wei
    }
}

/// Determine the Qi Men Ju (局) for an instant, by the San Yuan time-chart rule.
///
/// The Ju is read from [`SAN_YUAN_JU`] at the instant's solar term, choosing the
/// upper/middle/lower-yuan column via the day pillar's Fu Tou ([`san_yuan`]).
/// The returned value is stored 1–9 for **Yang Dun** (terms Dong Zhi → Mang
/// Zhong) and 10–18 for **Yin Dun** (offset by 9, terms Xia Zhi → Da Xue), so a
/// single `u8` carries both the Ju number and the dun polarity — matching
/// [`is_yang_dun`] / [`jia_palace`].
///
/// `jd` is the Julian Day of the instant; `day_stem_idx` / `day_branch_idx` are
/// the day pillar's stem (0..9) and branch (0..11) indices (e.g. from
/// `sexagenary_day`).
pub fn qimen_ju(jd: f64, day_stem_idx: usize, day_branch_idx: usize) -> u8 {
    let term = solar_term_index_dongzhi(jd);
    let yuan = san_yuan(day_stem_idx, day_branch_idx);
    let (u, m, l) = SAN_YUAN_JU[term];
    let ju_value = [u, m, l][yuan];
    let is_yang = term < 12;
    if is_yang { ju_value } else { ju_value + 9 }
}

/// Whether a Ju number represents Yang Dun (forward flight).
pub fn is_yang_dun(ju: u8) -> bool {
    ju <= 9
}

/// The base palace where Jia (甲) hides, determined by the Ju number.
///
/// In Yang Dun, Ju N means Jia hides under the Yi of palace N.
/// In Yin Dun, Ju 10+N means Jia hides in a reverse arrangement.
fn jia_palace(ju: u8) -> u8 {
    if ju <= 9 {
        ju
    } else {
        // Yin Dun: 10->9, 11->8, 12->7, 13->6, 14->5, 15->4, 16->3, 17->2, 18->1
        9 - (ju - 10)
    }
}

/// The 9 stems in their cycle order used for palace assignment.
/// The 3 Qi + 6 Yi form the sequence: Wu, Ji, Geng, Xin, Ren, Gui, Yi, Bing, Ding
/// (6 Yi first, then 3 Qi — this is the standard Qi Men stem order).
const STEM_ORDER: [HeavenlyStem; 9] = [
    HeavenlyStem::Wu,   // 戊 — Yi 1
    HeavenlyStem::Ji,   // 己 — Yi 2
    HeavenlyStem::Geng, // 庚 — Yi 3
    HeavenlyStem::Xin,  // 辛 — Yi 4
    HeavenlyStem::Ren,  // 壬 — Yi 5
    HeavenlyStem::Gui,  // 癸 — Yi 6
    HeavenlyStem::Yi,   // 乙 — Qi 1
    HeavenlyStem::Bing, // 丙 — Qi 2
    HeavenlyStem::Ding, // 丁 — Qi 3
];

/// The Lo Shu traversal order for Yang Dun (forward flight):
/// 1 -> 8 -> 3 -> 4 -> 9 -> 2 -> 7 -> 6 (skipping center 5).
const YANG_FLIGHT: [u8; 8] = [1, 8, 3, 4, 9, 2, 7, 6];

/// The Lo Shu traversal order for Yin Dun (reverse flight):
/// 9 -> 2 -> 7 -> 6 -> 1 -> 8 -> 3 -> 4 (skipping center 5).
const YIN_FLIGHT: [u8; 8] = [9, 2, 7, 6, 1, 8, 3, 4];

/// The Six Yi stem (六仪) that hides the Jia of each of the six Xun (旬).
///
/// Indexed by Xun head: Jia-Zi→Wu, Jia-Xu→Ji, Jia-Shen→Geng, Jia-Wu→Xin,
/// Jia-Chen→Ren, Jia-Yin→Gui. The Xun head index is `(pillar_index / 10)` for a
/// sexagenary pillar `0..60`.
const XUN_HEAD_YI: [HeavenlyStem; 6] = [
    HeavenlyStem::Wu,   // Jia-Zi
    HeavenlyStem::Ji,   // Jia-Xu
    HeavenlyStem::Geng, // Jia-Shen
    HeavenlyStem::Xin,  // Jia-Wu
    HeavenlyStem::Ren,  // Jia-Chen
    HeavenlyStem::Gui,  // Jia-Yin
];

/// Combine a stem (0..9) and branch (0..11) into the sexagenary index 0..59.
///
/// The pair occurs exactly once in the 60-cycle; the index is recovered by the
/// Chinese-remainder step `n ≡ stem (mod 10)`, `n ≡ branch (mod 12)`.
fn sexagenary_index(stem: usize, branch: usize) -> usize {
    // n = stem + 10*k with (stem + 10*k) % 12 == branch.
    let mut n = stem;
    while n % 12 != branch {
        n += 10;
    }
    n % 60
}

/// The full Heaven-Plate star order, used when the Zhi Fu star leads the
/// rotation. Palace-home stars in Lo Shu palace order 1..9 (center = TianQin).
const HOME_STARS: [Star; 9] = [
    Star::TianPeng,  // palace 1
    Star::TianRui,   // palace 2
    Star::TianChong, // palace 3
    Star::TianFu,    // palace 4
    Star::TianQin,   // palace 5 (center)
    Star::TianXin,   // palace 6
    Star::TianZhu,   // palace 7
    Star::TianRen,   // palace 8
    Star::TianYing,  // palace 9
];

/// Home doors in Lo Shu palace order 1..9 (palace 5 has no door — `None`).
const HOME_DOORS: [Option<Door>; 9] = [
    Some(Door::Rest),        // palace 1
    Some(Door::Death),       // palace 2
    Some(Door::Injury),      // palace 3
    Some(Door::Obstruction), // palace 4
    None,                    // palace 5 (center, no door)
    Some(Door::Opening),     // palace 6
    Some(Door::Surprise),    // palace 7
    Some(Door::Life),        // palace 8
    Some(Door::Scenery),     // palace 9
];

/// Compute a time-based Qi Men Dun Jia chart by the San Yuan placement school.
///
/// The pipeline is: (1) [`qimen_ju`] sets the Ju from the solar term + San Yuan
/// Fu Tou yuan; (2) the Earth-Plate stems are placed from the Jia palace along
/// the Yang/Yin flight; (3) the hour's Xun-head Yi locates the Zhi Fu palace;
/// (4) the Zhi Fu star, Zhi Shi door, and the eight Deities are seated led by
/// that palace and flown along the dun direction. See the module documentation
/// for the school this implements and its honest scope.
pub fn compute_qimen(year: i32, month: u32, day: u32, hour: u32) -> QiMenChart {
    // Day/hour pillars from the shared sexagenary calendar (continuous, anchored
    // to 2000-01-01 = Wu-Wu — no fixed-30-day-month drift).
    let jd = crate::gregorian_to_jd(year, month, day);
    let day_pillar = crate::sexagenary_day(jd);
    let day_stem = day_pillar.stem;
    let h_branch = crate::hour_branch(hour as f64);
    let hour_stem = crate::hour_stem(day_stem, h_branch);

    // Ju by the canonical San Yuan rule (solar term + Fu Tou yuan).
    let ju = qimen_ju(jd, day_stem as usize, day_pillar.branch as usize);
    let yang = is_yang_dun(ju);
    let base_palace = jia_palace(ju);
    let flight = if yang { &YANG_FLIGHT } else { &YIN_FLIGHT };

    // (2) Earth-Plate stems: Wu starts at the Jia palace, the rest follow the
    // flight path; the center palace takes the 9th stem (Ding).
    let mut stem_placement: [HeavenlyStem; 9] = [HeavenlyStem::Jia; 9];
    let start_idx = flight.iter().position(|&p| p == base_palace).unwrap_or(0);
    for i in 0..8 {
        let palace_num = flight[(start_idx + i) % 8];
        stem_placement[(palace_num - 1) as usize] = STEM_ORDER[i];
    }
    stem_placement[4] = STEM_ORDER[8];

    // (3) Zhi Fu palace: the palace where the hour's Xun-head Yi sits on the
    // Earth Plate. The hour Xun head comes from the hour pillar's sexagenary
    // index. (The hour pillar repeats every 60 double-hours; its Xun head Yi is
    // what "leads" the chart.)
    let hour_pillar_idx = sexagenary_index(hour_stem as usize, h_branch as usize);
    let xun_head_yi = XUN_HEAD_YI[hour_pillar_idx / 10];
    let zhifu_palace = stem_placement
        .iter()
        .position(|&s| s == xun_head_yi)
        .map(|i| (i + 1) as u8)
        .unwrap_or(base_palace);

    // (4a) Heaven-Plate stars led by the Zhi Fu star: the star whose home palace
    // is the Zhi Fu palace leads, and the nine stars rotate with the flight so
    // the Zhi Fu star sits over the hour's leading stem. The flight loop below
    // carries every home star generically, so the Zhi Fu star is placed by it.
    let mut star_placement: [Star; 9] = Star::ALL;
    star_placement[4] = Star::TianQin; // center never moves
    // Outer-palace flight: the Zhi Fu star is carried to the palace of the hour
    // stem's Earth-Plate position; the rest keep their relative order on the
    // flight ring.
    let hour_stem_palace = stem_placement
        .iter()
        .position(|&s| s == hour_stem)
        .map(|i| (i + 1) as u8)
        .unwrap_or(zhifu_palace);
    let zhifu_ring = flight.iter().position(|&p| p == zhifu_palace).unwrap_or(0);
    let target_ring = flight
        .iter()
        .position(|&p| p == hour_stem_palace)
        .unwrap_or(zhifu_ring);
    let star_shift = (target_ring + 8 - zhifu_ring) % 8;
    for src in 0..8 {
        let src_palace = flight[src];
        let dst_palace = flight[(src + star_shift) % 8];
        star_placement[(dst_palace - 1) as usize] = HOME_STARS[(src_palace - 1) as usize];
    }

    // (4b) Zhi Shi door: the home door of the Zhi Fu palace leads, the eight
    // doors flying along the same shift.
    let mut door_placement: [Door; 9] = [Door::Rest; 9];
    for src in 0..8 {
        let src_palace = flight[src];
        let dst_palace = flight[(src + star_shift) % 8];
        if let Some(d) = HOME_DOORS[(src_palace - 1) as usize] {
            door_placement[(dst_palace - 1) as usize] = d;
        }
    }
    // The center palace keeps its (door-less) slot filled with the Zhi Shi door
    // so the array is total; UIs ignore the center door.
    let zhishi_door = HOME_DOORS[(zhifu_palace - 1) as usize].unwrap_or(Door::Rest);
    door_placement[4] = zhishi_door;

    // (4c) Deities led by Zhi Fu at its palace, flown along the dun direction
    // (forward for Yang, reverse for Yin) around the eight outer palaces.
    let mut deity_placement: [Deity; 9] = [Deity::ZhiFu; 9];
    let zhifu_deity_ring = flight.iter().position(|&p| p == zhifu_palace).unwrap_or(0);
    for (i, &deity) in Deity::ALL.iter().enumerate() {
        let ring = if yang {
            (zhifu_deity_ring + i) % 8
        } else {
            (zhifu_deity_ring + 8 - i) % 8
        };
        let palace = flight[ring];
        deity_placement[(palace - 1) as usize] = deity;
    }
    deity_placement[4] = Deity::JiuDi; // center default (no deity flies through 5)

    let palaces = std::array::from_fn(|i| Palace {
        number: (i + 1) as u8,
        star: star_placement[i],
        door: door_placement[i],
        deity: deity_placement[i],
        stem: stem_placement[i],
    });

    QiMenChart {
        palaces,
        hour_stem,
        day_stem,
        ju_number: ju,
        is_yang_dun: yang,
    }
}

impl QiMenChart {
    /// The Lo Shu palace (1..9) hosting the Zhi Fu (值符) star — the chart's
    /// presiding deity/star, anchored to the hour's Xun-head Yi.
    pub fn zhi_fu_palace(&self) -> u8 {
        self.palaces
            .iter()
            .find(|p| p.deity == Deity::ZhiFu)
            .map(|p| p.number)
            .unwrap_or(5)
    }
}

/// Check whether a palace has a favorable combination:
/// auspicious door + Qi stem (Yi, Bing, or Ding).
pub fn is_favorable_palace(palace: &Palace) -> bool {
    let has_qi = matches!(
        palace.stem,
        HeavenlyStem::Yi | HeavenlyStem::Bing | HeavenlyStem::Ding
    );
    palace.door.is_auspicious() && has_qi
}

/// Verify the Lo Shu magic square property: all rows, columns, and
/// diagonals sum to 15.
pub fn verify_lo_shu() -> bool {
    // Rows
    for row in &LO_SHU {
        let sum: u8 = row.iter().sum();
        if sum != 15 {
            return false;
        }
    }
    // Columns
    for col in 0..3 {
        let sum: u8 = LO_SHU.iter().map(|row| row[col]).sum();
        if sum != 15 {
            return false;
        }
    }
    // Diagonals
    let diag1: u8 = (0..3).map(|i| LO_SHU[i][i]).sum();
    let diag2: u8 = (0..3).map(|i| LO_SHU[i][2 - i]).sum();
    diag1 == 15 && diag2 == 15
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lo_shu_is_magic_square() {
        assert!(verify_lo_shu(), "Lo Shu must sum to 15 in all directions");
    }

    #[test]
    fn lo_shu_contains_1_to_9() {
        let mut nums: Vec<u8> = LO_SHU.iter().flat_map(|row| row.iter().copied()).collect();
        nums.sort();
        assert_eq!(nums, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn all_9_stars_present() {
        assert_eq!(Star::ALL.len(), 9);
        for (i, star) in Star::ALL.iter().enumerate() {
            assert_eq!(star.home_palace(), (i + 1) as u8);
        }
    }

    #[test]
    fn star_names() {
        assert_eq!(Star::TianPeng.chinese_name(), "天蓬");
        assert_eq!(Star::TianXin.english_name(), "Heavenly Heart");
        assert_eq!(Star::TianYing.chinese_name(), "天英");
    }

    #[test]
    fn all_8_doors_present() {
        assert_eq!(Door::ALL.len(), 8);
    }

    #[test]
    fn door_auspicious() {
        assert!(Door::Rest.is_auspicious());
        assert!(Door::Life.is_auspicious());
        assert!(Door::Opening.is_auspicious());
        assert!(!Door::Death.is_auspicious());
        assert!(!Door::Injury.is_auspicious());
        assert!(!Door::Surprise.is_auspicious());
    }

    #[test]
    fn door_names() {
        assert_eq!(Door::Rest.chinese_name(), "休门");
        assert_eq!(Door::Opening.chinese_name(), "开门");
        assert_eq!(Door::Scenery.chinese_name(), "景门");
    }

    #[test]
    fn all_8_deities_present() {
        assert_eq!(Deity::ALL.len(), 8);
    }

    #[test]
    fn deity_names() {
        assert_eq!(Deity::ZhiFu.chinese_name(), "值符");
        assert_eq!(Deity::ZhiFu.english_name(), "Chief");
        assert_eq!(Deity::JiuTian.chinese_name(), "九天");
    }

    #[test]
    fn san_qi_are_yi_bing_ding() {
        assert_eq!(
            SAN_QI,
            [HeavenlyStem::Yi, HeavenlyStem::Bing, HeavenlyStem::Ding]
        );
    }

    #[test]
    fn liu_yi_are_wu_through_gui() {
        assert_eq!(LIU_YI.len(), 6);
        assert_eq!(LIU_YI[0], HeavenlyStem::Wu);
        assert_eq!(LIU_YI[5], HeavenlyStem::Gui);
    }

    // San Yuan Ju determination (solar term + Fu Tou yuan). Oracle values are
    // computed from the same Meeus solar-longitude approximation + noon-JDN day
    // pillar the crate uses, mirrored in Python.
    #[test]
    fn qimen_ju_winter_terms_are_yang_dun() {
        // Around the winter solstice the chart is Yang Dun (Ju 1..9).
        let jd = crate::gregorian_to_jd(2024, 12, 22); // Dong Zhi area
        let dp = crate::sexagenary_day(jd);
        let ju = qimen_ju(jd, dp.stem as usize, dp.branch as usize);
        assert!(is_yang_dun(ju), "winter solstice → Yang Dun, got Ju {ju}");
        assert_eq!(ju, 4, "2024-12-22 = Dong Zhi lower-yuan → Ju 4");
    }

    #[test]
    fn qimen_ju_summer_terms_are_yin_dun() {
        // Just after the summer solstice the chart is Yin Dun (stored 10..18).
        let jd = crate::gregorian_to_jd(2024, 7, 15); // Xiao Shu area
        let dp = crate::sexagenary_day(jd);
        let ju = qimen_ju(jd, dp.stem as usize, dp.branch as usize);
        assert!(!is_yang_dun(ju), "summer → Yin Dun, got Ju {ju}");
        assert_eq!(
            ju, 17,
            "2024-07-15 = Xiao Shu upper-yuan → Yin Ju 8 (stored 17)"
        );
    }

    #[test]
    fn qimen_ju_known_anchors() {
        // 2000-01-01: Xiao Han, middle yuan → Ju 7 (Yang).
        let jd0 = crate::gregorian_to_jd(2000, 1, 1);
        let d0 = crate::sexagenary_day(jd0);
        assert_eq!(qimen_ju(jd0, d0.stem as usize, d0.branch as usize), 7);
        // 2024-06-15: Mang Zhong, upper yuan → Ju 6 (Yang).
        let jd1 = crate::gregorian_to_jd(2024, 6, 15);
        let d1 = crate::sexagenary_day(jd1);
        assert_eq!(qimen_ju(jd1, d1.stem as usize, d1.branch as usize), 6);
    }

    #[test]
    fn san_yuan_fu_tou_groups() {
        // Jia-Zi (stem 0, branch 0) is a run head with branch Zi → upper (0).
        assert_eq!(san_yuan(0, 0), 0);
        // Ji-Si (stem 5, branch 5) is a run head with branch Si → middle (1).
        assert_eq!(san_yuan(5, 5), 1);
        // Jia-Chen would be lower, but Jia pairs with Zi-group branches; use the
        // run-head reduction: Ding-Wei (stem 3, branch 7) reduces to head branch
        // (7 - 3) = Chen (4) → lower (2).
        assert_eq!(san_yuan(3, 7), 2);
        // Every (stem, branch) yields a valid yuan in {0,1,2}.
        for s in 0..10usize {
            for b in 0..12usize {
                assert!(san_yuan(s, b) <= 2);
            }
        }
    }

    #[test]
    fn compute_qimen_produces_9_palaces() {
        let chart = compute_qimen(2024, 6, 15, 10);
        assert_eq!(chart.palaces.len(), 9);
        for (i, p) in chart.palaces.iter().enumerate() {
            assert_eq!(p.number, (i + 1) as u8, "Palace {} has wrong number", i);
        }
    }

    #[test]
    fn qimen_day_stem_is_continuous_and_anchored() {
        use crate::HeavenlyStem;

        // Anchor: 2000-01-01 = Wu day (index 4), matching the
        // sexagenary day pillar (same anchor as BaZi). The old fake fixed-30-day
        // calendar produced a wrong, discontinuous day stem here.
        assert_eq!(
            compute_qimen(2000, 1, 1, 12).day_stem,
            HeavenlyStem::from_index(4),
            "2000-01-01 must anchor to Wu day stem"
        );

        // Continuity across a LEAP-YEAR February boundary (Feb 28 -> 29 -> Mar 1):
        // each consecutive civil day advances the day stem by exactly 1 (mod 10).
        // This is the 366-day case the previous fake calendar got wrong.
        let leap_run = [(2000u32, 2u32, 28u32), (2000, 2, 29), (2000, 3, 1)];
        for w in leap_run.windows(2) {
            let (y0, m0, d0) = w[0];
            let (y1, m1, d1) = w[1];
            let s0 = compute_qimen(y0 as i32, m0, d0, 12).day_stem as usize;
            let s1 = compute_qimen(y1 as i32, m1, d1, 12).day_stem as usize;
            assert_eq!(
                (s0 + 1) % 10,
                s1,
                "day stem must advance by 1 from {y0}-{m0}-{d0} to {y1}-{m1}-{d1}"
            );
        }

        // Continuity across the YEAR boundary (2023-12-31 -> 2024-01-01):
        // Gui (9) -> Jia (0), a +1 wrap. The old per-year 365/366-day jump
        // broke exactly here.
        let dec31 = compute_qimen(2023, 12, 31, 12).day_stem as usize;
        let jan01 = compute_qimen(2024, 1, 1, 12).day_stem as usize;
        assert_eq!(
            (dec31 + 1) % 10,
            jan01,
            "day stem must advance across new year"
        );
        assert_eq!(
            compute_qimen(2024, 1, 1, 12).day_stem,
            HeavenlyStem::from_index(0)
        );

        // Hour stem is anchored to the day stem via the standard Five-Rats rule.
        // 2000-01-01 is a Wu day (base = (4 % 5) * 2 = 8); the Zi double-hour
        // (hour 0) therefore yields Ren (index 8), and noon (Wu hour) yields Wu (4).
        let day = compute_qimen(2000, 1, 1, 0);
        assert_eq!(
            day.hour_stem,
            HeavenlyStem::from_index(8),
            "Zi hour stem on Wu day = Ren"
        );
        assert_eq!(
            compute_qimen(2000, 1, 1, 12).hour_stem,
            HeavenlyStem::from_index(4),
            "Wu hour (noon) stem on Wu day = Wu"
        );

        // Determinism is preserved.
        let c1 = compute_qimen(2024, 3, 20, 8);
        let c2 = compute_qimen(2024, 3, 20, 8);
        assert_eq!(c1.day_stem, c2.day_stem);
        assert_eq!(c1.hour_stem, c2.hour_stem);
    }

    #[test]
    fn compute_qimen_different_seasons_differ_in_dun() {
        // A pre-summer-solstice date is Yang Dun; a post-summer-solstice date is
        // Yin Dun. This now follows from the real solar-term Ju determination.
        let yang = compute_qimen(2024, 1, 15, 10); // Xiao Han → Yang
        let yin = compute_qimen(2024, 7, 15, 10); // Xiao Shu → Yin
        assert!(yang.is_yang_dun);
        assert!(!yin.is_yang_dun);
        assert_ne!(yang.is_yang_dun, yin.is_yang_dun);
    }

    #[test]
    fn earth_plate_stems_are_the_nine_distinct_stems() {
        // The Earth Plate seats all six Yi + three Qi, each exactly once.
        let chart = compute_qimen(2024, 6, 15, 10);
        let mut stems: Vec<usize> = chart.palaces.iter().map(|p| p.stem as usize).collect();
        stems.sort_unstable();
        assert_eq!(
            stems,
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
            "Earth Plate must seat the nine stems Yi..Gui+Bing+Ding exactly once"
        );
    }

    #[test]
    fn all_eight_deities_seated_and_zhifu_consistent() {
        let chart = compute_qimen(2024, 6, 15, 10);
        // Every one of the eight outer deities appears.
        for deity in Deity::ALL {
            assert!(
                chart.palaces.iter().any(|p| p.deity == deity),
                "deity {:?} must be seated",
                deity.english_name()
            );
        }
        // The Zhi Fu palace is the palace seating the Zhi Fu deity and lies on
        // an outer (non-center) palace.
        let zf = chart.zhi_fu_palace();
        assert!((1..=9).contains(&zf));
        assert_ne!(zf, 5, "Zhi Fu never seats in the center palace");
        assert_eq!(
            chart.palaces[(zf - 1) as usize].deity,
            Deity::ZhiFu,
            "zhi_fu_palace() must point at the ZhiFu seat"
        );
    }

    /// Zhi Fu is anchored to the hour's Xun-head Yi (旬首): the Zhi Fu palace is
    /// where that Six-Yi stem sits on the Earth Plate. Verified directly from the
    /// hour pillar, not via an `hour % 8` rotation.
    #[test]
    fn zhifu_anchored_to_xun_head_yi() {
        let year = 2024;
        let month = 6;
        let day = 15;
        for hour in [0u32, 2, 6, 10, 14, 18, 22] {
            let chart = compute_qimen(year, month, day, hour);
            let jd = crate::gregorian_to_jd(year, month, day);
            let day_stem = crate::sexagenary_day(jd).stem;
            let h_branch = crate::hour_branch(hour as f64);
            let hour_stem = crate::hour_stem(day_stem, h_branch);
            let idx = sexagenary_index(hour_stem as usize, h_branch as usize);
            let xun_yi = XUN_HEAD_YI[idx / 10];
            let zf = chart.zhi_fu_palace();
            assert_eq!(
                chart.palaces[(zf - 1) as usize].stem,
                xun_yi,
                "hour {hour}: Zhi Fu palace must seat the Xun-head Yi {:?}",
                xun_yi
            );
        }
    }

    #[test]
    fn compute_qimen_is_deterministic() {
        let a = compute_qimen(2024, 6, 15, 10);
        let b = compute_qimen(2024, 6, 15, 10);
        assert_eq!(a.ju_number, b.ju_number);
        assert_eq!(a.zhi_fu_palace(), b.zhi_fu_palace());
        for i in 0..9 {
            assert_eq!(a.palaces[i].star, b.palaces[i].star);
            assert_eq!(a.palaces[i].door, b.palaces[i].door);
            assert_eq!(a.palaces[i].deity, b.palaces[i].deity);
            assert_eq!(a.palaces[i].stem, b.palaces[i].stem);
        }
    }

    #[test]
    fn favorable_palace_check() {
        let favorable = Palace {
            number: 1,
            star: Star::TianPeng,
            door: Door::Opening,
            deity: Deity::ZhiFu,
            stem: HeavenlyStem::Yi,
        };
        assert!(is_favorable_palace(&favorable));

        let unfavorable = Palace {
            number: 2,
            star: Star::TianRui,
            door: Door::Death,
            deity: Deity::TengShe,
            stem: HeavenlyStem::Geng,
        };
        assert!(!is_favorable_palace(&unfavorable));
    }

    #[test]
    fn jia_palace_yang_dun() {
        assert_eq!(jia_palace(1), 1);
        assert_eq!(jia_palace(5), 5);
        assert_eq!(jia_palace(9), 9);
    }

    #[test]
    fn jia_palace_yin_dun() {
        assert_eq!(jia_palace(10), 9);
        assert_eq!(jia_palace(14), 5);
        assert_eq!(jia_palace(18), 1);
    }
}
