//! # Feng Shui Systems (风水)
//!
//! This module implements two classical Feng Shui systems:
//!
//! 1. **Flying Star Feng Shui (Xuan Kong Fei Xing / 玄空飞星)** -- maps the nine
//!    stars of the Lo Shu magic square to a 3x3 grid, then "flies" them based on
//!    the period (yun), year, and month.
//!
//! 2. **Eight Mansions (Ba Zhai / 八宅)** -- derives a person's Gua number from
//!    birth year and gender, then assigns four auspicious and four inauspicious
//!    compass directions.

use serde::{Deserialize, Serialize};

// ===========================================================================
// Compass Direction
// ===========================================================================

/// The eight compass directions used in both Flying Star and Eight Mansions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    N,
    S,
    E,
    W,
    NE,
    NW,
    SE,
    SW,
}

impl Direction {
    pub const ALL: [Direction; 8] = [
        Direction::N,
        Direction::S,
        Direction::E,
        Direction::W,
        Direction::NE,
        Direction::NW,
        Direction::SE,
        Direction::SW,
    ];

    /// Human-readable name.
    pub fn name(self) -> &'static str {
        match self {
            Direction::N => "North",
            Direction::S => "South",
            Direction::E => "East",
            Direction::W => "West",
            Direction::NE => "Northeast",
            Direction::NW => "Northwest",
            Direction::SE => "Southeast",
            Direction::SW => "Southwest",
        }
    }
}

// ===========================================================================
// Lo Shu Palace Layout
// ===========================================================================

/// Lo Shu palace positions in the 3x3 grid.
///
/// ```text
///   SE(4) | S(9)  | SW(2)
///   ------+-------+------
///   E(3)  | C(5)  | W(7)
///   ------+-------+------
///   NE(8) | N(1)  | NW(6)
/// ```
///
/// Index mapping: grid[row][col] where row 0 = top (south side of the compass).
const LO_SHU_GRID: [[u8; 3]; 3] = [[4, 9, 2], [3, 5, 7], [8, 1, 6]];

/// Palace number to (row, col) mapping in the Lo Shu grid.
fn palace_pos(num: u8) -> (usize, usize) {
    for r in 0..3 {
        for c in 0..3 {
            if LO_SHU_GRID[r][c] == num {
                return (r, c);
            }
        }
    }
    (1, 1) // center fallback
}

/// Direction associated with each Lo Shu palace number.
/// Returns `None` for palace 5 (center).
pub fn palace_direction(num: u8) -> Option<Direction> {
    match num {
        1 => Some(Direction::N),
        2 => Some(Direction::SW),
        3 => Some(Direction::E),
        4 => Some(Direction::SE),
        6 => Some(Direction::NW),
        7 => Some(Direction::W),
        8 => Some(Direction::NE),
        9 => Some(Direction::S),
        _ => None, // 5 = center, no direction
    }
}

// ===========================================================================
// Flying Star (Xuan Kong Fei Xing) flight paths
// ===========================================================================

/// Forward (yang) flight path through Lo Shu palaces.
/// Center -> NW -> W -> NE -> S -> N -> SW -> E -> SE
const FORWARD_FLIGHT: [u8; 9] = [5, 6, 7, 8, 9, 1, 2, 3, 4];

/// Reverse (yin) flight path through Lo Shu palaces.
/// Center -> SE -> E -> SW -> N -> S -> NE -> W -> NW
const REVERSE_FLIGHT: [u8; 9] = [5, 4, 3, 2, 1, 9, 8, 7, 6];

// ===========================================================================
// Flying Star Chart
// ===========================================================================

/// A complete Flying Star Feng Shui chart.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// A Flying Star (Xuan Kong) Feng Shui chart.
pub struct FlyingStarChart {
    /// Base (period) star grid -- the Lo Shu arrangement for the current period.
    pub grid: [[u8; 3]; 3],
    /// The 20-year period number (1-9).
    pub period: u8,
    /// Facing (water) star grid.
    pub facing_star: [[u8; 3]; 3],
    /// Sitting (mountain) star grid.
    pub sitting_star: [[u8; 3]; 3],
}

