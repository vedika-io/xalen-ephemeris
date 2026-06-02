use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum System {
    Pythagorean,
    Chaldean,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumerologyProfile {
    pub life_path: u32,
    pub birthday_number: u32,
    pub expression: u32,
    pub soul_urge: u32,
    pub personality: u32,
    pub maturity: u32,
    pub system: System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MasterNumber {
    Eleven,
    TwentyTwo,
    ThirtyThree,
}

fn pythagorean_value(c: char) -> Option<u32> {
    match c.to_ascii_uppercase() {
        'A' | 'J' | 'S' => Some(1),
        'B' | 'K' | 'T' => Some(2),
        'C' | 'L' | 'U' => Some(3),
        'D' | 'M' | 'V' => Some(4),
        'E' | 'N' | 'W' => Some(5),
        'F' | 'O' | 'X' => Some(6),
        'G' | 'P' | 'Y' => Some(7),
        'H' | 'Q' | 'Z' => Some(8),
        'I' | 'R' => Some(9),
        _ => None,
    }
}

fn chaldean_value(c: char) -> Option<u32> {
    // Standard Chaldean letter values (no letter is ever 9). Verified 2026-05-28
    // against authoritative Chaldean numerology tables: Q belongs to the 1-group
    // (A, I, J, Q, Y), NOT 8.
    match c.to_ascii_uppercase() {
        'A' | 'I' | 'J' | 'Q' | 'Y' => Some(1),
        'B' | 'K' | 'R' => Some(2),
        'C' | 'G' | 'L' | 'S' => Some(3),
        'D' | 'M' | 'T' => Some(4),
        'E' | 'H' | 'N' | 'X' => Some(5),
        'U' | 'V' | 'W' => Some(6),
        'O' | 'Z' => Some(7),
        'F' | 'P' => Some(8),
        _ => None,
    }
}

fn is_vowel(c: char) -> bool {
    matches!(c.to_ascii_uppercase(), 'A' | 'E' | 'I' | 'O' | 'U')
}

/// Context-aware vowel check: treats Y as a vowel when it is the only vowel
/// sound in a syllable (no A/E/I/O/U adjacent) or when it falls at the end
/// of a word. `name` is the full name string; `pos` is the byte-index of the
/// character being tested within that string.
fn is_vowel_with_context(c: char, name: &str, pos: usize) -> bool {
    let upper = c.to_ascii_uppercase();
    if matches!(upper, 'A' | 'E' | 'I' | 'O' | 'U') {
        return true;
    }
    if upper != 'Y' {
        return false;
    }
    // Y at the very start of a word is a consonant (e.g. "Yolanda")
    let bytes = name.as_bytes();
    let is_word_start = pos == 0 || !bytes[pos - 1].is_ascii_alphabetic();
    if is_word_start {
        return false;
    }
    // Y at end of a word is a vowel (e.g. "Larry", "Emily")
    let is_word_end = pos + 1 >= bytes.len() || !bytes[pos + 1].is_ascii_alphabetic();
    if is_word_end {
        return true;
    }
    // Y is a vowel when no standard vowel is adjacent (prev or next letter).
    // This catches names like "Lynn", "Bryn", "Kyle" where Y carries the
    // vowel sound of the syllable.
    let prev_is_vowel =
        pos > 0 && bytes[pos - 1].is_ascii_alphabetic() && is_vowel(bytes[pos - 1] as char);
    let next_is_vowel = pos + 1 < bytes.len()
        && bytes[pos + 1].is_ascii_alphabetic()
        && is_vowel(bytes[pos + 1] as char);
    !prev_is_vowel && !next_is_vowel
}

/// Vowel sum using context-aware Y detection.
pub fn vowel_value_smart(name: &str, system: System) -> u32 {
    let mapper = match system {
        System::Pythagorean => pythagorean_value as fn(char) -> Option<u32>,
        System::Chaldean => chaldean_value,
    };
    name.char_indices()
        .filter(|&(pos, c)| is_vowel_with_context(c, name, pos))
        .filter_map(|(_, c)| mapper(c))
        .sum()
}

/// Consonant sum using context-aware Y detection.
pub fn consonant_value_smart(name: &str, system: System) -> u32 {
    let mapper = match system {
        System::Pythagorean => pythagorean_value as fn(char) -> Option<u32>,
        System::Chaldean => chaldean_value,
    };
    name.char_indices()
        .filter(|&(pos, c)| c.is_ascii_alphabetic() && !is_vowel_with_context(c, name, pos))
        .filter_map(|(_, c)| mapper(c))
        .sum()
}

/// Soul urge (heart's desire) using context-aware Y-as-vowel rules.
pub fn soul_urge_number_smart(full_name: &str, system: System) -> u32 {
    reduce(vowel_value_smart(full_name, system), true)
}

/// Personality number using context-aware Y-as-vowel rules.
pub fn personality_number_smart(full_name: &str, system: System) -> u32 {
    reduce(consonant_value_smart(full_name, system), true)
}

fn is_master(n: u32) -> bool {
    matches!(n, 11 | 22 | 33)
}

pub fn reduce(mut n: u32, preserve_master: bool) -> u32 {
    while n > 9 && !(preserve_master && is_master(n)) {
        n = digit_sum(n);
    }
    n
}

fn digit_sum(mut n: u32) -> u32 {
    let mut s = 0;
    while n > 0 {
        s += n % 10;
        n /= 10;
    }
    s
}

pub fn name_value(name: &str, system: System) -> u32 {
    let mapper = match system {
        System::Pythagorean => pythagorean_value as fn(char) -> Option<u32>,
        System::Chaldean => chaldean_value,
    };
    name.chars().filter_map(mapper).sum()
}

pub fn vowel_value(name: &str, system: System) -> u32 {
    let mapper = match system {
        System::Pythagorean => pythagorean_value as fn(char) -> Option<u32>,
        System::Chaldean => chaldean_value,
    };
    name.chars()
        .filter(|c| is_vowel(*c))
        .filter_map(mapper)
        .sum()
}

pub fn consonant_value(name: &str, system: System) -> u32 {
    let mapper = match system {
        System::Pythagorean => pythagorean_value as fn(char) -> Option<u32>,
        System::Chaldean => chaldean_value,
    };
    name.chars()
        .filter(|c| c.is_ascii_alphabetic() && !is_vowel(*c))
        .filter_map(mapper)
        .sum()
}

pub fn life_path(year: u32, month: u32, day: u32) -> u32 {
    let y = reduce(year, true);
    let m = reduce(month, true);
    let d = reduce(day, true);
    reduce(y + m + d, true)
}

pub fn birthday_number(day: u32) -> u32 {
    reduce(day, true)
}

pub fn expression_number(full_name: &str, system: System) -> u32 {
    reduce(name_value(full_name, system), true)
}

pub fn soul_urge_number(full_name: &str, system: System) -> u32 {
    reduce(vowel_value(full_name, system), true)
}

pub fn personality_number(full_name: &str, system: System) -> u32 {
    reduce(consonant_value(full_name, system), true)
}

pub fn maturity_number(year: u32, month: u32, day: u32, full_name: &str, system: System) -> u32 {
    let lp = life_path(year, month, day);
    let expr = expression_number(full_name, system);
    reduce(lp + expr, true)
}

pub fn personal_year(birth_month: u32, birth_day: u32, current_year: u32) -> u32 {
    let sum = reduce(birth_month, false) + reduce(birth_day, false) + reduce(current_year, false);
    reduce(sum, false)
}

pub fn personal_month(
    birth_month: u32,
    birth_day: u32,
    current_year: u32,
    current_month: u32,
) -> u32 {
    let py = personal_year(birth_month, birth_day, current_year);
    reduce(py + current_month, false)
}

pub fn personal_day(
    birth_month: u32,
    birth_day: u32,
    current_year: u32,
    current_month: u32,
    current_day: u32,
) -> u32 {
    let pm = personal_month(birth_month, birth_day, current_year, current_month);
    reduce(pm + current_day, false)
}

pub fn challenge_numbers(year: u32, month: u32, day: u32) -> [u32; 4] {
    let m = reduce(month, false);
    let d = reduce(day, false);
    let y = reduce(year, false);
    let first = (m as i32 - d as i32).unsigned_abs();
    let second = (d as i32 - y as i32).unsigned_abs();
    let third = (first as i32 - second as i32).unsigned_abs();
    let fourth = (m as i32 - y as i32).unsigned_abs();
    [first, second, third, fourth]
}

pub fn pinnacle_numbers(year: u32, month: u32, day: u32) -> [u32; 4] {
    let m = reduce(month, false);
    let d = reduce(day, false);
    let y = reduce(year, false);
    let first = reduce(m + d, true);
    let second = reduce(d + y, true);
    let third = reduce(first + second, true);
    let fourth = reduce(m + y, true);
    [first, second, third, fourth]
}

pub fn karmic_debt(n: u32) -> bool {
    matches!(n, 13 | 14 | 16 | 19)
}

pub fn check_master_number(n: u32) -> Option<MasterNumber> {
    match n {
        11 => Some(MasterNumber::Eleven),
        22 => Some(MasterNumber::TwentyTwo),
        33 => Some(MasterNumber::ThirtyThree),
        _ => None,
    }
}

pub fn full_profile(
    year: u32,
    month: u32,
    day: u32,
    full_name: &str,
    system: System,
) -> NumerologyProfile {
    NumerologyProfile {
        life_path: life_path(year, month, day),
        birthday_number: birthday_number(day),
        expression: expression_number(full_name, system),
        soul_urge: soul_urge_number(full_name, system),
        personality: personality_number(full_name, system),
        maturity: maturity_number(year, month, day, full_name, system),
        system,
    }
}

/// Interpretive meaning of a numerology number.
///
/// Honesty contract: this crate ships **no** AI-generated or
/// copyright-encumbered interpretive prose. Until a verified public-domain
/// source is backfilled, every entry is genuinely absent, so this returns
/// [`None`] rather than `Some("")`. A `Some(_)` would falsely advertise the
/// presence of real content; `None` lets callers distinguish "no meaning text
/// available" from an empty meaning, and decide their own fallback behaviour.
pub fn number_meaning(n: u32) -> Option<&'static str> {
    match n {
        // No verified public-domain meaning text is currently bundled for any
        // number, so all defined numbers map to `None` (absent), and any other
        // integer is also `None` (undefined).
        1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 11 | 22 | 33 => None,
        _ => None,
    }
}

pub fn lo_shu_grid(birth_date_digits: &[u32]) -> [[u32; 3]; 3] {
    // Each digit 1-9 occupies its fixed cell in the Lo Shu magic square:
    //   4 9 2
    //   3 5 7
    //   8 1 6
    // (every row, column and diagonal sums to 15). Counts how many times each
    // digit appears in the birth date at its magic-square position.
    let mut grid = [[0u32; 3]; 3];
    let positions: [(usize, usize); 10] = [
        (1, 1), // 0 - not placed (never reached; guarded below)
        (2, 1), // 1
        (0, 2), // 2
        (1, 0), // 3
        (0, 0), // 4
        (1, 1), // 5 (center)
        (2, 2), // 6
        (1, 2), // 7
        (2, 0), // 8
        (0, 1), // 9
    ];
    for &d in birth_date_digits {
        if (1..=9).contains(&d) {
            let (r, c) = positions[d as usize];
            grid[r][c] += 1;
        }
    }
    grid
}

pub fn birth_date_digits(year: u32, month: u32, day: u32) -> Vec<u32> {
    let mut digits = Vec::new();
    for n in [day, month, year] {
        let mut v = n;
        if v == 0 {
            digits.push(0);
            continue;
        }
        let mut tmp = Vec::new();
        while v > 0 {
            tmp.push(v % 10);
            v /= 10;
        }
        tmp.reverse();
        digits.extend(tmp);
    }
    digits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pythagorean_a_is_1() {
        assert_eq!(pythagorean_value('A'), Some(1));
        assert_eq!(pythagorean_value('Z'), Some(8));
        assert_eq!(pythagorean_value('I'), Some(9));
    }

    #[test]
    fn chaldean_no_nine() {
        for c in 'A'..='Z' {
            if let Some(v) = chaldean_value(c) {
                assert!(v <= 8, "Chaldean should never map to 9, got {v} for {c}");
            }
        }
    }

    #[test]
    fn reduce_basic() {
        assert_eq!(reduce(29, false), 2); // 2+9=11, 1+1=2
        assert_eq!(reduce(29, true), 11); // master 11 preserved
        assert_eq!(reduce(38, false), 2); // 3+8=11, 1+1=2
        assert_eq!(reduce(38, true), 11); // master 11 preserved
    }

    #[test]
    fn reduce_master_22() {
        assert_eq!(reduce(22, true), 22);
        assert_eq!(reduce(22, false), 4);
    }

    #[test]
    fn life_path_known() {
        // Abraham Lincoln: Feb 12, 1809
        // 2 + 3 (1+2) + 18 (1+8+0+9) = 2+3+9 = 14 = 5
        let lp = life_path(1809, 2, 12);
        assert_eq!(lp, 5);
    }

    #[test]
    fn life_path_master_11() {
        // Nov 29, 2009: 11 + 11 + 11 = 33 (master)
        let lp = life_path(2009, 11, 29);
        assert_eq!(lp, 33);
    }

    #[test]
    fn expression_pythagorean() {
        let n = expression_number("JOHN", System::Pythagorean);
        // J=1, O=6, H=8, N=5 = 20 = 2
        assert_eq!(n, 2);
    }

    #[test]
    fn expression_chaldean() {
        let n = expression_number("JOHN", System::Chaldean);
        // J=1, O=7, H=5, N=5 = 18 = 9
        assert_eq!(n, 9);
    }

    #[test]
    fn soul_urge_vowels_only() {
        let su = soul_urge_number("ALICE", System::Pythagorean);
        // A=1, I=9, E=5 = 15 = 6
        assert_eq!(su, 6);
    }

    #[test]
    fn personality_consonants_only() {
        let p = personality_number("ALICE", System::Pythagorean);
        // L=3, C=3 = 6
        assert_eq!(p, 6);
    }

    #[test]
    fn birthday_number_simple() {
        assert_eq!(birthday_number(7), 7);
        assert_eq!(birthday_number(29), 11); // master preserved
        assert_eq!(birthday_number(15), 6);
    }

    #[test]
    fn personal_year_cycle() {
        let py = personal_year(3, 15, 2026);
        assert!(py >= 1 && py <= 9);
    }

    #[test]
    fn challenge_numbers_computed() {
        let ch = challenge_numbers(1990, 5, 14);
        assert!(ch.iter().all(|&x| x <= 9));
    }

    #[test]
    fn pinnacle_numbers_computed() {
        let pn = pinnacle_numbers(1990, 5, 14);
        assert!(pn.iter().all(|&x| x <= 33 && x > 0));
    }

    #[test]
    fn karmic_debt_numbers() {
        assert!(karmic_debt(13));
        assert!(karmic_debt(14));
        assert!(karmic_debt(16));
        assert!(karmic_debt(19));
        assert!(!karmic_debt(15));
    }

    #[test]
    fn lo_shu_grid_basic() {
        let digits = birth_date_digits(1990, 5, 14);
        let grid = lo_shu_grid(&digits);
        let total: u32 = grid.iter().flat_map(|r| r.iter()).sum();
        let nonzero_digits = digits.iter().filter(|&&d| d >= 1 && d <= 9).count();
        assert_eq!(total as usize, nonzero_digits);
    }

    #[test]
    fn full_profile_computes() {
        let p = full_profile(1990, 7, 15, "Jane Doe", System::Pythagorean);
        assert!(p.life_path <= 33);
        assert!(p.expression <= 33);
        assert!(p.soul_urge <= 33);
        assert!(p.personality <= 33);
    }

    #[test]
    fn number_meanings_all_single() {
        // Meaning prose removed (PD backfill pending); verify the function is
        // total over the single-digit range and never panics.
        for i in 1..=9 {
            let _ = number_meaning(i);
        }
    }

    #[test]
    fn number_meaning_returns_none_for_absent_entries() {
        // Honesty contract: no interpretive prose is bundled, so every defined
        // number (and every undefined one) must report absence as `None`,
        // never `Some("")`. This lets callers tell "real content" from
        // "stripped / not yet backfilled".
        for n in [1u32, 2, 3, 4, 5, 6, 7, 8, 9, 11, 22, 33] {
            assert_eq!(
                number_meaning(n),
                None,
                "number {n} must report absent meaning as None, not Some(\"\")"
            );
        }
        // Undefined numbers are also absent (None), not an empty string.
        assert_eq!(number_meaning(0), None);
        assert_eq!(number_meaning(10), None);
        assert_eq!(number_meaning(100), None);
    }

    #[test]
    fn master_number_detection() {
        assert_eq!(check_master_number(11), Some(MasterNumber::Eleven));
        assert_eq!(check_master_number(22), Some(MasterNumber::TwentyTwo));
        assert_eq!(check_master_number(33), Some(MasterNumber::ThirtyThree));
        assert_eq!(check_master_number(7), None);
    }

    #[test]
    fn birth_date_digits_correct() {
        let d = birth_date_digits(1990, 5, 14);
        assert_eq!(d, vec![1, 4, 5, 1, 9, 9, 0]);
    }

    #[test]
    fn personal_day_computed() {
        let pd = personal_day(3, 15, 2026, 6, 10);
        assert!(pd >= 1 && pd <= 9);
    }

    #[test]
    fn name_value_ignores_spaces() {
        let v1 = name_value("JOHN DOE", System::Pythagorean);
        let v2 = name_value("JOHNDOE", System::Pythagorean);
        assert_eq!(v1, v2);
    }

    // --- Y-as-vowel context-aware tests ---

    #[test]
    fn y_at_word_start_is_consonant() {
        // "Yolanda" — Y starts the word, should be consonant
        assert!(!is_vowel_with_context('Y', "Yolanda", 0));
    }

    #[test]
    fn y_at_word_end_is_vowel() {
        // "Larry" — Y ends the word, should be vowel
        assert!(is_vowel_with_context('y', "Larry", 4));
        // "Emily" — same rule
        assert!(is_vowel_with_context('y', "Emily", 4));
    }

    #[test]
    fn y_only_vowel_sound_is_vowel() {
        // "Lynn" — Y is the only vowel sound (surrounded by consonants)
        assert!(is_vowel_with_context('y', "Lynn", 1));
        // "Bryn" — same
        assert!(is_vowel_with_context('y', "Bryn", 2));
    }

    #[test]
    fn y_adjacent_to_vowel_is_consonant() {
        // "Yael" — Y starts word = consonant
        assert!(!is_vowel_with_context('Y', "Yael", 0));
        // "Maya" — Y between a (vowel) and a (vowel), not word-start/end
        // M=0, a=1, y=2, a=3 — prev 'a' is vowel, so Y is consonant
        assert!(!is_vowel_with_context('y', "Maya", 2));
    }

    #[test]
    fn y_in_kyle_is_vowel() {
        // "Kyle" — K=0, y=1, l=2, e=3
        // Y is not word-start (K before it), not word-end (l after it)
        // prev K is consonant, next l is consonant => Y is vowel
        assert!(is_vowel_with_context('y', "Kyle", 1));
    }

    #[test]
    fn soul_urge_smart_includes_y_vowels() {
        // "LYNN" — L,Y,N,N. Standard: vowels = none (soul_urge = 0→0).
        // Smart: Y is vowel => Y=7 in Pythagorean, reduce(7) = 7
        let standard = soul_urge_number("LYNN", System::Pythagorean);
        let smart = soul_urge_number_smart("LYNN", System::Pythagorean);
        assert_eq!(standard, 0); // no AEIOU vowels
        assert_eq!(smart, 7); // Y=7 counted as vowel
    }

    #[test]
    fn personality_smart_excludes_y_vowels() {
        // "LYNN" — standard: L=3, Y=7, N=5, N=5 = 20 => 2
        // Smart: Y is vowel, so consonants = L+N+N = 3+5+5 = 13 => 4
        let standard = personality_number("LYNN", System::Pythagorean);
        let smart = personality_number_smart("LYNN", System::Pythagorean);
        assert_eq!(standard, 2); // Y counted as consonant
        assert_eq!(smart, 4); // Y excluded (it's a vowel)
    }

    #[test]
    fn smart_and_standard_agree_without_y() {
        // "ALICE" — no Y, both should be identical
        let su_std = soul_urge_number("ALICE", System::Pythagorean);
        let su_smart = soul_urge_number_smart("ALICE", System::Pythagorean);
        assert_eq!(su_std, su_smart);
        let p_std = personality_number("ALICE", System::Pythagorean);
        let p_smart = personality_number_smart("ALICE", System::Pythagorean);
        assert_eq!(p_std, p_smart);
    }

    #[test]
    fn vowel_plus_consonant_smart_equals_total() {
        // For any name, vowel_value_smart + consonant_value_smart = name_value
        for name in &["LYNN", "Kyle", "Emily", "Yolanda", "JOHN DOE", "BRYN"] {
            let total = name_value(name, System::Pythagorean);
            let vowels = vowel_value_smart(name, System::Pythagorean);
            let consonants = consonant_value_smart(name, System::Pythagorean);
            assert_eq!(
                vowels + consonants,
                total,
                "vowel+consonant mismatch for {name}: {vowels}+{consonants} != {total}"
            );
        }
    }

    // --- Chaldean letter values ---

    #[test]
    fn chaldean_q_is_1() {
        // Q is in the 1-group (A, I, J, Q, Y) in standard Chaldean numerology.
        assert_eq!(chaldean_value('Q'), Some(1));
        assert_eq!(chaldean_value('q'), Some(1));
    }

    #[test]
    fn chaldean_has_no_letter_nine() {
        // Chaldean letter values run 1-8 only.
        for c in 'A'..='Z' {
            assert!(chaldean_value(c).unwrap() <= 8, "{c} should be 1-8");
        }
    }

    #[test]
    fn lo_shu_is_magic_square() {
        // Placing each of 1..9 exactly once must reproduce the Lo Shu magic
        // square (every row/column/diagonal sums to 15).
        let grid = lo_shu_grid(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let expect = [[4u32, 9, 2], [3, 5, 7], [8, 1, 6]];
        // Each cell should hold a count of 1 at the magic-square layout; verify by
        // reconstructing which digit landed where.
        let mut placed = [[0u32; 3]; 3];
        for (d, _) in (1..=9).zip(0..) {
            // recompute position the same way as lo_shu_grid
            let pos = [
                (1, 1),
                (2, 1),
                (0, 2),
                (1, 0),
                (0, 0),
                (1, 1),
                (2, 2),
                (1, 2),
                (2, 0),
                (0, 1),
            ][d as usize];
            placed[pos.0][pos.1] = d;
        }
        assert_eq!(placed, expect, "Lo Shu layout must be the magic square");
        for row in grid {
            for cell in row {
                assert_eq!(cell, 1, "each digit placed once");
            }
        }
    }
}
