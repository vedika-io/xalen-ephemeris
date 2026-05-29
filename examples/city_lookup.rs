//! Look up city coordinates and timezone for chart calculation.
//!
//! Run with: cargo run --example city_lookup

use xalen_houses::geocoding::{available_cities, city_coordinates};

fn main() {
    println!("=== City Coordinate Lookup ===\n");

    let test_cities = ["delhi", "mumbai", "london", "new york", "tokyo", "sydney"];

    for city in &test_cities {
        match city_coordinates(city) {
            Some(loc) => println!(
                "{:<14} lat {:.4} deg  lon {:.4} deg",
                city,
                loc.lat_deg(),
                loc.lon_deg()
            ),
            None => println!("{:<14} not found in database", city),
        }
    }

    let all = available_cities();
    println!("\nTotal cities in database: {}", all.len());

    // Show Indian astrology centers
    println!("\n--- Indian Astrology Centers ---");
    let centers = ["ujjain", "varanasi", "rishikesh", "pune", "jaipur"];
    for city in &centers {
        if let Some(loc) = city_coordinates(city) {
            println!(
                "  {:<14} {:.4}N, {:.4}E",
                city,
                loc.lat_deg(),
                loc.lon_deg()
            );
        }
    }
}
