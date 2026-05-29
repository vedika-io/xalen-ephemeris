//! # Qi Men Dun Jia (奇门遁甲) — Strategic Divination
//!
//! Qi Men Dun Jia ("Mysterious Gates Escaping Technique") is one of the
//! three supreme arts of Chinese metaphysics. It uses a 9-palace grid based
//! on the Lo Shu magic square, combined with 8 Doors, 8 Deities, 3 Qi
//! (Heavenly Stems Yi/Bing/Ding), and 6 Yi (Wu/Ji/Geng/Xin/Ren/Gui).
//!
//! This module provides the foundational structures and a simplified
//! computation that maps date/time to a Qi Men chart.

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

/// A complete Qi Men Dun Jia chart.
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

/// Determine the Ju number from the month (simplified solar-term mapping).
///
/// In full Qi Men Dun Jia, the Ju is determined by the exact solar term
/// and whether we are in a Yang Dun (winter solstice to summer solstice)
/// or Yin Dun (summer solstice to winter solstice) period. This simplified
/// version maps months to approximate Ju numbers.
///
/// - Months 11, 12, 1, 2, 3, 4, 5 (winter half) -> Yang Dun (Ju 1-9)
/// - Months 6, 7, 8, 9, 10 (summer half) -> Yin Dun (Ju 10-18)
pub fn ju_from_month(month: u32) -> u8 {
    match month {
        12 | 1 => 1, // Winter Solstice area — Yang Dun Ju 1
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
        6 => 10, // Summer Solstice area — Yin Dun Ju 1 (stored as 10)
        7 => 11,
        8 => 12,
        9 => 13,
        10 => 14,
        11 => 6, // Late autumn, still Yang Dun
        _ => 1,  // fallback
    }
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

/// Compute a simplified Qi Men Dun Jia chart.
///
/// This uses a month-based Ju determination and simplified stem/star/door/deity
/// placement. For production-grade charts, the exact solar term boundaries
/// and hour-pillar computations are needed.
pub fn compute_qimen(year: i32, month: u32, day: u32, hour: u32) -> QiMenChart {
    let ju = ju_from_month(month);
    let yang = is_yang_dun(ju);
    let base_palace = jia_palace(ju);

    // Determine day and hour stems using a simplified sexagenary calculation.
    // Day stem: continuous count from a known epoch (same as BaZi day pillar).
    // We use a simplified Julian day approximation.
    let approx_day_count =
        year as i64 * 365 + (year as i64 / 4) + (month as i64 - 1) * 30 + day as i64;
    let day_stem_idx = (approx_day_count.rem_euclid(10)) as usize;
    let day_stem = HeavenlyStem::from_index(day_stem_idx);

    // Hour stem derived from day stem and hour.
    let hour_branch_idx = hour.div_ceil(2) as usize % 12;
    let hour_stem_base = (day_stem_idx % 5) * 2;
    let hour_stem = HeavenlyStem::from_index(hour_stem_base + hour_branch_idx);

    // Determine rotation offset from hour.
    let rotation = (hour / 2) as usize % 8;

    // Select flight path.
    let flight = if yang { &YANG_FLIGHT } else { &YIN_FLIGHT };

    // Place stems on palaces: stem[0] (Wu) goes to base_palace,
    // then follow flight path for remaining stems.
    let mut stem_placement: [HeavenlyStem; 9] = [HeavenlyStem::Jia; 9];
    // Find starting index in flight path.
    let start_idx = flight.iter().position(|&p| p == base_palace).unwrap_or(0);

    // Place the first 8 stems on the 8 outer palaces.
    for i in 0..8 {
        let flight_idx = (start_idx + i) % 8;
        let palace_num = flight[flight_idx];
        stem_placement[(palace_num - 1) as usize] = STEM_ORDER[i];
    }
    // Palace 5 (center) gets the 9th stem (Ding).
    stem_placement[4] = STEM_ORDER[8];

    // Place stars: rotate from home positions by the rotation offset.
    let mut star_placement: [Star; 9] = Star::ALL;
    // Center star (TianQin) stays in palace 5; rotate the other 8.
    let outer_stars = [
        Star::TianPeng,
        Star::TianRui,
        Star::TianChong,
        Star::TianFu,
        Star::TianXin,
        Star::TianZhu,
        Star::TianRen,
        Star::TianYing,
    ];
    for (i, &star) in outer_stars.iter().enumerate() {
        let target_flight_idx = (i + rotation) % 8;
        let target_palace = flight[target_flight_idx];
        star_placement[(target_palace - 1) as usize] = star;
    }
    star_placement[4] = Star::TianQin; // center stays

    // Place doors: rotate similarly.
    let mut door_placement: [Door; 9] = [
        Door::Rest,
        Door::Death,
        Door::Injury,
        Door::Obstruction,
        Door::Rest, // placeholder for center (no door)
        Door::Opening,
        Door::Surprise,
        Door::Life,
        Door::Scenery,
    ];
    for (i, &door) in Door::ALL.iter().enumerate() {
        let target_flight_idx = (i + rotation) % 8;
        let target_palace = flight[target_flight_idx];
        door_placement[(target_palace - 1) as usize] = door;
    }
    // Center gets whichever door lands there through rotation (keep as-is).

    // Place deities: Zhi Fu follows the hour stem position, others follow in order.
    let mut deity_placement: [Deity; 9] = [Deity::ZhiFu; 9];
    let zhifu_palace_idx = (hour as usize) % 8;
    for (i, &deity) in Deity::ALL.iter().enumerate() {
        let target_flight_idx = (zhifu_palace_idx + i) % 8;
        let target_palace = flight[target_flight_idx];
        deity_placement[(target_palace - 1) as usize] = deity;
    }
    // Center gets JiuDi as default.
    deity_placement[4] = Deity::JiuDi;

    // Build palaces.
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

    #[test]
    fn ju_from_month_winter_is_yang() {
        assert!(is_yang_dun(ju_from_month(12)));
        assert!(is_yang_dun(ju_from_month(1)));
        assert!(is_yang_dun(ju_from_month(2)));
        assert!(is_yang_dun(ju_from_month(3)));
    }

    #[test]
    fn ju_from_month_summer_is_yin() {
        assert!(!is_yang_dun(ju_from_month(6)));
        assert!(!is_yang_dun(ju_from_month(7)));
        assert!(!is_yang_dun(ju_from_month(8)));
        assert!(!is_yang_dun(ju_from_month(9)));
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
    fn compute_qimen_deterministic() {
        let c1 = compute_qimen(2024, 3, 20, 8);
        let c2 = compute_qimen(2024, 3, 20, 8);
        assert_eq!(c1.ju_number, c2.ju_number);
        assert_eq!(c1.is_yang_dun, c2.is_yang_dun);
        assert_eq!(c1.day_stem, c2.day_stem);
        assert_eq!(c1.hour_stem, c2.hour_stem);
        for i in 0..9 {
            assert_eq!(c1.palaces[i].star, c2.palaces[i].star);
            assert_eq!(c1.palaces[i].door, c2.palaces[i].door);
            assert_eq!(c1.palaces[i].stem, c2.palaces[i].stem);
        }
    }

    #[test]
    fn compute_qimen_different_months_differ() {
        let c1 = compute_qimen(2024, 1, 15, 10);
        let c2 = compute_qimen(2024, 7, 15, 10);
        assert_ne!(
            c1.is_yang_dun, c2.is_yang_dun,
            "January (Yang) and July (Yin) should differ"
        );
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
