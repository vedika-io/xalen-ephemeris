use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Tibetan 12 Animals (same as Chinese zodiac)
// ---------------------------------------------------------------------------

/// The 12 animals of the Tibetan zodiac cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TibetanAnimal {
    Hare,
    Dragon,
    Snake,
    Horse,
    Sheep,
    Monkey,
    Bird,
    Dog,
    Pig,
    Mouse,
    Ox,
    Tiger,
}

impl TibetanAnimal {
    /// All 12 animals in traditional Tibetan order (starting from Hare).
    pub const ALL: [TibetanAnimal; 12] = [
        TibetanAnimal::Hare,
        TibetanAnimal::Dragon,
        TibetanAnimal::Snake,
        TibetanAnimal::Horse,
        TibetanAnimal::Sheep,
        TibetanAnimal::Monkey,
        TibetanAnimal::Bird,
        TibetanAnimal::Dog,
        TibetanAnimal::Pig,
        TibetanAnimal::Mouse,
        TibetanAnimal::Ox,
        TibetanAnimal::Tiger,
    ];

    /// Return the animal from a 0-based index (wraps at 12).
    pub fn from_index(i: usize) -> Self {
        Self::ALL[i % 12]
    }

    /// Human-readable English name.
    pub fn name(&self) -> &'static str {
        match self {
            TibetanAnimal::Hare => "Hare",
            TibetanAnimal::Dragon => "Dragon",
            TibetanAnimal::Snake => "Snake",
            TibetanAnimal::Horse => "Horse",
            TibetanAnimal::Sheep => "Sheep",
            TibetanAnimal::Monkey => "Monkey",
            TibetanAnimal::Bird => "Bird",
            TibetanAnimal::Dog => "Dog",
            TibetanAnimal::Pig => "Pig",
            TibetanAnimal::Mouse => "Mouse",
            TibetanAnimal::Ox => "Ox",
            TibetanAnimal::Tiger => "Tiger",
        }
    }
}

// ---------------------------------------------------------------------------
// Tibetan 5 Elements
// ---------------------------------------------------------------------------

/// The 5 elements (*'byung ba lnga*) of Tibetan astrology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TibetanElement {
    Wood,
    Fire,
    Earth,
    Iron,
    Water,
}

impl TibetanElement {
    /// All 5 elements in traditional order.
    pub const ALL: [TibetanElement; 5] = [
        TibetanElement::Wood,
        TibetanElement::Fire,
        TibetanElement::Earth,
        TibetanElement::Iron,
        TibetanElement::Water,
    ];

    /// Return the element from a 0-based index (wraps at 5).
    pub fn from_index(i: usize) -> Self {
        Self::ALL[i % 5]
    }

    /// Human-readable English name.
    pub fn name(&self) -> &'static str {
        match self {
            TibetanElement::Wood => "Wood",
            TibetanElement::Fire => "Fire",
            TibetanElement::Earth => "Earth",
            TibetanElement::Iron => "Iron",
            TibetanElement::Water => "Water",
        }
    }
}

// ---------------------------------------------------------------------------
// Gender (Pho / Mo)
// ---------------------------------------------------------------------------

/// Gender of a Tibetan year: Male (*pho*) or Female (*mo*).
/// Each element spans two consecutive years: first male, then female.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TibetanGender {
    Male,
    Female,
}

impl TibetanGender {
    pub fn name(&self) -> &'static str {
        match self {
            TibetanGender::Male => "Male",
            TibetanGender::Female => "Female",
        }
    }
}

// ---------------------------------------------------------------------------
// Tibetan Month
// ---------------------------------------------------------------------------

/// The 12 Tibetan months (lunar months, *zla ba*).
///
/// Tibetan months are lunar. The 1st month (*dbo*) roughly corresponds to
/// February/March. Month numbering is 1-12 in traditional use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TibetanMonth {
    /// 1st month (Mchu / Dbo)
    First,
    /// 2nd month (Dbo)
    Second,
    /// 3rd month (Nag pa)
    Third,
    /// 4th month (Sa ga)
    Fourth,
    /// 5th month (Snron)
    Fifth,
    /// 6th month (Chu stod)
    Sixth,
    /// 7th month (Gro bzhin)
    Seventh,
    /// 8th month (Khrums)
    Eighth,
    /// 9th month (Tha skar)
    Ninth,
    /// 10th month (Smin drug)
    Tenth,
    /// 11th month (Mgo)
    Eleventh,
    /// 12th month (Rgyal)
    Twelfth,
}

