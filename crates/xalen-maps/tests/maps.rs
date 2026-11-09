//! Tests for the astrocartography lines.
//!
//! Hand-computed cases first, then structural invariants, then a
//! self-consistency check against the independent topocentric altitude and
//! hour-angle code in `xalen-ephem`.

use xalen_coords::gmst_deg;
use xalen_ephem::{Almanac, Body};
use xalen_maps::{
    HorizonCrossing, LatLon, LineKind, MapError, MapLine, MapOptions, astro_map, horizon_half_arc,
    horizon_lines, ic_longitude_deg, mc_longitude_deg,
};
use xalen_time::JdUT1;

const JD_2024: f64 = 2_460_482.5; // 2024-06-21 00:00 UT

fn sdiff(a: f64, b: f64) -> f64 {
    (a - b + 180.0).rem_euclid(360.0) - 180.0
}

fn line(body: &xalen_maps::BodyLines, kind: LineKind) -> &MapLine {
    body.lines.iter().find(|l| l.kind == kind).unwrap()
}

// ------------------------------------------------------------------ meridians

#[test]
fn mc_and_ic_hand_values() {
    // RA 100, GST 40: MC at 60, IC at -120.
    assert!((mc_longitude_deg(100.0, 40.0) - 60.0).abs() < 1e-12);
    assert!((ic_longitude_deg(100.0, 40.0) + 120.0).abs() < 1e-12);
    // RA 300, GST 20: MC -80, IC 100.
    assert!((mc_longitude_deg(300.0, 20.0) + 80.0).abs() < 1e-12);
    assert!((ic_longitude_deg(300.0, 20.0) - 100.0).abs() < 1e-12);
    // Wrap: RA 10, GST 300 gives 70.
    assert!((mc_longitude_deg(10.0, 300.0) - 70.0).abs() < 1e-12);
    // The result is always in [-180, 180).
    for ra in [0.0, 90.0, 179.9, 180.0, 270.0, 359.9] {
        for gst in [0.0, 45.0, 180.0, 359.0] {
            for f in [mc_longitude_deg, ic_longitude_deg] {
                let l = f(ra, gst);
                assert!((-180.0..180.0).contains(&l), "{l}");
            }
        }
    }
}

#[test]
fn body_at_greenwich_sidereal_time_is_on_the_greenwich_meridian() {
    // At J2000 GMST is 280.46061837 degrees; a body with that RA transits at longitude 0.
    let gmst = gmst_deg(2_451_545.0);
    assert!((gmst - 280.460_618_37).abs() < 1e-6);
    assert!(mc_longitude_deg(280.460_618_37, gmst).abs() < 1e-6);
}

// ----------------------------------------------------------------- half arc

#[test]
fn half_arc_values() {
    match horizon_half_arc(45.0, 23.4393, 0.0) {
        HorizonCrossing::Crosses { half_arc_deg } => {
            assert!((half_arc_deg - 115.70).abs() < 0.02, "{half_arc_deg}");
            // cos H0 = -tan 45 tan 23.4393
            let expected = (-(23.4393_f64.to_radians().tan())).acos().to_degrees();
            assert!((half_arc_deg - expected).abs() < 1e-9);
        }
        other => panic!("expected a crossing, got {other:?}"),
    }
    // Equator, any declination within range: 90 degrees at dec 0.
    match horizon_half_arc(0.0, 0.0, 0.0) {
        HorizonCrossing::Crosses { half_arc_deg } => assert!((half_arc_deg - 90.0).abs() < 1e-9),
        other => panic!("{other:?}"),
    }
    // A lower horizon (refraction, -0.5667) lengthens the arc.
    let base = match horizon_half_arc(45.0, 23.4393, 0.0) {
        HorizonCrossing::Crosses { half_arc_deg } => half_arc_deg,
        _ => unreachable!(),
    };
    match horizon_half_arc(45.0, 23.4393, -0.5667) {
        HorizonCrossing::Crosses { half_arc_deg } => assert!(half_arc_deg > base),
        other => panic!("{other:?}"),
    }
}

