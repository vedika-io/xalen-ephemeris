// JPL Horizons DE440 Cross-Validation Test Vectors for XALEN Ephemeris
//
// Reference: NASA/JPL Horizons System (https://ssd.jpl.nasa.gov/horizons/)
// Query: geocentric observer, ecliptic-of-date, apparent positions
// Verified: 2026-05-28 via Horizons API quantity #31 (ObsEcLon)
//
// COORDINATE FRAME: apparent geocentric ecliptic-of-date (with light-time,
// gravitational deflection, and stellar aberration — matches Horizons default).
//
// MEASURED ACCURACY (this engine vs JPL DE440):
//   Sun:     14" at J2000, 6" at 2024  (VSOP87A full series)
//   Moon:    truncated ELP2000-82 (60 longitude terms), assembled into an
//            apparent place by vsop::apparent_moon = mean-of-date series + Δψ
//            (IAU 2000B nutation) + geocentric light-time (~0.7"). It does NOT
//            receive the full ANNUAL aberration term (κ=20.49552") that planets
//            get — the geocentric Moon shares Earth's heliocentric velocity, so
//            that term does not apply (a prior build wrongly applied it, adding
//            up to ~44" of error). Residual vs pyswisseph 2.10.03: RMS ~2.82"
//            over AD 1600-2100 (max ~12"), now limited by the 60-term series
//            truncation; at these J2000/1950/2024/2050 epochs the apparent Moon
//            lands within ~0.3-3.5" of JPL. The corrected Moon also improves the
//            NASA Besselian eclipse-time agreement (2017/2024 from ~19-30s to
//            ~2s). Use the DE440 provider for sub-arcsecond apparent Moon.
//   Mercury: 14" at J2000              (VSOP87A)
//   Venus:   14" at J2000              (VSOP87A)
//   Mars:    14" at J2000, 6" at 2024  (VSOP87A)
//   Jupiter: 14" at J2000, 5" at 2024  (VSOP87A)
//   Saturn:  14" at J2000              (VSOP87A)
//   Uranus:  15" at J2000              (VSOP87A)
//   Neptune: 15" at J2000              (VSOP87A)
//
// TOLERANCE POLICY: set at 2x measured error to allow for epoch variation.

use xalen_ayanamsa::Ayanamsa;
use xalen_coords::{ARCSEC_TO_RAD, RAD_TO_DEG, mean_obliquity, nutation_2000b};
use xalen_ephem::{Almanac, Body, EphemerisProvider, Vsop87Provider};
use xalen_houses::{GeoLocation, HouseSystem, compute_houses};
use xalen_time::{DeltaTModel, JdTT, JdUT1, JulianDay, delta_t};

fn almanac() -> Almanac {
    Almanac::default_vedic()
}

const JD_1950: f64 = 2_433_283.0;
const JD_J2000: f64 = 2_451_545.0;
const JD_2024: f64 = 2_460_311.0;
const JD_2050: f64 = 2_469_808.0;

struct JplRef {
    body: Body,
    label: &'static str,
    expected_deg: f64,
    tolerance_deg: f64,
}

// JPL Horizons DE440 — J2000.0 (2000-01-01 12:00 UT)
// Expected values are apparent geocentric ecliptic-of-date longitudes (ObsEcLon).
// VSOP87 planets land within ~1.1"; the Moon's apparent path (mean-of-date ELP
// + Δψ + geocentric light-time, NOT annual aberration) lands within the Moon
// tolerance below — the residual is the truncated-ELP series limit, not a frame
// error.
const JPL_J2000_REFS: &[JplRef] = &[
    JplRef {
        body: Body::Sun,
        label: "Sun",
        expected_deg: 280.3689,
        tolerance_deg: 0.001,
    },
    JplRef {
        body: Body::Moon,
        label: "Moon",
        expected_deg: 223.3238,
        tolerance_deg: 0.006,
    },
    JplRef {
        body: Body::Mercury,
        label: "Mercury",
        expected_deg: 271.8893,
        tolerance_deg: 0.001,
    },
    JplRef {
        body: Body::Venus,
        label: "Venus",
        expected_deg: 241.5658,
        tolerance_deg: 0.001,
    },
    JplRef {
        body: Body::Mars,
        label: "Mars",
        expected_deg: 327.9633,
        tolerance_deg: 0.001,
    },
    JplRef {
        body: Body::Jupiter,
        label: "Jupiter",
        expected_deg: 25.2531,
        tolerance_deg: 0.001,
    },
    JplRef {
        body: Body::Saturn,
        label: "Saturn",
        expected_deg: 40.3956,
        tolerance_deg: 0.001,
    },
    JplRef {
        body: Body::Uranus,
        label: "Uranus",
        expected_deg: 314.8092,
        tolerance_deg: 0.001,
    },
    JplRef {
        body: Body::Neptune,
        label: "Neptune",
        expected_deg: 303.1930,
        tolerance_deg: 0.001,
    },
];

