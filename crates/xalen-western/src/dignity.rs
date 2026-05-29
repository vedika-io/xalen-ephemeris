use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Essential dignity classification in Western astrology.
pub enum WesternDignity {
    Domicile,   // +5
    Exaltation, // +4
    Triplicity, // +3
    Term,       // +2
    Face,       // +1
    Detriment,  // -5
    Fall,       // -4
    Peregrine,  // -5
}

/// Compute the essential dignity score (+5 domicile to -5 fall).
pub fn essential_dignity_score(planet: &str, sign_index: usize, degree_in_sign: f64) -> i32 {
    let mut score = 0i32;

    if is_domicile(planet, sign_index) {
        score += 5;
    }
    if is_exaltation(planet, sign_index) {
        score += 4;
    }
    if is_triplicity_ruler(planet, sign_index) {
        score += 3;
    }
    if is_term_ruler(planet, sign_index, degree_in_sign) {
        score += 2;
    }
    if is_face_ruler(planet, sign_index, degree_in_sign) {
        score += 1;
    }
    if is_detriment(planet, sign_index) {
        score -= 5;
    }
    if is_fall(planet, sign_index) {
        score -= 4;
    }
    if score == 0 {
        score = -5;
    } // Peregrine

    score
}

/// Returns `true` if the planet rules the given sign.
pub fn is_domicile(planet: &str, sign: usize) -> bool {
    matches!(
        (planet, sign),
        ("Sun", 4)
            | ("Moon", 3)
            | ("Mercury", 2)
            | ("Mercury", 5)
            | ("Venus", 1)
            | ("Venus", 6)
            | ("Mars", 0)
            | ("Mars", 7)
            | ("Jupiter", 8)
            | ("Jupiter", 11)
            | ("Saturn", 9)
            | ("Saturn", 10)
    )
}

/// Returns `true` if the planet is exalted in the given sign.
///
/// Exaltation signs (with traditional exact degrees, for reference):
///   Sun in Aries (19°), Moon in Taurus (3°), Mercury in Virgo (15°),
///   Venus in Pisces (27°), Mars in Capricorn (28°), Jupiter in Cancer (15°),
///   Saturn in Libra (21°).
///
/// Verified: Wikipedia "Exaltation (astrology)"; al-Biruni; Ptolemy Tetrabiblos I.19;
///   Kelly Surtees "Exaltation Degrees"; renaissanceastrology.com/exaltationdegrees.
pub fn is_exaltation(planet: &str, sign: usize) -> bool {
    matches!(
        (planet, sign),
        ("Sun", 0)
            | ("Moon", 1)
            | ("Mercury", 5)
            | ("Venus", 11)
            | ("Mars", 9)
            | ("Jupiter", 3)
            | ("Saturn", 6)
    )
}

/// Returns `true` if the planet is in detriment in the given sign.
pub fn is_detriment(planet: &str, sign: usize) -> bool {
    matches!(
        (planet, sign),
        ("Sun", 10)
            | ("Moon", 9)
            | ("Mercury", 8)
            | ("Mercury", 11)
            | ("Venus", 0)
            | ("Venus", 7)
            | ("Mars", 1)
            | ("Mars", 6)
            | ("Jupiter", 2)
            | ("Jupiter", 5)
            | ("Saturn", 3)
            | ("Saturn", 4)
    )
}

/// Returns `true` if the planet is in fall in the given sign.
pub fn is_fall(planet: &str, sign: usize) -> bool {
    matches!(
        (planet, sign),
        ("Sun", 6)
            | ("Moon", 7)
            | ("Mercury", 11)
            | ("Venus", 5)
            | ("Mars", 3)
            | ("Jupiter", 9)
            | ("Saturn", 0)
    )
}

/// Returns `true` if the planet rules the triplicity of the given sign.
// Note: triplicity currently returns the day ruler only. Classical essential
// dignity includes day/night/participating rulers based on sect.
pub fn is_triplicity_ruler(planet: &str, sign: usize) -> bool {
    let element = sign % 4;
    match (planet, element) {
        ("Sun", 0) | ("Jupiter", 0) => true,    // Fire
        ("Venus", 1) | ("Moon", 1) => true,     // Earth
        ("Saturn", 2) | ("Mercury", 2) => true, // Air
        ("Mars", 3) => true,                    // Water
        _ => false,
    }
}

