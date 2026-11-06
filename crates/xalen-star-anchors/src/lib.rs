//! Reference star **anchors** for sidereal ayanamsa computation.
//!
//! # Why this crate exists
//!
//! Sidereal frames such as **True Chitra** (Chitrapaksha) are *defined* by
//! anchoring a single bright star (Spica / Chitra) at an exact ecliptic
//! longitude (180°). Computing that ayanamsa needs only **one star's** J2000
//! ecliptic position and proper motion — a handful of individual factual
//! coordinates, NOT a catalogue compilation.
//!
//! The ayanamsa crate reads these constants instead of searching a star list, so
//! the sidereal output is the same in every build configuration (with or without
//! the `bright-catalog` feature of `xalen-stars`), with no dependency on any
//! star catalogue.
//!
//! ## Provenance
//!
//! The [`SPICA`] values are derived from the same two sources as the
//! `xalen-stars` Bright Star Catalogue list (HR 5056, α Virginis):
//!
//! * **position**: the J2000 right ascension and declination of Spica in the
//!   IAU Catalog of Star Names (IAU WGSN, Creative Commons Attribution), taken
//!   to 6 decimals of a degree and converted to the J2000 mean ecliptic with the
//!   IAU 2006 obliquity (84381.406″);
//! * **proper motion**: the proper motion of the Yale Bright Star Catalogue,
//!   5th Revised Ed. (Hoffleit & Warren 1991, CDS V/50), rotated into ecliptic
//!   components (mas/yr).
//!
//! A test in `xalen-stars` (`spica_anchor_equals_the_catalogue_row`) checks that
//! these constants equal the catalogue row bit for bit.

/// A single reference star's J2000.0 mean-ecliptic position and linear proper
/// motion — the only quantities a star-anchored sidereal frame needs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StarAnchor {
    /// Ecliptic longitude at J2000.0 (degrees).
    pub longitude_j2000: f64,
    /// Ecliptic latitude at J2000.0 (degrees, signed).
    pub latitude_j2000: f64,
    /// Proper motion in ecliptic longitude (milliarcseconds per year).
    pub pm_lon_mas_per_year: f64,
    /// Proper motion in ecliptic latitude (milliarcseconds per year).
    pub pm_lat_mas_per_year: f64,
}

/// **Spica** (α Virginis, HR 5056), the anchor for the True Chitra /
/// Chitrapaksha sidereal frame. Equal to the `xalen-stars` Bright Star Catalogue
/// row of Spica; see the crate-level provenance note.
pub const SPICA: StarAnchor = StarAnchor {
    longitude_j2000: 203.841357,
    latitude_j2000: -2.054493,
    pm_lon_mas_per_year: -27.711,
    pm_lat_mas_per_year: -41.208,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// Bit-exact guard. The Spica anchor feeds the True Chitra ayanamsa; if it
    /// changes, sidereal output shifts. This pins the exact f64 bit patterns
    /// (203.841357, -2.054493, -27.711, -41.208) so any accidental edit fails CI.
    /// Update ONLY with a deliberate, reviewed reason, together with the
    /// `Bright Star Catalogue` row it must equal.
    #[test]
    fn spica_anchor_is_bit_exact() {
        assert_eq!(SPICA.longitude_j2000.to_bits(), 0x40697aec6583e857);
        assert_eq!(SPICA.latitude_j2000.to_bits(), 0xc0006f9a06a6e32e);
        assert_eq!(SPICA.pm_lon_mas_per_year.to_bits(), 0xc03bb604189374bc);
        assert_eq!(SPICA.pm_lat_mas_per_year.to_bits(), 0xc0449a9fbe76c8b4);
    }
}
