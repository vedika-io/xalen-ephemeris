//! Commercial-OK reference star **anchors** for sidereal ayanamsa computation.
//!
//! # Why this crate exists
//!
//! Sidereal frames such as **True Chitra** (Chitrapaksha) are *defined* by
//! anchoring a single bright star (Spica / Chitra) at an exact ecliptic
//! longitude (180°). Computing that ayanamsa needs only **one star's** J2000
//! ecliptic position and proper motion — a handful of individual factual
//! coordinates, NOT a catalogue compilation.
//!
//! Previously `xalen-ayanamsa` read Spica through
//! `xalen_stars::find_by_name("Spica")`, which sources its position from the
//! HIP-reconciled catalog — data that is gated off commercial builds (the
//! Hipparcos / CDS I/239 compilation is CC BY-NC). Reading it there would have
//! **forked the core sidereal output** between the open and commercial builds.
//!
//! This crate holds those single anchor coordinates as plain `Apache-2.0`
//! constants so the ayanamsa is identical in every build configuration, with no
//! dependency on the non-commercial catalogue.
//!
//! ## Provenance (ASK-SENIOR)
//!
//! A single star's measured ecliptic coordinate is a fact, not a protectable
//! compilation. The [`SPICA`] values below are the exact ones the open library
//! has been emitting (so True Chitra stays byte-identical across the
//! `hip-catalog` feature). An equally valid, even-cleaner-provenance choice is
//! the self-derived FK5/Hipparcos equatorial place already carried in
//! `xalen-stars`' Apache `CATALOG` (203.841355, −2.054487, pm −27.72/−45.23),
//! which differs by ~1e-6° (~0.01″) and would shift today's output by that
//! amount. The exact sourcing to cite is a counsel/product decision — see the
//! crate-lock execution map. The engineering is correct either way.

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

/// **Spica** (α Virginis), the anchor for the True Chitra / Chitrapaksha
/// sidereal frame. Values match what the open `xalen-stars` library emits for
/// `find_by_name("Spica")`, so True Chitra is byte-identical with or without the
/// `hip-catalog` feature. See the crate-level provenance note.
pub const SPICA: StarAnchor = StarAnchor {
    longitude_j2000: 203.841358,
    latitude_j2000: -2.054496,
    pm_lon_mas_per_year: -27.720,
    pm_lat_mas_per_year: -45.228,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// Bit-exact guard. The Spica anchor feeds the True Chitra ayanamsa; if it
    /// changes, sidereal output shifts and the open/commercial builds would
    /// diverge. This pins the exact f64 bit patterns (captured from the
    /// HIP-reconciled `find_by_name("Spica")` the open library emits) so any
    /// accidental edit fails CI. Update ONLY with a deliberate, reviewed reason.
    #[test]
    fn spica_anchor_is_bit_exact() {
        assert_eq!(SPICA.longitude_j2000.to_bits(), 0x40697aec679cc74c);
        assert_eq!(SPICA.latitude_j2000.to_bits(), 0xc0006f9b994e1a3f);
        assert_eq!(SPICA.pm_lon_mas_per_year.to_bits(), 0xc03bb851eb851eb8);
        assert_eq!(SPICA.pm_lat_mas_per_year.to_bits(), 0xc0469d2f1a9fbe77);
    }
}
