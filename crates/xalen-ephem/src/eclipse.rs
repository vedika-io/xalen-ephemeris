//! Solar and lunar eclipse detection and classification.
//!
//! # Approach
//!
//! 1. Step through the date range; locate exact New Moon (solar) or Full Moon
//!    (lunar) by bisection on the Sun–Moon elongation function.
//! 2. A cheap `|Moon ecliptic latitude|` pre-filter nominates candidate
//!    lunations near an ecliptic node.
//! 3. **Solar eclipses** are then determined and classified by the rigorous
//!    Besselian shadow geometry ([`crate::besselian`]): γ (least shadow-axis
//!    distance from Earth's centre) gives the global type and confirms whether
//!    an eclipse actually occurs — NASA-validated for 2017/2024.
//! 4. **Lunar eclipses** are still classified by comparing the Moon's ecliptic
//!    latitude against the Earth-shadow cone limits (Meeus Ch. 54); a Besselian
//!    lunar treatment is not yet implemented.
//!
//! # Magnitude honesty
//!
//! Neither result type exposes a true astronomical eclipse magnitude. Solar
//! events carry [`SolarEclipse::coverage_proxy`] (a geocentric diameter-ratio /
//! parallax-overlap figure) and lunar events carry
//! [`LunarEclipse::shadow_depth_proxy`] (a latitude-derived depth measure).
//! Both are honestly named proxies — the rigorous per-observer/umbral magnitude
//! is not computed. The authoritative geometric quantity for solar eclipses is
//! [`SolarEclipse::gamma`].

use crate::almanac::Almanac;
use crate::besselian::{GlobalSolarType, classify_solar_eclipse};
use crate::body::Body;
use xalen_time::{DeltaTModel, JdUT1, JulianDay, delta_t};

// ── Constants ────────────────────────────────────────────────────────────────

/// Step size for scanning elongation (days). Must be < ~14.7 days (half a
/// synodic month) to guarantee we never skip a lunation.
const SCAN_STEP: f64 = 1.0;

/// Minimum gap between consecutive syzygies of the same type (days).
/// The synodic month is ~29.53 days; after finding a New/Full Moon we skip
/// forward by this amount to avoid double-counting the same lunation.
const MIN_SYNODIC_GAP: f64 = 25.0;

// Lunar eclipse latitude thresholds (degrees, absolute value).
// Derived from Meeus Ch. 54 / Explanatory Supplement Table 11.1a.
// The penumbral limit is the maximum latitude at which the Moon can enter
// Earth's penumbral shadow. The partial limit is where the umbral shadow
// begins to cover the Moon, and the total limit is full umbral immersion.
const LUNAR_PENUMBRAL_LIMIT: f64 = 1.58;
const LUNAR_PARTIAL_LIMIT: f64 = 0.85;
const LUNAR_TOTAL_LIMIT: f64 = 0.43;

// Solar eclipse latitude pre-filter (degrees, absolute value). The maximum
// |Moon latitude| at New Moon for any eclipse geometry is ~1.58° (Meeus Ch. 55);
// we widen it to 1.9° as a deliberately CONSERVATIVE pre-filter so it never
// suppresses a marginal eclipse. It only nominates candidate lunations — the
// Besselian engine (`besselian::classify_solar_eclipse`) is the authority on
// whether an eclipse actually occurs and its type.
const SOLAR_ECLIPSE_LIMIT: f64 = 1.9;

// Sub-classification constants for solar eclipses.
// Mean apparent semi-diameters (degrees).
const SUN_MEAN_ANGULAR_RADIUS_DEG: f64 = 0.2666; // ~16 arcmin
const MOON_MEAN_ANGULAR_RADIUS_DEG: f64 = 0.2586; // ~15.5 arcmin at mean distance

// Mean distances for apparent-size scaling.
const MOON_MEAN_DISTANCE_AU: f64 = 0.00257;
const SUN_MEAN_DISTANCE_AU: f64 = 1.0;

// ── Result types ─────────────────────────────────────────────────────────────

/// Classification of a lunar eclipse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LunarEclipseType {
    /// The Moon passes through Earth's penumbral shadow only.
    Penumbral,
    /// Part of the Moon enters the umbral shadow.
    Partial,
    /// The entire Moon is immersed in the umbral shadow.
    Total,
}

