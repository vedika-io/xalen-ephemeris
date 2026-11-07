//! Tests for parans (crossings of two bodies' lines).
//!
//! Hand-computed crossings first, then structure (pair order, symmetry,
//! latitude limits, skipped pairs), then a self-consistency check against the
//! independent topocentric altitude and hour-angle code in `xalen-ephem`.

use xalen_ephem::{Almanac, Body};
use xalen_maps::{
    AstroMap, BodyLines, LatLon, LineKind, MapError, MapOptions, Paran, ParanOptions, ParanSet,
    SkipReason, astro_map, parans, parans_near,
};
use xalen_time::JdUT1;

const JD_2024: f64 = 2_460_482.5; // 2024-06-21 00:00 UT

fn sdiff(a: f64, b: f64) -> f64 {
    (a - b + 180.0).rem_euclid(360.0) - 180.0
}

fn is_meridian(kind: LineKind) -> bool {
    matches!(kind, LineKind::Mc | LineKind::Ic)
}

/// A map built by hand from right ascensions and declinations.
fn sky_map(gast_deg: f64, bodies: &[(Body, f64, f64)]) -> AstroMap {
    AstroMap {
        jd_ut1: JD_2024,
        gast_deg,
        bodies: bodies
            .iter()
            .map(|&(body, ra_deg, dec_deg)| BodyLines {
                body,
                ra_deg,
                dec_deg,
                zenith: (dec_deg, sdiff(ra_deg - gast_deg, 0.0)),
                lines: Vec::new(),
            })
            .collect(),
    }
}

fn body_of(map: &AstroMap, body: Body) -> &BodyLines {
    map.bodies.iter().find(|b| b.body == body).unwrap()
}

/// Geocentric altitude from the map's own right ascension, declination and
/// sidereal time.
fn geo_altitude(map: &AstroMap, body: Body, lat: f64, lon: f64) -> f64 {
    let b = body_of(map, body);
    let h = (map.gast_deg + lon - b.ra_deg).to_radians();
    let (phi, dec) = (lat.to_radians(), b.dec_deg.to_radians());
    (phi.sin() * dec.sin() + phi.cos() * dec.cos() * h.cos())
        .asin()
        .to_degrees()
}

fn geo_hour_angle(map: &AstroMap, body: Body, lon: f64) -> f64 {
    sdiff(map.gast_deg + lon - body_of(map, body).ra_deg, 0.0)
}

fn real_map(bodies: &[Body]) -> AstroMap {
    astro_map(
        &Almanac::default_vedic(),
        JdUT1(JD_2024),
        bodies,
        &MapOptions::default(),
    )
    .unwrap()
}

const FOUR: [Body; 4] = [Body::Sun, Body::Moon, Body::Mars, Body::Jupiter];

// ------------------------------------------------------------ hand values

