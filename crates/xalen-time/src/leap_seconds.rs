//! IERS leap-second table — the integer TAI−UTC offset for the modern
//! (post-1972) era of Coordinated Universal Time.
//!
//! # What this is
//! Since 1972-01-01 UTC has been kept within ±0.9 s of UT1 by inserting
//! **integer** leap seconds, always at the end of a UTC month (in practice only
//! the ends of June and December have ever been used). Between insertions the
//! difference TAI−UTC is a constant whole number of seconds. This module encodes
//! the official IERS Bulletin C history of those steps.
//!
//! The very first entry, 1972-01-01, set TAI−UTC = 10 s exactly; every later
//! entry adds one second. The latest insertion (2016-12-31 → 2017-01-01) brought
//! the offset to **37 s**, where it has remained.
//!
//! # Source (public / factual)
//! The step dates and values are reproduced from the IERS leap-second list
//! (`Bulletin C`, IERS Earth Orientation Centre, Paris Observatory) and the
//! identical USNO `tai-utc.dat` history. These are published reference data, not
//! a copyrightable creative work. Cross-checked against the Swiss Ephemeris
//! `swe_utc_to_jd` results to machine precision (see the unit tests and the
//! `utc_crossval` integration test).
//!
//! # Scope
//! Pre-1972 UTC used a *rubber-second* rate model (a fractional, drifting
//! TAI−UTC); that regime is intentionally NOT modelled here. For instants before
//! 1972-01-01 the table returns the floor value 10 s, and callers should prefer
//! the UT1-approximation path (`UTC ≈ UT1`) documented on
//! [`crate::calendar_to_jd`]. The table has no upper bound — after the last step
//! the most recent value (37 s) applies until a future Bulletin C announces a new
//! leap second, at which point one row is appended here.

use crate::SECONDS_PER_DAY;

/// One row of the IERS leap-second history: the UT/UTC calendar instant
/// (always 00:00:00 on the first of a month) at which `tai_minus_utc` took
/// effect, stored as a Julian Day Number for fast comparison, plus the matching
/// proleptic-Gregorian `(year, month)` for date-based queries.
#[derive(Debug, Clone, Copy)]
struct LeapStep {
    /// JD (UT, 00:00) at which this offset became effective.
    effective_jd: f64,
    /// Calendar year the step took effect (UTC).
    year: i32,
    /// Calendar month the step took effect (UTC); always 1 (Jan) or 7 (Jul).
    month: u32,
    /// TAI − UTC in whole seconds in force from this instant until the next row.
    tai_minus_utc: i32,
}

