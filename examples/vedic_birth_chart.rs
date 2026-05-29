//! Complete Vedic birth chart: planets, nakshatras, houses, dasha lord.
//!
//! Run with: cargo run --example vedic_birth_chart

use xalen_ayanamsa::Ayanamsa;
use xalen_coords::RAD_TO_DEG;
use xalen_ephem::{Almanac, Body};
use xalen_time::{CalendarSystem, JulianDay, calendar_to_jd};
use xalen_vedic::nakshatra::Nakshatra;
use xalen_vedic::rashi::Rashi;

fn main() {
    // India Independence: 15 Aug 1947, 00:00 IST (UTC+5:30), New Delhi
    let jd = calendar_to_jd(1947, 8, 15, 0.0 - 5.5, CalendarSystem::default());
    let almanac = Almanac::default_vedic();
    let aya_deg = Ayanamsa::Lahiri.compute_deg(jd.as_f64());

    println!("=== Vedic Birth Chart: India Independence ===");
    println!("Date: 15 Aug 1947, 00:00 IST, New Delhi");
    println!("Ayanamsa (Lahiri): {:.4} deg\n", aya_deg);

    let bodies = [
        Body::Sun,
        Body::Moon,
        Body::Mars,
        Body::Mercury,
        Body::Jupiter,
        Body::Venus,
        Body::Saturn,
        Body::MeanNode,
    ];

    for &body in &bodies {
        let pos = almanac.geocentric_ecliptic(body, jd).unwrap();
        let trop = pos.longitude * RAD_TO_DEG;
        let sid = (trop - aya_deg).rem_euclid(360.0);
        let rashi = Rashi::from_longitude_deg(sid);
        let nak = Nakshatra::from_longitude_deg(sid);
        let pada = Nakshatra::pada(sid);

        println!(
            "{:<20} {:>7.2} deg  {:>12}  {} pada {}",
            body, sid, rashi, nak, pada
        );
    }

    // Ketu (opposite of Rahu)
    let rahu_pos = almanac.geocentric_ecliptic(Body::MeanNode, jd).unwrap();
    let rahu_trop = rahu_pos.longitude * RAD_TO_DEG;
    let ketu_sid = (rahu_trop + 180.0 - aya_deg).rem_euclid(360.0);
    let ketu_rashi = Rashi::from_longitude_deg(ketu_sid);
    println!("{:<20} {:>7.2} deg  {:>12}", "Ketu", ketu_sid, ketu_rashi);

    // Moon's dasha lord
    let moon_pos = almanac.geocentric_ecliptic(Body::Moon, jd).unwrap();
    let moon_sid = (moon_pos.longitude * RAD_TO_DEG - aya_deg).rem_euclid(360.0);
    let moon_nak = Nakshatra::from_longitude_deg(moon_sid);
    println!(
        "\nMoon Nakshatra: {} | Dasha Lord: {}",
        moon_nak,
        moon_nak.lord()
    );
}
