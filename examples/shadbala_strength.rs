//! Six-fold planetary strength (Shadbala) analysis per BPHS Ch. 27.
//!
//! Run with: cargo run --example shadbala_strength

use xalen_ayanamsa::Ayanamsa;
use xalen_coords::RAD_TO_DEG;
use xalen_ephem::{Almanac, Body};
use xalen_time::{CalendarSystem, JulianDay, calendar_to_jd};
use xalen_vedic::shadbala::{PlanetPosition, ShadBalaInput, Shadbala};

fn main() {
    let jd = calendar_to_jd(1947, 8, 15, 0.0 - 5.5, CalendarSystem::default());
    let almanac = Almanac::default_vedic();
    let aya_deg = Ayanamsa::Lahiri.compute_deg(jd.as_f64());

    let planets = [
        "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn",
    ];
    let bodies = [
        Body::Sun,
        Body::Moon,
        Body::Mars,
        Body::Mercury,
        Body::Jupiter,
        Body::Venus,
        Body::Saturn,
    ];

    // Compute all sidereal longitudes
    let lons: Vec<f64> = bodies
        .iter()
        .map(|&b| {
            let pos = almanac.geocentric_ecliptic(b, jd).unwrap();
            (pos.longitude * RAD_TO_DEG - aya_deg).rem_euclid(360.0)
        })
        .collect();

    // Realistic average daily speeds (degrees/day) for each planet
    let typical_speeds: [f64; 7] = [
        0.985,  // Sun
        13.2,   // Moon
        0.52,   // Mars
        1.38,   // Mercury
        0.083,  // Jupiter
        1.2,    // Venus
        0.034,  // Saturn
    ];

    let all_planets: Vec<PlanetPosition> = planets
        .iter()
        .zip(lons.iter())
        .zip(typical_speeds.iter())
        .map(|((&n, &l), &s)| PlanetPosition {
            name: n,
            longitude: l,
            speed: s,
        })
        .collect();

    // India Independence: midnight UT on Aug 15 1947 ≈ 5:30 AM IST.
    // day_fraction 0.0 = midnight UT; 0.229 ≈ 5:30 AM IST
    let day_fraction = 5.5 / 24.0; // IST offset from UT expressed as fraction of day

    let input = ShadBalaInput {
        jd: jd.as_f64(),
        sun_lon: lons[0],
        moon_lon: lons[1],
        day_fraction,
        all_planets,
    };

    println!("=== Shadbala: India Independence Chart ===\n");
    println!(
        "{:<10} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>6}",
        "Planet", "Sthana", "Dig", "Kala", "Cheshta", "Naisarg", "Drik", "TOTAL", "Ratio"
    );

    for (i, &name) in planets.iter().enumerate() {
        let house = (lons[i] / 30.0) as usize + 1;
        let sb = Shadbala::compute_full(name, lons[i], house, 1.0, &input);
        println!(
            "{:<10} {:>8.1} {:>8.1} {:>8.1} {:>8.1} {:>8.1} {:>8.1} {:>8.1} {:>6.2}",
            name,
            sb.sthana_bala.total,
            sb.dig_bala,
            sb.kala_bala.total,
            sb.cheshta_bala,
            sb.naisargika_bala,
            sb.drik_bala,
            sb.total,
            sb.ratio
        );
    }
}
