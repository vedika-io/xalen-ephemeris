//! Besselian-elements solar-eclipse geometry (global classification).
//!
//! This is the rigorous Bessel/Chauvenet method (Explanatory Supplement to the
//! Astronomical Almanac §11; Meeus, *Astronomical Algorithms* 2nd ed. Ch. 54),
//! replacing the crude `|Moon ecliptic latitude|`-threshold classifier for the
//! GLOBAL properties of a solar eclipse: the minimum shadow-axis distance from
//! Earth's centre (γ), the global type (total / annular / hybrid / partial), and
//! the instant of greatest eclipse.
//!
//! # What this computes (and what it does not)
//!
//! The Besselian elements `(x, y, d, l1, l2, f1, f2)` are pure GEOCENTRIC shadow
//! geometry — they need no observer. From them we derive γ, the global type, and
//! the greatest-eclipse time, and we validate those against NASA's published
//! values for 2017-08-21 and 2024-04-08 (see tests). The Besselian element `μ`
//! (Greenwich hour angle of the axis) and the observer reduction `(ξ, η, ζ)` —
//! which give per-location magnitude and C1–C4 contact times — are a separate
//! layer not implemented here.
//!
//! # Inputs
//!
//! Apparent geocentric equatorial coordinates (of date) and distances of BOTH
//! the Sun and the Moon, taken from the [`Almanac`]. The element computation is
//! a direct evaluation at each instant (no polynomial fit); greatest eclipse is
//! found by minimising √(x²+y²) over the conjunction window.

use crate::{Almanac, Body};
use xalen_coords::{ecliptic_to_equatorial, mean_obliquity, nutation_2000b};
use xalen_time::{JdTT, JdUT1, JulianDay};

/// 1 astronomical unit in Earth equatorial radii (AU / a_E, a_E = 6378.137 km).
const AU_IN_EARTH_RADII: f64 = 149_597_870.7 / 6378.137;

/// Solar radius in Earth equatorial radii (IAU nominal 6.957e8 m / 6.378137e6 m).
const SUN_RADIUS_EARTH_RADII: f64 = 6.957e8 / 6.378_137e6;

/// Moon/Earth radius ratio for the PENUMBRAL cone (mean lunar radius `k1`).
const K_PENUMBRA: f64 = 0.2725076;

/// Moon/Earth radius ratio for the UMBRAL cone (`k2`, with the crater
/// correction NASA adopts for umbral/antumbral contacts).
const K_UMBRA: f64 = 0.272281;

/// The Besselian elements of a solar eclipse at one instant, in the fundamental
/// plane (Earth equatorial radii). The shadow axis pierces the fundamental plane
/// at `(x, y)`; `d` is the axis declination; `l1`/`l2` are the penumbral/umbral
/// shadow radii in that plane (`l2 < 0` ⇒ total, `l2 > 0` ⇒ annular).
#[derive(Debug, Clone, Copy)]
pub struct BesselianElements {
    /// Shadow-axis x in the fundamental plane (Earth radii, towards equator/east).
    pub x: f64,
    /// Shadow-axis y in the fundamental plane (Earth radii, towards north).
    pub y: f64,
    /// Declination of the shadow axis (radians).
    pub d: f64,
    /// Penumbral cone half-angle tangent.
    pub tan_f1: f64,
    /// Umbral/antumbral cone half-angle tangent.
    pub tan_f2: f64,
    /// Penumbral shadow radius in the fundamental plane (Earth radii).
    pub l1: f64,
    /// Umbral shadow radius in the fundamental plane (Earth radii); negative for
    /// a total eclipse, positive for annular.
    pub l2: f64,
}

