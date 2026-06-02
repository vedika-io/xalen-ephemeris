//! Burmese Mahabote (မဟာဘုတ်) — day-sign profile **and** the 7-house square.
//!
//! # Two layers
//!
//! 1. **Day-sign / ruling-planet profile** ([`mahabote_profile`]): the birth
//!    weekday and its associated ruling planet, animal, direction, element, and
//!    favourable/unfavourable day relationships. Each of the 7 weekdays is ruled
//!    by a planet; Wednesday is uniquely split — AM (Mercury, Tusked Elephant)
//!    and PM (Rahu, Tuskless Elephant).
//!
//! 2. **The Mahabote 7-house square** ([`mahabote_house_square`]): the seven
//!    houses Binga, Ahtun, Yaza, Adipati, Marana, Thike, Puti, with the seven
//!    planet-lords arranged around them. This is the deterministic positional
//!    cast: the birth-weekday lord is seated in Binga (house 1) and the
//!    remaining lords follow the fixed Burmese weekday-lord sequence clockwise
//!    around the seven houses (see [`mahabote_house_square`] for the exact rule
//!    and the convention it implements).
//!
//! # Honest scope of the house square
//!
//! The **skeleton** — seven named houses in fixed order, the birth-lord seated
//! in Binga, and the lords laid out in the Burmese weekday sequence — is the
//! attested deterministic core of Mahabote and is what [`mahabote_house_square`]
//! computes. House **meanings/interpretation** carry regional and lineage
//! variation; this module returns the structural placement (which planet sits in
//! which named house) and the canonical short gloss of each house, not a
//! lineage-specific predictive reading.

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

/// The Mahabote **day-sign / ruling-planet profile** for a birth weekday.
///
/// This is the weekday-ruler profile (planet, animal, direction, element and
/// favourable/unfavourable days), NOT the cast Mahabote 7-house square. See the
/// module-level scope note.
#[derive(Debug, Clone, Serialize)]
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
#[derive(Debug, Clone, Serialize)]
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
// Mahabote 7-house square
// ---------------------------------------------------------------------------

/// The seven Mahabote houses, in their fixed cyclic order.
///
/// House 1 (Binga) seats the birth-weekday lord; the remaining houses follow
/// clockwise. The English gloss is the canonical short meaning of each house.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MahaboteHouse {
    /// House 1 — origin / self.
    Binga,
    /// House 2 — support / sustenance.
    Ahtun,
    /// House 3 — power / authority (Yaza ← "raza", king).
    Yaza,
    /// House 4 — mastery / lordship.
    Adipati,
    /// House 5 — decline / mortality.
    Marana,
    /// House 6 — foundation / wisdom.
    Thike,
    /// House 7 — completion / renewal.
    Puti,
}

impl MahaboteHouse {
    /// The seven houses in fixed cyclic order, indexed 0 (Binga) .. 6 (Puti).
    pub const ALL: [MahaboteHouse; 7] = [
        MahaboteHouse::Binga,
        MahaboteHouse::Ahtun,
        MahaboteHouse::Yaza,
        MahaboteHouse::Adipati,
        MahaboteHouse::Marana,
        MahaboteHouse::Thike,
        MahaboteHouse::Puti,
    ];

    /// Romanized house name.
    pub fn name(&self) -> &'static str {
        match self {
            MahaboteHouse::Binga => "Binga",
            MahaboteHouse::Ahtun => "Ahtun",
            MahaboteHouse::Yaza => "Yaza",
            MahaboteHouse::Adipati => "Adipati",
            MahaboteHouse::Marana => "Marana",
            MahaboteHouse::Thike => "Thike",
            MahaboteHouse::Puti => "Puti",
        }
    }

    /// Canonical short meaning of the house.
    pub fn meaning(&self) -> &'static str {
        match self {
            MahaboteHouse::Binga => "origin / self",
            MahaboteHouse::Ahtun => "support / sustenance",
            MahaboteHouse::Yaza => "power / authority",
            MahaboteHouse::Adipati => "mastery / lordship",
            MahaboteHouse::Marana => "decline / mortality",
            MahaboteHouse::Thike => "foundation / wisdom",
            MahaboteHouse::Puti => "completion / renewal",
        }
    }
}

/// One seated house in the Mahabote square: the house and the planet-lord
/// placed in it.
#[derive(Debug, Clone, Serialize)]
pub struct MahaboteSeat {
    /// The house position (Binga .. Puti).
    pub house: MahaboteHouse,
    /// The planet-lord seated in this house.
    pub planet: &'static str,
    /// The weekday whose lord this is (its index 0 = Sunday .. 6 = Saturday;
    /// the Rahu seat carries the Wednesday index with the PM flag set).
    pub from_weekday: MahaboteDay,
    /// `true` when this seat is the Wednesday-PM (Rahu) substitution.
    pub is_rahu: bool,
}