/// Returns `true` if the planet rules the Egyptian term at the given degree.
///
/// Verified against: Ptolemy Tetrabiblos I.20-21 (Robbins trans.); Vettius Valens
/// Anthologies (Egyptian terms table); twowander.com/blog/astrological-bounds;
/// kiraryberg.com/blog/the-bounds.  Aries confirmed: Jup 6, Ven 6, Mer 8, Mar 5,
/// Sat 5.  Aquarius confirmed: Mer 7, Ven 6, Jup 7, Mar 5, Sat 5.  Virgo Venus 10
/// confirmed via Tetrabiblos.  Total degrees across zodiac: Sat 57, Jup 79, Mar 66,
/// Ven 82, Mer 76 = 360.
pub fn is_term_ruler(planet: &str, sign: usize, deg: f64) -> bool {
    let d = deg as u32;
    // Egyptian terms per Tetrabiblos I.20-21, Robbins translation.
    // Each range is inclusive: 0..=5 means degrees 0,1,2,3,4,5 (6 degrees).
    let ruler = match sign {
        // Aries: Jupiter 0-5(6°), Venus 6-11(6°), Mercury 12-19(8°), Mars 20-24(5°), Saturn 25-29(5°)
        0 => match d {
            0..=5 => "Jupiter",
            6..=11 => "Venus",
            12..=19 => "Mercury",
            20..=24 => "Mars",
            _ => "Saturn",
        },
        // Taurus: Venus 0-7(8°), Mercury 8-13(6°), Jupiter 14-21(8°), Saturn 22-26(5°), Mars 27-29(3°)
        1 => match d {
            0..=7 => "Venus",
            8..=13 => "Mercury",
            14..=21 => "Jupiter",
            22..=26 => "Saturn",
            _ => "Mars",
        },
        // Gemini: Mercury 0-5(6°), Jupiter 6-11(6°), Venus 12-16(5°), Mars 17-23(7°), Saturn 24-29(6°)
        2 => match d {
            0..=5 => "Mercury",
            6..=11 => "Jupiter",
            12..=16 => "Venus",
            17..=23 => "Mars",
            _ => "Saturn",
        },
        // Cancer: Mars 0-6(7°), Venus 7-12(6°), Mercury 13-18(6°), Jupiter 19-25(7°), Saturn 26-29(4°)
        3 => match d {
            0..=6 => "Mars",
            7..=12 => "Venus",
            13..=18 => "Mercury",
            19..=25 => "Jupiter",
            _ => "Saturn",
        },
        // Leo: Jupiter 0-5(6°), Venus 6-10(5°), Saturn 11-17(7°), Mercury 18-23(6°), Mars 24-29(6°)
        4 => match d {
            0..=5 => "Jupiter",
            6..=10 => "Venus",
            11..=17 => "Saturn",
            18..=23 => "Mercury",
            _ => "Mars",
        },
        // Virgo: Mercury 0-6(7°), Venus 7-16(10°), Jupiter 17-20(4°), Mars 21-27(7°), Saturn 28-29(2°)
        5 => match d {
            0..=6 => "Mercury",
            7..=16 => "Venus",
            17..=20 => "Jupiter",
            21..=27 => "Mars",
            _ => "Saturn",
        },
        // Libra: Saturn 0-5(6°), Mercury 6-13(8°), Jupiter 14-20(7°), Venus 21-27(7°), Mars 28-29(2°)
        6 => match d {
            0..=5 => "Saturn",
            6..=13 => "Mercury",
            14..=20 => "Jupiter",
            21..=27 => "Venus",
            _ => "Mars",
        },
        // Scorpio: Mars 0-6(7°), Venus 7-10(4°), Mercury 11-18(8°), Jupiter 19-23(5°), Saturn 24-29(6°)
        7 => match d {
            0..=6 => "Mars",
            7..=10 => "Venus",
            11..=18 => "Mercury",
            19..=23 => "Jupiter",
            _ => "Saturn",
        },
        // Sagittarius: Jupiter 0-11(12°), Venus 12-16(5°), Mercury 17-20(4°), Saturn 21-25(5°), Mars 26-29(4°)
        8 => match d {
            0..=11 => "Jupiter",
            12..=16 => "Venus",
            17..=20 => "Mercury",
            21..=25 => "Saturn",
            _ => "Mars",
        },
        // Capricorn: Mercury 0-6(7°), Jupiter 7-13(7°), Venus 14-21(8°), Saturn 22-25(4°), Mars 26-29(4°)
        9 => match d {
            0..=6 => "Mercury",
            7..=13 => "Jupiter",
            14..=21 => "Venus",
            22..=25 => "Saturn",
            _ => "Mars",
        },
        // Aquarius: Mercury 0-6(7°), Venus 7-12(6°), Jupiter 13-19(7°), Mars 20-24(5°), Saturn 25-29(5°)
        10 => match d {
            0..=6 => "Mercury",
            7..=12 => "Venus",
            13..=19 => "Jupiter",
            20..=24 => "Mars",
            _ => "Saturn",
        },
        // Pisces: Venus 0-11(12°), Jupiter 12-15(4°), Mercury 16-18(3°), Mars 19-27(9°), Saturn 28-29(2°)
        11 => match d {
            0..=11 => "Venus",
            12..=15 => "Jupiter",
            16..=18 => "Mercury",
            19..=27 => "Mars",
            _ => "Saturn",
        },
        _ => return false,
    };
    ruler == planet
}

