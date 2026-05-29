//! Western natal chart with tropical positions and Ptolemaic aspects.
//!
//! Run with: cargo run --example western_natal

use xalen_coords::RAD_TO_DEG;
use xalen_ephem::{Almanac, Body};
use xalen_time::{CalendarSystem, calendar_to_jd};
use xalen_western::aspects::{AspectType, find_aspect};

fn zodiac_sign(deg: f64) -> &'static str {
    const SIGNS: [&str; 12] = [
        "Aries",
        "Taurus",
        "Gemini",
        "Cancer",
        "Leo",
        "Virgo",
        "Libra",
        "Scorpio",
        "Sagittarius",
        "Capricorn",
        "Aquarius",
        "Pisces",
    ];
    SIGNS[(deg / 30.0) as usize % 12]
}

fn main() {
    let jd = calendar_to_jd(1947, 8, 15, 0.0 - 5.5, CalendarSystem::default());
    let almanac = Almanac::default_vedic();

    let bodies = [
        ("Sun", Body::Sun),
        ("Moon", Body::Moon),
        ("Mercury", Body::Mercury),
        ("Venus", Body::Venus),
        ("Mars", Body::Mars),
        ("Jupiter", Body::Jupiter),
        ("Saturn", Body::Saturn),
        ("Uranus", Body::Uranus),
        ("Neptune", Body::Neptune),
        ("Pluto", Body::Pluto),
    ];

    println!("=== Western Natal Chart (Tropical) ===\n");
    let mut positions: Vec<(&str, f64)> = Vec::new();
    for (name, body) in &bodies {
        let pos = almanac.geocentric_ecliptic(*body, jd).unwrap();
        let deg = (pos.longitude * RAD_TO_DEG).rem_euclid(360.0);
        let sign = zodiac_sign(deg);
        let deg_in_sign = deg % 30.0;
        println!(
            "{:<10} {:>7.2} deg  {} {:>5.1} deg",
            name, deg, sign, deg_in_sign
        );
        positions.push((name, deg));
    }

    println!("\n--- Major Aspects ---");
    for i in 0..positions.len() {
        for j in (i + 1)..positions.len() {
            if let Some(asp) = find_aspect(
                positions[i].1,
                positions[j].1,
                1.0,
                1.0,
                AspectType::MAJOR,
                1.0,
            ) {
                println!(
                    "{:<8} {:<14} {:<8}  orb {:.1} deg",
                    positions[i].0,
                    format!("{:?}", asp.aspect_type),
                    positions[j].0,
                    asp.orb_deg
                );
            }
        }
    }
}