/// Sun at RA 0, dec +20 and Mars at RA 60, dec -10, sidereal time 0. The MC of
/// Mars is longitude 60, where the Sun's hour angle is +60 (setting); with
/// `h0 = 0` the crossing is at `tan φ = -cos H / tan δ`.
#[test]
fn horizon_meridian_hand_values() {
    let map = sky_map(0.0, &[(Body::Sun, 0.0, 20.0), (Body::Mars, 60.0, -10.0)]);
    let o = ParanOptions {
        include_horizon_pairs: false,
        ..ParanOptions::default()
    };
    let set = parans(&map, &o).unwrap();
    let lat = |cos_h: f64, dec: f64| (-cos_h / dec.to_radians().tan()).atan().to_degrees();
    let expected = [
        (LineKind::Dsc, LineKind::Mc, lat(0.5, 20.0), 60.0),
        (LineKind::Asc, LineKind::Ic, lat(-0.5, 20.0), -120.0),
    ];
    assert_eq!(set.parans.len(), 2, "{:?}", set.parans);
    // Ordered by latitude: -53.95 then +53.95.
    for (p, (ka, kb, la, lo)) in set.parans.iter().zip(expected) {
        assert_eq!(
            (p.body_a, p.kind_a, p.body_b, p.kind_b),
            (Body::Sun, ka, Body::Mars, kb)
        );
        assert!((p.latitude_deg - la).abs() < 1e-9, "{p:?} vs {la}");
        assert!((p.longitude_deg - lo).abs() < 1e-9, "{p:?} vs {lo}");
    }
    assert!((set.parans[0].latitude_deg + 53.95).abs() < 1e-2);

    // Mars's horizon meets the Sun's MC (longitude 0, Mars hour angle -60) at
    // +70.57 and its IC (hour angle +120) at -70.57: outside -66..66, so
    // skipped with the place given.
    let mars_lat = lat(0.5, -10.0);
    assert!((mars_lat - 70.57).abs() < 0.01, "{mars_lat}");
    assert_eq!(set.skipped.len(), 2, "{:?}", set.skipped);
    for s in &set.skipped {
        assert_eq!((s.body_a, s.body_b), (Body::Sun, Body::Mars));
        let SkipReason::OutsideLatitudes {
            latitude_deg,
            longitude_deg,
        } = s.reason
        else {
            panic!("{s:?}");
        };
        match s.kind_a {
            Some(LineKind::Mc) => {
                assert_eq!(s.kind_b, Some(LineKind::Asc));
                assert!((latitude_deg - mars_lat).abs() < 1e-9);
                assert!(longitude_deg.abs() < 1e-9);
            }
            Some(LineKind::Ic) => {
                assert_eq!(s.kind_b, Some(LineKind::Dsc));
                assert!((latitude_deg + mars_lat).abs() < 1e-9);
                assert!((longitude_deg + 180.0).abs() < 1e-9);
            }
            other => panic!("{other:?}"),
        }
    }
}

/// Two bodies on the geometric horizon at once: two antipodal points, each on
/// the horizon of both bodies.
#[test]
fn horizon_pairs_are_antipodal_points_on_both_horizons() {
    let map = sky_map(0.0, &[(Body::Sun, 0.0, 20.0), (Body::Mars, 60.0, -10.0)]);
    let set = parans(&map, &ParanOptions::default()).unwrap();
    let pairs: Vec<&Paran> = set
        .parans
        .iter()
        .filter(|p| !is_meridian(p.kind_a) && !is_meridian(p.kind_b))
        .collect();
    assert_eq!(pairs.len(), 2, "{:?}", set.parans);
    for p in &pairs {
        for body in [Body::Sun, Body::Mars] {
            let alt = geo_altitude(&map, body, p.latitude_deg, p.longitude_deg);
            assert!(alt.abs() < 1e-9, "{body:?} altitude {alt} at {p:?}");
        }
        let ha = geo_hour_angle(&map, Body::Sun, p.longitude_deg);
        assert_eq!(p.kind_a == LineKind::Asc, ha < 0.0, "{p:?}");
        let ha = geo_hour_angle(&map, Body::Mars, p.longitude_deg);
        assert_eq!(p.kind_b == LineKind::Asc, ha < 0.0, "{p:?}");
    }
    assert!((pairs[0].latitude_deg + pairs[1].latitude_deg).abs() < 1e-9);
    assert!((sdiff(pairs[0].longitude_deg, pairs[1].longitude_deg).abs() - 180.0).abs() < 1e-9);
    // The four crossings come out south to north.
    assert_eq!(set.parans.len(), 4);
    for w in set.parans.windows(2) {
        assert!(w[0].latitude_deg <= w[1].latitude_deg);
    }
}

// ------------------------------------------------------------- structure

/// Each horizon line of one body crosses each meridian line of another exactly
/// once when the horizon altitude is 0; the crossing is either returned or
/// skipped as outside the latitudes. Body A is always the earlier one.
#[test]
fn every_crossing_appears_once_in_map_order() {
    let map = real_map(&FOUR);
    let set = parans(&map, &ParanOptions::default()).unwrap();
    assert!(set.parans.len() >= 12, "{} parans", set.parans.len());
    let index = |b: Body| FOUR.iter().position(|&x| x == b).unwrap();
    for p in &set.parans {
        assert!(index(p.body_a) < index(p.body_b), "{p:?}");
        assert!(!(is_meridian(p.kind_a) && is_meridian(p.kind_b)), "{p:?}");
    }
    for (i, &a) in FOUR.iter().enumerate() {
        for &b in &FOUR[i + 1..] {
            for meridian in [LineKind::Mc, LineKind::Ic] {
                // A's horizon with B's meridian, then B's horizon with A's.
                let returned = set
                    .parans
                    .iter()
                    .filter(|p| {
                        p.body_a == a
                            && p.body_b == b
                            && (p.kind_b == meridian || p.kind_a == meridian)
                    })
                    .count();
                let skipped = set
                    .skipped
                    .iter()
                    .filter(|s| {
                        s.body_a == a
                            && s.body_b == b
                            && (s.kind_b == Some(meridian) || s.kind_a == Some(meridian))
                    })
                    .count();
                assert_eq!(returned + skipped, 2, "{a:?} {b:?} {meridian:?}");
            }
        }
    }
    assert!(
        set.skipped
            .iter()
            .all(|s| matches!(s.reason, SkipReason::OutsideLatitudes { .. })),
        "{:?}",
        set.skipped
    );
}

