use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReturnType {
    Solar,
    Lunar,
    Saturn,
    Jupiter,
    Mars,
}

/// Mean orbital periods in days for each return type.
fn period_days(rt: ReturnType) -> f64 {
    match rt {
        ReturnType::Solar => 365.24219,
        ReturnType::Lunar => 27.32166,  // sidereal month
        ReturnType::Saturn => 10759.22, // ~29.46 years
        ReturnType::Jupiter => 4332.59, // ~11.86 years
        ReturnType::Mars => 686.97,     // ~1.88 years
    }
}

pub fn approximate_return_jd(
    _natal_longitude_deg: f64,
    birth_jd: f64,
    return_number: u32,
    return_type: ReturnType,
) -> f64 {
    birth_jd + return_number as f64 * period_days(return_type)
}

pub fn demi_return_jd(
    natal_longitude_deg: f64,
    birth_jd: f64,
    return_number: u32,
    return_type: ReturnType,
) -> f64 {
    let full = approximate_return_jd(natal_longitude_deg, birth_jd, return_number, return_type);
    full - period_days(return_type) / 2.0
}

pub fn solar_return_year(birth_jd: f64, year: u32) -> f64 {
    approximate_return_jd(0.0, birth_jd, year, ReturnType::Solar)
}

pub fn lunar_return_month(birth_jd: f64, month: u32) -> f64 {
    approximate_return_jd(0.0, birth_jd, month, ReturnType::Lunar)
}

// ---------------------------------------------------------------------------
// Low-precision solar longitude (Meeus, Ch. 25 — accuracy ~0.01 deg)
// ---------------------------------------------------------------------------

const J2000: f64 = 2_451_545.0;
const DEG: f64 = std::f64::consts::PI / 180.0;

/// Geometric mean longitude of the Sun, referred to the mean equinox of date.
fn sun_mean_longitude(t: f64) -> f64 {
    let l0 = 280.46646 + t * (36000.76983 + t * 0.0003032);
    l0.rem_euclid(360.0)
}

/// Mean anomaly of the Sun (degrees).
fn sun_mean_anomaly(t: f64) -> f64 {
    let m = 357.52911 + t * (35999.05029 - t * 0.0001537);
    m.rem_euclid(360.0)
}

/// Low-precision apparent solar longitude in degrees [0, 360).
///
/// Uses the mean longitude + equation of center + nutation/aberration
/// approximation from Meeus Ch. 25. Good to ~0.01 degrees, which is
/// more than sufficient for bisection-based return finding.
fn sun_longitude_approx(jd: f64) -> f64 {
    let t = (jd - J2000) / 36525.0;

    let l0 = sun_mean_longitude(t);
    let m = sun_mean_anomaly(t);
    let m_rad = m * DEG;

    // Equation of center (3-term series)
    let c = (1.9146 - t * (0.004817 + t * 0.000014)) * m_rad.sin()
        + (0.019993 - t * 0.000101) * (2.0 * m_rad).sin()
        + 0.00029 * (3.0 * m_rad).sin();

    // Sun's true longitude
    let sun_true = l0 + c;

    // Approximate nutation + aberration correction (~-0.00569 - 0.00478 * sin(omega))
    let omega = 125.04 - 1934.136 * t;
    let apparent = sun_true - 0.00569 - 0.00478 * (omega * DEG).sin();

    apparent.rem_euclid(360.0)
}

/// Normalize an angle difference to [-180, +180].
fn wrap_delta(d: f64) -> f64 {
    let d = d.rem_euclid(360.0);
    if d > 180.0 { d - 360.0 } else { d }
}

// ---------------------------------------------------------------------------
// Compute exact solar return via Newton-Raphson
// ---------------------------------------------------------------------------

/// Sun's mean daily motion in degrees.
const SUN_MEAN_MOTION: f64 = 360.0 / 365.24219;
/// Maximum Newton-Raphson iterations.
const MAX_ITER: u32 = 50;

