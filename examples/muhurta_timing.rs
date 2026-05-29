//! Find auspicious Choghadiya periods for muhurta timing.
//!
//! Run with: cargo run --example muhurta_timing

use xalen_time::{CalendarSystem, calendar_to_jd, jd_to_calendar};
use xalen_vedic::muhurta::compute_choghadiya;

fn jd_to_time(jd: f64) -> String {
    let (_, _, _, h) = jd_to_calendar(jd, CalendarSystem::default());
    let hh = h as u32;
    let mm = ((h - hh as f64) * 60.0) as u32;
    format!("{:02}:{:02}", hh, mm)
}

fn main() {
    // Approximate sunrise/sunset for Delhi, 15 Aug 1947
    let sunrise = calendar_to_jd(1947, 8, 15, 5.75 - 5.5, CalendarSystem::default());
    let sunset = calendar_to_jd(1947, 8, 15, 18.75 - 5.5, CalendarSystem::default());
    let next_sunrise = calendar_to_jd(1947, 8, 16, 5.75 - 5.5, CalendarSystem::default());
    let weekday = 5; // Friday (0=Sun)

    let periods = compute_choghadiya(sunrise.0, sunset.0, next_sunrise.0, weekday);

    println!("=== Choghadiya Muhurta: 15 Aug 1947, Delhi ===\n");
    println!("{:<8} {:<14} {:<10}", "Time", "Choghadiya", "Quality");

    for p in &periods {
        let label = if p.is_day { "Day" } else { "Night" };
        let quality = if p.choghadiya.is_auspicious() {
            "Auspicious"
        } else {
            "Avoid"
        };
        println!(
            "{} {}-{}  {:<12} {}",
            label,
            jd_to_time(p.start_jd),
            jd_to_time(p.end_jd),
            format!("{:?}", p.choghadiya),
            quality
        );
    }
}
