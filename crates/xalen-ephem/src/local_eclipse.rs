//! Local (per-observer) circumstances of a solar eclipse.
//!
//! Builds the OBSERVER REDUCTION on top of the geocentric Besselian engine
//! ([`crate::besselian`]). Given the Besselian elements `(x, y, d, l1, l2,
//! tan f1, tan f2)` and the Greenwich apparent sidereal time, the observer's
//! position in the fundamental plane `(ξ, η, ζ)` is computed, from which we
//! derive — for one location — the magnitude, obscuration, and the four
//! contact instants C1–C4 (Explanatory Supplement to the Astronomical Almanac,
//! 3rd ed. §11.3; Meeus, *Astronomical Algorithms* 2nd ed. Ch. 54).
//!
//! # Method (closed-form, no external data)
//!
//! For a station at geographic latitude φ, east longitude λ, height h, form the
//! geocentric quantities ρ·sin φ′ and ρ·cos φ′ (Meeus Ch. 11). The Besselian
//! ephemeris hour angle of the shadow axis is
//!
//! ```text
//!   H = (GAST + λ) − a            (a = right ascension of the shadow axis)
//! ```
//!
//! where `a` is recovered from the elements by `a = α_axis`. We obtain `a`
//! directly from the Besselian computation (it is the RA of the Sun−Moon
//! vector). The observer's fundamental-plane coordinates are
//!
//! ```text
//!   ξ = ρ·cos φ′ · sin H
//!   η = ρ·sin φ′ · cos d − ρ·cos φ′ · sin d · cos H
//!   ζ = ρ·sin φ′ · sin d + ρ·cos φ′ · cos d · cos H
//! ```
//!
//! The observer-to-axis separation in the plane is `m = √((x−ξ)² + (y−η)²)`.
//! The shadow radii at the observer's ζ-level are
//!
//! ```text
//!   L1′ = l1 − ζ·tan f1     (penumbra)
//!   L2′ = l2 − ζ·tan f2     (umbra/antumbra)
//! ```
//!
//! and the eclipse magnitude (fraction of the solar DIAMETER covered) is
//!
//! ```text
//!   mag = (L1′ − m) / (L1′ + L2′).
//! ```
//!
//! Contacts are the instants where `m` equals a shadow radius:
//! C1/C4 (penumbral, partial start/end) where `m = L1′`, and C2/C3 (umbral,
//! total/annular start/end) where `m = |L2′|`. They are found by bracketing the
//! sign change of `m − radius` around maximum eclipse and bisecting.
//!
//! # Validation
//!
//! The 2017-08-21 total eclipse at Madras, Oregon is cross-checked against the
//! Swiss Ephemeris `sol_eclipse_when_loc`/`sol_eclipse_how` oracle (see the test
//! module) and against the NASA Five Millennium Catalog global circumstances.

use crate::Almanac;
use crate::besselian::{BesselianElements, besselian_elements};
use std::f64::consts::TAU;
use xalen_coords::{ecliptic_to_equatorial, mean_obliquity, nutation_2000b};
use xalen_time::{DeltaTModel, JdTT, JdUT1, JulianDay, delta_t};

/// Earth equatorial radius in metres (IERS), matching `topocentric.rs`.
const EARTH_RADIUS_M: f64 = 6_378_136.6;
/// Polar/equatorial axis ratio b/a for the reference ellipsoid (Meeus Ch. 11).
const FLATTENING_BA: f64 = 0.996_647_19;
/// 1 AU in Earth equatorial radii (must match `besselian.rs`).
const AU_IN_EARTH_RADII: f64 = 149_597_870.7 / 6378.137;

/// Per-observer eclipse type at one location.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalSolarType {
    /// The umbra reaches the observer — totality (`L2′ < 0`, `m ≤ |L2′|`).
    Total,
    /// The antumbra reaches the observer — annularity (`L2′ > 0`, `m ≤ L2′`).
    Annular,
    /// Only the penumbra reaches the observer — a partial eclipse.
    Partial,
    /// No part of any shadow reaches the observer at maximum.
    None,
}

