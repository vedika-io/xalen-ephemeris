//! # I Ching (Yijing) — Book of Changes
//!
//! Complete implementation of the 64 hexagram system with 8 trigrams (Ba Gua),
//! date-based hexagram derivation, nuclear hexagrams, and relating hexagrams.
//!
//! ## Text source
//! The judgment (Thwan / T'uan) and image (Great Symbolism / Xiang) text for all
//! 64 hexagrams is taken verbatim from James Legge, *The Yî King*, Sacred Books of
//! the East Vol. XVI (1882) — public domain. Legge's editorial parentheticals
//! (e.g. "(represents)") and diacritics (ă, ĕ, ǔ, etc.) are preserved as published.

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

    /// Map a Pre-Heaven (Xian Tian / Fu Xi) Ba Gua sequence number to a trigram.
    ///
    /// This is the numbering used by Mei Hua Yi Shu (Plum Blossom Numerology):
    ///   1=Qian ☰, 2=Dui ☱, 3=Li ☲, 4=Zhen ☳, 5=Xun ☴, 6=Kan ☵, 7=Gen ☶, 8=Kun ☷.
    ///
    /// The input is reduced `mod 8`; a remainder of 0 maps to the 8th trigram,
    /// Kun, exactly as in the classical method where the divisor 8 yields Kun.
    /// Reference: Shao Yong, *Mei Hua Yi Shu* (Plum Blossom Numerology); the
    /// Pre-Heaven (先天八卦) sequence Qian–Dui–Li–Zhen–Xun–Kan–Gen–Kun.
    pub fn from_pre_heaven_number(n: u64) -> Trigram {
        // Reduce to 1..=8 (remainder 0 -> 8 = Kun).
        let r = n % 8;
        match r {
            1 => Trigram::Qian,
            2 => Trigram::Dui,
            3 => Trigram::Li,
            4 => Trigram::Zhen,
            5 => Trigram::Xun,
            6 => Trigram::Kan,
            7 => Trigram::Gen,
            _ => Trigram::Kun, // r == 0  ->  8th trigram
        }
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

impl Hexagram {
    /// Legge's six line texts (yao ci) for this hexagram, bottom line (index 0)
    /// to top line (index 5). Verbatim public-domain Legge (SBE XVI, 1882).
    pub fn line_texts(&self) -> &'static [&'static str; 6] {
        // `self.number` is always 1..=64 for any hexagram from this table.
        line_texts(self.number).expect("hexagram number is always 1-64")
    }

    /// Legge's line text (yao ci) for a single line of this hexagram, `line`
    /// being the 0-based index from the bottom (0 = first line, 5 = top line).
    /// Returns `None` if `line >= 6`. Verbatim public-domain Legge (SBE XVI).
    pub fn line_text(&self, line: usize) -> Option<&'static str> {
        line_text(self.number, line)
    }

    /// Legge's supplementary "use of the number" statement, present only for
    /// Hexagram 1 and Hexagram 2; `None` for all others. See [`use_line_text`].
    pub fn use_line_text(&self) -> Option<&'static str> {
        use_line_text(self.number)
    }
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
    /// Legge's line text (yao ci) for the changing line of the primary
    /// hexagram — i.e. `line_text(primary.number, changing_line)`. This is the
    /// statement traditionally consulted when a single line is moving.
    /// Verbatim from James Legge, *The Yî King* (SBE XVI, 1882), public domain.
    pub changing_line_text: &'static str,
}

