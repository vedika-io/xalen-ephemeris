use crate::ARCSEC_TO_RAD;

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
}