/// One contact instant (UT1) plus the body's altitude there (a contact below
/// the horizon is not actually observable, so the altitude is reported).
#[derive(Debug, Clone, Copy)]
pub struct Contact {
    /// Julian Date (UT1) of the contact.
    pub jd_ut: f64,
    /// Sun altitude (degrees) at the contact (negative = below horizon).
    pub altitude_deg: f64,
}

/// Local circumstances of a solar eclipse for one observer.
#[derive(Debug, Clone, Copy)]
pub struct LocalCircumstances {
    /// Per-observer eclipse type at this location.
    pub eclipse_type: LocalSolarType,
    /// Instant of maximum eclipse for this observer (UT1).
    pub jd_maximum_ut: f64,
    /// Eclipse magnitude at maximum (fraction of the solar DIAMETER covered).
    /// `> 1` for total/annular at the moment the limbs are internally tangent.
    pub magnitude: f64,
    /// Obscuration: fraction of the solar AREA covered at maximum (0..=1).
    pub obscuration: f64,
    /// First contact C1 — partial begins (penumbra first touches).
    pub c1: Option<Contact>,
    /// Second contact C2 — totality/annularity begins (umbra/antumbra touches).
    pub c2: Option<Contact>,
    /// Third contact C3 — totality/annularity ends.
    pub c3: Option<Contact>,
    /// Fourth contact C4 — partial ends (penumbra last touches).
    pub c4: Option<Contact>,
    /// Sun altitude (degrees) at maximum eclipse.
    pub altitude_at_max_deg: f64,
}

/// Observer geocentric quantities ρ·sin φ′ and ρ·cos φ′ for the reference
/// ellipsoid (Meeus Ch. 11), given geographic latitude and height.
fn observer_rho(lat_deg: f64, elevation_m: f64) -> (f64, f64) {
    let lat = lat_deg.to_radians();
    let u = (FLATTENING_BA * lat.tan()).atan();
    let h_over_r = elevation_m / EARTH_RADIUS_M;
    let rho_sin = FLATTENING_BA * u.sin() + h_over_r * lat.sin();
    let rho_cos = u.cos() + h_over_r * lat.cos();
    (rho_sin, rho_cos)
}

/// Greenwich APPARENT sidereal time (radians) for a UT1 Julian Day: GMST (IAU
/// 1982) plus the equation of the equinoxes `Δψ·cos ε_true`.
///
/// Delegates to the canonical [`xalen_coords::gast_rad`] (the single source of
/// truth for sidereal time). The equation-of-equinoxes argument `t` is derived
/// from UT1 here, preserving this engine's historical behaviour exactly — the
/// UT1-vs-TT difference in the slow-varying nutation terms is far below the
/// sub-millisecond precision the eclipse contact search resolves.
fn gast_rad(jd_ut1: f64) -> f64 {
    let t = (jd_ut1 - 2_451_545.0) / 36525.0;
    xalen_coords::gast_rad(jd_ut1, t)
}

/// Right ascension `a` of the shadow axis (Sun−Moon direction) at a TT instant.
/// This is the geocentric axis RA the Besselian elements are referred to; the
/// observer's hour angle of the axis is `H = (GAST + λ) − a`.
fn shadow_axis_ra(almanac: &Almanac, jd_tt: JdTT) -> Option<f64> {
    let t = jd_tt.julian_centuries_from_j2000();
    let eps = mean_obliquity(t) + nutation_2000b(t).delta_epsilon;
    let sun_ecl = almanac
        .geocentric_ecliptic_tt(crate::Body::Sun, jd_tt)
        .ok()?;
    let moon_ecl = almanac
        .geocentric_ecliptic_tt(crate::Body::Moon, jd_tt)
        .ok()?;
    let sun = ecliptic_to_equatorial(&sun_ecl, eps);
    let moon = ecliptic_to_equatorial(&moon_ecl, eps);
    let r_sun = sun_ecl.distance * AU_IN_EARTH_RADII;
    let r_moon = moon_ecl.distance * AU_IN_EARTH_RADII;
    let (sa_s, ca_s) = sun.right_ascension.sin_cos();
    let cd_s = sun.declination.cos();
    let (sa_m, ca_m) = moon.right_ascension.sin_cos();
    let cd_m = moon.declination.cos();
    let gx = r_sun * cd_s * ca_s - r_moon * cd_m * ca_m;
    let gy = r_sun * cd_s * sa_s - r_moon * cd_m * sa_m;
    Some(gy.atan2(gx))
}

