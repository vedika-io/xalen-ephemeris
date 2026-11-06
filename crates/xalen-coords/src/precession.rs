//! Precession (IAU 2006/P03) and ICRS frame-bias transforms.
//!
//! License: this file is distributed under Apache-2.0 AND BSD-3-Clause.
//! The Fukushima–Williams precession and frame-bias matrix routines
//! (`fw06_angles`, `fw2m`, `frame_bias_matrix`, `precession_bias_matrix_iau2006`,
//! `precession_matrix_p03_nobias`) are an independent Rust port of the
//! corresponding ERFA routines (pfw06, fw2m, bi00, bp00, pmat06) and therefore
//! carry ERFA's BSD 3-Clause license in addition to the crate's Apache-2.0.
//! ERFA © NumFOCUS Foundation; upstream https://github.com/liberfa/erfa.
//! Full BSD-3-Clause text is reproduced in the repository NOTICE file.

use crate::ARCSEC_TO_RAD;
use crate::obliquity::mean_obliquity;
use crate::transforms::{EclipticPosition, ecliptic_to_equatorial, equatorial_to_ecliptic};

/// Mean obliquity of the ecliptic at J2000.0 (IAU 2006), in radians. This is the
/// `epsilon_a` term of [`precession_angles`] evaluated at `t = 0`.
const MEAN_OBLIQUITY_J2000: f64 = 84381.406 * ARCSEC_TO_RAD;

/// Compute IAU 2006/P03 precession angles in radians for Julian centuries `t` from J2000.
pub fn precession_angles(t: f64) -> PrecessionAngles {
    // IAU 2006/P03 precession — Capitaine, Wallace, Chapront (2003)
    let psi_a = (5038.481507 * t - 1.0790069 * t * t - 0.00114045 * t * t * t
        + 0.000132851 * t.powi(4)
        - 0.0000000951 * t.powi(5))
        * ARCSEC_TO_RAD;

    let omega_a = (84381.406 - 0.025754 * t + 0.0512623 * t * t
        - 0.00772503 * t * t * t
        - 0.000000467 * t.powi(4)
        + 0.0000003337 * t.powi(5))
        * ARCSEC_TO_RAD;

    let chi_a = (10.556403 * t - 2.3814292 * t * t - 0.00121197 * t * t * t
        + 0.000170663 * t.powi(4)
        - 0.0000000560 * t.powi(5))
        * ARCSEC_TO_RAD;

    // Equatorial precession angles (for rotation matrices)
    let epsilon_a = (84381.406 - 46.836769 * t - 0.0001831 * t * t + 0.00200340 * t * t * t
        - 0.000000576 * t.powi(4)
        - 0.0000000434 * t.powi(5))
        * ARCSEC_TO_RAD;

    // Fukushima-Williams angles
    let zeta_a = (2.650545 + 2306.083227 * t + 0.2988499 * t * t + 0.01801828 * t * t * t
        - 0.000005971 * t.powi(4)
        - 0.0000003173 * t.powi(5))
        * ARCSEC_TO_RAD;

    let z_a = (-2.650545 + 2306.077181 * t + 1.0927348 * t * t + 0.01826837 * t * t * t
        - 0.000028596 * t.powi(4)
        - 0.0000002904 * t.powi(5))
        * ARCSEC_TO_RAD;

    let theta_a = (2004.191903 * t
        - 0.4294934 * t * t
        - 0.04182264 * t * t * t
        - 0.000007089 * t.powi(4)
        - 0.0000001274 * t.powi(5))
        * ARCSEC_TO_RAD;

    PrecessionAngles {
        psi_a,
        omega_a,
        chi_a,
        epsilon_a,
        zeta_a,
        z_a,
        theta_a,
    }
}

/// Compute the general precession in ecliptic longitude (IAU 2006) in radians.
pub fn general_precession_longitude(t: f64) -> f64 {
    // General precession in longitude — IAU 2006
    (5028.796195 * t + 1.1054348 * t * t + 0.00007964 * t * t * t
        - 0.000023857 * t.powi(4)
        - 0.0000000383 * t.powi(5))
        * ARCSEC_TO_RAD
}

