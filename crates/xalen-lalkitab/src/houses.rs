//! House-based chart representation and natural house rulers.
//!
//! In Lal Kitab the twelve houses have fixed natural rulers that never change,
//! regardless of the ascendant sign. A planet's results depend primarily on
//! which house it occupies, not which sign it is in.

use serde::{Deserialize, Serialize};

pub use xalen_coords::Planet;

/// A chart in the Lal Kitab system.
///
/// Each house (1-12) holds zero or more planets. The chart is constructed
/// from a standard horoscope but interpreted through Lal Kitab rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// A Lal Kitab chart with planet placements by house.
pub struct LalKitabChart {
    /// `placements[i]` = planets occupying house `i+1` (index 0 = house 1).
    pub placements: [Vec<Planet>; 12],
}

impl LalKitabChart {
    /// Create an empty chart (no planets placed yet).
    pub fn empty() -> Self {
        Self {
            placements: std::array::from_fn(|_| Vec::new()),
        }
    }

    /// Place a planet in a house (1-12).
    ///
    /// **Clamping:** Out-of-range house values are silently clamped to the
    /// nearest valid house (0 -> house 1, 13+ -> house 12). This defensive
    /// behavior prevents panics from malformed input while keeping the planet
    /// in the chart. Callers that need strict validation should check the
    /// house range before calling this method.
    pub fn place(&mut self, planet: Planet, house: usize) {
        let house = house.clamp(1, 12);
        self.placements[house - 1].push(planet);
    }

    /// Return the house (1-12) where `planet` sits, or `None`.
    pub fn house_of(&self, planet: Planet) -> Option<usize> {
        self.placements
            .iter()
            .position(|h| h.contains(&planet))
            .map(|i| i + 1)
    }

    /// Return planets in the given house (1-12).
    ///
    /// **Clamping:** Out-of-range house values are silently clamped (same as
    /// [`place`](Self::place)).
    pub fn planets_in(&self, house: usize) -> &[Planet] {
        let house = house.clamp(1, 12);
        &self.placements[house - 1]
    }
}

/// Natural house rulers per Lal Kitab.
///
/// Returns the primary natural ruler(s) for a house (1-12).
/// Some houses have two co-rulers (e.g. house 6 = Mercury + Ketu).
///
/// **Clamping:** Out-of-range house values are silently clamped to 1-12,
/// same defensive behavior as [`LalKitabChart::place`].
pub fn natural_rulers(house: usize) -> &'static [Planet] {
    let house = house.clamp(1, 12);
    match house {
        1 => &[Planet::Sun],
        2 => &[Planet::Jupiter],
        3 => &[Planet::Mars],
        4 => &[Planet::Moon],
        5 => &[Planet::Jupiter],
        6 => &[Planet::Mercury, Planet::Ketu],
        7 => &[Planet::Venus],
        8 => &[Planet::Mars, Planet::Saturn],
        9 => &[Planet::Jupiter],
        10 => &[Planet::Saturn],
        11 => &[Planet::Saturn],
        12 => &[Planet::Jupiter, Planet::Ketu],
        _ => unreachable!("clamp guarantees 1-12"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn place_and_find() {
        let mut c = LalKitabChart::empty();
        c.place(Planet::Sun, 1);
        c.place(Planet::Moon, 4);
        assert_eq!(c.house_of(Planet::Sun), Some(1));
        assert_eq!(c.house_of(Planet::Moon), Some(4));
        assert_eq!(c.house_of(Planet::Mars), None);
    }

    #[test]
    fn planets_in_house() {
        let mut c = LalKitabChart::empty();
        c.place(Planet::Mars, 3);
        c.place(Planet::Rahu, 3);
        let ps = c.planets_in(3);
        assert_eq!(ps.len(), 2);
        assert!(ps.contains(&Planet::Mars));
        assert!(ps.contains(&Planet::Rahu));
    }

    #[test]
    fn natural_rulers_check() {
        assert_eq!(natural_rulers(1), &[Planet::Sun]);
        assert_eq!(natural_rulers(6), &[Planet::Mercury, Planet::Ketu]);
        assert_eq!(natural_rulers(8), &[Planet::Mars, Planet::Saturn]);
        assert_eq!(natural_rulers(12), &[Planet::Jupiter, Planet::Ketu]);
    }

    #[test]
    fn house_out_of_range_clamps() {
        // Out-of-range house 0 clamps to 1 (Sun)
        assert_eq!(natural_rulers(0), &[Planet::Sun]);
        // Out-of-range house 13 clamps to 12
        assert_eq!(natural_rulers(13), &[Planet::Jupiter, Planet::Ketu]);
    }
}
