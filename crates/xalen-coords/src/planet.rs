//! Canonical `Planet` enum shared across all astrological traditions.
//!
//! This is the **interpretation-layer** planet type.  It is distinct from
//! [`xalen_ephem::Body`](../../xalen-ephem/body/enum.Body.html), which
//! represents ephemeris computation targets (including Earth, Chiron, Lilith,
//! mean vs. true node, etc.).
//!
//! The enum is a **superset**: Vedic modules use `Rahu` / `Ketu`, Western
//! modules use `NorthNode` / `SouthNode` and the outer planets, and classical
//! modules use only the seven traditional bodies.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// A planet used in astrological interpretation.
///
/// Covers all traditions supported by XALEN:
/// - **Classical seven** (Sun..Saturn) -- used everywhere.
/// - **Vedic lunar nodes** (`Rahu`, `Ketu`).
/// - **Western lunar nodes** (`NorthNode`, `SouthNode`) -- aliases for
///   Rahu/Ketu in Hellenistic & Medieval systems.
/// - **Outer planets** (`Uranus`, `Neptune`, `Pluto`) -- modern Western only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Planet {
    Sun,
    Moon,
    Mars,
    Mercury,
    Jupiter,
    Venus,
    Saturn,
    // Vedic lunar nodes
    Rahu,
    Ketu,
    // Western lunar nodes (Hellenistic / Medieval)
    NorthNode,
    SouthNode,
    // Modern outer planets
    Uranus,
    Neptune,
    Pluto,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

impl Planet {
    /// The nine Vedic grahas in traditional order.
    pub const VEDIC_NINE: [Planet; 9] = [
        Planet::Sun,
        Planet::Moon,
        Planet::Mars,
        Planet::Mercury,
        Planet::Jupiter,
        Planet::Venus,
        Planet::Saturn,
        Planet::Rahu,
        Planet::Ketu,
    ];

    /// The seven classical (visible) planets, common to all traditions.
    pub const CLASSICAL_SEVEN: [Planet; 7] = [
        Planet::Sun,
        Planet::Moon,
        Planet::Mars,
        Planet::Mercury,
        Planet::Jupiter,
        Planet::Venus,
        Planet::Saturn,
    ];

    /// All 14 variants.
    pub const ALL: [Planet; 14] = [
        Planet::Sun,
        Planet::Moon,
        Planet::Mars,
        Planet::Mercury,
        Planet::Jupiter,
        Planet::Venus,
        Planet::Saturn,
        Planet::Rahu,
        Planet::Ketu,
        Planet::NorthNode,
        Planet::SouthNode,
        Planet::Uranus,
        Planet::Neptune,
        Planet::Pluto,
    ];
}

// ---------------------------------------------------------------------------
// Classification helpers
// ---------------------------------------------------------------------------

impl Planet {
    /// Is this one of the seven classical (visible) planets?
    pub fn is_classical(self) -> bool {
        matches!(
            self,
            Planet::Sun
                | Planet::Moon
                | Planet::Mars
                | Planet::Mercury
                | Planet::Jupiter
                | Planet::Venus
                | Planet::Saturn
        )
    }

    /// Is this a Vedic lunar node (Rahu or Ketu)?
    pub fn is_vedic_node(self) -> bool {
        matches!(self, Planet::Rahu | Planet::Ketu)
    }

    /// Is this a Western lunar node (North Node or South Node)?
    pub fn is_western_node(self) -> bool {
        matches!(self, Planet::NorthNode | Planet::SouthNode)
    }

    /// Is this any kind of lunar node?
    pub fn is_node(self) -> bool {
        self.is_vedic_node() || self.is_western_node()
    }

    /// Is this one of the three outer (trans-Saturnian) planets?
    pub fn is_outer(self) -> bool {
        matches!(self, Planet::Uranus | Planet::Neptune | Planet::Pluto)
    }

