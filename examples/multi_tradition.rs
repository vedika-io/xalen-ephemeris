//! Same birth data analyzed in Vedic, Western, and Chinese traditions.
//!
//! Run with: cargo run --example multi_tradition

use xalen_ayanamsa::Ayanamsa;
use xalen_chinese::{compute_bazi, sexagenary_year};
use xalen_coords::RAD_TO_DEG;
use xalen_ephem::{Almanac, Body};
use xalen_time::{CalendarSystem, JulianDay, calendar_to_jd};
use xalen_vedic::nakshatra::Nakshatra;
use xalen_vedic::rashi::Rashi;

fn western_sign(deg: f64) -> &'static str {
    const S: [&str; 12] = [
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
    S[(deg / 30.0) as usize % 12]
}

fn main() {
    // India Independence: 15 Aug 1947, 00:00 IST
    let jd = calendar_to_jd(1947, 8, 15, 0.0 - 5.5, CalendarSystem::default());
    let almanac = Almanac::default_vedic();

    let sun = almanac.geocentric_ecliptic(Body::Sun, jd).unwrap();
    let moon = almanac.geocentric_ecliptic(Body::Moon, jd).unwrap();
    let sun_trop = (sun.longitude * RAD_TO_DEG).rem_euclid(360.0);
    let moon_trop = (moon.longitude * RAD_TO_DEG).rem_euclid(360.0);
    let aya = Ayanamsa::Lahiri.compute_deg(jd.as_f64());

    println!("=== Multi-Tradition Analysis: 15 Aug 1947 ===\n");

    // Western (tropical)
    println!("WESTERN (Tropical):");
    println!("  Sun:  {} ({:.2} deg)", western_sign(sun_trop), sun_trop);
    println!(
        "  Moon: {} ({:.2} deg)\n",
        western_sign(moon_trop),
        moon_trop
    );

    // Vedic (sidereal)
    let sun_sid = (sun_trop - aya).rem_euclid(360.0);
    let moon_sid = (moon_trop - aya).rem_euclid(360.0);
    println!("VEDIC (Sidereal, Lahiri {:.2} deg):", aya);
    println!(
        "  Sun:  {} ({:.2} deg)",
        Rashi::from_longitude_deg(sun_sid),
        sun_sid
    );
    println!(
        "  Moon: {} - {} pada {}\n",
        Rashi::from_longitude_deg(moon_sid),
        Nakshatra::from_longitude_deg(moon_sid),
        Nakshatra::pada(moon_sid)
    );

    // Chinese (BaZi)
    let bazi = compute_bazi(1947, jd.0, 0.0);
    let year = sexagenary_year(1947);
    println!("CHINESE (BaZi):");
    println!("  Year:  {} ({})", year, year.branch.animal());
    println!(
        "  Day Master: {:?} ({})",
        bazi.day_master,
        bazi.day_master.element().name()
    );
}