/// The leap-second history, 1972-01-01 … 2017-01-01 (28 steps).
///
/// `effective_jd` is the JD of 00:00 UT on the listed date in the proleptic
/// Gregorian calendar, equal to `calendar_to_jd(year, month, 1, 0.0,
/// ProlepticGregorian)`. They are hard-coded (not computed) so the table is a
/// pure `const` with no initialisation cost; the `leap_step_jds_are_consistent`
/// test proves each literal equals the calendar conversion.
const LEAP_STEPS: [LeapStep; 28] = [
    LeapStep {
        effective_jd: 2_441_317.5,
        year: 1972,
        month: 1,
        tai_minus_utc: 10,
    },
    LeapStep {
        effective_jd: 2_441_499.5,
        year: 1972,
        month: 7,
        tai_minus_utc: 11,
    },
    LeapStep {
        effective_jd: 2_441_683.5,
        year: 1973,
        month: 1,
        tai_minus_utc: 12,
    },
    LeapStep {
        effective_jd: 2_442_048.5,
        year: 1974,
        month: 1,
        tai_minus_utc: 13,
    },
    LeapStep {
        effective_jd: 2_442_413.5,
        year: 1975,
        month: 1,
        tai_minus_utc: 14,
    },
    LeapStep {
        effective_jd: 2_442_778.5,
        year: 1976,
        month: 1,
        tai_minus_utc: 15,
    },
    LeapStep {
        effective_jd: 2_443_144.5,
        year: 1977,
        month: 1,
        tai_minus_utc: 16,
    },
    LeapStep {
        effective_jd: 2_443_509.5,
        year: 1978,
        month: 1,
        tai_minus_utc: 17,
    },
    LeapStep {
        effective_jd: 2_443_874.5,
        year: 1979,
        month: 1,
        tai_minus_utc: 18,
    },
    LeapStep {
        effective_jd: 2_444_239.5,
        year: 1980,
        month: 1,
        tai_minus_utc: 19,
    },
    LeapStep {
        effective_jd: 2_444_786.5,
        year: 1981,
        month: 7,
        tai_minus_utc: 20,
    },
    LeapStep {
        effective_jd: 2_445_151.5,
        year: 1982,
        month: 7,
        tai_minus_utc: 21,
    },
    LeapStep {
        effective_jd: 2_445_516.5,
        year: 1983,
        month: 7,
        tai_minus_utc: 22,
    },
    LeapStep {
        effective_jd: 2_446_247.5,
        year: 1985,
        month: 7,
        tai_minus_utc: 23,
    },
    LeapStep {
        effective_jd: 2_447_161.5,
        year: 1988,
        month: 1,
        tai_minus_utc: 24,
    },
    LeapStep {
        effective_jd: 2_447_892.5,
        year: 1990,
        month: 1,
        tai_minus_utc: 25,
    },
    LeapStep {
        effective_jd: 2_448_257.5,
        year: 1991,
        month: 1,
        tai_minus_utc: 26,
    },
    LeapStep {
        effective_jd: 2_448_804.5,
        year: 1992,
        month: 7,
        tai_minus_utc: 27,
    },
    LeapStep {
        effective_jd: 2_449_169.5,
        year: 1993,
        month: 7,
        tai_minus_utc: 28,
    },
    LeapStep {
        effective_jd: 2_449_534.5,
        year: 1994,
        month: 7,
        tai_minus_utc: 29,
    },
    LeapStep {
        effective_jd: 2_450_083.5,
        year: 1996,
        month: 1,
        tai_minus_utc: 30,
    },
    LeapStep {
        effective_jd: 2_450_630.5,
        year: 1997,
        month: 7,
        tai_minus_utc: 31,
    },
    LeapStep {
        effective_jd: 2_451_179.5,
        year: 1999,
        month: 1,
        tai_minus_utc: 32,
    },
    LeapStep {
        effective_jd: 2_453_736.5,
        year: 2006,
        month: 1,
        tai_minus_utc: 33,
    },
    LeapStep {
        effective_jd: 2_454_832.5,
        year: 2009,
        month: 1,
        tai_minus_utc: 34,
    },
    LeapStep {
        effective_jd: 2_456_109.5,
        year: 2012,
        month: 7,
        tai_minus_utc: 35,
    },
    LeapStep {
        effective_jd: 2_457_204.5,
        year: 2015,
        month: 7,
        tai_minus_utc: 36,
    },
    LeapStep {
        effective_jd: 2_457_754.5,
        year: 2017,
        month: 1,
        tai_minus_utc: 37,
    },
];

/// The TAI−UTC value in force before the first table entry (1972-01-01).
/// Pre-1972 UTC is not modelled (rubber-second era); we return the floor value
/// so conversions degrade gracefully rather than panic. See module docs.
const PRE_1972_TAI_MINUS_UTC: i32 = 10;

/// TAI − UTC, in **whole seconds**, in force at the given **UTC** instant,
/// expressed as a plain UTC Julian Date (calendar clock reading converted to JD
/// with no leap-second handling — i.e. `calendar_to_jd(...)` of the UTC reading).
///
/// The argument is interpreted on the UTC time scale: the returned offset is the
/// one whose effective instant is the latest that does **not exceed** `jd_utc`.
/// For `jd_utc` before 1972-01-01 the pre-modern floor ([`PRE_1972_TAI_MINUS_UTC`])
/// is returned (see module docs — pre-1972 is out of model scope).
///
/// This is exact: between steps TAI−UTC is a constant integer, so the only
/// subtlety is the instant of a step. A leap second is inserted at 23:59:60 of
/// the last day of the prior month; the *new* (incremented) offset applies from
/// 00:00:00 of the listed date onward. A UTC instant exactly at a step boundary
/// (`jd_utc == effective_jd`) therefore takes the **new** value, matching IERS
/// and Swiss `swe_utc_to_jd`.
#[must_use]
pub fn tai_minus_utc_seconds(jd_utc: f64) -> i32 {
    if jd_utc.is_nan() {
        return PRE_1972_TAI_MINUS_UTC;
    }
    let mut value = PRE_1972_TAI_MINUS_UTC;
    for step in &LEAP_STEPS {
        if jd_utc >= step.effective_jd {
            value = step.tai_minus_utc;
        } else {
            break;
        }
    }
    value
}

