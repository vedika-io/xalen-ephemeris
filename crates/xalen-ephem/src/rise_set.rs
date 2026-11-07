//! Rise, transit (upper culmination), set, and twilight times for a body and observer.
//!
//! Two search modes are provided:
//!
//! * [`compute`] — the legacy 24-hour *window* form: scans `[jd_start, jd_start + 1d]`
//!   and reports the first rise/set inside that window plus the window's culmination.
//! * [`next_events`] — Swiss-Ephemeris-style *forward* search: from `jd_start` it
//!   independently finds the **next** rise, the **next** transit, and the **next**
//!   set, each refined to sub-second precision. This matches `swe_rise_trans`
//!   semantics (each event found by searching strictly forward) and is what the
//!   pyswisseph cross-validation exercises.
//!
//! Altitude convention. A body is "risen" when the **topocentric** altitude of its
//! centre (the value [`Almanac::topocentric_altitude_deg`] returns, which already
//! applies diurnal parallax) equals the standard rise/set altitude `h0`:
//!
//! * Sun:    `h0 = −0.8333°`  (−34′ refraction − 16′ solar semidiameter, upper limb).
//! * Moon:   `h0 = −0.5667° − 0.2725·π`  (−34′ refraction − geometric semidiameter
//!   `SD = 0.2725·π`). NOTE this differs from Meeus 15.1's `+0.7275·π − 0.5667°`:
//!   Meeus applies the `+π` parallax lift because his altitude is **geocentric**,
//!   whereas ours is **topocentric** (parallax already folded in), so re-adding it
//!   would double-count. With this topocentric `h0` the residual against Swiss at
//!   the event time is ≈0.01° (≈6 s), versus ≈0.95° (≈6 min) with the geocentric
//!   form — confirmed by the pyswisseph cross-validation in `tests/`.
//! * Stars / planets: `h0 = −0.5667°` (−34′ refraction; the disc is a point).
//!
//! This is the convention `swe_rise_trans` uses with its default flags (upper limb +
//! standard refraction), so the altitudes computed here are directly comparable to
//! Swiss output. Twilight uses the Sun's centre at −6° (civil), −12° (nautical), and
//! −18° (astronomical), with no refraction or semidiameter term, per the standard
//! definitions.

use crate::almanac::Almanac;
use crate::body::Body;
use crate::provider::EphemerisError;
use crate::topocentric::EARTH_RADIUS_AU;
use xalen_time::{JdUT1, JulianDay};

/// Rise / transit / set for one body and observer over a 24-hour window.
#[derive(Debug, Clone, Copy)]
pub struct RiseTransitSet {
    /// Time the body's center crosses the rise altitude going up (UT1).
    pub rise: Option<JdUT1>,
    /// Time of upper culmination — maximum altitude (UT1).
    pub transit: Option<JdUT1>,
    /// Time the body's center crosses the set altitude going down (UT1).
    pub set: Option<JdUT1>,
    /// Altitude (degrees) at transit — distinguishes circumpolar from never-rises.
    pub transit_altitude_deg: f64,
    /// Body stays above the rise altitude for the whole window (circumpolar).
    pub always_above: bool,
    /// Body stays below the rise altitude for the whole window (never rises).
    pub always_below: bool,
}

/// Which twilight boundary to compute (Sun's geometric centre depression).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Twilight {
    /// Sun centre at −6°.
    Civil,
    /// Sun centre at −12°.
    Nautical,
    /// Sun centre at −18°.
    Astronomical,
}

impl Twilight {
    /// Sun-centre altitude (degrees) defining this twilight boundary.
    pub fn altitude_deg(self) -> f64 {
        match self {
            Twilight::Civil => -6.0,
            Twilight::Nautical => -12.0,
            Twilight::Astronomical => -18.0,
        }
    }
}

/// Morning (dawn) and evening (dusk) crossings of a twilight boundary, found by
/// searching forward from a reference instant.
#[derive(Debug, Clone, Copy)]
pub struct TwilightTimes {
    /// Next time the Sun centre rises through the boundary (dawn), UT1.
    pub dawn: Option<JdUT1>,
    /// Next time the Sun centre sinks through the boundary (dusk), UT1.
    pub dusk: Option<JdUT1>,
}

/// Refine a horizon crossing in `[lo, hi]` (where the residual `altitude − h0`
/// changes sign) to ~sub-second precision via bisection. `resid(t)` returns
/// `altitude(t) − h0(t)`; passing `h0` as a function of time lets the Moon's
/// distance-dependent semidiameter be evaluated exactly at the crossing.
fn bisect<F>(resid: &F, mut lo: f64, mut hi: f64) -> Result<f64, EphemerisError>
where
    F: Fn(f64) -> Result<f64, EphemerisError>,
{
    let lo_below = resid(lo)? < 0.0;
    // 44 halvings of a 1-day bracket → < 5e-9 day ≈ 0.4 ms; sample noise dominates.
    for _ in 0..44 {
        let mid = 0.5 * (lo + hi);
        if (resid(mid)? < 0.0) == lo_below {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Ok(0.5 * (lo + hi))
}

/// Bisect the local hour angle to zero in `[lo, hi]` (where `ha` changes sign from
/// negative=east to positive=west — i.e. an upper meridian transit). This is the
/// definition `swe_rise_trans` uses for `MTRANSIT`; for the fast-moving Moon it
/// differs from the altitude maximum by minutes (the declination shifts during the
/// daily arc), so locating HA=0 directly is what matches Swiss.
fn bisect_meridian<H>(ha: &H, mut lo: f64, mut hi: f64) -> Result<f64, EphemerisError>
where
    H: Fn(f64) -> Result<f64, EphemerisError>,
{
    let lo_neg = ha(lo)? < 0.0;
    for _ in 0..44 {
        let mid = 0.5 * (lo + hi);
        if (ha(mid)? < 0.0) == lo_neg {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Ok(0.5 * (lo + hi))
}

/// Build the topocentric-altitude closure for a body and observer.
fn altitude_fn<'a>(
    almanac: &'a Almanac,
    body: Body,
    lat: f64,
    lon: f64,
    elev: f64,
) -> impl Fn(f64) -> Result<f64, EphemerisError> + 'a {
    move |t: f64| almanac.topocentric_altitude_deg(body, JdUT1(t), lat, lon, elev)
}

/// Build the topocentric local-hour-angle closure (degrees, wrapped to (−180,180])
/// for a body and observer. Zero at upper meridian transit.
fn hour_angle_fn<'a>(
    almanac: &'a Almanac,
    body: Body,
    lat: f64,
    lon: f64,
    elev: f64,
) -> impl Fn(f64) -> Result<f64, EphemerisError> + 'a {
    move |t: f64| almanac.topocentric_hour_angle_deg(body, JdUT1(t), lat, lon, elev)
}

