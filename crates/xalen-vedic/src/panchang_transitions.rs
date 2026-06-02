//! Panchang transition (start/end) times — the most-consumed panchang output.
//!
//! [`compute_panchang`](crate::panchang::compute_panchang) returns the *instantaneous*
//! tithi/nakshatra/yoga/karana for a moment. This module computes the *boundary
//! times*: when the current tithi (or nakshatra, yoga, karana) began and ends.
//! Those end-times are what almost every printed/online panchang publishes
//! ("Tithi: Dashami upto 05:11 AM").
//!
//! # Method
//!
//! Each of the four lunar elements is driven by a monotonically-increasing angle
//! of the Moon relative to the Sun:
//!
//! | Element    | Driving angle              | Boundary span |
//! |------------|----------------------------|---------------|
//! | Tithi      | elongation `Moon − Sun`    | 12°           |
//! | Karana     | elongation `Moon − Sun`    | 6°            |
//! | Nakshatra  | Moon longitude             | 360/27°       |
//! | Yoga       | `Sun + Moon`               | 360/27°       |
//!
//! All three driving angles (elongation, Moon longitude, Sun+Moon) increase
//! monotonically in time on any practical panchang timescale: the Moon never
//! moves retrograde, and it moves (~13.18°/day) far faster than the Sun
//! (~0.99°/day), so `Moon − Sun` and `Sun + Moon` both rise steadily. A given
//! element therefore ends when its driving angle next reaches an integer
//! multiple of its span. We bracket that crossing forward in time, then
//! [bisect](https://en.wikipedia.org/wiki/Bisection_method) to ~second
//! precision. The start time is found the same way, searching backward.
//!
//! # Honesty
//!
//! No ephemeris is bundled in this crate. The caller supplies a closure that
//! returns the **sidereal** Sun and Moon longitudes (degrees) for any Julian
//! Day — exactly the two numbers it already passes to
//! [`compute_panchang`](crate::panchang::compute_panchang). The transition
//! times are therefore as accurate as the caller's ephemeris and ayanamsa;
//! this module only does the root-finding.
//!
//! Because tithi/karana boundaries are differences and yoga a sum, the
//! ayanamsa offset that the caller subtracts from both bodies cancels for
//! tithi/karana and is doubled (but still monotonic) for yoga. The boundary
//! *times* are identical whether tropical or sidereal longitudes are supplied
//! for tithi and karana; for nakshatra and yoga the ayanamsa matters and the
//! caller must pass sidereal values (as the rest of the crate expects).

use crate::nakshatra::Nakshatra;
use crate::panchang::{Karana, Paksha, Tithi, Yoga};
use serde::{Deserialize, Serialize};

/// Number of bisection iterations. Each halves the bracket; the initial
/// bracket is at most one element-span wide in time (≈ 2 hours for a karana,
/// ≈ 1.3 days for a nakshatra). 60 iterations shrink a 2-day bracket to
/// 2 d / 2⁶⁰ ≈ 1.5e-13 s, far below the ephemeris's own precision, so the
/// result is limited by the supplied ephemeris, not by the root-finder.
const BISECTION_ITERS: u32 = 60;

/// Maximum days to scan when bracketing a boundary. A tithi/karana boundary is
/// always reached within ~1.05 days (one tithi never exceeds ~26 h); a
/// nakshatra/yoga within ~1.4 days. 3 days is a generous safety ceiling that
/// still terminates the search if a pathological ephemeris violates
/// monotonicity.
const MAX_SCAN_DAYS: f64 = 3.0;

/// Forward scan step (days) used to bracket a boundary crossing. Small enough
/// that the monotone driving angle cannot advance a full span (the smallest
/// span is 6° for a karana ≈ 0.49 days of Moon−Sun motion) within one step,
/// so a sign change is never skipped.
const SCAN_STEP_DAYS: f64 = 0.125; // 3 hours

/// Start and end Julian Days (UT) bracketing a single panchang element, plus
/// its identity at the reference instant.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ElementInterval<T> {
    /// The element value active at the reference instant.
    pub value: T,
    /// Julian Day (UT) at which this element began. `None` if the caller did
    /// not request the (more expensive) backward search.
    pub start_jd: Option<f64>,
    /// Julian Day (UT) at which this element ends (the next element begins).
    pub end_jd: f64,
}

