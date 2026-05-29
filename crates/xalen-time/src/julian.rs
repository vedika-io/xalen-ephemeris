use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Sub};

use crate::delta_t::{DeltaTModel, delta_t};
use crate::{DAYS_PER_JULIAN_CENTURY, J2000_JD, SECONDS_PER_DAY};

/// Trait for types representing a Julian Date in a specific time scale.
pub trait JulianDay: Copy + PartialOrd {
    /// Return the Julian Date as a raw f64 value.
    fn as_f64(self) -> f64;
    fn julian_centuries_from_j2000(self) -> f64 {
        (self.as_f64() - J2000_JD) / DAYS_PER_JULIAN_CENTURY
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
/// Julian Date in the Terrestrial Time (TT) time scale.
pub struct JdTT(pub f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
/// Julian Date in the Universal Time (UT1) time scale.
pub struct JdUT1(pub f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
/// Julian Date in the Barycentric Dynamical Time (TDB) time scale.
pub struct JdTDB(pub f64);

impl JulianDay for JdTT {
    fn as_f64(self) -> f64 {
        self.0
    }
}
impl JulianDay for JdUT1 {
    fn as_f64(self) -> f64 {
        self.0
    }
}
impl JulianDay for JdTDB {
    fn as_f64(self) -> f64 {
        self.0
    }
}

impl JdUT1 {
    /// Convert UT1 to TT by adding delta-T.
    pub fn to_tt(self, model: &DeltaTModel) -> JdTT {
        let dt = delta_t(self.0, model);
        JdTT(self.0 + dt / SECONDS_PER_DAY)
    }

    /// Convert UT1 to TDB via TT intermediate.
    pub fn to_tdb(self, model: &DeltaTModel) -> JdTDB {
        let tt = self.to_tt(model);
        tt.to_tdb()
    }
}

impl JdTT {
    /// Convert TT to UT1 by iteratively subtracting delta-T.
    pub fn to_ut1(self, model: &DeltaTModel) -> JdUT1 {
        // P0 fix: iterative solve — delta_t expects UT1-scale argument
        let mut ut1_est = self.0;
        for _ in 0..3 {
            let dt = delta_t(ut1_est, model);
            ut1_est = self.0 - dt / SECONDS_PER_DAY;
        }
        JdUT1(ut1_est)
    }

    /// Convert TT to TDB using the Fairhead & Bretagnon approximation.
    pub fn to_tdb(self) -> JdTDB {
        // Fairhead & Bretagnon (1990) / Meeus approximation
        let t = self.julian_centuries_from_j2000();
        let g = (357.528 + 35999.050 * t).to_radians();
        let tdb_minus_tt = 0.001658 * g.sin() + 0.000014 * (2.0 * g).sin();
        JdTDB(self.0 + tdb_minus_tt / SECONDS_PER_DAY)
    }

    /// The J2000.0 standard epoch as a TT Julian Date.
    pub const J2000: Self = JdTT(J2000_JD);
}

impl JdTDB {
    pub fn to_tt(self) -> JdTT {
        let t = (self.0 - J2000_JD) / DAYS_PER_JULIAN_CENTURY;
        let g = (357.528 + 35999.050 * t).to_radians();
        let tdb_minus_tt = 0.001658 * g.sin() + 0.000014 * (2.0 * g).sin();
        JdTT(self.0 - tdb_minus_tt / SECONDS_PER_DAY)
    }
}

impl Add<f64> for JdTDB {
    type Output = JdTDB;
    fn add(self, days: f64) -> JdTDB {
        JdTDB(self.0 + days)
    }
}

impl Sub for JdTDB {
    type Output = f64;
    fn sub(self, rhs: JdTDB) -> f64 {
        self.0 - rhs.0
    }
}

impl Add<f64> for JdTT {
    type Output = JdTT;
    fn add(self, days: f64) -> JdTT {
        JdTT(self.0 + days)
    }
}

impl Sub for JdTT {
    type Output = f64;
    fn sub(self, rhs: JdTT) -> f64 {
        self.0 - rhs.0
    }
}

impl Add<f64> for JdUT1 {
    type Output = JdUT1;
    fn add(self, days: f64) -> JdUT1 {
        JdUT1(self.0 + days)
    }
}

impl Sub for JdUT1 {
    type Output = f64;
    fn sub(self, rhs: JdUT1) -> f64 {
        self.0 - rhs.0
    }
}

impl fmt::Display for JdTT {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JD(TT) {:.6}", self.0)
    }
}

impl fmt::Display for JdUT1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JD(UT1) {:.6}", self.0)
    }
}

impl fmt::Display for JdTDB {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JD(TDB) {:.6}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn j2000_roundtrip() {
        let tt = JdTT::J2000;
        assert!((tt.0 - 2_451_545.0).abs() < 1e-10);
        assert!(tt.julian_centuries_from_j2000().abs() < 1e-15);
    }

    #[test]
    fn tt_tdb_roundtrip() {
        let tt = JdTT(2_460_000.5);
        let tdb = tt.to_tdb();
        let tt2 = tdb.to_tt();
        assert!((tt.0 - tt2.0).abs() < 1e-10, "TT->TDB->TT should roundtrip");
    }

    #[test]
    fn ut1_tt_conversion() {
        let ut1 = JdUT1(2_451_545.0);
        let model = DeltaTModel::StephensonMorrisonHohenkerk2016;
        let tt = ut1.to_tt(&model);
        assert!(tt.0 > ut1.0, "TT should be ahead of UT1 for modern dates");
        let delta_days = tt.0 - ut1.0;
        let delta_seconds = delta_days * SECONDS_PER_DAY;
        assert!(
            (delta_seconds - 63.83).abs() < 1.0,
            "delta-T at J2000 should be ~63.83s, got {delta_seconds}"
        );
    }

    #[test]
    fn tdb_tt_difference_bounded() {
        for jd in [2_451_545.0, 2_460_000.5, 2_440_000.5, 2_470_000.5] {
            let tt = JdTT(jd);
            let tdb = tt.to_tdb();
            let diff_ms = (tdb.0 - tt.0) * SECONDS_PER_DAY * 1000.0;
            assert!(
                diff_ms.abs() < 2.0,
                "TDB-TT should be < 2ms, got {diff_ms}ms at JD {jd}"
            );
        }
    }
}
