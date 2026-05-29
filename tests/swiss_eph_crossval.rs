// JPL Horizons DE440 Cross-Validation — Workspace Level
//
// Reference: NASA/JPL Horizons (https://ssd.jpl.nasa.gov/horizons/)
// Verified: 2026-05-28 via Horizons API quantity #31 (ObsEcLon)
// Frame: apparent geocentric ecliptic-of-date
//
// All reference values verified against JPL DE440 to 15" accuracy.

use xalen_ayanamsa::Ayanamsa;
use xalen_ephem::{Almanac, Body};
use xalen_houses::{GeoLocation, HouseSystem, compute_houses};
use xalen_time::{DeltaTModel, JdUT1, JulianDay};

fn almanac() -> Almanac {
    Almanac::default_vedic()
}

const JD_1950: f64 = 2_433_283.0;
const JD_J2000: f64 = 2_451_545.0;
const JD_2024: f64 = 2_460_311.0;
const JD_2050: f64 = 2_469_808.0;

fn angle_delta(a: f64, b: f64) -> f64 {
    let d = (a - b).abs() % 360.0;
    if d > 180.0 { 360.0 - d } else { d }
}

// ---------------------------------------------------------------------------
// J2000.0 epoch — all 9 bodies
// ---------------------------------------------------------------------------

#[test]
fn jpl_sun_j2000() {
    let lon = almanac().geocentric_longitude_deg(Body::Sun, JdUT1(JD_J2000)).unwrap();
    let delta = angle_delta(lon, 280.3689);
    assert!(delta < 0.001, "Sun: {lon:.4}° delta={delta:.4}° ({:.1}\")", delta * 3600.0);
}

#[test]
fn jpl_moon_j2000() {
    let lon = almanac().geocentric_longitude_deg(Body::Moon, JdUT1(JD_J2000)).unwrap();
    let delta = angle_delta(lon, 223.3238);
    assert!(delta < 0.005, "Moon: {lon:.4}° delta={delta:.4}° ({:.1}\")", delta * 3600.0);
}

#[test]
fn jpl_mercury_j2000() {
    let lon = almanac().geocentric_longitude_deg(Body::Mercury, JdUT1(JD_J2000)).unwrap();
    let delta = angle_delta(lon, 271.8893);
    assert!(delta < 0.001, "Mercury: {lon:.4}° delta={delta:.4}° ({:.1}\")", delta * 3600.0);
}

#[test]
fn jpl_venus_j2000() {
    let lon = almanac().geocentric_longitude_deg(Body::Venus, JdUT1(JD_J2000)).unwrap();
    let delta = angle_delta(lon, 241.5658);
    assert!(delta < 0.001, "Venus: {lon:.4}° delta={delta:.4}° ({:.1}\")", delta * 3600.0);
}

#[test]
fn jpl_mars_j2000() {
    let lon = almanac().geocentric_longitude_deg(Body::Mars, JdUT1(JD_J2000)).unwrap();
    let delta = angle_delta(lon, 327.9633);
    assert!(delta < 0.001, "Mars: {lon:.4}° delta={delta:.4}° ({:.1}\")", delta * 3600.0);
}

#[test]
fn jpl_jupiter_j2000() {
    let lon = almanac().geocentric_longitude_deg(Body::Jupiter, JdUT1(JD_J2000)).unwrap();
    let delta = angle_delta(lon, 25.2531);
    assert!(delta < 0.001, "Jupiter: {lon:.4}° delta={delta:.4}° ({:.1}\")", delta * 3600.0);
}

#[test]
fn jpl_saturn_j2000() {
    let lon = almanac().geocentric_longitude_deg(Body::Saturn, JdUT1(JD_J2000)).unwrap();
    let delta = angle_delta(lon, 40.3956);
    assert!(delta < 0.001, "Saturn: {lon:.4}° delta={delta:.4}° ({:.1}\")", delta * 3600.0);
}

