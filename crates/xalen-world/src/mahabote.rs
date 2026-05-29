//! Burmese Mahabote (မဟာဘုတ်) astrology system.
//!
//! Based on the day of the week of birth. Each of the 7 days is ruled by a
//! planet, associated with an animal, direction, and element. Wednesday is
//! uniquely split: AM (Mercury, Tusked Elephant) and PM (Rahu, Tuskless
//! Elephant), giving 8 "day signs" in practice.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// MahaboteDay
// ---------------------------------------------------------------------------

/// The 7 days of the week in Burmese astrology.
///
/// Wednesday is conceptually split (AM = Mercury, PM = Rahu) but represented
/// as a single variant here. Use [`MahaboteProfile::is_wednesday_pm`] for the
/// AM/PM distinction when the birth hour is known.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MahaboteDay {
    Sunday,    // တနင်္ဂနွေ Taninganwe — Sun
    Monday,    // တနင်္လာ Taninla — Moon
    Tuesday,   // အင်္ဂါ Inga — Mars
    Wednesday, // ဗုဒ္ဓဟူး Buddahu — Mercury (AM) / Rahu (PM)
    Thursday,  // ကြာသပတေး Kyathabade — Jupiter
    Friday,    // သောကြာ Thaukkya — Venus
    Saturday,  // စနေ Sanay — Saturn
}

impl MahaboteDay {
    pub const ALL: [MahaboteDay; 7] = [
        MahaboteDay::Sunday,
        MahaboteDay::Monday,
        MahaboteDay::Tuesday,
        MahaboteDay::Wednesday,
        MahaboteDay::Thursday,
        MahaboteDay::Friday,
        MahaboteDay::Saturday,
    ];

    /// Construct from a weekday index: 0 = Sunday, 1 = Monday, ..., 6 = Saturday.
    pub fn from_weekday(w: usize) -> Self {
        Self::ALL[w % 7]
    }

    /// Burmese name in Myanmar script.
    pub fn burmese(&self) -> &'static str {
        match self {
            MahaboteDay::Sunday => "တနင်္ဂနွေ",
            MahaboteDay::Monday => "တနင်္လာ",
            MahaboteDay::Tuesday => "အင်္ဂါ",
            MahaboteDay::Wednesday => "ဗုဒ္ဓဟူး",
            MahaboteDay::Thursday => "ကြာသပတေး",
            MahaboteDay::Friday => "သောကြာ",
            MahaboteDay::Saturday => "စနေ",
        }
    }

    /// Romanized Burmese name.
    pub fn burmese_romanized(&self) -> &'static str {
        match self {
            MahaboteDay::Sunday => "Taninganwe",
            MahaboteDay::Monday => "Taninla",
            MahaboteDay::Tuesday => "Inga",
            MahaboteDay::Wednesday => "Buddahu",
            MahaboteDay::Thursday => "Kyathabade",
            MahaboteDay::Friday => "Thaukkya",
            MahaboteDay::Saturday => "Sanay",
        }
    }

    /// English name.
    pub fn name(&self) -> &'static str {
        match self {
            MahaboteDay::Sunday => "Sunday",
            MahaboteDay::Monday => "Monday",
            MahaboteDay::Tuesday => "Tuesday",
            MahaboteDay::Wednesday => "Wednesday",
            MahaboteDay::Thursday => "Thursday",
            MahaboteDay::Friday => "Friday",
            MahaboteDay::Saturday => "Saturday",
        }
    }

    /// Weekday index: Sunday = 0, ..., Saturday = 6.
    pub fn index(&self) -> usize {
        *self as usize
    }
}

// ---------------------------------------------------------------------------
// MahaboteProfile
// ---------------------------------------------------------------------------

/// A Mahabote profile derived from the birth day of the week.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MahaboteProfile {
    /// Birth day of the week.
    pub birth_day: MahaboteDay,
    /// Ruling planet.
    pub ruling_planet: &'static str,
    /// Burmese animal associated with this day.
    pub animal: &'static str,
    /// Cardinal/intercardinal direction.
    pub direction: &'static str,
    /// Element.
    pub element: &'static str,
    /// Days that are favorable (harmonious).
    pub favorable_days: Vec<MahaboteDay>,
    /// Days that are unfavorable (conflicting).
    pub unfavorable_days: Vec<MahaboteDay>,
    /// Whether this is the PM Wednesday (Rahu) variant.
    pub is_wednesday_pm: bool,
}

