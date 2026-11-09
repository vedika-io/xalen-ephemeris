//! Optional Hipparcos-derived fixed-star catalog **data** for XALEN Ephemeris.
//!
//! # Licence: non-commercial, unpublished add-on
//!
//! The catalogue embedded here is derived from the Hipparcos Main Catalogue
//! (CDS I/239, `hip_main.dat`, (C) ESA 1997), which ESA publishes under
//! **CC BY-NC 3.0 IGO** (non-commercial). This crate is therefore licensed
//! `CC-BY-NC-3.0-IGO`, **not** the workspace Apache-2.0, `publish = false`, and
//! must **not** be linked into any commercial / paid build.
//!
//! No published XALEN crate depends on it. The default star list of
//! [`xalen-stars`](../xalen_stars/index.html) (`BRIGHT_CATALOG`) is built from
//! the Yale Bright Star Catalogue and the IAU Catalog of Star Names. This crate
//! depends on `xalen-stars`; it exists for users who accept the
//! non-commercial terms and want the Hipparcos-derived list (8,870 stars,
//! Vmag <= 6.5), and for the Hipparcos-based cross-validation tests that live
//! in this crate.
//!
//! The record type [`GeneratedStar`] is pure data; the epoch-propagation
//! methods come from the [`GeneratedStarExt`] trait, which reuses the
//! Apache-2.0 precession math of `xalen-stars`.

use serde::{Deserialize, Serialize};
use xalen_stars::FixedStar;

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
/// (`longitude_at_epoch`, `ecliptic_at_epoch`, …) are provided by
/// [`GeneratedStarExt`].
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

/// Julian Date of the J2000.0 epoch (2000-01-01 12:00 TT).
const J2000_JD: f64 = 2_451_545.0;

/// Epoch-propagation methods for the Hipparcos [`GeneratedStar`] data type.
///
/// The rotation is `xalen_stars::precessed_ecliptic_of_date` (IAU 2006/P03
/// precession plus linear proper motion), the same code path that
/// `xalen_stars::BrightStar` uses. Bring the trait into scope with
/// `use xalen_stars_hip_data::GeneratedStarExt;`.
pub trait GeneratedStarExt {
    /// Ecliptic longitude (degrees, `[0, 360)`) at a given decimal year.
    fn longitude_at_epoch(&self, year: f64) -> f64;

    /// Ecliptic latitude (degrees, signed) at a given decimal year.
    fn latitude_at_epoch(&self, year: f64) -> f64;

    /// Both ecliptic coordinates at once (degrees): `(longitude, latitude)`.
    fn ecliptic_at_epoch(&self, year: f64) -> (f64, f64);

    /// Ecliptic longitude at a given Julian Date.
    fn longitude_at_jd(&self, jd: f64) -> f64;

    /// Adapt a named HIP-derived star into the [`FixedStar`] shape of
    /// `xalen-stars`. `name` supplies the canonical `'static` name string.
    fn to_fixed_star(&self, name: &'static str) -> FixedStar;
}

impl GeneratedStarExt for GeneratedStar {
    fn to_fixed_star(&self, name: &'static str) -> FixedStar {
        FixedStar {
            name,
            constellation: self.constellation,
            longitude_j2000: self.longitude_j2000,
            latitude_j2000: self.latitude_j2000,
            magnitude: self.magnitude,
            nature: self.nature,
            pm_lon_mas_per_year: self.pm_lon_mas_per_year,
            pm_lat_mas_per_year: self.pm_lat_mas_per_year,
        }
    }

    fn longitude_at_epoch(&self, year: f64) -> f64 {
        self.ecliptic_at_epoch(year).0
    }

    fn latitude_at_epoch(&self, year: f64) -> f64 {
        self.ecliptic_at_epoch(year).1
    }

    fn ecliptic_at_epoch(&self, year: f64) -> (f64, f64) {
        xalen_stars::precessed_ecliptic_of_date(
            self.longitude_j2000,
            self.latitude_j2000,
            self.pm_lon_mas_per_year,
            self.pm_lat_mas_per_year,
            year,
        )
    }

    fn longitude_at_jd(&self, jd: f64) -> f64 {
        let year = 2000.0 + (jd - J2000_JD) / 365.25;
        self.longitude_at_epoch(year)
    }
}

/// Find a generated (HIP) star by its Hipparcos identifier.
pub fn find_generated_by_hip(hip: u32) -> Option<&'static GeneratedStar> {
    GENERATED_CATALOG
        .binary_search_by_key(&hip, |s| s.hip)
        .ok()
        .map(|i| &GENERATED_CATALOG[i])
}

/// Find a generated star by traditional name (case-insensitive). Only the
/// IAU-joined stars carry a name.
pub fn find_generated_by_name(name: &str) -> Option<&'static GeneratedStar> {
    GENERATED_CATALOG
        .iter()
        .find(|s| s.name.is_some_and(|n| n.eq_ignore_ascii_case(name)))
}

/// Find conjunctions in the Hipparcos-derived catalog (all 8,870 stars).
///
/// Returns `(star, separation_deg)` for every generated star within `orb_deg`
/// of `planet_lon_deg` at the given epoch year.
pub fn find_conjunctions_expanded(
    planet_lon_deg: f64,
    orb_deg: f64,
    year: f64,
) -> Vec<(&'static GeneratedStar, f64)> {
    GENERATED_CATALOG
        .iter()
        .filter_map(|star| {
            let star_lon = star.longitude_at_epoch(year);
            let diff = (planet_lon_deg - star_lon).rem_euclid(360.0);
            let dist = if diff > 180.0 { 360.0 - diff } else { diff };
            if dist <= orb_deg {
                Some((star, dist))
            } else {
                None
            }
        })
        .collect()
}
