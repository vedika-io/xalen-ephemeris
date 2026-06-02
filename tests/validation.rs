// Known-chart validation tests for XALEN Ephemeris
// These test against positions that can be independently verified

use xalen_ayanamsa::Ayanamsa;
use xalen_ephem::{Almanac, Body};
use xalen_time::{JdUT1, JulianDay};
use xalen_vedic::nakshatra::Nakshatra;
use xalen_vedic::panchang::Vara;
use xalen_vedic::rashi::Rashi;

fn almanac() -> Almanac {
    Almanac::default_vedic()
}

// J2000.0 = 2000-01-01 12:00:00 TT (Saturday)
const J2000: f64 = 2_451_545.0;

#[test]
fn sun_at_j2000_tropical() {
    let a = almanac();
    let lon = a.geocentric_longitude_deg(Body::Sun, JdUT1(J2000)).unwrap();
    // Sun at J2000 should be ~280.5° tropical (in Capricorn)
    assert!(
        (lon - 280.5).abs() < 2.0,
        "Sun at J2000 should be ~280.5°, got {lon}°"
    );
}

#[test]
fn sun_at_j2000_sidereal_in_sagittarius() {
    let a = almanac();
    let aya = Ayanamsa::Lahiri;
    let jd = JdUT1(J2000);
    let tt = jd.to_tt(&xalen_time::DeltaTModel::StephensonMorrisonHohenkerk2016);
    let aya_deg = aya.compute_deg(tt.as_f64());
    let sid = a.sidereal_longitude_deg(Body::Sun, jd, aya_deg).unwrap();

    // Sun sidereal at J2000 ≈ 280.5 - 23.85 ≈ 256.65° → Sagittarius (240-270°)
    let rashi = Rashi::from_longitude_deg(sid);
    assert_eq!(
        rashi,
        Rashi::Dhanu,
        "Sun at J2000 sidereal should be in Dhanu (Sagittarius), got {rashi}"
    );
}

#[test]
fn moon_position_is_reasonable() {
    let a = almanac();
    let lon = a
        .geocentric_longitude_deg(Body::Moon, JdUT1(J2000))
        .unwrap();
    assert!(
        lon >= 0.0 && lon < 360.0,
        "Moon longitude should be 0-360°, got {lon}°"
    );
}

#[test]
fn moon_moves_correctly() {
    let a = almanac();
    let lon1 = a
        .geocentric_longitude_deg(Body::Moon, JdUT1(J2000))
        .unwrap();
    let lon2 = a
        .geocentric_longitude_deg(Body::Moon, JdUT1(J2000 + 1.0))
        .unwrap();
    let mut diff = lon2 - lon1;
    if diff < 0.0 {
        diff += 360.0;
    }
    // Moon moves ~13.2°/day
    assert!(
        diff > 10.0 && diff < 16.0,
        "Moon should move ~13°/day, got {diff}°/day"
    );
}

#[test]
fn all_vedic_grahas_computable() {
    let a = almanac();
    for body in Body::VEDIC_GRAHAS {
        let result = a.geocentric_longitude_deg(*body, JdUT1(J2000));
        assert!(result.is_ok(), "Failed to compute {body}");
        let lon = result.unwrap();
        assert!(
            lon >= 0.0 && lon < 360.0,
            "{body} longitude {lon}° out of range"
        );
    }
}

#[test]
fn vara_j2000_is_saturday() {
    let vara = Vara::from_jd(J2000);
    assert_eq!(vara, Vara::Saturday, "2000-01-01 was Saturday");
}

#[test]
fn lahiri_ayanamsa_at_j2000() {
    let aya = Ayanamsa::Lahiri;
    let jd = JdUT1(J2000);
    let tt = jd.to_tt(&xalen_time::DeltaTModel::StephensonMorrisonHohenkerk2016);
    let val = aya.compute_deg(tt.as_f64());
    // Lahiri at J2000 should be ~23.85°
    assert!(
        (val - 23.85).abs() < 0.1,
        "Lahiri at J2000 should be ~23.85°, got {val}°"
    );
}

