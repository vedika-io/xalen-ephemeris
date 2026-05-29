use serde::{Deserialize, Serialize};

const NAKSHATRA_SPAN: f64 = 360.0 / 27.0; // 13.3333... degrees
const PADA_SPAN: f64 = NAKSHATRA_SPAN / 4.0; // 3.3333... degrees

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// The 27 lunar mansions (nakshatras) of Vedic astrology.
pub enum Nakshatra {
    Ashwini = 0,
    Bharani,
    Krittika,
    Rohini,
    Mrigashira,
    Ardra,
    Punarvasu,
    Pushya,
    Ashlesha,
    Magha,
    PurvaPhalguni,
    UttaraPhalguni,
    Hasta,
    Chitra,
    Swati,
    Vishakha,
    Anuradha,
    Jyeshtha,
    Mula,
    PurvaAshadha,
    UttaraAshadha,
    Shravana,
    Dhanishta,
    Shatabhisha,
    PurvaBhadrapada,
    UttaraBhadrapada,
    Revati,
}

impl Nakshatra {
    /// All 27 nakshatras in zodiacal order.
    pub const ALL: [Nakshatra; 27] = [
        Nakshatra::Ashwini,
        Nakshatra::Bharani,
        Nakshatra::Krittika,
        Nakshatra::Rohini,
        Nakshatra::Mrigashira,
        Nakshatra::Ardra,
        Nakshatra::Punarvasu,
        Nakshatra::Pushya,
        Nakshatra::Ashlesha,
        Nakshatra::Magha,
        Nakshatra::PurvaPhalguni,
        Nakshatra::UttaraPhalguni,
        Nakshatra::Hasta,
        Nakshatra::Chitra,
        Nakshatra::Swati,
        Nakshatra::Vishakha,
        Nakshatra::Anuradha,
        Nakshatra::Jyeshtha,
        Nakshatra::Mula,
        Nakshatra::PurvaAshadha,
        Nakshatra::UttaraAshadha,
        Nakshatra::Shravana,
        Nakshatra::Dhanishta,
        Nakshatra::Shatabhisha,
        Nakshatra::PurvaBhadrapada,
        Nakshatra::UttaraBhadrapada,
        Nakshatra::Revati,
    ];

    /// Determine the nakshatra from a sidereal longitude in degrees.
    pub fn from_longitude_deg(sidereal_lon: f64) -> Self {
        let idx = (sidereal_lon.rem_euclid(360.0) / NAKSHATRA_SPAN) as usize;
        Self::ALL[idx % 27]
    }

    /// Return the 0-based index of this nakshatra (0=Ashwini, 26=Revati).
    pub fn index(&self) -> usize {
        *self as usize
    }

    /// Compute the pada (quarter, 1-4) within the nakshatra.
    pub fn pada(sidereal_lon: f64) -> u8 {
        let pos_in_nak = sidereal_lon.rem_euclid(360.0) % NAKSHATRA_SPAN;
        (pos_in_nak / PADA_SPAN) as u8 + 1
    }

    /// Return the Vimshottari dasha lord of this nakshatra.
    pub fn lord(&self) -> DashaLord {
        const LORDS: [DashaLord; 9] = [
            DashaLord::Ketu,
            DashaLord::Venus,
            DashaLord::Sun,
            DashaLord::Moon,
            DashaLord::Mars,
            DashaLord::Rahu,
            DashaLord::Jupiter,
            DashaLord::Saturn,
            DashaLord::Mercury,
        ];
        LORDS[self.index() % 9]
    }