fn key(p: &Paran) -> (String, String) {
    let mut ends = [
        format!("{:?} {:?}", p.body_a, p.kind_a),
        format!("{:?} {:?}", p.body_b, p.kind_b),
    ];
    ends.sort();
    (ends[0].clone(), ends[1].clone())
}

#[test]
fn swapping_the_body_order_gives_the_same_crossings() {
    let forward = parans(&real_map(&FOUR), &ParanOptions::default()).unwrap();
    let mut reversed_bodies = FOUR;
    reversed_bodies.reverse();
    let backward = parans(&real_map(&reversed_bodies), &ParanOptions::default()).unwrap();
    assert!(!forward.parans.is_empty());
    assert_eq!(forward.parans.len(), backward.parans.len());
    for p in &forward.parans {
        let matches: Vec<&Paran> = backward
            .parans
            .iter()
            .filter(|q| {
                key(q) == key(p)
                    && (q.latitude_deg - p.latitude_deg).abs() < 1e-9
                    && sdiff(q.longitude_deg, p.longitude_deg).abs() < 1e-9
            })
            .collect();
        assert_eq!(matches.len(), 1, "{p:?}");
    }
}

#[test]
fn latitude_limits_are_respected() {
    let map = real_map(&FOUR);
    let full = parans(&map, &ParanOptions::default()).unwrap();
    let o = ParanOptions {
        min_lat_deg: 10.0,
        max_lat_deg: 50.0,
        ..ParanOptions::default()
    };
    let narrow = parans(&map, &o).unwrap();
    assert!(!narrow.parans.is_empty());
    let want: Vec<Paran> = full
        .parans
        .iter()
        .filter(|p| (10.0..=50.0).contains(&p.latitude_deg))
        .copied()
        .collect();
    assert_eq!(narrow.parans, want);
    // Every crossing left out is reported, with its latitude.
    let outside = narrow
        .skipped
        .iter()
        .filter(|s| match s.reason {
            SkipReason::OutsideLatitudes { latitude_deg, .. } => {
                assert!(!(10.0..=50.0).contains(&latitude_deg), "{s:?}");
                true
            }
            _ => false,
        })
        .count();
    let full_outside = full.skipped.len();
    assert_eq!(
        narrow.parans.len() + outside,
        full.parans.len() + full_outside
    );
}

/// A body of declination 80 is circumpolar from latitude 10 north, so between
/// 20 and 60 north it has no rising or setting line and no horizon parans.
#[test]
fn a_circumpolar_body_has_no_horizon_parans_and_they_are_reported() {
    let map = sky_map(0.0, &[(Body::Sun, 0.0, 80.0), (Body::Mars, 100.0, 5.0)]);
    let o = ParanOptions {
        min_lat_deg: 20.0,
        max_lat_deg: 60.0,
        ..ParanOptions::default()
    };
    let set = parans(&map, &o).unwrap();
    for p in &set.parans {
        assert!(
            is_meridian(p.kind_a),
            "the Sun has no horizon line here: {p:?}"
        );
    }
    for meridian in [LineKind::Mc, LineKind::Ic] {
        let s = set
            .skipped
            .iter()
            .find(|s| s.body_a == Body::Sun && s.kind_b == Some(meridian))
            .unwrap_or_else(|| panic!("Sun horizon x Mars {meridian:?} not reported"));
        match s.reason {
            SkipReason::OutsideLatitudes { latitude_deg, .. } => {
                assert!(latitude_deg.abs() <= 10.0 + 1e-9, "{s:?}");
            }
            other => panic!("{other:?}"),
        }
    }
}