/// A single lunar eclipse event.
#[derive(Debug, Clone)]
pub struct LunarEclipse {
    /// Julian Date (UT1) of the Full Moon (syzygy) instant.
    ///
    /// NOTE: unlike [`SolarEclipse::jd_maximum`], which is the Besselian
    /// greatest-eclipse time (closest shadow-axis approach), this is the exact
    /// Sun–Moon opposition instant. For lunar eclipses the geocentric maximum
    /// (deepest immersion in Earth's shadow) is very close to syzygy — within a
    /// few minutes — but the two are not identical. A rigorous greatest-eclipse
    /// reduction for lunar eclipses is not yet implemented; this field reports
    /// the syzygy, which always lies inside the requested `[jd_start, jd_end]`.
    pub jd_maximum: f64,
    /// Eclipse classification.
    pub eclipse_type: LunarEclipseType,
    /// **Shadow-depth proxy, NOT the astronomical eclipse magnitude.**
    ///
    /// A crude `1 − |β| / 1.58°` measure of how close the Full Moon sits to the
    /// ecliptic (1.0 = Moon centred on the node-crossing, 0.0 = at the
    /// penumbral edge). It is monotonic with shadow depth but is NOT the
    /// umbral/penumbral magnitude (fraction of the lunar diameter immersed),
    /// which requires Earth-shadow cone radii at the Moon's distance — not yet
    /// implemented. Do not report this as "magnitude" to users.
    pub shadow_depth_proxy: f64,
    /// Moon's ecliptic latitude at maximum (degrees).
    pub moon_latitude_deg: f64,
}

/// Classification of a solar eclipse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolarEclipseType {
    /// Only part of the Sun is obscured.
    Partial,
    /// The Moon's apparent disk fits inside the Sun's -- a ring of sunlight
    /// remains visible.
    Annular,
    /// The Moon completely covers the Sun.
    Total,
    /// Total over part of the central path, annular over another part.
    Hybrid,
}

