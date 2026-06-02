use crate::calendar::{CalendarSystem, calendar_to_jd, jd_to_calendar};
use crate::delta_t::DeltaTModel;
use crate::julian::{JdTT, JdUT1};
use crate::utc::{JdUTC, utc_calendar_to_jd};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A fully-specified astronomical epoch combining UT1, calendar, and delta-T model.
pub struct Epoch {
    /// The epoch expressed as a UT1 Julian Date.
    pub jd_ut1: JdUT1,
    /// Calendar system used for date display and input.
    pub calendar: CalendarSystem,
    /// Delta-T model for TT conversion.
    pub delta_t_model: DeltaTModel,
    /// Leap-second-exact TT, populated only when the epoch was built from a
    /// genuine UTC instant via [`Epoch::from_utc`].
    ///
    /// When `Some`, [`Epoch::jd_tt`] returns this value directly (UTC→TAI→TT via
    /// the IERS leap-second table, matching Swiss `swe_utc_to_jd`). When `None`
    /// (every legacy constructor: [`Epoch::new`], [`Epoch::from_jd_ut1`]) TT is
    /// derived from `jd_ut1` through the configured ΔT model exactly as before —
    /// **no existing number changes.**
    ///
    /// Serde-skipped when absent so the on-the-wire shape of legacy epochs is
    /// unchanged; defaults to `None` when deserializing older data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tt_from_utc: Option<JdTT>,
}

impl Epoch {
    /// Create an epoch from a local date/time with timezone offset (hours east of UTC).
    ///
    /// The local time minus the offset yields civil UTC, which is stored in
    /// `jd_ut1` as an **approximation of UT1** (UTC ≈ UT1 to within ±0.9 s — see
    /// [`crate::calendar_to_jd`]). No DUT1 correction is applied; pass an
    /// already-UT1 instant via [`Epoch::from_jd_ut1`] if sub-second UT1 accuracy
    /// is required.
    pub fn new(year: i32, month: u32, day: u32, hour: f64, tz_offset_hours: f64) -> Self {
        let ut_hour = hour - tz_offset_hours;
        let jd = calendar_to_jd(
            year,
            month,
            day,
            ut_hour,
            CalendarSystem::ProlepticGregorian,
        );
        Self {
            jd_ut1: jd,
            calendar: CalendarSystem::ProlepticGregorian,
            delta_t_model: DeltaTModel::StephensonMorrisonHohenkerk2016,
            tt_from_utc: None,
        }
    }

    /// Create an epoch directly from a UT1 Julian Date.
    pub fn from_jd_ut1(jd: f64) -> Self {
        Self {
            jd_ut1: JdUT1(jd),
            calendar: CalendarSystem::ProlepticGregorian,
            delta_t_model: DeltaTModel::StephensonMorrisonHohenkerk2016,
            tt_from_utc: None,
        }
    }

    /// Create an epoch from a **UTC** date/time, applying the IERS leap-second
    /// table to form a Swiss-grade UT1/TT chain.
    ///
    /// This is the real UTC-aware constructor closing the gap against
    /// `swe_utc_to_jd`. Unlike [`Epoch::new`] (which removes a fixed tz offset and
    /// then *labels* the civil clock reading as UT1, accurate to ±0.9 s), this
    /// path:
    ///
    /// 1. Builds the plain UTC Julian Date from the calendar fields, correctly
    ///    handling a `second` of `60` during an inserted leap second.
    /// 2. Computes the **leap-second-exact TT** via UTC→TAI→TT
    ///    (`TT = UTC + (TAI−UTC) + 32.184 s`) and stores it in
    ///    [`Epoch::tt_from_utc`] so [`Epoch::jd_tt`] returns it directly.
    /// 3. Computes **UT1 = UTC + DUT1** (`dut1_seconds` = observed UT1−UTC from
    ///    IERS Bulletin A/B, |DUT1| < 0.9 s) and stores it in `jd_ut1`. Pass
    ///    `dut1_seconds = 0.0` to reproduce the legacy UTC≈UT1 approximation for
    ///    Earth-rotation-dependent quantities while still getting exact TT.
    ///
    /// `hour`/`minute` are whole; `second` is fractional and is the SI-seconds
    /// reading of the UTC clock (so `60.0 ≤ second < 61.0` is only valid at
    /// 23:59 on the last day of a leap-second month — see
    /// [`crate::leap_seconds`]).
    ///
    /// The calendar field is set to proleptic Gregorian (UTC is defined on the
    /// Gregorian calendar); use [`Epoch::with_calendar`] afterwards only for
    /// display purposes.
    pub fn from_utc(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: f64,
        dut1_seconds: f64,
    ) -> Self {
        let (jd_utc, is_leap): (JdUTC, bool) =
            utc_calendar_to_jd(year, month, day, hour, minute, second);
        // For a genuine inserted leap second the TT uses the post-step offset
        // (one SI second beyond the aliased clock position); ordinary instants
        // use the plain position-based conversion.
        let tt = if is_leap {
            jd_utc.to_tt_for_leap_second()
        } else {
            jd_utc.to_tt()
        };
        let ut1 = jd_utc.to_ut1(dut1_seconds);
        Self {
            jd_ut1: ut1,
            calendar: CalendarSystem::ProlepticGregorian,
            delta_t_model: DeltaTModel::StephensonMorrisonHohenkerk2016,
            tt_from_utc: Some(tt),
        }
    }

    /// Set the calendar system (builder pattern).
    pub fn with_calendar(mut self, cal: CalendarSystem) -> Self {
        self.calendar = cal;
        self
    }

    /// Set the delta-T model (builder pattern).
    pub fn with_delta_t(mut self, model: DeltaTModel) -> Self {
        self.delta_t_model = model;
        self
    }

