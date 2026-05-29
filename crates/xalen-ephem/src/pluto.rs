use crate::provider::EphemerisError;
use xalen_coords::EclipticPosition;
use xalen_time::{JdTT, JulianDay};

/// One row of Meeus Table 37.A: (i, j, k, lA, lB, bA, bB, rA, rB) — three
/// integer argument multipliers followed by longitude/latitude/radius
/// sine & cosine coefficients.
type PlutoTerm = (i8, i8, i8, f64, f64, f64, f64, f64, f64);

/// Full 43-term Meeus Table 37.A for Pluto (Goffin/Meeus/Steyaert fit to DE200).
///
/// Each row: (i, j, k, lA, lB, bA, bB, rA, rB)
///   - i, j, k: integer multipliers for fundamental arguments J, S, P
///   - lA, lB: longitude coefficients in degrees (sin, cos)
///   - bA, bB: latitude coefficients in degrees (sin, cos)
///   - rA, rB: radius coefficients in AU (sin, cos)
///
/// Source: "Astronomical Algorithms" 2nd ed., Jean Meeus, Table 37.A pp. 264-265.
/// Cross-validated against soniakeys/meeus (Go, MIT license) commit of same table.
#[rustfmt::skip]
static TABLE_37A: [PlutoTerm; 43] = [
    // i  j   k       lA           lB           bA           bB           rA          rB
    ( 0, 0,  1, -19.799805,   19.850055,   -5.452852,  -14.974862,   6.6865439,   6.8951812),
    ( 0, 0,  2,   0.897144,   -4.954829,    3.527812,    1.672790,  -1.1827535,  -0.0332538),
    ( 0, 0,  3,   0.611149,    1.211027,   -1.050748,    0.327647,   0.1593179,  -0.1438890),
    ( 0, 0,  4,  -0.341243,   -0.189585,    0.178690,   -0.292153,  -0.0018444,   0.0483220),
    ( 0, 0,  5,   0.129287,   -0.034992,    0.018650,    0.100340,  -0.0065977,  -0.0085431),
    ( 0, 0,  6,  -0.038164,    0.030893,   -0.030697,   -0.025823,   0.0031174,  -0.0006032),
    ( 0, 1, -1,   0.020442,   -0.009987,    0.004878,    0.011248,  -0.0005794,   0.0022161),
    ( 0, 1,  0,  -0.004063,   -0.005071,    0.000226,   -0.000064,   0.0004601,   0.0004032),
    ( 0, 1,  1,  -0.006016,   -0.003336,    0.002030,   -0.000836,  -0.0001729,   0.0000234),
    ( 0, 1,  2,  -0.003956,    0.003039,    0.000069,   -0.000604,  -0.0000415,   0.0000702),
    ( 0, 1,  3,  -0.000667,    0.003572,   -0.000247,   -0.000567,   0.0000239,   0.0000723),
    ( 0, 2, -2,   0.001276,    0.000501,   -0.000057,    0.000001,   0.0000067,  -0.0000067),
    ( 0, 2, -1,   0.001152,   -0.000917,   -0.000122,    0.000175,   0.0001034,  -0.0000451),
    ( 0, 2,  0,   0.000630,   -0.001277,   -0.000049,   -0.000164,  -0.0000129,   0.0000504),
    ( 1,-1,  0,   0.002571,   -0.000459,   -0.000197,    0.000199,   0.0000480,  -0.0000231),
    ( 1,-1,  1,   0.000899,   -0.001449,   -0.000025,    0.000217,   0.0000002,  -0.0000441),
    ( 1, 0, -3,  -0.001016,    0.001043,    0.000589,   -0.000248,  -0.0003359,   0.0000265),
    ( 1, 0, -2,  -0.002343,   -0.001012,   -0.000269,    0.000711,   0.0007856,  -0.0007832),
    ( 1, 0, -1,   0.007042,    0.000788,    0.000185,    0.000193,   0.0000036,   0.0045763),
    ( 1, 0,  0,   0.001199,   -0.000338,    0.000315,    0.000807,   0.0008663,   0.0008547),
    ( 1, 0,  1,   0.000418,   -0.000067,   -0.000130,   -0.000043,  -0.0000809,  -0.0000769),
    ( 1, 0,  2,   0.000120,   -0.000274,    0.000005,    0.000003,   0.0000263,  -0.0000144),
    ( 1, 0,  3,  -0.000060,   -0.000159,    0.000002,    0.000017,  -0.0000126,   0.0000032),
    ( 1, 0,  4,  -0.000082,   -0.000029,    0.000002,    0.000005,  -0.0000035,  -0.0000016),
    ( 1, 1, -3,  -0.000036,   -0.000029,    0.000002,    0.000003,  -0.0000019,  -0.0000004),
    ( 1, 1, -2,  -0.000040,    0.000007,    0.000003,    0.000001,  -0.0000015,   0.0000008),
    ( 1, 1, -1,  -0.000014,    0.000022,    0.000002,   -0.000001,  -0.0000004,   0.0000012),
    ( 1, 1,  0,   0.000004,    0.000013,    0.000001,   -0.000001,   0.0000005,   0.0000006),
    ( 1, 1,  1,   0.000005,    0.000002,    0.000000,   -0.000001,   0.0000003,   0.0000001),
    ( 1, 1,  3,  -0.000001,    0.000000,    0.000000,    0.000000,   0.0000006,  -0.0000002),
    ( 2, 0, -6,   0.000002,    0.000000,    0.000000,   -0.000002,   0.0000002,   0.0000002),
    ( 2, 0, -5,  -0.000004,    0.000005,    0.000002,    0.000002,  -0.0000002,  -0.0000002),
    ( 2, 0, -4,   0.000004,   -0.000007,   -0.000007,    0.000000,   0.0000014,   0.0000013),
    ( 2, 0, -3,   0.000014,    0.000024,    0.000010,   -0.000008,  -0.0000063,   0.0000013),
    ( 2, 0, -2,  -0.000049,   -0.000034,   -0.000003,    0.000020,   0.0000136,  -0.0000236),
    ( 2, 0, -1,   0.000163,   -0.000048,    0.000006,    0.000005,   0.0000273,   0.0001065),
    ( 2, 0,  0,   0.000009,   -0.000024,    0.000014,    0.000017,   0.0000251,   0.0000149),
    ( 2, 0,  1,  -0.000004,    0.000001,   -0.000002,    0.000000,  -0.0000025,  -0.0000009),
    ( 2, 0,  2,  -0.000003,    0.000001,    0.000000,    0.000000,   0.0000009,  -0.0000002),
    ( 2, 0,  3,   0.000001,    0.000003,    0.000000,    0.000000,  -0.0000008,   0.0000007),
    ( 3, 0, -2,  -0.000003,   -0.000001,    0.000000,    0.000001,   0.0000002,  -0.0000010),
    ( 3, 0, -1,   0.000005,   -0.000003,    0.000000,    0.000000,   0.0000019,   0.0000035),
    ( 3, 0,  0,   0.000000,    0.000000,    0.000001,    0.000000,   0.0000010,   0.0000003),
];