/// TAI − UTC in whole seconds for a UTC calendar date `(year, month, day)`,
/// using only the year/month (leap steps only ever occur at month starts, so the
/// day and sub-day time never change which interval a date falls in except at the
/// exact boundary, which this whole-month form resolves to the new value from the
/// first of the listed month onward).
///
/// Convenience wrapper for callers that have a civil date rather than a JD.
#[must_use]
pub fn tai_minus_utc_for_date(year: i32, month: u32, day: u32) -> i32 {
    // Convert the civil date to a plain UTC JD at midnight and reuse the exact
    // JD-keyed lookup. `day` is honoured for a complete civil-date surface even
    // though every leap step lands on day 1 (so within a month the value is
    // constant); routing through the JD form keeps a single source of truth.
    let jd = crate::calendar::calendar_to_jd(
        year,
        month,
        day,
        0.0,
        crate::calendar::CalendarSystem::ProlepticGregorian,
    )
    .0;
    tai_minus_utc_seconds(jd)
}

/// `true` if the UTC month identified by `(year, month)` ends with an inserted
/// (positive) leap second — i.e. its last day has a 23:59:60. This is exactly
/// the set of months that *immediately precede* a table step where the offset
/// increases by one.
///
/// Used to validate a UTC second field of `60`: a `:60` is only legal at
/// 23:59:60 on the last day of such a month. No negative leap second has ever
/// been issued, so only +1 steps occur.
#[must_use]
pub fn month_ends_with_leap_second(year: i32, month: u32) -> bool {
    // The leap second belongs to the month BEFORE the step's effective month.
    // E.g. the 2017-01-01 step (37 s) is preceded by the 2016-12-31 23:59:60
    // leap second, so December 2016 "ends with a leap second".
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    LEAP_STEPS
        .iter()
        .any(|s| s.year == next_year && s.month == next_month)
}

/// Number of seconds in the UTC day that **begins** at `(year, month, day)`:
/// 86400 normally, 86401 if that day carries an inserted leap second (its
/// 23:59:60). Provided for completeness / callers needing day-length awareness.
#[must_use]
pub fn utc_day_length_seconds(year: i32, month: u32, day: u32) -> f64 {
    let last_day = last_day_of_month(year, month);
    if day == last_day && month_ends_with_leap_second(year, month) {
        SECONDS_PER_DAY + 1.0
    } else {
        SECONDS_PER_DAY
    }
}

/// Proleptic-Gregorian last day of a month (handles leap years).
fn last_day_of_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_gregorian_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