/// With the horizon raised to 50 degrees, the altitude circles are 40 degrees
/// across; bodies 100 degrees apart on the equator never share a horizon, and
/// an equatorial body never reaches altitude 50 on a meridian 80 or 100 degrees
/// from its own.
#[test]
fn lines_that_never_meet_are_reported() {
    let map = sky_map(0.0, &[(Body::Sun, 0.0, 0.0), (Body::Mars, 100.0, 0.0)]);
    let o = ParanOptions {
        horizon_altitude_deg: 50.0,
        ..ParanOptions::default()
    };
    let set = parans(&map, &o).unwrap();
    assert!(set.parans.is_empty(), "{:?}", set.parans);
    assert_eq!(set.skipped.len(), 5, "{:?}", set.skipped);
    for s in &set.skipped {
        assert_eq!(s.reason, SkipReason::NeverCross, "{s:?}");
        assert!(s.kind_a.is_none() || s.kind_b.is_none(), "{s:?}");
    }
}

/// An equatorial body's rising and setting lines are the meridians 90 degrees
/// either side of its MC; a body on one of those meridians shares the line.
#[test]
fn coincident_lines_are_reported() {
    let map = sky_map(0.0, &[(Body::Sun, 0.0, 0.0), (Body::Mars, 90.0, 10.0)]);
    let set = parans(&map, &ParanOptions::default()).unwrap();
    for meridian in [LineKind::Mc, LineKind::Ic] {
        assert!(
            set.skipped.iter().any(|s| s.body_a == Body::Sun
                && s.kind_a.is_none()
                && s.kind_b == Some(meridian)
                && s.reason == SkipReason::SameLine),
            "{:?}",
            set.skipped
        );
    }
    // Two bodies at the same place share both horizon lines.
    let twins = sky_map(0.0, &[(Body::Sun, 30.0, 10.0), (Body::Mars, 30.0, 10.0)]);
    let set = parans(&twins, &ParanOptions::default()).unwrap();
    assert!(
        set.skipped
            .iter()
            .any(|s| s.kind_a.is_none() && s.kind_b.is_none() && s.reason == SkipReason::SameLine)
    );
}

#[test]
fn horizon_pairs_can_be_left_out() {
    let map = real_map(&FOUR);
    let with = parans(&map, &ParanOptions::default()).unwrap();
    let without = parans(
        &map,
        &ParanOptions {
            include_horizon_pairs: false,
            ..ParanOptions::default()
        },
    )
    .unwrap();
    let horizon_pairs = with
        .parans
        .iter()
        .filter(|p| !is_meridian(p.kind_a) && !is_meridian(p.kind_b))
        .count();
    assert!(horizon_pairs > 0);
    assert_eq!(without.parans.len() + horizon_pairs, with.parans.len());
    assert!(
        without
            .parans
            .iter()
            .all(|p| is_meridian(p.kind_a) || is_meridian(p.kind_b))
    );
}

#[test]
fn a_repeated_body_is_used_once() {
    let once = sky_map(10.0, &[(Body::Sun, 0.0, 20.0), (Body::Mars, 60.0, -10.0)]);
    let twice = sky_map(
        10.0,
        &[
            (Body::Sun, 0.0, 20.0),
            (Body::Mars, 60.0, -10.0),
            (Body::Sun, 0.0, 20.0),
        ],
    );
    let o = ParanOptions::default();
    let a = parans(&once, &o).unwrap();
    assert!(!a.parans.is_empty());
    assert_eq!(a, parans(&twice, &o).unwrap());
}

