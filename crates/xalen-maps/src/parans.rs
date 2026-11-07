//! Parans: the places where two bodies stand on angles at the same moment.
//!
//! On the map a paran is a crossing of one body's line with another body's
//! line. Every crossing is solved in closed form from the right ascension,
//! declination and sidereal time on the [`AstroMap`]; the drawn polylines are
//! not used.

use serde::{Deserialize, Serialize};
use xalen_ephem::Body;

use crate::lines::wrap180;
use crate::{AstroMap, LineKind, MapError, ic_longitude_deg, mc_longitude_deg};

/// Which crossings [`parans`] looks for.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ParanOptions {
    /// Southern limit in degrees, greater than -90. Default `-66`.
    pub min_lat_deg: f64,
    /// Northern limit in degrees, less than 90 and not below `min_lat_deg`.
    /// Default `66`.
    pub max_lat_deg: f64,
    /// Also return crossings of two horizon lines (one body rising or setting
    /// while the other rises or sets). Default `true`.
    pub include_horizon_pairs: bool,
    /// Altitude of the horizon in degrees, strictly between -90 and 90. Use the
    /// same value as [`MapOptions::horizon_altitude_deg`](crate::MapOptions)
    /// so the crossings lie on the drawn lines. Default `0.0`.
    pub horizon_altitude_deg: f64,
}

impl Default for ParanOptions {
    fn default() -> Self {
        ParanOptions {
            min_lat_deg: -66.0,
            max_lat_deg: 66.0,
            include_horizon_pairs: true,
            horizon_altitude_deg: 0.0,
        }
    }
}

/// One crossing: `body_a` is on the angle `kind_a` while `body_b` is on
/// `kind_b`, at this point on Earth.
///
/// `body_a` is always the body listed first in [`AstroMap::bodies`]. At most one
/// of the two kinds is a meridian (MC or IC); two meridians never cross.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Paran {
    /// The first body of the pair.
    pub body_a: Body,
    /// Its angle.
    pub kind_a: LineKind,
    /// The second body of the pair.
    pub body_b: Body,
    /// Its angle.
    pub kind_b: LineKind,
    /// Latitude of the crossing in degrees, north-positive.
    pub latitude_deg: f64,
    /// Longitude of the crossing in degrees, east-positive in `[-180, 180)`.
    pub longitude_deg: f64,
}

/// Why a pair of lines has no crossing in the result.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SkipReason {
    /// The lines never meet anywhere on Earth (possible only with a horizon
    /// altitude other than zero).
    NeverCross,
    /// The lines meet, but outside the requested latitudes. Near the poles a
    /// body is circumpolar or never rises, so its horizon line ends at the limit
    /// latitude `90 - |dec|` and crossings beyond it fall here.
    OutsideLatitudes {
        /// Latitude of the crossing in degrees.
        latitude_deg: f64,
        /// Longitude of the crossing in degrees.
        longitude_deg: f64,
    },
    /// The two lines are the same curve, so there is no single crossing: a body
    /// on the celestial equator whose rising or setting meridian is the other
    /// body's MC or IC, or two bodies at the same place in the sky.
    SameLine,
}

/// A pair of lines that gave no crossing in the result.
///
/// A kind of `None` stands for "the rising or setting line": when the lines
/// never meet, which of the two horizon lines would have met is undefined.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SkippedParan {
    /// The first body of the pair (listed first in the map).
    pub body_a: Body,
    /// Its line, or `None` for its horizon (ASC or DSC) line.
    pub kind_a: Option<LineKind>,
    /// The second body.
    pub body_b: Body,
    /// Its line, or `None` for its horizon (ASC or DSC) line.
    pub kind_b: Option<LineKind>,
    /// Why there is no crossing.
    pub reason: SkipReason,
}

/// The crossings found by [`parans`], and the line pairs that gave none.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParanSet {
    /// Crossings within the latitude limits, ordered by latitude (south to
    /// north), then longitude.
    pub parans: Vec<Paran>,
    /// Line pairs with no crossing within the limits, in the order they were
    /// examined.
    pub skipped: Vec<SkippedParan>,
}

/// One body as the solver needs it.
struct Sky {
    body: Body,
    dec_deg: f64,
    mc_lon: f64,
    ic_lon: f64,
    /// Unit vector of the substellar point (where the body is overhead).
    z: [f64; 3],
}

