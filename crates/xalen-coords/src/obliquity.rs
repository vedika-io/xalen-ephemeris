use crate::ARCSEC_TO_RAD;

const OBLIQUITY_J2000_ARCSEC: f64 = 84381.406;

/// Compute the mean obliquity of the ecliptic (IAU 2006) in radians.
///
/// `t` is Julian centuries from J2000.0.
pub fn mean_obliquity(t: f64) -> f64 {
    let eps = OBLIQUITY_J2000_ARCSEC - 46.836769 * t - 0.0001831 * t * t + 0.00200340 * t * t * t
        - 0.000000576 * t.powi(4)
        - 0.0000000434 * t.powi(5);
    eps * ARCSEC_TO_RAD
}

/// Compute the true obliquity (mean + nutation in obliquity) in radians.
pub fn true_obliquity(t: f64, delta_epsilon: f64) -> f64 {
    mean_obliquity(t) + delta_epsilon
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RAD_TO_DEG;

    #[test]
    fn obliquity_at_j2000() {
        let eps = mean_obliquity(0.0) * RAD_TO_DEG;
        assert!(
            (eps - 23.4393).abs() < 0.001,
            "Obliquity at J2000 should be ~23.4393°, got {eps}°"
        );
    }

    #[test]
    fn obliquity_decreases_over_centuries() {
        let eps_2000 = mean_obliquity(0.0);
        let eps_2100 = mean_obliquity(1.0);
        assert!(eps_2100 < eps_2000, "Obliquity should decrease over time");
    }
}