/// Search forward from `jd_start` for the **next** upper meridian transit (local
/// hour angle crossing 0 from negative=east to positive=west). Coarse-scans, then
/// bisects HA→0. Returns `(transit_jd, altitude_deg_at_transit)`.
fn next_meridian_transit<H, A>(
    ha: &H,
    alt: &A,
    jd_start: f64,
    step_min: f64,
    horizon_days: f64,
) -> Result<Option<(f64, f64)>, EphemerisError>
where
    H: Fn(f64) -> Result<f64, EphemerisError>,
    A: Fn(f64) -> Result<f64, EphemerisError>,
{
    let step = step_min / 1440.0;
    let n = (horizon_days / step).ceil() as usize;
    let mut prev_t = jd_start;
    let mut prev_h = ha(prev_t)?;
    for i in 1..=n {
        let t = jd_start + i as f64 * step;
        let h = ha(t)?;
        // Upper transit: HA rises through 0 (east → west). Guard against the
        // ±180° wrap (lower transit / antimeridian) where prev_h≈+180, h≈−180.
        if prev_h < 0.0 && h >= 0.0 && (h - prev_h).abs() < 180.0 {
            let tt = bisect_meridian(ha, prev_t, t)?;
            return Ok(Some((tt, alt(tt)?)));
        }
        prev_t = t;
        prev_h = h;
    }
    Ok(None)
}

/// Build the rise/set **residual** closure `altitude(t) − h0(t)` for a body. For the
/// Moon, `h0` tracks the (distance-dependent) semidiameter at each instant, so a
/// crossing of this residual is exact even when the event is ~1 day forward and the
/// lunar distance has changed. For the Sun and point sources `h0` is constant.
/// This is [`crossing_residual_fn`] with the default [`HorizonCrossing`].
fn residual_fn<'a>(
    almanac: &'a Almanac,
    body: Body,
    lat: f64,
    lon: f64,
    elev: f64,
) -> impl Fn(f64) -> Result<f64, EphemerisError> + 'a {
    crossing_residual_fn(almanac, body, lat, lon, elev, HorizonCrossing::default())
}

/// The residual `altitude(t) − target(t)` for a body under a
/// [`HorizonCrossing`]: positive when the body is above the crossing altitude.
fn crossing_residual_fn<'a>(
    almanac: &'a Almanac,
    body: Body,
    lat: f64,
    lon: f64,
    elev: f64,
    crossing: HorizonCrossing,
) -> impl Fn(f64) -> Result<f64, EphemerisError> + 'a {
    move |t: f64| {
        let jd = JdUT1(t);
        let alt = if crossing.geocentric_no_latitude {
            almanac.geocentric_ecliptic_altitude_deg(body, jd, lat, lon)?
        } else {
            almanac.topocentric_altitude_deg(body, jd, lat, lon, elev)?
        };
        let dist = if crossing.needs_distance(body) {
            almanac.geocentric_ecliptic(body, jd)?.distance
        } else {
            1.0 // unused for the Sun, point sources and a fixed lunar disc
        };
        Ok(alt - crossing.target_altitude_deg(body, dist))
    }
}

/// Compute rise/transit/set over the 24 hours starting at `jd_start` (UT1).
///
/// Window form: reports the first rise and first set inside `[jd_start, +1d]`, the
/// window's maximum-altitude culmination (refined to sub-second), and circumpolar /
/// never-rises flags. For Swiss-style "next event from now" use [`next_events`].
pub fn compute(
    almanac: &Almanac,
    body: Body,
    jd_start: JdUT1,
    lat_deg: f64,
    lon_deg: f64,
    elevation_m: f64,
) -> Result<RiseTransitSet, EphemerisError> {
    let alt = altitude_fn(almanac, body, lat_deg, lon_deg, elevation_m);
    let resid = residual_fn(almanac, body, lat_deg, lon_deg, elevation_m);
    let ha = hour_angle_fn(almanac, body, lat_deg, lon_deg, elevation_m);

    let t0 = jd_start.as_f64();
    let n = 144usize; // 10-minute samples across 24 h
    let step = 1.0 / n as f64;

    let mut prev_t = t0;
    let mut prev_r = resid(t0)?;
    let (mut rise, mut set) = (None, None);
    let mut min_r = prev_r;
    let mut max_r = prev_r;

    for i in 1..=n {
        let t = t0 + i as f64 * step;
        let r = resid(t)?;
        if r < min_r {
            min_r = r;
        }
        if r > max_r {
            max_r = r;
        }
        if rise.is_none() && prev_r < 0.0 && r >= 0.0 {
            rise = Some(JdUT1(bisect(&resid, prev_t, t)?));
        }
        if set.is_none() && prev_r >= 0.0 && r < 0.0 {
            set = Some(JdUT1(bisect(&resid, prev_t, t)?));
        }
        prev_t = t;
        prev_r = r;
    }

    // Meridian transit (hour angle = 0) — the Swiss MTRANSIT definition. This is the
    // moment of upper culmination; for the fast Moon it differs from the altitude
    // peak by minutes, so we locate HA=0 directly rather than maximising altitude.
    let transit = next_meridian_transit(&ha, &alt, t0, 10.0, 1.0)?;

    Ok(RiseTransitSet {
        rise,
        set,
        transit: transit.map(|(t, _)| JdUT1(t)),
        transit_altitude_deg: transit.map(|(_, a)| a).unwrap_or(f64::NAN),
        // Circumpolar: residual never dips below 0 in the window. Never-rises: the
        // residual stays below 0 across the whole window.
        always_above: min_r >= 0.0,
        always_below: max_r < 0.0,
    })
}

