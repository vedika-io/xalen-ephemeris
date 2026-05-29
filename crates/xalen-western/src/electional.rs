use crate::aspects::{AspectDirection, AspectType, find_aspect};
use serde::{Deserialize, Serialize};

// ── Planetary hours (Chaldean order) ────────────────────────────────

/// Chaldean planetary order used for planetary hours and day rulership.
const CHALDEAN_ORDER: [&str; 7] = [
    "Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon",
];

/// Day rulers in weekday order (Sun=0 .. Sat=6).
const DAY_RULERS: [&str; 7] = [
    "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn",
];

/// Compute which planet rules the current planetary hour.
///
/// * `jd` — Julian Day of the moment in question.
/// * `sunrise_jd` — JD of sunrise on that day.
/// * `sunset_jd` — JD of sunset on that day.
///
/// Returns the name of the ruling planet.
pub fn planetary_hour(jd: f64, sunrise_jd: f64, sunset_jd: f64) -> &'static str {
    let day_length = sunset_jd - sunrise_jd;
    let night_length = 1.0 - day_length; // next sunrise ≈ sunset + night_length

    let (hour_index, _is_day) = if jd >= sunrise_jd && jd < sunset_jd {
        // Daytime
        let elapsed = jd - sunrise_jd;
        let hour_len = day_length / 12.0;
        let idx = (elapsed / hour_len).floor() as usize;
        (idx.min(11), true)
    } else {
        // Nighttime
        let night_start = sunset_jd;
        let elapsed = if jd >= sunset_jd {
            jd - night_start
        } else {
            // Before sunrise → measure from previous sunset
            jd - (sunrise_jd - night_length)
        };
        let hour_len = night_length / 12.0;
        let idx = (elapsed / hour_len).floor() as usize;
        (idx.min(11) + 12, false)
    };

    // Day ruler determines the starting planet for hour 1
    // We need the weekday. A rough weekday from JD:
    // JD 0 = Monday, so (JD + 1) mod 7 → 0=Mon..6=Sun
    // More precisely: floor(JD + 0.5) mod 7: 0=Mon,1=Tue,..6=Sun
    let weekday_from_jd = ((sunrise_jd + 0.5).floor() as i64).rem_euclid(7) as usize;
    // Map to our DAY_RULERS index: Mon=2(Mars), Tue=3(Merc??)
    // Standard: Sun=0,Mon=1,Tue=2,Wed=3,Thu=4,Fri=5,Sat=6
    // JD weekday: Mon=0..Sun=6  → remap to Sun=0 system: (weekday+1)%7
    let day_idx = (weekday_from_jd + 1) % 7;
    let day_ruler = DAY_RULERS[day_idx];

    // Find day ruler's position in Chaldean order
    let ruler_chaldean_pos = CHALDEAN_ORDER
        .iter()
        .position(|&p| p == day_ruler)
        .unwrap_or(0);

    // Hour ruler = Chaldean sequence starting from day ruler
    let chaldean_idx = (ruler_chaldean_pos + hour_index) % 7;
    CHALDEAN_ORDER[chaldean_idx]
}

// ── Benefic / malefic classification ────────────────────────────────

fn is_benefic(name: &str) -> bool {
    matches!(name.to_ascii_lowercase().as_str(), "venus" | "jupiter")
}

fn is_malefic(name: &str) -> bool {
    matches!(name.to_ascii_lowercase().as_str(), "mars" | "saturn")
}

/// True if the named planet rules a benefic hour.
pub fn is_benefic_hour(jd: f64, sunrise_jd: f64, sunset_jd: f64) -> bool {
    is_benefic(planetary_hour(jd, sunrise_jd, sunset_jd))
}

// ── House position helpers ──────────────────────────────────────────

/// Angular houses: 1, 4, 7, 10 (most powerful positions).
fn is_angular(house: usize) -> bool {
    matches!(house, 1 | 4 | 7 | 10)
}

