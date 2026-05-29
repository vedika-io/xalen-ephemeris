use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Tajaka Yoga enumeration per Neelakanthi / Tajaka Paddhati
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TajakaYoga {
    /// Planet in own or exalted sign in the annual chart.
    Ikkabal,
    /// All planets in kendras (1, 4, 7, 10).
    Induvara,
    /// Faster planet applying to slower within orb — the most important Tajaka yoga.
    Ithasala,
    /// Separating aspect (opposite of Ithasala).
    Ishrafa,
    /// Like Ithasala but through a third (intermediary) planet.
    Nakta,
    /// Mutual reception (parivartana) in the annual chart.
    Yamaya,
    /// Like Ithasala but with a wide orb (beyond standard orb but within
    /// extended orb).
    Manau,
    /// Ithasala where one planet is retrograde.
    Kamboola,
    /// Like Kamboola but the retrograde planet is in debilitation.
    Gairi,
    /// Planet separating from its debilitation lord.
    Khalasara,
    /// Cancellation of Ithasala by an intervening planet.
    Radda,
    /// Planet hemmed between two malefics (papa-kartari in Tajaka context).
    Duhphali,
    /// Planet transferring light between two other planets.
    Dutthotthadabira,
    /// Planet escaping combustion.
    Tambira,
    /// Mutual enmity between two planets in the annual chart.
    Kuttha,
    /// Planet hemmed between any two planets (Tajaka Durudhura).
    DurudhuraTajaka,
}

// ---------------------------------------------------------------------------
// Aspect types recognized in Tajaka
// ---------------------------------------------------------------------------

/// Major Ptolemaic aspect types used in Tajaka yoga detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TajakaAspect {
    Conjunction, // 0 degrees
    Sextile,     // 60 degrees
    Square,      // 90 degrees
    Trine,       // 120 degrees
    Opposition,  // 180 degrees
}

impl TajakaAspect {
    /// The exact angle for this aspect.
    pub fn angle(&self) -> f64 {
        match self {
            TajakaAspect::Conjunction => 0.0,
            TajakaAspect::Sextile => 60.0,
            TajakaAspect::Square => 90.0,
            TajakaAspect::Trine => 120.0,
            TajakaAspect::Opposition => 180.0,
        }
    }

    /// All aspect types in order.
    pub const ALL: [TajakaAspect; 5] = [
        TajakaAspect::Conjunction,
        TajakaAspect::Sextile,
        TajakaAspect::Square,
        TajakaAspect::Trine,
        TajakaAspect::Opposition,
    ];
}

// ---------------------------------------------------------------------------
// Ithasala result
// ---------------------------------------------------------------------------

/// Result of an Ithasala (applying aspect) check between two planets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IthasalaResult {
    /// The type of aspect being formed.
    pub aspect: TajakaAspect,
    /// Angular distance remaining to exact aspect (degrees).
    pub distance_to_exact: f64,
    /// Whether the faster planet is genuinely gaining on the slower.
    pub is_applying: bool,
    /// Estimated days until exact aspect (distance / relative speed).
    pub days_to_exact: f64,
}

// ---------------------------------------------------------------------------
// Muntha
// ---------------------------------------------------------------------------

/// Compute the Muntha sign for a given year.
/// Muntha advances one sign per year from the natal ascendant sign.
pub fn compute_muntha(natal_asc_sign: usize, age: u32) -> usize {
    (natal_asc_sign + age as usize) % 12
}

/// Return the lord of the Muntha sign (Vedic rulership).
pub fn muntha_lord(muntha_sign: usize) -> &'static str {
    match muntha_sign % 12 {
        0 | 7 => "Mars",
        1 | 6 => "Venus",
        2 | 5 => "Mercury",
        3 => "Moon",
        4 => "Sun",
        8 | 11 => "Jupiter",
        9 | 10 => "Saturn",
        _ => unreachable!(),
    }
}

/// Check which planets aspect the Muntha sign from their positions.
///
/// Returns a list of (planet_name, aspect_type) for all planets that
/// form a Ptolemaic aspect to the Muntha sign. `planet_signs` maps
/// planet name to 0-based sign index.
pub fn muntha_aspects<'a>(
    muntha_sign: usize,
    planet_signs: &'a [(&'a str, usize)],
) -> Vec<(&'a str, TajakaAspect)> {
    let m = muntha_sign % 12;
    let mut results = Vec::new();
    for &(name, sign) in planet_signs {
        let s = sign % 12;
        let diff = ((s as i32 - m as i32).rem_euclid(12)) as usize;
        let aspect = match diff {
            0 => Some(TajakaAspect::Conjunction),
            2 | 10 => Some(TajakaAspect::Sextile),
            3 | 9 => Some(TajakaAspect::Square),
            4 | 8 => Some(TajakaAspect::Trine),
            6 => Some(TajakaAspect::Opposition),
            _ => None,
        };
        if let Some(a) = aspect {
            results.push((name, a));
        }
    }
    results
}

