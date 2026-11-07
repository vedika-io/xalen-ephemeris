//! Meridian and horizon line mathematics.

use crate::{HorizonCrossing, LatLon, MapError, MapOptions, Segments};

/// Wrap a longitude to `[-180, 180)`.
pub(crate) fn wrap180(lon: f64) -> f64 {
    let x = lon.rem_euclid(360.0);
    if x >= 180.0 { x - 360.0 } else { x }
}

/// East longitude, in `[-180, 180)`, of the MC line of a body with right
/// ascension `ra_deg` when Greenwich sidereal time is `gst_deg`.
///
/// The body is on the upper meridian where the local sidereal time equals its
/// right ascension: `GST + longitude = RA`.
///
/// ```
/// use xalen_maps::mc_longitude_deg;
/// assert!((mc_longitude_deg(100.0, 40.0) - 60.0).abs() < 1e-12);
/// ```
pub fn mc_longitude_deg(ra_deg: f64, gst_deg: f64) -> f64 {
    wrap180(ra_deg - gst_deg)
}

/// East longitude, in `[-180, 180)`, of the IC line: the MC line plus 180
/// degrees.
///
/// ```
/// use xalen_maps::ic_longitude_deg;
/// assert!((ic_longitude_deg(100.0, 40.0) + 120.0).abs() < 1e-12);
/// ```
pub fn ic_longitude_deg(ra_deg: f64, gst_deg: f64) -> f64 {
    wrap180(ra_deg - gst_deg + 180.0)
}

/// `cos H0` for a body of declination `dec_deg` at latitude `lat_deg` and
/// horizon altitude `h0_deg`. Infinite when the denominator vanishes (at a pole
/// or for a body at a celestial pole), signed by the numerator.
fn cos_half_arc(lat_deg: f64, dec_deg: f64, h0_deg: f64) -> f64 {
    let (phi, dec, h0) = (
        lat_deg.to_radians(),
        dec_deg.to_radians(),
        h0_deg.to_radians(),
    );
    let num = h0.sin() - phi.sin() * dec.sin();
    let den = phi.cos() * dec.cos();
    if den.abs() < 1e-12 {
        if num > 0.0 {
            f64::INFINITY
        } else if num < 0.0 {
            f64::NEG_INFINITY
        } else {
            0.0
        }
    } else {
        num / den
    }
}

/// The semi-diurnal arc `H0` of a body at a latitude (Meeus ch. 15, eq. 15.1):
/// `cos H0 = (sin h0 - sin φ sin δ) / (cos φ cos δ)`.
///
/// For `h0 = 0` this is `cos H0 = -tan φ tan δ`. Returns
/// [`HorizonCrossing::AlwaysAbove`] when `cos H0 <= -1` and
/// [`HorizonCrossing::AlwaysBelow`] when `cos H0 >= 1`. A non-finite argument
/// has no crossing and gives `AlwaysBelow`.
///
/// ```
/// use xalen_maps::{horizon_half_arc, HorizonCrossing};
/// // Declination of the June solstice Sun at latitude 45 north.
/// match horizon_half_arc(45.0, 23.4393, 0.0) {
///     HorizonCrossing::Crosses { half_arc_deg } => assert!((half_arc_deg - 115.70).abs() < 0.02),
///     other => panic!("{other:?}"),
/// }
/// assert_eq!(horizon_half_arc(80.0, 20.0, 0.0), HorizonCrossing::AlwaysAbove);
/// ```
pub fn horizon_half_arc(lat_deg: f64, dec_deg: f64, h0_deg: f64) -> HorizonCrossing {
    if !(lat_deg.is_finite() && dec_deg.is_finite() && h0_deg.is_finite()) {
        return HorizonCrossing::AlwaysBelow;
    }
    let c = cos_half_arc(lat_deg, dec_deg, h0_deg);
    if c >= 1.0 {
        HorizonCrossing::AlwaysBelow
    } else if c <= -1.0 {
        HorizonCrossing::AlwaysAbove
    } else {
        HorizonCrossing::Crosses {
            half_arc_deg: c.acos().to_degrees(),
        }
    }
}

