//! # I Ching (Yijing) — Book of Changes
//!
//! Complete implementation of the 64 hexagram system with 8 trigrams (Ba Gua),
//! date-based hexagram derivation, nuclear hexagrams, and relating hexagrams.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Line type
// ---------------------------------------------------------------------------

/// A single line of a hexagram.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Line {
    /// ⚊ — solid, active, odd
    Yang,
    /// ⚋ — broken, receptive, even
    Yin,
}

impl Line {
    /// `true` for Yang, `false` for Yin.
    pub fn is_yang(self) -> bool {
        self == Line::Yang
    }

    /// Flip Yang ↔ Yin.
    pub fn flip(self) -> Line {
        match self {
            Line::Yang => Line::Yin,
            Line::Yin => Line::Yang,
        }
    }
}

// ---------------------------------------------------------------------------
// Trigram (Ba Gua)
// ---------------------------------------------------------------------------

/// The eight trigrams (八卦 Ba Gua).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Trigram {
    /// ☰ Heaven, Creative (乾)
    Qian = 0,
    /// ☷ Earth, Receptive (坤)
    Kun = 1,
    /// ☳ Thunder, Arousing (震)
    Zhen = 2,
    /// ☵ Water, Abysmal (坎)
    Kan = 3,
    /// ☶ Mountain, Keeping Still (艮)
    Gen = 4,
    /// ☴ Wind, Gentle (巽)
    Xun = 5,
    /// ☲ Fire, Clinging (離)
    Li = 6,
    /// ☱ Lake, Joyous (兌)
    Dui = 7,
}

impl Trigram {
    pub const ALL: [Trigram; 8] = [
        Trigram::Qian,
        Trigram::Kun,
        Trigram::Zhen,
        Trigram::Kan,
        Trigram::Gen,
        Trigram::Xun,
        Trigram::Li,
        Trigram::Dui,
    ];

    /// Retrieve a trigram by its index (0-7), wrapping on overflow.
    pub fn from_index(i: usize) -> Trigram {
        Trigram::ALL[i % 8]
    }

    /// The three lines of this trigram (bottom to top).
    pub fn lines(self) -> [Line; 3] {
        use Line::*;
        match self {
            Trigram::Qian => [Yang, Yang, Yang], // ☰
            Trigram::Kun => [Yin, Yin, Yin],     // ☷
            Trigram::Zhen => [Yang, Yin, Yin],   // ☳
            Trigram::Kan => [Yin, Yang, Yin],    // ☵
            Trigram::Gen => [Yin, Yin, Yang],    // ☶
            Trigram::Xun => [Yin, Yang, Yang],   // ☴
            Trigram::Li => [Yang, Yin, Yang],    // ☲
            Trigram::Dui => [Yang, Yang, Yin],   // ☱
        }
    }

    /// English name of the trigram.
    pub fn name_en(self) -> &'static str {
        match self {
            Trigram::Qian => "Heaven",
            Trigram::Kun => "Earth",
            Trigram::Zhen => "Thunder",
            Trigram::Kan => "Water",
            Trigram::Gen => "Mountain",
            Trigram::Xun => "Wind",
            Trigram::Li => "Fire",
            Trigram::Dui => "Lake",
        }
    }

    /// Chinese name of the trigram.
    pub fn name_zh(self) -> &'static str {
        match self {
            Trigram::Qian => "乾",
            Trigram::Kun => "坤",
            Trigram::Zhen => "震",
            Trigram::Kan => "坎",
            Trigram::Gen => "艮",
            Trigram::Xun => "巽",
            Trigram::Li => "離",
            Trigram::Dui => "兌",
        }
    }

    /// Attribute: Creative quality of the trigram.
    pub fn attribute(self) -> &'static str {
        match self {
            Trigram::Qian => "Creative",
            Trigram::Kun => "Receptive",
            Trigram::Zhen => "Arousing",
            Trigram::Kan => "Abysmal",
            Trigram::Gen => "Keeping Still",
            Trigram::Xun => "Gentle",
            Trigram::Li => "Clinging",
            Trigram::Dui => "Joyous",
        }
    }

    /// Unicode symbol for the trigram.
    pub fn symbol(self) -> char {
        match self {
            Trigram::Qian => '☰',
            Trigram::Kun => '☷',
            Trigram::Zhen => '☳',
            Trigram::Kan => '☵',
            Trigram::Gen => '☶',
            Trigram::Xun => '☴',
            Trigram::Li => '☲',
            Trigram::Dui => '☱',
        }
    }
}

/// Identify a trigram from its three lines (bottom to top).
/// `true` = Yang, `false` = Yin.
pub fn trigram_from_lines(lines: [bool; 3]) -> Trigram {
    match lines {
        [true, true, true] => Trigram::Qian,
        [false, false, false] => Trigram::Kun,
        [true, false, false] => Trigram::Zhen,
        [false, true, false] => Trigram::Kan,
        [false, false, true] => Trigram::Gen,
        [false, true, true] => Trigram::Xun,
        [true, false, true] => Trigram::Li,
        [true, true, false] => Trigram::Dui,
    }
}

