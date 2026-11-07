//! Synastry: the aspects between two charts and where each chart's points
//! fall in the other's houses.
//!
//! Method: Hand, *Horoscope Symbols* (1981), ch. on synastry. Every point of
//! chart A is compared with every point of chart B, which is the grid
//! (aspectarian) layout used in synastry tables. Orbs come from
//! [`AspectType::default_orb_deg`], scaled by a multiplier.

use serde::{Deserialize, Serialize};
use xalen_houses::HouseCusps;
use xalen_western::aspects::{AspectType, angular_distance, find_aspect};

use crate::error::{RelationshipError, Result};
use crate::natal::{NatalChart, PointId};

/// What [`synastry`] should look for.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynastryOptions {
    /// Aspect types to test. Default: the five major aspects.
    pub aspects: Vec<AspectType>,
    /// Multiplies every default orb. Must be finite and greater than zero.
    /// Default `1.0`.
    pub orb_multiplier: f64,
    /// Also test the Ascendant, Midheaven, Descendant and Imum. Default
    /// `false` (bodies only).
    pub include_angles: bool,
}

impl Default for SynastryOptions {
    fn default() -> Self {
        SynastryOptions {
            aspects: AspectType::MAJOR.to_vec(),
            orb_multiplier: 1.0,
            include_angles: false,
        }
    }
}

/// One aspect between a point of chart A and a point of chart B.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SynastryAspect {
    /// The point in chart A.
    pub a: PointId,
    /// The point in chart B.
    pub b: PointId,
    /// The aspect found.
    pub aspect_type: AspectType,
    /// Distance from exactness in degrees, never negative.
    pub orb_deg: f64,
    /// The shorter-arc separation of the two longitudes, in `[0, 180]`.
    pub separation_deg: f64,
}

/// A point and the house of the other chart it falls in.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HouseOverlay {
    /// The point being placed.
    pub point: PointId,
    /// Its longitude in degrees.
    pub longitude_deg: f64,
    /// House number `1..=12` in the other chart.
    pub house: usize,
}

/// The result of [`synastry`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Synastry {
    /// The points of chart A, one per row of the grid.
    pub rows: Vec<PointId>,
    /// The points of chart B, one per column of the grid.
    pub cols: Vec<PointId>,
    /// Row-major grid of length `rows.len() * cols.len()`; `None` where the
    /// pair forms no aspect.
    pub cells: Vec<Option<SynastryAspect>>,
    /// Chart B's points placed in chart A's houses.
    pub b_in_a: Vec<HouseOverlay>,
    /// Chart A's points placed in chart B's houses.
    pub a_in_b: Vec<HouseOverlay>,
}

impl Synastry {
    /// The aspect between row `i` (chart A) and column `j` (chart B), or
    /// `None` if there is none or the indexes are out of range.
    pub fn cell(&self, i: usize, j: usize) -> Option<&SynastryAspect> {
        if j >= self.cols.len() {
            return None;
        }
        let idx = i.checked_mul(self.cols.len())?.checked_add(j)?;
        self.cells.get(idx)?.as_ref()
    }

    /// Every aspect found, in row-major order.
    pub fn aspects(&self) -> impl Iterator<Item = &SynastryAspect> {
        self.cells.iter().flatten()
    }
}

fn check_longitudes(points: &[(PointId, f64)], which: &str) -> Result<()> {
    if let Some((p, lon)) = points.iter().find(|(_, lon)| !lon.is_finite()) {
        return Err(RelationshipError::InvalidInput(format!(
            "chart {which}: longitude of {p:?} is not finite ({lon})"
        )));
    }
    Ok(())
}

