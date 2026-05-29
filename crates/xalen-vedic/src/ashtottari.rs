use crate::dasha::{DashaLevel, DashaPeriod};
use crate::nakshatra::DashaLord;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
const ASHTOTTARI_TOTAL: f64 = 108.0;
const YEAR_IN_DAYS: f64 = 365.25;

// Ashtottari uses 8 planets (no Ketu) with different year assignments.
// Years: Sun=6, Moon=15, Mars=8, Mercury=17, Saturn=10, Jupiter=19, Rahu=12, Venus=21 (total=108).
// Advance order after the starting lord: Sun -> Moon -> Mars -> Mercury -> Saturn ->
//   Jupiter -> Rahu -> Venus -> (Sun).
// Source: Brihat Parashara Hora Shastra, Ashtottari (Ardradi) Dasha chapter, vv. 17-20
//   (years given verbatim in v.20). The starting lord is set by the Ardra-anchored
//   4-3-4-3-4-3-4-3 nakshatra grouping, NOT by an Ashwini-anchored equal division.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AshtottariLord {
    Sun,
    Moon,
    Mars,
    Mercury,
    Saturn,
    Jupiter,
    Rahu,
    Venus,
}

impl AshtottariLord {
    pub fn years(&self) -> f64 {
        match self {
            AshtottariLord::Sun => 6.0,
            AshtottariLord::Moon => 15.0,
            AshtottariLord::Mars => 8.0,
            AshtottariLord::Mercury => 17.0,
            AshtottariLord::Saturn => 10.0,
            AshtottariLord::Jupiter => 19.0,
            AshtottariLord::Rahu => 12.0,
            AshtottariLord::Venus => 21.0,
        }
    }

    pub const SEQUENCE: [AshtottariLord; 8] = [
        AshtottariLord::Sun,
        AshtottariLord::Moon,
        AshtottariLord::Mars,
        AshtottariLord::Mercury,
        AshtottariLord::Saturn,
        AshtottariLord::Jupiter,
        AshtottariLord::Rahu,
        AshtottariLord::Venus,
    ];

    fn to_dasha_lord(self) -> DashaLord {
        match self {
            AshtottariLord::Sun => DashaLord::Sun,
            AshtottariLord::Moon => DashaLord::Moon,
            AshtottariLord::Mars => DashaLord::Mars,
            AshtottariLord::Mercury => DashaLord::Mercury,
            AshtottariLord::Saturn => DashaLord::Saturn,
            AshtottariLord::Jupiter => DashaLord::Jupiter,
            AshtottariLord::Rahu => DashaLord::Rahu,
            AshtottariLord::Venus => DashaLord::Venus,
        }
    }
}

/// Starting Mahadasha lord for each of the 27 nakshatras (Ashwini = index 0).
/// Derived from the BPHS Ardradi 4-3-4-3-4-3-4-3 grouping (Abhijit dropped for
/// the 27-nakshatra scheme, shrinking Saturn's block to 3): Sun starts at Ardra.
///   Sun:     Ardra, Punarvasu, Pushya, Ashlesha        (idx 5-8)
///   Moon:    Magha, P.Phalguni, U.Phalguni             (idx 9-11)
///   Mars:    Hasta, Chitra, Swati, Vishakha            (idx 12-15)
///   Mercury: Anuradha, Jyeshtha, Mula                  (idx 16-18)
///   Saturn:  P.Ashadha, U.Ashadha, Shravana            (idx 19-21)
///   Jupiter: Dhanishta, Shatabhisha, P.Bhadrapada      (idx 22-24)
///   Rahu:    U.Bhadrapada, Revati, Ashwini, Bharani    (idx 25,26,0,1)
///   Venus:   Krittika, Rohini, Mrigashira              (idx 2-4)
const STARTING_LORD: [AshtottariLord; 27] = {
    use AshtottariLord::*;
    [
        Rahu, Rahu, // 0 Ashwini, 1 Bharani
        Venus, Venus, Venus, // 2 Krittika, 3 Rohini, 4 Mrigashira
        Sun, Sun, Sun, Sun, // 5 Ardra, 6 Punarvasu, 7 Pushya, 8 Ashlesha
        Moon, Moon, Moon, // 9 Magha, 10 P.Phalguni, 11 U.Phalguni
        Mars, Mars, Mars, Mars, // 12 Hasta, 13 Chitra, 14 Swati, 15 Vishakha
        Mercury, Mercury, Mercury, // 16 Anuradha, 17 Jyeshtha, 18 Mula
        Saturn, Saturn, Saturn, // 19 P.Ashadha, 20 U.Ashadha, 21 Shravana
        Jupiter, Jupiter, Jupiter, // 22 Dhanishta, 23 Shatabhisha, 24 P.Bhadrapada
        Rahu, Rahu, // 25 U.Bhadrapada, 26 Revati
    ]
};

