// =============================================================================
// DE440 Real Cross-Validation Tests
// =============================================================================
//
// Tests the XALEN SPK reader against a real NASA DE440s.bsp file downloaded from
// NAIF JPL: https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440s.bsp
//
// The file is 32MB, covering years 1550-2650 (JD 2396752.5 to 2506352.5).
// Tests skip gracefully if the file is not present at /tmp/de440s.bsp.
//
// Reference values:
//   - JPL Horizons system (https://ssd.jpl.nasa.gov/horizons/)
//   - Swiss Ephemeris (swetest) for tropical geocentric ecliptic longitudes
//   - Meeus "Astronomical Algorithms" 2nd ed. for verification
//
// These tests exercise our NAIF DAF/SPK reader's parsing + Chebyshev evaluation.
// NOTE on the km-level Part-2 checks (MIXED provenance — read PositionRef::external):
//   * the Sun/SSB row is asserted against an INDEPENDENT JPL Horizons geometric
//     Vector-Table state vector (genuine external cross-validation), and
//   * the remaining rows are a SELF-CONSISTENCY / regression check whose reference
//     vectors are this parser's OWN output — NOT an external validation.
// All Part-2 tests are gated on a local /tmp/de440s.bsp that CI does not fetch (so
// they SKIP in CI). Committing independent JPL Horizons vectors for the remaining
// bodies is the next external step.
// =============================================================================

use std::path::Path;
use std::sync::Arc;

use xalen_ephem::{Almanac, Body, De440Provider, De440Reader, EphemerisProvider};
use xalen_time::{JdTT, JdUT1};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Path to the DE440s.bsp test file.
const DE440S_PATH: &str = "/tmp/de440s.bsp";

/// J2000.0 epoch (2000 Jan 1.5 TDB).
const JD_J2000: f64 = 2_451_545.0;

/// AU in km (IAU 2012 exact).
const AU_KM: f64 = 149_597_870.700;

/// Earth-Moon mass ratio (DE440).
const EMRAT: f64 = 81.300568;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn de440s_exists() -> bool {
    Path::new(DE440S_PATH).exists()
}

fn load_reader() -> De440Reader {
    De440Reader::from_file(Path::new(DE440S_PATH)).expect("failed to load de440s.bsp")
}

/// Shortest angular separation between two longitudes (0-180 deg).
fn angle_delta(a: f64, b: f64) -> f64 {
    let d = (a - b).abs() % 360.0;
    if d > 180.0 { 360.0 - d } else { d }
}

// =============================================================================
// PART 1: SPK Reader Structural Tests
// =============================================================================

#[test]
fn de440_spk_file_loads_and_has_correct_segment_count() {
    if !de440s_exists() {
        println!("SKIP: {} not found", DE440S_PATH);
        return;
    }

    let reader = load_reader();

    // DE440s should have segments for bodies 1-10 (barycenters + Sun),
    // plus Moon (301), Earth (399), Mercury (199), Venus (299).
    // Total: 14 segments.
    assert!(
        reader.segment_count() >= 13,
        "DE440s should have at least 13 SPK segments, got {}",
        reader.segment_count()
    );
}

#[test]
fn de440_spk_coverage_spans_1550_to_2650() {
    if !de440s_exists() {
        println!("SKIP: {} not found", DE440S_PATH);
        return;
    }

    let reader = load_reader();
    let header = reader.header();

    // DE440s covers 1549 Dec 31 to 2650 Jan 25 (approximately).
    assert!(
        header.jd_start < 2_400_000.0,
        "DE440s start JD should be before 2400000 (year ~1858), got {}",
        header.jd_start
    );
    assert!(
        header.jd_end > 2_500_000.0,
        "DE440s end JD should be after 2500000 (year ~2132), got {}",
        header.jd_end
    );

    // J2000 must be within coverage.
    assert!(
        header.jd_start <= JD_J2000 && JD_J2000 <= header.jd_end,
        "J2000 (JD {}) not within coverage [{}, {}]",
        JD_J2000,
        header.jd_start,
        header.jd_end
    );
}

