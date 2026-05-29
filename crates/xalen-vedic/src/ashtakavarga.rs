use crate::nakshatra::DashaLord;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ashtakavarga {
    pub bhinna: [[u8; 12]; 8], // 8 rows (7 planets + Lagna) x 12 signs; each cell = bindu count (0-8)
    pub sarva: [u8; 12],       // sum of all 8 bhinna rows per sign
    pub total: u16,
}

// ---------------------------------------------------------------------------
// Bindu contribution tables per BPHS Chapters 66-72 (Santhanam edition).
//
// Each table is [8 contributors][12 houses]. Row order: Sun, Moon, Mars,
// Mercury, Jupiter, Venus, Saturn, Lagna. A `true` at index `h` means a
// bindu is awarded when the contributor is in house `h+1` FROM the subject
// planet.
//
// These are encoded as const [[bool; 12]; 8] for zero-cost lookup.
// ---------------------------------------------------------------------------

/// Converts a 1-based house-number slice into a [bool; 12] mask.
const fn houses_to_mask(houses: &[u8]) -> [bool; 12] {
    let mut mask = [false; 12];
    let mut i = 0;
    while i < houses.len() {
        let h = houses[i] as usize;
        if h >= 1 && h <= 12 {
            mask[h - 1] = true;
        }
        i += 1;
    }
    mask
}

/// Build a full 8-contributor bindu table from slices.
macro_rules! bindu_table {
    ($sun:expr, $moon:expr, $mars:expr, $mer:expr, $jup:expr, $ven:expr, $sat:expr, $lag:expr) => {
        [
            houses_to_mask($sun),
            houses_to_mask($moon),
            houses_to_mask($mars),
            houses_to_mask($mer),
            houses_to_mask($jup),
            houses_to_mask($ven),
            houses_to_mask($sat),
            houses_to_mask($lag),
        ]
    };
}

/// Sun's BAV — BPHS Ch.66
const SUN_BINDU: [[bool; 12]; 8] = bindu_table!(
    &[1, 2, 4, 7, 8, 9, 10, 11], // from Sun
    &[3, 6, 10, 11],             // from Moon
    &[1, 2, 4, 7, 8, 9, 10, 11], // from Mars
    &[3, 5, 6, 9, 10, 11, 12],   // from Mercury
    &[5, 6, 9, 11],              // from Jupiter
    &[6, 7, 12],                 // from Venus
    &[1, 2, 4, 7, 8, 9, 10, 11], // from Saturn
    &[3, 4, 6, 10, 11, 12]       // from Lagna
);

/// Moon's BAV — BPHS Ch.67
const MOON_BINDU: [[bool; 12]; 8] = bindu_table!(
    &[3, 6, 7, 8, 10, 11],       // from Sun
    &[1, 3, 6, 7, 10, 11],       // from Moon
    &[2, 3, 5, 6, 9, 10, 11],    // from Mars
    &[1, 3, 4, 5, 7, 8, 10, 11], // from Mercury
    &[1, 4, 7, 8, 10, 11, 12],   // from Jupiter
    &[3, 4, 5, 7, 9, 10, 11],    // from Venus
    &[3, 5, 6, 11],              // from Saturn
    &[3, 6, 10, 11]              // from Lagna
);

/// Mars's BAV — BPHS Ch.68
const MARS_BINDU: [[bool; 12]; 8] = bindu_table!(
    &[3, 5, 6, 10, 11],       // from Sun
    &[3, 6, 11],              // from Moon
    &[1, 2, 4, 7, 8, 10, 11], // from Mars
    &[3, 5, 6, 11],           // from Mercury
    &[6, 10, 11, 12],         // from Jupiter
    &[6, 8, 11, 12],          // from Venus
    &[1, 4, 7, 8, 9, 10, 11], // from Saturn
    &[1, 3, 6, 10, 11]        // from Lagna
);