/// Search forward from `jd_start` for the **next** zero-crossing of the residual
/// `resid(t) = altitude(t) − h0(t)` in the given direction (`rising = true` → going
/// up, i.e. a rise/dawn). Coarse-scans in `step_min`-minute increments out to
/// `horizon_days`, then bisects the first sign change. Returns `None` if no crossing
/// occurs within the horizon (e.g. polar day/night).
fn next_crossing<F>(
    resid: &F,
    jd_start: f64,
    rising: bool,
    step_min: f64,
    horizon_days: f64,
) -> Result<Option<f64>, EphemerisError>
where
    F: Fn(f64) -> Result<f64, EphemerisError>,
{
    let step = step_min / 1440.0;
    let n = (horizon_days / step).ceil() as usize;
    let mut prev_t = jd_start;
    let mut prev_r = resid(prev_t)?;
    for i in 1..=n {
        let t = jd_start + i as f64 * step;
        let r = resid(t)?;
        let crossed = if rising {
            prev_r < 0.0 && r >= 0.0
        } else {
            prev_r >= 0.0 && r < 0.0
        };
        if crossed {
            return Ok(Some(bisect(resid, prev_t, t)?));
        }
        prev_t = t;
        prev_r = r;
    }
    Ok(None)
}

/// Swiss-style forward search: independently find the **next** rise, transit, and
/// set after `jd_start` (UT1). Each is refined to sub-second precision. This mirrors
/// `swe_rise_trans` (default flags = upper limb + standard refraction; MTRANSIT =
/// meridian hour-angle zero) and is the form validated against pyswisseph. The
/// rise/set altitude `h0` is evaluated at the event time itself (the Moon's
/// distance-dependent semidiameter is tracked exactly).
pub fn next_events(
    almanac: &Almanac,
    body: Body,
    jd_start: JdUT1,
    lat_deg: f64,
    lon_deg: f64,
    elevation_m: f64,
) -> Result<RiseTransitSet, EphemerisError> {
    let alt = altitude_fn(almanac, body, lat_deg, lon_deg, elevation_m);
    let resid = residual_fn(almanac, body, lat_deg, lon_deg, elevation_m);
    let ha = hour_angle_fn(almanac, body, lat_deg, lon_deg, elevation_m);
    let t0 = jd_start.as_f64();
    // 5-minute coarse scan over 1.2 days catches every rise/set/transit even when
    // the first event is ~1 sidereal day away (forward-search semantics).
    let (step_min, horizon) = (5.0, 1.2);

    let rise = next_crossing(&resid, t0, true, step_min, horizon)?.map(JdUT1);
    let set = next_crossing(&resid, t0, false, step_min, horizon)?.map(JdUT1);
    let transit = next_meridian_transit(&ha, &alt, t0, step_min, horizon)?;
    let transit_a = transit.map(|(_, a)| a).unwrap_or(f64::NAN);

    // Window flags from a quick 24h min/max residual sample (cheap, informational).
    let mut min_r = f64::INFINITY;
    let mut max_r = f64::NEG_INFINITY;
    for i in 0..=24 {
        let r = resid(t0 + i as f64 / 24.0)?;
        min_r = min_r.min(r);
        max_r = max_r.max(r);
    }

    Ok(RiseTransitSet {
        rise,
        transit: transit.map(|(t, _)| JdUT1(t)),
        set,
        transit_altitude_deg: transit_a,
        always_above: min_r >= 0.0,
        always_below: max_r < 0.0,
    })
}

/// Next dawn and dusk for a given twilight boundary, searching forward from
/// `jd_start` (UT1). Dawn = Sun centre rising through the boundary; dusk = Sun centre
/// sinking through it. Uses the Sun's centre at a fixed depression (no semidiameter
/// or parallax term — the standard twilight definition). Returns `None` for a leg
/// that does not occur within ~1.2 days (high-latitude polar day / night).
pub fn twilight(
    almanac: &Almanac,
    twilight: Twilight,
    jd_start: JdUT1,
    lat_deg: f64,
    lon_deg: f64,
    elevation_m: f64,
) -> Result<TwilightTimes, EphemerisError> {
    let h0 = twilight.altitude_deg();
    let alt = altitude_fn(almanac, Body::Sun, lat_deg, lon_deg, elevation_m);
    // Twilight h0 is a fixed depression: residual = altitude − h0.
    let resid = |t: f64| Ok(alt(t)? - h0);
    let t0 = jd_start.as_f64();
    let dawn = next_crossing(&resid, t0, true, 5.0, 1.2)?.map(JdUT1);
    let dusk = next_crossing(&resid, t0, false, 5.0, 1.2)?.map(JdUT1);
    Ok(TwilightTimes { dawn, dusk })
}