impl std::fmt::Display for MahaboteProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pm_tag = if self.is_wednesday_pm {
            " (PM/Rahu)"
        } else {
            ""
        };
        write!(
            f,
            "{}{} — {} — {} — {} — {}",
            self.birth_day.name(),
            pm_tag,
            self.ruling_planet,
            self.animal,
            self.direction,
            self.element,
        )
    }
}

// ---------------------------------------------------------------------------
// CompatibilityResult
// ---------------------------------------------------------------------------

/// Compatibility result between two Mahabote day signs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityResult {
    /// Overall compatibility score (0-100).
    pub score: u32,
    /// Whether the two days are in a favorable relationship.
    pub favorable: bool,
    /// Whether the two days are in an unfavorable (clashing) relationship.
    pub unfavorable: bool,
    /// Textual summary.
    pub summary: &'static str,
}

// ---------------------------------------------------------------------------
// Profile data
// ---------------------------------------------------------------------------

/// Compute a Mahabote profile from a weekday index (0 = Sunday, 6 = Saturday).
///
/// For Wednesday, this returns the AM (Mercury) profile by default. Use
/// [`mahabote_profile_wednesday_pm`] for the PM (Rahu) variant.
pub fn mahabote_profile(weekday: usize) -> MahaboteProfile {
    let day = MahaboteDay::from_weekday(weekday);
    build_profile(day, false)
}

/// Compute the PM Wednesday (Rahu / Tuskless Elephant) profile.
pub fn mahabote_profile_wednesday_pm() -> MahaboteProfile {
    build_profile(MahaboteDay::Wednesday, true)
}

fn build_profile(day: MahaboteDay, wednesday_pm: bool) -> MahaboteProfile {
    // Special case: Wednesday PM has different planet/animal
    if day == MahaboteDay::Wednesday && wednesday_pm {
        return MahaboteProfile {
            birth_day: day,
            ruling_planet: "Rahu",
            animal: "Tuskless Elephant",
            direction: "Northwest",
            element: "Water",
            favorable_days: vec![MahaboteDay::Saturday, MahaboteDay::Monday],
            unfavorable_days: vec![MahaboteDay::Sunday, MahaboteDay::Tuesday],
            is_wednesday_pm: true,
        };
    }

    match day {
        MahaboteDay::Sunday => MahaboteProfile {
            birth_day: day,
            ruling_planet: "Sun",
            animal: "Garuda",
            direction: "Northeast",
            element: "Fire",
            favorable_days: vec![MahaboteDay::Tuesday, MahaboteDay::Friday],
            unfavorable_days: vec![MahaboteDay::Wednesday, MahaboteDay::Saturday],
            is_wednesday_pm: false,
        },
        MahaboteDay::Monday => MahaboteProfile {
            birth_day: day,
            ruling_planet: "Moon",
            animal: "Tiger",
            direction: "East",
            element: "Earth",
            favorable_days: vec![MahaboteDay::Wednesday, MahaboteDay::Thursday],
            unfavorable_days: vec![MahaboteDay::Friday, MahaboteDay::Sunday],
            is_wednesday_pm: false,
        },
        MahaboteDay::Tuesday => MahaboteProfile {
            birth_day: day,
            ruling_planet: "Mars",
            animal: "Lion",
            direction: "Southeast",
            element: "Fire",
            favorable_days: vec![MahaboteDay::Sunday, MahaboteDay::Thursday],
            unfavorable_days: vec![MahaboteDay::Monday, MahaboteDay::Saturday],
            is_wednesday_pm: false,
        },
        MahaboteDay::Wednesday => MahaboteProfile {
            birth_day: day,
            ruling_planet: "Mercury",
            animal: "Tusked Elephant",
            direction: "South",
            element: "Water",
            favorable_days: vec![MahaboteDay::Monday, MahaboteDay::Thursday],
            unfavorable_days: vec![MahaboteDay::Sunday, MahaboteDay::Friday],
            is_wednesday_pm: false,
        },
        MahaboteDay::Thursday => MahaboteProfile {
            birth_day: day,
            ruling_planet: "Jupiter",
            animal: "Rat",
            direction: "West",
            element: "Air",
            favorable_days: vec![MahaboteDay::Monday, MahaboteDay::Tuesday],
            unfavorable_days: vec![MahaboteDay::Friday, MahaboteDay::Saturday],
            is_wednesday_pm: false,
        },
        MahaboteDay::Friday => MahaboteProfile {
            birth_day: day,
            ruling_planet: "Venus",
            animal: "Guinea Pig",
            direction: "North",
            element: "Water",
            favorable_days: vec![MahaboteDay::Sunday, MahaboteDay::Saturday],
            unfavorable_days: vec![MahaboteDay::Monday, MahaboteDay::Thursday],
            is_wednesday_pm: false,
        },
        MahaboteDay::Saturday => MahaboteProfile {
            birth_day: day,
            ruling_planet: "Saturn",
            animal: "Naga",
            direction: "Southwest",
            element: "Earth",
            favorable_days: vec![MahaboteDay::Friday, MahaboteDay::Wednesday],
            unfavorable_days: vec![MahaboteDay::Sunday, MahaboteDay::Tuesday],
            is_wednesday_pm: false,
        },
    }
}