/// Compute the Besselian elements of a solar eclipse at the given TT instant
/// from the Almanac's apparent geocentric Sun and Moon positions. Returns `None`
/// if either body's position is unavailable at this epoch.
pub fn besselian_elements(almanac: &Almanac, jd_tt: JdTT) -> Option<BesselianElements> {
    let t = jd_tt.julian_centuries_from_j2000();
    let eps = mean_obliquity(t) + nutation_2000b(t).delta_epsilon; // true obliquity of date

    let sun_ecl = almanac.geocentric_ecliptic_tt(Body::Sun, jd_tt).ok()?;
    let moon_ecl = almanac.geocentric_ecliptic_tt(Body::Moon, jd_tt).ok()?;
    let sun = ecliptic_to_equatorial(&sun_ecl, eps);
    let moon = ecliptic_to_equatorial(&moon_ecl, eps);

    // Geocentric rectangular equatorial coordinates, Earth radii.
    let r_sun = sun_ecl.distance * AU_IN_EARTH_RADII;
    let r_moon = moon_ecl.distance * AU_IN_EARTH_RADII;
    let (sa_s, ca_s) = sun.right_ascension.sin_cos();
    let (sd_s, cd_s) = sun.declination.sin_cos();
    let (sa_m, ca_m) = moon.right_ascension.sin_cos();
    let (sd_m, cd_m) = moon.declination.sin_cos();
    let (xs, ys, zs) = (r_sun * cd_s * ca_s, r_sun * cd_s * sa_s, r_sun * sd_s);
    let (xm, ym, zm) = (r_moon * cd_m * ca_m, r_moon * cd_m * sa_m, r_moon * sd_m);

    // Shadow axis = Sun − Moon direction; its RA `a` and declination `d`.
    let (gx, gy, gz) = (xs - xm, ys - ym, zs - zm);
    let g = (gx * gx + gy * gy + gz * gz).sqrt();
    let a = gy.atan2(gx);
    let d = (gz / g).asin();

    // Moon projected onto the fundamental plane (the axis is along the plane's
    // z, so (x, y) is also where the axis pierces the plane).
    let (sd, cd) = d.sin_cos();
    let ha = moon.right_ascension - a; // (α − a)
    let (sha, cha) = ha.sin_cos();
    let x = r_moon * cd_m * sha;
    let y = r_moon * (sd_m * cd - cd_m * sd * cha);
    let z = r_moon * (sd_m * sd + cd_m * cd * cha);

    // Cone half-angles and shadow radii.
    let sin_f1 = (SUN_RADIUS_EARTH_RADII + K_PENUMBRA) / g;
    let sin_f2 = (SUN_RADIUS_EARTH_RADII - K_UMBRA) / g;
    let tan_f1 = sin_f1 / (1.0 - sin_f1 * sin_f1).sqrt();
    let tan_f2 = sin_f2 / (1.0 - sin_f2 * sin_f2).sqrt();
    let c1 = z + K_PENUMBRA / sin_f1;
    let c2 = z - K_UMBRA / sin_f2;
    let l1 = c1 * tan_f1;
    let l2 = c2 * tan_f2;

    Some(BesselianElements {
        x,
        y,
        d,
        tan_f1,
        tan_f2,
        l1,
        l2,
    })
}

/// Global type of a solar eclipse (the eclipse as a whole, not per-observer).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalSolarType {
    /// Umbral axis reaches Earth and the umbra is total along the path.
    Total,
    /// Antumbral axis reaches Earth (Moon too far for total) — ring eclipse.
    Annular,
    /// Total over part of the path, annular over another part.
    Hybrid,
    /// Only the penumbra (or only a grazing umbra that misses the surface):
    /// no point sees a central eclipse.
    Partial,
}

/// Global circumstances of a solar eclipse: greatest eclipse instant, γ, type.
#[derive(Debug, Clone, Copy)]
pub struct GlobalEclipse {
    /// Instant of greatest eclipse (TT) — when the axis is closest to centre.
    pub jd_greatest_tt: JdTT,
    /// γ: signed least distance of the shadow axis from Earth's centre, in Earth
    /// equatorial radii (sign = which side of centre the axis passes).
    pub gamma: f64,
    /// Global eclipse type.
    pub eclipse_type: GlobalSolarType,
    /// Umbral shadow radius `l2` at greatest eclipse (Earth radii); `< 0` total.
    pub l2_at_greatest: f64,
}