/// Returns `true` if the planet rules the decan/face at the given degree.
pub fn is_face_ruler(planet: &str, sign: usize, deg: f64) -> bool {
    // Chaldean decan faces
    const FACE_SEQUENCE: [&str; 7] = [
        "Mars", "Sun", "Venus", "Mercury", "Moon", "Saturn", "Jupiter",
    ];
    let absolute_decan = sign * 3 + (deg as usize / 10);
    let ruler = FACE_SEQUENCE[absolute_decan % 7];
    ruler == planet
}

/// Check if two planets are in mutual reception (each in the other's sign).
pub fn mutual_reception(planet_a: &str, sign_a: usize, planet_b: &str, sign_b: usize) -> bool {
    is_domicile(planet_a, sign_b) && is_domicile(planet_b, sign_a)
}

/// Compute accidental dignity score based on house placement and motion.
pub fn accidental_dignity_score(
    angular: bool,
    succedent: bool,
    direct: bool,
    swift: bool,
    oriental_if_superior: bool,
    free_combustion: bool,
    in_hayz: bool,
) -> i32 {
    let mut score = 0i32;
    if angular {
        score += 5;
    } else if succedent {
        score += 3;
    } else {
        score += 1;
    }
    if direct {
        score += 4;
    }
    if swift {
        score += 2;
    }
    if oriental_if_superior {
        score += 2;
    }
    if free_combustion {
        score += 5;
    }
    if in_hayz {
        score += 1;
    }
    score
}

/// Combine essential and accidental dignity into a total score.
pub fn total_dignity(essential: i32, accidental: i32) -> i32 {
    essential + accidental
}

/// Find the almuten (strongest ruler) of a sign degree by dignity weighting.
pub fn almuten(sign: usize, degree: f64) -> Vec<(&'static str, i32)> {
    let planets = [
        "Sun", "Moon", "Mercury", "Venus", "Mars", "Jupiter", "Saturn",
    ];
    let mut scores: Vec<(&str, i32)> = planets
        .iter()
        .map(|&p| (p, essential_dignity_score(p, sign, degree)))
        .collect();
    scores.sort_by_key(|x| std::cmp::Reverse(x.1));
    scores
}

/// Compute the almuten figuris (strongest essential dignitary) for a degree.
///
/// Returns a ranked list of every planet that holds ANY essential dignity at
/// the given sign+degree, sorted strongest first.  This is the "almuten of a
/// degree" used in medieval and Renaissance Western astrology (Bonatti, Lilly).
///
/// Scoring follows the standard Ptolemaic weighting:
///   Domicile +5, Exaltation +4, Triplicity +3, Term +2, Face +1.
///
/// Only planets with a positive score are included (detriment/fall/peregrine
/// are excluded because they hold no dignity).
pub fn almuten_figuris(sign: usize, degree: f64) -> Vec<(&'static str, i32)> {
    let planets = [
        "Sun", "Moon", "Mercury", "Venus", "Mars", "Jupiter", "Saturn",
    ];
    let mut scores: Vec<(&str, i32)> = Vec::new();

    for &planet in &planets {
        let mut score = 0i32;
        if is_domicile(planet, sign) {
            score += 5;
        }
        if is_exaltation(planet, sign) {
            score += 4;
        }
        if is_triplicity_ruler(planet, sign) {
            score += 3;
        }
        if is_term_ruler(planet, sign, degree) {
            score += 2;
        }
        if is_face_ruler(planet, sign, degree) {
            score += 1;
        }
        if score > 0 {
            scores.push((planet, score));
        }
    }
    scores.sort_by_key(|x| std::cmp::Reverse(x.1));
    scores
}

