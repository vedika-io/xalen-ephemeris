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
    /// Gauquelin sectors — the 36-fold mundane division of statistical
    /// astrology (Swiss Ephemeris code `G`).
    ///
    /// Gauquelin analysis divides the diurnal and nocturnal arcs into 36
    /// sectors counted clockwise from the Ascendant: the semi-diurnal arc and
    /// the semi-nocturnal arc of every point are each split into nine equal
    /// parts. Every third sector boundary coincides with a Placidus house cusp
    /// (sector 1 = Ascendant, sector 10 = Midheaven, sector 19 = Descendant,
    /// sector 28 = Imum Coeli), so the 36 sectors are the Placidus houses
    /// trisected in mundo.
    ///
    /// The 36-sector division cannot be represented by this crate's 12-cusp
    /// `[f64; 12]` array, so the dedicated entry points are
    /// [`crate::gauquelin_sectors`] (the 36 sector-boundary longitudes) and
    /// [`crate::gauquelin_position`] (the continuous `[1, 37)` sector value of
    /// a body). When passed to [`crate::compute_houses`] this variant fills the
    /// 12-cusp array with the twelve every-third sector boundaries — which are
    /// exactly the Placidus cusps — so the conventional `HouseCusps` contract
    /// still returns genuine Gauquelin boundaries.
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

    /// Returns `true` if the computed cusps depend on geographic latitude.
    ///
    /// Only Morinus, Meridian and Zariel (Axial Rotation) are latitude-
    /// independent: their cusps derive purely from RAMC and the obliquity.
    /// EVERY other system — including Equal and Carter Poli-Equatorial — is
    /// anchored on the Ascendant (`compute_ascendant`), which is itself
    /// latitude-dependent, so those cusps move with latitude. Callers that use
    /// this flag for validation, caching, or UI must treat Equal and Carter as
    /// latitude-dependent (their cusp output changes when only latitude changes).
    pub fn needs_latitude(&self) -> bool {
        !matches!(
            self,
            HouseSystem::Morinus | HouseSystem::Meridian | HouseSystem::Zariel
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

    /// Return the single-character Swiss Ephemeris `hsys` code for this system,
    /// or `None` when no *distinct* Swiss Ephemeris code exists.
    ///
    /// The returned letters are the real Swiss Ephemeris `swe_houses()` `hsys`
    /// codes (per the Swiss Ephemeris programmer's documentation, `swehouse.c`):
    /// `W`=Whole-sign, `A`=Equal, `P`=Placidus, `K`=Koch, `O`=Porphyry,
    /// `R`=Regiomontanus, `C`=Campanus, `M`=Morinus, `B`=Alcabitius,
    /// `T`=Polich-Page ("topocentric"), `X`=Axial-rotation/Meridian,
    /// `V`=Vehlow, `S`=Sripati, `U`=Krusinski-Pisa-Goelzer, `G`=Gauquelin,
    /// `i`=Sunshine (Makransky), `I`=Sunshine (Treindl), `L`=Pullen-SD,
    /// `Q`=Pullen-SR, `F`=Carter Poli-Equatorial, `Y`=APC.
    ///
    /// Two of our systems have NO genuine Swiss code and therefore return
    /// `None` instead of advertising a fabricated letter:
    /// * `Zariel` is our axial-rotation alias — Swiss reuses `X` (Meridian) for
    ///   it, so it has no distinct code (the earlier `'Z'` was invented).
    /// * `AlcabitiusClassic` is a historical MC-arc variant Swiss does not
    ///   expose as a separate `hsys` value (the earlier `'b'` was invented).
    ///
    /// NOTE: the letter only certifies which Swiss *system* a code names — it is
    /// NOT a claim of bit-for-bit Swiss output. `Gauquelin` (`G`) returns the
    /// twelve every-third sector boundaries through the 12-cusp API; its full
    /// 36-sector output is exposed by [`crate::gauquelin_sectors`] and matches
    /// Swiss `swe_house_pos(..., 'G')` to better than 1e-4° (see `cusps.rs`).
    /// Sunshine remains a documented MC/ASC approximation in `cusps.rs`. The
    /// letters here are the correct Swiss identifiers for those systems.
    pub fn swiss_ephem_code(&self) -> Option<char> {
        match self {
            HouseSystem::WholeSign => Some('W'),
            HouseSystem::Equal => Some('A'),
            HouseSystem::Placidus => Some('P'),
            HouseSystem::Koch => Some('K'),
            HouseSystem::Porphyry => Some('O'),
            HouseSystem::Regiomontanus => Some('R'),
            HouseSystem::Campanus => Some('C'),
            HouseSystem::Morinus => Some('M'),
            HouseSystem::Alcabitius => Some('B'),
            HouseSystem::Topocentric => Some('T'),
            HouseSystem::Meridian => Some('X'),
            HouseSystem::Vehlow => Some('V'),
            HouseSystem::Sripati => Some('S'),
            HouseSystem::KrusinskiPisa => Some('U'),
            HouseSystem::Gauquelin => Some('G'),
            HouseSystem::SunshineMakransky => Some('i'),
            HouseSystem::SunshineTreindl => Some('I'),
            HouseSystem::PullenSinusoidalDelta => Some('L'),
            HouseSystem::PullenSinusoidalRatio => Some('Q'),
            HouseSystem::CarterPoliEquatorial => Some('F'),
            HouseSystem::APC => Some('Y'),
            // No distinct Swiss Ephemeris code — see doc comment above.
            HouseSystem::Zariel => None,
            HouseSystem::AlcabitiusClassic => None,
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
