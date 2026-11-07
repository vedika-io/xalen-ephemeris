//! # xalen-maps — astrocartography lines
//!
//! For a chosen moment, where on Earth is each body on the Midheaven (MC), the
//! Imum Coeli (IC), rising (Ascendant, ASC) or setting (Descendant, DSC)? The
//! answer is four lines per body on a world map, plus the zenith point where the
//! body is directly overhead. This is the method of Jim Lewis,
//! *Astro\*Carto\*Graphy* (1976), computed with the spherical astronomy of
//! Meeus, *Astronomical Algorithms*, 2nd ed., ch. 12 (sidereal time), ch. 13
//! (coordinate transformation) and ch. 15 (rising, transit and setting).
//!
//! ## Conventions
//!
//! * Latitude is north-positive, longitude **east-positive** in `[-180, 180)`.
//!   (Meeus counts longitude west-positive; his signs are not used here.)
//! * A point is a [`LatLon`] tuple `(lat_deg, lon_deg)`.
//! * Right ascension and declination are the apparent place on the true equinox
//!   of date; sidereal time is Greenwich apparent sidereal time (GAST). Both are
//!   built from the same IAU 2000B nutation, so they agree.
//!
//! ## Method
//!
//! With hour angle `H = GST + longitude - RA`:
//!
//! * MC line: `H = 0`, so `longitude = RA - GST`. IC line: `H = 180`.
//! * Rising: `H = -H0`, setting: `H = +H0`, where
//!   `cos H0 = (sin h0 - sin φ sin δ) / (cos φ cos δ)` and `h0` is the horizon
//!   altitude (0 by default). So `longitude = RA ∓ H0 - GST`, one point per
//!   sampled latitude `φ`.
//! * Where `cos H0` falls outside `[-1, 1]` the body never sets or never rises
//!   at that latitude; no point is emitted and the line is broken. The exact
//!   limit-latitude vertex is found by bisection, where the ASC and DSC lines
//!   meet the MC or IC line.
//! * A line that crosses the antimeridian is split into separate segments, with
//!   end points placed on the meridian at ±180 degrees.
//!
//! ## Parans
//!
//! A paran (Brady, *The Eagle and the Lark*) is a place where two bodies are on
//! angles at the same moment: one rising or setting while the other culminates
//! or anti-culminates, or both on the horizon together. On the map these are the
//! crossings of one body's lines with another's. [`parans`] solves every
//! crossing in closed form from the right ascensions, declinations and sidereal
//! time (not from the drawn polylines), and [`parans_near`] gives the parans
//! within an orb of a latitude, such as a birthplace's.
//!
//! ## Example
//!
//! ```no_run
//! use xalen_maps::{astro_map, MapOptions, LineKind};
//! use xalen_ephem::{Almanac, Body};
//! use xalen_time::JdUT1;
//!
//! let almanac = Almanac::default_vedic();
//! let map = astro_map(&almanac, JdUT1(2_460_482.5), &[Body::Sun, Body::Jupiter],
//!                     &MapOptions::default()).unwrap();
//! let sun = &map.bodies[0];
//! let mc = sun.lines.iter().find(|l| l.kind == LineKind::Mc).unwrap();
//! println!("Sun on the MC at longitude {:.2}", mc.segments[0][0].1);
//! println!("Sun overhead at {:?}", sun.zenith);
//! ```
//!
//! ## Limits
//!
//! * Geocentric positions only. Lunar parallax (up to about one degree) is
//!   ignored, so the Moon's lines are accurate to that amount; for the Sun and
//!   planets the parallax is at most a few arc seconds.
//! * No atmospheric refraction unless you set
//!   [`MapOptions::horizon_altitude_deg`] (for example `-0.5667` for the Sun's
//!   centre at the standard horizon).
//! * Latitudes are sampled; the default step is one degree. Parans are exact
//!   and do not depend on the step.

mod lines;
mod map;
mod parans;

use serde::{Deserialize, Serialize};
use xalen_ephem::{Body, EphemerisError};

