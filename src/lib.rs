//! # XALEN Ephemeris
//!
//! Pure-Rust astronomical ephemeris library for astrology applications.
//! Zero C FFI, zero global state, thread-safe via `Arc<T>`.
//!
//! Covers Vedic, Western, Chinese, and world astrology traditions.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use xalen_ephemeris::ephem::{Almanac, Body};
//! use xalen_ephemeris::time::JdUT1;
//!
//! let almanac = Almanac::default_vedic();
//! let jd = JdUT1(2451545.0); // J2000
//! let sun_lon = almanac.geocentric_longitude_deg(Body::Sun, jd).unwrap();
//! ```

/// Ayanamsa (precession correction): 17 systems including Lahiri, KP, Raman.
pub use xalen_ayanamsa as ayanamsa;
/// SVG chart rendering: North Indian, South Indian, Western wheel.
pub use xalen_chart as chart;
/// Chinese astrology: BaZi, Zi Wei Dou Shu, Qi Men Dun Jia, Feng Shui.
pub use xalen_chinese as chinese;
/// Coordinate systems, obliquity, nutation, precession.
pub use xalen_coords as coords;
/// Planetary ephemeris: VSOP87A, ELP2000 Moon, DE440 reader.
pub use xalen_ephem as ephem;
/// House cusp systems: Placidus, Koch, Campanus, Regiomontanus, etc.
pub use xalen_houses as houses;
/// I Ching: 64 hexagrams, Mei Hua Yi Shu.
pub use xalen_iching as iching;
/// Lal Kitab astrology: effects, debts, remedies.
pub use xalen_lalkitab as lalkitab;
/// Pythagorean and Chaldean numerology.
pub use xalen_numerology as numerology;
/// Fixed star catalog with proper motion and nakshatra yogataras.
pub use xalen_stars as stars;
/// Julian Day newtypes, calendar conversion, Delta T models.
pub use xalen_time as time;
/// Vedic astrology: dasha, shadbala, KP, ashtakavarga, compatibility, panchang.
pub use xalen_vedic as vedic;
/// Western astrology: aspects, dignity, lots, Sabian, hellenistic, returns.
pub use xalen_western as western;
/// World traditions: Saju, Nine Star Ki, Mahabote, Mayan, Aztec, Persian.
pub use xalen_world as world;
