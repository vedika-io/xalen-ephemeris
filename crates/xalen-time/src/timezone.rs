//! Timezone-aware birth time conversion.
//!
//! Provides a hardcoded offset table for major astrology cities worldwide,
//! and a `local_to_ut` function that converts a local date/time + timezone
//! offset into a Julian Date in UT.
//!
//! This does NOT handle daylight saving time transitions (those require a
//! full timezone database). The offsets represent the **standard time** for
//! each city. For birth chart work, the user should supply the actual UTC
//! offset that was in effect at the time of birth.

use crate::calendar::{CalendarSystem, calendar_to_jd};
use crate::julian::JdUT1;

/// Timezone entry: city name (lowercase key), standard UTC offset in hours.
struct TzEntry {
    city: &'static str,
    offset_hours: f64,
}

/// Master timezone offset table — 100+ major astrology cities.
/// All offsets are **standard time** (no DST).
const TIMEZONE_TABLE: &[TzEntry] = &[
    // ── India (IST +5:30) ────────────────────────────────────────────
    TzEntry {
        city: "pune",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "mumbai",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "delhi",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "new delhi",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "chennai",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "kolkata",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "bangalore",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "bengaluru",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "hyderabad",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "ahmedabad",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "jaipur",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "lucknow",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "kanpur",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "nagpur",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "indore",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "thane",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "bhopal",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "visakhapatnam",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "patna",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "vadodara",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "ghaziabad",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "ludhiana",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "agra",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "nashik",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "varanasi",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "meerut",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "rajkot",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "srinagar",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "aurangabad",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "coimbatore",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "madurai",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "jodhpur",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "guwahati",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "chandigarh",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "thiruvananthapuram",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "kochi",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "surat",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "mangalore",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "ujjain",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "rishikesh",
        offset_hours: 5.5,
    },
    // ── South Asia ───────────────────────────────────────────────────
    TzEntry {
        city: "colombo",
        offset_hours: 5.5,
    },
    TzEntry {
        city: "kathmandu",
        offset_hours: 5.75,
    },
    TzEntry {
        city: "dhaka",
        offset_hours: 6.0,
    },
    TzEntry {
        city: "karachi",
        offset_hours: 5.0,
    },
    TzEntry {
        city: "lahore",
        offset_hours: 5.0,
    },
    TzEntry {
        city: "islamabad",
        offset_hours: 5.0,
    },
    // ── Southeast Asia ───────────────────────────────────────────────
    TzEntry {
        city: "singapore",
        offset_hours: 8.0,
    },
    TzEntry {
        city: "bangkok",
        offset_hours: 7.0,
    },
    TzEntry {
        city: "jakarta",
        offset_hours: 7.0,
    },
    TzEntry {
        city: "kuala lumpur",
        offset_hours: 8.0,
    },
    TzEntry {
        city: "manila",
        offset_hours: 8.0,
    },
    TzEntry {
        city: "ho chi minh city",
        offset_hours: 7.0,
    },
    TzEntry {
        city: "hanoi",
        offset_hours: 7.0,
    },
    TzEntry {
        city: "yangon",
        offset_hours: 6.5,
    },
    // ── East Asia ────────────────────────────────────────────────────
    TzEntry {
        city: "tokyo",
        offset_hours: 9.0,
    },
    TzEntry {
        city: "osaka",
        offset_hours: 9.0,
    },
    TzEntry {
        city: "beijing",
        offset_hours: 8.0,
    },
    TzEntry {
        city: "shanghai",
        offset_hours: 8.0,
    },
    TzEntry {
        city: "hong kong",
        offset_hours: 8.0,
    },
    TzEntry {
        city: "taipei",
        offset_hours: 8.0,
    },
    TzEntry {
        city: "seoul",
        offset_hours: 9.0,
    },
    // ── Middle East ──────────────────────────────────────────────────
    TzEntry {
        city: "dubai",
        offset_hours: 4.0,
    },
    TzEntry {
        city: "abu dhabi",
        offset_hours: 4.0,
    },
    TzEntry {
        city: "riyadh",
        offset_hours: 3.0,
    },
    TzEntry {
        city: "tehran",
        offset_hours: 3.5,
    },
    TzEntry {
        city: "doha",
        offset_hours: 3.0,
    },
    TzEntry {
        city: "muscat",
        offset_hours: 4.0,
    },
    TzEntry {
        city: "baghdad",
        offset_hours: 3.0,
    },
    TzEntry {
        city: "istanbul",
        offset_hours: 3.0,
    },
    TzEntry {
        city: "jerusalem",
        offset_hours: 2.0,
    },
    TzEntry {
        city: "tel aviv",
        offset_hours: 2.0,
    },
    // ── Africa ───────────────────────────────────────────────────────
    TzEntry {
        city: "cairo",
        offset_hours: 2.0,
    },
    TzEntry {
        city: "nairobi",
        offset_hours: 3.0,
    },
    TzEntry {
        city: "lagos",
        offset_hours: 1.0,
    },
    TzEntry {
        city: "johannesburg",
        offset_hours: 2.0,
    },
    TzEntry {
        city: "cape town",
        offset_hours: 2.0,
    },
    TzEntry {
        city: "addis ababa",
        offset_hours: 3.0,
    },
    TzEntry {
        city: "casablanca",
        offset_hours: 1.0,
    },
    // ── Europe ───────────────────────────────────────────────────────
    TzEntry {
        city: "london",
        offset_hours: 0.0,
    },
    TzEntry {
        city: "paris",
        offset_hours: 1.0,
    },
    TzEntry {
        city: "berlin",
        offset_hours: 1.0,
    },
    TzEntry {
        city: "rome",
        offset_hours: 1.0,
    },
    TzEntry {
        city: "madrid",
        offset_hours: 1.0,
    },
    TzEntry {
        city: "amsterdam",
        offset_hours: 1.0,
    },
    TzEntry {
        city: "brussels",
        offset_hours: 1.0,
    },
    TzEntry {
        city: "vienna",
        offset_hours: 1.0,
    },
    TzEntry {
        city: "zurich",
        offset_hours: 1.0,
    },
    TzEntry {
        city: "athens",
        offset_hours: 2.0,
    },
    TzEntry {
        city: "moscow",
        offset_hours: 3.0,
    },
    TzEntry {
        city: "stockholm",
        offset_hours: 1.0,
    },
    TzEntry {
        city: "oslo",
        offset_hours: 1.0,
    },
    TzEntry {
        city: "helsinki",
        offset_hours: 2.0,
    },
    TzEntry {
        city: "warsaw",
        offset_hours: 1.0,
    },
    TzEntry {
        city: "prague",
        offset_hours: 1.0,
    },
    TzEntry {
        city: "bucharest",
        offset_hours: 2.0,
    },
    TzEntry {
        city: "lisbon",
        offset_hours: 0.0,
    },
    TzEntry {
        city: "dublin",
        offset_hours: 0.0,
    },
    TzEntry {
        city: "copenhagen",
        offset_hours: 1.0,
    },
    TzEntry {
        city: "edinburgh",
        offset_hours: 0.0,
    },
    // ── Americas ─────────────────────────────────────────────────────
    TzEntry {
        city: "new york",
        offset_hours: -5.0,
    },
    TzEntry {
        city: "los angeles",
        offset_hours: -8.0,
    },
    TzEntry {
        city: "chicago",
        offset_hours: -6.0,
    },
    TzEntry {
        city: "houston",
        offset_hours: -6.0,
    },
    TzEntry {
        city: "phoenix",
        offset_hours: -7.0,
    },
    TzEntry {
        city: "philadelphia",
        offset_hours: -5.0,
    },
    TzEntry {
        city: "san antonio",
        offset_hours: -6.0,
    },
    TzEntry {
        city: "san diego",
        offset_hours: -8.0,
    },
    TzEntry {
        city: "san francisco",
        offset_hours: -8.0,
    },
    TzEntry {
        city: "seattle",
        offset_hours: -8.0,
    },
    TzEntry {
        city: "denver",
        offset_hours: -7.0,
    },
    TzEntry {
        city: "boston",
        offset_hours: -5.0,
    },
    TzEntry {
        city: "washington dc",
        offset_hours: -5.0,
    },
    TzEntry {
        city: "atlanta",
        offset_hours: -5.0,
    },
    TzEntry {
        city: "miami",
        offset_hours: -5.0,
    },
    TzEntry {
        city: "dallas",
        offset_hours: -6.0,
    },
    TzEntry {
        city: "toronto",
        offset_hours: -5.0,
    },
    TzEntry {
        city: "vancouver",
        offset_hours: -8.0,
    },
    TzEntry {
        city: "montreal",
        offset_hours: -5.0,
    },
    TzEntry {
        city: "mexico city",
        offset_hours: -6.0,
    },
    TzEntry {
        city: "sao paulo",
        offset_hours: -3.0,
    },
    TzEntry {
        city: "rio de janeiro",
        offset_hours: -3.0,
    },
    TzEntry {
        city: "buenos aires",
        offset_hours: -3.0,
    },
    TzEntry {
        city: "bogota",
        offset_hours: -5.0,
    },
    TzEntry {
        city: "lima",
        offset_hours: -5.0,
    },
    TzEntry {
        city: "santiago",
        offset_hours: -4.0,
    },
    // ── Oceania ──────────────────────────────────────────────────────
    TzEntry {
        city: "sydney",
        offset_hours: 10.0,
    },
    TzEntry {
        city: "melbourne",
        offset_hours: 10.0,
    },
    TzEntry {
        city: "brisbane",
        offset_hours: 10.0,
    },
    TzEntry {
        city: "perth",
        offset_hours: 8.0,
    },
    TzEntry {
        city: "auckland",
        offset_hours: 12.0,
    },
    TzEntry {
        city: "wellington",
        offset_hours: 12.0,
    },
];