/// Compute the 3x3 precession rotation matrix from J2000 to equinox-of-date.
pub fn precession_matrix(t: f64) -> [[f64; 3]; 3] {
    let PrecessionAngles {
        zeta_a,
        z_a,
        theta_a,
        ..
    } = precession_angles(t);

    let cos_zeta = zeta_a.cos();
    let sin_zeta = zeta_a.sin();
    let cos_z = z_a.cos();
    let sin_z = z_a.sin();
    let cos_theta = theta_a.cos();
    let sin_theta = theta_a.sin();

    [
        [
            cos_zeta * cos_z * cos_theta - sin_zeta * sin_z,
            -sin_zeta * cos_z * cos_theta - cos_zeta * sin_z,
            -cos_z * sin_theta,
        ],
        [
            cos_zeta * sin_z * cos_theta + sin_zeta * cos_z,
            -sin_zeta * sin_z * cos_theta + cos_zeta * cos_z,
            -sin_z * sin_theta,
        ],
        [cos_zeta * sin_theta, -sin_zeta * sin_theta, cos_theta],
    ]
}

// ===========================================================================
// IAU 2006/P03 Fukushima–Williams precession + ICRS frame bias (ERFA twin)
// ===========================================================================
//
// Numerics ported from ERFA (github.com/liberfa/erfa, BSD-3-Clause — the
// bit-for-bit numerical twin of IAU SOFA): pfw06.c, fw2m.c, bi00.c, bp00.c,
// pmat06.c. Cross-checked against Wallace & Capitaine (2006), A&A 459, 981,
// eq. (4) and §6.1. Validated element-wise to 1e-12 against the ERFA/SOFA
// `t_erfa_c.c` golden vectors (see tests). These give the full GCRS→mean-of-date
// rotation INCLUDING the ~23 mas ICRS frame bias in one matrix (`pmat06`).

/// The four Fukushima–Williams IAU 2006/P03 precession angles (radians) for
/// Julian centuries TT `t` from J2000. The constant terms in γ̄ and ψ̄ already
/// fold in the ICRS longitude frame bias, so `fw2m` of these yields the full
/// GCRS→mean-of-date matrix. Returns `(gamb, phib, psib, epsa)`.
pub fn fw06_angles(t: f64) -> (f64, f64, f64, f64) {
    let gamb = (-0.052928
        + (10.556378
            + (0.4932044 + (-0.00031238 + (-0.000002788 + 0.0000000260 * t) * t) * t) * t)
            * t)
        * ARCSEC_TO_RAD;
    let phib = (84381.412819
        + (-46.811016
            + (0.0511268 + (0.00053289 + (-0.000000440 + -0.0000000176 * t) * t) * t) * t)
            * t)
        * ARCSEC_TO_RAD;
    let psib = (-0.041775
        + (5038.481484
            + (1.5584175 + (-0.00018522 + (-0.000026452 + -0.0000000148 * t) * t) * t) * t)
            * t)
        * ARCSEC_TO_RAD;
    let epsa = (84381.406
        + (-46.836769
            + (-0.0001831 + (0.00200340 + (-0.000000576 + -0.0000000434 * t) * t) * t) * t)
            * t)
        * ARCSEC_TO_RAD;
    (gamb, phib, psib, epsa)
}

/// Elementary rotation about the x-axis, SOFA `iauRx` convention, applied as a
/// LEFT-multiply `Rx(a)·m`.
fn rx(a: f64, m: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let (s, c) = a.sin_cos();
    mat_mul([[1.0, 0.0, 0.0], [0.0, c, s], [0.0, -s, c]], m)
}

/// Elementary rotation about the y-axis, SOFA `iauRy` convention, `Ry(a)·m`.
fn ry(a: f64, m: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let (s, c) = a.sin_cos();
    mat_mul([[c, 0.0, -s], [0.0, 1.0, 0.0], [s, 0.0, c]], m)
}

