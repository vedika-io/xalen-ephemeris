//! Egyptian decan system — 36 decans of 10 degrees each.
//!
//! Each decan spans 10 degrees of the ecliptic (zodiac). The decanate rulers
//! follow the **Chaldean order** (Saturn, Jupiter, Mars, Sun, Venus, Mercury,
//! Moon) cycling continuously: the first decan of Aries is ruled by Mars
//! (Aries' sign ruler), then Sun, Venus, and so on through all 36 decans.
//!
//! Traditional star-group (constellation) names are simplified transliterations
//! of the Egyptian originals attested in temple ceilings (Dendera, Esna).

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Decan data
// ---------------------------------------------------------------------------

/// A single decan (10-degree arc of the ecliptic).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Decan {
    /// Decan number, 1-36.
    pub number: u8,
    /// Simplified traditional Egyptian name.
    pub name: &'static str,
    /// Start degree (inclusive), 0-350 in steps of 10.
    pub start_degree: f64,
    /// End degree (exclusive), 10-360 in steps of 10.
    pub end_degree: f64,
    /// Chaldean decanate ruler.
    pub ruling_planet: &'static str,
    /// Traditional star group associated with this decan.
    pub star_group: &'static str,
}

/// Chaldean order of planetary rulers, cycling continuously.
const CHALDEAN_RULERS: [&str; 7] = [
    "Mars", "Sun", "Venus", "Mercury", "Moon", "Saturn", "Jupiter",
];

/// 36 traditional decan names (simplified transliterations).
const DECAN_NAMES: [&str; 36] = [
    // Aries (0-30)
    "Khnumis", // 0-10
    "Situ",    // 10-20
    "Kherau",  // 20-30
    // Taurus (30-60)
    "Arostoaris", // 30-40
    "Usir-hau",   // 40-50
    "Sesmut",     // 50-60
    // Gemini (60-90)
    "Sothis",      // 60-70
    "Sit",         // 70-80
    "Khentet-hrt", // 80-90
    // Cancer (90-120)
    "Khentet-kht",      // 90-100
    "Temu-hrt",         // 100-110
    "Ueshati-besheset", // 110-120
    // Leo (120-150)
    "Kher-khept-sahu", // 120-130
    "Ha-djat",         // 130-140
    "Pehui-djat",      // 140-150
    // Virgo (150-180)
    "Temu-kht", // 150-160
    "Ustha",    // 160-170
    "Seshmu",   // 170-180
    // Libra (180-210)
    "Khentet-hru", // 180-190
    "Qetui",       // 190-200
    "Hatedat",     // 200-210
    // Scorpio (210-240)
    "Kher-khept-khentet", // 210-220
    "Seshmu-hor",         // 220-230
    "Kenmu",              // 230-240
    // Sagittarius (240-270)
    "Semed", // 240-250
    "Seret", // 250-260
    "Sasa",  // 260-270
    // Capricorn (270-300)
    "Kenmu-kht",     // 270-280
    "Sapu-khentet",  // 280-290
    "Kherp-khentet", // 290-300
    // Aquarius (300-330)
    "Hau",              // 300-310
    "Art",              // 310-320
    "Remen-hru-an-sah", // 320-330
    // Pisces (330-360)
    "Thespy-hrt", // 330-340
    "Uaret",      // 340-350
    "Phui-hor",   // 350-360
];

/// Star-group names for each decan.
const STAR_GROUPS: [&str; 36] = [
    "Triangulum",        // 1
    "Musca Borealis",    // 2
    "Perseus (head)",    // 3
    "Pleiades",          // 4
    "Hyades",            // 5
    "Auriga",            // 6
    "Sirius group",      // 7
    "Orion (belt)",      // 8
    "Canis Minor",       // 9
    "Argo (prow)",       // 10
    "Hydra (head)",      // 11
    "Argo (stern)",      // 12
    "Crater",            // 13
    "Corvus",            // 14
    "Hydra (tail)",      // 15
    "Bootes (north)",    // 16
    "Centaurus (head)",  // 17
    "Centaurus (body)",  // 18
    "Corona Borealis",   // 19
    "Serpens (head)",    // 20
    "Lupus",             // 21
    "Scorpius (head)",   // 22
    "Scorpius (body)",   // 23
    "Scorpius (tail)",   // 24
    "Sagittarius (bow)", // 25
    "Lyra",              // 26
    "Aquila",            // 27
    "Piscis Austrinus",  // 28
    "Cygnus",            // 29
    "Delphinus",         // 30
    "Pegasus (head)",    // 31
    "Pegasus (body)",    // 32
    "Andromeda",         // 33
    "Cetus (head)",      // 34
    "Cetus (body)",      // 35
    "Aries (rear)",      // 36
];

