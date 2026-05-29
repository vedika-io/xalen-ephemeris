use crate::nutation::{NutationResult, nutation_2000b};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EclipticPosition {
    pub longitude: f64, // radians
    pub latitude: f64,  // radians
    pub distance: f64,  // AU
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EquatorialPosition {
    pub right_ascension: f64, // radians
    pub declination: f64,     // radians
    pub distance: f64,        // AU
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CartesianPosition {
    pub x: f64, // AU
    pub y: f64, // AU
    pub z: f64, // AU
}

impl EclipticPosition {
    pub fn longitude_deg(&self) -> f64 {
        self.longitude.to_degrees()
    }
    pub fn latitude_deg(&self) -> f64 {
        self.latitude.to_degrees()
    }

    pub fn normalize(mut self) -> Self {
        self.longitude = self.longitude.rem_euclid(std::f64::consts::TAU);
        self
    }
}

impl EquatorialPosition {
    pub fn ra_hours(&self) -> f64 {
        self.right_ascension.to_degrees() / 15.0
    }
    pub fn dec_deg(&self) -> f64 {
        self.declination.to_degrees()
    }
}

pub fn ecliptic_to_equatorial(ecl: &EclipticPosition, epsilon: f64) -> EquatorialPosition {
    let cos_eps = epsilon.cos();
    let sin_eps = epsilon.sin();
    let cos_lat = ecl.latitude.cos();
    let sin_lat = ecl.latitude.sin();
    let cos_lon = ecl.longitude.cos();
    let sin_lon = ecl.longitude.sin();

    let ra = (sin_lon * cos_eps - sin_lat / cos_lat * sin_eps).atan2(cos_lon);
    let dec = (sin_lat * cos_eps + cos_lat * sin_eps * sin_lon).asin();

    EquatorialPosition {
        right_ascension: ra.rem_euclid(std::f64::consts::TAU),
        declination: dec,
        distance: ecl.distance,
    }
}

pub fn equatorial_to_ecliptic(eq: &EquatorialPosition, epsilon: f64) -> EclipticPosition {
    let cos_eps = epsilon.cos();
    let sin_eps = epsilon.sin();
    let cos_dec = eq.declination.cos();
    let sin_dec = eq.declination.sin();
    let cos_ra = eq.right_ascension.cos();
    let sin_ra = eq.right_ascension.sin();

    let lon = (sin_ra * cos_eps + sin_dec / cos_dec * sin_eps).atan2(cos_ra);
    let lat = (sin_dec * cos_eps - cos_dec * sin_eps * sin_ra).asin();

    EclipticPosition {
        longitude: lon.rem_euclid(std::f64::consts::TAU),
        latitude: lat,
        distance: eq.distance,
    }
}

pub fn ecliptic_to_cartesian(ecl: &EclipticPosition) -> CartesianPosition {
    let cos_lat = ecl.latitude.cos();
    CartesianPosition {
        x: ecl.distance * cos_lat * ecl.longitude.cos(),
        y: ecl.distance * cos_lat * ecl.longitude.sin(),
        z: ecl.distance * ecl.latitude.sin(),
    }
}

pub fn cartesian_to_ecliptic(cart: &CartesianPosition) -> EclipticPosition {
    let r = (cart.x * cart.x + cart.y * cart.y + cart.z * cart.z).sqrt();
    let lon = cart.y.atan2(cart.x);
    let lat = (cart.z / r).asin();

    EclipticPosition {
        longitude: lon.rem_euclid(std::f64::consts::TAU),
        latitude: lat,
        distance: r,
    }
}

/// Apply nutation to geometric ecliptic coordinates to get apparent position.
pub fn apparent_ecliptic(
    geo_ecliptic: &EclipticPosition,
    t: f64,
) -> (EclipticPosition, NutationResult) {
    let nut = nutation_2000b(t);
    let apparent = EclipticPosition {
        longitude: geo_ecliptic.longitude + nut.delta_psi,
        latitude: geo_ecliptic.latitude,
        distance: geo_ecliptic.distance,
    };
    (apparent, nut)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DEG_TO_RAD, RAD_TO_DEG};

    #[test]
    fn ecliptic_equatorial_roundtrip() {
        let epsilon = 23.4393 * DEG_TO_RAD;
        let ecl = EclipticPosition {
            longitude: 45.0 * DEG_TO_RAD,
            latitude: 5.0 * DEG_TO_RAD,
            distance: 1.0,
        };
        let eq = ecliptic_to_equatorial(&ecl, epsilon);
        let ecl2 = equatorial_to_ecliptic(&eq, epsilon);
        assert!(
            (ecl.longitude - ecl2.longitude).abs() < 1e-10,
            "Longitude roundtrip failed"
        );
        assert!(
            (ecl.latitude - ecl2.latitude).abs() < 1e-10,
            "Latitude roundtrip failed"
        );
    }

    #[test]
    fn cartesian_ecliptic_roundtrip() {
        let ecl = EclipticPosition {
            longitude: 120.0 * DEG_TO_RAD,
            latitude: -3.0 * DEG_TO_RAD,
            distance: 5.2,
        };
        let cart = ecliptic_to_cartesian(&ecl);
        let ecl2 = cartesian_to_ecliptic(&cart);
        assert!((ecl.longitude - ecl2.longitude).abs() < 1e-10);
        assert!((ecl.latitude - ecl2.latitude).abs() < 1e-10);
        assert!((ecl.distance - ecl2.distance).abs() < 1e-10);
    }

    #[test]
    fn vernal_equinox_ra_is_zero() {
        let epsilon = 23.4393 * DEG_TO_RAD;
        let ecl = EclipticPosition {
            longitude: 0.0,
            latitude: 0.0,
            distance: 1.0,
        };
        let eq = ecliptic_to_equatorial(&ecl, epsilon);
        assert!(
            eq.right_ascension.abs() < 1e-10,
            "RA at vernal equinox should be 0"
        );
        assert!(
            eq.declination.abs() < 1e-10,
            "Dec at vernal equinox should be 0"
        );
    }

    #[test]
    fn summer_solstice_dec_equals_obliquity() {
        let epsilon = 23.4393 * DEG_TO_RAD;
        let ecl = EclipticPosition {
            longitude: 90.0 * DEG_TO_RAD,
            latitude: 0.0,
            distance: 1.0,
        };
        let eq = ecliptic_to_equatorial(&ecl, epsilon);
        assert!(
            (eq.declination - epsilon).abs() < 1e-10,
            "Dec at summer solstice should equal obliquity: {} vs {}",
            eq.dec_deg(),
            epsilon * RAD_TO_DEG
        );
    }

    #[test]
    fn normalize_longitude() {
        let ecl = EclipticPosition {
            longitude: -30.0 * DEG_TO_RAD,
            latitude: 0.0,
            distance: 1.0,
        };
        let n = ecl.normalize();
        assert!(n.longitude >= 0.0 && n.longitude < std::f64::consts::TAU);
    }
}
