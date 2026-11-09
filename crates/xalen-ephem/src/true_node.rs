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
/// Returns the rigorous osculating node (see [`true_node_longitude`]), in radians
/// [0, 2*pi). This is the node type Swiss Ephemeris reports as `SE_TRUE_NODE`.
pub fn true_lunar_node(jd_tt: JdTT) -> Result<f64, EphemerisError> {
    true_node_longitude(jd_tt)
}

/// True (osculating) longitude of the ascending lunar node — the rigorous
/// definition Swiss Ephemeris uses for `SE_TRUE_NODE`.
///
/// The true node is the ascending node of the Moon's *instantaneous* orbital
/// plane: the line where that plane meets the ecliptic. We derive it directly
/// from the Moon's geocentric state vector rather than approximating the
/// mean→true correction with a handful of sine terms:
///
/// ```text
///   r = Moon position (ecliptic of date, Cartesian)
///   v = dr/dt                      (central finite difference)
///   h = r × v                      (angular momentum ⟂ orbital plane)
///   n = ẑ × h = (−h_y, h_x, 0)     (line of nodes → ascending node)
///   Ω = atan2(n_y, n_x) = atan2(h_x, −h_y)
/// ```
///
/// Only the *direction* of `h` enters Ω, so the distance units and the velocity
/// scale cancel — solely the orbital-plane orientation matters. This replaced a
/// 5-term sine approximation that drifted up to 0.32° vs Swiss across 1920–2050;
/// the osculating method tracks Swiss's true node to well under 0.1°.
///
/// Returns ecliptic longitude in radians [0, 2*pi).
pub(crate) fn true_node_longitude(jd_tt: JdTT) -> Result<f64, EphemerisError> {
    use xalen_coords::ecliptic_to_cartesian;

    // Central-difference step for the velocity. 0.05 d ≈ 72 min: small vs the
    // 27.3-day orbit, yet above the lunar series' arcsecond noise floor.
    const DT: f64 = 0.05;

    let r_prev = ecliptic_to_cartesian(&crate::moon::geocentric_moon(JdTT(jd_tt.as_f64() - DT))?);
    let r_now = ecliptic_to_cartesian(&crate::moon::geocentric_moon(jd_tt)?);
    let r_next = ecliptic_to_cartesian(&crate::moon::geocentric_moon(JdTT(jd_tt.as_f64() + DT))?);

    // Velocity by central finite difference.
    let vx = (r_next.x - r_prev.x) / (2.0 * DT);
    let vy = (r_next.y - r_prev.y) / (2.0 * DT);
    let vz = (r_next.z - r_prev.z) / (2.0 * DT);

    // Specific angular momentum h = r × v (normal to the orbital plane).
    let hx = r_now.y * vz - r_now.z * vy;
    let hy = r_now.z * vx - r_now.x * vz;

    // Line of nodes n = ẑ × h = (−h_y, h_x, 0); its longitude is the ascending node.
    let omega = hx.atan2(-hy).rem_euclid(std::f64::consts::TAU);
    Ok(omega)
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
                (0.0..std::f64::consts::TAU).contains(&lon),
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
        assert!((0.0..std::f64::consts::TAU).contains(&lon));
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
                (0.0..std::f64::consts::TAU).contains(&ketu),
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
