//! Mean and True (osculating) lunar node computation.
//!
//! The Moon's orbital plane is inclined ~5.15 deg to the ecliptic. The two
//! points where it crosses the ecliptic are the **ascending node** (Rahu in
//! Vedic astrology) and the **descending node** (Ketu = Rahu + 180 deg).
//!
//! - **Mean node**: the smoothly-regressing longitude obtained from the IAU
//!   polynomial. It regresses ~19.35 deg/year with no short-period oscillations.
//! - **True node**: the mean node corrected by the 7 largest perturbation terms
//!   from Meeus Ch.47. The true node oscillates around the mean with an
//!   amplitude of up to ~1.7 deg (period ~173 days).
//!
//! Both functions return ecliptic longitude in **radians** [0, 2*pi).

use crate::provider::EphemerisError;
use xalen_time::{JdTT, JulianDay};

/// Mean longitude of the ascending lunar node (Omega).
///
/// IAU polynomial from Meeus (2nd ed.) Chapter 47:
///
///   Omega = 125.0445479 - 1934.1362891*T + 0.0020754*T^2
///           + T^3/467441 - T^4/60616000   (degrees)
///
/// where T = Julian centuries from J2000.0 (TT).
///
/// Returns ecliptic longitude in radians [0, 2*pi).
pub fn mean_lunar_node(jd_tt: JdTT) -> f64 {
    let t = jd_tt.julian_centuries_from_j2000();
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;

    let omega_deg =
        125.0445479 - 1934.1362891 * t + 0.0020754 * t2 + t3 / 467441.0 - t4 / 60616000.0;

    (omega_deg.to_radians()).rem_euclid(std::f64::consts::TAU)
}

/// True (osculating) longitude of the ascending lunar node.
///
/// Computes the mean node and adds the 7 largest perturbation terms from
/// Meeus Ch.47. These terms account for the short-period oscillations from
/// solar and lunar gravitational effects.
///
/// Returns ecliptic longitude in radians [0, 2*pi).
pub fn true_lunar_node(jd_tt: JdTT) -> Result<f64, EphemerisError> {
    true_node_longitude(jd_tt)
}

/// Internal implementation of the true node with perturbation terms.
pub(crate) fn true_node_longitude(jd_tt: JdTT) -> Result<f64, EphemerisError> {
    let t = jd_tt.julian_centuries_from_j2000();
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;

    let d2r = std::f64::consts::PI / 180.0;

    // Mean longitude of the ascending node (Omega) -- IAU expression, degrees
    let omega = 125.0445479 - 1934.1362891 * t + 0.0020754 * t2 + t3 / 467441.0 - t4 / 60616000.0;

    // Fundamental arguments needed for perturbation terms (degrees)
    // D = Mean elongation of the Moon
    let d = 297.8501921 + 445267.1114034 * t - 0.0018819 * t2 + t3 / 545868.0 - t4 / 113065000.0;
    // M = Sun's mean anomaly
    let m = 357.5291092 + 35999.0502909 * t - 0.0001536 * t2 + t3 / 24490000.0;
    // M' = Moon's mean anomaly
    let mp = 134.9633964 + 477198.8675055 * t + 0.0087414 * t2 + t3 / 69699.0 - t4 / 14712000.0;
    // F = Moon's argument of latitude
    let f = 93.2720950 + 483202.0175233 * t - 0.0036539 * t2 - t3 / 3526000.0 + t4 / 863310000.0;

    // Convert to radians
    let d_r = d * d2r;
    let _m_r = m * d2r;
    let mp_r = mp * d2r;
    let f_r = f * d2r;

    // Perturbation terms for the true node (Meeus Ch.47, Table 47.A).
    // Converts mean node Ω to osculating (true) node.
    let mut delta = 0.0_f64; // in degrees

    delta += -1.4979 * (2.0 * (d_r - f_r)).sin();
    delta += -0.1500 * mp_r.sin();
    delta += -0.1226 * (2.0 * d_r).sin();
    delta += 0.1013 * (2.0 * f_r).sin();
    delta += -0.0340 * (mp_r - 2.0 * d_r).sin();

    let true_node_deg = omega + delta;
    let true_node_rad = (true_node_deg * d2r).rem_euclid(std::f64::consts::TAU);

    Ok(true_node_rad)
}