/// Look up the standard UTC offset in hours for a city.
///
/// Case-insensitive lookup. Returns `None` if the city is not in the table.
pub fn timezone_offset_hours(city: &str) -> Option<f64> {
    let lower = city.trim().to_lowercase();
    TIMEZONE_TABLE
        .iter()
        .find(|e| e.city == lower)
        .map(|e| e.offset_hours)
}

/// Convert a local date/time to Julian Date UT (Universal Time).
///
/// # Arguments
/// * `year` — Calendar year (Gregorian).
/// * `month` — Month 1-12.
/// * `day` — Day 1-31.
/// * `hour` — Local time as decimal hours (e.g. 10:30 = 10.5).
/// * `tz_offset_hours` — UTC offset in hours (e.g. +5.5 for IST, -5.0 for EST).
///
/// # Returns
/// Julian Date in the UT1 time scale.
pub fn local_to_ut(year: i32, month: u32, day: u32, hour: f64, tz_offset_hours: f64) -> JdUT1 {
    let ut_hour = hour - tz_offset_hours;
    // calendar_to_jd handles hour overflow/underflow (e.g. negative hours roll to previous day)
    calendar_to_jd(
        year,
        month,
        day,
        ut_hour,
        CalendarSystem::ProlepticGregorian,
    )
}

