//! Real UTC ↔ TAI ↔ TT ↔ UT1 layer.
//!
//! This module closes the largest time-scale gap against Swiss Ephemeris'
//! `swe_utc_to_jd`: it treats UTC as a genuine, leap-second-bearing time scale
//! rather than the previous `UTC ≈ UT1` approximation.
//!
//! # The chain
//! ```text
//!   UTC ──(+ (TAI−UTC) leap seconds)──▶ TAI ──(+ 32.184 s)──▶ TT
//!   UTC ──(+ DUT1, |DUT1| < 0.9 s)─────────────────────────▶ UT1
//! ```
//! * TAI−UTC comes from the IERS integer leap-second table
//!   ([`crate::leap_seconds`]).
//! * TT−TAI is the defined constant 32.184 s ([`TimeScale::TAI_TT_OFFSET_SECONDS`]).
//! * UT1−UTC is `DUT1`, a small (sub-second) observed quantity broadcast in IERS
//!   Bulletin A/B. It is supplied by the caller (it cannot be predicted in
//!   closed form); passing `0.0` yields the `UTC ≈ UT1` legacy behaviour.
//!
//! # Newtypes
//! [`JdUTC`] and [`JdTAI`] are the two scales that previously had only the dead
//! [`TimeScale`] enum variants. They wrap a Julian Date that is the *plain*
//! calendar→JD conversion of the clock reading on that scale (i.e. each scale
//! counts its own seconds; the leap-second jumps live in the *conversions*, not
//! inside a single scale's running JD).
//!
//! # Validation
//! `utc_to_tt` was checked to machine precision against pyswisseph
//! `swe.utc_to_jd(...)` at 1972-01-01, 1985-07-01, J2000, 2006-01-01, a 1990
//! Pune instant and 2017-01-01 (see `tests/utc_crossval.rs`).

use crate::SECONDS_PER_DAY;
use crate::calendar::{CalendarSystem, calendar_to_jd};
use crate::julian::{JdTT, JdUT1};
use crate::leap_seconds::{tai_minus_utc_seconds, utc_day_length_seconds};
use crate::timescale::TimeScale;
use serde::{Deserialize, Serialize};

/// Julian Date on the **Coordinated Universal Time (UTC)** scale.
///
/// The wrapped `f64` is the plain calendar→JD value of the UTC clock reading
/// (leap seconds are *not* folded into this number; they appear only when
/// converting to a continuous scale such as TAI/TT). Two UTC clock readings one
/// civil second apart differ by `1/86400` here even across a leap second — the
/// extra physical second is materialised in the UTC→TAI conversion.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct JdUTC(pub f64);

/// Julian Date on the **International Atomic Time (TAI)** scale — a continuous
/// count with no leap seconds.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct JdTAI(pub f64);

impl JdUTC {
    /// Raw JD value (plain UTC clock reading as a JD).
    #[must_use]
    pub fn as_f64(self) -> f64 {
        self.0
    }

    /// UTC → TAI: add the integer leap-second offset `TAI−UTC` in force at this
    /// instant. Continuous, monotonic across leap-second boundaries.
    #[must_use]
    pub fn to_tai(self) -> JdTAI {
        let offset = f64::from(tai_minus_utc_seconds(self.0));
        JdTAI(self.0 + offset / SECONDS_PER_DAY)
    }

    /// UTC → TT, via TAI. Equals `swe_utc_to_jd(...)[0]` to machine precision
    /// for every ordinary (non-leap-second) UTC instant.
    ///
    /// # Leap seconds
    /// A `JdUTC` is a continuous Julian Date and therefore *cannot* by itself
    /// distinguish the inserted leap second `23:59:60` from the ordinary
    /// `23:59:59` one second earlier — both alias onto the same JD band just
    /// before the step. This method applies the position-based leap table, so on
    /// a leap-second JD it returns the **pre-step** TT (the `:59`-equivalent). To
    /// get the leap-second-correct TT (one SI second later), use
    /// [`JdUTC::to_tt_for_leap_second`], or build the epoch with
    /// [`crate::Epoch::from_utc`] / [`utc_calendar_to_jd`], which carry the leap
    /// flag explicitly.
    #[must_use]
    pub fn to_tt(self) -> JdTT {
        self.to_tai().to_tt()
    }