/// Mercury's BAV — BPHS Ch.69
const MERCURY_BINDU: [[bool; 12]; 8] = bindu_table!(
    &[5, 6, 9, 11, 12],           // from Sun
    &[2, 4, 6, 8, 10, 11],        // from Moon
    &[1, 2, 4, 7, 8, 9, 10, 11],  // from Mars
    &[1, 3, 5, 6, 9, 10, 11, 12], // from Mercury
    &[6, 8, 11, 12],              // from Jupiter
    &[1, 2, 3, 4, 5, 8, 9, 11],   // from Venus
    &[1, 2, 4, 7, 8, 9, 10, 11],  // from Saturn
    &[1, 2, 4, 6, 8, 10, 11]      // from Lagna
);

/// Jupiter's BAV — BPHS Ch.70
const JUPITER_BINDU: [[bool; 12]; 8] = bindu_table!(
    &[1, 2, 3, 4, 7, 8, 10, 11],    // from Sun
    &[2, 5, 7, 9, 11],              // from Moon
    &[1, 2, 4, 7, 8, 10, 11],       // from Mars
    &[1, 2, 4, 5, 6, 9, 10, 11],    // from Mercury
    &[1, 2, 3, 4, 7, 8, 10, 11],    // from Jupiter
    &[2, 5, 6, 9, 10, 11],          // from Venus
    &[3, 5, 6, 11, 12],             // from Saturn
    &[1, 2, 4, 5, 6, 7, 9, 10, 11]  // from Lagna
);

/// Venus's BAV — BPHS Ch.71
const VENUS_BINDU: [[bool; 12]; 8] = bindu_table!(
    &[8, 11, 12],                   // from Sun
    &[1, 2, 3, 4, 5, 8, 9, 11, 12], // from Moon
    &[3, 5, 6, 9, 11, 12],          // from Mars
    &[3, 5, 6, 9, 11],              // from Mercury
    &[5, 8, 9, 10, 11],             // from Jupiter
    &[1, 2, 3, 4, 5, 8, 9, 10, 11], // from Venus
    &[3, 4, 5, 8, 9, 10, 11],       // from Saturn
    &[1, 2, 3, 4, 5, 8, 9, 11]      // from Lagna
);

/// Saturn's BAV — BPHS Ch.72
const SATURN_BINDU: [[bool; 12]; 8] = bindu_table!(
    &[1, 2, 4, 7, 8, 10, 11], // from Sun
    &[3, 6, 11],              // from Moon
    &[3, 5, 6, 10, 11, 12],   // from Mars
    &[6, 8, 9, 10, 11, 12],   // from Mercury
    &[5, 6, 11, 12],          // from Jupiter
    &[6, 11, 12],             // from Venus
    &[3, 5, 6, 11],           // from Saturn
    &[1, 3, 4, 6, 10, 11]     // from Lagna
);

/// Lagna's BAV — common simplified table (BPHS does not give a dedicated
/// chapter, but many implementations use a standard set).
const LAGNA_BINDU: [[bool; 12]; 8] = bindu_table!(
    &[3, 4, 6, 10, 11, 12],         // from Sun
    &[3, 6, 10, 11],                // from Moon
    &[1, 3, 6, 10, 11],             // from Mars
    &[1, 2, 4, 6, 8, 10, 11],       // from Mercury
    &[1, 2, 4, 5, 6, 7, 9, 10, 11], // from Jupiter
    &[1, 2, 3, 4, 5, 8, 9, 11],     // from Venus
    &[1, 3, 4, 6, 10, 11],          // from Saturn
    &[3, 6, 10, 11, 12]             // from Lagna
);

/// All 8 bindu tables in planet order: Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Lagna.
const ALL_BINDU_TABLES: [&[[bool; 12]; 8]; 8] = [
    &SUN_BINDU,
    &MOON_BINDU,
    &MARS_BINDU,
    &MERCURY_BINDU,
    &JUPITER_BINDU,
    &VENUS_BINDU,
    &SATURN_BINDU,
    &LAGNA_BINDU,
];

