//! A computed natal chart: geocentric tropical body positions plus houses.

use serde::{Deserialize, Serialize};
use xalen_chart::{ChartData, PlanetPosition};
use xalen_coords::{EclipticPosition, EclipticSpeed};
use xalen_ephem::{Almanac, Body};
use xalen_houses::{GeoLocation, HouseCusps, HouseSystem, compute_houses_true};
use xalen_time::JdUT1;

use crate::error::{RelationshipError, Result};

/// One body's position in a chart. All angles are degrees, tropical zodiac,
/// geocentric, referred to the true equinox of date (apparent place).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ChartBody {
    /// Which body.
    pub body: Body,
    /// Ecliptic longitude in `[0, 360)`.
    pub longitude_deg: f64,
    /// Ecliptic latitude, north positive.
    pub latitude_deg: f64,
    /// Daily motion in longitude; negative means retrograde.
    pub speed_deg_per_day: f64,
}

/// A point that can take part in an aspect or a house overlay: a body or one
/// of the four chart angles.
///
/// Ketu is not a [`Body`]; add 180 degrees to the mean node (or use
/// [`Body::ketu_longitude`], which works in radians) if you need it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PointId {
    /// A planet, luminary, node, apogee or Chiron.
    Body(Body),
    /// The Ascendant (cusp of house 1 in most systems).
    Ascendant,
    /// The Midheaven.
    Midheaven,
    /// The Descendant, opposite the Ascendant.
    Descendant,
    /// The Imum Coeli, opposite the Midheaven.
    Imum,
}

impl PointId {
    /// A short label such as `"Su"`, `"Mo"`, `"Asc"`, suitable for chart cells.
    pub fn abbreviation(self) -> &'static str {
        match self {
            PointId::Body(b) => match b {
                Body::Sun => "Su",
                Body::Moon => "Mo",
                Body::Mercury => "Me",
                Body::Venus => "Ve",
                Body::Earth => "Ea",
                Body::Mars => "Ma",
                Body::Jupiter => "Ju",
                Body::Saturn => "Sa",
                Body::Uranus => "Ur",
                Body::Neptune => "Ne",
                Body::Pluto => "Pl",
                Body::MeanNode => "Ra",
                Body::TrueNode => "TN",
                Body::MeanApogee => "Li",
                Body::OsculatingApogee => "OL",
                Body::Chiron => "Ch",
            },
            PointId::Ascendant => "Asc",
            PointId::Midheaven => "MC",
            PointId::Descendant => "Dsc",
            PointId::Imum => "IC",
        }
    }
}

/// The bodies a chart contains when the caller does not choose: the ten
/// luminaries and planets ([`Body::ALL_PLANETS`]) plus the mean node.
/// Chiron and the other points are opt-in.
///
/// ```
/// use xalen_relationship::default_bodies;
/// assert_eq!(default_bodies().len(), 11);
/// ```
pub fn default_bodies() -> Vec<Body> {
    let mut v = Body::ALL_PLANETS.to_vec();
    v.push(Body::MeanNode);
    v
}

/// A natal chart for one moment and place: tropical geocentric positions and
/// house cusps on the true equinox of date.
///
/// The body positions do not depend on the place; only the houses do. That is
/// why a relocated chart ([`NatalChart::relocated`]) is cheap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatalChart {
    /// Julian Day (UT1) of the moment.
    pub jd_ut1: f64,
    /// Place the houses were computed for.
    pub location: GeoLocation,
    /// Body positions, in the order they were requested.
    pub bodies: Vec<ChartBody>,
    /// House cusps and angles, in radians (see [`HouseCusps`]).
    pub houses: HouseCusps,
}

pub(crate) fn check_location(loc: &GeoLocation) -> Result<()> {
    let (lat, lon) = (loc.lat_deg(), loc.lon_deg());
    if !lat.is_finite() || !lon.is_finite() || lat.abs() > 90.0 {
        return Err(RelationshipError::InvalidInput(format!(
            "location must have a finite latitude within 90 degrees (lat {lat}, lon {lon})"
        )));
    }
    Ok(())
}