// ---------------------------------------------------------------------------
// Hexagram
// ---------------------------------------------------------------------------

/// A hexagram from the I Ching (Book of Changes).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hexagram {
    /// King Wen sequence number (1-64).
    pub number: u8,
    /// English name.
    pub name_en: &'static str,
    /// Chinese name.
    pub name_zh: &'static str,
    /// Upper (outer) trigram.
    pub upper_trigram: Trigram,
    /// Lower (inner) trigram.
    pub lower_trigram: Trigram,
    /// Six lines, bottom (1) to top (6).
    pub lines: [Line; 6],
    /// Brief judgment text (Tuan).
    pub judgment: &'static str,
    /// Brief image text (Xiang).
    pub image: &'static str,
}

/// Result of a date-based hexagram reading.
///
/// Note: only `Serialize` is derived because the struct holds `&'static`
/// references into the compiled hexagram table, which cannot be deserialized.
#[derive(Debug, Clone, Serialize)]
pub struct HexagramReading {
    /// The primary hexagram.
    pub primary: &'static Hexagram,
    /// The index (0-5, bottom to top) of the changing line.
    pub changing_line: usize,
    /// The relating (transformed) hexagram after flipping the changing line.
    pub relating: &'static Hexagram,
}

// ---------------------------------------------------------------------------
// Static hexagram table — all 64 hexagrams in King Wen sequence
// ---------------------------------------------------------------------------

macro_rules! hex {
    ($num:expr, $en:expr, $zh:expr, $upper:ident, $lower:ident,
     [$l1:ident,$l2:ident,$l3:ident,$l4:ident,$l5:ident,$l6:ident],
     $judgment:expr, $image:expr) => {
        Hexagram {
            number: $num,
            name_en: $en,
            name_zh: $zh,
            upper_trigram: Trigram::$upper,
            lower_trigram: Trigram::$lower,
            lines: [
                Line::$l1,
                Line::$l2,
                Line::$l3,
                Line::$l4,
                Line::$l5,
                Line::$l6,
            ],
            judgment: $judgment,
            image: $image,
        }
    };
}

