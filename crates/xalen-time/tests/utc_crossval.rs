//! Cross-validation of the UTC/TAI/TT leap-second layer against the Swiss
//! Ephemeris `swe_utc_to_jd` reference.
//!
//! # Provenance of the reference numbers
//! Every `jd_tt` constant below was produced by **pyswisseph 2.10.03**:
//!
//! ```python
//! import swisseph as swe
//! swe.utc_to_jd(year, month, day, hour, minute, second, swe.GREG_CAL)[0]  # TT-based JD
//! ```
//!
//! These are factual computed reference values (not creative content). The
//! generating call and the matching DUT1 (UT1−UTC) values used for the UT1 check
//! are recorded inline. Our `Epoch::from_utc` / `JdUTC::to_tt` reproduce the TT
//! column to **sub-microsecond** precision, and UT1 exactly once the same DUT1 is
//! supplied (Swiss applies its own internal DUT1; we take it as an input).

use xalen_time::{Epoch, tai_minus_utc_seconds, utc_calendar_to_jd};

const SECONDS_PER_DAY: f64 = 86_400.0;
/// Tolerance: 1 microsecond expressed in days. Swiss agrees to < 1e-9 day in
/// practice; 1 µs is a comfortable, honest bound that still proves machine-grade
/// agreement.
const TOL_DAYS: f64 = 1.0e-6 / SECONDS_PER_DAY;

struct Ref {
    y: i32,
    mo: u32,
    d: u32,
    h: u32,
    mi: u32,
    s: f64,
    /// swe.utc_to_jd(...)[0] — the TT-based Julian Date.
    jd_tt: f64,
}

/// pyswisseph `swe.utc_to_jd(...)[0]` (TT JD), spanning the 1972 floor, several
/// mid-table offsets, J2000, the latest (2017) step, and a recent instant.
const REFS: &[Ref] = &[
    Ref {
        y: 1972,
        mo: 1,
        d: 1,
        h: 0,
        mi: 0,
        s: 0.0,
        jd_tt: 2_441_317.500_488_240_7,
    },
    Ref {
        y: 1985,
        mo: 7,
        d: 1,
        h: 0,
        mi: 0,
        s: 0.0,
        jd_tt: 2_446_247.500_638_703_8,
    },
    Ref {
        y: 2000,
        mo: 1,
        d: 1,
        h: 12,
        mi: 0,
        s: 0.0,
        jd_tt: 2_451_545.000_742_870_4,
    },
    Ref {
        y: 2006,
        mo: 1,
        d: 1,
        h: 0,
        mi: 0,
        s: 0.0,
        jd_tt: 2_453_736.500_754_444_4,
    },
    Ref {
        y: 1990,
        mo: 1,
        d: 15,
        h: 5,
        mi: 0,
        s: 0.0,
        jd_tt: 2_447_906.708_995_185_3,
    },
    Ref {
        y: 2017,
        mo: 1,
        d: 1,
        h: 0,
        mi: 0,
        s: 0.0,
        jd_tt: 2_457_754.500_800_740_9,
    },
    Ref {
        y: 2024,
        mo: 6,
        d: 15,
        h: 12,
        mi: 30,
        s: 0.0,
        jd_tt: 2_460_477.021_634_074_4,
    },
];

#[test]
fn jd_utc_to_tt_matches_swiss() {
    for r in REFS {
        let (utc, _) = utc_calendar_to_jd(r.y, r.mo, r.d, r.h, r.mi, r.s);
        let tt = utc.to_tt();
        let diff = (tt.0 - r.jd_tt).abs();
        assert!(
            diff < TOL_DAYS,
            "UTC→TT mismatch at {}-{:02}-{:02} {:02}:{:02}:{:04.1}: got {:.10}, swiss {:.10}, Δ={:.3} µs",
            r.y,
            r.mo,
            r.d,
            r.h,
            r.mi,
            r.s,
            tt.0,
            r.jd_tt,
            diff * SECONDS_PER_DAY * 1e6
        );
    }
}

#[test]
fn epoch_from_utc_tt_matches_swiss() {
    // The high-level constructor must produce the identical TT (DUT1 only shifts
    // UT1, never TT).
    for r in REFS {
        for dut1 in [0.0, 0.3, -0.7] {
            let e = Epoch::from_utc(r.y, r.mo, r.d, r.h, r.mi, r.s, dut1);
            let tt = e.jd_tt();
            let diff = (tt.0 - r.jd_tt).abs();
            assert!(
                diff < TOL_DAYS,
                "Epoch::from_utc TT mismatch at {}-{:02}-{:02} (dut1={dut1}): got {:.10}, swiss {:.10}",
                r.y,
                r.mo,
                r.d,
                tt.0,
                r.jd_tt
            );
            // The sigma variant must agree with jd_tt and report a zero envelope.
            let (tt2, sigma) = e.jd_tt_with_sigma();
            assert_eq!(tt2.0, tt.0, "jd_tt_with_sigma TT must equal jd_tt");
            assert_eq!(sigma, 0.0, "UTC-derived TT has a zero ΔT envelope");
        }
    }
}