// ---------------------------------------------------------------------------
// Static hexagram table — all 64 hexagrams in King Wen sequence
//
// I-Ching judgment (Thwan) and image (Great Symbolism) text from James Legge,
// The Yî King, Sacred Books of the East Vol. XVI (1882) — public domain.
// Copied verbatim; Legge's parentheticals and diacritics are intentional.
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
        "Khien (represents) what is great and originating, penetrating, advantageous, correct and firm.",
        "Heaven, in its motion, (gives the idea of) strength. The superior man, in accordance with this, nerves himself to ceaseless activity."
    ),
    hex!(
        2,
        "The Receptive",
        "坤",
        Kun,
        Kun,
        [Yin, Yin, Yin, Yin, Yin, Yin],
        "Khwăn (represents) what is great and originating, penetrating, advantageous, correct and having the firmness of a mare. When the superior man (here intended) has to make any movement, if he take the initiative, he will go astray; if he follow, he will find his (proper) lord. The advantageousness will be seen in his getting friends in the south-west, and losing friends in the north-east. If he rest in correctness and firmness, there will be good fortune.",
        "The (capacity and sustaining) power of the earth is what is denoted by Khwăn. The superior man, in accordance with this, with his large virtue supports (men and) things."
    ),
    hex!(
        3,
        "Difficulty at the Beginning",
        "屯",
        Kan,
        Zhen,
        [Yang, Yin, Yin, Yin, Yang, Yin],
        "Kun (indicates that in the case which it presupposes) there will be great progress and success, and the advantage will come from being correct and firm. (But) any movement in advance should not be (lightly) undertaken. There will be advantage in appointing feudal princes.",
        "(The trigram representing) clouds and (that representing) thunder form Kun. The superior man, in accordance with this, (adjusts his measures of government) as in sorting the threads of the warp and woof."
    ),
    hex!(
        4,
        "Youthful Folly",
        "蒙",
        Gen,
        Kan,
        [Yin, Yang, Yin, Yin, Yin, Yang],
        "Măng (indicates that in the case which it presupposes) there will be progress and success. I do not (go and) seek the youthful and inexperienced, but he comes and seeks me. When he shows (the sincerity that marks) the first recourse to divination, I instruct him. If he apply a second and third time, that is troublesome; and I do not instruct the troublesome. There will be advantage in being firm and correct.",
        "(The trigram representing) a mountain, and beneath it that for a spring issuing forth form Măng. The superior man, in accordance with this, strives to be resolute in his conduct and nourishes his virtue."
    ),
    hex!(
        5,
        "Waiting",
        "需",
        Kan,
        Qian,
        [Yang, Yang, Yang, Yin, Yang, Yin],
        "Hsü intimates that, with the sincerity which is declared in it, there will be brilliant success. With firmness there will be good fortune; and it will be advantageous to cross the great stream.",
        "(The trigram for) clouds ascending over that for the sky forms Hsü. The superior man, in accordance with this, eats and drinks, feasts and enjoys himself (as if there were nothing else to employ him)."
    ),
    hex!(
        6,
        "Conflict",
        "訟",
        Qian,
        Kan,
        [Yin, Yang, Yin, Yang, Yang, Yang],
        "Sung intimates how, though there is sincerity in one's contention, he will yet meet with opposition and obstruction; but if he cherish an apprehensive caution, there will be good fortune, while, if he must prosecute the contention to the (bitter) end, there will be evil. It will be advantageous to see the great man; it will not be advantageous to cross the great stream.",
        "(The trigram representing) heaven and (that representing) water, moving away from each other, form Sung. The superior man, in accordance with this, in the transaction of affairs takes good counsel about his first steps."
    ),
    hex!(
        7,
        "The Army",
        "師",
        Kun,
        Kan,
        [Yin, Yang, Yin, Yin, Yin, Yin],
        "Sze indicates how, in the case which it supposes, with firmness and correctness, and (a leader of) age and experience, there will be good fortune and no error.",
        "(The trigram representing) the earth and in the midst of it that representing water, form Sze. The superior man, in accordance with this, nourishes and educates the people, and collects (from among them) the multitudes (of the hosts)."
    ),
    hex!(
        8,
        "Holding Together",
        "比",
        Kan,
        Kun,
        [Yin, Yin, Yin, Yin, Yang, Yin],
        "Pî indicates that (under the conditions which it supposes) there is good fortune. But let (the principal party intended in it) re-examine himself, (as if) by divination, whether his virtue be great, unintermitting, and firm. If it be so, there will be no error. Those who have not rest will then come to him; and with those who are (too) late in coming it will be ill.",
        "(The trigram representing) the earth, and over it (that representing) water, form Pî. The ancient kings, in accordance with this, established the various states and maintained an affectionate relation to their princes."
    ),
    hex!(
        9,
        "The Taming Power of the Small",
        "小畜",
        Xun,
        Qian,
        [Yang, Yang, Yang, Yin, Yang, Yang],
        "Hsiâo Khû indicates that (under its conditions) there will be progress and success. (We see) dense clouds, but no rain coming from our borders in the west.",
        "(The trigram representing) the sky, and that representing wind moving above it, form Hsiâo Khû The superior man, in accordance with this, adorns the outward manifestation of his virtue."
    ),
    hex!(
        10,
        "Treading",
        "履",
        Qian,
        Dui,
        [Yang, Yang, Yin, Yang, Yang, Yang],
        "(Lî suggests the idea of) one treading on the tail of a tiger, which does not bite him. There will be progress and success.",
        "(The trigram representing) the sky above, and below it (that representing the waters of) a marsh, form Lî. The superior man, in accordance with this, discriminates between high and low, and gives settlement to the aims of the people."
    ),
    hex!(
        11,
        "Peace",
        "泰",
        Kun,
        Qian,
        [Yang, Yang, Yang, Yin, Yin, Yin],
        "In Thâi (we see) the little gone and the great come. (It indicates that) there will be good fortune, with progress and success.",
        "(The trigrams for) heaven and earth in communication together form Thâi. The (sage) sovereign, in harmony with this, fashions and completes (his regulations) after the courses of heaven and earth, and assists the application of the adaptations furnished by them,--in order to benefit the people."
    ),
    hex!(
        12,
        "Standstill",
        "否",
        Qian,
        Kun,
        [Yin, Yin, Yin, Yang, Yang, Yang],
        "In Phî there is the want of good understanding between the (different classes of) men, and its indication is unfavourable to the firm and correct course of the superior man. We see in it the great gone and the little come.",
        "(The trigrams of) heaven and earth, not in intercommunication, form Phî. The superior man, in accordance with this, restrains (the manifestation of) his virtue, and avoids the calamities (that threaten him). There is no opportunity of conferring on him the glory of emolument."
    ),
    hex!(
        13,
        "Fellowship with Men",
        "同人",
        Qian,
        Li,
        [Yang, Yin, Yang, Yang, Yang, Yang],
        "Thung Zăn (or 'Union of men') appears here (as we find it) in the (remote districts of the) country, indicating progress and success. It will be advantageous to cross the great stream. It will be advantageous to maintain the firm correctness of the superior man.",
        "(The trigrams for) heaven and fire form Thung Zăn. The superior man, in accordance with this, distinguishes things according to their kinds and classes."
    ),
    hex!(
        14,
        "Possession in Great Measure",
        "大有",
        Li,
        Qian,
        [Yang, Yang, Yang, Yang, Yin, Yang],
        "Tâ Yû indicates that, (under the circumstances which it implies), there will be great progress and success.",
        "(The trigram for) heaven and (that of) fire above it form Tâ Yû The superior man, in accordance with this, represses what is evil and gives distinction to what is good, in sympathy with the excellent Heaven-conferred (nature)."
    ),
    hex!(
        15,
        "Modesty",
        "謙",
        Kun,
        Gen,
        [Yin, Yin, Yang, Yin, Yin, Yin],
        "Khien indicates progress and success. The superior man, (being humble as it implies), will have a (good) issue (to his undertakings).",
        "(The trigram for) the earth and (that of) a mountain in the midst of it form Khien. The superior man, in accordance with this, diminishes what is excessive (in himself), and increases where there is any defect, bringing about an equality, according to the nature of the case, in his treatment (of himself and others)."
    ),
    hex!(
        16,
        "Enthusiasm",
        "豫",
        Zhen,
        Kun,
        [Yin, Yin, Yin, Yang, Yin, Yin],
        "Yü indicates that, (in the state which it implies), feudal princes may be set up, and the hosts put in motion, with advantage.",
        "(The trigrams for) the earth and thunder issuing from it with its crashing noise form Yü. The ancient kings, in accordance with this, composed their music and did honour to virtue, presenting it especially and most grandly to God, when they associated with Him (at the service) their highest ancestor and their father."
    ),
    hex!(
        17,
        "Following",
        "隨",
        Dui,
        Zhen,
        [Yang, Yin, Yin, Yang, Yang, Yin],
        "Sui indicates that (under its conditions) there will be great progress and success. But it will be advantageous to be firm and correct. There will (then) be no error.",
        "(The trigram for the waters of) a marsh and (that for) thunder (hidden) in the midst of it form Sui. The superior man in accordance with this, when it is getting towards dark, enters (his house) and rests."
    ),
    hex!(
        18,
        "Work on What Has Been Spoiled",
        "蠱",
        Gen,
        Xun,
        [Yin, Yang, Yang, Yin, Yin, Yang],
        "Kû indicates great progress and success (to him who deals properly with the condition represented by it). There will be advantage in (efforts like that of) crossing the great stream. (He should weigh well, however, the events of) three days before the turning point, and those (to be done) three days after it.",
        "(The trigram for) a mountain, and below it that for wind, form Kû. The superior man, in accordance with this, (addresses himself to) help the people and nourish his own virtue."
    ),
    hex!(
        19,
        "Approach",
        "臨",
        Kun,
        Dui,
        [Yang, Yang, Yin, Yin, Yin, Yin],
        "Lin (indicates that under the conditions supposed in it) there will be great progress and success, while it will be advantageous to be firmly correct. In the eighth month there will be evil.",
        "(The trigram for) the waters of a marsh and that for the earth above it form Lin. The superior man, in accordance with this, has his purposes of instruction that are inexhaustible, and nourishes and supports the people without limit."
    ),
    hex!(
        20,
        "Contemplation",
        "觀",
        Xun,
        Kun,
        [Yin, Yin, Yin, Yin, Yang, Yang],
        "Kwân shows (how he whom it represents should be like) the worshipper who has washed his hands, but not (yet) presented his offerings;--with sincerity and an appearance of dignity (commanding reverent regard).",
        "(The trigram representing) the earth, and that for wind moving above it, form Kwan. The ancient kings, in accordance with this, examined the (different) regions (of the kingdom), to see the (ways of the) people, and set forth their instructions."
    ),
    hex!(
        21,
        "Biting Through",
        "噬嗑",
        Li,
        Zhen,
        [Yang, Yin, Yin, Yang, Yin, Yang],
        "Shih Ho indicates successful progress (in the condition of things which it supposes). It will be advantageous to use legal constraints.",
        "(The trigrams representing) thunder and lightning form Shih Ho. The ancient kings, in accordance with this, framed their penalties with intelligence, and promulgated their laws."
    ),
    hex!(
        22,
        "Grace",
        "賁",
        Gen,
        Li,
        [Yang, Yin, Yang, Yin, Yin, Yang],
        "Pî indicates that there should be free course (in what it denotes). There will be little advantage (however) if it be allowed to advance (and take the lead).",
        "(The trigram representing) a mountain and that for fire under it form Pî. The superior man, in accordance with this, throws a brilliancy around his various processes of government, but does not dare (in a similar way) to decide cases of criminal litigation."
    ),
    hex!(
        23,
        "Splitting Apart",
        "剝",
        Gen,
        Kun,
        [Yin, Yin, Yin, Yin, Yin, Yang],
        "Po indicates that (in the state which it symbolises) it will not be advantageous to make a movement in any direction whatever.",
        "(The trigrams representing) the earth, and (above it) that for a mountain, which adheres to the earth, form Po. Superiors, in accordance with this, seek to strengthen those below them, to secure the peace and stability of their own position."
    ),
    hex!(
        24,
        "Return",
        "復",
        Kun,
        Zhen,
        [Yang, Yin, Yin, Yin, Yin, Yin],
        "Fû indicates that there will be free course and progress (in what it denotes). (The subject of it) finds no one to distress him in his exits and entrances; friends come to him, and no error is committed. He will return and repeat his (proper) course. In seven days comes his return. There will be advantage in whatever direction movement is made.",
        "(The trigram representing) the earth and that for thunder in the midst of it form Fû. The ancient kings, in accordance with this, on the day of the (winter) solstice, shut the gates of the passes (from one state to another), so that the travelling merchants could not (then) pursue their journeys, nor the princes go on with the inspection of their states."
    ),
    hex!(
        25,
        "Innocence",
        "無妄",
        Qian,
        Zhen,
        [Yang, Yin, Yin, Yang, Yang, Yang],
        "Wû Wang indicates great progress and success, while there will be advantage in being firm and correct. If (its subject and his action) be not correct, he will fall into errors, and it will not be advantageous for him to move in any direction.",
        "The thunder rolls all under the sky, and to (every)thing there is given (its nature), free from all insincerity. The ancient kings, in accordance with this, (made their regulations) in complete accordance with the seasons, thereby nourishing all things."
    ),
    hex!(
        26,
        "The Taming Power of the Great",
        "大畜",
        Gen,
        Qian,
        [Yang, Yang, Yang, Yin, Yin, Yang],
        "Under the conditions of Tâ Khû it will be advantageous to be firm and correct. (If its subject do not seek to) enjoy his revenues in his own family (without taking service at court), there will be good fortune. It will be advantageous for him to cross the great stream.",
        "(The trigram representing) a mountain, and in the midst of it that (representing) heaven, form Tâ Khû. The superior man, in accordance with this, stores largely in his memory the words and deeds of former men, to subserve the accumulation of his virtue."
    ),
    hex!(
        27,
        "The Corners of the Mouth",
        "頤",
        Gen,
        Zhen,
        [Yang, Yin, Yin, Yin, Yin, Yang],
        "Î indicates that with firm correctness there will be good fortune (in what is denoted by it). We must look at what we are seeking to nourish, and by the exercise of our thoughts seek for the proper aliment.",
        "(The trigram representing) a mountain and under it that for thunder form Î. The superior man, in accordance with this, (enjoins) watchfulness over our words, and the temperate regulation of our eating and drinking."
    ),
    hex!(
        28,
        "Preponderance of the Great",
        "大過",
        Dui,
        Xun,
        [Yin, Yang, Yang, Yang, Yang, Yin],
        "Tâ Kwo suggests to us a beam that is weak. There will be advantage in moving (under its conditions) in any direction whatever; there will be success.",
        "(The trigram representing) trees hidden beneath that for the waters of a marsh forms Tâ Kwo. The superior man, in accordance with this, stands up alone and has no fear, and keeps retired from the world without regret."
    ),
    hex!(
        29,
        "The Abysmal",
        "坎",
        Kan,
        Kan,
        [Yin, Yang, Yin, Yin, Yang, Yin],
        "Khan, here repeated, shows the possession of sincerity, through which the mind is penetrating. Action (in accordance with this) will be of high value.",
        "(The representation of) water flowing on continuously forms the repeated Khan. The superior man, in accordance with this, maintains constantly the virtue (of his heart) and (the integrity of) his conduct, and practises the business of instruction."
    ),
    hex!(
        30,
        "The Clinging",
        "離",
        Li,
        Li,
        [Yang, Yin, Yang, Yang, Yin, Yang],
        "Lî indicates that, (in regard to what it denotes), it will be advantageous to be firm and correct, and that thus there will be free course and success. Let (its subject) also nourish (a docility like that of) the cow, and there will be good fortune.",
        "(The trigram for) brightness, repeated, forms Lî. The great man, in accordance with this, cultivates more and more his brilliant (virtue), and diffuses its brightness over the four quarters (of the land)."
    ),
    hex!(
        31,
        "Influence",
        "咸",
        Dui,
        Gen,
        [Yin, Yin, Yang, Yang, Yang, Yin],
        "Hsien indicates that, (on the fulfilment of the conditions implied in it), there will be free course and success. Its advantageousness will depend on the being firm and correct, (as) in marrying a young lady. There will be good fortune.",
        "(The trigram representing) a mountain and above it that for (the waters of) a marsh form Hsien. The superior man, in accordance with this, keeps his mind free from pre-occupation, and open to receive (the influences of) others."
    ),
    hex!(
        32,
        "Duration",
        "恆",
        Zhen,
        Xun,
        [Yin, Yang, Yang, Yang, Yin, Yin],
        "Hăng indicates successful progress and no error (in what it denotes). But the advantage will come from being firm and correct; and movement in any direction whatever will be advantageous.",
        "(The trigram representing) thunder and that for wind form Hăng. The superior man, in accordance with this, stands firm, and does not change his method (of operation)."
    ),
    hex!(
        33,
        "Retreat",
        "遯",
        Qian,
        Gen,
        [Yin, Yin, Yang, Yang, Yang, Yang],
        "Thun indicates successful progress (in its circumstances). To a small extent it will (still) be advantageous to be firm and correct.",
        "(The trigram representing) the sky and below it that for a mountain form Thun. The superior man, in accordance with this, keeps small men at a distance, not by showing that he hates them, but by his own dignified gravity."
    ),
    hex!(
        34,
        "The Power of the Great",
        "大壯",
        Zhen,
        Qian,
        [Yang, Yang, Yang, Yang, Yin, Yin],
        "Tâ Kwang indicates that (under the conditions which it symbolises) it will be advantageous to be firm and correct.",
        "(The trigram representing) heaven and above it that for thunder form Tâ Kwang. The superior man, in accordance with this, does not take a step which is not according to propriety."
    ),
    hex!(
        35,
        "Progress",
        "晉",
        Li,
        Kun,
        [Yin, Yin, Yin, Yang, Yin, Yang],
        "In Žin we see a prince who secures the tranquillity (of the people) presented on that account with numerous horses (by the king), and three times in a day received at interviews.",
        "(The trigram representing) the earth and that for the bright (sun) coming forth above it form Žin. The superior man, according to this, gives himself to make more brilliant his bright virtue."
    ),
    hex!(
        36,
        "Darkening of the Light",
        "明夷",
        Kun,
        Li,
        [Yang, Yin, Yang, Yin, Yin, Yin],
        "Ming Î indicates that (in the circumstances which it denotes) it will be advantageous to realise the difficulty (of the position), and maintain firm correctness.",
        "(The trigram representing) the earth and that for the bright (sun) entering within it form Ming Î. The superior man, in accordance with this, conducts his management of men;--he shows his intelligence by keeping it obscured."
    ),
    hex!(
        37,
        "The Family",
        "家人",
        Xun,
        Li,
        [Yang, Yin, Yang, Yin, Yang, Yang],
        "For (the realisation of what is taught in) Kiâ Zăn, (or for the regulation of the family), what is most advantageous is that the wife be firm and correct.",
        "(The trigram representing) fire, and that for wind coming forth from it, form Kiâ Zăn. The superior man, in accordance with this, orders his words according to (the truth of) things, and his conduct so that it is uniformly consistent."
    ),
    hex!(
        38,
        "Opposition",
        "睽",
        Li,
        Dui,
        [Yang, Yang, Yin, Yang, Yin, Yang],
        "Khwei indicates that, (notwithstanding the condition of things which it denotes), in small matters there will (still) be good success.",
        "(The trigram representing) fire above, and that for (the waters of) a marsh below, form Khwei. The superior man, in accordance with this, where there is a general agreement, yet admits diversity."
    ),
    hex!(
        39,
        "Obstruction",
        "蹇",
        Kan,
        Gen,
        [Yin, Yin, Yang, Yin, Yang, Yin],
        "In (the state indicated by) Kien advantage will be found in the south-west, and the contrary in the north-east. It will be advantageous (also) to meet with the great man. (In these circumstances), with firmness and correctness, there will be good fortune.",
        "(The trigram representing) a mountain, and above it that for water, form Kien. The superior man, in accordance with this, turns round (and examines) himself, and cultivates his virtue."
    ),
    hex!(
        40,
        "Deliverance",
        "解",
        Zhen,
        Kan,
        [Yin, Yang, Yin, Yang, Yin, Yin],
        "In (the state indicated by) Kieh advantage will be found in the south-west. If no (further) operations be called for, there will be good fortune in coming back (to the old conditions). If some operations be called for, there will be good fortune in the early conducting of them.",
        "(The trigram representing) thunder and that for rain, with these phenomena in a state of manifestation, form Kieh. The superior man, in accordance with this, forgives errors, and deals gently with crimes."
    ),
    hex!(
        41,
        "Decrease",
        "損",
        Gen,
        Dui,
        [Yang, Yang, Yin, Yin, Yin, Yang],
        "In (what is denoted by) Sun, if there be sincerity (in him who employs it), there will be great good fortune:--freedom from error; firmness and correctness that can be maintained; and advantage in every movement that shall be made. In what shall this (sincerity in the exercise of Sun) be employed? (Even) in sacrifice two baskets of grain, (though there be nothing else), may be presented.",
        "(The trigram representing) a mountain and beneath it that for the waters of a marsh form Sun. The superior man, in accordance with this, restrains his wrath and represses his desires."
    ),
    hex!(
        42,
        "Increase",
        "益",
        Xun,
        Zhen,
        [Yang, Yin, Yin, Yin, Yang, Yang],
        "Yî indicates that (in the state which it denotes) there will be advantage in every movement which shall be undertaken, that it will be advantageous (even) to cross the great stream.",
        "(The trigram representing) wind and that for thunder form Yî. The superior man, in accordance with this, when he sees what is good, moves towards it; and when he sees his errors, he turns from them."
    ),
    hex!(
        43,
        "Breakthrough",
        "夬",
        Dui,
        Qian,
        [Yang, Yang, Yang, Yang, Yang, Yin],
        "Kwâi requires (in him who would fulfil its meaning) the exhibition (of the culprit's guilt) in the royal court, and a sincere and earnest appeal (for sympathy and support), with a consciousness of the peril (involved in cutting off the criminal). He should (also) make announcement in his own city, and show that it will not be well to have recourse at once to arms. (In this way) there will be advantage in whatever he shall go forward to.",
        "(The trigram representing) heaven and that for the waters of a marsh mounting above it form Kwâi. The superior man, in accordance with this, bestows emolument on those below him, and dislikes allowing his gifts to accumulate (undispensed)."
    ),
    hex!(
        44,
        "Coming to Meet",
        "姤",
        Qian,
        Xun,
        [Yin, Yang, Yang, Yang, Yang, Yang],
        "Kâu shows a female who is bold and strong. It will not be good to marry (such) a female.",
        "(The trigram representing) wind and that for the sky above it form Kâu. The sovereign, in accordance with this, delivers his charges, and promulgates his announcements throughout the four quarters (of the kingdom)."
    ),
    hex!(
        45,
        "Gathering Together",
        "萃",
        Dui,
        Kun,
        [Yin, Yin, Yin, Yang, Yang, Yin],
        "In (the state denoted by) Žhui, the king will repair to his ancestral temple. It will be advantageous (also) to meet with the great man; and then there will be progress and success, though the advantage must come through firm correctness. The use of great victims will conduce to good fortune; and in whatever direction movement is made, it will be advantageous.",
        "(The trigram representing the) earth and that for the waters of a marsh raised above it form Žhui. The superior man, in accordance with this, has his weapons of war put in good repair, to be prepared against unforeseen contingencies."
    ),
    hex!(
        46,
        "Pushing Upward",
        "升",
        Kun,
        Xun,
        [Yin, Yang, Yang, Yin, Yin, Yin],
        "Shăng indicates that (under its conditions) there will be great progress and success. Seeking by (the qualities implied in it) to meet with the great man, its subject need have no anxiety. Advance to the south will be fortunate.",
        "(The trigram representing) wood and that for the earth with the wood growing in the midst of it form Shăng. The superior man, in accordance with this, pays careful attention to his virtue, and accumulates the small developments of it till it is high and great."
    ),
    hex!(
        47,
        "Oppression",
        "困",
        Dui,
        Kan,
        [Yin, Yang, Yin, Yang, Yang, Yin],
        "In (the condition denoted by) Khwăn there may (yet be) progress and success. For the firm and correct, the (really) great man, there will be good fortune. He will fall into no error. If he make speeches, his words cannot be made good.",
        "(The trigram representing) a marsh, and (below it that for a defile, which has drained the other dry so that there is) no water in it, form Khwăn. The superior man, in accordance with this, will sacrifice his life in order to carry out his purpose."
    ),
    hex!(
        48,
        "The Well",
        "井",
        Kan,
        Xun,
        [Yin, Yang, Yang, Yin, Yang, Yin],
        "(Looking at) Žing, (we think of) how (the site of) a town may be changed, while (the fashion of) its wells undergoes no change. (The water of a well) never disappears and never receives (any great) increase, and those who come and those who go can draw and enjoy the benefit. If (the drawing) have nearly been accomplished, but, before the rope has quite reached the water, the bucket is broken, this is evil.",
        "(The trigram representing) wood and above it that for water form Žing. The superior man, in accordance with this, comforts the people, and stimulates them to mutual helpfulness."
    ),
    hex!(
        49,
        "Revolution",
        "革",
        Dui,
        Li,
        [Yang, Yin, Yang, Yang, Yang, Yin],
        "(What takes place as indicated by) Ko is believed in only after it has been accomplished. There will be great progress and success. Advantage will come from being firm and correct. (In that case) occasion for repentance will disappear.",
        "(The trigram representing the waters of) a marsh and that for fire in the midst of them form Ko. The superior man, in accordance with this, regulates his (astronomical) calculations, and makes clear the seasons and times."
    ),
    hex!(
        50,
        "The Cauldron",
        "鼎",
        Li,
        Xun,
        [Yin, Yang, Yang, Yang, Yin, Yang],
        "Ting gives the intimation of great progress and success.",
        "(The trigram representing) wood and above it that for fire form Ting. The superior man, in accordance with this, keeps his every position correct, and maintains secure the appointment (of Heaven)."
    ),
    hex!(
        51,
        "The Arousing",
        "震",
        Zhen,
        Zhen,
        [Yang, Yin, Yin, Yang, Yin, Yin],
        "Kăn gives the intimation of ease and development. When (the time of) movement (which it indicates) comes, (the subject of the hexagram) will be found looking out with apprehension, and yet smiling and talking cheerfully. When the movement (like a crash of thunder) terrifies all within a hundred lî, he will be (like the sincere worshipper) who is not (startled into) letting go his ladle and (cup of) sacrificial spirits.",
        "(The trigram representing) thunder, being repeated, forms Kăn. The superior man, in accordance with this, is fearful and apprehensive, cultivates (his virtue), and examines (his faults)."
    ),
    hex!(
        52,
        "Keeping Still",
        "艮",
        Gen,
        Gen,
        [Yin, Yin, Yang, Yin, Yin, Yang],
        "When one's resting is like that of the back, and he loses all consciousness of self; when he walks in his courtyard, and does not see any (of the persons) in it,--there will be no error.",
        "(Two trigrams representing) a mountain, one over the other, form Kăn. The superior man, in accordance with this, does not go in his thoughts beyond the (duties of the) position in which he is."
    ),
    hex!(
        53,
        "Development",
        "漸",
        Xun,
        Gen,
        [Yin, Yin, Yang, Yin, Yang, Yang],
        "Kien suggests to us the marriage of a young lady, and the good fortune (attending it). There will be advantage in being firm and correct.",
        "(The trigram representing) a mountain and above it that for a tree form Kien. The superior man, in accordance with this, attains to and maintains his extraordinary virtue, and makes the manners of the people good."
    ),
    hex!(
        54,
        "The Marrying Maiden",
        "歸妹",
        Zhen,
        Dui,
        [Yang, Yang, Yin, Yang, Yin, Yin],
        "Kwei Mei indicates that (under the conditions which it denotes) action will be evil, and in no wise advantageous.",
        "(The trigram representing the waters of) a marsh and over it that for thunder form Kwei Mei. The superior man, in accordance with this, having regard to the far-distant end, knows the mischief (that may be done at the beginning)."
    ),
    hex!(
        55,
        "Abundance",
        "豐",
        Zhen,
        Li,
        [Yang, Yin, Yang, Yang, Yin, Yin],
        "Făng intimates progress and development. When a king has reached the point (which the name denotes) there is no occasion to be anxious (through fear of a change). Let him be as the sun at noon.",
        "(The trigrams representing) thunder and lightning combine to form Făng. The superior man, in accordance with this, decides cases of litigation, and apportions punishments with exactness."
    ),
    hex!(
        56,
        "The Wanderer",
        "旅",
        Li,
        Gen,
        [Yin, Yin, Yang, Yang, Yin, Yang],
        "Lü intimates that (in the condition which it denotes) there may be some little attainment and progress. If the stranger or traveller be firm and correct as he ought to be, there will be good fortune.",
        "(The trigram representing) a mountain and above it that for fire form Lü. The superior man, in accordance with this, exerts his wisdom and caution in the use of punishments and not allowing litigations to continue."
    ),
    hex!(
        57,
        "The Gentle",
        "巽",
        Xun,
        Xun,
        [Yin, Yang, Yang, Yin, Yang, Yang],
        "Sun intimates that (under the conditions which it denotes) there will be some little attainment and progress. There will be advantage in movement onward in whatever direction. It will be advantageous (also) to see the great man.",
        "(Two trigrams representing) wind, following each other, form Sun. The superior man, in accordance with this, reiterates his orders, and secures the practice of his affairs."
    ),
    hex!(
        58,
        "The Joyous",
        "兌",
        Dui,
        Dui,
        [Yang, Yang, Yin, Yang, Yang, Yin],
        "Tui intimates that (under its conditions) there will be progress and attainment. (But) it will be advantageous to be firm and correct.",
        "(Two symbols representing) the waters of a marsh, one over the other, form Tui. The superior man, in accordance with this, (encourages) the conversation of friends and (the stimulus of) their (common) practice."
    ),
    hex!(
        59,
        "Dispersion",
        "渙",
        Xun,
        Kan,
        [Yin, Yang, Yin, Yin, Yang, Yang],
        "Hwân intimates that (under its conditions) there will be progress and success. The king goes to his ancestral temple; and it will be advantageous to cross the great stream. It will be advantageous to be firm and correct.",
        "(The trigram representing) water and that for wind moving above the water form Hwân. The ancient kings, in accordance with this, presented offerings to God and established the ancestral temple."
    ),
    hex!(
        60,
        "Limitation",
        "節",
        Kan,
        Dui,
        [Yang, Yang, Yin, Yin, Yang, Yin],
        "Kieh intimates that (under its conditions) there will be progress and attainment. (But) if the regulations (which it prescribes) be severe and difficult, they cannot be permanent.",
        "(The trigram representing) a lake, and above it that for water, form Kieh. The superior man, in accordance with this, constructs his (methods of) numbering and measurement, and discusses (points of) virtue and conduct."
    ),
    hex!(
        61,
        "Inner Truth",
        "中孚",
        Xun,
        Dui,
        [Yang, Yang, Yin, Yin, Yang, Yang],
        "Kung Fû (moves even) pigs and fish, and leads to good fortune. There will be advantage in crossing the great stream. There will be advantage in being firm and correct.",
        "(The trigram representing the waters of) a marsh and that for wind above it form Kung Fû. The superior man, in accordance with this, deliberates about cases of litigation and delays (the infliction of) death."
    ),
    hex!(
        62,
        "Preponderance of the Small",
        "小過",
        Zhen,
        Gen,
        [Yin, Yin, Yang, Yang, Yin, Yin],
        "Hsiâo Kwo indicates that (in the circumstances which it implies) there will be progress and attainment. But it will be advantageous to be firm and correct. (What the name denotes) may be done in small affairs, but not in great affairs. (It is like) the notes that come down from a bird on the wing;--to descend is better than to ascend. There will (in this way) be great good fortune.",
        "(The trigram representing) a hill and that for thunder above it form Hsiâo Kwo. The superior man, in accordance with this, in his conduct exceeds in humility, in mourning exceeds in sorrow, and in his expenditure exceeds in economy."
    ),
    hex!(
        63,
        "After Completion",
        "既濟",
        Kan,
        Li,
        [Yang, Yin, Yang, Yin, Yang, Yin],
        "Kî Žî intimates progress and success in small matters. There will be advantage in being firm and correct. There has been good fortune in the beginning; there may be disorder in the end.",
        "(The trigram representing) fire and that for water above it form Kî Žî. The superior man, in accordance with this, thinks of evil (that may come), and beforehand guards against it."
    ),
    hex!(
        64,
        "Before Completion",
        "未濟",
        Li,
        Kan,
        [Yin, Yang, Yin, Yang, Yin, Yang],
        "Wei Žî intimates progress and success (in the circumstances which it implies). (We see) a young fox that has nearly crossed (the stream), when its tail gets immersed. There will be no advantage in any way.",
        "(The trigram representing) water and that for fire above it form Wei Žî. The superior man, in accordance with this, carefully discriminates among (the qualities of) things, and the (different) positions they (naturally) occupy."
    ),
];