// ---------------------------------------------------------------------------
// Period calculation
// ---------------------------------------------------------------------------

/// Determine the current 20-year Feng Shui period (1-9) from a Gregorian year.
///
/// Period 1: 1864-1883, Period 2: 1884-1903, ... Period 9: 2024-2043.
/// The cycle repeats every 180 years (9 periods x 20 years).
pub fn current_period(year: i32) -> u8 {
    let offset = (year - 1864).rem_euclid(180);
    (offset / 20) as u8 + 1
}

// ---------------------------------------------------------------------------
// Annual star
// ---------------------------------------------------------------------------

/// Determine the annual flying star number occupying the center palace.
///
/// The annual star cycles backward: 2024 = star 3, 2025 = star 2, 2026 = star 1,
/// 2027 = star 9, and so on. The cycle is based on the Chinese solar year
/// (Li Chun, approx Feb 4), but we use the Gregorian year for simplicity.
///
// Note: annual/monthly star calculation uses Gregorian year/month boundaries.
// For precise Feng Shui, use Li Chun (solar term) as the year boundary.
pub fn annual_center_star(year: i32) -> u8 {
    // 2024 -> 3.  Formula: (12 - (year - 2024)) mod 9, adjusted to 1-9.
    let offset = (year - 2024).rem_euclid(9);
    let star = (3i32 - offset).rem_euclid(9);
    if star == 0 { 9 } else { star as u8 }
}

// ---------------------------------------------------------------------------
// Monthly star
// ---------------------------------------------------------------------------

/// Determine the monthly flying star center number.
///
/// Years are grouped by their annual center star modulo 3:
/// - Group A (annual center 1, 4, 7): month cycle starts at star 8 in Feb
/// - Group B (annual center 2, 5, 8): month cycle starts at star 5 in Feb
/// - Group C (annual center 3, 6, 9): month cycle starts at star 2 in Feb
///
/// Each subsequent month decreases by 1 (wrapping 1 -> 9).
/// `month` is 1-12 (January = 1).
///
// Note: monthly star calculation uses Gregorian month boundaries.
// For precise Feng Shui, use the solar term (Jie Qi) boundary for each month.
pub fn monthly_center_star(year: i32, month: u32) -> u8 {
    let annual = annual_center_star(year);
    let base = match annual % 3 {
        1 => 8u8, // group 1/4/7
        2 => 5u8, // group 2/5/8
        _ => 2u8, // group 3/6/9
    };
    // Feb (month 2) = base.  Each month after Feb decreases by 1.
    // months_from_feb: Jan=-1, Feb=0, Mar=1, ... Dec=10
    let months_from_feb = (month as i32 - 2).rem_euclid(12);
    let star = (base as i32 - months_from_feb).rem_euclid(9);
    if star == 0 { 9 } else { star as u8 }
}

// ---------------------------------------------------------------------------
// Star flight
// ---------------------------------------------------------------------------

/// Fly stars from a given center number into a 3x3 Lo Shu grid.
///
/// `forward = true` uses yang (forward) flight; `false` uses yin (reverse).
///
/// The flight places the center number at palace 5, then distributes the
/// remaining numbers along the flight path with sequential star numbers.
pub fn fly_stars(center: u8, forward: bool) -> [[u8; 3]; 3] {
    let path = if forward {
        &FORWARD_FLIGHT
    } else {
        &REVERSE_FLIGHT
    };
    let mut grid = [[0u8; 3]; 3];

    for (step, &palace_num) in path.iter().enumerate() {
        let star = ((center as i32 - 1 + step as i32).rem_euclid(9) + 1) as u8;
        let (r, c) = palace_pos(palace_num);
        grid[r][c] = star;
    }
    grid
}

// ---------------------------------------------------------------------------
// Composite chart
// ---------------------------------------------------------------------------

