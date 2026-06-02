use serde::{Deserialize, Serialize};
use std::f64::consts::{PI, TAU};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
/// Geographic location with latitude and longitude in radians.
pub struct GeoLocation {
    pub latitude: f64,
    pub longitude: f64,
}

impl GeoLocation {
    /// Create a location from latitude and longitude in degrees.
    pub fn new(lat_deg: f64, lon_deg: f64) -> Self {
        Self {
            latitude: lat_deg.to_radians(),
            longitude: lon_deg.to_radians(),
        }
    }

    /// Fallibly create a location from latitude and longitude in degrees,
    /// returning `None` for non-finite or out-of-range coordinates
    /// (`|lat| > 90` or `|lon| > 180`). `new` keeps its infallible contract;
    /// callers handling untrusted input should prefer `try_new`.
    pub fn try_new(lat_deg: f64, lon_deg: f64) -> Option<Self> {
        if !lat_deg.is_finite()
            || !lon_deg.is_finite()
            || lat_deg.abs() > 90.0
            || lon_deg.abs() > 180.0
        {
            return None;
        }
        Some(Self::new(lat_deg, lon_deg))
    }

    /// Return latitude in degrees.
    pub fn lat_deg(&self) -> f64 {
        self.latitude.to_degrees()
    }
    /// Return longitude in degrees.
    pub fn lon_deg(&self) -> f64 {
        self.longitude.to_degrees()
    }
}

/// Compute the Right Ascension of the MC (RAMC) from local sidereal time.
pub fn compute_ramc(lst_hours: f64) -> f64 {
    (lst_hours * 15.0).to_radians().rem_euclid(TAU)
}

/// Compute the Midheaven (MC) ecliptic longitude from RAMC and obliquity.
pub fn compute_mc(ramc: f64, epsilon: f64) -> f64 {
    let mc = ramc.sin().atan2(ramc.cos() * epsilon.cos());
    mc.rem_euclid(TAU)
}

/// Compute the Ascendant ecliptic longitude from RAMC, obliquity, and latitude.
///
/// The Ascendant is the rising point on the EASTERN horizon:
///   atan2(cos θ, -(sin ε · tan φ + cos ε · sin θ)),  θ = RAMC.
/// The sign-flipped form atan2(-cos θ, +(…)) returns a value exactly 180° away —
/// i.e. the Descendant. Verified against Swiss Ephemeris ground truth
/// (2026-05-29): the earlier signs were returning the Descendant.
pub fn compute_ascendant(ramc: f64, epsilon: f64, phi: f64) -> f64 {
    let y = ramc.cos();
    let x = -(epsilon.sin() * phi.tan() + epsilon.cos() * ramc.sin());
    let asc = y.atan2(x);
    asc.rem_euclid(TAU)
}

/// Compute the Imum Coeli (IC) as the point opposite the MC.
pub fn compute_ic(mc: f64) -> f64 {
    (mc + PI).rem_euclid(TAU)
}

/// Compute the Descendant as the point opposite the Ascendant.
pub fn compute_descendant(asc: f64) -> f64 {
    (asc + PI).rem_euclid(TAU)
}

/// Compute local sidereal time from GMST and geographic longitude.
pub fn local_sidereal_time(gmst_hours: f64, longitude_deg: f64) -> f64 {
    let lst = gmst_hours + longitude_deg / 15.0;
    lst.rem_euclid(24.0)
}

/// Compute Greenwich Mean Sidereal Time in hours for a given UT1 epoch.
///
/// Thin re-export of the canonical [`xalen_coords::gmst_hours`] — the single
/// source of truth for sidereal time across the workspace. Kept here under its
/// historical name so existing `xalen_houses::gmst` callers keep compiling.
pub fn gmst(jd_ut1: f64) -> f64 {
    xalen_coords::gmst_hours(jd_ut1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use xalen_coords::RAD_TO_DEG;

    #[test]
    fn ascendant_at_equator_vernal_equinox() {
        let ramc = 0.0;
        let epsilon = 23.4393_f64.to_radians();
        let phi = 0.0; // equator
        let asc = compute_ascendant(ramc, epsilon, phi);
        let asc_deg = asc * RAD_TO_DEG;
        assert!(
            (asc_deg - 90.0).abs() < 1.0 || (asc_deg - 270.0).abs() < 1.0,
            "ASC at equator with RAMC=0 should be ~90° or ~270°, got {asc_deg}°"
        );
    }

    #[test]
    fn mc_at_ramc_zero() {
        let epsilon = 23.4393_f64.to_radians();
        let mc = compute_mc(0.0, epsilon);
        let mc_deg = mc * RAD_TO_DEG;
        assert!(
            mc_deg.abs() < 1.0 || (mc_deg - 360.0).abs() < 1.0,
            "MC at RAMC=0 should be ~0°, got {mc_deg}°"
        );
    }

    #[test]
    fn ic_opposite_mc() {
        let mc = 120.0_f64.to_radians();
        let ic = compute_ic(mc);
        let diff = (ic - mc).abs() * RAD_TO_DEG;
        assert!((diff - 180.0).abs() < 0.01);
    }

    #[test]
    fn gmst_at_j2000() {
        let g = gmst(2_451_545.0);
        assert!(
            (g - 18.7).abs() < 0.5,
            "GMST at J2000 should be ~18.7h, got {g}h"
        );
    }

    #[test]
    fn lst_pune() {
        let g = gmst(2_451_545.0);
        let lst = local_sidereal_time(g, 73.85);
        assert!(lst >= 0.0 && lst < 24.0, "LST should be 0-24h, got {lst}h");
    }

    #[test]
    fn geo_location() {
        let loc = GeoLocation::new(18.52, 73.85);
        assert!((loc.lat_deg() - 18.52).abs() < 0.001);
        assert!((loc.lon_deg() - 73.85).abs() < 0.001);
    }

    #[test]
    fn geolocation_try_new_rejects_bad_input() {
        // Valid input round-trips through the same conversion as `new`.
        let ok = GeoLocation::try_new(18.52, 73.85).expect("valid coords accepted");
        assert!((ok.lat_deg() - 18.52).abs() < 0.001);
        assert!((ok.lon_deg() - 73.85).abs() < 0.001);
        // Boundary values are accepted (inclusive).
        assert!(GeoLocation::try_new(90.0, 180.0).is_some());
        assert!(GeoLocation::try_new(-90.0, -180.0).is_some());
        // Out-of-range and non-finite inputs are rejected.
        assert!(GeoLocation::try_new(90.1, 0.0).is_none());
        assert!(GeoLocation::try_new(-91.0, 0.0).is_none());
        assert!(GeoLocation::try_new(0.0, 180.5).is_none());
        assert!(GeoLocation::try_new(0.0, -181.0).is_none());
        assert!(GeoLocation::try_new(f64::NAN, 0.0).is_none());
        assert!(GeoLocation::try_new(0.0, f64::INFINITY).is_none());
        assert!(GeoLocation::try_new(f64::NEG_INFINITY, f64::NAN).is_none());
    }
}