static HEXAGRAMS: [Hexagram; 64] = [
    hex!(
        1,
        "The Creative",
        "乾",
        Qian,
        Qian,
        [Yang, Yang, Yang, Yang, Yang, Yang],
        "Sublime success. Perseverance furthers.",
        "Heaven above heaven: the image of the Creative."
    ),
    hex!(
        2,
        "The Receptive",
        "坤",
        Kun,
        Kun,
        [Yin, Yin, Yin, Yin, Yin, Yin],
        "Sublime success. The perseverance of a mare furthers.",
        "Earth above earth: the image of the Receptive."
    ),
    hex!(
        3,
        "Difficulty at the Beginning",
        "屯",
        Kan,
        Zhen,
        [Yang, Yin, Yin, Yin, Yang, Yin],
        "Sublime success. Perseverance furthers. Appoint helpers.",
        "Clouds and thunder: the image of Difficulty at the Beginning."
    ),
    hex!(
        4,
        "Youthful Folly",
        "蒙",
        Gen,
        Kan,
        [Yin, Yang, Yin, Yin, Yin, Yang],
        "Success. It is not I who seek the young fool; the young fool seeks me.",
        "A spring at the foot of the mountain: the image of Youth."
    ),
    hex!(
        5,
        "Waiting",
        "需",
        Kan,
        Qian,
        [Yang, Yang, Yang, Yin, Yang, Yin],
        "Sincerity brings splendid success. Perseverance brings good fortune.",
        "Clouds rise up to heaven: the image of Waiting."
    ),
    hex!(
        6,
        "Conflict",
        "訟",
        Qian,
        Kan,
        [Yin, Yang, Yin, Yang, Yang, Yang],
        "Sincerity is obstructed. Caution brings good fortune.",
        "Heaven and water go their opposite ways: the image of Conflict."
    ),
    hex!(
        7,
        "The Army",
        "師",
        Kun,
        Kan,
        [Yin, Yang, Yin, Yin, Yin, Yin],
        "The army needs perseverance and a strong man. Good fortune without blame.",
        "Water in the midst of the earth: the image of the Army."
    ),
    hex!(
        8,
        "Holding Together",
        "比",
        Kan,
        Kun,
        [Yin, Yin, Yin, Yin, Yang, Yin],
        "Good fortune. Inquire of the oracle once more whether you possess sublimity.",
        "Water on the earth: the image of Holding Together."
    ),
    hex!(
        9,
        "The Taming Power of the Small",
        "小畜",
        Xun,
        Qian,
        [Yang, Yang, Yang, Yin, Yang, Yang],
        "Success. Dense clouds, no rain from the western region.",
        "Wind drives across heaven: the image of the Small Taming."
    ),
    hex!(
        10,
        "Treading",
        "履",
        Qian,
        Dui,
        [Yang, Yang, Yin, Yang, Yang, Yang],
        "Treading upon the tail of the tiger. It does not bite. Success.",
        "Heaven above, the lake below: the image of Treading."
    ),
    hex!(
        11,
        "Peace",
        "泰",
        Kun,
        Qian,
        [Yang, Yang, Yang, Yin, Yin, Yin],
        "The small departs, the great approaches. Good fortune. Success.",
        "Heaven and earth unite: the image of Peace."
    ),
    hex!(
        12,
        "Standstill",
        "否",
        Qian,
        Kun,
        [Yin, Yin, Yin, Yang, Yang, Yang],
        "Standstill. Evil people do not further the perseverance of the superior man.",
        "Heaven and earth do not unite: the image of Standstill."
    ),
    hex!(
        13,
        "Fellowship with Men",
        "同人",
        Qian,
        Li,
        [Yang, Yin, Yang, Yang, Yang, Yang],
        "Fellowship with men in the open. Success. Perseverance furthers.",
        "Heaven with fire: the image of Fellowship."
    ),
    hex!(
        14,
        "Possession in Great Measure",
        "大有",
        Li,
        Qian,
        [Yang, Yang, Yang, Yang, Yin, Yang],
        "Supreme success.",
        "Fire in heaven above: the image of Possession in Great Measure."
    ),
    hex!(
        15,
        "Modesty",
        "謙",
        Kun,
        Gen,
        [Yin, Yin, Yang, Yin, Yin, Yin],
        "Modesty creates success. The superior man carries things through.",
        "Within the earth a mountain: the image of Modesty."
    ),
    hex!(
        16,
        "Enthusiasm",
        "豫",
        Zhen,
        Kun,
        [Yin, Yin, Yin, Yang, Yin, Yin],
        "Enthusiasm. It furthers one to install helpers and to set armies marching.",
        "Thunder comes resounding out of the earth: the image of Enthusiasm."
    ),
    hex!(
        17,
        "Following",
        "隨",
        Dui,
        Zhen,
        [Yang, Yin, Yin, Yang, Yang, Yin],
        "Following has supreme success. Perseverance furthers. No blame.",
        "Thunder in the midst of the lake: the image of Following."
    ),
    hex!(
        18,
        "Work on What Has Been Spoiled",
        "蠱",
        Gen,
        Xun,
        [Yin, Yang, Yang, Yin, Yin, Yang],
        "Supreme success. It furthers one to cross the great water.",
        "Wind at the foot of the mountain: the image of Decay."
    ),
    hex!(
        19,
        "Approach",
        "臨",
        Kun,
        Dui,
        [Yang, Yang, Yin, Yin, Yin, Yin],
        "Approach has supreme success. Perseverance furthers.",
        "Earth above the lake: the image of Approach."
    ),
    hex!(
        20,
        "Contemplation",
        "觀",
        Xun,
        Kun,
        [Yin, Yin, Yin, Yin, Yang, Yang],
        "Contemplation. The ablution has been made, but not yet the offering.",
        "Wind blows over the earth: the image of Contemplation."
    ),
    hex!(
        21,
        "Biting Through",
        "噬嗑",
        Li,
        Zhen,
        [Yang, Yin, Yin, Yang, Yin, Yang],
        "Biting Through has success. It is favorable to let justice be administered.",
        "Thunder and lightning: the image of Biting Through."
    ),
    hex!(
        22,
        "Grace",
        "賁",
        Gen,
        Li,
        [Yang, Yin, Yang, Yin, Yin, Yang],
        "Grace has success. In small matters it is favorable to undertake something.",
        "Fire at the foot of the mountain: the image of Grace."
    ),
    hex!(
        23,
        "Splitting Apart",
        "剝",
        Gen,
        Kun,
        [Yin, Yin, Yin, Yin, Yin, Yang],
        "Splitting Apart. It does not further one to go anywhere.",
        "The mountain rests on the earth: the image of Splitting Apart."
    ),
    hex!(
        24,
        "Return",
        "復",
        Kun,
        Zhen,
        [Yang, Yin, Yin, Yin, Yin, Yin],
        "Return. Success. Going out and coming in without error.",
        "Thunder within the earth: the image of the Turning Point."
    ),
    hex!(
        25,
        "Innocence",
        "無妄",
        Qian,
        Zhen,
        [Yang, Yin, Yin, Yang, Yang, Yang],
        "Supreme success. Perseverance furthers.",
        "Under heaven thunder rolls: the image of the Unexpected."
    ),
    hex!(
        26,
        "The Taming Power of the Great",
        "大畜",
        Gen,
        Qian,
        [Yang, Yang, Yang, Yin, Yin, Yang],
        "Perseverance furthers. Not eating at home brings good fortune.",
        "Heaven within the mountain: the image of the Great Taming."
    ),
    hex!(
        27,
        "The Corners of the Mouth",
        "頤",
        Gen,
        Zhen,
        [Yang, Yin, Yin, Yin, Yin, Yang],
        "Perseverance brings good fortune. Pay heed to the providing of nourishment.",
        "Thunder at the foot of the mountain: the image of Nourishment."
    ),
    hex!(
        28,
        "Preponderance of the Great",
        "大過",
        Dui,
        Xun,
        [Yin, Yang, Yang, Yang, Yang, Yin],
        "The ridgepole sags to the breaking point. It furthers one to have somewhere to go.",
        "The lake rises above the trees: the image of the Great Preponderance."
    ),
    hex!(
        29,
        "The Abysmal",
        "坎",
        Kan,
        Kan,
        [Yin, Yang, Yin, Yin, Yang, Yin],
        "The Abysmal repeated. Sincerity brings success of the heart.",
        "Water flows on: the image of the Abysmal."
    ),
    hex!(
        30,
        "The Clinging",
        "離",
        Li,
        Li,
        [Yang, Yin, Yang, Yang, Yin, Yang],
        "The Clinging. Perseverance furthers. Success. Care of the cow brings good fortune.",
        "Brightness rises twice: the image of Fire."
    ),
    hex!(
        31,
        "Influence",
        "咸",
        Dui,
        Gen,
        [Yin, Yin, Yang, Yang, Yang, Yin],
        "Influence. Success. Perseverance furthers. To take a maiden to wife brings good fortune.",
        "A lake on the mountain: the image of Influence."
    ),
    hex!(
        32,
        "Duration",
        "恆",
        Zhen,
        Xun,
        [Yin, Yang, Yang, Yang, Yin, Yin],
        "Duration. Success. No blame. Perseverance furthers.",
        "Thunder and wind: the image of Duration."
    ),
    hex!(
        33,
        "Retreat",
        "遯",
        Qian,
        Gen,
        [Yin, Yin, Yang, Yang, Yang, Yang],
        "Retreat. Success. In what is small, perseverance furthers.",
        "Mountain under heaven: the image of Retreat."
    ),
    hex!(
        34,
        "The Power of the Great",
        "大壯",
        Zhen,
        Qian,
        [Yang, Yang, Yang, Yang, Yin, Yin],
        "The Power of the Great. Perseverance furthers.",
        "Thunder in heaven above: the image of the Power of the Great."
    ),
    hex!(
        35,
        "Progress",
        "晉",
        Li,
        Kun,
        [Yin, Yin, Yin, Yang, Yin, Yang],
        "Progress. The powerful prince is honored with horses in large numbers.",
        "Fire over the earth: the image of Progress."
    ),
    hex!(
        36,
        "Darkening of the Light",
        "明夷",
        Kun,
        Li,
        [Yang, Yin, Yang, Yin, Yin, Yin],
        "Darkening of the Light. In adversity it furthers one to be persevering.",
        "The light has sunk into the earth: the image of Darkening of the Light."
    ),
    hex!(
        37,
        "The Family",
        "家人",
        Xun,
        Li,
        [Yang, Yin, Yang, Yin, Yang, Yang],
        "The Family. The perseverance of the woman furthers.",
        "Wind comes forth from fire: the image of the Family."
    ),
    hex!(
        38,
        "Opposition",
        "睽",
        Li,
        Dui,
        [Yang, Yang, Yin, Yang, Yin, Yang],
        "Opposition. In small matters, good fortune.",
        "Fire above, the lake below: the image of Opposition."
    ),
    hex!(
        39,
        "Obstruction",
        "蹇",
        Kan,
        Gen,
        [Yin, Yin, Yang, Yin, Yang, Yin],
        "Obstruction. The southwest furthers. The northeast does not further.",
        "Water on the mountain: the image of Obstruction."
    ),
    hex!(
        40,
        "Deliverance",
        "解",
        Zhen,
        Kan,
        [Yin, Yang, Yin, Yang, Yin, Yin],
        "Deliverance. The southwest furthers. If there is no longer anything where one has to go, return brings good fortune.",
        "Thunder and rain set in: the image of Deliverance."
    ),
    hex!(
        41,
        "Decrease",
        "損",
        Gen,
        Dui,
        [Yang, Yang, Yin, Yin, Yin, Yang],
        "Decrease combined with sincerity brings about supreme good fortune.",
        "At the foot of the mountain, the lake: the image of Decrease."
    ),
    hex!(
        42,
        "Increase",
        "益",
        Xun,
        Zhen,
        [Yang, Yin, Yin, Yin, Yang, Yang],
        "Increase. It furthers one to undertake something. It furthers one to cross the great water.",
        "Wind and thunder: the image of Increase."
    ),
    hex!(
        43,
        "Breakthrough",
        "夬",
        Dui,
        Qian,
        [Yang, Yang, Yang, Yang, Yang, Yin],
        "Breakthrough. One must resolutely make the matter known at the court of the king.",
        "The lake has risen up to heaven: the image of Breakthrough."
    ),
    hex!(
        44,
        "Coming to Meet",
        "姤",
        Qian,
        Xun,
        [Yin, Yang, Yang, Yang, Yang, Yang],
        "Coming to Meet. The maiden is powerful. One should not marry such a maiden.",
        "Under heaven, wind: the image of Coming to Meet."
    ),
    hex!(
        45,
        "Gathering Together",
        "萃",
        Dui,
        Kun,
        [Yin, Yin, Yin, Yang, Yang, Yin],
        "Gathering Together. Success. The king approaches his temple. It furthers one to see the great man.",
        "Over the earth, the lake: the image of Gathering Together."
    ),
    hex!(
        46,
        "Pushing Upward",
        "升",
        Kun,
        Xun,
        [Yin, Yang, Yang, Yin, Yin, Yin],
        "Pushing Upward has supreme success. One must see the great man.",
        "Within the earth, wood grows: the image of Pushing Upward."
    ),
    hex!(
        47,
        "Oppression",
        "困",
        Dui,
        Kan,
        [Yin, Yang, Yin, Yang, Yang, Yin],
        "Oppression. Success. Perseverance. The great man brings about good fortune.",
        "There is no water in the lake: the image of Exhaustion."
    ),
    hex!(
        48,
        "The Well",
        "井",
        Kan,
        Xun,
        [Yin, Yang, Yang, Yin, Yang, Yin],
        "The Well. The town may be changed but the well cannot be changed.",
        "Water over wood: the image of the Well."
    ),
    hex!(
        49,
        "Revolution",
        "革",
        Dui,
        Li,
        [Yang, Yin, Yang, Yang, Yang, Yin],
        "Revolution. On your own day you are believed. Supreme success.",
        "Fire in the lake: the image of Revolution."
    ),
    hex!(
        50,
        "The Cauldron",
        "鼎",
        Li,
        Xun,
        [Yin, Yang, Yang, Yang, Yin, Yang],
        "The Cauldron. Supreme good fortune. Success.",
        "Fire over wood: the image of the Cauldron."
    ),
    hex!(
        51,
        "The Arousing",
        "震",
        Zhen,
        Zhen,
        [Yang, Yin, Yin, Yang, Yin, Yin],
        "Shock brings success. Shock comes — oh, oh! Laughing words — ha, ha!",
        "Thunder repeated: the image of Shock."
    ),
    hex!(
        52,
        "Keeping Still",
        "艮",
        Gen,
        Gen,
        [Yin, Yin, Yang, Yin, Yin, Yang],
        "Keeping Still. Keeping his back still so that he no longer feels his body.",
        "Mountains standing close together: the image of Keeping Still."
    ),
    hex!(
        53,
        "Development",
        "漸",
        Xun,
        Gen,
        [Yin, Yin, Yang, Yin, Yang, Yang],
        "Development. The maiden is given in marriage. Good fortune. Perseverance furthers.",
        "On the mountain a tree: the image of Development."
    ),
    hex!(
        54,
        "The Marrying Maiden",
        "歸妹",
        Zhen,
        Dui,
        [Yang, Yang, Yin, Yang, Yin, Yin],
        "The Marrying Maiden. Undertakings bring misfortune. Nothing that would further.",
        "Thunder over the lake: the image of the Marrying Maiden."
    ),
    hex!(
        55,
        "Abundance",
        "豐",
        Zhen,
        Li,
        [Yang, Yin, Yang, Yang, Yin, Yin],
        "Abundance has success. The king attains abundance. Be not sad; be like the sun at midday.",
        "Both thunder and lightning come: the image of Abundance."
    ),
    hex!(
        56,
        "The Wanderer",
        "旅",
        Li,
        Gen,
        [Yin, Yin, Yang, Yang, Yin, Yang],
        "The Wanderer. Success through smallness. Perseverance brings good fortune to the wanderer.",
        "Fire on the mountain: the image of the Wanderer."
    ),
    hex!(
        57,
        "The Gentle",
        "巽",
        Xun,
        Xun,
        [Yin, Yang, Yang, Yin, Yang, Yang],
        "The Gentle. Success through what is small. It furthers one to have somewhere to go.",
        "Winds following one upon the other: the image of the Gently Penetrating."
    ),
    hex!(
        58,
        "The Joyous",
        "兌",
        Dui,
        Dui,
        [Yang, Yang, Yin, Yang, Yang, Yin],
        "The Joyous. Success. Perseverance is favorable.",
        "Lakes resting one on the other: the image of the Joyous."
    ),
    hex!(
        59,
        "Dispersion",
        "渙",
        Xun,
        Kan,
        [Yin, Yang, Yin, Yin, Yang, Yang],
        "Dispersion. Success. The king approaches his temple. It furthers one to cross the great water.",
        "Wind drives over water: the image of Dispersion."
    ),
    hex!(
        60,
        "Limitation",
        "節",
        Kan,
        Dui,
        [Yang, Yang, Yin, Yin, Yang, Yin],
        "Limitation. Success. Galling limitation must not be persevered in.",
        "Water over lake: the image of Limitation."
    ),
    hex!(
        61,
        "Inner Truth",
        "中孚",
        Xun,
        Dui,
        [Yang, Yang, Yin, Yin, Yang, Yang],
        "Inner Truth. Pigs and fishes. Good fortune. It furthers one to cross the great water.",
        "Wind over lake: the image of Inner Truth."
    ),
    hex!(
        62,
        "Preponderance of the Small",
        "小過",
        Zhen,
        Gen,
        [Yin, Yin, Yang, Yang, Yin, Yin],
        "Preponderance of the Small. Success. Perseverance furthers. Small things may be done.",
        "Thunder on the mountain: the image of Preponderance of the Small."
    ),
    hex!(
        63,
        "After Completion",
        "既濟",
        Kan,
        Li,
        [Yang, Yin, Yang, Yin, Yang, Yin],
        "After Completion. Success in small matters. Perseverance furthers.",
        "Water over fire: the image of After Completion."
    ),
    hex!(
        64,
        "Before Completion",
        "未濟",
        Li,
        Kan,
        [Yin, Yang, Yin, Yang, Yin, Yang],
        "Before Completion. Success. But if the little fox, after nearly completing the crossing, gets his tail in the water, there is nothing that would further.",
        "Fire over water: the image of Before Completion."
    ),
];

