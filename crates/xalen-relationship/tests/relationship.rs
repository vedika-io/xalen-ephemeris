//! Deterministic tests for synastry, composite and Davison charts.
//!
//! Tolerances: angles are compared to 1e-9 degrees where the arithmetic is
//! exact up to rounding, and to the stated figure otherwise.

use std::sync::OnceLock;
use xalen_ephem::{Almanac, Body};
use xalen_houses::{GeoLocation, HouseCusps, HouseSystem, compute_houses_true};
use xalen_relationship::{
    BirthMoment, ChartBody, CompositeAscendant, CompositeOptions, DavisonOptions, NatalChart,
    PointId, RelationshipError, SpaceMidpoint, SynastryOptions, composite_chart, davison_chart,
    default_bodies, house_overlay, inter_aspects, synastry,
};
use xalen_time::JdUT1;
use xalen_western::aspects::{AspectType, find_all_aspects};

const TOL: f64 = 1e-9;
const JD_2024: f64 = 2_460_482.5; // 2024-06-21 00:00 UT

fn almanac() -> &'static Almanac {
    static A: OnceLock<Almanac> = OnceLock::new();
    A.get_or_init(Almanac::default_vedic)
}

fn chart(jd: f64, lat: f64, lon: f64, system: HouseSystem) -> NatalChart {
    NatalChart::compute(
        almanac(),
        JdUT1(jd),
        &GeoLocation::new(lat, lon),
        system,
        &default_bodies(),
    )
    .expect("chart")
}

fn london(jd: f64) -> NatalChart {
    chart(jd, 51.5074, -0.1278, HouseSystem::Placidus)
}

fn sydney(jd: f64) -> NatalChart {
    chart(jd, -33.8688, 151.2093, HouseSystem::Placidus)
}

/// Signed shortest difference `a - b` in degrees, in `[-180, 180)`.
fn sdiff(a: f64, b: f64) -> f64 {
    (a - b + 180.0).rem_euclid(360.0) - 180.0
}

fn equal_cusps_literal() -> HouseCusps {
    let mut cusps = [0.0; 12];
    for (i, c) in cusps.iter_mut().enumerate() {
        *c = (30.0 * i as f64).to_radians();
    }
    HouseCusps {
        system: HouseSystem::Equal,
        cusps,
        ascendant: 0.0,
        mc: 270.0_f64.to_radians(),
        ic: 90.0_f64.to_radians(),
        descendant: 180.0_f64.to_radians(),
        vertex: 0.0,
        equatorial_ascendant: 0.0,
        co_ascendant_koch: 0.0,
        co_ascendant_munkasey: 0.0,
        polar_ascendant_munkasey: 0.0,
        fallback_used: false,
    }
}

/// A chart with chosen body longitudes and real houses, no ephemeris needed.
fn synthetic(jd: f64, lat: f64, lon: f64, bodies: &[(Body, f64)]) -> NatalChart {
    let loc = GeoLocation::new(lat, lon);
    NatalChart {
        jd_ut1: jd,
        location: loc,
        bodies: bodies
            .iter()
            .map(|&(body, longitude_deg)| ChartBody {
                body,
                longitude_deg,
                latitude_deg: 0.0,
                speed_deg_per_day: 1.0,
            })
            .collect(),
        houses: compute_houses_true(jd, &loc, HouseSystem::Placidus),
    }
}

// ---------------------------------------------------------------- natal chart

#[test]
fn natal_chart_holds_requested_bodies_houses_and_points() {
    let c = london(JD_2024);
    assert_eq!(c.bodies.len(), 11);
    assert_eq!(c.bodies[0].body, Body::Sun);
    // Sun at the June solstice 2024: about 90 degrees, direct motion about 0.95/day.
    let sun = c.bodies[0];
    assert!(
        (sun.longitude_deg - 90.0).abs() < 0.5,
        "Sun {}",
        sun.longitude_deg
    );
    assert!(sun.speed_deg_per_day > 0.9 && sun.speed_deg_per_day < 1.05);
    let pts = c.points(true);
    assert_eq!(pts.len(), 15);
    assert_eq!(pts[11].0, PointId::Ascendant);
    assert_eq!(pts[12].0, PointId::Midheaven);
    assert_eq!(pts[13].0, PointId::Descendant);
    assert_eq!(pts[14].0, PointId::Imum);
    assert!(sdiff(pts[11].1, pts[13].1).abs() > 179.999999);
    assert!(sdiff(pts[12].1, pts[14].1).abs() > 179.999999);
    assert_eq!(c.points(false).len(), 11);
}