#[test]
fn half_arc_circumpolar_and_never_rising() {
    assert_eq!(
        horizon_half_arc(80.0, 20.0, 0.0),
        HorizonCrossing::AlwaysAbove
    );
    assert_eq!(
        horizon_half_arc(-80.0, 20.0, 0.0),
        HorizonCrossing::AlwaysBelow
    );
    assert_eq!(
        horizon_half_arc(-80.0, -20.0, 0.0),
        HorizonCrossing::AlwaysAbove
    );
    // Limits: dec 30 is circumpolar from latitude 60 north.
    assert!(matches!(
        horizon_half_arc(59.9, 30.0, 0.0),
        HorizonCrossing::Crosses { .. }
    ));
    assert_eq!(
        horizon_half_arc(60.1, 30.0, 0.0),
        HorizonCrossing::AlwaysAbove
    );
    // The poles and non-finite input do not panic.
    for lat in [90.0, -90.0] {
        let _ = horizon_half_arc(lat, 10.0, 0.0);
        let _ = horizon_half_arc(lat, 90.0, 0.0);
    }
    assert_eq!(
        horizon_half_arc(f64::NAN, 10.0, 0.0),
        HorizonCrossing::AlwaysBelow
    );
}

// ------------------------------------------------------------ horizon lines

fn all_points(segs: &[Vec<LatLon>]) -> impl Iterator<Item = &LatLon> {
    segs.iter().flatten()
}

#[test]
fn equatorial_body_hand_values() {
    // RA 100, GST 40, dec 0: H0 = 90 at every latitude, so ASC -30 and DSC 150.
    let (asc, dsc) = horizon_lines(100.0, 0.0, 40.0, &MapOptions::default()).unwrap();
    assert_eq!(asc.len(), 1);
    assert_eq!(dsc.len(), 1);
    assert_eq!(asc[0].len(), 133);
    for &(_, lon) in all_points(&asc) {
        assert!((lon + 30.0).abs() < 1e-9);
    }
    for &(_, lon) in all_points(&dsc) {
        assert!((lon - 150.0).abs() < 1e-9);
    }
    // Latitudes run south to north over the requested range.
    assert_eq!(asc[0].first().unwrap().0, -66.0);
    assert_eq!(asc[0].last().unwrap().0, 66.0);
    // Check the latitude 0 point explicitly.
    let p0 = asc[0].iter().find(|p| p.0 == 0.0).unwrap();
    assert!((p0.1 + 30.0).abs() < 1e-9);
}

#[test]
fn limit_vertices_meet_the_meridian_lines() {
    // Declination 30: limit latitude 60. RA 100, GST 40 -> MC at 60, IC at -120.
    let (asc, dsc) = horizon_lines(100.0, 30.0, 40.0, &MapOptions::default()).unwrap();
    let mc = mc_longitude_deg(100.0, 40.0);
    let ic = ic_longitude_deg(100.0, 40.0);

    assert_eq!(asc.len(), 1, "ASC is one piece");
    let first = asc[0].first().unwrap();
    let last = asc[0].last().unwrap();
    assert!(
        (first.0 + 60.0).abs() < 1e-8,
        "south vertex lat {}",
        first.0
    );
    assert!(
        sdiff(first.1, mc).abs() < 1e-6,
        "south vertex meets the MC line"
    );
    assert!((last.0 - 60.0).abs() < 1e-8, "north vertex lat {}", last.0);
    assert!(
        sdiff(last.1, ic).abs() < 1e-6,
        "north vertex meets the IC line"
    );

    // The DSC line has the same limits and crosses the antimeridian once.
    let dfirst = dsc.first().unwrap().first().unwrap();
    let dlast = dsc.last().unwrap().last().unwrap();
    assert!((dfirst.0 + 60.0).abs() < 1e-8 && sdiff(dfirst.1, mc).abs() < 1e-6);
    assert!((dlast.0 - 60.0).abs() < 1e-8 && sdiff(dlast.1, ic).abs() < 1e-6);
    assert_eq!(dsc.len(), 2, "split at the antimeridian");
    let end_of_first = dsc[0].last().unwrap();
    let start_of_second = dsc[1].first().unwrap();
    assert!((end_of_first.1 - 180.0).abs() < 1e-9);
    assert!((start_of_second.1 + 180.0).abs() < 1e-9);
    assert!((end_of_first.0 - start_of_second.0).abs() < 1e-9);

    // No emitted point is beyond the limit latitude.
    for &(lat, _) in all_points(&asc).chain(all_points(&dsc)) {
        assert!(lat.abs() <= 60.0 + 1e-8);
    }
}

