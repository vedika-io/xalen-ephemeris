//! Find upcoming solar and lunar eclipses in a date range.
//!
//! Run with: cargo run --example eclipse_finder

use xalen_ephem::Almanac;
use xalen_ephem::eclipse::{find_lunar_eclipses, find_solar_eclipses};
use xalen_time::{CalendarSystem, calendar_to_jd, jd_to_calendar};

fn main() {
    let almanac = Almanac::default_vedic();
    let start = calendar_to_jd(2026, 1, 1, 0.0, CalendarSystem::default());
    let end = calendar_to_jd(2027, 1, 1, 0.0, CalendarSystem::default());

    println!("=== Eclipses in 2026 ===\n");

    let solar = find_solar_eclipses(&almanac, start.0, end.0);
    println!("Solar eclipses: {}", solar.len());
    for e in &solar {
        let (y, m, d, h) = jd_to_calendar(e.jd_maximum, CalendarSystem::default());
        println!(
            "  {:04}-{:02}-{:02} {:02.0}:{:02.0} UT  {:?}  mag={:.3}",
            y,
            m,
            d,
            h.floor(),
            (h.fract() * 60.0).floor(),
            e.eclipse_type,
            e.magnitude
        );
    }

    println!();
    let lunar = find_lunar_eclipses(&almanac, start.0, end.0);
    println!("Lunar eclipses: {}", lunar.len());
    for e in &lunar {
        let (y, m, d, h) = jd_to_calendar(e.jd_maximum, CalendarSystem::default());
        println!(
            "  {:04}-{:02}-{:02} {:02.0}:{:02.0} UT  {:?}  moon_lat={:.3} deg",
            y,
            m,
            d,
            h.floor(),
            (h.fract() * 60.0).floor(),
            e.eclipse_type,
            e.moon_latitude_deg
        );
    }
}