/// Generate a Flying Star chart for a given year.
///
/// This produces the period (base) star grid, plus the annual facing and
/// sitting star grids. The facing star uses forward flight; the sitting star
/// uses reverse flight. For a full property-specific chart, the facing
/// direction of the building is needed; this function uses the annual star
/// as both facing and sitting center for a general overview.
pub fn flying_star_chart(year: i32) -> FlyingStarChart {
    let period = current_period(year);
    let base_grid = fly_stars(period, true);
    let annual = annual_center_star(year);
    let facing = fly_stars(annual, true);
    let sitting = fly_stars(annual, false);

    FlyingStarChart {
        grid: base_grid,
        period,
        facing_star: facing,
        sitting_star: sitting,
    }
}

// ---------------------------------------------------------------------------
// Star metadata
// ---------------------------------------------------------------------------

/// Meaning of each flying star (1-9).
pub fn star_meaning(star: u8) -> &'static str {
    match star {
        1 => "Career & Nobility (Tan Lang)",
        2 => "Illness & Affliction (Ju Men)",
        3 => "Conflict & Litigation (Lu Cun)",
        4 => "Romance & Academic (Wen Qu)",
        5 => "Disaster & Misfortune (Lian Zhen)",
        6 => "Authority & Windfall (Wu Qu)",
        7 => "Robbery & Betrayal (Po Jun)",
        8 => "Wealth & Prosperity (Zuo Fu)",
        9 => "Fame & Celebration (You Bi)",
        _ => "Unknown",
    }
}

/// Wu Xing element associated with each flying star.
pub fn star_element(star: u8) -> &'static str {
    match star {
        1 => "Water",
        2 => "Earth",
        3 => "Wood",
        4 => "Wood",
        5 => "Earth",
        6 => "Metal",
        7 => "Metal",
        8 => "Earth",
        9 => "Fire",
        _ => "Unknown",
    }
}

/// Check whether a mountain (sitting) + water (facing) star combination is
/// auspicious.
///
/// The classic auspicious combinations:
/// - "Double 8" (both 8): excellent wealth
/// - "1-6" or "6-1": metal produces water, authority + career
/// - "1-8" or "8-1": water feeds earth indirectly, wealth + career
/// - "6-8" or "8-6": metal and earth support, authority + wealth
/// - "4-1" or "1-4": water nourishes wood, career + academic
/// - "9-9": double fame (timely in period 9)
pub fn is_auspicious_combo(mountain: u8, water: u8) -> bool {
    let pair = if mountain <= water {
        (mountain, water)
    } else {
        (water, mountain)
    };
    matches!(pair, (1, 4) | (1, 6) | (1, 8) | (6, 8) | (8, 8) | (9, 9))
}

// ===========================================================================
// Eight Mansions (Ba Zhai / 八宅)
// ===========================================================================

/// A complete Eight Mansions profile derived from a person's Gua number.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaZhaiProfile {
    /// The person's Gua number (1-9, excluding 5).
    pub gua: u8,
    /// East or West life group.
    pub group: &'static str,
    /// Sheng Qi (Generating Qi) -- best direction.
    pub sheng_qi: Direction,
    /// Tian Yi (Heavenly Doctor) -- health/healing.
    pub tian_yi: Direction,
    /// Yan Nian (Longevity) -- relationships.
    pub yan_nian: Direction,
    /// Fu Wei (Stability) -- personal growth.
    pub fu_wei: Direction,
    /// Huo Hai (Mishap) -- minor setbacks.
    pub huo_hai: Direction,
    /// Liu Sha (Six Killings) -- harm to relationships.
    pub liu_sha: Direction,
    /// Wu Gui (Five Ghosts) -- backstabbing/fire.
    pub wu_gui: Direction,
    /// Jue Ming (Total Loss) -- worst direction.
    pub jue_ming: Direction,
}

// ---------------------------------------------------------------------------
// Gua number
// ---------------------------------------------------------------------------