#[test]
fn negative_declination_swaps_the_limits() {
    // Declination -30: south limit -60 meets the IC line, north limit 60 meets the MC line.
    let (asc, _) = horizon_lines(100.0, -30.0, 40.0, &MapOptions::default()).unwrap();
    let mc = mc_longitude_deg(100.0, 40.0);
    let ic = ic_longitude_deg(100.0, 40.0);
    let first = asc[0].first().unwrap();
    let last = asc[0].last().unwrap();
    assert!((first.0 + 60.0).abs() < 1e-8 && sdiff(first.1, ic).abs() < 1e-6);
    assert!((last.0 - 60.0).abs() < 1e-8 && sdiff(last.1, mc).abs() < 1e-6);
}

#[test]
fn circumpolar_body_breaks_the_line() {
    // Declination 80: crosses the horizon only within 10 degrees of the equator.
    let (asc, dsc) = horizon_lines(0.0, 80.0, 0.0, &MapOptions::default()).unwrap();
    for segs in [&asc, &dsc] {
        // The DSC line reaches the antimeridian at its northern end and is split there.
        assert!((1..=2).contains(&segs.len()));
        for &(lat, _) in all_points(segs) {
            assert!(lat.abs() <= 10.0 + 1e-8);
            assert!(matches!(
                horizon_half_arc(lat, 80.0, 0.0),
                HorizonCrossing::Crosses { .. }
            ));
        }
        assert!((segs[0].first().unwrap().0 + 10.0).abs() < 1e-8);
        assert!((segs.last().unwrap().last().unwrap().0 - 10.0).abs() < 1e-8);
    }
    // A range wholly inside the circumpolar zone has no horizon line at all.
    let o = MapOptions {
        lat_min_deg: 20.0,
        lat_max_deg: 60.0,
        ..MapOptions::default()
    };
    let (asc, dsc) = horizon_lines(0.0, 80.0, 0.0, &o).unwrap();
    assert!(asc.is_empty() && dsc.is_empty());
}

#[test]
fn segments_stay_on_the_map_for_every_orientation() {
    let o = MapOptions {
        lat_step_deg: 2.0,
        ..MapOptions::default()
    };
    for ra in (0..360).step_by(30) {
        for gst in (0..360).step_by(70) {
            for dec in [-60.0, -23.4, 0.0, 10.0, 23.4, 45.0, 70.0] {
                let (asc, dsc) = horizon_lines(ra as f64, dec, gst as f64, &o).unwrap();
                for seg in asc.iter().chain(dsc.iter()) {
                    assert!(seg.len() >= 2);
                    for w in seg.windows(2) {
                        assert!(w[1].0 >= w[0].0 - 1e-9, "latitude must not decrease");
                        assert!(
                            (w[1].1 - w[0].1).abs() <= 180.0,
                            "ra {ra} gst {gst} dec {dec}: jump {:?} -> {:?}",
                            w[0],
                            w[1]
                        );
                    }
                    for &(lat, lon) in seg {
                        assert!((-66.0 - 1e-9..=66.0 + 1e-9).contains(&lat));
                        assert!((-180.0..=180.0).contains(&lon), "lon {lon}");
                    }
                }
            }
        }
    }
}

#[test]
fn the_northern_limit_is_always_sampled() {
    // Step 10 over -66..66 does not land on 66; the last point must still be 66.
    let o = MapOptions {
        lat_step_deg: 10.0,
        ..MapOptions::default()
    };
    let (asc, _) = horizon_lines(100.0, 0.0, 40.0, &o).unwrap();
    assert_eq!(asc[0].last().unwrap().0, 66.0);
    assert_eq!(asc[0].first().unwrap().0, -66.0);
}

#[test]
fn horizon_altitude_shifts_the_lines() {
    let flat = horizon_lines(100.0, 0.0, 40.0, &MapOptions::default()).unwrap();
    let o = MapOptions {
        horizon_altitude_deg: -0.5667,
        ..MapOptions::default()
    };
    let low = horizon_lines(100.0, 0.0, 40.0, &o).unwrap();
    // At the equator, h0 = -0.5667 lengthens the arc by about 0.5667 degrees.
    let eq = |segs: &[Vec<LatLon>]| segs[0].iter().find(|p| p.0 == 0.0).unwrap().1;
    assert!((eq(&flat.0) - eq(&low.0) - 0.5667).abs() < 1e-3);
}

