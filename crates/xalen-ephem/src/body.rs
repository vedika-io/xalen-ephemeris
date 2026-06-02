use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// A celestial body for which ephemeris positions can be computed.
pub enum Body {
    Sun,
    Moon,
    Mercury,
    Venus,
    Earth,
    Mars,
    Jupiter,
    Saturn,
    Uranus,
    Neptune,
    Pluto,
    /// Rahu — mean ascending node.
    MeanNode,
    /// Rahu — true (osculating) ascending node.
    TrueNode,
    /// Lilith — mean lunar apogee (Swiss `SE_MEAN_APOG`, 12).
    MeanApogee,
    /// Lilith — true (osculating) lunar apogee (Swiss `SE_OSCU_APOG`, 13).
    OsculatingApogee,
    Chiron,
}

impl Body {
    /// The eight computable Vedic grahas (Ketu = Rahu + 180°, compute separately).
    pub const VEDIC_GRAHAS: &[Body] = &[
        Body::Sun,
        Body::Moon,
        Body::Mars,
        Body::Mercury,
        Body::Jupiter,
        Body::Venus,
        Body::Saturn,
        Body::MeanNode, // Rahu — Ketu derived as MeanNode + 180°
    ];

    /// True if this body is Rahu (mean node), whose opposite gives Ketu.
    pub fn is_rahu(&self) -> bool {
        matches!(self, Body::MeanNode)
    }

    /// All ten major solar system bodies (luminaries + planets).
    pub const ALL_PLANETS: &[Body] = &[
        Body::Sun,
        Body::Moon,
        Body::Mercury,
        Body::Venus,
        Body::Mars,
        Body::Jupiter,
        Body::Saturn,
        Body::Uranus,
        Body::Neptune,
        Body::Pluto,
    ];

    /// Returns `true` if this body is a planet (Mercury through Pluto).
    pub fn is_planet(&self) -> bool {
        matches!(
            self,
            Body::Mercury
                | Body::Venus
                | Body::Mars
                | Body::Jupiter
                | Body::Saturn
                | Body::Uranus
                | Body::Neptune
                | Body::Pluto
        )
    }

    /// Returns `true` if this body is a luminary (Sun or Moon).
    pub fn is_luminary(&self) -> bool {
        matches!(self, Body::Sun | Body::Moon)
    }

    /// Compute Ketu's longitude as the point opposite Rahu (+ 180 degrees).
    pub fn ketu_longitude(rahu_longitude: f64) -> f64 {
        (rahu_longitude + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU)
    }
}

impl std::fmt::Display for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Body::Sun => write!(f, "Sun"),
            Body::Moon => write!(f, "Moon"),
            Body::Mercury => write!(f, "Mercury"),
            Body::Venus => write!(f, "Venus"),
            Body::Earth => write!(f, "Earth"),
            Body::Mars => write!(f, "Mars"),
            Body::Jupiter => write!(f, "Jupiter"),
            Body::Saturn => write!(f, "Saturn"),
            Body::Uranus => write!(f, "Uranus"),
            Body::Neptune => write!(f, "Neptune"),
            Body::Pluto => write!(f, "Pluto"),
            Body::MeanNode => write!(f, "Rahu (Mean Node)"),
            Body::TrueNode => write!(f, "Rahu (True Node)"),
            Body::MeanApogee => write!(f, "Lilith (Mean Apogee)"),
            Body::OsculatingApogee => write!(f, "Lilith (Osculating Apogee)"),
            Body::Chiron => write!(f, "Chiron"),
        }
    }
}