/// Calculate the personal Gua (Kua) number from birth year and gender.
///
/// ## Pre-2000 births (before 2000)
/// - Males: sum digits of last two digits to a single digit, subtract from 10.
///   Formula: `(10 - digit_sum(year % 100)) % 9`; if 0 use 9; if 5 use 2.
/// - Females: sum digits of last two digits to a single digit, add 5.
///   Formula: `(digit_sum(year % 100) + 5) % 9`; if 0 use 9; if 5 use 8.
///
/// ## Post-2000 births (born 2000 onward)
/// - Males: sum digits of last two digits to a single digit, subtract from 9.
///   Formula: `(9 - digit_sum(year % 100))`; if 0 use 9; if 5 use 2.
/// - Females: sum digits of last two digits to a single digit, add 6.
///   Formula: `(digit_sum(year % 100) + 6) % 9`; if 0 use 9; if 5 use 8.
///
/// Gua 5 does not exist in Ba Zhai -- males with Gua 5 use 2 (Kun), females use 8 (Gen).
///
/// Verified against: https://fengshuinexus.com/feng-shui-rules/how-to-find-feng-shui-life-kua-number/
/// and https://calculator.academy/kua-number-calculator/
pub fn gua_number(year: i32, is_male: bool) -> u8 {
    let last_two = (year % 100).unsigned_abs();
    // Reduce last two digits to a single digit by repeated sum
    let mut s = (last_two / 10) + (last_two % 10);
    while s >= 10 {
        s = (s / 10) + (s % 10);
    }
    if is_male {
        // Pre-2000: subtract from 10; post-2000: subtract from 9
        let base = if year < 2000 { 10u32 } else { 9u32 };
        let raw = if s == 0 { base } else { base - s };
        let gua = if raw == 0 || raw > 9 { 9u8 } else { raw as u8 };
        if gua == 5 { 2 } else { gua }
    } else {
        // Pre-2000: add 5; post-2000: add 6
        let addend = if year < 2000 { 5u32 } else { 6u32 };
        let raw = (s + addend) % 9;
        let gua = if raw == 0 { 9 } else { raw as u8 };
        if gua == 5 { 8 } else { gua }
    }
}

// ---------------------------------------------------------------------------
// Direction tables
// ---------------------------------------------------------------------------

