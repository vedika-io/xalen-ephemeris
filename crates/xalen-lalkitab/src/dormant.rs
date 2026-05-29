//! Dormant planet (Soya Hua Graha) detection.
//!
//! A planet becomes dormant when it occupies the house of its enemy or is
//! conjunct with enemy planets. Dormant planets cannot deliver their natural
//! significations until awakened through specific remedies.

use crate::houses::Planet;

/// Permanent natural enemies per Lal Kitab.
///
/// These differ from BPHS enemies. Lal Kitab treats enmity as inherent
/// and non-temporal.
pub fn enemies_of(planet: Planet) -> &'static [Planet] {
    match planet {
        Planet::Sun => &[Planet::Saturn, Planet::Venus, Planet::Rahu],
        Planet::Moon => &[Planet::Rahu, Planet::Ketu],
        Planet::Mars => &[Planet::Mercury, Planet::Ketu],
        Planet::Mercury => &[Planet::Moon, Planet::Mars],
        Planet::Jupiter => &[Planet::Mercury, Planet::Venus, Planet::Rahu],
        Planet::Venus => &[Planet::Sun, Planet::Rahu],
        Planet::Saturn => &[Planet::Sun, Planet::Moon, Planet::Mars],
        Planet::Rahu => &[Planet::Sun, Planet::Mars, Planet::Venus],
        Planet::Ketu => &[Planet::Moon, Planet::Mars],
        _ => &[],
    }
}

/// Houses naturally owned by each planet (where it is "at home").
///
/// These are the houses where the planet is a natural ruler per Lal Kitab.
fn owned_houses(planet: Planet) -> &'static [usize] {
    match planet {
        Planet::Sun => &[1],
        Planet::Moon => &[4],
        Planet::Mars => &[3, 8],
        Planet::Mercury => &[6],
        Planet::Jupiter => &[2, 5, 9, 12],
        Planet::Venus => &[7],
        Planet::Saturn => &[8, 10, 11],
        Planet::Rahu => &[6, 12],
        Planet::Ketu => &[6, 12],
        _ => &[],
    }
}

/// Check whether `planet` is in a house owned by one of its enemies.
fn in_enemy_house(planet: Planet, house: usize) -> bool {
    let foes = enemies_of(planet);
    for &enemy in foes {
        if owned_houses(enemy).contains(&house) {
            return true;
        }
    }
    false
}

/// Determine if `planet` is dormant (Soya Hua) given its house and conjunctions.
///
/// A planet is dormant when:
/// 1. It occupies a house owned by one of its enemies, **or**
/// 2. It is conjunct with one or more of its enemies.
///
/// `conjunct` should list the other planets sharing the same house.
pub fn is_dormant(planet: Planet, house: usize, conjunct: &[Planet]) -> bool {
    let house = house.clamp(1, 12);

    // Condition 1: sitting in an enemy's house
    if in_enemy_house(planet, house) {
        return true;
    }

    // Condition 2: conjunct with an enemy
    let foes = enemies_of(planet);
    for &companion in conjunct {
        if foes.contains(&companion) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sun_dormant_in_saturn_house() {
        // Saturn owns 10, 11; Sun's enemy is Saturn.
        assert!(is_dormant(Planet::Sun, 10, &[]));
        assert!(is_dormant(Planet::Sun, 11, &[]));
    }

    #[test]
    fn sun_not_dormant_in_own_house() {
        assert!(!is_dormant(Planet::Sun, 1, &[]));
    }

    #[test]
    fn sun_dormant_conjunct_saturn() {
        // Even in a neutral house, conjunction with enemy triggers dormancy.
        assert!(is_dormant(Planet::Sun, 3, &[Planet::Saturn]));
    }

    #[test]
    fn moon_dormant_conjunct_rahu() {
        assert!(is_dormant(Planet::Moon, 1, &[Planet::Rahu]));
    }

    #[test]
    fn moon_not_dormant_alone_in_own_house() {
        assert!(!is_dormant(Planet::Moon, 4, &[]));
    }

    #[test]
    fn jupiter_dormant_in_venus_house() {
        // Venus owns 7; Jupiter's enemy is Venus.
        assert!(is_dormant(Planet::Jupiter, 7, &[]));
    }

    #[test]
    fn mars_not_dormant_in_own_house_alone() {
        assert!(!is_dormant(Planet::Mars, 3, &[]));
        assert!(!is_dormant(Planet::Mars, 8, &[]));
    }

    #[test]
    fn mars_dormant_conjunct_mercury() {
        assert!(is_dormant(Planet::Mars, 1, &[Planet::Mercury]));
    }

    #[test]
    fn saturn_dormant_in_sun_house() {
        // Sun owns 1; Saturn's enemy is Sun.
        assert!(is_dormant(Planet::Saturn, 1, &[]));
    }

    #[test]
    fn enemy_table_symmetry_not_required() {
        // Lal Kitab enmity is NOT symmetric. Mercury is enemy of Mars,
        // but Mars is also enemy of Mercury. Check both directions hold
        // per the tables above (they do happen to be symmetric here, but
        // that's by data, not by rule).
        let mars_foes = enemies_of(Planet::Mars);
        let merc_foes = enemies_of(Planet::Mercury);
        assert!(mars_foes.contains(&Planet::Mercury));
        assert!(merc_foes.contains(&Planet::Mars));
    }
}