#[test]
fn bad_options_and_maps_are_errors() {
    let map = real_map(&[Body::Sun, Body::Mars]);
    let ok = ParanOptions::default();
    for o in [
        ParanOptions {
            min_lat_deg: 20.0,
            max_lat_deg: 10.0,
            ..ok
        },
        ParanOptions {
            max_lat_deg: 90.0,
            ..ok
        },
        ParanOptions {
            min_lat_deg: f64::NAN,
            ..ok
        },
        ParanOptions {
            horizon_altitude_deg: 90.0,
            ..ok
        },
        ParanOptions {
            horizon_altitude_deg: f64::INFINITY,
            ..ok
        },
    ] {
        assert!(
            matches!(parans(&map, &o), Err(MapError::InvalidInput(_))),
            "{o:?}"
        );
    }
    let mut broken = map.clone();
    broken.bodies[0].dec_deg = f64::NAN;
    assert!(matches!(
        parans(&broken, &ok),
        Err(MapError::InvalidInput(_))
    ));
    let mut broken = map.clone();
    broken.gast_deg = f64::INFINITY;
    assert!(matches!(
        parans(&broken, &ok),
        Err(MapError::InvalidInput(_))
    ));
    for (lat, orb) in [(90.0, 1.0), (f64::NAN, 1.0), (40.0, -1.0), (40.0, f64::NAN)] {
        assert!(
            matches!(
                parans_near(&map, lat, orb, &ok),
                Err(MapError::InvalidInput(_))
            ),
            "{lat} {orb}"
        );
    }
    // An empty map has no parans.
    let empty = real_map(&[]);
    assert_eq!(
        parans(&empty, &ok).unwrap(),
        ParanSet {
            parans: Vec::new(),
            skipped: Vec::new()
        }
    );
}

// ------------------------------------------------------------ parans_near

#[test]
fn parans_near_returns_exactly_those_within_the_orb() {
    let map = real_map(&FOUR);
    let o = ParanOptions::default();
    let full = parans(&map, &o).unwrap().parans;
    assert!(!full.is_empty());
    let mut non_empty = 0;
    for (lat, orb) in [
        (40.0, 1.0),
        (-23.5, 3.0),
        (0.0, 10.0),
        (55.0, 2.5),
        (28.6, 1.0),
        (-60.0, 6.0),
    ] {
        let near = parans_near(&map, lat, orb, &o).unwrap();
        let want: Vec<Paran> = full
            .iter()
            .filter(|p| (p.latitude_deg - lat).abs() <= orb)
            .copied()
            .collect();
        assert_eq!(near, want, "lat {lat} orb {orb}");
        non_empty += usize::from(!near.is_empty());
    }
    assert!(non_empty >= 2, "only {non_empty} bands held a paran");
    // Each paran is found at its own latitude with a zero orb.
    for p in &full {
        assert!(
            parans_near(&map, p.latitude_deg, 0.0, &o)
                .unwrap()
                .contains(p)
        );
    }
    // Outside the default -66..66 the band replaces the limits.
    let polar = parans_near(&map, 70.0, 3.0, &o).unwrap();
    let band = ParanOptions {
        min_lat_deg: 67.0,
        max_lat_deg: 73.0,
        ..o
    };
    assert_eq!(polar, parans(&map, &band).unwrap().parans);
}

// ---------------------------------------------------- self-consistency

/// The independent check. At every crossing, a body on the horizon has
/// topocentric altitude `h0` (within 0.01 degrees) and an hour angle whose sign
/// matches rising or setting, and a body on the MC or IC has topocentric hour
/// angle 0 or 180 (within 0.01 degrees).
///
/// The Moon's diurnal parallax (up to about one degree) is not part of this
/// geocentric map: at the Moon's horizon crossings its topocentric altitude is
/// checked against the parallax bound and its geocentric altitude, from the
/// map's own right ascension and declination, to 1e-7 degrees. On the meridian
/// the parallax in hour angle vanishes, so the Moon's MC and IC are held to
/// 0.01 degrees like the others.
fn check_against_the_almanac(set: &ParanSet, map: &AstroMap, h0: f64) -> usize {
    let a = Almanac::default_vedic();
    let jd = JdUT1(map.jd_ut1);
    let mut checked = 0;
    for p in &set.parans {
        let (lat, lon) = (p.latitude_deg, p.longitude_deg);
        for (body, kind) in [(p.body_a, p.kind_a), (p.body_b, p.kind_b)] {
            let ha = a
                .topocentric_hour_angle_deg(body, jd, lat, lon, 0.0)
                .unwrap();
            match kind {
                LineKind::Mc | LineKind::Ic => {
                    let want = if kind == LineKind::Mc { 0.0 } else { 180.0 };
                    assert!(
                        sdiff(ha, want).abs() < 0.01,
                        "{p:?}: {body:?} hour angle {ha}"
                    );
                }
                LineKind::Asc | LineKind::Dsc => {
                    let alt = a.topocentric_altitude_deg(body, jd, lat, lon, 0.0).unwrap();
                    if body == Body::Moon {
                        // Horizontal parallax reaches about 1.02 degrees at perigee.
                        assert!((alt - h0).abs() < 1.1, "{p:?}: Moon altitude {alt}");
                        let geo = geo_altitude(map, body, lat, lon);
                        assert!((geo - h0).abs() < 1e-7, "{p:?}: Moon geocentric {geo}");
                        let geo_ha = geo_hour_angle(map, body, lon);
                        assert_eq!(kind == LineKind::Asc, geo_ha < 0.0, "{p:?}");
                    } else {
                        assert!((alt - h0).abs() < 0.01, "{p:?}: {body:?} altitude {alt}");
                        assert_eq!(kind == LineKind::Asc, ha < 0.0, "{p:?}: hour angle {ha}");
                    }
                }
            }
            checked += 1;
        }
    }
    checked
}

