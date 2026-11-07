//! Composite charts: the midpoint chart of two people.
//!
//! Method: Robert Hand, *Planets in Composite* (1975). Each composite body sits
//! at the shorter-arc midpoint of the two natal longitudes. The houses are
//! built from the midpoint of the two Midheavens, either by deriving the
//! Ascendant from that Midheaven at the mean latitude (the default, and the
//! standard "derived" or "Hand" composite) or by taking the midpoint of the two
//! Ascendants.

use serde::{Deserialize, Serialize};
use std::f64::consts::TAU;
use xalen_coords::{mean_obliquity, nutation_2000b};
use xalen_houses::{GeoLocation, HouseCusps, HouseSystem, compute_houses_from_ramc};
use xalen_time::{DeltaTModel, JdUT1, JulianDay};
use xalen_western::midpoints::compute_midpoint;

use crate::error::{RelationshipError, Result};
use crate::natal::{ChartBody, NatalChart};

/// How the composite Ascendant is found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CompositeAscendant {
    /// Midpoint of the two Midheavens, turned into a sidereal time and then
    /// into an Ascendant at the mean latitude of the two birthplaces. ASC and
    /// MC stay geometrically linked, so every house system works.
    #[default]
    DerivedFromMidpointMc,
    /// Midpoint of the two Ascendants, with the Midheaven still the midpoint
    /// of the Midheavens. The two angles are then not linked by the
    /// ecliptic-horizon geometry. Only [`HouseSystem::Equal`] and
    /// [`HouseSystem::WholeSign`] are allowed.
    MidpointOfAscendants,
}

/// Options for [`composite_chart`].
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CompositeOptions {
    /// House system for the composite houses.
    pub house_system: HouseSystem,
    /// How the Ascendant is found.
    pub ascendant: CompositeAscendant,
}

impl Default for CompositeOptions {
    fn default() -> Self {
        CompositeOptions {
            house_system: HouseSystem::Placidus,
            ascendant: CompositeAscendant::DerivedFromMidpointMc,
        }
    }
}

/// A composite chart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeChart {
    /// Midpoint bodies, in the order of chart A, for the bodies both charts
    /// hold. Latitude and speed are arithmetic means.
    pub bodies: Vec<ChartBody>,
    /// Composite houses and angles, in radians (see [`HouseCusps`]).
    pub houses: HouseCusps,
    /// The mean birthplace: mean latitude and shorter-arc mean longitude. It
    /// is the latitude used for the composite Ascendant.
    pub reference_location: GeoLocation,
    /// Mean Julian Day (UT1) of the two births. It sets the obliquity.
    pub mean_jd_ut1: f64,
    /// The Ascendant method that was used.
    pub method: CompositeAscendant,
}

fn wrap180(lon: f64) -> f64 {
    let x = lon.rem_euclid(360.0);
    if x >= 180.0 { x - 360.0 } else { x }
}

/// True obliquity of date (mean obliquity plus nutation in obliquity), radians.
pub(crate) fn true_obliquity(jd_ut1: f64) -> f64 {
    let jd_tt = JdUT1(jd_ut1)
        .to_tt(&DeltaTModel::StephensonMorrisonHohenkerk2016)
        .as_f64();
    let t = (jd_tt - 2_451_545.0) / 36525.0;
    mean_obliquity(t) + nutation_2000b(t).delta_epsilon
}

fn check_chart(c: &NatalChart, which: &str) -> Result<()> {
    let h = &c.houses;
    let angles = [h.ascendant, h.mc, h.ic, h.descendant];
    let bodies_ok = c
        .bodies
        .iter()
        .all(|b| b.longitude_deg.is_finite() && b.latitude_deg.is_finite());
    if !c.jd_ut1.is_finite() || angles.iter().any(|a| !a.is_finite()) || !bodies_ok {
        return Err(RelationshipError::InvalidInput(format!(
            "chart {which} holds a non-finite value"
        )));
    }
    crate::natal::check_location(&c.location)
}