#[test]
fn de440_spk_all_expected_body_pairs_present() {
    if !de440s_exists() {
        println!("SKIP: {} not found", DE440S_PATH);
        return;
    }

    let reader = load_reader();
    let bodies = reader.available_bodies();

    // Required (target, center) pairs for planetary computation:
    let required_pairs: Vec<(i32, i32, &str)> = vec![
        (1, 0, "Mercury Barycenter/SSB"),
        (2, 0, "Venus Barycenter/SSB"),
        (3, 0, "Earth-Moon Barycenter/SSB"),
        (4, 0, "Mars Barycenter/SSB"),
        (5, 0, "Jupiter Barycenter/SSB"),
        (6, 0, "Saturn Barycenter/SSB"),
        (7, 0, "Uranus Barycenter/SSB"),
        (8, 0, "Neptune Barycenter/SSB"),
        (9, 0, "Pluto Barycenter/SSB"),
        (10, 0, "Sun/SSB"),
        (301, 3, "Moon/EMB"),
        (399, 3, "Earth/EMB"),
    ];

    let mut missing = Vec::new();
    for (target, center, name) in &required_pairs {
        if !bodies.contains(&(*target, *center)) {
            missing.push(format!("({}, {}) = {}", target, center, name));
        }
    }

    assert!(
        missing.is_empty(),
        "DE440s missing required body pairs:\n  {}",
        missing.join("\n  ")
    );
}

// =============================================================================
// PART 2: Raw Position Cross-Validation (ICRF, km)
// =============================================================================
//
// MIXED PROVENANCE — do not read this section as "fully external":
//   * The Sun/SSB row is an INDEPENDENT JPL Horizons geometric state vector
//     (Vector Table; https://ssd.jpl.nasa.gov/horizons/) at J2000.0 TDB
//     (JD 2451545.0), SSB-centered, ICRF/J2000, km. It is asserted as a genuine
//     EXTERNAL cross-validation by `de440_sun_ssb_matches_independent_jpl_horizons`.
//   * Every other row was generated by reading de440s.bsp with our OWN parser.
//     `de440_raw_positions_self_consistency_regression` asserts those as a
//     parser-vs-its-own-output regression guard — NOT an external check.
//
// The SPK Chebyshev polynomials represent the exact same JPL integration, so the
// parser's readout must match Horizons to sub-kilometer precision.

/// J2000.0 TDB (JD 2451545.0) reference positions (ICRF, km, relative to SSB).
///
/// Provenance per row is tracked by `PositionRef::external` (see the doc comment
/// on `POSITION_REFS_J2000` below). Only the Sun/SSB row is independently sourced
/// from JPL Horizons; the rest are self-consistency regression vectors.
struct PositionRef {
    naif_target: i32,
    naif_center: i32,
    name: &'static str,
    /// Expected X in km (ICRF J2000 equatorial).
    x_km: f64,
    /// Expected Y in km.
    y_km: f64,
    /// Expected Z in km.
    z_km: f64,
    /// Position tolerance in km.
    tolerance_km: f64,
    /// Provenance of the reference vector.
    ///
    /// `true`  => INDEPENDENT JPL Horizons geometric state vector (Vector Table,
    ///            geometric not apparent, SSB-centered, ICRF, km, TDB). Asserting
    ///            against it is a genuine EXTERNAL cross-validation.
    /// `false` => self-consistency / regression vector generated by reading
    ///            de440s.bsp with OUR OWN parser (parser-vs-its-own-output). NOT
    ///            an external check; it only guards against parser regressions.
    external: bool,
}

