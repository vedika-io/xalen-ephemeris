//! Generate a North Indian SVG chart image from birth data.
//!
//! Run with: cargo run --example full_chart_svg

use xalen_ayanamsa::Ayanamsa;
use xalen_chart::{ChartData, PlanetPosition, render_north_indian};
use xalen_coords::RAD_TO_DEG;
use xalen_ephem::{Almanac, Body};
use xalen_time::{CalendarSystem, JulianDay, calendar_to_jd};

fn main() {
    let jd = calendar_to_jd(1947, 8, 15, 0.0 - 5.5, CalendarSystem::default());
    let almanac = Almanac::default_vedic();
    let aya_deg = Ayanamsa::Lahiri.compute_deg(jd.as_f64());

    let grahas = [
        ("Su", Body::Sun),
        ("Mo", Body::Moon),
        ("Ma", Body::Mars),
        ("Me", Body::Mercury),
        ("Ju", Body::Jupiter),
        ("Ve", Body::Venus),
        ("Sa", Body::Saturn),
        ("Ra", Body::MeanNode),
    ];

    let mut positions = Vec::new();
    for (abbr, body) in &grahas {
        let pos = almanac.geocentric_ecliptic(*body, jd).unwrap();
        let sid = (pos.longitude * RAD_TO_DEG - aya_deg).rem_euclid(360.0);
        let house = (sid / 30.0) as usize + 1;
        positions.push(PlanetPosition {
            abbreviation: abbr.to_string(),
            house,
            longitude_deg: sid,
        });
    }

    let chart = ChartData {
        planet_positions: positions,
        house_cusps_deg: std::array::from_fn(|i| i as f64 * 30.0),
        ascendant_sign_index: 0,
        ayanamsa_deg: aya_deg,
    };

    let svg = render_north_indian(&chart);
    println!("=== North Indian Chart SVG ===");
    println!("SVG length: {} bytes", svg.len());
    println!("Starts with: {}", &svg[..60]);
    println!("\n(Write to file with: cargo run --example full_chart_svg > chart.svg)");

    // Uncomment to print full SVG:
    // println!("{}", svg);
}