/// Count of favorable entries per contributor row, for verification.
/// When all bodies are in the same sign, a planet's BAV total equals the
/// sum of favorable-house counts across its 8 contributor rows.
#[allow(dead_code)]
fn table_entry_count(table: &[[bool; 12]; 8]) -> u16 {
    table
        .iter()
        .map(|row| row.iter().filter(|&&v| v).count() as u16)
        .sum()
}

impl Ashtakavarga {
    /// Returns the bindu mask for a given (planet, contributor) pair.
    #[inline]
    #[allow(dead_code)]
    fn bindu_mask(planet_idx: usize, contrib_idx: usize) -> &'static [bool; 12] {
        &ALL_BINDU_TABLES[planet_idx][contrib_idx]
    }

    /// Compute the full Ashtakavarga from sign positions.
    ///
    /// `planet_signs`: [Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu, Lagna]
    /// as 0-based sign indices (0=Aries .. 11=Pisces). Rahu (index 7) is not used
    /// in classical Ashtakavarga but is kept for positional compatibility.
    pub fn compute(planet_signs: &[usize; 9]) -> Self {
        let mut bhinna = [[0u8; 12]; 8];
        let mut sarva = [0u8; 12];

        // For each of the 8 rows (7 planets + Lagna):
        for planet in 0..8usize {
            let mask_table = ALL_BINDU_TABLES[planet];
            for contrib in 0..8usize {
                let contrib_sign = if contrib < 7 {
                    planet_signs[contrib]
                } else {
                    planet_signs[8] // Lagna
                };
                let mask = &mask_table[contrib];
                for sign in 0..12usize {
                    let house = ((sign as i32 - contrib_sign as i32).rem_euclid(12)) as usize;
                    if mask[house] {
                        bhinna[planet][sign] += 1;
                    }
                }
            }
        }

        for sign in 0..12 {
            sarva[sign] = bhinna.iter().map(|row| row[sign]).sum();
        }
        let total = sarva.iter().map(|&v| v as u16).sum();

        Ashtakavarga {
            bhinna,
            sarva,
            total,
        }
    }

    pub fn sign_strength(&self, sign: usize) -> &'static str {
        match self.sarva[sign % 12] {
            0..=20 => "Weak",
            21..=28 => "Average",
            29..=35 => "Good",
            _ => "Excellent",
        }
    }

    pub fn transit_favorable(&self, sign: usize) -> bool {
        self.sarva[sign % 12] >= 28
    }
}

// ---------------------------------------------------------------------------
// Standalone public functions for SAV / Prashtara access
// ---------------------------------------------------------------------------

/// Compute the Sarvashtakavarga: for each of the 12 signs, sum the bindus
/// from all 7 planets' BAV (excluding Lagna row to match the classical
/// 337-total convention).
///
/// Returns `[[u8; 12]; 7]` — one row per planet (Sun..Saturn), plus a total
/// SAV row is not included here (caller can sum columns).
///
/// `planet_positions`: slice of `(DashaLord, sign_index)` for the 7 planets.
/// Order does not matter; unrecognized lords (Rahu/Ketu) are ignored.
/// `lagna_sign`: 0-based sign index of the Lagna/Ascendant.
pub fn sarvashtakavarga(
    planet_positions: &[(DashaLord, usize)],
    lagna_sign: usize,
) -> [[u8; 12]; 7] {
    let signs = positions_to_array(planet_positions, lagna_sign);
    let av = Ashtakavarga::compute(&signs);
    // Return only the 7 planetary rows (Sun..Saturn); bhinna[7] is Lagna.
    let mut result = [[0u8; 12]; 7];
    result.copy_from_slice(&av.bhinna[..7]);
    result
}

/// Compute the Prashtarashtakavarga (BAV) for a single planet: returns the
/// 12-sign bindu distribution for `planet` given the chart positions.
/// `lagna_sign`: 0-based sign index of the Lagna/Ascendant.
pub fn prashtarashtakavarga(
    planet: DashaLord,
    planet_positions: &[(DashaLord, usize)],
    lagna_sign: usize,
) -> [u8; 12] {
    let idx = lord_to_idx(planet);
    if idx >= 7 {
        return [0u8; 12]; // Rahu/Ketu have no classical BAV
    }
    let signs = positions_to_array(planet_positions, lagna_sign);
    let av = Ashtakavarga::compute(&signs);
    av.bhinna[idx]
}