// ---------------------------------------------------------------------------
// Static table — built once, used for all lookups
// ---------------------------------------------------------------------------

/// All 36 decans in order (initialised once on first access).
static ALL_DECANS: std::sync::LazyLock<[Decan; 36]> = std::sync::LazyLock::new(|| {
    // Build via array::from_fn — no unsafe required.
    std::array::from_fn(|i| {
        let start = (i as f64) * 10.0;
        Decan {
            number: (i + 1) as u8,
            name: DECAN_NAMES[i],
            start_degree: start,
            end_degree: start + 10.0,
            ruling_planet: CHALDEAN_RULERS[i % 7],
            star_group: STAR_GROUPS[i],
        }
    })
});

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Return all 36 decans.
pub fn all_decans() -> &'static [Decan; 36] {
    &ALL_DECANS
}

/// Return the decan for the given ecliptic degree (0..360).
///
/// Degrees are normalised to `[0, 360)` before lookup.
pub fn decan_for_degree(degree: f64) -> &'static Decan {
    let norm = degree.rem_euclid(360.0);
    let idx = (norm / 10.0).floor() as usize;
    &ALL_DECANS[idx.min(35)]
}

/// Return the Chaldean decanate ruler for the given ecliptic degree.
pub fn decan_ruler(degree: f64) -> &'static str {
    decan_for_degree(degree).ruling_planet
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_decans_count() {
        assert_eq!(all_decans().len(), 36);
    }

    #[test]
    fn decan_numbers_sequential() {
        for (i, d) in all_decans().iter().enumerate() {
            assert_eq!(d.number, (i + 1) as u8);
        }
    }

    #[test]
    fn decan_degrees_contiguous() {
        let decans = all_decans();
        for i in 1..36 {
            assert!(
                (decans[i].start_degree - decans[i - 1].end_degree).abs() < f64::EPSILON,
                "gap between decan {} and {}",
                i,
                i + 1
            );
        }
        assert!((decans[0].start_degree).abs() < f64::EPSILON);
        assert!((decans[35].end_degree - 360.0).abs() < f64::EPSILON);
    }

    #[test]
    fn first_decan_aries_is_mars() {
        let d = decan_for_degree(5.0);
        assert_eq!(d.number, 1);
        assert_eq!(d.name, "Khnumis");
        assert_eq!(d.ruling_planet, "Mars");
    }

    #[test]
    fn second_decan_aries_is_sun() {
        let d = decan_for_degree(15.0);
        assert_eq!(d.number, 2);
        assert_eq!(d.ruling_planet, "Sun");
    }

    #[test]
    fn third_decan_aries_is_venus() {
        let d = decan_for_degree(25.0);
        assert_eq!(d.number, 3);
        assert_eq!(d.ruling_planet, "Venus");
    }

    #[test]
    fn taurus_first_decan_is_mercury() {
        // Decan 4 (30-40) should be Mercury (continuing Chaldean cycle)
        let d = decan_for_degree(35.0);
        assert_eq!(d.number, 4);
        assert_eq!(d.ruling_planet, "Mercury");
    }

    #[test]
    fn degree_normalisation_wraps() {
        let a = decan_for_degree(5.0);
        let b = decan_for_degree(365.0);
        assert_eq!(a.number, b.number);
        assert_eq!(a.ruling_planet, b.ruling_planet);
    }

    #[test]
    fn degree_normalisation_negative() {
        // -5 deg => 355 deg => decan 36
        let d = decan_for_degree(-5.0);
        assert_eq!(d.number, 36);
    }

    #[test]
    fn decan_ruler_matches_decan() {
        for deg in (0..360).step_by(5) {
            let d = decan_for_degree(deg as f64);
            assert_eq!(decan_ruler(deg as f64), d.ruling_planet);
        }
    }

    #[test]
    fn chaldean_cycle_repeats_every_7() {
        // Decan 1 and decan 8 should have the same ruler
        let d1 = &all_decans()[0];
        let d8 = &all_decans()[7];
        assert_eq!(d1.ruling_planet, d8.ruling_planet);
    }

    #[test]
    fn boundary_degree_belongs_to_next_decan() {
        // Exactly at 10.0 should be decan 2
        let d = decan_for_degree(10.0);
        assert_eq!(d.number, 2);
    }

    #[test]
    fn last_decan_pisces() {
        let d = decan_for_degree(355.0);
        assert_eq!(d.number, 36);
        assert_eq!(d.name, "Phui-hor");
    }
}
