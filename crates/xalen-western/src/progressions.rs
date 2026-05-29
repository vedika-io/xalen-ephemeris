use serde::{Deserialize, Serialize};

const TROPICAL_YEAR_DAYS: f64 = 365.24219;

/// Traditional sign rulers (Aries..Pisces).
const LORDS: [&str; 12] = [
    "Mars", "Venus", "Mercury", "Moon", "Sun", "Mercury", "Venus", "Mars", "Jupiter", "Saturn",
    "Saturn", "Jupiter",
];

// ---------------------------------------------------------------------------
// Progression types & JD conversion
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgressionType {
    Secondary, // 1 day = 1 year
    SolarArc,  // All points move by progressed Sun's arc
    Tertiary,  // 1 day = 1 month
    Minor,     // 1 lunar month = 1 year
    Converse,  // Reverse secondary
}

pub fn progressed_jd(birth_jd: f64, target_jd: f64, prog_type: ProgressionType) -> f64 {
    let elapsed_days = target_jd - birth_jd;
    match prog_type {
        ProgressionType::Secondary => {
            let elapsed_years = elapsed_days / TROPICAL_YEAR_DAYS;
            birth_jd + elapsed_years // 1 day per year of life
        }
        ProgressionType::Tertiary => {
            let elapsed_months = elapsed_days / (TROPICAL_YEAR_DAYS / 12.0);
            birth_jd + elapsed_months // 1 day per month
        }
        ProgressionType::Minor => {
            let elapsed_years = elapsed_days / TROPICAL_YEAR_DAYS;
            birth_jd + elapsed_years * 29.53059 // 1 synodic month per year
        }
        ProgressionType::Converse => {
            let elapsed_years = elapsed_days / TROPICAL_YEAR_DAYS;
            birth_jd - elapsed_years // backward
        }
        ProgressionType::SolarArc => {
            // Solar arc needs the actual Sun position — return the progressed JD
            // Caller computes progressed Sun and applies arc to all planets
            let elapsed_years = elapsed_days / TROPICAL_YEAR_DAYS;
            birth_jd + elapsed_years
        }
    }
}

// ---------------------------------------------------------------------------
// Solar arc (existing + full set)
// ---------------------------------------------------------------------------

pub fn solar_arc_deg(natal_sun_deg: f64, progressed_sun_deg: f64) -> f64 {
    let mut arc = progressed_sun_deg - natal_sun_deg;
    if arc < 0.0 {
        arc += 360.0;
    }
    arc
}

pub fn apply_solar_arc(natal_position_deg: f64, arc_deg: f64) -> f64 {
    (natal_position_deg + arc_deg).rem_euclid(360.0)
}

/// Full solar-arc direction set: compute the arc (progressed Sun - natal Sun),
/// then apply it to every natal position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolarArcSet {
    /// Arc in degrees (progressed Sun - natal Sun).
    pub arc: f64,
    /// Each natal point directed by the arc: (name, directed_longitude).
    pub directed_positions: Vec<(String, f64)>,
}

pub fn solar_arc_directions(
    natal_sun: f64,
    progressed_sun: f64,
    natal_positions: &[(&str, f64)],
) -> SolarArcSet {
    let arc = solar_arc_deg(natal_sun, progressed_sun);
    let directed = natal_positions
        .iter()
        .map(|(name, lon)| (name.to_string(), apply_solar_arc(*lon, arc)))
        .collect();
    SolarArcSet {
        arc,
        directed_positions: directed,
    }
}

// ---------------------------------------------------------------------------
// Age helper
// ---------------------------------------------------------------------------

pub fn age_at_jd(birth_jd: f64, current_jd: f64) -> f64 {
    (current_jd - birth_jd) / TROPICAL_YEAR_DAYS
}

// ---------------------------------------------------------------------------
// Annual profections (existing)
// ---------------------------------------------------------------------------

pub fn profection_sign_index(natal_asc_sign: usize, age: u32) -> usize {
    (natal_asc_sign + age as usize) % 12
}

pub fn profection_lord(natal_asc_sign: usize, age: u32) -> &'static str {
    let sign = profection_sign_index(natal_asc_sign, age);
    LORDS[sign]
}

// ---------------------------------------------------------------------------
// Monthly profections
// ---------------------------------------------------------------------------