/// A single solar eclipse event.
#[derive(Debug, Clone)]
pub struct SolarEclipse {
    /// Julian Date (UT1) of greatest eclipse — the Besselian closest shadow-axis
    /// approach (can differ from the New Moon instant by up to ~30 min).
    pub jd_maximum: f64,
    /// Eclipse classification, from the Besselian shadow geometry.
    pub eclipse_type: SolarEclipseType,
    /// **Coverage proxy, NOT the astronomical (local) eclipse magnitude.**
    ///
    /// For central (Total/Annular/Hybrid) geometry this is the geocentric
    /// apparent Moon/Sun **diameter ratio** (>1 ⇒ total disc coverage, <1 ⇒
    /// annular ring); for Partial geometry it is a parallax-corrected
    /// disc-overlap heuristic seeded by the New-Moon separation. Neither is the
    /// rigorous local magnitude at the greatest-eclipse sub-point, which needs a
    /// per-observer Besselian reduction (not implemented). The authoritative
    /// geometric quantity here is [`Self::gamma`]; treat this field as an
    /// indicative coverage figure only.
    pub coverage_proxy: f64,
    /// Moon's ecliptic latitude at maximum (degrees).
    pub moon_latitude_deg: f64,
    /// γ: least distance of the shadow axis from Earth's centre at greatest
    /// eclipse, in Earth equatorial radii (Besselian). |γ| ≲ 0.9972 ⇒ central.
    pub gamma: f64,
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Find all lunar eclipses between `jd_start` and `jd_end` (UT1).
///
/// The search locates every Full Moon in the range, checks the Moon's ecliptic
/// latitude, and classifies any eclipse that falls within the penumbral limit.
pub fn find_lunar_eclipses(almanac: &Almanac, jd_start: f64, jd_end: f64) -> Vec<LunarEclipse> {
    if jd_start >= jd_end {
        return Vec::new();
    }

    let full_moons = find_full_moons(almanac, jd_start, jd_end);
    let mut eclipses = Vec::new();

    for jd_full in full_moons {
        // JD bounds guard: the syzygy scan steps a fixed `SCAN_STEP` and can
        // refine a Full Moon a fraction of a step past the requested window.
        // The reported maximum (syzygy) must never fall outside [jd_start,
        // jd_end], so skip any out-of-range lunation.
        if jd_full < jd_start || jd_full > jd_end {
            continue;
        }
        let moon_pos = match almanac.geocentric_ecliptic(Body::Moon, JdUT1(jd_full)) {
            Ok(p) => p,
            // Skip this lunation if the Moon's position cannot be computed.
            // This can happen for JD values at the extreme edges of the
            // ephemeris validity range. Since eclipse detection scans many
            // lunations, one failed position should not abort the entire
            // search — the caller still receives all successfully detected
            // eclipses.
            Err(_) => continue,
        };
        let lat_deg = moon_pos.latitude.to_degrees();

        if lat_deg.abs() < LUNAR_PENUMBRAL_LIMIT {
            let eclipse_type = if lat_deg.abs() < LUNAR_TOTAL_LIMIT {
                LunarEclipseType::Total
            } else if lat_deg.abs() < LUNAR_PARTIAL_LIMIT {
                LunarEclipseType::Partial
            } else {
                LunarEclipseType::Penumbral
            };

            // Shadow-depth PROXY (not the astronomical magnitude): how close the
            // Moon sits to the node-crossing. 1.0 = Moon's centre on the
            // ecliptic, 0.0 = at the penumbral-limit edge. See the field doc.
            let shadow_depth_proxy = 1.0 - (lat_deg.abs() / LUNAR_PENUMBRAL_LIMIT);

            eclipses.push(LunarEclipse {
                jd_maximum: jd_full,
                eclipse_type,
                shadow_depth_proxy,
                moon_latitude_deg: lat_deg,
            });
        }
    }

    eclipses
}

/// Find all solar eclipses between `jd_start` and `jd_end` (UT1).
///
/// The search locates every New Moon in the range, checks the Moon's ecliptic
/// latitude, and classifies any eclipse that falls within the solar limit.
pub fn find_solar_eclipses(almanac: &Almanac, jd_start: f64, jd_end: f64) -> Vec<SolarEclipse> {
    if jd_start >= jd_end {
        return Vec::new();
    }

    let new_moons = find_new_moons(almanac, jd_start, jd_end);
    let mut eclipses = Vec::new();

    for jd_new in new_moons {
        let moon_pos = match almanac.geocentric_ecliptic(Body::Moon, JdUT1(jd_new)) {
            Ok(p) => p,
            // Skip this lunation if the Moon's position cannot be computed.
            // Same rationale as in find_lunar_eclipses: one failed position
            // at the edge of the ephemeris range should not abort the entire
            // search. The caller still receives all successfully detected
            // eclipses from other lunations.
            Err(_) => continue,
        };
        let lat_deg = moon_pos.latitude.to_degrees();

        if lat_deg.abs() < SOLAR_ECLIPSE_LIMIT {
            // Apparent angular radii scaled by actual distance at this epoch.
            let moon_angular_radius =
                MOON_MEAN_ANGULAR_RADIUS_DEG * (MOON_MEAN_DISTANCE_AU / moon_pos.distance);

            let sun_pos = match almanac.geocentric_ecliptic(Body::Sun, JdUT1(jd_new)) {
                Ok(p) => p,
                // Skip if Sun position fails — same resilience rationale as
                // the Moon position check above.
                Err(_) => continue,
            };
            let sun_angular_radius =
                SUN_MEAN_ANGULAR_RADIUS_DEG * (SUN_MEAN_DISTANCE_AU / sun_pos.distance);

            let diameter_ratio = moon_angular_radius / sun_angular_radius;

            // The geocentric angular separation at maximum eclipse is
            // approximately the Moon's ecliptic latitude (since the longitude
            // difference is ~0 at New Moon by definition).
            let separation_deg = lat_deg.abs();

            // Classification via the rigorous Besselian shadow geometry
            // (`besselian::classify_solar_eclipse`): locate greatest eclipse,
            // then derive γ (least shadow-axis distance from Earth's centre) and
            // the global type. This replaces the former crude approximation that
            // compared |Moon ecliptic latitude| against degree-level threshold
            // cones. The cheap |latitude| pre-filter above only nominates
            // candidate lunations; the Besselian engine is the authority on
            // whether an eclipse actually occurs and what type it is.
            let (eclipse_type, gamma, jd_greatest_tt) =
                match classify_solar_eclipse(almanac, jd_new) {
                    Some(g) => {
                        let kind = match g.eclipse_type {
                            GlobalSolarType::Total => SolarEclipseType::Total,
                            GlobalSolarType::Annular => SolarEclipseType::Annular,
                            GlobalSolarType::Hybrid => SolarEclipseType::Hybrid,
                            GlobalSolarType::Partial => SolarEclipseType::Partial,
                        };
                        (kind, g.gamma, g.jd_greatest_tt.as_f64())
                    }
                    // The Besselian axis misses the Earth entirely — the |latitude|
                    // pre-filter produced a false positive at this New Moon; skip it.
                    None => continue,
                };

            // Maximum eclipse instant = the Besselian greatest-eclipse time
            // (closest shadow-axis approach), NOT the New Moon — the two differ
            // by up to ~30 min. Convert TT → UT1 via ΔT.
            let jd_maximum = jd_greatest_tt
                - delta_t(
                    jd_greatest_tt,
                    &DeltaTModel::StephensonMorrisonHohenkerk2016,
                ) / 86400.0;

            // Coverage PROXY (NOT a Besselian local magnitude): central
            // eclipses report the geocentric apparent diameter ratio; partials a
            // lunar-parallax overlap heuristic seeded by the New-Moon separation.
            // A rigorous magnitude needs the per-observer reduction at the
            // greatest-eclipse sub-point, which is not implemented.
            //
            // For partial eclipses, the geocentric separation may exceed the
            // sum of angular radii because the eclipse is only visible from
            // certain locations on Earth due to the Moon's parallax (~0.95
            // degrees). We compute the parallax-corrected separation for the
            // optimal observer location.
            let coverage_proxy = match eclipse_type {
                SolarEclipseType::Total | SolarEclipseType::Annular | SolarEclipseType::Hybrid => {
                    diameter_ratio
                }
                SolarEclipseType::Partial => {
                    // Lunar equatorial horizontal parallax: the angular size
                    // of Earth's radius as seen from the Moon. This tells us
                    // how much the Moon's apparent position can shift for a
                    // surface observer vs the geocenter.
                    //   parallax = asin(R_earth / d_moon)
                    // R_earth ~ 4.2635e-5 AU (6371 km / 1.496e8 km)
                    const EARTH_RADIUS_AU: f64 = 4.2635e-5;
                    let lunar_parallax_deg =
                        (EARTH_RADIUS_AU / moon_pos.distance).asin().to_degrees();

                    // The minimum separation achievable from an optimal
                    // surface location is reduced by up to the parallax.
                    let effective_sep = (separation_deg - lunar_parallax_deg).max(0.0);

                    let overlap_deg = moon_angular_radius + sun_angular_radius - effective_sep;
                    if overlap_deg <= 0.0 {
                        // No overlap even with parallax correction -- marginal
                        // eclipse at the detection boundary.
                        0.001
                    } else {
                        (overlap_deg / (2.0 * sun_angular_radius)).clamp(0.001, 1.0)
                    }
                }
            };

            eclipses.push(SolarEclipse {
                jd_maximum,
                eclipse_type,
                coverage_proxy,
                moon_latitude_deg: lat_deg,
                gamma,
            });
        }
    }

    eclipses
}

// ── Internal helpers ─────────────────────────────────────────────────────────

/// Compute the Sun-Moon elongation in degrees, wrapped to [-180, +180).
///
/// At New Moon the elongation is ~0; at Full Moon it is ~+/-180. Using a
/// signed wrapping lets us distinguish the two syzygy types.
fn elongation_deg(almanac: &Almanac, jd: f64) -> f64 {
    let jd_ut1 = JdUT1(jd);
    let sun_lon = almanac
        .geocentric_longitude_deg(Body::Sun, jd_ut1)
        .unwrap_or(0.0);
    let moon_lon = almanac
        .geocentric_longitude_deg(Body::Moon, jd_ut1)
        .unwrap_or(0.0);
    let mut diff = moon_lon - sun_lon;
    // Wrap to (-180, +180].
    while diff > 180.0 {
        diff -= 360.0;
    }
    while diff <= -180.0 {
        diff += 360.0;
    }
    diff
}

/// Find all New Moons (Sun-Moon conjunction, elongation = 0) in the range.
///
/// The elongation (Moon lon - Sun lon, wrapped to [-180, +180]) smoothly
/// increases from ~0 toward +180 after New Moon (the Moon moves faster than
/// the Sun). At Full Moon it wraps from +180 to -180, then climbs back to 0
/// at the next New Moon. So at a true New Moon the elongation crosses zero
/// from *below* (negative to positive). At Full Moon the discontinuity may
/// also produce a false zero-crossing that goes from positive to negative.
///
/// We filter by checking the derivative sign: a true New Moon has the
/// elongation increasing (positive slope) through zero.
///
/// After each detection, the scan jumps forward by `MIN_SYNODIC_GAP` (~25
/// days) to prevent the same lunation from being counted twice.
fn find_new_moons(almanac: &Almanac, jd_start: f64, jd_end: f64) -> Vec<f64> {
    let mut results = Vec::new();
    let mut jd = jd_start;
    let mut prev = elongation_deg(almanac, jd);

    while jd < jd_end {
        jd += SCAN_STEP;
        let curr = elongation_deg(almanac, jd);

        // Skip discontinuities (Full Moon wraps). A jump > 180 degrees in a
        // single step is physically impossible; it's a wrap artefact.
        let delta = curr - prev;
        if delta.abs() > 180.0 {
            prev = curr;
            continue;
        }

        // Detect a smooth zero-crossing where elongation goes from negative
        // to positive (or hits zero exactly). This is the New Moon condition.
        if prev < 0.0
            && curr >= 0.0
            && let Some(nm_jd) = bisect_new_moon(almanac, jd - SCAN_STEP, jd)
        {
            results.push(nm_jd);
            // Skip forward to avoid double-counting this lunation.
            jd = nm_jd + MIN_SYNODIC_GAP;
            prev = elongation_deg(almanac, jd);
            continue;
        }

        prev = curr;
    }

    results
}

/// Bisect an interval known to contain a New Moon (elongation crosses 0
/// from negative to positive).
fn bisect_new_moon(almanac: &Almanac, mut lo: f64, mut hi: f64) -> Option<f64> {
    for _ in 0..60 {
        let mid = (lo + hi) / 2.0;
        if (hi - lo) < 1e-8 {
            return Some(mid);
        }
        let e_mid = elongation_deg(almanac, mid);
        if e_mid < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some((lo + hi) / 2.0)
}

/// Find all Full Moons (Sun-Moon opposition, elongation wraps through +/-180).
///
/// Because the elongation wraps at +/-180, Full Moons appear as
/// discontinuities rather than smooth zero-crossings. We detect them by
/// scanning for jumps where the elongation goes from near +180 to near -180.
///
/// After each detection, the scan jumps forward by `MIN_SYNODIC_GAP` (~25
/// days) to prevent the same lunation from being counted twice.
fn find_full_moons(almanac: &Almanac, jd_start: f64, jd_end: f64) -> Vec<f64> {
    let mut results = Vec::new();
    let mut jd = jd_start;
    let mut prev = elongation_deg(almanac, jd);

    while jd < jd_end {
        jd += SCAN_STEP;
        let curr = elongation_deg(almanac, jd);

        // A Full Moon causes a wrap: prev near +180 and curr near -180.
        // The Moon's elongation increases monotonically between syzygies, so
        // the wrap always goes positive-to-negative.
        if prev > 90.0
            && curr < -90.0
            && let Some(fm_jd) = bisect_full_moon(almanac, jd - SCAN_STEP, jd)
        {
            results.push(fm_jd);
            // Skip forward to avoid double-counting this lunation.
            jd = fm_jd + MIN_SYNODIC_GAP;
            prev = elongation_deg(almanac, jd);
            continue;
        }

        prev = curr;
    }

    results
}

/// Bisect an interval known to contain a Full Moon (elongation wrap from
/// positive to negative through the +/-180 discontinuity).
fn bisect_full_moon(almanac: &Almanac, mut lo: f64, mut hi: f64) -> Option<f64> {
    // At the Full Moon, the elongation wraps from +180 to -180. We bisect
    // by finding the boundary between "elongation > 0" and "elongation < 0".
    for _ in 0..60 {
        let mid = (lo + hi) / 2.0;
        if (hi - lo) < 1e-8 {
            return Some(mid);
        }
        let e_mid = elongation_deg(almanac, mid);

        // The elongation is positive before the wrap, negative after.
        if e_mid > 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some((lo + hi) / 2.0)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: Julian Date for a calendar date (Meeus Ch.7).
    fn jd_from_date(year: i32, month: u32, day: f64) -> f64 {
        let (y, m) = if month <= 2 {
            (year as f64 - 1.0, month as f64 + 12.0)
        } else {
            (year as f64, month as f64)
        };
        let a = (y / 100.0).floor();
        let b = 2.0 - a + (a / 4.0).floor();
        (365.25 * (y + 4716.0)).floor() + (30.6001 * (m + 1.0)).floor() + day + b - 1524.5
    }

    fn almanac() -> Almanac {
        Almanac::default_vedic()
    }

    // ── Lunar eclipse tests ──────────────────────────────────────────────

    #[test]
    fn find_lunar_eclipses_year_2024() {
        // 2024 had 2 lunar eclipses:
        //   - Mar 25, 2024 -- penumbral
        //   - Sep 18, 2024 -- partial
        let a = almanac();
        let jd_start = jd_from_date(2024, 1, 1.0);
        let jd_end = jd_from_date(2025, 1, 1.0);
        let eclipses = find_lunar_eclipses(&a, jd_start, jd_end);

        assert!(
            eclipses.len() >= 2,
            "Expected at least 2 lunar eclipses in 2024, found {}",
            eclipses.len()
        );

        // The eclipses should be in the first and second halves of the year.
        let mid_year = jd_from_date(2024, 7, 1.0);
        let first_half: Vec<_> = eclipses
            .iter()
            .filter(|e| e.jd_maximum < mid_year)
            .collect();
        let second_half: Vec<_> = eclipses
            .iter()
            .filter(|e| e.jd_maximum >= mid_year)
            .collect();
        assert!(
            !first_half.is_empty(),
            "Should find a lunar eclipse in H1 2024"
        );
        assert!(
            !second_half.is_empty(),
            "Should find a lunar eclipse in H2 2024"
        );
    }

    #[test]
    fn lunar_eclipse_shadow_depth_proxy_range() {
        let a = almanac();
        let jd_start = jd_from_date(2020, 1, 1.0);
        let jd_end = jd_from_date(2030, 1, 1.0);
        let eclipses = find_lunar_eclipses(&a, jd_start, jd_end);

        assert!(
            !eclipses.is_empty(),
            "Should find lunar eclipses in 2020-2030"
        );

        for e in &eclipses {
            assert!(
                e.shadow_depth_proxy > 0.0 && e.shadow_depth_proxy <= 1.0,
                "Shadow-depth proxy out of range: {} at JD {}",
                e.shadow_depth_proxy,
                e.jd_maximum
            );
            assert!(
                e.moon_latitude_deg.abs() < LUNAR_PENUMBRAL_LIMIT,
                "Latitude should be within penumbral limit: {}",
                e.moon_latitude_deg
            );
        }
    }

    #[test]
    fn lunar_eclipse_type_coverage() {
        // Over a decade, we should see all three types.
        let a = almanac();
        let jd_start = jd_from_date(2020, 1, 1.0);
        let jd_end = jd_from_date(2030, 1, 1.0);
        let eclipses = find_lunar_eclipses(&a, jd_start, jd_end);

        let has_penumbral = eclipses
            .iter()
            .any(|e| e.eclipse_type == LunarEclipseType::Penumbral);
        let has_partial = eclipses
            .iter()
            .any(|e| e.eclipse_type == LunarEclipseType::Partial);
        let has_total = eclipses
            .iter()
            .any(|e| e.eclipse_type == LunarEclipseType::Total);

        assert!(
            has_penumbral,
            "Should find at least one penumbral lunar eclipse in 2020-2030"
        );
        assert!(
            has_partial,
            "Should find at least one partial lunar eclipse in 2020-2030"
        );
        assert!(
            has_total,
            "Should find at least one total lunar eclipse in 2020-2030"
        );
    }

    #[test]
    fn lunar_eclipses_sorted_chronologically() {
        let a = almanac();
        let jd_start = jd_from_date(2020, 1, 1.0);
        let jd_end = jd_from_date(2030, 1, 1.0);
        let eclipses = find_lunar_eclipses(&a, jd_start, jd_end);

        for pair in eclipses.windows(2) {
            assert!(
                pair[0].jd_maximum < pair[1].jd_maximum,
                "Eclipses should be chronologically sorted"
            );
        }
    }

    #[test]
    fn lunar_eclipse_empty_for_reversed_range() {
        let a = almanac();
        let eclipses = find_lunar_eclipses(&a, 2460000.0, 2450000.0);
        assert!(eclipses.is_empty(), "Reversed range should return empty");
    }

    // ── Solar eclipse tests ──────────────────────────────────────────────

    #[test]
    fn find_solar_eclipses_year_2024() {
        // 2024 had 2 solar eclipses:
        //   - Apr 8, 2024  -- total
        //   - Oct 2, 2024  -- annular
        let a = almanac();
        let jd_start = jd_from_date(2024, 1, 1.0);
        let jd_end = jd_from_date(2025, 1, 1.0);
        let eclipses = find_solar_eclipses(&a, jd_start, jd_end);

        assert!(
            eclipses.len() >= 2,
            "Expected at least 2 solar eclipses in 2024, found {}",
            eclipses.len()
        );

        // One in the first half, one in the second.
        let mid_year = jd_from_date(2024, 7, 1.0);
        let first_half: Vec<_> = eclipses
            .iter()
            .filter(|e| e.jd_maximum < mid_year)
            .collect();
        let second_half: Vec<_> = eclipses
            .iter()
            .filter(|e| e.jd_maximum >= mid_year)
            .collect();
        assert!(
            !first_half.is_empty(),
            "Should find a solar eclipse in H1 2024"
        );
        assert!(
            !second_half.is_empty(),
            "Should find a solar eclipse in H2 2024"
        );
    }

    #[test]
    fn solar_eclipse_type_coverage() {
        // Over a decade we should see partial, total, and annular.
        let a = almanac();
        let jd_start = jd_from_date(2020, 1, 1.0);
        let jd_end = jd_from_date(2030, 1, 1.0);
        let eclipses = find_solar_eclipses(&a, jd_start, jd_end);

        assert!(
            !eclipses.is_empty(),
            "Should find solar eclipses in 2020-2030"
        );

        let has_partial = eclipses
            .iter()
            .any(|e| e.eclipse_type == SolarEclipseType::Partial);
        let has_central = eclipses.iter().any(|e| {
            matches!(
                e.eclipse_type,
                SolarEclipseType::Total | SolarEclipseType::Annular
            )
        });

        assert!(
            has_partial,
            "Should find at least one partial solar eclipse in 2020-2030"
        );
        assert!(
            has_central,
            "Should find at least one total or annular solar eclipse in 2020-2030"
        );
    }

    #[test]
    fn solar_eclipse_coverage_proxy_positive() {
        let a = almanac();
        let jd_start = jd_from_date(2020, 1, 1.0);
        let jd_end = jd_from_date(2030, 1, 1.0);
        let eclipses = find_solar_eclipses(&a, jd_start, jd_end);

        for e in &eclipses {
            assert!(
                e.coverage_proxy > 0.0,
                "Solar eclipse coverage proxy should be positive: {} at JD {} ({:?})",
                e.coverage_proxy,
                e.jd_maximum,
                e.eclipse_type
            );
        }
    }

    #[test]
    fn solar_eclipses_sorted_chronologically() {
        let a = almanac();
        let jd_start = jd_from_date(2020, 1, 1.0);
        let jd_end = jd_from_date(2030, 1, 1.0);
        let eclipses = find_solar_eclipses(&a, jd_start, jd_end);

        for pair in eclipses.windows(2) {
            assert!(
                pair[0].jd_maximum < pair[1].jd_maximum,
                "Eclipses should be chronologically sorted"
            );
        }
    }

    #[test]
    fn solar_eclipse_empty_for_reversed_range() {
        let a = almanac();
        let eclipses = find_solar_eclipses(&a, 2460000.0, 2450000.0);
        assert!(eclipses.is_empty(), "Reversed range should return empty");
    }

    // ── Cross-check: eclipse frequency sanity ────────────────────────────

    #[test]
    fn eclipse_frequency_per_decade() {
        // On average, ~2-3 eclipses of each type per year.
        // Over a decade: ~15-40 lunar, ~15-40 solar.
        let a = almanac();
        let jd_start = jd_from_date(2020, 1, 1.0);
        let jd_end = jd_from_date(2030, 1, 1.0);

        let lunar = find_lunar_eclipses(&a, jd_start, jd_end);
        let solar = find_solar_eclipses(&a, jd_start, jd_end);

        assert!(
            lunar.len() >= 10 && lunar.len() <= 50,
            "Expected 10-50 lunar eclipses in a decade, found {}",
            lunar.len()
        );
        assert!(
            solar.len() >= 10 && solar.len() <= 50,
            "Expected 10-50 solar eclipses in a decade, found {}",
            solar.len()
        );
    }

    // ── NASA catalog verification (2024-2025) ───────────────────────────
    //
    // Reference dates from NASA Eclipse Catalog:
    //   https://eclipse.gsfc.nasa.gov/
    //
    // 2024-Mar-25: Penumbral lunar eclipse
    // 2024-Apr-08: Total solar eclipse
    // 2024-Sep-18: Partial lunar eclipse
    // 2024-Oct-02: Annular solar eclipse
    // 2025-Mar-14: Total lunar eclipse
    // 2025-Mar-29: Partial solar eclipse

    #[test]
    fn nasa_2024_mar25_penumbral_lunar() {
        let a = almanac();
        let jd_target = jd_from_date(2024, 3, 25.0);
        let eclipses = find_lunar_eclipses(&a, jd_target - 2.0, jd_target + 2.0);

        assert!(
            !eclipses.is_empty(),
            "Should detect the 2024-Mar-25 lunar eclipse"
        );
        let e = &eclipses[0];
        assert!(
            (e.jd_maximum - jd_target).abs() < 1.0,
            "Eclipse should be within 1 day of 2024-Mar-25, got offset {:.2} days",
            e.jd_maximum - jd_target
        );
        assert_eq!(
            e.eclipse_type,
            LunarEclipseType::Penumbral,
            "NASA classifies 2024-Mar-25 as penumbral lunar, got {:?}",
            e.eclipse_type
        );
    }

    #[test]
    fn nasa_2024_apr08_total_solar() {
        let a = almanac();
        let jd_target = jd_from_date(2024, 4, 8.0);
        let eclipses = find_solar_eclipses(&a, jd_target - 2.0, jd_target + 2.0);

        assert!(
            !eclipses.is_empty(),
            "Should detect the 2024-Apr-08 solar eclipse"
        );
        let e = &eclipses[0];
        assert!(
            (e.jd_maximum - jd_target).abs() < 1.0,
            "Eclipse should be within 1 day of 2024-Apr-08, got offset {:.2} days",
            e.jd_maximum - jd_target
        );
        assert_eq!(
            e.eclipse_type,
            SolarEclipseType::Total,
            "NASA classifies 2024-Apr-08 as total solar, got {:?}",
            e.eclipse_type
        );
        assert!(
            e.coverage_proxy > 1.0,
            "Total solar eclipse coverage proxy (diameter ratio) should be >1.0, got {}",
            e.coverage_proxy
        );
    }

    #[test]
    fn nasa_2024_sep18_partial_lunar() {
        let a = almanac();
        let jd_target = jd_from_date(2024, 9, 18.0);
        let eclipses = find_lunar_eclipses(&a, jd_target - 2.0, jd_target + 2.0);

        assert!(
            !eclipses.is_empty(),
            "Should detect the 2024-Sep-18 lunar eclipse"
        );
        let e = &eclipses[0];
        assert!(
            (e.jd_maximum - jd_target).abs() < 1.0,
            "Eclipse should be within 1 day of 2024-Sep-18, got offset {:.2} days",
            e.jd_maximum - jd_target
        );
        // NASA classifies this as partial. Our latitude-based model may classify
        // it as penumbral or partial depending on the threshold, but it should
        // NOT be total.
        assert_ne!(
            e.eclipse_type,
            LunarEclipseType::Total,
            "2024-Sep-18 should NOT be classified as total lunar"
        );
    }

    #[test]
    fn nasa_2024_oct02_annular_solar() {
        let a = almanac();
        let jd_target = jd_from_date(2024, 10, 2.0);
        let eclipses = find_solar_eclipses(&a, jd_target - 2.0, jd_target + 2.0);

        assert!(
            !eclipses.is_empty(),
            "Should detect the 2024-Oct-02 solar eclipse"
        );
        let e = &eclipses[0];
        assert!(
            (e.jd_maximum - jd_target).abs() < 1.0,
            "Eclipse should be within 1 day of 2024-Oct-02, got offset {:.2} days",
            e.jd_maximum - jd_target
        );
        assert_eq!(
            e.eclipse_type,
            SolarEclipseType::Annular,
            "NASA classifies 2024-Oct-02 as annular solar, got {:?}",
            e.eclipse_type
        );
        assert!(
            e.coverage_proxy > 0.0 && e.coverage_proxy < 1.0,
            "Annular solar eclipse coverage proxy (diameter ratio) should be in (0, 1), got {}",
            e.coverage_proxy
        );
    }

    #[test]
    fn nasa_2025_mar14_total_lunar() {
        let a = almanac();
        let jd_target = jd_from_date(2025, 3, 14.0);
        let eclipses = find_lunar_eclipses(&a, jd_target - 2.0, jd_target + 2.0);

        assert!(
            !eclipses.is_empty(),
            "Should detect the 2025-Mar-14 lunar eclipse"
        );
        let e = &eclipses[0];
        assert!(
            (e.jd_maximum - jd_target).abs() < 1.0,
            "Eclipse should be within 1 day of 2025-Mar-14, got offset {:.2} days",
            e.jd_maximum - jd_target
        );
        assert_eq!(
            e.eclipse_type,
            LunarEclipseType::Total,
            "NASA classifies 2025-Mar-14 as total lunar, got {:?}",
            e.eclipse_type
        );
    }

    #[test]
    fn nasa_2025_mar29_partial_solar() {
        let a = almanac();
        let jd_target = jd_from_date(2025, 3, 29.0);
        let eclipses = find_solar_eclipses(&a, jd_target - 2.0, jd_target + 2.0);

        assert!(
            !eclipses.is_empty(),
            "Should detect the 2025-Mar-29 solar eclipse"
        );
        let e = &eclipses[0];
        assert!(
            (e.jd_maximum - jd_target).abs() < 1.0,
            "Eclipse should be within 1 day of 2025-Mar-29, got offset {:.2} days",
            e.jd_maximum - jd_target
        );
        assert_eq!(
            e.eclipse_type,
            SolarEclipseType::Partial,
            "NASA classifies 2025-Mar-29 as partial solar, got {:?}",
            e.eclipse_type
        );
        assert!(
            e.coverage_proxy > 0.01,
            "Partial solar eclipse should have a meaningful coverage proxy, got {}",
            e.coverage_proxy
        );
    }

    // ── All 6 NASA eclipses: date proximity ─────────────────────────────

    #[test]
    fn nasa_all_six_eclipses_found_within_one_day() {
        let a = almanac();
        let jd_2024_start = jd_from_date(2024, 1, 1.0);
        let jd_2026_start = jd_from_date(2026, 1, 1.0);

        let all_lunar = find_lunar_eclipses(&a, jd_2024_start, jd_2026_start);
        let all_solar = find_solar_eclipses(&a, jd_2024_start, jd_2026_start);

        let nasa_dates = [
            (
                "2024-Mar-25 Penumbral Lunar",
                jd_from_date(2024, 3, 25.0),
                true,
            ),
            ("2024-Apr-08 Total Solar", jd_from_date(2024, 4, 8.0), false),
            (
                "2024-Sep-18 Partial Lunar",
                jd_from_date(2024, 9, 18.0),
                true,
            ),
            (
                "2024-Oct-02 Annular Solar",
                jd_from_date(2024, 10, 2.0),
                false,
            ),
            ("2025-Mar-14 Total Lunar", jd_from_date(2025, 3, 14.0), true),
            (
                "2025-Mar-29 Partial Solar",
                jd_from_date(2025, 3, 29.0),
                false,
            ),
        ];

        for (label, jd_nasa, is_lunar) in &nasa_dates {
            let found = if *is_lunar {
                all_lunar
                    .iter()
                    .any(|e| (e.jd_maximum - jd_nasa).abs() < 1.0)
            } else {
                all_solar
                    .iter()
                    .any(|e| (e.jd_maximum - jd_nasa).abs() < 1.0)
            };
            assert!(
                found,
                "NASA eclipse '{}' not found within 1 day tolerance",
                label
            );
        }
    }

    // ── Internal helper tests ────────────────────────────────────────────

    #[test]
    fn new_moons_in_one_year() {
        let a = almanac();
        let jd_start = jd_from_date(2024, 1, 1.0);
        let jd_end = jd_from_date(2025, 1, 1.0);
        let nms = find_new_moons(&a, jd_start, jd_end);

        // A year has 12 or 13 New Moons. The synodic-gap guard prevents
        // double-counting.
        assert!(
            nms.len() >= 12 && nms.len() <= 13,
            "Expected 12-13 New Moons in 2024, found {}",
            nms.len()
        );

        // They should be ~29.5 days apart.
        for pair in nms.windows(2) {
            let gap = pair[1] - pair[0];
            assert!(
                gap > 28.0 && gap < 31.0,
                "New Moons should be ~29.5 days apart, got {gap}"
            );
        }
    }

    #[test]
    fn full_moons_in_one_year() {
        let a = almanac();
        let jd_start = jd_from_date(2024, 1, 1.0);
        let jd_end = jd_from_date(2025, 1, 1.0);
        let fms = find_full_moons(&a, jd_start, jd_end);

        assert!(
            fms.len() >= 12 && fms.len() <= 13,
            "Expected 12-13 Full Moons in 2024, found {}",
            fms.len()
        );

        for pair in fms.windows(2) {
            let gap = pair[1] - pair[0];
            assert!(
                gap > 28.0 && gap < 31.0,
                "Full Moons should be ~29.5 days apart, got {gap}"
            );
        }
    }
}
