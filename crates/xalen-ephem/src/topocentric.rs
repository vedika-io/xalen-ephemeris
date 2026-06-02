//! Topocentric position correction — geocentric → observer's location.
//!
//! Applies diurnal parallax per Meeus, *Astronomical Algorithms* (2nd ed.),
//! Chapter 40. The shift is only ~8.8″ for the Sun but reaches ~1° for the
//! Moon, so it matters for lunar work, muhurta/electional timing, and local
//! rise/set phenomena.

use std::f64::consts::TAU;
use xalen_coords::{
    EclipticPosition, EquatorialPosition, ecliptic_to_equatorial, equatorial_to_ecliptic,
    mean_obliquity, nutation_2000b,
};

/// Earth equatorial radius in AU (6 378 136.6 m / 1 AU).
pub(crate) const EARTH_RADIUS_AU: f64 = 4.263_523e-5;
/// Earth equatorial radius in metres (IERS).
const EARTH_RADIUS_M: f64 = 6_378_136.6;
/// Polar/equatorial axis ratio b/a for the reference ellipsoid (Meeus Ch.11).
const FLATTENING_BA: f64 = 0.996_647_19;

/// Greenwich Mean Sidereal Time in degrees [0, 360) for a UT1 Julian Day.
///
/// Thin re-export of the canonical [`xalen_coords::gmst_deg`] (the single
/// source of truth for sidereal time); kept as a crate-private alias so the
/// in-crate call sites read unchanged.
pub(crate) fn gmst_deg(jd_ut1: f64) -> f64 {
    xalen_coords::gmst_deg(jd_ut1)
}

/// Convert a geocentric ecliptic position to **topocentric** (observer-centered).
///
/// - `geo`: geocentric ecliptic position (longitude/latitude in radians, distance
///   in AU) at the same instant.
/// - `jd_ut1`: UT1 Julian Day (sidereal time, hence the hour angle, needs UT1).
/// - `t_centuries`: Julian centuries (TT) from J2000, for the obliquity.
/// - observer: geographic `lat_deg`, `lon_deg` (east positive), `elevation_m`.
///
/// Returns the topocentric ecliptic position. Distance is left at its geocentric
/// value (the topocentric distance change is below the parallax angle and does
/// not affect longitude/latitude meaningfully).
pub fn topocentric_ecliptic(
    geo: &EclipticPosition,
    jd_ut1: f64,
    t_centuries: f64,
    lat_deg: f64,
    lon_deg: f64,
    elevation_m: f64,
) -> EclipticPosition {
    // `geo` is referred to the TRUE equinox of date (apparent ecliptic-of-date —
    // the provider contract: Sun, planets, Moon and DE440 all carry nutation in
    // longitude). The ecliptic→equatorial rotation must therefore use the TRUE
    // obliquity (mean + delta_epsilon), and the hour angle must use APPARENT
    // sidereal time (GMST + equation of equinoxes), so the resulting RA and the
    // sidereal time are referred to the SAME (true) equinox. Using mean obliquity
    // and bare GMST against apparent-of-date coordinates left a delta_epsilon-
    // (~9") and equation-of-equinoxes- (~16" ≈ 1.05 s) sized inconsistency.
    // Cf. besselian.rs, which already uses the true obliquity. (Meeus Ch.22/12.)
    let nut = nutation_2000b(t_centuries);
    let eps = mean_obliquity(t_centuries) + nut.delta_epsilon; // true obliquity of date
    let eq = ecliptic_to_equatorial(geo, eps);
    let (alpha, delta, dist) = (eq.right_ascension, eq.declination, eq.distance);

    // Equation of the equinoxes (radians): delta_psi · cos(eps_true). Converts
    // GMST → GAST (apparent sidereal time).
    let eq_of_equinoxes = nut.delta_psi * eps.cos();

    // Observer's geocentric quantities rho*sin(phi'), rho*cos(phi') (Meeus Ch.11/40).
    let lat = lat_deg.to_radians();
    let u = (FLATTENING_BA * lat.tan()).atan();
    let h_over_r = elevation_m / EARTH_RADIUS_M; // elevation in Earth radii
    let rho_sin = FLATTENING_BA * u.sin() + h_over_r * lat.sin();
    let rho_cos = u.cos() + h_over_r * lat.cos();

    // Equatorial horizontal parallax: sin(pi) = Earth radius / distance.
    let sin_pi = EARTH_RADIUS_AU / dist;

    // Local APPARENT sidereal time = GMST + equation of equinoxes + observer
    // longitude (all in radians); local hour angle H = LAST − apparent RA.
    let last = (gmst_deg(jd_ut1) + lon_deg).to_radians() + eq_of_equinoxes;
    let h_angle = (last - alpha).rem_euclid(TAU);
    let (sin_h, cos_h) = (h_angle.sin(), h_angle.cos());

    // Parallax in right ascension and topocentric declination (Meeus 40.2-40.3).
    let denom = delta.cos() - rho_cos * sin_pi * cos_h;
    let d_alpha = (-rho_cos * sin_pi * sin_h).atan2(denom);
    let alpha_topo = (alpha + d_alpha).rem_euclid(TAU);
    let delta_topo = ((delta.sin() - rho_sin * sin_pi) * d_alpha.cos()).atan2(denom);

    let eq_topo = EquatorialPosition {
        right_ascension: alpha_topo,
        declination: delta_topo,
        distance: dist,
    };
    equatorial_to_ecliptic(&eq_topo, eps)
}