impl TibetanMonth {
    /// All 12 months in order.
    pub const ALL: [TibetanMonth; 12] = [
        TibetanMonth::First,
        TibetanMonth::Second,
        TibetanMonth::Third,
        TibetanMonth::Fourth,
        TibetanMonth::Fifth,
        TibetanMonth::Sixth,
        TibetanMonth::Seventh,
        TibetanMonth::Eighth,
        TibetanMonth::Ninth,
        TibetanMonth::Tenth,
        TibetanMonth::Eleventh,
        TibetanMonth::Twelfth,
    ];

    /// Return the month from a 1-based number (wraps).
    pub fn from_number(n: u32) -> Self {
        Self::ALL[((n - 1) % 12) as usize]
    }

    /// 1-based month number.
    pub fn number(&self) -> u32 {
        match self {
            TibetanMonth::First => 1,
            TibetanMonth::Second => 2,
            TibetanMonth::Third => 3,
            TibetanMonth::Fourth => 4,
            TibetanMonth::Fifth => 5,
            TibetanMonth::Sixth => 6,
            TibetanMonth::Seventh => 7,
            TibetanMonth::Eighth => 8,
            TibetanMonth::Ninth => 9,
            TibetanMonth::Tenth => 10,
            TibetanMonth::Eleventh => 11,
            TibetanMonth::Twelfth => 12,
        }
    }
}

// ---------------------------------------------------------------------------
// TibetanYear
// ---------------------------------------------------------------------------

/// A Tibetan year with its animal, element, gender, and Rabjung cycle number.
///
/// The 60-year Rabjung cycle combines 12 animals with 5 elements (each
/// element spanning 2 years: male then female), giving 60 unique
/// combinations. The 1st Rabjung cycle began in 1027 CE.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TibetanYear {
    /// Western calendar year.
    pub western_year: i32,
    /// Animal of the year.
    pub animal: TibetanAnimal,
    /// Element of the year.
    pub element: TibetanElement,
    /// Gender (male/female).
    pub gender: TibetanGender,
    /// Rabjung cycle number (1-based). The 1st Rabjung began in 1027 CE.
    pub rabjung_cycle: i32,
    /// Year within the Rabjung cycle (1-60).
    pub year_in_cycle: u32,
}

impl std::fmt::Display for TibetanYear {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} (Rabjung {}, year {})",
            self.gender.name(),
            self.element.name(),
            self.animal.name(),
            self.rabjung_cycle,
            self.year_in_cycle,
        )
    }
}

// ---------------------------------------------------------------------------
// Computation
// ---------------------------------------------------------------------------

/// Compute the Tibetan year designation for a given Western year.
///
/// The 60-year Rabjung cycle started in 1027 CE. Each cycle of 60 years
/// is composed of 5 elements x 2 genders x 12 animals = 60 unique
/// year designations.
///
/// The year 1027 CE = Fire-Female-Hare (Rabjung 1, year 1).
///
/// The Tibetan 60-year cycle uses the same sexagenary system as the Chinese
/// calendar.  Element and gender are derived from the Heavenly Stem of the
/// Chinese sexagenary year (year 4 CE = Jia-Zi), which guarantees alignment
/// with the Chinese zodiac element (e.g. 2024 = Wood Dragon in both systems).
///
/// Verified: 2024 = Male Wood Dragon
/// Reference: https://ravencypresswood.com/2024/01/20/2024-year-of-the-male-wood-dragon/
pub fn tibetan_year(western_year: i32) -> TibetanYear {
    // Offset from the start of the Rabjung system (1027 CE).
    let offset = (western_year - 1027).rem_euclid(60);

    // Rabjung cycle (1-based): the 1st cycle is 1027-1086.
    let rabjung_cycle = (western_year - 1027).div_euclid(60) + 1;
    let year_in_cycle = (offset as u32) + 1;

    // Animal: cycles every 12 years. 1027 = Hare (index 0).
    let animal = TibetanAnimal::from_index(offset as usize);

    // Element and gender: derived from the Chinese Heavenly Stem.
    // The sexagenary cycle starts at year 4 CE (Jia-Zi).
    // Stem index = (year - 4) % 10.
    // Stems 0,1=Wood  2,3=Fire  4,5=Earth  6,7=Iron(Metal)  8,9=Water
    // Even stem = Male (Yang), Odd stem = Female (Yin).
    // Verified: 1027 CE -> stem = (1027-4)%10 = 3 -> Fire, Female. Correct.
    //           2024 CE -> stem = (2024-4)%10 = 0 -> Wood, Male.   Correct.
    let stem_idx = (western_year - 4).rem_euclid(10) as usize;
    let element = TibetanElement::from_index(stem_idx / 2);
    let gender = if stem_idx.is_multiple_of(2) {
        TibetanGender::Male
    } else {
        TibetanGender::Female
    };

    TibetanYear {
        western_year,
        animal,
        element,
        gender,
        rabjung_cycle,
        year_in_cycle,
    }
}

