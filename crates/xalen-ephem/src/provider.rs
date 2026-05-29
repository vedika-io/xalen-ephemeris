use crate::body::Body;
use xalen_coords::EclipticPosition;
use xalen_time::JdTT;

#[derive(Debug, thiserror::Error)]
/// Errors that can occur during ephemeris computation.
pub enum EphemerisError {
    #[error("body {0} not available in this provider")]
    /// The requested body is not supported by this provider.
    BodyNotAvailable(Body),
    #[error("epoch JD {0} outside coverage")]
    /// The requested epoch falls outside the provider's valid date range.
    EpochOutOfRange(f64),
    #[error("computation failed: {0}")]
    /// A numerical or algorithmic error occurred during computation.
    ComputationFailed(String),
    #[error("I/O error: {0}")]
    /// A file I/O error occurred (e.g. reading a binary ephemeris file).
    IoError(#[from] std::io::Error),
    #[error("invalid file format: {0}")]
    /// The binary ephemeris file has an invalid or unsupported format.
    InvalidFormat(String),
}

/// Trait for computing planetary positions at a given epoch.
pub trait EphemerisProvider: Send + Sync {
    /// Compute heliocentric ecliptic position for a body at the given TT epoch.
    fn heliocentric_ecliptic(
        &self,
        body: Body,
        jd_tt: JdTT,
    ) -> Result<EclipticPosition, EphemerisError>;
    /// Compute geocentric ecliptic position for a body at the given TT epoch.
    fn geocentric_ecliptic(
        &self,
        body: Body,
        jd_tt: JdTT,
    ) -> Result<EclipticPosition, EphemerisError>;
    /// Return the valid Julian Date range (start, end) for this provider.
    fn coverage(&self) -> (f64, f64);
    /// Nominal accuracy of this provider in arcseconds.
    fn accuracy_arcsec(&self) -> f64;
    /// Human-readable name identifying this provider and its tier.
    fn name(&self) -> &str;
}