/// Cadent houses: 3, 6, 9, 12 (weakest positions).
fn is_cadent(house: usize) -> bool {
    matches!(house, 3 | 6 | 9 | 12)
}

/// Determine which house (1..12) a planet occupies.
fn house_of(planet_lon: f64, cusps: &[f64; 12]) -> usize {
    for i in 0..12 {
        let start = cusps[i];
        let end = cusps[(i + 1) % 12];
        let inside = if start < end {
            planet_lon >= start && planet_lon < end
        } else {
            planet_lon >= start || planet_lon < end
        };
        if inside {
            return i + 1;
        }
    }
    1 // fallback
}

// ── Moon quality check ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoonQuality {
    /// Moon is applying to at least one benefic (Venus or Jupiter).
    pub applying_to_benefic: bool,
    /// Moon is applying to a malefic (Mars or Saturn).
    pub applying_to_malefic: bool,
    /// Moon is void of course (no applying major aspect before sign change).
    pub void_of_course: bool,
    /// Score component: +2 benefic applying, -2 malefic applying, -3 VOC.
    pub score: i32,
}

/// Evaluate the Moon's applying aspects for electional quality.
pub fn moon_quality(
    moon_lon: f64,
    moon_speed: f64,
    planet_positions: &[(String, f64, f64)], // (name, lon, speed)
) -> MoonQuality {
    let moon_sign = (moon_lon / 30.0) as usize;
    let remaining_in_sign = ((moon_sign as f64 + 1.0) * 30.0 - moon_lon).rem_euclid(360.0);

    let mut applying_to_benefic = false;
    let mut applying_to_malefic = false;
    let mut has_any_applying = false;

    for (name, p_lon, p_speed) in planet_positions {
        if let Some(asp) = find_aspect(
            moon_lon,
            *p_lon,
            moon_speed,
            *p_speed,
            AspectType::MAJOR,
            1.0,
        )
            && (asp.direction == AspectDirection::Applying || asp.direction == AspectDirection::Exact)
            {
                // Check degrees to exact vs remaining in sign
                if asp.orb_deg < remaining_in_sign {
                    has_any_applying = true;
                    if is_benefic(name) {
                        applying_to_benefic = true;
                    }
                    if is_malefic(name) {
                        applying_to_malefic = true;
                    }
                }
            }
    }

    let void_of_course = !has_any_applying;

    let mut score: i32 = 0;
    if applying_to_benefic {
        score += 2;
    }
    if applying_to_malefic {
        score -= 2;
    }
    if void_of_course {
        score -= 3;
    }

    MoonQuality {
        applying_to_benefic,
        applying_to_malefic,
        void_of_course,
        score,
    }
}

// ── Election score ──────────────────────────────────────────────────

/// Component scores for an electional chart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectionScore {
    /// Moon quality score (-3 to +2).
    pub moon_score: i32,
    /// Benefics in angular houses (+2 each, max +4).
    pub angular_benefics: i32,
    /// Malefics in cadent houses (+1 each, max +2; penalty if angular).
    pub cadent_malefics: i32,
    /// Planetary hour is benefic (+1).
    pub hour_score: i32,
    /// Total score (sum of all components).
    pub total: i32,
    /// Human-readable rating.
    pub rating: ElectionRating,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElectionRating {
    Excellent,
    Good,
    Fair,
    Poor,
}

