use crate::leap_seconds::tai_minus_utc_seconds;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Astronomical time scale identifiers.
pub enum TimeScale {
    /// International Atomic Time — continuous, no leap seconds
    TAI,
    /// Terrestrial Time — TAI + 32.184s, for geocentric ephemerides
    TT,
    /// Barycentric Dynamical Time — for JPL DE ephemerides
    TDB,
    /// Universal Time 1 — Earth rotation based
    UT1,
    /// Coordinated Universal Time — civil time with leap seconds
    UTC,
}

impl TimeScale {
    /// Fixed offset from TAI to TT in seconds (32.184 s).
    pub const TAI_TT_OFFSET_SECONDS: f64 = 32.184;

    /// Convert TAI seconds to TT seconds by adding the fixed offset.
    pub fn tai_to_tt(tai_seconds: f64) -> f64 {
        tai_seconds + Self::TAI_TT_OFFSET_SECONDS
    }

    /// Convert TT seconds to TAI seconds by subtracting the fixed offset.
    pub fn tt_to_tai(tt_seconds: f64) -> f64 {
        tt_seconds - Self::TAI_TT_OFFSET_SECONDS
    }

    /// Offset, in **seconds**, to add to a Julian Date on this scale to reach the
    /// **TAI** scale, evaluated at the UTC instant `jd_utc` (a plain UTC clock
    /// JD; only used for the leap-second-dependent UTC/UT1 cases — TT/TDB/TAI
    /// offsets are constants).
    ///
    /// This is the real, scale-aware wiring of the previously-dead [`Self::UTC`]
    /// and [`Self::TAI`] variants. `dut1_seconds` is the observed UT1−UTC value
    /// (IERS Bulletin A/B, |DUT1| < 0.9 s); it is only consulted for [`Self::UT1`]
    /// and ignored for the other scales. Pass `0.0` for the legacy UTC≈UT1
    /// behaviour.
    ///
    /// Sign convention: `tai_jd = this_jd + offset_to_tai_seconds(...) / 86400`.
    ///   * `TAI  → TAI`:  0
    ///   * `TT   → TAI`: −32.184
    ///   * `TDB  → TAI`: ≈ −32.184 (TDB−TT ≲ 2 ms ignored at this seconds-level API)
    ///   * `UTC  → TAI`: +(TAI−UTC) leap seconds
    ///   * `UT1  → TAI`: +(TAI−UTC) − DUT1   (since TAI−UT1 = (TAI−UTC) − (UT1−UTC))
    #[must_use]
    pub fn offset_to_tai_seconds(self, jd_utc: f64, dut1_seconds: f64) -> f64 {
        let leap = f64::from(tai_minus_utc_seconds(jd_utc));
        match self {
            TimeScale::TAI => 0.0,
            TimeScale::TT | TimeScale::TDB => -Self::TAI_TT_OFFSET_SECONDS,
            TimeScale::UTC => leap,
            TimeScale::UT1 => leap - dut1_seconds,
        }
    }

    /// Offset, in **seconds**, to add to a Julian Date on `from` to reach `to`,
    /// evaluated at the UTC instant `jd_utc` with the given `dut1_seconds`
    /// (UT1−UTC). Computed by routing through TAI:
    /// `to_jd = from_jd + convert_offset_seconds(...) / 86400`.
    #[must_use]
    pub fn convert_offset_seconds(from: Self, to: Self, jd_utc: f64, dut1_seconds: f64) -> f64 {
        // (X→TAI) then (TAI→Y) = (X→TAI) − (Y→TAI).
        from.offset_to_tai_seconds(jd_utc, dut1_seconds)
            - to.offset_to_tai_seconds(jd_utc, dut1_seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SECONDS_PER_DAY;

    // 2017-01-01 UTC: TAI−UTC = 37 s.
    const JD_2017: f64 = 2_457_754.5;

    #[test]
    fn utc_to_tai_offset_is_leap_seconds() {
        assert!((TimeScale::UTC.offset_to_tai_seconds(JD_2017, 0.0) - 37.0).abs() < 1e-9);
    }

    #[test]
    fn tt_to_tai_offset_is_constant() {
        assert!((TimeScale::TT.offset_to_tai_seconds(JD_2017, 0.0) + 32.184).abs() < 1e-9);
    }

    #[test]
    fn ut1_offset_includes_dut1() {
        // TAI − UT1 = (TAI−UTC) − (UT1−UTC) = 37 − (−0.3) = 37.3 s.
        let off = TimeScale::UT1.offset_to_tai_seconds(JD_2017, -0.3);
        assert!((off - 37.3).abs() < 1e-9, "got {off}");
    }

    #[test]
    fn utc_to_tt_via_convert_matches_known_value() {
        // UTC→TT = (UTC→TAI) − (TT→TAI) = 37 − (−32.184) = 69.184 s.
        let off = TimeScale::convert_offset_seconds(TimeScale::UTC, TimeScale::TT, JD_2017, 0.0);
        assert!((off - (37.0 + 32.184)).abs() < 1e-9, "got {off}");
        // Apply it and compare against the seconds-level expectation.
        let tt_jd = JD_2017 + off / SECONDS_PER_DAY;
        // ~50 µs f64-JD floor on the round-trip subtraction; the direct offset above is exact.
        assert!(((tt_jd - JD_2017) * SECONDS_PER_DAY - 69.184).abs() < 1e-3);
    }

    #[test]
    fn convert_is_antisymmetric() {
        let a = TimeScale::convert_offset_seconds(TimeScale::UTC, TimeScale::TT, JD_2017, 0.2);
        let b = TimeScale::convert_offset_seconds(TimeScale::TT, TimeScale::UTC, JD_2017, 0.2);
        assert!(
            (a + b).abs() < 1e-9,
            "convert should be antisymmetric: {a} vs {b}"
        );
    }

    #[test]
    fn same_scale_is_zero() {
        for sc in [
            TimeScale::TAI,
            TimeScale::TT,
            TimeScale::TDB,
            TimeScale::UT1,
            TimeScale::UTC,
        ] {
            let off = TimeScale::convert_offset_seconds(sc, sc, JD_2017, 0.1);
            assert!(off.abs() < 1e-12, "{sc:?}→{sc:?} should be 0, got {off}");
        }
    }
}