#[test]
fn natal_chart_rejects_bad_input_without_panicking() {
    let loc = GeoLocation::new(51.5, -0.1);
    let p = HouseSystem::Placidus;
    let r = NatalChart::compute(almanac(), JdUT1(f64::NAN), &loc, p, &[Body::Sun]);
    assert!(matches!(r, Err(RelationshipError::InvalidInput(_))));
    let r = NatalChart::compute(almanac(), JdUT1(JD_2024), &loc, p, &[Body::Earth]);
    assert!(matches!(r, Err(RelationshipError::InvalidInput(_))));
    let bad = GeoLocation::new(91.0, 0.0);
    let r = NatalChart::compute(almanac(), JdUT1(JD_2024), &bad, p, &[Body::Sun]);
    assert!(matches!(r, Err(RelationshipError::InvalidInput(_))));
    let nan = GeoLocation::new(f64::NAN, 0.0);
    let r = NatalChart::compute(almanac(), JdUT1(JD_2024), &nan, p, &[Body::Sun]);
    assert!(matches!(r, Err(RelationshipError::InvalidInput(_))));
    // Pluto is outside its analytic series in antiquity: the ephemeris error is propagated.
    let r = NatalChart::compute(almanac(), JdUT1(2_300_000.0), &loc, p, &[Body::Pluto]);
    assert!(matches!(r, Err(RelationshipError::Ephemeris(_))));
}

#[test]
fn duplicate_bodies_are_charted_once() {
    let loc = GeoLocation::new(0.0, 0.0);
    let c = NatalChart::compute(
        almanac(),
        JdUT1(JD_2024),
        &loc,
        HouseSystem::Equal,
        &[Body::Mars, Body::Sun, Body::Mars],
    )
    .unwrap();
    let order: Vec<Body> = c.bodies.iter().map(|b| b.body).collect();
    assert_eq!(order, vec![Body::Mars, Body::Sun]);
}

#[test]
fn relocation_changes_houses_only() {
    let natal = london(JD_2024);
    let tokyo = GeoLocation::new(35.6762, 139.6503);
    let moved = natal.relocated(&tokyo, HouseSystem::Placidus).unwrap();
    assert_eq!(moved.bodies, natal.bodies);
    assert_eq!(moved.jd_ut1, natal.jd_ut1);
    assert!(
        sdiff(
            moved.houses.ascendant.to_degrees(),
            natal.houses.ascendant.to_degrees()
        )
        .abs()
            > 1.0
    );
    // Relocating to the birthplace reproduces the natal houses.
    let same = natal
        .relocated(&natal.location, HouseSystem::Placidus)
        .unwrap();
    for i in 0..12 {
        assert!(sdiff(same.houses.cusp_deg(i), natal.houses.cusp_deg(i)).abs() < TOL);
    }
    let r = natal.relocated(&GeoLocation::new(f64::INFINITY, 0.0), HouseSystem::Placidus);
    assert!(matches!(r, Err(RelationshipError::InvalidInput(_))));
}

#[test]
fn chart_data_matches_the_chart() {
    let c = london(JD_2024);
    let d = c.to_chart_data();
    assert_eq!(d.planet_positions.len(), c.bodies.len());
    assert_eq!(d.ayanamsa_deg, 0.0);
    assert_eq!(d.planet_positions[0].abbreviation, "Su");
    assert_eq!(
        d.planet_positions[0].longitude_deg,
        c.bodies[0].longitude_deg
    );
    assert!(
        d.planet_positions
            .iter()
            .all(|p| (1..=12).contains(&p.house))
    );
    for i in 0..12 {
        assert!((d.house_cusps_deg[i] - c.houses.cusp_deg(i)).abs() < TOL);
    }
    let asc = c.houses.ascendant.to_degrees().rem_euclid(360.0);
    assert_eq!(d.ascendant_sign_index, (asc / 30.0) as usize);
}

// ------------------------------------------------------------------- synastry