/// Compute the full synastry between two charts.
///
/// Orbs are `default_orb_deg * orb_multiplier`. When a pair fits more than one
/// listed aspect (only possible with large multipliers or many minor
/// aspects), the tightest one is kept.
///
/// Errors with [`RelationshipError::InvalidInput`] if `orb_multiplier` is not
/// finite and positive, or a chart holds a non-finite longitude.
///
/// ```no_run
/// use xalen_relationship::{NatalChart, SynastryOptions, default_bodies, synastry};
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
/// let syn = synastry(&a, &b, &SynastryOptions::default()).unwrap();
/// println!("{} aspects", syn.aspects().count());
/// ```
pub fn synastry(a: &NatalChart, b: &NatalChart, o: &SynastryOptions) -> Result<Synastry> {
    if !o.orb_multiplier.is_finite() || o.orb_multiplier <= 0.0 {
        return Err(RelationshipError::InvalidInput(format!(
            "orb_multiplier must be finite and greater than zero (got {})",
            o.orb_multiplier
        )));
    }
    let pa = a.points(o.include_angles);
    let pb = b.points(o.include_angles);
    check_longitudes(&pa, "A")?;
    check_longitudes(&pb, "B")?;
    let cells = inter_aspects(&pa, &pb, &o.aspects, o.orb_multiplier);
    Ok(Synastry {
        rows: pa.iter().map(|p| p.0).collect(),
        cols: pb.iter().map(|p| p.0).collect(),
        cells,
        b_in_a: house_overlay(&pb, &a.houses),
        a_in_b: house_overlay(&pa, &b.houses),
    })
}

/// The aspect grid between two lists of `(point, longitude in degrees)`.
///
/// The result is row-major with `a.len() * b.len()` entries: entry
/// `i * b.len() + j` pairs `a[i]` with `b[j]`. Unlike
/// `xalen_western::aspects::find_all_aspects`, which pairs the points of one
/// list with each other, this pairs across two lists.
///
/// A pair with a non-finite longitude, or any pair when `orb_multiplier` is
/// not finite and positive, yields `None`; use [`synastry`] for validation
/// with an error.
///
/// ```
/// use xalen_relationship::{PointId, inter_aspects};
/// use xalen_ephem::Body;
/// use xalen_western::aspects::AspectType;
///
/// let a = [(PointId::Body(Body::Sun), 10.0)];
/// let b = [(PointId::Body(Body::Moon), 130.0)];
/// let grid = inter_aspects(&a, &b, AspectType::MAJOR, 1.0);
/// let hit = grid[0].unwrap();
/// assert_eq!(hit.aspect_type, AspectType::Trine);
/// assert!(hit.orb_deg < 1e-9);
/// ```
pub fn inter_aspects(
    a: &[(PointId, f64)],
    b: &[(PointId, f64)],
    aspects: &[AspectType],
    orb_multiplier: f64,
) -> Vec<Option<SynastryAspect>> {
    let valid_mult = orb_multiplier.is_finite() && orb_multiplier > 0.0;
    let mut out = Vec::with_capacity(a.len().saturating_mul(b.len()));
    for &(id_a, lon_a) in a {
        for &(id_b, lon_b) in b {
            let mut best: Option<SynastryAspect> = None;
            if valid_mult && lon_a.is_finite() && lon_b.is_finite() {
                for &t in aspects {
                    if let Some(hit) = find_aspect(lon_a, lon_b, 0.0, 0.0, &[t], orb_multiplier) {
                        let better = best.is_none_or(|cur| hit.orb_deg < cur.orb_deg);
                        if better {
                            best = Some(SynastryAspect {
                                a: id_a,
                                b: id_b,
                                aspect_type: hit.aspect_type,
                                orb_deg: hit.orb_deg,
                                separation_deg: angular_distance(lon_a, lon_b),
                            });
                        }
                    }
                }
            }
            out.push(best);
        }
    }
    out
}

/// Place each `(point, longitude in degrees)` in a house of `cusps`.
///
/// Points with a non-finite longitude are skipped. A point exactly on a cusp
/// belongs to the house that starts there, up to floating-point rounding of
/// the degree-to-radian conversion.
///
/// ```
/// use xalen_relationship::{PointId, house_overlay};
/// use xalen_ephem::Body;
/// use xalen_houses::{GeoLocation, HouseSystem, compute_houses_true};
///
/// let cusps = compute_houses_true(2_451_545.0, &GeoLocation::new(0.0, 0.0), HouseSystem::Equal);
/// let asc = cusps.ascendant.to_degrees();
/// let o = house_overlay(&[(PointId::Body(Body::Sun), asc + 5.0)], &cusps);
/// assert_eq!(o[0].house, 1);
/// ```
pub fn house_overlay(points: &[(PointId, f64)], cusps: &HouseCusps) -> Vec<HouseOverlay> {
    points
        .iter()
        .filter(|(_, lon)| lon.is_finite())
        .map(|&(point, lon)| HouseOverlay {
            point,
            longitude_deg: lon,
            house: cusps.planet_in_house(lon.rem_euclid(360.0).to_radians()),
        })
        .collect()
}