/// Approximate Julian Day of Losar (Tibetan New Year) for the given Western
/// year.
///
/// Losar falls on the first day of the first Tibetan month, which is the new
/// moon closest to the start of the Tibetan calendar year. This is
/// approximately the new moon in February, roughly 1-2 months after the
/// Western New Year.
///
/// This is an approximation based on the average synodic month. For precise
/// dates, a full lunar ephemeris is needed.
pub fn losar_approximate_jd(western_year: i32) -> f64 {
    // The Tibetan New Year typically falls in February.
    // We approximate it as the new moon near February 1.
    //
    // JD of a known new moon: January 6, 2000 = JD 2451550.1
    // Synodic month = 29.530588853 days
    const KNOWN_NEW_MOON_JD: f64 = 2_451_550.1;
    const SYNODIC_MONTH: f64 = 29.530_588_853;

    // Target: approximately February 1 of the given year.
    // JD of Jan 1 2000 = 2451544.5.
    let jan1_jd = 2_451_544.5 + (western_year - 2000) as f64 * 365.25;
    let target_jd = jan1_jd + 31.0; // ~February 1

    // Find the new moon closest to the target.
    let months_since = ((target_jd - KNOWN_NEW_MOON_JD) / SYNODIC_MONTH).round();
    KNOWN_NEW_MOON_JD + months_since * SYNODIC_MONTH
}

/// Return the Tibetan animal for a given Western year.
pub fn animal_for_year(western_year: i32) -> TibetanAnimal {
    tibetan_year(western_year).animal
}