// ---------------------------------------------------------------------------
// Saham (Tajaka lots)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Saham {
    pub name: &'static str,
    pub longitude_deg: f64,
}

/// Compute Tajaka Sahams using actual planet positions.
///
/// Formulas verified against Tajik Neelakanthi / Encyclopedia of Vedic Astrology
/// (Dr. Shanker Adawal, Ch. V Part 2) and multiple corroborating sources.
///
/// Web-verified:
/// - https://tagtoadawal.blogspot.com/2013/01/encyclopedia-of-vedic-astrology-tajik_2654.html
/// - https://astrosutras.in/index.php/2025/03/07/sahama-in-tajika-shastra-calculation-significance/
/// - https://blog.indianastrologysoftware.com/predictive-technique-of-varshaphal/
/// - https://www.astrogle.com/astrology/important-role-of-sahams-in-varshaphala-annual-chart.html
///
/// Day/night reversal rule: for day charts A - B + C; for night charts B - A + C.
pub fn compute_sahams(
    asc: f64,
    sun: f64,
    moon: f64,
    is_day: bool,
    planet_positions: &HashMap<&str, f64>,
) -> Vec<Saham> {
    let _mercury = *planet_positions.get("Mercury").unwrap_or(&0.0);
    let jupiter = *planet_positions.get("Jupiter").unwrap_or(&0.0);
    let venus = *planet_positions.get("Venus").unwrap_or(&0.0);
    let saturn = *planet_positions.get("Saturn").unwrap_or(&0.0);
    let mars = *planet_positions.get("Mars").unwrap_or(&0.0);

    // Punya must be computed first because Yashas and Mitra depend on it.
    // Punya (Fortune): Day = Moon - Sun + ASC, Night = Sun - Moon + ASC
    let punya = if is_day {
        (moon - sun + asc).rem_euclid(360.0)
    } else {
        (sun - moon + asc).rem_euclid(360.0)
    };

    vec![
        // 1. Punya (Fortune/Good Deeds): Day = Moon - Sun + ASC
        //    Tajik Neelakanthi standard. Same as Part of Fortune in western tradition.
        Saham {
            name: "Punya (Fortune)",
            longitude_deg: punya,
        },
        // 2. Vidya (Education): Day = Sun - Moon + ASC, Night = Moon - Sun + ASC
        //    Per Tajik Neelakanthi (the inverse of Punya).
        //    Note: some modern software uses Mercury instead of Moon; we follow
        //    the classical Neelakanthi text.
        Saham {
            name: "Vidya (Education)",
            longitude_deg: if is_day {
                (sun - moon + asc).rem_euclid(360.0)
            } else {
                (moon - sun + asc).rem_euclid(360.0)
            },
        },
        // 3. Yashas (Fame/Keerti): Day = Jupiter - Punya + ASC
        //    Night = Punya - Jupiter + ASC
        //    Per Tajik Neelakanthi: compound saham depending on Punya.
        Saham {
            name: "Yashas (Fame)",
            longitude_deg: if is_day {
                (jupiter - punya + asc).rem_euclid(360.0)
            } else {
                (punya - jupiter + asc).rem_euclid(360.0)
            },
        },
        // 4. Mitra (Friendship): Day = Jupiter - Punya + Venus
        //    Night = Punya - Jupiter + Venus
        //    Per Tajik Neelakanthi: compound saham using Punya + Jupiter + Venus.
        Saham {
            name: "Mitra (Friendship)",
            longitude_deg: if is_day {
                (jupiter - punya + venus).rem_euclid(360.0)
            } else {
                (punya - jupiter + venus).rem_euclid(360.0)
            },
        },
        // 5. Gaurav (Respect/Dignity): Day = Jupiter - Moon + Sun
        //    Night = Moon - Jupiter + Sun
        //    Per Tajik Neelakanthi / multiple web sources.
        Saham {
            name: "Gaurav (Respect)",
            longitude_deg: if is_day {
                (jupiter - moon + sun).rem_euclid(360.0)
            } else {
                (moon - jupiter + sun).rem_euclid(360.0)
            },
        },
        // 6. Mahatmya (Greatness): Day = Punya - Mars + ASC
        //    Night = Mars - Punya + ASC
        //    Per Tajik Neelakanthi.
        Saham {
            name: "Mahatmya (Greatness)",
            longitude_deg: if is_day {
                (punya - mars + asc).rem_euclid(360.0)
            } else {
                (mars - punya + asc).rem_euclid(360.0)
            },
        },
        // 7. Pitri (Father): Day = Saturn - Sun + ASC
        //    Night = Sun - Saturn + ASC
        //    Per Tajik Neelakanthi (same as Rajya Saham).
        Saham {
            name: "Pitri (Father)",
            longitude_deg: if is_day {
                (saturn - sun + asc).rem_euclid(360.0)
            } else {
                (sun - saturn + asc).rem_euclid(360.0)
            },
        },
        // 8. Matri (Mother): Day = Moon - Venus + ASC
        //    Night = Venus - Moon + ASC
        //    Per Tajik Neelakanthi.
        Saham {
            name: "Matri (Mother)",
            longitude_deg: if is_day {
                (moon - venus + asc).rem_euclid(360.0)
            } else {
                (venus - moon + asc).rem_euclid(360.0)
            },
        },
        // 9. Putra (Children/Progeny): Jupiter - Moon + ASC
        //    Not day/night reversed per Neelakanthi.
        Saham {
            name: "Putra (Children)",
            longitude_deg: (jupiter - moon + asc).rem_euclid(360.0),
        },
    ]
}

