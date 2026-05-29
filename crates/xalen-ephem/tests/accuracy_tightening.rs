// =============================================================================
// Accuracy Tightening Test Suite
// =============================================================================
//
// Multi-epoch validation, moon phase geometry, and speed/velocity sanity checks
// for the XALEN Ephemeris engine.
//
// These tests complement swiss_eph_crossval.rs by verifying:
//   1. Positions remain stable across 5 epochs spanning 1900-2100
//   2. Moon phase angles are geometrically correct at full/new moon
//   3. Daily speeds match known orbital mechanics
// =============================================================================

use xalen_ephem::{Almanac, Body};
use xalen_time::JdUT1;

fn almanac() -> Almanac {
    Almanac::default_vedic()
}

// ---------------------------------------------------------------------------
// Constants: 5 epochs spanning 1900-2100
// ---------------------------------------------------------------------------

/// 1900-Jan-0.5 (JD 2415020.0)
const JD_1900: f64 = 2_415_020.0;
/// 1950-Jan-1.0 (JD 2433282.5)
const JD_1950: f64 = 2_433_282.5;
/// J2000.0 = 2000-Jan-1.5 TT (JD 2451545.0)
const JD_2000: f64 = 2_451_545.0;
/// 2050-Jan-1.0 (JD 2469807.5)
const JD_2050: f64 = 2_469_807.5;
/// 2100-Jan-1.0 (JD 2488069.5)
const JD_2100: f64 = 2_488_069.5;

const MULTI_EPOCHS: &[(f64, &str)] = &[
    (JD_1900, "1900-01-01"),
    (JD_1950, "1950-01-01"),
    (JD_2000, "2000-01-01"),
    (JD_2050, "2050-01-01"),
    (JD_2100, "2100-01-01"),
];

// ===========================================================================
// Section 1: Multi-epoch position validation
// ===========================================================================

#[test]
fn multiepoch_sun_positions_valid() {
    let a = almanac();
    let mut failures = Vec::new();

    for (jd, label) in MULTI_EPOCHS {
        let lon = a.geocentric_longitude_deg(Body::Sun, JdUT1(*jd)).unwrap();
        if lon < 0.0 || lon >= 360.0 || lon.is_nan() {
            failures.push(format!("  {label}: Sun longitude invalid: {lon:.4} deg"));
        }
    }

    assert!(
        failures.is_empty(),
        "Sun position failures across epochs:\n{}",
        failures.join("\n")
    );
}