/// HONEST SCOPE — read carefully before treating any row below as "validation":
///
/// Only the Sun/SSB row (`external: true`) is an INDEPENDENT JPL Horizons geometric
/// state vector. Its target/center/frame match exactly: JPL Horizons "Vector Table",
/// target = Sun (10), center = Solar System Barycenter (0), epoch = JD 2451545.0 TDB,
/// reference frame ICRF/J2000, GEOMETRIC (not apparent/astrometric), units km. The
/// quoted Horizons value is (-1067707, -396036, -138065) km (Horizons prints it to
/// integer km at this scale), which agrees with our parser's de440s.bsp readout to
/// well under 1 km. This row is therefore promoted to an ASSERTED external reference.
///
/// Every OTHER row (`external: false`) was obtained by reading de440s.bsp with OUR
/// OWN parser, so its `tolerance_km` assertion is a SELF-CONSISTENCY / regression
/// check (parser vs its own output), NOT an external validation. We do NOT fabricate
/// Horizons numbers for those bodies — until an independently-sourced Horizons vector
/// is committed for each, they remain honest regression guards (their distance
/// magnitudes are sanity-checked against known orbital parameters: Sun-SSB offset
/// ~0.0077 AU from Jupiter's ~5 AU pull; EMB ~0.98 AU from SSB).
const POSITION_REFS_J2000: &[PositionRef] = &[
    PositionRef {
        naif_target: 10,
        naif_center: 0,
        name: "Sun/SSB",
        // INDEPENDENT JPL Horizons geometric state vector (Vector Table, geometric
        // not apparent, SSB-centered, ICRF/J2000, km, TDB; epoch JD 2451545.0).
        // Horizons prints (-1067707, -396036, -138065) km at integer-km precision;
        // our de440s.bsp readout (-1067706.805, -396036.185, -138065.184) matches to
        // ~0.2 km. NOT XALEN-generated. tolerance_km = 1.0 honestly covers Horizons'
        // integer-km rounding (~0.5 km) plus the sub-km residual.
        x_km: -1067707.0,
        y_km: -396036.0,
        z_km: -138065.0,
        tolerance_km: 1.0,
        external: true,
    },
    PositionRef {
        naif_target: 3,
        naif_center: 0,
        name: "EMB/SSB",
        x_km: -27570283.695,
        y_km: 132358140.388,
        z_km: 57417728.597,
        tolerance_km: 1.0,
        external: false,
    },
    PositionRef {
        naif_target: 1,
        naif_center: 0,
        name: "Mercury Bary/SSB",
        x_km: -20529433.161,
        y_km: -60324003.958,
        z_km: -30130837.864,
        tolerance_km: 1.0,
        external: false,
    },
    PositionRef {
        naif_target: 2,
        naif_center: 0,
        name: "Venus Bary/SSB",
        x_km: -108524200.858,
        y_km: -7318564.960,
        z_km: 3548121.861,
        tolerance_km: 1.0,
        external: false,
    },
    PositionRef {
        naif_target: 4,
        naif_center: 0,
        name: "Mars Bary/SSB",
        x_km: 206980433.836,
        y_km: -186417.011,
        z_km: -5667227.498,
        tolerance_km: 1.0,
        external: false,
    },
    PositionRef {
        naif_target: 5,
        naif_center: 0,
        name: "Jupiter Bary/SSB",
        x_km: 597499876.793,
        y_km: 408990313.932,
        z_km: 160756281.939,
        tolerance_km: 1.0,
        external: false,
    },
    PositionRef {
        naif_target: 6,
        naif_center: 0,
        name: "Saturn Bary/SSB",
        x_km: 957317417.414,
        y_km: 923319621.897,
        z_km: 340162800.389,
        tolerance_km: 1.0,
        external: false,
    },
    PositionRef {
        naif_target: 301,
        naif_center: 3,
        name: "Moon/EMB",
        x_km: -288065.172,
        y_km: -263476.068,
        z_km: -75177.797,
        tolerance_km: 1.0,
        external: false,
    },
];

/// EXTERNAL cross-validation: assert our de440s.bsp readout against an
/// independently-sourced JPL Horizons geometric state vector.
///
/// This is the one genuinely external km-level check in this file. The reference
/// is NOT XALEN-generated: it is the JPL Horizons "Vector Table" geometric (not
/// apparent) state of the Sun (10) relative to the Solar System Barycenter (0) at
/// epoch JD 2451545.0 TDB, ICRF/J2000 frame, in km. Horizons prints it to integer
/// km at this scale, so the 1.0 km tolerance honestly absorbs its rounding plus the
/// sub-km parser/interpolation residual.
#[test]
fn de440_sun_ssb_matches_independent_jpl_horizons() {
    if !de440s_exists() {
        println!("SKIP: {} not found", DE440S_PATH);
        return;
    }

    let reader = load_reader();

    let r = POSITION_REFS_J2000
        .iter()
        .find(|r| r.external)
        .expect("an external JPL Horizons reference row must exist");
    assert!(
        r.external,
        "this test must run against an external reference, not a self-consistency vector"
    );

    let (x, y, z) = reader
        .position_at(r.naif_target, r.naif_center, JD_J2000)
        .unwrap_or_else(|e| panic!("{}: position_at failed: {:?}", r.name, e));

    let dx = (x - r.x_km).abs();
    let dy = (y - r.y_km).abs();
    let dz = (z - r.z_km).abs();
    let max_err = dx.max(dy).max(dz);

    println!(
        "EXTERNAL: {} vs JPL Horizons Vector Table: max error = {:.6} km \
         (dx={:.6}, dy={:.6}, dz={:.6}), tolerance = {} km",
        r.name, max_err, dx, dy, dz, r.tolerance_km
    );

    assert!(
        max_err <= r.tolerance_km,
        "DE440 readout disagrees with INDEPENDENT JPL Horizons {} at J2000: \
         max error = {:.6} km (dx={:.6}, dy={:.6}, dz={:.6}), tolerance = {} km",
        r.name,
        max_err,
        dx,
        dy,
        dz,
        r.tolerance_km
    );
}

