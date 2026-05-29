use crate::nakshatra::{DashaLord, Nakshatra};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A single dasha period with optional nested sub-periods.
pub struct DashaPeriod {
    pub lord: DashaLord,
    pub start_jd: f64,
    pub end_jd: f64,
    pub level: DashaLevel,
    pub sub_periods: Vec<DashaPeriod>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Hierarchical level within the Vimshottari dasha system.
pub enum DashaLevel {
    Mahadasha,
    Antardasha,
    Pratyantardasha,
    Sookshmadasha,
    Pranadasha,
}

const YEAR_IN_DAYS: f64 = 365.25;

/// Compute the full Vimshottari dasha sequence from Moon longitude at birth.
pub fn vimshottari_dasha(
    moon_sidereal_deg: f64,
    birth_jd: f64,
    depth: DashaLevel,
) -> Vec<DashaPeriod> {
    let nak = Nakshatra::from_longitude_deg(moon_sidereal_deg);
    let starting_lord = nak.lord();

    // Balance: fraction of nakshatra remaining at birth
    let pos_in_nak = moon_sidereal_deg.rem_euclid(360.0) % (360.0 / 27.0);
    let elapsed_fraction = pos_in_nak / (360.0 / 27.0);
    let remaining_fraction = 1.0 - elapsed_fraction;

    let start_idx = DashaLord::SEQUENCE
        .iter()
        .position(|l| *l == starting_lord)
        .expect("DashaLord must be in SEQUENCE");

    let mut periods = Vec::new();
    let mut current_jd = birth_jd;

    for cycle in 0..9 {
        let lord_idx = (start_idx + cycle) % 9;
        let lord = DashaLord::SEQUENCE[lord_idx];
        let full_years = lord.vimshottari_years();

        let years = if cycle == 0 {
            full_years * remaining_fraction
        } else {
            full_years
        };

        let duration_days = years * YEAR_IN_DAYS;
        let end_jd = current_jd + duration_days;

        let sub_periods = if depth > DashaLevel::Mahadasha {
            compute_sub_periods(lord, current_jd, end_jd, DashaLevel::Antardasha, depth)
        } else {
            Vec::new()
        };

        periods.push(DashaPeriod {
            lord,
            start_jd: current_jd,
            end_jd,
            level: DashaLevel::Mahadasha,
            sub_periods,
        });

        current_jd = end_jd;
    }

    periods
}

fn compute_sub_periods(
    parent_lord: DashaLord,
    start_jd: f64,
    end_jd: f64,
    current_level: DashaLevel,
    target_depth: DashaLevel,
) -> Vec<DashaPeriod> {
    let total_duration = end_jd - start_jd;
    let start_idx = DashaLord::SEQUENCE
        .iter()
        .position(|l| *l == parent_lord)
        .expect("DashaLord must be in SEQUENCE");

    let mut sub_periods = Vec::new();
    let mut current_jd = start_jd;

    for cycle in 0..9 {
        let lord_idx = (start_idx + cycle) % 9;
        let lord = DashaLord::SEQUENCE[lord_idx];
        let fraction = lord.vimshottari_years() / DashaLord::TOTAL_YEARS;
        let duration = total_duration * fraction;
        let end = current_jd + duration;

        let children = if target_depth > current_level {
            compute_sub_periods(
                lord,
                current_jd,
                end,
                next_level(current_level),
                target_depth,
            )
        } else {
            Vec::new()
        };

        sub_periods.push(DashaPeriod {
            lord,
            start_jd: current_jd,
            end_jd: end,
            level: current_level,
            sub_periods: children,
        });

        current_jd = end;
    }

    sub_periods
}

fn next_level(current: DashaLevel) -> DashaLevel {
    match current {
        DashaLevel::Mahadasha => DashaLevel::Antardasha,
        DashaLevel::Antardasha => DashaLevel::Pratyantardasha,
        DashaLevel::Pratyantardasha => DashaLevel::Sookshmadasha,
        DashaLevel::Sookshmadasha => DashaLevel::Pranadasha,
        DashaLevel::Pranadasha => DashaLevel::Pranadasha, // terminal
    }
}

impl PartialOrd for DashaLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DashaLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let rank = |l: &DashaLevel| match l {
            DashaLevel::Mahadasha => 0,
            DashaLevel::Antardasha => 1,
            DashaLevel::Pratyantardasha => 2,
            DashaLevel::Sookshmadasha => 3,
            DashaLevel::Pranadasha => 4,
        };
        rank(self).cmp(&rank(other))
    }
}