#[test]
fn chart_against_itself_has_exact_conjunctions_on_the_diagonal() {
    let c = london(JD_2024);
    let o = SynastryOptions {
        include_angles: true,
        ..SynastryOptions::default()
    };
    let s = synastry(&c, &c, &o).unwrap();
    assert_eq!(s.rows.len(), 15);
    assert_eq!(s.cols.len(), 15);
    assert_eq!(s.cells.len(), 225);
    for i in 0..s.rows.len() {
        let cell = s.cell(i, i).expect("diagonal aspect");
        assert_eq!(cell.aspect_type, AspectType::Conjunction);
        assert!(cell.orb_deg < TOL);
        assert!(cell.separation_deg < TOL);
        assert_eq!(cell.a, s.rows[i]);
        assert_eq!(cell.b, s.cols[i]);
    }
    // Out of range indexes are None, not a panic.
    assert!(s.cell(15, 0).is_none());
    assert!(s.cell(0, 15).is_none());
    assert!(s.cell(usize::MAX, usize::MAX).is_none());
}

#[test]
fn every_natal_aspect_appears_in_both_triangles() {
    let c = london(JD_2024);
    let s = synastry(&c, &c, &SynastryOptions::default()).unwrap();
    let positions: Vec<(String, f64, f64)> = c
        .bodies
        .iter()
        .map(|b| {
            (
                format!("{:?}", b.body),
                b.longitude_deg,
                b.speed_deg_per_day,
            )
        })
        .collect();
    let natal = find_all_aspects(&positions, AspectType::MAJOR);
    assert!(!natal.is_empty(), "fixture should contain natal aspects");
    for asp in &natal {
        let i = positions.iter().position(|p| p.0 == asp.body1).unwrap();
        let j = positions.iter().position(|p| p.0 == asp.body2).unwrap();
        for (r, k) in [(i, j), (j, i)] {
            let cell = s
                .cell(r, k)
                .unwrap_or_else(|| panic!("missing cell ({r},{k})"));
            assert_eq!(cell.aspect_type, asp.aspect_type);
            assert!((cell.orb_deg - asp.orb_deg).abs() < TOL);
        }
    }
    // The grid is symmetric for a chart against itself.
    for i in 0..s.rows.len() {
        for j in 0..s.cols.len() {
            let (x, y) = (s.cell(i, j), s.cell(j, i));
            assert_eq!(x.map(|a| a.aspect_type), y.map(|a| a.aspect_type));
        }
    }
}

#[test]
fn inter_aspects_hand_computed_cases() {
    let sun = |lon: f64| [(PointId::Body(Body::Sun), lon)];
    let moon = |lon: f64| [(PointId::Body(Body::Moon), lon)];
    let major = AspectType::MAJOR;

    // 10 and 130 are exactly a trine.
    let g = inter_aspects(&sun(10.0), &moon(130.0), major, 1.0);
    let hit = g[0].expect("trine");
    assert_eq!(hit.aspect_type, AspectType::Trine);
    assert!(hit.orb_deg < TOL);
    assert!((hit.separation_deg - 120.0).abs() < TOL);

    // 350 and 10 are 20 degrees apart: no aspect at the default orbs ...
    let g = inter_aspects(&sun(350.0), &moon(10.0), major, 1.0);
    assert!(g[0].is_none());
    // ... and a conjunction at three times the orb (24 degrees), orb 20.
    let g = inter_aspects(&sun(350.0), &moon(10.0), major, 3.0);
    let hit = g[0].expect("wide conjunction");
    assert_eq!(hit.aspect_type, AspectType::Conjunction);
    assert!((hit.separation_deg - 20.0).abs() < TOL);
    assert!((hit.orb_deg - 20.0).abs() < TOL);

    // The shorter arc is used across 0 degrees: 355 and 3 are a conjunction of orb 8 at 1.0.
    let g = inter_aspects(&sun(355.0), &moon(3.0), major, 1.0);
    assert!((g[0].unwrap().separation_deg - 8.0).abs() < TOL);

    // Bad numbers give no aspect rather than a panic.
    assert!(inter_aspects(&sun(f64::NAN), &moon(0.0), major, 1.0)[0].is_none());
    assert!(inter_aspects(&sun(0.0), &moon(0.0), major, 0.0)[0].is_none());
    assert!(inter_aspects(&sun(0.0), &moon(0.0), major, f64::NAN)[0].is_none());
    assert!(inter_aspects(&sun(0.0), &moon(0.0), &[], 1.0)[0].is_none());
    assert!(inter_aspects(&[], &moon(0.0), major, 1.0).is_empty());
}

