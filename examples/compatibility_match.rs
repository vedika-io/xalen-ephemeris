//! Ashta Koota (8-fold) compatibility matching between two people.
//!
//! Run with: cargo run --example compatibility_match

use xalen_vedic::compatibility::ashtakoota_match;
use xalen_vedic::nakshatra::Nakshatra;

fn main() {
    // Example: Boy's Moon in Rohini, Girl's Moon in Hasta
    let boy_nak = Nakshatra::Rohini;
    let girl_nak = Nakshatra::Hasta;
    let boy_rashi = 1; // Vrishabha (Taurus)
    let girl_rashi = 5; // Kanya (Virgo)

    let result = ashtakoota_match(boy_nak, girl_nak, boy_rashi, girl_rashi);

    println!("=== Ashta Koota Compatibility ===");
    println!("Boy:  {} (Vrishabha)", boy_nak);
    println!("Girl: {} (Kanya)\n", girl_nak);

    println!("  Varna (1):        {}/1", result.varna);
    println!("  Vashya (2):       {}/2", result.vashya);
    println!("  Tara (3):         {}/3", result.tara);
    println!("  Yoni (4):         {}/4", result.yoni);
    println!("  Graha Maitri (5): {}/5", result.graha_maitri);
    println!("  Gana (6):         {}/6", result.gana);
    println!("  Bhakoot (7):      {}/7", result.bhakoot);
    println!("  Nadi (8):         {}/8", result.nadi);
    println!("  ─────────────────────────");
    println!("  TOTAL:            {}/36", result.total);

    let verdict = if result.total >= 18 {
        "GOOD MATCH"
    } else {
        "BELOW THRESHOLD"
    };
    println!("\nVerdict (>=18 favorable): {}", verdict);
}
