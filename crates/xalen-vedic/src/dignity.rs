use crate::rashi::Rashi;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Classical planetary dignity level in Vedic astrology.
pub enum Dignity {
    Exalted,
    Moolatrikona,
    OwnSign,
    Friend,
    Neutral,
    Enemy,
    Debilitated,
}

/// Return a planet's sign of exaltation.
pub fn exaltation_sign(planet: &str) -> Option<Rashi> {
    match planet {
        "Sun" => Some(Rashi::Mesha),
        "Moon" => Some(Rashi::Vrishabha),
        "Mars" => Some(Rashi::Makara),
        "Mercury" => Some(Rashi::Kanya),
        "Jupiter" => Some(Rashi::Karka),
        "Venus" => Some(Rashi::Meena),
        "Saturn" => Some(Rashi::Tula),
        "Rahu" => Some(Rashi::Vrishabha),
        "Ketu" => Some(Rashi::Vrishchika),
        _ => None,
    }
}

/// Return a planet's sign of debilitation.
pub fn debilitation_sign(planet: &str) -> Option<Rashi> {
    match planet {
        "Sun" => Some(Rashi::Tula),
        "Moon" => Some(Rashi::Vrishchika),
        "Mars" => Some(Rashi::Karka),
        "Mercury" => Some(Rashi::Meena),
        "Jupiter" => Some(Rashi::Makara),
        "Venus" => Some(Rashi::Kanya),
        "Saturn" => Some(Rashi::Mesha),
        "Rahu" => Some(Rashi::Vrishchika),
        "Ketu" => Some(Rashi::Vrishabha),
        _ => None,
    }
}

/// Return a planet's exact degree of exaltation.
pub fn exaltation_degree(planet: &str) -> Option<f64> {
    match planet {
        "Sun" => Some(10.0),
        "Moon" => Some(33.0),     // 3° Taurus
        "Mars" => Some(298.0),    // 28° Capricorn
        "Mercury" => Some(165.0), // 15° Virgo
        "Jupiter" => Some(95.0),  // 5° Cancer
        "Venus" => Some(357.0),   // 27° Pisces
        "Saturn" => Some(200.0),  // 20° Libra
        _ => None,
    }
}

/// Return the sign(s) owned/ruled by a planet.
pub fn own_signs(planet: &str) -> Vec<Rashi> {
    match planet {
        "Sun" => vec![Rashi::Simha],
        "Moon" => vec![Rashi::Karka],
        "Mars" => vec![Rashi::Mesha, Rashi::Vrishchika],
        "Mercury" => vec![Rashi::Mithuna, Rashi::Kanya],
        "Jupiter" => vec![Rashi::Dhanu, Rashi::Meena],
        "Venus" => vec![Rashi::Vrishabha, Rashi::Tula],
        "Saturn" => vec![Rashi::Makara, Rashi::Kumbha],
        _ => vec![],
    }
}

/// Return the moolatrikona sign and degree range for a planet.
pub fn moolatrikona_range(planet: &str) -> Option<(Rashi, f64, f64)> {
    match planet {
        "Sun" => Some((Rashi::Simha, 0.0, 20.0)),
        "Moon" => Some((Rashi::Vrishabha, 4.0, 30.0)),
        "Mars" => Some((Rashi::Mesha, 0.0, 12.0)),
        "Mercury" => Some((Rashi::Kanya, 16.0, 20.0)),
        "Jupiter" => Some((Rashi::Dhanu, 0.0, 10.0)),
        "Venus" => Some((Rashi::Tula, 0.0, 15.0)),
        "Saturn" => Some((Rashi::Kumbha, 0.0, 20.0)),
        _ => None,
    }
}