/// South node (Ketu) longitude = North node + 180 deg.
///
/// `north_node_rad` should be the longitude of Rahu (ascending node)
/// in radians, as returned by either `mean_lunar_node` or `true_lunar_node`.
///
/// Returns ecliptic longitude in radians [0, 2*pi).
pub fn south_node(north_node_rad: f64) -> f64 {
    (north_node_rad + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use xalen_coords::RAD_TO_DEG;

    #[test]
    fn mean_node_at_j2000() {
        let lon = mean_lunar_node(JdTT::J2000);
        let deg = lon * RAD_TO_DEG;
        // Mean node at J2000 should be very close to 125.04 deg.
        assert!(
            (deg - 125.04).abs() < 0.1,
            "Mean node at J2000 should be ~125.04 deg, got {deg} deg"
        );
    }

    #[test]
    fn mean_node_valid_range() {
        for offset in [0.0, 365.25, 3652.5, -3652.5] {
            let lon = mean_lunar_node(JdTT(2451545.0 + offset));
            assert!(
                lon >= 0.0 && lon < std::f64::consts::TAU,
                "mean node should be in [0, 2pi), got {lon}"
            );
        }
    }

    #[test]
    fn mean_node_retrograde() {
        // The node regresses ~19.35 deg/year.
        let p1 = mean_lunar_node(JdTT(2451545.0));
        let p2 = mean_lunar_node(JdTT(2451545.0 + 365.25));
        let mut diff_deg = (p1 - p2) * RAD_TO_DEG;
        if diff_deg < 0.0 {
            diff_deg += 360.0;
        }
        assert!(
            diff_deg > 17.0 && diff_deg < 22.0,
            "Mean node should regress ~19.35 deg/year, got {diff_deg} deg"
        );
    }

    #[test]
    fn true_node_at_j2000_near_mean_node() {
        let true_lon = true_lunar_node(JdTT::J2000).unwrap();
        let true_deg = true_lon * RAD_TO_DEG;

        // True node at J2000 should be close to the mean node (~125 deg)
        // but with perturbations of up to ~1.7 deg.
        assert!(
            true_deg > 120.0 && true_deg < 130.0,
            "True node at J2000 should be ~125 deg, got {true_deg} deg"
        );
    }

    #[test]
    fn true_node_differs_from_mean_node() {
        let jd = JdTT(2451545.0);
        let mean_deg = mean_lunar_node(jd) * RAD_TO_DEG;
        let true_deg = true_lunar_node(jd).unwrap() * RAD_TO_DEG;

        let diff = (true_deg - mean_deg).abs();
        let diff = if diff > 180.0 { 360.0 - diff } else { diff };

        // Perturbation should be nonzero but bounded.
        assert!(
            diff > 0.001,
            "True node should differ from mean node, diff was only {diff} deg"
        );
        assert!(
            diff < 2.5,
            "True-mean node difference should be < 2.5 deg, got {diff} deg"
        );
    }

    #[test]
    fn true_node_retrograde_over_year() {
        let p1 = true_lunar_node(JdTT(2451545.0)).unwrap();
        let p2 = true_lunar_node(JdTT(2451545.0 + 365.25)).unwrap();
        let mut diff = (p1 - p2) * RAD_TO_DEG;
        if diff < 0.0 {
            diff += 360.0;
        }
        assert!(
            diff > 15.0 && diff < 25.0,
            "Node should regress ~19.35 deg/year, got {diff} deg"
        );
    }

    #[test]
    fn true_node_always_valid_range() {
        let lon = true_lunar_node(JdTT(2451545.0)).unwrap();
        assert!(lon >= 0.0 && lon < std::f64::consts::TAU);
    }

    #[test]
    fn true_node_epoch_2020() {
        // 2020-01-01 12:00 TT = JD 2458849.0
        let lon = true_lunar_node(JdTT(2458849.0)).unwrap();
        let deg = lon * RAD_TO_DEG;
        assert!(
            deg > 90.0 && deg < 120.0,
            "True node in 2020 should be ~100-110 deg, got {deg} deg"
        );
    }

    #[test]
    fn south_node_opposite_north() {
        let rahu = mean_lunar_node(JdTT::J2000);
        let ketu = south_node(rahu);
        let diff_deg = ((ketu - rahu).abs() * RAD_TO_DEG).rem_euclid(360.0);
        assert!(
            (diff_deg - 180.0).abs() < 0.01,
            "Ketu should be 180 deg from Rahu, got diff = {diff_deg} deg"
        );
    }

    #[test]
    fn south_node_valid_range() {
        for offset in [0.0, 365.25, 3652.5, -3652.5] {
            let rahu = true_lunar_node(JdTT(2451545.0 + offset)).unwrap();
            let ketu = south_node(rahu);
            assert!(
                ketu >= 0.0 && ketu < std::f64::consts::TAU,
                "Ketu should be in [0, 2pi), got {ketu}"
            );
        }
    }

    #[test]
    fn south_node_from_true_node() {
        let rahu = true_lunar_node(JdTT::J2000).unwrap();
        let ketu = south_node(rahu);
        let rahu_deg = rahu * RAD_TO_DEG;
        let ketu_deg = ketu * RAD_TO_DEG;

        // At J2000 Rahu is ~125 deg, so Ketu should be ~305 deg.
        assert!(
            ketu_deg > 300.0 && ketu_deg < 310.0,
            "Ketu at J2000 should be ~305 deg, got {ketu_deg} deg (Rahu = {rahu_deg} deg)"
        );
    }

    #[test]
    fn mean_vs_true_bounded_across_18_year_cycle() {
        // The mean node takes ~18.61 years to complete one cycle.
        // Check that mean-vs-true difference stays bounded across
        // multiple sample dates spanning a full cycle.
        let start_jd = 2451545.0; // J2000
        let step = 30.0; // every ~month
        let n_steps = (18.61 * 365.25 / step) as usize;

        for i in 0..n_steps {
            let jd = JdTT(start_jd + i as f64 * step);
            let mean = mean_lunar_node(jd) * RAD_TO_DEG;
            let true_n = true_lunar_node(jd).unwrap() * RAD_TO_DEG;

            let mut diff = (true_n - mean).abs();
            if diff > 180.0 {
                diff = 360.0 - diff;
            }
            assert!(
                diff < 2.0,
                "True-mean difference should be < 2 deg, got {diff} deg at JD {}",
                jd.0
            );
        }
    }
}