#[test]
fn parans_agree_with_topocentric_altitude_and_hour_angle() {
    let map = real_map(&FOUR);
    let set = parans(&map, &ParanOptions::default()).unwrap();
    let checked = check_against_the_almanac(&set, &map, 0.0);
    assert!(checked >= 24, "only {checked} angles were checked");
    // Every pair of the four bodies has a paran in -66..66 except Mars and
    // Jupiter, which are close in right ascension on this date, so all their
    // crossings lie beyond 66 degrees and are reported as skipped.
    for (i, &a) in FOUR.iter().enumerate() {
        for &b in &FOUR[i + 1..] {
            let found = set.parans.iter().any(|p| p.body_a == a && p.body_b == b);
            if (a, b) == (Body::Mars, Body::Jupiter) {
                assert!(!found);
                let far = set
                    .skipped
                    .iter()
                    .filter(|s| s.body_a == a && s.body_b == b)
                    .map(|s| match s.reason {
                        SkipReason::OutsideLatitudes { latitude_deg, .. } => latitude_deg,
                        other => panic!("{s:?}: {other:?}"),
                    })
                    .collect::<Vec<f64>>();
                assert_eq!(far.len(), 6, "{far:?}");
                assert!(far.iter().all(|l| l.abs() > 66.0), "{far:?}");
            } else {
                assert!(found, "{a:?} {b:?}");
            }
        }
    }
}

#[test]
fn parans_with_a_lowered_horizon_agree_with_the_almanac() {
    let h0 = -0.5667;
    let map = real_map(&[Body::Sun, Body::Mars, Body::Jupiter]);
    let o = ParanOptions {
        horizon_altitude_deg: h0,
        ..ParanOptions::default()
    };
    let set = parans(&map, &o).unwrap();
    assert!(!set.parans.is_empty());
    let checked = check_against_the_almanac(&set, &map, h0);
    assert!(checked >= 12, "only {checked} angles were checked");
}

// ------------------------------------------------- raised horizon, numerics

/// Every crossing of a hand-built map has the horizon body at altitude `h0`
/// and the meridian body at hour angle 0 or 180, from the map's own numbers.
fn check_geometry(set: &ParanSet, map: &AstroMap, h0: f64) {
    for p in &set.parans {
        let (lat, lon) = (p.latitude_deg, p.longitude_deg);
        assert!(lat.is_finite() && lon.is_finite(), "{p:?}");
        for (body, kind) in [(p.body_a, p.kind_a), (p.body_b, p.kind_b)] {
            let ha = geo_hour_angle(map, body, lon);
            match kind {
                LineKind::Mc | LineKind::Ic => {
                    let off = if kind == LineKind::Mc { ha } else { sdiff(ha, 180.0) };
                    assert!(off.abs() < 1e-9, "{p:?}: {body:?} hour angle {ha}");
                }
                LineKind::Asc | LineKind::Dsc => {
                    let alt = geo_altitude(map, body, lat, lon);
                    assert!((alt - h0).abs() < 1e-9, "{p:?}: {body:?} altitude {alt}");
                    assert_eq!(kind == LineKind::Asc, ha < 0.0, "{p:?}: hour angle {ha}");
                }
            }
        }
    }
}

