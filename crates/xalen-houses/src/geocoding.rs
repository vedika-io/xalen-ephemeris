//! City geocoding lookup — hardcoded coordinates for 100+ astrology-relevant cities.
//!
//! Zero external dependencies, zero network calls. Provides latitude/longitude
//! for the top cities used in birth chart calculation worldwide.

use crate::angles::GeoLocation;

/// A city entry with name and coordinates.
struct CityEntry {
    city: &'static str,
    lat: f64,
    lon: f64,
}

/// Master city coordinate table — 100+ cities.
/// Coordinates are WGS84 (standard GPS datum).
const CITY_TABLE: &[CityEntry] = &[
    // ── India — major cities + astrology centers ─────────────────────
    CityEntry {
        city: "pune",
        lat: 18.5204,
        lon: 73.8567,
    },
    CityEntry {
        city: "mumbai",
        lat: 19.0760,
        lon: 72.8777,
    },
    CityEntry {
        city: "delhi",
        lat: 28.6139,
        lon: 77.2090,
    },
    CityEntry {
        city: "new delhi",
        lat: 28.6139,
        lon: 77.2090,
    },
    CityEntry {
        city: "chennai",
        lat: 13.0827,
        lon: 80.2707,
    },
    CityEntry {
        city: "kolkata",
        lat: 22.5726,
        lon: 88.3639,
    },
    CityEntry {
        city: "bangalore",
        lat: 12.9716,
        lon: 77.5946,
    },
    CityEntry {
        city: "bengaluru",
        lat: 12.9716,
        lon: 77.5946,
    },
    CityEntry {
        city: "hyderabad",
        lat: 17.3850,
        lon: 78.4867,
    },
    CityEntry {
        city: "ahmedabad",
        lat: 23.0225,
        lon: 72.5714,
    },
    CityEntry {
        city: "jaipur",
        lat: 26.9124,
        lon: 75.7873,
    },
    CityEntry {
        city: "lucknow",
        lat: 26.8467,
        lon: 80.9462,
    },
    CityEntry {
        city: "kanpur",
        lat: 26.4499,
        lon: 80.3319,
    },
    CityEntry {
        city: "nagpur",
        lat: 21.1458,
        lon: 79.0882,
    },
    CityEntry {
        city: "indore",
        lat: 22.7196,
        lon: 75.8577,
    },
    CityEntry {
        city: "thane",
        lat: 19.2183,
        lon: 72.9781,
    },
    CityEntry {
        city: "bhopal",
        lat: 23.2599,
        lon: 77.4126,
    },
    CityEntry {
        city: "visakhapatnam",
        lat: 17.6868,
        lon: 83.2185,
    },
    CityEntry {
        city: "patna",
        lat: 25.6093,
        lon: 85.1376,
    },
    CityEntry {
        city: "vadodara",
        lat: 22.3072,
        lon: 73.1812,
    },
    CityEntry {
        city: "ghaziabad",
        lat: 28.6692,
        lon: 77.4538,
    },
    CityEntry {
        city: "ludhiana",
        lat: 30.9010,
        lon: 75.8573,
    },
    CityEntry {
        city: "agra",
        lat: 27.1767,
        lon: 78.0081,
    },
    CityEntry {
        city: "nashik",
        lat: 19.9975,
        lon: 73.7898,
    },
    CityEntry {
        city: "varanasi",
        lat: 25.3176,
        lon: 83.0103,
    },
    CityEntry {
        city: "meerut",
        lat: 28.9845,
        lon: 77.7064,
    },
    CityEntry {
        city: "rajkot",
        lat: 22.3039,
        lon: 70.8022,
    },
    CityEntry {
        city: "srinagar",
        lat: 34.0837,
        lon: 74.7973,
    },
    CityEntry {
        city: "aurangabad",
        lat: 19.8762,
        lon: 75.3433,
    },
    CityEntry {
        city: "coimbatore",
        lat: 11.0168,
        lon: 76.9558,
    },
    CityEntry {
        city: "madurai",
        lat: 9.9252,
        lon: 78.1198,
    },
    CityEntry {
        city: "jodhpur",
        lat: 26.2389,
        lon: 73.0243,
    },
    CityEntry {
        city: "guwahati",
        lat: 26.1445,
        lon: 91.7362,
    },
    CityEntry {
        city: "chandigarh",
        lat: 30.7333,
        lon: 76.7794,
    },
    CityEntry {
        city: "thiruvananthapuram",
        lat: 8.5241,
        lon: 76.9366,
    },
    CityEntry {
        city: "kochi",
        lat: 9.9312,
        lon: 76.2673,
    },
    CityEntry {
        city: "surat",
        lat: 21.1702,
        lon: 72.8311,
    },
    CityEntry {
        city: "mangalore",
        lat: 12.9141,
        lon: 74.8560,
    },
    // Astrology centers of India
    CityEntry {
        city: "ujjain",
        lat: 23.1765,
        lon: 75.7885,
    },
    CityEntry {
        city: "rishikesh",
        lat: 30.0869,
        lon: 78.2676,
    },
    CityEntry {
        city: "haridwar",
        lat: 29.9457,
        lon: 78.1642,
    },
    CityEntry {
        city: "tiruchirappalli",
        lat: 10.7905,
        lon: 78.7047,
    },
    CityEntry {
        city: "mysore",
        lat: 12.2958,
        lon: 76.6394,
    },
    CityEntry {
        city: "mysuru",
        lat: 12.2958,
        lon: 76.6394,
    },
    // ── South Asia ───────────────────────────────────────────────────
    CityEntry {
        city: "colombo",
        lat: 6.9271,
        lon: 79.8612,
    },
    CityEntry {
        city: "kathmandu",
        lat: 27.7172,
        lon: 85.3240,
    },
    CityEntry {
        city: "dhaka",
        lat: 23.8103,
        lon: 90.4125,
    },
    CityEntry {
        city: "karachi",
        lat: 24.8607,
        lon: 67.0011,
    },
    CityEntry {
        city: "lahore",
        lat: 31.5204,
        lon: 74.3587,
    },
    CityEntry {
        city: "islamabad",
        lat: 33.6844,
        lon: 73.0479,
    },
    // ── Southeast Asia ───────────────────────────────────────────────
    CityEntry {
        city: "singapore",
        lat: 1.3521,
        lon: 103.8198,
    },
    CityEntry {
        city: "bangkok",
        lat: 13.7563,
        lon: 100.5018,
    },
    CityEntry {
        city: "jakarta",
        lat: -6.2088,
        lon: 106.8456,
    },
    CityEntry {
        city: "kuala lumpur",
        lat: 3.1390,
        lon: 101.6869,
    },
    CityEntry {
        city: "manila",
        lat: 14.5995,
        lon: 120.9842,
    },
    CityEntry {
        city: "ho chi minh city",
        lat: 10.8231,
        lon: 106.6297,
    },
    CityEntry {
        city: "hanoi",
        lat: 21.0278,
        lon: 105.8342,
    },
    // ── East Asia ────────────────────────────────────────────────────
    CityEntry {
        city: "tokyo",
        lat: 35.6762,
        lon: 139.6503,
    },
    CityEntry {
        city: "osaka",
        lat: 34.6937,
        lon: 135.5023,
    },
    CityEntry {
        city: "beijing",
        lat: 39.9042,
        lon: 116.4074,
    },
    CityEntry {
        city: "shanghai",
        lat: 31.2304,
        lon: 121.4737,
    },
    CityEntry {
        city: "hong kong",
        lat: 22.3193,
        lon: 114.1694,
    },
    CityEntry {
        city: "taipei",
        lat: 25.0330,
        lon: 121.5654,
    },
    CityEntry {
        city: "seoul",
        lat: 37.5665,
        lon: 126.9780,
    },
    // ── Middle East ──────────────────────────────────────────────────
    CityEntry {
        city: "dubai",
        lat: 25.2048,
        lon: 55.2708,
    },
    CityEntry {
        city: "abu dhabi",
        lat: 24.4539,
        lon: 54.3773,
    },
    CityEntry {
        city: "riyadh",
        lat: 24.7136,
        lon: 46.6753,
    },
    CityEntry {
        city: "tehran",
        lat: 35.6892,
        lon: 51.3890,
    },
    CityEntry {
        city: "doha",
        lat: 25.2854,
        lon: 51.5310,
    },
    CityEntry {
        city: "muscat",
        lat: 23.5880,
        lon: 58.3829,
    },
    CityEntry {
        city: "baghdad",
        lat: 33.3128,
        lon: 44.3615,
    },
    CityEntry {
        city: "istanbul",
        lat: 41.0082,
        lon: 28.9784,
    },
    CityEntry {
        city: "jerusalem",
        lat: 31.7683,
        lon: 35.2137,
    },
    CityEntry {
        city: "tel aviv",
        lat: 32.0853,
        lon: 34.7818,
    },
    // ── Africa ───────────────────────────────────────────────────────
    CityEntry {
        city: "cairo",
        lat: 30.0444,
        lon: 31.2357,
    },
    CityEntry {
        city: "nairobi",
        lat: -1.2921,
        lon: 36.8219,
    },
    CityEntry {
        city: "lagos",
        lat: 6.5244,
        lon: 3.3792,
    },
    CityEntry {
        city: "johannesburg",
        lat: -26.2041,
        lon: 28.0473,
    },
    CityEntry {
        city: "cape town",
        lat: -33.9249,
        lon: 18.4241,
    },
    CityEntry {
        city: "addis ababa",
        lat: 9.0250,
        lon: 38.7469,
    },
    CityEntry {
        city: "casablanca",
        lat: 33.5731,
        lon: -7.5898,
    },
    // ── Europe ───────────────────────────────────────────────────────
    CityEntry {
        city: "london",
        lat: 51.5074,
        lon: -0.1278,
    },
    CityEntry {
        city: "paris",
        lat: 48.8566,
        lon: 2.3522,
    },
    CityEntry {
        city: "berlin",
        lat: 52.5200,
        lon: 13.4050,
    },
    CityEntry {
        city: "rome",
        lat: 41.9028,
        lon: 12.4964,
    },
    CityEntry {
        city: "madrid",
        lat: 40.4168,
        lon: -3.7038,
    },
    CityEntry {
        city: "amsterdam",
        lat: 52.3676,
        lon: 4.9041,
    },
    CityEntry {
        city: "brussels",
        lat: 50.8503,
        lon: 4.3517,
    },
    CityEntry {
        city: "vienna",
        lat: 48.2082,
        lon: 16.3738,
    },
    CityEntry {
        city: "zurich",
        lat: 47.3769,
        lon: 8.5417,
    },
    CityEntry {
        city: "athens",
        lat: 37.9838,
        lon: 23.7275,
    },
    CityEntry {
        city: "moscow",
        lat: 55.7558,
        lon: 37.6173,
    },
    CityEntry {
        city: "stockholm",
        lat: 59.3293,
        lon: 18.0686,
    },
    CityEntry {
        city: "oslo",
        lat: 59.9139,
        lon: 10.7522,
    },
    CityEntry {
        city: "helsinki",
        lat: 60.1699,
        lon: 24.9384,
    },
    CityEntry {
        city: "warsaw",
        lat: 52.2297,
        lon: 21.0122,
    },
    CityEntry {
        city: "prague",
        lat: 50.0755,
        lon: 14.4378,
    },
    CityEntry {
        city: "bucharest",
        lat: 44.4268,
        lon: 26.1025,
    },
    CityEntry {
        city: "lisbon",
        lat: 38.7223,
        lon: -9.1393,
    },
    CityEntry {
        city: "dublin",
        lat: 53.3498,
        lon: -6.2603,
    },
    CityEntry {
        city: "copenhagen",
        lat: 55.6761,
        lon: 12.5683,
    },
    CityEntry {
        city: "edinburgh",
        lat: 55.9533,
        lon: -3.1883,
    },
    CityEntry {
        city: "barcelona",
        lat: 41.3851,
        lon: 2.1734,
    },
    CityEntry {
        city: "munich",
        lat: 48.1351,
        lon: 11.5820,
    },
    CityEntry {
        city: "milan",
        lat: 45.4642,
        lon: 9.1900,
    },
    // ── Americas ─────────────────────────────────────────────────────
    CityEntry {
        city: "new york",
        lat: 40.7128,
        lon: -74.0060,
    },
    CityEntry {
        city: "los angeles",
        lat: 34.0522,
        lon: -118.2437,
    },
    CityEntry {
        city: "chicago",
        lat: 41.8781,
        lon: -87.6298,
    },
    CityEntry {
        city: "houston",
        lat: 29.7604,
        lon: -95.3698,
    },
    CityEntry {
        city: "phoenix",
        lat: 33.4484,
        lon: -112.0740,
    },
    CityEntry {
        city: "philadelphia",
        lat: 39.9526,
        lon: -75.1652,
    },
    CityEntry {
        city: "san antonio",
        lat: 29.4241,
        lon: -98.4936,
    },
    CityEntry {
        city: "san diego",
        lat: 32.7157,
        lon: -117.1611,
    },
    CityEntry {
        city: "san francisco",
        lat: 37.7749,
        lon: -122.4194,
    },
    CityEntry {
        city: "seattle",
        lat: 47.6062,
        lon: -122.3321,
    },
    CityEntry {
        city: "denver",
        lat: 39.7392,
        lon: -104.9903,
    },
    CityEntry {
        city: "boston",
        lat: 42.3601,
        lon: -71.0589,
    },
    CityEntry {
        city: "washington dc",
        lat: 38.9072,
        lon: -77.0369,
    },
    CityEntry {
        city: "atlanta",
        lat: 33.7490,
        lon: -84.3880,
    },
    CityEntry {
        city: "miami",
        lat: 25.7617,
        lon: -80.1918,
    },
    CityEntry {
        city: "dallas",
        lat: 32.7767,
        lon: -96.7970,
    },
    CityEntry {
        city: "toronto",
        lat: 43.6532,
        lon: -79.3832,
    },
    CityEntry {
        city: "vancouver",
        lat: 49.2827,
        lon: -123.1207,
    },
    CityEntry {
        city: "montreal",
        lat: 45.5017,
        lon: -73.5673,
    },
    CityEntry {
        city: "mexico city",
        lat: 19.4326,
        lon: -99.1332,
    },
    CityEntry {
        city: "sao paulo",
        lat: -23.5558,
        lon: -46.6396,
    },
    CityEntry {
        city: "rio de janeiro",
        lat: -22.9068,
        lon: -43.1729,
    },
    CityEntry {
        city: "buenos aires",
        lat: -34.6037,
        lon: -58.3816,
    },
    CityEntry {
        city: "bogota",
        lat: 4.7110,
        lon: -74.0721,
    },
    CityEntry {
        city: "lima",
        lat: -12.0464,
        lon: -77.0428,
    },
    CityEntry {
        city: "santiago",
        lat: -33.4489,
        lon: -70.6693,
    },
    // ── Oceania ──────────────────────────────────────────────────────
    CityEntry {
        city: "sydney",
        lat: -33.8688,
        lon: 151.2093,
    },
    CityEntry {
        city: "melbourne",
        lat: -37.8136,
        lon: 144.9631,
    },
    CityEntry {
        city: "brisbane",
        lat: -27.4698,
        lon: 153.0251,
    },
    CityEntry {
        city: "perth",
        lat: -31.9505,
        lon: 115.8605,
    },
    CityEntry {
        city: "auckland",
        lat: -36.8485,
        lon: 174.7633,
    },
    CityEntry {
        city: "wellington",
        lat: -41.2865,
        lon: 174.7762,
    },
];

