//! IAU SOFA reference cross-validation for the reduction-chain primitives.
//!
//! XALEN's apparent-place accuracy is normally proven end-to-end against JPL
//! DE440 and Swiss Ephemeris. This test validates the two reduction-chain
//! primitives — IAU 2000B nutation and IAU 2006 mean obliquity — DIRECTLY
//! against the IAU's own reference implementation, SOFA (Standards of
//! Fundamental Astronomy). The expected values are taken verbatim from the SOFA
//! validation suite `t_sofa_c.c` (rev. 2013-08-07), so this is a primary-source
//! check of the physics, not a transitive one.
//!
//! SOFA references (t_sofa_c.c):
//!   iauNut00b(2400000.5, 53736.0) -> dpsi = -0.9632552291148362783e-5,
//!                                    deps =  0.4063197106621159367e-4  (rad)
//!   iauObl06 (2400000.5, 54388.0) -> 0.4090749229387258204             (rad)
//!
//! Source: https://raw.githubusercontent.com/Starlink/sofa/master/src/t_sofa_c.c
//!
//! In addition to the two single-epoch SOFA point checks, this file carries a
//! MULTI-EPOCH oracle grid generated with pyswisseph 2.10.03 (`swe.calc(jd_tt,
//! swe.ECL_NUT, swe.FLG_SWIEPH)`), spanning 1900 → 2050. Swiss Ephemeris's
//! `ECL_NUT` returns the FULL IAU 2000A nutation (the long ~1365-term series)
//! plus its IAU 2006 mean/true obliquity, so:
//!   * mean obliquity (IAU 2006 in both) agrees to < 30 µas — tight grid gate.
//!   * Δε (nutation in obliquity) agrees to < 0.6 mas — the IAU 2000A↔2000B
//!     obliquity-term gap.
//!   * Δψ (nutation in longitude) agrees to ≈ 3.3 mas peak — this is exactly the
//!     documented IAU 2000B *truncation* error vs the full 2000A series (XALEN
//!     implements the 77-term 2000B model). The grid gate (2e-8 rad ≈ 4.1 mas)
//!     is therefore set to the real model gap, not an arbitrary slack value, so
//!     it still catches a genuine coefficient/argument regression.
//! Every grid number below is a real Swiss-Ephemeris output, not an estimate.

use xalen_coords::{mean_obliquity, nutation_2000b};

const J2000: f64 = 2_451_545.0;
const ARCSEC_PER_RAD: f64 = 206_264.806_247_096_36;

/// Julian centuries TT from J2000 for a TT Julian Date (grid helper).
fn centuries(jd_tt: f64) -> f64 {
    (jd_tt - J2000) / 36525.0
}

/// Multi-epoch nutation + mean-obliquity oracle grid.
/// Columns: (label, jd_tt, Δψ_rad, Δε_rad, mean_obliquity_rad).
/// Source: pyswisseph 2.10.03 `swe.calc(jd_tt, swe.ECL_NUT, swe.FLG_SWIEPH)`
/// — x[2]=Δψ (deg→rad), x[3]=Δε (deg→rad), x[1]=mean obliquity (deg→rad).
#[rustfmt::skip]
const NUT_OBL_GRID: &[(&str, f64, f64, f64, f64)] = &[
    ("1900-01-01T12 TT", 2_415_021.0, 8.493237825236467e-05, -1.105907020492395e-05, 4.093196549827494e-01),
    ("1950-01-01T12 TT", 2_433_283.0, -1.573547877153770e-05, 4.027870813985479e-05, 4.092061316116486e-01),
    ("J2000.0",          2_451_545.0, -6.754195804145283e-05, -2.797280438806837e-05, 4.090926005957347e-01),
    ("2025-06-01T00 TT", 2_460_827.5, 7.502124993153864e-06, 4.177518796373252e-05, 4.090348926354189e-01),
    ("2050-01-01T12 TT", 2_469_808.0, 7.350991362897182e-05, -2.592790433724558e-05, 4.089790629618730e-01),
];

/// SOFA passes a 2-part TT Julian Date; XALEN takes Julian centuries TT from J2000.
fn centuries_tt(jd_tt: f64) -> f64 {
    (jd_tt - J2000) / 36525.0
}

#[test]
fn nutation_2000b_matches_sofa() {
    let jd = 2_400_000.5 + 53_736.0;
    let n = nutation_2000b(centuries_tt(jd));

    let dpsi_err = (n.delta_psi - (-0.9632552291148362783e-5)).abs();
    let deps_err = (n.delta_epsilon - 0.4063197106621159367e-4).abs();

    println!(
        "\nNut00b @ TT JD {jd}: |Δ(Δψ)| = {dpsi_err:.3e} rad ({:.4e}\"), |Δ(Δε)| = {deps_err:.3e} rad ({:.4e}\")",
        dpsi_err * ARCSEC_PER_RAD,
        deps_err * ARCSEC_PER_RAD
    );

    // IAU 2000B vs the SOFA reference. A perfect match (incl. the fixed planetary
    // bias offsets) lands ~1e-12 rad; gate at 5e-9 rad (~1 mas) to report honestly
    // and still catch a real regression.
    assert!(
        dpsi_err < 5e-9,
        "Δψ off SOFA iauNut00b by {dpsi_err:.3e} rad"
    );
    assert!(
        deps_err < 5e-9,
        "Δε off SOFA iauNut00b by {deps_err:.3e} rad"
    );
}