#[test]
fn bad_options_are_errors_not_panics() {
    let ok = MapOptions::default();
    let bad: Vec<MapOptions> = vec![
        MapOptions {
            lat_step_deg: 0.0,
            ..ok
        },
        MapOptions {
            lat_step_deg: -1.0,
            ..ok
        },
        MapOptions {
            lat_step_deg: f64::NAN,
            ..ok
        },
        MapOptions {
            lat_min_deg: f64::NEG_INFINITY,
            ..ok
        },
        MapOptions {
            lat_max_deg: 90.0,
            ..ok
        },
        MapOptions {
            lat_min_deg: -90.0,
            ..ok
        },
        MapOptions {
            lat_min_deg: 10.0,
            lat_max_deg: 10.0,
            ..ok
        },
        MapOptions {
            lat_min_deg: 20.0,
            lat_max_deg: 10.0,
            ..ok
        },
        MapOptions {
            lat_step_deg: 0.001,
            ..ok
        },
        MapOptions {
            max_points: 1,
            ..ok
        },
        MapOptions {
            horizon_altitude_deg: 90.0,
            ..ok
        },
        MapOptions {
            horizon_altitude_deg: f64::NAN,
            ..ok
        },
    ];
    for (i, o) in bad.iter().enumerate() {
        assert!(
            matches!(
                horizon_lines(100.0, 10.0, 40.0, o),
                Err(MapError::InvalidInput(_))
            ),
            "case {i}"
        );
    }
    for (ra, dec, gst) in [
        (f64::NAN, 0.0, 0.0),
        (0.0, f64::NAN, 0.0),
        (0.0, 0.0, f64::INFINITY),
        (0.0, 91.0, 0.0),
        (0.0, -90.5, 0.0),
    ] {
        assert!(matches!(
            horizon_lines(ra, dec, gst, &ok),
            Err(MapError::InvalidInput(_))
        ));
    }
    // Declination exactly at a pole is allowed.
    assert!(horizon_lines(0.0, 90.0, 0.0, &ok).is_ok());
}

// --------------------------------------------------- full map and consistency

fn almanac() -> Almanac {
    Almanac::default_vedic()
}

#[test]
fn astro_map_has_four_lines_and_a_zenith_per_body() {
    let a = almanac();
    let bodies = [Body::Sun, Body::Mars, Body::Jupiter];
    let map = astro_map(&a, JdUT1(JD_2024), &bodies, &MapOptions::default()).unwrap();
    assert_eq!(map.bodies.len(), 3);
    assert_eq!(map.jd_ut1, JD_2024);
    assert!((0.0..360.0).contains(&map.gast_deg));
    for (b, want) in map.bodies.iter().zip(bodies) {
        assert_eq!(b.body, want);
        let kinds: Vec<LineKind> = b.lines.iter().map(|l| l.kind).collect();
        assert_eq!(
            kinds,
            vec![LineKind::Mc, LineKind::Ic, LineKind::Asc, LineKind::Dsc]
        );
        let mc = &line(b, LineKind::Mc).segments;
        assert_eq!(mc.len(), 1);
        assert_eq!(mc[0], vec![(-66.0, b.zenith.1), (66.0, b.zenith.1)]);
        assert_eq!(b.zenith.0, b.dec_deg);
        let ic = &line(b, LineKind::Ic).segments[0];
        assert!(sdiff(ic[0].1, b.zenith.1 + 180.0).abs() < 1e-9);
    }
    // The Sun at the June solstice: declination about +23.4, RA about 90.
    let sun = &map.bodies[0];
    assert!((sun.dec_deg - 23.44).abs() < 0.05, "{}", sun.dec_deg);
    assert!((sun.ra_deg - 90.0).abs() < 1.0, "{}", sun.ra_deg);
}

#[test]
fn astro_map_rejects_bad_input() {
    let a = almanac();
    let o = MapOptions::default();
    assert!(matches!(
        astro_map(&a, JdUT1(f64::NAN), &[Body::Sun], &o),
        Err(MapError::InvalidInput(_))
    ));
    assert!(matches!(
        astro_map(&a, JdUT1(JD_2024), &[Body::Earth], &o),
        Err(MapError::InvalidInput(_))
    ));
    let bad = MapOptions {
        lat_step_deg: 0.0,
        ..o
    };
    assert!(astro_map(&a, JdUT1(JD_2024), &[Body::Sun], &bad).is_err());
    // Pluto is outside its analytic series in antiquity: the ephemeris error is propagated.
    assert!(matches!(
        astro_map(&a, JdUT1(2_300_000.0), &[Body::Pluto], &o),
        Err(MapError::Ephemeris(_))
    ));
    // An empty body list is a valid, empty map.
    assert!(
        astro_map(&a, JdUT1(JD_2024), &[], &o)
            .unwrap()
            .bodies
            .is_empty()
    );
}