#[test]
fn inter_aspects_keeps_the_tightest_when_orbs_overlap() {
    // At 20 times the orb every major aspect would match; the closest wins.
    let a = [(PointId::Body(Body::Sun), 0.0)];
    let b = [(PointId::Body(Body::Moon), 118.0)];
    let hit = inter_aspects(&a, &b, AspectType::MAJOR, 20.0)[0].unwrap();
    assert_eq!(hit.aspect_type, AspectType::Trine);
    assert!((hit.orb_deg - 2.0).abs() < TOL);
}

#[test]
fn house_overlay_with_equal_cusps() {
    let cusps = equal_cusps_literal();
    let pts = [
        (PointId::Body(Body::Sun), 95.0),
        (PointId::Body(Body::Moon), 359.9),
        (PointId::Body(Body::Mars), 10.0),
        (PointId::Body(Body::Venus), 125.0),
        (PointId::Body(Body::Saturn), -5.0),
        (PointId::Body(Body::Jupiter), f64::NAN),
    ];
    let o = house_overlay(&pts, &cusps);
    let houses: Vec<usize> = o.iter().map(|x| x.house).collect();
    // NaN is skipped; -5 is 355 which is house 12.
    assert_eq!(houses, vec![4, 12, 1, 5, 12]);
    assert_eq!(o[0].point, PointId::Body(Body::Sun));
    assert_eq!(o[0].longitude_deg, 95.0);
}

#[test]
fn synastry_overlays_use_the_other_charts_houses() {
    let a = london(JD_2024);
    let b = sydney(JD_2024 + 400.0);
    let s = synastry(&a, &b, &SynastryOptions::default()).unwrap();
    assert_eq!(s.b_in_a.len(), 11);
    assert_eq!(s.a_in_b.len(), 11);
    for ov in &s.b_in_a {
        assert_eq!(
            ov.house,
            a.houses.planet_in_house(ov.longitude_deg.to_radians())
        );
        assert!((1..=12).contains(&ov.house));
    }
    for ov in &s.a_in_b {
        assert_eq!(
            ov.house,
            b.houses.planet_in_house(ov.longitude_deg.to_radians())
        );
    }
}

#[test]
fn synastry_validates_options_and_charts() {
    let a = london(JD_2024);
    let bad = SynastryOptions {
        orb_multiplier: 0.0,
        ..SynastryOptions::default()
    };
    assert!(matches!(
        synastry(&a, &a, &bad),
        Err(RelationshipError::InvalidInput(_))
    ));
    let neg = SynastryOptions {
        orb_multiplier: -1.0,
        ..SynastryOptions::default()
    };
    assert!(synastry(&a, &a, &neg).is_err());
    let mut broken = a.clone();
    broken.bodies[3].longitude_deg = f64::NAN;
    assert!(matches!(
        synastry(&a, &broken, &SynastryOptions::default()),
        Err(RelationshipError::InvalidInput(_))
    ));
}

// ------------------------------------------------------------------ composite

#[test]
fn composite_sun_from_10_and_350_is_0_not_180() {
    let a = synthetic(JD_2024, 40.0, 10.0, &[(Body::Sun, 10.0)]);
    let b = synthetic(JD_2024, 40.0, 10.0, &[(Body::Sun, 350.0)]);
    let c = composite_chart(&a, &b, &CompositeOptions::default()).unwrap();
    assert!(sdiff(c.bodies[0].longitude_deg, 0.0).abs() < TOL);
    assert!(c.bodies[0].longitude_deg < 1.0 || c.bodies[0].longitude_deg > 359.0);
}

#[test]
fn composite_of_points_exactly_180_apart_depends_on_chart_order() {
    // Documented limit of compute_midpoint: the tie is broken towards lon_a + 90.
    let a = synthetic(JD_2024, 40.0, 10.0, &[(Body::Sun, 0.0)]);
    let b = synthetic(JD_2024, 40.0, 10.0, &[(Body::Sun, 180.0)]);
    let opts = CompositeOptions::default();
    let ab = composite_chart(&a, &b, &opts).unwrap();
    let ba = composite_chart(&b, &a, &opts).unwrap();
    assert!((ab.bodies[0].longitude_deg - 90.0).abs() < TOL);
    assert!((ba.bodies[0].longitude_deg - 270.0).abs() < TOL);
}