// ---------------------------------------------------------------------------
// Lookup table: upper trigram x lower trigram -> King Wen hexagram number
// ---------------------------------------------------------------------------

/// King Wen hexagram lookup: `KING_WEN_TABLE[upper][lower]` gives the hexagram
/// number (1-64) for the given trigram pair.
///
/// Row = upper trigram index, Column = lower trigram index.
/// Ordering: Qian(0) Kun(1) Zhen(2) Kan(3) Gen(4) Xun(5) Li(6) Dui(7).
static KING_WEN_TABLE: [[u8; 8]; 8] = [
    // Upper Qian
    [1, 12, 25, 6, 33, 44, 13, 10],
    // Upper Kun
    [11, 2, 24, 7, 15, 46, 36, 19],
    // Upper Zhen
    [34, 16, 51, 40, 62, 32, 55, 54],
    // Upper Kan
    [5, 8, 3, 29, 39, 48, 63, 60],
    // Upper Gen
    [26, 23, 27, 4, 52, 18, 22, 41],
    // Upper Xun
    [9, 20, 42, 59, 53, 57, 37, 61],
    // Upper Li
    [14, 35, 21, 64, 56, 50, 30, 38],
    // Upper Dui
    [43, 45, 17, 47, 31, 28, 49, 58],
];

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Retrieve a hexagram by its King Wen sequence number (1-64).
///
/// Returns `None` if `number` is 0 or greater than 64.
pub fn hexagram(number: u8) -> Option<&'static Hexagram> {
    if (1..=64).contains(&number) {
        Some(&HEXAGRAMS[(number - 1) as usize])
    } else {
        None
    }
}

