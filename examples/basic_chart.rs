//! Basic chart: compute Sun/Moon positions with nakshatra and rashi.
//!
//! Run with: cargo run --example basic_chart

use xalen_ayanamsa::Ayanamsa;
use xalen_coords::RAD_TO_DEG;
use xalen_ephem::{Almanac, Body};
use xalen_time::{CalendarSystem, JulianDay, calendar_to_jd};
use xalen_vedic::nakshatra::Nakshatra;
use xalen_vedic::rashi::Rashi;

fn main() {
    // Birth data: 15 August 1947, 00:00 IST (UTC+5:30)
    // Indian Independence — New Delhi
    let jd_ut1 = calendar_to_jd(1947, 8, 15, 0.0 - 5.5, CalendarSystem::default());

    // Create the ephemeris engine (VSOP87 analytical, zero dependencies)
    let almanac = Almanac::default_vedic();

    // Compute Lahiri ayanamsa for this date
    let ayanamsa = Ayanamsa::Lahiri;
    let aya_deg = ayanamsa.compute_deg(jd_ut1.as_f64());
    println!("Ayanamsa (Lahiri): {:.4} deg", aya_deg);
    println!();

    // Compute tropical and sidereal positions for Sun and Moon
    for &body in &[Body::Sun, Body::Moon] {
        let pos = almanac.geocentric_ecliptic(body, jd_ut1).unwrap();
        let tropical_deg = pos.longitude * RAD_TO_DEG;
        let sidereal_deg = (tropical_deg - aya_deg).rem_euclid(360.0);

        let rashi = Rashi::from_longitude_deg(sidereal_deg);
        let nak = Nakshatra::from_longitude_deg(sidereal_deg);
        let pada = Nakshatra::pada(sidereal_deg);

        println!("{body}");
        println!("  Tropical:  {:.4} deg", tropical_deg);
        println!("  Sidereal:  {:.4} deg", sidereal_deg);
        println!("  Rashi:     {rashi}");
        println!("  Nakshatra: {nak} (pada {pada})");
        println!("  Lord:      {:?}", nak.lord());
        println!("  Deity:     {}", nak.deity());
        println!();
    }
}