/// Classify a solar eclipse near a given New-Moon instant (UT1) by the Besselian
/// method: locate greatest eclipse (min axis distance), then classify by the
/// umbral-radius sign and whether the axis reaches the Earth ellipsoid.
///
/// Returns `None` if no eclipse occurs near this conjunction (axis misses Earth
/// by more than the penumbral limit) or positions are unavailable.
pub fn classify_solar_eclipse(almanac: &Almanac, jd_new_ut: f64) -> Option<GlobalEclipse> {
    // Convert the UT1 conjunction instant to TT, then refine greatest eclipse by
    // minimising the axis distance √(x²+y²) over a ±0.12 d (~3 h) window.
    let jd_tt0 = JdUT1(jd_new_ut)
        .to_tt(&xalen_time::DeltaTModel::StephensonMorrisonHohenkerk2016)
        .as_f64();
    let axis_dist = |jd: f64| -> Option<f64> {
        let b = besselian_elements(almanac, JdTT(jd))?;
        Some((b.x * b.x + b.y * b.y).sqrt())
    };

    // Coarse scan (1-min steps) for the minimum, then golden-section refine.
    let (mut best_jd, mut best) = (jd_tt0, f64::INFINITY);
    let step = 1.0 / 1440.0; // 1 minute
    let mut jd = jd_tt0 - 0.125;
    while jd <= jd_tt0 + 0.125 {
        if let Some(dist) = axis_dist(jd)
            && dist < best
        {
            best = dist;
            best_jd = jd;
        }
        jd += step;
    }
    let (mut lo, mut hi) = (best_jd - step, best_jd + step);
    const PHI: f64 = 0.618_033_988_749_895;
    for _ in 0..60 {
        let c = hi - (hi - lo) * PHI;
        let dd = lo + (hi - lo) * PHI;
        let fc = axis_dist(c)?;
        let fd = axis_dist(dd)?;
        if fc < fd {
            hi = dd;
        } else {
            lo = c;
        }
    }
    let jd_greatest = 0.5 * (lo + hi);
    let b = besselian_elements(almanac, JdTT(jd_greatest))?;
    // γ is SIGNED: positive when the shadow axis passes north of Earth's centre
    // (the sign of the fundamental-plane y at greatest eclipse). The coarse scan
    // above minimised the unsigned axis distance; limit tests below use |γ|.
    let gamma = b.x.hypot(b.y).copysign(b.y);
    let gamma_abs = gamma.abs();

    // No eclipse if even the penumbra misses the Earth ellipsoid. The penumbral
    // grazing limit is EVENT-SPECIFIC — |γ| ≲ 1 + l1 (l1 = penumbral radius in
    // the fundamental plane at greatest eclipse) — not a fixed constant.
    if gamma_abs > 1.0 + b.l1 {
        return None;
    }

    // Central if the shadow axis reaches the Earth ellipsoid (|γ| < 0.9972, the
    // standard geocentric central limit). Total / annular / hybrid is then judged
    // by the umbral radius at the greatest-eclipse SUB-POINT on the surface,
    //   L2_sub = l2 − ζ·tan f2,   ζ = √(1 − γ²)  (axis–ellipsoid intersection z),
    // NOT by the geocentric l2 sign:
    //   L2_sub < 0 and l2 < 0  → Total  (umbra over the whole central path)
    //   L2_sub < 0 and l2 > 0  → Hybrid (total at the sub-point, annular toward
    //                            the path ends where ζ → 0 and the antumbra shows)
    //   L2_sub ≥ 0             → Annular
    //
    // LIMITATION: rare NON-central total/annular eclipses (axis just misses the
    // ellipsoid yet the umbra grazes the limb, 0.9972 < |γ| ≲ 1) are reported as
    // Partial here — resolving those needs the full observer reduction (the per-
    // location ξ/η/ζ layer), which is not implemented.
    const CENTRAL_LIMIT: f64 = 0.9972;
    let eclipse_type = if gamma_abs < CENTRAL_LIMIT {
        let zeta = (1.0 - gamma_abs * gamma_abs).max(0.0).sqrt();
        let l2_sub = b.l2 - zeta * b.tan_f2;
        if l2_sub < 0.0 {
            if b.l2 > 0.0 {
                GlobalSolarType::Hybrid
            } else {
                GlobalSolarType::Total
            }
        } else {
            GlobalSolarType::Annular
        }
    } else {
        GlobalSolarType::Partial
    };

    Some(GlobalEclipse {
        jd_greatest_tt: JdTT(jd_greatest),
        gamma,
        eclipse_type,
        l2_at_greatest: b.l2,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn almanac() -> Almanac {
        Almanac::default_vedic()
    }

    /// Julian Date (TT) for a Gregorian calendar instant.
    fn jd_tt(year: i32, month: u32, day: u32, hour: f64) -> f64 {
        let (y, m) = if month <= 2 {
            (year - 1, month + 12)
        } else {
            (year, month)
        };
        let a = (y as f64 / 100.0).floor();
        let b = 2.0 - a + (a / 4.0).floor();
        (365.25 * (y as f64 + 4716.0)).floor()
            + (30.6001 * (m as f64 + 1.0)).floor()
            + day as f64
            + b
            - 1524.5
            + hour / 24.0
    }

    /// Total Solar Eclipse 2017-08-21. NASA Five Millennium Canon:
    /// γ = 0.4367, global type Total, greatest eclipse 18:25:31.8 UT (ΔT=68.4s).
    #[test]
    fn solar_eclipse_2017_aug_21_matches_nasa() {
        let a = almanac();
        // New Moon ~18:30 UT on 2017-08-21; pass the UT1 conjunction instant.
        let jd_new_ut = jd_tt(2017, 8, 21, 18.5) - 68.4 / 86400.0; // approx UT seed
        let g = classify_solar_eclipse(&a, jd_new_ut).expect("eclipse found");
        assert_eq!(
            g.eclipse_type,
            GlobalSolarType::Total,
            "2017 is a total eclipse"
        );
        // γ matches NASA to ~0.0015 ER. The residual is the truncated-ELP Moon
        // series limit (the apparent Moon now carries Δψ + geocentric light-time
        // but NOT annual aberration — see vsop::apparent_moon); the cone angles
        // match NASA to <0.05%, so the Besselian method itself is sound. The
        // greatest-eclipse time agrees with NASA to ~2 s (was ~19-30 s before the
        // Moon annual-aberration bug was removed).
        assert!(
            (g.gamma - 0.4367).abs() < 0.0025,
            "γ should match NASA 0.4367 within ephemeris-limited tolerance, got {:.5}",
            g.gamma
        );
        // Greatest eclipse 18:25:31.8 UT → TT = +68.4 s. Compare in UT.
        let greatest_ut = g.jd_greatest_tt.as_f64() - 68.4 / 86400.0;
        let target_ut = jd_tt(2017, 8, 21, 18.0 + (25.0 + 31.8 / 60.0) / 60.0);
        let err_sec = (greatest_ut - target_ut).abs() * 86400.0;
        assert!(
            err_sec < 30.0,
            "greatest eclipse should be within 30 s of 18:25:31.8 UT, off by {err_sec:.1} s"
        );
    }

    /// Total Solar Eclipse 2024-04-08. NASA: γ = 0.3431, Total,
    /// greatest eclipse 18:17:18.3 UT (ΔT=70.6s).
    #[test]
    fn solar_eclipse_2024_apr_08_matches_nasa() {
        let a = almanac();
        let jd_new_ut = jd_tt(2024, 4, 8, 18.3) - 70.6 / 86400.0;
        let g = classify_solar_eclipse(&a, jd_new_ut).expect("eclipse found");
        assert_eq!(
            g.eclipse_type,
            GlobalSolarType::Total,
            "2024 is a total eclipse"
        );
        // γ matches NASA to ~0.0019 ER (ELP Moon-position limited, see 2017 test).
        assert!(
            (g.gamma - 0.3431).abs() < 0.0025,
            "γ should match NASA 0.3431 within ELP-limited tolerance, got {:.5}",
            g.gamma
        );
        let greatest_ut = g.jd_greatest_tt.as_f64() - 70.6 / 86400.0;
        let target_ut = jd_tt(2024, 4, 8, 18.0 + (17.0 + 18.3 / 60.0) / 60.0);
        let err_sec = (greatest_ut - target_ut).abs() * 86400.0;
        assert!(
            err_sec < 30.0,
            "greatest eclipse should be within 30 s of 18:17:18.3 UT, off by {err_sec:.1} s"
        );
    }

    /// Hybrid (annular-total) Solar Eclipse 2023-04-20 — the "Ningaloo" eclipse.
    /// This is the regression case that the OLD geocentric-l2 classifier got
    /// wrong (its l2 is positive/antumbral at greatest eclipse, so it would have
    /// been mislabelled Annular); the sub-point criterion L2_sub = l2 − ζ·tan f2
    /// correctly yields Hybrid (total at the sub-point, annular toward the path
    /// ends). γ is not pinned to a fetched NASA value here (the canon page was
    /// unreachable); the type is the meaningful assertion.
    #[test]
    fn solar_eclipse_2023_apr_20_is_hybrid() {
        let a = almanac();
        // New Moon ~04:13 UT on 2023-04-20; greatest eclipse ~04:17 UT.
        let jd_new_ut = jd_tt(2023, 4, 20, 4.22) - 69.2 / 86400.0;
        let g = classify_solar_eclipse(&a, jd_new_ut).expect("eclipse found");
        assert_eq!(
            g.eclipse_type,
            GlobalSolarType::Hybrid,
            "2023-04-20 is the canonical hybrid eclipse; got {:?} (l2={:.6})",
            g.eclipse_type,
            g.l2_at_greatest
        );
        // |γ| ≈ 0.39 (the well-known γ-magnitude of this eclipse). The sign is
        // negative here — the path lay in the southern hemisphere, so the axis
        // passes south of Earth's centre — consistent with the signed convention
        // (north = +) that produced +0.4382 for the northern 2017 path. The exact
        // signed NASA value is not asserted (the canon page was unreachable), so
        // we check the magnitude only.
        assert!(
            (0.35..0.43).contains(&g.gamma.abs()),
            "|γ| should be ~0.39 for 2023-04-20, got {:.4}",
            g.gamma
        );
    }

    /// tan f1 / tan f2 at the 2017 epoch should match NASA's published cone
    /// angles (0.0046222 / 0.0045992) to a fraction of a percent.
    #[test]
    fn cone_angles_2017_match_nasa() {
        let a = almanac();
        let b = besselian_elements(&a, JdTT(jd_tt(2017, 8, 21, 18.0 + 68.4 / 3600.0)))
            .expect("elements");
        assert!(
            (b.tan_f1 - 0.0046222).abs() < 0.00002,
            "tan f1 ≈ 0.0046222, got {:.7}",
            b.tan_f1
        );
        assert!(
            (b.tan_f2 - 0.0045992).abs() < 0.00002,
            "tan f2 ≈ 0.0045992, got {:.7}",
            b.tan_f2
        );
    }
}