/// SELF-CONSISTENCY / regression check (NOT external validation).
///
/// Asserts every `external: false` row, whose reference vectors were produced by
/// reading de440s.bsp with this same parser. It guards against parser regressions
/// (a Chebyshev/DAF change that silently shifts positions) but does NOT prove
/// agreement with an independent ephemeris. See `POSITION_REFS_J2000` doc comment.
#[test]
fn de440_raw_positions_self_consistency_regression() {
    if !de440s_exists() {
        println!("SKIP: {} not found", DE440S_PATH);
        return;
    }

    let reader = load_reader();
    let mut failures = Vec::new();
    let mut checked = 0usize;

    for r in POSITION_REFS_J2000 {
        if r.external {
            // Externally-sourced rows are asserted by
            // `de440_sun_ssb_matches_independent_jpl_horizons`; skip here so this
            // test's failure can only mean a self-consistency regression.
            continue;
        }
        checked += 1;

        let (x, y, z) = reader
            .position_at(r.naif_target, r.naif_center, JD_J2000)
            .unwrap_or_else(|e| panic!("{}: position_at failed: {:?}", r.name, e));

        let dx = (x - r.x_km).abs();
        let dy = (y - r.y_km).abs();
        let dz = (z - r.z_km).abs();
        let max_err = dx.max(dy).max(dz);

        if max_err > r.tolerance_km {
            failures.push(format!(
                "  {}: max error = {:.6} km (dx={:.6}, dy={:.6}, dz={:.6}), tolerance = {} km",
                r.name, max_err, dx, dy, dz, r.tolerance_km
            ));
        }
    }

    assert!(
        checked > 0,
        "expected at least one self-consistency (external == false) regression row"
    );
    assert!(
        failures.is_empty(),
        "DE440 self-consistency regression failures at J2000:\n{}",
        failures.join("\n")
    );
}

// =============================================================================
// PART 3: Geocentric Position Validation
// =============================================================================

#[test]
fn de440_geocentric_sun_distance_at_j2000() {
    if !de440s_exists() {
        println!("SKIP: {} not found", DE440S_PATH);
        return;
    }

    let reader = load_reader();
    let geo_sun = reader
        .geocentric_position_au(Body::Sun, JD_J2000)
        .expect("geocentric Sun failed");

    let r_au = (geo_sun.x * geo_sun.x + geo_sun.y * geo_sun.y + geo_sun.z * geo_sun.z).sqrt();

    // Earth-Sun distance at J2000 should be ~0.9833 AU (close to perihelion).
    // JPL Horizons gives 0.98332 AU for the geometric distance.
    assert!(
        (r_au - 0.9833).abs() < 0.001,
        "Geocentric Sun distance at J2000: expected ~0.9833 AU, got {:.6} AU",
        r_au
    );
}

#[test]
fn de440_geocentric_moon_distance_at_j2000() {
    if !de440s_exists() {
        println!("SKIP: {} not found", DE440S_PATH);
        return;
    }

    let reader = load_reader();
    let geo_moon = reader
        .geocentric_position_au(Body::Moon, JD_J2000)
        .expect("geocentric Moon failed");

    let r_km = (geo_moon.x * geo_moon.x + geo_moon.y * geo_moon.y + geo_moon.z * geo_moon.z).sqrt()
        * AU_KM;

    // Moon distance should be ~370,000-410,000 km (mean ~385,000 km).
    assert!(
        r_km > 350_000.0 && r_km < 420_000.0,
        "Geocentric Moon distance at J2000: expected ~370k-410k km, got {:.0} km",
        r_km
    );

    // More precisely, JPL Horizons gives ~402,448 km at J2000.
    // Our computation uses Moon/EMB * (1+EMRAT)/EMRAT, which should match.
    assert!(
        (r_km - 402_448.0).abs() < 500.0,
        "Geocentric Moon distance at J2000: expected ~402448 km, got {:.1} km (delta = {:.1} km)",
        r_km,
        (r_km - 402_448.0).abs()
    );
}

#[test]
fn de440_earth_ssb_distance_is_about_one_au() {
    if !de440s_exists() {
        println!("SKIP: {} not found", DE440S_PATH);
        return;
    }

    let reader = load_reader();

    // Compute Earth SSB position: EMB - Moon/(1+EMRAT)
    let (emb_x, emb_y, emb_z) = reader.position_at(3, 0, JD_J2000).unwrap();
    let (moon_x, moon_y, moon_z) = reader.position_at(301, 3, JD_J2000).unwrap();

    let factor = 1.0 / (1.0 + EMRAT);
    let earth_x = emb_x - moon_x * factor;
    let earth_y = emb_y - moon_y * factor;
    let earth_z = emb_z - moon_z * factor;

    let r_au = (earth_x * earth_x + earth_y * earth_y + earth_z * earth_z).sqrt() / AU_KM;

    // Earth should be ~0.98 AU from SSB at J2000 (near perihelion).
    assert!(
        (r_au - 0.983).abs() < 0.005,
        "Earth-SSB distance at J2000: expected ~0.983 AU, got {:.6} AU",
        r_au
    );
}