#[test]
fn rahu_ketu_opposite() {
    let a = almanac();
    let rahu = a
        .geocentric_longitude_deg(Body::MeanNode, JdUT1(J2000))
        .unwrap();
    let ketu = Body::ketu_longitude(rahu.to_radians())
        .to_degrees()
        .rem_euclid(360.0);
    let diff = (rahu - ketu).abs();
    let diff = if diff > 180.0 { 360.0 - diff } else { diff };
    assert!(
        (diff - 180.0).abs() < 0.1,
        "Rahu-Ketu should be 180° apart, got {diff}°"
    );
}

#[test]
fn concurrent_chart_computation() {
    use std::sync::Arc;
    let a = Arc::new(almanac());

    let handles: Vec<_> = (0..20)
        .map(|i| {
            let almanac = a.clone();
            std::thread::spawn(move || {
                let jd = JdUT1(J2000 + i as f64 * 30.0);
                let sun = almanac.geocentric_longitude_deg(Body::Sun, jd).unwrap();
                let moon = almanac.geocentric_longitude_deg(Body::Moon, jd).unwrap();
                (sun, moon)
            })
        })
        .collect();

    for h in handles {
        let (sun, moon) = h.join().unwrap();
        assert!(sun >= 0.0 && sun < 360.0);
        assert!(moon >= 0.0 && moon < 360.0);
    }
}

#[test]
fn panchang_at_j2000() {
    let a = almanac();
    let jd = JdUT1(J2000);
    let tt = jd.to_tt(&xalen_time::DeltaTModel::StephensonMorrisonHohenkerk2016);
    let aya_deg = Ayanamsa::Lahiri.compute_deg(tt.as_f64());

    let sun = a.sidereal_longitude_deg(Body::Sun, jd, aya_deg).unwrap();
    let moon = a.sidereal_longitude_deg(Body::Moon, jd, aya_deg).unwrap();

    let panchang = xalen_vedic::panchang::compute_panchang(sun, moon, J2000);

    // Basic sanity
    assert!(panchang.tithi.number >= 1 && panchang.tithi.number <= 30);
    assert!(panchang.yoga.number >= 1 && panchang.yoga.number <= 27);
    assert_eq!(panchang.vara, Vara::Saturday);
}

