//! Load DE440 ephemeris for sub-arcsecond planetary positions.
//!
//! Requires DE440 binary file. Falls back to VSOP87 if not available.
//! Run with: cargo run --example de440_precision

use std::sync::Arc;
use xalen_coords::RAD_TO_DEG;
use xalen_ephem::{Almanac, Body, De440Provider};
use xalen_time::{CalendarSystem, calendar_to_jd};

fn main() {
    let jd = calendar_to_jd(2026, 5, 25, 12.0, CalendarSystem::default());
    let de440_path = std::path::Path::new("data/de440.bsp");

    let almanac = if de440_path.exists() {
        let provider = De440Provider::try_from_file(de440_path);
        println!("=== DE440 High-Precision Ephemeris ===");
        println!("Source: JPL DE440 binary (sub-arcsecond accuracy)\n");
        Almanac::default_vedic().with_provider(Arc::new(provider))
    } else {
        println!("=== VSOP87 Analytical Ephemeris ===");
        println!("(DE440 not found at data/de440.bsp, using built-in VSOP87)\n");
        Almanac::default_vedic()
    };

    let bodies = [
        Body::Sun,
        Body::Moon,
        Body::Mercury,
        Body::Venus,
        Body::Mars,
        Body::Jupiter,
        Body::Saturn,
    ];

    println!("{:<12} {:>12} {:>12}", "Body", "Longitude", "Latitude");
    for &body in &bodies {
        let pos = almanac.geocentric_ecliptic(body, jd).unwrap();
        let lon = pos.longitude * RAD_TO_DEG;
        let lat = pos.latitude * RAD_TO_DEG;
        println!("{:<12} {:>12.6} {:>12.6}", body, lon, lat);
    }
}
