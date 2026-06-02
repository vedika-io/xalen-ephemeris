mod almanac;
/// Simplified Keplerian ephemeris for the four major asteroids.
pub mod asteroids;
/// Besselian-elements solar-eclipse geometry (global classification).
pub mod besselian;
mod body;
mod chiron;
/// Swiss Ephemeris API compatibility layer for drop-in migration.
pub mod compat;
mod de440;
/// Solar and lunar eclipse detection and classification.
pub mod eclipse;
/// Numerical root-finding for sign ingresses, crossings, and planetary stations.
pub mod event_search;
/// Optional automatic provisioning of the public NASA NAIF `de440s.bsp` kernel.
/// Compiled only with the `kernel-autodownload` feature.
#[cfg(feature = "kernel-autodownload")]
pub mod kernel_cache;
/// Black Moon Lilith (mean lunar apogee), True (osculating) Lilith, and Priapus.
pub mod lilith;
/// Local (per-observer) solar-eclipse circumstances on the Besselian engine.
pub mod local_eclipse;
mod moon;
/// Alternative output frames / coordinate types for the [`Almanac`]
/// (equatorial RA/Dec, heliocentric, rectangular XYZ — Swiss SEFLG_* parity).
mod output;
mod pluto;
mod provider;
/// Exact return finders (Solar/Lunar/Saturn/Jupiter/Mars) on the real almanac.
pub mod returns;
/// Rise, transit (culmination), and set times for a body and observer.
pub mod rise_set;
/// Topocentric (observer-centered) position correction via diurnal parallax.
pub mod topocentric;
/// Mean and True (osculating) lunar node computation (Rahu/Ketu).
pub mod true_node;
mod vsop;

pub use almanac::Almanac;
pub use body::Body;
pub use de440::{
    AccuracyTier, De440Header, De440Provider, De440Reader, De440Record, IptEntry, NaifId,
    chebyshev_compute, chebyshev_derivative,
};
pub use eclipse::{
    LunarEclipse, LunarEclipseType, SolarEclipse, SolarEclipseType, find_lunar_eclipses,
    find_solar_eclipses,
};
pub use event_search::{EventSearchResult, find_crossing, find_sign_ingress, find_station};
#[cfg(feature = "kernel-autodownload")]
pub use kernel_cache::{
    CACHE_DIR_ENV, DE440S_FILENAME, DE440S_URL, KernelFetch, SHA256_ENV, ensure_de440s_kernel,
};
pub use lilith::{mean_lilith, mean_perigee_longitude, priapus, true_lilith};
pub use local_eclipse::{
    Contact, LocalCircumstances, LocalSolarType, contact_ut1, local_circumstances,
};
pub use provider::{EphemerisError, EphemerisProvider};
pub use returns::{ReturnBody, find_return, find_return_tt};
pub use rise_set::{RiseTransitSet, Twilight, TwilightTimes};
pub use true_node::{mean_lunar_node, south_node, true_lunar_node};
pub use vsop::Vsop87Provider;

pub use xalen_coords::{CartesianPosition, EclipticPosition, EquatorialPosition};