// ---------------------------------------------------------------------------
// Per-line (yao ci / changing-line) texts — James Legge, The Yî King,
// Sacred Books of the East Vol. XVI (1882). PUBLIC DOMAIN.
//
// All 384 line statements (64 hexagrams × 6 lines, bottom line 1 → top
// line 6, King Wen sequence) were transcribed verbatim from the Internet
// Sacred Text Archive edition (sacred-texts.com/ich). Legge's editorial
// parentheticals and romanisation diacritics (â, î, Î, Ž) are preserved as
// published. Page-break markers ("p. 60") were removed. A small number of
// obvious artifacts inherited from the source OCR were corrected against
// independent public-domain Legge mirrors (e.g. baharna.com/iching/legge),
// without altering any wording:
//   - Hex 59 line 5: stray "[paragraph continues]" navigation marker removed.
//   - Hex 17 line 3 ("lets go. the little boy"), Hex 25 line 3 ("accused
//     and. apprehended"), Hex 43 line 4 ("like. a sheep led"), Hex 62 line 4
//     ("his natural. course"), Hex 62 line 6 ("shows. its subject"): a stray
//     period that the OCR dropped mid-clause was removed.
//   - Hex 12 image ("(the manifestation) of)") and Hex 13 image ("with
//     this),"): a misplaced parenthesis was rebalanced to the published form.
// Legge's own occasional unbalanced parenthesis (e.g. Hex 4 line 2,
// "admitting (even the goodness of women,") is intentionally LEFT AS PUBLISHED.
// No interpretive content was authored or generated — every line is genuine
// Legge text. Coverage: 384 / 384 lines, 0 MISSING.
// ---------------------------------------------------------------------------
static LINE_TEXTS: [[&str; 6]; 64] = [
    // Hexagram 1
    [
        "In the first (or lowest) NINE, undivided, (we see its subject as) the dragon lying hid (in the deep). It is not the time for active doing.",
        "In the second NINE, undivided, (we see its subject as) the dragon appearing in the field. It will be advantageous to meet with the great man.",
        "In the third NINE, undivided, (we see its subject as) the superior man active and vigilant all the day, and in the evening still careful and apprehensive. (The position is) dangerous, but there will be no mistake.",
        "In the fourth NINE, undivided, (we see its subject as the dragon looking) as if he were leaping up, but still in the deep. There will be no mistake.",
        "In the fifth NINE, undivided, (we see its subject as) the dragon on the wing in the sky. It will be advantageous to meet with the great man.",
        "In the sixth (or topmost) NINE, undivided, (we see its subject as) the dragon exceeding the proper limits. There will be occasion for repentance.",
    ],
    // Hexagram 2
    [
        "In the first SIX, divided, (we see its subject) treading on hoarfrost. The strong ice will come (by and by).",
        "The second SIX, divided, (shows the attribute of) being straight, square, and great. (Its operation), without repeated efforts, will be in every respect advantageous.",
        "The third SIX, divided, (shows its subject) keeping his excellence under restraint, but firmly maintaining it. If he should have occasion to engage in the king's service, though he will not claim the success (for himself), he will bring affairs to a good issue.",
        "The fourth SIX, divided, (shows the symbol of) a sack tied up. There will be no ground for blame or for praise.",
        "The fifth SIX, divided, (shows) the yellow lower garment. There will be great good fortune.",
        "The sixth SIX, divided (shows) dragons fighting in the wild. Their blood is purple and yellow.",
    ],
    // Hexagram 3
    [
        "The first NINE, undivided, shows the difficulty (its subject has) in advancing. It will be advantageous for him to abide correct and firm; advantageous (also) to be made a feudal ruler.",
        "The second SIX, divided, shows (its subject) distressed and obliged to return; (even) the horses of her chariot (also) seem to be retreating. (But) not by a spoiler (is she assailed), but by one who seeks her to be his wife. The young lady maintains her firm correctness, and declines a union. After ten years she will be united, and have children.",
        "The third SIX, divided, shows one following the deer without (the guidance of) the forester, and only finding himself in the midst of the forest. The superior man, acquainted with the secret risks, thinks it better to give up the chase. If he went forward, he would regret it.",
        "The fourth SIX, divided, shows (its subject as a lady), the horses of whose chariot appear in retreat. She seeks, however, (the help of) him who seeks her to be his wife. Advance will be fortunate; all will turn out advantageously.",
        "The fifth NINE, undivided, shows the difficulties in the way of (its subject's) dispensing the rich favours that might be expected from him. With firmness and correctness there will be good fortune in small things; (even) with them in great things there will be evil.",
        "The topmost SIX, divided, shows (its subject) with the horses of his chariot obliged to retreat, and weeping tears of blood in streams.",
    ],
    // Hexagram 4
    [
        "The first SIX, divided, (has respect to) the dispelling of ignorance. It will be advantageous to use punishment (for that purpose), and to remove the shackles (from the mind). But going on in that way (of punishment) will give occasion for regret.",
        "The second NINE, undivided, (shows its subject) exercising forbearance with the ignorant, in which there will be good fortune; and admitting (even the goodness of women, which will also be fortunate. (He may be described also as) a son able to (sustain the burden of) his family.",
        "The third SIX, divided, (seems to say) that one should not marry a woman whose emblem it might be, for that, when she sees a man of wealth, she will not keep her person from him, and in no wise will advantage come from her.",
        "The fourth SIX, divided, (shows its subject as if) bound in chains of ignorance. There will be occasion for regret.",
        "The fifth SIX, divided, shows its subject as a simple lad without experience. There will be good fortune.",
        "In the topmost NINE, undivided, we see one smiting the ignorant (youth). But no advantage will come from doing him an injury. Advantage would come from warding off injury from him.",
    ],
    // Hexagram 5
    [
        "The first NINE, undivided, shows its subject waiting in the distant border. It will be well for him constantly to maintain (the purpose thus shown), in which case there will be no error.",
        "The second NINE, undivided, shows its subject waiting on the sand (of the mountain stream). He will (suffer) the small (injury of) being spoken (against), but in the end there will be good fortune.",
        "The third NINE, undivided, shows its subject in the mud (close by the stream). He thereby invites the approach of injury.",
        "The fourth SIX, divided, shows its subject waiting in (the place of) blood. But he will get out of the cavern.",
        "The fifth NINE, undivided, shows its subject waiting amidst the appliances of a feast. Through his firmness and correctness there will be good fortune.",
        "The topmost SIX, divided, shows its subject entered into the cavern. (But) there are three guests coming, without being urged, (to his help). If he receive them respectfully, there will be good fortune in the end.",
    ],
    // Hexagram 6
    [
        "The first SIX, divided, shows its subject not perpetuating the matter about which (the contention is). He will suffer the small (injury) of being spoken against, but the end will be fortunate.",
        "The second NINE, undivided, shows its subject unequal to the contention. If he retire and keep concealed (where) the inhabitants of his city are (only) three hundred families, he will fall into no mistake.",
        "The third SIX, divided, shows its subject keeping in the old place assigned for his support, and firmly correct. Perilous as the position is, there will be good fortune in the end. Should he perchance engage in the king's business, he will not (claim the merit of) achievement.",
        "The fourth NINE, undivided, shows its subject unequal to the contention. He returns to (the study of Heaven's) ordinances, changes (his wish to contend), and rests in being firm and correct. There will be good fortune.",
        "The fifth NINE, undivided, shows its subject contending;--and with great good fortune.",
        "The topmost NINE, undivided, shows how its subject may have the leathern belt conferred on him (by the sovereign), and thrice it shall be taken from him in a morning.",
    ],
    // Hexagram 7
    [
        "The first SIX, divided, shows the host going forth according to the rules (for such a movement). If these be not good, there will be evil.",
        "The second NINE, undivided, shows (the leader) in the midst of the host. There will be good fortune and no error. The king has thrice conveyed to him the orders (of his favour).",
        "The third SIX, divided, shows how the host may, possibly, have many inefficient leaders. There will be evil.",
        "The fourth SIX, divided, shows the host in retreat. There is no error.",
        "The fifth SIX, divided, shows birds in the fields, which it will be advantageous to seize (and destroy). In that case there will be no error. If the oldest son leads the host, and younger men (idly occupy offices assigned to them), however firm and correct he may be, there will be evil.",
        "The topmost SIX, divided, shows the great ruler delivering his charges, (appointing some) to be rulers of states, and others to undertake the headship of clans; but small men should not be employed (in such positions).",
    ],
    // Hexagram 8
    [
        "The first SIX, divided, shows its subject seeking by his sincerity to win the attachment of his object. There will be no error. Let (the breast) be full of sincerity as an earthenware vessel is of its contents, and it will in the end bring other advantages.",
        "In the second SIX, divided, we see the movement towards union and attachment proceeding from the inward (mind). With firm correctness there will be good fortune.",
        "In the third SIX, divided, we see its subject seeking for union with such as ought not to be associated with.",
        "In the fourth SIX, divided, we see its subject seeking for union with the one beyond himself. With firm correctness there will be good fortune.",
        "The fifth NINE, undivided, affords the most illustrious instance of seeking union and attachment. (We seem to see in it) the king urging his pursuit of the game (only) in three directions, and allowing the escape of all the animals before him, while the people of his towns do not warn one another (to prevent it). There will be good fortune.",
        "In the topmost SIX, divided, we see one seeking union and attachment without having taken the first step (to such an end). There will be evil.",
    ],
    // Hexagram 9
    [
        "The first NINE, undivided, shows its subject returning and pursuing his own course. What mistake should he fall into? There will be good fortune.",
        "The second NINE, undivided, shows its subject, by the attraction (of the former line), returning (to the proper course). There will be good fortune.",
        "The third NINE, undivided, suggests the idea of a carriage, the strap beneath which has been removed, or of a husband and wife looking on each other with averted eyes.",
        "The fourth SIX, divided, shows its subject possessed of sincerity. The danger of bloodshed is thereby averted, and his (ground for) apprehension dismissed. There will be no mistake.",
        "The fifth NINE, undivided, shows its subject possessed of sincerity, and drawing others to unite with him. Rich in resources, he employs his neighbours (in the same cause with himself).",
        "The topmost NINE, undivided, shows how the rain has fallen, and the (onward progress) is stayed;--(so) must we value the full accumulation of the virtue (represented by the upper trigram). But a wife (exercising restraint), however firm and correct she may be, is in a position of peril, (and like) the moon approaching to the full. If the superior man prosecute his measures (in such circumstances), there will be evil.",
    ],
    // Hexagram 10
    [
        "The first NINE, undivided, shows its subject treading his accustomed path. If he go forward, there will be no error.",
        "The second NINE, undivided, shows its subject treading the path that is level and easy;--a quiet and solitary man, to whom, if he be firm and correct, there will be good fortune.",
        "The third SIX, divided, shows a one-eyed man (who thinks he) can see; a lame man (who thinks he) can walk well; one who treads on the tail of a tiger and is bitten. (All this indicates) ill fortune. We have a (mere) bravo acting the part of a great ruler.",
        "The fourth NINE, undivided, shows its subject treading on the tail of a tiger. He becomes full of apprehensive caution, and in the end there will be good fortune.",
        "The fifth NINE, undivided, shows the resolute tread of its subject. Though he be firm and correct, there will be peril.",
        "The sixth NINE, undivided, tells us to look at (the whole course) that is trodden, and examine the presage which that gives. If it be complete and without failure, there will be great good fortune.",
    ],
    // Hexagram 11
    [
        "The first NINE, undivided, suggests the idea of grass pulled up, and bringing with it other stalks with whose roots it is connected. Advance (on the part of its subject) will be fortunate.",
        "The second NINE, undivided, shows one who can bear with the uncultivated, will cross the Ho without a boat, does not forget the distant, and has no (selfish) friendships. Thus does he prove himself acting in accordance with the course of the due Mean.",
        "The third NINE, undivided, shows that, while there is no state of peace that is not liable to be disturbed, and no departure (of evil men) so that they shall not return, yet when one is firm and correct, as he realises the distresses that may arise, he will commit no error. There is no occasion for sadness at the certainty (of such recurring changes); and in this mood the happiness (of the present) may be (long) enjoyed.",
        "The fourth SIX, divided, shows its subject fluttering (down);--not relying on his own rich resources, but calling in his neighbours. (They all come) not as having received warning, but in the sincerity (of their hearts).",
        "The fifth six, divided, reminds us of (king) Tî-yî's (rule about the) marriage of his younger sister. By such a course there is happiness and there will be great good fortune.",
        "The sixth six, divided, shows us the city wall returned into the moat. It is not the time to use the army. (The subject of the line) may, indeed, announce his orders to the people of his own city; but however correct and firm he may be, he will have cause for regret.",
    ],
    // Hexagram 12
    [
        "The first SIX, divided, suggests the idea of grass pulled up, and bringing with it other stalks with whose roots it is connected. With firm correctness (on the part of its subject), there will be good fortune and progress.",
        "The second SIX, divided, shows its subject patient and obedient. To the small man (comporting himself so) there will be good fortune. If the great man (comport himself) as the distress and obstruction require, he will have success.",
        "The third SIX, divided, shows its subject ashamed of the purpose folded (in his breast).",
        "The fourth NINE, undivided, shows its subject acting in accordance with the ordination (of Heaven), and committing no error. His companions will come and share in his happiness.",
        "In the fifth NINE, undivided, we see him who brings the distress and obstruction to a close,--the great man and fortunate. (But let him say), 'We may perish! We may perish!' (so shall the state of things become firm, as if) bound to a clump of bushy mulberry trees.",
        "The sixth NINE, undivided, shows the overthrow (and removal of) the condition of distress and obstruction. Before this there was that condition. Hereafter there will be joy.",
    ],
    // Hexagram 13
    [
        "The first NINE, undivided, (shows the representative of) the union of men just issuing from his gate. There will be no error.",
        "The second SIX, divided, (shows the representative of) the union of men in relation with his kindred. There will be occasion for regret.",
        "The third NINE, undivided, (shows its subject) with his arms hidden in the thick grass, and at the top of a high mound. (But) for three years he makes no demonstration.",
        "The fourth NINE, undivided, (shows its subject) mounted on the city wall; but he does not proceed to make the attack (he contemplates). There will be good fortune.",
        "In the fifth NINE, undivided, (the representative of) the union of men first wails and cries out, and then laughs. His great host conquers, and he (and the subject of the second line) meet together.",
        "The topmost NINE, undivided, (shows the representative of) the union of men in the suburbs. There will be no occasion for repentance.",
    ],
    // Hexagram 14
    [
        "In the first NINE, undivided, there is no approach to what is injurious, and there is no error. Let there be a realisation of the difficulty (and danger of the position), and there will be no error (to the end).",
        "In the second NINE, undivided, we have a large waggon with its load. In whatever direction advance is made, there will be no error.",
        "The third NINE, undivided, shows us a feudal prince presenting his offerings to the Son of Heaven. A small man would be unequal (to such a duty).",
        "The fourth NINE, undivided, shows its subject keeping his great resources under restraint. There will be no error.",
        "The fifth SIX, divided, shows the sincerity of its subject reciprocated by that of all the others (represented in the hexagram). Let him display a proper majesty, and there will be good fortune.",
        "The topmost NINE, undivided, shows its subject with help accorded to him from Heaven. There will be good fortune, advantage in every respect.",
    ],
    // Hexagram 15
    [
        "The first SIX, divided, shows us the superior man who adds humility to humility. (Even) the great stream may be crossed with this, and there will be good fortune.",
        "The second SIX, divided, shows us humility that has made itself recognised. With firm correctness there will be good fortune.",
        "The third NINE, undivided, shows the superior man of (acknowledged) merit. He will maintain his success to the end, and have good fortune.",
        "The fourth SIX, divided, shows one, whose action would be in every way advantageous, stirring up (the more) his humility.",
        "The fifth SIX, divided, shows one who, without being rich, is able to employ his neighbours. He may advantageously use the force of arms. All his movements will be advantageous.",
        "The sixth SIX, divided, shows us humility that has made itself recognised. The subject of it will with advantage put his hosts in motion; but (he will only) punish his own towns and state.",
    ],
    // Hexagram 16
    [
        "The first SIX, divided, shows its subject proclaiming his pleasure and satisfaction. There will be evil.",
        "The second SIX, divided, shows one who is firm as a rock. (He sees a thing) without waiting till it has come to pass; with his firm correctness there will be good fortune.",
        "The third SIX, divided, shows one looking up (for favours), while he indulges the feeling of pleasure and satisfaction. If he would understand!--If he be late in doing so, there will indeed be occasion for repentance.",
        "The fourth NINE, undivided, shows him from whom the harmony and satisfaction come. Great is the success which he obtains. Let him not allow suspicions to enter his mind, and thus friends will gather around him.",
        "The fifth six, divided, shows one with a chronic complaint, but who lives on without dying.",
        "The topmost six, divided, shows its subject with darkened mind devoted to the pleasure and satisfaction (of the time); but if he change his course even when (it may be considered as) completed, there will be no error.",
    ],
    // Hexagram 17
    [
        "The first NINE, undivided, shows us one changing the object of his pursuit; but if he be firm and correct, there will he good fortune. Going beyond (his own) gate to find associates, he will achieve merit.",
        "The second SIX, divided, shows us one who cleaves to the little boy, and lets go the man of age and experience.",
        "The third SIX, divided, shows us one who cleaves to the man of age and experience, and lets go the little boy. Such following will get what it seeks; but it will be advantageous to adhere to what is firm and correct.",
        "The fourth NINE, undivided, shows us one followed and obtaining (adherents). Though he be firm and correct, there will be evil. If he be sincere (however) in his course, and make that evident, into what error will he fall?",
        "The fifth NINE, undivided, shows us (the ruler) sincere in (fostering all) that is excellent. There will be good fortune.",
        "The topmost SIX, divided, shows us (that sincerity) firmly held and clung to, yea, and bound fast. (We see) the king with it presenting his offerings on the western mountain.",
    ],
    // Hexagram 18
    [
        "The first SIX, divided, shows (a son) dealing with the troubles caused by his father. If he be an (able) son, the father will escape the blame of having erred. The position is perilous, but there will be good fortune in the end.",
        "The second NINE, undivided, shows (a son) dealing with the troubles caused by his mother. He should not (carry) his firm correctness (to the utmost).",
        "The third NINE, undivided, shows (a son) dealing with the troubles caused by his father. There may be some small occasion for repentance, but there will not be any great error.",
        "The fourth SIX, divided, shows (a son) viewing indulgently the troubles caused by his father. If he go forward, he will find cause to regret it.",
        "The fifth SIX, divided, shows (a son) dealing with the troubles caused by his father. He obtains the praise of using (the fit instrument for his work).",
        "The sixth NINE, undivided, shows us one who does not serve either king or feudal lord, but in a lofty spirit prefers (to attend to) his own affairs.",
    ],
    // Hexagram 19
    [
        "The first NINE, undivided, shows its subject advancing in company (with the subject of the second line). Through his firm correctness there will be good fortune.",
        "The second NINE, undivided, shows its subject advancing in company (with the subject of the first line). There will be good fortune; (advancing) will be in every way advantageous.",
        "The third SIX, divided, shows one well pleased (indeed) to advance, (but whose action) will be in no way advantageous. If he become anxious about it (however), there will be no error.",
        "The fourth SIX, divided, shows one advancing in the highest mode. There will be no error.",
        "The fifth SIX, divided, shows the advance of wisdom, such as befits the great ruler. There will be good fortune.",
        "The sixth SIX, divided, shows the advance of honesty and generosity. There will be good fortune, and no error.",
    ],
    // Hexagram 20
    [
        "The first SIX, divided, shows the looking of a lad;--not blamable in men of inferior rank, but matter for regret in superior men.",
        "The second SIX, divided, shows one peeping out from a door. It would be advantageous if it were (merely) the firm correctness of a female.",
        "The third SIX, divided, shows one looking at (the course of) his own life, to advance or recede (accordingly).",
        "The fourth SIX, divided, shows one contemplating the glory of the kingdom. It will be advantageous for him, being such as he is, (to seek) to be a guest of the king.",
        "The fifth NINE, undivided, shows its subject contemplating his own life(-course). A superior man, he will (thus) fall into no error.",
        "The sixth NINE, undivided, shows its subject contemplating his character to see if it be indeed that of a superior man. He will not fall into error.",
    ],
    // Hexagram 21
    [
        "The first NINE, undivided, shows one with his feet in the stocks and deprived of his toes. There will be no error.",
        "The second SIX, divided, shows one biting through the soft flesh, and (going on to) bite off the nose. There will be no error.",
        "The third SIX, divided, shows one gnawing dried flesh, and meeting with what is disagreeable. There will be occasion for some small regret, but no (great) error.",
        "The fourth NINE, undivided, shows one gnawing the flesh dried on the bone, and getting the pledges of money and arrows. It will be advantageous to him to realise the difficulty of his task and be firm,--in which case there will be good fortune.",
        "The fifth SIX, divided, shows one gnawing at dried flesh, and finding the yellow gold. Let him be firm and correct, realising the peril (of his position). There will be no error.",
        "The sixth NINE, undivided, shows one wearing the cangue, and deprived of his cars. There will be evil.",
    ],
    // Hexagram 22
    [
        "The first NINE, undivided, shows one adorning (the way of) his feet. He can discard a carriage and walk on foot.",
        "The second SIX, divided, shows one adorning his beard.",
        "The third NINE, undivided, shows its subject with the appearance of being adorned and bedewed (with rich favours). But let him ever maintain his firm correctness, and there will be good fortune.",
        "The fourth SIX, divided, shows one looking as if adorned, but only in white. As if (mounted on) a white horse, and furnished with wings, (he seeks union with the subject of the first line), while (the intervening third pursues), not as a robber, but intent on a matrimonial alliance.",
        "The fifth SIX, divided, shows its subject adorned by (the occupants of) the heights and gardens. He bears his roll of silk, small and slight. He may appear stingy; but there will be good fortune in the end.",
        "The sixth NINE, undivided, shows one with white as his (only) ornament. There will be no error.",
    ],
    // Hexagram 23
    [
        "The first SIX, divided, shows one overturning the couch by injuring its legs. (The injury will go on to) the destruction of (all) firm correctness, and there will be evil.",
        "The second SIX, divided, shows one overthrowing the couch by injuring its frame. (The injury will go on to) the destruction of (all) firm correctness, and there will be evil.",
        "The third SIX, divided, shows its subject among the overthrowers; but there will be no error.",
        "The fourth SIX, divided, shows its subject having overthrown the couch, and (going to injure) the skin (of him who lies on it). There will be evil.",
        "The fifth SIX, divided, shows (its subject leading on the others like) a string of fishes, and (obtaining for them) the favour that lights on the inmates of the palace. There will be advantage in every way.",
        "The topmost NINE, undivided, shows its subject (as) a great fruit which has not been eaten. The superior man finds (the people again) as a chariot carrying him. The small men (by their course) overthrow their own dwellings.",
    ],
    // Hexagram 24
    [
        "The first NINE, undivided, shows its subject returning (from an error) of no great extent, which would not proceed to anything requiring repentance. There will be great good fortune.",
        "The second SIX, divided, shows the admirable return (of its subject). There will be good fortune.",
        "The third SIX, divided, shows one who has made repeated returns. The position is perilous, but there will be no error.",
        "The fourth SIX, divided, shows its subject moving right in the centre (among those represented by the other divided lines), and yet returning alone (to his proper path).",
        "The fifth SIX, divided, shows the noble return of its subject. There will be no ground for repentance.",
        "The topmost SIX, divided, shows its subject all astray on the subject of returning. There will be evil. There will be calamities and errors. If with his views he put the hosts in motion, the end will be a great defeat, whose issues will extend to the ruler of the state. Even in ten years he will not be able to repair the disaster.",
    ],
    // Hexagram 25
    [
        "The first NINE, undivided, shows its subject free from all insincerity. His advance will be accompanied with good fortune.",
        "The second SIX, divided, shows one who reaps without having ploughed (that he might reap), and gathers the produce of his third year's fields without having cultivated them the first year for that end. To such a one there will be advantage in whatever direction he may move.",
        "The third SIX, divided, shows calamity happening to one who is free from insincerity;--as in the case of an ox that has been tied up. A passer by finds it (and carries it off), while the people in the neighbourhood have the calamity (of being accused and apprehended).",
        "The fourth NINE, undivided, shows (a case) in which, if its subject can remain firm and correct, there will be no error.",
        "The fifth NINE, undivided, shows one who is free from insincerity, and yet has fallen ill. Let him not use medicine, and he will have occasion for joy (in his recovery).",
        "The topmost NINE, undivided, shows its subject free from insincerity, yet sure to fall into error, if he take action. (His action) will not be advantageous in any way.",
    ],
    // Hexagram 26
    [
        "The first NINE, undivided, shows its subject in a position of peril. It will be advantageous for him to stop his advance.",
        "The second NINE, undivided, shows a carriage with the strap under it removed.",
        "The third NINE, undivided, shows its subject urging his way with good horses. It will be advantageous for him to realise the difficulty (of his course), and to be firm and correct, exercising himself daily in his charioteering and methods of defence; then there will be advantage in whatever direction he may advance.",
        "The fourth six, divided, shows the young bull, (and yet) having the piece of wood over his horns. There will be great good fortune.",
        "The fifth six, divided, shows the teeth of a castrated hog. There will be good fortune.",
        "The sixth NINE, undivided, shows its subject (as) in command of the firmament of heaven. There will be progress.",
    ],
    // Hexagram 27
    [
        "The first NINE, undivided, (seems to be thus addressed), 'You leave your efficacious tortoise, and look at me till your lower jaw hangs down.' There will be evil.",
        "The second SIX, divided, shows one looking downwards for nourishment, which is contrary to what is proper; or seeking it from the height (above), advance towards which will lead to evil.",
        "The third SIX, divided, shows one acting contrary to the method of nourishing. However firm he may be, there will be evil. For ten years let him not take any action, (for) it will not be in any way advantageous.",
        "The fourth SIX, divided, shows one looking downwards for (the power to) nourish. There will be good fortune. Looking with a tiger's downward unwavering glare, and with his desire that impels him to spring after spring, he will fall into no error.",
        "The fifth SIX, divided, shows one acting contrary to what is regular and proper; but if he abide in firmness, there will be good fortune. He should not, (however, try to) cross the great stream.",
        "The sixth NINE, undivided, shows him from whom comes the nourishing. His position is perilous, but there will be good fortune. It will be advantageous to cross the great stream.",
    ],
    // Hexagram 28
    [
        "The first SIX, divided, shows one placing mats of the white mâo grass under things set on the ground. There will be no error.",
        "The second NINE, undivided, shows a decayed willow producing shoots, or an old husband in possession of his young wife. There will be advantage in every way.",
        "The third NINE, undivided, shows a beam that is weak. There will be evil.",
        "The fourth NINE, undivided, shows a beam curving upwards. There will be good fortune. If (the subject of it) looks for other (help but that of line one), there will be cause for regret.",
        "The fifth NINE, undivided, shows a decayed willow producing flowers, or an old wife in possession of her young husband. There will be occasion neither for blame nor for praise.",
        "The topmost SIX, divided, shows its subject with extraordinary (boldness) wading through a stream, till the water hides the crown of his head. There will be evil, but no ground for blame.",
    ],
    // Hexagram 29
    [
        "The first SIX, divided, shows its subject in the double defile, and (yet) entering a cavern within it. There will be evil.",
        "The second NINE, undivided, shows its subject in all the peril of the defile. He will, however, get a little (of the deliverance) that he seeks.",
        "The third SIX, divided, shows its subject, whether he comes or goes ( =descends or ascends), confronted by a defile. All is peril to him and unrest. (His endeavours) will lead him into the cavern of the pit. There should be no action (in such a case).",
        "The fourth SIX, divided, shows its subject (at a feast), with (simply) a bottle of spirits, and a subsidiary basket of rice, while (the cups and bowls) are (only) of earthenware. He introduces his important lessons (as his ruler's) intelligence admits. There will in the end be no error.",
        "The fifth NINE, undivided, shows the water of the defile not yet full, (so that it might flow away); but order will (soon) be brought about. There will be no error.",
        "The topmost SIX, divided, shows its subject bound with cords of three strands or two strands, and placed in the thicket of thorns. But in three years he does not learn the course for him to pursue. There will be evil.",
    ],
    // Hexagram 30
    [
        "The first NINE, undivided, shows one ready to move with confused steps. But he treads at the same time reverently, and there will be no mistake.",
        "The second SIX, divided, shows its subject in his place in yellow. There will be great good fortune.",
        "The third NINE, undivided, shows its subject in a position like that of the declining sun. Instead of playing on his instrument of earthenware, and singing to it, he utters the groans of an old man of eighty. There will be evil.",
        "The fourth NINE, undivided, shows the manner of its subject's coming. How abrupt it is, as with fire, with death, to be rejected (by all)!",
        "The fifth SIX, divided, shows its subject as one with tears flowing in torrents, and groaning in sorrow. There will be good fortune.",
        "The topmost NINE, undivided, shows the king employing its subject in his punitive expeditions. Achieving admirable (merit), he breaks (only) the chiefs (of the rebels). Where his prisoners were not their associates, he does not punish. There will be no error.",
    ],
    // Hexagram 31
    [
        "The first six, divided, shows one moving his great toes.",
        "The second SIX, divided, shows one moving the calves of his leg. There will be evil. If he abide (quiet in his place), there will be good fortune.",
        "The third NINE, undivided, shows one moving his thighs, and keeping close hold of those whom he follows. Going forward (in this way) will cause regret.",
        "The fourth NINE, undivided, shows that firm correctness whi.ch will lead to good fortune, and prevent all occasion for repentance. If its subject be unsettled in his movements, (only) his friends will follow his purpose.",
        "The fifth NINE, undivided, shows one moving the flesh along the spine above the heart. There will be no occasion for repentance.",
        "The sixth six, divided, shows one moving his jaws and tongue.",
    ],
    // Hexagram 32
    [
        "The first SIX, divided, shows its subject deeply (desirous) of long continuance. Even with firm correctness there will be evil; there will be no advantage in any way.",
        "The second NINE, undivided, shows all occasion for repentance disappearing.",
        "The third NINE, undivided, shows one who does not continuously maintain his virtue. There are those who will impute this to him as a disgrace. However firm he may be, there will be ground for regret.",
        "The fourth NINE, undivided, shows a field where there is no game.",
        "The fifth SIX, divided, shows its subject continuously maintaining the virtue indicated by it. In a wife this will be fortunate; in a husband, evil.",
        "The topmost SIX, divided, shows its subject exciting himself to long continuance. There will be evil.",
    ],
    // Hexagram 33
    [
        "The first SIX, divided, shows a retiring tail. The position is perilous. No movement in any direction should be made.",
        "The second SIX, divided, shows its subject holding (his purpose) fast as if by a (thong made from the) hide of a yellow ox, which cannot be broken.",
        "The third NINE, undivided, shows one retiring but bound,--to his distress and peril. (If he were to deal with his binders as in) nourishing a servant or concubine, it would be fortunate for him.",
        "The fourth NINE, undivided, shows its subject retiring notwithstanding his likings. In a superior man this will lead to good fortune; a small man cannot attain to this.",
        "The fifth NINE, undivided, shows its subject retiring in an admirable way. With firm correctness there will be good fortune.",
        "The sixth NINE, undivided, shows its subject retiring in a noble way. It will be advantageous in every respect.",
    ],
    // Hexagram 34
    [
        "The first NINE, undivided, shows its subject manifesting his strength in his toes. But advance will lead to evil,--most certainly.",
        "The second NINE, undivided, shows that with firm correctness there will be good fortune.",
        "The third NINE, undivided, shows, in the case of a small man, one using all his strength; and in the case of a superior man, one whose rule is not to do so. Even with firm correctness the position would be perilous. (The exercise of strength in it might be compared to the case of) a ram butting against a fence, and getting his horns entangled.",
        "The fourth NINE, undivided, shows (a case in which) firm correctness leads to good fortune, and occasion for repentance disappears. (We see) the fence opened without the horns being entangled. The strength is like that in the wheel-spokes of a large waggon.",
        "The fifth SIX, divided, shows one who loses his ram(-like strength) in the ease of his position. (But) there will be no occasion for repentance.",
        "The sixth SIX, divided, shows (one who may be compared to) the ram butting against the fence, and unable either to retreat, or to advance as he would fain do. There will not be advantage in any respect; but if he realise the difficulty (of his position), there will be good fortune.",
    ],
    // Hexagram 35
    [
        "The first SIX, divided, shows one wishing to advance, and (at the same time) kept back. Let him be firm and correct, and there will be good fortune. If trust be not reposed in him, let him maintain a large and generous mind, and there will be no error.",
        "The second SIX, divided, shows its subject with the appearance of advancing, and yet of being sorrowful. If he be firm and correct, there will be good fortune. He will receive this great blessing from his grandmother.",
        "The third SIX, divided, shows its subject trusted by all (around him). All occasion for repentance will disappear.",
        "The fourth NINE, undivided, shows its subject with the appearance of advancing, but like a marmot. However firm and correct he may be, the position is one of peril.",
        "The fifth SIX, divided, shows how all occasion for repentance disappears (from its subject). (But) let him not concern himself about whether he shall fail or succeed. To advance will be fortunate, and in every way advantageous.",
        "The topmost NINE, undivided, shows one advancing his horns. But he only uses them to punish the (rebellious people of his own) city. The position is perilous, but there will be good fortune. (Yet) however firm and correct he may be, there will be occasion for regret.",
    ],
    // Hexagram 36
    [
        "The first NINE, undivided, shows its subject, (in the condition indicated by) Ming Î, flying, but with drooping wings. When the superior man (is revolving) his going away, he may be for three days without eating. Wherever he goes, the people there may speak (derisively of him).",
        "The second SIX, divided, shows its subject, (in the condition indicated by) Ming Î, wounded in the left thigh. He saves himself by the strength of a (swift) horse; and is fortunate.",
        "The third NINE, undivided, shows its subject, (in the condition indicated by) Ming Î, hunting in the south, and taking the great chief (of the darkness). He should not be eager to make (all) correct (at once).",
        "The fourth six, divided, shows its subject (just) entered into the left side of the belly (of the dark land). (But) he is able to carry out the mind appropriate (in the condition indicated by) Ming Î, quitting the gate and courtyard (of the lord of darkness).",
        "The fifth six, divided, shows how the count of K î fulfilled the condition indicated by Ming Î. It will be advantageous to be firm and correct.",
        "The sixth six, divided, shows the case where there is no light, but (only) obscurity. (Its subject) had at first ascended to (the top of) the sky; his future shall be to go into the earth.",
    ],
    // Hexagram 37
    [
        "The first NINE, undivided, shows its subject establishing restrictive regulations in his household Occasion for repentance will disappear.",
        "The second SIX, divided, shows its subject taking nothing on herself, but in her central place attending to the preparation of the food. Through her firm correctness there will be good fortune.",
        "The third NINE, undivided, shows its subject (treating) the members of the household with stern severity. There will be occasion for repentance, there will be peril, (but) there will (also) be good fortune. If the wife and children were to be smirking and chattering, in the end there would be occasion for regret.",
        "The fourth SIX, divided, shows its subject enriching the family. There will be great good fortune.",
        "The fifth NINE, undivided, shows the influence of the king extending to his family. There need be no anxiety; there will be good fortune.",
        "The topmost NINE, undivided, shows its subject possessed of sincerity and arrayed in majesty. In the end there will be good fortune.",
    ],
    // Hexagram 38
    [
        "The first NINE, undivided, shows that (to its subject) occasion for repentance will disappear. He has lost his horses, but let him not seek for them;--they will return of themselves. Should he meet with bad men, he will not err (in communicating with them).",
        "The second NINE, undivided, shows its subject happening to meet with his lord in a bye-passage. There will be no error.",
        "In the third SIX, divided, we see one whose carriage is dragged back, while the oxen in it are pushed back, and he is himself subjected to the shaving of his head and the cutting off of his nose. There is no good beginning, but there will be a good end.",
        "The fourth NINE, undivided, shows its subject solitary amidst the (prevailing) disunion. (But) he meets with the good man (represented by the first line), and they blend their sincere desires together. The position is one of peril, but there will be no mistake.",
        "The fifth SIX, divided, shows that (to its subject) occasion for repentance will disappear. With his relative (and minister he unites closely and readily) as if he were biting through a piece of skin. When he goes forward (with this help), what error can there be?",
        "The topmost NINE, undivided, shows its subject solitary amidst the (prevailing) disunion. (In the subject of the third line, he seems to) see a pig bearing on its back a load of mud, (or fancies) there is a carriage full of ghosts. He first bends his bow against him, and afterwards unbends it, (for he discovers) that he is not an assailant to injure, but a near relative. Going forward, he shall meet with (genial) rain, and there will be good fortune.",
    ],
    // Hexagram 39
    [
        "From the first SIX, divided, we learn that advance (on the part of its subject) will lead to (greater) difficulties, while remaining stationary will afford ground for praise.",
        "The second SIX, divided, shows the minister of the king struggling with difficulty on difficulty, and not with a view to his own advantage.",
        "The third NINE, undivided, shows its subject advancing, (but only) to (greater) difficulties. He remains stationary, and returns (to his former associates).",
        "The fourth SIX, divided, shows its subject advancing, (but only) to (greater) difficulties. He remains stationary, and unites (with the subject of the line above).",
        "The fifth NINE, undivided, shows its subject struggling with the greatest difficulties, while friends are coming to help him.",
        "The topmost SIX, divided, shows its subject going forward, (only to increase) the difficulties, while his remaining stationary will be (productive of) great (merit). There will be good fortune, and it will be advantageous to meet with the great man.",
    ],
    // Hexagram 40
    [
        "The first SIX, divided, shows that its subject will commit no error.",
        "The second NINE, undivided, shows its subject catch, in hunting, three foxes, and obtain the yellow (= golden) arrows. With firm correctness there will be good fortune.",
        "The third SIX, divided, shows a porter with his burden, (yet) riding in a carriage. He will (only) tempt robbers to attack him. However firm and correct he may (try to) be, there will be cause for regret.",
        "(To the subject of) the fourth NINE, undivided, (it is said), 'Remove your toes. Friends will (then) come, between you and whom there will be mutual confidence.'",
        "The fifth SIX, divided, shows (its subject), the superior man (= the ruler), executing his function of removing (whatever is injurious to the idea of the hexagram), in which case there will he good fortune, and confidence in him will be shown even by the small men.",
        "In the sixth SIX, divided, we see a feudal prince (with his bow) shooting at a falcon on the top of a high wall, and hitting it. (The effect of his action) will be in every way advantageous.",
    ],
    // Hexagram 41
    [
        "The first NINE, undivided, shows its subject suspending his own affairs, and hurrying away (to help the subject of the fourth line). He will commit no error, but let him consider how far he should contribute of what is his (for the other).",
        "The second NINE, undivided, shows that it will be advantageous for its subject to maintain a firm correctness, and that action on his part will be evil. He can give increase (to his correlate) without taking from himself",
        "The third SIX, divided, shows how of three men walking together, the number is diminished by one; and how one, walking, finds his friend.",
        "The fourth SIX, divided, shows its subject diminishing the ailment under which he labours by making (the subject of the first line) hasten (to his help), and make him glad. There will be no error.",
        "The fifth SIX, divided, shows parties adding to (the stores of) its subject ten pairs of tortoise shells, and accepting no refusal. There will be great good fortune.",
        "The topmost NINE, undivided, shows its subject giving increase to others without taking from himself. There will be no error. With firm correctness there will be good fortune. There will be advantage in every movement that shall be made. He will find ministers more than can be counted by their clans.",
    ],
    // Hexagram 42
    [
        "The first NINE, undivided, shows that it will be advantageous for its subject in his position to make a great movement. If it be greatly fortunate, no blame will be imputed to him.",
        "The second SIX, divided, shows parties adding to the stores of its subject ten pairs of tortoise shells whose oracles cannot be opposed. Let him persevere in being firm and correct, and there will be good fortune. Let the king, (having the virtues thus distinguished), employ them in presenting his offerings to God, and there will be good fortune.",
        "The third SIX, divided, shows increase given to its subject by means of what is evil, so that he shall (be led to good), and be without blame. Let him be sincere and pursue the path of the Mean, (so shall he secure the recognition of the ruler, like) an officer who announces himself to his prince by the symbol of his rank.",
        "The fourth SIX, divided, shows its subject pursuing the due course. His advice to his prince is followed. He can with advantage be relied on in such a movement as that of removing the capital.",
        "The fifth NINE, undivided, shows its subject with sincere heart seeking to benefit (all below). There need be no question about it; the result will be great good fortune. (All below) will with sincere heart acknowledge his goodness.",
        "In the sixth NINE, undivided, we see one to whose increase none will contribute, while many will seek to assail him. He observes no regular rule in the ordering of his heart. There will be evil.",
    ],
    // Hexagram 43
    [
        "The first NINE, undivided, shows its subject in (the pride of) strength advancing with his toes. He goes forward, but will not succeed. There will be ground for blame.",
        "The second NINE, undivided, shows its subject full of apprehension and appealing (for sympathy and help). Late at night hostile measures may be (taken against him), but he need not be anxious about them.",
        "The third NINE, undivided, shows its subject (about to advance) with strong (and determined) looks. There will be evil. (But) the superior man, bent on cutting off (the criminal), will walk alone and encounter the rain, (till he be hated by his proper associates) as if he were contaminated (by the others). (In the end) there will be no blame against him.",
        "The fourth NINE, undivided, shows one from whose buttocks the skin has been stripped, and who walks slowly and with difficulty. (If he could act) like a sheep led (after its companions), occasion for repentance would disappear. But though he hear these words, he will not believe them.",
        "The fifth NINE, undivided, shows (the small men like) a bed of purslain, which ought to be uprooted with the utmost determination. (The subject of the line having such determination), his action, in harmony with his central position, will lead to no error or blame.",
        "The sixth SIX, divided, shows its subject without any (helpers) on whom to call. His end will be evil.",
    ],
    // Hexagram 44
    [
        "The first SIX, divided, shows how its subject should be kept (like a carriage) tied and fastened to a metal drag, in which case with firm correctness there will be good fortune. (But) if he move in any direction, evil will appear. He will be (like) a lean pig, which is sure to keep jumping about.",
        "The second NINE, undivided, shows its subject with a wallet of fish. There will be no error. But it will not be well to let (the subject of the first line) go forward to the guests.",
        "The third NINE, undivided, shows one from whose buttocks the skin has been stripped so that he walks with difficulty. The position is perilous, but there will be no great error.",
        "The fourth NINE, undivided, shows its subject with his wallet, but no fish in it. This will give rise to evil.",
        "The fifth NINE, undivided, (shows its subject as) a medlar tree overspreading the gourd (beneath it). If he keep his brilliant qualities concealed, (a good issue) will descend (as) from Heaven.",
        "The sixth NINE, undivided, shows its subject receiving others on his horns. There will be occasion for regret, but there will be no error.",
    ],
    // Hexagram 45
    [
        "The first SIX, divided, shows its subject with a sincere desire (for union), but unable to carry it out, so that disorder is brought into the sphere of his union. If he cry out (for help to his proper correlate), all at once (his tears) will give place to smiles. He need not mind (the temporary difficulty); as he goes forward, there will be no error.",
        "The second SIX, divided, shows its subject led forward (by his correlate). There will be good fortune, and freedom from error. There is entire sincerity, and in that case (even the small offerings of) the vernal sacrifice are acceptable.",
        "The third SIX, divided, shows its subject striving after union and seeming to sigh, yet nowhere finding any advantage. If he go forward, he will not err, though there may be some small cause for regret.",
        "The fourth NINE, undivided, shows its subject in such a state that, if he be greatly fortunate, he will receive no blame.",
        "The fifth NINE, undivided, shows the union (of all) under its subject in the place of dignity. There will be no error. If any do not have confidence in him, let him see to it that (his virtue) be great, long-continued, and firmly correct, and all occasion for repentance will disappear.",
        "The topmost SIX, divided, shows its subject sighing and weeping; but there will be no error.",
    ],
    // Hexagram 46
    [
        "The first SIX, divided, shows its subject advancing upwards with the welcome (of those above him). There will be great good fortune.",
        "The second NINE, undivided, shows its subject with that sincerity which will make even the (small) offerings of the vernal sacrifice acceptable. There will be no error.",
        "The third NINE, undivided, shows its subject ascending upwards (as into) an empty city.",
        "The fourth SIX, divided, shows its subject employed by the king to present his offerings on mount Kh î. There will be good fortune; there will be no mistake.",
        "The fifth six, divided, shows its subject firmly correct, and therefore enjoying good fortune. He ascends the stairs (with all due ceremony).",
        "The sixth six, divided, shows its subject advancing upwards blindly. Advantage will be found in a ceaseless maintenance of firm correctness.",
    ],
    // Hexagram 47
    [
        "The first SIX, divided, shows its subject with bare buttocks straitened under the stump of a tree. He enters a dark valley, and for three years has no prospect (of deliverance).",
        "The second NINE, undivided, shows its subject straitened amidst his wine and viands. There come to him anon the red knee-covers (of the ruler). It will be well for him (to maintain his sincerity as) in sacrificing. Active operations (on his part) will lead to evil, but he will be free from blame.",
        "The third SIX, divided, shows its subject straitened before a (frowning) rock. He lays hold of thorns. He enters his palace, and does not see his wife. There will be evil.",
        "The fourth NINE, undivided shows its subject proceeding very slowly (to help the subject of the first line), who is straitened by the carriage adorned with metal in front of him. There will be occasion for regret, but the end will be good.",
        "The fifth NINE, undivided, shows its subject with his nose and feet cut off. He is straitened by (his ministers in their) scarlet aprons. He is leisurely in his movements, however, and is satisfied. It will be well for him to be (as sincere) as in sacrificing (to spiritual beings).",
        "The sixth SIX, divided, shows its subject straitened, as if bound with creepers; or n a high and dangerous position, and saying (to himself), 'If I move, I shall repent it.' If he do repent of former errors, there will be good fortune in his going forward.",
    ],
    // Hexagram 48
    [
        "The first SIX, divided, shows a well so muddy that men will not drink of it; or an old well to which neither birds (nor other creatures) resort.",
        "The second NINE, undivided, shows a well from which by a hole the water escapes and flows away to the shrimps (and such small creatures among the grass), or one the water of which leaks away from a broken basket.",
        "The third NINE, undivided, shows a well, which has been cleared out, but is not used. Our hearts are sorry for this, for the water might be drawn out and used. If the king were (only) intelligent, both he and we might receive the benefit of it.",
        "The fourth SIX, divided, shows a well, the lining of which is well laid. There will be no error.",
        "The fifth NINE, undivided, shows a clear, limpid well, (the waters from) whose cold spring are (freely) drunk.",
        "The topmost SIX, divided, shows (the water from) the well brought to the top, which is not allowed to be covered. This suggests the idea of sincerity. There will be great good fortune.",
    ],
    // Hexagram 49
    [
        "The first NINE, undivided, shows its subject (as if he were) bound with the skin of a yellow ox.",
        "The second SIX, divided, shows its subject making his changes after some time has passed. Action taken will be fortunate. There will be no error.",
        "The third NINE, undivided, shows that action taken by its subject will be evil. Though he be firm and correct, his position is perilous. If the change (he contemplates) have been three times fully discussed, he will be believed in.",
        "The fourth NINE, undivided, shows occasion for repentance disappearing (from its subject). Let him be believed in; and though he change (existing) ordinances, there will be good fortune.",
        "The fifth NINE, undivided, shows the great man (producing his changes) as the tiger (does when he) changes (his stripes). Before he divines (and proceeds to action), faith has been reposed in him.",
        "The sixth SIX, divided, shows the superior man producing his changes as the leopard (does when he) changes (his spots), while small men change their faces (and show their obedience). To go forward (now) would lead to evil, but there will be good fortune in abiding firm and correct.",
    ],
    // Hexagram 50
    [
        "The first SIX, divided, shows the caldron overthrown and its feet turned up. (But) there will be advantage in its getting rid of what was bad in it. (Or it shows us) the concubine (whose position is improved) by means of her son. There will be no error.",
        "The second NINE, undivided, shows the caldron with the things (to be cooked) in it. (If its subject can say), 'My enemy dislikes me, but he cannot approach me,' there will be good fortune.",
        "The third NINE, undivided, shows the caldron with (the places of) its ears changed. The progress (of its subject) is (thus) stopped. The fat flesh of the pheasant (which is in the caldron) will not be eaten. But the (genial) rain will come, and the grounds for repentance will disappear. There will be good fortune in the end.",
        "The fourth NINE, undivided, shows the caldron with its feet broken; and its contents, designed for the ruler's use, overturned and spilt. Its Subject will be made to blush for shame. There will be evil.",
        "The fifth six, divided, shows the caldron with yellow ears and rings of metal in them. There will be advantage through being firm and correct.",
        "The sixth NINE, undivided, shows the caldron with rings of jade. There will be great good fortune, and all action taken will be in every way advantageous.",
    ],
    // Hexagram 51
    [
        "The first NINE, undivided, shows its subject, when the movement approaches, looking out and around with apprehension, and afterwards smiling and talking cheerfully. There will be good fortune.",
        "The second SIX, divided, shows its subject, when the movement approaches, in a position of peril. He judges it better to let go the articles (in his possession), and to ascend a very lofty height. There is no occasion for him to pursue after (the things he has let go); in seven days he will find them.",
        "The third six, divided, shows its subject distraught amid the startling movements going on. If those movements excite him to (right) action, there will be no mistake.",
        "The fourth NINE, undivided, shows its subject, amid the startling movements, supinely sinking (deeper) in the mud.",
        "The fifth SIX, divided, shows its subject going and coming amidst the startling movements (of the time), and always in peril; but perhaps he will not incur loss, and find business (which he can accomplish).",
        "The topmost SIX, divided, shows its subject, amidst the startling movements (of the time), in breathless dismay and looking round him with trembling apprehension. If he take action, there will be evil. If, while the startling movements have not reached his own person and his neighbourhood, (he were to take precautions), there would be no error, though his relatives might (still) speak against him.",
    ],
    // Hexagram 52
    [
        "The first SIX, divided, shows its subject keeping his toes at rest. There will be no error; but it will be advantageous for him to be persistently firm and correct.",
        "The second SIX, divided, shows its subject keeping the calves of his legs at rest. He cannot help (the subject of the line above) whom he follows, and is dissatisfied in his mind.",
        "The third NINE, undivided, shows its subject keeping his loins at rest, and separating the ribs (from the body below). The situation is perilous, and the heart glows with suppressed excitement.",
        "The fourth SIX, divided, shows its subject keeping his trunk at rest. There will be no error.",
        "The fifth SIX, divided, shows its subject keeping his jawbones at rest, so that his words are (all) orderly. Occasion for repentance will disappear.",
        "The sixth NINE, undivided, shows its subject devotedly maintaining his restfulness. There will be good fortune.",
    ],
    // Hexagram 53
    [
        "The first SIX, divided, shows the wild geese gradually approaching the shore. A young officer (in similar circumstances) will be in a position of danger, and be spoken against; but there will be no error.",
        "The second SIX, divided, shows the geese gradually approaching the large rocks, where they eat and drink joyfully and at ease. There will be good fortune.",
        "The third NINE, undivided, shows them gradually advanced to the dry plains. (It suggests also the idea of) a husband who goes on an expedition from which he does not return, and of a wife who is pregnant, but will not nourish her child. There will be evil. (The case symbolised) might be advantageous in resisting plunderers.",
        "The fourth SIX, divided, shows the geese gradually advanced to the trees. They may light on the flat branches. There will be no error.",
        "The fifth NINE, undivided, shows the geese gradually advanced to the high mound. (It suggests the idea of) a wife who for three years does not become pregnant; but in the end the natural issue cannot be prevented. There will be good fortune.",
        "The sixth NINE, undivided, shows the geese gradually advanced to the large heights (beyond). Their feathers can be used as ornaments. There will be good fortune.",
    ],
    // Hexagram 54
    [
        "The first NINE, undivided, shows the younger sister married off in a position ancillary to the real wife. (It suggests the idea of) a person lame on one leg who yet manages to tramp along. Going forward will be fortunate.",
        "The second NINE, undivided, shows her blind of one eye, and yet able to see. There will be advantage in her maintaining the firm correctness of a solitary widow.",
        "The third SIX, divided, shows the younger sister who was to be married off in a mean position. She returns and accepts an ancillary position.",
        "The fourth NINE, undivided, shows the younger sister who is to be married off protracting the time. She may be late in being married, but the time will come.",
        "The fifth SIX, divided, reminds us of the marrying of the younger sister of (king) Tî-yî, when the sleeves of her the princess were not equal to those of the (still) younger sister who accompanied her in an inferior capacity. (The case suggests the thought of) the moon almost full. There will be good fortune.",
        "The sixth SIX, divided, shows the young lady bearing the basket, but without anything in it, and the gentleman slaughtering the sheep, but without blood flowing from it. There will be no advantage in any way.",
    ],
    // Hexagram 55
    [
        "The first NINE, undivided, shows its subject meeting with his mate. Though they are both of the same character, there will be no error. Advance will call forth approval.",
        "The second SIX, divided, shows its subject surrounded by screens so large and thick that at midday he can see from them the constellation of the Bushel. If he go (and try to enlighten his ruler who is thus emblemed), he will make himself to be viewed with suspicion and dislike. Let him cherish his feeling of sincere devotion that he may thereby move (his ruler's mind), and there will be good fortune.",
        "The third NINE, undivided, shows its subject with an (additional) screen of a large and thick banner, through which at midday he can see (the small) Mei star. (In the darkness) he breaks his right arm; but there will be no error.",
        "The fourth NINE, undivided, shows its subject in a tent so large and thick that at midday he can see from it the constellation of the Bushel. But he meets with the subject of the (first) line, undivided like himself. There will be good fortune.",
        "The fifth SIX, divided, shows its subject bringing around him the men of brilliant ability. There will be occasion for congratulation and praise. There will be good fortune.",
        "The topmost SIX, divided, shows its subject with his house made large, but only serving as a screen to his household. When he looks at his door, it is still, and there is nobody about it. For three years no one is to be seen. There will be evil.",
    ],
    // Hexagram 56
    [
        "The first SIX, divided, shows the stranger mean and meanly occupied. It is thus that he brings on himself (further) calamity.",
        "The second SIX, divided, shows the stranger, occupying his lodging-house, carrying with him his means of livelihood, and provided with good and trusty servants.",
        "The third NINE, undivided, shows the stranger, burning his lodging-house, and having lost his servants. However firm and correct he (try to) be, he will be in peril.",
        "The fourth NINE, undivided, shows the traveller in a resting-place, having (also) the means of livelihood and the axe, (but still saying), 'I am not at ease in my mind.'",
        "The fifth SIX, divided, shows its subject shooting a pheasant. He will lose his arrow, but in the end he will obtain praise and a (high) charge.",
        "The sixth NINE, undivided, suggests the idea of a bird burning its nest. The stranger, (thus represented), first laughs and then cries out. He has lost his ox(-like docility) too readily and easily. There will be evil.",
    ],
    // Hexagram 57
    [
        "The first SIX, divided, shows its subject (now) advancing, (now) receding. It would be advantageous for him to have the firm correctness of a brave soldier.",
        "The second NINE, undivided, shows the representative of Sun beneath a couch, and employing diviners and exorcists in a way bordering on confusion. There will be good fortune and no error.",
        "The third NINE, undivided, shows its subject penetrating (only) by violent and repeated efforts. There will be occasion for regret.",
        "The fourth SIX, divided, shows all occasion for repentance (in its subject) passed away. He takes game for its threefold use in his hunting.",
        "The fifth NINE, undivided, shows that with firm correctness there will be good fortune (to its subject). All occasion for repentance will disappear, and all his movements will be advantageous. There may have been no (good) beginning, but there will be a (good) end. Three days before making any changes, (let him give notice of them); and three days after, (let him reconsider them). There will (thus) be good fortune.",
        "The sixth NINE, undivided, shows the representative of penetration beneath a couch, and having lost the axe with which he executed his decisions. However firm and correct he may (try to) be, there will be evil.",
    ],
    // Hexagram 58
    [
        "The first NINE, undivided, shows the pleasure of (inward) harmony. There will be good fortune.",
        "The second NINE, undivided, shows the pleasure arising from (inward) sincerity. There will be good fortune. Occasion for repentance will disappear.",
        "The third SIX, divided, shows its subject bringing round himself whatever can give pleasure. There will be evil.",
        "The fourth NINE, undivided, shows its subject deliberating about what to seek his pleasure in, and not at rest. He borders on what would be injurious, but there will be cause for joy.",
        "The fifth NINE, undivided, shows its subject trusting in one who would injure him. The situation is perilous.",
        "The topmost SIX, divided, shows the pleasure of its subject in leading and attracting others.",
    ],
    // Hexagram 59
    [
        "The first SIX, divided, shows its subject engaged in rescuing (from the impending evil) and having (the assistance of) a strong horse. There will be good fortune.",
        "The second NINE, undivided, shows its subject, amid the dispersion, hurrying to his contrivance (for security). All occasion for repentance will disappear.",
        "The third SIX, divided, shows its subject discarding any regard to his own person. There will be no occasion for repentance.",
        "The fourth SIX, divided, shows its subject scattering the (different) parties (in the state); which leads to great good fortune. From the dispersion (he collects again good men standing out, a crowd) like a mound, which is what ordinary men would not have thought of.",
        "The fifth NINE, undivided, shows its subject amidst the dispersion issuing his great announcements as the perspiration (flows from his body). He scatters abroad (also) the accumulations in the royal granaries. There will be no error.",
        "The topmost NINE, undivided, shows its subject disposing of (what may be called) its bloody wounds, and going and separating himself from its anxious fears. There will be no error.",
    ],
    // Hexagram 60
    [
        "The first NINE, undivided, shows its subject not quitting the courtyard outside his door. There will be no error.",
        "The second NINE, undivided, shows its subject not quitting the courtyard inside his gate. There will be evil.",
        "The third SIX, divided, shows its subject with no appearance of observing the (proper) regulations, in which case we shall see him lamenting. But there will be no one to blame (but himself).",
        "The fourth SIX, divided, shows its subject quietly and naturally (attentive to all) regulations. There will be progress and success.",
        "The fifth NINE, undivided, shows its subject sweetly and acceptably enacting his regulations. There will be good fortune. The onward progress with them will afford ground for admiration.",
        "The topmost SIX, divided, shows its subject enacting regulations severe and difficult. Even with firmness and correctness there will be evil. But though there will be cause for repentance, it will (by and by) disappear.",
    ],
    // Hexagram 61
    [
        "The first NINE, undivided, shows its subject resting (in himself). There will be good fortune. If he sought to any other, he would not find rest.",
        "The second NINE, undivided, shows its subject (like) the crane crying out in her hidden retirement, and her young ones responding to her. (It is as if it were said), 'I have a cup of good spirits,' (and the response were), 'I will partake of it with you.'",
        "The third SIX, divided, shows its subject having met with his mate. Now he beats his drum, and now he leaves off. Now he weeps, and now he sings.",
        "The fourth SIX, divided, shows its subject (like) the moon nearly full, and (like) a horse (in a chariot) whose fellow disappears. There will be no error.",
        "The fifth NINE, undivided, shows its subject perfectly sincere, and linking (others) to him in closest union. There will be no error.",
        "The topmost NINE, undivided, shows its subject in chanticleer (trying to) mount to heaven. Even with firm correctness there will be evil.",
    ],
    // Hexagram 62
    [
        "The first SIX, divided, suggests (the idea of) a bird flying, (and ascending) till the issue is evil.",
        "The second SIX, divided, shows its subject passing by his grandfather, and meeting with his grandmother; not attempting anything against his ruler, but meeting him as his minister. There will be no error.",
        "The third NINE, undivided, shows its subject taking no extraordinary precautions against danger; and some in consequence finding opportunity to assail and injure him. There will be evil.",
        "The fourth NINE, undivided, shows its subject falling into no error, but meeting (the exigency of his situation), without exceeding (in his natural course). If he go forward, there will be peril, and he must be cautious. There is no occasion to be using firmness perpetually.",
        "The fifth SIX, divided, (suggests the idea) of dense clouds, but no rain, coming from our borders in the west. It also (shows) the prince shooting his arrow, and taking the bird in a cave.",
        "The sixth SIX, divided, shows its subject not meeting (the exigency of his situation), and exceeding (his proper course). (It suggests the idea of) a bird flying far aloft. There will be evil. The case is what is called one of calamity and self-produced injury.",
    ],
    // Hexagram 63
    [
        "The first NINE, undivided, (shows its subject as a driver) who drags back his wheel, (or as a fox) which has wet his tail. There will be no error.",
        "The second SIX, divided, (shows its subject as) a wife who has lost her (carriage-)screen. There is no occasion to go in pursuit of it. In seven days she will find it.",
        "The third NINE, undivided, (suggests the case of) Kâo Žung, who attacked the Demon region, but was three years in subduing it. Small men should not be employed (in such enterprises).",
        "The fourth SIX, divided, shows its subject with rags provided against any leak (in his boat), and on his guard all day long.",
        "The fifth NINE, undivided, shows its subject (as) the neighbour in the east who slaughters an ox (for his sacrifice); but this is not equal to the (small) spring sacrifice of the neighbour in the west, whose sincerity receives the blessing.",
        "The topmost SIX, divided, shows its subject with (even) his head immersed. The position is perilous.",
    ],
    // Hexagram 64
    [
        "The first SIX, divided, shows its subject (like a fox) whose tail gets immersed. There will be occasion for regret.",
        "The second NINE, undivided, shows its subject dragging back his (carriage-) wheel. With firmness and correctness there will be good fortune.",
        "The third SIX, divided, shows its subject, with (the state of things) not yet remedied, advancing on; which will lead to evil. But there will be advantage in (trying to) cross the great stream.",
        "The fourth NINE, undivided, shows its subject by firm correctness obtaining good fortune, so that all occasion for repentance disappears. Let him stir himself up, as if he were invading the Demon region, where for three years rewards will come to him (and his troops) from the great kingdom.",
        "The fifth SIX, divided, shows its subject by firm correctness obtaining good fortune, and having no occasion for repentance. (We see in him) the brightness of a superior man, and the possession of sincerity. There will be good fortune.",
        "The topmost NINE, undivided, shows its subject full of confidence and therefore feasting (quietly). There will be no error. (If he) cherish this confidence, till he (is like the fox who) gets his head immersed, it will fail of what is right.",
    ],
];