fn unit(lat_deg: f64, lon_deg: f64) -> [f64; 3] {
    let (lat, lon) = (lat_deg.to_radians(), lon_deg.to_radians());
    [lat.cos() * lon.cos(), lat.cos() * lon.sin(), lat.sin()]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn to_lat_lon(p: [f64; 3]) -> (f64, f64) {
    let lat = p[2].atan2(p[0].hypot(p[1])).to_degrees();
    let lon = wrap180(p[1].atan2(p[0]).to_degrees());
    (lat, lon)
}

/// Rising (hour angle negative, east of the meridian) or setting.
fn horizon_kind(lon_deg: f64, mc_lon: f64) -> LineKind {
    if wrap180(lon_deg - mc_lon) < 0.0 {
        LineKind::Asc
    } else {
        LineKind::Dsc
    }
}

enum Roots<T> {
    Some(Vec<T>),
    Never,
    SameLine,
}

/// Latitudes on the meridian at `lon_deg` where the body is at altitude `h0`.
///
/// With hour angle `H = lon - mc_lon` fixed along the meridian, the altitude
/// equation `sin φ sin δ + cos φ cos δ cos H = sin h0` is
/// `R cos(φ - ψ) = sin h0` with `R = hypot(sin δ, cos δ cos H)` and
/// `ψ = atan2(sin δ, cos δ cos H)`, so `φ = ψ ± acos(sin h0 / R)`. Only roots
/// within ±90 degrees lie on this half-meridian; the others are on the opposite
/// one.
fn meridian_roots(sky: &Sky, lon_deg: f64, h0_deg: f64) -> Roots<f64> {
    let h = (lon_deg - sky.mc_lon).to_radians();
    let dec = sky.dec_deg.to_radians();
    let (s, c) = (dec.sin(), dec.cos() * h.cos());
    let r = s.hypot(c);
    let target = h0_deg.to_radians().sin();
    if r < 1e-12 {
        return if target.abs() < 1e-12 {
            Roots::SameLine
        } else {
            Roots::Never
        };
    }
    let ratio = target / r;
    if ratio.abs() > 1.0 {
        return Roots::Never;
    }
    let psi = s.atan2(c);
    let w = ratio.acos();
    let mut out: Vec<f64> = Vec::with_capacity(2);
    for phi in [psi - w, psi + w] {
        let lat = wrap180(phi.to_degrees());
        if lat.abs() <= 90.0 && !out.iter().any(|&x| (x - lat).abs() < 1e-12) {
            out.push(lat);
        }
    }
    if out.is_empty() {
        Roots::Never
    } else {
        Roots::Some(out)
    }
}

/// Points where both bodies are at altitude `h0`: the intersection of two
/// circles on the sphere, `P·za = P·zb = sin h0`.
///
/// Writing `P = a (za + zb) + c (za × zb)` with `g = za·zb` gives
/// `a = sin h0 / (1 + g)` and `c² = (1 - 2 sin² h0 / (1 + g)) / (1 - g²)`.
fn horizon_points(za: [f64; 3], zb: [f64; 3], h0_deg: f64) -> Roots<(f64, f64)> {
    let s = h0_deg.to_radians().sin();
    let g = dot(za, zb).clamp(-1.0, 1.0);
    let n = cross(za, zb);
    let n2 = dot(n, n);
    if n2 < 1e-24 {
        // The same or opposite substellar points: one circle, or two disjoint
        // ones (opposite points with a horizon off zero).
        return if g > 0.0 || s.abs() < 1e-12 {
            Roots::SameLine
        } else {
            Roots::Never
        };
    }
    let a = s / (1.0 + g);
    let c2 = (1.0 - 2.0 * s * a) / n2;
    if c2 < 0.0 {
        return Roots::Never;
    }
    let c = c2.sqrt();
    let mut out = Vec::with_capacity(2);
    for sign in [1.0, -1.0] {
        let p = [
            a * (za[0] + zb[0]) + sign * c * n[0],
            a * (za[1] + zb[1]) + sign * c * n[1],
            a * (za[2] + zb[2]) + sign * c * n[2],
        ];
        out.push(to_lat_lon(p));
        if c == 0.0 {
            break;
        }
    }
    Roots::Some(out)
}

struct Collector<'a> {
    options: &'a ParanOptions,
    parans: Vec<Paran>,
    skipped: Vec<SkippedParan>,
}

