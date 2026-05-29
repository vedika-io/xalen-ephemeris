//! Persian/Arabic astrology — Jarbakhtar chronocrators and Tasyir directions.
//!
//! **Jarbakhtar** (جاربختار): each planet rules a period whose length equals its
//! "minor years" (Abu Ma'shar). The 7 periods sum to 129 years, then the cycle
//! repeats. The *starting* planet is the Almuten (strongest planet in the
//! chart); the order then follows the Chaldean sequence from that starting
//! point: Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon.
//!
//! **Tasyir** (تسيير, primary directions — simplified): the arc in ecliptic
//! degrees between a significator and a promissor equals the number of years
//! until the promised event, at a rate of 1 degree per year.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Chaldean order of the 7 planets and their minor years.
const CHALDEAN_ORDER: [(&str, u32); 7] = [
    ("Saturn", 30),
    ("Jupiter", 12),
    ("Mars", 15),
    ("Sun", 19),
    ("Venus", 8),
    ("Mercury", 20),
    ("Moon", 25),
];

/// Total cycle length (sum of minor years): 30+12+15+19+8+20+25 = 129.
pub const CYCLE_YEARS: u32 = 129;

// ---------------------------------------------------------------------------
// Jarbakhtar
// ---------------------------------------------------------------------------

/// One Jarbakhtar planetary period.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JarbakhtarPeriod {
    /// Planet ruling this period.
    pub planet: &'static str,
    /// Duration in years (= the planet's minor years).
    pub years: u32,
    /// Absolute start year (fractional, from birth year).
    pub start_year: f64,
    /// Absolute end year (fractional).
    pub end_year: f64,
}

/// Find the index of `starting_planet` in `CHALDEAN_ORDER` (case-insensitive).
/// Returns `None` if the planet name is not recognised.
fn chaldean_index(starting_planet: &str) -> Option<usize> {
    let lower = starting_planet.to_ascii_lowercase();
    CHALDEAN_ORDER
        .iter()
        .position(|(name, _)| name.to_ascii_lowercase() == lower)
}

/// Compute a full Jarbakhtar sequence starting from `birth_year`.
///
/// `starting_planet` is the Almuten — the strongest planet in the nativity.
/// The function cycles through the Chaldean order beginning at that planet.
///
/// Returns one full 129-year cycle (7 periods). Panics-free: returns an empty
/// `Vec` if `starting_planet` is not one of the seven traditional planets.
pub fn compute_jarbakhtar(birth_year: f64, starting_planet: &str) -> Vec<JarbakhtarPeriod> {
    let start_idx = match chaldean_index(starting_planet) {
        Some(i) => i,
        None => return Vec::new(),
    };

    let mut periods = Vec::with_capacity(7);
    let mut cursor = birth_year;

    for offset in 0..7 {
        let idx = (start_idx + offset) % 7;
        let (planet, years) = CHALDEAN_ORDER[idx];
        let end = cursor + years as f64;
        periods.push(JarbakhtarPeriod {
            planet,
            years,
            start_year: cursor,
            end_year: end,
        });
        cursor = end;
    }

    periods
}

// ---------------------------------------------------------------------------
// Tasyir
// ---------------------------------------------------------------------------

/// Simplified Tasyir (primary direction): compute the arc between a
/// significator degree and a promissor degree.
///
/// At the classical rate of 1 degree = 1 year, the returned value is both the
/// arc in ecliptic degrees **and** the number of years until the event.
///
/// Both inputs should be in the range 0..360. The arc is always positive and
/// takes the shortest path along the ecliptic.
pub fn tasyir_arc(significator: f64, promissor: f64) -> f64 {
    let diff = (promissor - significator).rem_euclid(360.0);
    if diff > 180.0 { 360.0 - diff } else { diff }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_years_is_129() {
        let sum: u32 = CHALDEAN_ORDER.iter().map(|(_, y)| y).sum();
        assert_eq!(sum, CYCLE_YEARS);
        assert_eq!(CYCLE_YEARS, 129);
    }

    #[test]
    fn jarbakhtar_saturn_start() {
        let periods = compute_jarbakhtar(1990.0, "Saturn");
        assert_eq!(periods.len(), 7);
        assert_eq!(periods[0].planet, "Saturn");
        assert_eq!(periods[0].years, 30);
        assert!((periods[0].start_year - 1990.0).abs() < f64::EPSILON);
        assert!((periods[0].end_year - 2020.0).abs() < f64::EPSILON);
        assert_eq!(periods[1].planet, "Jupiter");
        assert_eq!(periods[6].planet, "Moon");
    }

    #[test]
    fn jarbakhtar_venus_start() {
        let periods = compute_jarbakhtar(2000.0, "Venus");
        assert_eq!(periods.len(), 7);
        assert_eq!(periods[0].planet, "Venus");
        assert_eq!(periods[0].years, 8);
        assert!((periods[0].end_year - 2008.0).abs() < f64::EPSILON);
        // After Venus: Mercury, Moon, Saturn, Jupiter, Mars, Sun
        assert_eq!(periods[1].planet, "Mercury");
        assert_eq!(periods[2].planet, "Moon");
        assert_eq!(periods[3].planet, "Saturn");
    }

    #[test]
    fn jarbakhtar_case_insensitive() {
        let a = compute_jarbakhtar(2000.0, "jupiter");
        let b = compute_jarbakhtar(2000.0, "JUPITER");
        let c = compute_jarbakhtar(2000.0, "Jupiter");
        assert_eq!(a.len(), 7);
        assert_eq!(a[0].planet, b[0].planet);
        assert_eq!(b[0].planet, c[0].planet);
    }

    #[test]
    fn jarbakhtar_unknown_planet_returns_empty() {
        let periods = compute_jarbakhtar(2000.0, "Pluto");
        assert!(periods.is_empty());
    }

    #[test]
    fn jarbakhtar_full_cycle_spans_129_years() {
        let periods = compute_jarbakhtar(1900.0, "Saturn");
        let total: f64 = periods.last().unwrap().end_year - periods[0].start_year;
        assert!((total - 129.0).abs() < f64::EPSILON);
    }

    #[test]
    fn jarbakhtar_periods_are_contiguous() {
        let periods = compute_jarbakhtar(1985.5, "Mars");
        for i in 1..periods.len() {
            assert!(
                (periods[i].start_year - periods[i - 1].end_year).abs() < f64::EPSILON,
                "gap between period {} and {}",
                i - 1,
                i
            );
        }
    }

    #[test]
    fn tasyir_same_degree_is_zero() {
        assert!((tasyir_arc(120.0, 120.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn tasyir_simple_arc() {
        // 30 degrees apart => 30 years
        assert!((tasyir_arc(100.0, 130.0) - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn tasyir_wraps_around() {
        // 350 -> 10 is 20 degrees (shortest path)
        assert!((tasyir_arc(350.0, 10.0) - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn tasyir_max_is_180() {
        // Exactly opposite = 180 degrees
        assert!((tasyir_arc(0.0, 180.0) - 180.0).abs() < f64::EPSILON);
        assert!((tasyir_arc(90.0, 270.0) - 180.0).abs() < f64::EPSILON);
    }

    #[test]
    fn tasyir_reverse_direction() {
        // Arc is always positive regardless of direction
        let a = tasyir_arc(100.0, 130.0);
        let b = tasyir_arc(130.0, 100.0);
        assert!((a - b).abs() < f64::EPSILON);
    }
}