/// Legge's supplementary "use of the number" statements, given only for
/// Hexagram 1 (Khien — "the use of the number NINE") and Hexagram 2
/// (Khwăn — "the use of the number six"). These are the seventh paragraph
/// unique to the two all-Yang / all-Yin hexagrams, not ordinary line texts.
/// Verbatim from Legge (SBE XVI); `None` for every other hexagram.
static USE_LINE_TEXTS: [Option<&str>; 64] = {
    let mut t: [Option<&str>; 64] = [None; 64];
    t[0] = Some(
        "(The lines of this hexagram are all strong and undivided, as appears from) the use of the number NINE. If the host of dragons (thus) appearing were to divest themselves of their heads, there would be good fortune.",
    );
    t[1] = Some(
        "(The lines of this hexagram are all weak and divided, as appears from) the use of the number six. If those (who are thus represented) be perpetually correct and firm, advantage will arise.",
    );
    t
};

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

/// Return Legge's six line texts (yao ci) for the hexagram with the given King
/// Wen number (1-64), bottom line (index 0) to top line (index 5).
///
/// Returns `None` if `number` is 0 or greater than 64. The text is verbatim
/// from James Legge, *The Yî King* (Sacred Books of the East Vol. XVI, 1882),
/// public domain. See [`LINE_TEXTS`].
pub fn line_texts(number: u8) -> Option<&'static [&'static str; 6]> {
    if (1..=64).contains(&number) {
        Some(&LINE_TEXTS[(number - 1) as usize])
    } else {
        None
    }
}