impl Collector<'_> {
    fn in_range(&self, lat: f64) -> bool {
        lat >= self.options.min_lat_deg && lat <= self.options.max_lat_deg
    }

    /// Record one crossing, or skip it when it is outside the latitudes.
    fn found(&mut self, a: (Body, LineKind), b: (Body, LineKind), lat: f64, lon: f64) {
        if self.in_range(lat) {
            self.parans.push(Paran {
                body_a: a.0,
                kind_a: a.1,
                body_b: b.0,
                kind_b: b.1,
                latitude_deg: lat,
                longitude_deg: lon,
            });
        } else {
            self.skip(
                (a.0, Some(a.1)),
                (b.0, Some(b.1)),
                SkipReason::OutsideLatitudes {
                    latitude_deg: lat,
                    longitude_deg: lon,
                },
            );
        }
    }

    fn skip(
        &mut self,
        a: (Body, Option<LineKind>),
        b: (Body, Option<LineKind>),
        reason: SkipReason,
    ) {
        self.skipped.push(SkippedParan {
            body_a: a.0,
            kind_a: a.1,
            body_b: b.0,
            kind_b: b.1,
            reason,
        });
    }

    /// The horizon line of `h` against the MC or IC line of `m`. `h_first`
    /// says whether `h` is body A of the pair.
    fn horizon_meridian(&mut self, h: &Sky, m: &Sky, meridian: LineKind, h_first: bool) {
        let lon = if meridian == LineKind::Mc {
            m.mc_lon
        } else {
            m.ic_lon
        };
        let reason = match meridian_roots(h, lon, self.options.horizon_altitude_deg) {
            Roots::Some(lats) => {
                let hk = (h.body, horizon_kind(lon, h.mc_lon));
                let mk = (m.body, meridian);
                let (a, b) = if h_first { (hk, mk) } else { (mk, hk) };
                for lat in lats {
                    self.found(a, b, lat, lon);
                }
                return;
            }
            Roots::Never => SkipReason::NeverCross,
            Roots::SameLine => SkipReason::SameLine,
        };
        let (hk, mk) = ((h.body, None), (m.body, Some(meridian)));
        let (a, b) = if h_first { (hk, mk) } else { (mk, hk) };
        self.skip(a, b, reason);
    }

    /// The horizon lines of `a` against the horizon lines of `b`.
    fn horizon_horizon(&mut self, a: &Sky, b: &Sky) {
        match horizon_points(a.z, b.z, self.options.horizon_altitude_deg) {
            Roots::Some(points) => {
                for (lat, lon) in points {
                    let ka = horizon_kind(lon, a.mc_lon);
                    let kb = horizon_kind(lon, b.mc_lon);
                    self.found((a.body, ka), (b.body, kb), lat, lon);
                }
            }
            Roots::Never => self.skip((a.body, None), (b.body, None), SkipReason::NeverCross),
            Roots::SameLine => self.skip((a.body, None), (b.body, None), SkipReason::SameLine),
        }
    }
}

fn invalid<T>(msg: &str) -> Result<T, MapError> {
    Err(MapError::InvalidInput(msg.to_string()))
}

fn check_options(o: &ParanOptions) -> Result<(), MapError> {
    if !(o.min_lat_deg.is_finite()
        && o.max_lat_deg.is_finite()
        && o.horizon_altitude_deg.is_finite())
    {
        return invalid("paran options must be finite");
    }
    if o.min_lat_deg.abs() >= 90.0 || o.max_lat_deg.abs() >= 90.0 {
        return invalid("paran latitude limits must be strictly between -90 and 90 degrees");
    }
    if o.min_lat_deg > o.max_lat_deg {
        return invalid("min_lat_deg must not be greater than max_lat_deg");
    }
    if o.horizon_altitude_deg.abs() >= 90.0 {
        return invalid("horizon_altitude_deg must be strictly between -90 and 90 degrees");
    }
    Ok(())
}

fn sky_of(map: &AstroMap) -> Result<Vec<Sky>, MapError> {
    if !map.gast_deg.is_finite() {
        return invalid("the map's sidereal time must be finite");
    }
    map.bodies
        .iter()
        .map(|b| {
            if !(b.ra_deg.is_finite() && b.dec_deg.is_finite()) || b.dec_deg.abs() > 90.0 {
                return invalid("right ascension and declination must be finite, |dec| <= 90");
            }
            let mc_lon = mc_longitude_deg(b.ra_deg, map.gast_deg);
            Ok(Sky {
                body: b.body,
                dec_deg: b.dec_deg,
                mc_lon,
                ic_lon: ic_longitude_deg(b.ra_deg, map.gast_deg),
                z: unit(b.dec_deg, mc_lon),
            })
        })
        .collect()
}