/// With the horizon raised, a meridian can cut a body's altitude circle twice:
/// `φ = ψ ± acos(sin h0 / R)` with both roots on the same half-meridian.
#[test]
fn a_raised_horizon_gives_two_crossings_on_one_meridian() {
    let map = sky_map(
        0.0,
        &[
            (Body::Sun, 0.0, 5.0),
            (Body::Mars, 50.0, -15.0),
            (Body::Jupiter, 250.0, 20.0),
        ],
    );
    for h0 in [10.0, 30.0] {
        let o = ParanOptions {
            min_lat_deg: -89.0,
            max_lat_deg: 89.0,
            horizon_altitude_deg: h0,
            ..ParanOptions::default()
        };
        let set = parans(&map, &o).unwrap();
        assert!(!set.parans.is_empty(), "h0 {h0}");
        check_geometry(&set, &map, h0);
        let doubles = set
            .parans
            .iter()
            .filter(|p| is_meridian(p.kind_a) || is_meridian(p.kind_b))
            .filter(|p| {
                let same = |q: &&Paran| key(q) == key(p) && q.longitude_deg == p.longitude_deg;
                set.parans.iter().filter(same).count() == 2
            })
            .count();
        assert!(doubles >= 2, "h0 {h0}: no meridian with two crossings");
        // The Sun's setting line crosses the MC of Mars (longitude 50) twice.
        let sun_mars: Vec<f64> = set
            .parans
            .iter()
            .filter(|p| {
                p.body_a == Body::Sun && p.body_b == Body::Mars && p.kind_b == LineKind::Mc
            })
            .map(|p| p.latitude_deg)
            .collect();
        assert_eq!(sun_mars.len(), 2, "h0 {h0}: {sun_mars:?}");
    }
}

/// `sin h0 / R = 1` up to rounding: the meridian only touches the altitude
/// circle, and there is one crossing, not two a hair apart. Here the Sun is on
/// the equator, Mars's MC is 60 degrees west of the Sun's, so `R = cos 60` and
/// `sin h0 = sin 30`.
#[test]
fn a_tangent_meridian_gives_one_crossing() {
    let map = sky_map(0.0, &[(Body::Sun, 0.0, 0.0), (Body::Mars, 60.0, 40.0)]);
    let o = ParanOptions {
        horizon_altitude_deg: 30.0,
        include_horizon_pairs: false,
        ..ParanOptions::default()
    };
    let set = parans(&map, &o).unwrap();
    let touching: Vec<&Paran> = set
        .parans
        .iter()
        .filter(|p| p.body_a == Body::Sun && p.body_b == Body::Mars && p.kind_b == LineKind::Mc)
        .collect();
    assert_eq!(touching.len(), 1, "{touching:?}");
    let p = touching[0];
    assert_eq!(p.kind_a, LineKind::Dsc);
    assert!(p.latitude_deg.abs() < 1e-9, "{p:?}");
    assert!((p.longitude_deg - 60.0).abs() < 1e-9, "{p:?}");
    check_geometry(&set, &map, 30.0);
}

/// Sub-points about 2e-9 radians from opposite: `1 + g` rounds to zero, but the
/// cross product does not. The crossings must be finite and on both horizons.
#[test]
fn nearly_opposite_bodies_give_finite_crossings() {
    let eps_deg = 2e-9_f64.to_degrees();
    let map = sky_map(
        0.0,
        &[(Body::Sun, 0.0, 0.0), (Body::Mars, 180.0, eps_deg)],
    );
    for h0 in [0.0, 10.0] {
        let o = ParanOptions {
            horizon_altitude_deg: h0,
            ..ParanOptions::default()
        };
        let set = parans(&map, &o).unwrap();
        check_geometry(&set, &map, h0);
        for s in &set.skipped {
            if let SkipReason::OutsideLatitudes {
                latitude_deg,
                longitude_deg,
            } = s.reason
            {
                assert!(
                    latitude_deg.is_finite() && longitude_deg.is_finite(),
                    "{s:?}"
                );
            }
        }
        let pairs: Vec<&Paran> = set
            .parans
            .iter()
            .filter(|p| !is_meridian(p.kind_a) && !is_meridian(p.kind_b))
            .collect();
        if h0 == 0.0 {
            // Two great circles still meet at two opposite points.
            assert_eq!(pairs.len(), 2, "{:?}", set.parans);
            for p in &pairs {
                for body in [Body::Sun, Body::Mars] {
                    let alt = geo_altitude(&map, body, p.latitude_deg, p.longitude_deg);
                    assert!(alt.abs() < 1e-6, "{body:?} altitude {alt} at {p:?}");
                }
            }
        } else {
            // Circles of radius 80 degrees around opposite points never meet.
            assert!(pairs.is_empty(), "{pairs:?}");
            let never = set.skipped.iter().any(|s| {
                s.kind_a.is_none() && s.kind_b.is_none() && s.reason == SkipReason::NeverCross
            });
            assert!(never, "{:?}", set.skipped);
        }
    }
}