/// Observer position `(ξ, η, ζ)` in the fundamental plane (Earth radii) and the
/// observer-to-axis separation `m`, the penumbral radius `l1'` and umbral radius
/// `l2'` at the observer's ζ-level, for one instant.
struct ObserverState {
    m: f64,
    l1p: f64,
    l2p: f64,
}

fn observer_state(
    b: &BesselianElements,
    a_axis: f64,
    gast: f64,
    lon_rad: f64,
    rho_sin: f64,
    rho_cos: f64,
) -> ObserverState {
    let h_angle = (gast + lon_rad - a_axis).rem_euclid(TAU);
    let (sin_h, cos_h) = h_angle.sin_cos();
    let (sd, cd) = b.d.sin_cos();
    let xi = rho_cos * sin_h;
    let eta = rho_sin * cd - rho_cos * sd * cos_h;
    let zeta = rho_sin * sd + rho_cos * cd * cos_h;
    let u = b.x - xi;
    let v = b.y - eta;
    let m = (u * u + v * v).sqrt();
    let l1p = b.l1 - zeta * b.tan_f1;
    let l2p = b.l2 - zeta * b.tan_f2;
    ObserverState { m, l1p, l2p }
}

/// Eclipse magnitude (fraction of the solar diameter covered) from the
/// observer state. `(l1' − m) / (l1' + l2')`, clamped at 0.
fn magnitude_from_state(s: &ObserverState) -> f64 {
    let mag = (s.l1p - s.m) / (s.l1p + s.l2p);
    mag.max(0.0)
}

/// Fraction of the solar AREA obscured given the diameter-magnitude and the
/// ratio of Moon to Sun apparent radii. Uses the standard lens-overlap formula
/// (Explanatory Supplement §11.3): the obscuration is the overlap area of two
/// circles of radii `r_sun` and `r_moon` whose centres are separated by `m`.
fn obscuration(s: &ObserverState) -> f64 {
    // Apparent Sun/Moon radii in the fundamental plane from the shadow radii.
    // The penumbral radius is the SUM of the projected radii, and the (signed)
    // umbral radius is r_sun − r_moon:
    //     l1' = r_sun + r_moon,   l2' = r_sun − r_moon
    //   ⇒ r_sun = (l1' + l2')/2,  r_moon = (l1' − l2')/2.
    // For a total eclipse l2' < 0 ⇒ r_moon > r_sun (Moon disk larger), so the
    // Sun is fully covered and obscuration → 1; for annular l2' > 0 ⇒ the Moon
    // disk is smaller and a ring of Sun remains.
    let r_sun = 0.5 * (s.l1p + s.l2p);
    let r_moon = 0.5 * (s.l1p - s.l2p);
    let d = s.m;
    let (rs, rm) = (r_sun.abs(), r_moon.abs());
    if d >= rs + rm {
        return 0.0; // no overlap
    }
    if d <= (rm - rs).abs() {
        // One disk fully inside the other.
        return if rm >= rs { 1.0 } else { (rm * rm) / (rs * rs) };
    }
    // Lens (circle-circle intersection) area, normalised by the Sun's area.
    let rs2 = rs * rs;
    let rm2 = rm * rm;
    let cos_a = ((d * d + rs2 - rm2) / (2.0 * d * rs)).clamp(-1.0, 1.0);
    let cos_b = ((d * d + rm2 - rs2) / (2.0 * d * rm)).clamp(-1.0, 1.0);
    let alpha = cos_a.acos();
    let beta = cos_b.acos();
    let area = rs2 * (alpha - 0.5 * (2.0 * alpha).sin()) + rm2 * (beta - 0.5 * (2.0 * beta).sin());
    (area / (std::f64::consts::PI * rs2)).clamp(0.0, 1.0)
}

