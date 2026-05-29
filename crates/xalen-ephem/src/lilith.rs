//! Black Moon Lilith (Mean Lunar Apogee) and Priapus computation.
//!
//! In Western astrology, **Black Moon Lilith** is the mean lunar apogee — the
//! point on the Moon's orbit farthest from Earth, projected onto the ecliptic.
//! It is NOT a physical body; it is the mean position of the empty focus of the
//! Moon's elliptical orbit (the Earth occupies the other focus).
//!
//! The mean apogee longitude advances at roughly 40.66° per year (one full
//! revolution in ~8.85 years).  Swiss Ephemeris exposes this as SE_MEAN_APOG
//! (body 12); most Western astrology apps label it "Lilith" or "Mean Lilith."
//!
//! **Priapus** (also called the "anti-Lilith" or "lunar perigee") is the point
//! diametrically opposite Lilith on the ecliptic: Priapus = Lilith + 180°.
//!
//! # Formula
//!
//! Mean longitude of the lunar perigee (varpi_Moon) from Meeus Ch.47
//! (IAU polynomial, same source as the mean node):
//!
//! ```text
//!   varpi = 83.3532465° + 4069.0137287° T − 0.0103200° T²
//!           − T³/80053 + T⁴/18999000
//! ```
//!
//! where T = Julian centuries from J2000.0 (TT).
//!
//! This is the longitude of the Moon's *perigee* (closest approach).
//! The *apogee* (Lilith) is the opposite point: `apogee = perigee + 180°`.
//!
//! All functions return ecliptic longitude in **radians** [0, 2π).

use xalen_time::{JdTT, JulianDay};

/// Mean longitude of the lunar apogee (Black Moon Lilith).
///
/// Computed as the mean perigee longitude (Meeus Ch.47 polynomial) + 180°.
///
/// Returns ecliptic longitude in radians [0, 2π).
pub fn mean_lilith(jd_tt: JdTT) -> f64 {
    let perigee = mean_perigee_longitude(jd_tt);
    (perigee + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU)
}

/// Mean longitude of the lunar perigee.
///
/// IAU polynomial from Meeus (2nd ed.) Chapter 47:
///
/// ```text
///   varpi = 83.3532465 + 4069.0137287*T − 0.0103200*T²
///           − T³/80053 + T⁴/18999000   (degrees)
/// ```
///
/// where T = Julian centuries from J2000.0 (TT).
///
/// Returns ecliptic longitude in radians [0, 2π).
pub fn mean_perigee_longitude(jd_tt: JdTT) -> f64 {
    let t = jd_tt.julian_centuries_from_j2000();
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;

    let varpi_deg = 83.3532465 + 4069.0137287 * t - 0.0103200 * t2 - t3 / 80053.0 + t4 / 18999000.0;

    varpi_deg.to_radians().rem_euclid(std::f64::consts::TAU)
}