/// Convenience: look up city timezone and convert in one step.
///
/// Returns `None` if the city is not found in the timezone table.
pub fn local_to_ut_city(year: i32, month: u32, day: u32, hour: f64, city: &str) -> Option<JdUT1> {
    timezone_offset_hours(city).map(|tz| local_to_ut(year, month, day, hour, tz))
}

/// Return a sorted list of all cities in the timezone table.
pub fn available_cities() -> Vec<&'static str> {
    let mut cities: Vec<&str> = TIMEZONE_TABLE.iter().map(|e| e.city).collect();
    cities.sort();
    cities.dedup();
    cities
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::julian::JulianDay;

    #[test]
    fn pune_offset() {
        assert_eq!(timezone_offset_hours("Pune"), Some(5.5));
        assert_eq!(timezone_offset_hours("pune"), Some(5.5));
        assert_eq!(timezone_offset_hours("PUNE"), Some(5.5));
    }

    #[test]
    fn major_indian_cities() {
        for city in &[
            "Mumbai",
            "Delhi",
            "Chennai",
            "Kolkata",
            "Bangalore",
            "Hyderabad",
            "Ahmedabad",
            "Jaipur",
            "Lucknow",
            "Surat",
        ] {
            assert_eq!(
                timezone_offset_hours(city),
                Some(5.5),
                "IST offset missing for {city}"
            );
        }
    }

    #[test]
    fn global_cities() {
        assert_eq!(timezone_offset_hours("New York"), Some(-5.0));
        assert_eq!(timezone_offset_hours("London"), Some(0.0));
        assert_eq!(timezone_offset_hours("Tokyo"), Some(9.0));
        assert_eq!(timezone_offset_hours("Sydney"), Some(10.0));
        assert_eq!(timezone_offset_hours("Los Angeles"), Some(-8.0));
        assert_eq!(timezone_offset_hours("Chicago"), Some(-6.0));
        assert_eq!(timezone_offset_hours("Dubai"), Some(4.0));
        assert_eq!(timezone_offset_hours("Singapore"), Some(8.0));
    }

    #[test]
    fn unknown_city_returns_none() {
        assert_eq!(timezone_offset_hours("Atlantis"), None);
        assert_eq!(timezone_offset_hours(""), None);
    }

    #[test]
    fn local_to_ut_pune_birth() {
        // 1990-06-15 10:30 IST = 1990-06-15 05:00 UT
        let jd = local_to_ut(1990, 6, 15, 10.5, 5.5);
        // Expected JD for 1990-06-15 05:00 UT
        let expected = calendar_to_jd(1990, 6, 15, 5.0, CalendarSystem::ProlepticGregorian);
        assert!(
            (jd.as_f64() - expected.as_f64()).abs() < 1e-8,
            "JD mismatch: got {}, expected {}",
            jd.as_f64(),
            expected.as_f64()
        );
    }

    #[test]
    fn local_to_ut_new_york() {
        // 2000-01-01 00:00 EST (-5) = 2000-01-01 05:00 UT
        let jd = local_to_ut(2000, 1, 1, 0.0, -5.0);
        let expected = calendar_to_jd(2000, 1, 1, 5.0, CalendarSystem::ProlepticGregorian);
        assert!((jd.as_f64() - expected.as_f64()).abs() < 1e-8);
    }

    #[test]
    fn local_to_ut_city_convenience() {
        let jd = local_to_ut_city(1990, 6, 15, 10.5, "Pune");
        assert!(jd.is_some());
        let jd = jd.unwrap();
        let expected = local_to_ut(1990, 6, 15, 10.5, 5.5);
        assert!((jd.as_f64() - expected.as_f64()).abs() < 1e-10);
    }

    #[test]
    fn local_to_ut_city_unknown() {
        assert!(local_to_ut_city(1990, 6, 15, 10.5, "Atlantis").is_none());
    }

    #[test]
    fn kathmandu_nepal_offset() {
        // Nepal is UTC+5:45
        assert_eq!(timezone_offset_hours("Kathmandu"), Some(5.75));
    }

    #[test]
    fn tehran_offset() {
        // Iran Standard Time is UTC+3:30
        assert_eq!(timezone_offset_hours("Tehran"), Some(3.5));
    }

    #[test]
    fn at_least_50_cities() {
        let cities = available_cities();
        assert!(
            cities.len() >= 50,
            "table has {} cities, need >= 50",
            cities.len()
        );
    }

    #[test]
    fn at_least_100_entries() {
        assert!(
            TIMEZONE_TABLE.len() >= 100,
            "table has {} entries, need >= 100",
            TIMEZONE_TABLE.len()
        );
    }

    #[test]
    fn midnight_crossing() {
        // 2000-01-01 02:00 IST = 1999-12-31 20:30 UT
        let jd = local_to_ut(2000, 1, 1, 2.0, 5.5);
        // UT hour = 2.0 - 5.5 = -3.5 → rolls to previous day 20:30 UT
        let expected = calendar_to_jd(1999, 12, 31, 20.5, CalendarSystem::ProlepticGregorian);
        assert!(
            (jd.as_f64() - expected.as_f64()).abs() < 1e-6,
            "midnight crossing: got {}, expected {}",
            jd.as_f64(),
            expected.as_f64()
        );
    }
}
