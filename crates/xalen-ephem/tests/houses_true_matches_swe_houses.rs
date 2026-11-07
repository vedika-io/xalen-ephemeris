//! `xalen_houses::compute_houses_true` must reproduce the Swiss-compatible
//! `compat::swe_houses` path, which it was extracted from, for every system
//! and several places. At J2000 the agreement is checked to 1e-9 degrees.

use xalen_ephem::compat::swe_houses;
use xalen_houses::{GeoLocation, HouseSystem, compute_houses_true};

const J2000: f64 = 2_451_545.0;

fn ang_diff(a: f64, b: f64) -> f64 {
    ((a - b + 180.0).rem_euclid(360.0) - 180.0).abs()
}

fn check(system: HouseSystem, code: char, jd: f64, lat: f64, lon: f64) {
    let swe = swe_houses(jd, lat, lon, code).expect("swe_houses");
    let ours = compute_houses_true(jd, &GeoLocation::new(lat, lon), system);
    let tol = 1e-9;
    assert!(
        ang_diff(ours.ascendant.to_degrees(), swe.ascendant) < tol,
        "{code} asc at ({lat},{lon}) jd {jd}"
    );
    assert!(
        ang_diff(ours.mc.to_degrees(), swe.mc) < tol,
        "{code} mc at ({lat},{lon}) jd {jd}"
    );
    assert!(
        ang_diff(ours.vertex.to_degrees(), swe.vertex) < tol,
        "{code} vertex at ({lat},{lon}) jd {jd}"
    );
    for i in 0..12 {
        assert!(
            ang_diff(ours.cusp_deg(i), swe.cusps[i]) < tol,
            "{code} cusp {} at ({lat},{lon}) jd {jd}: {} vs {}",
            i + 1,
            ours.cusp_deg(i),
            swe.cusps[i]
        );
    }
}

#[test]
fn placidus_at_j2000_matches_swe_houses_to_1e_9() {
    check(HouseSystem::Placidus, 'P', J2000, 18.52, 73.85);
}

#[test]
fn several_systems_and_places_match_swe_houses() {
    let systems = [
        (HouseSystem::Placidus, 'P'),
        (HouseSystem::Koch, 'K'),
        (HouseSystem::Equal, 'A'),
        (HouseSystem::WholeSign, 'W'),
        (HouseSystem::Porphyry, 'O'),
        (HouseSystem::Regiomontanus, 'R'),
        (HouseSystem::Campanus, 'C'),
    ];
    let places = [
        (51.5074, -0.1278),
        (-33.8688, 151.2093),
        (40.7128, -74.0060),
    ];
    for (system, code) in systems {
        for (lat, lon) in places {
            check(system, code, 2_460_482.5, lat, lon);
        }
    }
}