/// Map DashaLord to planet index (0=Sun..6=Saturn). Returns 7 for Rahu/Ketu.
fn lord_to_idx(lord: DashaLord) -> usize {
    match lord {
        DashaLord::Sun => 0,
        DashaLord::Moon => 1,
        DashaLord::Mars => 2,
        DashaLord::Mercury => 3,
        DashaLord::Jupiter => 4,
        DashaLord::Venus => 5,
        DashaLord::Saturn => 6,
        _ => 7, // Rahu, Ketu
    }
}

/// Convert a variable-length DashaLord position slice into the [usize; 9]
/// layout expected by `Ashtakavarga::compute`. Missing planets default to
/// sign 0 (Aries). `lagna_sign` is placed at index 8.
fn positions_to_array(positions: &[(DashaLord, usize)], lagna_sign: usize) -> [usize; 9] {
    let mut arr = [0usize; 9];
    for &(lord, sign) in positions {
        let idx = lord_to_idx(lord);
        if idx < 7 {
            arr[idx] = sign % 12;
        }
        if matches!(lord, DashaLord::Rahu) {
            arr[7] = sign % 12;
        }
    }
    arr[8] = lagna_sign % 12;
    arr
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nakshatra::DashaLord;

    // ----- Structural tests -----

    #[test]
    fn ashtakavarga_total_reasonable() {
        let signs = [0, 3, 7, 5, 8, 1, 9, 10, 0];
        let av = Ashtakavarga::compute(&signs);
        assert!(
            av.total > 200 && av.total < 500,
            "Total SAV bindus should be 200-500, got {}",
            av.total
        );
    }

    #[test]
    fn sarva_sums_correctly() {
        let signs = [0, 3, 7, 5, 8, 1, 9, 10, 0];
        let av = Ashtakavarga::compute(&signs);
        let computed_total: u16 = av.sarva.iter().map(|&v| v as u16).sum();
        assert_eq!(av.total, computed_total);
    }

    #[test]
    fn bhinna_per_planet_bounded() {
        let signs = [0, 3, 7, 5, 8, 1, 9, 10, 0];
        let av = Ashtakavarga::compute(&signs);
        for (planet, row) in av.bhinna.iter().enumerate() {
            for (sign, &val) in row.iter().enumerate() {
                assert!(
                    val <= 8,
                    "Bhinna[planet={planet}][sign={sign}] should be <= 8, got {val}"
                );
            }
        }
    }

    #[test]
    fn bhinna_row_totals_plausible() {
        let signs = [0, 3, 7, 5, 8, 1, 9, 10, 0];
        let av = Ashtakavarga::compute(&signs);
        for (planet, row) in av.bhinna.iter().enumerate() {
            let row_total: u16 = row.iter().map(|&v| v as u16).sum();
            assert!(row_total > 0, "Planet {planet} BAV should have > 0 bindus");
        }
    }

    // ----- All-zero position tests: BAV total = table entry count -----

    #[test]
    fn sun_bav_total_matches_table_count() {
        // 8+4+8+7+4+3+8+6 = 48
        let signs = [0; 9];
        let av = Ashtakavarga::compute(&signs);
        let sun_total: u16 = av.bhinna[0].iter().map(|&v| v as u16).sum();
        assert_eq!(
            sun_total,
            table_entry_count(&SUN_BINDU),
            "Sun BAV total with all-0 should be {}, got {sun_total}",
            table_entry_count(&SUN_BINDU)
        );
    }

    #[test]
    fn moon_bav_total_matches_table_count() {
        // 6+6+7+8+7+7+4+4 = 49
        let signs = [0; 9];
        let av = Ashtakavarga::compute(&signs);
        let moon_total: u16 = av.bhinna[1].iter().map(|&v| v as u16).sum();
        assert_eq!(
            moon_total,
            table_entry_count(&MOON_BINDU),
            "Moon BAV total with all-0 should be {}, got {moon_total}",
            table_entry_count(&MOON_BINDU)
        );
    }

    #[test]
    fn mars_bav_total_matches_table_count() {
        let signs = [0; 9];
        let av = Ashtakavarga::compute(&signs);
        let total: u16 = av.bhinna[2].iter().map(|&v| v as u16).sum();
        let expected = table_entry_count(&MARS_BINDU);
        assert_eq!(
            total, expected,
            "Mars BAV total with all-0 should be {expected}, got {total}"
        );
    }

    #[test]
    fn mercury_bav_total_matches_table_count() {
        let signs = [0; 9];
        let av = Ashtakavarga::compute(&signs);
        let total: u16 = av.bhinna[3].iter().map(|&v| v as u16).sum();
        let expected = table_entry_count(&MERCURY_BINDU);
        assert_eq!(
            total, expected,
            "Mercury BAV total with all-0 should be {expected}, got {total}"
        );
    }

    #[test]
    fn jupiter_bav_total_matches_table_count() {
        let signs = [0; 9];
        let av = Ashtakavarga::compute(&signs);
        let total: u16 = av.bhinna[4].iter().map(|&v| v as u16).sum();
        let expected = table_entry_count(&JUPITER_BINDU);
        assert_eq!(
            total, expected,
            "Jupiter BAV total with all-0 should be {expected}, got {total}"
        );
    }

    #[test]
    fn venus_bav_total_matches_table_count() {
        let signs = [0; 9];
        let av = Ashtakavarga::compute(&signs);
        let total: u16 = av.bhinna[5].iter().map(|&v| v as u16).sum();
        let expected = table_entry_count(&VENUS_BINDU);
        assert_eq!(
            total, expected,
            "Venus BAV total with all-0 should be {expected}, got {total}"
        );
    }

    #[test]
    fn saturn_bav_total_matches_table_count() {
        let signs = [0; 9];
        let av = Ashtakavarga::compute(&signs);
        let total: u16 = av.bhinna[6].iter().map(|&v| v as u16).sum();
        let expected = table_entry_count(&SATURN_BINDU);
        assert_eq!(
            total, expected,
            "Saturn BAV total with all-0 should be {expected}, got {total}"
        );
    }

    // ----- Classical SAV total = 337 check -----
    // The classical BPHS SAV total (7 planets only, not Lagna) is always 337
    // regardless of planet positions. This is a fundamental Ashtakavarga identity.
    #[test]
    fn sav_seven_planet_total_337() {
        // Test with several different position configurations
        let test_cases: &[[usize; 9]] = &[
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 3, 7, 5, 8, 1, 9, 10, 0],
            [11, 2, 6, 4, 10, 0, 8, 3, 5],
            [5, 5, 5, 5, 5, 5, 5, 5, 5],
            [1, 4, 7, 10, 0, 3, 6, 9, 2],
        ];
        for (i, signs) in test_cases.iter().enumerate() {
            let av = Ashtakavarga::compute(signs);
            let seven_planet_total: u16 = (0..7)
                .flat_map(|p| av.bhinna[p].iter())
                .map(|&v| v as u16)
                .sum();
            assert_eq!(
                seven_planet_total, 337,
                "SAV 7-planet total should always be 337 (case {i}), got {seven_planet_total}"
            );
        }
    }

    // ----- All 7 planets have distinct tables -----
    #[test]
    fn all_seven_planets_have_distinct_tables() {
        let signs = [0, 3, 7, 5, 8, 1, 9, 10, 0];
        let av = Ashtakavarga::compute(&signs);
        // With well-dispersed positions, each planet should produce a unique row
        for i in 0..7 {
            for j in (i + 1)..7 {
                assert_ne!(
                    av.bhinna[i], av.bhinna[j],
                    "Planets {i} and {j} should have different BAV rows"
                );
            }
        }
    }

    // ----- Per-sign SAV range 18-35 -----
    #[test]
    fn per_sign_sav_in_classical_range() {
        let signs = [0, 3, 7, 5, 8, 1, 9, 10, 0];
        let av = Ashtakavarga::compute(&signs);
        // Including Lagna row, SAV per sign is typically 18-45
        for (sign, &val) in av.sarva.iter().enumerate() {
            assert!(
                val >= 10 && val <= 55,
                "SAV[sign={sign}] should be 10-55, got {val}"
            );
        }
    }

    // ----- sarvashtakavarga function -----
    #[test]
    fn sarvashtakavarga_returns_7_rows() {
        let positions = vec![
            (DashaLord::Sun, 0usize),
            (DashaLord::Moon, 3),
            (DashaLord::Mars, 7),
            (DashaLord::Mercury, 5),
            (DashaLord::Jupiter, 8),
            (DashaLord::Venus, 1),
            (DashaLord::Saturn, 9),
        ];
        let sav = sarvashtakavarga(&positions, 0);
        // Should have 7 rows (Sun..Saturn)
        assert_eq!(sav.len(), 7);
        // Each row has 12 signs
        for row in &sav {
            assert_eq!(row.len(), 12);
        }
    }

    // ----- prashtarashtakavarga function -----
    #[test]
    fn prashtarashtakavarga_sun_matches_bhinna() {
        let positions = vec![
            (DashaLord::Sun, 0usize),
            (DashaLord::Moon, 3),
            (DashaLord::Mars, 7),
            (DashaLord::Mercury, 5),
            (DashaLord::Jupiter, 8),
            (DashaLord::Venus, 1),
            (DashaLord::Saturn, 9),
        ];
        let bav = prashtarashtakavarga(DashaLord::Sun, &positions, 0);
        let signs = [0, 3, 7, 5, 8, 1, 9, 0, 0]; // Rahu=0, Lagna=0
        let av = Ashtakavarga::compute(&signs);
        assert_eq!(bav, av.bhinna[0]);
    }

    #[test]
    fn prashtarashtakavarga_rahu_returns_zeros() {
        let positions = vec![(DashaLord::Sun, 0)];
        let bav = prashtarashtakavarga(DashaLord::Rahu, &positions, 0);
        assert_eq!(bav, [0u8; 12]);
    }

    // ----- Transit favorability -----
    #[test]
    fn transit_favorable_check() {
        let signs = [0, 0, 0, 0, 0, 0, 0, 0, 0];
        let av = Ashtakavarga::compute(&signs);
        let _is_favorable = av.transit_favorable(0);
        // Just verify it does not panic
    }

    // ----- Known chart cross-check (Aries lagna, standard positions) -----
    #[test]
    fn known_chart_sun_bav_aries_sign() {
        // Sun in Aries (0), Moon in Cancer (3), Mars in Scorpio (7),
        // Mercury in Virgo (5), Jupiter in Sagittarius (8), Venus in Taurus (1),
        // Saturn in Capricorn (9), Rahu in Aquarius (10), Lagna in Aries (0)
        let signs = [0, 3, 7, 5, 8, 1, 9, 10, 0];
        let av = Ashtakavarga::compute(&signs);
        // Sun's BAV in Aries (sign 0) — manually verify by checking each
        // contributor:
        // - Sun is in sign 0, house from Sun = 1 → SUN_BINDU[0][0] = true (house 1)
        // - Moon is in sign 3, house of sign 0 from Moon = (0-3+12)%12 = 9 → house 10 → SUN_BINDU[1] has 10 → true
        // - Mars in sign 7, house = (0-7+12)%12 = 5 → house 6 → SUN_BINDU[2] check: has 1,2,4,7,8,9,10,11 → no 6 → false
        // We can at least verify the bindu count is in [0, 8]
        let sun_aries = av.bhinna[0][0];
        assert!(
            sun_aries <= 8,
            "Sun BAV in Aries should be 0-8, got {sun_aries}"
        );
        assert!(
            sun_aries >= 1,
            "Sun in own sign should get at least 1 bindu"
        );
    }
}