/// Return the Tibetan element for a given Western year.
pub fn element_for_year(western_year: i32) -> TibetanElement {
    tibetan_year(western_year).element
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn year_1027_is_fire_female_hare() {
        let ty = tibetan_year(1027);
        assert_eq!(ty.animal, TibetanAnimal::Hare);
        assert_eq!(ty.element, TibetanElement::Fire);
        assert_eq!(ty.gender, TibetanGender::Female);
        assert_eq!(ty.rabjung_cycle, 1);
        assert_eq!(ty.year_in_cycle, 1);
    }

    #[test]
    fn year_1028_is_earth_male_dragon() {
        // 1028: stem = (1028-4)%10 = 4 -> Earth, Male (Yang)
        let ty = tibetan_year(1028);
        assert_eq!(ty.animal, TibetanAnimal::Dragon);
        assert_eq!(ty.element, TibetanElement::Earth);
        assert_eq!(ty.gender, TibetanGender::Male);
        assert_eq!(ty.rabjung_cycle, 1);
        assert_eq!(ty.year_in_cycle, 2);
    }

    #[test]
    fn year_1029_is_earth_female_snake() {
        // 1029: stem = (1029-4)%10 = 5 -> Earth, Female (Yin)
        let ty = tibetan_year(1029);
        assert_eq!(ty.animal, TibetanAnimal::Snake);
        assert_eq!(ty.element, TibetanElement::Earth);
        assert_eq!(ty.gender, TibetanGender::Female);
    }

    #[test]
    fn sixty_year_cycle_repeats() {
        let ty1 = tibetan_year(1027);
        let ty2 = tibetan_year(1087); // 1027 + 60
        assert_eq!(ty1.animal, ty2.animal);
        assert_eq!(ty1.element, ty2.element);
        assert_eq!(ty1.gender, ty2.gender);
        assert_eq!(ty2.rabjung_cycle, 2); // Second cycle
        assert_eq!(ty2.year_in_cycle, 1);
    }

    #[test]
    fn rabjung_17_starts_1987() {
        // The 17th Rabjung cycle started in 1987 CE (1027 + 16*60 = 1987)
        let ty = tibetan_year(1987);
        assert_eq!(ty.rabjung_cycle, 17);
        assert_eq!(ty.year_in_cycle, 1);
        assert_eq!(ty.animal, TibetanAnimal::Hare);
    }

    #[test]
    fn year_2024_designation() {
        // Verified: https://ravencypresswood.com/2024/01/20/2024-year-of-the-male-wood-dragon/
        let ty = tibetan_year(2024);
        // Animal: (2024-1027)%12 = 997%12 = 1 = Dragon
        assert_eq!(ty.animal, TibetanAnimal::Dragon);
        // Element: stem = (2024-4)%10 = 0 -> 0/2 = 0 = Wood
        assert_eq!(ty.element, TibetanElement::Wood);
        // Gender: stem 0 is even -> Male
        assert_eq!(ty.gender, TibetanGender::Male);
        // Rabjung: (997 / 60) + 1 = 16 + 1 = 17
        assert_eq!(ty.rabjung_cycle, 17);
        assert_eq!(ty.year_in_cycle, 38);
    }

    #[test]
    fn all_12_animals_covered() {
        let mut animals = std::collections::HashSet::new();
        for y in 1027..1039 {
            animals.insert(tibetan_year(y).animal);
        }
        assert_eq!(animals.len(), 12);
    }

    #[test]
    fn all_5_elements_covered_in_10_years() {
        let mut elements = std::collections::HashSet::new();
        for y in 1027..1037 {
            elements.insert(tibetan_year(y).element);
        }
        assert_eq!(elements.len(), 5);
    }

    #[test]
    fn gender_alternates() {
        // Gender is derived from the Chinese Heavenly Stem: even stem = Male, odd = Female.
        for y in 1027..1087 {
            let ty = tibetan_year(y);
            let stem_idx = (y - 4).rem_euclid(10);
            let expected = if stem_idx % 2 == 0 {
                TibetanGender::Male
            } else {
                TibetanGender::Female
            };
            assert_eq!(ty.gender, expected, "year {y}");
        }
    }

    #[test]
    fn losar_falls_in_february_range() {
        for year in [2020, 2024, 2025, 2026] {
            let jd = losar_approximate_jd(year);
            // Should be roughly JD for late Jan - early March
            let jan1_jd = 2_451_544.5 + (year - 2000) as f64 * 365.25;
            let offset = jd - jan1_jd;
            assert!(
                offset > 15.0 && offset < 75.0,
                "Losar {year} at JD {jd} is {offset} days after Jan 1 (expected 15-75)"
            );
        }
    }

    #[test]
    fn animal_for_year_helper() {
        assert_eq!(animal_for_year(2024), TibetanAnimal::Dragon);
        assert_eq!(animal_for_year(1027), TibetanAnimal::Hare);
    }

    #[test]
    fn element_for_year_helper() {
        assert_eq!(element_for_year(1027), TibetanElement::Fire);
        // 1028: stem = (1028-4)%10 = 4 -> Earth (not Fire; element follows Chinese stem)
        assert_eq!(element_for_year(1028), TibetanElement::Earth);
    }

    #[test]
    fn display_format() {
        let ty = tibetan_year(1027);
        let s = format!("{ty}");
        assert!(s.contains("Fire"), "should contain element");
        assert!(s.contains("Hare"), "should contain animal");
        assert!(s.contains("Female"), "should contain gender");
        assert!(s.contains("Rabjung 1"), "should contain cycle");
    }

    #[test]
    fn tibetan_month_from_number() {
        assert_eq!(TibetanMonth::from_number(1), TibetanMonth::First);
        assert_eq!(TibetanMonth::from_number(12), TibetanMonth::Twelfth);
        assert_eq!(TibetanMonth::from_number(13), TibetanMonth::First); // wraps
    }

    #[test]
    fn tibetan_month_number_roundtrip() {
        for m in &TibetanMonth::ALL {
            assert_eq!(TibetanMonth::from_number(m.number()), *m);
        }
    }
}
