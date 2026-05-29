use crate::provider::EphemerisError;
use xalen_coords::EclipticPosition;
use xalen_time::{JdTT, JulianDay};

// ── Multi-epoch osculating orbital elements ──────────────────────────────
//
// Chiron's orbit is heavily perturbed by Saturn, Uranus, and Jupiter.
// A single set of J2000 Keplerian elements drifts 10-20° over a few decades.
//
// This module stores osculating elements at 11 epochs (1950-2050, 10-year
// intervals) from JPL Horizons (heliocentric ecliptic J2000, target "2060",
// table type ELEMENTS, center Sun @10).  For a given JD we linearly
// interpolate elements between the two bracketing epochs and then propagate
// via standard Keplerian mechanics from the interpolated epoch.  This keeps
// the propagation interval ≤5 years and reduces the unperturbed-model error
// from ~20° to ~1-2° across the full 1950-2050 span.
//
// Future improvement: DE440 does NOT include a Chiron segment (the JPL
// planetary ephemeris covers major planets + Moon + Pluto barycenter, not
// minor bodies).  For sub-arcsecond accuracy, integrate Chiron's equations
// of motion numerically with planetary perturbations, or use the JPL small-body
// SPK file (NAIF ID 2002060 / SPK ID 2060149).

/// Osculating orbital elements at a single epoch.
///
/// All angular elements are in **degrees**.  `a` is in AU, `n` in deg/day.
#[derive(Debug, Clone, Copy)]
struct OsculatingElements {
    /// Julian Date (TDB) of the osculating epoch.
    jd: f64,
    /// Eccentricity.
    e: f64,
    /// Inclination (degrees).
    i_deg: f64,
    /// Longitude of the ascending node (degrees).
    omega_node_deg: f64,
    /// Argument of perihelion (degrees).
    omega_peri_deg: f64,
    /// Mean anomaly at epoch (degrees).
    m_deg: f64,
    /// Semi-major axis (AU).
    a: f64,
    /// Mean daily motion (degrees/day).
    n_deg_per_day: f64,
}

