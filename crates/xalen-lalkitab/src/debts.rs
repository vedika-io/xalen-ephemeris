//! Planetary debt (Rin) detection.
//!
//! Lal Kitab identifies five categories of karmic debt triggered by specific
//! planetary placements. Debts indicate unresolved obligations from past
//! lives and require targeted remedies.

use crate::effects::{LalKitabEffect, planet_effect};
use crate::houses::{LalKitabChart, Planet};
use serde::{Deserialize, Serialize};

/// The five types of karmic debt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// The five categories of planetary debt (Rin) in Lal Kitab.
pub enum DebtKind {
    /// Pitri Rin -- debt to ancestors/father. Triggered by Sun in
    /// houses 2, 6, 8, or 12.
    Ancestral,
    /// Matri Rin -- debt to mother. Triggered by Moon afflicted in
    /// houses 4, 6, 8, or 12.
    Maternal,
    /// Stri Rin -- debt to wife/spouse. Triggered by Venus in
    /// houses 2, 7, 8, or 12 with malefic conjunction.
    Spousal,
    /// Atma Rin -- self debt. Triggered by Jupiter afflicted in
    /// certain houses (3, 6, 7, 10 = troubled positions).
    SelfDebt,
    /// Dharma Rin -- duty/uncle debt. Triggered by Saturn afflicted
    /// in houses 1, 4, 5, or 6.
    Duty,
}

impl std::fmt::Display for DebtKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ancestral => f.write_str("Pitri Rin (Ancestral Debt)"),
            Self::Maternal => f.write_str("Matri Rin (Maternal Debt)"),
            Self::Spousal => f.write_str("Stri Rin (Spousal Debt)"),
            Self::SelfDebt => f.write_str("Atma Rin (Self Debt)"),
            Self::Duty => f.write_str("Dharma Rin (Duty Debt)"),
        }
    }
}

/// A detected planetary debt with context.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// A detected planetary debt with its kind and involved planets.
pub struct PlanetaryDebt {
    pub kind: DebtKind,
    /// The planet whose placement triggered the debt.
    pub trigger_planet: Planet,
    /// The house (1-12) where the trigger planet sits.
    pub house: usize,
    /// Whether malefic conjunction aggravates the debt.
    pub malefic_conjunction: bool,
}

/// Natural malefics in Lal Kitab.
const MALEFICS: [Planet; 4] = [Planet::Saturn, Planet::Mars, Planet::Rahu, Planet::Ketu];

/// Check whether `planet` is conjunct with any malefic in the chart.
fn has_malefic_conjunction(chart: &LalKitabChart, planet: Planet) -> bool {
    if let Some(house) = chart.house_of(planet) {
        let companions = chart.planets_in(house);
        companions
            .iter()
            .any(|&p| p != planet && MALEFICS.contains(&p))
    } else {
        false
    }
}