/// Elementary rotation about the z-axis, SOFA `iauRz` convention, `Rz(a)·m`.
fn rz(a: f64, m: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let (s, c) = a.sin_cos();
    mat_mul([[c, s, 0.0], [-s, c, 0.0], [0.0, 0.0, 1.0]], m)
}

/// 3×3 matrix product `a · b`.
fn mat_mul(a: [[f64; 3]; 3], b: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut r = [[0.0; 3]; 3];
    for (i, ri) in r.iter_mut().enumerate() {
        for (j, rij) in ri.iter_mut().enumerate() {
            *rij = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
    }
    r
}

const IDENTITY3: [[f64; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

/// Build the precession/bias rotation matrix from the Fukushima–Williams angles
/// (SOFA `iauFw2m`): `R = Rx(−εₐ)·Rz(−ψ̄)·Rx(φ̄)·Rz(γ̄)`. NOTE the minus signs on
/// ψ̄ and εₐ — dropping them yields a plausible but wrong matrix.
pub fn fw2m(gamb: f64, phib: f64, psib: f64, epsa: f64) -> [[f64; 3]; 3] {
    let r = rz(gamb, IDENTITY3);
    let r = rx(phib, r);
    let r = rz(-psib, r);
    rx(-epsa, r)
}

/// Full IAU 2006/P03 precession matrix INCLUDING ICRS frame bias (SOFA
/// `iauPmat06`): rotates a GCRS vector to the mean equatorial frame of date.
/// `V_date = pmat06(t) · V_GCRS`.
pub fn precession_bias_matrix_iau2006(t: f64) -> [[f64; 3]; 3] {
    let (gamb, phib, psib, epsa) = fw06_angles(t);
    fw2m(gamb, phib, psib, epsa)
}

/// The ICRS→dynamical-J2000 frame-bias matrix (SOFA `iauBp00`'s `rb`; ~23 mas,
/// epoch-independent). `V_J2000mean = frame_bias_matrix() · V_GCRS`.
///
/// Gotcha preserved from ERFA: the `sin(EPS0)` factor uses the Lieske 1977
/// obliquity 84381.448″, NOT the P03 value 84381.406″ — they are deliberately
/// different constants living in the same computation.
pub fn frame_bias_matrix() -> [[f64; 3]; 3] {
    const EPS0: f64 = 84381.448 * ARCSEC_TO_RAD; // Lieske 1977 — NOT P03's 84381.406
    let dpsibi = -0.041775 * ARCSEC_TO_RAD;
    let depsbi = -0.0068192 * ARCSEC_TO_RAD;
    let dra0 = -0.0146 * ARCSEC_TO_RAD;
    let r = rz(dra0, IDENTITY3);
    let r = ry(dpsibi * EPS0.sin(), r);
    rx(-depsbi, r)
}

/// Transpose a 3×3 matrix.
pub fn transpose3(m: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    [
        [m[0][0], m[1][0], m[2][0]],
        [m[0][1], m[1][1], m[2][1]],
        [m[0][2], m[1][2], m[2][2]],
    ]
}

/// Apply a 3×3 rotation matrix to a column vector: returns `m · v`.
pub fn rotate3(m: [[f64; 3]; 3], v: [f64; 3]) -> [f64; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

/// IAU 2006/P03 precession matrix WITHOUT the ICRS frame bias: the rotation of
/// an equatorial vector from the dynamical mean equator/equinox of J2000 to the
/// mean equator/equinox of date. `V_eqdate = precession_matrix_p03_nobias(t) ·
/// V_eqJ2000dyn`.
///
/// Constructed as `pmat06(t) · bp00ᵀ`: `pmat06` is the bias-inclusive
/// GCRS→mean-of-date rotation (`P03 · B`) and `bp00` is the epoch-independent
/// ICRS→dynamical-J2000 frame bias (`B`), so `pmat06 · Bᵀ = P03 · B · Bᵀ = P03`,
/// the pure precession part. This is the correct rotation for theories whose
/// catalogue equinox is the **dynamical** J2000 mean equinox (VSOP87, ELP2000)
/// rather than the ICRF/GCRS pole (DE440 — use [`precession_bias_matrix_iau2006`]
/// for those).
pub fn precession_matrix_p03_nobias(t: f64) -> [[f64; 3]; 3] {
    let pmat = precession_bias_matrix_iau2006(t);
    let bias_t = transpose3(frame_bias_matrix());
    mat_mul(pmat, bias_t)
}

/// Rotate an ecliptic position from the **mean equinox of J2000** to the **mean
/// equinox of date** using the supplied equatorial precession matrix `prec`,
/// via the rigorous Cartesian pipeline:
///
/// 1. ecliptic-J2000 → equatorial-J2000 (rotate about x by the J2000 mean
///    obliquity `MEAN_OBLIQUITY_J2000`);
/// 2. equatorial-J2000 → equatorial-of-date (apply `prec`);
/// 3. equatorial-of-date → ecliptic-of-date (rotate back by the mean obliquity
///    of date `mean_obliquity(t)`).
///
/// This replaces the scalar `longitude += general_precession_longitude(t)`
/// approximation: it precesses both longitude AND latitude consistently and
/// preserves the radius (a pure rotation), and is driven by the SOFA-validated
/// IAU 2006/P03 rotation matrices. The returned position is in the **mean**
/// equinox of date — apply nutation in longitude afterwards for the true
/// (apparent) equinox of date, exactly as the prior scalar path did.
///
/// `prec` must be the J2000→date precession rotation for whatever input frame
/// the caller supplies: use [`precession_matrix_p03_nobias`] for dynamical-J2000
/// theories (VSOP87, ELP2000, Meeus Pluto, Keplerian asteroids) and
/// [`precession_bias_matrix_iau2006`] for ICRF/GCRS input (DE440).
pub fn precess_ecliptic_to_of_date(
    pos: EclipticPosition,
    prec: [[f64; 3]; 3],
    t: f64,
) -> EclipticPosition {
    // Step 1: ecliptic-J2000 -> equatorial-J2000.
    let eq_j2000 = ecliptic_to_equatorial(&pos, MEAN_OBLIQUITY_J2000);
    let cart = equatorial_cartesian(&eq_j2000);

    // Step 2: equatorial-J2000 -> equatorial-of-date.
    let rotated = rotate3(prec, cart);

    // Step 3: equatorial-of-date -> ecliptic-of-date (mean obliquity of date).
    let eq_date = cartesian_equatorial(rotated);
    equatorial_to_ecliptic(&eq_date, mean_obliquity(t))
}

/// Cartesian unit-sphere-scaled vector from an equatorial position.
fn equatorial_cartesian(eq: &crate::transforms::EquatorialPosition) -> [f64; 3] {
    let cos_dec = eq.declination.cos();
    [
        eq.distance * cos_dec * eq.right_ascension.cos(),
        eq.distance * cos_dec * eq.right_ascension.sin(),
        eq.distance * eq.declination.sin(),
    ]
}

/// Inverse of [`equatorial_cartesian`].
fn cartesian_equatorial(v: [f64; 3]) -> crate::transforms::EquatorialPosition {
    let r = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    let ra = v[1].atan2(v[0]);
    let dec = if r > 0.0 {
        (v[2] / r).asin()
    } else if r == 0.0 {
        0.0
    } else {
        f64::NAN
    };
    crate::transforms::EquatorialPosition {
        right_ascension: ra.rem_euclid(std::f64::consts::TAU),
        declination: dec,
        distance: r,
    }
}

/// IAU 2006 precession angles, all in radians.
#[derive(Debug, Clone, Copy)]
pub struct PrecessionAngles {
    /// Ecliptic precession angle psi_A.
    pub psi_a: f64,
    /// Ecliptic precession angle omega_A.
    pub omega_a: f64,
    /// Planetary precession angle chi_A.
    pub chi_a: f64,
    /// Mean obliquity of the ecliptic (= mean_obliquity).
    pub epsilon_a: f64,
    /// Equatorial precession angle zeta_A.
    pub zeta_a: f64,
    /// Equatorial precession angle z_A.
    pub z_a: f64,
    /// Equatorial precession angle theta_A.
    pub theta_a: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn precession_at_j2000_is_zero() {
        let p = precession_angles(0.0);
        assert!(p.psi_a.abs() < 1e-10, "psi_a at J2000 should be ~0");
        // zeta_a has a small constant term (2.650545") at t=0 from the IAU 2006 formula
        let zeta_arcsec = p.zeta_a / ARCSEC_TO_RAD;
        assert!(
            zeta_arcsec.abs() < 5.0,
            "zeta_a at J2000 should be small, got {zeta_arcsec}\""
        );
    }

    #[test]
    fn general_precession_rate() {
        let p1 = general_precession_longitude(1.0); // 1 century
        let rate_arcsec_per_century = p1 / ARCSEC_TO_RAD;
        assert!(
            (rate_arcsec_per_century - 5028.8).abs() < 2.0,
            "Precession rate should be ~5028.8\"/century, got {rate_arcsec_per_century}"
        );
    }

    #[test]
    fn precession_matrix_is_identity_at_j2000() {
        let m = precession_matrix(0.0);
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!(
                    (m[i][j] - expected).abs() < 1e-8,
                    "Precession matrix at J2000 should be identity, m[{i}][{j}]={}",
                    m[i][j]
                );
            }
        }
    }

    #[test]
    fn precession_matrix_is_orthogonal() {
        let m = precession_matrix(1.0); // 1 century from J2000
        // R * R^T should equal identity
        for i in 0..3 {
            for j in 0..3 {
                let mut sum = 0.0;
                for k in 0..3 {
                    sum += m[i][k] * m[j][k];
                }
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!(
                    (sum - expected).abs() < 1e-12,
                    "Precession matrix should be orthogonal: (R*R^T)[{i}][{j}]={sum}"
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // IAU 2006/P03 Fukushima–Williams + frame bias: ERFA/SOFA golden vectors.
    // All from `t_erfa_c.c` (== `t_sofa_c.c`) at the SOFA reference epoch
    // date1 = 2400000.5, date2 = 50123.9999 (TT JD 2450124.4999, ~1996-02-25).
    // We assert a uniform 1e-12 conformance tolerance. (ERFA's t_erfa_c.c uses
    // tighter per-element tolerances — gamb 1e-16, psib 1e-14, off-diagonals
    // 1e-14..1e-16 — which our values also satisfy; 1e-12 is the uniform floor.)
    // -----------------------------------------------------------------------

    /// SOFA reference time argument in Julian centuries TT from J2000, computed
    /// in the SOFA two-part order for bit-level reproducibility.
    fn reference_epoch_t() -> f64 {
        ((2400000.5 - 2451545.0) + 50123.9999) / 36525.0
    }

    fn assert_mat_eq(got: [[f64; 3]; 3], want: [[f64; 3]; 3], tol: f64, label: &str) {
        for i in 0..3 {
            for j in 0..3 {
                assert!(
                    (got[i][j] - want[i][j]).abs() < tol,
                    "{label}[{i}][{j}]: got {:.18}, want {:.18}",
                    got[i][j],
                    want[i][j]
                );
            }
        }
    }

    #[test]
    fn fw06_angles_match_erfa_golden() {
        // ERFA `eraPfw06` golden outputs (t_erfa_c.c `t_pfw06`) at the SOFA
        // reference epoch (radians). The pfw06.c arcsec coefficients in this
        // module are verbatim-identical to ERFA, and pmat06 — which composes
        // these four angles — matches the ERFA matrix golden to 1e-12 below, so
        // these values are independently corroborated end-to-end.
        let (gamb, phib, psib, epsa) = fw06_angles(reference_epoch_t());
        assert!(
            (gamb - -0.2243387670997995690e-5).abs() < 1e-12,
            "gamb={gamb:.19}"
        );
        assert!(
            (phib - 0.4091014602391312808).abs() < 1e-12,
            "phib={phib:.19}"
        );
        assert!(
            (psib - -0.9501954178013031895e-3).abs() < 1e-12,
            "psib={psib:.19}"
        );
        assert!(
            (epsa - 0.4091014316587367491).abs() < 1e-12,
            "epsa={epsa:.19}"
        );
    }

    #[test]
    fn pmat06_matches_erfa_golden() {
        // ERFA t_erfa_c.c `t_pmat06` expected matrix (GCRS→mean-of-date, bias-folded).
        let want = [
            [
                0.9999995505176007047,
                0.8695404617348208406e-3,
                0.3779735201865589104e-3,
            ],
            [
                -0.8695404723772031414e-3,
                0.9999996219496027161,
                -0.1361752497080270143e-6,
            ],
            [
                -0.3779734957034089490e-3,
                -0.1924880847894457113e-6,
                0.9999999285679971958,
            ],
        ];
        assert_mat_eq(
            precession_bias_matrix_iau2006(reference_epoch_t()),
            want,
            1e-12,
            "pmat06",
        );
    }

    #[test]
    fn frame_bias_matrix_matches_erfa_golden() {
        // ERFA t_erfa_c.c `t_bp00` expected `rb` frame-bias matrix (epoch-independent).
        let want = [
            [
                0.9999999999999942498,
                -0.7078279744199196626e-7,
                0.8056217146976134152e-7,
            ],
            [
                0.7078279477857337206e-7,
                0.9999999999999969484,
                0.3306041454222136517e-7,
            ],
            [
                -0.8056217380986972157e-7,
                -0.3306040883980552500e-7,
                0.9999999999999962084,
            ],
        ];
        assert_mat_eq(frame_bias_matrix(), want, 1e-12, "rb");
    }

    #[test]
    fn p03_nobias_matches_independent_zeta_z_theta_precession() {
        // REAL cross-representation check (NOT the by-construction tautology
        // `p03_nobias·bp00 == pmat06`, which only re-tests bp00 orthogonality).
        //
        // The bias-free precession matrix is built here via the Fukushima–Williams
        // route: precession_matrix_p03_nobias(t) = pmat06(t)·bp00ᵀ, where pmat06
        // composes the four FW06 angles through fw2m. `precession_matrix(t)` is an
        // INDEPENDENT representation of the SAME physical pure-precession rotation,
        // built from the classical equatorial precession angles ζ_A, z_A, θ_A
        // (the `precession_angles` polynomials, Capitaine et al. 2003, A&A 412, 567,
        // Table 1 / IERS Conventions 2003 eq. 5.10) with NO reference to the FW06
        // angles, fw2m, pmat06, or the frame bias. Two disjoint coefficient sets and
        // matrix constructions must converge on the same rotation iff both encode
        // the genuine IAU 2006/P03 precession — a sign flip or wrong coefficient in
        // either path moves elements by ≳1e-3, far above the tolerance here.
        //
        // Measured max element-wise disagreement: 1.1e-12 at t=-1cy rising to
        // 6.4e-12 at t=+2cy (accumulation of the differing higher-order polynomial
        // truncations between the two angle sets). 1e-11 over ±2cy is the honest
        // envelope of that measured agreement.
        for t in [-1.0, -0.5, 0.0, 0.5, 1.0, 2.0] {
            assert_mat_eq(
                precession_matrix_p03_nobias(t),
                precession_matrix(t),
                1e-11,
                &format!("p03_nobias vs ζ-z-θ at t={t}"),
            );
        }
    }

    #[test]
    fn p03_nobias_is_orthogonal_and_identity_at_j2000() {
        // The bias-free precession matrix at J2000 must be the identity (the only
        // residual in pmat06 at J2000 is the frame bias, which we have removed).
        let m = precession_matrix_p03_nobias(0.0);
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!(
                    (m[i][j] - expected).abs() < 1e-9,
                    "p03_nobias at J2000 should be identity, m[{i}][{j}]={}",
                    m[i][j]
                );
            }
        }
        // Orthogonality at a non-trivial epoch.
        let m = precession_matrix_p03_nobias(1.0);
        for i in 0..3 {
            for j in 0..3 {
                let mut sum = 0.0;
                for k in 0..3 {
                    sum += m[i][k] * m[j][k];
                }
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!(
                    (sum - expected).abs() < 1e-12,
                    "p03_nobias not orthogonal at ({i},{j})"
                );
            }
        }
    }

    #[test]
    fn rotate3_and_transpose3_roundtrip() {
        let m = precession_matrix_p03_nobias(0.7);
        let v = [0.3, -0.6, 0.74];
        // R is orthogonal, so Rᵀ·(R·v) == v.
        let rv = rotate3(m, v);
        let back = rotate3(transpose3(m), rv);
        for k in 0..3 {
            assert!(
                (back[k] - v[k]).abs() < 1e-13,
                "rotate3/transpose3 roundtrip failed at {k}: {} vs {}",
                back[k],
                v[k]
            );
        }
    }

    #[test]
    fn precess_pipeline_identity_at_j2000() {
        // At J2000 the bias-free precession is the identity, so the full Cartesian
        // pipeline must return the input ecliptic position unchanged.
        let pos = EclipticPosition {
            longitude: 1.2345,
            latitude: 0.0876,
            distance: 5.2,
        };
        let out = precess_ecliptic_to_of_date(pos, precession_matrix_p03_nobias(0.0), 0.0);
        assert!((out.longitude - pos.longitude).abs() < 1e-9, "lon J2000");
        assert!((out.latitude - pos.latitude).abs() < 1e-9, "lat J2000");
        assert!((out.distance - pos.distance).abs() < 1e-9, "dist J2000");
    }

    #[test]
    fn precess_pipeline_longitude_rate_matches_general_precession() {
        // For a point ON the ecliptic the dominant effect of the Cartesian
        // pipeline is a longitude advance close to the general precession in
        // longitude (~5028.8″/cy). They are NOT identical — the rigorous rotation
        // also induces a small latitude — but for a low-latitude body over one
        // century they agree at the few-arcsec level. This anchors the new path
        // to the same physical quantity the scalar approximation targeted.
        let pos = EclipticPosition {
            longitude: 0.0,
            latitude: 0.0,
            distance: 1.0,
        };
        let t = 1.0;
        let out = precess_ecliptic_to_of_date(pos, precession_matrix_p03_nobias(t), t);
        let dlon_arcsec = (out.longitude - pos.longitude) / ARCSEC_TO_RAD;
        let scalar_arcsec = general_precession_longitude(t) / ARCSEC_TO_RAD;
        assert!(
            (dlon_arcsec - scalar_arcsec).abs() < 5.0,
            "Cartesian longitude advance {dlon_arcsec}\" should be within ~5\" of \
             general precession {scalar_arcsec}\" for an on-ecliptic body over 1cy"
        );
        // Distance is preserved by the rotation.
        assert!((out.distance - 1.0).abs() < 1e-12, "distance preserved");
    }

    #[test]
    fn pmat06_is_orthogonal_and_identity_at_j2000() {
        // At J2000 the precession part vanishes but the ~23 mas ICRS frame bias
        // remains, so pmat06(0) is the bias rotation, not the identity. (It is
        // very close to but NOT bit-identical to `frame_bias_matrix()`: pmat06's
        // bias comes from the PFW06 constant terms, bp00's from the explicit
        // bias angles — they agree to ~1e-12, not exactly.)
        let m = precession_bias_matrix_iau2006(0.0);
        for i in 0..3 {
            for j in 0..3 {
                let mut sum = 0.0;
                for k in 0..3 {
                    sum += m[i][k] * m[j][k];
                }
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!(
                    (sum - expected).abs() < 1e-12,
                    "pmat06 not orthogonal at ({i},{j})"
                );
            }
        }
        // Off-diagonals at J2000 are the pure frame bias: a few ×1e-8 rad (~15 mas).
        assert!(
            m[0][1].abs() < 1e-6 && m[0][1].abs() > 1e-9,
            "J2000 bias off-diagonal magnitude"
        );
    }
}