/// Build a Ba Zhai profile from the 8 directions for a given Gua.
///
/// Classical direction assignments per Gua number:
///
/// | Gua | Group | ShengQi | TianYi | YanNian | FuWei | HuoHai | LiuSha | WuGui | JueMing |
/// |-----|-------|---------|--------|---------|-------|--------|--------|-------|---------|
/// |  1  | East  |   SE    |   E    |   S     |  N    |   W    |   NW   |  NE   |   SW    |
/// |  2  | West  |   NE    |   W    |   NW    |  SW   |   E    |   S    |  SE   |   N     |
/// |  3  | East  |   S     |   N    |   SE    |  E    |   SW   |   NE   |  NW   |   W     |
/// |  4  | East  |   N     |   S    |   E     |  SE   |   NW   |   SW   |  W    |   NE    |
/// |  6  | West  |   W     |   NE   |   SW    |  NW   |   SE   |   N    |  E    |   S     |
/// |  7  | West  |   NW    |   SW   |   NE    |  W    |   S    |   SE   |  N    |   E     |
/// |  8  | West  |   SW    |   NW   |   W     |  NE   |   N    |   E    |  S    |   SE    |
/// |  9  | East  |   E     |   SE   |   N     |  S    |   NE   |   W    |  SW   |   NW    |
fn ba_zhai_directions(gua: u8) -> BaZhaiProfile {
    use Direction::*;
    match gua {
        1 => BaZhaiProfile {
            gua,
            group: "East",
            sheng_qi: SE,
            tian_yi: E,
            yan_nian: S,
            fu_wei: N,
            huo_hai: W,
            liu_sha: NW,
            wu_gui: NE,
            jue_ming: SW,
        },
        2 => BaZhaiProfile {
            gua,
            group: "West",
            sheng_qi: NE,
            tian_yi: W,
            yan_nian: NW,
            fu_wei: SW,
            huo_hai: E,
            liu_sha: S,
            wu_gui: SE,
            jue_ming: N,
        },
        3 => BaZhaiProfile {
            gua,
            group: "East",
            sheng_qi: S,
            tian_yi: N,
            yan_nian: SE,
            fu_wei: E,
            huo_hai: SW,
            liu_sha: NE,
            wu_gui: NW,
            jue_ming: W,
        },
        4 => BaZhaiProfile {
            gua,
            group: "East",
            sheng_qi: N,
            tian_yi: S,
            yan_nian: E,
            fu_wei: SE,
            huo_hai: NW,
            liu_sha: SW,
            wu_gui: W,
            jue_ming: NE,
        },
        6 => BaZhaiProfile {
            gua,
            group: "West",
            sheng_qi: W,
            tian_yi: NE,
            yan_nian: SW,
            fu_wei: NW,
            huo_hai: SE,
            liu_sha: N,
            wu_gui: E,
            jue_ming: S,
        },
        7 => BaZhaiProfile {
            gua,
            group: "West",
            sheng_qi: NW,
            tian_yi: SW,
            yan_nian: NE,
            fu_wei: W,
            huo_hai: S,
            liu_sha: SE,
            wu_gui: N,
            jue_ming: E,
        },
        8 => BaZhaiProfile {
            gua,
            group: "West",
            sheng_qi: SW,
            tian_yi: NW,
            yan_nian: W,
            fu_wei: NE,
            huo_hai: N,
            liu_sha: E,
            wu_gui: S,
            jue_ming: SE,
        },
        9 => BaZhaiProfile {
            gua,
            group: "East",
            sheng_qi: E,
            tian_yi: SE,
            yan_nian: N,
            fu_wei: S,
            huo_hai: NE,
            liu_sha: W,
            wu_gui: SW,
            jue_ming: NW,
        },
        // Gua 5 is redirected to 2 (male) or 8 (female) by gua_number(),
        // so it should never reach here.  Fallback to Gua 2.
        _ => ba_zhai_directions(2),
    }
}

// ---------------------------------------------------------------------------
// Public Ba Zhai API
// ---------------------------------------------------------------------------

/// Compute the full Eight Mansions profile for a person.
pub fn ba_zhai(year: i32, is_male: bool) -> BaZhaiProfile {
    let gua = gua_number(year, is_male);
    ba_zhai_directions(gua)
}

/// Return the best (Sheng Qi) direction from a Ba Zhai profile.
pub fn best_direction(profile: &BaZhaiProfile) -> Direction {
    profile.sheng_qi
}

/// Return the worst (Jue Ming) direction from a Ba Zhai profile.
pub fn worst_direction(profile: &BaZhaiProfile) -> Direction {
    profile.jue_ming
}

/// Return all four auspicious directions in order of strength.
pub fn auspicious_directions(profile: &BaZhaiProfile) -> [Direction; 4] {
    [
        profile.sheng_qi,
        profile.tian_yi,
        profile.yan_nian,
        profile.fu_wei,
    ]
}

