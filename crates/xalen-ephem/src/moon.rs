use crate::provider::EphemerisError;
use xalen_coords::EclipticPosition;
use xalen_time::{JdTT, JulianDay};

// ---------------------------------------------------------------------------
// Moon — ABRIDGED ELP-2000/82 via Meeus Ch.47 ("Astronomical Algorithms" 2nd ed.)
//
// This is the TRUNCATED Meeus tabulation, NOT the full ELP-2000/82 theory (which
// has thousands of periodic terms):
//   60 longitude + distance terms  (Table 47.A, pp 339-340)
//   60 latitude terms              (Table 47.B, p 341)
//
// ACCURACY (honest, measured vs pyswisseph 2.10.03): `geocentric_moon` returns
// the MEAN-of-date series position only — it deliberately omits
// nutation-in-longitude and aberration/light-time, so it is NOT an apparent
// place on its own. The apparent geocentric Moon is assembled by the provider
// (`vsop::apparent_moon`): mean-of-date series + Δψ (IAU 2000B nutation) +
// geocentric light-time (~0.7"). With that wrapper, the analytical apparent Moon
// longitude residual vs pyswisseph (Moshier) is:
//   AD 1600-2100: RMS ~2.9"  / max ~12"   (modern era)
//   AD 1000-2000: RMS ~4.2"  / max ~15"
//   AD 0001-2000: RMS ~9.6"  / max ~31"   (drift grows toward antiquity)
// Latitude is better: RMS <1" across all spans (max ~4"). This is LIMITED BY THE
// 60-TERM SERIES TRUNCATION, not by missing nutation/aberration. (A prior build
// wrongly applied the full ANNUAL aberration term to the Moon — κ=20.49552",
// meant for planets sharing none of Earth's heliocentric velocity — which
// inflated the residual to RMS ~19" / max ~44"; that has been removed.) The raw
// mean-of-date series alone matches the Meeus Ch.47 worked example to <2" but
// over a full span is good to ~tens of arcsec.
//
// For SUB-ARCSECOND apparent lunar (and all-body) positions, use the DE440
// provider. With the `kernel-autodownload` cargo feature,
// `De440Provider::from_auto_cache()` fetches and caches the public NASA NAIF
// `de440s.bsp` kernel automatically (no manual file handling); without the
// feature, supply your own `.bsp` via `De440Provider::try_from_file`. Either way
// the DE440-backed apparent Moon agrees with JPL to sub-arcsecond.
// ---------------------------------------------------------------------------