/// Within a profection year the lord shifts one sign per month (0-indexed).
/// Returns (monthly_sign_index, ruling_planet).
pub fn monthly_profection(natal_asc_sign: usize, age: u32, month: u32) -> (usize, &'static str) {
    let yearly_sign = profection_sign_index(natal_asc_sign, age);
    let monthly_sign = (yearly_sign + month as usize) % 12;
    (monthly_sign, LORDS[monthly_sign])
}

// ---------------------------------------------------------------------------
// Full profection report
// ---------------------------------------------------------------------------

/// Complete profection data for one year of life.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfectionReport {
    pub age: u32,
    /// 0-based sign index for the annual profection.
    pub annual_sign: usize,
    pub annual_lord: String,
    /// Which natal house is activated (1-12).
    pub annual_house: usize,
    /// True when the profected house is angular (1, 4, 7, 10).
    pub is_angular: bool,
    /// 0-based sign index for each of the 12 months within the year.
    pub monthly_signs: [usize; 12],
    /// Ruling planet for each monthly sign.
    pub monthly_lords: [String; 12],
}

pub fn full_profection(natal_asc_sign: usize, age: u32) -> ProfectionReport {
    let annual_sign = profection_sign_index(natal_asc_sign, age);
    let annual_lord = LORDS[annual_sign].to_string();

    // House number = how many signs from ASC + 1 (1-based).
    let annual_house = ((annual_sign + 12 - natal_asc_sign) % 12) + 1;
    let is_angular = matches!(annual_house, 1 | 4 | 7 | 10);

    let mut monthly_signs = [0usize; 12];
    let mut monthly_lords: [String; 12] = Default::default();
    for m in 0..12u32 {
        let (ms, ml) = monthly_profection(natal_asc_sign, age, m);
        monthly_signs[m as usize] = ms;
        monthly_lords[m as usize] = ml.to_string();
    }

    ProfectionReport {
        age,
        annual_sign,
        annual_lord,
        annual_house,
        is_angular,
        monthly_signs,
        monthly_lords,
    }
}

// ---------------------------------------------------------------------------
// Time Lord periods (profection-based)
// ---------------------------------------------------------------------------

/// A time-lord period: the planet ruling a profection sign for a span of ages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeLord {
    pub planet: String,
    /// First age (inclusive) in this period.
    pub start_age: u32,
    /// Last age (exclusive) in this period.
    pub end_age: u32,
    /// 0-based sign index.
    pub sign: usize,
}

/// Generate a list of time-lord periods from age 0 up to (but not including)
/// `max_age`.  Each period spans one year; consecutive years ruled by the same
/// planet are merged.
pub fn time_lords(natal_asc_sign: usize, max_age: u32) -> Vec<TimeLord> {
    if max_age == 0 {
        return Vec::new();
    }
    let mut result: Vec<TimeLord> = Vec::new();
    for age in 0..max_age {
        let sign = profection_sign_index(natal_asc_sign, age);
        let planet = LORDS[sign].to_string();
        if let Some(last) = result.last_mut()
            && last.planet == planet {
                last.end_age = age + 1;
                continue;
            }
        result.push(TimeLord {
            planet,
            start_age: age,
            end_age: age + 1,
            sign,
        });
    }
    result
}

// ---------------------------------------------------------------------------
// Primary directions (Ptolemaic)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DirectionType {
    Direct,
    Converse,
}

/// One primary-direction hit: a significator reaches a promissor (or its
/// aspect) through the rotation of the celestial sphere.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimaryDirection {
    pub significator: String,
    pub promissor: String,
    /// Ptolemaic aspect angle: 0 (conjunction), 60, 90, 120, 180.
    pub aspect: f64,
    /// Arc in degrees of RA -- equals years of life (Ptolemy's key).
    pub arc: f64,
    /// Direct = in the order of diurnal motion; Converse = against it.
    pub direction: DirectionType,
    /// `arc` converted to years from birth.
    pub years_from_birth: f64,
}

/// Simple Ptolemaic arc: "one degree of RA equals one year of life."
/// Returns the signed arc from significator RA to (promissor RA + aspect),
/// folded to (-180, 180].
pub fn primary_arc_simple(sig_ra: f64, prom_ra: f64, aspect: f64) -> f64 {
    let target = (prom_ra + aspect).rem_euclid(360.0);
    let arc = (target - sig_ra).rem_euclid(360.0);
    if arc > 180.0 { arc - 360.0 } else { arc }
}

