mod almanac;
/// Simplified Keplerian ephemeris for the four major asteroids.
pub mod asteroids;
mod body;
mod chiron;
/// Swiss Ephemeris API compatibility layer for drop-in migration.
pub mod compat;
mod de440;
/// Solar and lunar eclipse detection and classification.
pub mod eclipse;
/// Numerical root-finding for sign ingresses, crossings, and planetary stations.
pub mod event_search;
/// Black Moon Lilith (mean lunar apogee) and Priapus.
pub mod lilith;
mod moon;
mod pluto;
mod provider;
/// Mean and True (osculating) lunar node computation (Rahu/Ketu).
pub mod true_node;
mod vsop;

pub use almanac::Almanac;
pub use body::Body;
pub use de440::{
    De440Header, De440Provider, De440Reader, De440Record, IptEntry, NaifId, chebyshev_compute,
    chebyshev_derivative,
};
pub use eclipse::{
    LunarEclipse, LunarEclipseType, SolarEclipse, SolarEclipseType, find_lunar_eclipses,
    find_solar_eclipses,
};
pub use event_search::{EventSearchResult, find_crossing, find_sign_ingress, find_station};
pub use lilith::{mean_lilith, mean_perigee_longitude, priapus};
pub use provider::{EphemerisError, EphemerisProvider};
pub use true_node::{mean_lunar_node, south_node, true_lunar_node};
pub use vsop::Vsop87Provider;

pub use xalen_coords::{CartesianPosition, EclipticPosition, EquatorialPosition};
