use crate::nakshatra::Nakshatra;
use serde::{Deserialize, Serialize};

// Transition/end-time computation lives in a sibling module to keep this file
// within the per-file size budget. Re-exported here so callers can reach the
// most-consumed panchang output (`when does the current tithi end?`) directly
// off the `panchang` namespace alongside the instantaneous `compute_panchang`.
pub use crate::panchang_transitions::{
    ElementInterval, PanchangTransitions, compute_panchang_transitions, karana_transition,
    nakshatra_transition, tithi_transition, yoga_transition,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// The five limbs of the Hindu calendar for a given moment.
pub struct Panchang {
    pub tithi: Tithi,
    pub nakshatra: Nakshatra,
    pub yoga: Yoga,
    pub karana: Karana,
    pub vara: Vara,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Lunar day (1-30) indicating the Moon-Sun elongation.
pub struct Tithi {
    pub number: u8, // 1-30
    pub paksha: Paksha,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Lunar fortnight: Shukla (waxing) or Krishna (waning).
pub enum Paksha {
    Shukla,
    Krishna,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Nithya yoga (1-27) based on Sun+Moon longitude sum.
pub struct Yoga {
    pub number: u8, // 1-27
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Half-tithi unit (1-60), cycling through 11 named karanas.
pub struct Karana {
    pub number: u8,     // 1-60 (position in cycle)
    pub name_index: u8, // 0-10 (which of the 11 karanas)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Day of the week (weekday) with planetary ruler.
pub enum Vara {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

impl Tithi {
    /// Compute the tithi from Sun and Moon sidereal longitudes in degrees.
    pub fn from_sun_moon_deg(sun_lon: f64, moon_lon: f64) -> Self {
        let elongation = (moon_lon - sun_lon).rem_euclid(360.0);
        let tithi_num = (elongation / 12.0) as u8 + 1;
        let paksha = if tithi_num <= 15 {
            Paksha::Shukla
        } else {
            Paksha::Krishna
        };
        Tithi {
            number: tithi_num,
            paksha,
        }
    }

    /// Return the Sanskrit name of this tithi.
    pub fn name(&self) -> &'static str {
        const NAMES: [&str; 30] = [
            "Pratipada",
            "Dwitiya",
            "Tritiya",
            "Chaturthi",
            "Panchami",
            "Shashthi",
            "Saptami",
            "Ashtami",
            "Navami",
            "Dashami",
            "Ekadashi",
            "Dwadashi",
            "Trayodashi",
            "Chaturdashi",
            "Purnima",
            "Pratipada",
            "Dwitiya",
            "Tritiya",
            "Chaturthi",
            "Panchami",
            "Shashthi",
            "Saptami",
            "Ashtami",
            "Navami",
            "Dashami",
            "Ekadashi",
            "Dwadashi",
            "Trayodashi",
            "Chaturdashi",
            "Amavasya",
        ];
        // `number` (1-30) is the source of truth; `name()` is only its display
        // label. A value of 0 / out-of-range from a malformed deserialized
        // payload is clamped to a valid label rather than panicking (usize
        // underflow under debug overflow-checks) — a DELIBERATE display-safety
        // choice for a label accessor, not a silent computation fallback. The
        // computed Tithi `number` itself is never clamped.
        // (Validate-on-Deserialize would be the stricter alternative; tracked.)
        NAMES[(self.number.clamp(1, 30) - 1) as usize]
    }
}

impl Yoga {
    /// Compute the tithi from Sun and Moon sidereal longitudes in degrees.
    pub fn from_sun_moon_deg(sun_lon: f64, moon_lon: f64) -> Self {
        let sum = (sun_lon + moon_lon).rem_euclid(360.0);
        let number = (sum / (360.0 / 27.0)) as u8 + 1;
        Yoga { number }
    }

    /// Return the Sanskrit name of this tithi.
    pub fn name(&self) -> &'static str {
        const NAMES: [&str; 27] = [
            "Vishkambha",
            "Preeti",
            "Ayushman",
            "Saubhagya",
            "Shobhana",
            "Atiganda",
            "Sukarma",
            "Dhriti",
            "Shoola",
            "Ganda",
            "Vriddhi",
            "Dhruva",
            "Vyaghata",
            "Harshana",
            "Vajra",
            "Siddhi",
            "Vyatipata",
            "Variyan",
            "Parigha",
            "Shiva",
            "Siddha",
            "Sadhya",
            "Shubha",
            "Shukla",
            "Brahma",
            "Indra",
            "Vaidhriti",
        ];
        // Clamp `number` (1-27 by construction) — a 0 deserialized from untrusted
        // input would underflow `- 1` and panic in debug.
        // TODO: replace the public `number` field with a validating Deserialize.
        NAMES[(self.number.clamp(1, 27) - 1) as usize]
    }
}

impl Karana {
    pub fn from_tithi_and_elongation(tithi: &Tithi, elongation_deg: f64) -> Self {
        // Clamp before the `- 1`: a malformed Tithi { number: 0 } (e.g. from
        // serde) would otherwise underflow u16 and panic under debug
        // overflow-checks. Matches the clamp the Tithi/Yoga/Karana name lookups use.
        let tithi_idx = (tithi.number.clamp(1, 30) as u16) - 1; // 0-29
        let pos_in_tithi = elongation_deg.rem_euclid(12.0);
        let is_second_half = pos_in_tithi >= 6.0;
        let karana_idx = tithi_idx * 2 + if is_second_half { 1 } else { 0 };

        // 4 fixed karanas at specific positions, 7 movable cycle through the rest
        let name_index = if karana_idx == 0 {
            10 // Kimstughna (fixed, first half of Shukla Pratipada)
        } else if karana_idx == 57 {
            7 // Shakuni (fixed)
        } else if karana_idx == 58 {
            8 // Chatushpada (fixed)
        } else if karana_idx == 59 {
            9 // Naga (fixed)
        } else {
            ((karana_idx - 1) % 7) as u8 // Bava(0) through Vishti(6)
        };

        Karana {
            number: (karana_idx + 1) as u8,
            name_index,
        }
    }

    /// Return the Sanskrit name of this tithi.
    pub fn name(&self) -> &'static str {
        const NAMES: [&str; 11] = [
            "Bava",
            "Balava",
            "Kaulava",
            "Taitila",
            "Gara",
            "Vanija",
            "Vishti",
            "Shakuni",
            "Chatushpada",
            "Naga",
            "Kimstughna",
        ];
        // `name_index` is 0-10 by construction; clamp guards against an
        // out-of-range value deserialized from untrusted input.
        // TODO: replace the public `name_index` field with a validating Deserialize.
        NAMES[self.name_index.clamp(0, 10) as usize]
    }
}

impl Vara {
    pub fn from_jd(jd: f64) -> Self {
        let day = ((jd + 1.5) as i64 % 7) as u8;
        match day {
            0 => Vara::Sunday,
            1 => Vara::Monday,
            2 => Vara::Tuesday,
            3 => Vara::Wednesday,
            4 => Vara::Thursday,
            5 => Vara::Friday,
            _ => Vara::Saturday,
        }
    }

    /// Return the Sanskrit name of this tithi.
    pub fn name(&self) -> &'static str {
        match self {
            Vara::Sunday => "Ravivara",
            Vara::Monday => "Somavara",
            Vara::Tuesday => "Mangalavara",
            Vara::Wednesday => "Budhavara",
            Vara::Thursday => "Guruvara",
            Vara::Friday => "Shukravara",
            Vara::Saturday => "Shanivara",
        }
    }

    pub fn lord(&self) -> &'static str {
        match self {
            Vara::Sunday => "Sun",
            Vara::Monday => "Moon",
            Vara::Tuesday => "Mars",
            Vara::Wednesday => "Mercury",
            Vara::Thursday => "Jupiter",
            Vara::Friday => "Venus",
            Vara::Saturday => "Saturn",
        }
    }
}

/// Compute all five panchang elements from Sun/Moon positions and Julian Date.
pub fn compute_panchang(sun_sidereal_deg: f64, moon_sidereal_deg: f64, jd: f64) -> Panchang {
    let tithi = Tithi::from_sun_moon_deg(sun_sidereal_deg, moon_sidereal_deg);
    let nakshatra = Nakshatra::from_longitude_deg(moon_sidereal_deg);
    let yoga = Yoga::from_sun_moon_deg(sun_sidereal_deg, moon_sidereal_deg);
    let elongation = (moon_sidereal_deg - sun_sidereal_deg).rem_euclid(360.0);
    let karana = Karana::from_tithi_and_elongation(&tithi, elongation);
    let vara = Vara::from_jd(jd);

    Panchang {
        tithi,
        nakshatra,
        yoga,
        karana,
        vara,
    }
}

impl std::fmt::Display for Tithi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} ({})",
            self.paksha_name(),
            self.name(),
            self.number
        )
    }
}

