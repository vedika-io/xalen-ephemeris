//! Solar return chart: find the EXACT instant of a birthday solar return.
//!
//! The Sun's longitude does not advance uniformly (the equation of centre swings
//! its daily motion ±3 %), so a mean-period estimate drifts by hours. This
//! example uses the exact return finder [`xalen_ephem::find_return`], which roots
//! the body's natal longitude on the **real almanac Sun** — the same apparent
//! geocentric place the chart itself uses — so the date is correct to the second.
//!
//! Run with: cargo run --example solar_return

use xalen_coords::RAD_TO_DEG;
use xalen_ephem::{Almanac, Body, ReturnBody, find_return};
use xalen_time::{CalendarSystem, JulianDay, calendar_to_jd, jd_to_calendar};

fn main() {
    // Birth: 15 Aug 1947 (IST = UT+5:30), find recent solar returns.
    let birth_jd = calendar_to_jd(1947, 8, 15, 0.0 - 5.5, CalendarSystem::default());
    let almanac = Almanac::default_vedic();

    let natal_sun = almanac.geocentric_ecliptic(Body::Sun, birth_jd).unwrap();
    let natal_deg = (natal_sun.longitude * RAD_TO_DEG).rem_euclid(360.0);

    println!("=== Solar Return Chart (exact) ===");
    println!("Birth: 15 Aug 1947 | Natal Sun: {natal_deg:.4} deg\n");

    // Walk the exact solar returns forward: each search starts at the previous
    // return, so we always land on the next one.
    let mut search_start = birth_jd;
    for year_num in 1..=79 {
        let return_jd = find_return(&almanac, ReturnBody::Sun, natal_deg, search_start).unwrap();

        // Only print the last five returns (75th–79th birthdays).
        if year_num >= 75 {
            let (y, m, d, h) = jd_to_calendar(return_jd.as_f64(), CalendarSystem::default());
            let sun_pos = almanac.geocentric_ecliptic(Body::Sun, return_jd).unwrap();
            let sun_deg = (sun_pos.longitude * RAD_TO_DEG).rem_euclid(360.0);

            println!(
                "Return #{year_num:<3} {y:04}-{m:02}-{d:02} {:02.0}:{:02.0} UT  Sun: {sun_deg:.4} deg",
                h.floor(),
                (h.fract() * 60.0).floor(),
            );
        }

        search_start = return_jd;
    }
}