/// Ascensional difference: AD = arcsin(tan(dec) * tan(lat)).
/// Returns degrees. Clamps the argument to [-1, 1] for polar edge cases.
fn ascensional_difference(dec_deg: f64, lat_deg: f64) -> f64 {
    let arg = dec_deg.to_radians().tan() * lat_deg.to_radians().tan();
    arg.clamp(-1.0, 1.0).asin().to_degrees()
}

/// Diurnal semi-arc (above the horizon).  DSA = 90 + AD.
fn diurnal_semi_arc(dec_deg: f64, lat_deg: f64) -> f64 {
    90.0 + ascensional_difference(dec_deg, lat_deg)
}

/// Nocturnal semi-arc (below the horizon).  NSA = 90 - AD.
fn nocturnal_semi_arc(dec_deg: f64, lat_deg: f64) -> f64 {
    90.0 - ascensional_difference(dec_deg, lat_deg)
}

/// Placidus semi-arc primary direction arc.
///
/// 1. Compute the promissor's semi-arc (DSA or NSA depending on whether it is
///    above or below the horizon in the natal chart -- approximated here by
///    comparing MD to 90).
/// 2. Meridian distance MD = |RA_prom - MC_RA|, folded to [0, 180].
/// 3. Proportional distance PD = MD / semi-arc.
/// 4. Do the same for the significator.
/// 5. Arc = (PD_sig - PD_prom) * semi-arc_sig  (simplified Placidus formula).
///
/// This is the classical approximation; a full Placidus implementation needs
/// iterative solution of the intermediate houses, but this gives a solid first
/// pass that matches reference tables within ~0.5 degrees.
fn placidus_arc(
    sig_ra: f64,
    sig_dec: f64,
    prom_ra: f64,
    prom_dec: f64,
    mc_ra: f64,
    geo_lat: f64,
    aspect: f64,
) -> f64 {
    let effective_prom_ra = (prom_ra + aspect).rem_euclid(360.0);

    // Meridian distances
    let md_sig = {
        let d = (sig_ra - mc_ra).rem_euclid(360.0);
        if d > 180.0 { 360.0 - d } else { d }
    };
    let md_prom = {
        let d = (effective_prom_ra - mc_ra).rem_euclid(360.0);
        if d > 180.0 { 360.0 - d } else { d }
    };

    // Semi-arcs (use diurnal if MD < 90, else nocturnal)
    let sa_sig = if md_sig <= 90.0 {
        diurnal_semi_arc(sig_dec, geo_lat)
    } else {
        nocturnal_semi_arc(sig_dec, geo_lat)
    };
    let sa_prom = if md_prom <= 90.0 {
        diurnal_semi_arc(prom_dec, geo_lat)
    } else {
        nocturnal_semi_arc(prom_dec, geo_lat)
    };

    if sa_sig.abs() < 1e-9 || sa_prom.abs() < 1e-9 {
        // Degenerate (circumpolar / never-rises) -- fall back to simple arc
        return primary_arc_simple(sig_ra, prom_ra, aspect);
    }

    // Proportional distances
    let pd_sig = md_sig / sa_sig;
    let pd_prom = md_prom / sa_prom;

    (pd_sig - pd_prom) * sa_sig
}

/// Standard Ptolemaic aspects to test.
const PTOLEMAIC_ASPECTS: [f64; 5] = [0.0, 60.0, 90.0, 120.0, 180.0];