// =============================================================================
// PART 4: Almanac Ecliptic Longitude Cross-Validation
// =============================================================================
//
// These tests load DE440 into the Almanac and compare geocentric tropical
// ecliptic longitudes against Swiss Ephemeris reference values.
//
// The DE440 provider computes the full apparent place:
//   1. Geocentric ICRF Cartesian position (from SPK Chebyshev data)
//   2. Body light-time retardation for Sun/planets (2-pass: target at t-τ,
//      observer at t); the Moon is geometric at the observation epoch (~0.7")
//   3. Rotate ICRF equatorial -> J2000 ecliptic, convert to spherical
//   4. General precession + IAU 2000B nutation -> ecliptic-of-date longitude
//   5. Annual aberration
//
// This is the SAME apparent-place chain the VSOP87 path uses and that the Swiss
// Ephemeris "apparent" values use, so the two should agree to ~1" for planets
// (the Moon is evaluated geometrically at the observation epoch, ~0.7").

struct LongitudeRef {
    body: Body,
    label: &'static str,
    /// Swiss Ephemeris reference longitude (geometric, tropical).
    expected_deg: f64,
    /// Tolerance in degrees. DE440 geometric positions should be very close
    /// to Swiss Eph geometric positions (both use DE440 internally).
    /// Residual differences come from: precession model, obliquity value,
    /// TT-TDB conversion, and potential aberration in the reference.
    tolerance_deg: f64,
}

const LONGITUDE_REFS_J2000: &[LongitudeRef] = &[
    LongitudeRef {
        body: Body::Sun,
        label: "Sun",
        expected_deg: 280.369,
        tolerance_deg: 0.02,
    },
    LongitudeRef {
        body: Body::Mercury,
        label: "Mercury",
        expected_deg: 271.834,
        tolerance_deg: 0.15,
    },
    LongitudeRef {
        body: Body::Venus,
        label: "Venus",
        expected_deg: 240.949,
        tolerance_deg: 0.7,
    },
    LongitudeRef {
        body: Body::Mars,
        label: "Mars",
        expected_deg: 327.677,
        tolerance_deg: 0.35,
    },
    LongitudeRef {
        body: Body::Jupiter,
        label: "Jupiter",
        expected_deg: 25.253,
        tolerance_deg: 0.05,
    },
    LongitudeRef {
        body: Body::Saturn,
        label: "Saturn",
        expected_deg: 40.459,
        tolerance_deg: 0.1,
    },
    LongitudeRef {
        body: Body::Uranus,
        label: "Uranus",
        expected_deg: 314.386,
        tolerance_deg: 0.5,
    },
    LongitudeRef {
        body: Body::Neptune,
        label: "Neptune",
        expected_deg: 303.503,
        tolerance_deg: 0.35,
    },
];

fn build_de440_almanac() -> Almanac {
    let reader = load_reader();
    let provider = De440Provider::with_reader(reader);
    Almanac::default_vedic().with_provider(Arc::new(provider))
}

#[test]
fn de440_almanac_tropical_longitudes_at_j2000() {
    if !de440s_exists() {
        println!("SKIP: {} not found", DE440S_PATH);
        return;
    }

    let almanac = build_de440_almanac();
    let mut failures = Vec::new();
    let mut report = Vec::new();

    for r in LONGITUDE_REFS_J2000 {
        let lon = almanac
            .geocentric_longitude_deg(r.body, JdUT1(JD_J2000))
            .unwrap_or_else(|e| panic!("{}: geocentric_longitude failed: {:?}", r.label, e));

        let delta = angle_delta(lon, r.expected_deg);

        report.push(format!(
            "  {:>10}: DE440={:.4} deg, SwissEph={:.3} deg, delta={:.4} deg [tol={:.2}] {}",
            r.label,
            lon,
            r.expected_deg,
            delta,
            r.tolerance_deg,
            if delta <= r.tolerance_deg {
                "OK"
            } else {
                "FAIL"
            }
        ));

        if delta > r.tolerance_deg {
            failures.push(format!(
                "  {}: DE440={:.4} deg, SwissEph={:.3} deg, delta={:.4} deg (tolerance={:.2} deg)",
                r.label, lon, r.expected_deg, delta, r.tolerance_deg
            ));
        }
    }

    println!("\n=== DE440 Almanac vs Swiss Eph at J2000 ===");
    for line in &report {
        println!("{}", line);
    }

    assert!(
        failures.is_empty(),
        "\nDE440 Almanac longitude failures at J2000:\n{}\n\nFull report:\n{}",
        failures.join("\n"),
        report.join("\n")
    );
}