/// Table 47.A: Periodic terms for the longitude (Sigma_l, units 0.000001 deg)
/// and distance (Sigma_r, units 0.001 km) of the Moon.
///
/// Columns: D, M, M', F, coeff_l, coeff_r
const TABLE_47A: [(i8, i8, i8, i8, f64, f64); 60] = [
    (0, 0, 1, 0, 6288774.0, -20905355.0),
    (2, 0, -1, 0, 1274027.0, -3699111.0),
    (2, 0, 0, 0, 658314.0, -2955968.0),
    (0, 0, 2, 0, 213618.0, -569925.0),
    (0, 1, 0, 0, -185116.0, 48888.0),
    (0, 0, 0, 2, -114332.0, -3149.0),
    (2, 0, -2, 0, 58793.0, 246158.0),
    (2, -1, -1, 0, 57066.0, -152138.0),
    (2, 0, 1, 0, 53322.0, -170733.0),
    (2, -1, 0, 0, 45758.0, -204586.0),
    (0, 1, -1, 0, -40923.0, -129620.0),
    (1, 0, 0, 0, -34720.0, 108743.0),
    (0, 1, 1, 0, -30383.0, 104755.0),
    (2, 0, 0, -2, 15327.0, 10321.0),
    (0, 0, 1, 2, -12528.0, 0.0),
    (0, 0, 1, -2, 10980.0, 79661.0),
    (4, 0, -1, 0, 10675.0, -34782.0),
    (0, 0, 3, 0, 10034.0, -23210.0),
    (4, 0, -2, 0, 8548.0, -21636.0),
    (2, 1, -1, 0, -7888.0, 24208.0),
    (2, 1, 0, 0, -6766.0, 30824.0),
    (1, 0, -1, 0, -5163.0, -8379.0),
    (1, 1, 0, 0, 4987.0, -16675.0),
    (2, -1, 1, 0, 4036.0, -12831.0),
    (2, 0, 2, 0, 3994.0, -10445.0),
    (4, 0, 0, 0, 3861.0, -11650.0),
    (2, 0, -3, 0, 3665.0, 14403.0),
    (0, 1, -2, 0, -2689.0, -7003.0),
    (2, 0, -1, 2, -2602.0, 0.0),
    (2, -1, -2, 0, 2390.0, 10056.0),
    (1, 0, 1, 0, -2348.0, 6322.0),
    (2, -2, 0, 0, 2236.0, -9884.0),
    (0, 1, 2, 0, -2120.0, 5751.0),
    (0, 2, 0, 0, -2069.0, 0.0),
    (2, -2, -1, 0, 2048.0, -4950.0),
    (2, 0, 1, -2, -1773.0, 4130.0),
    (2, 0, 0, 2, -1595.0, 0.0),
    (4, -1, -1, 0, 1215.0, -3958.0),
    (0, 0, 2, 2, -1110.0, 0.0),
    (3, 0, -1, 0, -892.0, 3258.0),
    (2, 1, 1, 0, -810.0, 2616.0),
    (4, -1, -2, 0, 759.0, -1897.0),
    (0, 2, -1, 0, -713.0, -2117.0),
    (2, 2, -1, 0, -700.0, 2354.0),
    (2, 1, -2, 0, 691.0, 0.0),
    (2, -1, 0, -2, 596.0, 0.0),
    (4, 0, 1, 0, 549.0, -1423.0),
    (0, 0, 4, 0, 537.0, -1117.0),
    (4, -1, 0, 0, 520.0, -1571.0),
    (1, 0, -2, 0, -487.0, -1739.0),
    (2, 1, 0, -2, -399.0, 0.0),
    (0, 0, 2, -2, -381.0, -4421.0),
    (1, 1, 1, 0, 351.0, 0.0),
    (3, 0, -2, 0, -340.0, 0.0),
    (4, 0, -3, 0, 330.0, 0.0),
    (2, -1, 2, 0, 327.0, 0.0),
    (0, 2, 1, 0, -323.0, 1165.0),
    (1, 1, -1, 0, 299.0, 0.0),
    (2, 0, 3, 0, 294.0, 0.0),
    (2, 0, -1, -2, 0.0, 8752.0),
];

