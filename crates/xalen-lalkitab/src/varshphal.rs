//! Lal Kitab Varshphal (annual chart).
//!
//! The annual chart in Lal Kitab differs from Tajaka Varshphal. It is
//! constructed by rotating the birth chart based on the year number and
//! analyzing which planet occupies which house in the derived annual layout.
//! The annual predictions then follow the same house-effect and remedy
//! system as the natal chart.

use crate::effects::{LalKitabEffect, planet_effect};
use crate::houses::{LalKitabChart, Planet};
use serde::{Deserialize, Serialize};

/// An annual chart with year number and derived placements.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Lal Kitab annual (Varshphal) chart for a specific year.
pub struct AnnualChart {
    /// Year number (1-based: 1st year = birth year, 2nd year = age 1, etc.).
    pub year: u32,
    /// The derived annual placements.
    pub chart: LalKitabChart,
}

/// Per-planet annual summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnualPlanetSummary {
    pub planet: Planet,
    /// Natal house (1-12).
    pub natal_house: usize,
    /// Annual house (1-12) after rotation.
    pub annual_house: usize,
    /// Effect of the planet in the annual house.
    pub effect: LalKitabEffect,
    /// Whether the planet changed houses from natal to annual.
    pub house_changed: bool,
}

/// Build a Lal Kitab annual chart for the given `year` (1-based).
///
/// The construction rotates natal placements: each planet's annual house is
/// `((natal_house - 1 + (year - 1)) % 12) + 1`, which cycles through all
/// 12 houses over 12 years. This is the standard Lal Kitab Varshphal
/// rotation method.
pub fn build_annual_chart(natal: &LalKitabChart, year: u32) -> AnnualChart {
    let year = if year < 1 { 1 } else { year };

    let mut annual = LalKitabChart::empty();
    let shift = ((year - 1) % 12) as usize;

    for (idx, planets) in natal.placements.iter().enumerate() {
        let new_house = (idx + shift) % 12; // 0-based index
        for &p in planets {
            annual.placements[new_house].push(p);
        }
    }

    AnnualChart {
        year,
        chart: annual,
    }
}

/// Generate per-planet summaries for the annual chart.
pub fn annual_summary(natal: &LalKitabChart, year: u32) -> Vec<AnnualPlanetSummary> {
    let annual = build_annual_chart(natal, year);
    let mut summaries = Vec::new();

    for planet in Planet::VEDIC_NINE {
        if let Some(natal_house) = natal.house_of(planet) {
            let annual_house = annual
                .chart
                .house_of(planet)
                .expect("planet present in natal must be in annual");
            summaries.push(AnnualPlanetSummary {
                planet,
                natal_house,
                annual_house,
                effect: planet_effect(planet, annual_house),
                house_changed: natal_house != annual_house,
            });
        }
    }

    summaries
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_chart() -> LalKitabChart {
        let mut c = LalKitabChart::empty();
        c.place(Planet::Sun, 1);
        c.place(Planet::Moon, 4);
        c.place(Planet::Mars, 3);
        c.place(Planet::Jupiter, 5);
        c.place(Planet::Venus, 7);
        c.place(Planet::Saturn, 10);
        c.place(Planet::Rahu, 6);
        c.place(Planet::Ketu, 12);
        c.place(Planet::Mercury, 2);
        c
    }

    #[test]
    fn year_1_same_as_natal() {
        let natal = sample_chart();
        let annual = build_annual_chart(&natal, 1);
        for planet in Planet::VEDIC_NINE {
            assert_eq!(
                natal.house_of(planet),
                annual.chart.house_of(planet),
                "Year 1 should match natal for {planet}"
            );
        }
    }

    #[test]
    fn year_13_same_as_natal() {
        // 12-year cycle: year 13 = year 1.
        let natal = sample_chart();
        let annual = build_annual_chart(&natal, 13);
        for planet in Planet::VEDIC_NINE {
            assert_eq!(
                natal.house_of(planet),
                annual.chart.house_of(planet),
                "Year 13 should match natal for {planet}"
            );
        }
    }

    #[test]
    fn year_2_shifts_by_one() {
        let natal = sample_chart();
        let annual = build_annual_chart(&natal, 2);
        // Sun natal house 1 -> annual house 2
        assert_eq!(annual.chart.house_of(Planet::Sun), Some(2));
        // Moon natal house 4 -> annual house 5
        assert_eq!(annual.chart.house_of(Planet::Moon), Some(5));
    }

    #[test]
    fn year_12_wraps_around() {
        let natal = sample_chart();
        let annual = build_annual_chart(&natal, 12);
        // Sun natal house 1, shift 11 -> house 12
        assert_eq!(annual.chart.house_of(Planet::Sun), Some(12));
        // Mercury natal house 2, shift 11 -> house 1 (wraps)
        assert_eq!(annual.chart.house_of(Planet::Mercury), Some(1));
    }

    #[test]
    fn annual_summary_has_all_planets() {
        let natal = sample_chart();
        let sums = annual_summary(&natal, 5);
        assert_eq!(sums.len(), 9); // all 9 planets placed
    }

    #[test]
    fn annual_summary_year_1_no_change() {
        let natal = sample_chart();
        let sums = annual_summary(&natal, 1);
        for s in &sums {
            assert!(!s.house_changed, "{} should not change in year 1", s.planet);
        }
    }

    #[test]
    fn annual_summary_year_2_all_changed() {
        let natal = sample_chart();
        let sums = annual_summary(&natal, 2);
        for s in &sums {
            assert!(s.house_changed, "{} should change in year 2", s.planet);
        }
    }
}