#[test]
fn composite_means_for_latitude_and_speed() {
    let mut a = synthetic(JD_2024, 40.0, 10.0, &[(Body::Mars, 100.0)]);
    let mut b = synthetic(JD_2024, 40.0, 10.0, &[(Body::Mars, 140.0)]);
    a.bodies[0].latitude_deg = 1.0;
    b.bodies[0].latitude_deg = 2.0;
    a.bodies[0].speed_deg_per_day = -0.5;
    b.bodies[0].speed_deg_per_day = 0.7;
    let c = composite_chart(&a, &b, &CompositeOptions::default()).unwrap();
    let m = c.bodies[0];
    assert!((m.longitude_deg - 120.0).abs() < TOL);
    assert!((m.latitude_deg - 1.5).abs() < TOL);
    assert!((m.speed_deg_per_day - 0.1).abs() < TOL);
}

#[test]
fn composite_uses_only_common_bodies_in_chart_a_order() {
    let a = synthetic(
        JD_2024,
        40.0,
        10.0,
        &[(Body::Mars, 10.0), (Body::Sun, 20.0), (Body::Moon, 30.0)],
    );
    let b = synthetic(
        JD_2024,
        40.0,
        10.0,
        &[(Body::Moon, 50.0), (Body::Mars, 30.0)],
    );
    let c = composite_chart(&a, &b, &CompositeOptions::default()).unwrap();
    let order: Vec<Body> = c.bodies.iter().map(|x| x.body).collect();
    assert_eq!(order, vec![Body::Mars, Body::Moon]);
    let only_sun = synthetic(JD_2024, 40.0, 10.0, &[(Body::Sun, 1.0)]);
    let only_moon = synthetic(JD_2024, 40.0, 10.0, &[(Body::Moon, 1.0)]);
    assert!(matches!(
        composite_chart(&only_sun, &only_moon, &CompositeOptions::default()),
        Err(RelationshipError::NoCommonPoints)
    ));
}

#[test]
fn composite_of_identical_charts_reproduces_the_natal_chart() {
    let a = london(JD_2024);
    let c = composite_chart(&a, &a, &CompositeOptions::default()).unwrap();
    assert_eq!(c.bodies.len(), a.bodies.len());
    for (x, y) in c.bodies.iter().zip(&a.bodies) {
        assert_eq!(x.body, y.body);
        assert!(sdiff(x.longitude_deg, y.longitude_deg).abs() < TOL);
        assert!((x.speed_deg_per_day - y.speed_deg_per_day).abs() < TOL);
    }
    for i in 0..12 {
        let d = sdiff(c.houses.cusp_deg(i), a.houses.cusp_deg(i)).abs();
        assert!(d < 1e-8, "cusp {} differs by {d}", i + 1);
    }
    assert!(
        sdiff(
            c.houses.ascendant.to_degrees(),
            a.houses.ascendant.to_degrees()
        )
        .abs()
            < 1e-8
    );
    assert!(sdiff(c.houses.mc.to_degrees(), a.houses.mc.to_degrees()).abs() < 1e-8);
    assert!((c.reference_location.lat_deg() - a.location.lat_deg()).abs() < TOL);
    assert!(sdiff(c.reference_location.lon_deg(), a.location.lon_deg()).abs() < TOL);
    assert_eq!(c.mean_jd_ut1, a.jd_ut1);
    assert_eq!(c.method, CompositeAscendant::DerivedFromMidpointMc);
}

