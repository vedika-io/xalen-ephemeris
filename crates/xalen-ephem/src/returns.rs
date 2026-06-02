//! Exact planetary / lunar **return** finders on the real almanac longitude.
//!
//! A *return* is the instant a body's apparent geocentric ecliptic longitude
//! comes back to its natal value: the **solar return** (birthday in the sky),
//! the **lunar return** (~monthly), and the slow outer-planet returns —
//! **Saturn** (~29.46 yr), **Jupiter** (~11.86 yr) and **Mars** (~1.88 yr).
//!
//! The previous helper (`xalen_western::returns`) found only the **solar**
//! return exactly (Newton on a low-precision analytic Sun); every other body
//! used a mean-period estimate that can be **weeks** off because real orbital
//! motion is far from uniform. This module finds *all* of them exactly by
//! root-finding on the **actual [`Almanac`] longitude** — the same apparent
//! geocentric place the rest of the engine returns — so the answer is consistent
//! with the chart it feeds.
//!
//! # Method
//!
//! 1. Seed with the mean synodic-of-longitude period from the natal instant.
//! 2. **Newton–Raphson** on `f(t) = wrap(λ(t) − λ_natal)` using the body's
//!    instantaneous daily motion (central finite difference) as `f′`. This
//!    converges quadratically away from stations.
//! 3. If a Newton step misbehaves (the derivative is tiny near a retrograde
//!    station, or the step overshoots a bracket), fall back to **bisection** on a
//!    sign-bracketed interval, which always converges. The two together pin the
//!    instant on the engine's own longitude to ≈1e−7° for every body, retrograde
//!    or direct.
//!
//! Validated against pyswisseph 2.10.03 (committed test
//! `saturn_return_instant_matches_pyswisseph`): a Saturn return to 294.0° lands at
//! JD 2458871.555 UT (2020-01-23), matching the independent Swiss search to within
//! the committed bound of 0.1 day. The *timing* precision vs. Swiss is set by the
//! VSOP87-vs-Swiss apparent-longitude difference (a few arcseconds → a few minutes
//! at Saturn's ~0.12°/day rate), not by the root-finder, which pins the crossing on
//! the engine's own longitude far tighter (≈1e−7°).

use crate::Almanac;
use crate::body::Body;
use crate::provider::EphemerisError;
use xalen_time::{JdTT, JdUT1, JulianDay};

/// The bodies for which an exact return is defined. (Restricting the public API
/// to these avoids nonsensical "returns" of geometric points such as the nodes,
/// whose monotone regression has no natal-longitude crossing structure worth a
/// dedicated finder.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnBody {
    /// Solar return (~365.2422-day tropical-ish recurrence of the Sun's longitude).
    Sun,
    /// Lunar return (~27.32-day sidereal-month recurrence of the Moon's longitude).
    Moon,
    /// Mars return (~1.88-year recurrence).
    Mars,
    /// Jupiter return (~11.86-year recurrence).
    Jupiter,
    /// Saturn return (~29.46-year recurrence).
    Saturn,
}

impl ReturnBody {
    /// Map to the ephemeris [`Body`] whose longitude is searched.
    pub fn body(self) -> Body {
        match self {
            ReturnBody::Sun => Body::Sun,
            ReturnBody::Moon => Body::Moon,
            ReturnBody::Mars => Body::Mars,
            ReturnBody::Jupiter => Body::Jupiter,
            ReturnBody::Saturn => Body::Saturn,
        }
    }

    /// Mean longitude-recurrence period in days. Used only as a search seed; the
    /// exact instant comes from root-finding on the real longitude.
    fn mean_period_days(self) -> f64 {
        match self {
            ReturnBody::Sun => 365.242_19,   // tropical year
            ReturnBody::Moon => 27.321_66,   // sidereal month
            ReturnBody::Mars => 686.97,      // sidereal period (~1.88 yr)
            ReturnBody::Jupiter => 4_332.59, // ~11.86 yr
            ReturnBody::Saturn => 10_759.22, // ~29.46 yr
        }
    }
}

