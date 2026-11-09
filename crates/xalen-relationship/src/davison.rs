//! Davison relationship charts: a real chart cast for the midpoint in time and
//! in space between two births.
//!
//! Method: Ronald Davison, *Synastry* (1977). Unlike a composite chart, the
//! Davison chart is a genuine sky for a genuine moment, so its planets move,
//! retrograde and transit like any natal chart.

use serde::{Deserialize, Serialize};
use xalen_ephem::{Almanac, Body};
use xalen_houses::{GeoLocation, HouseSystem};
use xalen_time::JdUT1;
use xalen_western::midpoints::compute_midpoint;

use crate::error::{RelationshipError, Result};
use crate::natal::{NatalChart, check_location, default_bodies};

/// One birth: a UT1 Julian Day and a place.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BirthMoment {
    /// Julian Day (UT1). Convert civil time first.
    pub jd_ut1: f64,
    /// Birthplace.
    pub location: GeoLocation,
}

/// How the midpoint in space is taken.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SpaceMidpoint {
    /// Mean latitude and shorter-arc mean longitude (Davison's own
    /// arithmetic). Default.
    #[default]
    Arithmetic,
    /// The midpoint on the great circle between the two places (the
    /// normalised sum of the two unit vectors). Two antipodal places have no
    /// unique midpoint and give [`RelationshipError::InvalidInput`].
    GreatCircle,
}

/// Options for [`davison_chart`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DavisonOptions {
    /// Bodies to chart. Default: [`default_bodies`].
    pub bodies: Vec<Body>,
    /// House system. Default: Placidus.
    pub house_system: HouseSystem,
    /// How the midpoint in space is taken. Default: arithmetic.
    pub space: SpaceMidpoint,
}

impl Default for DavisonOptions {
    fn default() -> Self {
        DavisonOptions {
            bodies: default_bodies(),
            house_system: HouseSystem::Placidus,
            space: SpaceMidpoint::Arithmetic,
        }
    }
}

/// The midpoint of two places as `(latitude, longitude)` in degrees, longitude
/// in `[-180, 180)`.
fn space_midpoint(a: &GeoLocation, b: &GeoLocation, how: SpaceMidpoint) -> Result<(f64, f64)> {
    let wrap180 = |x: f64| {
        let w = x.rem_euclid(360.0);
        if w >= 180.0 { w - 360.0 } else { w }
    };
    match how {
        SpaceMidpoint::Arithmetic => {
            let lat = (a.lat_deg() + b.lat_deg()) / 2.0;
            let lon = wrap180(compute_midpoint(
                a.lon_deg().rem_euclid(360.0),
                b.lon_deg().rem_euclid(360.0),
            ));
            Ok((lat, lon))
        }
        SpaceMidpoint::GreatCircle => {
            let unit = |g: &GeoLocation| {
                let (lat, lon) = (g.latitude, g.longitude);
                [lat.cos() * lon.cos(), lat.cos() * lon.sin(), lat.sin()]
            };
            let (ua, ub) = (unit(a), unit(b));
            let s = [ua[0] + ub[0], ua[1] + ub[1], ua[2] + ub[2]];
            let norm = (s[0] * s[0] + s[1] * s[1] + s[2] * s[2]).sqrt();
            if norm < 1e-9 {
                return Err(RelationshipError::InvalidInput(
                    "the two birthplaces are antipodal; the great-circle midpoint is not unique"
                        .to_string(),
                ));
            }
            let lat = (s[2] / norm).clamp(-1.0, 1.0).asin().to_degrees();
            let lon = wrap180(s[1].atan2(s[0]).to_degrees());
            Ok((lat, lon))
        }
    }
}

/// Cast the Davison chart for two births.
///
/// The moment is the mean of the two Julian Days; the place is the midpoint of
/// the birthplaces ([`SpaceMidpoint`]). The result is an ordinary
/// [`NatalChart`], so it can be used with [`crate::synastry`] and
/// [`NatalChart::to_chart_data`].
///
/// Limit: the mean of two UT1 days is taken as is, so births centuries apart
/// give a chart in between with whatever ΔT applies there. Bodies the
/// ephemeris cannot supply at the mean date (Pluto outside 1885-2099) give
/// [`RelationshipError::Ephemeris`].
///
/// ```no_run
/// use xalen_relationship::{BirthMoment, DavisonOptions, davison_chart};
/// use xalen_ephem::Almanac;
/// use xalen_houses::GeoLocation;
///
/// let almanac = Almanac::default_vedic();
/// let a = BirthMoment { jd_ut1: 2_448_000.5, location: GeoLocation::new(10.0, 170.0) };
/// let b = BirthMoment { jd_ut1: 2_449_000.5, location: GeoLocation::new(30.0, -170.0) };
/// let chart = davison_chart(&almanac, &a, &b, &DavisonOptions::default()).unwrap();
/// assert_eq!(chart.jd_ut1, 2_448_500.5);
/// assert!((chart.location.lat_deg() - 20.0).abs() < 1e-9);
/// ```
pub fn davison_chart(
    almanac: &Almanac,
    a: &BirthMoment,
    b: &BirthMoment,
    o: &DavisonOptions,
) -> Result<NatalChart> {
    if !a.jd_ut1.is_finite() || !b.jd_ut1.is_finite() {
        return Err(RelationshipError::InvalidInput(
            "birth Julian Days must be finite".to_string(),
        ));
    }
    check_location(&a.location)?;
    check_location(&b.location)?;
    let mid_jd = (a.jd_ut1 + b.jd_ut1) / 2.0;
    let (lat, lon) = space_midpoint(&a.location, &b.location, o.space)?;
    let mid = GeoLocation::new(lat, lon);
    NatalChart::compute(almanac, JdUT1(mid_jd), &mid, o.house_system, &o.bodies)
}