/// End-to-end transition cross-check against an INDEPENDENT pyswisseph 2.10.03
/// computation (DE431 + Lahiri sidereal).
///
/// Reference instant: 25 May 2026 00:30 UTC (= 06:00 IST), the same date the
/// `panchang_today` example uses. pyswisseph reports:
///   sun_sid    = 39.6144887510835°
///   moon_sid   = 148.3420540261929°
///   elongation = 108.7275652751094° → tithi 10 (Dashami)
///   tithi-10 end (elongation → 120°) = JD_UT 2461186.4871043526
///                                    = 26 May 2026 05:11:26 IST
///
/// Here the root-finder runs over the crate's OWN ephemeris (`Almanac`), so the
/// only discrepancy vs pyswisseph is the underlying ephemeris/ΔT/ayanamsa model
/// difference — the bisection adds no error. We require agreement to within a
/// few minutes, which is dominated by the two ephemerides' own differences, not
/// by the transition solver.
#[test]
fn panchang_tithi_end_vs_pyswisseph_ref() {
    // 25 May 2026 00:30 UTC as a raw Julian Day (UT). pyswisseph: 2461185.5208333.
    const JD_REF: f64 = 2_461_185.520_833_333_5;
    // pyswisseph tithi-10 end (independent oracle).
    const PYSWE_TITHI_END_JD: f64 = 2_461_186.487_104_352_6;

    let a = almanac();

    // positions(jd) -> (sun_sidereal_deg, moon_sidereal_deg) using the SAME
    // path the panchang example uses (Lahiri + SMH-2016 ΔT). The transition
    // solver only ever needs these two numbers.
    let positions = |jd: f64| -> (f64, f64) {
        let j = JdUT1(jd);
        let tt = j.to_tt(&xalen_time::DeltaTModel::StephensonMorrisonHohenkerk2016);
        let aya_deg = Ayanamsa::Lahiri.compute_deg(tt.as_f64());
        let sun = a.sidereal_longitude_deg(Body::Sun, j, aya_deg).unwrap();
        let moon = a.sidereal_longitude_deg(Body::Moon, j, aya_deg).unwrap();
        (sun, moon)
    };

    // Sanity: our ephemeris agrees with pyswisseph at the reference instant.
    let (sun0, moon0) = positions(JD_REF);
    assert!(
        (sun0 - 39.6144887510835).abs() < 0.05,
        "sun_sid {sun0} vs pyswisseph 39.614°"
    );
    assert!(
        (moon0 - 148.3420540261929).abs() < 0.1,
        "moon_sid {moon0} vs pyswisseph 148.342°"
    );

    let it = xalen_vedic::panchang::tithi_transition(&positions, JD_REF, true)
        .expect("tithi transition should be found");
    assert_eq!(
        it.value.number, 10,
        "reference tithi should be 10 (Dashami)"
    );

    // The solved end must reproduce pyswisseph's 05:11 IST end-time to within a
    // few minutes (limited by the two ephemerides, not the solver).
    let diff_min = (it.end_jd - PYSWE_TITHI_END_JD).abs() * 1440.0;
    assert!(
        diff_min < 5.0,
        "tithi-10 end JD {} differs from pyswisseph {} by {:.2} min",
        it.end_jd,
        PYSWE_TITHI_END_JD,
        diff_min
    );

    // The end is after the reference; the start is before it.
    assert!(it.end_jd > JD_REF, "end must be after reference instant");
    let start = it.start_jd.expect("start requested");
    assert!(start <= JD_REF, "start must be at/before reference instant");

    // The elongation at the solved end is the exact 120° boundary.
    let (s_end, m_end) = positions(it.end_jd);
    let elong_end = (m_end - s_end).rem_euclid(360.0);
    assert!(
        (elong_end - 120.0).abs() < 0.01,
        "elongation at solved end is {elong_end}, expected 120.0°"
    );
}

#[test]
fn nakshatra_at_known_position() {
    // Moon at 0° sidereal = Ashwini
    assert_eq!(Nakshatra::from_longitude_deg(0.0), Nakshatra::Ashwini);
    // Moon at 120° = Magha (120° = start of Magha at nakshatra index 9)
    let nak = Nakshatra::from_longitude_deg(120.0);
    assert_eq!(nak, Nakshatra::Magha, "120° should be Magha, got {nak}");
}

#[test]
fn delta_t_j2000_value() {
    let dt = xalen_time::delta_t(
        J2000,
        &xalen_time::DeltaTModel::StephensonMorrisonHohenkerk2016,
    );
    // delta-T at J2000 ≈ 63.83 seconds
    assert!(
        (dt - 63.83).abs() < 2.0,
        "delta-T at J2000 should be ~63.83s, got {dt}s"
    );
}

#[test]
fn sun_annual_motion() {
    let a = almanac();
    let lon_jan = a.geocentric_longitude_deg(Body::Sun, JdUT1(J2000)).unwrap();
    let lon_jul = a
        .geocentric_longitude_deg(Body::Sun, JdUT1(J2000 + 182.5))
        .unwrap();
    let mut diff = lon_jul - lon_jan;
    if diff < 0.0 {
        diff += 360.0;
    }
    assert!(
        (diff - 180.0).abs() < 5.0,
        "Sun should move ~180° in half a year, got {diff}°"
    );
}