/// Detect all planetary debts active in the chart.
///
/// Returns a `Vec` of debts; an empty vector means no debts are triggered.
pub fn detect_debts(chart: &LalKitabChart) -> Vec<PlanetaryDebt> {
    let mut debts = Vec::new();

    // --- Pitri Rin (Ancestral) --- Sun in 2/6/8/12
    if let Some(h) = chart.house_of(Planet::Sun)
        && [2, 6, 8, 12].contains(&h) {
            debts.push(PlanetaryDebt {
                kind: DebtKind::Ancestral,
                trigger_planet: Planet::Sun,
                house: h,
                malefic_conjunction: has_malefic_conjunction(chart, Planet::Sun),
            });
        }

    // --- Matri Rin (Maternal) --- Moon afflicted in 4/6/8/12
    if let Some(h) = chart.house_of(Planet::Moon)
        && [4, 6, 8, 12].contains(&h) {
            let afflicted = planet_effect(Planet::Moon, h) == LalKitabEffect::Troubled
                || has_malefic_conjunction(chart, Planet::Moon);
            if afflicted {
                debts.push(PlanetaryDebt {
                    kind: DebtKind::Maternal,
                    trigger_planet: Planet::Moon,
                    house: h,
                    malefic_conjunction: has_malefic_conjunction(chart, Planet::Moon),
                });
            }
        }

    // --- Stri Rin (Spousal) --- Venus in 2/7/8/12 with malefics
    if let Some(h) = chart.house_of(Planet::Venus)
        && [2, 7, 8, 12].contains(&h) && has_malefic_conjunction(chart, Planet::Venus) {
            debts.push(PlanetaryDebt {
                kind: DebtKind::Spousal,
                trigger_planet: Planet::Venus,
                house: h,
                malefic_conjunction: true,
            });
        }

    // --- Atma Rin (Self) --- Jupiter in troubled houses (3/6/7/10)
    if let Some(h) = chart.house_of(Planet::Jupiter)
        && planet_effect(Planet::Jupiter, h) == LalKitabEffect::Troubled {
            debts.push(PlanetaryDebt {
                kind: DebtKind::SelfDebt,
                trigger_planet: Planet::Jupiter,
                house: h,
                malefic_conjunction: has_malefic_conjunction(chart, Planet::Jupiter),
            });
        }

    // --- Dharma Rin (Duty) --- Saturn in troubled houses (1/4/5/6)
    if let Some(h) = chart.house_of(Planet::Saturn)
        && planet_effect(Planet::Saturn, h) == LalKitabEffect::Troubled {
            debts.push(PlanetaryDebt {
                kind: DebtKind::Duty,
                trigger_planet: Planet::Saturn,
                house: h,
                malefic_conjunction: has_malefic_conjunction(chart, Planet::Saturn),
            });
        }

    debts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chart_with(placements: &[(Planet, usize)]) -> LalKitabChart {
        let mut c = LalKitabChart::empty();
        for &(p, h) in placements {
            c.place(p, h);
        }
        c
    }

    #[test]
    fn ancestral_debt_sun_in_6() {
        let c = chart_with(&[(Planet::Sun, 6)]);
        let debts = detect_debts(&c);
        assert_eq!(debts.len(), 1);
        assert_eq!(debts[0].kind, DebtKind::Ancestral);
        assert_eq!(debts[0].house, 6);
    }

    #[test]
    fn ancestral_debt_sun_in_1_no_debt() {
        let c = chart_with(&[(Planet::Sun, 1)]);
        let debts = detect_debts(&c);
        assert!(debts.iter().all(|d| d.kind != DebtKind::Ancestral));
    }

    #[test]
    fn maternal_debt_moon_in_8_with_malefic() {
        let c = chart_with(&[(Planet::Moon, 8), (Planet::Saturn, 8)]);
        let debts = detect_debts(&c);
        assert!(debts.iter().any(|d| d.kind == DebtKind::Maternal));
    }

    #[test]
    fn spousal_debt_venus_7_with_mars() {
        let c = chart_with(&[(Planet::Venus, 7), (Planet::Mars, 7)]);
        let debts = detect_debts(&c);
        assert!(debts.iter().any(|d| d.kind == DebtKind::Spousal));
    }

    #[test]
    fn spousal_no_debt_venus_7_alone() {
        let c = chart_with(&[(Planet::Venus, 7)]);
        let debts = detect_debts(&c);
        assert!(debts.iter().all(|d| d.kind != DebtKind::Spousal));
    }

    #[test]
    fn self_debt_jupiter_in_6() {
        let c = chart_with(&[(Planet::Jupiter, 6)]);
        let debts = detect_debts(&c);
        assert!(debts.iter().any(|d| d.kind == DebtKind::SelfDebt));
    }

    #[test]
    fn duty_debt_saturn_in_1() {
        let c = chart_with(&[(Planet::Saturn, 1)]);
        let debts = detect_debts(&c);
        assert!(debts.iter().any(|d| d.kind == DebtKind::Duty));
    }

    #[test]
    fn no_debts_clean_chart() {
        // Planets in generally good houses, no malefic conjunctions.
        let c = chart_with(&[
            (Planet::Sun, 1),
            (Planet::Moon, 2),
            (Planet::Jupiter, 5),
            (Planet::Venus, 3),
            (Planet::Saturn, 10),
        ]);
        let debts = detect_debts(&c);
        assert!(debts.is_empty(), "expected no debts, got {debts:?}");
    }

    #[test]
    fn multiple_debts() {
        let c = chart_with(&[
            (Planet::Sun, 12),
            (Planet::Jupiter, 10),
            (Planet::Saturn, 5),
        ]);
        let debts = detect_debts(&c);
        assert!(
            debts.len() >= 2,
            "expected multiple debts, got {}",
            debts.len()
        );
        let kinds: Vec<_> = debts.iter().map(|d| d.kind).collect();
        assert!(kinds.contains(&DebtKind::Ancestral));
        // Jupiter in 10 is Troubled -> SelfDebt, Saturn in 5 is Troubled -> Duty
    }
}