/// Return the single strongest dignitary (the almuten) of a degree,
/// or `None` if no planet holds any dignity there (should not happen in
/// practice -- every degree has at least a term ruler and a face ruler).
pub fn almuten_of_degree(sign: usize, degree: f64) -> Option<(&'static str, i32)> {
    almuten_figuris(sign, degree).into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sun_domicile_in_leo() {
        assert!(is_domicile("Sun", 4));
        let score = essential_dignity_score("Sun", 4, 15.0);
        assert!(
            score > 0,
            "Sun in Leo should have positive dignity, got {score}"
        );
    }

    #[test]
    fn sun_detriment_in_aquarius() {
        assert!(is_detriment("Sun", 10));
    }

    #[test]
    fn sun_exalted_in_aries() {
        assert!(is_exaltation("Sun", 0));
        let score = essential_dignity_score("Sun", 0, 10.0);
        assert!(
            score >= 4,
            "Sun exalted in Aries at 10° should score >= 4, got {score}"
        );
    }

    #[test]
    fn egyptian_terms_all_signs() {
        for sign in 0..12 {
            let mut covered = vec![false; 30];
            for deg in 0..30 {
                let found = [
                    "Sun", "Moon", "Mercury", "Venus", "Mars", "Jupiter", "Saturn",
                ]
                .iter()
                .any(|&p| is_term_ruler(p, sign, deg as f64));
                covered[deg] = found;
            }
            let count = covered.iter().filter(|&&x| x).count();
            assert_eq!(
                count, 30,
                "Sign {sign} should have term rulers for all 30 degrees, got {count}"
            );
        }
    }

    #[test]
    fn mutual_reception_venus_mars() {
        assert!(mutual_reception("Venus", 0, "Mars", 1));
        assert!(mutual_reception("Mars", 1, "Venus", 0));
        assert!(!mutual_reception("Sun", 0, "Mars", 1));
    }

    #[test]
    fn accidental_dignity_angular() {
        let score = accidental_dignity_score(true, false, true, true, false, true, false);
        assert!(
            score > 10,
            "Angular+direct+swift+free should be >10, got {score}"
        );
    }

    #[test]
    fn almuten_returns_sorted() {
        let a = almuten(0, 3.0);
        assert_eq!(a.len(), 7);
        for i in 1..a.len() {
            assert!(a[i - 1].1 >= a[i].1, "Almuten should be sorted descending");
        }
    }

    #[test]
    fn peregrine_negative() {
        let score = essential_dignity_score("Moon", 0, 15.0);
        assert!(
            score < 0,
            "Moon in Aries at 15° should be peregrine, got {score}"
        );
    }

    #[test]
    fn almuten_figuris_positive_only() {
        let af = almuten_figuris(0, 3.0); // Aries 3°
        for &(_, score) in &af {
            assert!(score > 0, "almuten_figuris should only include positive scores");
        }
        assert!(!af.is_empty(), "Should have at least one dignitary");
    }

    #[test]
    fn almuten_figuris_sorted_descending() {
        let af = almuten_figuris(4, 10.0); // Leo 10°
        for i in 1..af.len() {
            assert!(
                af[i - 1].1 >= af[i].1,
                "almuten_figuris should be sorted descending"
            );
        }
    }

    #[test]
    fn almuten_of_degree_returns_strongest() {
        // Sun in Leo (sign 4) at 15°: domicile (+5) + face ruler at 10-19° (+1 if Sun)
        let top = almuten_of_degree(4, 15.0);
        assert!(top.is_some());
        let (planet, score) = top.unwrap();
        assert!(score >= 5, "Leo almuten should score >= 5, got {score}");
        assert_eq!(planet, "Sun", "Sun should be almuten of Leo 15°");
    }

    #[test]
    fn almuten_figuris_every_degree_has_result() {
        for sign in 0..12 {
            for deg in 0..30 {
                let af = almuten_figuris(sign, deg as f64);
                assert!(
                    !af.is_empty(),
                    "Sign {sign} degree {deg} should have at least one dignitary"
                );
            }
        }
    }
}
