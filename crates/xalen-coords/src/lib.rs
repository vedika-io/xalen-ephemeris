/// IAU 2000B nutation (77-term lunisolar series).
pub mod nutation;
/// Mean and true obliquity of the ecliptic (IAU 2006).
pub mod obliquity;
pub mod planet;
/// IAU 2006/P03 precession angles and rotation matrices.
pub mod precession;
/// Coordinate type definitions and conversion functions.
pub mod transforms;

pub use nutation::{NutationResult, nutation_2000b};
pub use obliquity::mean_obliquity;
pub use precession::{general_precession_longitude, precession_angles};
pub use transforms::{
    CartesianPosition, EclipticPosition, EquatorialPosition, cartesian_to_ecliptic,
    ecliptic_to_cartesian, ecliptic_to_equatorial, equatorial_to_ecliptic,
};

pub use planet::Planet;

/// Conversion factor from arcseconds to radians.
pub const ARCSEC_TO_RAD: f64 = std::f64::consts::PI / (180.0 * 3600.0);
/// Conversion factor from degrees to radians.
pub const DEG_TO_RAD: f64 = std::f64::consts::PI / 180.0;
/// Conversion factor from radians to degrees.
pub const RAD_TO_DEG: f64 = 180.0 / std::f64::consts::PI;