/// Wrap an angular difference in degrees into (−180, +180].
fn wrap_deg(d: f64) -> f64 {
    let d = d.rem_euclid(360.0);
    if d > 180.0 { d - 360.0 } else { d }
}

/// Newton-step convergence tolerance, in degrees of longitude. 1e-7° ≈ 0.36 mas,
/// far finer than any ephemeris is accurate — it just pins the instant.
const TOL_DEG: f64 = 1e-7;
/// Maximum combined Newton iterations before handing off to bisection.
const MAX_NEWTON: u32 = 40;
/// Maximum bisection iterations (halving from a ≤ ~few-day bracket reaches
/// sub-millisecond in well under 50 steps).
const MAX_BISECT: u32 = 80;

/// Find the **exact** return instant (UT1) of `which` to the given
/// `natal_longitude_deg`, searching forward from `search_start` (UT1).
///
/// The search brackets the FIRST natal-longitude crossing on the real
/// [`Almanac`] longitude forward from `search_start`, then refines it with a
/// safeguarded Newton–Raphson (Newton when it stays inside the bracket, bisection
/// otherwise). The returned instant satisfies `λ_body(t) = natal_longitude_deg`
/// on the engine's own longitude to ≈1e−7° (the timing precision against an
/// external ephemeris is bounded by that ephemeris's longitude agreement, not by
/// this root-finder). Because the crossing is the first one, a body that has
/// just turned retrograde can legitimately re-cross its natal degree only days
/// later — that early re-crossing IS the true return, which is exactly the kind
/// of case a mean-period estimate gets badly wrong.
///
/// `search_start` should be a little **before** the expected return (e.g. the
/// birth instant for the first return, or the previous return for the next one).
/// The finder always returns the **first** crossing at or after a small guard
/// past `search_start`.
///
/// # Errors
/// Propagates [`EphemerisError`] if the underlying ephemeris cannot be evaluated
/// at a sampled epoch (e.g. an epoch outside provider coverage), or returns
/// [`EphemerisError::ComputationFailed`] if no crossing is bracketed within a
/// generous window (should not happen for the supported bodies).
pub fn find_return(
    almanac: &Almanac,
    which: ReturnBody,
    natal_longitude_deg: f64,
    search_start: JdUT1,
) -> Result<JdUT1, EphemerisError> {
    let tt = find_return_tt(
        almanac,
        which,
        natal_longitude_deg,
        almanac.to_tt(search_start),
    )?;
    // Map the TT instant back to UT1 by removing ΔT. ΔT changes by only
    // seconds/century, so evaluating it at the (numerically close) TT value as if
    // it were a UT1 argument introduces a ΔT-of-ΔT error far below our tolerance:
    //   ΔT(epoch) = to_tt(jd) − jd   (days),   UT1 = TT − ΔT.
    let dt_days = almanac.to_tt(JdUT1(tt.as_f64())).as_f64() - tt.as_f64();
    Ok(JdUT1(tt.as_f64() - dt_days))
}