// ---------------------------------------------------------------------------
// Angular distance utility
// ---------------------------------------------------------------------------

/// Shortest angular distance between two ecliptic longitudes (0-180).
fn angular_dist(a: f64, b: f64) -> f64 {
    let d = (b - a).rem_euclid(360.0);
    if d > 180.0 { 360.0 - d } else { d }
}

/// Check if `angle` is within `orb` of any Ptolemaic aspect. Returns the
/// closest matching aspect if found.
fn closest_aspect(angle: f64, orb: f64) -> Option<TajakaAspect> {
    let mut best: Option<(TajakaAspect, f64)> = None;
    for asp in &TajakaAspect::ALL {
        let diff = (angle - asp.angle()).abs();
        let diff = if diff > 180.0 { 360.0 - diff } else { diff };
        if diff <= orb
            && best.as_ref().is_none_or(|(_, prev)| diff < *prev) {
                best = Some((*asp, diff));
            }
    }
    best.map(|(a, _)| a)
}

// ---------------------------------------------------------------------------
// Ithasala check — the most important Tajaka yoga
// ---------------------------------------------------------------------------

/// Check for Ithasala (applying aspect) between two planets.
///
/// Ithasala occurs when:
/// 1. The faster planet is behind the slower planet (or approaching an
///    exact aspect) within the given orb.
/// 2. The faster planet has greater daily motion than the slower planet
///    (it is genuinely gaining).
/// 3. They are within orb of a major Ptolemaic aspect (conjunction,
///    sextile, square, trine, or opposition).
///
/// Returns `Some(IthasalaResult)` if Ithasala is found, `None` otherwise.
pub fn check_ithasala(
    faster_planet: f64,
    slower_planet: f64,
    faster_speed: f64,
    slower_speed: f64,
    orb: f64,
) -> Option<IthasalaResult> {
    // Must be genuinely applying
    if faster_speed <= slower_speed {
        return None;
    }

    let dist = angular_dist(faster_planet, slower_planet);

    // Check if the angular distance is near any Ptolemaic aspect
    let aspect = closest_aspect(dist, orb)?;

    // Distance to exact aspect
    let exact_angle = aspect.angle();
    let distance_to_exact = (dist - exact_angle).abs();

    // Determine if genuinely applying (faster planet behind, approaching exact)
    let forward_dist = (slower_planet - faster_planet).rem_euclid(360.0);
    let is_applying = forward_dist <= 180.0 && distance_to_exact <= orb;

    if !is_applying {
        return None;
    }

    let relative_speed = faster_speed - slower_speed;
    let days_to_exact = if relative_speed > 0.001 {
        distance_to_exact / relative_speed
    } else {
        f64::INFINITY
    };

    Some(IthasalaResult {
        aspect,
        distance_to_exact,
        is_applying,
        days_to_exact,
    })
}

/// Simple Ithasala detection (backward-compatible boolean version).
pub fn detect_ithasala(
    faster_lon: f64,
    slower_lon: f64,
    faster_speed: f64,
    slower_speed: f64,
    orb: f64,
) -> bool {
    let dist = angular_dist(faster_lon, slower_lon);
    let closing = faster_speed > slower_speed;
    dist <= orb && closing
}