/// Compute local solar-eclipse circumstances for one observer, around a known
/// greatest-eclipse TT instant (e.g. from
/// [`crate::besselian::classify_solar_eclipse`]).
///
/// `jd_greatest_tt` is used only to seed the maximum-eclipse search window; the
/// per-observer maximum is found independently. Returns `None` if no part of the
/// penumbra reaches the observer (no eclipse here) or positions are unavailable.
pub fn local_circumstances(
    almanac: &Almanac,
    jd_greatest_tt: JdTT,
    lat_deg: f64,
    lon_deg: f64,
    elevation_m: f64,
) -> Option<LocalCircumstances> {
    let model = DeltaTModel::StephensonMorrisonHohenkerk2016;
    let lon_rad = lon_deg.to_radians();
    let (rho_sin, rho_cos) = observer_rho(lat_deg, elevation_m);

    // Evaluate the observer state at a TT instant. GAST needs UT1, so convert.
    let state_at = |jd_tt_f: f64| -> Option<ObserverState> {
        let jd_tt = JdTT(jd_tt_f);
        let b = besselian_elements(almanac, jd_tt)?;
        let a = shadow_axis_ra(almanac, jd_tt)?;
        let jd_ut = jd_tt.to_ut1(&model).as_f64();
        let gast = gast_rad(jd_ut);
        Some(observer_state(&b, a, gast, lon_rad, rho_sin, rho_cos))
    };

    // ── Maximum eclipse for THIS observer: minimise m over ±3 h of greatest. ──
    let g0 = jd_greatest_tt.as_f64();
    let step = 1.0 / 1440.0; // 1 minute
    let (mut best_jd, mut best_m) = (g0, f64::INFINITY);
    let mut jd = g0 - 0.125;
    while jd <= g0 + 0.125 {
        if let Some(s) = state_at(jd)
            && s.m < best_m
        {
            best_m = s.m;
            best_jd = jd;
        }
        jd += step;
    }
    // Golden-section refine of the minimum-separation instant.
    let (mut lo, mut hi) = (best_jd - step, best_jd + step);
    const PHI: f64 = 0.618_033_988_749_895;
    for _ in 0..60 {
        let c = hi - (hi - lo) * PHI;
        let dd = lo + (hi - lo) * PHI;
        let fc = state_at(c)?.m;
        let fd = state_at(dd)?.m;
        if fc < fd {
            hi = dd;
        } else {
            lo = c;
        }
    }
    let jd_max_tt = 0.5 * (lo + hi);
    let s_max = state_at(jd_max_tt)?;

    // No eclipse here if even the penumbra misses the observer at maximum.
    if s_max.m > s_max.l1p {
        return None;
    }

    let magnitude = magnitude_from_state(&s_max);
    let obsc = obscuration(&s_max);
    let eclipse_type = if s_max.m <= s_max.l2p.abs() {
        if s_max.l2p < 0.0 {
            LocalSolarType::Total
        } else {
            LocalSolarType::Annular
        }
    } else {
        LocalSolarType::Partial
    };

    let jd_max_ut = JdTT(jd_max_tt).to_ut1(&model).as_f64();
    let alt_at_max = sun_altitude_deg(almanac, jd_max_ut, lat_deg, lon_deg, elevation_m);

    // ── Contacts: zero crossings of (m − radius) bracketing maximum. ──────────
    // C1/C4 use the penumbral radius l1'; C2/C3 use |l2'|. C1/C2 lie BEFORE
    // maximum, C3/C4 AFTER. Search outward up to 3.5 h on each side.
    let sep_minus = |jd_tt: f64, umbral: bool| -> Option<f64> {
        let s = state_at(jd_tt)?;
        let radius = if umbral { s.l2p.abs() } else { s.l1p };
        Some(s.m - radius)
    };
    let half_window = 3.5 / 24.0;

    let find_contact = |before: bool, umbral: bool| -> Option<Contact> {
        // f = m − radius; negative inside the shadow, positive outside. At
        // maximum f<0 (inside). Walk outward until f turns positive, then bisect.
        let dir = if before { -1.0 } else { 1.0 };
        let f_max = sep_minus(jd_max_tt, umbral)?;
        if f_max >= 0.0 {
            return None; // shadow (this kind) never covers the observer
        }
        let coarse = 0.5 / 1440.0; // 30 s scan
        let mut prev_jd = jd_max_tt;
        let mut t = coarse;
        while t <= half_window {
            let jd = jd_max_tt + dir * t;
            let f = sep_minus(jd, umbral)?;
            if f >= 0.0 {
                // Bracket [prev, jd] (in TT). Bisect on the separation function.
                let (mut a, mut b) = if before { (jd, prev_jd) } else { (prev_jd, jd) };
                let mut fa = sep_minus(a, umbral)?;
                for _ in 0..50 {
                    let mid = 0.5 * (a + b);
                    let fm = sep_minus(mid, umbral)?;
                    if (fm < 0.0) == (fa < 0.0) {
                        a = mid;
                        fa = fm;
                    } else {
                        b = mid;
                    }
                }
                let jd_c_tt = 0.5 * (a + b);
                let jd_c_ut = JdTT(jd_c_tt).to_ut1(&model).as_f64();
                let alt = sun_altitude_deg(almanac, jd_c_ut, lat_deg, lon_deg, elevation_m);
                return Some(Contact {
                    jd_ut: jd_c_ut,
                    altitude_deg: alt,
                });
            }
            prev_jd = jd;
            t += coarse;
        }
        None
    };

    let c1 = find_contact(true, false);
    let c4 = find_contact(false, false);
    let (c2, c3) =
        if eclipse_type == LocalSolarType::Total || eclipse_type == LocalSolarType::Annular {
            (find_contact(true, true), find_contact(false, true))
        } else {
            (None, None)
        };

    Some(LocalCircumstances {
        eclipse_type,
        jd_maximum_ut: jd_max_ut,
        magnitude,
        obscuration: obsc,
        c1,
        c2,
        c3,
        c4,
        altitude_at_max_deg: alt_at_max,
    })
}