/// Compute the full 120-year Vimshottari cycle (all 9 mahadashas at full duration).
/// Unlike `vimshottari_dasha` which starts with the balance of the first period,
/// this starts from the given lord with full durations. Useful for theoretical
/// cycle analysis and education.
pub fn vimshottari_full_cycle(
    starting_lord: DashaLord,
    start_jd: f64,
    depth: DashaLevel,
) -> Vec<DashaPeriod> {
    let start_idx = DashaLord::SEQUENCE
        .iter()
        .position(|l| *l == starting_lord)
        .expect("DashaLord must be in SEQUENCE");

    let mut periods = Vec::new();
    let mut current_jd = start_jd;

    for cycle in 0..9 {
        let lord_idx = (start_idx + cycle) % 9;
        let lord = DashaLord::SEQUENCE[lord_idx];
        let years = lord.vimshottari_years();
        let duration_days = years * YEAR_IN_DAYS;
        let end_jd = current_jd + duration_days;

        let sub_periods = if depth > DashaLevel::Mahadasha {
            compute_sub_periods(lord, current_jd, end_jd, DashaLevel::Antardasha, depth)
        } else {
            Vec::new()
        };

        periods.push(DashaPeriod {
            lord,
            level: DashaLevel::Mahadasha,
            start_jd: current_jd,
            end_jd,
            sub_periods,
        });
        current_jd = end_jd;
    }

    periods
}