/// Detect Ishrafa (separating aspect, opposite of Ithasala).
pub fn detect_ishrafa(
    faster_lon: f64,
    slower_lon: f64,
    faster_speed: f64,
    slower_speed: f64,
    orb: f64,
) -> bool {
    let dist = angular_dist(faster_lon, slower_lon);
    dist <= orb && !detect_ithasala(faster_lon, slower_lon, faster_speed, slower_speed, orb)
}

// ---------------------------------------------------------------------------
// Tajaka yoga detection helpers
// ---------------------------------------------------------------------------

/// Exaltation sign for each planet (0-based sign index).
fn exaltation_sign(planet: &str) -> Option<usize> {
    match planet {
        "Sun" => Some(0),     // Aries
        "Moon" => Some(1),    // Taurus
        "Mars" => Some(9),    // Capricorn
        "Mercury" => Some(5), // Virgo
        "Jupiter" => Some(3), // Cancer
        "Venus" => Some(11),  // Pisces
        "Saturn" => Some(6),  // Libra
        _ => None,
    }
}

/// Own signs for each planet.
fn own_signs(planet: &str) -> &'static [usize] {
    match planet {
        "Sun" => &[4],         // Leo
        "Moon" => &[3],        // Cancer
        "Mars" => &[0, 7],     // Aries, Scorpio
        "Mercury" => &[2, 5],  // Gemini, Virgo
        "Jupiter" => &[8, 11], // Sagittarius, Pisces
        "Venus" => &[1, 6],    // Taurus, Libra
        "Saturn" => &[9, 10],  // Capricorn, Aquarius
        _ => &[],
    }
}

/// Check Ikkabal — planet in own or exalted sign in the annual chart.
pub fn check_ikkabal(planet: &str, sign: usize) -> bool {
    let s = sign % 12;
    if let Some(exalt) = exaltation_sign(planet)
        && s == exalt {
            return true;
        }
    own_signs(planet).contains(&s)
}

/// Check Induvara — all planets in kendras (houses 1, 4, 7, 10 from Asc).
///
/// `planet_signs`: list of 0-based sign indices for all 7 planets.
/// `asc_sign`: 0-based sign index of the annual ascendant.
pub fn check_induvara(planet_signs: &[usize], asc_sign: usize) -> bool {
    let kendras = [0usize, 3, 6, 9]; // houses 1, 4, 7, 10 (0-based offset)
    for &p_sign in planet_signs {
        let house = ((p_sign as i32 - asc_sign as i32).rem_euclid(12)) as usize;
        if !kendras.contains(&house) {
            return false;
        }
    }
    !planet_signs.is_empty()
}

/// Check Yamaya — mutual reception between two planets in the annual chart.
///
/// `planet_a` is in sign of `planet_b`'s rulership AND `planet_b` is in
/// sign of `planet_a`'s rulership.
pub fn check_yamaya(planet_a: &str, sign_a: usize, planet_b: &str, sign_b: usize) -> bool {
    let a_in_b = own_signs(planet_b).contains(&(sign_a % 12));
    let b_in_a = own_signs(planet_a).contains(&(sign_b % 12));
    a_in_b && b_in_a
}

/// Check Kamboola — Ithasala where one planet is retrograde.
pub fn check_kamboola(
    faster_lon: f64,
    slower_lon: f64,
    faster_speed: f64,
    slower_speed: f64,
    orb: f64,
    faster_retrograde: bool,
    slower_retrograde: bool,
) -> bool {
    let has_ithasala = detect_ithasala(faster_lon, slower_lon, faster_speed, slower_speed, orb);
    has_ithasala && (faster_retrograde || slower_retrograde)
}

/// Check Duhphali — planet hemmed between two malefics.
///
/// `planet_sign`: the sign of the planet to check.
/// `malefic_signs`: signs of malefic planets (Mars, Saturn, Rahu, Ketu, Sun).
pub fn check_duhphali(planet_sign: usize, malefic_signs: &[usize]) -> bool {
    let s = planet_sign % 12;
    let prev = (s + 11) % 12; // sign before
    let next = (s + 1) % 12; // sign after
    let has_prev = malefic_signs.iter().any(|&m| m % 12 == prev);
    let has_next = malefic_signs.iter().any(|&m| m % 12 == next);
    has_prev && has_next
}