impl Tithi {
    fn paksha_name(&self) -> &'static str {
        match self.paksha {
            Paksha::Shukla => "Shukla",
            Paksha::Krishna => "Krishna",
        }
    }
}

impl std::fmt::Display for Panchang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Tithi: {} | Nakshatra: {} | Yoga: {} | Karana: {} | Vara: {}",
            self.tithi,
            self.nakshatra,
            self.yoga.name(),
            self.karana.name(),
            self.vara.name()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_moon_is_amavasya() {
        // New Moon: Sun and Moon at same longitude
        let t = Tithi::from_sun_moon_deg(100.0, 100.0);
        assert_eq!(t.number, 1);
        assert_eq!(t.paksha, Paksha::Shukla);
    }

    #[test]
    fn full_moon_is_purnima() {
        // Full Moon: Moon 180° ahead of Sun
        let t = Tithi::from_sun_moon_deg(100.0, 280.0);
        assert_eq!(t.number, 16); // Actually Shukla 15 → number 15 is Purnima
        // Let's check: (280-100)=180, 180/12=15 → tithi 16 which is Krishna Pratipada
        // Actually this is correct: elongation 180° → 180/12 = 15 → +1 = 16
        // Purnima is at elongation 168-180°, so 15th tithi
        // Let me test with elongation just under 180
        let t2 = Tithi::from_sun_moon_deg(100.0, 279.0);
        assert_eq!(t2.number, 15); // Purnima
        assert_eq!(t2.name(), "Purnima");
    }

    #[test]
    fn yoga_from_positions() {
        let y = Yoga::from_sun_moon_deg(0.0, 0.0);
        assert_eq!(y.number, 1);
        assert_eq!(y.name(), "Vishkambha");
    }

    #[test]
    fn vara_j2000_is_saturday() {
        // J2000 = 2000-01-01 = Saturday
        let v = Vara::from_jd(2451545.0);
        assert_eq!(v, Vara::Saturday);
        assert_eq!(v.lord(), "Saturn");
    }

    #[test]
    fn karana_names() {
        let t = Tithi {
            number: 1,
            paksha: Paksha::Shukla,
        };
        let k = Karana::from_tithi_and_elongation(&t, 3.0); // first half
        assert_eq!(k.name(), "Kimstughna");
        let k2 = Karana::from_tithi_and_elongation(&t, 9.0); // second half
        assert_ne!(
            k.number, k2.number,
            "Different halves should give different karanas"
        );
    }

    #[test]
    fn full_panchang() {
        let p = compute_panchang(280.0, 100.0, 2451545.0);
        assert!(p.tithi.number >= 1 && p.tithi.number <= 30);
        assert!(p.yoga.number >= 1 && p.yoga.number <= 27);
        assert_eq!(p.vara, Vara::Saturday);
    }

    #[test]
    fn tithi_yoga_name_never_panics_on_invalid_deserialized_number() {
        // A `number: 0` (or out-of-range) value reaching these structs via serde
        // Deserialize must NOT underflow `- 1` / panic — even under debug
        // overflow-checks. Run across the whole u8 range to be exhaustive.
        for n in 0..=u8::MAX {
            let tithi = Tithi {
                number: n,
                paksha: Paksha::Shukla,
            };
            let name = tithi.name();
            assert!(!name.is_empty(), "tithi name empty for number {n}");

            let yoga = Yoga { number: n };
            let yname = yoga.name();
            assert!(!yname.is_empty(), "yoga name empty for number {n}");

            let karana = Karana {
                number: n,
                name_index: n,
            };
            let kname = karana.name();
            assert!(!kname.is_empty(), "karana name empty for name_index {n}");
        }

        // Specifically the 0 case maps to the first valid entry, not a panic.
        assert_eq!(
            Tithi {
                number: 0,
                paksha: Paksha::Shukla
            }
            .name(),
            "Pratipada"
        );
        assert_eq!(Yoga { number: 0 }.name(), "Vishkambha");
        assert_eq!(
            Karana {
                number: 0,
                name_index: 0
            }
            .name(),
            "Bava"
        );
    }
}
