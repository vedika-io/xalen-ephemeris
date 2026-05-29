use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// The 12 rashis (zodiac signs) of Vedic astrology.
pub enum Rashi {
    Mesha = 0,      // Aries
    Vrishabha = 1,  // Taurus
    Mithuna = 2,    // Gemini
    Karka = 3,      // Cancer
    Simha = 4,      // Leo
    Kanya = 5,      // Virgo
    Tula = 6,       // Libra
    Vrishchika = 7, // Scorpio
    Dhanu = 8,      // Sagittarius
    Makara = 9,     // Capricorn
    Kumbha = 10,    // Aquarius
    Meena = 11,     // Pisces
}

impl Rashi {
    /// Determine the rashi from a sidereal longitude in degrees.
    pub fn from_longitude_deg(lon: f64) -> Self {
        let idx = (lon.rem_euclid(360.0) / 30.0) as usize;
        Self::from_index(idx)
    }

    /// Create a rashi from a 0-based index (0=Mesha, 11=Meena).
    pub fn from_index(idx: usize) -> Self {
        match idx % 12 {
            0 => Rashi::Mesha,
            1 => Rashi::Vrishabha,
            2 => Rashi::Mithuna,
            3 => Rashi::Karka,
            4 => Rashi::Simha,
            5 => Rashi::Kanya,
            6 => Rashi::Tula,
            7 => Rashi::Vrishchika,
            8 => Rashi::Dhanu,
            9 => Rashi::Makara,
            10 => Rashi::Kumbha,
            _ => Rashi::Meena,
        }
    }

    /// Return the 0-based index (0=Mesha through 11=Meena).
    pub fn index(&self) -> usize {
        *self as usize
    }

    /// Return this rashi's planetary ruler.
    pub fn lord(&self) -> &'static str {
        match self {
            Rashi::Mesha | Rashi::Vrishchika => "Mars",
            Rashi::Vrishabha | Rashi::Tula => "Venus",
            Rashi::Mithuna | Rashi::Kanya => "Mercury",
            Rashi::Karka => "Moon",
            Rashi::Simha => "Sun",
            Rashi::Dhanu | Rashi::Meena => "Jupiter",
            Rashi::Makara | Rashi::Kumbha => "Saturn",
        }
    }

    /// Return the element (Fire, Earth, Air, Water).
    pub fn element(&self) -> Element {
        match self.index() % 4 {
            0 => Element::Fire,
            1 => Element::Earth,
            2 => Element::Air,
            _ => Element::Water,
        }
    }

    /// Return the modality (Movable, Fixed, Dual).
    pub fn modality(&self) -> Modality {
        match self.index() % 3 {
            0 => Modality::Movable,
            1 => Modality::Fixed,
            _ => Modality::Dual,
        }
    }

    /// Return the Western zodiac name (e.g. "Aries" for Mesha).
    pub fn western_name(&self) -> &'static str {
        match self {
            Rashi::Mesha => "Aries",
            Rashi::Vrishabha => "Taurus",
            Rashi::Mithuna => "Gemini",
            Rashi::Karka => "Cancer",
            Rashi::Simha => "Leo",
            Rashi::Kanya => "Virgo",
            Rashi::Tula => "Libra",
            Rashi::Vrishchika => "Scorpio",
            Rashi::Dhanu => "Sagittarius",
            Rashi::Makara => "Capricorn",
            Rashi::Kumbha => "Aquarius",
            Rashi::Meena => "Pisces",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Classical element classification of zodiac signs.
pub enum Element {
    Fire,
    Earth,
    Air,
    Water,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Sign quality/modality classification.
pub enum Modality {
    Movable,
    Fixed,
    Dual,
}

impl std::fmt::Display for Rashi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({})",
            match self {
                Rashi::Mesha => "Mesha",
                Rashi::Vrishabha => "Vrishabha",
                Rashi::Mithuna => "Mithuna",
                Rashi::Karka => "Karka",
                Rashi::Simha => "Simha",
                Rashi::Kanya => "Kanya",
                Rashi::Tula => "Tula",
                Rashi::Vrishchika => "Vrishchika",
                Rashi::Dhanu => "Dhanu",
                Rashi::Makara => "Makara",
                Rashi::Kumbha => "Kumbha",
                Rashi::Meena => "Meena",
            },
            self.western_name()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rashi_from_longitude() {
        assert_eq!(Rashi::from_longitude_deg(0.0), Rashi::Mesha);
        assert_eq!(Rashi::from_longitude_deg(45.0), Rashi::Vrishabha);
        assert_eq!(Rashi::from_longitude_deg(359.9), Rashi::Meena);
        assert_eq!(Rashi::from_longitude_deg(90.0), Rashi::Karka);
    }

    #[test]
    fn lords() {
        assert_eq!(Rashi::Mesha.lord(), "Mars");
        assert_eq!(Rashi::Simha.lord(), "Sun");
        assert_eq!(Rashi::Karka.lord(), "Moon");
    }

    #[test]
    fn elements_and_modalities() {
        assert_eq!(Rashi::Mesha.element(), Element::Fire);
        assert_eq!(Rashi::Mesha.modality(), Modality::Movable);
        assert_eq!(Rashi::Vrishabha.modality(), Modality::Fixed);
        assert_eq!(Rashi::Mithuna.modality(), Modality::Dual);
    }
}