#[test]
fn composite_mc_is_the_midpoint_of_the_two_mcs() {
    let a = london(JD_2024);
    let b = sydney(JD_2024 + 4000.0);
    let c = composite_chart(&a, &b, &CompositeOptions::default()).unwrap();
    let mc_a = a.houses.mc.to_degrees();
    let mc_b = b.houses.mc.to_degrees();
    let d = sdiff(mc_b, mc_a); // shorter signed arc from A to B
    assert!(d.abs() < 179.0, "fixture must avoid the 180 degree tie");
    let expected = (mc_a + d / 2.0).rem_euclid(360.0);
    assert!(
        sdiff(c.houses.mc.to_degrees(), expected).abs() < TOL,
        "composite MC {} expected {expected}",
        c.houses.mc.to_degrees()
    );
    // The IC is opposite, the Ascendant is a valid longitude, mean place is the mean.
    assert!(sdiff(c.houses.ic.to_degrees(), expected + 180.0).abs() < TOL);
    assert!((c.reference_location.lat_deg() - (51.5074 - 33.8688) / 2.0).abs() < TOL);
    assert!((c.mean_jd_ut1 - (a.jd_ut1 + b.jd_ut1) / 2.0).abs() < 1e-9);
    assert!(c.houses.ascendant.is_finite());
}

#[test]
fn composite_midpoint_of_ascendants() {
    let a = london(JD_2024);
    let b = chart(JD_2024 + 0.05, 48.8566, 2.3522, HouseSystem::Placidus); // Paris, an hour later
    let asc_a = a.houses.ascendant.to_degrees();
    let asc_b = b.houses.ascendant.to_degrees();
    let d = sdiff(asc_b, asc_a);
    assert!(d.abs() < 179.0);
    let expected = (asc_a + d / 2.0).rem_euclid(360.0);

    let eq = CompositeOptions {
        house_system: HouseSystem::Equal,
        ascendant: CompositeAscendant::MidpointOfAscendants,
    };
    let c = composite_chart(&a, &b, &eq).unwrap();
    assert_eq!(c.method, CompositeAscendant::MidpointOfAscendants);
    assert!(sdiff(c.houses.ascendant.to_degrees(), expected).abs() < TOL);
    assert!(sdiff(c.houses.descendant.to_degrees(), expected + 180.0).abs() < TOL);
    for i in 0..12 {
        let want = expected + 30.0 * i as f64;
        assert!(sdiff(c.houses.cusp_deg(i), want).abs() < TOL);
    }
    // The MC is still the midpoint of the MCs, not tied to this Ascendant.
    let derived = composite_chart(&a, &b, &CompositeOptions::default()).unwrap();
    assert!(sdiff(c.houses.mc.to_degrees(), derived.houses.mc.to_degrees()).abs() < TOL);

    let ws = CompositeOptions {
        house_system: HouseSystem::WholeSign,
        ascendant: CompositeAscendant::MidpointOfAscendants,
    };
    let c = composite_chart(&a, &b, &ws).unwrap();
    let sign_start = (expected / 30.0).floor() * 30.0;
    for i in 0..12 {
        let want = sign_start + 30.0 * i as f64;
        assert!(sdiff(c.houses.cusp_deg(i), want).abs() < TOL);
    }
    assert!(sdiff(c.houses.ascendant.to_degrees(), expected).abs() < TOL);

    let placidus = CompositeOptions {
        house_system: HouseSystem::Placidus,
        ascendant: CompositeAscendant::MidpointOfAscendants,
    };
    assert!(matches!(
        composite_chart(&a, &b, &placidus),
        Err(RelationshipError::UnsupportedHouseSystem(
            HouseSystem::Placidus
        ))
    ));
}

#[test]
fn composite_rejects_non_finite_charts() {
    let a = london(JD_2024);
    let mut b = a.clone();
    b.jd_ut1 = f64::NAN;
    assert!(matches!(
        composite_chart(&a, &b, &CompositeOptions::default()),
        Err(RelationshipError::InvalidInput(_))
    ));
}

// -------------------------------------------------------------------- Davison

fn birth(jd: f64, lat: f64, lon: f64) -> BirthMoment {
    BirthMoment {
        jd_ut1: jd,
        location: GeoLocation::new(lat, lon),
    }
}

#[test]
fn davison_moment_and_place_are_exact_means() {
    let a = birth(2_458_000.5, 10.0, 170.0);
    let b = birth(2_458_100.5, 30.0, -170.0);
    let d = davison_chart(almanac(), &a, &b, &DavisonOptions::default()).unwrap();
    assert_eq!(d.jd_ut1, 2_458_050.5);
    assert!((d.location.lat_deg() - 20.0).abs() < TOL);
    assert!((d.location.lon_deg().abs() - 180.0).abs() < TOL);
    assert_eq!(d.bodies.len(), 11);
}