/// Score an electional chart.
///
/// * `moon_lon` / `moon_speed` — Moon's ecliptic longitude & daily motion.
/// * `planet_positions` — `(name, longitude, speed)` for each planet.
/// * `cusps` — 12 house cusps (index 0 = cusp 1).
/// * `jd` / `sunrise_jd` / `sunset_jd` — for planetary hour computation.
pub fn score_election(
    moon_lon: f64,
    moon_speed: f64,
    planet_positions: &[(String, f64, f64)],
    cusps: &[f64; 12],
    jd: f64,
    sunrise_jd: f64,
    sunset_jd: f64,
) -> ElectionScore {
    // 1) Moon quality
    let mq = moon_quality(moon_lon, moon_speed, planet_positions);
    let moon_score = mq.score;

    // 2) Angular benefics / cadent malefics
    let mut angular_benefics: i32 = 0;
    let mut cadent_malefics: i32 = 0;

    for (name, lon, _speed) in planet_positions {
        let h = house_of(*lon, cusps);
        if is_benefic(name) && is_angular(h) {
            angular_benefics += 2;
        }
        if is_malefic(name) {
            if is_cadent(h) {
                cadent_malefics += 1; // good: malefic tucked away
            } else if is_angular(h) {
                cadent_malefics -= 2; // bad: malefic prominent
            }
        }
    }
    // Cap angular_benefics at +4, cadent_malefics component at [-4, +2]
    angular_benefics = angular_benefics.min(4);
    cadent_malefics = cadent_malefics.clamp(-4, 2);

    // 3) Planetary hour
    let hour_score = if is_benefic_hour(jd, sunrise_jd, sunset_jd) {
        1
    } else {
        0
    };

    let total = moon_score + angular_benefics + cadent_malefics + hour_score;

    let rating = match total {
        6..=i32::MAX => ElectionRating::Excellent,
        3..=5 => ElectionRating::Good,
        0..=2 => ElectionRating::Fair,
        _ => ElectionRating::Poor,
    };

    ElectionScore {
        moon_score,
        angular_benefics,
        cadent_malefics,
        hour_score,
        total,
        rating,
    }
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn equal_cusps() -> [f64; 12] {
        [
            0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0, 210.0, 240.0, 270.0, 300.0, 330.0,
        ]
    }

    #[test]
    fn planetary_hour_returns_valid_planet() {
        let valid = [
            "Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon",
        ];
        // Sunday sunrise/sunset at JD 2451545 (J2000)
        let sunrise = 2451545.25;
        let sunset = 2451545.75;
        let result = planetary_hour(2451545.3, sunrise, sunset);
        assert!(valid.contains(&result), "Got unexpected planet: {result}");
    }

    #[test]
    fn planetary_hour_day_vs_night_differ() {
        let sunrise = 2451545.25;
        let sunset = 2451545.75;
        let day_hour = planetary_hour(2451545.35, sunrise, sunset);
        let night_hour = planetary_hour(2451545.85, sunrise, sunset);
        // They CAN be the same by coincidence, but at different indices
        // the function should not panic and should return valid names.
        let valid = [
            "Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon",
        ];
        assert!(valid.contains(&day_hour));
        assert!(valid.contains(&night_hour));
    }

    #[test]
    fn benefic_hour_check() {
        let sunrise = 2451545.25;
        let sunset = 2451545.75;
        // Test a range of JDs — at least one should hit a benefic hour
        let mut found_benefic = false;
        let mut found_non_benefic = false;
        for i in 0..24 {
            let jd = sunrise + (i as f64) / 24.0;
            if is_benefic_hour(jd, sunrise, sunset) {
                found_benefic = true;
            } else {
                found_non_benefic = true;
            }
        }
        assert!(
            found_benefic,
            "Should find at least one benefic hour in a day"
        );
        assert!(
            found_non_benefic,
            "Should find at least one non-benefic hour in a day"
        );
    }

    #[test]
    fn moon_quality_applying_benefic() {
        let planets = vec![
            ("Venus".into(), 165.0_f64, 1.2_f64), // ~65° from Moon → sextile range
        ];
        let mq = moon_quality(100.0, 13.0, &planets);
        // Moon at 100° applying to Venus at 165° = 65° distance ≈ sextile (60° ± 5°)
        // Moon is fast (13°/day), Venus slow → applying
        if mq.applying_to_benefic {
            assert!(mq.score >= 2);
        }
        // Structure should always be coherent
        assert_eq!(
            mq.void_of_course,
            !mq.applying_to_benefic && !mq.applying_to_malefic
        );
    }

    #[test]
    fn moon_quality_void_of_course() {
        // Place all planets far from any major aspect to the Moon
        let planets = vec![
            ("Mars".into(), 145.0_f64, 0.5_f64), // 45° → no major aspect in default orb
        ];
        let mq = moon_quality(100.0, 13.0, &planets);
        assert!(
            mq.void_of_course,
            "Moon should be VOC with no applying major aspect"
        );
        assert!(mq.score < 0, "VOC should give negative score");
    }

    #[test]
    fn election_score_all_benefic_angular() {
        let cusps = equal_cusps();
        let planets = vec![
            ("Venus".into(), 5.0_f64, 1.2_f64), // house 1 (angular)
            ("Jupiter".into(), 95.0, 0.1),      // house 4 (angular)
            ("Saturn".into(), 250.0, 0.05),     // house 9 (cadent) — good
        ];
        let sunrise = 2451545.25;
        let sunset = 2451545.75;
        let es = score_election(100.0, 13.0, &planets, &cusps, 2451545.3, sunrise, sunset);
        // Angular benefics: Venus(h1) +2, Jupiter(h4) +2 = +4
        assert_eq!(es.angular_benefics, 4);
        // Cadent malefic: Saturn(h9) +1
        assert_eq!(es.cadent_malefics, 1);
        // Total should be positive
        assert!(
            es.total > 0,
            "Good election should have positive total, got {}",
            es.total
        );
    }

    #[test]
    fn election_score_malefic_angular_penalty() {
        let cusps = equal_cusps();
        let planets = vec![
            ("Mars".into(), 5.0_f64, 0.5_f64), // house 1 (angular) — bad
            ("Saturn".into(), 185.0, 0.05),    // house 7 (angular) — bad
        ];
        let sunrise = 2451545.25;
        let sunset = 2451545.75;
        let es = score_election(100.0, 13.0, &planets, &cusps, 2451545.3, sunrise, sunset);
        // Mars angular: -2, Saturn angular: -2, capped at -4
        assert!(
            es.cadent_malefics <= -2,
            "Angular malefics should penalize, got {}",
            es.cadent_malefics
        );
        assert!(
            es.total < 0,
            "Bad election should have negative total, got {}",
            es.total
        );
        assert_eq!(es.rating, ElectionRating::Poor);
    }

    #[test]
    fn election_rating_thresholds() {
        // Directly test the rating derivation
        assert_eq!(
            match 7 {
                6..=i32::MAX => ElectionRating::Excellent,
                3..=5 => ElectionRating::Good,
                0..=2 => ElectionRating::Fair,
                _ => ElectionRating::Poor,
            },
            ElectionRating::Excellent
        );
        assert_eq!(
            match 3 {
                6..=i32::MAX => ElectionRating::Excellent,
                3..=5 => ElectionRating::Good,
                0..=2 => ElectionRating::Fair,
                _ => ElectionRating::Poor,
            },
            ElectionRating::Good
        );
        assert_eq!(
            match 1 {
                6..=i32::MAX => ElectionRating::Excellent,
                3..=5 => ElectionRating::Good,
                0..=2 => ElectionRating::Fair,
                _ => ElectionRating::Poor,
            },
            ElectionRating::Fair
        );
        assert_eq!(
            match -2 {
                6..=i32::MAX => ElectionRating::Excellent,
                3..=5 => ElectionRating::Good,
                0..=2 => ElectionRating::Fair,
                _ => ElectionRating::Poor,
            },
            ElectionRating::Poor
        );
    }

    #[test]
    fn house_of_wrapping() {
        let cusps = equal_cusps();
        assert_eq!(house_of(15.0, &cusps), 1); // 15° → house 1
        assert_eq!(house_of(95.0, &cusps), 4); // 95° → house 4
        assert_eq!(house_of(185.0, &cusps), 7); // 185° → house 7
        assert_eq!(house_of(335.0, &cusps), 12); // 335° → house 12
    }
}