fn is_in_moolatrikona(planet: &str, rashi: Rashi, degree_in_sign: f64) -> bool {
    if let Some((mt_rashi, start, end)) = moolatrikona_range(planet) {
        rashi == mt_rashi && degree_in_sign >= start && degree_in_sign < end
    } else {
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Planetary relationship classification (natural or compound).
pub enum Relationship {
    AdhiMitra,
    Mitra,
    Sama,
    Shatru,
    AdhiShatru,
}

/// Return a planet's natural friends per BPHS.
pub fn natural_friends(planet: &str) -> &'static [&'static str] {
    match planet {
        "Sun" => &["Moon", "Mars", "Jupiter"],
        "Moon" => &["Sun", "Mercury"],
        "Mars" => &["Sun", "Moon", "Jupiter"],
        "Mercury" => &["Sun", "Venus"],
        "Jupiter" => &["Sun", "Moon", "Mars"],
        "Venus" => &["Mercury", "Saturn"],
        "Saturn" => &["Mercury", "Venus"],
        _ => &[],
    }
}

/// Return a planet's natural enemies per BPHS.
pub fn natural_enemies(planet: &str) -> &'static [&'static str] {
    match planet {
        "Sun" => &["Venus", "Saturn"],
        "Moon" => &[],
        "Mars" => &["Mercury"],
        "Mercury" => &["Moon"],
        "Jupiter" => &["Mercury", "Venus"],
        "Venus" => &["Sun", "Moon"],
        "Saturn" => &["Sun", "Moon", "Mars"],
        _ => &[],
    }
}

/// Compute the natural relationship between two planets.
pub fn natural_relationship(planet: &str, other: &str) -> Relationship {
    if natural_friends(planet).contains(&other) {
        return Relationship::Mitra;
    }
    if natural_enemies(planet).contains(&other) {
        return Relationship::Shatru;
    }
    Relationship::Sama
}

/// Compute the temporal relationship based on sign positions.
pub fn temporal_relationship(planet_sign_idx: usize, other_sign_idx: usize) -> Relationship {
    let diff = ((other_sign_idx as i32 - planet_sign_idx as i32).rem_euclid(12)) as usize;
    match diff {
        1 | 2 | 3 | 4 | 9 | 10 | 11 => Relationship::Mitra,
        _ => Relationship::Shatru,
    }
}

/// Compute compound relationship (natural + temporal combined).
pub fn compound_relationship(
    planet: &str,
    other: &str,
    planet_sign_idx: usize,
    other_sign_idx: usize,
) -> Relationship {
    let nat = natural_relationship(planet, other);
    let temp = temporal_relationship(planet_sign_idx, other_sign_idx);
    match (nat, temp) {
        (Relationship::Mitra, Relationship::Mitra) => Relationship::AdhiMitra, // Friend+Friend = Great Friend
        (Relationship::Shatru, Relationship::Shatru) => Relationship::AdhiShatru, // Enemy+Enemy = Great Enemy
        (Relationship::Mitra, Relationship::Shatru)
        | (Relationship::Shatru, Relationship::Mitra) => Relationship::Sama, // Friend+Enemy = Neutral
        (Relationship::Sama, Relationship::Mitra) | (Relationship::Mitra, Relationship::Sama) => {
            Relationship::Mitra
        } // Neutral+Friend = Friend
        (Relationship::Sama, Relationship::Shatru) | (Relationship::Shatru, Relationship::Sama) => {
            Relationship::Shatru
        } // Neutral+Enemy = Enemy
        (Relationship::Sama, Relationship::Sama) => Relationship::Sama, // Neutral+Neutral = Neutral
        _ => Relationship::Sama,
    }
}

/// Determine planetary dignity from sign placement alone.
pub fn planet_dignity(planet: &str, rashi: Rashi) -> Dignity {
    if exaltation_sign(planet) == Some(rashi) {
        return Dignity::Exalted;
    }
    if debilitation_sign(planet) == Some(rashi) {
        return Dignity::Debilitated;
    }
    if own_signs(planet).contains(&rashi) {
        return Dignity::OwnSign;
    }
    Dignity::Neutral
}

/// Determine planetary dignity with moolatrikona degree awareness.
pub fn planet_dignity_with_degree(planet: &str, rashi: Rashi, degree_in_sign: f64) -> Dignity {
    if debilitation_sign(planet) == Some(rashi) {
        return Dignity::Debilitated;
    }
    if is_in_moolatrikona(planet, rashi, degree_in_sign) {
        return Dignity::Moolatrikona;
    }
    if exaltation_sign(planet) == Some(rashi) {
        return Dignity::Exalted;
    }
    if own_signs(planet).contains(&rashi) {
        return Dignity::OwnSign;
    }
    Dignity::Neutral
}

/// Check if a debilitated planet has cancellation of debilitation.
pub fn neecha_bhanga(planet: &str, chart_lords: &[(&str, Rashi)]) -> bool {
    let deb_sign = match debilitation_sign(planet) {
        Some(s) => s,
        None => return false,
    };
    let ex_sign = match exaltation_sign(planet) {
        Some(s) => s,
        None => return false,
    };
    let deb_lord = sign_lord(deb_sign);
    let ex_lord = sign_lord(ex_sign);
    for &(p, r) in chart_lords {
        if p == deb_lord && (exaltation_sign(p) == Some(r) || r == Rashi::from_index(0)) {
            return true;
        }
        if p == ex_lord && (exaltation_sign(p) == Some(r)) {
            return true;
        }
    }
    false
}