#[test]
fn davison_of_identical_births_equals_a_natal_chart() {
    let a = birth(JD_2024, 51.5074, -0.1278);
    let d = davison_chart(almanac(), &a, &a, &DavisonOptions::default()).unwrap();
    let n = london(JD_2024);
    assert_eq!(d.jd_ut1, n.jd_ut1);
    assert_eq!(d.bodies, n.bodies);
    for i in 0..12 {
        assert!(sdiff(d.houses.cusp_deg(i), n.houses.cusp_deg(i)).abs() < 1e-8);
    }
    assert!(sdiff(d.houses.mc.to_degrees(), n.houses.mc.to_degrees()).abs() < 1e-8);
}

#[test]
fn davison_sun_lies_between_the_natal_suns() {
    let (jd_a, jd_b) = (2_458_000.5, 2_458_120.5); // 120 days apart
    let a = birth(jd_a, 40.0, -74.0);
    let b = birth(jd_b, 48.0, 2.0);
    let opts = DavisonOptions {
        bodies: vec![Body::Sun],
        ..DavisonOptions::default()
    };
    let d = davison_chart(almanac(), &a, &b, &opts).unwrap();
    let sun = |jd: f64| {
        almanac()
            .geocentric_longitude_deg(Body::Sun, JdUT1(jd))
            .unwrap()
    };
    let (sa, sb) = (sun(jd_a), sun(jd_b));
    let span = sdiff(sb, sa);
    assert!(
        span > 100.0 && span < 179.0,
        "Sun moves ~0.98 deg/day, span {span}"
    );
    let mid = sdiff(d.bodies[0].longitude_deg, sa);
    assert!(mid > 0.0 && mid < span, "mid {mid} within (0, {span})");
    // The Sun is nearly uniform: the Davison Sun is close to the arithmetic middle.
    assert!((mid - span / 2.0).abs() < 3.0);
}

#[test]
fn davison_great_circle_midpoint() {
    let a = birth(JD_2024, 0.0, 0.0);
    let b = birth(JD_2024, 0.0, 90.0);
    let opts = DavisonOptions {
        bodies: vec![Body::Sun],
        space: SpaceMidpoint::GreatCircle,
        ..DavisonOptions::default()
    };
    let d = davison_chart(almanac(), &a, &b, &opts).unwrap();
    assert!(d.location.lat_deg().abs() < TOL);
    assert!((d.location.lon_deg() - 45.0).abs() < TOL);

    // At latitude 60 the great circle bows poleward of the arithmetic mean (60).
    let a = birth(JD_2024, 60.0, 0.0);
    let b = birth(JD_2024, 60.0, 90.0);
    let d = davison_chart(almanac(), &a, &b, &opts).unwrap();
    assert!(d.location.lat_deg() > 65.0 && d.location.lat_deg() < 70.0);
    assert!((d.location.lon_deg() - 45.0).abs() < TOL);

    // Antipodal places have no unique midpoint.
    let a = birth(JD_2024, 0.0, 0.0);
    let b = birth(JD_2024, 0.0, 180.0);
    assert!(matches!(
        davison_chart(almanac(), &a, &b, &opts),
        Err(RelationshipError::InvalidInput(_))
    ));
}

#[test]
fn davison_rejects_bad_input() {
    let ok = birth(JD_2024, 10.0, 10.0);
    let nan = birth(f64::NAN, 10.0, 10.0);
    let polar = BirthMoment {
        jd_ut1: JD_2024,
        location: GeoLocation::new(95.0, 0.0),
    };
    let o = DavisonOptions::default();
    assert!(davison_chart(almanac(), &ok, &nan, &o).is_err());
    assert!(davison_chart(almanac(), &polar, &ok, &o).is_err());
    let earth = DavisonOptions {
        bodies: vec![Body::Earth],
        ..DavisonOptions::default()
    };
    assert!(davison_chart(almanac(), &ok, &ok, &earth).is_err());
}

#[test]
fn davison_chart_feeds_synastry() {
    let a = london(JD_2024);
    let d = davison_chart(
        almanac(),
        &birth(JD_2024, 51.5074, -0.1278),
        &birth(JD_2024 + 200.0, -33.8688, 151.2093),
        &DavisonOptions::default(),
    )
    .unwrap();
    let s = synastry(&a, &d, &SynastryOptions::default()).unwrap();
    assert_eq!(s.cells.len(), 121);
}