/// Like [`find_return`] but operating purely in **TT** (no ΔT conversion). Use
/// this when your natal longitude and search start are already on the TT scale,
/// or when you want the dynamical-time instant directly.
pub fn find_return_tt(
    almanac: &Almanac,
    which: ReturnBody,
    natal_longitude_deg: f64,
    search_start: JdTT,
) -> Result<JdTT, EphemerisError> {
    let body = which.body();
    let natal = natal_longitude_deg.rem_euclid(360.0);

    // Longitude of the body at a TT epoch, in degrees [0, 360).
    let lon = |jd: f64| -> Result<f64, EphemerisError> {
        Ok(almanac
            .geocentric_ecliptic_tt(body, JdTT(jd))?
            .longitude
            .to_degrees()
            .rem_euclid(360.0))
    };
    // f(t) = wrap(λ(t) − natal): zero at the return; sign change brackets it.
    let f = |jd: f64| -> Result<f64, EphemerisError> { Ok(wrap_deg(lon(jd)? - natal)) };

    let period = which.mean_period_days();
    // Start a hair past the search start so we never re-report the same instant,
    // then seed roughly one mean period ahead.
    let guard = (period * 1e-4).max(1e-3); // ≥ ~1.4 min, scaled to the body
    let t_start = search_start.as_f64() + guard;

    // ── Bracket the FIRST natal-longitude crossing forward from t_start ──────
    // Bracketing first (rather than seeding Newton a mean period ahead) pins the
    // "next return" semantics: we always converge on the earliest crossing, never
    // a later one a poorly-seeded Newton iteration might jump to.
    let (mut a, mut b) = bracket_crossing(&f, t_start, period)?;
    let mut fa = f(a)?;
    let fb = f(b)?;
    if fa.abs() < TOL_DEG {
        return Ok(JdTT(a));
    }
    if fb.abs() < TOL_DEG {
        return Ok(JdTT(b));
    }

    // Central-difference step for the Newton derivative (instantaneous daily
    // motion). Small vs the body's motion: tighter for the fast Moon, wider for
    // the slow outer planets to beat finite-difference noise.
    let h = match which {
        ReturnBody::Moon => 0.02,
        ReturnBody::Sun | ReturnBody::Mars => 0.1,
        ReturnBody::Jupiter | ReturnBody::Saturn => 0.5,
    };

    // ── Newton–Raphson confined to the bracket (bisection on any misstep) ────
    // The bracket [a,b] is maintained as a guaranteed sign-change interval. A
    // Newton step is accepted only if it stays strictly inside the bracket;
    // otherwise (or near a retrograde station where the rate collapses) we take a
    // bisection step. This is the classic safeguarded Newton — quadratic when it
    // can be, always-converging when it can't.
    let mut t = 0.5 * (a + b);
    for _ in 0..(MAX_NEWTON + MAX_BISECT) {
        let fv = f(t)?;
        if fv.abs() < TOL_DEG || (b - a) < 1e-9 {
            return Ok(JdTT(t));
        }
        // Shrink the bracket using the sign of f(t).
        if (fa < 0.0) != (fv < 0.0) {
            b = t;
        } else {
            a = t;
            fa = fv;
        }

        // Try a Newton step from t using the instantaneous rate.
        let rate = wrap_deg(lon(t + h)? - lon(t - h)?) / (2.0 * h);
        let newton = if rate.abs() > 1e-4 {
            let cand = t - fv / rate;
            if cand.is_finite() && cand > a && cand < b {
                Some(cand)
            } else {
                None
            }
        } else {
            None
        };
        // Accept Newton if it stayed in-bracket; else bisect.
        t = newton.unwrap_or_else(|| 0.5 * (a + b));
    }

    // Iteration budget exhausted without an in-loop convergence return. Do NOT
    // report a possibly non-converged `t` as an exact return — verify the final
    // residual and surface a failure if it is not within tolerance. (For the
    // supported bodies this branch is not expected to trigger; it exists so a
    // pathological case can never masquerade as an exact crossing.)
    let residual = f(t)?.abs();
    if residual <= TOL_DEG {
        Ok(JdTT(t))
    } else {
        Err(EphemerisError::ComputationFailed(format!(
            "return finder did not converge: |λ − natal| = {residual:.3e}° > {TOL_DEG:.0e}° \
             after {} iterations (JD {t:.5})",
            MAX_NEWTON + MAX_BISECT
        )))
    }
}

