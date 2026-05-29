//! # xalen-chart — SVG Chart Rendering
//!
//! Renders astrological charts as SVG strings. Zero external dependencies beyond
//! the XALEN core crates. No browser, no canvas, no image library needed.
//!
//! ## Supported chart styles
//!
//! - **North Indian** diamond chart (Vedic standard)
//! - **South Indian** box chart
//! - **Western wheel** chart
//!
//! ## Quick start
//!
//! ```rust
//! use xalen_chart::{ChartData, PlanetPosition, render_north_indian};
//!
//! let chart = ChartData {
//!     planet_positions: vec![
//!         PlanetPosition { abbreviation: "Su".into(), house: 1, longitude_deg: 15.0 },
//!         PlanetPosition { abbreviation: "Mo".into(), house: 4, longitude_deg: 105.0 },
//!     ],
//!     house_cusps_deg: [0.0, 30.0, 60.0, 90.0, 120.0, 150.0,
//!                       180.0, 210.0, 240.0, 270.0, 300.0, 330.0],
//!     ascendant_sign_index: 0,
//!     ayanamsa_deg: 23.87,
//! };
//!
//! let svg = render_north_indian(&chart);
//! assert!(svg.starts_with("<svg"));
//! ```

mod north_indian;
mod south_indian;
mod western_wheel;

use serde::{Deserialize, Serialize};

/// A planet's position for chart rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetPosition {
    /// Short label shown in the chart cell (e.g. "Su", "Mo", "Ma", "Ra").
    pub abbreviation: String,
    /// House number 1-12 (the house this planet occupies).
    pub house: usize,
    /// Ecliptic longitude in degrees [0, 360).
    pub longitude_deg: f64,
}

/// All data needed to render any chart style.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    /// Planet positions to place in the chart.
    pub planet_positions: Vec<PlanetPosition>,
    /// Ecliptic longitude of each house cusp in degrees, index 0 = house 1.
    pub house_cusps_deg: [f64; 12],
    /// 0-based sign index of the ascendant (0 = Aries/Mesha, 11 = Pisces/Meena).
    pub ascendant_sign_index: usize,
    /// Ayanamsa applied (displayed as annotation).
    pub ayanamsa_deg: f64,
}

/// Sign abbreviations used in chart labels.
const SIGN_ABBREV: [&str; 12] = [
    "Ar", "Ta", "Ge", "Cn", "Le", "Vi", "Li", "Sc", "Sg", "Cp", "Aq", "Pi",
];

/// Full sign names (available for chart annotations and tooltips).
#[allow(dead_code)]
const SIGN_NAMES: [&str; 12] = [
    "Aries",
    "Taurus",
    "Gemini",
    "Cancer",
    "Leo",
    "Virgo",
    "Libra",
    "Scorpio",
    "Sagittarius",
    "Capricorn",
    "Aquarius",
    "Pisces",
];

/// Render a **North Indian diamond chart** (Vedic standard).
///
/// Returns a complete `<svg>` string. The chart is 400x400 with 12 triangular
/// houses arranged in the classic diamond pattern, planet abbreviations placed
/// inside the correct houses, and sign labels on each house.
pub fn render_north_indian(data: &ChartData) -> String {
    north_indian::render(data)
}

/// Render a **South Indian box chart**.
///
/// Returns a complete `<svg>` string. 4x4 grid of boxes, fixed sign positions
/// (Pisces top-left), planets placed in the box matching their sign.
pub fn render_south_indian(data: &ChartData) -> String {
    south_indian::render(data)
}

/// Render a **Western wheel chart**.
///
/// Returns a complete `<svg>` string. Circular wheel with 12 house segments,
/// zodiac ring, and planet glyphs placed at their ecliptic longitude.
pub fn render_western_wheel(data: &ChartData) -> String {
    western_wheel::render(data)
}

/// Escape a string for safe embedding inside SVG text elements.
///
/// Prevents SVG/XML injection by replacing `<`, `>`, `&`, `"`, and `'`.
fn svg_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Sanitize a longitude value: replace NaN/Inf with 0.0.
fn safe_longitude(deg: f64) -> f64 {
    if deg.is_nan() || deg.is_infinite() {
        0.0
    } else {
        deg
    }
}

/// Collect planet abbreviations per house (1-indexed).
fn planets_in_house(data: &ChartData, house: usize) -> Vec<&str> {
    data.planet_positions
        .iter()
        .filter(|p| p.house == house)
        .map(|p| p.abbreviation.as_str())
        .collect()
}