// ------------------------------------------------- against the drawn lines

/// Distance in degrees (planar, longitude scaled by cos latitude) from a point
/// to the nearest piece of a polyline.
fn distance_to_line(segments: &[Vec<LatLon>], lat: f64, lon: f64) -> f64 {
    let k = lat.to_radians().cos();
    let mut best = f64::INFINITY;
    for w in segments.iter().flat_map(|seg| seg.windows(2)) {
        let (ax, ay) = (sdiff(w[0].1, lon) * k, w[0].0 - lat);
        let (bx, by) = (sdiff(w[1].1, lon) * k, w[1].0 - lat);
        let (dx, dy) = (bx - ax, by - ay);
        let len2 = dx * dx + dy * dy;
        let t = if len2 > 0.0 {
            (-(ax * dx + ay * dy) / len2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        best = best.min((ax + t * dx).hypot(ay + t * dy));
    }
    best
}

fn drawn(map: &AstroMap, body: Body, kind: LineKind) -> &[Vec<LatLon>] {
    &body_of(map, body)
        .lines
        .iter()
        .find(|l| l.kind == kind)
        .unwrap()
        .segments
}

/// Every paran lies on the lines `astro_map` draws: on the meridian exactly,
/// and within 0.5 degrees of the sampled horizon polyline (one-degree steps).
#[test]
fn parans_lie_on_the_drawn_lines() {
    let map = real_map(&FOUR);
    let set = parans(&map, &ParanOptions::default()).unwrap();
    assert!(!set.parans.is_empty());
    for p in &set.parans {
        for (body, kind) in [(p.body_a, p.kind_a), (p.body_b, p.kind_b)] {
            let line = drawn(&map, body, kind);
            if is_meridian(kind) {
                assert!(sdiff(p.longitude_deg, line[0][0].1).abs() < 1e-9, "{p:?}");
            } else {
                let d = distance_to_line(line, p.latitude_deg, p.longitude_deg);
                assert!(d < 0.5, "{p:?}: {d} degrees from the {body:?} {kind:?} line");
            }
        }
    }
}

/// A brute-force count: scan each meridian line of one body in 0.25-degree
/// steps and count where the other body's altitude changes sign. That must be
/// the number of parans for that line pair.
#[test]
fn a_latitude_scan_finds_the_same_crossings() {
    let map = real_map(&[Body::Sun, Body::Jupiter]);
    let set = parans(&map, &ParanOptions::default()).unwrap();
    let mut total = 0;
    for (h, m) in [(Body::Sun, Body::Jupiter), (Body::Jupiter, Body::Sun)] {
        for meridian in [LineKind::Mc, LineKind::Ic] {
            let lon = drawn(&map, m, meridian)[0][0].1;
            let mut changes = 0;
            let mut prev = geo_altitude(&map, h, -66.0, lon);
            for i in 1..=528 {
                let alt = geo_altitude(&map, h, -66.0 + 0.25 * f64::from(i), lon);
                if (alt < 0.0) != (prev < 0.0) {
                    changes += 1;
                }
                prev = alt;
            }
            let found = set
                .parans
                .iter()
                .filter(|p| {
                    (p.body_a == h && !is_meridian(p.kind_a) && p.kind_b == meridian)
                        || (p.body_b == h && !is_meridian(p.kind_b) && p.kind_a == meridian)
                })
                .count();
            assert_eq!(found, changes, "{h:?} horizon x {m:?} {meridian:?}");
            total += found;
        }
    }
    assert!(total >= 2, "only {total} crossings in range");
}
