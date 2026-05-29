//! BaZi (Four Pillars of Destiny) chart computation.
//!
//! Run with: cargo run --example bazi_chart

use xalen_chinese::compute_bazi;

fn main() {
    // India Independence: 15 Aug 1947, 00:00 local time
    let year = 1947;
    let jd = xalen_time::calendar_to_jd(
        year,
        8,
        15,
        0.0 - 5.5,
        xalen_time::CalendarSystem::default(),
    );
    let hour = 0.0; // midnight local

    let chart = compute_bazi(year, jd.0, hour);

    println!("=== BaZi (Four Pillars): 15 Aug 1947 ===\n");
    println!("  Year:   {}", chart.year);
    println!("  Month:  {}", chart.month);
    println!("  Day:    {}", chart.day);
    println!("  Hour:   {}", chart.hour);
    println!(
        "\nDay Master: {:?} ({} {})",
        chart.day_master,
        chart.day_master.element().name(),
        if chart.day_master.is_yang() {
            "Yang"
        } else {
            "Yin"
        }
    );

    // Wu Xing analysis
    let elements = [
        chart.year.stem.element(),
        chart.month.stem.element(),
        chart.day.stem.element(),
        chart.hour.stem.element(),
    ];
    println!("\nStem elements: {:?}", elements.map(|e| e.name()));
}