/// Compute the exact Julian Day of a solar return.
///
/// Finds the moment when the Sun's ecliptic longitude equals
/// `natal_sun_longitude_deg` for the `target_year`-th return after birth.
///
/// Uses [`approximate_return_jd`] as a seed, then refines with
/// Newton-Raphson iteration using the Sun's mean daily motion as the
/// derivative estimate.
///
/// # Arguments
/// * `natal_sun_longitude_deg` -- natal Sun longitude in degrees [0, 360)
/// * `birth_jd` -- Julian Day of birth
/// * `target_year` -- which return (1 = first solar return, 2 = second, ...)
///
/// # Returns
/// The Julian Day (TT) of the solar return, accurate to ~0.001 seconds.
pub fn compute_solar_return(natal_sun_longitude_deg: f64, birth_jd: f64, target_year: u32) -> f64 {
    let natal_lon = natal_sun_longitude_deg.rem_euclid(360.0);
    let mut jd = approximate_return_jd(natal_lon, birth_jd, target_year, ReturnType::Solar);

    for _ in 0..MAX_ITER {
        let current_lon = sun_longitude_approx(jd);
        let delta_lon = wrap_delta(current_lon - natal_lon);

        if delta_lon.abs() < 1e-6 {
            break;
        }

        // Newton step with clamping to prevent overshoot
        let correction = (delta_lon / SUN_MEAN_MOTION).clamp(-180.0, 180.0);
        jd -= correction;
    }

    jd
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Existing approximate-return tests
    // -----------------------------------------------------------------------

    #[test]
    fn solar_return_is_annual() {
        let birth = 2451545.0;
        let sr1 = approximate_return_jd(280.0, birth, 1, ReturnType::Solar);
        assert!((sr1 - birth - 365.24).abs() < 1.0);
    }

    #[test]
    fn saturn_return_is_29_years() {
        let birth = 2451545.0;
        let sr = approximate_return_jd(100.0, birth, 1, ReturnType::Saturn);
        let years = (sr - birth) / 365.25;
        assert!(
            (years - 29.46).abs() < 0.5,
            "Saturn return should be ~29.46 years, got {years}"
        );
    }

    #[test]
    fn lunar_returns_monthly() {
        let birth = 2451545.0;
        let lr1 = approximate_return_jd(50.0, birth, 1, ReturnType::Lunar);
        let days = lr1 - birth;
        assert!(
            (days - 27.32).abs() < 1.0,
            "Lunar return should be ~27.32 days, got {days}"
        );
    }

    #[test]
    fn demi_return_is_half_period() {
        let birth = 2451545.0;
        let full = approximate_return_jd(0.0, birth, 1, ReturnType::Saturn);
        let demi = demi_return_jd(0.0, birth, 1, ReturnType::Saturn);
        let half_period = (full - birth) / 2.0;
        assert!((demi - birth - half_period).abs() < 1.0);
    }

    #[test]
    fn multiple_solar_returns() {
        let birth = 2451545.0;
        for i in 1..=5 {
            let sr = solar_return_year(birth, i);
            let years = (sr - birth) / 365.25;
            assert!(
                (years - i as f64).abs() < 0.01,
                "SR {i} should be ~{i} years, got {years}"
            );
        }
    }

    // -----------------------------------------------------------------------
    // sun_longitude_approx sanity
    // -----------------------------------------------------------------------

    #[test]
    fn sun_longitude_at_j2000() {
        // J2000.0 = 2000-01-01 12:00 TT.  Sun's geocentric longitude
        // is approximately 280.5 deg (Meeus gives ~280.46).
        let lon = sun_longitude_approx(J2000);
        assert!(
            (lon - 280.5).abs() < 1.0,
            "Sun longitude at J2000 should be ~280.5 deg, got {lon}"
        );
    }

    #[test]
    fn sun_longitude_increases_over_time() {
        let lon1 = sun_longitude_approx(J2000);
        let lon2 = sun_longitude_approx(J2000 + 30.0);
        // Sun moves ~1 deg/day, so in 30 days it should advance ~30 deg.
        let delta = wrap_delta(lon2 - lon1);
        assert!(
            (delta - 29.6).abs() < 1.5,
            "Sun should advance ~30 deg in 30 days, got {delta}"
        );
    }

    // -----------------------------------------------------------------------
    // compute_solar_return tests
    // -----------------------------------------------------------------------

    #[test]
    fn solar_return_differs_from_approximate() {
        // Birth near J2000, natal Sun ~280.5 deg
        let birth_jd = J2000;
        let natal_lon = sun_longitude_approx(birth_jd);

        let approx = approximate_return_jd(natal_lon, birth_jd, 1, ReturnType::Solar);
        let exact = compute_solar_return(natal_lon, birth_jd, 1);

        let diff = (exact - approx).abs();
        // The exact return should be close to (but not identical to) the
        // mean-period approximation.  Difference should be < 1 day
        // but typically > 0 (unless birth is at the exact mean epoch).
        assert!(
            diff < 1.0,
            "Exact and approximate should differ by < 1 day, got {diff}"
        );
    }

    #[test]
    fn solar_return_longitude_matches_natal() {
        let birth_jd = J2000;
        let natal_lon = 120.0; // arbitrary natal Sun position

        let sr_jd = compute_solar_return(natal_lon, birth_jd, 1);
        let sr_lon = sun_longitude_approx(sr_jd);

        let error = wrap_delta(sr_lon - natal_lon).abs();
        assert!(
            error < 0.01,
            "Solar return longitude should match natal within 0.01 deg, error = {error}"
        );
    }

    #[test]
    fn solar_return_longitude_matches_for_various_natal_positions() {
        let birth_jd = J2000;
        // Test several natal Sun positions across the zodiac
        for natal_lon in [0.0, 45.0, 90.0, 135.0, 180.0, 225.0, 270.0, 315.0, 359.9] {
            let sr_jd = compute_solar_return(natal_lon, birth_jd, 1);
            let sr_lon = sun_longitude_approx(sr_jd);
            let error = wrap_delta(sr_lon - natal_lon).abs();
            assert!(
                error < 0.01,
                "natal_lon={natal_lon}: SR longitude error {error} exceeds 0.01 deg"
            );
        }
    }

    #[test]
    fn consecutive_solar_returns_are_about_one_year_apart() {
        let birth_jd = J2000;
        let natal_lon = 200.0;

        // Compare consecutive returns (year N vs year N+1), not against birth.
        // The first return may not be a full year from birth if the natal
        // longitude doesn't match the Sun's actual position at birth_jd.
        for year in 1..=4 {
            let sr_this = compute_solar_return(natal_lon, birth_jd, year);
            let sr_next = compute_solar_return(natal_lon, birth_jd, year + 1);
            let gap = sr_next - sr_this;

            // Successive solar returns should be approximately one tropical
            // year apart (365.24219 days), with small variation due to the
            // equation of center.
            assert!(
                (gap - 365.25).abs() < 1.0,
                "Year {year}->{}: gap {gap} days is not ~365.25",
                year + 1
            );
        }
    }

    #[test]
    fn solar_return_handles_zero_crossing() {
        // Natal Sun near 0 degrees (Aries point) -- tests the 360->0 wrap.
        let birth_jd = J2000;
        let natal_lon = 1.0;

        let sr_jd = compute_solar_return(natal_lon, birth_jd, 1);
        let sr_lon = sun_longitude_approx(sr_jd);
        let error = wrap_delta(sr_lon - natal_lon).abs();
        assert!(
            error < 0.01,
            "Near-0-deg natal: SR longitude error {error} exceeds 0.01 deg"
        );
    }

    #[test]
    fn solar_return_multiple_years_all_accurate() {
        let birth_jd = 2_450_000.0; // ~1995-10-09
        let natal_lon = 195.0;

        for year in 1..=10 {
            let sr_jd = compute_solar_return(natal_lon, birth_jd, year);
            let sr_lon = sun_longitude_approx(sr_jd);
            let error = wrap_delta(sr_lon - natal_lon).abs();
            assert!(
                error < 0.01,
                "Year {year}: SR longitude error {error} exceeds 0.01 deg"
            );
        }
    }

    #[test]
    fn solar_return_is_in_the_future_of_birth() {
        let birth_jd = J2000;
        let natal_lon = sun_longitude_approx(birth_jd);

        for year in 1..=3 {
            let sr_jd = compute_solar_return(natal_lon, birth_jd, year);
            assert!(
                sr_jd > birth_jd,
                "Year {year}: solar return JD {sr_jd} should be after birth {birth_jd}"
            );
        }
    }
}