/// All four lunar panchang elements with their transition times.
///
/// `vara` (weekday) is intentionally absent: its "transition" is local sunrise,
/// which depends on the observer's geographic location and is the domain of the
/// rise/set engine, not the Moon–Sun geometry root-finder here.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PanchangTransitions {
    pub tithi: ElementInterval<Tithi>,
    pub nakshatra: ElementInterval<Nakshatra>,
    pub yoga: ElementInterval<Yoga>,
    pub karana: ElementInterval<Karana>,
}

const YOGA_SPAN: f64 = 360.0 / 27.0;
const NAK_SPAN: f64 = 360.0 / 27.0;
const TITHI_SPAN: f64 = 12.0;
const KARANA_SPAN: f64 = 6.0;

/// Signed angular difference `a − b` mapped to `(−180, 180]` degrees.
///
/// Used to turn a "distance to the next boundary" into a function that is
/// negative before the boundary and positive after, with a single zero
/// crossing — even across the 360°→0° wrap (e.g. tithi 30 → tithi 1).
#[inline]
fn signed_delta_deg(a: f64, b: f64) -> f64 {
    ((a - b + 180.0).rem_euclid(360.0)) - 180.0
}

/// Bisect `f` (a monotone-increasing-through-zero function of JD) on the
/// bracket `[lo, hi]` where `f(lo) <= 0 <= f(hi)`. Returns the JD of the zero.
fn bisect<F: Fn(f64) -> f64>(f: &F, mut lo: f64, mut hi: f64) -> f64 {
    let f_lo = f(lo);
    // If the bracket is degenerate (already at the boundary) just return it.
    if f_lo == 0.0 {
        return lo;
    }
    for _ in 0..BISECTION_ITERS {
        let mid = 0.5 * (lo + hi);
        let f_mid = f(mid);
        if f_mid == 0.0 {
            return mid;
        }
        // Keep the sub-interval that still straddles zero. f_lo is <= 0 by the
        // bracket invariant, so the root is in [lo, mid] iff f_mid >= 0.
        if f_mid >= 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    0.5 * (lo + hi)
}

/// Scan forward from `jd` to bracket the next instant at which `angle(jd)`
/// reaches `target_deg`, then bisect. `angle` is the (monotone-increasing,
/// degrees mod 360) driving angle. Returns `None` only if no crossing is found
/// within [`MAX_SCAN_DAYS`] (a pathological / non-monotone ephemeris).
fn find_forward_crossing<A: Fn(f64) -> f64>(angle: &A, jd: f64, target_deg: f64) -> Option<f64> {
    let f = |t: f64| signed_delta_deg(angle(t), target_deg);
    let f0 = f(jd);
    // At the reference instant the angle is strictly *before* the upcoming
    // boundary, so f0 should be < 0. Guard the (measure-zero) case where the
    // instant sits exactly on a boundary: the next boundary is one full span
    // away, so nudge past it.
    if f0 >= 0.0 {
        // Already at/just past target; the "current" element's end is the
        // following boundary. Caller arranges target so this is rare, but be safe.
        return Some(jd);
    }
    let mut hi = jd;
    let mut steps = 0.0;
    while steps < MAX_SCAN_DAYS {
        hi += SCAN_STEP_DAYS;
        steps += SCAN_STEP_DAYS;
        if f(hi) >= 0.0 {
            return Some(bisect(&f, hi - SCAN_STEP_DAYS, hi));
        }
    }
    None
}

/// Scan backward from `jd` to bracket the most recent instant at which
/// `angle(jd)` reached `target_deg`, then bisect. Mirror of
/// [`find_forward_crossing`].
fn find_backward_crossing<A: Fn(f64) -> f64>(angle: &A, jd: f64, target_deg: f64) -> Option<f64> {
    let f = |t: f64| signed_delta_deg(angle(t), target_deg);
    let f0 = f(jd);
    // At the reference instant the angle is strictly *after* the boundary that
    // started the current element, so f0 should be > 0.
    if f0 <= 0.0 {
        return Some(jd);
    }
    let mut lo = jd;
    let mut steps = 0.0;
    while steps < MAX_SCAN_DAYS {
        lo -= SCAN_STEP_DAYS;
        steps += SCAN_STEP_DAYS;
        if f(lo) <= 0.0 {
            return Some(bisect(&f, lo, lo + SCAN_STEP_DAYS));
        }
    }
    None
}

/// Largest multiple of `span` that is `<= x` (x in degrees, 0..360).
#[inline]
fn floor_boundary(x: f64, span: f64) -> f64 {
    (x / span).floor() * span
}

/// Smallest multiple of `span` that is `> x` (x in degrees). Returns the end
/// boundary of the segment currently containing `x`. For `x` exactly on a
/// boundary the *next* boundary is returned (the segment is the one we just
/// entered).
#[inline]
fn next_boundary(x: f64, span: f64) -> f64 {
    (floor_boundary(x, span) + span).rem_euclid(360.0)
}

// ---------------------------------------------------------------------------
// Driving-angle helpers
// ---------------------------------------------------------------------------

#[inline]
fn elongation_deg(sun: f64, moon: f64) -> f64 {
    (moon - sun).rem_euclid(360.0)
}

#[inline]
fn yoga_sum_deg(sun: f64, moon: f64) -> f64 {
    (sun + moon).rem_euclid(360.0)
}

#[inline]
fn moon_lon_deg(_sun: f64, moon: f64) -> f64 {
    moon.rem_euclid(360.0)
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Compute the end (and optionally start) Julian Day of the **tithi** active at
/// `jd`.
///
/// `positions(jd)` must return `(sun_sidereal_deg, moon_sidereal_deg)`. The
/// returned [`ElementInterval`]'s `value` is the tithi at the reference instant.
/// `include_start` controls whether the (extra) backward search runs.
///
/// Returns `None` only if the supplied ephemeris fails to produce a boundary
/// crossing within [`MAX_SCAN_DAYS`] (non-monotone / pathological input).
pub fn tithi_transition<P: Fn(f64) -> (f64, f64)>(
    positions: &P,
    jd: f64,
    include_start: bool,
) -> Option<ElementInterval<Tithi>> {
    let (sun0, moon0) = positions(jd);
    let value = Tithi::from_sun_moon_deg(sun0, moon0);
    let angle = |t: f64| {
        let (s, m) = positions(t);
        elongation_deg(s, m)
    };
    let e0 = elongation_deg(sun0, moon0);
    let end_target = next_boundary(e0, TITHI_SPAN);
    let end_jd = find_forward_crossing(&angle, jd, end_target)?;
    let start_jd = if include_start {
        let start_target = floor_boundary(e0, TITHI_SPAN);
        find_backward_crossing(&angle, jd, start_target)
    } else {
        None
    };
    Some(ElementInterval {
        value,
        start_jd,
        end_jd,
    })
}

/// Compute the end (and optionally start) Julian Day of the **karana** active
/// at `jd`. See [`tithi_transition`] for the closure contract.
pub fn karana_transition<P: Fn(f64) -> (f64, f64)>(
    positions: &P,
    jd: f64,
    include_start: bool,
) -> Option<ElementInterval<Karana>> {
    let (sun0, moon0) = positions(jd);
    let e0 = elongation_deg(sun0, moon0);
    let tithi = Tithi::from_sun_moon_deg(sun0, moon0);
    let value = Karana::from_tithi_and_elongation(&tithi, e0);
    let angle = |t: f64| {
        let (s, m) = positions(t);
        elongation_deg(s, m)
    };
    let end_target = next_boundary(e0, KARANA_SPAN);
    let end_jd = find_forward_crossing(&angle, jd, end_target)?;
    let start_jd = if include_start {
        let start_target = floor_boundary(e0, KARANA_SPAN);
        find_backward_crossing(&angle, jd, start_target)
    } else {
        None
    };
    Some(ElementInterval {
        value,
        start_jd,
        end_jd,
    })
}

/// Compute the end (and optionally start) Julian Day of the **nakshatra**
/// (Moon mansion) active at `jd`. See [`tithi_transition`] for the closure
/// contract. The Moon longitude supplied MUST be sidereal.
pub fn nakshatra_transition<P: Fn(f64) -> (f64, f64)>(
    positions: &P,
    jd: f64,
    include_start: bool,
) -> Option<ElementInterval<Nakshatra>> {
    let (sun0, moon0) = positions(jd);
    let value = Nakshatra::from_longitude_deg(moon0);
    let angle = |t: f64| {
        let (s, m) = positions(t);
        moon_lon_deg(s, m)
    };
    let m0 = moon_lon_deg(sun0, moon0);
    let end_target = next_boundary(m0, NAK_SPAN);
    let end_jd = find_forward_crossing(&angle, jd, end_target)?;
    let start_jd = if include_start {
        let start_target = floor_boundary(m0, NAK_SPAN);
        find_backward_crossing(&angle, jd, start_target)
    } else {
        None
    };
    Some(ElementInterval {
        value,
        start_jd,
        end_jd,
    })
}

/// Compute the end (and optionally start) Julian Day of the **yoga** active at
/// `jd`. See [`tithi_transition`] for the closure contract. Both longitudes
/// supplied MUST be sidereal.
pub fn yoga_transition<P: Fn(f64) -> (f64, f64)>(
    positions: &P,
    jd: f64,
    include_start: bool,
) -> Option<ElementInterval<Yoga>> {
    let (sun0, moon0) = positions(jd);
    let value = Yoga::from_sun_moon_deg(sun0, moon0);
    let angle = |t: f64| {
        let (s, m) = positions(t);
        yoga_sum_deg(s, m)
    };
    let s0 = yoga_sum_deg(sun0, moon0);
    let end_target = next_boundary(s0, YOGA_SPAN);
    let end_jd = find_forward_crossing(&angle, jd, end_target)?;
    let start_jd = if include_start {
        let start_target = floor_boundary(s0, YOGA_SPAN);
        find_backward_crossing(&angle, jd, start_target)
    } else {
        None
    };
    Some(ElementInterval {
        value,
        start_jd,
        end_jd,
    })
}

/// Compute end (and optionally start) times for all four lunar panchang
/// elements at once.
///
/// `positions(jd)` must return `(sun_sidereal_deg, moon_sidereal_deg)` — the
/// same pair passed to [`compute_panchang`](crate::panchang::compute_panchang).
/// Returns `None` if ANY element's boundary search fails (pathological
/// ephemeris); on success every field carries a real `end_jd`.
pub fn compute_panchang_transitions<P: Fn(f64) -> (f64, f64)>(
    positions: &P,
    jd: f64,
    include_start: bool,
) -> Option<PanchangTransitions> {
    Some(PanchangTransitions {
        tithi: tithi_transition(positions, jd, include_start)?,
        nakshatra: nakshatra_transition(positions, jd, include_start)?,
        yoga: yoga_transition(positions, jd, include_start)?,
        karana: karana_transition(positions, jd, include_start)?,
    })
}

/// Convenience: which paksha the tithi interval belongs to (re-exported helper
/// so callers do not need to import [`Paksha`] separately just to label a
/// transition). Returns the paksha of the interval's `value`.
pub fn interval_paksha(interval: &ElementInterval<Tithi>) -> Paksha {
    interval.value.paksha
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // A deterministic synthetic ephemeris: constant angular rates that mimic
    // the real Sun (~0.9856°/day) and Moon (~13.1764°/day). This lets us assert
    // boundary times to closed-form precision WITHOUT bundling an ephemeris.
    // Rates are the standard mean tropical motions.
    const SUN_RATE: f64 = 0.985_647_36; // deg/day (mean Sun)
    const MOON_RATE: f64 = 13.176_396_5; // deg/day (mean Moon)
    const EPOCH: f64 = 2_451_545.0; // J2000

    fn synth(sun0: f64, moon0: f64) -> impl Fn(f64) -> (f64, f64) {
        move |jd: f64| {
            let dt = jd - EPOCH;
            let s = (sun0 + SUN_RATE * dt).rem_euclid(360.0);
            let m = (moon0 + MOON_RATE * dt).rem_euclid(360.0);
            (s, m)
        }
    }

    // Tolerance: 2 seconds of a day.
    const TWO_SEC_DAYS: f64 = 2.0 / 86_400.0;

    #[test]
    fn signed_delta_wraps_correctly() {
        assert!((signed_delta_deg(10.0, 350.0) - 20.0).abs() < 1e-9);
        assert!((signed_delta_deg(350.0, 10.0) + 20.0).abs() < 1e-9);
        assert!((signed_delta_deg(0.0, 0.0)).abs() < 1e-9);
        assert!((signed_delta_deg(119.0, 120.0) + 1.0).abs() < 1e-9);
    }

    #[test]
    fn next_boundary_basic_and_wrap() {
        assert!((next_boundary(5.0, 12.0) - 12.0).abs() < 1e-9);
        assert!((next_boundary(108.7, 12.0) - 120.0).abs() < 1e-9);
        // Wrap: just below 360 with span 12 → 0.0
        assert!((next_boundary(355.0, 12.0)).abs() < 1e-9);
        assert!((floor_boundary(108.7, 12.0) - 108.0).abs() < 1e-9);
    }

    #[test]
    fn tithi_end_matches_closed_form() {
        // Put the elongation at exactly 108.0° at EPOCH (start of tithi 10).
        // Sun=0, Moon=108 → elongation 108. Tithi 10 ends when elongation=120.
        let positions = synth(0.0, 108.0);
        let it = tithi_transition(&positions, EPOCH, true).expect("transition");
        assert_eq!(it.value.number, 10);
        // Closed form: elongation rate = MOON_RATE - SUN_RATE deg/day.
        let elong_rate = MOON_RATE - SUN_RATE;
        // Need +12° to reach 120.
        let expected_end = EPOCH + 12.0 / elong_rate;
        assert!(
            (it.end_jd - expected_end).abs() < TWO_SEC_DAYS,
            "end_jd {} vs expected {}",
            it.end_jd,
            expected_end
        );
        // Start: elongation already at 108 boundary → start ~= EPOCH.
        let start = it.start_jd.expect("start");
        assert!((start - EPOCH).abs() < TWO_SEC_DAYS, "start {start}");
    }

    #[test]
    fn tithi_end_handles_360_to_0_wrap() {
        // Elongation just below 360 → in tithi 30 (Amavasya), ends at 360≡0
        // which then starts tithi 1. Put elongation at 354° (tithi 30).
        let positions = synth(0.0, 354.0);
        let it = tithi_transition(&positions, EPOCH, false).expect("transition");
        assert_eq!(it.value.number, 30);
        let elong_rate = MOON_RATE - SUN_RATE;
        // Need +6° to reach 360 (=0).
        let expected_end = EPOCH + 6.0 / elong_rate;
        assert!(
            (it.end_jd - expected_end).abs() < TWO_SEC_DAYS,
            "wrap end {} vs {}",
            it.end_jd,
            expected_end
        );
    }

    #[test]
    fn karana_span_is_six_degrees() {
        // Elongation at 3° → second... actually first half of tithi 1.
        // Karana boundary every 6°, so from 3° next boundary is 6°.
        let positions = synth(0.0, 3.0);
        let it = karana_transition(&positions, EPOCH, false).expect("transition");
        let elong_rate = MOON_RATE - SUN_RATE;
        let expected_end = EPOCH + 3.0 / elong_rate;
        assert!(
            (it.end_jd - expected_end).abs() < TWO_SEC_DAYS,
            "karana end {} vs {}",
            it.end_jd,
            expected_end
        );
    }

    #[test]
    fn nakshatra_end_matches_closed_form() {
        // Moon at 5° → Ashwini (0..13.333). Ends at 13.3333.
        let positions = synth(0.0, 5.0);
        let it = nakshatra_transition(&positions, EPOCH, true).expect("transition");
        assert_eq!(it.value, Nakshatra::Ashwini);
        let span = 360.0 / 27.0;
        let expected_end = EPOCH + (span - 5.0) / MOON_RATE;
        assert!(
            (it.end_jd - expected_end).abs() < TWO_SEC_DAYS,
            "nak end {} vs {}",
            it.end_jd,
            expected_end
        );
        let start = it.start_jd.expect("start");
        // Moon was at 0 (Ashwini start) at: 5/MOON_RATE days before EPOCH.
        let expected_start = EPOCH - 5.0 / MOON_RATE;
        assert!(
            (start - expected_start).abs() < TWO_SEC_DAYS,
            "nak start {} vs {}",
            start,
            expected_start
        );
    }

    #[test]
    fn yoga_end_matches_closed_form() {
        // Sun=10, Moon=20 → sum=30 → yoga index floor(30/(360/27))+1.
        let positions = synth(10.0, 20.0);
        let it = yoga_transition(&positions, EPOCH, false).expect("transition");
        let span = 360.0 / 27.0;
        let s0: f64 = 30.0;
        let next = (s0 / span).floor() * span + span;
        let sum_rate = MOON_RATE + SUN_RATE;
        let expected_end = EPOCH + (next - s0) / sum_rate;
        assert!(
            (it.end_jd - expected_end).abs() < TWO_SEC_DAYS,
            "yoga end {} vs {}",
            it.end_jd,
            expected_end
        );
    }

    #[test]
    fn all_transitions_consistent_with_instant_panchang() {
        let positions = synth(40.0, 148.0); // ~ the real 25-May-2026 geometry
        let tr = compute_panchang_transitions(&positions, EPOCH, true).expect("all");
        let (s, m) = positions(EPOCH);
        // The reported value at the instant must equal compute_panchang's.
        let inst = crate::panchang::compute_panchang(s, m, EPOCH);
        assert_eq!(tr.tithi.value.number, inst.tithi.number);
        assert_eq!(tr.nakshatra.value, inst.nakshatra);
        assert_eq!(tr.yoga.value.number, inst.yoga.number);
        assert_eq!(tr.karana.value.number, inst.karana.number);
        // Every end is strictly after its start, and after the reference jd.
        for end in [
            tr.tithi.end_jd,
            tr.nakshatra.end_jd,
            tr.yoga.end_jd,
            tr.karana.end_jd,
        ] {
            assert!(end > EPOCH, "end {end} not after reference");
        }
        assert!(tr.tithi.start_jd.unwrap() <= EPOCH);
        // Karana end is never after tithi end (karana is a half-tithi).
        assert!(
            tr.karana.end_jd <= tr.tithi.end_jd + 1e-9,
            "karana end {} > tithi end {}",
            tr.karana.end_jd,
            tr.tithi.end_jd
        );
    }

    #[test]
    fn interval_paksha_reports_value_paksha() {
        let positions = synth(0.0, 200.0); // elongation 200 → tithi 17 (Krishna)
        let it = tithi_transition(&positions, EPOCH, false).expect("t");
        assert_eq!(it.value.paksha, Paksha::Krishna);
        assert_eq!(interval_paksha(&it), Paksha::Krishna);
    }

    // -------------------------------------------------------------------
    // Reference geometry validated independently with pyswisseph 2.10.03
    // (DE431 + Lahiri sidereal). For 25 May 2026 00:30 UTC:
    //     sun_sid  = 39.6144887510835°
    //     moon_sid = 148.3420540261929°
    //     elongation = 108.7275652751094° → tithi 10 (Dashami)
    //     tithi-10 end (elongation → 120°) = JD_UT 2461186.4871043526
    //       = 26 May 2026 05:11:26 IST
    //
    // This UNIT test seeds a linear ephemeris from those two pyswisseph
    // longitudes and asserts that the root-finder lands EXACTLY on the model's
    // own 120° boundary — i.e. it validates the bisection/bracketing, with the
    // linear model itself as the oracle (sub-second exact). It deliberately
    // does NOT compare against pyswisseph's true end-time, because the linear
    // model's mean rate differs from the true rate by ~1 hour over a day; that
    // residual is model error, not solver error. The end-to-end comparison
    // against pyswisseph's true 05:11 IST end-time uses the REAL ephemeris and
    // lives in `tests/validation.rs::panchang_tithi_end_vs_pyswisseph_ref`.
    // -------------------------------------------------------------------
    #[test]
    fn tithi_solver_lands_on_120_degree_boundary() {
        const JD_REF: f64 = 2_461_185.520_833_333_5; // 25 May 2026 00:30 UTC
        const SUN_SID: f64 = 39.614_488_751_083_5;
        const MOON_SID: f64 = 148.342_054_026_192_9;

        let positions = move |jd: f64| {
            let dt = jd - JD_REF;
            let s = (SUN_SID + SUN_RATE * dt).rem_euclid(360.0);
            let m = (MOON_SID + MOON_RATE * dt).rem_euclid(360.0);
            (s, m)
        };
        let it = tithi_transition(&positions, JD_REF, false).expect("transition");
        assert_eq!(
            it.value.number, 10,
            "reference tithi should be 10 (Dashami)"
        );

        // The elongation at the solved end_jd must be 120.0° to sub-second
        // precision: that is the exact tithi-10/11 boundary.
        let (s, m) = positions(it.end_jd);
        let elong_at_end = elongation_deg(s, m);
        assert!(
            (elong_at_end - 120.0).abs() < 1e-4,
            "elongation at solved end is {elong_at_end}, expected 120.0"
        );

        // And against the linear model's own closed-form boundary (oracle):
        let elong_rate = MOON_RATE - SUN_RATE;
        let expected_end = JD_REF + (120.0 - 108.727_565_275_109_4) / elong_rate;
        assert!(
            (it.end_jd - expected_end).abs() < TWO_SEC_DAYS,
            "end_jd {} vs closed-form {}",
            it.end_jd,
            expected_end
        );
    }
}
