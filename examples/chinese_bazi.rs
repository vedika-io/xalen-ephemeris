//! BaZi (Four Pillars of Destiny) chart computation.
//!
//! Run with: cargo run --example chinese_bazi

use xalen_chinese::{compute_bazi, sexagenary_year};
use xalen_time::{CalendarSystem, JulianDay, calendar_to_jd};

fn main() {
    // Birth data: 4 February 2024, 08:15 local time (Year of the Wood Dragon)
    let year = 2024;
    let jd_ut1 = calendar_to_jd(year, 2, 4, 8.25, CalendarSystem::default());
    let hour = 8.25;

    println!("=== BaZi (Four Pillars) ===");
    println!("Date: 4 February 2024, 08:15");
    println!();

    // Compute the full Four Pillars chart
    let chart = compute_bazi(year, jd_ut1.as_f64(), hour);

    println!("--- Four Pillars ---");
    println!("  Year:  {}", chart.year);
    println!("  Month: {}", chart.month);
    println!("  Day:   {}", chart.day);
    println!("  Hour:  {}", chart.hour);
    println!();

    // Day Master analysis
    let dm = chart.day_master;
    println!("--- Day Master ---");
    println!("  Stem:    {:?}", dm);
    println!("  Element: {}", dm.element().name());
    println!("  Polarity: {}", if dm.is_yang() { "Yang" } else { "Yin" });
    println!();

    // Five Elements (Wu Xing) analysis
    println!("--- Wu Xing (Five Elements) in Chart ---");
    let pillars = [
        ("Year", &chart.year),
        ("Month", &chart.month),
        ("Day", &chart.day),
        ("Hour", &chart.hour),
    ];
    for (name, pillar) in &pillars {
        println!(
            "  {} Stem: {:?} ({}) | Branch: {:?} ({}, {})",
            name,
            pillar.stem,
            pillar.stem.element().name(),
            pillar.branch,
            pillar.branch.animal(),
            pillar.branch.element().name()
        );
    }
    println!();

    // Element cycle relationships
    let dm_element = dm.element();
    println!("--- Day Master Element Relationships ---");
    println!(
        "  {} generates {}",
        dm_element.name(),
        dm_element.generates().name()
    );
    println!(
        "  {} overcomes {}",
        dm_element.name(),
        dm_element.overcomes().name()
    );
    println!();

    // Sexagenary year cycle for a range
    println!("--- Sexagenary Cycle (2020-2031) ---");
    for y in 2020..=2031 {
        let p = sexagenary_year(y);
        println!(
            "  {}: {:?}-{:?} ({} {})",
            y,
            p.stem,
            p.branch,
            p.stem.element().name(),
            p.branch.animal()
        );
    }
}