/// Return the sign abbreviation for a given house, based on the ascendant sign.
fn sign_for_house(asc_sign: usize, house: usize) -> &'static str {
    let idx = (asc_sign + house - 1) % 12;
    SIGN_ABBREV[idx]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_chart() -> ChartData {
        ChartData {
            planet_positions: vec![
                PlanetPosition {
                    abbreviation: "Su".into(),
                    house: 1,
                    longitude_deg: 15.0,
                },
                PlanetPosition {
                    abbreviation: "Mo".into(),
                    house: 4,
                    longitude_deg: 100.0,
                },
                PlanetPosition {
                    abbreviation: "Ma".into(),
                    house: 7,
                    longitude_deg: 195.0,
                },
                PlanetPosition {
                    abbreviation: "Me".into(),
                    house: 1,
                    longitude_deg: 20.0,
                },
                PlanetPosition {
                    abbreviation: "Ju".into(),
                    house: 10,
                    longitude_deg: 280.0,
                },
                PlanetPosition {
                    abbreviation: "Ve".into(),
                    house: 2,
                    longitude_deg: 45.0,
                },
                PlanetPosition {
                    abbreviation: "Sa".into(),
                    house: 5,
                    longitude_deg: 135.0,
                },
                PlanetPosition {
                    abbreviation: "Ra".into(),
                    house: 8,
                    longitude_deg: 220.0,
                },
                PlanetPosition {
                    abbreviation: "Ke".into(),
                    house: 2,
                    longitude_deg: 40.0,
                },
            ],
            house_cusps_deg: [
                0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0, 210.0, 240.0, 270.0, 300.0, 330.0,
            ],
            ascendant_sign_index: 0,
            ayanamsa_deg: 24.17,
        }
    }

    #[test]
    fn north_indian_produces_valid_svg() {
        let svg = render_north_indian(&sample_chart());
        assert!(svg.starts_with("<svg"), "should start with <svg");
        assert!(svg.ends_with("</svg>"), "should end with </svg>");
        assert!(svg.contains("Su"), "should contain Sun");
        assert!(svg.contains("Mo"), "should contain Moon");
        assert!(svg.contains("Ar"), "should contain Aries sign label");
    }

    #[test]
    fn south_indian_produces_valid_svg() {
        let svg = render_south_indian(&sample_chart());
        assert!(svg.starts_with("<svg"), "should start with <svg");
        assert!(svg.ends_with("</svg>"), "should end with </svg>");
        assert!(svg.contains("Su"), "should contain Sun");
        assert!(svg.contains("Pi"), "Pisces sign should appear");
    }

    #[test]
    fn western_wheel_produces_valid_svg() {
        let svg = render_western_wheel(&sample_chart());
        assert!(svg.starts_with("<svg"), "should start with <svg");
        assert!(svg.ends_with("</svg>"), "should end with </svg>");
        assert!(svg.contains("circle"), "should contain circle elements");
        assert!(svg.contains("Su"), "should contain Sun");
    }

    #[test]
    fn planets_in_house_grouping() {
        let chart = sample_chart();
        let h1 = planets_in_house(&chart, 1);
        assert_eq!(h1.len(), 2);
        assert!(h1.contains(&"Su"));
        assert!(h1.contains(&"Me"));
    }

    #[test]
    fn sign_for_house_wraps() {
        assert_eq!(sign_for_house(0, 1), "Ar"); // Aries asc, house 1 = Aries
        assert_eq!(sign_for_house(0, 12), "Pi"); // house 12 = Pisces
        assert_eq!(sign_for_house(3, 1), "Cn"); // Cancer asc, house 1 = Cancer
        assert_eq!(sign_for_house(3, 10), "Ar"); // house 10 from Cancer = Aries
    }

    #[test]
    fn empty_chart_no_panic() {
        let empty = ChartData {
            planet_positions: vec![],
            house_cusps_deg: [0.0; 12],
            ascendant_sign_index: 0,
            ayanamsa_deg: 0.0,
        };
        let svg = render_north_indian(&empty);
        assert!(svg.starts_with("<svg"));
        let svg2 = render_south_indian(&empty);
        assert!(svg2.starts_with("<svg"));
        let svg3 = render_western_wheel(&empty);
        assert!(svg3.starts_with("<svg"));
    }

    #[test]
    fn all_houses_populated() {
        let mut positions = Vec::new();
        for h in 1..=12 {
            positions.push(PlanetPosition {
                abbreviation: format!("P{h}"),
                house: h,
                longitude_deg: (h as f64 - 1.0) * 30.0 + 15.0,
            });
        }
        let chart = ChartData {
            planet_positions: positions,
            house_cusps_deg: [
                0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0, 210.0, 240.0, 270.0, 300.0, 330.0,
            ],
            ascendant_sign_index: 6, // Libra ascendant
            ayanamsa_deg: 24.17,
        };
        let svg = render_north_indian(&chart);
        for h in 1..=12 {
            assert!(
                svg.contains(&format!("P{h}")),
                "house {h} planet missing from SVG"
            );
        }
    }
}