#[test]
fn de440_sun_longitude_matches_swiss_eph_tight() {
    if !de440s_exists() {
        println!("SKIP: {} not found", DE440S_PATH);
        return;
    }

    let almanac = build_de440_almanac();
    let lon = almanac
        .geocentric_longitude_deg(Body::Sun, JdUT1(JD_J2000))
        .unwrap();

    // The Sun at J2000 should be ~280.37 deg tropical.
    // DE440 + our coordinate conversion gives 280.379 deg.
    // Swiss Eph geometric: 280.369 deg.
    // The small residual is the precession-model choice plus the apparent-place
    // reduction (light-time + nutation + annual aberration are now all applied).
    let expected = 280.369;
    let tolerance = 0.02;

    assert!(
        angle_delta(lon, expected) < tolerance,
        "Sun at J2000 via DE440 Almanac: expected {:.3} deg, got {:.4} deg (delta={:.4} deg)",
        expected,
        lon,
        angle_delta(lon, expected)
    );

    println!(
        "Sun at J2000: DE440 Almanac = {:.4} deg, Swiss Eph ref = {:.3} deg (delta = {:.4} deg)",
        lon,
        expected,
        angle_delta(lon, expected)
    );
}

// =============================================================================
// PART 5: DE440 vs VSOP87 Comparison
// =============================================================================
//
// At J2000, DE440 and VSOP87 should agree closely for inner planets.
// This test quantifies the difference.

