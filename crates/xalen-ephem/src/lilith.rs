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

use crate::provider::EphemerisError;
use xalen_time::{JdTT, JulianDay};

/// Standard gravitational parameter of the **Earth–Moon system**, in
/// AU³ · day⁻². Derived from the canonical Earth GM = 398600.4418 km³ s⁻² and
/// the DE Moon/Earth mass ratio 1/81.30056:
///
/// ```text
///   GM_EM = GM_Earth · (1 + 1/81.30056) · 86400² / (149_597_870.7)³
///         = 8.997011546044162e-10  AU³/day²
/// ```
///
/// Used to build the osculating (instantaneous Keplerian) orbit of the Moon for
/// the True (osculating) apogee — the apogee direction is the +180° flip of the
/// eccentricity (Laplace–Runge–Lenz) vector, which depends on this GM.
const GM_EARTH_MOON_AU3_DAY2: f64 = 8.997_011_546_044_162e-10;

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

/// True (osculating) longitude of the lunar apogee — the **Black Moon Lilith**
/// that Swiss Ephemeris reports as `SE_OSCU_APOG` (body 13).
///
/// Where [`mean_lilith`] is the smoothly-advancing mean apogee (Meeus Ch.47
/// polynomial), the *osculating* apogee is the apogee of the Moon's
/// **instantaneous** Keplerian orbit at this instant — it oscillates by several
/// degrees about the mean with the lunar evection/variation rhythm. We derive it
/// directly from the Moon's geocentric state vector, exactly as
/// [`crate::true_node::true_node_longitude`] derives the osculating node:
///
/// ```text
///   r = Moon position (ecliptic of date, Cartesian, AU)
///   v = dr/dt                            (central finite difference, AU/day)
///   e = ((|v|² − μ/|r|)·r − (r·v)·v) / μ  (Laplace–Runge–Lenz vector)
/// ```
///
/// The eccentricity vector `e` points toward **perigee**; the apogee is the
/// opposite direction, so `apogee = atan2(e_y, e_x) + 180°`. `μ` is the
/// Earth–Moon [`GM_EARTH_MOON_AU3_DAY2`]. Only the orbital geometry enters, so
/// the result is the osculating apogee in the ecliptic of date.
///
/// Validated vs pyswisseph 2.10.03 `SE_OSCU_APOG` at committed spot fixtures —
/// J2000 (252.979°) and the 1992-04-12 Meeus epoch (331.958°) — each within 0.5°.
/// (The osculating apogee is intrinsically model-sensitive — Swiss itself
/// documents large differences between osculating-apogee definitions; the small
/// definitional spread, not a coding error, is what sets this bound.)
///
/// Returns ecliptic longitude in radians [0, 2π).
pub fn true_lilith(jd_tt: JdTT) -> Result<f64, EphemerisError> {
    osculating_apogee_longitude(jd_tt)
}