pub use lines::{horizon_half_arc, horizon_lines, ic_longitude_deg, mc_longitude_deg};
pub use map::astro_map;
pub use parans::{Paran, ParanOptions, ParanSet, SkipReason, SkippedParan, parans, parans_near};

/// A point on Earth: `(latitude_deg, longitude_deg)`, latitude north-positive,
/// longitude east-positive in `[-180, 180)`.
pub type LatLon = (f64, f64);

/// Polylines of a line: each inner vector is one drawable piece.
pub type Segments = Vec<Vec<LatLon>>;

/// Everything that can go wrong. No function panics on caller data.
#[derive(Debug, thiserror::Error)]
pub enum MapError {
    /// A number was not finite or out of range, or the options are inconsistent
    /// (zero step, empty latitude range, too many points).
    #[error("invalid input: {0}")]
    InvalidInput(String),
    /// The ephemeris could not supply a position.
    #[error(transparent)]
    Ephemeris(#[from] EphemerisError),
}

/// Which angle a line marks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LineKind {
    /// The body is on the Midheaven (upper meridian).
    Mc,
    /// The body is on the Imum Coeli (lower meridian).
    Ic,
    /// The body is rising.
    Asc,
    /// The body is setting.
    Dsc,
}

/// How lines are sampled and where the horizon is.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MapOptions {
    /// Southern limit of the lines, degrees, greater than -90. Default `-66`.
    pub lat_min_deg: f64,
    /// Northern limit, degrees, less than 90. Default `66`.
    pub lat_max_deg: f64,
    /// Latitude step for the horizon lines, degrees, greater than zero.
    /// Default `1.0`.
    pub lat_step_deg: f64,
    /// Altitude of the horizon in degrees, between -90 and 90 exclusive.
    /// Default `0.0` (geometric horizon, body centre).
    pub horizon_altitude_deg: f64,
    /// Upper bound on the number of sampled latitudes per line. A request for
    /// more is an error. Default `10_000`.
    pub max_points: usize,
}

impl Default for MapOptions {
    fn default() -> Self {
        MapOptions {
            lat_min_deg: -66.0,
            lat_max_deg: 66.0,
            lat_step_deg: 1.0,
            horizon_altitude_deg: 0.0,
            max_points: 10_000,
        }
    }
}

/// One line of a body. A line is a list of segments; each segment is a polyline
/// of [`LatLon`] points that can be drawn without wrapping or crossing a gap.
/// MC and IC lines are a single two-point segment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapLine {
    /// Which angle.
    pub kind: LineKind,
    /// Polylines, in increasing latitude. Empty when the body never reaches the
    /// angle within the latitude range (a circumpolar body has no ASC or DSC).
    pub segments: Segments,
}

/// The four lines of one body, in the order MC, IC, ASC, DSC.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BodyLines {
    /// The body.
    pub body: Body,
    /// Right ascension in degrees `[0, 360)`.
    pub ra_deg: f64,
    /// Declination in degrees.
    pub dec_deg: f64,
    /// The zenith point: latitude equal to the declination, longitude of the MC
    /// line. The body stands directly overhead here.
    pub zenith: LatLon,
    /// MC, IC, ASC and DSC lines.
    pub lines: Vec<MapLine>,
}

/// An astrocartography map for one moment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AstroMap {
    /// Julian Day (UT1).
    pub jd_ut1: f64,
    /// Greenwich apparent sidereal time in degrees `[0, 360)`.
    pub gast_deg: f64,
    /// One entry per requested body.
    pub bodies: Vec<BodyLines>,
}

/// Whether, and how far, a body crosses the horizon at a latitude.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum HorizonCrossing {
    /// It rises and sets. `half_arc_deg` is the hour angle `H0` in `[0, 180]`
    /// at rising (negative) and setting (positive).
    Crosses {
        /// Semi-diurnal arc in degrees.
        half_arc_deg: f64,
    },
    /// Circumpolar: always above the horizon.
    AlwaysAbove,
    /// Never rises: always below the horizon.
    AlwaysBelow,
}
