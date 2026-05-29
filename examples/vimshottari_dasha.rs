//! Full Vimshottari Dasha timeline with sub-periods (Antardasha).
//!
//! Run with: cargo run --example vimshottari_dasha

use xalen_ayanamsa::Ayanamsa;
use xalen_coords::RAD_TO_DEG;
use xalen_ephem::{Almanac, Body};
use xalen_time::{CalendarSystem, JulianDay, calendar_to_jd, jd_to_calendar};
use xalen_vedic::dasha::{DashaLevel, vimshottari_dasha};

fn jd_to_date(jd: f64) -> String {
    let (y, m, d, _) = jd_to_calendar(jd, CalendarSystem::default());
    format!("{:04}-{:02}-{:02}", y, m, d)
}

fn main() {
    // India Independence: 15 Aug 1947, 00:00 IST
    let jd = calendar_to_jd(1947, 8, 15, 0.0 - 5.5, CalendarSystem::default());
    let almanac = Almanac::default_vedic();
    let aya_deg = Ayanamsa::Lahiri.compute_deg(jd.as_f64());

    let moon_pos = almanac.geocentric_ecliptic(Body::Moon, jd).unwrap();
    let moon_sid = (moon_pos.longitude * RAD_TO_DEG - aya_deg).rem_euclid(360.0);

    let periods = vimshottari_dasha(moon_sid, jd.as_f64(), DashaLevel::Antardasha);

    println!("=== Vimshottari Dasha Timeline ===");
    println!("Moon sidereal: {:.2} deg\n", moon_sid);

    for maha in &periods {
        let years = (maha.end_jd - maha.start_jd) / 365.25;
        println!(
            "{} Mahadasha  {} to {}  ({:.1} yrs)",
            maha.lord,
            jd_to_date(maha.start_jd),
            jd_to_date(maha.end_jd),
            years
        );

        for antar in &maha.sub_periods {
            println!(
                "    {}-{}  {} to {}",
                maha.lord,
                antar.lord,
                jd_to_date(antar.start_jd),
                jd_to_date(antar.end_jd)
            );
        }
        println!();
    }
}