#[test]
fn multiepoch_sun_near_280_at_jan1() {
    // The Sun is always near ~280 deg tropical longitude on January 1 (Capricorn).
    // It drifts ~0.014 deg/year due to calendar/orbital precession interaction,
    // but should stay within a few degrees across 1900-2100.
    let a = almanac();
    let mut failures = Vec::new();

    for (jd, label) in MULTI_EPOCHS {
        let lon = a.geocentric_longitude_deg(Body::Sun, JdUT1(*jd)).unwrap();
        let delta = angle_delta(lon, 280.0);
        if delta > 2.0 {
            failures.push(format!(
                "  {label}: Sun at Jan 1 should be ~280 deg, got {lon:.2} deg (off by {delta:.2} deg)"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "Sun should be near 280 deg on Jan 1 across all epochs:\n{}",
        failures.join("\n")
    );
}

#[test]
fn multiepoch_moon_valid() {
    let a = almanac();
    let mut failures = Vec::new();

    for (jd, label) in MULTI_EPOCHS {
        let lon = a.geocentric_longitude_deg(Body::Moon, JdUT1(*jd)).unwrap();
        if lon < 0.0 || lon >= 360.0 || lon.is_nan() {
            failures.push(format!("  {label}: Moon longitude invalid: {lon:.4} deg"));
        }
    }

    assert!(
        failures.is_empty(),
        "Moon position failures across epochs:\n{}",
        failures.join("\n")
    );
}

#[test]
fn multiepoch_all_planets_computable() {
    let a = almanac();
    let bodies = [
        Body::Sun,
        Body::Moon,
        Body::Mercury,
        Body::Venus,
        Body::Mars,
        Body::Jupiter,
        Body::Saturn,
        Body::Uranus,
        Body::Neptune,
    ];
    let mut failures = Vec::new();

    for (jd, label) in MULTI_EPOCHS {
        for body in &bodies {
            match a.geocentric_longitude_deg(*body, JdUT1(*jd)) {
                Ok(lon) => {
                    if lon < 0.0 || lon >= 360.0 || lon.is_nan() {
                        failures.push(format!("  {label}: {body} invalid longitude: {lon:.4} deg"));
                    }
                }
                Err(e) => {
                    failures.push(format!("  {label}: {body} computation failed: {e}"));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "Multi-epoch computation failures:\n{}",
        failures.join("\n")
    );
}

#[test]
fn multiepoch_mars_jupiter_saturn_computable() {
    // Outer planets should be computable at all 5 epochs with valid longitudes
    let a = almanac();
    let outer_bodies = [
        (Body::Mars, "Mars"),
        (Body::Jupiter, "Jupiter"),
        (Body::Saturn, "Saturn"),
    ];
    let mut failures = Vec::new();

    for (jd, label) in MULTI_EPOCHS {
        for (body, name) in &outer_bodies {
            let lon = a.geocentric_longitude_deg(*body, JdUT1(*jd)).unwrap();
            if lon < 0.0 || lon >= 360.0 || lon.is_nan() {
                failures.push(format!("  {label}: {name} invalid: {lon:.4} deg"));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "Outer planet failures:\n{}",
        failures.join("\n")
    );
}

// ===========================================================================
// Section 2: Moon phase angle tests
// ===========================================================================

#[test]
fn moon_phase_full_moon_near_180() {
    // Full Moon occurs when Sun-Moon elongation is ~180 deg.
    // We search for a day near J2000 where the Moon is closest to opposition.
    let a = almanac();

    // Full Moon near J2000 was ~2000-01-21 (JD 2451565.0)
    // Scan days 18-23 to find the one closest to 180 deg
    let mut best_jd = 0.0;
    let mut best_phase = 0.0;
    let mut best_delta = 360.0;

    for day in 18..=24 {
        let jd = JD_2000 + day as f64;
        let sun = a.geocentric_longitude_deg(Body::Sun, JdUT1(jd)).unwrap();
        let moon = a.geocentric_longitude_deg(Body::Moon, JdUT1(jd)).unwrap();
        let phase = (moon - sun + 360.0) % 360.0;
        let delta = (phase - 180.0).abs();
        if delta < best_delta {
            best_delta = delta;
            best_phase = phase;
            best_jd = jd;
        }
    }

    // At the closest day, phase should be within ~10 deg of 180 deg
    // (exact full moon may fall between integer JD values)
    assert!(
        best_delta < 10.0,
        "Full Moon near J2000: best phase angle = {best_phase:.2} deg at JD {best_jd:.1} \
         (off from 180 by {best_delta:.2} deg, expected < 10 deg)"
    );
}

#[test]
fn moon_phase_new_moon_near_0() {
    // New Moon occurs when Sun-Moon elongation is ~0 deg.
    // New Moon near J2000 was ~2000-01-06 (JD 2451550.5)
    let a = almanac();

    let mut best_jd = 0.0;
    let mut best_phase = 360.0;
    let mut best_delta = 360.0;

    for day in 3..=10 {
        let jd = JD_2000 + day as f64;
        let sun = a.geocentric_longitude_deg(Body::Sun, JdUT1(jd)).unwrap();
        let moon = a.geocentric_longitude_deg(Body::Moon, JdUT1(jd)).unwrap();
        let phase = (moon - sun + 360.0) % 360.0;
        // For new moon, phase is near 0 or 360
        let delta = if phase > 180.0 { 360.0 - phase } else { phase };
        if delta < best_delta {
            best_delta = delta;
            best_phase = phase;
            best_jd = jd;
        }
    }

    // At the closest day, phase should be within ~15 deg of 0/360
    // (wider tolerance due to simplified lunar theory)
    assert!(
        best_delta < 15.0,
        "New Moon near J2000: best phase angle = {best_phase:.2} deg at JD {best_jd:.1} \
         (off from 0/360 by {best_delta:.2} deg, expected < 15 deg)"
    );
}

#[test]
fn moon_phase_angle_cycles() {
    // Over one synodic month (~29.53 days), the Moon-Sun elongation should
    // complete one full 0-360 cycle.
    let a = almanac();
    let synodic_month = 29.53;

    let start_sun = a
        .geocentric_longitude_deg(Body::Sun, JdUT1(JD_2000))
        .unwrap();
    let start_moon = a
        .geocentric_longitude_deg(Body::Moon, JdUT1(JD_2000))
        .unwrap();
    let start_phase = (start_moon - start_sun + 360.0) % 360.0;

    let end_sun = a
        .geocentric_longitude_deg(Body::Sun, JdUT1(JD_2000 + synodic_month))
        .unwrap();
    let end_moon = a
        .geocentric_longitude_deg(Body::Moon, JdUT1(JD_2000 + synodic_month))
        .unwrap();
    let end_phase = (end_moon - end_sun + 360.0) % 360.0;

    // After one synodic month, the phase should return to approximately the same value
    let phase_diff = angle_delta(start_phase, end_phase);
    assert!(
        phase_diff < 20.0,
        "Moon phase should return to ~same value after {synodic_month} days: \
         start={start_phase:.2} deg, end={end_phase:.2} deg, diff={phase_diff:.2} deg"
    );
}

// ===========================================================================
// Section 3: Speed / velocity sanity checks
// ===========================================================================

#[test]
fn sun_speed_about_1_deg_per_day() {
    // The Sun moves approximately 0.9856 deg/day on average (360/365.25).
    // It ranges from ~0.95 deg/day (aphelion, July) to ~1.02 deg/day (perihelion, Jan).
    let a = almanac();
    let mut failures = Vec::new();

    // Check 30 consecutive days starting at J2000 (early January, near perihelion)
    for day in 0..30 {
        let jd1 = JdUT1(JD_2000 + day as f64);
        let jd2 = JdUT1(JD_2000 + day as f64 + 1.0);
        let lon1 = a.geocentric_longitude_deg(Body::Sun, jd1).unwrap();
        let lon2 = a.geocentric_longitude_deg(Body::Sun, jd2).unwrap();

        let mut speed = lon2 - lon1;
        if speed < 0.0 {
            speed += 360.0;
        }

        // Near perihelion: expect 0.95 - 1.05 deg/day
        if speed < 0.85 || speed > 1.1 {
            failures.push(format!(
                "  Day {day}: Sun speed = {speed:.4} deg/day (expected 0.85-1.10)"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "Sun speed anomalies:\n{}",
        failures.join("\n")
    );
}

#[test]
fn sun_speed_aphelion_slower() {
    // Sun near aphelion (~July, JD_2000 + 182) should be slower than near perihelion
    let a = almanac();

    // Perihelion speed (early January)
    let peri_1 = a
        .geocentric_longitude_deg(Body::Sun, JdUT1(JD_2000))
        .unwrap();
    let peri_2 = a
        .geocentric_longitude_deg(Body::Sun, JdUT1(JD_2000 + 1.0))
        .unwrap();
    let mut peri_speed = peri_2 - peri_1;
    if peri_speed < 0.0 {
        peri_speed += 360.0;
    }

    // Aphelion speed (early July)
    let aph_1 = a
        .geocentric_longitude_deg(Body::Sun, JdUT1(JD_2000 + 182.0))
        .unwrap();
    let aph_2 = a
        .geocentric_longitude_deg(Body::Sun, JdUT1(JD_2000 + 183.0))
        .unwrap();
    let mut aph_speed = aph_2 - aph_1;
    if aph_speed < 0.0 {
        aph_speed += 360.0;
    }

    assert!(
        peri_speed > aph_speed,
        "Sun should be faster at perihelion ({peri_speed:.4} deg/day) than aphelion ({aph_speed:.4} deg/day)"
    );
}

#[test]
fn moon_speed_about_13_deg_per_day() {
    // The Moon moves approximately 13.18 deg/day on average.
    // It ranges from ~11.8 deg/day (apogee) to ~15.4 deg/day (perigee).
    let a = almanac();
    let mut failures = Vec::new();

    for day in 0..30 {
        let jd1 = JdUT1(JD_2000 + day as f64);
        let jd2 = JdUT1(JD_2000 + day as f64 + 1.0);
        let lon1 = a.geocentric_longitude_deg(Body::Moon, jd1).unwrap();
        let lon2 = a.geocentric_longitude_deg(Body::Moon, jd2).unwrap();

        let mut speed = lon2 - lon1;
        if speed < 0.0 {
            speed += 360.0;
        }
        if speed > 180.0 {
            speed = 360.0 - speed;
        } // handle retrograde wrap

        // Accept 10-16 deg/day (wider than actual range for simplified theory)
        if speed < 10.0 || speed > 16.0 {
            failures.push(format!(
                "  Day {day}: Moon speed = {speed:.4} deg/day (expected 10-16)"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "Moon speed anomalies:\n{}",
        failures.join("\n")
    );
}

#[test]
fn moon_average_speed_near_13() {
    // Over 30 days, the Moon's average daily motion should be near 13 deg/day
    let a = almanac();
    let mut total_motion = 0.0;

    for day in 0..30 {
        let jd1 = JdUT1(JD_2000 + day as f64);
        let jd2 = JdUT1(JD_2000 + day as f64 + 1.0);
        let lon1 = a.geocentric_longitude_deg(Body::Moon, jd1).unwrap();
        let lon2 = a.geocentric_longitude_deg(Body::Moon, jd2).unwrap();

        let mut motion = lon2 - lon1;
        if motion < 0.0 {
            motion += 360.0;
        }
        total_motion += motion;
    }

    let avg_speed = total_motion / 30.0;
    assert!(
        (avg_speed - 13.0).abs() < 1.5,
        "Moon average speed over 30 days should be ~13 deg/day, got {avg_speed:.4} deg/day"
    );
}

#[test]
fn mercury_speed_variable() {
    // Mercury has a highly variable geocentric speed due to retrograde periods.
    // Over 88 days (one orbit), it should show both direct and potentially
    // retrograde motion. Total accumulated motion should be positive overall.
    let a = almanac();
    let mut total_motion = 0.0;
    let mut max_speed = 0.0_f64;
    let mut min_speed = 360.0_f64;

    for day in 0..88 {
        let jd1 = JdUT1(JD_2000 + day as f64);
        let jd2 = JdUT1(JD_2000 + day as f64 + 1.0);
        let lon1 = a.geocentric_longitude_deg(Body::Mercury, jd1).unwrap();
        let lon2 = a.geocentric_longitude_deg(Body::Mercury, jd2).unwrap();

        let mut motion = lon2 - lon1;
        if motion > 180.0 {
            motion -= 360.0;
        }
        if motion < -180.0 {
            motion += 360.0;
        }

        max_speed = max_speed.max(motion);
        min_speed = min_speed.min(motion);
        total_motion += motion;
    }

    // Mercury's total forward motion over one orbit should be positive
    assert!(
        total_motion > 0.0,
        "Mercury net motion over 88 days should be positive, got {total_motion:.2} deg"
    );

    // Mercury should show significant speed variation (> 1 deg/day range)
    let speed_range = max_speed - min_speed;
    assert!(
        speed_range > 1.0,
        "Mercury speed range over 88 days should be > 1 deg/day, got {speed_range:.2} deg/day \
         (max={max_speed:.2}, min={min_speed:.2})"
    );
}

#[test]
fn jupiter_speed_about_0_08_deg_per_day() {
    // Jupiter moves about 0.083 deg/day on average (12-year orbit).
    // Geocentric motion includes retrograde periods.
    let a = almanac();

    // Take average over 365 days to smooth out retrograde
    let mut total_abs_motion = 0.0;
    for day in 0..365 {
        let jd1 = JdUT1(JD_2000 + day as f64);
        let jd2 = JdUT1(JD_2000 + day as f64 + 1.0);
        let lon1 = a.geocentric_longitude_deg(Body::Jupiter, jd1).unwrap();
        let lon2 = a.geocentric_longitude_deg(Body::Jupiter, jd2).unwrap();

        let mut motion = (lon2 - lon1).abs();
        if motion > 180.0 {
            motion = 360.0 - motion;
        }
        total_abs_motion += motion;
    }

    let avg_speed = total_abs_motion / 365.0;
    // Jupiter's average absolute daily motion should be 0.05-0.20 deg/day
    assert!(
        avg_speed > 0.03 && avg_speed < 0.25,
        "Jupiter average absolute speed should be 0.03-0.25 deg/day, got {avg_speed:.4} deg/day"
    );
}

#[test]
fn saturn_slower_than_jupiter() {
    // Saturn (29.5-year orbit) should accumulate less total motion than Jupiter
    // (12-year orbit) over the same time span.
    let a = almanac();
    let mut jup_total = 0.0;
    let mut sat_total = 0.0;

    for day in 0..365 {
        let jd1 = JdUT1(JD_2000 + day as f64);
        let jd2 = JdUT1(JD_2000 + day as f64 + 1.0);

        let j1 = a.geocentric_longitude_deg(Body::Jupiter, jd1).unwrap();
        let j2 = a.geocentric_longitude_deg(Body::Jupiter, jd2).unwrap();
        let mut dj = (j2 - j1).abs();
        if dj > 180.0 {
            dj = 360.0 - dj;
        }
        jup_total += dj;

        let s1 = a.geocentric_longitude_deg(Body::Saturn, jd1).unwrap();
        let s2 = a.geocentric_longitude_deg(Body::Saturn, jd2).unwrap();
        let mut ds = (s2 - s1).abs();
        if ds > 180.0 {
            ds = 360.0 - ds;
        }
        sat_total += ds;
    }

    assert!(
        jup_total > sat_total,
        "Jupiter ({jup_total:.1} deg/yr) should move more than Saturn ({sat_total:.1} deg/yr)"
    );
}

#[test]
fn planet_speed_ordering() {
    // Over one year, the absolute accumulated motion should be ordered:
    // Moon >> Sun >> Mercury/Venus >> Mars >> Jupiter >> Saturn >> Uranus >> Neptune
    // (Mercury and Venus are inner planets with complex geocentric motion, but
    //  their accumulated absolute motion should exceed Mars over a full year)
    let a = almanac();
    let bodies = [
        (Body::Moon, "Moon"),
        (Body::Sun, "Sun"),
        (Body::Mars, "Mars"),
        (Body::Jupiter, "Jupiter"),
        (Body::Saturn, "Saturn"),
    ];

    let mut accumulated: Vec<(&str, f64)> = Vec::new();

    for (body, name) in &bodies {
        let mut total = 0.0;
        for day in 0..365 {
            let jd1 = JdUT1(JD_2000 + day as f64);
            let jd2 = JdUT1(JD_2000 + day as f64 + 1.0);
            let lon1 = a.geocentric_longitude_deg(*body, jd1).unwrap();
            let lon2 = a.geocentric_longitude_deg(*body, jd2).unwrap();
            let mut d = (lon2 - lon1).abs();
            if d > 180.0 {
                d = 360.0 - d;
            }
            total += d;
        }
        accumulated.push((name, total));
    }

    // Moon should move more than Sun
    assert!(
        accumulated[0].1 > accumulated[1].1,
        "Moon ({:.0} deg/yr) should move more than Sun ({:.0} deg/yr)",
        accumulated[0].1,
        accumulated[1].1
    );

    // Sun should move more than Mars
    assert!(
        accumulated[1].1 > accumulated[2].1,
        "Sun ({:.0} deg/yr) should move more than Mars ({:.0} deg/yr)",
        accumulated[1].1,
        accumulated[2].1
    );

    // Jupiter should move more than Saturn
    assert!(
        accumulated[3].1 > accumulated[4].1,
        "Jupiter ({:.0} deg/yr) should move more than Saturn ({:.0} deg/yr)",
        accumulated[3].1,
        accumulated[4].1
    );
}

#[test]
fn sun_completes_full_circle_in_year() {
    // The Sun should traverse ~360 deg in one tropical year (~365.2422 days).
    let a = almanac();
    let tropical_year = 365.2422;

    let start = a
        .geocentric_longitude_deg(Body::Sun, JdUT1(JD_2000))
        .unwrap();
    let end = a
        .geocentric_longitude_deg(Body::Sun, JdUT1(JD_2000 + tropical_year))
        .unwrap();

    // After one tropical year, Sun should return to nearly the same longitude
    let diff = angle_delta(start, end);
    assert!(
        diff < 1.0,
        "Sun should return to ~same longitude after tropical year: start={start:.2} deg, \
         end={end:.2} deg, diff={diff:.4} deg"
    );
}

#[test]
fn moon_completes_13_circles_per_year() {
    // The Moon orbits ~13.37 times per year (360/27.32 * 365.25/360).
    // Accumulated motion should be ~4800-5000 deg/year.
    let a = almanac();
    let mut total = 0.0;

    for day in 0..365 {
        let jd1 = JdUT1(JD_2000 + day as f64);
        let jd2 = JdUT1(JD_2000 + day as f64 + 1.0);
        let lon1 = a.geocentric_longitude_deg(Body::Moon, jd1).unwrap();
        let lon2 = a.geocentric_longitude_deg(Body::Moon, jd2).unwrap();

        let mut d = lon2 - lon1;
        if d < 0.0 {
            d += 360.0;
        }
        total += d;
    }

    // Moon should accumulate ~4700-5000 deg in one year (13+ full circles)
    assert!(
        total > 4500.0 && total < 5200.0,
        "Moon should accumulate 4500-5200 deg/year, got {total:.0} deg"
    );
}

// ===========================================================================
// Section 4: Continuity checks
// ===========================================================================

#[test]
fn sun_no_discontinuities() {
    // The Sun should not show jumps larger than 1.2 deg between consecutive days.
    let a = almanac();
    let mut prev = a
        .geocentric_longitude_deg(Body::Sun, JdUT1(JD_2000))
        .unwrap();
    let mut failures = Vec::new();

    for day in 1..=730 {
        // Two full years
        let lon = a
            .geocentric_longitude_deg(Body::Sun, JdUT1(JD_2000 + day as f64))
            .unwrap();
        let mut jump = lon - prev;
        if jump < 0.0 {
            jump += 360.0;
        }
        if jump > 1.2 || jump < 0.85 {
            failures.push(format!(
                "  Day {day}: jump = {jump:.4} deg (prev={prev:.2}, curr={lon:.2})"
            ));
        }
        prev = lon;
    }

    assert!(
        failures.is_empty(),
        "Sun discontinuities detected:\n{}",
        failures.join("\n")
    );
}

#[test]
fn moon_no_large_discontinuities() {
    // The Moon should not show jumps larger than 18 deg between consecutive days.
    // (Max daily motion ~15.4 deg at perigee; 18 deg gives comfortable margin)
    let a = almanac();
    let mut prev = a
        .geocentric_longitude_deg(Body::Moon, JdUT1(JD_2000))
        .unwrap();
    let mut failures = Vec::new();

    for day in 1..=365 {
        let lon = a
            .geocentric_longitude_deg(Body::Moon, JdUT1(JD_2000 + day as f64))
            .unwrap();
        let mut jump = lon - prev;
        if jump < 0.0 {
            jump += 360.0;
        }
        if jump > 18.0 || jump < 8.0 {
            failures.push(format!(
                "  Day {day}: jump = {jump:.4} deg (prev={prev:.2}, curr={lon:.2})"
            ));
        }
        prev = lon;
    }

    assert!(
        failures.is_empty(),
        "Moon discontinuities detected:\n{}",
        failures.join("\n")
    );
}

// ===========================================================================
// Section 5: Multi-epoch stability
// ===========================================================================

#[test]
fn multiepoch_sun_longitude_monotonic_near_jan1() {
    // Due to calendar drift and precession, the Sun's tropical longitude on
    // Jan 1 should increase very slightly over the centuries.
    // 1900: ~279.64, 1950: ~280.01, 2000: ~280.38, 2050: ~280.75, 2100: ~280.61
    // (not strictly monotonic due to nutation, but should stay within a 2 deg band)
    let a = almanac();
    let mut lons = Vec::new();

    for (jd, label) in MULTI_EPOCHS {
        let lon = a.geocentric_longitude_deg(Body::Sun, JdUT1(*jd)).unwrap();
        lons.push((*label, lon));
    }

    let min = lons.iter().map(|(_, l)| *l).fold(f64::INFINITY, f64::min);
    let max = lons
        .iter()
        .map(|(_, l)| *l)
        .fold(f64::NEG_INFINITY, f64::max);
    let range = max - min;

    assert!(
        range < 2.0,
        "Sun longitude on Jan 1 should stay within a 2 deg band across 1900-2100: \
         range={range:.4} deg, values={:?}",
        lons
    );
}

#[test]
fn multiepoch_jupiter_traverses_zodiac() {
    // Jupiter has a ~12 year orbit, so across 200 years it should be
    // at very different longitudes. Verify they are well-distributed.
    let a = almanac();
    let mut lons = Vec::new();

    for (jd, _) in MULTI_EPOCHS {
        let lon = a
            .geocentric_longitude_deg(Body::Jupiter, JdUT1(*jd))
            .unwrap();
        lons.push(lon);
    }

    // At least 3 of 5 epochs should be in different quadrants
    let mut quadrants = [false; 4];
    for lon in &lons {
        quadrants[(lon / 90.0) as usize % 4] = true;
    }
    let distinct = quadrants.iter().filter(|&&q| q).count();

    assert!(
        distinct >= 3,
        "Jupiter should span at least 3 quadrants across 5 epochs: \
         lons={:?}, quadrants={distinct}",
        lons
    );
}

// ===========================================================================
// Helper
// ===========================================================================

fn angle_delta(a: f64, b: f64) -> f64 {
    let d = (a - b).abs() % 360.0;
    if d > 180.0 { 360.0 - d } else { d }
}
