use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// House division system for astrological chart computation.
pub enum HouseSystem {
    WholeSign,
    Equal,
    Placidus,
    Koch,
    Porphyry,
    Regiomontanus,
    Campanus,
    Morinus,
    Alcabitius,
    Topocentric,
    Meridian,
    Vehlow,
    Sripati,
    KrusinskiPisa,
    /// Gauquelin sectors — 36-sector division projected onto 12 cusps.
    /// Note: Gauquelin sectors require a 36-sector model not yet implemented.
    /// Currently returns Placidus as approximation.
    Gauquelin,
    /// Sunshine house system (Makransky variant) — sun-based houses.
    /// Note: this is the MC/ASC declination-based approximation. Full Sunshine
    /// houses require the Sun's position at the moment.
    SunshineMakransky,
    /// Sunshine house system (Treindl variant) — sun-based houses.
    /// Note: this is the MC/ASC declination-based approximation. Full Sunshine
    /// houses require the Sun's position at the moment.
    SunshineTreindl,
    /// Pullen Sinusoidal Delta — sinusoidal interpolation between quadrant boundaries.
    PullenSinusoidalDelta,
    /// Pullen Sinusoidal Ratio — sinusoidal ratio variant.
    PullenSinusoidalRatio,
    /// Carter Poli-Equatorial — equatorial-based house division.
    CarterPoliEquatorial,
    /// APC (Ascendant Parallel Circle) houses.
    APC,
    /// Axial Rotation system (Zariel) — equatorial division similar to Meridian.
    Zariel,
    /// Alcabitius (historical Arabic variant) — uses MC semi-arc instead of ASC semi-arc.
    AlcabitiusClassic,
}

impl HouseSystem {
    /// Return the default house system for Vedic astrology (Whole Sign).
    pub fn vedic_default() -> Self {
        HouseSystem::WholeSign
    }
    /// Return the default house system for Western astrology (Placidus).
    pub fn western_default() -> Self {
        HouseSystem::Placidus
    }
    /// Return the default house system for KP astrology (Placidus).
    pub fn kp_default() -> Self {
        HouseSystem::Placidus
    }

    /// Returns `true` if this system requires geographic latitude.
    pub fn needs_latitude(&self) -> bool {
        !matches!(
            self,
            HouseSystem::Morinus
                | HouseSystem::Equal
                | HouseSystem::Meridian
                | HouseSystem::Zariel
                | HouseSystem::CarterPoliEquatorial
        )
    }

    /// Returns `true` if this system fails at high latitudes.
    pub fn has_polar_limitation(&self) -> bool {
        matches!(
            self,
            HouseSystem::Placidus
                | HouseSystem::Koch
                | HouseSystem::Alcabitius
                | HouseSystem::Topocentric
                | HouseSystem::SunshineMakransky
                | HouseSystem::SunshineTreindl
                | HouseSystem::APC
                | HouseSystem::AlcabitiusClassic
        )
    }

    /// Return the single-character Swiss Ephemeris house system code.
    pub fn swiss_ephem_code(&self) -> char {
        match self {
            HouseSystem::WholeSign => 'W',
            HouseSystem::Equal => 'A',
            HouseSystem::Placidus => 'P',
            HouseSystem::Koch => 'K',
            HouseSystem::Porphyry => 'O',
            HouseSystem::Regiomontanus => 'R',
            HouseSystem::Campanus => 'C',
            HouseSystem::Morinus => 'M',
            HouseSystem::Alcabitius => 'B',
            HouseSystem::Topocentric => 'T',
            HouseSystem::Meridian => 'X',
            HouseSystem::Vehlow => 'V',
            HouseSystem::Sripati => 'S',
            HouseSystem::KrusinskiPisa => 'U',
            HouseSystem::Gauquelin => 'G',
            HouseSystem::SunshineMakransky => 'i',
            HouseSystem::SunshineTreindl => 'I',
            HouseSystem::PullenSinusoidalDelta => 'L',
            HouseSystem::PullenSinusoidalRatio => 'Q',
            HouseSystem::CarterPoliEquatorial => 'F',
            HouseSystem::APC => 'Y',
            HouseSystem::Zariel => 'Z',
            HouseSystem::AlcabitiusClassic => 'b',
        }
    }
}

impl std::fmt::Display for HouseSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HouseSystem::WholeSign => write!(f, "Whole Sign"),
            HouseSystem::Equal => write!(f, "Equal"),
            HouseSystem::Placidus => write!(f, "Placidus"),
            HouseSystem::Koch => write!(f, "Koch"),
            HouseSystem::Porphyry => write!(f, "Porphyry"),
            HouseSystem::Regiomontanus => write!(f, "Regiomontanus"),
            HouseSystem::Campanus => write!(f, "Campanus"),
            HouseSystem::Morinus => write!(f, "Morinus"),
            HouseSystem::Alcabitius => write!(f, "Alcabitius"),
            HouseSystem::Topocentric => write!(f, "Topocentric (Polich-Page)"),
            HouseSystem::Meridian => write!(f, "Meridian"),
            HouseSystem::Vehlow => write!(f, "Vehlow Equal"),
            HouseSystem::Sripati => write!(f, "Sripati"),
            HouseSystem::KrusinskiPisa => write!(f, "Krusinski-Pisa"),
            HouseSystem::Gauquelin => write!(f, "Gauquelin Sectors"),
            HouseSystem::SunshineMakransky => write!(f, "Sunshine (Makransky)"),
            HouseSystem::SunshineTreindl => write!(f, "Sunshine (Treindl)"),
            HouseSystem::PullenSinusoidalDelta => write!(f, "Pullen Sinusoidal Delta"),
            HouseSystem::PullenSinusoidalRatio => write!(f, "Pullen Sinusoidal Ratio"),
            HouseSystem::CarterPoliEquatorial => write!(f, "Carter Poli-Equatorial"),
            HouseSystem::APC => write!(f, "APC (Ascendant Parallel Circle)"),
            HouseSystem::Zariel => write!(f, "Axial Rotation (Zariel)"),
            HouseSystem::AlcabitiusClassic => write!(f, "Alcabitius (Classic)"),
        }
    }
}