// ---------------------------------------------------------------------------
// Julian Day -> weekday
// ---------------------------------------------------------------------------

/// Compute a Mahabote profile from a Julian Day number.
///
/// The weekday is derived from the JD: `(floor(jd + 1.5) % 7)` gives
/// 0 = Sunday, 1 = Monday, ..., 6 = Saturday.
pub fn mahabote_from_jd(jd: f64) -> MahaboteProfile {
    let weekday = ((jd + 1.5).floor() as i64).rem_euclid(7) as usize;
    mahabote_profile(weekday)
}

// ---------------------------------------------------------------------------
// Compatibility
// ---------------------------------------------------------------------------

/// Compute compatibility between two Mahabote day signs.
///
/// Uses the favorable/unfavorable day relationships:
/// - If A's favorable list contains B's day and vice versa: excellent
/// - If one contains the other: good
/// - If A's unfavorable list contains B's day: challenging
/// - Otherwise: neutral
pub fn mahabote_compatibility(day_a: MahaboteDay, day_b: MahaboteDay) -> CompatibilityResult {
    let profile_a = mahabote_profile(day_a.index());
    let profile_b = mahabote_profile(day_b.index());

    let a_favors_b = profile_a.favorable_days.contains(&day_b);
    let b_favors_a = profile_b.favorable_days.contains(&day_a);
    let a_disfavors_b = profile_a.unfavorable_days.contains(&day_b);
    let b_disfavors_a = profile_b.unfavorable_days.contains(&day_a);

    if day_a == day_b {
        return CompatibilityResult {
            score: 70,
            favorable: true,
            unfavorable: false,
            summary: "Same day — neutral to positive, shared planetary influence",
        };
    }

    if a_favors_b && b_favors_a {
        CompatibilityResult {
            score: 95,
            favorable: true,
            unfavorable: false,
            summary: "Excellent compatibility — mutual harmony",
        }
    } else if a_favors_b || b_favors_a {
        let unfavorable = a_disfavors_b || b_disfavors_a;
        CompatibilityResult {
            score: if unfavorable { 55 } else { 75 },
            favorable: true,
            unfavorable,
            summary: if unfavorable {
                "Mixed — one side favorable, the other conflicting"
            } else {
                "Good compatibility — one-sided harmony"
            },
        }
    } else if a_disfavors_b && b_disfavors_a {
        CompatibilityResult {
            score: 20,
            favorable: false,
            unfavorable: true,
            summary: "Challenging — mutual conflict between planetary rulers",
        }
    } else if a_disfavors_b || b_disfavors_a {
        CompatibilityResult {
            score: 40,
            favorable: false,
            unfavorable: true,
            summary: "Somewhat challenging — one-sided conflict",
        }
    } else {
        CompatibilityResult {
            score: 60,
            favorable: false,
            unfavorable: false,
            summary: "Neutral — no strong harmony or conflict",
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sunday_is_garuda_sun() {
        let p = mahabote_profile(0);
        assert_eq!(p.birth_day, MahaboteDay::Sunday);
        assert_eq!(p.ruling_planet, "Sun");
        assert_eq!(p.animal, "Garuda");
    }

    #[test]
    fn wednesday_am_is_mercury() {
        let p = mahabote_profile(3);
        assert_eq!(p.ruling_planet, "Mercury");
        assert_eq!(p.animal, "Tusked Elephant");
        assert!(!p.is_wednesday_pm);
    }

    #[test]
    fn wednesday_pm_is_rahu() {
        let p = mahabote_profile_wednesday_pm();
        assert_eq!(p.ruling_planet, "Rahu");
        assert_eq!(p.animal, "Tuskless Elephant");
        assert!(p.is_wednesday_pm);
    }

    #[test]
    fn all_7_days_have_profiles() {
        for i in 0..7 {
            let p = mahabote_profile(i);
            assert!(!p.ruling_planet.is_empty());
            assert!(!p.animal.is_empty());
            assert!(!p.direction.is_empty());
            assert!(!p.element.is_empty());
        }
    }

    #[test]
    fn favorable_and_unfavorable_are_different() {
        for i in 0..7 {
            let p = mahabote_profile(i);
            for fav in &p.favorable_days {
                assert!(
                    !p.unfavorable_days.contains(fav),
                    "day {} has {:?} in both favorable and unfavorable",
                    i,
                    fav
                );
            }
        }
    }

    #[test]
    fn jd_known_date() {
        // 2024-01-01 = JD 2460310.5 = Monday
        let p = mahabote_from_jd(2_460_310.5);
        assert_eq!(p.birth_day, MahaboteDay::Monday);
        assert_eq!(p.ruling_planet, "Moon");
        assert_eq!(p.animal, "Tiger");
    }

    #[test]
    fn jd_weekday_cycle() {
        // 7 consecutive days starting from a known Monday
        let jd_monday = 2_460_310.5; // 2024-01-01 Monday
        let expected = [
            MahaboteDay::Monday,
            MahaboteDay::Tuesday,
            MahaboteDay::Wednesday,
            MahaboteDay::Thursday,
            MahaboteDay::Friday,
            MahaboteDay::Saturday,
            MahaboteDay::Sunday,
        ];
        for (i, exp) in expected.iter().enumerate() {
            let p = mahabote_from_jd(jd_monday + i as f64);
            assert_eq!(p.birth_day, *exp, "day offset {i}");
        }
    }

    #[test]
    fn compatibility_same_day() {
        let r = mahabote_compatibility(MahaboteDay::Monday, MahaboteDay::Monday);
        assert!(r.favorable);
        assert!(!r.unfavorable);
        assert_eq!(r.score, 70);
    }

    #[test]
    fn compatibility_mutual_harmony() {
        // Sunday favors Tuesday, Tuesday favors Sunday
        let r = mahabote_compatibility(MahaboteDay::Sunday, MahaboteDay::Tuesday);
        assert!(r.favorable);
        assert_eq!(r.score, 95);
    }

    #[test]
    fn compatibility_score_range() {
        for a in 0..7 {
            for b in 0..7 {
                let r = mahabote_compatibility(
                    MahaboteDay::from_weekday(a),
                    MahaboteDay::from_weekday(b),
                );
                assert!(r.score <= 100, "score {}", r.score);
            }
        }
    }

    #[test]
    fn display_format() {
        let p = mahabote_profile(4); // Thursday
        let s = format!("{p}");
        assert!(s.contains("Thursday"));
        assert!(s.contains("Jupiter"));
        assert!(s.contains("Rat"));
    }

    #[test]
    fn burmese_names_nonempty() {
        for day in &MahaboteDay::ALL {
            assert!(!day.burmese().is_empty());
            assert!(!day.burmese_romanized().is_empty());
        }
    }

    #[test]
    fn saturday_is_naga_saturn() {
        let p = mahabote_profile(6);
        assert_eq!(p.birth_day, MahaboteDay::Saturday);
        assert_eq!(p.ruling_planet, "Saturn");
        assert_eq!(p.animal, "Naga");
    }
}