/// Find the dasha period active at a given Julian Date.
pub fn find_current_dasha(periods: &[DashaPeriod], jd: f64) -> Option<&DashaPeriod> {
    periods.iter().find(|p| jd >= p.start_jd && jd < p.end_jd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vimshottari_covers_120_years_from_boundary() {
        // At exact nakshatra boundary (0°), total = 120 years
        let periods = vimshottari_dasha(0.0, 2451545.0, DashaLevel::Mahadasha);
        assert_eq!(periods.len(), 9);
        let total_days: f64 = periods.iter().map(|p| p.end_jd - p.start_jd).sum();
        let total_years = total_days / YEAR_IN_DAYS;
        assert!(
            (total_years - 120.0).abs() < 0.01,
            "Total should be 120 years at boundary, got {total_years}"
        );
    }

    #[test]
    fn vimshottari_with_balance_less_than_120() {
        // Mid-nakshatra: total < 120 because first dasha uses only balance
        let periods = vimshottari_dasha(100.0, 2451545.0, DashaLevel::Mahadasha);
        assert_eq!(periods.len(), 9);
        let total_days: f64 = periods.iter().map(|p| p.end_jd - p.start_jd).sum();
        let total_years = total_days / YEAR_IN_DAYS;
        assert!(
            total_years < 120.0 && total_years > 100.0,
            "With balance, total should be 100-120 years, got {total_years}"
        );
    }

    #[test]
    fn first_dasha_is_balance() {
        let moon_deg = 5.0; // ~37.5% through Ashwini
        let periods = vimshottari_dasha(moon_deg, 2451545.0, DashaLevel::Mahadasha);
        let first = &periods[0];
        assert_eq!(first.lord, DashaLord::Ketu);
        let first_years = (first.end_jd - first.start_jd) / YEAR_IN_DAYS;
        assert!(
            first_years < 7.0,
            "First dasha should be < 7 years (balance), got {first_years}"
        );
        assert!(
            first_years > 3.0,
            "First dasha should be > 3 years, got {first_years}"
        );
    }

    #[test]
    fn with_antardasha() {
        let periods = vimshottari_dasha(0.0, 2451545.0, DashaLevel::Antardasha);
        let first = &periods[0];
        assert_eq!(first.sub_periods.len(), 9);
        let sub_total: f64 = first
            .sub_periods
            .iter()
            .map(|p| p.end_jd - p.start_jd)
            .sum();
        let maha_total = first.end_jd - first.start_jd;
        assert!(
            (sub_total - maha_total).abs() < 0.01,
            "Sub-periods should sum to mahadasha duration"
        );
    }

    #[test]
    fn find_current() {
        let periods = vimshottari_dasha(0.0, 2451545.0, DashaLevel::Mahadasha);
        let current = find_current_dasha(&periods, 2451545.0 + 365.0);
        assert!(current.is_some());
        assert_eq!(current.unwrap().lord, DashaLord::Ketu);
    }

    #[test]
    fn three_level_deep_dasha() {
        // Requesting Pratyantardasha depth should produce 3 levels:
        // Mahadasha -> Antardasha -> Pratyantardasha
        let periods = vimshottari_dasha(0.0, 2451545.0, DashaLevel::Pratyantardasha);
        assert_eq!(periods.len(), 9);

        let first_maha = &periods[0];
        assert_eq!(first_maha.level, DashaLevel::Mahadasha);
        assert_eq!(
            first_maha.sub_periods.len(),
            9,
            "Mahadasha should have 9 Antardasha sub-periods"
        );

        let first_antar = &first_maha.sub_periods[0];
        assert_eq!(first_antar.level, DashaLevel::Antardasha);
        assert_eq!(
            first_antar.sub_periods.len(),
            9,
            "Antardasha should have 9 Pratyantardasha sub-periods"
        );

        let first_pratyantar = &first_antar.sub_periods[0];
        assert_eq!(first_pratyantar.level, DashaLevel::Pratyantardasha);
        assert!(
            first_pratyantar.sub_periods.is_empty(),
            "Pratyantardasha should have no children at depth=3"
        );

        // Verify sub-period durations sum to parent duration
        let antar_total: f64 = first_maha
            .sub_periods
            .iter()
            .map(|p| p.end_jd - p.start_jd)
            .sum();
        let maha_duration = first_maha.end_jd - first_maha.start_jd;
        assert!(
            (antar_total - maha_duration).abs() < 0.01,
            "Antardasha periods should sum to Mahadasha duration"
        );

        let pratyantar_total: f64 = first_antar
            .sub_periods
            .iter()
            .map(|p| p.end_jd - p.start_jd)
            .sum();
        let antar_duration = first_antar.end_jd - first_antar.start_jd;
        assert!(
            (pratyantar_total - antar_duration).abs() < 0.01,
            "Pratyantardasha periods should sum to Antardasha duration"
        );
    }

    #[test]
    fn full_cycle_always_120_years() {
        for lord in DashaLord::SEQUENCE {
            let periods = vimshottari_full_cycle(lord, 2451545.0, DashaLevel::Mahadasha);
            assert_eq!(periods.len(), 9);
            let total_days: f64 = periods.iter().map(|p| p.end_jd - p.start_jd).sum();
            let total_years = total_days / YEAR_IN_DAYS;
            assert!(
                (total_years - 120.0).abs() < 0.01,
                "Full cycle from {lord:?} should be 120 years, got {total_years}"
            );
        }
    }

    #[test]
    fn full_cycle_starts_with_given_lord() {
        let periods = vimshottari_full_cycle(DashaLord::Venus, 2451545.0, DashaLevel::Mahadasha);
        assert_eq!(periods[0].lord, DashaLord::Venus);
        let years = (periods[0].end_jd - periods[0].start_jd) / YEAR_IN_DAYS;
        assert!(
            (years - 20.0).abs() < 0.01,
            "Venus full dasha should be 20 years, got {years}"
        );
    }

    #[test]
    fn lords_follow_sequence() {
        let periods = vimshottari_dasha(0.0, 2451545.0, DashaLevel::Mahadasha);
        let lords: Vec<DashaLord> = periods.iter().map(|p| p.lord).collect();
        // Starting from Ashwini = Ketu
        assert_eq!(lords[0], DashaLord::Ketu);
        assert_eq!(lords[1], DashaLord::Venus);
        assert_eq!(lords[2], DashaLord::Sun);
    }
}