fn invalid<T>(msg: impl Into<String>) -> Result<T, MapError> {
    Err(MapError::InvalidInput(msg.into()))
}

/// Check the options and return the sampled latitudes, south to north. The
/// northern limit is always the last sample.
pub(crate) fn latitudes(o: &MapOptions) -> Result<Vec<f64>, MapError> {
    let finite = o.lat_min_deg.is_finite()
        && o.lat_max_deg.is_finite()
        && o.lat_step_deg.is_finite()
        && o.horizon_altitude_deg.is_finite();
    if !finite {
        return invalid("map options must be finite");
    }
    if o.lat_min_deg.abs() >= 90.0 || o.lat_max_deg.abs() >= 90.0 {
        return invalid("latitude limits must be strictly between -90 and 90 degrees");
    }
    if o.lat_min_deg >= o.lat_max_deg {
        return invalid("lat_min_deg must be less than lat_max_deg");
    }
    if o.lat_step_deg <= 0.0 {
        return invalid("lat_step_deg must be greater than zero");
    }
    if o.horizon_altitude_deg.abs() >= 90.0 {
        return invalid("horizon_altitude_deg must be strictly between -90 and 90 degrees");
    }
    if o.max_points < 2 {
        return invalid("max_points must be at least 2");
    }
    let steps = ((o.lat_max_deg - o.lat_min_deg) / o.lat_step_deg).floor();
    if steps + 1.0 > o.max_points as f64 {
        return invalid(format!(
            "{} latitude samples requested, more than max_points ({})",
            steps + 1.0,
            o.max_points
        ));
    }
    let n = steps as usize;
    let mut lats: Vec<f64> = (0..=n)
        .map(|i| o.lat_min_deg + i as f64 * o.lat_step_deg)
        .collect();
    if lats.last().is_some_and(|&l| l < o.lat_max_deg - 1e-9) {
        lats.push(o.lat_max_deg);
    }
    if lats.len() > o.max_points {
        return invalid(format!(
            "{} latitude samples requested, more than max_points ({})",
            lats.len(),
            o.max_points
        ));
    }
    Ok(lats)
}

/// Collects a polyline and splits it at the antimeridian and at gaps.
struct SegBuilder {
    done: Vec<Vec<LatLon>>,
    cur: Vec<LatLon>,
}

impl SegBuilder {
    fn new() -> Self {
        SegBuilder {
            done: Vec::new(),
            cur: Vec::new(),
        }
    }

    fn flush(&mut self) {
        let cur = std::mem::take(&mut self.cur);
        if cur.len() >= 2 {
            self.done.push(cur);
        }
    }

    fn push(&mut self, lat: f64, lon: f64) {
        if let Some(&(plat, plon)) = self.cur.last() {
            if (lat - plat).abs() < 1e-9 && (lon - plon).abs() < 1e-9 {
                return;
            }
            if (lon - plon).abs() > 180.0 {
                // The line goes round the back of the map: end this piece on the
                // meridian at +-180, start the next one on the other side.
                let lon_u = if lon > plon { lon - 360.0 } else { lon + 360.0 };
                let edge_from = if lon_u < plon { -180.0 } else { 180.0 };
                let span = lon_u - plon;
                let t = if span == 0.0 {
                    0.0
                } else {
                    ((edge_from - plon) / span).clamp(0.0, 1.0)
                };
                let lat_x = plat + t * (lat - plat);
                self.cur.push((lat_x, edge_from));
                self.flush();
                self.cur.push((lat_x, -edge_from));
            }
        }
        self.cur.push((lat, lon));
    }

    fn finish(mut self) -> Vec<Vec<LatLon>> {
        self.flush();
        self.done
    }
}