/// Return all four inauspicious directions in order of severity.
pub fn inauspicious_directions(profile: &BaZhaiProfile) -> [Direction; 4] {
    [
        profile.huo_hai,
        profile.liu_sha,
        profile.wu_gui,
        profile.jue_ming,
    ]
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Flying Star: period calculation
    // -----------------------------------------------------------------------

    #[test]
    fn period_boundaries() {
        assert_eq!(current_period(1864), 1);
        assert_eq!(current_period(1883), 1);
        assert_eq!(current_period(1884), 2);
        assert_eq!(current_period(2003), 7);
        assert_eq!(current_period(2004), 8);
        assert_eq!(current_period(2023), 8);
        assert_eq!(current_period(2024), 9);
        assert_eq!(current_period(2043), 9);
    }

    #[test]
    fn period_wraps_after_180_years() {
        assert_eq!(current_period(1864), current_period(1864 + 180));
        assert_eq!(current_period(2024), current_period(2024 + 180));
    }

    // -----------------------------------------------------------------------
    // Flying Star: annual center star
    // -----------------------------------------------------------------------

    #[test]
    fn annual_star_known_years() {
        assert_eq!(annual_center_star(2024), 3);
        assert_eq!(annual_center_star(2025), 2);
        assert_eq!(annual_center_star(2026), 1);
        assert_eq!(annual_center_star(2027), 9);
        assert_eq!(annual_center_star(2028), 8);
    }

    #[test]
    fn annual_star_always_1_to_9() {
        for y in 1900..2100 {
            let s = annual_center_star(y);
            assert!(
                s >= 1 && s <= 9,
                "Annual star {} out of range for year {}",
                s,
                y
            );
        }
    }

    #[test]
    fn annual_star_9_year_cycle() {
        let s1 = annual_center_star(2024);
        let s2 = annual_center_star(2024 + 9);
        assert_eq!(s1, s2, "Annual star should repeat every 9 years");
    }

    // -----------------------------------------------------------------------
    // Flying Star: monthly center star
    // -----------------------------------------------------------------------

    #[test]
    fn monthly_star_always_1_to_9() {
        for y in 2020..2030 {
            for m in 1..=12 {
                let s = monthly_center_star(y, m);
                assert!(
                    s >= 1 && s <= 9,
                    "Monthly star {} out of range for {}-{}",
                    s,
                    y,
                    m
                );
            }
        }
    }

    #[test]
    fn monthly_star_feb_matches_group_base() {
        // 2024 annual=3, group C (3%3=0) -> base=2 -> Feb center=2
        assert_eq!(monthly_center_star(2024, 2), 2);
        // 2025 annual=2, group B (2%3=2) -> base=5 -> Feb center=5
        assert_eq!(monthly_center_star(2025, 2), 5);
        // 2026 annual=1, group A (1%3=1) -> base=8 -> Feb center=8
        assert_eq!(monthly_center_star(2026, 2), 8);
    }

    // -----------------------------------------------------------------------
    // Flying Star: fly_stars grid
    // -----------------------------------------------------------------------

    #[test]
    fn fly_stars_center_is_input() {
        for center in 1..=9 {
            let grid = fly_stars(center, true);
            assert_eq!(
                grid[1][1], center,
                "Center of grid must be the input star {}",
                center
            );
        }
    }

    #[test]
    fn fly_stars_contains_all_9() {
        let grid = fly_stars(5, true);
        let mut nums: Vec<u8> = grid.iter().flat_map(|row| row.iter().copied()).collect();
        nums.sort();
        assert_eq!(
            nums,
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
            "Grid must contain all numbers 1-9"
        );
    }

    #[test]
    fn fly_stars_forward_vs_reverse_differ() {
        let fwd = fly_stars(5, true);
        let rev = fly_stars(5, false);
        // Both have 5 in center, but the outer 8 cells should differ.
        let mut diffs = 0;
        for r in 0..3 {
            for c in 0..3 {
                if fwd[r][c] != rev[r][c] {
                    diffs += 1;
                }
            }
        }
        assert!(
            diffs > 0,
            "Forward and reverse flight should produce different grids"
        );
    }

    #[test]
    fn fly_stars_center_5_forward_is_lo_shu() {
        // Flying star 5 forward should reproduce the standard Lo Shu square.
        let grid = fly_stars(5, true);
        assert_eq!(
            grid, LO_SHU_GRID,
            "Flying 5 forward must equal the Lo Shu square"
        );
    }

    // -----------------------------------------------------------------------
    // Flying Star: composite chart
    // -----------------------------------------------------------------------

    #[test]
    fn flying_star_chart_period_matches() {
        let chart = flying_star_chart(2024);
        assert_eq!(chart.period, 9);
        let chart2 = flying_star_chart(2020);
        assert_eq!(chart2.period, 8);
    }

    #[test]
    fn flying_star_chart_grids_valid() {
        let chart = flying_star_chart(2025);
        // Each grid must contain exactly 1-9.
        for grid_ref in [&chart.grid, &chart.facing_star, &chart.sitting_star] {
            let mut nums: Vec<u8> = grid_ref
                .iter()
                .flat_map(|row| row.iter().copied())
                .collect();
            nums.sort();
            assert_eq!(nums, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
        }
    }

    // -----------------------------------------------------------------------
    // Flying Star: star metadata
    // -----------------------------------------------------------------------

    #[test]
    fn star_meanings_all_defined() {
        for s in 1..=9 {
            let m = star_meaning(s);
            assert!(!m.contains("Unknown"), "Star {} should have a meaning", s);
        }
    }

    #[test]
    fn star_elements_all_defined() {
        for s in 1..=9 {
            let e = star_element(s);
            assert!(!e.contains("Unknown"), "Star {} should have an element", s);
        }
    }

    #[test]
    fn star_8_is_wealth() {
        assert!(star_meaning(8).contains("Wealth"));
        assert_eq!(star_element(8), "Earth");
    }

    // -----------------------------------------------------------------------
    // Flying Star: auspicious combos
    // -----------------------------------------------------------------------

    #[test]
    fn double_8_is_auspicious() {
        assert!(is_auspicious_combo(8, 8));
    }

    #[test]
    fn auspicious_combos_symmetric() {
        assert_eq!(is_auspicious_combo(1, 6), is_auspicious_combo(6, 1));
        assert_eq!(is_auspicious_combo(1, 8), is_auspicious_combo(8, 1));
        assert_eq!(is_auspicious_combo(6, 8), is_auspicious_combo(8, 6));
    }

    #[test]
    fn inauspicious_combos() {
        assert!(!is_auspicious_combo(2, 5));
        assert!(!is_auspicious_combo(3, 7));
        assert!(!is_auspicious_combo(5, 5));
    }

    // -----------------------------------------------------------------------
    // Ba Zhai: Gua number
    // -----------------------------------------------------------------------

    #[test]
    fn gua_number_known_examples() {
        // Male born 1990: digit_sum(90)=9, 10-9=1 -> Gua 1
        assert_eq!(gua_number(1990, true), 1);
        // Female born 1990: digit_sum(90)=9, (9+5)%9=14%9=5 -> substituted to 8
        assert_eq!(gua_number(1990, false), 8);
        // Male born 1985: digit_sum(85)=13->4, 10-4=6 -> Gua 6
        assert_eq!(gua_number(1985, true), 6);
        // Female born 1985: digit_sum(85)=13->4, (4+5)%9=0 -> 9
        assert_eq!(gua_number(1985, false), 9);
    }

    #[test]
    fn gua_never_5() {
        for y in 1950..2020 {
            for male in [true, false] {
                let g = gua_number(y, male);
                assert_ne!(
                    g, 5,
                    "Gua 5 should never be returned (year={}, male={})",
                    y, male
                );
            }
        }
    }

    #[test]
    fn gua_always_valid_range() {
        for y in 1900..2100 {
            for male in [true, false] {
                let g = gua_number(y, male);
                assert!(
                    g >= 1 && g <= 9 && g != 5,
                    "Gua {} invalid for year={}, male={}",
                    g,
                    y,
                    male
                );
            }
        }
    }

    #[test]
    fn gua_male_5_becomes_2() {
        // Pre-2000: digit_sum(05)=5, 10-5=5 -> becomes 2
        assert_eq!(gua_number(1905, true), 2, "Male Gua 5 should become 2");
        // Post-2000: digit_sum(05)=5, 9-5=4, not 5, so stays 4
        assert_eq!(gua_number(2005, true), 4);
        // Post-2000: digit_sum(14)=5, 9-5=4
        assert_eq!(gua_number(2014, true), 4);
    }

    #[test]
    fn gua_female_5_becomes_8() {
        // Female 1909: digit_sum(09)=9, (9+5)%9=14%9=5 -> becomes 8
        assert_eq!(gua_number(1909, false), 8, "Female Gua 5 should become 8");
    }

    #[test]
    fn gua_post_2000_male() {
        // Verified: https://fengshuinexus.com/feng-shui-rules/how-to-find-feng-shui-life-kua-number/
        // Post-2000 male formula: 9 - digit_sum(last_two)
        // Male 2000: digit_sum(00)=0, 9-0=9
        assert_eq!(gua_number(2000, true), 9, "Male 2000 Kua should be 9");
        // Male 2001: digit_sum(01)=1, 9-1=8
        assert_eq!(gua_number(2001, true), 8);
        // Male 2009: digit_sum(09)=9, 9-9=0 -> 9
        assert_eq!(gua_number(2009, true), 9);
    }

    // -----------------------------------------------------------------------
    // Ba Zhai: direction profiles
    // -----------------------------------------------------------------------

    #[test]
    fn ba_zhai_east_group() {
        let profile = ba_zhai(1990, true); // Gua 1
        assert_eq!(profile.group, "East");
        assert_eq!(profile.gua, 1);
        assert_eq!(profile.sheng_qi, Direction::SE);
        assert_eq!(profile.jue_ming, Direction::SW);
    }

    #[test]
    fn ba_zhai_west_group() {
        let profile = ba_zhai(1985, true); // Gua 6
        assert_eq!(profile.group, "West");
        assert_eq!(profile.gua, 6);
        assert_eq!(profile.sheng_qi, Direction::W);
        assert_eq!(profile.jue_ming, Direction::S);
    }

    #[test]
    fn ba_zhai_best_worst() {
        let profile = ba_zhai(1990, true); // Gua 1
        assert_eq!(best_direction(&profile), Direction::SE);
        assert_eq!(worst_direction(&profile), Direction::SW);
    }

    #[test]
    fn ba_zhai_auspicious_inauspicious_no_overlap() {
        for y in 1970..1980 {
            for male in [true, false] {
                let p = ba_zhai(y, male);
                let ausp = auspicious_directions(&p);
                let inausp = inauspicious_directions(&p);
                for a in &ausp {
                    for i in &inausp {
                        assert_ne!(
                            a, i,
                            "Gua {}: {:?} appears in both auspicious and inauspicious",
                            p.gua, a
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn ba_zhai_covers_all_8_directions() {
        for gua in [1u8, 2, 3, 4, 6, 7, 8, 9] {
            let profile = ba_zhai_directions(gua);
            let mut dirs = vec![
                profile.sheng_qi,
                profile.tian_yi,
                profile.yan_nian,
                profile.fu_wei,
                profile.huo_hai,
                profile.liu_sha,
                profile.wu_gui,
                profile.jue_ming,
            ];
            dirs.sort_by_key(|d| *d as u8);
            dirs.dedup();
            assert_eq!(
                dirs.len(),
                8,
                "Gua {} must assign all 8 unique directions",
                gua
            );
        }
    }

    // -----------------------------------------------------------------------
    // Direction enum
    // -----------------------------------------------------------------------

    #[test]
    fn direction_names() {
        assert_eq!(Direction::N.name(), "North");
        assert_eq!(Direction::SE.name(), "Southeast");
        assert_eq!(Direction::NW.name(), "Northwest");
    }

    #[test]
    fn all_8_directions() {
        assert_eq!(Direction::ALL.len(), 8);
    }

    // -----------------------------------------------------------------------
    // Palace helpers
    // -----------------------------------------------------------------------

    #[test]
    fn palace_directions_match_lo_shu() {
        assert_eq!(palace_direction(1), Some(Direction::N));
        assert_eq!(palace_direction(9), Some(Direction::S));
        assert_eq!(palace_direction(3), Some(Direction::E));
        assert_eq!(palace_direction(7), Some(Direction::W));
        assert_eq!(palace_direction(5), None); // center
    }
}