    /// Return the presiding deity's name for this nakshatra.
    pub fn deity(&self) -> &'static str {
        match self {
            Nakshatra::Ashwini => "Ashwini Kumaras",
            Nakshatra::Bharani => "Yama",
            Nakshatra::Krittika => "Agni",
            Nakshatra::Rohini => "Brahma",
            Nakshatra::Mrigashira => "Soma",
            Nakshatra::Ardra => "Rudra",
            Nakshatra::Punarvasu => "Aditi",
            Nakshatra::Pushya => "Brihaspati",
            Nakshatra::Ashlesha => "Sarpa",
            Nakshatra::Magha => "Pitris",
            Nakshatra::PurvaPhalguni => "Bhaga",
            Nakshatra::UttaraPhalguni => "Aryaman",
            Nakshatra::Hasta => "Savitar",
            Nakshatra::Chitra => "Tvashtar",
            Nakshatra::Swati => "Vayu",
            Nakshatra::Vishakha => "Indra-Agni",
            Nakshatra::Anuradha => "Mitra",
            Nakshatra::Jyeshtha => "Indra",
            Nakshatra::Mula => "Nirriti",
            Nakshatra::PurvaAshadha => "Apas",
            Nakshatra::UttaraAshadha => "Vishvadevas",
            Nakshatra::Shravana => "Vishnu",
            Nakshatra::Dhanishta => "Vasu",
            Nakshatra::Shatabhisha => "Varuna",
            Nakshatra::PurvaBhadrapada => "Aja Ekapada",
            Nakshatra::UttaraBhadrapada => "Ahir Budhnya",
            Nakshatra::Revati => "Pushan",
        }
    }

    /// Return the gana (temperament group: Deva, Manushya, or Rakshasa).
    pub fn gana(&self) -> Gana {
        match self {
            Nakshatra::Ashwini
            | Nakshatra::Mrigashira
            | Nakshatra::Punarvasu
            | Nakshatra::Pushya
            | Nakshatra::Hasta
            | Nakshatra::Swati
            | Nakshatra::Anuradha
            | Nakshatra::Shravana
            | Nakshatra::Revati => Gana::Deva,
            Nakshatra::Bharani
            | Nakshatra::Rohini
            | Nakshatra::Ardra
            | Nakshatra::PurvaPhalguni
            | Nakshatra::UttaraPhalguni
            | Nakshatra::PurvaAshadha
            | Nakshatra::UttaraAshadha
            | Nakshatra::PurvaBhadrapada
            | Nakshatra::UttaraBhadrapada => Gana::Manushya,
            _ => Gana::Rakshasa,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// The nine planetary lords of the Vimshottari dasha system.
pub enum DashaLord {
    Ketu,
    Venus,
    Sun,
    Moon,
    Mars,
    Rahu,
    Jupiter,
    Saturn,
    Mercury,
}

impl DashaLord {
    /// Return the number of years this lord rules in Vimshottari dasha.
    pub fn vimshottari_years(&self) -> f64 {
        match self {
            DashaLord::Ketu => 7.0,
            DashaLord::Venus => 20.0,
            DashaLord::Sun => 6.0,
            DashaLord::Moon => 10.0,
            DashaLord::Mars => 7.0,
            DashaLord::Rahu => 18.0,
            DashaLord::Jupiter => 16.0,
            DashaLord::Saturn => 19.0,
            DashaLord::Mercury => 17.0,
        }
    }

    /// The Vimshottari dasha lord sequence starting from Ketu.
    pub const SEQUENCE: [DashaLord; 9] = [
        DashaLord::Ketu,
        DashaLord::Venus,
        DashaLord::Sun,
        DashaLord::Moon,
        DashaLord::Mars,
        DashaLord::Rahu,
        DashaLord::Jupiter,
        DashaLord::Saturn,
        DashaLord::Mercury,
    ];

    /// Total Vimshottari dasha cycle length in years.
    pub const TOTAL_YEARS: f64 = 120.0;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Nakshatra temperament classification.
pub enum Gana {
    Deva,
    Manushya,
    Rakshasa,
}

impl std::fmt::Display for Nakshatra {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Nakshatra::Ashwini => "Ashwini",
                Nakshatra::Bharani => "Bharani",
                Nakshatra::Krittika => "Krittika",
                Nakshatra::Rohini => "Rohini",
                Nakshatra::Mrigashira => "Mrigashira",
                Nakshatra::Ardra => "Ardra",
                Nakshatra::Punarvasu => "Punarvasu",
                Nakshatra::Pushya => "Pushya",
                Nakshatra::Ashlesha => "Ashlesha",
                Nakshatra::Magha => "Magha",
                Nakshatra::PurvaPhalguni => "Purva Phalguni",
                Nakshatra::UttaraPhalguni => "Uttara Phalguni",
                Nakshatra::Hasta => "Hasta",
                Nakshatra::Chitra => "Chitra",
                Nakshatra::Swati => "Swati",
                Nakshatra::Vishakha => "Vishakha",
                Nakshatra::Anuradha => "Anuradha",
                Nakshatra::Jyeshtha => "Jyeshtha",
                Nakshatra::Mula => "Mula",
                Nakshatra::PurvaAshadha => "Purva Ashadha",
                Nakshatra::UttaraAshadha => "Uttara Ashadha",
                Nakshatra::Shravana => "Shravana",
                Nakshatra::Dhanishta => "Dhanishta",
                Nakshatra::Shatabhisha => "Shatabhisha",
                Nakshatra::PurvaBhadrapada => "Purva Bhadrapada",
                Nakshatra::UttaraBhadrapada => "Uttara Bhadrapada",
                Nakshatra::Revati => "Revati",
            }
        )
    }
}