#[test]
fn mars_position_range() {
    let a = almanac();
    for day_offset in [0.0, 100.0, 200.0, 365.0] {
        let lon = a
            .geocentric_longitude_deg(Body::Mars, JdUT1(J2000 + day_offset))
            .unwrap();
        assert!(
            lon >= 0.0 && lon < 360.0,
            "Mars out of range at offset {day_offset}: {lon}°"
        );
    }
}

#[test]
fn jupiter_slow_motion() {
    let a = almanac();
    let lon1 = a
        .geocentric_longitude_deg(Body::Jupiter, JdUT1(J2000))
        .unwrap();
    let lon2 = a
        .geocentric_longitude_deg(Body::Jupiter, JdUT1(J2000 + 30.0))
        .unwrap();
    let mut diff = (lon2 - lon1).abs();
    if diff > 180.0 {
        diff = 360.0 - diff;
    }
    assert!(
        diff < 10.0,
        "Jupiter should move slowly (~2.5°/month), got {diff}° in 30 days"
    );
}

#[test]
fn saturn_very_slow() {
    let a = almanac();
    let lon1 = a
        .geocentric_longitude_deg(Body::Saturn, JdUT1(J2000))
        .unwrap();
    let lon2 = a
        .geocentric_longitude_deg(Body::Saturn, JdUT1(J2000 + 30.0))
        .unwrap();
    let mut diff = (lon2 - lon1).abs();
    if diff > 180.0 {
        diff = 360.0 - diff;
    }
    assert!(
        diff < 5.0,
        "Saturn should move ~1°/month, got {diff}° in 30 days"
    );
}

#[test]
fn multiple_ayanamsa_at_j2000() {
    let jd = JdUT1(J2000);
    let tt = jd.to_tt(&xalen_time::DeltaTModel::StephensonMorrisonHohenkerk2016);
    let jd_tt = tt.as_f64();

    let lahiri = Ayanamsa::Lahiri.compute_deg(jd_tt);
    let raman = Ayanamsa::Raman.compute_deg(jd_tt);
    let kp = Ayanamsa::KPKrishnamurti.compute_deg(jd_tt);

    assert!(
        (lahiri - 23.85).abs() < 0.2,
        "Lahiri ~23.85°, got {lahiri}°"
    );
    assert!((raman - 22.40).abs() < 0.5, "Raman ~22.4°, got {raman}°");
    assert!(
        (kp - lahiri).abs() < 0.1,
        "KP close to Lahiri, got {kp}° vs {lahiri}°"
    );
}

#[test]
fn vedic_dignities_consistent() {
    use xalen_vedic::dignity;
    for planet in [
        "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn",
    ] {
        let ex = dignity::exaltation_sign(planet).unwrap();
        let deb = dignity::debilitation_sign(planet).unwrap();
        assert_ne!(
            ex, deb,
            "{planet}: exaltation and debilitation should differ"
        );
        let ex_idx = ex.index();
        let deb_idx = deb.index();
        let diff = ((ex_idx as i32 - deb_idx as i32).abs()) % 12;
        assert_eq!(
            diff, 6,
            "{planet}: exaltation/debilitation should be 6 signs apart, got {diff}"
        );
    }
}

#[test]
fn numerology_pythagorean_vs_chaldean() {
    use xalen_numerology::{System, expression_number};
    let py = expression_number("VEDIKA", System::Pythagorean);
    let ch = expression_number("VEDIKA", System::Chaldean);
    assert!(py >= 1 && py <= 33);
    assert!(ch >= 1 && ch <= 33);
}

