//! Zi Wei Dou Shu (Purple Star Astrology) chart computation.
//!
//! Run with: cargo run --example ziwei_chart

use xalen_chinese::EarthlyBranch;
use xalen_chinese::ziwei::compute_chart;

fn main() {
    // Example birth data for Zi Wei: lunar month 7, day 15, Zi hour
    let lunar_month = 7;
    let lunar_day = 15;
    let year_stem = xalen_chinese::HeavenlyStem::Ding; // 1947 = Ding-Hai
    let birth_hour = EarthlyBranch::Zi; // midnight

    let chart = compute_chart(year_stem, lunar_month, lunar_day, birth_hour);

    println!("=== Zi Wei Dou Shu Chart ===\n");
    println!(
        "Wu Xing Ju: {:?} (value={})",
        chart.wu_xing_ju,
        chart.wu_xing_ju.value()
    );
    println!(
        "Ming Gong:  {:?} ({})",
        chart.ming_gong_branch,
        chart.ming_gong_branch.animal()
    );
    println!(
        "Shen Gong:  {:?} ({})\n",
        chart.shen_gong_branch,
        chart.shen_gong_branch.animal()
    );

    println!("{:<16} {:<8} Stars", "Palace", "Branch");
    println!("{}", "-".repeat(50));

    for info in &chart.palaces {
        let stars: Vec<String> = info
            .major_stars
            .iter()
            .map(|s| format!("{:?}", s))
            .collect();
        let star_str = if stars.is_empty() {
            "-".to_string()
        } else {
            stars.join(", ")
        };
        println!(
            "{:<16} {:<8} {}",
            format!("{:?} ({})", info.palace, info.palace.chinese_name()),
            format!("{:?}", info.branch),
            star_str
        );
    }
}
