//! # xalen-relationship — synastry, composite and Davison charts
//!
//! Relationship astrology on top of the XALEN ephemeris:
//!
//! * [`NatalChart`]: tropical geocentric positions plus house cusps on the true
//!   equinox of date, with a [relocation helper](NatalChart::relocated) and a
//!   converter to the `xalen-chart` render type.
//! * [`synastry`]: the aspect grid between two charts and the house overlays
//!   (Hand, *Horoscope Symbols*, 1981).
//! * [`composite_chart`]: the midpoint chart (Hand, *Planets in Composite*,
//!   1975), with the Ascendant derived from the midpoint Midheaven or taken as
//!   the midpoint of the Ascendants.
//! * [`davison_chart`]: a real chart for the midpoint in time and space
//!   (Davison, *Synastry*, 1977).
//!
//! All longitudes are degrees, east-positive on the ecliptic and for places.
//! The house cusps inside [`HouseCusps`](xalen_houses::HouseCusps) are radians,
//! as everywhere in `xalen-houses`.
//!
//! No function panics on caller data; failures are [`RelationshipError`].
//!
//! ## Example
//!
//! ```no_run
//! use xalen_relationship::{
//!     NatalChart, SynastryOptions, composite_chart, CompositeOptions, default_bodies, synastry,
//! };
//! use xalen_ephem::Almanac;
//! use xalen_houses::{GeoLocation, HouseSystem};
//! use xalen_time::JdUT1;
//!
//! let almanac = Almanac::default_vedic();
//! let chart = |jd: f64, lat: f64, lon: f64| {
//!     NatalChart::compute(
//!         &almanac,
//!         JdUT1(jd),
//!         &GeoLocation::new(lat, lon),
//!         HouseSystem::Placidus,
//!         &default_bodies(),
//!     )
//!     .unwrap()
//! };
//! let a = chart(2_448_000.5, 51.5, -0.1);
//! let b = chart(2_449_500.5, 40.7, -74.0);
//! let syn = synastry(&a, &b, &SynastryOptions::default()).unwrap();
//! let comp = composite_chart(&a, &b, &CompositeOptions::default()).unwrap();
//! println!("{} aspects, composite MC {:.2}", syn.aspects().count(), comp.houses.mc.to_degrees());
//! ```
//!
//! ## Limits
//!
//! Positions are geocentric (no topocentric parallax, which matters only for the
//! Moon), the analytic Pluto series covers 1885-2099, and a composite of two
//! points exactly 180 degrees apart depends on the order of the charts.

mod composite;
mod davison;
mod error;
mod natal;
mod synastry;

pub use composite::{CompositeAscendant, CompositeChart, CompositeOptions, composite_chart};
pub use davison::{BirthMoment, DavisonOptions, SpaceMidpoint, davison_chart};
pub use error::{RelationshipError, Result};
pub use natal::{ChartBody, NatalChart, PointId, default_bodies};
pub use synastry::{
    HouseOverlay, Synastry, SynastryAspect, SynastryOptions, house_overlay, inter_aspects, synastry,
};
