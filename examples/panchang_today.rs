//! Today's Panchang: Tithi, Nakshatra, Yoga, Karana, Vara.
//!
//! Run with: cargo run --example panchang_today

use xalen_ayanamsa::Ayanamsa;
use xalen_coords::RAD_TO_DEG;
use xalen_ephem::{Almanac, Body};
use xalen_time::{CalendarSystem, JulianDay, calendar_to_jd};
use xalen_vedic::panchang::compute_panchang;

fn main() {
    // Use a fixed reference date: 25 May 2026, 06:00 IST
    let jd = calendar_to_jd(2026, 5, 25, 6.0 - 5.5, CalendarSystem::default());
    let almanac = Almanac::default_vedic();
    let aya_deg = Ayanamsa::Lahiri.compute_deg(jd.as_f64());

    let sun_pos = almanac.geocentric_ecliptic(Body::Sun, jd).unwrap();
    let moon_pos = almanac.geocentric_ecliptic(Body::Moon, jd).unwrap();
    let sun_sid = (sun_pos.longitude * RAD_TO_DEG - aya_deg).rem_euclid(360.0);
    let moon_sid = (moon_pos.longitude * RAD_TO_DEG - aya_deg).rem_euclid(360.0);

    let panchang = compute_panchang(sun_sid, moon_sid, jd.as_f64());

    println!("=== Panchang for 25 May 2026 (06:00 IST) ===\n");
    println!(
        "  Tithi:     {} ({})",
        panchang.tithi,
        panchang.tithi.name()
    );
    println!("  Nakshatra: {}", panchang.nakshatra);
    println!(
        "  Yoga:      {} ({})",
        panchang.yoga.name(),
        panchang.yoga.number
    );
    println!("  Karana:    {}", panchang.karana.name());
    println!(
        "  Vara:      {} ({})",
        panchang.vara.name(),
        panchang.vara.lord()
    );
}