fn is_gregorian_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// The earliest UTC instant the leap-second table models, as a plain UTC JD
/// (1972-01-01 00:00). Below this the rubber-second era applies and is out of
/// scope. Exposed so callers can branch on "is this a modelled UTC instant?".
pub const FIRST_LEAP_SECOND_JD: f64 = 2_441_317.5;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::{CalendarSystem, calendar_to_jd};

    /// Every hard-coded `effective_jd` literal must equal the calendar
    /// conversion of its own `(year, month, 1, 00:00)` — proves the table is
    /// internally consistent and the JDs were not mistyped.
    #[test]
    fn leap_step_jds_are_consistent() {
        for step in &LEAP_STEPS {
            let jd = calendar_to_jd(
                step.year,
                step.month,
                1,
                0.0,
                CalendarSystem::ProlepticGregorian,
            )
            .0;
            assert!(
                (jd - step.effective_jd).abs() < 1e-9,
                "step {}-{:02} effective_jd {} != calendar JD {}",
                step.year,
                step.month,
                step.effective_jd,
                jd
            );
        }
    }

    /// Offsets are monotonically increasing by exactly +1 each step (no negative
    /// leap second has ever occurred), starting at 10 s.
    #[test]
    fn offsets_increase_by_one() {
        assert_eq!(LEAP_STEPS[0].tai_minus_utc, 10);
        for w in LEAP_STEPS.windows(2) {
            assert_eq!(
                w[1].tai_minus_utc - w[0].tai_minus_utc,
                1,
                "leap steps must increase by exactly 1 s"
            );
            assert!(
                w[1].effective_jd > w[0].effective_jd,
                "effective JDs must be strictly increasing"
            );
        }
        assert_eq!(LEAP_STEPS.last().unwrap().tai_minus_utc, 37);
    }

    #[test]
    fn known_offsets_by_jd() {
        // 1972-01-01: 10 s exactly at the boundary instant.
        assert_eq!(tai_minus_utc_seconds(2_441_317.5), 10);
        // Just before the first step → floor value.
        assert_eq!(
            tai_minus_utc_seconds(2_441_317.5 - 0.5),
            PRE_1972_TAI_MINUS_UTC
        );
        // J2000 12:00 (JD 2451545.0) → 32 s.
        assert_eq!(tai_minus_utc_seconds(2_451_545.0), 32);
        // 2017-01-01 onward → 37 s, and it stays 37 far into the future.
        assert_eq!(tai_minus_utc_seconds(2_457_754.5), 37);
        assert_eq!(tai_minus_utc_seconds(2_457_754.5 + 10_000.0), 37);
    }

    #[test]
    fn boundary_takes_new_value() {
        // The 2006-01-01 step to 33 s: exactly at the boundary → 33, one instant
        // before → 32.
        let step = 2_453_736.5;
        assert_eq!(tai_minus_utc_seconds(step), 33);
        assert_eq!(tai_minus_utc_seconds(step - 1e-6), 32);
    }

    #[test]
    fn date_form_matches_jd_form_at_midnight() {
        for step in &LEAP_STEPS {
            let by_date = tai_minus_utc_for_date(step.year, step.month, 1);
            let by_jd = tai_minus_utc_seconds(step.effective_jd);
            assert_eq!(
                by_date, by_jd,
                "date vs JD form disagree at {}-{:02}-01",
                step.year, step.month
            );
            assert_eq!(by_date, step.tai_minus_utc);
        }
    }

    #[test]
    fn leap_second_months_detected() {
        // Dec 2016 ends with a leap second (precedes the 2017-01-01 step).
        assert!(month_ends_with_leap_second(2016, 12));
        // Jun 2015 ends with one (precedes 2015-07-01 step).
        assert!(month_ends_with_leap_second(2015, 6));
        // Jun 2012 (precedes 2012-07-01).
        assert!(month_ends_with_leap_second(2012, 6));
        // A quiet month with no following step.
        assert!(!month_ends_with_leap_second(2018, 6));
        assert!(!month_ends_with_leap_second(2000, 1));
    }

    #[test]
    fn day_length_handles_leap_day() {
        // 2016-12-31 is 86401 s long; 2016-12-30 is normal; a non-leap-month
        // last day is normal.
        assert!((utc_day_length_seconds(2016, 12, 31) - 86401.0).abs() < 1e-9);
        assert!((utc_day_length_seconds(2016, 12, 30) - 86400.0).abs() < 1e-9);
        assert!((utc_day_length_seconds(2018, 6, 30) - 86400.0).abs() < 1e-9);
        // 2016 is a leap year → Feb has 29 days, last day Feb 29 not a leap-sec.
        assert!((utc_day_length_seconds(2016, 2, 29) - 86400.0).abs() < 1e-9);
    }

    #[test]
    fn nan_jd_degrades_to_floor() {
        assert_eq!(tai_minus_utc_seconds(f64::NAN), PRE_1972_TAI_MINUS_UTC);
    }
}