/// Every paran of the map: each crossing of one body's line with another
/// body's line within the latitude limits (Brady, *The Eagle and the Lark*;
/// Lewis's astrocartography).
///
/// For each pair of bodies, in the order they are listed in the map (the
/// earlier one is body A):
///
/// * **Horizon with meridian.** The MC or IC line of B is the meridian at its
///   longitude `λ`, where A's hour angle `H = λ - λ_MC(A)` is fixed. A's rising
///   or setting line passes through it where
///   `sin φ sin δ_A + cos φ cos δ_A cos H = sin h0`, solved in closed form as
///   `φ = ψ ± acos(sin h0 / R)`, `R = hypot(sin δ_A, cos δ_A cos H)`,
///   `ψ = atan2(sin δ_A, cos δ_A cos H)`. A is rising (ASC) when `H < 0` and
///   setting (DSC) when `H > 0`. The same is done for B's horizon against A's
///   MC and IC. With `h0 = 0` there is exactly one crossing per line pair.
/// * **Horizon with horizon** (when `include_horizon_pairs`). Both bodies are
///   at altitude `h0`: the intersection of two circles on the sphere centred on
///   the substellar points, at most two points. Each body is rising or setting
///   by the sign of its hour angle there.
///
/// Two meridian lines never cross. A body listed twice is used once, at its
/// first place, and is never paired with itself. Line pairs with no crossing inside the limits
/// are listed in [`ParanSet::skipped`] with the reason.
///
/// Errors with [`MapError::InvalidInput`] for bad options or a map holding
/// non-finite values.
///
/// ```no_run
/// use xalen_maps::{astro_map, parans, MapOptions, ParanOptions};
/// use xalen_ephem::{Almanac, Body};
/// use xalen_time::JdUT1;
///
/// let map = astro_map(&Almanac::default_vedic(), JdUT1(2_460_482.5),
///                     &[Body::Sun, Body::Jupiter], &MapOptions::default()).unwrap();
/// for p in parans(&map, &ParanOptions::default()).unwrap().parans {
///     println!("{:?} {:?} / {:?} {:?} at {:.2}, {:.2}",
///              p.body_a, p.kind_a, p.body_b, p.kind_b, p.latitude_deg, p.longitude_deg);
/// }
/// ```
pub fn parans(map: &AstroMap, options: &ParanOptions) -> Result<ParanSet, MapError> {
    check_options(options)?;
    let mut sky = sky_of(map)?;
    // A body listed twice is used once, at its first place.
    let mut seen = Vec::with_capacity(sky.len());
    sky.retain(|s| {
        let first = !seen.contains(&s.body);
        seen.push(s.body);
        first
    });
    let mut c = Collector {
        options,
        parans: Vec::new(),
        skipped: Vec::new(),
    };
    for (i, a) in sky.iter().enumerate() {
        for b in &sky[i + 1..] {
            for meridian in [LineKind::Mc, LineKind::Ic] {
                c.horizon_meridian(a, b, meridian, true);
            }
            for meridian in [LineKind::Mc, LineKind::Ic] {
                c.horizon_meridian(b, a, meridian, false);
            }
            if options.include_horizon_pairs {
                c.horizon_horizon(a, b);
            }
        }
    }
    c.parans.sort_by(|x, y| {
        x.latitude_deg
            .total_cmp(&y.latitude_deg)
            .then(x.longitude_deg.total_cmp(&y.longitude_deg))
    });
    Ok(ParanSet {
        parans: c.parans,
        skipped: c.skipped,
    })
}

/// The parans active at a latitude: those within `orb_deg` of `lat_deg`
/// (Brady works with a band of about one degree).
///
/// The search band `lat_deg ± orb_deg` (kept inside ±90) replaces the latitude
/// limits of `options`; its horizon altitude and `include_horizon_pairs` are
/// used as given. The result is ordered by latitude, like [`parans`].
///
/// Errors with [`MapError::InvalidInput`] when the latitude is not strictly
/// between -90 and 90, the orb is negative or not finite, or as [`parans`].
pub fn parans_near(
    map: &AstroMap,
    lat_deg: f64,
    orb_deg: f64,
    options: &ParanOptions,
) -> Result<Vec<Paran>, MapError> {
    if !lat_deg.is_finite() || lat_deg.abs() >= 90.0 {
        return invalid("latitude must be strictly between -90 and 90 degrees");
    }
    if !orb_deg.is_finite() || orb_deg < 0.0 {
        return invalid("orb must be a finite number of degrees, zero or more");
    }
    const LIMIT: f64 = 90.0 - 1e-9;
    let band = ParanOptions {
        min_lat_deg: (lat_deg - orb_deg).max(-LIMIT),
        max_lat_deg: (lat_deg + orb_deg).min(LIMIT),
        ..*options
    };
    let mut found = parans(map, &band)?.parans;
    found.retain(|p| (p.latitude_deg - lat_deg).abs() <= orb_deg);
    Ok(found)
}