/// Look up geographic coordinates for a city by name (case-insensitive).
///
/// Returns a `GeoLocation` with latitude and longitude, or `None` if not found.
///
/// # Example
/// ```
/// use xalen_houses::geocoding::city_coordinates;
/// let loc = city_coordinates("Pune").unwrap();
/// assert!((loc.lat_deg() - 18.52).abs() < 0.1);
/// assert!((loc.lon_deg() - 73.86).abs() < 0.1);
/// ```
pub fn city_coordinates(city: &str) -> Option<GeoLocation> {
    let lower = city.trim().to_lowercase();
    CITY_TABLE
        .iter()
        .find(|e| e.city == lower)
        .map(|e| GeoLocation::new(e.lat, e.lon))
}

/// Return a sorted list of all city names in the geocoding table.
pub fn available_cities() -> Vec<&'static str> {
    let mut cities: Vec<&str> = CITY_TABLE.iter().map(|e| e.city).collect();
    cities.sort();
    cities.dedup();
    cities
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pune_coordinates() {
        let loc = city_coordinates("Pune").unwrap();
        assert!(
            (loc.lat_deg() - 18.52).abs() < 0.1,
            "Pune lat: {}",
            loc.lat_deg()
        );
        assert!(
            (loc.lon_deg() - 73.86).abs() < 0.1,
            "Pune lon: {}",
            loc.lon_deg()
        );
    }

    #[test]
    fn mumbai_coordinates() {
        let loc = city_coordinates("mumbai").unwrap();
        assert!((loc.lat_deg() - 19.08).abs() < 0.1);
        assert!((loc.lon_deg() - 72.88).abs() < 0.1);
    }

    #[test]
    fn case_insensitive() {
        assert!(city_coordinates("NEW YORK").is_some());
        assert!(city_coordinates("new york").is_some());
        assert!(city_coordinates("New York").is_some());
    }

    #[test]
    fn unknown_city_returns_none() {
        assert!(city_coordinates("Atlantis").is_none());
        assert!(city_coordinates("").is_none());
    }

    #[test]
    fn all_major_indian_cities() {
        let indian = [
            "Pune",
            "Mumbai",
            "Delhi",
            "Chennai",
            "Kolkata",
            "Bangalore",
            "Hyderabad",
            "Ahmedabad",
            "Jaipur",
            "Lucknow",
            "Nagpur",
            "Indore",
            "Surat",
            "Varanasi",
            "Ujjain",
            "Kochi",
        ];
        for city in &indian {
            assert!(
                city_coordinates(city).is_some(),
                "missing Indian city: {city}"
            );
        }
    }

    #[test]
    fn global_capitals() {
        let capitals = [
            "London",
            "Paris",
            "Berlin",
            "Tokyo",
            "Beijing",
            "Moscow",
            "Cairo",
            "Nairobi",
            "Sydney",
            "Buenos Aires",
            "Bogota",
        ];
        for city in &capitals {
            assert!(city_coordinates(city).is_some(), "missing capital: {city}");
        }
    }

    #[test]
    fn new_york_coordinates() {
        let loc = city_coordinates("New York").unwrap();
        assert!((loc.lat_deg() - 40.71).abs() < 0.1);
        assert!((loc.lon_deg() - (-74.01)).abs() < 0.1);
    }

    #[test]
    fn southern_hemisphere() {
        let loc = city_coordinates("Sydney").unwrap();
        assert!(loc.lat_deg() < 0.0, "Sydney should be negative latitude");

        let loc2 = city_coordinates("Sao Paulo").unwrap();
        assert!(
            loc2.lat_deg() < 0.0,
            "Sao Paulo should be negative latitude"
        );
    }

    #[test]
    fn western_hemisphere() {
        let loc = city_coordinates("Los Angeles").unwrap();
        assert!(loc.lon_deg() < 0.0, "LA should be negative longitude");
    }

    #[test]
    fn at_least_100_cities() {
        let cities = available_cities();
        assert!(
            cities.len() >= 100,
            "table has {} cities, need >= 100",
            cities.len()
        );
    }

    #[test]
    fn alias_cities_same_coords() {
        let b1 = city_coordinates("Bangalore").unwrap();
        let b2 = city_coordinates("Bengaluru").unwrap();
        assert!((b1.lat_deg() - b2.lat_deg()).abs() < 0.001);
        assert!((b1.lon_deg() - b2.lon_deg()).abs() < 0.001);

        let d1 = city_coordinates("Delhi").unwrap();
        let d2 = city_coordinates("New Delhi").unwrap();
        assert!((d1.lat_deg() - d2.lat_deg()).abs() < 0.001);
    }

    #[test]
    fn ujjain_astrology_center() {
        // Ujjain is the reference point for Indian astronomical calculations
        let loc = city_coordinates("Ujjain").unwrap();
        assert!((loc.lat_deg() - 23.18).abs() < 0.1);
        assert!((loc.lon_deg() - 75.79).abs() < 0.1);
    }
}
