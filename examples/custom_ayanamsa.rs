//! Use a custom ayanamsa system and compare with built-in systems.
//!
//! Run with: cargo run --example custom_ayanamsa

use xalen_ayanamsa::Ayanamsa;
use xalen_time::{CalendarSystem, JulianDay, calendar_to_jd};

fn main() {
    let jd = calendar_to_jd(2026, 5, 25, 12.0, CalendarSystem::default());
    let jd_tt = jd.as_f64();

    println!("=== Ayanamsa Comparison at 25 May 2026 ===\n");

    // Built-in systems
    let systems: &[(&str, Ayanamsa)] = &[
        ("Lahiri", Ayanamsa::Lahiri),
        ("KP", Ayanamsa::KPKrishnamurti),
        ("Raman", Ayanamsa::Raman),
        ("True Chitra", Ayanamsa::TrueChitra),
        ("Fagan-Bradley", Ayanamsa::FaganBradley),
        ("Yukteswar", Ayanamsa::YukteswarSriSS),
    ];

    for (name, aya) in systems {
        println!("{:<16} {:>8.4} deg", name, aya.compute_deg(jd_tt));
    }

    // Custom ayanamsa: zero point at 2000 CE with 50"/yr precession
    let custom = Ayanamsa::Custom {
        epoch_jd: 2_451_545.0,          // J2000.0
        ayanamsa_at_epoch: 23.5,        // degrees at epoch
        precession_rate: 50.0 / 3600.0, // 50 arcsec/yr in deg/yr
    };
    println!(
        "\n{:<16} {:>8.4} deg",
        "Custom (23.5@J2000)",
        custom.compute_deg(jd_tt)
    );

    // Show total named systems available
    let all = Ayanamsa::all_named();
    println!("\nTotal built-in ayanamsa systems: {}", all.len());
    println!("Swiss Ephemeris IDs covered: 0-46 (all 47)");
}
