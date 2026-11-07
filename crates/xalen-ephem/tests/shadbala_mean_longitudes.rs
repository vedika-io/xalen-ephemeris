//! Cross-check of the mean longitudes that Shadbala's Cheshta Bala uses
//! (`xalen_vedic::shadbala::mean_sun_longitude_deg` and
//! `mean_heliocentric_longitude_deg`, Meeus polynomials) against the VSOP87D
//! theory, which gives the true heliocentric longitude of each planet in the
//! mean equinox of date: the same frame as the polynomials.
//!
//! True minus mean longitude is the planet's equation of the centre plus small
//! perturbations. It must therefore stay inside the equation-of-centre
//! amplitude (about 2e radians), average to nearly zero over many orbits, and
//! have no trend: a wrong constant term shows as a non-zero average, a wrong
//! mean motion as a drift between two distant windows.

use xalen_vedic::shadbala::{mean_heliocentric_longitude_deg, mean_sun_longitude_deg};

const J2000: f64 = 2_451_545.0;

/// VSOP87D heliocentric longitude of date in degrees, 0 to 360.
fn vsop_longitude_deg(planet: &str, jd: f64) -> f64 {
    let c = match planet {
        "Mercury" => vsop87::vsop87d::mercury(jd),
        "Venus" => vsop87::vsop87d::venus(jd),
        "Earth" => vsop87::vsop87d::earth(jd),
        "Mars" => vsop87::vsop87d::mars(jd),
        "Jupiter" => vsop87::vsop87d::jupiter(jd),
        "Saturn" => vsop87::vsop87d::saturn(jd),
        other => panic!("no VSOP87D series for {other}"),
    };
    c.longitude().to_degrees().rem_euclid(360.0)
}

/// Mean longitude under test. "Earth" is the Sun's geometric mean longitude
/// minus 180 degrees, since the Sun is seen opposite the Earth.
fn mean_deg(planet: &str, jd: f64) -> f64 {
    if planet == "Earth" {
        (mean_sun_longitude_deg(jd) + 180.0).rem_euclid(360.0)
    } else {
        mean_heliocentric_longitude_deg(planet, jd).unwrap()
    }
}

fn wrap(x: f64) -> f64 {
    let r = x.rem_euclid(360.0);
    if r > 180.0 { r - 360.0 } else { r }
}

/// True minus mean, in degrees, in -180..180.
fn true_minus_mean(planet: &str, jd: f64) -> f64 {
    wrap(vsop_longitude_deg(planet, jd) - mean_deg(planet, jd))
}

// (planet, bound on |true - mean| in degrees). The bound is the equation of
// the centre 2e (Mercury 23.7, Venus 0.8, Earth 1.9, Mars 10.7, Jupiter 5.6,
// Saturn 6.4) plus room for planetary perturbations.
const BOUNDS: [(&str, f64); 6] = [
    ("Mercury", 25.0),
    ("Venus", 1.5),
    ("Earth", 2.2),
    ("Mars", 11.5),
    ("Jupiter", 6.5),
    ("Saturn", 8.0),
];

#[test]
fn true_minus_mean_stays_within_the_equation_of_centre() {
    for (planet, bound) in BOUNDS {
        let mut worst = 0.0_f64;
        // 1800 to 2200, every 7 days.
        let mut jd = J2000 - 73_050.0;
        while jd < J2000 + 73_050.0 {
            worst = worst.max(true_minus_mean(planet, jd).abs());
            jd += 7.0;
        }
        eprintln!("{planet}: max |true - mean| = {worst:.3} deg (bound {bound})");
        assert!(worst < bound, "{planet}: {worst} >= {bound}");
    }
}

/// Mean of true minus mean over a window of `years` starting at `start_jd`,
/// sampled every 5 days.
fn window_average(planet: &str, start_jd: f64, years: f64) -> f64 {
    let n = (years * 365.25 / 5.0) as usize;
    let sum: f64 = (0..n)
        .map(|i| true_minus_mean(planet, start_jd + i as f64 * 5.0))
        .sum();
    sum / n as f64
}

// A window of 118 years is nearly whole numbers of orbits for every planet
// (Jupiter 9.95, Saturn 4.01), so the equation of the centre averages out.
const YEARS: f64 = 118.0;

#[test]
fn mean_offset_is_small_near_j2000() {
    // (planet, tolerance in degrees on the window average)
    let tol = [
        ("Mercury", 0.05),
        ("Venus", 0.05),
        ("Earth", 0.05),
        ("Mars", 0.1),
        ("Jupiter", 0.4),
        ("Saturn", 0.6),
    ];
    for (planet, t) in tol {
        let avg = window_average(planet, J2000 - YEARS * 365.25 / 2.0, YEARS);
        eprintln!("{planet}: average true - mean around J2000 = {avg:.4} deg (tol {t})");
        assert!(avg.abs() < t, "{planet}: {avg}");
    }
}

#[test]
fn mean_motion_matches_vsop87_over_eight_centuries() {
    // Compare the window average four centuries before J2000 with the one
    // four centuries after: a wrong rate shows as a drift per century.
    let tol = [
        ("Mercury", 0.01),
        ("Venus", 0.01),
        ("Earth", 0.01),
        ("Mars", 0.03),
        ("Jupiter", 0.1),
        ("Saturn", 0.15),
    ];
    let half = YEARS * 365.25 / 2.0;
    for (planet, t) in tol {
        let early = window_average(planet, J2000 - 400.0 * 365.25 - half, YEARS);
        let late = window_average(planet, J2000 + 400.0 * 365.25 - half, YEARS);
        let drift_per_century = (late - early) / 8.0;
        eprintln!(
            "{planet}: window averages {early:.4} and {late:.4}; drift {drift_per_century:.5} deg/century (tol {t})"
        );
        assert!(drift_per_century.abs() < t, "{planet}: {drift_per_century}");
    }
}