/// Bisect between a latitude where the body crosses the horizon and one where
/// it does not. Returns the crossing-side latitude (within 90/2^40 degrees of
/// the limit) and the half arc at the limit, snapped to exactly 0 or 180.
fn limit_vertex(valid_lat: f64, invalid_lat: f64, dec_deg: f64, h0_deg: f64) -> (f64, f64) {
    let (mut v, mut i) = (valid_lat, invalid_lat);
    for _ in 0..40 {
        let m = 0.5 * (v + i);
        if matches!(
            horizon_half_arc(m, dec_deg, h0_deg),
            HorizonCrossing::Crosses { .. }
        ) {
            v = m;
        } else {
            i = m;
        }
    }
    // Beyond the limit cos H0 >= 1 means H0 -> 0 (the lines meet the MC line);
    // cos H0 <= -1 means H0 -> 180 (they meet the IC line).
    let half_arc = if cos_half_arc(i, dec_deg, h0_deg) >= 1.0 {
        0.0
    } else {
        180.0
    };
    (v, half_arc)
}

/// The rising (ASC) and setting (DSC) lines of a body.
///
/// `ra_deg` and `dec_deg` are the body's right ascension and declination and
/// `gst_deg` the Greenwich sidereal time, all in degrees. Latitudes are sampled
/// as set in `o`. The returned pair is `(asc_segments, dsc_segments)`; each
/// segment is a polyline of `(lat, lon)` points in increasing latitude.
///
/// * Rising: `longitude = RA - H0 - GST`; setting: `longitude = RA + H0 - GST`.
/// * Where the body is circumpolar or never rises, no point is emitted and the
///   line is broken. The exact limit latitude is added as a vertex on each side:
///   for a body of positive declination (and `h0 = 0`) the northern limit is
///   `90 - dec` where both lines meet the IC line, and the southern limit is
///   `-(90 - dec)` where they meet the MC line. The signs swap for negative
///   declination.
/// * A segment is split where its longitude jumps across the antimeridian.
///
/// Errors with [`MapError::InvalidInput`] for non-finite numbers, `|dec| > 90`,
/// or bad options (see [`MapOptions`]).
///
/// ```
/// use xalen_maps::{horizon_lines, MapOptions};
/// let (asc, dsc) = horizon_lines(100.0, 0.0, 40.0, &MapOptions::default()).unwrap();
/// // Declination 0: the half arc is 90 degrees at every latitude.
/// assert!((asc[0][0].1 + 30.0).abs() < 1e-9);
/// assert!((dsc[0][0].1 - 150.0).abs() < 1e-9);
/// ```
pub fn horizon_lines(
    ra_deg: f64,
    dec_deg: f64,
    gst_deg: f64,
    o: &MapOptions,
) -> Result<(Segments, Segments), MapError> {
    if !(ra_deg.is_finite() && dec_deg.is_finite() && gst_deg.is_finite()) {
        return invalid("right ascension, declination and sidereal time must be finite");
    }
    if dec_deg.abs() > 90.0 {
        return invalid("declination must be within 90 degrees");
    }
    let lats = latitudes(o)?;
    let h0 = o.horizon_altitude_deg;
    let mut asc = SegBuilder::new();
    let mut dsc = SegBuilder::new();
    let mut prev: Option<(f64, bool)> = None;

    for &lat in &lats {
        match horizon_half_arc(lat, dec_deg, h0) {
            HorizonCrossing::Crosses { half_arc_deg } => {
                if let Some((prev_lat, false)) = prev {
                    let (vlat, vh) = limit_vertex(lat, prev_lat, dec_deg, h0);
                    asc.push(vlat, wrap180(ra_deg - vh - gst_deg));
                    dsc.push(vlat, wrap180(ra_deg + vh - gst_deg));
                }
                asc.push(lat, wrap180(ra_deg - half_arc_deg - gst_deg));
                dsc.push(lat, wrap180(ra_deg + half_arc_deg - gst_deg));
                prev = Some((lat, true));
            }
            _ => {
                if let Some((prev_lat, true)) = prev {
                    let (vlat, vh) = limit_vertex(prev_lat, lat, dec_deg, h0);
                    asc.push(vlat, wrap180(ra_deg - vh - gst_deg));
                    dsc.push(vlat, wrap180(ra_deg + vh - gst_deg));
                }
                asc.flush();
                dsc.flush();
                prev = Some((lat, false));
            }
        }
    }
    Ok((asc.finish(), dsc.finish()))
}