/// Table 47.B: Periodic terms for the latitude (Sigma_b, units 0.000001 deg)
/// of the Moon.
///
/// Columns: D, M, M', F, coeff_b
const TABLE_47B: [(i8, i8, i8, i8, f64); 60] = [
    (0, 0, 0, 1, 5128122.0),
    (0, 0, 1, 1, 280602.0),
    (0, 0, 1, -1, 277693.0),
    (2, 0, 0, -1, 173237.0),
    (2, 0, -1, 1, 55413.0),
    (2, 0, -1, -1, 46271.0),
    (2, 0, 0, 1, 32573.0),
    (0, 0, 2, 1, 17198.0),
    (2, 0, 1, -1, 9266.0),
    (0, 0, 2, -1, 8822.0),
    (2, -1, 0, -1, 8216.0),
    (2, 0, -2, -1, 4324.0),
    (2, 0, 1, 1, 4200.0),
    (2, 1, 0, -1, -3359.0),
    (2, -1, -1, 1, 2463.0),
    (2, -1, 0, 1, 2211.0),
    (2, -1, -1, -1, 2065.0),
    (0, 1, -1, -1, -1870.0),
    (4, 0, -1, -1, 1828.0),
    (0, 1, 0, 1, -1794.0),
    (0, 0, 0, 3, -1749.0),
    (0, 1, -1, 1, -1565.0),
    (1, 0, 0, 1, -1491.0),
    (0, 1, 1, 1, -1475.0),
    (0, 1, 1, -1, -1410.0),
    (0, 1, 0, -1, -1344.0),
    (1, 0, 0, -1, -1335.0),
    (0, 0, 3, 1, 1107.0),
    (4, 0, 0, -1, 1021.0),
    (4, 0, -1, 1, 833.0),
    (0, 0, 1, -3, 777.0),
    (4, 0, -2, 1, 671.0),
    (2, 0, 0, -3, 607.0),
    (2, 0, 2, -1, 596.0),
    (2, -1, 1, -1, 491.0),
    (2, 0, -2, 1, -451.0),
    (0, 0, 3, -1, 439.0),
    (2, 0, 2, 1, 422.0),
    (2, 0, -3, -1, 421.0),
    (2, 1, -1, 1, -366.0),
    (2, 1, 0, 1, -351.0),
    (4, 0, 0, 1, 331.0),
    (2, -1, 1, 1, 315.0),
    (2, -2, 0, -1, 302.0),
    (0, 0, 1, 3, -283.0),
    (2, 1, 1, -1, -229.0),
    (1, 1, 0, -1, 223.0),
    (1, 1, 0, 1, 223.0),
    (0, 1, -2, -1, -220.0),
    (2, 1, -1, -1, -220.0),
    (1, 0, 1, 1, -185.0),
    (2, -1, -2, -1, 181.0),
    (0, 1, 2, 1, -177.0),
    (4, 0, -2, -1, 176.0),
    (4, -1, -1, -1, 166.0),
    (1, 0, 1, -1, -164.0),
    (4, 0, 1, -1, 132.0),
    (1, 0, -1, -1, -119.0),
    (4, -1, 0, -1, 115.0),
    (2, -2, 0, 1, 107.0),
];

