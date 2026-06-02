//! Canonical sidereal-time functions.
//!
//! This is the single source of truth for Greenwich sidereal time across the
//! workspace. Previously three byte-for-byte-identical copies of the IAU 1982
//! GMST polynomial lived in `xalen-ephem` (`topocentric.rs`, `local_eclipse.rs`)
//! and `xalen-houses` (`angles.rs`); they have all been collapsed onto these
//! functions so a future correction (e.g. moving to IAU 2006 GMST) only has to
//! happen here.
//!
//! * [`gmst_deg`] — Greenwich **Mean** Sidereal Time, degrees `[0, 360)`.
//! * [`gmst_hours`] — the same value in hours `[0, 24)`.
//! * [`equation_of_equinoxes`] — `Δψ · cos ε_true`, the GMST → GAST offset.
//! * [`gast_deg`] / [`gast_rad`] — Greenwich **Apparent** Sidereal Time, the
//!   value Swiss Ephemeris uses for chart angles, hour angles, and the eclipse
//!   engine (it is referred to the *true* equinox of date, matching nutated
//!   apparent positions).

use crate::{mean_obliquity, nutation_2000b};
use std::f64::consts::TAU;

/// Greenwich Mean Sidereal Time in degrees `[0, 360)` for a UT1 Julian Day
/// (IAU 1982 expression, Meeus eq. 12.4).
///
/// The polynomial coefficients are exactly those that previously lived in
/// `xalen_ephem::topocentric::gmst_deg`, `xalen_ephem::local_eclipse::gast_rad`
/// (its GMST term), and `xalen_houses::angles::gmst`, so this is a behaviour-
/// preserving consolidation.
pub fn gmst_deg(jd_ut1: f64) -> f64 {
    let t = (jd_ut1 - 2_451_545.0) / 36525.0;
    let g = 280.460_618_37 + 360.985_647_366_29 * (jd_ut1 - 2_451_545.0) + 0.000_387_933 * t * t
        - t * t * t / 38_710_000.0;
    g.rem_euclid(360.0)
}

/// Greenwich Mean Sidereal Time in hours `[0, 24)` for a UT1 Julian Day.
///
/// Identical value to [`gmst_deg`] divided by 15; provided for callers (chart
/// angles) that work in sidereal hours.
pub fn gmst_hours(jd_ut1: f64) -> f64 {
    (gmst_deg(jd_ut1) / 15.0).rem_euclid(24.0)
}

/// Equation of the equinoxes in radians: `Δψ · cos ε_true`.
///
/// This is the offset that converts GMST to GAST (apparent sidereal time).
/// `t_tt` is Julian centuries from J2000 in **Terrestrial Time** (nutation and
/// obliquity are functions of TT, not UT1; the difference is ΔT, ~69 s today).
pub fn equation_of_equinoxes(t_tt: f64) -> f64 {
    let nut = nutation_2000b(t_tt);
    let eps_true = mean_obliquity(t_tt) + nut.delta_epsilon;
    nut.delta_psi * eps_true.cos()
}

/// Greenwich Apparent Sidereal Time in radians `[0, 2π)` for a UT1 Julian Day.
///
/// `GAST = GMST + Δψ·cos ε_true`. Swiss Ephemeris uses this (not GMST) for
/// `swe_houses` ARMC, hour angles, and Besselian eclipse circumstances, because
/// it is referred to the same *true* equinox of date as the nutated apparent
/// positions XALEN's providers emit.
///
/// `t_tt` is Julian centuries from J2000 in Terrestrial Time, used for the
/// nutation/obliquity in the equation of the equinoxes.
pub fn gast_rad(jd_ut1: f64, t_tt: f64) -> f64 {
    (gmst_deg(jd_ut1).to_radians() + equation_of_equinoxes(t_tt)).rem_euclid(TAU)
}

/// Greenwich Apparent Sidereal Time in degrees `[0, 360)` for a UT1 Julian Day.
///
/// See [`gast_rad`].
pub fn gast_deg(jd_ut1: f64, t_tt: f64) -> f64 {
    gast_rad(jd_ut1, t_tt).to_degrees().rem_euclid(360.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const J2000: f64 = 2_451_545.0;

    #[test]
    fn gmst_deg_at_j2000() {
        // IAU 1982 polynomial constant term at J2000.
        let g = gmst_deg(J2000);
        assert!((g - 280.460_618_37).abs() < 1e-6, "gmst_deg J2000 = {g}");
    }

    #[test]
    fn gmst_hours_matches_deg_over_15() {
        for &jd in &[J2000, 2_460_000.5, 2_440_000.0, 2_300_000.0] {
            let h = gmst_hours(jd);
            let expected = (gmst_deg(jd) / 15.0).rem_euclid(24.0);
            assert!((h - expected).abs() < 1e-12, "hours vs deg/15 at {jd}");
            assert!(h >= 0.0 && h < 24.0);
        }
    }

    #[test]
    fn gmst_hours_about_18h7_at_j2000() {
        // Classic check value: GMST at J2000.0 noon ≈ 18.697 h.
        let h = gmst_hours(J2000);
        assert!((h - 18.697_374_558).abs() < 1e-6, "gmst hours = {h}");
    }

    #[test]
    fn gast_is_gmst_plus_equation_of_equinoxes() {
        // At J2000 the equation of the equinoxes is ≈ −12.7″ ≈ −0.00353°.
        let t_tt = 0.0;
        let eoe = equation_of_equinoxes(t_tt);
        let gmst = gmst_deg(J2000).to_radians();
        let gast = gast_rad(J2000, t_tt);
        let recombined = (gmst + eoe).rem_euclid(TAU);
        assert!((gast - recombined).abs() < 1e-12);
        // Equation of equinoxes magnitude sanity (a few arcseconds).
        let eoe_arcsec = eoe.to_degrees() * 3600.0;
        assert!(
            eoe_arcsec.abs() < 20.0 && eoe_arcsec.abs() > 1.0,
            "equation of equinoxes ≈ a few arcsec, got {eoe_arcsec}″"
        );
    }

    #[test]
    fn gast_deg_in_range() {
        let g = gast_deg(J2000, 0.0);
        assert!(g >= 0.0 && g < 360.0);
        // GAST ≈ 280.457° at J2000 (GMST 280.4606 − 0.0035°).
        assert!((g - 280.457).abs() < 0.01, "gast_deg = {g}");
    }
}