    /// Planet name as a static str.
    pub fn name(self) -> &'static str {
        match self {
            Planet::Sun => "Sun",
            Planet::Moon => "Moon",
            Planet::Mars => "Mars",
            Planet::Mercury => "Mercury",
            Planet::Jupiter => "Jupiter",
            Planet::Venus => "Venus",
            Planet::Saturn => "Saturn",
            Planet::Rahu => "Rahu",
            Planet::Ketu => "Ketu",
            Planet::NorthNode => "North Node",
            Planet::SouthNode => "South Node",
            Planet::Uranus => "Uranus",
            Planet::Neptune => "Neptune",
            Planet::Pluto => "Pluto",
        }
    }
}

// ---------------------------------------------------------------------------
// Display / FromStr
// ---------------------------------------------------------------------------

impl fmt::Display for Planet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Parsing error returned by [`Planet::from_str`].
#[derive(Debug, Clone)]
pub struct ParsePlanetError(String);

impl fmt::Display for ParsePlanetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown planet: '{}'", self.0)
    }
}

impl std::error::Error for ParsePlanetError {}

impl FromStr for Planet {
    type Err = ParsePlanetError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "Sun" | "sun" => Ok(Planet::Sun),
            "Moon" | "moon" => Ok(Planet::Moon),
            "Mars" | "mars" => Ok(Planet::Mars),
            "Mercury" | "mercury" => Ok(Planet::Mercury),
            "Jupiter" | "jupiter" => Ok(Planet::Jupiter),
            "Venus" | "venus" => Ok(Planet::Venus),
            "Saturn" | "saturn" => Ok(Planet::Saturn),
            "Rahu" | "rahu" => Ok(Planet::Rahu),
            "Ketu" | "ketu" => Ok(Planet::Ketu),
            "North Node" | "NorthNode" | "north node" => Ok(Planet::NorthNode),
            "South Node" | "SouthNode" | "south node" => Ok(Planet::SouthNode),
            "Uranus" | "uranus" => Ok(Planet::Uranus),
            "Neptune" | "neptune" => Ok(Planet::Neptune),
            "Pluto" | "pluto" => Ok(Planet::Pluto),
            other => Err(ParsePlanetError(other.to_string())),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_round_trips() {
        for &p in &Planet::ALL {
            let name = p.to_string();
            let parsed: Planet = name.parse().expect(&format!("failed to parse '{name}'"));
            assert_eq!(p, parsed);
        }
    }

    #[test]
    fn classification_helpers() {
        assert!(Planet::Sun.is_classical());
        assert!(Planet::Saturn.is_classical());
        assert!(!Planet::Rahu.is_classical());
        assert!(!Planet::Uranus.is_classical());

        assert!(Planet::Rahu.is_vedic_node());
        assert!(Planet::Ketu.is_vedic_node());
        assert!(!Planet::NorthNode.is_vedic_node());

        assert!(Planet::NorthNode.is_western_node());
        assert!(Planet::SouthNode.is_western_node());

        assert!(Planet::Rahu.is_node());
        assert!(Planet::NorthNode.is_node());
        assert!(!Planet::Sun.is_node());

        assert!(Planet::Uranus.is_outer());
        assert!(Planet::Neptune.is_outer());
        assert!(Planet::Pluto.is_outer());
        assert!(!Planet::Saturn.is_outer());
    }

    #[test]
    fn from_str_variants() {
        assert_eq!("sun".parse::<Planet>().unwrap(), Planet::Sun);
        assert_eq!("North Node".parse::<Planet>().unwrap(), Planet::NorthNode);
        assert_eq!("NorthNode".parse::<Planet>().unwrap(), Planet::NorthNode);
        assert!("unknown".parse::<Planet>().is_err());
    }

    #[test]
    fn vedic_nine_count() {
        assert_eq!(Planet::VEDIC_NINE.len(), 9);
    }

    #[test]
    fn classical_seven_count() {
        assert_eq!(Planet::CLASSICAL_SEVEN.len(), 7);
    }
}