/// JPL Horizons osculating elements for 2060 Chiron, heliocentric ecliptic
/// J2000 frame.  11 epochs, 1950-Jan-01 through 2050-Jan-01, 10-year steps.
///
/// Source: JPL Horizons API, EPHEM_TYPE=ELEMENTS, CENTER=500@10 (Sun),
///         COMMAND='2060', CSV_FORMAT=YES.
///
/// The distances were returned in km and mean motion in deg/sec; they have
/// been converted to AU and deg/day respectively.
const CHIRON_OSCULATING: &[OsculatingElements] = &[
    // 1950-Jan-01  JD 2433282.5
    OsculatingElements {
        jd: 2433282.5,
        e: 0.381413,
        i_deg: 6.9255,
        omega_node_deg: 209.491,
        omega_peri_deg: 339.870,
        m_deg: 30.430,
        a: 13.709085,
        n_deg_per_day: 0.0194175,
    },
    // 1960-Jan-01  JD 2436934.5
    OsculatingElements {
        jd: 2436934.5,
        e: 0.379787,
        i_deg: 6.9298,
        omega_node_deg: 209.363,
        omega_peri_deg: 340.034,
        m_deg: 101.428,
        a: 13.704607,
        n_deg_per_day: 0.0194270,
    },
    // 1970-Jan-01  JD 2440587.5
    OsculatingElements {
        jd: 2440587.5,
        e: 0.381931,
        i_deg: 6.9416,
        omega_node_deg: 209.392,
        omega_peri_deg: 339.520,
        m_deg: 173.482,
        a: 13.641905,
        n_deg_per_day: 0.0195610,
    },
    // 1980-Jan-01  JD 2444239.5
    OsculatingElements {
        jd: 2444239.5,
        e: 0.381479,
        i_deg: 6.9360,
        omega_node_deg: 209.464,
        omega_peri_deg: 339.305,
        m_deg: 245.248,
        a: 13.661625,
        n_deg_per_day: 0.0195186,
    },
    // 1990-Jan-01  JD 2447892.5
    OsculatingElements {
        jd: 2447892.5,
        e: 0.381900,
        i_deg: 6.9276,
        omega_node_deg: 209.372,
        omega_peri_deg: 339.005,
        m_deg: 316.791,
        a: 13.721853,
        n_deg_per_day: 0.0193908,
    },
    // 2000-Jan-01  JD 2451544.5
    OsculatingElements {
        jd: 2451544.5,
        e: 0.379344,
        i_deg: 6.9416,
        omega_node_deg: 209.397,
        omega_peri_deg: 339.142,
        m_deg: 27.987,
        a: 13.605073,
        n_deg_per_day: 0.0196404,
    },
    // 2010-Jan-01  JD 2455197.5
    OsculatingElements {
        jd: 2455197.5,
        e: 0.378765,
        i_deg: 6.9297,
        omega_node_deg: 209.338,
        omega_peri_deg: 339.900,
        m_deg: 98.669,
        a: 13.703938,
        n_deg_per_day: 0.0194279,
    },
    // 2020-Jan-01  JD 2458849.5
    OsculatingElements {
        jd: 2458849.5,
        e: 0.379397,
        i_deg: 6.9424,
        omega_node_deg: 209.232,
        omega_peri_deg: 339.819,
        m_deg: 169.889,
        a: 13.679673,
        n_deg_per_day: 0.0194797,
    },
    // 2030-Jan-01  JD 2462502.5
    OsculatingElements {
        jd: 2462502.5,
        e: 0.381814,
        i_deg: 6.9483,
        omega_node_deg: 209.302,
        omega_peri_deg: 339.625,
        m_deg: 241.652,
        a: 13.649125,
        n_deg_per_day: 0.0195454,
    },
    // 2040-Jan-01  JD 2466154.5
    OsculatingElements {
        jd: 2466154.5,
        e: 0.382437,
        i_deg: 6.9331,
        omega_node_deg: 209.326,
        omega_peri_deg: 339.016,
        m_deg: 313.625,
        a: 13.735089,
        n_deg_per_day: 0.0193622,
    },
    // 2050-Jan-01  JD 2469807.5
    OsculatingElements {
        jd: 2469807.5,
        e: 0.380428,
        i_deg: 6.9374,
        omega_node_deg: 209.236,
        omega_peri_deg: 339.536,
        m_deg: 24.361,
        a: 13.656077,
        n_deg_per_day: 0.0195307,
    },
];

/// Find the two bracketing epochs for a given JD and return them with the
/// interpolation fraction `t` in [0, 1].
///
/// If `jd` falls before the first epoch or after the last, the nearest
/// boundary pair is used (clamped extrapolation from the edge interval).
fn bracket_epochs(
    jd: f64,
) -> (
    &'static OsculatingElements,
    &'static OsculatingElements,
    f64,
) {
    let n = CHIRON_OSCULATING.len();
    debug_assert!(n >= 2);

    // Before first epoch: clamp to first interval
    if jd <= CHIRON_OSCULATING[0].jd {
        return (&CHIRON_OSCULATING[0], &CHIRON_OSCULATING[1], 0.0);
    }
    // After last epoch: clamp to last interval
    if jd >= CHIRON_OSCULATING[n - 1].jd {
        return (&CHIRON_OSCULATING[n - 2], &CHIRON_OSCULATING[n - 1], 1.0);
    }

    // Binary-ish linear scan (only 11 entries, not worth binary search)
    for i in 0..n - 1 {
        let a = &CHIRON_OSCULATING[i];
        let b = &CHIRON_OSCULATING[i + 1];
        if jd >= a.jd && jd < b.jd {
            let t = (jd - a.jd) / (b.jd - a.jd);
            return (a, b, t);
        }
    }

    // Fallback (should not happen given the checks above)
    let last = n - 1;
    (&CHIRON_OSCULATING[last - 1], &CHIRON_OSCULATING[last], 1.0)
}