/// Map a sidereal Moon longitude to its Ashtottari starting Mahadasha lord.
fn ashtottari_starting_lord(moon_deg: f64) -> AshtottariLord {
    let nak_span = 360.0 / 27.0; // 13.333° per nakshatra
    let idx = (moon_deg.rem_euclid(360.0) / nak_span) as usize % 27;
    STARTING_LORD[idx]
}

pub fn ashtottari_dasha(moon_sidereal_deg: f64, birth_jd: f64) -> Vec<DashaPeriod> {
    let starting_lord = ashtottari_starting_lord(moon_sidereal_deg);
    let nak_span = 360.0 / 27.0;
    let pos_in_nak = moon_sidereal_deg.rem_euclid(360.0) % nak_span;
    let remaining_fraction = 1.0 - pos_in_nak / nak_span;

    let start_idx = AshtottariLord::SEQUENCE
        .iter()
        .position(|l| *l == starting_lord)
        .expect("AshtottariLord must be in SEQUENCE");

    let mut periods = Vec::new();
    let mut current_jd = birth_jd;

    for cycle in 0..8 {
        let lord_idx = (start_idx + cycle) % 8;
        let lord = AshtottariLord::SEQUENCE[lord_idx];
        let full_years = lord.years();
        let years = if cycle == 0 {
            full_years * remaining_fraction
        } else {
            full_years
        };
        let duration_days = years * YEAR_IN_DAYS;
        let end_jd = current_jd + duration_days;

        periods.push(DashaPeriod {
            lord: lord.to_dasha_lord(),
            start_jd: current_jd,
            end_jd,
            level: DashaLevel::Mahadasha,
            sub_periods: Vec::new(),
        });

        current_jd = end_jd;
    }
    periods
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ashtottari_total_108_from_boundary() {
        let periods = ashtottari_dasha(0.0, 2451545.0);
        assert_eq!(periods.len(), 8);
        let total_years: f64 = periods
            .iter()
            .map(|p| (p.end_jd - p.start_jd) / YEAR_IN_DAYS)
            .sum();
        assert!(
            (total_years - 108.0).abs() < 0.01,
            "Ashtottari total should be 108 years at boundary, got {total_years}"
        );
    }

    #[test]
    fn ashtottari_8_periods() {
        let periods = ashtottari_dasha(50.0, 2451545.0);
        assert_eq!(periods.len(), 8);
    }

    #[test]
    fn ashtottari_balance() {
        let periods = ashtottari_dasha(5.0, 2451545.0);
        let first_years = (periods[0].end_jd - periods[0].start_jd) / YEAR_IN_DAYS;
        // Compare against the ACTUAL starting lord's full span (Moon at 5° is in
        // Ashwini, whose Ashtottari starting lord is Rahu, 12 years).
        let lord_full = ashtottari_starting_lord(5.0).years();
        assert!(
            first_years < lord_full,
            "First (balance) period {first_years} should be < full span {lord_full}"
        );
    }

    #[test]
    fn all_lord_years_sum_108() {
        let total: f64 = AshtottariLord::SEQUENCE.iter().map(|l| l.years()).sum();
        assert_eq!(total, 108.0);
    }

    // Ground-truth starting-lord checks (BPHS Ardradi grouping, verified 2026-05-28).
    fn lord_at(nak_idx: usize) -> AshtottariLord {
        let nak_span = 360.0 / 27.0;
        ashtottari_starting_lord(nak_idx as f64 * nak_span + 1.0)
    }

    #[test]
    fn moon_in_ardra_starts_sun() {
        // Ardra is nakshatra index 5 (Ashwini = 0).
        assert_eq!(lord_at(5), AshtottariLord::Sun);
    }

    #[test]
    fn moon_in_ashwini_starts_rahu() {
        assert_eq!(lord_at(0), AshtottariLord::Rahu);
    }

    #[test]
    fn moon_in_krittika_starts_venus() {
        // Krittika is index 2.
        assert_eq!(lord_at(2), AshtottariLord::Venus);
    }

    #[test]
    fn moon_in_shravana_starts_saturn_not_jupiter() {
        // Shravana is index 21; classical texts place it in Saturn's block.
        assert_eq!(lord_at(21), AshtottariLord::Saturn);
    }

    #[test]
    fn starting_lord_table_block_widths() {
        // 4-3-4-3-3-3-4-3 across the 27-nakshatra scheme (Saturn loses Abhijit).
        let mut counts = std::collections::HashMap::new();
        for &l in STARTING_LORD.iter() {
            *counts.entry(l).or_insert(0) += 1;
        }
        assert_eq!(counts[&AshtottariLord::Sun], 4);
        assert_eq!(counts[&AshtottariLord::Moon], 3);
        assert_eq!(counts[&AshtottariLord::Mars], 4);
        assert_eq!(counts[&AshtottariLord::Mercury], 3);
        assert_eq!(counts[&AshtottariLord::Saturn], 3);
        assert_eq!(counts[&AshtottariLord::Jupiter], 3);
        assert_eq!(counts[&AshtottariLord::Rahu], 4);
        assert_eq!(counts[&AshtottariLord::Venus], 3);
    }
}