/// Look up the hexagram formed by the given upper and lower trigrams.
pub fn hexagram_from_trigrams(upper: Trigram, lower: Trigram) -> &'static Hexagram {
    let number = KING_WEN_TABLE[upper as usize][lower as usize];
    // Safety: KING_WEN_TABLE values are always 1-64 by construction
    hexagram(number).expect("KING_WEN_TABLE contains only valid 1-64 values")
}

/// Derive a hexagram reading from a date/time.
///
/// The method used (Mei Hua Yi Shu — Plum Blossom Numerology):
/// - Upper trigram: `(year + month + day) % 8`
/// - Lower trigram: `(year + month + day + hour) % 8`
/// - Changing line:  `(year + month + day + hour) % 6`
pub fn hexagram_from_date(year: i32, month: u32, day: u32, hour: u32) -> HexagramReading {
    let sum_upper = (year.unsigned_abs() as u64)
        .wrapping_add(month as u64)
        .wrapping_add(day as u64);
    let sum_lower = sum_upper.wrapping_add(hour as u64);

    let upper_idx = (sum_upper % 8) as usize;
    let lower_idx = (sum_lower % 8) as usize;
    let changing = (sum_lower % 6) as usize;

    let upper = Trigram::from_index(upper_idx);
    let lower = Trigram::from_index(lower_idx);

    let primary = hexagram_from_trigrams(upper, lower);
    let relating = relating_hexagram_inner(&primary.lines, changing);

    HexagramReading {
        primary,
        changing_line: changing,
        relating,
    }
}

