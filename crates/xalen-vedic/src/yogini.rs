use crate::dasha::{DashaLevel, DashaPeriod};
use crate::nakshatra::DashaLord;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
const YOGINI_TOTAL: f64 = 36.0;
const YEAR_IN_DAYS: f64 = 365.25;

// Web-verified: Mangala=1(Moon), Pingala=2(Sun), Dhanya=3(Jupiter), Bhramari=4(Mars),
// Bhadrika=5(Mercury), Ulka=6(Saturn), Siddha=7(Venus), Sankata=8(Rahu). Total=36.
// Source: https://www.shreekundli.com/vedic-astrology/dasha/yogini-dasha
// Source: https://astronidan.com/dashas/yogini-dasha
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum YoginiName {
    Mangala,
    Pingala,
    Dhanya,
    Bhramari,
    Bhadrika,
    Ulka,
    Siddha,
    Sankata,
}

impl YoginiName {
    pub fn years(&self) -> f64 {
        match self {
            YoginiName::Mangala => 1.0,
            YoginiName::Pingala => 2.0,
            YoginiName::Dhanya => 3.0,
            YoginiName::Bhramari => 4.0,
            YoginiName::Bhadrika => 5.0,
            YoginiName::Ulka => 6.0,
            YoginiName::Siddha => 7.0,
            YoginiName::Sankata => 8.0,
        }
    }

    pub fn ruling_planet(&self) -> DashaLord {
        match self {
            YoginiName::Mangala => DashaLord::Moon,
            YoginiName::Pingala => DashaLord::Sun,
            YoginiName::Dhanya => DashaLord::Jupiter,
            YoginiName::Bhramari => DashaLord::Mars,
            YoginiName::Bhadrika => DashaLord::Mercury,
            YoginiName::Ulka => DashaLord::Saturn,
            YoginiName::Siddha => DashaLord::Venus,
            YoginiName::Sankata => DashaLord::Rahu,
        }
    }

    pub const SEQUENCE: [YoginiName; 8] = [
        YoginiName::Mangala,
        YoginiName::Pingala,
        YoginiName::Dhanya,
        YoginiName::Bhramari,
        YoginiName::Bhadrika,
        YoginiName::Ulka,
        YoginiName::Siddha,
        YoginiName::Sankata,
    ];
}

pub fn yogini_dasha(moon_sidereal_deg: f64, birth_jd: f64) -> Vec<DashaPeriod> {
    let nak_idx = (moon_sidereal_deg.rem_euclid(360.0) / (360.0 / 27.0)) as usize;
    let start_idx = (nak_idx + 3) % 8; // traditional offset

    let nak_span = 360.0 / 27.0;
    let pos_in_nak = moon_sidereal_deg.rem_euclid(360.0) % nak_span;
    let remaining_fraction = 1.0 - pos_in_nak / nak_span;

    let mut periods = Vec::new();
    let mut current_jd = birth_jd;

    for cycle in 0..8 {
        let lord_idx = (start_idx + cycle) % 8;
        let yogini = YoginiName::SEQUENCE[lord_idx];
        let full_years = yogini.years();
        let years = if cycle == 0 {
            full_years * remaining_fraction
        } else {
            full_years
        };
        let duration_days = years * YEAR_IN_DAYS;
        let end_jd = current_jd + duration_days;

        periods.push(DashaPeriod {
            lord: yogini.ruling_planet(),
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
    fn yogini_total_36_from_boundary() {
        let periods = yogini_dasha(0.0, 2451545.0);
        assert_eq!(periods.len(), 8);
        let total: f64 = periods
            .iter()
            .map(|p| (p.end_jd - p.start_jd) / YEAR_IN_DAYS)
            .sum();
        assert!(
            (total - 36.0).abs() < 0.01,
            "Yogini total should be 36 years, got {total}"
        );
    }

    #[test]
    fn yogini_years_sum() {
        let total: f64 = YoginiName::SEQUENCE.iter().map(|y| y.years()).sum();
        assert_eq!(total, 36.0);
    }

    #[test]
    fn yogini_8_periods() {
        let periods = yogini_dasha(100.0, 2451545.0);
        assert_eq!(periods.len(), 8);
    }

    #[test]
    fn yogini_balance() {
        let periods = yogini_dasha(5.0, 2451545.0);
        let first_years = (periods[0].end_jd - periods[0].start_jd) / YEAR_IN_DAYS;
        assert!(first_years < 8.0, "First period should be balance");
    }
}