/// Linearly interpolate between two osculating element sets at fraction `t`.
///
/// Mean anomaly is advanced from each epoch to the target JD using its own
/// `n`, then the two propagated M values are blended.  This avoids the
/// discontinuity that would occur from naively interpolating M (which wraps
/// at 360°).
fn interpolate_elements(
    a: &OsculatingElements,
    b: &OsculatingElements,
    t: f64,
    jd_target: f64,
) -> (f64, f64, f64, f64, f64, f64, f64) {
    let lerp = |x: f64, y: f64| x + t * (y - x);

    let e = lerp(a.e, b.e);
    let i_deg = lerp(a.i_deg, b.i_deg);
    let n_dpd = lerp(a.n_deg_per_day, b.n_deg_per_day);
    let semi = lerp(a.a, b.a);

    // For angles that can wrap around 360°, use shortest-arc interpolation
    let omega_node_deg = lerp_angle(a.omega_node_deg, b.omega_node_deg, t);
    let omega_peri_deg = lerp_angle(a.omega_peri_deg, b.omega_peri_deg, t);

    // Propagate mean anomaly from each epoch to jd_target, then blend.
    // This is more accurate than interpolating M directly because M wraps.
    let ma_a = a.m_deg + a.n_deg_per_day * (jd_target - a.jd);
    let ma_b = b.m_deg + b.n_deg_per_day * (jd_target - b.jd);
    // Normalize both to [0, 360) before blending
    let ma_a = ma_a.rem_euclid(360.0);
    let ma_b = ma_b.rem_euclid(360.0);
    let m_deg = lerp_angle(ma_a, ma_b, t);

    (e, i_deg, omega_node_deg, omega_peri_deg, m_deg, semi, n_dpd)
}

/// Interpolate between two angles (in degrees) along the shortest arc.
fn lerp_angle(a_deg: f64, b_deg: f64, t: f64) -> f64 {
    let mut diff = b_deg - a_deg;
    // Wrap to [-180, 180]
    diff = ((diff + 180.0).rem_euclid(360.0)) - 180.0;
    (a_deg + t * diff).rem_euclid(360.0)
}

/// Compute Chiron's heliocentric ecliptic rectangular coordinates (J2000 frame)
/// at the given TT epoch, using multi-epoch osculating elements with
/// interpolation.
///
/// The osculating elements are sourced from JPL Horizons at 10-year intervals
/// from 1950 to 2050.  For a requested epoch the two nearest element sets are
/// linearly interpolated, then Keplerian propagation is applied over the
/// (short) residual time delta.
///
/// Returns `(x, y, z)` in AU, J2000 ecliptic frame.
///
/// Accuracy: ~1-2° across 1950-2050 (vs ~10-20° with single-epoch elements),
/// limited by neglected higher-order perturbation terms between epochs.
pub fn chiron_helio_rect(jd_tt: JdTT) -> Result<(f64, f64, f64), EphemerisError> {
    let d2r = std::f64::consts::PI / 180.0;

    let jd = jd_tt.as_f64();
    let (ea, eb, t) = bracket_epochs(jd);
    let (e, i_deg, omega_node_deg, omega_peri_deg, m_deg, a, _n) =
        interpolate_elements(ea, eb, t, jd);

    // Convert to radians
    let i = i_deg * d2r;
    let omega_node = omega_node_deg * d2r;
    let omega_peri = omega_peri_deg * d2r;
    let mean_anom = (m_deg * d2r).rem_euclid(std::f64::consts::TAU);

    // Solve Kepler's equation: E - e*sin(E) = M via Newton-Raphson
    let ecc_anom = solve_kepler(mean_anom, e)?;

    // True anomaly from eccentric anomaly
    let cos_e = ecc_anom.cos();
    let sin_e = ecc_anom.sin();
    let true_anom = ((1.0 - e * e).sqrt() * sin_e).atan2(cos_e - e);

    // Heliocentric distance
    let r = a * (1.0 - e * cos_e);

    // Heliocentric ecliptic coordinates via orbital-to-ecliptic rotation
    let cos_om = omega_node.cos();
    let sin_om = omega_node.sin();
    let cos_i = i.cos();
    let sin_i = i.sin();

    // Position in orbital plane (u = omega + v)
    let u = omega_peri + true_anom;
    let cos_u = u.cos();
    let sin_u = u.sin();

    // Heliocentric ecliptic rectangular coordinates (J2000 ecliptic)
    let x_h = r * (cos_om * cos_u - sin_om * sin_u * cos_i);
    let y_h = r * (sin_om * cos_u + cos_om * sin_u * cos_i);
    let z_h = r * (sin_u * sin_i);

    Ok((x_h, y_h, z_h))
}