/// Priapus (anti-Lilith) longitude = Lilith + 180° = mean perigee longitude.
///
/// `lilith_rad` should be the longitude of Lilith (mean apogee) in radians,
/// as returned by [`mean_lilith`].
///
/// Returns ecliptic longitude in radians [0, 2π).
pub fn priapus(lilith_rad: f64) -> f64 {
    // Priapus = Lilith + 180° = perigee (by construction)
    (lilith_rad + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use xalen_coords::RAD_TO_DEG;

    #[test]
    fn mean_lilith_at_j2000() {
        let lon = mean_lilith(JdTT::J2000);
        let deg = lon * RAD_TO_DEG;
        // Mean perigee at J2000 ≈ 83.35°, so mean apogee (Lilith) ≈ 263.35°.
        // Swiss Ephemeris gives Mean Apogee at J2000 ≈ 263.4°.
        assert!(
            (deg - 263.35).abs() < 0.5,
            "Mean Lilith at J2000 should be ~263.35°, got {deg}°"
        );
    }

    #[test]
    fn mean_perigee_at_j2000() {
        let lon = mean_perigee_longitude(JdTT::J2000);
        let deg = lon * RAD_TO_DEG;
        // Mean perigee at J2000 ≈ 83.35° (direct from polynomial at T=0).
        assert!(
            (deg - 83.35).abs() < 0.5,
            "Mean perigee at J2000 should be ~83.35°, got {deg}°"
        );
    }

    #[test]
    fn lilith_valid_range() {
        for offset in [0.0, 365.25, 3652.5, -3652.5, 18262.5] {
            let lon = mean_lilith(JdTT(2451545.0 + offset));
            assert!(
                lon >= 0.0 && lon < std::f64::consts::TAU,
                "Lilith should be in [0, 2π), got {lon}"
            );
        }
    }

    #[test]
    fn lilith_prograde_motion() {
        // The mean apogee advances ~40.66°/year (prograde).
        let p1 = mean_lilith(JdTT(2451545.0));
        let p2 = mean_lilith(JdTT(2451545.0 + 365.25));
        let mut diff_deg = (p2 - p1) * RAD_TO_DEG;
        if diff_deg < -180.0 {
            diff_deg += 360.0;
        }
        if diff_deg > 180.0 {
            diff_deg -= 360.0;
        }
        // Should advance roughly 40-42° per year.
        assert!(
            diff_deg > 38.0 && diff_deg < 44.0,
            "Lilith should advance ~40.66°/year, got {diff_deg}°"
        );
    }

    #[test]
    fn lilith_full_cycle_about_8_85_years() {
        // The mean apogee completes one revolution in ~8.85 years (≈3232 days).
        // After 8.85 years the longitude should return near the starting value.
        let start = mean_lilith(JdTT(2451545.0));
        let after_cycle = mean_lilith(JdTT(2451545.0 + 8.85 * 365.25));
        let mut diff = (after_cycle - start) * RAD_TO_DEG;
        if diff > 180.0 {
            diff -= 360.0;
        }
        if diff < -180.0 {
            diff += 360.0;
        }
        assert!(
            diff.abs() < 5.0,
            "Lilith should return near start after ~8.85 years, got diff = {diff}°"
        );
    }

    #[test]
    fn priapus_opposite_lilith() {
        let lilith = mean_lilith(JdTT::J2000);
        let pri = priapus(lilith);
        let diff_deg = ((pri - lilith).abs() * RAD_TO_DEG).rem_euclid(360.0);
        assert!(
            (diff_deg - 180.0).abs() < 0.01,
            "Priapus should be 180° from Lilith, got diff = {diff_deg}°"
        );
    }

    #[test]
    fn priapus_equals_perigee() {
        // Priapus = Lilith + 180° = perigee by construction.
        let lilith = mean_lilith(JdTT::J2000);
        let pri = priapus(lilith);
        let perigee = mean_perigee_longitude(JdTT::J2000);
        let diff = ((pri - perigee) * RAD_TO_DEG).rem_euclid(360.0);
        assert!(
            diff < 0.001 || (360.0 - diff).abs() < 0.001,
            "Priapus should equal mean perigee, got diff = {diff}°"
        );
    }

    #[test]
    fn priapus_valid_range() {
        for offset in [0.0, 365.25, 3652.5, -3652.5] {
            let lilith = mean_lilith(JdTT(2451545.0 + offset));
            let pri = priapus(lilith);
            assert!(
                pri >= 0.0 && pri < std::f64::consts::TAU,
                "Priapus should be in [0, 2π), got {pri}"
            );
        }
    }

    #[test]
    fn lilith_at_known_epoch_2020() {
        // 2020-01-01 12:00 TT = JD 2458849.0
        // Swiss Ephemeris mean apogee on 2020-01-01: ~11° Pisces ≈ 341°
        let lon = mean_lilith(JdTT(2458849.0));
        let deg = lon * RAD_TO_DEG;
        assert!(
            deg > 330.0 || deg < 10.0,
            "Lilith in 2020 should be near 341°, got {deg}°"
        );
    }

    #[test]
    fn lilith_continuous_over_decade() {
        // Verify no jumps or discontinuities in a sweep across 10 years.
        let start_jd = 2451545.0; // J2000
        let step = 30.0; // ~monthly
        let n_steps = (10.0 * 365.25 / step) as usize;

        let mut prev = mean_lilith(JdTT(start_jd)) * RAD_TO_DEG;
        for i in 1..=n_steps {
            let jd = JdTT(start_jd + i as f64 * step);
            let curr = mean_lilith(jd) * RAD_TO_DEG;
            let mut delta = curr - prev;
            if delta > 180.0 {
                delta -= 360.0;
            }
            if delta < -180.0 {
                delta += 360.0;
            }
            // Monthly motion should be ~3.4° (40.66° / 12). Allow wide margin.
            assert!(
                delta.abs() < 10.0,
                "Discontinuity at step {i}: {prev:.2}° -> {curr:.2}° (delta {delta:.2}°)"
            );
            prev = curr;
        }
    }
}