/// Detect all Tajaka yogas present in an annual chart.
///
/// This is a convenience function that checks multiple yogas at once.
/// `planets`: list of (name, sign, longitude, daily_speed, is_retrograde).
/// `asc_sign`: annual chart ascendant sign (0-based).
/// `orb`: standard orb for aspect checks (typically 8-12 degrees).
pub fn detect_tajaka_yogas(
    planets: &[(&str, usize, f64, f64, bool)],
    asc_sign: usize,
    orb: f64,
) -> Vec<(TajakaYoga, String)> {
    let mut yogas = Vec::new();

    // Ikkabal check for each planet
    for &(name, sign, _, _, _) in planets {
        if check_ikkabal(name, sign) {
            yogas.push((TajakaYoga::Ikkabal, format!("{name} in own/exalted sign")));
        }
    }

    // Induvara check
    let signs: Vec<usize> = planets.iter().map(|p| p.1).collect();
    if check_induvara(&signs, asc_sign) {
        yogas.push((TajakaYoga::Induvara, "All planets in kendras".into()));
    }

    // Malefic signs for Duhphali
    let malefics = ["Mars", "Saturn", "Rahu", "Ketu", "Sun"];
    let malefic_signs: Vec<usize> = planets
        .iter()
        .filter(|(name, _, _, _, _)| malefics.contains(name))
        .map(|p| p.1)
        .collect();

    // Pairwise checks
    for i in 0..planets.len() {
        let (name_i, sign_i, lon_i, speed_i, retro_i) = planets[i];

        // Duhphali
        if !malefics.contains(&name_i)
            && check_duhphali(sign_i, &malefic_signs) {
                yogas.push((TajakaYoga::Duhphali, format!("{name_i} hemmed by malefics")));
            }

        for j in (i + 1)..planets.len() {
            let (name_j, sign_j, lon_j, speed_j, retro_j) = planets[j];

            // Determine which is faster
            let (
                faster_name,
                faster_lon,
                faster_speed,
                faster_retro,
                slower_name,
                slower_lon,
                slower_speed,
                slower_retro,
            ) = if speed_i.abs() >= speed_j.abs() {
                (
                    name_i, lon_i, speed_i, retro_i, name_j, lon_j, speed_j, retro_j,
                )
            } else {
                (
                    name_j, lon_j, speed_j, retro_j, name_i, lon_i, speed_i, retro_i,
                )
            };

            // Ithasala
            if detect_ithasala(faster_lon, slower_lon, faster_speed, slower_speed, orb) {
                yogas.push((
                    TajakaYoga::Ithasala,
                    format!("{faster_name} applying to {slower_name}"),
                ));

                // Kamboola (Ithasala + retrograde)
                if faster_retro || slower_retro {
                    yogas.push((
                        TajakaYoga::Kamboola,
                        format!("{faster_name}-{slower_name} Ithasala with retrograde"),
                    ));
                }
            }

            // Ishrafa
            if detect_ishrafa(faster_lon, slower_lon, faster_speed, slower_speed, orb) {
                yogas.push((
                    TajakaYoga::Ishrafa,
                    format!("{faster_name} separating from {slower_name}"),
                ));
            }

            // Yamaya
            if check_yamaya(name_i, sign_i, name_j, sign_j) {
                yogas.push((
                    TajakaYoga::Yamaya,
                    format!("{name_i}-{name_j} mutual reception"),
                ));
            }
        }
    }

    yogas
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn muntha_at_age_0() {
        assert_eq!(compute_muntha(0, 0), 0);
    }

    #[test]
    fn muntha_cycles_12() {
        assert_eq!(compute_muntha(3, 12), 3);
        assert_eq!(compute_muntha(3, 1), 4);
    }

    #[test]
    fn muntha_lord_correct() {
        assert_eq!(muntha_lord(0), "Mars"); // Aries
        assert_eq!(muntha_lord(4), "Sun"); // Leo
        assert_eq!(muntha_lord(11), "Jupiter"); // Pisces
        assert_eq!(muntha_lord(9), "Saturn"); // Capricorn
        assert_eq!(muntha_lord(3), "Moon"); // Cancer
    }

    #[test]
    fn muntha_aspects_conjunction() {
        let planets = vec![("Jupiter", 5usize), ("Saturn", 11)];
        let aspects = muntha_aspects(5, &planets);
        assert!(
            aspects
                .iter()
                .any(|&(name, asp)| name == "Jupiter" && asp == TajakaAspect::Conjunction)
        );
        assert!(
            aspects
                .iter()
                .any(|&(name, asp)| name == "Saturn" && asp == TajakaAspect::Opposition)
        );
    }

    #[test]
    fn muntha_aspects_trine_and_square() {
        let planets = vec![("Mars", 1usize), ("Venus", 8)];
        // Muntha in sign 5 (Virgo). Mars in sign 1: diff = (1-5+12)%12 = 8 = trine
        // Venus in sign 8: diff = (8-5)%12 = 3 = square
        let aspects = muntha_aspects(5, &planets);
        assert!(
            aspects
                .iter()
                .any(|&(name, asp)| name == "Mars" && asp == TajakaAspect::Trine)
        );
        assert!(
            aspects
                .iter()
                .any(|&(name, asp)| name == "Venus" && asp == TajakaAspect::Square)
        );
    }

    fn test_positions() -> HashMap<&'static str, f64> {
        let mut m = HashMap::new();
        m.insert("Mars", 45.0);
        m.insert("Mercury", 120.0);
        m.insert("Jupiter", 200.0);
        m.insert("Venus", 310.0);
        m.insert("Saturn", 250.0);
        m
    }

    #[test]
    fn sahams_count() {
        let s = compute_sahams(100.0, 280.0, 50.0, true, &test_positions());
        assert!(
            s.len() >= 9,
            "Should compute at least 9 sahams (added Mahatmya)"
        );
    }

    #[test]
    fn sahams_in_range() {
        let s = compute_sahams(100.0, 280.0, 50.0, true, &test_positions());
        for saham in &s {
            assert!(
                saham.longitude_deg >= 0.0 && saham.longitude_deg < 360.0,
                "{} out of range: {}",
                saham.name,
                saham.longitude_deg
            );
        }
    }

    #[test]
    fn punya_day_formula() {
        // Punya Day = Moon - Sun + ASC (Tajik Neelakanthi)
        let positions = test_positions();
        let asc = 100.0;
        let sun = 280.0;
        let moon = 50.0;
        let s = compute_sahams(asc, sun, moon, true, &positions);
        let punya = s.iter().find(|s| s.name == "Punya (Fortune)").unwrap();
        let expected = (moon - sun + asc).rem_euclid(360.0);
        assert!(
            (punya.longitude_deg - expected).abs() < 0.001,
            "Punya day: expected {expected}, got {}",
            punya.longitude_deg
        );
    }

    #[test]
    fn punya_night_formula() {
        // Punya Night = Sun - Moon + ASC (Tajik Neelakanthi)
        let positions = test_positions();
        let asc = 100.0;
        let sun = 280.0;
        let moon = 50.0;
        let s = compute_sahams(asc, sun, moon, false, &positions);
        let punya = s.iter().find(|s| s.name == "Punya (Fortune)").unwrap();
        let expected = (sun - moon + asc).rem_euclid(360.0);
        assert!(
            (punya.longitude_deg - expected).abs() < 0.001,
            "Punya night: expected {expected}, got {}",
            punya.longitude_deg
        );
    }

    #[test]
    fn vidya_day_is_inverse_of_punya() {
        // Vidya Day = Sun - Moon + ASC (inverse of Punya per Neelakanthi)
        let positions = test_positions();
        let asc = 100.0;
        let sun = 280.0;
        let moon = 50.0;
        let s = compute_sahams(asc, sun, moon, true, &positions);
        let vidya = s.iter().find(|s| s.name == "Vidya (Education)").unwrap();
        let expected = (sun - moon + asc).rem_euclid(360.0);
        assert!(
            (vidya.longitude_deg - expected).abs() < 0.001,
            "Vidya day: expected {expected}, got {}",
            vidya.longitude_deg
        );
    }

    #[test]
    fn yashas_uses_punya_and_jupiter() {
        // Yashas Day = Jupiter - Punya + ASC (compound saham per Neelakanthi)
        let positions = test_positions();
        let asc: f64 = 100.0;
        let sun: f64 = 280.0;
        let moon: f64 = 50.0;
        let jupiter: f64 = 200.0;
        let punya = (moon - sun + asc).rem_euclid(360.0);
        let s = compute_sahams(asc, sun, moon, true, &positions);
        let yashas = s.iter().find(|s| s.name == "Yashas (Fame)").unwrap();
        let expected = (jupiter - punya + asc).rem_euclid(360.0);
        assert!(
            (yashas.longitude_deg - expected).abs() < 0.001,
            "Yashas day: expected {expected}, got {}",
            yashas.longitude_deg
        );
    }

    #[test]
    fn mitra_uses_punya_jupiter_venus() {
        // Mitra Day = Jupiter - Punya + Venus (compound saham per Neelakanthi)
        let positions = test_positions();
        let asc: f64 = 100.0;
        let sun: f64 = 280.0;
        let moon: f64 = 50.0;
        let jupiter: f64 = 200.0;
        let venus: f64 = 310.0;
        let punya = (moon - sun + asc).rem_euclid(360.0);
        let s = compute_sahams(asc, sun, moon, true, &positions);
        let mitra = s.iter().find(|s| s.name == "Mitra (Friendship)").unwrap();
        let expected = (jupiter - punya + venus).rem_euclid(360.0);
        assert!(
            (mitra.longitude_deg - expected).abs() < 0.001,
            "Mitra day: expected {expected}, got {}",
            mitra.longitude_deg
        );
    }

    #[test]
    fn gaurav_uses_jupiter_moon_sun() {
        // Gaurav Day = Jupiter - Moon + Sun (per Neelakanthi)
        let positions = test_positions();
        let asc = 100.0;
        let sun = 280.0;
        let moon = 50.0;
        let jupiter = 200.0;
        let s = compute_sahams(asc, sun, moon, true, &positions);
        let gaurav = s.iter().find(|s| s.name == "Gaurav (Respect)").unwrap();
        let expected = (jupiter - moon + sun).rem_euclid(360.0);
        assert!(
            (gaurav.longitude_deg - expected).abs() < 0.001,
            "Gaurav day: expected {expected}, got {}",
            gaurav.longitude_deg
        );
    }

    #[test]
    fn mahatmya_present() {
        // Mahatmya (Greatness) should be in the output
        let positions = test_positions();
        let s = compute_sahams(100.0, 280.0, 50.0, true, &positions);
        assert!(
            s.iter().any(|s| s.name == "Mahatmya (Greatness)"),
            "Mahatmya saham should be present"
        );
    }

    #[test]
    fn sahams_use_real_positions_not_zero() {
        // Gaurav depends on Jupiter, so it should differ with real vs empty positions
        let positions = test_positions();
        let real = compute_sahams(100.0, 280.0, 50.0, true, &positions);
        let empty: HashMap<&str, f64> = HashMap::new();
        let zero = compute_sahams(100.0, 280.0, 50.0, true, &empty);
        let real_gaurav = real.iter().find(|s| s.name == "Gaurav (Respect)").unwrap();
        let zero_gaurav = zero.iter().find(|s| s.name == "Gaurav (Respect)").unwrap();
        assert!(
            (real_gaurav.longitude_deg - zero_gaurav.longitude_deg).abs() > 1.0,
            "Gaurav should change when Jupiter is non-zero"
        );
    }

    // ----- Ithasala tests -----

    #[test]
    fn ithasala_applying() {
        assert!(detect_ithasala(100.0, 105.0, 1.5, 0.5, 8.0));
    }

    #[test]
    fn ithasala_not_when_separating() {
        assert!(!detect_ithasala(100.0, 105.0, 0.3, 0.5, 8.0));
    }

    #[test]
    fn check_ithasala_conjunction() {
        // Moon at 100 deg (speed 13.0/day), Saturn at 103 deg (speed 0.05/day)
        // Distance = 3 deg, within 8 deg orb of conjunction (0 deg)
        let result = check_ithasala(100.0, 103.0, 13.0, 0.05, 8.0);
        assert!(result.is_some(), "Should detect Ithasala");
        let r = result.unwrap();
        assert_eq!(r.aspect, TajakaAspect::Conjunction);
        assert!(r.is_applying);
        assert!(r.days_to_exact < 1.0, "Should be less than 1 day to exact");
    }

    #[test]
    fn check_ithasala_trine() {
        // Planet A at 10 deg (speed 1.2/day), Planet B at 127 deg (speed 0.1/day)
        // Distance = 117 deg, within 8 deg of trine (120 deg)
        let result = check_ithasala(10.0, 127.0, 1.2, 0.1, 8.0);
        assert!(result.is_some(), "Should detect Ithasala near trine");
        let r = result.unwrap();
        assert_eq!(r.aspect, TajakaAspect::Trine);
    }

    #[test]
    fn check_ithasala_none_when_slower() {
        // Slower planet cannot form Ithasala
        let result = check_ithasala(100.0, 103.0, 0.03, 0.05, 8.0);
        assert!(
            result.is_none(),
            "Should not detect Ithasala when faster_speed <= slower_speed"
        );
    }

    #[test]
    fn check_ithasala_none_when_out_of_orb() {
        // Distance of 50 degrees, no Ptolemaic aspect near
        let result = check_ithasala(100.0, 150.0, 1.5, 0.5, 5.0);
        assert!(
            result.is_none(),
            "Should not detect Ithasala far from any aspect"
        );
    }

    // ----- Ikkabal tests -----

    #[test]
    fn ikkabal_sun_in_leo() {
        assert!(check_ikkabal("Sun", 4)); // Leo = own sign
        assert!(check_ikkabal("Sun", 0)); // Aries = exaltation
        assert!(!check_ikkabal("Sun", 6)); // Libra = debilitation
    }

    #[test]
    fn ikkabal_jupiter_in_cancer() {
        assert!(check_ikkabal("Jupiter", 3)); // Cancer = exalted
        assert!(check_ikkabal("Jupiter", 8)); // Sagittarius = own
        assert!(check_ikkabal("Jupiter", 11)); // Pisces = own
        assert!(!check_ikkabal("Jupiter", 0)); // Aries = neither
    }

    // ----- Induvara test -----

    #[test]
    fn induvara_all_in_kendras() {
        // Asc in Aries (0). Kendras = signs 0, 3, 6, 9.
        assert!(check_induvara(&[0, 3, 6, 9, 0, 3, 6], 0));
        // One planet in sign 1 (not kendra) → false
        assert!(!check_induvara(&[0, 1, 6, 9, 0, 3, 6], 0));
    }

    // ----- Yamaya test -----

    #[test]
    fn yamaya_mutual_reception() {
        // Moon in Leo (4=Sun's sign), Sun in Cancer (3=Moon's sign)
        assert!(check_yamaya("Moon", 4, "Sun", 3));
        // Not mutual: Moon in Leo, Sun in Aries
        assert!(!check_yamaya("Moon", 4, "Sun", 0));
    }

    // ----- Kamboola test -----

    #[test]
    fn kamboola_ithasala_with_retrograde() {
        assert!(check_kamboola(100.0, 105.0, 1.5, 0.5, 8.0, true, false));
        assert!(check_kamboola(100.0, 105.0, 1.5, 0.5, 8.0, false, true));
        // No Ithasala → no Kamboola even if retrograde
        assert!(!check_kamboola(100.0, 105.0, 0.3, 0.5, 8.0, true, false));
    }

    // ----- Duhphali test -----

    #[test]
    fn duhphali_hemmed_by_malefics() {
        // Planet in sign 5 (Virgo), malefics in signs 4 and 6
        assert!(check_duhphali(5, &[4, 6]));
        // Not hemmed: malefic only on one side
        assert!(!check_duhphali(5, &[4]));
        assert!(!check_duhphali(5, &[6]));
        // Wrap-around: planet in sign 0, malefics in 11 and 1
        assert!(check_duhphali(0, &[11, 1]));
    }

    // ----- detect_tajaka_yogas integration test -----

    #[test]
    fn detect_tajaka_yogas_finds_ikkabal() {
        let planets = vec![
            ("Sun", 4, 130.0, 1.0, false),  // Sun in Leo (own sign) → Ikkabal
            ("Moon", 1, 45.0, 13.0, false), // Moon in Taurus (exalted) → Ikkabal
            ("Mars", 2, 75.0, 0.5, false),
        ];
        let yogas = detect_tajaka_yogas(&planets, 0, 8.0);
        let ikkabal_count = yogas
            .iter()
            .filter(|(y, _)| *y == TajakaYoga::Ikkabal)
            .count();
        assert!(
            ikkabal_count >= 2,
            "Should find at least 2 Ikkabal yogas, got {ikkabal_count}"
        );
    }

    #[test]
    fn detect_tajaka_yogas_finds_ithasala() {
        let planets = vec![
            ("Moon", 3, 100.0, 13.0, false),   // fast Moon
            ("Saturn", 9, 105.0, 0.05, false), // slow Saturn nearby
        ];
        let yogas = detect_tajaka_yogas(&planets, 0, 8.0);
        assert!(
            yogas.iter().any(|(y, _)| *y == TajakaYoga::Ithasala),
            "Should detect Ithasala between Moon and Saturn"
        );
    }

    #[test]
    fn angular_dist_symmetric() {
        assert!((angular_dist(10.0, 20.0) - 10.0).abs() < 0.001);
        assert!((angular_dist(20.0, 10.0) - 10.0).abs() < 0.001);
        assert!((angular_dist(350.0, 10.0) - 20.0).abs() < 0.001);
        assert!((angular_dist(10.0, 350.0) - 20.0).abs() < 0.001);
    }
}
