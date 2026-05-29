//! Solar return chart: find the exact date of a birthday solar return.
//!
//! Run with: cargo run --example solar_return

use xalen_coords::RAD_TO_DEG;
use xalen_ephem::{Almanac, Body};
use xalen_time::{CalendarSystem, JulianDay, calendar_to_jd, jd_to_calendar};
use xalen_western::returns::solar_return_year;

fn main() {
    // Birth: 15 Aug 1947, find the 2026 solar return (79th birthday)
    let birth_jd = calendar_to_jd(1947, 8, 15, 0.0 - 5.5, CalendarSystem::default());
    let almanac = Almanac::default_vedic();

    let natal_sun = almanac.geocentric_ecliptic(Body::Sun, birth_jd).unwrap();
    let natal_deg = (natal_sun.longitude * RAD_TO_DEG).rem_euclid(360.0);

    println!("=== Solar Return Chart ===");
    println!("Birth: 15 Aug 1947 | Natal Sun: {:.4} deg\n", natal_deg);

    // Show the last 5 solar returns
    for year_num in 75..=79 {
        let return_jd = solar_return_year(birth_jd.as_f64(), year_num);
        let (y, m, d, h) = jd_to_calendar(return_jd, CalendarSystem::default());
        let return_jd_ut1 = xalen_time::JdUT1(return_jd);
        let sun_pos = almanac
            .geocentric_ecliptic(Body::Sun, return_jd_ut1)
            .unwrap();
        let sun_deg = (sun_pos.longitude * RAD_TO_DEG).rem_euclid(360.0);

        println!(
            "Return #{:<3} {:04}-{:02}-{:02} {:02.0}:{:02.0} UT  Sun: {:.2} deg",
            year_num,
            y,
            m,
            d,
            h.floor(),
            (h.fract() * 60.0).floor(),
            sun_deg
        );
    }
}