#[test]
fn fixed_stars_catalog_integrity() {
    for star in xalen_stars::CATALOG {
        assert!(
            star.longitude_j2000 >= 0.0 && star.longitude_j2000 < 360.0,
            "{} longitude out of range: {}",
            star.name,
            star.longitude_j2000
        );
        assert!(
            star.latitude_j2000 >= -90.0 && star.latitude_j2000 <= 90.0,
            "{} latitude out of range: {}",
            star.name,
            star.latitude_j2000
        );
        // Threshold raised to 6.0 to accommodate traditional Jyotish yogatara
        // stars (e.g. Revati / zeta Piscium, mag 5.21) that are faint but
        // astrologically essential as nakshatra reference points.
        assert!(
            star.magnitude < 6.0,
            "{} too dim: {}",
            star.name,
            star.magnitude
        );
    }
}

#[test]
fn divisional_vargottama() {
    use xalen_vedic::divisional::{VargaChart, compute_varga_sign, is_vargottama};
    let lon = 5.0;
    let d1 = xalen_vedic::rashi::Rashi::from_longitude_deg(lon);
    let d9 = compute_varga_sign(lon, VargaChart::D9);
    let _vargottama = is_vargottama(d1, d9);
    assert_eq!(d1, xalen_vedic::rashi::Rashi::Mesha, "5° should be Aries");
}

#[test]
fn western_aspect_detection() {
    use xalen_western::aspects::*;
    let positions = vec![
        ("Sun".to_string(), 0.0, 1.0),
        ("Moon".to_string(), 120.0, 13.0),
        ("Mars".to_string(), 180.5, 0.5),
    ];
    let aspects = find_all_aspects(&positions, AspectType::MAJOR);
    assert!(
        !aspects.is_empty(),
        "Should find aspects between these positions"
    );
}

#[test]
fn historical_date_computation() {
    // JD 2000000.0 ≈ 763 CE — well within VSOP87 coverage (J2000 ± 4000 years)
    let a = almanac();
    let lon = a
        .geocentric_longitude_deg(Body::Sun, JdUT1(2_000_000.0))
        .unwrap();
    assert!(
        lon >= 0.0 && lon < 360.0,
        "Sun at JD 2000000 (~763 CE) should be valid 0-360°, got {lon}°"
    );
}

#[test]
fn future_date_computation() {
    // JD 2500000.0 ≈ 2132 CE — within VSOP87 coverage
    let a = almanac();
    let lon = a
        .geocentric_longitude_deg(Body::Sun, JdUT1(2_500_000.0))
        .unwrap();
    assert!(
        lon >= 0.0 && lon < 360.0,
        "Sun at JD 2500000 (~2132 CE) should be valid 0-360°, got {lon}°"
    );
}

#[test]
fn venus_faster_than_mars() {
    // Over 100 days, Venus (orbital period ~225d) should accumulate more total
    // motion than Mars (orbital period ~687d).
    let a = almanac();
    let mut venus_total = 0.0_f64;
    let mut mars_total = 0.0_f64;
    for i in 0..100 {
        let jd1 = JdUT1(J2000 + i as f64);
        let jd2 = JdUT1(J2000 + (i + 1) as f64);
        let v1 = a.geocentric_longitude_deg(Body::Venus, jd1).unwrap();
        let v2 = a.geocentric_longitude_deg(Body::Venus, jd2).unwrap();
        let mut dv = (v2 - v1).abs();
        if dv > 180.0 {
            dv = 360.0 - dv;
        }
        venus_total += dv;

        let m1 = a.geocentric_longitude_deg(Body::Mars, jd1).unwrap();
        let m2 = a.geocentric_longitude_deg(Body::Mars, jd2).unwrap();
        let mut dm = (m2 - m1).abs();
        if dm > 180.0 {
            dm = 360.0 - dm;
        }
        mars_total += dm;
    }
    assert!(
        venus_total > mars_total,
        "Venus should move more than Mars over 100 days: Venus={venus_total:.1}° Mars={mars_total:.1}°"
    );
}