    /// UTC → TT for an instant that is the **inserted leap second** `23:59:60.f`.
    ///
    /// The leap second carries the *post-step* TAI−UTC offset (it belongs to the
    /// new regime), one SI second beyond what the position-based table assigns to
    /// its aliased clock position. This adds that one second, reproducing Swiss
    /// `swe_utc_to_jd` for the leap label (exact at `:60.0` and `:60.999`,
    /// matching Swiss to ~µs in between). Only meaningful for a `JdUTC` produced
    /// by [`utc_calendar_to_jd`] with its `is_leap` flag set; using it on an
    /// ordinary instant would overstate TT by one second.
    #[must_use]
    pub fn to_tt_for_leap_second(self) -> JdTT {
        let tt = self.to_tt();
        JdTT(tt.0 + 1.0 / SECONDS_PER_DAY)
    }

    /// UTC → UT1 given the observed `dut1 = UT1 − UTC` in **seconds**
    /// (IERS Bulletin A/B; |DUT1| < 0.9 s by construction). Passing `0.0`
    /// reproduces the legacy `UTC ≈ UT1` approximation exactly.
    #[must_use]
    pub fn to_ut1(self, dut1_seconds: f64) -> JdUT1 {
        JdUT1(self.0 + dut1_seconds / SECONDS_PER_DAY)
    }

    /// The integer `TAI − UTC` (leap-second count) in force at this instant.
    #[must_use]
    pub fn leap_seconds(self) -> i32 {
        tai_minus_utc_seconds(self.0)
    }
}

impl JdTAI {
    /// Raw JD value.
    #[must_use]
    pub fn as_f64(self) -> f64 {
        self.0
    }

    /// TAI → TT: add the defined constant 32.184 s.
    #[must_use]
    pub fn to_tt(self) -> JdTT {
        JdTT(self.0 + TimeScale::TAI_TT_OFFSET_SECONDS / SECONDS_PER_DAY)
    }

    /// TAI → UTC: subtract the leap-second offset. Because the offset is keyed by
    /// UTC, this inverts [`JdUTC::to_tai`] by first estimating the UTC instant
    /// and re-evaluating the table; a single re-evaluation is exact except within
    /// the one-second window of a positive leap second, where the inverse is
    /// inherently ambiguous (two UTC labels — 23:59:60 and 00:00:00 — map to
    /// adjacent TAI instants). We resolve that window to the *later* (post-step)
    /// UTC label, matching the convention that a boundary instant takes the new
    /// offset.
    #[must_use]
    pub fn to_utc(self) -> JdUTC {
        // First guess using the offset at the TAI instant interpreted as UTC.
        let guess_offset = f64::from(tai_minus_utc_seconds(self.0));
        let utc_guess = self.0 - guess_offset / SECONDS_PER_DAY;
        // Re-evaluate the table at the estimated UTC instant and correct once.
        let offset = f64::from(tai_minus_utc_seconds(utc_guess));
        JdUTC(self.0 - offset / SECONDS_PER_DAY)
    }
}