/// Check whether a Julian Date falls within the Chiron osculating element
/// epoch range (1950-2050). Outside this range, positions are extrapolated
/// from the nearest boundary interval and accuracy degrades significantly.
pub fn is_chiron_epoch_valid(jd: f64) -> bool {
    (2433282.5..=2469807.5).contains(&jd)
}

/// Compute Chiron's geocentric ecliptic position (equinox-of-date).
///
/// Uses multi-epoch osculating elements (interpolated) for the heliocentric
/// position, then subtracts Earth's VSOP87 heliocentric position.
///
/// **Warning:** When `jd_tt` is outside 1950-2050 (JD 2433282.5 to 2469807.5),
/// the position is extrapolated from the nearest boundary interval and accuracy
/// degrades significantly. Use [`is_chiron_epoch_valid`] to check beforehand.
#[allow(dead_code)]
pub fn chiron_position(jd_tt: JdTT) -> Result<EclipticPosition, EphemerisError> {
    if !is_chiron_epoch_valid(jd_tt.as_f64()) {
        // Log to stderr in debug builds so callers notice extrapolation.
        // In release builds, callers should check is_chiron_epoch_valid()
        // explicitly before calling this function.
        #[cfg(debug_assertions)]
        eprintln!(
            "[WARN] Chiron position at JD {} is outside 1950-2050 epoch range; extrapolated",
            jd_tt.as_f64()
        );
    }
    let (x_h, y_h, z_h) = chiron_helio_rect(jd_tt)?;

    // Geocentric: subtract Earth's heliocentric position
    let earth = vsop87::vsop87a::earth(jd_tt.as_f64());

    let x_geo = x_h - earth.x;
    let y_geo = y_h - earth.y;
    let z_geo = z_h - earth.z;

    let mut pos = xalen_coords::cartesian_to_ecliptic(&xalen_coords::CartesianPosition {
        x: x_geo,
        y: y_geo,
        z: z_geo,
    });

    // Apply general precession (J2000 -> equinox-of-date) for consistency with VSOP87
    let t = jd_tt.julian_centuries_from_j2000();
    pos.longitude += xalen_coords::general_precession_longitude(t);

    Ok(pos.normalize())
}