/// Topocentric Sun altitude (degrees) at a UT1 instant — used to tag whether a
/// contact is above the horizon. Computed from the geocentric Sun via the local
/// hour angle and a parallax-free altitude (adequate for the horizon flag; the
/// Sun's parallax is < 9″ and does not change observability decisions).
fn sun_altitude_deg(
    almanac: &Almanac,
    jd_ut: f64,
    lat_deg: f64,
    lon_deg: f64,
    _elevation_m: f64,
) -> f64 {
    let model = DeltaTModel::StephensonMorrisonHohenkerk2016;
    let dt = delta_t(jd_ut, &model);
    let jd_tt = JdTT(jd_ut + dt / 86400.0);
    let t = jd_tt.julian_centuries_from_j2000();
    let eps = mean_obliquity(t) + nutation_2000b(t).delta_epsilon;
    let sun_ecl = match almanac.geocentric_ecliptic_tt(crate::Body::Sun, jd_tt) {
        Ok(p) => p,
        Err(_) => return f64::NAN,
    };
    let sun = ecliptic_to_equatorial(&sun_ecl, eps);
    let gast = gast_rad(jd_ut);
    let lst = gast + lon_deg.to_radians();
    let h = (lst - sun.right_ascension).rem_euclid(TAU);
    let lat = lat_deg.to_radians();
    let alt =
        (lat.sin() * sun.declination.sin() + lat.cos() * sun.declination.cos() * h.cos()).asin();
    alt.to_degrees()
}