/// Return Legge's line text (yao ci) for a single line of a hexagram.
///
/// `number` is the King Wen hexagram number (1-64); `line` is the 0-based line
/// index counting from the bottom (0 = first/lowest line, 5 = top line). This
/// matches [`HexagramReading::changing_line`] and the `changing_line` argument
/// of [`relating_hexagram`], so the text for a moving line is simply
/// `line_text(reading.primary.number, reading.changing_line)`.
///
/// Returns `None` if `number` is out of range (0 or >64) or `line` is >= 6.
/// Text is verbatim public-domain Legge (SBE XVI, 1882).
pub fn line_text(number: u8, line: usize) -> Option<&'static str> {
    if line >= 6 {
        return None;
    }
    line_texts(number).map(|lines| lines[line])
}

/// Return Legge's supplementary "use of the number" statement, which exists
/// only for Hexagram 1 (Khien, "the use of the number NINE") and Hexagram 2
/// (Khwăn, "the use of the number six"). Every other hexagram — and any
/// out-of-range `number` — returns `None`.
///
/// This is **not** one of the six ordinary line texts; it is the seventh
/// paragraph Legge gives only for the two all-Yang / all-Yin hexagrams.
/// Verbatim public-domain Legge (SBE XVI, 1882). See [`USE_LINE_TEXTS`].
pub fn use_line_text(number: u8) -> Option<&'static str> {
    if (1..=64).contains(&number) {
        USE_LINE_TEXTS[(number - 1) as usize]
    } else {
        None
    }
}