/// The fully cast Mahabote 7-house square.
#[derive(Debug, Clone, Serialize)]
pub struct MahaboteHouseSquare {
    /// Birth weekday whose lord opens the square in Binga.
    pub birth_day: MahaboteDay,
    /// `true` when the birth lord is the Wednesday-PM (Rahu) variant.
    pub birth_is_rahu: bool,
    /// The seven seats, in house order Binga (index 0) .. Puti (index 6).
    pub seats: [MahaboteSeat; 7],
}

impl MahaboteHouseSquare {
    /// Look up which house a given planet-lord occupies, by weekday index.
    pub fn house_of_weekday(&self, day: MahaboteDay) -> MahaboteHouse {
        self.seats
            .iter()
            .find(|s| s.from_weekday == day)
            .map(|s| s.house)
            .unwrap_or(MahaboteHouse::Binga)
    }
}

/// The Burmese weekday-lord sequence used to lay out the house square.
///
/// This is the ordered list of planet-lords by weekday (Sunday → Saturday),
/// i.e. the heptagram/weekday order Sun, Moon, Mars, Mercury, Jupiter, Venus,
/// Saturn. The seven planet-lords are laid into the seven houses starting from
/// the birth-weekday lord; stepping forward in this weekday sequence walks the
/// houses Binga → Ahtun → … → Puti.
const WEEKDAY_LORDS: [&str; 7] = [
    "Sun",     // Sunday
    "Moon",    // Monday
    "Mars",    // Tuesday
    "Mercury", // Wednesday (AM)
    "Jupiter", // Thursday
    "Venus",   // Friday
    "Saturn",  // Saturday
];

/// Cast the Mahabote 7-house square for a birth weekday.
///
/// # Rule (deterministic)
/// The birth-weekday lord is seated in **Binga** (house 1). The remaining six
/// lords follow in the fixed Burmese weekday sequence
/// ([`WEEKDAY_LORDS`]: Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn) laid
/// clockwise around the houses Ahtun → Yaza → Adipati → Marana → Thike → Puti.
/// Concretely the lord whose weekday index is `(birth + h) mod 7` occupies
/// house `h` (`h = 0` = Binga). The placement is therefore a pure rotation of
/// the weekday sequence anchored on the birth day — fully determined by the
/// birth weekday alone.
///
/// `weekday` is `0 = Sunday .. 6 = Saturday`. The Wednesday-PM (Rahu) variant
/// is selected with `wednesday_pm = true`, in which case Binga seats Rahu in
/// place of Mercury; the rest of the cycle is unchanged (Rahu substitutes for
/// the Wednesday lord wherever it falls).
pub fn mahabote_house_square(weekday: usize, wednesday_pm: bool) -> MahaboteHouseSquare {
    let birth = weekday % 7;
    let birth_is_rahu = birth == MahaboteDay::Wednesday.index() && wednesday_pm;

    let seats: [MahaboteSeat; 7] = std::array::from_fn(|h| {
        let lord_weekday = (birth + h) % 7;
        let is_rahu = lord_weekday == MahaboteDay::Wednesday.index() && wednesday_pm;
        let planet = if is_rahu {
            "Rahu"
        } else {
            WEEKDAY_LORDS[lord_weekday]
        };
        MahaboteSeat {
            house: MahaboteHouse::ALL[h],
            planet,
            from_weekday: MahaboteDay::from_weekday(lord_weekday),
            is_rahu,
        }
    });

    MahaboteHouseSquare {
        birth_day: MahaboteDay::from_weekday(birth),
        birth_is_rahu,
        seats,
    }
}

