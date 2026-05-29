use super::{EarthlyBranch, HeavenlyStem, WuXing};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MajorStar {
    ZiWei,     // Emperor
    TianJi,    // Heavenly Secret
    TaiYang,   // Sun
    WuQu,      // Military Music
    TianTong,  // Heavenly Unity
    LianZhen,  // Purity
    TianFu,    // Heavenly Treasury
    TaiYin,    // Moon
    TanLang,   // Greedy Wolf
    JuMen,     // Giant Gate
    TianXiang, // Heavenly Minister
    TianLiang, // Heavenly Beam
    QiSha,     // Seven Killings
    PoJun,     // Broken Army
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Palace {
    Life,      // 命宮
    Siblings,  // 兄弟
    Spouse,    // 夫妻
    Children,  // 子女
    Wealth,    // 財帛
    Health,    // 疾厄
    Travel,    // 遷移
    Friends,   // 交友
    Career,    // 事業
    Property,  // 田宅
    Happiness, // 福德
    Parents,   // 父母
}

impl Palace {
    pub const ALL: [Palace; 12] = [
        Palace::Life,
        Palace::Siblings,
        Palace::Spouse,
        Palace::Children,
        Palace::Wealth,
        Palace::Health,
        Palace::Travel,
        Palace::Friends,
        Palace::Career,
        Palace::Property,
        Palace::Happiness,
        Palace::Parents,
    ];

    pub fn chinese_name(&self) -> &'static str {
        match self {
            Palace::Life => "命宮",
            Palace::Siblings => "兄弟宮",
            Palace::Spouse => "夫妻宮",
            Palace::Children => "子女宮",
            Palace::Wealth => "財帛宮",
            Palace::Health => "疾厄宮",
            Palace::Travel => "遷移宮",
            Palace::Friends => "交友宮",
            Palace::Career => "事業宮",
            Palace::Property => "田宅宮",
            Palace::Happiness => "福德宮",
            Palace::Parents => "父母宮",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StarBrightness {
    Miao,
    Wang,
    De,
    Li,
    Ping,
    Xian,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZiWeiChart {
    pub palaces: [PalaceInfo; 12],
    pub ming_gong_branch: EarthlyBranch,
    pub shen_gong_branch: EarthlyBranch,
    pub wu_xing_ju: WuXingJu,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PalaceInfo {
    pub palace: Palace,
    pub branch: EarthlyBranch,
    pub major_stars: Vec<MajorStar>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WuXingJu {
    Water2,
    Wood3,
    Metal4,
    Earth5,
    Fire6,
}

impl WuXingJu {
    pub fn value(&self) -> u32 {
        match self {
            WuXingJu::Water2 => 2,
            WuXingJu::Wood3 => 3,
            WuXingJu::Metal4 => 4,
            WuXingJu::Earth5 => 5,
            WuXingJu::Fire6 => 6,
        }
    }
}

pub fn ming_gong_branch(lunar_month: u32, birth_hour_branch: EarthlyBranch) -> EarthlyBranch {
    let m = (lunar_month.clamp(1, 12) - 1) as usize;
    let h = birth_hour_branch as usize;
    // Classical ZWDS: count backward from Yin by birth hour
    let idx = (2_i32 + m as i32 - h as i32).rem_euclid(12) as usize;
    EarthlyBranch::from_index(idx)
}

pub fn shen_gong_branch(lunar_month: u32, birth_hour_branch: EarthlyBranch) -> EarthlyBranch {
    let m = (lunar_month.clamp(1, 12) - 1) as usize;
    let h = birth_hour_branch as usize;
    // Classical ZWDS: count FORWARD from Yin by birth hour (opposite of Ming Gong)
    // Ming = Yin + month - hour; Shen = Yin + month + hour
    let idx = (2 + m + h) % 12;
    EarthlyBranch::from_index(idx)
}

/// Derive the Five-Element Bureau from the year stem and Ming Gong branch.
///
/// This 5x12 lookup table maps (stem_group, branch) to one of the five bureaus.
/// The table follows the classical Nayin (纳音) grouping used in standard ZWDS texts.
/// Reference algorithm: https://howto-calculate.com/how-to-calculate-zi-wei-dou-shu-chart-step-by-step/
///
/// Note: Wu Xing Ju uses a simplified stem-group lookup. Classical ZWDS derives
/// the bureau from Ming palace stem-branch Na Yin. This simplified version
/// produces correct results for the most common configurations but may differ
/// for edge cases where the full 60 Jia-Zi Na Yin cycle is needed.
pub fn wu_xing_ju(year_stem: HeavenlyStem, ming_branch: EarthlyBranch) -> WuXingJu {
    let stem_group = (year_stem as usize) / 2;
    let branch_idx = ming_branch as usize;
    let table: [[WuXingJu; 12]; 5] = [
        // Jia/Yi
        [
            WuXingJu::Water2,
            WuXingJu::Fire6,
            WuXingJu::Fire6,
            WuXingJu::Wood3,
            WuXingJu::Wood3,
            WuXingJu::Metal4,
            WuXingJu::Metal4,
            WuXingJu::Earth5,
            WuXingJu::Earth5,
            WuXingJu::Water2,
            WuXingJu::Water2,
            WuXingJu::Fire6,
        ],
        // Bing/Ding
        [
            WuXingJu::Fire6,
            WuXingJu::Earth5,
            WuXingJu::Earth5,
            WuXingJu::Water2,
            WuXingJu::Water2,
            WuXingJu::Wood3,
            WuXingJu::Wood3,
            WuXingJu::Metal4,
            WuXingJu::Metal4,
            WuXingJu::Fire6,
            WuXingJu::Fire6,
            WuXingJu::Earth5,
        ],
        // Wu/Ji
        [
            WuXingJu::Metal4,
            WuXingJu::Water2,
            WuXingJu::Water2,
            WuXingJu::Fire6,
            WuXingJu::Fire6,
            WuXingJu::Earth5,
            WuXingJu::Earth5,
            WuXingJu::Wood3,
            WuXingJu::Wood3,
            WuXingJu::Metal4,
            WuXingJu::Metal4,
            WuXingJu::Water2,
        ],
        // Geng/Xin
        [
            WuXingJu::Earth5,
            WuXingJu::Wood3,
            WuXingJu::Wood3,
            WuXingJu::Metal4,
            WuXingJu::Metal4,
            WuXingJu::Water2,
            WuXingJu::Water2,
            WuXingJu::Fire6,
            WuXingJu::Fire6,
            WuXingJu::Earth5,
            WuXingJu::Earth5,
            WuXingJu::Wood3,
        ],
        // Ren/Gui
        [
            WuXingJu::Wood3,
            WuXingJu::Metal4,
            WuXingJu::Metal4,
            WuXingJu::Earth5,
            WuXingJu::Earth5,
            WuXingJu::Fire6,
            WuXingJu::Fire6,
            WuXingJu::Water2,
            WuXingJu::Water2,
            WuXingJu::Wood3,
            WuXingJu::Wood3,
            WuXingJu::Metal4,
        ],
    ];
    table[stem_group][branch_idx]
}

pub fn zi_wei_palace(lunar_day: u32, ju: WuXingJu) -> usize {
    // Classical 起紫微星诀: with D = lunar day, B = bureau number (Water2..Fire6):
    //   Q   = ceil(D/B)                       (商, round up)
    //   cha = Q*B - D   in 0..B-1             (差, shortfall to next multiple)
    //   base = count Q palaces forward from Yin (寅, branch index 2)
    //   cha == 0 -> base;  cha odd -> count back;  cha even -> count forward.
    // The parity test is on `cha` (the shortfall), NOT `D mod B`. Verified across
    // all 30x5 cases against the iztro reference (SylarLong/iztro). 2026-05-28.
    let day = lunar_day.clamp(1, 30) as i32;
    let b = ju.value() as i32; // 2..=6
    let q = (day + b - 1) / b; // ceil(day / b)
    let cha = q * b - day; // 0..b-1
    let base = (2 + (q - 1)).rem_euclid(12);
    let idx = if cha == 0 {
        base
    } else if cha % 2 == 1 {
        (base - cha).rem_euclid(12)
    } else {
        (base + cha).rem_euclid(12)
    };
    idx as usize
}

const ZIWEI_GROUP_OFFSETS: [(MajorStar, i32); 6] = [
    (MajorStar::ZiWei, 0),
    (MajorStar::TianJi, -1),
    (MajorStar::TaiYang, -3),
    (MajorStar::WuQu, -4),
    (MajorStar::TianTong, -5),
    (MajorStar::LianZhen, -8),
];

const TIANFU_GROUP_OFFSETS: [(MajorStar, i32); 8] = [
    (MajorStar::TianFu, 0),
    (MajorStar::TaiYin, 1),
    (MajorStar::TanLang, 2),
    (MajorStar::JuMen, 3),
    (MajorStar::TianXiang, 4),
    (MajorStar::TianLiang, 5),
    (MajorStar::QiSha, 6),
    (MajorStar::PoJun, 8),
];

pub fn place_major_stars(zi_wei_idx: usize) -> [(MajorStar, usize); 14] {
    let tian_fu_idx = (16_i32 - zi_wei_idx as i32).rem_euclid(12) as usize;
    let mut result = [(MajorStar::ZiWei, 0); 14];
    for (i, &(star, offset)) in ZIWEI_GROUP_OFFSETS.iter().enumerate() {
        result[i] = (star, (zi_wei_idx as i32 + offset).rem_euclid(12) as usize);
    }
    for (i, &(star, offset)) in TIANFU_GROUP_OFFSETS.iter().enumerate() {
        result[6 + i] = (star, (tian_fu_idx as i32 + offset).rem_euclid(12) as usize);
    }
    result
}

pub fn compute_chart(
    year_stem: HeavenlyStem,
    lunar_month: u32,
    lunar_day: u32,
    birth_hour_branch: EarthlyBranch,
) -> ZiWeiChart {
    let ming = ming_gong_branch(lunar_month, birth_hour_branch);
    let shen = shen_gong_branch(lunar_month, birth_hour_branch);
    let ju = wu_xing_ju(year_stem, ming);
    let zi_wei_idx = zi_wei_palace(lunar_day.clamp(1, 30), ju);
    let stars = place_major_stars(zi_wei_idx);

    let ming_idx = ming as usize;
    let mut palaces: [PalaceInfo; 12] = std::array::from_fn(|i| {
        let branch_idx = (ming_idx + 12 - i) % 12;
        PalaceInfo {
            palace: Palace::ALL[i],
            branch: EarthlyBranch::from_index(branch_idx),
            major_stars: Vec::new(),
        }
    });

    for &(star, palace_branch_idx) in &stars {
        for p in palaces.iter_mut() {
            if p.branch as usize == palace_branch_idx {
                p.major_stars.push(star);
                break;
            }
        }
    }

    ZiWeiChart {
        palaces,
        ming_gong_branch: ming,
        shen_gong_branch: shen,
        wu_xing_ju: ju,
    }
}

impl MajorStar {
    pub fn chinese_name(&self) -> &'static str {
        match self {
            MajorStar::ZiWei => "紫微",
            MajorStar::TianJi => "天機",
            MajorStar::TaiYang => "太陽",
            MajorStar::WuQu => "武曲",
            MajorStar::TianTong => "天同",
            MajorStar::LianZhen => "廉貞",
            MajorStar::TianFu => "天府",
            MajorStar::TaiYin => "太陰",
            MajorStar::TanLang => "貪狼",
            MajorStar::JuMen => "巨門",
            MajorStar::TianXiang => "天相",
            MajorStar::TianLiang => "天梁",
            MajorStar::QiSha => "七殺",
            MajorStar::PoJun => "破軍",
        }
    }

    pub fn element(&self) -> WuXing {
        match self {
            MajorStar::ZiWei | MajorStar::TianFu => WuXing::Earth,
            MajorStar::TianJi | MajorStar::TianLiang => WuXing::Wood,
            MajorStar::TaiYang | MajorStar::LianZhen => WuXing::Fire,
            MajorStar::WuQu | MajorStar::QiSha => WuXing::Metal,
            MajorStar::TianTong | MajorStar::PoJun => WuXing::Water,
            MajorStar::TaiYin | MajorStar::TianXiang => WuXing::Water,
            MajorStar::TanLang => WuXing::Wood,
            MajorStar::JuMen => WuXing::Earth,
        }
    }

    pub fn is_benefic(&self) -> bool {
        matches!(
            self,
            MajorStar::ZiWei
                | MajorStar::TianFu
                | MajorStar::TianTong
                | MajorStar::TianLiang
                | MajorStar::TianXiang
        )
    }
}

// ---------------------------------------------------------------------------
// Minor Stars (輔星)
// ---------------------------------------------------------------------------

/// The 14 commonly used minor stars in Zi Wei Dou Shu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MinorStar {
    // Lucky stars (吉星)
    WenChang, // 文昌 — Literary Brilliance
    WenQu,    // 文曲 — Literary Music
    ZuoFu,    // 左輔 — Left Assistant
    YouBi,    // 右弼 — Right Assistant
    TianKui,  // 天魁 — Heavenly Leader
    TianYue,  // 天鉞 — Heavenly Halberd
    // Unlucky stars (煞星)
    HuoXing,  // 火星 — Fire Star
    LingXing, // 鈴星 — Bell Star
    QingYang, // 擎羊 — Ram (Blade Above)
    TuoLuo,   // 陀羅 — Spinning Top (Blade Below)
    DiKong,   // 地空 — Earthly Void
    DiJie,    // 地劫 — Earthly Robbery
    // Transformation stars (四化)
    HuaLu,   // 化祿 — Transformation of Wealth
    HuaQuan, // 化權 — Transformation of Power
}

impl MinorStar {
    pub fn chinese_name(&self) -> &'static str {
        match self {
            MinorStar::WenChang => "文昌",
            MinorStar::WenQu => "文曲",
            MinorStar::ZuoFu => "左輔",
            MinorStar::YouBi => "右弼",
            MinorStar::TianKui => "天魁",
            MinorStar::TianYue => "天鉞",
            MinorStar::HuoXing => "火星",
            MinorStar::LingXing => "鈴星",
            MinorStar::QingYang => "擎羊",
            MinorStar::TuoLuo => "陀羅",
            MinorStar::DiKong => "地空",
            MinorStar::DiJie => "地劫",
            MinorStar::HuaLu => "化祿",
            MinorStar::HuaQuan => "化權",
        }
    }

    pub fn is_lucky(&self) -> bool {
        matches!(
            self,
            MinorStar::WenChang
                | MinorStar::WenQu
                | MinorStar::ZuoFu
                | MinorStar::YouBi
                | MinorStar::TianKui
                | MinorStar::TianYue
                | MinorStar::HuaLu
                | MinorStar::HuaQuan
        )
    }
}

/// Place minor stars based on year stem, year branch, birth hour, and Ming palace branch.
///
/// Standard placement rules from classical ZWDS texts:
/// - Wen Chang: derived from year stem (counter to Lu Cun order)
/// - Wen Qu: derived from year stem (counter to Wen Chang)
/// - Zuo Fu: lunar month + 3 (branch index)
///   -- we use ming_branch as a proxy since lunar month maps to ming
/// - You Bi: 11 - lunar month (branch index)
///   -- derived from ming_branch analogously
/// - Tian Kui / Tian Yue: year-stem lookup table
/// - Huo Xing / Ling Xing: year-branch group base + birth-hour offset
/// - Qing Yang: Lu Cun + 1 (branch index)
/// - Tuo Luo: Lu Cun - 1 (branch index)
/// - Di Kong: 11 - birth-hour (branch index)
/// - Di Jie: birth-hour - 1 (branch index)
/// - Hua Lu / Hua Quan: year-stem transformation table
pub fn place_minor_stars(
    year_stem: HeavenlyStem,
    year_branch: EarthlyBranch,
    ming_branch: EarthlyBranch,
    birth_hour_branch: EarthlyBranch,
) -> Vec<(MinorStar, usize)> {
    let si = year_stem as usize;
    let hi = birth_hour_branch as usize;
    let mi = ming_branch as usize;

    // Lu Cun palace (祿存) index by year stem — needed for Qing Yang / Tuo Luo.
    // Jia=Yin(2), Yi=Mao(3), Bing/Wu=Si(5), Ding/Ji=Wu(6),
    // Geng=Shen(8), Xin=You(9), Ren=Hai(11), Gui=Zi(0)
    let lu_cun: [usize; 10] = [2, 3, 5, 6, 5, 6, 8, 9, 11, 0];
    let lc = lu_cun[si];

    // Wen Chang (文昌): year stem lookup
    // Jia=Si(5), Yi=Wu(6), Bing=Shen(8), Ding=You(9), Wu=Shen(8),
    // Ji=You(9), Geng=Hai(11), Xin=Zi(0), Ren=Yin(2), Gui=Mao(3)
    let wen_chang_table: [usize; 10] = [5, 6, 8, 9, 8, 9, 11, 0, 2, 3];

    // Wen Qu (文曲): year stem lookup (reverse direction from Wen Chang)
    // Jia=You(9), Yi=Shen(8), Bing=Wu(6), Ding=Si(5), Wu=Wu(6),
    // Ji=Si(5), Geng=Mao(3), Xin=Yin(2), Ren=Zi(0), Gui=Hai(11)
    let wen_qu_table: [usize; 10] = [9, 8, 6, 5, 6, 5, 3, 2, 0, 11];

    // Tian Kui (天魁): year stem lookup
    // Jia/Wu=Chou(1), Yi/Ji=Zi(0), Bing/Ding=Hai(11),
    // Geng/Xin=Wu(6), Ren/Gui=Mao(3)
    let tian_kui_table: [usize; 10] = [1, 0, 11, 11, 1, 0, 6, 6, 3, 3];

    // Tian Yue (天鉞): year stem lookup
    // Jia/Wu=Wei(7), Yi/Ji=Shen(8), Bing/Ding=You(9),
    // Geng/Xin=Yin(2), Ren/Gui=Si(5)
    let tian_yue_table: [usize; 10] = [7, 8, 9, 9, 7, 8, 2, 2, 5, 5];

    // Hua Lu (化祿): year stem determines which major star gets Hua Lu;
    // we place Hua Lu at the same palace branch as that major star's
    // canonical stem-based position.
    // Jia=Yin(2), Yi=Mao(3), Bing=Si(5), Ding=Wu(6), Wu=Si(5),
    // Ji=Wu(6), Geng=Shen(8), Xin=You(9), Ren=Hai(11), Gui=Zi(0)
    // (Same as Lu Cun positions — Lu Cun and Hua Lu share the stem table.)
    let hua_lu_table: [usize; 10] = [2, 3, 5, 6, 5, 6, 8, 9, 11, 0];

    // Hua Quan (化權): year stem lookup
    // Jia=Chen(4), Yi=Hai(11), Bing=Yin(2), Ding=Si(5), Wu=Mao(3),
    // Ji=Wu(6), Geng=Wu(6), Xin=Si(5), Ren=Mao(3), Gui=Yin(2)
    let hua_quan_table: [usize; 10] = [4, 11, 2, 5, 3, 6, 6, 5, 3, 2];

    // Zuo Fu (左輔): (ming_branch + 4) mod 12
    let zuo_fu_idx = (mi + 4) % 12;
    // You Bi (右弼): (mi + 12 - 4) mod 12 = (mi + 8) mod 12
    let you_bi_idx = (mi + 8) % 12;

    // Huo Xing (火星): year-branch group base + birth-hour offset.
    // Yin/Wu/Xu group → base Chou(1), Shen/Zi/Chen group → base Yin(2),
    // Si/You/Chou group → base Mao(3), Hai/Mao/Wei group → base You(9)
    let huo_base = match year_branch {
        EarthlyBranch::Yin | EarthlyBranch::Wu | EarthlyBranch::Xu => 1,
        EarthlyBranch::Shen | EarthlyBranch::Zi | EarthlyBranch::Chen => 2,
        EarthlyBranch::Si | EarthlyBranch::You | EarthlyBranch::Chou => 3,
        EarthlyBranch::Hai | EarthlyBranch::Mao | EarthlyBranch::Wei => 9,
    };
    let huo_idx = (huo_base + hi) % 12;

    // Ling Xing (鈴星): year-branch group base + birth-hour offset.
    // Yin/Wu/Xu → base Mao(3), Shen/Zi/Chen → base Xu(10),
    // Si/You/Chou → base Xu(10), Hai/Mao/Wei → base Xu(10)
    let ling_base = match year_branch {
        EarthlyBranch::Yin | EarthlyBranch::Wu | EarthlyBranch::Xu => 3,
        _ => 10,
    };
    let ling_idx = (ling_base + hi) % 12;

    // Qing Yang (擎羊): Lu Cun + 1
    let qing_yang_idx = (lc + 1) % 12;
    // Tuo Luo (陀羅): Lu Cun - 1
    let tuo_luo_idx = (lc + 11) % 12;

    // Di Kong (地空): 11 - birth_hour (count backward from Hai)
    let di_kong_idx = (11 + 12 - hi) % 12;
    // Di Jie (地劫): birth_hour - 1 (count forward from Hai)
    let di_jie_idx = (hi + 11) % 12;

    vec![
        (MinorStar::WenChang, wen_chang_table[si]),
        (MinorStar::WenQu, wen_qu_table[si]),
        (MinorStar::ZuoFu, zuo_fu_idx),
        (MinorStar::YouBi, you_bi_idx),
        (MinorStar::TianKui, tian_kui_table[si]),
        (MinorStar::TianYue, tian_yue_table[si]),
        (MinorStar::HuoXing, huo_idx),
        (MinorStar::LingXing, ling_idx),
        (MinorStar::QingYang, qing_yang_idx),
        (MinorStar::TuoLuo, tuo_luo_idx),
        (MinorStar::DiKong, di_kong_idx),
        (MinorStar::DiJie, di_jie_idx),
        (MinorStar::HuaLu, hua_lu_table[si]),
        (MinorStar::HuaQuan, hua_quan_table[si]),
    ]
}

// ---------------------------------------------------------------------------
// Da Xian (大限 - Decade Luck / Major Periods)
// ---------------------------------------------------------------------------

/// A single 10-year major period in the ZWDS chart.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaXianPeriod {
    /// Palace index (0-11, where 0 = Zi position in the 12-palace ring).
    pub palace_idx: usize,
    /// Starting age of this period (inclusive).
    pub start_age: u32,
    /// Ending age of this period (inclusive).
    pub end_age: u32,
}

/// Compute the Da Xian (大限) decade-luck periods.
///
/// - `wu_xing_ju`: the Five-Element Bureau determines the starting age
///   (Water=2, Wood=3, Metal=4, Earth=5, Fire=6).
/// - `ming_idx`: the palace-ring index of Ming Gong (0-11).
/// - `is_yang_chart`: true for Yang charts (male born in yang year OR female
///   born in yin year); Yang charts advance clockwise, Yin charts counter-clockwise.
/// - `num_periods`: how many 10-year periods to generate (typically 12).
pub fn compute_da_xian(
    wu_xing_ju: WuXingJu,
    ming_idx: usize,
    is_yang_chart: bool,
    num_periods: u32,
) -> Vec<DaXianPeriod> {
    let start_age = wu_xing_ju.value();
    let mut periods = Vec::with_capacity(num_periods as usize);

    for i in 0..num_periods {
        let offset = i as i32;
        let palace = if is_yang_chart {
            (ming_idx as i32 + offset).rem_euclid(12) as usize
        } else {
            (ming_idx as i32 - offset).rem_euclid(12) as usize
        };
        let age_start = start_age + i * 10;
        let age_end = age_start + 9;
        periods.push(DaXianPeriod {
            palace_idx: palace,
            start_age: age_start,
            end_age: age_end,
        });
    }

    periods
}

// ---------------------------------------------------------------------------
// Liu Nian (流年 - Annual Luck)
// ---------------------------------------------------------------------------

/// Determine the Liu Nian (流年) annual palace for a given year.
///
/// The annual palace is the palace whose earthly branch matches the year's
/// earthly branch.  Since the 12 palaces are always mapped to the 12 earthly
/// branches, this simply returns the branch index (0-11).
pub fn liu_nian_palace(year_branch: EarthlyBranch) -> usize {
    year_branch as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    // Branch ring (Zi=0): Zi,Chou,Yin,Mao,Chen,Si,Wu,Wei,Shen,You,Xu,Hai = 0..11.
    #[test]
    fn zi_wei_palace_canonical_cases() {
        // Fire-6 bureau, day 13: Q=3, cha=5 (odd) -> count back -> Hai (11).
        assert_eq!(zi_wei_palace(13, WuXingJu::Fire6), 11);
        // Earth-5 bureau, day 15: Q=3, cha=0 -> base -> Chen (4).
        assert_eq!(zi_wei_palace(15, WuXingJu::Earth5), 4);
        // Water-2 bureau, day 1: Q=1, cha=1 (odd) -> back from Yin -> Chou (1).
        assert_eq!(zi_wei_palace(1, WuXingJu::Water2), 1);
        // Every (day, bureau) must land on a valid branch 0..11.
        for d in 1..=30u32 {
            for ju in [
                WuXingJu::Water2,
                WuXingJu::Wood3,
                WuXingJu::Metal4,
                WuXingJu::Earth5,
                WuXingJu::Fire6,
            ] {
                assert!(zi_wei_palace(d, ju) < 12);
            }
        }
    }

    #[test]
    fn ming_gong_month1_zi_hour() {
        let mg = ming_gong_branch(1, EarthlyBranch::Zi);
        assert_eq!(mg, EarthlyBranch::Yin);
    }

    #[test]
    fn ming_gong_varies_with_hour() {
        let mg1 = ming_gong_branch(1, EarthlyBranch::Zi);
        let mg2 = ming_gong_branch(1, EarthlyBranch::Chou);
        assert_ne!(mg1, mg2);
    }

    #[test]
    fn shen_gong_computed() {
        let sg = shen_gong_branch(1, EarthlyBranch::Zi);
        // Shen Gong = (2 + 0 + 0) % 12 = 2 = Yin
        assert_eq!(sg, EarthlyBranch::Yin);
    }

    #[test]
    fn shen_gong_differs_from_ming_for_nonzero_hours() {
        // For any non-Zi birth hour, Shen Gong should differ from Ming Gong.
        // Ming = (2 + m - h) % 12, Shen = (2 + m + h) % 12.
        // They differ when 2h % 12 != 0, i.e. h != 0 and h != 6.
        let mut differ_count = 0;
        for month in 1..=12 {
            for h in 0..12_usize {
                let hour = EarthlyBranch::from_index(h);
                let ming = ming_gong_branch(month, hour);
                let shen = shen_gong_branch(month, hour);
                if ming != shen {
                    differ_count += 1;
                }
            }
        }
        // Out of 144 combos, h=0 (Zi) and h=6 (Wu) make them equal = 24 equal cases.
        // So at least 120 should differ.
        assert!(
            differ_count >= 120,
            "Shen Gong should differ from Ming Gong in most cases, got {differ_count}/144 differing"
        );
    }

    #[test]
    fn shen_gong_specific_values() {
        // Month 3, hour Yin(2): Ming = (2+2-2)%12 = 2 (Yin), Shen = (2+2+2)%12 = 6 (Wu)
        let ming = ming_gong_branch(3, EarthlyBranch::Yin);
        let shen = shen_gong_branch(3, EarthlyBranch::Yin);
        assert_eq!(ming, EarthlyBranch::Yin);
        assert_eq!(shen, EarthlyBranch::Wu);
        assert_ne!(ming, shen);

        // Month 6, hour Si(5): Ming = (2+5-5)%12 = 2 (Yin), Shen = (2+5+5)%12 = 0 (Zi)
        let ming2 = ming_gong_branch(6, EarthlyBranch::Si);
        let shen2 = shen_gong_branch(6, EarthlyBranch::Si);
        assert_eq!(ming2, EarthlyBranch::Yin);
        assert_eq!(shen2, EarthlyBranch::Zi);
        assert_ne!(ming2, shen2);
    }

    #[test]
    fn shen_gong_always_valid_range() {
        for month in 1..=12 {
            for h in 0..12_usize {
                let hour = EarthlyBranch::from_index(h);
                let shen = shen_gong_branch(month, hour);
                let idx = shen as usize;
                assert!(
                    idx < 12,
                    "Shen Gong out of range for month={month}, hour={h}: {idx}"
                );
            }
        }
    }

    #[test]
    fn wu_xing_ju_is_2_to_6() {
        let ju = wu_xing_ju(HeavenlyStem::Jia, EarthlyBranch::Zi);
        let v = ju.value();
        assert!(v >= 2 && v <= 6, "Wu Xing Ju should be 2-6, got {v}");
    }

    #[test]
    fn zi_wei_palace_day_zero_no_underflow() {
        // lunar_day == 0 should be clamped to 1, not underflow
        for ju in [
            WuXingJu::Water2,
            WuXingJu::Wood3,
            WuXingJu::Metal4,
            WuXingJu::Earth5,
            WuXingJu::Fire6,
        ] {
            let p = zi_wei_palace(0, ju);
            let p1 = zi_wei_palace(1, ju);
            assert!(p < 12, "Palace index out of range for day=0, ju={:?}", ju);
            assert_eq!(p, p1, "day=0 should produce same result as day=1 for ju={:?}", ju);
        }
    }

    #[test]
    fn zi_wei_palace_in_range() {
        for day in 1..=30 {
            for ju in [
                WuXingJu::Water2,
                WuXingJu::Wood3,
                WuXingJu::Metal4,
                WuXingJu::Earth5,
                WuXingJu::Fire6,
            ] {
                let p = zi_wei_palace(day, ju);
                assert!(
                    p < 12,
                    "Palace index out of range for day={day}, ju={:?}",
                    ju
                );
            }
        }
    }

    #[test]
    fn place_14_major_stars() {
        let stars = place_major_stars(2);
        assert_eq!(stars.len(), 14);
        let unique_stars: std::collections::HashSet<_> = stars.iter().map(|(s, _)| *s).collect();
        assert_eq!(unique_stars.len(), 14, "All 14 stars should be unique");
    }

    #[test]
    fn all_star_palaces_in_range() {
        for zi_wei_pos in 0..12 {
            let stars = place_major_stars(zi_wei_pos);
            for (star, palace) in &stars {
                assert!(
                    *palace < 12,
                    "{:?} placed at invalid palace {}",
                    star,
                    palace
                );
            }
        }
    }

    #[test]
    fn compute_chart_has_12_palaces() {
        let chart = compute_chart(HeavenlyStem::Jia, 1, 15, EarthlyBranch::Zi);
        assert_eq!(chart.palaces.len(), 12);
        assert_eq!(chart.palaces[0].palace, Palace::Life);
    }

    #[test]
    fn chart_places_all_14_stars() {
        let chart = compute_chart(HeavenlyStem::Bing, 6, 20, EarthlyBranch::Wu);
        let total_stars: usize = chart.palaces.iter().map(|p| p.major_stars.len()).sum();
        assert_eq!(total_stars, 14, "Chart should place all 14 major stars");
    }

    #[test]
    fn major_star_chinese_names() {
        assert_eq!(MajorStar::ZiWei.chinese_name(), "紫微");
        assert_eq!(MajorStar::QiSha.chinese_name(), "七殺");
    }

    #[test]
    fn major_star_elements() {
        assert_eq!(MajorStar::ZiWei.element(), WuXing::Earth);
        assert_eq!(MajorStar::TianJi.element(), WuXing::Wood);
        assert_eq!(MajorStar::WuQu.element(), WuXing::Metal);
    }

    #[test]
    fn benefic_stars() {
        assert!(MajorStar::ZiWei.is_benefic());
        assert!(MajorStar::TianFu.is_benefic());
        assert!(!MajorStar::QiSha.is_benefic());
        assert!(!MajorStar::PoJun.is_benefic());
    }

    #[test]
    fn palace_chinese_names() {
        assert_eq!(Palace::Life.chinese_name(), "命宮");
        assert_eq!(Palace::Career.chinese_name(), "事業宮");
    }

    #[test]
    fn different_months_different_charts() {
        let c1 = compute_chart(HeavenlyStem::Jia, 1, 15, EarthlyBranch::Zi);
        let c2 = compute_chart(HeavenlyStem::Jia, 6, 15, EarthlyBranch::Zi);
        assert_ne!(c1.ming_gong_branch, c2.ming_gong_branch);
    }

    // -------------------------------------------------------------------
    // Minor Stars tests
    // -------------------------------------------------------------------

    #[test]
    fn place_14_minor_stars() {
        let stars = place_minor_stars(
            HeavenlyStem::Jia,
            EarthlyBranch::Zi,
            EarthlyBranch::Yin,
            EarthlyBranch::Zi,
        );
        assert_eq!(stars.len(), 14, "Must place exactly 14 minor stars");
        let unique: std::collections::HashSet<_> = stars.iter().map(|(s, _)| *s).collect();
        assert_eq!(unique.len(), 14, "All 14 minor stars should be distinct");
    }

    #[test]
    fn minor_star_palaces_in_range() {
        for stem in HeavenlyStem::ALL.iter() {
            for branch in EarthlyBranch::ALL.iter() {
                for hour in EarthlyBranch::ALL.iter() {
                    let stars = place_minor_stars(*stem, *branch, EarthlyBranch::Zi, *hour);
                    for (star, idx) in &stars {
                        assert!(
                            *idx < 12,
                            "{:?} placed at invalid index {} for {:?}/{:?}/{:?}",
                            star,
                            idx,
                            stem,
                            branch,
                            hour
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn minor_star_chinese_names() {
        assert_eq!(MinorStar::WenChang.chinese_name(), "文昌");
        assert_eq!(MinorStar::HuoXing.chinese_name(), "火星");
        assert_eq!(MinorStar::HuaLu.chinese_name(), "化祿");
        assert_eq!(MinorStar::DiKong.chinese_name(), "地空");
    }

    #[test]
    fn minor_star_lucky_classification() {
        assert!(MinorStar::WenChang.is_lucky());
        assert!(MinorStar::ZuoFu.is_lucky());
        assert!(MinorStar::TianKui.is_lucky());
        assert!(MinorStar::HuaLu.is_lucky());
        assert!(!MinorStar::HuoXing.is_lucky());
        assert!(!MinorStar::QingYang.is_lucky());
        assert!(!MinorStar::DiKong.is_lucky());
        assert!(!MinorStar::DiJie.is_lucky());
    }

    #[test]
    fn qing_yang_tuo_luo_flank_lu_cun() {
        // For Jia year, Lu Cun is at Yin(2), so QingYang=3, TuoLuo=1.
        let stars = place_minor_stars(
            HeavenlyStem::Jia,
            EarthlyBranch::Zi,
            EarthlyBranch::Yin,
            EarthlyBranch::Zi,
        );
        let qy = stars
            .iter()
            .find(|(s, _)| *s == MinorStar::QingYang)
            .unwrap()
            .1;
        let tl = stars
            .iter()
            .find(|(s, _)| *s == MinorStar::TuoLuo)
            .unwrap()
            .1;
        // Lu Cun for Jia = Yin(2)
        assert_eq!(qy, 3, "QingYang should be Lu Cun + 1 = 3");
        assert_eq!(tl, 1, "TuoLuo should be Lu Cun - 1 = 1");
    }

    #[test]
    fn wen_chang_jia_at_si() {
        let stars = place_minor_stars(
            HeavenlyStem::Jia,
            EarthlyBranch::Zi,
            EarthlyBranch::Zi,
            EarthlyBranch::Zi,
        );
        let wc = stars
            .iter()
            .find(|(s, _)| *s == MinorStar::WenChang)
            .unwrap()
            .1;
        assert_eq!(wc, 5, "Wen Chang for Jia year should be at Si(5)");
    }

    #[test]
    fn huo_xing_varies_with_birth_hour() {
        // Huo Xing should shift with birth hour (base + hour offset).
        // Yin/Wu/Xu group base = 1. Hour Zi(0) → 1, Hour Yin(2) → 3.
        let stars_zi = place_minor_stars(
            HeavenlyStem::Jia,
            EarthlyBranch::Yin,
            EarthlyBranch::Yin,
            EarthlyBranch::Zi,
        );
        let stars_yin = place_minor_stars(
            HeavenlyStem::Jia,
            EarthlyBranch::Yin,
            EarthlyBranch::Yin,
            EarthlyBranch::Yin,
        );
        let huo_zi = stars_zi
            .iter()
            .find(|(s, _)| *s == MinorStar::HuoXing)
            .unwrap()
            .1;
        let huo_yin = stars_yin
            .iter()
            .find(|(s, _)| *s == MinorStar::HuoXing)
            .unwrap()
            .1;
        assert_eq!(
            huo_zi, 1,
            "Huo Xing at Zi hour with Yin year = base 1 + 0 = 1"
        );
        assert_eq!(
            huo_yin, 3,
            "Huo Xing at Yin hour with Yin year = base 1 + 2 = 3"
        );
        assert_ne!(huo_zi, huo_yin, "Huo Xing should vary with birth hour");
    }

    #[test]
    fn di_kong_di_jie_use_birth_hour() {
        // Di Kong = (11 - hour) % 12, Di Jie = (hour - 1) % 12 = (hour + 11) % 12
        // Hour Zi(0): Di Kong = 11, Di Jie = 11
        // Hour Yin(2): Di Kong = 9, Di Jie = 1
        let stars_yin = place_minor_stars(
            HeavenlyStem::Jia,
            EarthlyBranch::Zi,
            EarthlyBranch::Yin,
            EarthlyBranch::Yin,
        );
        let dk = stars_yin
            .iter()
            .find(|(s, _)| *s == MinorStar::DiKong)
            .unwrap()
            .1;
        let dj = stars_yin
            .iter()
            .find(|(s, _)| *s == MinorStar::DiJie)
            .unwrap()
            .1;
        assert_eq!(dk, 9, "Di Kong at Yin(2) hour = (11-2)%12 = 9");
        assert_eq!(dj, 1, "Di Jie at Yin(2) hour = (2+11)%12 = 1");
    }

    // -------------------------------------------------------------------
    // Da Xian tests
    // -------------------------------------------------------------------

    #[test]
    fn da_xian_12_periods() {
        let periods = compute_da_xian(WuXingJu::Water2, 2, true, 12);
        assert_eq!(periods.len(), 12);
    }

    #[test]
    fn da_xian_starts_at_wu_xing_value() {
        let periods_w = compute_da_xian(WuXingJu::Water2, 0, true, 1);
        assert_eq!(periods_w[0].start_age, 2);
        assert_eq!(periods_w[0].end_age, 11);

        let periods_f = compute_da_xian(WuXingJu::Fire6, 0, true, 1);
        assert_eq!(periods_f[0].start_age, 6);
        assert_eq!(periods_f[0].end_age, 15);
    }

    #[test]
    fn da_xian_yang_clockwise() {
        let periods = compute_da_xian(WuXingJu::Wood3, 4, true, 3);
        assert_eq!(periods[0].palace_idx, 4);
        assert_eq!(periods[1].palace_idx, 5);
        assert_eq!(periods[2].palace_idx, 6);
    }

    #[test]
    fn da_xian_yin_counterclockwise() {
        let periods = compute_da_xian(WuXingJu::Metal4, 4, false, 3);
        assert_eq!(periods[0].palace_idx, 4);
        assert_eq!(periods[1].palace_idx, 3);
        assert_eq!(periods[2].palace_idx, 2);
    }

    #[test]
    fn da_xian_wraps_around() {
        // Yang chart starting at palace 10, should wrap: 10, 11, 0, 1, ...
        let periods = compute_da_xian(WuXingJu::Earth5, 10, true, 4);
        assert_eq!(periods[0].palace_idx, 10);
        assert_eq!(periods[1].palace_idx, 11);
        assert_eq!(periods[2].palace_idx, 0);
        assert_eq!(periods[3].palace_idx, 1);

        // Yin chart starting at palace 1, should wrap: 1, 0, 11, 10, ...
        let periods_yin = compute_da_xian(WuXingJu::Earth5, 1, false, 4);
        assert_eq!(periods_yin[0].palace_idx, 1);
        assert_eq!(periods_yin[1].palace_idx, 0);
        assert_eq!(periods_yin[2].palace_idx, 11);
        assert_eq!(periods_yin[3].palace_idx, 10);
    }

    #[test]
    fn da_xian_age_ranges_consecutive() {
        let periods = compute_da_xian(WuXingJu::Wood3, 0, true, 5);
        for i in 1..periods.len() {
            assert_eq!(
                periods[i].start_age,
                periods[i - 1].end_age + 1,
                "Period {} start should follow period {} end",
                i,
                i - 1
            );
        }
        for p in &periods {
            assert_eq!(p.end_age - p.start_age, 9, "Each period spans 10 years");
        }
    }

    // -------------------------------------------------------------------
    // Liu Nian tests
    // -------------------------------------------------------------------

    #[test]
    fn liu_nian_matches_branch_index() {
        assert_eq!(liu_nian_palace(EarthlyBranch::Zi), 0);
        assert_eq!(liu_nian_palace(EarthlyBranch::Wu), 6);
        assert_eq!(liu_nian_palace(EarthlyBranch::Hai), 11);
    }

    #[test]
    fn liu_nian_all_12_branches() {
        let palaces: Vec<usize> = EarthlyBranch::ALL
            .iter()
            .map(|b| liu_nian_palace(*b))
            .collect();
        for i in 0..12 {
            assert_eq!(
                palaces[i], i,
                "Branch index {} should map to palace {}",
                i, i
            );
        }
    }
}