/// Convenience: a UT1 Julian Day from the contact, as a typed value.
pub fn contact_ut1(c: &Contact) -> JdUT1 {
    JdUT1(c.jd_ut)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::besselian::classify_solar_eclipse;

    fn almanac() -> Almanac {
        Almanac::default_vedic()
    }

    fn jd_tt(year: i32, month: u32, day: u32, hour: f64) -> f64 {
        let (y, m) = if month <= 2 {
            (year - 1, month + 12)
        } else {
            (year, month)
        };
        let a = (y as f64 / 100.0).floor();
        let b = 2.0 - a + (a / 4.0).floor();
        (365.25 * (y as f64 + 4716.0)).floor()
            + (30.6001 * (m as f64 + 1.0)).floor()
            + day as f64
            + b
            - 1524.5
            + hour / 24.0
    }

    /// Format a UT1 JD as HH:MM:SS for diagnostics.
    fn hms(jd_ut: f64) -> String {
        let frac = (jd_ut + 0.5).fract();
        let secs = frac * 86400.0;
        let h = (secs / 3600.0).floor();
        let mnt = ((secs - h * 3600.0) / 60.0).floor();
        let s = secs - h * 3600.0 - mnt * 60.0;
        format!("{:02.0}:{:02.0}:{:05.2} UT", h, mnt, s)
    }

    /// Total Solar Eclipse 2017-08-21 at Madras, Oregon (44.6334°N, 121.1296°W,
    /// 683 m) — a NASA/GSFC path-of-totality city.
    ///
    /// REFERENCE ORACLE (re-runnable, NOT memorised):
    ///   python3 -c "import swisseph as swe; swe.set_ephe_path(None);
    ///     print(swe.sol_eclipse_when_loc(swe.julday(2017,8,21,0.0),
    ///           [-121.1296,44.6334,683.0], swe.FLG_MOSEPH))"
    ///   → Swiss Ephemeris 2.10.03 (Moshier) gives, in UT:
    ///       C1 16:06:43.5   C2 17:19:36.7   max 17:20:37.2
    ///       C3 17:21:37.9   C4 18:41:06.5
    ///     and sol_eclipse_how → magnitude 1.0113, obscuration 1.0 (TOTAL).
    /// GLOBAL CITATION: NASA Five Millennium Canon of Solar Eclipses
    ///   (Espenak & Meeus), catalog #09546:
    ///   https://eclipse.gsfc.nasa.gov/SEcat5/SE2001-2100.html
    ///   → TD greatest 18:26:40, γ = 0.4367, magnitude 1.0306, TOTAL.
    #[test]
    fn local_2017_madras_oregon_matches_swiss_oracle() {
        let a = almanac();
        // Seed greatest eclipse from the global Besselian classifier.
        let jd_new_ut = jd_tt(2017, 8, 21, 18.5) - 68.4 / 86400.0;
        let g = classify_solar_eclipse(&a, jd_new_ut).expect("global eclipse");

        let (lat, lon, elev) = (44.6334, -121.1296, 683.0);
        let lc = local_circumstances(&a, g.jd_greatest_tt, lat, lon, elev)
            .expect("eclipse visible at Madras OR");

        // 1) Type must be TOTAL (Swiss: obscuration 1.0; NASA path city).
        assert_eq!(
            lc.eclipse_type,
            LocalSolarType::Total,
            "Madras OR was on the path of totality"
        );

        // 2) Magnitude vs Swiss 1.0113 (diameter fraction).
        assert!(
            (lc.magnitude - 1.0113).abs() < 0.01,
            "magnitude {:.4} should match Swiss oracle 1.0113 ±0.01",
            lc.magnitude
        );

        // Helper: seconds between a computed UT JD and an HH:MM:SS.s target.
        let target = |h: f64, m: f64, s: f64| jd_tt(2017, 8, 21, h + (m + s / 60.0) / 60.0);
        let err = |c: Option<Contact>, tgt: f64| -> f64 {
            (c.expect("contact present").jd_ut - tgt).abs() * 86400.0
        };

        // 4) Contact times vs the Swiss oracle (UT), each within 30 s. The
        //    residual is dominated by the truncated analytical Moon/Sun the
        //    Besselian engine uses (γ matches NASA to ~0.002 ER — see
        //    besselian.rs tests), which maps to a few seconds of timing.
        let c1_err = err(lc.c1, target(16.0, 6.0, 43.5));
        let c2_err = err(lc.c2, target(17.0, 19.0, 36.7));
        let c3_err = err(lc.c3, target(17.0, 21.0, 37.9));
        let c4_err = err(lc.c4, target(18.0, 41.0, 6.5));
        let max_err = (lc.jd_maximum_ut - target(17.0, 20.0, 37.2)).abs() * 86400.0;

        println!(
            "Madras OR 2017-08-21 local circumstances:\n  \
             C1  {} (Swiss 16:06:43.5, Δ{:.1}s)\n  \
             C2  {} (Swiss 17:19:36.7, Δ{:.1}s)\n  \
             max {} (Swiss 17:20:37.2, Δ{:.1}s)\n  \
             C3  {} (Swiss 17:21:37.9, Δ{:.1}s)\n  \
             C4  {} (Swiss 18:41:06.5, Δ{:.1}s)\n  \
             mag {:.4}  obsc {:.4}  alt(max) {:.1}°",
            hms(lc.c1.unwrap().jd_ut),
            c1_err,
            hms(lc.c2.unwrap().jd_ut),
            c2_err,
            hms(lc.jd_maximum_ut),
            max_err,
            hms(lc.c3.unwrap().jd_ut),
            c3_err,
            hms(lc.c4.unwrap().jd_ut),
            c4_err,
            lc.magnitude,
            lc.obscuration,
            lc.altitude_at_max_deg
        );

        // 3) Obscuration is total (Swiss reports 1.0). Allow the area formula's
        //    rounding near internal tangency.
        assert!(
            lc.obscuration > 0.999,
            "obscuration {:.5} should be ~1.0 for a total eclipse",
            lc.obscuration
        );

        assert!(c1_err < 30.0, "C1 off by {c1_err:.1}s vs Swiss oracle");
        assert!(c2_err < 30.0, "C2 off by {c2_err:.1}s vs Swiss oracle");
        assert!(c3_err < 30.0, "C3 off by {c3_err:.1}s vs Swiss oracle");
        assert!(c4_err < 30.0, "C4 off by {c4_err:.1}s vs Swiss oracle");
        assert!(max_err < 30.0, "max off by {max_err:.1}s vs Swiss oracle");

        // 5) Sun well above the horizon (Swiss true altitude 41.6°).
        assert!(
            (lc.altitude_at_max_deg - 41.6).abs() < 1.5,
            "Sun altitude {:.1}° should match Swiss 41.6° ±1.5°",
            lc.altitude_at_max_deg
        );
    }

    /// A location far OFF the path (e.g. mid-Pacific south of the equator) should
    /// see no eclipse, or at most a small partial — never totality.
    #[test]
    fn off_path_location_is_not_total() {
        let a = almanac();
        let jd_new_ut = jd_tt(2017, 8, 21, 18.5) - 68.4 / 86400.0;
        let g = classify_solar_eclipse(&a, jd_new_ut).expect("global eclipse");
        // 40°S, 150°W — southern Pacific, far from the northern-hemisphere path.
        let lc = local_circumstances(&a, g.jd_greatest_tt, -40.0, -150.0, 0.0);
        if let Some(lc) = lc {
            assert_ne!(
                lc.eclipse_type,
                LocalSolarType::Total,
                "a far-southern site cannot see totality (got mag {:.3})",
                lc.magnitude
            );
        }
    }
}