/// Build the composite chart of two natal charts.
///
/// * Bodies: only bodies present in both charts, in chart A's order. If there
///   are none, [`RelationshipError::NoCommonPoints`].
/// * Longitudes: the shorter-arc midpoint
///   ([`xalen_western::midpoints::compute_midpoint`]), so 10 and 350 give 0,
///   not 180. **Limit:** two points exactly 180 degrees apart have two equally
///   good midpoints; `compute_midpoint` picks `lon_a + 90`, so the result
///   depends on the order of the charts ((0, 180) gives 90, (180, 0) gives
///   270).
/// * Houses: see [`CompositeAscendant`]. With the default, the composite MC is
///   the midpoint of the two MCs exactly, and the Ascendant follows from it at
///   the mean latitude with the true obliquity at the mean Julian Day.
///   [`CompositeAscendant::MidpointOfAscendants`] with a system other than
///   Equal or Whole Sign gives [`RelationshipError::UnsupportedHouseSystem`];
///   with it the vertex and auxiliary points remain those of the derived
///   geometry.
///
/// ```no_run
/// use xalen_relationship::{CompositeOptions, NatalChart, composite_chart, default_bodies};
/// use xalen_ephem::Almanac;
/// use xalen_houses::{GeoLocation, HouseSystem};
/// use xalen_time::JdUT1;
///
/// let almanac = Almanac::default_vedic();
/// let make = |jd: f64, lat: f64, lon: f64| {
///     NatalChart::compute(&almanac, JdUT1(jd), &GeoLocation::new(lat, lon),
///         HouseSystem::Placidus, &default_bodies()).unwrap()
/// };
/// let a = make(2_448_000.5, 51.5, -0.1);
/// let b = make(2_449_500.5, 40.7, -74.0);
/// let c = composite_chart(&a, &b, &CompositeOptions::default()).unwrap();
/// assert_eq!(c.bodies.len(), 11);
/// ```
pub fn composite_chart(
    a: &NatalChart,
    b: &NatalChart,
    o: &CompositeOptions,
) -> Result<CompositeChart> {
    check_chart(a, "A")?;
    check_chart(b, "B")?;
    if o.ascendant == CompositeAscendant::MidpointOfAscendants
        && !matches!(o.house_system, HouseSystem::Equal | HouseSystem::WholeSign)
    {
        return Err(RelationshipError::UnsupportedHouseSystem(o.house_system));
    }

    let bodies: Vec<ChartBody> = a
        .bodies
        .iter()
        .filter_map(|ba| {
            let bb = b.bodies.iter().find(|x| x.body == ba.body)?;
            Some(ChartBody {
                body: ba.body,
                longitude_deg: compute_midpoint(ba.longitude_deg, bb.longitude_deg),
                latitude_deg: (ba.latitude_deg + bb.latitude_deg) / 2.0,
                speed_deg_per_day: (ba.speed_deg_per_day + bb.speed_deg_per_day) / 2.0,
            })
        })
        .collect();
    if bodies.is_empty() {
        return Err(RelationshipError::NoCommonPoints);
    }

    let mean_jd = (a.jd_ut1 + b.jd_ut1) / 2.0;
    let eps = true_obliquity(mean_jd);

    let mc_a = a.houses.mc.to_degrees().rem_euclid(360.0);
    let mc_b = b.houses.mc.to_degrees().rem_euclid(360.0);
    let lam_mc = compute_midpoint(mc_a, mc_b).to_radians();
    // Invert MC = atan2(sin RAMC, cos RAMC cos eps) for the RAMC.
    let ramc = (eps.cos() * lam_mc.sin())
        .atan2(lam_mc.cos())
        .rem_euclid(TAU);

    let mean_lat = (a.location.lat_deg() + b.location.lat_deg()) / 2.0;
    let mean_lon = wrap180(compute_midpoint(
        a.location.lon_deg().rem_euclid(360.0),
        b.location.lon_deg().rem_euclid(360.0),
    ));
    let reference_location = GeoLocation::new(mean_lat, mean_lon);

    let mut houses = compute_houses_from_ramc(ramc, &reference_location, eps, o.house_system);

    if o.ascendant == CompositeAscendant::MidpointOfAscendants {
        let asc_a = a.houses.ascendant.to_degrees().rem_euclid(360.0);
        let asc_b = b.houses.ascendant.to_degrees().rem_euclid(360.0);
        let asc = compute_midpoint(asc_a, asc_b);
        let start = match o.house_system {
            HouseSystem::WholeSign => (asc / 30.0).floor() * 30.0,
            _ => asc,
        };
        let mut cusps = [0.0_f64; 12];
        for (i, c) in cusps.iter_mut().enumerate() {
            *c = (start + 30.0 * i as f64).rem_euclid(360.0).to_radians();
        }
        houses.cusps = cusps;
        houses.ascendant = asc.to_radians();
        houses.descendant = (asc + 180.0).rem_euclid(360.0).to_radians();
        houses.fallback_used = false;
    }

    Ok(CompositeChart {
        bodies,
        houses,
        reference_location,
        mean_jd_ut1: mean_jd,
        method: o.ascendant,
    })
}