/// Standard horizontal refraction (degrees) of the default rise/set altitude:
/// 34′, valid for 1013.25 hPa and 15 °C.
const STANDARD_REFRACTION_DEG: f64 = 0.5667;
/// Solar semidiameter (degrees) of the default rise/set altitude
/// (`−0.8333° = −0.5667° − 0.2666°`).
const SUN_SEMIDIAMETER_DEG: f64 = 0.2666;
/// Ratio of the Moon's radius to Earth's equatorial radius (Moon semidiameter
/// `SD = 0.2725·π`).
const MOON_RADIUS_RATIO: f64 = 0.2725;
/// Mean Earth–Moon distance (384 400 km) in AU, for a fixed lunar disc size.
const MOON_MEAN_DISTANCE_AU: f64 = 384_400.0 / 149_597_870.7;
/// Pressure (hPa) and temperature (°C) at which the 34′ refraction applies.
const STANDARD_PRESSURE_HPA: f64 = 1013.25;
const STANDARD_TEMPERATURE_C: f64 = 15.0;

/// Which point of a body's disc defines a rise or a set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiscPoint {
    /// The upper limb touches the horizon (the astronomical convention).
    #[default]
    UpperLimb,
    /// The centre of the disc is on the horizon.
    Center,
    /// The lower limb touches the horizon.
    LowerLimb,
}

/// The conditions that define a horizon crossing for
/// [`next_horizon_crossing`].
///
/// The [`Default`] is the convention [`next_events`] uses: upper limb, 34′
/// refraction (1013.25 hPa, 15 °C), a true horizon at 0°, the Moon's disc size
/// from its actual distance, and no twilight depression. With the default the
/// two searches return the same instants (both use it). Its target altitude
/// for the body's topocentric centre is −0.8333° for the Sun (−34′ refraction
/// − 16′ semidiameter; solar parallax is below the timing floor),
/// `−0.5667° − 0.2725·π` for the Moon (no `+π` parallax lift, because the
/// altitude is already topocentric; see the module docs) and −0.5667° for
/// point sources.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HorizonCrossing {
    /// Which point of the disc is put on the horizon.
    pub disc: DiscPoint,
    /// Apply atmospheric refraction.
    pub refraction: bool,
    /// Air pressure at the observer in hPa (scales the refraction).
    pub pressure_hpa: f64,
    /// Air temperature at the observer in °C (scales the refraction).
    pub temperature_c: f64,
    /// Apparent altitude of the local horizon in degrees (0 = true horizon,
    /// positive for a raised horizon such as a mountain ridge).
    pub horizon_height_deg: f64,
    /// Use the Moon's disc at its mean distance instead of its actual one.
    pub fixed_disc_size: bool,
    /// When set and the body is the Sun, the crossing is the Sun's centre at
    /// this twilight depression (no refraction or disc term) and the disc,
    /// refraction and horizon fields are ignored. Twilight is defined for the
    /// Sun only (as in `swe_rise_trans`), so for every other body this field is
    /// ignored.
    pub twilight: Option<Twilight>,
    /// Use the body's geocentric position with its ecliptic latitude set to
    /// zero instead of its topocentric position (the "Hindu rising"
    /// convention; see [`Almanac::geocentric_ecliptic_altitude_deg`]).
    pub geocentric_no_latitude: bool,
}

/// Air pressure (hPa) of the ICAO standard atmosphere at `elevation_m` metres:
/// `1013.25 · (1 − 0.0065·h / 288.15)^5.255`. Used when a caller gives no
/// pressure.
pub fn standard_pressure_hpa(elevation_m: f64) -> f64 {
    let ratio = (1.0 - 0.0065 * elevation_m / 288.15).max(0.0);
    STANDARD_PRESSURE_HPA * ratio.powf(5.255)
}

impl Default for HorizonCrossing {
    fn default() -> Self {
        Self {
            disc: DiscPoint::UpperLimb,
            refraction: true,
            pressure_hpa: STANDARD_PRESSURE_HPA,
            temperature_c: STANDARD_TEMPERATURE_C,
            horizon_height_deg: 0.0,
            fixed_disc_size: false,
            twilight: None,
            geocentric_no_latitude: false,
        }
    }
}

/// Bennett's refraction (arcminutes) for an apparent altitude in degrees.
fn bennett_arcmin(apparent_alt_deg: f64) -> f64 {
    let h = apparent_alt_deg.max(-3.0);
    1.0 / (h + 7.31 / (h + 4.4)).to_radians().tan()
}

impl HorizonCrossing {
    /// Refraction (degrees) at the horizon altitude: the standard 34′ scaled by
    /// Bennett's altitude dependence and by pressure / absolute temperature.
    /// Zero when [`Self::refraction`] is off.
    pub fn refraction_deg(&self) -> f64 {
        if !self.refraction {
            return 0.0;
        }
        let shape = bennett_arcmin(self.horizon_height_deg) / bennett_arcmin(0.0);
        let air = (self.pressure_hpa / STANDARD_PRESSURE_HPA)
            * ((273.15 + STANDARD_TEMPERATURE_C) / (273.15 + self.temperature_c));
        STANDARD_REFRACTION_DEG * shape * air
    }

    /// Angular semidiameter (degrees) of the body's disc; zero for point sources.
    /// `dist_au` is the body's geocentric distance (used for the Moon).
    pub fn semidiameter_deg(&self, body: Body, dist_au: f64) -> f64 {
        match body {
            Body::Sun => SUN_SEMIDIAMETER_DEG,
            Body::Moon => {
                let d = if self.fixed_disc_size {
                    MOON_MEAN_DISTANCE_AU
                } else {
                    dist_au
                };
                MOON_RADIUS_RATIO * (EARTH_RADIUS_AU / d).asin().to_degrees()
            }
            _ => 0.0,
        }
    }

    /// Topocentric altitude (degrees) of the body's **centre** at the crossing.
    pub fn target_altitude_deg(&self, body: Body, dist_au: f64) -> f64 {
        if let Some(t) = self.solar_twilight(body) {
            return t.altitude_deg();
        }
        let limb = match self.disc {
            DiscPoint::UpperLimb => -self.semidiameter_deg(body, dist_au),
            DiscPoint::Center => 0.0,
            DiscPoint::LowerLimb => self.semidiameter_deg(body, dist_au),
        };
        self.horizon_height_deg - self.refraction_deg() + limb
    }