impl NatalChart {
    /// Compute a chart.
    ///
    /// * `jd_ut1` is a UT1 Julian Day; convert civil time first.
    /// * `bodies` are charted in the order given; a body listed twice is
    ///   charted once. [`Body::Earth`] has no geocentric position and is
    ///   rejected. See [`default_bodies`] for the usual set.
    /// * Houses come from [`compute_houses_true`], so planets and angles share
    ///   one frame. Polar latitudes fall back to Porphyry as in `xalen-houses`.
    ///
    /// Errors: [`RelationshipError::InvalidInput`] for a non-finite epoch,
    /// a bad location or `Body::Earth`; [`RelationshipError::Ephemeris`] when
    /// a position is unavailable (Pluto outside 1885-2099 on the built-in
    /// series, for example).
    ///
    /// ```no_run
    /// use xalen_relationship::{NatalChart, default_bodies};
    /// use xalen_ephem::Almanac;
    /// use xalen_houses::{GeoLocation, HouseSystem};
    /// use xalen_time::JdUT1;
    ///
    /// let almanac = Almanac::default_vedic();
    /// let london = GeoLocation::new(51.5074, -0.1278);
    /// let chart = NatalChart::compute(
    ///     &almanac,
    ///     JdUT1(2_460_482.5),
    ///     &london,
    ///     HouseSystem::Placidus,
    ///     &default_bodies(),
    /// )
    /// .unwrap();
    /// assert_eq!(chart.bodies.len(), 11);
    /// ```
    pub fn compute(
        almanac: &Almanac,
        jd_ut1: JdUT1,
        loc: &GeoLocation,
        system: HouseSystem,
        bodies: &[Body],
    ) -> Result<Self> {
        if !jd_ut1.0.is_finite() {
            return Err(RelationshipError::InvalidInput(
                "Julian Day must be finite".to_string(),
            ));
        }
        check_location(loc)?;
        let mut out: Vec<ChartBody> = Vec::with_capacity(bodies.len());
        for &body in bodies {
            if body == Body::Earth {
                return Err(RelationshipError::InvalidInput(
                    "Earth has no geocentric position".to_string(),
                ));
            }
            if out.iter().any(|c| c.body == body) {
                continue;
            }
            let pos: EclipticPosition = almanac.geocentric_ecliptic(body, jd_ut1)?;
            let speed: EclipticSpeed = almanac.geocentric_speed(body, jd_ut1)?;
            out.push(ChartBody {
                body,
                longitude_deg: pos.longitude.to_degrees().rem_euclid(360.0),
                latitude_deg: pos.latitude.to_degrees(),
                speed_deg_per_day: speed.longitude_deg_per_day(),
            });
        }
        Ok(NatalChart {
            jd_ut1: jd_ut1.0,
            location: *loc,
            bodies: out,
            houses: compute_houses_true(jd_ut1.0, loc, system),
        })
    }

    /// The same moment seen from another place: planets unchanged, houses and
    /// angles recomputed for `loc` (relocation astrology).
    ///
    /// ```no_run
    /// # use xalen_relationship::{NatalChart, default_bodies};
    /// # use xalen_ephem::Almanac;
    /// # use xalen_houses::{GeoLocation, HouseSystem};
    /// # use xalen_time::JdUT1;
    /// # let almanac = Almanac::default_vedic();
    /// # let london = GeoLocation::new(51.5074, -0.1278);
    /// # let natal = NatalChart::compute(&almanac, JdUT1(2_460_482.5), &london,
    /// #     HouseSystem::Placidus, &default_bodies()).unwrap();
    /// let tokyo = GeoLocation::new(35.6762, 139.6503);
    /// let moved = natal.relocated(&tokyo, HouseSystem::Placidus).unwrap();
    /// assert_eq!(moved.bodies, natal.bodies);
    /// assert_ne!(moved.houses.ascendant, natal.houses.ascendant);
    /// ```
    pub fn relocated(&self, loc: &GeoLocation, system: HouseSystem) -> Result<NatalChart> {
        check_location(loc)?;
        Ok(NatalChart {
            jd_ut1: self.jd_ut1,
            location: *loc,
            bodies: self.bodies.clone(),
            houses: compute_houses_true(self.jd_ut1, loc, system),
        })
    }

    /// Longitudes in degrees of the charted bodies, in chart order. With
    /// `include_angles` the Ascendant, Midheaven, Descendant and Imum follow,
    /// in that order.
    pub fn points(&self, include_angles: bool) -> Vec<(PointId, f64)> {
        let mut v: Vec<(PointId, f64)> = self
            .bodies
            .iter()
            .map(|b| (PointId::Body(b.body), b.longitude_deg))
            .collect();
        if include_angles {
            let deg = |r: f64| r.to_degrees().rem_euclid(360.0);
            v.push((PointId::Ascendant, deg(self.houses.ascendant)));
            v.push((PointId::Midheaven, deg(self.houses.mc)));
            v.push((PointId::Descendant, deg(self.houses.descendant)));
            v.push((PointId::Imum, deg(self.houses.ic)));
        }
        v
    }

    /// A render-ready [`ChartData`] for `xalen-chart` (for example
    /// `render_western_wheel`). Tropical, so the ayanamsa is `0.0`.
    ///
    /// Houses are counted with the chart's own cusps. If a longitude is not
    /// finite the body is placed in house 1 rather than failing.
    pub fn to_chart_data(&self) -> ChartData {
        let planet_positions = self
            .bodies
            .iter()
            .map(|b| PlanetPosition {
                abbreviation: PointId::Body(b.body).abbreviation().to_string(),
                house: self.houses.planet_in_house(b.longitude_deg.to_radians()),
                longitude_deg: b.longitude_deg,
            })
            .collect();
        let mut house_cusps_deg = [0.0_f64; 12];
        for (i, c) in house_cusps_deg.iter_mut().enumerate() {
            *c = self.houses.cusp_deg(i);
        }
        let asc = self.houses.ascendant.to_degrees().rem_euclid(360.0);
        ChartData {
            planet_positions,
            house_cusps_deg,
            ascendant_sign_index: ((asc / 30.0).floor() as usize) % 12,
            ayanamsa_deg: 0.0,
        }
    }
}