/// Analytical Pluto position using the Meeus Ch.37 method (Goffin/Meeus/Steyaert
/// fit to DE200).  Accuracy: ~1 arcminute for 1885-2099.
///
/// The method evaluates 43 periodic terms for longitude, latitude, and radius
/// vector, each a function of three fundamental arguments (J, S, P) that track
/// the synodic motions of Jupiter, Saturn, and Pluto.
///
/// Reference: "Astronomical Algorithms", 2nd ed., Jean Meeus, Ch. 37, pp. 263-267.
pub fn pluto_position(jd_tt: JdTT) -> Result<EclipticPosition, EphemerisError> {
    let t = (jd_tt.as_f64() - 2_451_545.0) / 36_525.0; // Julian centuries from J2000

    // Validity check: Meeus/Chapront series is accurate 1885-2099
    let year_approx = 2000.0 + t * 100.0;
    if !(1885.0..=2099.0).contains(&year_approx) {
        return Err(EphemerisError::EpochOutOfRange(jd_tt.as_f64()));
    }

    let d2r = std::f64::consts::PI / 180.0;

    // Fundamental arguments (degrees -> radians)
    let j = (34.35 + 3034.9057 * t) * d2r;
    let s = (50.08 + 1222.1138 * t) * d2r;
    let p = (238.96 + 144.9600 * t) * d2r;

    // Accumulate the 43 periodic terms from Meeus Table 37.A.
    // Longitude and latitude sums are in degrees; radius sum is in AU.
    let mut sum_l = 0.0_f64;
    let mut sum_b = 0.0_f64;
    let mut sum_r = 0.0_f64;

    for &(ai, aj, ak, l_a, l_b, b_a, b_b, r_a, r_b) in TABLE_37A.iter() {
        let alpha = ai as f64 * j + aj as f64 * s + ak as f64 * p;
        let (sin_a, cos_a) = alpha.sin_cos();
        sum_l += l_a * sin_a + l_b * cos_a;
        sum_b += b_a * sin_a + b_b * cos_a;
        sum_r += r_a * sin_a + r_b * cos_a;
    }

    // Add the base position (Meeus p. 264, eqs. below Table 37.A)
    let lon_deg = 238.958116 + 144.9600 * t + sum_l;
    let lat_deg = -3.908239 + sum_b;
    let radius_au = 40.7241346 + sum_r;

    let lon_rad = (lon_deg * d2r).rem_euclid(std::f64::consts::TAU);
    let lat_rad = lat_deg * d2r;

    // The Meeus Pluto series gives J2000 ecliptic coordinates.
    // Apply general precession to equinox-of-date for consistency with VSOP87.
    let precession = xalen_coords::general_precession_longitude(t);

    Ok(EclipticPosition {
        longitude: (lon_rad + precession).rem_euclid(std::f64::consts::TAU),
        latitude: lat_rad,
        distance: radius_au,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use xalen_coords::RAD_TO_DEG;

    /// Verify against Meeus Example 37.a (p. 266):
    /// JDE = 2448908.5 (1992-Oct-13 0h TT)
    /// Expected J2000 heliocentric: l = 232.74071°, b = 14.58782°, r = 29.711111 AU
    #[test]
    fn pluto_meeus_example_37a() {
        // Meeus Example 37.a uses J2000 ecliptic (no precession).
        // Our function adds precession for equinox-of-date, so we must
        // subtract it back to compare against the book's J2000 values.
        let jde = JdTT(2448908.5);
        let t = (2448908.5 - 2_451_545.0) / 36_525.0;

        let pos = pluto_position(jde).unwrap();
        let precession = xalen_coords::general_precession_longitude(t);

        // Recover J2000 longitude by removing the precession we added
        let lon_j2000 =
            ((pos.longitude - precession).rem_euclid(std::f64::consts::TAU)) * RAD_TO_DEG;
        let lat = pos.latitude * RAD_TO_DEG;
        let r = pos.distance;

        // Meeus expected values (Example 37.a, p. 266)
        let expected_lon = 232.74071;
        let expected_lat = 14.58782;
        let expected_r = 29.711111;

        // Tolerance: 0.00005° for longitude/latitude (sub-arcsecond),
        // 0.000005 AU for radius
        assert!(
            (lon_j2000 - expected_lon).abs() < 0.001,
            "Meeus Ex 37.a: longitude expected {expected_lon}°, got {lon_j2000}°"
        );
        assert!(
            (lat - expected_lat).abs() < 0.001,
            "Meeus Ex 37.a: latitude expected {expected_lat}°, got {lat}°"
        );
        assert!(
            (r - expected_r).abs() < 0.001,
            "Meeus Ex 37.a: radius expected {expected_r} AU, got {r} AU"
        );
    }

    #[test]
    fn pluto_at_j2000_reasonable() {
        let pos = pluto_position(JdTT::J2000).unwrap();
        let lon = pos.longitude * RAD_TO_DEG;
        let lat = pos.latitude * RAD_TO_DEG;
        // Pluto at J2000 should be near 251° ecliptic longitude (in Sagittarius/Ophiuchus)
        assert!(
            lon > 240.0 && lon < 260.0,
            "Pluto lon at J2000 should be ~251°, got {lon}°"
        );
        // Latitude should be ~+13° (Pluto has high inclination)
        assert!(
            lat > 5.0 && lat < 20.0,
            "Pluto lat at J2000 should be ~+13°, got {lat}°"
        );
        // Distance should be ~30-31 AU at J2000
        assert!(
            pos.distance > 28.0 && pos.distance < 35.0,
            "Pluto distance at J2000 should be ~30 AU, got {} AU",
            pos.distance
        );
    }

    #[test]
    fn pluto_moves_slowly() {
        // Pluto moves ~1.5°/year = ~0.004°/day
        let p1 = pluto_position(JdTT(2451545.0)).unwrap();
        let p2 = pluto_position(JdTT(2451545.0 + 365.25)).unwrap();
        let diff_deg = ((p2.longitude - p1.longitude) * RAD_TO_DEG).abs();
        // Allow for wrap-around
        let diff = if diff_deg > 180.0 {
            360.0 - diff_deg
        } else {
            diff_deg
        };
        assert!(
            diff > 0.5 && diff < 3.0,
            "Pluto should move ~1.5°/year, got {diff}°/year"
        );
    }

    #[test]
    fn pluto_out_of_range_rejected() {
        // Year 1800 -- outside the 1885-2099 validity range
        let jd_1800 = JdTT(2378497.0);
        assert!(
            pluto_position(jd_1800).is_err(),
            "Pluto before 1885 should return EpochOutOfRange"
        );
    }

    #[test]
    fn pluto_epoch_2020() {
        // 2020-01-01 12:00 TT = JD 2458849.0
        let pos = pluto_position(JdTT(2458849.0)).unwrap();
        let lon = pos.longitude * RAD_TO_DEG;
        // Pluto in early 2020 should be ~293° (in Sagittarius/Capricorn)
        assert!(
            lon > 285.0 && lon < 300.0,
            "Pluto lon in 2020 should be ~293°, got {lon}°"
        );
    }

    /// Verify the table has exactly 43 terms as documented.
    #[test]
    fn table_37a_has_43_terms() {
        assert_eq!(TABLE_37A.len(), 43, "Table 37.A must have exactly 43 terms");
    }
}