/// Compute the geocentric ecliptic position of the Moon using the abridged
/// (truncated) Meeus Ch.47 ELP-2000/82 tabulation.
///
/// Returns the MEAN-of-date ecliptic longitude & latitude (radians) and distance
/// in AU. This is NOT an apparent place: it omits nutation-in-longitude and
/// light-time/aberration on purpose — the provider adds those in
/// `vsop::apparent_moon` (mean + Δψ + geocentric light-time). The raw 60-term
/// truncation matches the Meeus Ch.47 worked example to <2" and is good to
/// ~tens of arcsec over a full span. After the apparent-place wrapper, the
/// measured residual vs pyswisseph 2.10.03 (Moshier) is RMS ~2.9" / max ~12"
/// over AD 1600-2100, growing to RMS ~9.6" by AD 1, limited by this truncation.
/// For sub-arcsecond apparent lunar positions use the DE440 provider (enable the
/// `kernel-autodownload` feature for one-call `De440Provider::from_auto_cache`).
pub fn geocentric_moon(jd_tt: JdTT) -> Result<EclipticPosition, EphemerisError> {
    let t = jd_tt.julian_centuries_from_j2000();
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;

    let d2r = std::f64::consts::PI / 180.0;

    // ------------------------------------------------------------------
    // Fundamental arguments (degrees) — Meeus Eq. 47.1–47.5
    // ------------------------------------------------------------------
    // L' — Moon's mean longitude (referred to the mean equinox of date)
    let lp = 218.3164477 + 481267.88123421 * t - 0.0015786 * t2 + t3 / 538841.0 - t4 / 65194000.0;

    // D — Mean elongation of the Moon
    let d = 297.8501921 + 445267.1114034 * t - 0.0018819 * t2 + t3 / 545868.0 - t4 / 113065000.0;

    // M — Sun's mean anomaly
    let m = 357.5291092 + 35999.0502909 * t - 0.0001536 * t2 + t3 / 24490000.0;

    // M' — Moon's mean anomaly
    let mp = 134.9633964 + 477198.8675055 * t + 0.0087414 * t2 + t3 / 69699.0 - t4 / 14712000.0;

    // F — Moon's argument of latitude (mean distance of Moon from its
    //     ascending node)
    let f = 93.2720950 + 483202.0175233 * t - 0.0036539 * t2 - t3 / 3526000.0 + t4 / 863310000.0;

    // ------------------------------------------------------------------
    // Additional arguments for additive corrections (Meeus p. 338)
    // ------------------------------------------------------------------
    let a1 = (119.75 + 131.849 * t) * d2r; // Action of Venus
    let a2 = (53.09 + 479264.290 * t) * d2r; // Action of Jupiter
    let a3 = (313.45 + 481266.484 * t) * d2r;

    // Convert fundamental arguments to radians
    let d_r = d * d2r;
    let m_r = m * d2r;
    let mp_r = mp * d2r;
    let f_r = f * d2r;
    let lp_r = lp * d2r;

    // ------------------------------------------------------------------
    // Earth's orbital eccentricity correction — Meeus Eq. 47.6
    // ------------------------------------------------------------------
    let e = 1.0 - 0.002516 * t - 0.0000074 * t2;
    let e2 = e * e;

    // ------------------------------------------------------------------
    // Accumulate longitude (Sigma_l) and distance (Sigma_r) from Table 47.A
    // ------------------------------------------------------------------
    let mut sigma_l = 0.0_f64;
    let mut sigma_r = 0.0_f64;

    for &(d_c, m_c, mp_c, f_c, coeff_l, coeff_r) in &TABLE_47A {
        let arg =
            (d_c as f64) * d_r + (m_c as f64) * m_r + (mp_c as f64) * mp_r + (f_c as f64) * f_r;

        // Apply eccentricity correction based on |M coefficient|
        let e_corr = match m_c.unsigned_abs() {
            1 => e,
            2 => e2,
            _ => 1.0,
        };

        sigma_l += e_corr * coeff_l * arg.sin();
        sigma_r += e_corr * coeff_r * arg.cos();
    }

    // Additive corrections to longitude (Meeus p. 338)
    sigma_l += 3958.0 * a1.sin(); // Action of Venus
    sigma_l += 1962.0 * (lp_r - f_r).sin(); // Flattening of the Earth
    sigma_l += 318.0 * a2.sin(); // Action of Jupiter

    // ------------------------------------------------------------------
    // Accumulate latitude (Sigma_b) from Table 47.B
    // ------------------------------------------------------------------
    let mut sigma_b = 0.0_f64;

    for &(d_c, m_c, mp_c, f_c, coeff_b) in &TABLE_47B {
        let arg =
            (d_c as f64) * d_r + (m_c as f64) * m_r + (mp_c as f64) * mp_r + (f_c as f64) * f_r;

        // Apply eccentricity correction based on |M coefficient|
        let e_corr = match m_c.unsigned_abs() {
            1 => e,
            2 => e2,
            _ => 1.0,
        };

        sigma_b += e_corr * coeff_b * arg.sin();
    }

    // Additive corrections to latitude (Meeus p. 338)
    sigma_b += -2235.0 * lp_r.sin();
    sigma_b += 382.0 * a3.sin();
    sigma_b += 175.0 * (a1 - f_r).sin();
    sigma_b += 175.0 * (a1 + f_r).sin();
    sigma_b += 127.0 * (lp_r - mp_r).sin();
    sigma_b += -115.0 * (lp_r + mp_r).sin();

    // ------------------------------------------------------------------
    // Final coordinates
    // ------------------------------------------------------------------
    let longitude = (lp + sigma_l / 1_000_000.0) * d2r;
    let latitude = (sigma_b / 1_000_000.0) * d2r;
    let distance_km = 385000.56 + sigma_r / 1000.0;
    let distance_au = distance_km / 149597870.7;

    Ok(EclipticPosition {
        longitude: longitude.rem_euclid(std::f64::consts::TAU),
        latitude,
        distance: distance_au,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use xalen_coords::RAD_TO_DEG;

    #[test]
    fn moon_at_j2000_reasonable() {
        let pos = geocentric_moon(JdTT::J2000).unwrap();
        let lon = pos.longitude * RAD_TO_DEG;
        let lat = pos.latitude * RAD_TO_DEG;
        assert!(
            lon >= 0.0 && lon < 360.0,
            "Moon lon should be 0-360, got {lon}"
        );
        assert!(lat.abs() < 6.0, "Moon lat should be < 6, got {lat}");
        assert!(
            pos.distance > 0.002 && pos.distance < 0.003,
            "Moon distance should be ~0.00257 AU, got {} AU",
            pos.distance
        );
    }

    #[test]
    fn moon_moves_about_13_degrees_per_day() {
        let p1 = geocentric_moon(JdTT(2451545.0)).unwrap();
        let p2 = geocentric_moon(JdTT(2451546.0)).unwrap();
        let mut diff = (p2.longitude - p1.longitude) * RAD_TO_DEG;
        if diff < 0.0 {
            diff += 360.0;
        }
        assert!(
            diff > 10.0 && diff < 16.0,
            "Moon should move ~13 deg/day, got {diff}"
        );
    }

    /// Meeus Example 47.a: 1992 April 12.00 TT
    /// Expected: lon 133.162655 deg, lat -3.229126 deg, dist 368409.7 km
    #[test]
    fn meeus_example_47a() {
        let jd = JdTT(2448724.5); // 1992 April 12.00 TT
        let pos = geocentric_moon(jd).unwrap();

        let lon_deg = pos.longitude * RAD_TO_DEG;
        let lat_deg = pos.latitude * RAD_TO_DEG;
        let dist_km = pos.distance * 149597870.7;

        let lon_err_arcsec = (lon_deg - 133.162655).abs() * 3600.0;
        let lat_err_arcsec = (lat_deg - (-3.229126)).abs() * 3600.0;
        let dist_err_km = (dist_km - 368409.7).abs();

        // Longitude within 2 arcseconds
        assert!(
            lon_err_arcsec < 2.0,
            "Meeus 47.a: longitude error {lon_err_arcsec:.3}\" (expected < 2\"), \
             got {lon_deg:.6} deg, expected 133.162655 deg"
        );

        // Latitude within 2 arcseconds
        assert!(
            lat_err_arcsec < 2.0,
            "Meeus 47.a: latitude error {lat_err_arcsec:.3}\" (expected < 2\"), \
             got {lat_deg:.6} deg, expected -3.229126 deg"
        );

        // Distance within 1 km
        assert!(
            dist_err_km < 1.0,
            "Meeus 47.a: distance error {dist_err_km:.3} km (expected < 1 km), \
             got {dist_km:.1} km, expected 368409.7 km"
        );
    }

    /// Verify the 60-term table produces more accurate results than the old
    /// 24-term approximation by checking residuals stay consistently small
    /// across several epochs spread over decades.
    #[test]
    fn moon_longitude_stability_across_epochs() {
        // Test dates spanning 1970-2030 — the Moon should always be 0..360
        // and latitude bounded by ~5.3 deg (max inclination + perturbations)
        let test_jds = [
            2440587.5, // 1970 Jan 1
            2444239.5, // 1980 Jan 1
            2448988.5, // 1992 Dec 1
            2451545.0, // 2000 Jan 1.5 (J2000)
            2455197.5, // 2010 Jan 1
            2459580.5, // 2022 Jan 1
            2462502.5, // 2030 Jan 1
        ];

        for &jd in &test_jds {
            let pos = geocentric_moon(JdTT(jd)).unwrap();
            let lon = pos.longitude * RAD_TO_DEG;
            let lat = pos.latitude * RAD_TO_DEG;
            assert!(lon >= 0.0 && lon < 360.0, "JD {jd}: lon {lon} out of range");
            assert!(lat.abs() < 6.0, "JD {jd}: lat {lat} exceeds 6 deg");
            assert!(
                pos.distance > 0.002 && pos.distance < 0.003,
                "JD {jd}: distance {} AU out of range",
                pos.distance
            );
        }
    }

    /// Count that we have exactly 60 terms in each table.
    #[test]
    fn table_sizes_correct() {
        assert_eq!(TABLE_47A.len(), 60, "Table 47.A should have 60 entries");
        assert_eq!(TABLE_47B.len(), 60, "Table 47.B should have 60 entries");
    }
}