/// Cast the Mahabote 7-house square from a Julian Day number (AM/Mercury
/// Wednesday). For the Wednesday-PM (Rahu) variant call
/// [`mahabote_house_square`] with `wednesday_pm = true`.
pub fn mahabote_house_square_from_jd(jd: f64) -> MahaboteHouseSquare {
    let weekday = ((jd + 1.5).floor() as i64).rem_euclid(7) as usize;
    mahabote_house_square(weekday, false)
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

    // -------------------------------------------------------------------
    // Mahabote 7-house square
    // -------------------------------------------------------------------

    #[test]
    fn square_has_seven_distinct_houses_in_order() {
        let sq = mahabote_house_square(0, false);
        assert_eq!(sq.seats.len(), 7);
        for (h, seat) in sq.seats.iter().enumerate() {
            assert_eq!(
                seat.house,
                MahaboteHouse::ALL[h],
                "seat {h} must be house {}",
                MahaboteHouse::ALL[h].name()
            );
        }
        // All seven houses are present exactly once.
        let mut houses: Vec<_> = sq.seats.iter().map(|s| s.house).collect();
        houses.sort_by_key(|h| *h as usize);
        houses.dedup();
        assert_eq!(houses.len(), 7, "all seven houses must be distinct");
    }

    #[test]
    fn birth_lord_seated_in_binga() {
        // Sunday → Sun in Binga; Saturday → Saturn in Binga.
        let sun = mahabote_house_square(0, false);
        assert_eq!(sun.seats[0].house, MahaboteHouse::Binga);
        assert_eq!(sun.seats[0].planet, "Sun");
        assert_eq!(
            sun.house_of_weekday(MahaboteDay::Sunday),
            MahaboteHouse::Binga
        );

        let sat = mahabote_house_square(6, false);
        assert_eq!(sat.seats[0].planet, "Saturn");
        assert_eq!(sat.birth_day, MahaboteDay::Saturday);
    }

    /// Worked example: a Wednesday-AM birth seats Mercury in Binga, then the
    /// weekday sequence rotates: Thursday/Jupiter → Ahtun, Friday/Venus → Yaza,
    /// Saturday/Saturn → Adipati, Sunday/Sun → Marana, Monday/Moon → Thike,
    /// Tuesday/Mars → Puti. This is the pure (birth + h) mod 7 rotation.
    #[test]
    fn wednesday_am_square_worked_example() {
        let sq = mahabote_house_square(MahaboteDay::Wednesday.index(), false);
        let expected = [
            (MahaboteHouse::Binga, "Mercury"),
            (MahaboteHouse::Ahtun, "Jupiter"),
            (MahaboteHouse::Yaza, "Venus"),
            (MahaboteHouse::Adipati, "Saturn"),
            (MahaboteHouse::Marana, "Sun"),
            (MahaboteHouse::Thike, "Moon"),
            (MahaboteHouse::Puti, "Mars"),
        ];
        for (i, (house, planet)) in expected.iter().enumerate() {
            assert_eq!(sq.seats[i].house, *house, "seat {i} house");
            assert_eq!(sq.seats[i].planet, *planet, "seat {i} planet");
        }
        assert!(!sq.birth_is_rahu);
    }

    /// Wednesday-PM seats Rahu (not Mercury) in Binga; the rest of the cycle is
    /// the same rotation, and only the Wednesday seat carries the Rahu flag.
    #[test]
    fn wednesday_pm_seats_rahu_in_binga() {
        let sq = mahabote_house_square(MahaboteDay::Wednesday.index(), true);
        assert_eq!(sq.seats[0].house, MahaboteHouse::Binga);
        assert_eq!(sq.seats[0].planet, "Rahu");
        assert!(sq.seats[0].is_rahu);
        assert!(sq.birth_is_rahu);
        // Exactly one Rahu seat in the whole square.
        let rahu_count = sq.seats.iter().filter(|s| s.is_rahu).count();
        assert_eq!(rahu_count, 1, "exactly one Rahu seat (the Wednesday lord)");
        // A non-Wednesday birth with the PM flag set still substitutes Rahu for
        // the Wednesday lord wherever it lands.
        let friday = mahabote_house_square(MahaboteDay::Friday.index(), true);
        let rahu_seat = friday.seats.iter().find(|s| s.is_rahu).unwrap();
        assert_eq!(rahu_seat.from_weekday, MahaboteDay::Wednesday);
        assert_eq!(rahu_seat.planet, "Rahu");
    }

    /// The placement is a deterministic rotation: the lord of weekday
    /// `(birth + h) mod 7` sits in house `h`, for every birth day.
    #[test]
    fn placement_is_birth_anchored_rotation() {
        for birth in 0..7usize {
            let sq = mahabote_house_square(birth, false);
            for h in 0..7usize {
                let expected_weekday = (birth + h) % 7;
                assert_eq!(
                    sq.seats[h].from_weekday,
                    MahaboteDay::from_weekday(expected_weekday),
                    "birth {birth}, house {h}: lord must be weekday {expected_weekday}"
                );
                assert_eq!(sq.seats[h].planet, WEEKDAY_LORDS[expected_weekday]);
            }
        }
    }

    #[test]
    fn square_from_jd_matches_weekday() {
        // 2024-01-01 JD 2460310.5 = Monday → Moon in Binga.
        let sq = mahabote_house_square_from_jd(2_460_310.5);
        assert_eq!(sq.birth_day, MahaboteDay::Monday);
        assert_eq!(sq.seats[0].planet, "Moon");
        assert_eq!(sq.seats[0].house, MahaboteHouse::Binga);
    }

    #[test]
    fn house_names_and_meanings_nonempty() {
        for house in &MahaboteHouse::ALL {
            assert!(!house.name().is_empty());
            assert!(!house.meaning().is_empty());
        }
    }
}
