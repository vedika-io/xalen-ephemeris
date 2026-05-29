//! Find fixed stars conjunct natal planet positions.
//!
//! Run with: cargo run --example fixed_stars

use xalen_coords::RAD_TO_DEG;
use xalen_ephem::{Almanac, Body};
use xalen_time::{CalendarSystem, calendar_to_jd};
use xalen_western::stars::CATALOG;

fn main() {
    let jd = calendar_to_jd(1947, 8, 15, 0.0 - 5.5, CalendarSystem::default());
    let almanac = Almanac::default_vedic();

    let bodies = [
        ("Sun", Body::Sun),
        ("Moon", Body::Moon),
        ("Mars", Body::Mars),
        ("Jupiter", Body::Jupiter),
        ("Saturn", Body::Saturn),
    ];

    println!("=== Fixed Stars Conjunct Natal Planets ===");
    println!("(within 1 degree orb)\n");

    for (name, body) in &bodies {
        let pos = almanac.geocentric_ecliptic(*body, jd).unwrap();
        let planet_lon = (pos.longitude * RAD_TO_DEG).rem_euclid(360.0);

        let mut found = false;
        for star in CATALOG {
            let dist = (star.ecl_lon_deg - planet_lon).rem_euclid(360.0);
            let orb = if dist > 180.0 { 360.0 - dist } else { dist };
            if orb <= 1.0 && star.magnitude < 3.0 {
                println!(
                    "{:<8} ({:>6.2} deg) conjunct {} ({:>6.2} deg, mag {:.1})  orb {:.2} deg",
                    name, planet_lon, star.name, star.ecl_lon_deg, star.magnitude, orb
                );
                found = true;
            }
        }
        if !found {
            println!(
                "{:<8} ({:>6.2} deg) - no bright star conjunction",
                name, planet_lon
            );
        }
    }

    println!("\nCatalog size: {} stars", CATALOG.len());
}
