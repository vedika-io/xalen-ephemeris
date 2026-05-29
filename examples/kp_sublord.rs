//! KP (Krishnamurti Paddhati) position: star lord, sub lord, sub-sub lord.
//!
//! Run with: cargo run --example kp_sublord

use xalen_ayanamsa::Ayanamsa;
use xalen_coords::RAD_TO_DEG;
use xalen_ephem::{Almanac, Body};
use xalen_time::{CalendarSystem, JulianDay, calendar_to_jd};
use xalen_vedic::kp::kp_position;

fn main() {
    // India Independence: 15 Aug 1947, 00:00 IST
    let jd = calendar_to_jd(1947, 8, 15, 0.0 - 5.5, CalendarSystem::default());
    let almanac = Almanac::default_vedic();
    let aya_deg = Ayanamsa::KPKrishnamurti.compute_deg(jd.as_f64());

    println!("=== KP Sub-Lord Analysis ===");
    println!("Ayanamsa (KP): {:.4} deg\n", aya_deg);

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

    println!(
        "{:<10} {:>8}  {:<10} {:<10} {:<10} {:<10}  KP#",
        "Planet", "Sid Deg", "Sign Lord", "Star Lord", "Sub Lord", "Sub-Sub"
    );

    for &body in &bodies {
        let pos = almanac.geocentric_ecliptic(body, jd).unwrap();
        let sid = (pos.longitude * RAD_TO_DEG - aya_deg).rem_euclid(360.0);
        let kp = kp_position(sid);

        println!(
            "{:<10} {:>8.2}  {:<10} {:<10} {:<10} {:<10}  {}",
            body, sid, kp.sign_lord, kp.star_lord, kp.sub_lord, kp.sub_sub_lord, kp.kp_number
        );
    }
}
