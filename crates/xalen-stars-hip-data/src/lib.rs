//! Hipparcos-derived fixed-star catalog **data** for XALEN Ephemeris.
//!
//! # License — non-commercial
//!
//! The catalogue embedded here is derived from the Hipparcos Main Catalogue
//! (CDS I/239, `hip_main.dat`, (C) ESA 1997), distributed by CDS under
//! **CC BY-NC** (non-commercial). This crate is therefore licensed
//! `CC-BY-NC-3.0-IGO`, **not** the workspace Apache-2.0, and must **not** be
//! linked into any commercial / paid build. It is consumed only behind the
//! [`xalen-stars`](../xalen_stars/index.html) `hip-catalog` feature.
//!
//! This crate intentionally contains **only data** (the [`GeneratedStar`]
//! record type and the [`GENERATED_CATALOG`] array). The precession / proper-
//! motion math lives in `xalen-stars` (Apache-2.0) via an extension trait, so
//! the restricted asset stays cleanly isolated in one package.

use serde::{Deserialize, Serialize};

/// A fixed star generated from the Hipparcos Main Catalogue (CDS I/239).
///
/// Every field is derived from a real `hip_main.dat` record (see the
/// `catalog_generated.rs` header for full provenance). `name`, `constellation`
/// and `nature` are populated ONLY where a curated traditional name joined to
/// this HIP via the IAU Catalog of Star Names; otherwise they are `None`/`""`.
///
/// Positions are J2000.0 ecliptic (the Hipparcos J1991.25 epoch is propagated
/// forward 8.75 yr by proper motion before the equatorial→ecliptic rotation).
///
/// This is a **pure-data** type. The epoch-propagation methods
/// (`longitude_at_epoch`, `ecliptic_at_epoch`, …) are provided by the
/// `GeneratedStarExt` extension trait in the `xalen-stars` crate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedStar {
    /// Hipparcos Catalogue (HIP) identifier.
    pub hip: u32,
    /// Traditional name, if a curated/IAU name maps to this HIP.
    pub name: Option<&'static str>,
    /// Constellation (only set when a curated name is present).
    pub constellation: &'static str,
    pub longitude_j2000: f64, // degrees ecliptic
    pub latitude_j2000: f64,  // degrees ecliptic
    pub magnitude: f64,
    /// Ptolemaic planetary nature (only set when a curated name is present).
    pub nature: &'static str,
    /// Proper motion in ecliptic longitude (milliarcseconds per year).
    pub pm_lon_mas_per_year: f64,
    /// Proper motion in ecliptic latitude (milliarcseconds per year).
    pub pm_lat_mas_per_year: f64,
}

#[rustfmt::skip]
mod catalog_generated;
pub use catalog_generated::{GENERATED_CATALOG, GENERATED_STAR_COUNT};