    /// Convert this epoch to TT.
    ///
    /// If the epoch was built from a UTC instant ([`Epoch::from_utc`]), this
    /// returns the leap-second-exact TT stored in [`Epoch::tt_from_utc`]
    /// (matching Swiss `swe_utc_to_jd`). Otherwise TT is derived from `jd_ut1`
    /// through the configured ΔT model (unchanged legacy behaviour).
    pub fn jd_tt(&self) -> JdTT {
        match self.tt_from_utc {
            Some(tt) => tt,
            None => self.jd_ut1.to_tt(&self.delta_t_model),
        }
    }

    /// Convert this epoch to TT, also returning the 1σ uncertainty envelope of
    /// the conversion.
    ///
    /// The returned `JdTT` is identical to [`Epoch::jd_tt`] — purely additive.
    /// The second element is the **1σ uncertainty of the ΔT (TT−UT1) model in
    /// SECONDS of time** — the time-conversion envelope, NOT a position or
    /// output covariance. Mirrors [`crate::JdUT1::to_tt_with_sigma`].
    ///
    /// For a UTC-derived epoch ([`Epoch::from_utc`]) the TT is the
    /// leap-second-exact value (no ΔT model is involved), so the conversion
    /// envelope is `0.0` — TT is known to definitional precision, not modelled.
    pub fn jd_tt_with_sigma(&self) -> (JdTT, f64) {
        match self.tt_from_utc {
            Some(tt) => (tt, 0.0),
            None => self.jd_ut1.to_tt_with_sigma(&self.delta_t_model),
        }
    }

    /// Convert this epoch to a calendar date (year, month, day, hour).
    pub fn to_calendar(&self) -> (i32, u32, u32, f64) {
        jd_to_calendar(self.jd_ut1.0, self.calendar)
    }

    /// Return the decimal year (e.g. 2000.5 for mid-2000).
    pub fn year(&self) -> f64 {
        2000.0 + (self.jd_ut1.0 - 2_451_545.0) / 365.25
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_pune_ist() {
        let e = Epoch::new(1990, 1, 15, 10.5, 5.5);
        let (y, m, d, h) = e.to_calendar();
        assert_eq!(y, 1990);
        assert_eq!(m, 1);
        assert_eq!(d, 15);
        assert!((h - 5.0).abs() < 0.01, "10:30 IST = 05:00 UTC, got {h}");
    }

    #[test]
    fn epoch_year() {
        let e = Epoch::from_jd_ut1(2_451_545.0);
        assert!((e.year() - 2000.0).abs() < 0.01);
    }

    /// Legacy constructors must leave `tt_from_utc` unset so `jd_tt` keeps using
    /// the ΔT model — proving the UTC path is purely additive.
    #[test]
    fn legacy_constructors_keep_delta_t_path() {
        let e1 = Epoch::new(1990, 1, 15, 10.5, 5.5);
        let e2 = Epoch::from_jd_ut1(2_451_545.0);
        assert!(e1.tt_from_utc.is_none());
        assert!(e2.tt_from_utc.is_none());
        // jd_tt equals the direct ΔT-model conversion (bit-for-bit).
        assert_eq!(e2.jd_tt().0, e2.jd_ut1.to_tt(&e2.delta_t_model).0);
    }

    /// `from_utc` produces the leap-second-exact TT (32 s + 32.184 s ahead of the
    /// UTC clock at J2000) and stores it in `tt_from_utc`.
    #[test]
    fn from_utc_tt_uses_leap_seconds() {
        // J2000 noon UTC: TAI−UTC = 32 s, so TT−UTC = 32 + 32.184 = 64.184 s.
        let e = Epoch::from_utc(2000, 1, 1, 12, 0, 0.0, 0.0);
        assert!(e.tt_from_utc.is_some());
        let plain_utc = 2_451_545.0; // J2000 noon plain JD.
        let tt_minus_utc_s = (e.jd_tt().0 - plain_utc) * crate::SECONDS_PER_DAY;
        // Tolerance 1e-4 s: representing a sub-second offset in a ~2.45e6 JD is
        // f64-limited to ≈5 µs; the value is otherwise bit-exact vs Swiss.
        assert!(
            (tt_minus_utc_s - 64.184).abs() < 1e-4,
            "TT−UTC at J2000 should be 64.184 s, got {tt_minus_utc_s}"
        );
        // σ is exactly 0 for the UTC path.
        let (_, sigma) = e.jd_tt_with_sigma();
        assert_eq!(sigma, 0.0);
    }

    /// DUT1 shifts UT1 but never TT.
    #[test]
    fn from_utc_dut1_only_moves_ut1() {
        let zero = Epoch::from_utc(2017, 1, 1, 0, 0, 0.0, 0.0);
        let with_dut1 = Epoch::from_utc(2017, 1, 1, 0, 0, 0.0, -0.4);
        // TT identical.
        assert_eq!(zero.jd_tt().0, with_dut1.jd_tt().0);
        // UT1 differs by exactly 0.4 s.
        let dut1_days = (with_dut1.jd_ut1.0 - zero.jd_ut1.0) * crate::SECONDS_PER_DAY;
        // f64-JD subtraction at ~2.46e6 is precision-limited to ~50 µs; 1 ms still
        // proves DUT1 moved UT1 by the right amount (a sign/units bug is >= 0.4 s).
        assert!(
            (dut1_days + 0.4).abs() < 1e-3,
            "UT1 should shift by DUT1, got {dut1_days}"
        );
        // With DUT1 = 0, UT1 == plain UTC (legacy behaviour preserved).
        assert!((zero.jd_ut1.0 - 2_457_754.5).abs() < 1e-12);
    }
}
