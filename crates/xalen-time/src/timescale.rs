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
}