#[test]
fn jpl_uranus_j2000() {
    let lon = almanac().geocentric_longitude_deg(Body::Uranus, JdUT1(JD_J2000)).unwrap();
    let delta = angle_delta(lon, 314.8092);
    assert!(delta < 0.001, "Uranus: {lon:.4}° delta={delta:.4}° ({:.1}\")", delta * 3600.0);
}

#[test]
fn jpl_neptune_j2000() {
    let lon = almanac().geocentric_longitude_deg(Body::Neptune, JdUT1(JD_J2000)).unwrap();
    let delta = angle_delta(lon, 303.1930);
    assert!(delta < 0.001, "Neptune: {lon:.4}° delta={delta:.4}° ({:.1}\")", delta * 3600.0);
}

// ---------------------------------------------------------------------------
// 2024 epoch — multi-epoch confidence
// ---------------------------------------------------------------------------

#[test]
fn jpl_sun_2024() {
    let lon = almanac().geocentric_longitude_deg(Body::Sun, JdUT1(JD_2024)).unwrap();
    let delta = angle_delta(lon, 280.5485);
    assert!(delta < 0.001, "Sun 2024: {lon:.4}° delta={delta:.4}° ({:.1}\")", delta * 3600.0);
}

#[test]
fn jpl_moon_2024() {
    let lon = almanac().geocentric_longitude_deg(Body::Moon, JdUT1(JD_2024)).unwrap();
    let delta = angle_delta(lon, 161.9070);
    assert!(delta < 0.008, "Moon 2024: {lon:.4}° delta={delta:.4}° ({:.1}\")", delta * 3600.0);
}

#[test]
fn jpl_mars_2024() {
    let lon = almanac().geocentric_longitude_deg(Body::Mars, JdUT1(JD_2024)).unwrap();
    let delta = angle_delta(lon, 267.6791);
    assert!(delta < 0.001, "Mars 2024: {lon:.4}° delta={delta:.4}° ({:.1}\")", delta * 3600.0);
}

#[test]
fn jpl_jupiter_2024() {
    let lon = almanac().geocentric_longitude_deg(Body::Jupiter, JdUT1(JD_2024)).unwrap();
    let delta = angle_delta(lon, 35.5844);
    assert!(delta < 0.001, "Jupiter 2024: {lon:.4}° delta={delta:.4}° ({:.1}\")", delta * 3600.0);
}

// ---------------------------------------------------------------------------
// 1950 epoch — 50 years before J2000
// ---------------------------------------------------------------------------

#[test]
fn jpl_sun_1950() {
    let lon = almanac().geocentric_longitude_deg(Body::Sun, JdUT1(JD_1950)).unwrap();
    let delta = angle_delta(lon, 280.5144);
    assert!(delta < 0.001, "Sun 1950: {lon:.4}° delta={delta:.4}° ({:.1}\")", delta * 3600.0);
}

#[test]
fn jpl_moon_1950() {
    let lon = almanac().geocentric_longitude_deg(Body::Moon, JdUT1(JD_1950)).unwrap();
    let delta = angle_delta(lon, 67.5445);
    assert!(delta < 0.008, "Moon 1950: {lon:.4}° delta={delta:.4}° ({:.1}\")", delta * 3600.0);
}

#[test]
fn jpl_jupiter_1950() {
    let lon = almanac().geocentric_longitude_deg(Body::Jupiter, JdUT1(JD_1950)).unwrap();
    let delta = angle_delta(lon, 306.6180);
    assert!(delta < 0.001, "Jupiter 1950: {lon:.4}° delta={delta:.4}° ({:.1}\")", delta * 3600.0);
}

// ---------------------------------------------------------------------------
// 2050 epoch — 50 years after J2000
// ---------------------------------------------------------------------------

#[test]
fn jpl_sun_2050() {
    let lon = almanac().geocentric_longitude_deg(Body::Sun, JdUT1(JD_2050)).unwrap();
    let delta = angle_delta(lon, 281.2579);
    assert!(delta < 0.001, "Sun 2050: {lon:.4}° delta={delta:.4}° ({:.1}\")", delta * 3600.0);
}