/// Rigorous osculating-apogee longitude (radians, [0, 2π)). Backing
/// implementation of [`true_lilith`]; reuses the Moon state vector machinery.
pub(crate) fn osculating_apogee_longitude(jd_tt: JdTT) -> Result<f64, EphemerisError> {
    use xalen_coords::ecliptic_to_cartesian;

    // Same central-difference step as the true-node derivation: 0.05 d ≈ 72 min,
    // small vs the 27.3-day orbit yet above the lunar series' arcsecond noise.
    const DT: f64 = 0.05;

    let r_prev = ecliptic_to_cartesian(&crate::moon::geocentric_moon(JdTT(jd_tt.as_f64() - DT))?);
    let r_now = ecliptic_to_cartesian(&crate::moon::geocentric_moon(jd_tt)?);
    let r_next = ecliptic_to_cartesian(&crate::moon::geocentric_moon(JdTT(jd_tt.as_f64() + DT))?);

    // Velocity by central finite difference (AU/day).
    let vx = (r_next.x - r_prev.x) / (2.0 * DT);
    let vy = (r_next.y - r_prev.y) / (2.0 * DT);
    let vz = (r_next.z - r_prev.z) / (2.0 * DT);

    let r_mag = (r_now.x * r_now.x + r_now.y * r_now.y + r_now.z * r_now.z).sqrt();
    let v2 = vx * vx + vy * vy + vz * vz;
    let r_dot_v = r_now.x * vx + r_now.y * vy + r_now.z * vz;
    let mu = GM_EARTH_MOON_AU3_DAY2;

    // Laplace–Runge–Lenz (eccentricity) vector e = ((v² − μ/r)·r − (r·v)·v)/μ,
    // pointing toward PERIGEE.
    let coeff = v2 - mu / r_mag;
    let ex = (coeff * r_now.x - r_dot_v * vx) / mu;
    let ey = (coeff * r_now.y - r_dot_v * vy) / mu;

    // Perigee longitude is the e-vector direction in the ecliptic plane; the
    // apogee (Lilith) is the opposite point.
    let perigee = ey.atan2(ex);
    Ok((perigee + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU))
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

    // ── True (osculating) Lilith — SE_OSCU_APOG ──────────────────────────────

    /// True (osculating) Lilith at J2000 vs pyswisseph 2.10.03 `SE_OSCU_APOG`:
    /// 252.979401°. The osculating apogee is model-sensitive (Swiss documents
    /// large definitional spread), so 0.5° is a meaningful, achievable bound at
    /// this committed spot fixture.
    #[test]
    fn true_lilith_at_j2000_matches_pyswisseph() {
        let lon = true_lilith(JdTT::J2000).unwrap() * RAD_TO_DEG;
        let diff = ((lon - 252.979401).abs()) % 360.0;
        let diff = diff.min(360.0 - diff);
        assert!(
            diff < 0.5,
            "True Lilith at J2000 should be ~252.979° (Swiss), got {lon}° (diff {diff}°)"
        );
    }

    /// Osculating apogee at 1992-04-12 (Meeus example epoch) vs pyswisseph
    /// `SE_OSCU_APOG` = 331.957857°.
    #[test]
    fn true_lilith_at_1992_matches_pyswisseph() {
        let lon = true_lilith(JdTT(2448724.5)).unwrap() * RAD_TO_DEG;
        let diff = ((lon - 331.957857).abs()) % 360.0;
        let diff = diff.min(360.0 - diff);
        assert!(
            diff < 0.5,
            "True Lilith 1992-04-12 should be ~331.96°, got {lon}° (diff {diff}°)"
        );
    }

    /// The osculating apogee must OSCILLATE about the mean apogee by several
    /// degrees (that is its whole point — it is not the smooth mean). Sample a
    /// few epochs and confirm a non-trivial spread, while staying bounded.
    #[test]
    fn true_lilith_oscillates_about_mean() {
        let mut max_diff: f64 = 0.0;
        let mut any_nonzero = false;
        for k in 0..40 {
            let jd = JdTT(2451545.0 + k as f64 * 20.0);
            let mean = mean_lilith(jd) * RAD_TO_DEG;
            let osc = true_lilith(jd).unwrap() * RAD_TO_DEG;
            let mut d = (osc - mean).abs() % 360.0;
            if d > 180.0 {
                d = 360.0 - d;
            }
            if d > 0.01 {
                any_nonzero = true;
            }
            max_diff = max_diff.max(d);
        }
        assert!(any_nonzero, "osculating apogee should differ from mean");
        // The osculating apogee swings by up to ~30° about the mean.
        assert!(
            max_diff > 1.0,
            "osculating swing should exceed 1°, got {max_diff}°"
        );
        assert!(
            max_diff < 40.0,
            "osculating swing should stay < 40°, got {max_diff}°"
        );
    }

    #[test]
    fn true_lilith_valid_range() {
        for offset in [0.0, 100.0, 365.25, 3652.5, -3652.5] {
            let lon = true_lilith(JdTT(2451545.0 + offset)).unwrap();
            assert!(
                lon >= 0.0 && lon < std::f64::consts::TAU,
                "True Lilith should be in [0, 2π), got {lon}"
            );
        }
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
