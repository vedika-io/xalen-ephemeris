//! Error type shared by every function in this crate.

use xalen_ephem::EphemerisError;
use xalen_houses::HouseSystem;

/// Everything that can go wrong when building a relationship chart.
///
/// No function in this crate panics on caller data; bad input comes back as
/// one of these variants.
#[derive(Debug, thiserror::Error)]
pub enum RelationshipError {
    /// A number was not finite or out of range, a body cannot be charted, or
    /// the options make no sense (for example a non-positive orb multiplier).
    #[error("invalid input: {0}")]
    InvalidInput(String),
    /// The ephemeris could not produce a position, for example Pluto outside
    /// 1885-2099 with the built-in analytic series.
    #[error(transparent)]
    Ephemeris(#[from] EphemerisError),
    /// The chosen house system cannot be used with the chosen composite
    /// method. The midpoint-of-ascendants method supports only
    /// [`HouseSystem::Equal`] and [`HouseSystem::WholeSign`].
    #[error("house system {0:?} unsupported for this method")]
    UnsupportedHouseSystem(HouseSystem),
    /// The two charts have no body in common, so there is nothing to combine.
    #[error("charts share no common points")]
    NoCommonPoints,
}

/// Result alias used throughout this crate.
pub type Result<T> = std::result::Result<T, RelationshipError>;