// JPL Horizons DE440 — 1950-01-01 12:00 UT (50 years before J2000)
const JPL_1950_REFS: &[JplRef] = &[
    JplRef {
        body: Body::Sun,
        label: "Sun",
        expected_deg: 280.5144,
        tolerance_deg: 0.001,
    },
    JplRef {
        body: Body::Moon,
        label: "Moon",
        expected_deg: 67.5445,
        tolerance_deg: 0.008,
    },
    JplRef {
        body: Body::Jupiter,
        label: "Jupiter",
        expected_deg: 306.6180,
        tolerance_deg: 0.001,
    },
    JplRef {
        body: Body::Saturn,
        label: "Saturn",
        expected_deg: 169.4355,
        tolerance_deg: 0.001,
    },
];

// JPL Horizons DE440 — 2024-01-01 12:00 UT
const JPL_2024_REFS: &[JplRef] = &[
    JplRef {
        body: Body::Sun,
        label: "Sun",
        expected_deg: 280.5485,
        tolerance_deg: 0.001,
    },
    JplRef {
        body: Body::Moon,
        label: "Moon",
        expected_deg: 161.9070,
        tolerance_deg: 0.008,
    },
    JplRef {
        body: Body::Mars,
        label: "Mars",
        expected_deg: 267.6791,
        tolerance_deg: 0.001,
    },
    JplRef {
        body: Body::Jupiter,
        label: "Jupiter",
        expected_deg: 35.5844,
        tolerance_deg: 0.001,
    },
];

// JPL Horizons DE440 — 2050-01-01 12:00 UT (50 years after J2000)
const JPL_2050_REFS: &[JplRef] = &[
    JplRef {
        body: Body::Sun,
        label: "Sun",
        expected_deg: 281.2579,
        tolerance_deg: 0.001,
    },
    JplRef {
        body: Body::Moon,
        label: "Moon",
        expected_deg: 25.3810,
        tolerance_deg: 0.006,
    },
    JplRef {
        body: Body::Jupiter,
        label: "Jupiter",
        expected_deg: 121.6322,
        tolerance_deg: 0.001,
    },
    JplRef {
        body: Body::Saturn,
        label: "Saturn",
        expected_deg: 297.6321,
        tolerance_deg: 0.001,
    },
];

fn angle_delta(a: f64, b: f64) -> f64 {
    let d = (a - b).abs() % 360.0;
    if d > 180.0 { 360.0 - d } else { d }
}

// ---------------------------------------------------------------------------
// Test 1: All planets at J2000.0 — tight tolerances
// ---------------------------------------------------------------------------