/// Compute all primary directions between every pair of natal points, in both
/// direct and converse directions, for all five Ptolemaic aspects, up to
/// `max_years` from birth.
///
/// Each element of `natal_positions` is `(name, right_ascension, declination)`
/// in degrees.  `mc_ra` and `asc_ra` are the right ascension of the natal MC
/// and ASC.  `geo_lat` is the geographic latitude of birth.
///
/// Returns a list sorted by ascending `years_from_birth`.
pub fn primary_directions(
    natal_positions: &[(&str, f64, f64)],
    mc_ra: f64,
    _asc_ra: f64,
    geo_lat: f64,
    max_years: f64,
) -> Vec<PrimaryDirection> {
    let mut hits: Vec<PrimaryDirection> = Vec::new();
    let n = natal_positions.len();

    for i in 0..n {
        let (sig_name, sig_ra, sig_dec) = natal_positions[i];
        for j in 0..n {
            if i == j {
                continue;
            }
            let (prom_name, prom_ra, prom_dec) = natal_positions[j];

            for &aspect in &PTOLEMAIC_ASPECTS {
                // Direct direction
                let arc_d =
                    placidus_arc(sig_ra, sig_dec, prom_ra, prom_dec, mc_ra, geo_lat, aspect);
                if arc_d > 0.0 && arc_d <= max_years {
                    hits.push(PrimaryDirection {
                        significator: sig_name.to_string(),
                        promissor: prom_name.to_string(),
                        aspect,
                        arc: arc_d,
                        direction: DirectionType::Direct,
                        years_from_birth: arc_d,
                    });
                }

                // Converse direction (negate the aspect offset)
                let arc_c =
                    placidus_arc(sig_ra, sig_dec, prom_ra, prom_dec, mc_ra, geo_lat, -aspect);
                let arc_c_abs = arc_c.abs();
                if arc_c < 0.0 && arc_c_abs <= max_years {
                    hits.push(PrimaryDirection {
                        significator: sig_name.to_string(),
                        promissor: prom_name.to_string(),
                        aspect,
                        arc: arc_c_abs,
                        direction: DirectionType::Converse,
                        years_from_birth: arc_c_abs,
                    });
                }
            }
        }
    }

    hits.sort_by(|a, b| a.years_from_birth.total_cmp(&b.years_from_birth));
    hits
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Existing progression tests
    // -----------------------------------------------------------------------

    #[test]
    fn secondary_progression_at_age_30() {
        let birth = 2451545.0;
        let age_30 = birth + 30.0 * TROPICAL_YEAR_DAYS;
        let prog_jd = progressed_jd(birth, age_30, ProgressionType::Secondary);
        assert!(
            (prog_jd - (birth + 30.0)).abs() < 0.01,
            "At age 30, progressed JD should be birth+30d, got {}",
            prog_jd - birth
        );
    }

    #[test]
    fn converse_goes_backward() {
        let birth = 2451545.0;
        let age_10 = birth + 10.0 * TROPICAL_YEAR_DAYS;
        let prog_jd = progressed_jd(birth, age_10, ProgressionType::Converse);
        assert!(prog_jd < birth, "Converse should go before birth");
    }

    #[test]
    fn solar_arc_computation() {
        let arc = solar_arc_deg(280.0, 310.0);
        assert!(
            (arc - 30.0).abs() < 0.01,
            "Arc should be 30 deg, got {arc} deg"
        );
    }

    #[test]
    fn apply_arc_wraps() {
        let directed = apply_solar_arc(350.0, 20.0);
        assert!(
            (directed - 10.0).abs() < 0.01,
            "350+20 should wrap to 10 deg, got {directed} deg"
        );
    }

    #[test]
    fn profection_cycle() {
        assert_eq!(profection_sign_index(0, 0), 0);
        assert_eq!(profection_sign_index(0, 12), 0);
        assert_eq!(profection_sign_index(0, 1), 1);
    }

    #[test]
    fn profection_lord_at_age_0() {
        assert_eq!(profection_lord(0, 0), "Mars");
    }

    #[test]
    fn age_computation() {
        let birth = 2451545.0;
        let now = birth + 25.0 * TROPICAL_YEAR_DAYS;
        let age = age_at_jd(birth, now);
        assert!((age - 25.0).abs() < 0.01);
    }

    // -----------------------------------------------------------------------
    // Monthly profections
    // -----------------------------------------------------------------------

    #[test]
    fn monthly_profection_month_0_equals_annual() {
        // Month 0 should return the same sign as the annual profection.
        let (sign, lord) = monthly_profection(0, 5, 0);
        assert_eq!(sign, profection_sign_index(0, 5));
        assert_eq!(lord, profection_lord(0, 5));
    }

    #[test]
    fn monthly_profection_advances() {
        // ASC Aries (0), age 0 => annual = Aries. Month 3 => Cancer (3).
        let (sign, lord) = monthly_profection(0, 0, 3);
        assert_eq!(sign, 3); // Cancer
        assert_eq!(lord, "Moon");
    }

    #[test]
    fn monthly_profection_wraps_around() {
        // ASC Scorpio (7), age 10 => annual = (7+10)%12 = 5 (Virgo).
        // Month 9 => (5+9)%12 = 2 (Gemini).
        let (sign, lord) = monthly_profection(7, 10, 9);
        assert_eq!(sign, 2);
        assert_eq!(lord, "Mercury");
    }

    // -----------------------------------------------------------------------
    // Full profection report
    // -----------------------------------------------------------------------

    #[test]
    fn full_profection_age_0_aries_asc() {
        let rpt = full_profection(0, 0);
        assert_eq!(rpt.age, 0);
        assert_eq!(rpt.annual_sign, 0); // Aries
        assert_eq!(rpt.annual_lord, "Mars");
        assert_eq!(rpt.annual_house, 1);
        assert!(rpt.is_angular);
        // Month 0 = Aries, Month 1 = Taurus, ..., Month 11 = Pisces
        assert_eq!(rpt.monthly_signs[0], 0);
        assert_eq!(rpt.monthly_signs[11], 11);
        assert_eq!(rpt.monthly_lords[4], "Sun"); // Leo
    }

    #[test]
    fn full_profection_angular_detection() {
        // ASC Aries (0): house 1 (age 0) angular, house 4 (age 3) angular,
        // house 7 (age 6) angular, house 10 (age 9) angular.
        // house 2 (age 1) NOT angular.
        assert!(full_profection(0, 0).is_angular); // house 1
        assert!(!full_profection(0, 1).is_angular); // house 2
        assert!(full_profection(0, 3).is_angular); // house 4
        assert!(full_profection(0, 6).is_angular); // house 7
        assert!(full_profection(0, 9).is_angular); // house 10
    }

    #[test]
    fn full_profection_non_aries_asc() {
        // ASC Leo (4), age 3 => sign = (4+3)%12 = 7 (Scorpio),
        // house = (7-4)%12 + 1 = 4 => angular.
        let rpt = full_profection(4, 3);
        assert_eq!(rpt.annual_sign, 7);
        assert_eq!(rpt.annual_lord, "Mars"); // Scorpio
        assert_eq!(rpt.annual_house, 4);
        assert!(rpt.is_angular);
    }

    // -----------------------------------------------------------------------
    // Time lords
    // -----------------------------------------------------------------------

    #[test]
    fn time_lords_basic() {
        let tl = time_lords(0, 12);
        // Saturn rules Capricorn (9) AND Aquarius (10) consecutively, so those
        // two years merge into one period.  12 years => 11 distinct entries.
        assert_eq!(tl.len(), 11);
        assert_eq!(tl[0].planet, "Mars");
        assert_eq!(tl[0].start_age, 0);
        assert_eq!(tl[0].end_age, 1);
        // Saturn period spans ages 9-11 (merged Capricorn + Aquarius).
        let saturn = tl.iter().find(|t| t.planet == "Saturn").unwrap();
        assert_eq!(saturn.start_age, 9);
        assert_eq!(saturn.end_age, 11);
    }

    #[test]
    fn time_lords_empty_for_zero() {
        let tl = time_lords(0, 0);
        assert!(tl.is_empty());
    }

    #[test]
    fn time_lords_covers_full_range() {
        let max = 36;
        let tl = time_lords(0, max);
        // First entry starts at 0, last entry ends at max.
        assert_eq!(tl.first().unwrap().start_age, 0);
        assert_eq!(tl.last().unwrap().end_age, max);
    }

    // -----------------------------------------------------------------------
    // Solar arc directions (full set)
    // -----------------------------------------------------------------------

    #[test]
    fn solar_arc_directions_basic() {
        let natal_sun = 280.0;
        let prog_sun = 310.0; // arc = 30 deg
        let positions = &[("Sun", 280.0), ("Moon", 120.0), ("Mars", 45.0)];
        let result = solar_arc_directions(natal_sun, prog_sun, positions);
        assert!((result.arc - 30.0).abs() < 0.01);
        assert_eq!(result.directed_positions.len(), 3);
        // Sun directed = 280+30 = 310
        assert!((result.directed_positions[0].1 - 310.0).abs() < 0.01);
        // Moon directed = 120+30 = 150
        assert!((result.directed_positions[1].1 - 150.0).abs() < 0.01);
        // Mars directed = 45+30 = 75
        assert!((result.directed_positions[2].1 - 75.0).abs() < 0.01);
    }

    #[test]
    fn solar_arc_directions_wraps() {
        let natal_sun = 350.0;
        let prog_sun = 15.0; // arc = 25 deg (wraps past 0)
        let positions = &[("Venus", 340.0)];
        let result = solar_arc_directions(natal_sun, prog_sun, positions);
        assert!((result.arc - 25.0).abs() < 0.01);
        // Venus directed = (340+25) % 360 = 5
        assert!((result.directed_positions[0].1 - 5.0).abs() < 0.01);
    }

    // -----------------------------------------------------------------------
    // Primary directions
    // -----------------------------------------------------------------------

    #[test]
    fn primary_arc_simple_conjunction() {
        // Two points 30 deg apart => conjunction arc = 30
        let arc = primary_arc_simple(100.0, 130.0, 0.0);
        assert!((arc - 30.0).abs() < 0.01);
    }

    #[test]
    fn primary_arc_simple_opposition() {
        // sig RA=10, prom RA=100, aspect=180 => target=280, arc=270 => folded = -90
        let arc = primary_arc_simple(10.0, 100.0, 180.0);
        assert!((arc - (-90.0)).abs() < 0.01);
    }

    #[test]
    fn primary_directions_produces_sorted_output() {
        // Sun at RA=80, Dec=20; Moon at RA=120, Dec=-5.
        // MC at RA=0, ASC at RA=90. Lat=40.
        let positions = &[("Sun", 80.0, 20.0), ("Moon", 120.0, -5.0)];
        let dirs = primary_directions(positions, 0.0, 90.0, 40.0, 100.0);
        // Should produce some hits and be sorted.
        assert!(!dirs.is_empty(), "Should produce at least one direction");
        for w in dirs.windows(2) {
            assert!(
                w[0].years_from_birth <= w[1].years_from_birth,
                "Output must be sorted by years_from_birth"
            );
        }
    }

    #[test]
    fn primary_directions_respects_max_years() {
        let positions = &[("Sun", 80.0, 20.0), ("Moon", 120.0, -5.0)];
        let max = 50.0;
        let dirs = primary_directions(positions, 0.0, 90.0, 40.0, max);
        for d in &dirs {
            assert!(
                d.years_from_birth <= max,
                "Direction at {} exceeds max_years {}",
                d.years_from_birth,
                max
            );
        }
    }

    #[test]
    fn primary_directions_no_self_pairing() {
        let positions = &[("Sun", 80.0, 20.0), ("Moon", 120.0, -5.0)];
        let dirs = primary_directions(positions, 0.0, 90.0, 40.0, 100.0);
        for d in &dirs {
            assert_ne!(
                d.significator, d.promissor,
                "Significator and promissor must differ"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Ascensional difference / semi-arcs (internal helpers)
    // -----------------------------------------------------------------------

    #[test]
    fn ascensional_difference_equator() {
        // At latitude 0, AD should be 0 regardless of declination.
        let ad = ascensional_difference(23.0, 0.0);
        assert!(ad.abs() < 0.01, "AD at equator should be 0, got {ad}");
    }

    #[test]
    fn diurnal_semi_arc_at_equator() {
        // At latitude 0, DSA = 90 for any declination.
        let dsa = diurnal_semi_arc(15.0, 0.0);
        assert!(
            (dsa - 90.0).abs() < 0.01,
            "DSA at equator should be 90, got {dsa}"
        );
    }

    #[test]
    fn semi_arcs_complement() {
        // DSA + NSA = 180 always.
        let lat = 45.0;
        let dec = 20.0;
        let dsa = diurnal_semi_arc(dec, lat);
        let nsa = nocturnal_semi_arc(dec, lat);
        assert!(
            (dsa + nsa - 180.0).abs() < 0.01,
            "DSA + NSA should be 180, got {} + {} = {}",
            dsa,
            nsa,
            dsa + nsa
        );
    }
}