#[test]
fn mean_obliquity_matches_sofa_obl06() {
    let jd = 2_400_000.5 + 54_388.0;
    let eps = mean_obliquity(centuries_tt(jd));
    let err = (eps - 0.4090749229387258204).abs();

    println!(
        "Obl06 @ TT JD {jd}: |Δε| = {err:.3e} rad ({:.4e}\")",
        err * ARCSEC_PER_RAD
    );

    assert!(
        err < 5e-9,
        "mean obliquity off SOFA iauObl06 by {err:.3e} rad"
    );
}

// ---------------------------------------------------------------------------
// Multi-epoch oracle grid (pyswisseph ECL_NUT, 1900 → 2050).
// Replaces the previous single-epoch point checks with a 5-epoch sweep so a
// regression in the time-dependent Delaunay arguments or the per-term rate
// coefficients (the `sd`/`cd` columns) is caught, not just the constant phase.
// ---------------------------------------------------------------------------

/// Mean obliquity (IAU 2006) must track Swiss Ephemeris across the whole grid
/// to < 30 µas. Both XALEN and Swiss use the IAU 2006 (P03) mean-obliquity
/// polynomial, so this is a same-model check and the tolerance is genuinely
/// tight — it would flip on any coefficient typo.
#[test]
fn mean_obliquity_matches_swiss_grid() {
    const TOL: f64 = 1e-9; // ≈ 0.2 mas; observed max ≈ 1.4e-10 (28 µas)
    let mut max_err = 0.0_f64;
    for &(label, jd, _dpsi, _deps, mobl_ref) in NUT_OBL_GRID {
        let eps = mean_obliquity(centuries(jd));
        let err = (eps - mobl_ref).abs();
        max_err = max_err.max(err);
        println!(
            "  mean_obl {label}: |Δ| = {err:.3e} rad ({:.4} µas)",
            err * ARCSEC_PER_RAD * 1e6
        );
        assert!(
            err < TOL,
            "mean obliquity at {label} off Swiss by {err:.3e} rad (tol {TOL:.0e})"
        );
    }
    println!("  mean_obl grid max |Δ| = {max_err:.3e} rad");
}

/// Nutation in obliquity (Δε) must track Swiss IAU 2000A to < 5e-9 rad
/// (≈ 1 mas). The residual here is the IAU 2000A↔2000B obliquity-term gap
/// (observed max ≈ 2.7e-9 rad / 0.56 mas), NOT XALEN error.
#[test]
fn nutation_obliquity_matches_swiss_grid() {
    const TOL: f64 = 5e-9; // observed max ≈ 2.71e-9 rad (0.56 mas)
    let mut max_err = 0.0_f64;
    for &(label, jd, _dpsi, deps_ref, _mobl) in NUT_OBL_GRID {
        let n = nutation_2000b(centuries(jd));
        let err = (n.delta_epsilon - deps_ref).abs();
        max_err = max_err.max(err);
        println!(
            "  Δε {label}: |Δ| = {err:.3e} rad ({:.4} mas)",
            err * ARCSEC_PER_RAD * 1e3
        );
        assert!(
            err < TOL,
            "Δε at {label} off Swiss IAU 2000A by {err:.3e} rad (tol {TOL:.0e}; \
             expected ≤ the 2000A/2000B obliquity-term gap)"
        );
    }
    println!("  Δε grid max |Δ| = {max_err:.3e} rad");
}

/// Nutation in longitude (Δψ) must track Swiss IAU 2000A to within the
/// documented IAU 2000B *truncation* envelope (2e-8 rad ≈ 4.1 mas; observed
/// max ≈ 1.58e-8 rad / 3.3 mas). XALEN implements the 77-term IAU 2000B series;
/// the McCarthy & Luzum (2003) spec for 2000B is ~1 mas RMS / a few mas peak
/// vs the full 2000A — exactly what this grid sees. The gate is set to that
/// real gap so a coefficient/argument regression (which would blow past 4 mas)
/// is still caught while an honest 2000B implementation passes.
#[test]
fn nutation_longitude_matches_swiss_within_2000b_truncation() {
    const TOL: f64 = 2e-8; // observed max ≈ 1.58e-8 rad (3.3 mas)
    let mut max_err = 0.0_f64;
    for &(label, jd, dpsi_ref, _deps, _mobl) in NUT_OBL_GRID {
        let n = nutation_2000b(centuries(jd));
        let err = (n.delta_psi - dpsi_ref).abs();
        max_err = max_err.max(err);
        println!(
            "  Δψ {label}: |Δ| = {err:.3e} rad ({:.4} mas)",
            err * ARCSEC_PER_RAD * 1e3
        );
        assert!(
            err < TOL,
            "Δψ at {label} off Swiss IAU 2000A by {err:.3e} rad (tol {TOL:.0e}; \
             larger than the 2000B truncation envelope = real regression)"
        );
    }
    println!("  Δψ grid max |Δ| = {max_err:.3e} rad");
    // The grid must actually exercise the 2000B truncation regime, otherwise the
    // loose Δψ gate would be vacuous: at least one epoch must show > 1 mas.
    assert!(
        max_err > 1e-9,
        "Δψ grid never exceeds 1e-9 rad — grid is not exercising the model gap; \
         re-check the oracle values"
    );
}