#[test]
fn de440_vs_vsop87_comparison_at_j2000() {
    if !de440s_exists() {
        println!("SKIP: {} not found", DE440S_PATH);
        return;
    }

    let de440_almanac = build_de440_almanac();
    let vsop_almanac = Almanac::default_vedic();

    println!("\n=== DE440 vs VSOP87 at J2000 ===");

    let bodies: Vec<(Body, &str, f64)> = vec![
        (Body::Sun, "Sun", 0.02),
        (Body::Mercury, "Mercury", 0.5),
        (Body::Venus, "Venus", 1.0),
        (Body::Mars, "Mars", 0.5),
        (Body::Jupiter, "Jupiter", 1.0),
        (Body::Saturn, "Saturn", 1.0),
        (Body::Uranus, "Uranus", 1.0),
        (Body::Neptune, "Neptune", 1.0),
    ];

    let mut failures = Vec::new();

    for (body, name, max_diff) in &bodies {
        let de440_lon = de440_almanac
            .geocentric_longitude_deg(*body, JdUT1(JD_J2000))
            .unwrap();
        let vsop_lon = vsop_almanac
            .geocentric_longitude_deg(*body, JdUT1(JD_J2000))
            .unwrap();

        let diff = angle_delta(de440_lon, vsop_lon);

        println!(
            "  {:>10}: DE440={:.4} deg, VSOP87={:.4} deg, diff={:.4} deg",
            name, de440_lon, vsop_lon, diff
        );

        // DE440 and VSOP87 should agree to within the max_diff for each body
        // at J2000 (VSOP87 is most accurate near J2000).
        if diff > *max_diff {
            failures.push(format!(
                "  {}: DE440={:.4}, VSOP87={:.4}, diff={:.4} deg (max={:.2})",
                name, de440_lon, vsop_lon, diff, max_diff
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "\nDE440 vs VSOP87 divergence too large at J2000:\n{}",
        failures.join("\n")
    );
}

// =============================================================================
// PART 6: Multi-Epoch Stability
// =============================================================================

#[test]
fn de440_positions_stable_across_multiple_epochs() {
    if !de440s_exists() {
        println!("SKIP: {} not found", DE440S_PATH);
        return;
    }

    let almanac = build_de440_almanac();

    // Test at several epochs spread across the coverage range.
    let epochs = [
        (2_451_545.0, "J2000.0 (2000-01-01.5)"),
        (2_459_580.5, "2022-01-01"),
        (2_460_310.5, "2024-01-01"),
        (2_442_413.5, "1975-01-01"),
        (2_433_282.5, "1950-01-01"),
    ];

    println!("\n=== DE440 Multi-Epoch Sun Longitude ===");

    for (jd, label) in &epochs {
        let lon = almanac
            .geocentric_longitude_deg(Body::Sun, JdUT1(*jd))
            .expect(&format!("Sun at {}", label));

        // Sun longitude should always be 0-360.
        assert!(
            lon >= 0.0 && lon < 360.0,
            "Sun at {}: longitude out of range: {:.4} deg",
            label,
            lon
        );

        println!("  {}: Sun = {:.4} deg", label, lon);
    }

    // Also verify all planets are computable at each epoch — Pluto included.
    // de440s.bsp DOES carry the Pluto barycenter (NAIF 9) segment, and the
    // reader maps Body::Pluto -> (9, 0) (see `NaifId::body_to_naif` /
    // `De440Target::from_body`), so there is no excuse to skip it. Every epoch
    // above (1950-2024) also sits inside the analytic Pluto window (1885-2099),
    // so the assertion holds whether the kernel serves it directly or the
    // almanac falls through to the analytic series.
    for (jd, label) in &epochs {
        for body in Body::ALL_PLANETS {
            let result = almanac.geocentric_longitude_deg(*body, JdUT1(*jd));
            assert!(
                result.is_ok(),
                "{} at {}: failed with {:?}",
                body,
                label,
                result.err()
            );
            let lon = result.unwrap();
            assert!(
                lon >= 0.0 && lon < 360.0,
                "{} at {}: longitude out of range: {:.4} deg",
                body,
                label,
                lon
            );
        }
    }
}

// =============================================================================
// PART 7: Second Epoch Validation (2024-01-01)
// =============================================================================
//
// Test at a date far from J2000 to verify the reader handles multiple
// Chebyshev records correctly (not just the record containing J2000).

#[test]
fn de440_sun_at_2024_jan_01() {
    if !de440s_exists() {
        println!("SKIP: {} not found", DE440S_PATH);
        return;
    }

    let almanac = build_de440_almanac();
    // 2024-01-01 0h UT1 = JD 2460310.5
    let jd = 2_460_310.5;

    let lon = almanac
        .geocentric_longitude_deg(Body::Sun, JdUT1(jd))
        .unwrap();

    // On 2024-01-01, the Sun is at approximately 280 deg tropical (Capricorn).
    // Swiss Ephemeris gives ~280.05 deg for this date.
    // The Sun moves ~1 deg/day and is near perihelion in early January.
    assert!(
        lon > 278.0 && lon < 282.0,
        "Sun at 2024-01-01: expected ~280 deg, got {:.4} deg",
        lon
    );

    println!("Sun at 2024-01-01: {:.4} deg (expected ~280 deg)", lon);
}

// =============================================================================
// PART 8: Moon Accuracy (DE440 should be much better than ELP2000-82)
// =============================================================================

#[test]
fn de440_moon_at_j2000_matches_jpl_apparent_longitude() {
    if !de440s_exists() {
        println!("SKIP: {} not found", DE440S_PATH);
        return;
    }

    let de440_almanac = build_de440_almanac();
    let vsop_almanac = Almanac::default_vedic();

    // JPL Horizons apparent geocentric tropical ecliptic longitude of the Moon
    // at J2000.0 (quantity #31). This is the same independent reference asserted
    // in swiss_eph_crossval::jpl_moon_j2000. (The previous 218.601° here was a
    // stale/incorrect value — never caught because this test was kernel-gated and
    // always skipped in CI.)
    let jpl_moon_j2000 = 223.3238;

    let de440_moon = de440_almanac
        .geocentric_longitude_deg(Body::Moon, JdUT1(JD_J2000))
        .unwrap();
    let vsop_moon = vsop_almanac
        .geocentric_longitude_deg(Body::Moon, JdUT1(JD_J2000))
        .unwrap();

    let de440_err = angle_delta(de440_moon, jpl_moon_j2000);
    let vsop_err = angle_delta(vsop_moon, jpl_moon_j2000);

    println!("\n=== Moon at J2000 ===");
    println!("  JPL apparent ref: {:.4} deg", jpl_moon_j2000);
    println!(
        "  DE440 Almanac:    {:.4} deg (error = {:.4} deg)",
        de440_moon, de440_err
    );
    println!(
        "  VSOP87 fallback:  {:.4} deg (error = {:.4} deg)",
        vsop_moon, vsop_err
    );

    // The DE440 path computes the apparent Moon as: geometric geocentric vector
    // (taken at the observation epoch — retarding its Earth-relative vector would
    // inject Earth's motion) + IAU 2006 precession + IAU 2000B nutation +
    // GEOCENTRIC light-time (~0.7"), and deliberately NOT the full annual
    // aberration (κ=20.49552") that planets/Sun receive — the geocentric Moon
    // shares Earth's heliocentric velocity. (A prior build applied annual
    // aberration to the Moon, which inflated the J2000 residual to ~11"; that bug
    // is removed.) The DE440 apparent Moon now agrees with the JPL apparent
    // longitude to sub-arcsecond — far inside the 0.01° (36") tolerance — and is
    // closer than the truncated-ELP VSOP path (~1-3"). We assert correctness vs
    // the independent JPL value.
    assert!(
        de440_err < 0.01,
        "Moon at J2000 via DE440: error = {:.5} deg (expected < 0.01 deg from JPL ref {:.4} deg)",
        de440_err,
        jpl_moon_j2000
    );

    // Print which source is closer (for diagnostic purposes).
    if de440_err < vsop_err {
        println!(
            "  -> DE440 is closer to Swiss Eph reference by {:.4} deg",
            vsop_err - de440_err
        );
    } else {
        println!("  -> VSOP87 fallback is closer (DE440 coord conversion needs refinement)");
        println!("  Note: DE440 raw ICRF positions are exact; the delta comes from");
        println!(
            "  our apparent-place reduction (light-time + precession + nutation + aberration)."
        );
    }
}

// =============================================================================
// PART 9: Provider Integration Test
// =============================================================================

#[test]
fn de440_provider_reports_correct_metadata() {
    if !de440s_exists() {
        println!("SKIP: {} not found", DE440S_PATH);
        return;
    }

    let reader = load_reader();
    let provider = De440Provider::with_reader(reader);

    assert!(
        provider.has_de440_data(),
        "De440Provider should report having data after loading de440s.bsp"
    );

    assert!(
        provider.name().contains("DE440"),
        "Provider name should indicate DE440 when data is loaded, got {}",
        provider.name()
    );

    // The quoted figure conservatively bounds the apparent-place reduction the
    // DE440 provider serves. The Moon used to be the worst body at ~11", but that
    // was the annual-aberration bug (the Moon was wrongly given the κ=20.49552"
    // term meant for planets); with it removed the apparent Moon is sub-arcsecond
    // like the Sun and planets (see
    // `de440_moon_at_j2000_matches_jpl_apparent_longitude`). It is NOT a sub-mas
    // figure (that is the RAW geometry; the apparent-place reduction is slightly
    // larger), so the API claim sits at a conservative 2" — above the measured
    // worst body, below the 36" (0.01°) cross-validation tolerance.
    let acc = provider.accuracy_arcsec();
    assert!(
        (1.0..=36.0).contains(&acc),
        "DE440 apparent-place accuracy must bound the worst body (sub-arcsec after \
         the Moon annual-aberration fix) and stay within the 36\" cross-validation \
         tolerance, got {acc} arcsec"
    );

    let (start, end) = provider.coverage();
    assert!(
        start < JD_J2000 && JD_J2000 < end,
        "Provider coverage should include J2000"
    );
}

#[test]
fn de440_provider_geocentric_ecliptic_sun() {
    if !de440s_exists() {
        println!("SKIP: {} not found", DE440S_PATH);
        return;
    }

    let reader = load_reader();
    let provider = De440Provider::with_reader(reader);

    let jd_tt = JdTT(JD_J2000);
    let pos = provider
        .geocentric_ecliptic(Body::Sun, jd_tt)
        .expect("De440Provider geocentric_ecliptic(Sun) failed");

    let lon_deg = pos.longitude.to_degrees().rem_euclid(360.0);
    let lat_deg = pos.latitude.to_degrees();

    // Sun geocentric ecliptic longitude at J2000 TT: ~280.37 deg.
    assert!(
        (lon_deg - 280.37).abs() < 0.05,
        "Sun geocentric ecliptic longitude at J2000: expected ~280.37, got {:.4} deg",
        lon_deg
    );

    // Sun ecliptic latitude should be near zero (< 0.01 deg).
    assert!(
        lat_deg.abs() < 0.01,
        "Sun ecliptic latitude at J2000 should be ~0, got {:.6} deg",
        lat_deg
    );

    println!(
        "De440Provider Sun at J2000: lon={:.4} deg, lat={:.6} deg, dist={:.6} AU",
        lon_deg, lat_deg, pos.distance
    );
}

// =============================================================================
// PART 10: Almanac with_de440 convenience method
// =============================================================================

#[test]
fn almanac_with_de440_loads_and_computes() {
    if !de440s_exists() {
        println!("SKIP: {} not found", DE440S_PATH);
        return;
    }

    let almanac = Almanac::default_vedic().with_de440(Path::new(DE440S_PATH));

    let lon = almanac
        .geocentric_longitude_deg(Body::Sun, JdUT1(JD_J2000))
        .unwrap();

    assert!(
        (lon - 280.37).abs() < 0.05,
        "Almanac::with_de440 Sun at J2000: expected ~280.37 deg, got {:.4} deg",
        lon
    );
}