/// Compute the nuclear hexagram.
///
/// The nuclear hexagram is formed by:
/// - Inner lower trigram: lines 2, 3, 4 (indices 1, 2, 3)
/// - Inner upper trigram: lines 3, 4, 5 (indices 2, 3, 4)
pub fn nuclear_hexagram(hex: &Hexagram) -> &'static Hexagram {
    let inner_lower = trigram_from_lines([
        hex.lines[1].is_yang(),
        hex.lines[2].is_yang(),
        hex.lines[3].is_yang(),
    ]);
    let inner_upper = trigram_from_lines([
        hex.lines[2].is_yang(),
        hex.lines[3].is_yang(),
        hex.lines[4].is_yang(),
    ]);
    hexagram_from_trigrams(inner_upper, inner_lower)
}

/// Compute the relating hexagram by flipping the changing line.
///
/// `changing_line` is 0-5 (bottom to top). Values >= 6 are clamped to 5.
pub fn relating_hexagram(hex: &Hexagram, changing_line: usize) -> &'static Hexagram {
    let changing_line = changing_line.min(5);
    relating_hexagram_inner(&hex.lines, changing_line)
}

fn relating_hexagram_inner(lines: &[Line; 6], changing_line: usize) -> &'static Hexagram {
    let mut new_lines = *lines;
    new_lines[changing_line] = new_lines[changing_line].flip();

    let lower = trigram_from_lines([
        new_lines[0].is_yang(),
        new_lines[1].is_yang(),
        new_lines[2].is_yang(),
    ]);
    let upper = trigram_from_lines([
        new_lines[3].is_yang(),
        new_lines[4].is_yang(),
        new_lines[5].is_yang(),
    ]);
    hexagram_from_trigrams(upper, lower)
}