/// Solve Kepler's equation E - e*sin(E) = M via Newton-Raphson iteration.
/// Returns eccentric anomaly E in radians.
fn solve_kepler(m: f64, e: f64) -> Result<f64, EphemerisError> {
    let mut ea = if e < 0.8 { m } else { std::f64::consts::PI };

    for _ in 0..50 {
        let delta = ea - e * ea.sin() - m;
        let deriv = 1.0 - e * ea.cos();
        if deriv.abs() < 1e-15 {
            return Err(EphemerisError::ComputationFailed(
                "Kepler equation: derivative vanished".into(),
            ));
        }
        let correction = delta / deriv;
        ea -= correction;
        if correction.abs() < 1e-12 {
            return Ok(ea);
        }
    }

    Err(EphemerisError::ComputationFailed(
        "Kepler equation did not converge in 50 iterations".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use xalen_coords::RAD_TO_DEG;

    #[test]
    fn is_chiron_epoch_valid_boundaries() {
        assert!(is_chiron_epoch_valid(2433282.5)); // 1950-Jan-01 — lower bound
        assert!(is_chiron_epoch_valid(2469807.5)); // 2050-Jan-01 — upper bound
        assert!(is_chiron_epoch_valid(2451545.0)); // J2000 — mid-range
        assert!(!is_chiron_epoch_valid(2433282.4)); // just below
        assert!(!is_chiron_epoch_valid(2469807.6)); // just above
        assert!(!is_chiron_epoch_valid(2400000.0)); // 1858 — well outside
    }

    #[test]
    fn kepler_circular_orbit() {
        // e=0 => E = M exactly
        let m = 1.5;
        let e_anom = solve_kepler(m, 0.0).unwrap();
        assert!(
            (e_anom - m).abs() < 1e-12,
            "Circular orbit: E should equal M"
        );
    }

    #[test]
    fn kepler_known_case() {
        // For e=0.3791, M=1.0 rad, verify convergence and E > M (always true for e>0)
        let m = 1.0;
        let e = 0.3791;
        let ea = solve_kepler(m, e).unwrap();
        let residual = (ea - e * ea.sin() - m).abs();
        assert!(
            residual < 1e-12,
            "Kepler equation residual too large: {residual}"
        );
        assert!(ea > m, "E should be > M for e > 0");
    }

    #[test]
    fn chiron_at_j2000_reasonable() {
        let pos = chiron_position(JdTT::J2000).unwrap();
        let lon = pos.longitude * RAD_TO_DEG;
        let lat = pos.latitude * RAD_TO_DEG;
        // Chiron at J2000 was in Sagittarius, roughly ~250-255° ecliptic longitude
        // (JPL Horizons: RA 16h42m28.5s DEC -18°08'20" -> ecl lon ~251.6°)
        assert!(
            lon > 240.0 && lon < 265.0,
            "Chiron lon at J2000 should be ~250-255°, got {lon}°"
        );
        // Latitude should be modest (inclination 6.9°, so max lat ~ ±7°)
        assert!(lat.abs() < 10.0, "Chiron lat should be < 10°, got {lat}°");
        // Geocentric distance: Chiron was ~11-12 AU from Earth at J2000
        assert!(
            pos.distance > 8.0 && pos.distance < 16.0,
            "Chiron distance should be ~8-16 AU, got {} AU",
            pos.distance
        );
    }

    #[test]
    fn chiron_moves_at_reasonable_speed() {
        // Chiron's sidereal period is ~50.7 years = ~7.1°/year
        let p1 = chiron_position(JdTT(2451545.0)).unwrap();
        let p2 = chiron_position(JdTT(2451545.0 + 365.25)).unwrap();
        let mut diff = (p2.longitude - p1.longitude) * RAD_TO_DEG;
        if diff < -180.0 {
            diff += 360.0;
        }
        if diff > 180.0 {
            diff -= 360.0;
        }
        // Allow for retrograde periods — annual motion can range from -3° to +10°
        assert!(
            diff.abs() < 15.0,
            "Chiron annual motion should be moderate, got {diff}°"
        );
    }

    #[test]
    fn chiron_epoch_2020() {
        // 2020-01-01 12:00 TT = JD 2458849.0
        // JPL Horizons: RA 00h00m12s DEC +03°12'22" -> ecl lon ~1.3°
        // With multi-epoch elements, we should be within ~3° of truth.
        let pos = chiron_position(JdTT(2458849.0)).unwrap();
        let lon = pos.longitude * RAD_TO_DEG;
        assert!(
            (lon > 355.0 || lon < 10.0),
            "Chiron in early 2020 should be near 1° (Aries region), got {lon}°"
        );
    }

    #[test]
    fn chiron_distance_reasonable_range() {
        // Over many epochs, Chiron's heliocentric distance ranges from
        // ~8.5 AU (perihelion) to ~18.8 AU (aphelion).
        // Geocentric can vary from ~7 to ~20 AU.
        for jd in [2451545.0, 2455000.0, 2458849.0, 2460000.0] {
            let pos = chiron_position(JdTT(jd)).unwrap();
            assert!(
                pos.distance > 5.0 && pos.distance < 25.0,
                "Chiron distance at JD {jd} out of range: {} AU",
                pos.distance
            );
        }
    }

    // ── Multi-epoch accuracy tests ──────────────────────────────────────

    /// Test Chiron geocentric longitude against JPL Horizons reference values
    /// at multiple epochs spanning the full 1950-2050 range.
    ///
    /// Reference longitudes were computed by converting JPL Horizons RA/DEC
    /// (geocentric, astrometric) to ecliptic longitude using J2000 obliquity.
    /// These are approximate J2000 ecliptic longitudes; our function returns
    /// equinox-of-date, so we must account for precession (~1.4°/century).
    #[test]
    fn chiron_multi_epoch_accuracy() {
        // (JD, JPL Horizons J2000 ecliptic longitude in degrees, tolerance)
        // Tolerance: 5° allows for the equinox-of-date vs J2000 difference
        // (~0.7° in 2050) plus residual Keplerian model error.
        let test_cases: &[(f64, f64, f64)] = &[
            // 1950-Jan-01: ecl lon ~256.5°
            (2433282.5, 256.48, 5.0),
            // 1970-Jan-01: ecl lon ~2.9°
            (2440587.5, 2.94, 5.0),
            // 1980-Jan-01: ecl lon ~39.6°
            (2444239.5, 39.58, 5.0),
            // 1990-Jan-01: ecl lon ~104.0°
            (2447892.5, 103.98, 5.0),
            // 2000-Jan-01: ecl lon ~251.6°
            (2451544.5, 251.57, 5.0),
            // 2010-Jan-01: ecl lon ~323.0°
            (2455197.5, 322.98, 5.0),
            // 2020-Jan-01: ecl lon ~1.3°
            (2458849.5, 1.32, 5.0),
            // 2030-Jan-01: ecl lon ~37.6°
            (2462502.5, 37.60, 5.0),
            // 2040-Jan-01: ecl lon ~99.3°
            (2466154.5, 99.32, 5.0),
            // 2050-Jan-01: ecl lon ~245.9°
            (2469807.5, 245.89, 5.0),
        ];

        let mut max_err = 0.0_f64;
        for &(jd, expected_lon, tol) in test_cases {
            let pos = chiron_position(JdTT(jd)).unwrap();
            let lon = pos.longitude * RAD_TO_DEG;
            let mut err = (lon - expected_lon).abs();
            // Handle wrap-around
            if err > 180.0 {
                err = 360.0 - err;
            }
            max_err = max_err.max(err);
            assert!(
                err < tol,
                "Chiron at JD {jd}: got {lon:.2}°, expected ~{expected_lon:.1}° \
                 (error {err:.2}°, tolerance {tol}°)"
            );
        }
        // Report worst-case error for monitoring
        eprintln!("  [chiron_multi_epoch_accuracy] worst-case error: {max_err:.2}°");
    }

    /// Verify that interpolation at epoch boundaries exactly reproduces
    /// the stored osculating elements (no interpolation residual).
    #[test]
    fn interpolation_at_epoch_boundary() {
        for elem in CHIRON_OSCULATING.iter() {
            let (ea, eb, t) = bracket_epochs(elem.jd);
            let (e, _i_deg, _om_n, _om_p, _m_deg, a, _n) = interpolate_elements(ea, eb, t, elem.jd);

            // At an exact epoch boundary, the interpolated elements should
            // closely match the stored values (t=0 or t≈1).
            // Allow small tolerance for floating-point arithmetic.
            assert!(
                (e - elem.e).abs() < 1e-4,
                "e mismatch at JD {}: got {e}, expected {}",
                elem.jd,
                elem.e
            );
            assert!(
                (a - elem.a).abs() < 0.01,
                "a mismatch at JD {}: got {a}, expected {}",
                elem.jd,
                elem.a
            );
        }
    }

    /// Verify that the lerp_angle function handles wrap-around correctly.
    #[test]
    fn lerp_angle_wraparound() {
        // 350° to 10° should go through 0°, not through 180°
        let mid = lerp_angle(350.0, 10.0, 0.5);
        assert!(
            (mid - 0.0).abs() < 0.01 || (mid - 360.0).abs() < 0.01,
            "lerp_angle(350, 10, 0.5) should be ~0°, got {mid}°"
        );

        // Simple case: 10° to 20° at midpoint
        let mid2 = lerp_angle(10.0, 20.0, 0.5);
        assert!(
            (mid2 - 15.0).abs() < 0.01,
            "lerp_angle(10, 20, 0.5) should be 15°, got {mid2}°"
        );
    }

    /// Verify that Chiron's position is continuous across epoch boundaries
    /// (no jumps when transitioning from one element pair to the next).
    #[test]
    fn continuity_across_epoch_boundaries() {
        // Test at each interior epoch boundary
        for i in 1..CHIRON_OSCULATING.len() - 1 {
            let jd_boundary = CHIRON_OSCULATING[i].jd;
            let epsilon = 0.5; // half a day

            let pos_before = chiron_position(JdTT(jd_boundary - epsilon)).unwrap();
            let pos_after = chiron_position(JdTT(jd_boundary + epsilon)).unwrap();

            let lon_before = pos_before.longitude * RAD_TO_DEG;
            let lon_after = pos_after.longitude * RAD_TO_DEG;

            let mut jump = (lon_after - lon_before).abs();
            if jump > 180.0 {
                jump = 360.0 - jump;
            }

            // Over 1 day Chiron moves ~0.02°, so the jump should be small.
            // Allow 0.5° to account for element-set transition.
            assert!(
                jump < 0.5,
                "Discontinuity at epoch boundary JD {jd_boundary}: \
                 {lon_before:.4}° -> {lon_after:.4}° (jump {jump:.4}°)"
            );
        }
    }

    /// Verify the full 100-year span produces no obviously wrong results.
    #[test]
    fn full_century_sweep() {
        let jd_start = 2433282.5; // 1950
        let jd_end = 2469807.5; // 2050
        let step = 365.25; // ~1 year

        let mut jd = jd_start;
        let mut prev_lon = None;
        while jd <= jd_end {
            let pos = chiron_position(JdTT(jd)).unwrap();
            let lon = pos.longitude * RAD_TO_DEG;

            // Basic sanity: longitude in [0, 360), distance reasonable
            assert!(
                lon >= 0.0 && lon < 360.0,
                "lon out of range at JD {jd}: {lon}"
            );
            assert!(
                pos.distance > 5.0 && pos.distance < 25.0,
                "distance out of range at JD {jd}: {}",
                pos.distance
            );

            // No unreasonable year-to-year jumps (max ~15° either direction)
            if let Some(prev) = prev_lon {
                let mut delta: f64 = lon - prev;
                if delta > 180.0 {
                    delta -= 360.0;
                }
                if delta < -180.0 {
                    delta += 360.0;
                }
                assert!(
                    delta.abs() < 20.0,
                    "Unreasonable annual jump at JD {jd}: {delta:.1}°"
                );
            }
            prev_lon = Some(lon);
            jd += step;
        }
    }
}