/// The independent check. `Almanac::topocentric_altitude_deg` and
/// `topocentric_hour_angle_deg` start from the same geocentric position but
/// apply their own parallax, sidereal time and equatorial conversion. At every
/// sampled vertex of a body's ASC or DSC line the altitude must be zero and the
/// hour angle must be -H0 (rising) or +H0 (setting); on the MC line the hour
/// angle must be zero and on the IC line 180.
///
/// The Moon is left out on purpose: its diurnal parallax (up to about 1 degree)
/// is not part of this geocentric map, so its topocentric altitude at a
/// geocentric horizon point is off by that much by construction.
#[test]
fn lines_agree_with_topocentric_altitude_and_hour_angle() {
    let a = almanac();
    let jd = JdUT1(JD_2024);
    let o = MapOptions::default();
    let map = astro_map(&a, jd, &[Body::Sun, Body::Mars, Body::Jupiter], &o).unwrap();
    let mut checked = 0;
    for b in &map.bodies {
        for kind in [LineKind::Asc, LineKind::Dsc] {
            for seg in &line(b, kind).segments {
                for &(lat, lon) in seg {
                    let alt = a
                        .topocentric_altitude_deg(b.body, jd, lat, lon, 0.0)
                        .unwrap();
                    assert!(
                        alt.abs() < 0.01,
                        "{:?} {kind:?} at ({lat},{lon}): altitude {alt}",
                        b.body
                    );
                    let ha = a
                        .topocentric_hour_angle_deg(b.body, jd, lat, lon, 0.0)
                        .unwrap();
                    let h0 = match horizon_half_arc(lat, b.dec_deg, 0.0) {
                        HorizonCrossing::Crosses { half_arc_deg } => half_arc_deg,
                        other => panic!("({lat},{lon}) is not on a horizon crossing: {other:?}"),
                    };
                    let want = if kind == LineKind::Asc { -h0 } else { h0 };
                    assert!(
                        sdiff(ha, want).abs() < 0.01,
                        "{:?} {kind:?} at ({lat},{lon}): hour angle {ha}, wanted {want}",
                        b.body
                    );
                    checked += 1;
                }
            }
        }
        for (kind, want) in [(LineKind::Mc, 0.0), (LineKind::Ic, 180.0)] {
            for &(lat, lon) in &line(b, kind).segments[0] {
                let ha = a
                    .topocentric_hour_angle_deg(b.body, jd, lat, lon, 0.0)
                    .unwrap();
                assert!(
                    sdiff(ha, want).abs() < 0.01,
                    "{:?} {kind:?} at ({lat},{lon}): hour angle {ha}",
                    b.body
                );
                checked += 1;
            }
        }
    }
    assert!(checked > 500, "only {checked} vertices were checked");
}

#[test]
fn zenith_point_has_the_body_overhead() {
    let a = almanac();
    let jd = JdUT1(JD_2024);
    let map = astro_map(
        &a,
        jd,
        &[Body::Sun, Body::Mars, Body::Jupiter],
        &MapOptions::default(),
    )
    .unwrap();
    for b in &map.bodies {
        let (lat, lon) = b.zenith;
        let alt = a
            .topocentric_altitude_deg(b.body, jd, lat, lon, 0.0)
            .unwrap();
        assert!(
            alt > 89.99,
            "{:?} altitude at its zenith point: {alt}",
            b.body
        );
    }
}

#[test]
fn a_lowered_horizon_matches_the_altitude_of_the_sun() {
    // Standard rising of the Sun's centre: h0 = -0.5667 degrees (geometric altitude).
    let a = almanac();
    let jd = JdUT1(JD_2024);
    let o = MapOptions {
        horizon_altitude_deg: -0.5667,
        lat_step_deg: 3.0,
        ..MapOptions::default()
    };
    let map = astro_map(&a, jd, &[Body::Sun], &o).unwrap();
    let sun = &map.bodies[0];
    let mut n = 0;
    for kind in [LineKind::Asc, LineKind::Dsc] {
        for seg in &line(sun, kind).segments {
            for &(lat, lon) in seg {
                let alt = a
                    .topocentric_altitude_deg(Body::Sun, jd, lat, lon, 0.0)
                    .unwrap();
                assert!((alt + 0.5667).abs() < 0.01, "({lat},{lon}) altitude {alt}");
                n += 1;
            }
        }
    }
    assert!(n > 40);
}