/// Return the opposite hexagram (all lines flipped).
pub fn opposite_hexagram(hex: &Hexagram) -> &'static Hexagram {
    let new_lines: [bool; 3] = [
        !hex.lines[0].is_yang(),
        !hex.lines[1].is_yang(),
        !hex.lines[2].is_yang(),
    ];
    let new_upper: [bool; 3] = [
        !hex.lines[3].is_yang(),
        !hex.lines[4].is_yang(),
        !hex.lines[5].is_yang(),
    ];
    let lower = trigram_from_lines(new_lines);
    let upper = trigram_from_lines(new_upper);
    hexagram_from_trigrams(upper, lower)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_64_hexagrams_present() {
        assert_eq!(HEXAGRAMS.len(), 64);
        for (i, h) in HEXAGRAMS.iter().enumerate() {
            assert_eq!(
                h.number,
                (i + 1) as u8,
                "Hexagram at index {i} has wrong number"
            );
        }
    }

    #[test]
    fn hexagram_lookup_by_number() {
        let h1 = hexagram(1).unwrap();
        assert_eq!(h1.name_en, "The Creative");
        assert_eq!(h1.name_zh, "乾");
        assert_eq!(h1.upper_trigram, Trigram::Qian);
        assert_eq!(h1.lower_trigram, Trigram::Qian);

        let h64 = hexagram(64).unwrap();
        assert_eq!(h64.name_en, "Before Completion");
        assert_eq!(h64.name_zh, "未濟");
    }

    #[test]
    fn hexagram_zero_returns_none() {
        assert!(hexagram(0).is_none());
    }

    #[test]
    fn hexagram_65_returns_none() {
        assert!(hexagram(65).is_none());
    }

    #[test]
    fn trigram_from_lines_roundtrip() {
        for t in Trigram::ALL {
            let lines = t.lines();
            let bools = [lines[0].is_yang(), lines[1].is_yang(), lines[2].is_yang()];
            assert_eq!(trigram_from_lines(bools), t, "Roundtrip failed for {:?}", t);
        }
    }

    #[test]
    fn all_8_trigrams_have_names() {
        for t in Trigram::ALL {
            assert!(!t.name_en().is_empty());
            assert!(!t.name_zh().is_empty());
            assert!(!t.attribute().is_empty());
        }
    }

    #[test]
    fn trigram_symbols() {
        assert_eq!(Trigram::Qian.symbol(), '☰');
        assert_eq!(Trigram::Kun.symbol(), '☷');
        assert_eq!(Trigram::Li.symbol(), '☲');
    }

    #[test]
    fn hexagram_from_trigrams_creative() {
        let h = hexagram_from_trigrams(Trigram::Qian, Trigram::Qian);
        assert_eq!(h.number, 1);
        assert_eq!(h.name_en, "The Creative");
    }

    #[test]
    fn hexagram_from_trigrams_receptive() {
        let h = hexagram_from_trigrams(Trigram::Kun, Trigram::Kun);
        assert_eq!(h.number, 2);
        assert_eq!(h.name_en, "The Receptive");
    }

    #[test]
    fn hexagram_from_trigrams_all_64_covered() {
        let mut seen = [false; 64];
        for upper in Trigram::ALL {
            for lower in Trigram::ALL {
                let h = hexagram_from_trigrams(upper, lower);
                assert!(h.number >= 1 && h.number <= 64);
                seen[(h.number - 1) as usize] = true;
            }
        }
        for (i, &s) in seen.iter().enumerate() {
            assert!(
                s,
                "Hexagram {} was never produced by trigram pair lookup",
                i + 1
            );
        }
    }

    #[test]
    fn hexagram_lines_match_trigrams() {
        for h in HEXAGRAMS.iter() {
            let lower_lines = h.lower_trigram.lines();
            let upper_lines = h.upper_trigram.lines();
            assert_eq!(
                h.lines[0], lower_lines[0],
                "Hex {} line 1 mismatch",
                h.number
            );
            assert_eq!(
                h.lines[1], lower_lines[1],
                "Hex {} line 2 mismatch",
                h.number
            );
            assert_eq!(
                h.lines[2], lower_lines[2],
                "Hex {} line 3 mismatch",
                h.number
            );
            assert_eq!(
                h.lines[3], upper_lines[0],
                "Hex {} line 4 mismatch",
                h.number
            );
            assert_eq!(
                h.lines[4], upper_lines[1],
                "Hex {} line 5 mismatch",
                h.number
            );
            assert_eq!(
                h.lines[5], upper_lines[2],
                "Hex {} line 6 mismatch",
                h.number
            );
        }
    }

    #[test]
    fn nuclear_hexagram_creative() {
        // Creative (all yang) -> nuclear should also be Creative
        let h = hexagram(1).unwrap();
        let nuc = nuclear_hexagram(h);
        assert_eq!(nuc.number, 1, "Nuclear of Creative should be Creative");
    }

    #[test]
    fn nuclear_hexagram_receptive() {
        // Receptive (all yin) -> nuclear should also be Receptive
        let h = hexagram(2).unwrap();
        let nuc = nuclear_hexagram(h);
        assert_eq!(nuc.number, 2, "Nuclear of Receptive should be Receptive");
    }

    #[test]
    fn nuclear_hexagram_hex3() {
        // Hex 3 (Difficulty): lines [Yang,Yin,Yin,Yin,Yang,Yin] (indices 0-5)
        // Inner lower (lines 1,2,3) = Yin,Yin,Yin = Kun
        // Inner upper (lines 2,3,4) = Yin,Yin,Yang = Gen
        // Gen over Kun = hex 23 (Splitting Apart)
        let h = hexagram(3).unwrap();
        let nuc = nuclear_hexagram(h);
        let expected = hexagram_from_trigrams(Trigram::Gen, Trigram::Kun);
        assert_eq!(
            nuc.number, expected.number,
            "Nuclear of hex 3 should be Gen/Kun = {}",
            expected.number
        );
    }

    #[test]
    fn relating_hexagram_flip_line_0() {
        // Creative (all yang), flip line 0 -> bottom trigram becomes Dui
        // Qian over Dui = hex 10 (Treading)
        let h = hexagram(1).unwrap();
        let rel = relating_hexagram(h, 0);
        assert_eq!(
            rel.number, 44,
            "Flipping line 1 of Creative: Qian/Xun = hex 44 Coming to Meet"
        );
    }

    #[test]
    fn relating_hexagram_flip_line_5() {
        // Creative (all yang), flip line 5 -> upper trigram becomes Dui
        // Dui over Qian = hex 43 (Breakthrough)
        let h = hexagram(1).unwrap();
        let rel = relating_hexagram(h, 5);
        assert_eq!(
            rel.number, 43,
            "Flipping line 6 of Creative: Dui/Qian = hex 43 Breakthrough"
        );
    }

    #[test]
    fn hexagram_from_date_produces_valid_reading() {
        let reading = hexagram_from_date(2024, 6, 15, 10);
        assert!(reading.primary.number >= 1 && reading.primary.number <= 64);
        assert!(reading.relating.number >= 1 && reading.relating.number <= 64);
        assert!(reading.changing_line < 6);
    }

    #[test]
    fn hexagram_from_date_deterministic() {
        let r1 = hexagram_from_date(2024, 6, 15, 10);
        let r2 = hexagram_from_date(2024, 6, 15, 10);
        assert_eq!(r1.primary.number, r2.primary.number);
        assert_eq!(r1.relating.number, r2.relating.number);
        assert_eq!(r1.changing_line, r2.changing_line);
    }

    #[test]
    fn hexagram_from_date_different_hours_differ() {
        let r1 = hexagram_from_date(2024, 1, 1, 0);
        let r2 = hexagram_from_date(2024, 1, 1, 3);
        // Different hours should produce different lower trigram or changing line
        assert!(
            r1.primary.number != r2.primary.number || r1.changing_line != r2.changing_line,
            "Different hours should generally produce different readings"
        );
    }

    #[test]
    fn opposite_hexagram_creative_is_receptive() {
        let h = hexagram(1).unwrap();
        let opp = opposite_hexagram(h);
        assert_eq!(opp.number, 2, "Opposite of Creative should be Receptive");
    }

    #[test]
    fn opposite_hexagram_receptive_is_creative() {
        let h = hexagram(2).unwrap();
        let opp = opposite_hexagram(h);
        assert_eq!(opp.number, 1, "Opposite of Receptive should be Creative");
    }

    #[test]
    fn opposite_hexagram_involutory() {
        // For every hexagram, opposite(opposite(h)) == h
        for n in 1..=64 {
            let h = hexagram(n).unwrap();
            let opp = opposite_hexagram(h);
            let opp2 = opposite_hexagram(opp);
            assert_eq!(
                opp2.number, h.number,
                "Double opposite of hex {} should be itself, got {}",
                n, opp2.number
            );
        }
    }

    #[test]
    fn line_flip() {
        assert_eq!(Line::Yang.flip(), Line::Yin);
        assert_eq!(Line::Yin.flip(), Line::Yang);
    }

    #[test]
    fn trigram_from_index_wraps() {
        assert_eq!(Trigram::from_index(0), Trigram::Qian);
        assert_eq!(Trigram::from_index(8), Trigram::Qian);
        assert_eq!(Trigram::from_index(15), Trigram::Dui);
    }

    #[test]
    fn king_wen_table_consistency() {
        // Every hexagram in the table should match the trigrams stored in the hexagram struct
        for upper in Trigram::ALL {
            for lower in Trigram::ALL {
                let h = hexagram_from_trigrams(upper, lower);
                assert_eq!(
                    h.upper_trigram, upper,
                    "Hex {} upper trigram mismatch",
                    h.number
                );
                assert_eq!(
                    h.lower_trigram, lower,
                    "Hex {} lower trigram mismatch",
                    h.number
                );
            }
        }
    }
}