#[test]
fn mercury_fastest_inner() {
    // Over 30 days, Mercury (orbital period ~88d) should on average move more
    // total degrees than Venus (orbital period ~225d).
    let a = almanac();
    let mut mercury_total = 0.0_f64;
    let mut venus_total = 0.0_f64;
    for i in 0..30 {
        let jd1 = JdUT1(J2000 + i as f64);
        let jd2 = JdUT1(J2000 + (i + 1) as f64);

        let me1 = a.geocentric_longitude_deg(Body::Mercury, jd1).unwrap();
        let me2 = a.geocentric_longitude_deg(Body::Mercury, jd2).unwrap();
        let mut dm = (me2 - me1).abs();
        if dm > 180.0 {
            dm = 360.0 - dm;
        }
        mercury_total += dm;

        let v1 = a.geocentric_longitude_deg(Body::Venus, jd1).unwrap();
        let v2 = a.geocentric_longitude_deg(Body::Venus, jd2).unwrap();
        let mut dv = (v2 - v1).abs();
        if dv > 180.0 {
            dv = 360.0 - dv;
        }
        venus_total += dv;
    }
    assert!(
        mercury_total > venus_total,
        "Mercury should move more than Venus over 30 days: Mercury={mercury_total:.1}° Venus={venus_total:.1}°"
    );
}

#[test]
fn all_planets_different() {
    // At any given Julian Day, no two planets should share the exact same longitude.
    // The probability of an exact f64 match is essentially zero.
    let a = almanac();
    let jd = JdUT1(J2000);
    let bodies = [
        Body::Sun,
        Body::Moon,
        Body::Mercury,
        Body::Venus,
        Body::Mars,
        Body::Jupiter,
        Body::Saturn,
    ];
    let lons: Vec<(Body, f64)> = bodies
        .iter()
        .map(|&b| (b, a.geocentric_longitude_deg(b, jd).unwrap()))
        .collect();
    for i in 0..lons.len() {
        for j in (i + 1)..lons.len() {
            assert_ne!(
                lons[i].1, lons[j].1,
                "{} and {} should not share the exact same longitude ({:.6}°)",
                lons[i].0, lons[j].0, lons[i].1
            );
        }
    }
}

#[test]
fn ayanamsa_increases_over_time() {
    // Due to precession, Lahiri ayanamsa at 2100 CE should be greater than at J2000.
    let jd_j2000 = JdUT1(J2000);
    let jd_2100 = JdUT1(J2000 + 36525.0); // J2000 + 100 years

    let tt_j2000 = jd_j2000.to_tt(&xalen_time::DeltaTModel::StephensonMorrisonHohenkerk2016);
    let tt_2100 = jd_2100.to_tt(&xalen_time::DeltaTModel::StephensonMorrisonHohenkerk2016);

    let aya_j2000 = Ayanamsa::Lahiri.compute_deg(tt_j2000.as_f64());
    let aya_2100 = Ayanamsa::Lahiri.compute_deg(tt_2100.as_f64());

    assert!(
        aya_2100 > aya_j2000,
        "Lahiri ayanamsa at 2100 CE ({aya_2100:.4}°) should be > at J2000 ({aya_j2000:.4}°)"
    );
}

#[test]
fn chinese_bazi_year_cycle_60() {
    use xalen_chinese::sexagenary_year;
    let p1 = sexagenary_year(1984);
    let p2 = sexagenary_year(2044);
    assert_eq!(p1.stem, p2.stem);
    assert_eq!(p1.branch, p2.branch);
}

#[test]
fn house_cusps_cover_360() {
    use xalen_houses::{GeoLocation, HouseSystem, compute_houses};
    let loc = GeoLocation::new(28.6, 77.2);
    let cusps = compute_houses(J2000, &loc, 23.4393, HouseSystem::Placidus);
    for i in 0..12 {
        let cusp = cusps.cusps[i];
        assert!(
            cusp >= 0.0 && cusp < 360.0,
            "Cusp {i} out of range: {cusp}°"
        );
    }
}