    /// The twilight depression that applies to `body`: [`Self::twilight`] for
    /// the Sun, `None` for every other body.
    fn solar_twilight(&self, body: Body) -> Option<Twilight> {
        if body == Body::Sun {
            self.twilight
        } else {
            None
        }
    }

    /// Whether the target depends on the Moon's distance at each instant.
    fn needs_distance(&self, body: Body) -> bool {
        body == Body::Moon && self.disc != DiscPoint::Center && !self.fixed_disc_size
    }
}

/// Dip of the sea horizon (degrees, negative) seen from `elevation_m` metres:
/// `−0.0293°·√h`, i.e. 1.76′·√h, which includes standard terrestrial
/// refraction. Zero at or below sea level. This is the horizon
/// `swe_rise_trans_true_hor` uses when it is given a horizon height of −100.
pub fn horizon_dip_deg(elevation_m: f64) -> f64 {
    -0.0293 * elevation_m.max(0.0).sqrt()
}

/// Atmospheric refraction (degrees) for a body at **true** (airless)
/// altitude `true_alt_deg`, from Saemundsson's formula scaled by pressure and
/// absolute temperature. Below −1° the value is held at its −1° level, so the
/// apparent altitude stays continuous and increasing.
pub fn refraction_from_true_deg(true_alt_deg: f64, pressure_hpa: f64, temperature_c: f64) -> f64 {
    let h = true_alt_deg.max(-1.0);
    let arcmin = 1.02 / (h + 10.3 / (h + 5.11)).to_radians().tan();
    let air = (pressure_hpa / 1010.0) * (283.0 / (273.0 + temperature_c));
    (arcmin * air / 60.0).max(0.0)
}

/// Apparent (refracted) altitude in degrees of a body at true altitude
/// `true_alt_deg`; see [`refraction_from_true_deg`].
pub fn apparent_altitude_deg(true_alt_deg: f64, pressure_hpa: f64, temperature_c: f64) -> f64 {
    true_alt_deg + refraction_from_true_deg(true_alt_deg, pressure_hpa, temperature_c)
}

/// Search forward from `jd_start` (UT1) for the **next** rise (`rising = true`)
/// or set of `body` under the given [`HorizonCrossing`], refined to sub-second
/// precision. Returns `None` when no such crossing occurs within ~1.2 days
/// (the body stays above or below the crossing altitude: polar day or night,
/// circumpolar stars).
pub fn next_horizon_crossing(
    almanac: &Almanac,
    body: Body,
    jd_start: JdUT1,
    lat_deg: f64,
    lon_deg: f64,
    elevation_m: f64,
    rising: bool,
    crossing: &HorizonCrossing,
) -> Result<Option<JdUT1>, EphemerisError> {
    let resid = crossing_residual_fn(almanac, body, lat_deg, lon_deg, elevation_m, *crossing);
    Ok(next_crossing(&resid, jd_start.as_f64(), rising, 5.0, 1.2)?.map(JdUT1))
}

/// Search forward from `jd_start` (UT1) for the next **upper** (`upper = true`,
/// local hour angle 0) or **lower** (hour angle 180°) meridian transit of
/// `body`. Returns `None` only if no transit occurs within ~1.2 days.
pub fn next_transit(
    almanac: &Almanac,
    body: Body,
    jd_start: JdUT1,
    lat_deg: f64,
    lon_deg: f64,
    elevation_m: f64,
    upper: bool,
) -> Result<Option<JdUT1>, EphemerisError> {
    let alt = altitude_fn(almanac, body, lat_deg, lon_deg, elevation_m);
    let ha = hour_angle_fn(almanac, body, lat_deg, lon_deg, elevation_m);
    let t0 = jd_start.as_f64();
    let found = if upper {
        next_meridian_transit(&ha, &alt, t0, 5.0, 1.2)?
    } else {
        // Lower transit: shift the hour angle by 180° so that HA = 180° maps to
        // the same "rises through zero" condition as an upper transit.
        let ha_lower = |t: f64| -> Result<f64, EphemerisError> {
            let mut h = (ha(t)? + 180.0).rem_euclid(360.0);
            if h > 180.0 {
                h -= 360.0;
            }
            Ok(h)
        };
        next_meridian_transit(&ha_lower, &alt, t0, 5.0, 1.2)?
    };
    Ok(found.map(|(t, _)| JdUT1(t)))
}