/// Derive a hexagram reading from a date/time.
///
/// The method used (Mei Hua Yi Shu — Plum Blossom Numerology, Shao Yong):
/// - Upper trigram: `(year + month + day) % 8`, read in the Pre-Heaven
///   (Xian Tian / Fu Xi) Ba Gua sequence (1=Qian … 8=Kun; remainder 0 → Kun).
/// - Lower trigram: `(year + month + day + hour) % 8`, same Pre-Heaven sequence.
/// - Changing line:  `(year + month + day + hour) % 6`, where the divisor 6
///   yields the 6th line (so a remainder of 0 maps to the top line, index 5).
///
/// The Pre-Heaven sequence is essential: the previous code indexed the trigram
/// table by enum-declaration order, which silently mapped remainder 0 → Qian
/// (and 1 → Kun, …), inverting the classical assignment. See
/// [`Trigram::from_pre_heaven_number`].
pub fn hexagram_from_date(year: i32, month: u32, day: u32, hour: u32) -> HexagramReading {
    let sum_upper = (year.unsigned_abs() as u64)
        .wrapping_add(month as u64)
        .wrapping_add(day as u64);
    let sum_lower = sum_upper.wrapping_add(hour as u64);

    let upper = Trigram::from_pre_heaven_number(sum_upper);
    let lower = Trigram::from_pre_heaven_number(sum_lower);

    // Changing line: remainder mod 6. Classical method counts lines 1..=6, so a
    // remainder of 0 means the 6th line. Convert to a 0-based index (0..=5).
    let line_number = match sum_lower % 6 {
        0 => 6,
        r => r,
    };
    let changing = (line_number - 1) as usize;

    let primary = hexagram_from_trigrams(upper, lower);
    let relating = relating_hexagram_inner(&primary.lines, changing);
    // `changing` is always 0..=5 and `primary.number` is always 1..=64, so the
    // lookup is guaranteed to succeed; fall back to "" only to stay total.
    let changing_line_text = line_text(primary.number, changing).unwrap_or("");

    HexagramReading {
        primary,
        changing_line: changing,
        relating,
        changing_line_text,
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
    fn pre_heaven_sequence_matches_mei_hua() {
        // Mei Hua Yi Shu Pre-Heaven (Xian Tian) numbering: 1=Qian … 8=Kun.
        assert_eq!(Trigram::from_pre_heaven_number(1), Trigram::Qian);
        assert_eq!(Trigram::from_pre_heaven_number(2), Trigram::Dui);
        assert_eq!(Trigram::from_pre_heaven_number(3), Trigram::Li);
        assert_eq!(Trigram::from_pre_heaven_number(4), Trigram::Zhen);
        assert_eq!(Trigram::from_pre_heaven_number(5), Trigram::Xun);
        assert_eq!(Trigram::from_pre_heaven_number(6), Trigram::Kan);
        assert_eq!(Trigram::from_pre_heaven_number(7), Trigram::Gen);
        assert_eq!(Trigram::from_pre_heaven_number(8), Trigram::Kun);
        // The defining property: a remainder of 0 (divisor 8) is Kun, the 8th.
        assert_eq!(Trigram::from_pre_heaven_number(0), Trigram::Kun);
        assert_eq!(Trigram::from_pre_heaven_number(16), Trigram::Kun);
        // And it wraps past 8.
        assert_eq!(Trigram::from_pre_heaven_number(9), Trigram::Qian);
    }

    #[test]
    fn hexagram_from_date_uses_pre_heaven_trigrams() {
        // Hand-computed: pick a date whose digit sums land on known trigrams.
        // sum_upper = 8 + 4 + 4 = 16 -> 16 % 8 = 0 -> Kun (Pre-Heaven 8th).
        // sum_lower = 16 + 6 = 22 -> 22 % 8 = 6 -> Kan (Pre-Heaven 6th).
        // changing  = 22 % 6 = 4 -> 4th line -> index 3.
        let reading = hexagram_from_date(8, 4, 4, 6);
        assert_eq!(
            reading.primary.upper_trigram,
            Trigram::Kun,
            "upper should be Kun (remainder 0 -> 8th trigram)"
        );
        assert_eq!(
            reading.primary.lower_trigram,
            Trigram::Kan,
            "lower should be Kan (Pre-Heaven 6)"
        );
        assert_eq!(reading.changing_line, 3, "4th line -> 0-based index 3");
    }

    #[test]
    fn hexagram_from_date_changing_line_zero_remainder_is_top_line() {
        // sum_lower divisible by 6 must map to the 6th line (index 5), never panic.
        // year+month+day+hour = 6 + 0 + 0 + 0 = 6 ; 6 % 6 == 0 -> line 6 -> index 5.
        let reading = hexagram_from_date(6, 0, 0, 0);
        assert_eq!(reading.changing_line, 5);
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

    // -----------------------------------------------------------------------
    // Per-line (yao ci) Legge text — coverage & correctness
    // -----------------------------------------------------------------------

    #[test]
    fn line_texts_table_has_64_rows() {
        assert_eq!(LINE_TEXTS.len(), 64);
    }

    #[test]
    fn all_384_lines_present_and_nonempty() {
        // Every one of the 64 hexagrams must carry six non-empty Legge line
        // texts: 64 * 6 = 384. None may be the empty string (that would mean
        // a MISSING line slipped through the transcription).
        let mut count = 0usize;
        for n in 1..=64u8 {
            let lines = line_texts(n).expect("hexagram in range must have line texts");
            for (i, &t) in lines.iter().enumerate() {
                assert!(
                    !t.trim().is_empty(),
                    "Hexagram {n} line {} is empty — transcription gap",
                    i + 1
                );
                count += 1;
            }
        }
        assert_eq!(count, 384, "expected exactly 384 line texts");
    }

    #[test]
    fn each_line_opens_with_its_ordinal() {
        // Legge opens every line statement with the ordinal of the line it
        // describes ("first", "second", … "sixth"/"topmost"). This guards
        // against any row being misaligned (a line stored in the wrong slot).
        // A few statements open with a preposition or parenthetical before the
        // ordinal (e.g. hex 39 "From the first SIX", hex 40 "(To the subject
        // of) the fourth NINE"), so we only require the ordinal word to appear
        // near the start of the statement.
        let ordinals: [&[&str]; 6] = [
            &["first"],
            &["second"],
            &["third"],
            &["fourth"],
            &["fifth"],
            &["sixth", "topmost"],
        ];
        for n in 1..=64u8 {
            let lines = line_texts(n).unwrap();
            for (i, &t) in lines.iter().enumerate() {
                let head = t.chars().take(50).collect::<String>().to_lowercase();
                let ok = ordinals[i].iter().any(|w| head.contains(w));
                // Char-boundary-safe preview for the failure message (line texts
                // contain multibyte diacritics, so never slice by byte index).
                let preview: String = t.chars().take(60).collect();
                assert!(
                    ok,
                    "Hexagram {n} line {} does not open with ordinal {:?}: {:?}",
                    i + 1,
                    ordinals[i],
                    preview
                );
            }
        }
    }

    #[test]
    fn line_text_known_values() {
        // Hexagram 1 (Khien / The Creative), bottom line: the famous
        // "dragon lying hid" statement — verbatim Legge.
        assert_eq!(
            line_text(1, 0).unwrap(),
            "In the first (or lowest) NINE, undivided, (we see its subject as) the dragon lying hid (in the deep). It is not the time for active doing."
        );
        // Hexagram 1, fifth line: "the dragon on the wing in the sky".
        assert!(
            line_text(1, 4)
                .unwrap()
                .contains("the dragon on the wing in the sky")
        );
        // Hexagram 2 (Khwăn / The Receptive), bottom line: "treading on
        // hoarfrost" — preserves the diacritic context of Khwăn.
        assert_eq!(
            line_text(2, 0).unwrap(),
            "In the first SIX, divided, (we see its subject) treading on hoarfrost. The strong ice will come (by and by)."
        );
        // Hexagram 2, fifth line: "the yellow lower garment".
        assert!(
            line_text(2, 4)
                .unwrap()
                .contains("the yellow lower garment")
        );
    }

    #[test]
    fn line_text_out_of_range() {
        assert!(line_text(0, 0).is_none(), "hexagram 0 is invalid");
        assert!(line_text(65, 0).is_none(), "hexagram 65 is invalid");
        assert!(line_text(1, 6).is_none(), "line index 6 is invalid");
        assert!(line_text(1, 99).is_none(), "line index 99 is invalid");
        assert!(line_texts(0).is_none());
        assert!(line_texts(65).is_none());
    }

    #[test]
    fn hexagram_method_line_text_matches_free_function() {
        for n in 1..=64u8 {
            let h = hexagram(n).unwrap();
            assert_eq!(h.line_texts(), line_texts(n).unwrap());
            for i in 0..6 {
                assert_eq!(h.line_text(i), line_text(n, i));
            }
            assert!(h.line_text(6).is_none());
        }
    }

    #[test]
    fn use_line_texts_only_for_hex_1_and_2() {
        // Legge gives the "use of the number" paragraph only for the all-Yang
        // (Khien) and all-Yin (Khwăn) hexagrams.
        assert!(
            use_line_text(1)
                .unwrap()
                .contains("the use of the number NINE")
        );
        assert!(
            use_line_text(2)
                .unwrap()
                .contains("the use of the number six")
        );
        for n in 3..=64u8 {
            assert!(
                use_line_text(n).is_none(),
                "Hexagram {n} must not have a use-line"
            );
        }
        assert!(use_line_text(0).is_none());
        assert!(use_line_text(65).is_none());
        // Method form agrees with the free function.
        assert_eq!(hexagram(1).unwrap().use_line_text(), use_line_text(1));
        assert_eq!(hexagram(2).unwrap().use_line_text(), use_line_text(2));
        assert_eq!(hexagram(3).unwrap().use_line_text(), None);
    }

    #[test]
    fn use_line_not_appended_to_line_six() {
        // Regression guard: the seventh "use of the number" paragraph for
        // hexagrams 1 and 2 must NOT be folded into the sixth line text.
        for n in [1u8, 2] {
            let line6 = line_text(n, 5).unwrap();
            assert!(
                !line6.to_lowercase().contains("use of the number"),
                "Hexagram {n} line 6 wrongly absorbed the use-of-number paragraph"
            );
        }
    }

    #[test]
    fn date_reading_changing_line_text_is_consistent() {
        // The reading's changing_line_text must equal the line text for the
        // primary hexagram's changing line, for any input.
        for (y, m, d, h) in [
            (2024, 6, 15, 10),
            (8, 4, 4, 6),
            (6, 0, 0, 0),
            (1990, 11, 29, 23),
            (-44, 3, 15, 12),
        ] {
            let r = hexagram_from_date(y, m, d, h);
            assert_eq!(
                r.changing_line_text,
                line_text(r.primary.number, r.changing_line).unwrap(),
                "changing_line_text mismatch for {y}-{m}-{d} {h}h"
            );
            assert!(!r.changing_line_text.is_empty());
        }
    }

    #[test]
    fn line_texts_preserve_legge_diacritics_somewhere() {
        // The transcription preserves Legge's romanisation diacritics
        // (â, î, Î, Ž). At least one line text must contain a non-ASCII
        // character — proof we did not silently flatten the source to ASCII.
        let has_non_ascii = (1..=64u8)
            .flat_map(|n| line_texts(n).unwrap().iter())
            .any(|t| t.chars().any(|c| !c.is_ascii()));
        assert!(
            has_non_ascii,
            "expected Legge diacritics (non-ASCII) to be preserved in line texts"
        );
    }

    #[test]
    fn no_source_ocr_artifacts_in_any_text() {
        // Regression guard against scan artifacts inherited from the source
        // OCR. Every line statement plus every judgment and image text must be
        // free of editorial navigation markers and stray square brackets. The
        // "[paragraph continues]" marker that once sat in Hexagram 59 line 5 is
        // the canonical example.
        for n in 1..=64u8 {
            let h = hexagram(n).unwrap();
            let mut texts: Vec<&str> = line_texts(n).unwrap().to_vec();
            texts.push(h.judgment);
            texts.push(h.image);
            if let Some(u) = use_line_text(n) {
                texts.push(u);
            }
            for t in texts {
                assert!(
                    !t.contains("[paragraph continues]"),
                    "Hexagram {n}: '[paragraph continues]' OCR marker present"
                );
                assert!(
                    !t.contains('[') && !t.contains(']'),
                    "Hexagram {n}: stray square bracket present in {t:?}"
                );
            }
        }
    }

    #[test]
    fn corrected_ocr_lines_read_correctly() {
        // The exact lines that carried a dropped mid-clause period or a stray
        // navigation marker in the source OCR must now read as the published
        // Legge text. Each assertion targets the specific corrected fragment.

        // Hex 59 line 5: the "[paragraph continues]" marker is gone and the two
        // clauses join into one continuous Legge sentence.
        let h59l5 = line_text(59, 4).unwrap();
        assert!(h59l5.contains(
            "as the perspiration (flows from his body). He scatters abroad (also) the accumulations in the royal granaries."
        ));

        // Hex 17 line 3: "lets go the little boy" (no period after "go").
        assert!(
            line_text(17, 2)
                .unwrap()
                .contains("and lets go the little boy. Such following")
        );

        // Hex 25 line 3: "accused and apprehended" (no period after "and").
        assert!(
            line_text(25, 2)
                .unwrap()
                .contains("(of being accused and apprehended).")
        );

        // Hex 43 line 4: "like a sheep led" (no period after "like").
        assert!(
            line_text(43, 3)
                .unwrap()
                .contains("(If he could act) like a sheep led (after its companions)")
        );

        // Hex 62 line 4: "his natural course" (no period after "natural").
        assert!(
            line_text(62, 3)
                .unwrap()
                .contains("without exceeding (in his natural course).")
        );

        // Hex 62 line 6: "shows its subject" (no period after "shows").
        assert!(
            line_text(62, 5)
                .unwrap()
                .contains("The sixth SIX, divided, shows its subject not meeting")
        );

        // Hex 12 image: rebalanced parenthetical "(the manifestation of)".
        assert!(
            hexagram(12)
                .unwrap()
                .image
                .contains("restrains (the manifestation of) his virtue,")
        );

        // Hex 13 image: stray ")" after "this" removed.
        assert!(
            hexagram(13)
                .unwrap()
                .image
                .contains("in accordance with this, distinguishes things")
        );
    }

    #[test]
    fn parentheses_balanced_except_legges_one_intentional_imbalance() {
        // Round-trip every text and require balanced parentheses, with the sole
        // documented exception of Hexagram 4 line 2 — where Legge's own 1882
        // text opens "admitting (even the goodness of women," and never closes
        // the parenthesis. That imbalance is preserved verbatim, so it is the
        // one and only text allowed to be unbalanced.
        for n in 1..=64u8 {
            let h = hexagram(n).unwrap();
            let mut texts: Vec<(usize, &str)> =
                line_texts(n).unwrap().iter().copied().enumerate().collect();
            texts.push((100, h.judgment));
            texts.push((200, h.image));
            for (slot, t) in texts {
                let opens = t.matches('(').count();
                let closes = t.matches(')').count();
                let is_legge_intentional = n == 4 && slot == 1; // hex 4, line 2 (0-based slot 1)
                if is_legge_intentional {
                    assert_ne!(
                        opens, closes,
                        "Hex 4 line 2 should still carry Legge's unbalanced paren verbatim"
                    );
                } else {
                    assert_eq!(
                        opens, closes,
                        "Hexagram {n} slot {slot}: unbalanced parentheses in {t:?}"
                    );
                }
            }
        }
    }
}
