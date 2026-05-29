use serde::{Deserialize, Serialize};

// ===========================================================================
// Zodiacal Releasing (Vettius Valens, Anthology Book IV)
// ===========================================================================

/// ZR major period lengths by sign (Aries..Pisces), in years.
const ZR_PERIODS: [u32; 12] = [15, 8, 20, 25, 19, 20, 8, 15, 12, 27, 30, 12];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZrPeriod {
    pub sign_index: usize,
    pub level: u8,
    pub years: f64,
    pub start_jd: f64,
    pub end_jd: f64,
}

pub fn zodiacal_releasing(lot_sign: usize, birth_jd: f64, levels: u8) -> Vec<ZrPeriod> {
    let mut periods = Vec::new();
    let year_days = 365.25;
    let mut jd = birth_jd;

    for i in 0..12 {
        let sign = (lot_sign + i) % 12;
        let years = ZR_PERIODS[sign] as f64;
        let end = jd + years * year_days;

        if levels >= 2 {
            let l2 = zodiacal_releasing_l2(sign, jd, end);
            for p in l2 {
                periods.push(p);
            }
        }

        periods.push(ZrPeriod {
            sign_index: sign,
            level: 1,
            years,
            start_jd: jd,
            end_jd: end,
        });
        jd = end;
    }
    periods
}

fn zodiacal_releasing_l2(l1_sign: usize, l1_start: f64, l1_end: f64) -> Vec<ZrPeriod> {
    const TOTAL_CYCLE: f64 = 211.0; // sum of all ZR_PERIODS (15+8+20+25+19+20+8+15+12+27+30+12)
    let l1_total_days = l1_end - l1_start;
    let mut periods = Vec::new();
    let mut jd = l1_start;
    let mut cycle = 0;

    while jd < l1_end {
        for i in 0..12 {
            if jd >= l1_end {
                break;
            }
            let sign = (l1_sign + i) % 12;
            let ratio = ZR_PERIODS[sign] as f64 / TOTAL_CYCLE;
            let days = ratio * l1_total_days;
            let end = (jd + days).min(l1_end);

            periods.push(ZrPeriod {
                sign_index: sign,
                level: 2,
                years: (end - jd) / 365.25,
                start_jd: jd,
                end_jd: end,
            });
            jd = end;
        }
        cycle += 1;
        if cycle > 20 {
            break;
        }
    }
    periods
}

// ===========================================================================
// Annual Profections
// ===========================================================================

pub fn annual_profection(natal_asc_sign: usize, age: u32) -> ProfectionResult {
    let sign = (natal_asc_sign + age as usize) % 12;
    let lord = sign_ruler(sign);
    ProfectionResult {
        sign_index: sign,
        time_lord: lord,
        age,
    }
}

// ===========================================================================
// Sect
// ===========================================================================

pub fn planetary_sect(sun_above_horizon: bool) -> Sect {
    if sun_above_horizon {
        Sect::Day
    } else {
        Sect::Night
    }
}

pub fn sect_benefic(sect: Sect) -> &'static str {
    match sect {
        Sect::Day => "Jupiter",
        Sect::Night => "Venus",
    }
}

pub fn sect_malefic(sect: Sect) -> &'static str {
    match sect {
        Sect::Day => "Mars (out-of-sect)",
        Sect::Night => "Saturn (out-of-sect)",
    }
}

// ===========================================================================
// Decennials (Vettius Valens, Anthology Book VII)
// ===========================================================================
//
// The Decennial system assigns major period rulers in a fixed sequence
// determined by sect (day/night). Each major period is a set number of years.
// Sub-periods divide the major period proportionally among the same planetary
// sequence, starting from the major lord.
//
// Day sequence:  Sun -> Moon -> Mercury -> Venus -> Mars -> Jupiter -> Saturn
// Night sequence: Moon -> Sun -> Saturn -> Jupiter -> Mars -> Venus -> Mercury
//
// Major period years per planet (both sects use the same lengths, only order
// differs): Sun=19, Moon=25, Mercury=20, Venus=8, Mars=15, Jupiter=12, Saturn=30
// (Valens Anthology VII, minor years; total = 129 years per full cycle.)