fn sign_lord(rashi: Rashi) -> &'static str {
    match rashi {
        Rashi::Mesha | Rashi::Vrishchika => "Mars",
        Rashi::Vrishabha | Rashi::Tula => "Venus",
        Rashi::Mithuna | Rashi::Kanya => "Mercury",
        Rashi::Karka => "Moon",
        Rashi::Simha => "Sun",
        Rashi::Dhanu | Rashi::Meena => "Jupiter",
        Rashi::Makara | Rashi::Kumbha => "Saturn",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sun_exalted_in_aries() {
        assert_eq!(planet_dignity("Sun", Rashi::Mesha), Dignity::Exalted);
    }

    #[test]
    fn sun_debilitated_in_libra() {
        assert_eq!(planet_dignity("Sun", Rashi::Tula), Dignity::Debilitated);
    }

    #[test]
    fn mars_own_signs() {
        assert_eq!(planet_dignity("Mars", Rashi::Mesha), Dignity::OwnSign);
        assert_eq!(planet_dignity("Mars", Rashi::Vrishchika), Dignity::OwnSign);
    }

    #[test]
    fn saturn_neutral_in_gemini() {
        assert_eq!(planet_dignity("Saturn", Rashi::Mithuna), Dignity::Neutral);
    }

    #[test]
    fn sun_moolatrikona_in_leo_0_20() {
        assert_eq!(
            planet_dignity_with_degree("Sun", Rashi::Simha, 10.0),
            Dignity::Moolatrikona
        );
        assert_eq!(
            planet_dignity_with_degree("Sun", Rashi::Simha, 25.0),
            Dignity::OwnSign
        );
    }

    #[test]
    fn moon_moolatrikona_in_taurus() {
        assert_eq!(
            planet_dignity_with_degree("Moon", Rashi::Vrishabha, 15.0),
            Dignity::Moolatrikona
        );
        assert_eq!(
            planet_dignity_with_degree("Moon", Rashi::Vrishabha, 2.0),
            Dignity::Exalted
        );
    }

    #[test]
    fn natural_friendship_sun_moon() {
        assert_eq!(natural_relationship("Sun", "Moon"), Relationship::Mitra);
    }

    #[test]
    fn natural_enmity_sun_saturn() {
        assert_eq!(natural_relationship("Sun", "Saturn"), Relationship::Shatru);
    }

    #[test]
    fn natural_neutral_moon_mars() {
        assert_eq!(natural_relationship("Moon", "Mars"), Relationship::Sama);
    }

    #[test]
    fn temporal_friends_adjacent() {
        assert_eq!(temporal_relationship(0, 1), Relationship::Mitra);
        assert_eq!(temporal_relationship(0, 4), Relationship::Mitra);
    }

    #[test]
    fn temporal_enemies_5_to_8() {
        assert_eq!(temporal_relationship(0, 5), Relationship::Shatru);
        assert_eq!(temporal_relationship(0, 6), Relationship::Shatru);
        assert_eq!(temporal_relationship(0, 7), Relationship::Shatru);
        assert_eq!(temporal_relationship(0, 8), Relationship::Shatru);
    }

    #[test]
    fn compound_adhi_mitra() {
        assert_eq!(
            compound_relationship("Sun", "Moon", 0, 1),
            Relationship::AdhiMitra
        );
    }

    #[test]
    fn compound_adhi_shatru() {
        assert_eq!(
            compound_relationship("Sun", "Saturn", 0, 6),
            Relationship::AdhiShatru
        );
    }

    #[test]
    fn moolatrikona_ranges_exist() {
        for planet in [
            "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn",
        ] {
            assert!(
                moolatrikona_range(planet).is_some(),
                "Missing MT for {planet}"
            );
        }
        assert!(moolatrikona_range("Rahu").is_none());
    }

    #[test]
    fn sign_lord_table() {
        assert_eq!(sign_lord(Rashi::Mesha), "Mars");
        assert_eq!(sign_lord(Rashi::Simha), "Sun");
        assert_eq!(sign_lord(Rashi::Kumbha), "Saturn");
    }
}