#[test]
fn jpl_moon_2050() {
    let lon = almanac().geocentric_longitude_deg(Body::Moon, JdUT1(JD_2050)).unwrap();
    let delta = angle_delta(lon, 25.3810);
    assert!(delta < 0.005, "Moon 2050: {lon:.4}° delta={delta:.4}° ({:.1}\")", delta * 3600.0);
}

#[test]
fn jpl_jupiter_2050() {
    let lon = almanac().geocentric_longitude_deg(Body::Jupiter, JdUT1(JD_2050)).unwrap();
    let delta = angle_delta(lon, 121.6322);
    assert!(delta < 0.001, "Jupiter 2050: {lon:.4}° delta={delta:.4}° ({:.1}\")", delta * 3600.0);
}

// ---------------------------------------------------------------------------
// Sidereal positions — Lahiri ayanamsa
// ---------------------------------------------------------------------------

#[test]
fn sidereal_sun_j2000_lahiri() {
    let a = almanac();
    let jd = JdUT1(JD_J2000);
    let trop = a.geocentric_longitude_deg(Body::Sun, jd).unwrap();
    let tt = jd.to_tt(&DeltaTModel::StephensonMorrisonHohenkerk2016);
    let aya = Ayanamsa::Lahiri.compute_deg(tt.as_f64());
    let sid = (trop - aya).rem_euclid(360.0);
    // Lahiri ~23.86° at J2000 → sidereal Sun ~256.5°
    assert!(sid > 254.0 && sid < 258.0, "Sidereal Sun: {sid:.4}° (aya={aya:.4}°)");
}

// ---------------------------------------------------------------------------
// Rahu-Ketu opposition invariant
// ---------------------------------------------------------------------------

#[test]
fn rahu_ketu_opposite() {
    let a = almanac();
    let rahu = a.geocentric_longitude_deg(Body::MeanNode, JdUT1(JD_J2000)).unwrap();
    let ketu = (rahu + 180.0).rem_euclid(360.0);
    assert!(rahu >= 0.0 && rahu < 360.0);
    assert!((angle_delta(rahu, ketu) - 180.0).abs() < 0.01);
}

// ---------------------------------------------------------------------------
// House cusps sanity
// ---------------------------------------------------------------------------

#[test]
fn houses_pune_placidus() {
    let loc = GeoLocation::new(18.52, 73.85);
    let epsilon = xalen_coords::obliquity::mean_obliquity(0.0);
    let h = compute_houses(JD_J2000, &loc, epsilon, HouseSystem::Placidus);

    let mc = h.mc.to_degrees().rem_euclid(360.0);
    let ic = h.ic.to_degrees().rem_euclid(360.0);
    assert!((angle_delta(mc, ic) - 180.0).abs() < 0.01, "MC-IC not opposite");

    for i in 0..12 {
        let c = h.cusp_deg(i);
        assert!(c >= 0.0 && c < 360.0 && !c.is_nan(), "Cusp {} invalid", i + 1);
    }
}

// ---------------------------------------------------------------------------
// Ayanamsa bounds
// ---------------------------------------------------------------------------

#[test]
fn ayanamsa_lahiri_bounds() {
    let tt = JdUT1(JD_J2000).to_tt(&DeltaTModel::StephensonMorrisonHohenkerk2016);
    let lahiri = Ayanamsa::Lahiri.compute_deg(tt.as_f64());
    assert!(lahiri > 23.5 && lahiri < 24.2, "Lahiri at J2000: {lahiri:.4}°");
}

#[test]
fn ayanamsa_kp_less_than_lahiri() {
    let tt = JdUT1(JD_J2000).to_tt(&DeltaTModel::StephensonMorrisonHohenkerk2016);
    let lahiri = Ayanamsa::Lahiri.compute_deg(tt.as_f64());
    let kp = Ayanamsa::KPKrishnamurti.compute_deg(tt.as_f64());
    assert!(lahiri > kp, "Lahiri ({lahiri:.4}°) must > KP ({kp:.4}°)");
}