impl std::fmt::Display for DashaLord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DashaLord::Ketu => "Ketu",
                DashaLord::Venus => "Venus",
                DashaLord::Sun => "Sun",
                DashaLord::Moon => "Moon",
                DashaLord::Mars => "Mars",
                DashaLord::Rahu => "Rahu",
                DashaLord::Jupiter => "Jupiter",
                DashaLord::Saturn => "Saturn",
                DashaLord::Mercury => "Mercury",
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nakshatra_boundaries() {
        assert_eq!(Nakshatra::from_longitude_deg(0.0), Nakshatra::Ashwini);
        assert_eq!(Nakshatra::from_longitude_deg(13.33), Nakshatra::Ashwini);
        assert_eq!(Nakshatra::from_longitude_deg(13.34), Nakshatra::Bharani);
        assert_eq!(Nakshatra::from_longitude_deg(359.9), Nakshatra::Revati);
    }

    #[test]
    fn pada_computation() {
        assert_eq!(Nakshatra::pada(0.0), 1);
        assert_eq!(Nakshatra::pada(3.34), 2);
        assert_eq!(Nakshatra::pada(6.67), 3);
        assert_eq!(Nakshatra::pada(10.0), 4);
    }

    #[test]
    fn nakshatra_lords_cycle() {
        assert_eq!(Nakshatra::Ashwini.lord(), DashaLord::Ketu);
        assert_eq!(Nakshatra::Bharani.lord(), DashaLord::Venus);
        assert_eq!(Nakshatra::Krittika.lord(), DashaLord::Sun);
        assert_eq!(Nakshatra::Magha.lord(), DashaLord::Ketu); // 10th = cycle restart
    }

    #[test]
    fn vimshottari_total_120() {
        let total: f64 = DashaLord::SEQUENCE
            .iter()
            .map(|l| l.vimshottari_years())
            .sum();
        assert_eq!(total, 120.0);
    }

    #[test]
    fn all_27_nakshatras() {
        for (i, nak) in Nakshatra::ALL.iter().enumerate() {
            assert_eq!(nak.index(), i);
            let mid = i as f64 * NAKSHATRA_SPAN + NAKSHATRA_SPAN / 2.0;
            assert_eq!(
                Nakshatra::from_longitude_deg(mid),
                *nak,
                "Midpoint {mid}° should map to {nak}"
            );
        }
    }
}