/// Build a [`JdUTC`] from a UTC calendar date and time-of-day fields, correctly
/// handling a `second` value of `60` during an inserted leap second.
///
/// `hour`/`minute` are whole, `second` is fractional and may reach `< 61.0` only
/// at 23:59 on the last day of a leap-second month (e.g. 2016-12-31 23:59:60).
/// In every other case `second` must be `< 60.0`.
///
/// # Leap-second mapping
/// A genuine 23:59:60 UTC label has no place on a plain 86400-second day. We
/// place it on the UTC clock timeline as the **final SI second before the
/// following midnight**: `:60.0` sits exactly one second before the next
/// midnight, `:60.f` `f` seconds into that interval. The returned [`JdUTC`]
/// therefore aliases the ordinary `23:59:59` band and is *below* the leap step;
/// callers that need the leap-second-correct TT (post-step offset, +1 SI second)
/// must use [`JdUTC::to_tt_for_leap_second`] (or [`crate::Epoch::from_utc`],
/// which does so automatically). With that correction the TT matches Swiss
/// `swe_utc_to_jd` to ~µs and is continuous and monotonic across the insertion
/// (`:59 < :60 < 00:00`).
///
/// Returns the [`JdUTC`] and a `bool` that is `true` when the input named a
/// real inserted leap second (i.e. `60.0 ≤ second < 61.0` at 23:59 on the last
/// day of a leap-second month). A `:60` outside such an instant is `false` and
/// is still placed on the clock timeline, but it is not a physical leap second.
#[must_use]
pub fn utc_calendar_to_jd(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: f64,
) -> (JdUTC, bool) {
    // The "plain" branch: treat seconds as an ordinary fraction of an 86400 s
    // day. Valid for every non-leap second.
    if second < 60.0 {
        let frac_hour = f64::from(hour) + f64::from(minute) / 60.0 + second / 3600.0;
        let jd = calendar_to_jd(
            year,
            month,
            day,
            frac_hour,
            CalendarSystem::ProlepticGregorian,
        )
        .0;
        return (JdUTC(jd), false);
    }

    // second >= 60.0 → only legal as 23:59:60 on the LAST day of a leap month.
    // `utc_day_length_seconds` returns 86401 only when `day` is the final day of
    // the month AND that month carries an inserted leap second, so it folds both
    // the last-day check and the leap-month check into one test. Without the
    // last-day guard a `:60` on any other day of a leap month (e.g.
    // 2016-12-01 23:59:60 or 2016-12-30 23:59:60) would be wrongly accepted as a
    // physical leap second and earn the +1 s TT step.
    let is_real_leap = hour == 23
        && minute == 59
        && second < 61.0
        && utc_day_length_seconds(year, month, day) == SECONDS_PER_DAY + 1.0;

    // Place the leap second on the UTC clock timeline: it occupies the final
    // SI second before the following midnight. `next_midnight` is that step
    // instant; `:60.0` sits exactly one second earlier and `:60.f` `f` seconds
    // into that interval. The returned `JdUTC` is therefore strictly below the
    // step (it aliases the `23:59:59` band — see `JdUTC::to_tt`). The
    // leap-second-correct TT (which uses the post-step offset, +1 s) is obtained
    // via `JdUTC::to_tt_for_leap_second` when `is_real_leap` is true.
    let next_midnight =
        calendar_to_jd(year, month, day, 24.0, CalendarSystem::ProlepticGregorian).0;
    let frac = (second - 60.0).clamp(0.0, 1.0); // 0.0 at :60.0, →1.0 at :61.0
    let jd = next_midnight - (1.0 - frac) / SECONDS_PER_DAY;
    (JdUTC(jd), is_real_leap)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MS_PER_DAY: f64 = SECONDS_PER_DAY * 1000.0;

    fn jd_utc_plain(y: i32, mo: u32, d: u32, h: u32, mi: u32, s: f64) -> f64 {
        let frac = f64::from(h) + f64::from(mi) / 60.0 + s / 3600.0;
        calendar_to_jd(y, mo, d, frac, CalendarSystem::ProlepticGregorian).0
    }

    #[test]
    fn utc_to_tai_to_tt_chain() {
        // 2017-01-01 00:00 UTC: TAI−UTC = 37, TT−TAI = 32.184.
        let utc = JdUTC(jd_utc_plain(2017, 1, 1, 0, 0, 0.0));
        let tai = utc.to_tai();
        let tt = utc.to_tt();
        // Offsets recovered by subtracting two large (~2.46e6) f64 Julian Dates
        // are precision-limited to ~50 µs (f64 ULP at this magnitude); 1 ms still
        // catches any real bug — a wrong leap-second count is a full ±1 s.
        assert!(((tai.0 - utc.0) * SECONDS_PER_DAY - 37.0).abs() < 1e-3);
        assert!(((tt.0 - utc.0) * SECONDS_PER_DAY - (37.0 + 32.184)).abs() < 1e-3);
        // to_tt must equal to_tai().to_tt() exactly.
        assert_eq!(tt.0, tai.to_tt().0);
    }

    #[test]
    fn utc_to_ut1_uses_dut1() {
        let utc = JdUTC(2_457_754.5);
        // DUT1 = 0 → legacy UTC≈UT1.
        assert_eq!(utc.to_ut1(0.0).0, utc.0);
        // DUT1 = -0.5 s → UT1 is half a second behind UTC.
        let ut1 = utc.to_ut1(-0.5);
        // ~50 µs f64-JD floor at this epoch (large-JD subtraction); 1 ms catches real bugs.
        assert!(((ut1.0 - utc.0) * SECONDS_PER_DAY + 0.5).abs() < 1e-3);
    }

    #[test]
    fn tai_utc_roundtrip() {
        for jd in [
            2_441_317.5,
            2_451_545.0,
            2_453_736.5,
            2_457_754.5,
            2_460_000.0,
        ] {
            let utc = JdUTC(jd);
            let back = utc.to_tai().to_utc();
            assert!(
                (back.0 - utc.0).abs() * MS_PER_DAY < 1e-3,
                "TAI roundtrip failed at {jd}: {} vs {}",
                back.0,
                utc.0
            );
        }
    }

    #[test]
    fn leap_second_label_is_handled() {
        // 2016-12-31 23:59:60 is a real leap second.
        let (jd, is_leap) = utc_calendar_to_jd(2016, 12, 31, 23, 59, 60.0);
        assert!(
            is_leap,
            ":60 on 2016-12-31 must be flagged as a real leap second"
        );
        // :60.0 sits exactly one SI second before the next midnight on the UTC
        // clock timeline (and thus strictly below the leap step).
        let next_midnight = jd_utc_plain(2017, 1, 1, 0, 0, 0.0);
        // Tolerance 1e-4 s: a sub-second offset in a ~2.45e6 JD is f64-limited to
        // ≈5 µs; the placement is otherwise exact.
        let secs_before = (next_midnight - jd.0) * SECONDS_PER_DAY;
        assert!(
            (secs_before - 1.0).abs() < 1e-4,
            ":60.0 should sit 1 s before the next midnight, got {secs_before} s",
        );
        assert!(
            jd.0 < next_midnight,
            "leap-second JD must be below the step"
        );

        // TT must be continuous and strictly monotonic across the leap second
        // using the leap-correct conversion: 23:59:59 < 23:59:60 < 00:00:00.
        let before = utc_calendar_to_jd(2016, 12, 31, 23, 59, 59.0).0.to_tt().0;
        let leap = jd.to_tt_for_leap_second().0; // post-step offset (+1 s)
        let after = JdUTC(next_midnight).to_tt().0;
        assert!(
            before < leap,
            "TT not monotonic: before {before} !< leap {leap}"
        );
        assert!(
            leap <= after,
            "TT not monotonic: leap {leap} !<= after {after}"
        );
        // The three are spaced ≈1 SI second apart in TT (to f64 precision).
        assert!(((leap - before) * SECONDS_PER_DAY - 1.0).abs() < 1e-4);
        assert!(((after - leap) * SECONDS_PER_DAY - 1.0).abs() < 1e-4);
    }

    #[test]
    fn sixty_seconds_outside_leap_month_not_flagged() {
        // :60 on a date with no leap second is NOT a real leap label.
        let (_, is_leap) = utc_calendar_to_jd(2018, 6, 30, 23, 59, 60.0);
        assert!(!is_leap, ":60 on 2018-06-30 is not a real leap second");
    }

    #[test]
    fn sixty_seconds_on_non_last_day_of_leap_month_not_flagged() {
        // Regression: December 2016 ends with a leap second, but the leap second
        // belongs ONLY to its last day (the 31st). A `:60` label on any earlier
        // day of that same month must NOT be treated as a physical leap second —
        // otherwise `Epoch::from_utc` would wrongly apply the +1 s TT step.
        let (_, is_leap_30) = utc_calendar_to_jd(2016, 12, 30, 23, 59, 60.0);
        assert!(
            !is_leap_30,
            ":60 on 2016-12-30 (not month end) is not a real leap second"
        );
        let (_, is_leap_01) = utc_calendar_to_jd(2016, 12, 1, 23, 59, 60.0);
        assert!(
            !is_leap_01,
            ":60 on 2016-12-01 (not month end) is not a real leap second"
        );

        // The genuine instant on the LAST day of the same month is still flagged.
        let (_, is_leap_31) = utc_calendar_to_jd(2016, 12, 31, 23, 59, 60.0);
        assert!(
            is_leap_31,
            ":60 on 2016-12-31 (month end) must remain a real leap second"
        );
    }

    #[test]
    fn plain_seconds_path() {
        // An ordinary instant: second < 60 → plain conversion, not flagged.
        let (jd, is_leap) = utc_calendar_to_jd(1990, 1, 15, 5, 0, 0.0);
        assert!(!is_leap);
        assert!((jd.0 - jd_utc_plain(1990, 1, 15, 5, 0, 0.0)).abs() < 1e-12);
    }
}
