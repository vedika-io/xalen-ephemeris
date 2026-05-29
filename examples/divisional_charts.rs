//! Divisional (Varga) charts: D9 Navamsa, D10 Dasamsa, D30 Trimsamsa.
//!
//! Run with: cargo run --example divisional_charts

use xalen_ayanamsa::Ayanamsa;
use xalen_coords::RAD_TO_DEG;
use xalen_ephem::{Almanac, Body};
use xalen_time::{CalendarSystem, JulianDay, calendar_to_jd};
use xalen_vedic::divisional::{VargaChart, compute_varga_sign};

fn main() {
    let jd = calendar_to_jd(1947, 8, 15, 0.0 - 5.5, CalendarSystem::default());
    let almanac = Almanac::default_vedic();
    let aya_deg = Ayanamsa::Lahiri.compute_deg(jd.as_f64());

    let bodies = [
        ("Sun", Body::Sun),
        ("Moon", Body::Moon),
        ("Mars", Body::Mars),
        ("Mercury", Body::Mercury),
        ("Jupiter", Body::Jupiter),
        ("Venus", Body::Venus),
        ("Saturn", Body::Saturn),
    ];

    let vargas = [
        ("D1 Rashi", VargaChart::D1),
        ("D9 Navamsa", VargaChart::D9),
        ("D10 Dasamsa", VargaChart::D10),
        ("D30 Trimsamsa", VargaChart::D30),
    ];

    println!("=== Divisional Charts: India Independence ===\n");
    print!("{:<10}", "Planet");
    for (name, _) in &vargas {
        print!("{:>14}", name);
    }
    println!();

    for (name, body) in &bodies {
        let pos = almanac.geocentric_ecliptic(*body, jd).unwrap();
        let sid = (pos.longitude * RAD_TO_DEG - aya_deg).rem_euclid(360.0);

        print!("{:<10}", name);
        for (_, varga) in &vargas {
            let sign = compute_varga_sign(sid, *varga);
            print!("{:>14}", sign);
        }
        println!();
    }
}