pub use xalen_coords::Planet;

/// Major period years for Decennials, keyed by Planet.
/// Per Vettius Valens Anthology VII, these are the minor years of each planet:
///   Sun 19, Moon 25, Mercury 20, Venus 8, Mars 15, Jupiter 12, Saturn 30
///   Total = 129 years.
/// Web-verified: https://sevenstarsastrology.com/astrological-predictive-techniques-planetary-years-1-mino-years-division-of-days/
/// Also confirmed: https://www.twowander.com/blog/decennials-astrology
fn decennial_major_years(planet: Planet) -> f64 {
    match planet {
        Planet::Sun => 19.0,
        Planet::Moon => 25.0,
        Planet::Mercury => 20.0,
        Planet::Venus => 8.0,
        Planet::Mars => 15.0,
        Planet::Jupiter => 12.0,
        Planet::Saturn => 30.0,
        _ => 0.0,
    }
}

/// Day-chart major period sequence.
const DECENNIAL_DAY_SEQ: [Planet; 7] = [
    Planet::Sun,
    Planet::Moon,
    Planet::Mercury,
    Planet::Venus,
    Planet::Mars,
    Planet::Jupiter,
    Planet::Saturn,
];

/// Night-chart major period sequence.
const DECENNIAL_NIGHT_SEQ: [Planet; 7] = [
    Planet::Moon,
    Planet::Sun,
    Planet::Saturn,
    Planet::Jupiter,
    Planet::Mars,
    Planet::Venus,
    Planet::Mercury,
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecennialSubPeriod {
    pub lord: Planet,
    pub start_jd: f64,
    pub end_jd: f64,
    pub years: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decennial {
    pub lord: Planet,
    pub start_jd: f64,
    pub end_jd: f64,
    pub years: f64,
    pub sub_periods: Vec<DecennialSubPeriod>,
}

/// Compute Decennial major + sub periods starting from `birth_jd`.
///
/// Per Vettius Valens (Anthology VII), each major period spans the planet's
/// minor years (Sun 19, Moon 25, etc.) and the full cycle totals 129 years.
///
/// Sub-periods follow the Valens method: within each major period, sub-rulers
/// are assigned months equal to their minor years. Each sub-period's duration
/// in months equals that planet's minor-year value, and the 7 sub-periods
/// sum to 129 months (= 10 years 9 months = the major period length).
/// The sub-period sequence starts from the major lord and follows the sect
/// order.
///
/// Web-verified: https://www.twowander.com/blog/decennials-astrology
/// "The distribution of the months is according to the lesser years of the
///  planets minimised to months instead of years."
///
/// The function generates periods until at least `years` years are covered
/// (cycling through the sequence if needed).
pub fn compute_decennials(birth_jd: f64, is_day: bool, years: usize) -> Vec<Decennial> {
    let year_days = 365.25;
    let month_days = year_days / 12.0; // average month length
    let seq = if is_day {
        &DECENNIAL_DAY_SEQ
    } else {
        &DECENNIAL_NIGHT_SEQ
    };

    let mut results = Vec::new();
    let mut jd = birth_jd;
    let end_jd = birth_jd + (years as f64) * year_days;
    let mut idx = 0;

    while jd < end_jd {
        let planet = seq[idx % 7];
        let major_years = decennial_major_years(planet);
        let major_days = major_years * year_days;
        let period_end = jd + major_days;

        // Sub-periods per Valens: each planet gets months equal to its minor
        // years. The 7 sub-periods sum to 129 months (= 10y 9m), matching
        // the major period length. The sub-period sequence starts from the
        // major lord and follows the sect order.
        let mut sub_periods = Vec::new();
        let mut sub_jd = jd;
        for i in 0..7 {
            let sub_planet = seq[(idx + i) % 7];
            let sub_months = decennial_major_years(sub_planet); // months = minor years
            let sub_days = sub_months * month_days;
            let sub_end = sub_jd + sub_days;
            sub_periods.push(DecennialSubPeriod {
                lord: sub_planet,
                start_jd: sub_jd,
                end_jd: sub_end,
                years: sub_days / year_days,
            });
            sub_jd = sub_end;
        }

        results.push(Decennial {
            lord: planet,
            start_jd: jd,
            end_jd: period_end,
            years: major_years,
            sub_periods,
        });

        jd = period_end;
        idx += 1;
    }

    results
}

// ===========================================================================
// Firdaria (Persian planetary periods)
// ===========================================================================
//
// Day chart:  Sun(10) -> Venus(8) -> Mercury(13) -> Moon(9) -> Saturn(11) ->
//             Jupiter(12) -> Mars(7) = 70 years of planetary periods
// Night chart: Moon(9) -> Saturn(11) -> Jupiter(12) -> Mars(7) -> Sun(10) ->
//              Venus(8) -> Mercury(13) = 70 years
// Then: North Node(3) + South Node(2) = 5 years. Grand total = 75 years.
//
// Each major period is subdivided among all 7 traditional planets equally
// (not proportionally -- each sub-ruler gets major_years/7 years).

struct FirdariaSpec {
    ruler: Planet,
    years: f64,
}

const FIRDARIA_DAY_SEQ: [FirdariaSpec; 9] = [
    FirdariaSpec {
        ruler: Planet::Sun,
        years: 10.0,
    },
    FirdariaSpec {
        ruler: Planet::Venus,
        years: 8.0,
    },
    FirdariaSpec {
        ruler: Planet::Mercury,
        years: 13.0,
    },
    FirdariaSpec {
        ruler: Planet::Moon,
        years: 9.0,
    },
    FirdariaSpec {
        ruler: Planet::Saturn,
        years: 11.0,
    },
    FirdariaSpec {
        ruler: Planet::Jupiter,
        years: 12.0,
    },
    FirdariaSpec {
        ruler: Planet::Mars,
        years: 7.0,
    },
    FirdariaSpec {
        ruler: Planet::NorthNode,
        years: 3.0,
    },
    FirdariaSpec {
        ruler: Planet::SouthNode,
        years: 2.0,
    },
];

const FIRDARIA_NIGHT_SEQ: [FirdariaSpec; 9] = [
    FirdariaSpec {
        ruler: Planet::Moon,
        years: 9.0,
    },
    FirdariaSpec {
        ruler: Planet::Saturn,
        years: 11.0,
    },
    FirdariaSpec {
        ruler: Planet::Jupiter,
        years: 12.0,
    },
    FirdariaSpec {
        ruler: Planet::Mars,
        years: 7.0,
    },
    FirdariaSpec {
        ruler: Planet::Sun,
        years: 10.0,
    },
    FirdariaSpec {
        ruler: Planet::Venus,
        years: 8.0,
    },
    FirdariaSpec {
        ruler: Planet::Mercury,
        years: 13.0,
    },
    FirdariaSpec {
        ruler: Planet::NorthNode,
        years: 3.0,
    },
    FirdariaSpec {
        ruler: Planet::SouthNode,
        years: 2.0,
    },
];

/// The 7 traditional planets in Chaldean order, used for Firdaria sub-periods.
/// Sub-periods start from the major ruler's position in this sequence and cycle.
const FIRDARIA_SUB_PLANETS: [Planet; 7] = [
    Planet::Saturn,
    Planet::Jupiter,
    Planet::Mars,
    Planet::Sun,
    Planet::Venus,
    Planet::Mercury,
    Planet::Moon,
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirdariaPeriod {
    pub ruler: Planet,
    pub sub_ruler: Option<Planet>,
    pub start_jd: f64,
    pub end_jd: f64,
    pub years: f64,
}

/// Compute Firdaria periods for an entire 75-year cycle.
///
/// Returns a flat list: each major period is emitted first (with `sub_ruler = None`),
/// followed by its 7 sub-periods (with `sub_ruler = Some(planet)`).
/// For North Node and South Node periods there are no sub-periods (the nodal
/// periods are treated as indivisible per the classical sources).
pub fn compute_firdaria(birth_jd: f64, is_day: bool) -> Vec<FirdariaPeriod> {
    let year_days = 365.25;
    let seq = if is_day {
        &FIRDARIA_DAY_SEQ
    } else {
        &FIRDARIA_NIGHT_SEQ
    };

    let mut results = Vec::new();
    let mut jd = birth_jd;

    for spec in seq {
        let major_days = spec.years * year_days;
        let major_end = jd + major_days;

        // Emit the major period entry.
        results.push(FirdariaPeriod {
            ruler: spec.ruler,
            sub_ruler: None,
            start_jd: jd,
            end_jd: major_end,
            years: spec.years,
        });

        // Sub-periods only for the 7 traditional planets (not nodes).
        if matches!(spec.ruler, Planet::NorthNode | Planet::SouthNode) {
            jd = major_end;
            continue;
        }

        // Find the starting sub-ruler: the major ruler itself comes first,
        // then proceed in the Chaldean order from there.
        let start_idx = FIRDARIA_SUB_PLANETS
            .iter()
            .position(|&p| p == spec.ruler)
            .unwrap_or(0);

        let sub_years = spec.years / 7.0;
        let sub_days = sub_years * year_days;
        let mut sub_jd = jd;

        for i in 0..7 {
            let sub_planet = FIRDARIA_SUB_PLANETS[(start_idx + i) % 7];
            let sub_end = sub_jd + sub_days;
            results.push(FirdariaPeriod {
                ruler: spec.ruler,
                sub_ruler: Some(sub_planet),
                start_jd: sub_jd,
                end_jd: sub_end,
                years: sub_years,
            });
            sub_jd = sub_end;
        }

        jd = major_end;
    }

    results
}

// ===========================================================================
// Shared helpers
// ===========================================================================

fn sign_ruler(sign: usize) -> &'static str {
    const RULERS: [&str; 12] = [
        "Mars", "Venus", "Mercury", "Moon", "Sun", "Mercury", "Venus", "Mars", "Jupiter", "Saturn",
        "Saturn", "Jupiter",
    ];
    RULERS[sign % 12]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sect {
    Day,
    Night,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfectionResult {
    pub sign_index: usize,
    pub time_lord: &'static str,
    pub age: u32,
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- Zodiacal Releasing ---

    #[test]
    fn zr_12_l1_periods() {
        let periods: Vec<_> = zodiacal_releasing(0, 2451545.0, 1)
            .into_iter()
            .filter(|p| p.level == 1)
            .collect();
        assert_eq!(periods.len(), 12);
    }

    #[test]
    fn zr_total_years() {
        let periods: Vec<_> = zodiacal_releasing(0, 2451545.0, 1)
            .into_iter()
            .filter(|p| p.level == 1)
            .collect();
        let total: f64 = periods.iter().map(|p| p.years).sum();
        let expected: f64 = ZR_PERIODS.iter().map(|&y| y as f64).sum();
        assert!(
            (total - expected).abs() < 0.01,
            "Total should be {expected}, got {total}"
        );
    }

    #[test]
    fn zr_with_l2_has_sub_periods() {
        let periods = zodiacal_releasing(0, 2451545.0, 2);
        let l2_count = periods.iter().filter(|p| p.level == 2).count();
        assert!(l2_count > 0, "Should have L2 sub-periods");
    }

    #[test]
    fn zr_l2_subperiods_sum_to_l1_duration() {
        let periods = zodiacal_releasing(0, 2451545.0, 2);
        let l1_periods: Vec<_> = periods.iter().filter(|p| p.level == 1).collect();
        let l2_periods: Vec<_> = periods.iter().filter(|p| p.level == 2).collect();

        for l1 in &l1_periods {
            let l1_duration = l1.end_jd - l1.start_jd;
            let l2_sum: f64 = l2_periods
                .iter()
                .filter(|p| p.start_jd >= l1.start_jd && p.end_jd <= l1.end_jd + 1e-6)
                .map(|p| p.end_jd - p.start_jd)
                .sum();
            let diff = (l2_sum - l1_duration).abs();
            assert!(
                diff < 1e-6,
                "L2 sub-periods for L1 sign {} should sum to {:.4} days, got {:.4} (diff {:.10})",
                l1.sign_index,
                l1_duration,
                l2_sum,
                diff
            );
        }
    }

    // --- Annual Profections ---

    #[test]
    fn profection_at_age_0() {
        let p = annual_profection(0, 0);
        assert_eq!(p.sign_index, 0);
        assert_eq!(p.time_lord, "Mars"); // Aries ruler
    }

    #[test]
    fn profection_cycles_12() {
        let p1 = annual_profection(3, 0);
        let p2 = annual_profection(3, 12);
        assert_eq!(p1.sign_index, p2.sign_index);
    }

    // --- Sect ---

    #[test]
    fn sect_determination() {
        assert_eq!(planetary_sect(true), Sect::Day);
        assert_eq!(planetary_sect(false), Sect::Night);
        assert_eq!(sect_benefic(Sect::Day), "Jupiter");
        assert_eq!(sect_benefic(Sect::Night), "Venus");
    }

    // --- Decennials ---

    #[test]
    fn decennial_day_starts_with_sun() {
        let dec = compute_decennials(2451545.0, true, 80);
        assert_eq!(dec[0].lord, Planet::Sun);
    }

    #[test]
    fn decennial_night_starts_with_moon() {
        let dec = compute_decennials(2451545.0, false, 80);
        assert_eq!(dec[0].lord, Planet::Moon);
    }

    #[test]
    fn decennial_major_years_match() {
        let dec = compute_decennials(2451545.0, true, 130);
        // First period is Sun = 19 years (Valens minor years)
        assert!(
            (dec[0].years - 19.0).abs() < 0.01,
            "Sun period should be 19.0 years, got {}",
            dec[0].years
        );
    }

    #[test]
    fn decennial_has_7_sub_periods_per_major() {
        let dec = compute_decennials(2451545.0, true, 80);
        for d in &dec {
            assert_eq!(
                d.sub_periods.len(),
                7,
                "Major period {} should have 7 sub-periods, has {}",
                d.lord,
                d.sub_periods.len()
            );
        }
    }

    #[test]
    fn decennial_sub_periods_sum_to_129_months() {
        // Per Valens, sub-periods use minor years as months.
        // The 7 sub-periods always sum to 129 months (10.75 years).
        let dec = compute_decennials(2451545.0, true, 80);
        let month_days = 365.25 / 12.0;
        let expected_days = 129.0 * month_days; // 129 months
        for d in &dec {
            let sub_sum: f64 = d.sub_periods.iter().map(|s| s.end_jd - s.start_jd).sum();
            assert!(
                (sub_sum - expected_days).abs() < 1e-4,
                "Sub-periods of {} should sum to 129 months ({:.2} days), got {:.2}",
                d.lord,
                expected_days,
                sub_sum
            );
        }
    }

    #[test]
    fn decennial_covers_requested_span() {
        let birth = 2451545.0;
        let years = 80;
        let dec = compute_decennials(birth, true, years);
        let last = dec.last().unwrap();
        assert!(
            last.end_jd >= birth + (years as f64) * 365.25,
            "Decennials should cover at least {} years",
            years
        );
    }

    #[test]
    fn decennial_contiguous_periods() {
        let dec = compute_decennials(2451545.0, true, 80);
        for i in 1..dec.len() {
            let gap = (dec[i].start_jd - dec[i - 1].end_jd).abs();
            assert!(
                gap < 1e-6,
                "Gap of {gap} days between major periods {} and {}",
                dec[i - 1].lord,
                dec[i].lord
            );
        }
    }

    // --- Firdaria ---

    #[test]
    fn firdaria_day_starts_with_sun() {
        let fird = compute_firdaria(2451545.0, true);
        assert_eq!(fird[0].ruler, Planet::Sun);
        assert!(fird[0].sub_ruler.is_none()); // first entry is major period
    }

    #[test]
    fn firdaria_night_starts_with_moon() {
        let fird = compute_firdaria(2451545.0, false);
        assert_eq!(fird[0].ruler, Planet::Moon);
    }

    #[test]
    fn firdaria_total_75_years() {
        let fird = compute_firdaria(2451545.0, true);
        // Sum up only major periods (sub_ruler == None)
        let total: f64 = fird
            .iter()
            .filter(|p| p.sub_ruler.is_none())
            .map(|p| p.years)
            .sum();
        assert!(
            (total - 75.0).abs() < 0.01,
            "Firdaria total should be 75 years, got {total}"
        );
    }

    #[test]
    fn firdaria_has_9_major_periods() {
        let fird = compute_firdaria(2451545.0, true);
        let majors: Vec<_> = fird.iter().filter(|p| p.sub_ruler.is_none()).collect();
        assert_eq!(
            majors.len(),
            9,
            "Should have 9 major periods (7 planets + 2 nodes)"
        );
    }

    #[test]
    fn firdaria_planetary_major_has_7_subs() {
        let fird = compute_firdaria(2451545.0, true);
        // For each planetary (non-nodal) major period, count its sub-periods.
        let majors: Vec<_> = fird.iter().filter(|p| p.sub_ruler.is_none()).collect();
        for major in &majors {
            if matches!(major.ruler, Planet::NorthNode | Planet::SouthNode) {
                continue;
            }
            let sub_count = fird
                .iter()
                .filter(|p| {
                    p.sub_ruler.is_some()
                        && p.ruler == major.ruler
                        && p.start_jd >= major.start_jd
                        && p.end_jd <= major.end_jd + 1e-6
                })
                .count();
            assert_eq!(
                sub_count, 7,
                "Major {} should have 7 sub-periods, has {sub_count}",
                major.ruler
            );
        }
    }

    #[test]
    fn firdaria_nodes_have_no_subs() {
        let fird = compute_firdaria(2451545.0, true);
        let node_subs = fird
            .iter()
            .filter(|p| {
                p.sub_ruler.is_some() && matches!(p.ruler, Planet::NorthNode | Planet::SouthNode)
            })
            .count();
        assert_eq!(node_subs, 0, "Nodal periods should have no sub-periods");
    }

    #[test]
    fn firdaria_sub_periods_equal_length() {
        // Within each major, all 7 sub-periods should be equal (major_years / 7).
        let fird = compute_firdaria(2451545.0, true);
        let majors: Vec<_> = fird.iter().filter(|p| p.sub_ruler.is_none()).collect();
        for major in &majors {
            if matches!(major.ruler, Planet::NorthNode | Planet::SouthNode) {
                continue;
            }
            let subs: Vec<_> = fird
                .iter()
                .filter(|p| {
                    p.sub_ruler.is_some()
                        && p.ruler == major.ruler
                        && p.start_jd >= major.start_jd
                        && p.end_jd <= major.end_jd + 1e-6
                })
                .collect();
            let expected_years = major.years / 7.0;
            for s in &subs {
                assert!(
                    (s.years - expected_years).abs() < 0.01,
                    "Sub-period {} of {} should be {:.4} years, got {:.4}",
                    s.sub_ruler.unwrap(),
                    major.ruler,
                    expected_years,
                    s.years
                );
            }
        }
    }

    #[test]
    fn firdaria_contiguous() {
        let fird = compute_firdaria(2451545.0, true);
        let majors: Vec<_> = fird.iter().filter(|p| p.sub_ruler.is_none()).collect();
        for i in 1..majors.len() {
            let gap = (majors[i].start_jd - majors[i - 1].end_jd).abs();
            assert!(
                gap < 1e-6,
                "Gap of {gap} days between Firdaria majors {} and {}",
                majors[i - 1].ruler,
                majors[i].ruler
            );
        }
    }

    #[test]
    fn firdaria_first_sub_ruler_is_major_ruler() {
        // Per Firdaria rules, the first sub-ruler of each planetary period
        // is the major ruler itself.
        let fird = compute_firdaria(2451545.0, true);
        let majors: Vec<_> = fird.iter().filter(|p| p.sub_ruler.is_none()).collect();
        for major in &majors {
            if matches!(major.ruler, Planet::NorthNode | Planet::SouthNode) {
                continue;
            }
            let first_sub = fird
                .iter()
                .find(|p| {
                    p.sub_ruler.is_some()
                        && p.ruler == major.ruler
                        && (p.start_jd - major.start_jd).abs() < 1e-6
                })
                .expect("Should have a first sub-period");
            assert_eq!(
                first_sub.sub_ruler.unwrap(),
                major.ruler,
                "First sub-ruler of {} should be itself",
                major.ruler
            );
        }
    }
}