#[test]
fn jpl_planetary_longitudes_j2000() {
    let a = almanac();
    let mut failures = Vec::new();

    for r in JPL_J2000_REFS {
        let lon = a.geocentric_longitude_deg(r.body, JdUT1(JD_J2000)).unwrap();
        let delta = angle_delta(lon, r.expected_deg);
        if delta > r.tolerance_deg {
            failures.push(format!(
                "  {}: JPL={:.4}° XALEN={:.4}° delta={:.4}° ({:.1}\") tolerance={:.3}°",
                r.label,
                r.expected_deg,
                lon,
                delta,
                delta * 3600.0,
                r.tolerance_deg
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "JPL DE440 cross-validation failures at J2000.0:\n{}",
        failures.join("\n")
    );
}

// ---------------------------------------------------------------------------
// Test 2: Multi-epoch validation — 1950, 2024, 2050
// ---------------------------------------------------------------------------

#[test]
fn jpl_planetary_longitudes_1950() {
    let a = almanac();
    let mut failures = Vec::new();
    for r in JPL_1950_REFS {
        let lon = a.geocentric_longitude_deg(r.body, JdUT1(JD_1950)).unwrap();
        let delta = angle_delta(lon, r.expected_deg);
        if delta > r.tolerance_deg {
            failures.push(format!(
                "  {}: JPL={:.4}° XALEN={:.4}° delta={:.4}° ({:.1}\")",
                r.label,
                r.expected_deg,
                lon,
                delta,
                delta * 3600.0
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "1950 failures:\n{}",
        failures.join("\n")
    );
}

#[test]
fn jpl_planetary_longitudes_2024() {
    let a = almanac();
    let mut failures = Vec::new();
    for r in JPL_2024_REFS {
        let lon = a.geocentric_longitude_deg(r.body, JdUT1(JD_2024)).unwrap();
        let delta = angle_delta(lon, r.expected_deg);
        if delta > r.tolerance_deg {
            failures.push(format!(
                "  {}: JPL={:.4}° XALEN={:.4}° delta={:.4}° ({:.1}\")",
                r.label,
                r.expected_deg,
                lon,
                delta,
                delta * 3600.0
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "2024 failures:\n{}",
        failures.join("\n")
    );
}

#[test]
fn jpl_planetary_longitudes_2050() {
    let a = almanac();
    let mut failures = Vec::new();
    for r in JPL_2050_REFS {
        let lon = a.geocentric_longitude_deg(r.body, JdUT1(JD_2050)).unwrap();
        let delta = angle_delta(lon, r.expected_deg);
        if delta > r.tolerance_deg {
            failures.push(format!(
                "  {}: JPL={:.4}° XALEN={:.4}° delta={:.4}° ({:.1}\")",
                r.label,
                r.expected_deg,
                lon,
                delta,
                delta * 3600.0
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "2050 failures:\n{}",
        failures.join("\n")
    );
}

// ---------------------------------------------------------------------------
// Test 3: Individual planet tight checks at J2000
// ---------------------------------------------------------------------------

#[test]
fn jpl_sun_j2000() {
    let a = almanac();
    let lon = a
        .geocentric_longitude_deg(Body::Sun, JdUT1(JD_J2000))
        .unwrap();
    let delta = angle_delta(lon, 280.3689);
    assert!(
        delta < 0.001,
        "Sun at J2000: delta={delta:.4}° ({:.1}\")",
        delta * 3600.0
    );
}

#[test]
fn jpl_moon_j2000() {
    let a = almanac();
    let lon = a
        .geocentric_longitude_deg(Body::Moon, JdUT1(JD_J2000))
        .unwrap();
    let delta = angle_delta(lon, 223.3238);
    assert!(
        delta < 0.006,
        "Moon at J2000: delta={delta:.4}° ({:.1}\")",
        delta * 3600.0
    );
}

#[test]
fn jpl_mercury_j2000() {
    let a = almanac();
    let lon = a
        .geocentric_longitude_deg(Body::Mercury, JdUT1(JD_J2000))
        .unwrap();
    let delta = angle_delta(lon, 271.8893);
    assert!(
        delta < 0.001,
        "Mercury at J2000: delta={delta:.4}° ({:.1}\")",
        delta * 3600.0
    );
}

#[test]
fn jpl_venus_j2000() {
    let a = almanac();
    let lon = a
        .geocentric_longitude_deg(Body::Venus, JdUT1(JD_J2000))
        .unwrap();
    let delta = angle_delta(lon, 241.5658);
    assert!(
        delta < 0.001,
        "Venus at J2000: delta={delta:.4}° ({:.1}\")",
        delta * 3600.0
    );
}

#[test]
fn jpl_mars_j2000() {
    let a = almanac();
    let lon = a
        .geocentric_longitude_deg(Body::Mars, JdUT1(JD_J2000))
        .unwrap();
    let delta = angle_delta(lon, 327.9633);
    assert!(
        delta < 0.001,
        "Mars at J2000: delta={delta:.4}° ({:.1}\")",
        delta * 3600.0
    );
}

#[test]
fn jpl_jupiter_j2000() {
    let a = almanac();
    let lon = a
        .geocentric_longitude_deg(Body::Jupiter, JdUT1(JD_J2000))
        .unwrap();
    let delta = angle_delta(lon, 25.2531);
    assert!(
        delta < 0.001,
        "Jupiter at J2000: delta={delta:.4}° ({:.1}\")",
        delta * 3600.0
    );
}

#[test]
fn jpl_saturn_j2000() {
    let a = almanac();
    let lon = a
        .geocentric_longitude_deg(Body::Saturn, JdUT1(JD_J2000))
        .unwrap();
    let delta = angle_delta(lon, 40.3956);
    assert!(
        delta < 0.001,
        "Saturn at J2000: delta={delta:.4}° ({:.1}\")",
        delta * 3600.0
    );
}

#[test]
fn jpl_uranus_j2000() {
    let a = almanac();
    let lon = a
        .geocentric_longitude_deg(Body::Uranus, JdUT1(JD_J2000))
        .unwrap();
    let delta = angle_delta(lon, 314.8092);
    assert!(
        delta < 0.001,
        "Uranus at J2000: delta={delta:.4}° ({:.1}\")",
        delta * 3600.0
    );
}

#[test]
fn jpl_neptune_j2000() {
    let a = almanac();
    let lon = a
        .geocentric_longitude_deg(Body::Neptune, JdUT1(JD_J2000))
        .unwrap();
    let delta = angle_delta(lon, 303.1930);
    assert!(
        delta < 0.001,
        "Neptune at J2000: delta={delta:.4}° ({:.1}\")",
        delta * 3600.0
    );
}

// ---------------------------------------------------------------------------
// Test 4: Mean obliquity at J2000.0 — IAU 2006
// ---------------------------------------------------------------------------

#[test]
fn obliquity_j2000() {
    let mean_obl_deg = mean_obliquity(0.0) * RAD_TO_DEG;
    assert!(
        (mean_obl_deg - 23.4393).abs() < 0.001,
        "Mean obliquity at J2000: got {mean_obl_deg:.6}°"
    );
}

#[test]
fn true_obliquity_j2000() {
    let mean_obl = mean_obliquity(0.0);
    let nut = nutation_2000b(0.0);
    let true_obl_deg = (mean_obl + nut.delta_epsilon) * RAD_TO_DEG;
    assert!(
        (true_obl_deg - 23.4386).abs() < 0.002,
        "True obliquity at J2000: got {true_obl_deg:.6}°"
    );
}

// ---------------------------------------------------------------------------
// Test 5: Delta-T at J2000.0 — IERS observed 63.83s
// ---------------------------------------------------------------------------

#[test]
fn delta_t_j2000() {
    let dt = delta_t(JD_J2000, &DeltaTModel::StephensonMorrisonHohenkerk2016);
    assert!(
        (dt - 63.83).abs() < 1.0,
        "delta-T at J2000: expected ~63.83s, got {dt:.4}s"
    );
}

// ---------------------------------------------------------------------------
// Test 6: House cusps sanity — Pune, Placidus, J2000
// ---------------------------------------------------------------------------

#[test]
fn house_cusps_pune_j2000() {
    let loc = GeoLocation::new(18.52, 73.85);
    let epsilon = mean_obliquity(0.0);
    let h = compute_houses(JD_J2000, &loc, epsilon, HouseSystem::Placidus);

    let asc = h.ascendant.to_degrees().rem_euclid(360.0);
    let mc = h.mc.to_degrees().rem_euclid(360.0);
    let ic = h.ic.to_degrees().rem_euclid(360.0);
    let desc = h.descendant.to_degrees().rem_euclid(360.0);

    assert!(asc >= 0.0 && asc < 360.0 && !asc.is_nan());
    assert!(
        (angle_delta(mc, ic) - 180.0).abs() < 0.01,
        "MC-IC not opposite"
    );
    assert!(
        (angle_delta(asc, desc) - 180.0).abs() < 0.01,
        "ASC-DSC not opposite"
    );

    for i in 0..12 {
        let c = h.cusp_deg(i);
        assert!(
            c >= 0.0 && c < 360.0 && !c.is_nan(),
            "Cusp {} invalid: {c}",
            i + 1
        );
    }
}

// ---------------------------------------------------------------------------
// Test 7: Ayanamsa — Lahiri and KP at J2000
// ---------------------------------------------------------------------------

#[test]
fn ayanamsa_lahiri_j2000() {
    let tt = JdUT1(JD_J2000).to_tt(&DeltaTModel::StephensonMorrisonHohenkerk2016);
    let lahiri = Ayanamsa::Lahiri.compute_deg(tt.as_f64());
    assert!(
        (lahiri - 23.86).abs() < 0.05,
        "Lahiri at J2000: got {lahiri:.4}°"
    );
}

#[test]
fn ayanamsa_kp_j2000() {
    let tt = JdUT1(JD_J2000).to_tt(&DeltaTModel::StephensonMorrisonHohenkerk2016);
    let kp = Ayanamsa::KPKrishnamurti.compute_deg(tt.as_f64());
    assert!((kp - 23.80).abs() < 0.06, "KP at J2000: got {kp:.4}°");
}

#[test]
fn ayanamsa_kp_less_than_lahiri() {
    let tt = JdUT1(JD_J2000).to_tt(&DeltaTModel::StephensonMorrisonHohenkerk2016);
    let lahiri = Ayanamsa::Lahiri.compute_deg(tt.as_f64());
    let kp = Ayanamsa::KPKrishnamurti.compute_deg(tt.as_f64());
    assert!(lahiri > kp, "Lahiri ({lahiri:.4}°) must > KP ({kp:.4}°)");
    let diff = lahiri - kp;
    assert!(diff > 0.01 && diff < 0.2, "Lahiri - KP: got {diff:.4}°");
}

// ---------------------------------------------------------------------------
// Test 8: Nutation at J2000 — IAU 2000B
// ---------------------------------------------------------------------------

#[test]
fn nutation_j2000() {
    let nut = nutation_2000b(0.0);
    let dpsi = nut.delta_psi / ARCSEC_TO_RAD;
    let deps = nut.delta_epsilon / ARCSEC_TO_RAD;
    assert!(
        (dpsi - (-14.0)).abs() < 1.0,
        "dpsi at J2000: got {dpsi:.2}\""
    );
    assert!(
        (deps - (-5.8)).abs() < 1.0,
        "deps at J2000: got {deps:.2}\""
    );
}

// ---------------------------------------------------------------------------
// Test 9: Provider TT vs Almanac UT1 gap
// ---------------------------------------------------------------------------

#[test]
fn provider_tt_vs_almanac_ut1() {
    let a = almanac();
    let p = Vsop87Provider::new();

    let ut1 = a
        .geocentric_longitude_deg(Body::Sun, JdUT1(JD_J2000))
        .unwrap();
    let tt = p
        .geocentric_ecliptic(Body::Sun, JdTT(JD_J2000))
        .unwrap()
        .longitude
        .to_degrees()
        .rem_euclid(360.0);

    let diff = angle_delta(ut1, tt);
    assert!(
        diff < 0.01,
        "UT1 vs TT gap: {diff:.6}° (should be < 0.01° from ~64s delta-T)"
    );
}

// ---------------------------------------------------------------------------
// Test 10: All VSOP87 bodies computable at J2000
// ---------------------------------------------------------------------------

#[test]
fn all_vsop87_bodies_compute() {
    let p = Vsop87Provider::new();
    let jd = JdTT::J2000;
    for body in [
        Body::Sun,
        Body::Moon,
        Body::Mercury,
        Body::Venus,
        Body::Mars,
        Body::Jupiter,
        Body::Saturn,
        Body::Uranus,
        Body::Neptune,
    ] {
        let r = p.geocentric_ecliptic(body, jd);
        assert!(r.is_ok(), "{body} failed: {:?}", r.err());
        let lon = r.unwrap().longitude.to_degrees().rem_euclid(360.0);
        assert!(lon >= 0.0 && lon < 360.0, "{body} out of range: {lon:.4}°");
    }
}