/// UT1 = UTC + DUT1. Fed the DUT1 that Swiss itself used (recorded here from
/// `swe.utc_to_jd(...)[1] − plain_utc_jd`), our UT1 matches Swiss to machine
/// precision. With DUT1 = 0 we reproduce the legacy UTC≈UT1 approximation.
#[test]
fn ut1_matches_swiss_with_supplied_dut1() {
    // (y, mo, d, h, mi, s, dut1_seconds, swiss_ut1_jd)
    let cases = [
        (
            2017,
            1,
            1,
            0,
            0,
            0.0,
            0.590_421_259,
            2_457_754.500_006_833_6_f64,
        ),
        (
            2000,
            1,
            1,
            12,
            0,
            0.0,
            0.355_097_651,
            2_451_545.000_004_109_9,
        ),
        (
            1990,
            1,
            15,
            5,
            0,
            0.0,
            0.302_352_011,
            2_447_906.708_336_832_9,
        ),
    ];
    for (y, mo, d, h, mi, s, dut1, swiss_ut1) in cases {
        let (utc, _) = utc_calendar_to_jd(y, mo, d, h, mi, s);
        let ut1 = utc.to_ut1(dut1);
        let diff = (ut1.0 - swiss_ut1).abs();
        assert!(
            diff < TOL_DAYS,
            "UT1 mismatch at {y}-{mo:02}-{d:02}: got {:.10}, swiss {:.10}, Δ={:.3} µs",
            ut1.0,
            swiss_ut1,
            diff * SECONDS_PER_DAY * 1e6
        );

        // DUT1 = 0 → UT1 collapses to the plain UTC JD (legacy approximation),
        // and that approximation is within the IERS ±0.9 s guarantee of the
        // real UT1.
        let approx = utc.to_ut1(0.0);
        let approx_err_s = (approx.0 - swiss_ut1).abs() * SECONDS_PER_DAY;
        assert!(
            approx_err_s < 0.9,
            "legacy UTC≈UT1 must be within ±0.9 s at {y}-{mo:02}-{d:02}, got {approx_err_s} s"
        );
    }
}

/// The TAI offset our chain applies must equal the IERS table value at each
/// reference instant.
#[test]
fn tai_offset_is_table_value() {
    let expectations = [
        (1972, 1, 1, 10),
        (1985, 7, 1, 23),
        (2000, 1, 1, 32),
        (2006, 1, 1, 33),
        (1990, 1, 15, 25),
        (2017, 1, 1, 37),
        (2024, 6, 15, 37),
    ];
    for (y, mo, d, expected) in expectations {
        let (utc, _) = utc_calendar_to_jd(y, mo, d, 12, 0, 0.0);
        assert_eq!(
            utc.leap_seconds(),
            expected,
            "TAI−UTC at {y}-{mo:02}-{d:02} should be {expected}"
        );
        // And the standalone function agrees.
        assert_eq!(tai_minus_utc_seconds(utc.0), expected);
    }
}

/// Leap-second-boundary case: TT is continuous and strictly increasing across
/// the 2016-12-31 23:59:60 insertion, the three instants are spaced one SI
/// second apart in TT, and the leap-second TT matches the Swiss reference. Swiss
/// `swe.utc_to_jd(2016,12,31,23,59,60,GREG)[0]` = 2457754.5007891669.
#[test]
fn leap_second_boundary_is_continuous() {
    let before = utc_calendar_to_jd(2016, 12, 31, 23, 59, 59.0).0.to_tt().0;
    let (leap_jd, is_leap) = utc_calendar_to_jd(2016, 12, 31, 23, 59, 60.0);
    // Leap-correct conversion (post-step offset, +1 SI second).
    let leap = leap_jd.to_tt_for_leap_second().0;
    let after = utc_calendar_to_jd(2017, 1, 1, 0, 0, 0.0).0.to_tt().0;

    assert!(
        is_leap,
        "2016-12-31 23:59:60 must be recognised as a real leap second"
    );
    assert!(
        before < leap,
        "TT must increase 23:59:59 → 23:59:60 (got {before} !< {leap})"
    );
    assert!(
        leap <= after,
        "TT must not decrease 23:59:60 → 00:00:00 (got {leap} !> {after})"
    );

    // ≈1 SI second apart in TT, both intervals (1e-4 s tolerance: a sub-second
    // offset in a ~2.45e6 JD is f64-limited to ≈5 µs).
    assert!(
        ((leap - before) * SECONDS_PER_DAY - 1.0).abs() < 1e-4,
        "23:59:59 → 23:59:60 should be 1 s in TT"
    );
    assert!(
        ((after - leap) * SECONDS_PER_DAY - 1.0).abs() < 1e-4,
        "23:59:60 → 00:00:00 should be 1 s in TT"
    );

    // The leap-second TT matches the Swiss reference, and the post-leap midnight
    // matches the Swiss 2017-01-01 value (37 s offset).
    let swiss_leap_tt = 2_457_754.500_789_166_9_f64;
    let swiss_2017_tt = 2_457_754.500_800_740_9_f64;
    assert!(
        (leap - swiss_leap_tt).abs() < TOL_DAYS,
        "leap-second TT should match Swiss, got {leap} vs {swiss_leap_tt}"
    );
    assert!(
        (after - swiss_2017_tt).abs() < TOL_DAYS,
        "post-leap TT should match Swiss 2017-01-01 value"
    );

    // The :60.0 clock position sits exactly one SI second before the next
    // midnight (it aliases the 23:59:59 band, below the step).
    let next_midnight = utc_calendar_to_jd(2017, 1, 1, 0, 0, 0.0).0.0;
    let secs_before = (next_midnight - leap_jd.0) * SECONDS_PER_DAY;
    assert!(
        (secs_before - 1.0).abs() < 1e-4,
        "leap label should sit 1 s before the next midnight, got {secs_before} s"
    );

    // Epoch::from_utc applies the leap correction automatically.
    let e = Epoch::from_utc(2016, 12, 31, 23, 59, 60.0, 0.0);
    assert!(
        (e.jd_tt().0 - swiss_leap_tt).abs() < TOL_DAYS,
        "Epoch::from_utc leap-second TT should match Swiss"
    );
}
