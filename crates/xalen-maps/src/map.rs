//! The full astrocartography map for a moment.

use xalen_coords::{ecliptic_to_equatorial, gast_deg, mean_obliquity, nutation_2000b};
use xalen_ephem::{Almanac, Body};
use xalen_time::{DeltaTModel, JdUT1, JulianDay};

use crate::lines::{horizon_lines, ic_longitude_deg, latitudes, mc_longitude_deg};
use crate::{AstroMap, BodyLines, LineKind, MapError, MapLine, MapOptions};

/// Compute MC, IC, ASC and DSC lines and the zenith point of each body.
///
/// Positions are geocentric apparent places from the `Almanac`, converted to
/// right ascension and declination with the true obliquity of date, and
/// referred to Greenwich apparent sidereal time. `bodies` are mapped in the
/// order given. [`Body::Earth`] cannot be mapped.
///
/// The MC and IC lines are two-point meridians from `lat_min_deg` to
/// `lat_max_deg`. See [`horizon_lines`] for the ASC and DSC lines and
/// [`MapOptions`] for sampling.
///
/// Errors: [`MapError::InvalidInput`] for a non-finite epoch, `Body::Earth` or
/// bad options; [`MapError::Ephemeris`] when a position is unavailable.
///
/// ```no_run
/// use xalen_maps::{astro_map, MapOptions};
/// use xalen_ephem::{Almanac, Body};
/// use xalen_time::JdUT1;
///
/// let almanac = Almanac::default_vedic();
/// let map = astro_map(&almanac, JdUT1(2_460_482.5), Body::ALL_PLANETS, &MapOptions::default())
///     .unwrap();
/// assert_eq!(map.bodies.len(), 10);
/// ```
pub fn astro_map(
    almanac: &Almanac,
    jd_ut1: JdUT1,
    bodies: &[Body],
    o: &MapOptions,
) -> Result<AstroMap, MapError> {
    if !jd_ut1.0.is_finite() {
        return Err(MapError::InvalidInput(
            "Julian Day must be finite".to_string(),
        ));
    }
    let lats = latitudes(o)?;
    let (lat_min, lat_max) = match (lats.first(), lats.last()) {
        (Some(&a), Some(&b)) => (a, b),
        _ => return Err(MapError::InvalidInput("no latitudes to map".to_string())),
    };

    let jd_tt = jd_ut1
        .to_tt(&DeltaTModel::StephensonMorrisonHohenkerk2016)
        .as_f64();
    let t = (jd_tt - 2_451_545.0) / 36525.0;
    let eps = mean_obliquity(t) + nutation_2000b(t).delta_epsilon;
    let gast = gast_deg(jd_ut1.0, t);

    let mut out = Vec::with_capacity(bodies.len());
    for &body in bodies {
        if body == Body::Earth {
            return Err(MapError::InvalidInput(
                "Earth has no geocentric position".to_string(),
            ));
        }
        let ecl = almanac.geocentric_ecliptic(body, jd_ut1)?;
        let eq = ecliptic_to_equatorial(&ecl, eps);
        let ra = eq.right_ascension.to_degrees().rem_euclid(360.0);
        let dec = eq.declination.to_degrees();

        let mc_lon = mc_longitude_deg(ra, gast);
        let ic_lon = ic_longitude_deg(ra, gast);
        let (asc, dsc) = horizon_lines(ra, dec, gast, o)?;
        out.push(BodyLines {
            body,
            ra_deg: ra,
            dec_deg: dec,
            zenith: (dec, mc_lon),
            lines: vec![
                MapLine {
                    kind: LineKind::Mc,
                    segments: vec![vec![(lat_min, mc_lon), (lat_max, mc_lon)]],
                },
                MapLine {
                    kind: LineKind::Ic,
                    segments: vec![vec![(lat_min, ic_lon), (lat_max, ic_lon)]],
                },
                MapLine {
                    kind: LineKind::Asc,
                    segments: asc,
                },
                MapLine {
                    kind: LineKind::Dsc,
                    segments: dsc,
                },
            ],
        });
    }
    Ok(AstroMap {
        jd_ut1: jd_ut1.0,
        gast_deg: gast,
        bodies: out,
    })
}