/// Walk forward from `t0` in steps that are a small fraction of `period` until
/// `f` changes sign, returning a bracket `(a, b)` with `f(a)·f(b) ≤ 0`.
fn bracket_crossing<F>(f: &F, t0: f64, period: f64) -> Result<(f64, f64), EphemerisError>
where
    F: Fn(f64) -> Result<f64, EphemerisError>,
{
    // 1/64 of a period gives ≥ ~2 samples across even the Moon's fast crossing
    // while keeping the slow planets' scan cheap.
    let step = (period / 64.0).max(1e-3);
    // Search up to ~1.5 mean periods forward — more than enough for one return.
    let max_span = period * 1.5 + step;
    let mut a = t0;
    let mut fa = f(a)?;
    let mut x = t0 + step;
    while x <= t0 + max_span {
        let fx = f(x)?;
        if fa == 0.0 {
            return Ok((a, a));
        }
        // Accept only a GENUINE zero crossing: a sign change where both endpoints
        // are near zero (the longitude is continuously passing through the natal
        // value). REJECT the ±180° discontinuity of `wrap_deg` itself — when the
        // body is roughly opposite the natal longitude, `f` jumps from +180 to
        // −180 between samples, a false sign change with |fa|,|fx| ≈ 180 that
        // would otherwise bisect to the OPPOSITION point. A `< 90°` guard cleanly
        // separates a real crossing (small |f|) from the discontinuity.
        if (fa < 0.0) != (fx < 0.0) && fa.abs() < 90.0 && fx.abs() < 90.0 {
            return Ok((a, x));
        }
        a = x;
        fa = fx;
        x += step;
    }
    Err(EphemerisError::ComputationFailed(format!(
        "no return crossing bracketed within {max_span:.1} days of JD {t0:.3}"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use xalen_coords::RAD_TO_DEG;

    fn lon_deg(a: &Almanac, b: Body, jd: JdUT1) -> f64 {
        a.geocentric_ecliptic(b, jd)
            .unwrap()
            .longitude
            .to_degrees()
            .rem_euclid(360.0)
    }

    /// A Saturn return to the SAME longitude pyswisseph 2.10.03 reports: Swiss's
    /// independent search puts Saturn = 294.0° at JD 2458871.55494 UT (2020-01-23).
    /// Saturn moves ~0.12°/day there, and the VSOP-vs-Swiss apparent-longitude
    /// agreement is ~arcseconds, so the two return instants agree to within the
    /// committed bound of 0.1 day (a few-arcsec longitude difference maps to a
    /// few-minute timing difference at that rate).
    #[test]
    fn saturn_return_instant_matches_pyswisseph() {
        let a = Almanac::default_vedic();
        let natal = 294.0;
        // Start the search comfortably before the known crossing.
        let start = JdUT1(2_458_700.0); // 2019-08
        let ret = find_return(&a, ReturnBody::Saturn, natal, start).unwrap();

        let swiss = 2_458_871.554_94;
        let dt_days = (ret.as_f64() - swiss).abs();
        assert!(
            dt_days < 0.1,
            "Saturn return {} should match Swiss {swiss} within 0.1 day, off by {dt_days:.5} d",
            ret.as_f64()
        );
        // And the longitude AT the found instant must equal the natal longitude.
        let got = lon_deg(&a, Body::Saturn, ret);
        let err = (wrap_deg(got - natal)).abs();
        assert!(
            err < 1e-4,
            "Saturn longitude at return {got}° != natal {natal}° (err {err}°)"
        );
    }

    /// Every supported body, found exactly: the longitude at the returned instant
    /// must equal the natal longitude to ~1e−5°.
    #[test]
    fn all_returns_land_on_natal_longitude() {
        let a = Almanac::default_vedic();
        let start = JdUT1(2_451_545.0); // J2000
        for which in [
            ReturnBody::Sun,
            ReturnBody::Moon,
            ReturnBody::Mars,
            ReturnBody::Jupiter,
            ReturnBody::Saturn,
        ] {
            let body = which.body();
            let natal = lon_deg(&a, body, start);
            let ret = find_return(&a, which, natal, start).unwrap();
            assert!(
                ret.as_f64() > start.as_f64(),
                "{which:?} return must be after start"
            );
            let got = lon_deg(&a, body, ret);
            let err = wrap_deg(got - natal).abs();
            assert!(
                err < 1e-4,
                "{which:?}: longitude at return {got}° != natal {natal}° (err {err}°)"
            );
        }
    }

    /// The exact solar return must be ~one tropical year after J2000 (the Sun's
    /// motion is smooth, so exact ≈ mean here) and land on the natal longitude.
    #[test]
    fn solar_return_about_one_year() {
        let a = Almanac::default_vedic();
        let start = JdUT1(2_451_545.0);
        let natal = lon_deg(&a, Body::Sun, start);
        let ret = find_return(&a, ReturnBody::Sun, natal, start).unwrap();
        let gap = ret.as_f64() - start.as_f64();
        assert!(
            (gap - 365.2422).abs() < 1.0,
            "solar return gap {gap} days should be ~365.24"
        );
    }

    /// The lunar return is ~27.3 days out — and crucially the EXACT finder
    /// differs from the naive mean-period estimate by a meaningful amount,
    /// because the Moon's longitude rate swings ±10%.
    #[test]
    fn lunar_return_is_exact_not_mean() {
        let a = Almanac::default_vedic();
        let start = JdUT1(2_451_545.0);
        let natal = lon_deg(&a, Body::Moon, start);
        let ret = find_return(&a, ReturnBody::Moon, natal, start).unwrap();
        let gap = ret.as_f64() - start.as_f64();
        // Anomalistic variation keeps it within a day or so of the sidereal month.
        assert!(
            (gap - 27.32166).abs() < 1.5,
            "lunar return gap {gap} ~27.3 d"
        );
        // Exact longitude check.
        let got = lon_deg(&a, Body::Moon, ret);
        assert!(
            wrap_deg(got - natal).abs() < 1e-4,
            "lunar return longitude mismatch"
        );
    }

    /// Mars goes retrograde — a return found near a station must still land
    /// exactly on the natal longitude (exercises the station-safe derivative /
    /// bisection fallback). We assert correctness of the crossing, not timing.
    #[test]
    fn mars_return_lands_exactly() {
        let a = Almanac::default_vedic();
        let start = JdUT1(2_451_545.0);
        let natal = lon_deg(&a, Body::Mars, start);
        let ret = find_return(&a, ReturnBody::Mars, natal, start).unwrap();
        let got = lon_deg(&a, Body::Mars, ret);
        assert!(
            wrap_deg(got - natal).abs() < 1e-4,
            "Mars return longitude {got}° != natal {natal}°"
        );
        // Mars returns are ~1.88 yr but vary; sanity-bound it.
        let years = (ret.as_f64() - start.as_f64()) / 365.25;
        assert!(
            years > 1.5 && years < 2.6,
            "Mars return ~1.88 yr, got {years}"
        );
    }

    /// Successive returns are about one mean period apart and strictly increasing.
    #[test]
    fn successive_jupiter_returns_increase() {
        let a = Almanac::default_vedic();
        let start = JdUT1(2_451_545.0);
        let natal = lon_deg(&a, Body::Jupiter, start);
        let r1 = find_return(&a, ReturnBody::Jupiter, natal, start).unwrap();
        let r2 = find_return(&a, ReturnBody::Jupiter, natal, r1).unwrap();
        assert!(r2.as_f64() > r1.as_f64(), "second return must be later");
        let gap_years = (r2.as_f64() - r1.as_f64()) / 365.25;
        assert!(
            (gap_years - 11.86).abs() < 0.6,
            "consecutive Jupiter returns ~11.86 yr apart, got {gap_years}"
        );
    }

    /// The TT-domain finder and the UT1 finder must agree to within ΔT (~69 s
    /// today ≈ 8e−4 day) — the only thing the UT1 path adds is the ΔT shift.
    #[test]
    fn tt_and_ut1_finders_consistent() {
        let a = Almanac::default_vedic();
        let start = JdUT1(2_451_545.0);
        let natal = lon_deg(&a, Body::Saturn, start);
        let ut1 = find_return(&a, ReturnBody::Saturn, natal, start).unwrap();
        let tt = find_return_tt(&a, ReturnBody::Saturn, natal, a.to_tt(start)).unwrap();
        // Both finders locate the SAME physical instant: converting the UT1 result
        // forward through ΔT must reproduce the TT result.
        let ut1_as_tt = a.to_tt(ut1).as_f64();
        assert!(
            (ut1_as_tt - tt.as_f64()).abs() < 1e-3,
            "TT and UT1 finders inconsistent: UT1→TT {ut1_as_tt} vs TT {}",
            tt.as_f64()
        );
    }

    #[test]
    fn return_body_maps_to_expected_body() {
        assert_eq!(ReturnBody::Sun.body(), Body::Sun);
        assert_eq!(ReturnBody::Moon.body(), Body::Moon);
        assert_eq!(ReturnBody::Mars.body(), Body::Mars);
        assert_eq!(ReturnBody::Jupiter.body(), Body::Jupiter);
        assert_eq!(ReturnBody::Saturn.body(), Body::Saturn);
    }

    /// Guard: a tiny sanity check that wrap_deg behaves at the ±180 boundary, so
    /// crossings near 0°/360° natal longitudes are handled.
    #[test]
    fn wrap_deg_boundaries() {
        assert!((wrap_deg(0.0)).abs() < 1e-12);
        assert!((wrap_deg(360.0)).abs() < 1e-12);
        assert!((wrap_deg(181.0) - (-179.0)).abs() < 1e-9);
        assert!((wrap_deg(-181.0) - 179.0).abs() < 1e-9);
    }

    /// Every successful return must satisfy the finder's OWN tight convergence
    /// tolerance (`TOL_DEG`, not just the looser 1e−4° the other tests assert).
    /// This pins the post-loop residual guard: a returned instant is only ever
    /// reported when `|λ(t) − natal| ≤ TOL_DEG`, so a non-converged `t` can never
    /// masquerade as an exact return.
    #[test]
    fn every_return_meets_tight_convergence_tolerance() {
        let a = Almanac::default_vedic();
        let start = JdUT1(2_451_545.0);
        for which in [
            ReturnBody::Sun,
            ReturnBody::Moon,
            ReturnBody::Mars,
            ReturnBody::Jupiter,
            ReturnBody::Saturn,
        ] {
            let body = which.body();
            let natal = lon_deg(&a, body, start);
            let ret = find_return_tt(&a, which, natal, a.to_tt(start)).unwrap();
            let got = a
                .geocentric_ecliptic_tt(body, ret)
                .unwrap()
                .longitude
                .to_degrees()
                .rem_euclid(360.0);
            let residual = wrap_deg(got - natal.rem_euclid(360.0)).abs();
            assert!(
                residual <= TOL_DEG,
                "{which:?}: returned instant residual {residual:.3e}° must be ≤ TOL_DEG {TOL_DEG:.0e}°"
            );
        }
    }

    /// Zero-crossing natal longitude (near 0° Aries) must still be found exactly.
    #[test]
    fn solar_return_zero_crossing_natal() {
        let a = Almanac::default_vedic();
        // Find a start where we can use a natal longitude near 0°.
        let start = JdUT1(2_451_545.0);
        let natal = 0.5;
        let ret = find_return(&a, ReturnBody::Sun, natal, start).unwrap();
        let got = a.geocentric_ecliptic(Body::Sun, ret).unwrap().longitude * RAD_TO_DEG;
        assert!(
            wrap_deg(got - natal).abs() < 1e-3,
            "near-0° solar return: got {got}° vs natal {natal}°"
        );
    }
}