/// Whether `body` is above the standard rise/set altitude (upper limb, 34′
/// refraction) for the observer at `jd` (UT1), i.e. whether it can be seen.
pub fn is_above_horizon(
    almanac: &Almanac,
    body: Body,
    jd: JdUT1,
    lat_deg: f64,
    lon_deg: f64,
    elevation_m: f64,
) -> Result<bool, EphemerisError> {
    let resid = residual_fn(almanac, body, lat_deg, lon_deg, elevation_m);
    Ok(resid(jd.as_f64())? >= 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn secs(a: JdUT1, b: f64) -> f64 {
        (a.as_f64() - b).abs() * 86400.0
    }

    // Swiss oracle (pyswisseph 2.10.03, FLG_MOSEPH, default flags = upper limb +
    // refraction, press=1013.25 temp=15), forward search from julday(y,m,d,0.0) UT.
    // Each tuple: (lat, lon, elev, jd_start, body, rise, transit, set) in JD UT.
    // Generated by tests/data/swiss_rise_set_oracle.json — see that file for the
    // exact re-runnable pyswisseph command.

    /// One Swiss oracle row: (label, lat, lon, elev, jd_start, body, rise, transit, set).
    type Row = (&'static str, f64, f64, f64, f64, Body, f64, f64, f64);

    /// All 14 Swiss oracle rows (Sun+Moon × Delhi/London/Quito × Jun/Dec/Mar
    /// solstices+equinox). Values copied verbatim from
    /// `tests/data/swiss_rise_set_oracle.json` (the re-runnable pyswisseph command
    /// is recorded in that file's header). NOT typed from memory.
    const ORACLE: &[Row] = &[
        (
            "Delhi-Jun-Sun",
            28.6139,
            77.2090,
            216.0,
            2460482.5,
            Body::Sun,
            2460483.496030195,
            2460482.786833312,
            2460483.0777908363,
        ),
        (
            "Delhi-Jun-Moon",
            28.6139,
            77.2090,
            216.0,
            2460482.5,
            Body::Moon,
            2460483.065541837,
            2460483.276299793,
            2460483.4865981843,
        ),
        (
            "Delhi-Dec-Sun",
            28.6139,
            77.2090,
            216.0,
            2460665.5,
            Body::Sun,
            2460665.569261926,
            2460665.7842695178,
            2460665.99927599,
        ),
        (
            "Delhi-Dec-Moon",
            28.6139,
            77.2090,
            216.0,
            2460665.5,
            Body::Moon,
            2460666.243577112,
            2460666.5097756856,
            2460665.752418408,
        ),
        (
            "Delhi-Mar-Sun",
            28.6139,
            77.2090,
            216.0,
            2460389.5,
            Body::Sun,
            2460389.5381450094,
            2460389.7906495747,
            2460390.0434560175,
        ),
        (
            "Delhi-Mar-Moon",
            28.6139,
            77.2090,
            216.0,
            2460389.5,
            Body::Moon,
            2460389.864726836,
            2460390.1616545147,
            2460390.4542521285,
        ),
        (
            "London-Jun-Sun",
            51.5074,
            -0.1276,
            11.0,
            2460482.5,
            Body::Sun,
            2460482.6550674746,
            2460483.001689346,
            2460483.348295645,
        ),
        (
            "London-Jun-Moon",
            51.5074,
            -0.1276,
            11.0,
            2460482.5,
            Body::Moon,
            2460483.363964687,
            2460483.5000201254,
            2460482.6002419777,
        ),
        (
            "London-Dec-Sun",
            51.5074,
            -0.1276,
            11.0,
            2460665.5,
            Body::Sun,
            2460665.836117957,
            2460665.9991674577,
            2460666.1622180315,
        ),
        (
            "London-Dec-Moon",
            51.5074,
            -0.1276,
            11.0,
            2460665.5,
            Body::Moon,
            2460666.4562710966,
            2460665.7025849028,
            2460665.9867603537,
        ),
        (
            "Quito-Jun-Sun",
            -0.1807,
            -78.4678,
            2850.0,
            2460482.5,
            Body::Sun,
            2460482.9670312866,
            2460483.2193337195,
            2460483.4716355144,
        ),
        (
            "Quito-Jun-Moon",
            -0.1807,
            -78.4678,
            2850.0,
            2460482.5,
            Body::Moon,
            2460483.4664642387,
            2460482.6856126394,
            2460482.945551798,
        ),
        (
            "Quito-Dec-Sun",
            -0.1807,
            -78.4678,
            2850.0,
            2460665.5,
            Body::Sun,
            2460665.9640411884,
            2460666.2168543357,
            2460666.469666951,
        ),
        (
            "Quito-Dec-Moon",
            -0.1807,
            -78.4678,
            2850.0,
            2460665.5,
            Body::Moon,
            2460665.66966709,
            2460665.9263858595,
            2460666.183046383,
        ),
    ];

    #[test]
    fn next_events_matches_swiss_all_cases() {
        // Tolerance is the measured worst case across all rows (rise/set ~6.5 s,
        // transit ~1.9 s) with headroom: the residual is dominated by VSOP87/ELP vs
        // DE ephemeris and ΔT-model differences, not by the root-finder.
        let a = Almanac::default_vedic();
        let (mut mr, mut mt, mut ms) = (0.0f64, 0.0f64, 0.0f64);
        for &(name, lat, lon, el, jd, body, sr, st, ss) in ORACLE {
            let r = a
                .rise_transit_set_next(body, JdUT1(jd), lat, lon, el)
                .unwrap();
            let dr = secs(r.rise.unwrap(), sr);
            let dt = secs(r.transit.unwrap(), st);
            let ds = secs(r.set.unwrap(), ss);
            assert!(dr < 15.0, "{name}: rise off by {dr:.2}s");
            assert!(dt < 15.0, "{name}: transit off by {dt:.2}s");
            assert!(ds < 15.0, "{name}: set off by {ds:.2}s");
            mr = mr.max(dr);
            mt = mt.max(dt);
            ms = ms.max(ds);
        }
        // Lock in the achieved agreement so a regression that loosens it is caught.
        assert!(
            mr < 8.0 && mt < 5.0 && ms < 8.0,
            "worst-case agreement regressed: rise={mr:.2}s transit={mt:.2}s set={ms:.2}s"
        );
    }

    #[test]
    fn transit_is_meridian_crossing() {
        // The reported transit must sit at hour angle ≈ 0 (the Swiss MTRANSIT
        // definition), not merely the altitude maximum. For the Moon these differ by
        // minutes; HA must be within a few arcseconds of zero at the reported time.
        let a = Almanac::default_vedic();
        for body in [Body::Sun, Body::Moon] {
            let r = a
                .rise_transit_set_next(body, JdUT1(2460665.5), 51.5074, -0.1276, 11.0)
                .unwrap();
            let tt = r.transit.unwrap().as_f64();
            let ha = a
                .topocentric_hour_angle_deg(body, JdUT1(tt), 51.5074, -0.1276, 11.0)
                .unwrap();
            assert!(
                ha.abs() < 0.01,
                "{body:?} transit not at meridian: HA = {ha} deg"
            );
        }
    }

    #[test]
    fn civil_twilight_brackets_sunrise() {
        // Civil dawn must precede sunrise, and civil dusk must follow sunset.
        let a = Almanac::default_vedic();
        let jd = JdUT1(2460482.5); // Delhi 2024-06-21 00:00 UT
        let rts = next_events(&a, Body::Sun, jd, 28.6139, 77.2090, 216.0).unwrap();
        let tw = twilight(&a, Twilight::Civil, jd, 28.6139, 77.2090, 216.0).unwrap();
        let dawn = tw.dawn.unwrap().as_f64();
        let rise = rts.rise.unwrap().as_f64();
        assert!(
            dawn < rise,
            "civil dawn {dawn} should precede sunrise {rise}"
        );
        // Dawn within ~40 min before sunrise at this latitude/season.
        assert!(
            (rise - dawn) * 1440.0 < 60.0,
            "civil dawn too early: {} min",
            (rise - dawn) * 1440.0
        );
    }

    #[test]
    fn twilight_ordering_civil_nautical_astronomical() {
        // For a forward search, dawn(astronomical) < dawn(nautical) < dawn(civil):
        // the Sun reaches the deeper depression earlier in the morning.
        let a = Almanac::default_vedic();
        let jd = JdUT1(2460665.5); // London 2024-12-21 00:00 UT (clear ordering)
        let civ = twilight(&a, Twilight::Civil, jd, 51.5074, -0.1276, 11.0).unwrap();
        let nau = twilight(&a, Twilight::Nautical, jd, 51.5074, -0.1276, 11.0).unwrap();
        let ast = twilight(&a, Twilight::Astronomical, jd, 51.5074, -0.1276, 11.0).unwrap();
        let (c, n, s) = (
            civ.dawn.unwrap().as_f64(),
            nau.dawn.unwrap().as_f64(),
            ast.dawn.unwrap().as_f64(),
        );
        assert!(
            s < n && n < c,
            "dawn order astro<naut<civil failed: astro={s} naut={n} civil={c}"
        );
    }

    #[test]
    fn polar_summer_sun_never_sets() {
        // Longyearbyen (78.2°N) on the June solstice: the midnight Sun is up for the
        // whole day, so there is no set within the window and `always_above` holds.
        // The forward search must return None for the set leg without panicking.
        let a = Almanac::default_vedic();
        let r = a
            .rise_transit_set(Body::Sun, JdUT1(2460482.5), 78.22, 15.65, 0.0)
            .unwrap();
        assert!(r.always_above, "polar summer Sun should be circumpolar");
        assert!(!r.always_below);
        assert!(r.transit.is_some(), "a meridian transit still occurs daily");
        // next_events likewise yields no rise/set crossing in the polar-day window.
        let n = a
            .rise_transit_set_next(Body::Sun, JdUT1(2460482.5), 78.22, 15.65, 0.0)
            .unwrap();
        assert!(
            n.rise.is_none() && n.set.is_none(),
            "no horizon crossing during polar day"
        );
    }

    #[test]
    fn default_horizon_crossing_is_the_standard_altitude() {
        // The classic rise/set altitudes: Sun −0.8333°, point sources −0.5667°,
        // Moon −0.5667° − 0.2725·π (topocentric, so no +π lift).
        let c = HorizonCrossing::default();
        let moon_dist = 0.00257;
        let moon_want = -0.5667 - 0.2725 * (EARTH_RADIUS_AU / moon_dist).asin().to_degrees();
        for (body, dist, want) in [
            (Body::Sun, 1.0, -0.8333),
            (Body::Mars, 1.5, -0.5667),
            (Body::Moon, moon_dist, moon_want),
        ] {
            let got = c.target_altitude_deg(body, dist);
            assert!((got - want).abs() < 1e-12, "{body:?}: {got} vs {want}");
        }
    }

    #[test]
    fn twilight_applies_to_the_sun_only() {
        let civil = HorizonCrossing {
            twilight: Some(Twilight::Civil),
            ..HorizonCrossing::default()
        };
        let plain = HorizonCrossing::default();
        assert_eq!(civil.target_altitude_deg(Body::Sun, 1.0), -6.0);
        for body in [Body::Moon, Body::Mars] {
            assert_eq!(
                civil.target_altitude_deg(body, 0.00257),
                plain.target_altitude_deg(body, 0.00257),
                "{body:?}"
            );
        }
        assert!(civil.needs_distance(Body::Moon));
    }

    #[test]
    fn horizon_dip_grows_with_the_square_root_of_height() {
        assert_eq!(horizon_dip_deg(0.0), 0.0);
        assert_eq!(horizon_dip_deg(-10.0), 0.0);
        // 1.76′·√100 = 17.6′ = 0.293°.
        assert!((horizon_dip_deg(100.0) + 0.293).abs() < 1e-9);
        assert!((horizon_dip_deg(400.0) - 2.0 * horizon_dip_deg(100.0)).abs() < 1e-12);
    }

    #[test]
    fn apparent_altitude_adds_saemundsson_refraction() {
        // About 29′ at the true horizon for 1013.25 hPa and 15 °C, a few
        // arcseconds at 60°, none at the zenith, and continuous below −1°.
        let r0 = refraction_from_true_deg(0.0, 1013.25, 15.0) * 60.0;
        assert!(
            (27.5..30.0).contains(&r0),
            "refraction at 0 deg = {r0} arcmin"
        );
        let r60 = refraction_from_true_deg(60.0, 1013.25, 15.0) * 3600.0;
        assert!(
            (25.0..40.0).contains(&r60),
            "refraction at 60 deg = {r60} arcsec"
        );
        assert!(refraction_from_true_deg(90.0, 1013.25, 15.0) * 3600.0 < 1.0);
        let below = apparent_altitude_deg(-1.0 - 1e-9, 1013.25, 15.0);
        let at = apparent_altitude_deg(-1.0, 1013.25, 15.0);
        assert!((at - below).abs() < 1e-6);
        assert!(apparent_altitude_deg(-0.5, 1013.25, 15.0) > at);
        // Thinner, warmer air refracts less.
        let thin = refraction_from_true_deg(0.0, 900.0, 30.0);
        assert!(thin < refraction_from_true_deg(0.0, 1013.25, 15.0));
    }

    #[test]
    fn default_horizon_crossing_matches_next_events() {
        let a = Almanac::default_vedic();
        let jd = JdUT1(2460482.5);
        for body in [Body::Sun, Body::Moon] {
            let n = next_events(&a, body, jd, 28.6139, 77.2090, 216.0).unwrap();
            let c = HorizonCrossing::default();
            let rise =
                next_horizon_crossing(&a, body, jd, 28.6139, 77.2090, 216.0, true, &c).unwrap();
            let set =
                next_horizon_crossing(&a, body, jd, 28.6139, 77.2090, 216.0, false, &c).unwrap();
            let transit = next_transit(&a, body, jd, 28.6139, 77.2090, 216.0, true).unwrap();
            assert!(secs(rise.unwrap(), n.rise.unwrap().as_f64()) < 0.01);
            assert!(secs(set.unwrap(), n.set.unwrap().as_f64()) < 0.01);
            assert!(secs(transit.unwrap(), n.transit.unwrap().as_f64()) < 0.01);
        }
    }

    #[test]
    fn disc_centre_without_refraction_rises_later_and_sets_earlier() {
        let a = Almanac::default_vedic();
        let jd = JdUT1(2460482.5);
        let standard = HorizonCrossing::default();
        let geo = HorizonCrossing {
            disc: DiscPoint::Center,
            refraction: false,
            ..HorizonCrossing::default()
        };
        let at = |c: &HorizonCrossing, rising: bool| {
            next_horizon_crossing(&a, Body::Sun, jd, 28.6139, 77.2090, 0.0, rising, c)
                .unwrap()
                .unwrap()
                .as_f64()
        };
        // The target altitude rises from −0.8333° to 0°: about 3-4 minutes at Delhi.
        let rise_delay = (at(&geo, true) - at(&standard, true)) * 1440.0;
        let set_advance = (at(&standard, false) - at(&geo, false)) * 1440.0;
        assert!(
            (2.5..5.0).contains(&rise_delay),
            "rise delay {rise_delay} min"
        );
        assert!(
            (2.5..5.0).contains(&set_advance),
            "set advance {set_advance} min"
        );
    }

    #[test]
    fn raised_horizon_delays_rise_and_cuts_refraction() {
        let raised = HorizonCrossing {
            horizon_height_deg: 2.0,
            ..HorizonCrossing::default()
        };
        // Refraction at 2° apparent altitude is about 18′, roughly half of 34′.
        let r = raised.refraction_deg() * 60.0;
        assert!(
            (16.0..20.0).contains(&r),
            "refraction at 2 deg = {r} arcmin"
        );
        let a = Almanac::default_vedic();
        let jd = JdUT1(2460482.25); // 2024-06-20 23:30 IST, before the next sunrise
        let standard = next_horizon_crossing(
            &a,
            Body::Sun,
            jd,
            28.6139,
            77.2090,
            0.0,
            true,
            &HorizonCrossing::default(),
        )
        .unwrap()
        .unwrap();
        let high = next_horizon_crossing(&a, Body::Sun, jd, 28.6139, 77.2090, 0.0, true, &raised)
            .unwrap()
            .unwrap();
        assert!(high.as_f64() > standard.as_f64() + 5.0 / 1440.0);
    }

    #[test]
    fn standard_pressure_falls_with_height() {
        assert!((standard_pressure_hpa(0.0) - 1013.25).abs() < 1e-9);
        // ICAO standard atmosphere: about 898.7 hPa at 1000 m.
        assert!((standard_pressure_hpa(1000.0) - 898.7).abs() < 0.5);
    }

    #[test]
    fn hindu_rising_sun_matches_disc_centre_without_refraction() {
        // The Sun's ecliptic latitude and parallax are both arcseconds, so the
        // geocentric no-latitude convention moves its rise by well under a second.
        let a = Almanac::default_vedic();
        let jd = JdUT1(2460482.5);
        let centre = HorizonCrossing {
            disc: DiscPoint::Center,
            refraction: false,
            ..HorizonCrossing::default()
        };
        let hindu = HorizonCrossing {
            geocentric_no_latitude: true,
            ..centre
        };
        let r1 = next_horizon_crossing(&a, Body::Sun, jd, 28.6139, 77.2090, 0.0, true, &centre)
            .unwrap()
            .unwrap();
        let r2 = next_horizon_crossing(&a, Body::Sun, jd, 28.6139, 77.2090, 0.0, true, &hindu)
            .unwrap()
            .unwrap();
        assert!(secs(r1, r2.as_f64()) < 2.0);
    }

    #[test]
    fn lower_transit_is_half_a_day_from_upper() {
        let a = Almanac::default_vedic();
        let jd = JdUT1(2460482.5);
        let up = next_transit(&a, Body::Sun, jd, 28.6139, 77.2090, 0.0, true)
            .unwrap()
            .unwrap()
            .as_f64();
        let low = next_transit(&a, Body::Sun, jd, 28.6139, 77.2090, 0.0, false)
            .unwrap()
            .unwrap()
            .as_f64();
        let gap_h = ((low - up).abs() * 24.0 - 12.0).abs();
        assert!(
            gap_h < 0.05,
            "upper/lower transit gap off 12 h by {gap_h} h"
        );
    }
}
