//! External star catalog loader — load stars from CSV/TSV files at runtime.
//!
//! This module allows loading the full Hipparcos catalog (118,218 stars) or
//! any custom star catalog from CSV files, and merging them with the built-in
//! 108-star catalog.
//!
//! # CSV Format
//!
//! ```text
//! name,ra_deg,dec_deg,magnitude,pm_ra,pm_dec,ecl_lon,ecl_lat
//! ```
//!
//! - `name`: Star name (string, may be quoted if it contains commas)
//! - `ra_deg`: Right ascension in degrees (J2000)
//! - `dec_deg`: Declination in degrees (J2000)
//! - `magnitude`: Visual magnitude
//! - `pm_ra`: Proper motion in RA (mas/yr)
//! - `pm_dec`: Proper motion in Dec (mas/yr)
//! - `ecl_lon`: Ecliptic longitude in degrees (J2000)
//! - `ecl_lat`: Ecliptic latitude in degrees (J2000)
//!
//! Lines starting with `#` are treated as comments and skipped.
//! The first line is treated as a header and skipped if it starts with a letter.

use std::path::Path;

/// A fixed star loaded from an external catalog file.
///
/// Unlike the built-in `FixedStar` (which uses `&'static str` for names),
/// this struct owns its name string so it can be loaded at runtime.
#[derive(Debug, Clone)]
pub struct CatalogStar {
    pub name: String,
    pub longitude_j2000: f64,
    pub latitude_j2000: f64,
    pub magnitude: f64,
    /// Proper motion in ecliptic longitude (milliarcseconds per year).
    pub pm_lon_mas_per_year: f64,
    /// Proper motion in ecliptic latitude (milliarcseconds per year).
    pub pm_lat_mas_per_year: f64,
    /// Right ascension in degrees (J2000), if available from the source file.
    pub ra_deg: Option<f64>,
    /// Declination in degrees (J2000), if available from the source file.
    pub dec_deg: Option<f64>,
}

/// IAU 2006 J2000 general-precession linear rate (50.28796″/yr); see
/// `lib.rs::PRECESSION_DEG_PER_YEAR` for the provenance note.
///
/// Retained for the precession-rate regression test below; the actual
/// precession is now the rigorous IAU 2006/P03 rotation (see
/// `crate::precessed_ecliptic_of_date`), not this scalar.
#[allow(dead_code)]
const PRECESSION_DEG_PER_YEAR: f64 = 50.28796 / 3600.0;

impl CatalogStar {
    /// Ecliptic longitude (degrees, `[0, 360)`) at a given decimal year,
    /// rigorously precessed from J2000 with the IAU 2006/P03 rotation plus
    /// proper motion. See `crate::precessed_ecliptic_of_date` for the method
    /// and its Swiss-validated accuracy. (The previous planar `longitude +=
    /// rate·dt` model held ecliptic latitude FIXED, misplacing high-latitude
    /// stars by up to ~0.06°/1000 yr vs Swiss for Vega.)
    pub fn longitude_at_epoch(&self, year: f64) -> f64 {
        self.ecliptic_at_epoch(year).0
    }

    /// Ecliptic latitude (degrees, signed) at a given decimal year, now
    /// precession-coupled (not proper-motion-only).
    pub fn latitude_at_epoch(&self, year: f64) -> f64 {
        self.ecliptic_at_epoch(year).1
    }

    /// Both ecliptic coordinates at once (degrees): `(longitude, latitude)`.
    pub fn ecliptic_at_epoch(&self, year: f64) -> (f64, f64) {
        crate::precessed_ecliptic_of_date(
            self.longitude_j2000,
            self.latitude_j2000,
            self.pm_lon_mas_per_year,
            self.pm_lat_mas_per_year,
            year,
        )
    }

    /// Ecliptic longitude at a given Julian Date.
    pub fn longitude_at_jd(&self, jd: f64) -> f64 {
        let year = 2000.0 + (jd - 2_451_545.0) / 365.25;
        self.longitude_at_epoch(year)
    }
}

/// Errors that can occur when loading a star catalog.
#[derive(Debug)]
pub enum CatalogError {
    /// An I/O error occurred reading the file.
    IoError(std::io::Error),
    /// A line in the CSV file could not be parsed.
    ParseError { line_number: usize, message: String },
}

impl std::fmt::Display for CatalogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CatalogError::IoError(e) => write!(f, "I/O error: {e}"),
            CatalogError::ParseError {
                line_number,
                message,
            } => {
                write!(f, "parse error at line {line_number}: {message}")
            }
        }
    }
}

impl std::error::Error for CatalogError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CatalogError::IoError(e) => Some(e),
            CatalogError::ParseError { .. } => None,
        }
    }
}

impl From<std::io::Error> for CatalogError {
    fn from(e: std::io::Error) -> Self {
        CatalogError::IoError(e)
    }
}

/// Load a star catalog from a CSV file.
///
/// The expected format is:
/// ```text
/// name,ra_deg,dec_deg,magnitude,pm_ra,pm_dec,ecl_lon,ecl_lat
/// ```
///
/// Lines starting with `#` are skipped.  The first non-comment line is
/// skipped if it appears to be a header (first field starts with a letter).
///
/// Proper motion values `pm_ra` and `pm_dec` are in equatorial coordinates
/// (mas/yr) as typically provided by Hipparcos.  They are stored as ecliptic
/// proper motion approximations (the conversion is approximate for the
/// purpose of astrological position tracking).
pub fn load_catalog_from_csv(path: &Path) -> Result<Vec<CatalogStar>, CatalogError> {
    let content = std::fs::read_to_string(path)?;
    load_catalog_from_str(&content)
}

/// Load a star catalog from a CSV string (same format as the file loader).
///
/// This is useful for testing or embedding catalog data.
pub fn load_catalog_from_str(content: &str) -> Result<Vec<CatalogStar>, CatalogError> {
    let mut stars = Vec::new();
    let mut skipped_header = false;

    for (idx, line) in content.lines().enumerate() {
        let line_number = idx + 1;
        let trimmed = line.trim();

        // Skip empty lines and comments.
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Skip header line (first non-comment line that starts with a letter).
        if !skipped_header {
            if trimmed.starts_with(|c: char| c.is_ascii_alphabetic()) {
                skipped_header = true;
                continue;
            }
            skipped_header = true;
        }

        let fields: Vec<&str> = trimmed.split(',').collect();
        if fields.len() < 8 {
            return Err(CatalogError::ParseError {
                line_number,
                message: format!("expected 8 fields, got {}", fields.len()),
            });
        }

        let name = fields[0].trim().trim_matches('"').to_string();

        let ra_deg = parse_field(fields[1], line_number, "ra_deg")?;
        let dec_deg = parse_field(fields[2], line_number, "dec_deg")?;
        let magnitude = parse_field(fields[3], line_number, "magnitude")?;
        let pm_ra = parse_field(fields[4], line_number, "pm_ra")?;
        let pm_dec = parse_field(fields[5], line_number, "pm_dec")?;
        let ecl_lon = parse_field(fields[6], line_number, "ecl_lon")?;
        let ecl_lat = parse_field(fields[7], line_number, "ecl_lat")?;

        // Approximate equatorial -> ecliptic PM conversion.
        // For astrological purposes this is sufficient; the obliquity-based
        // rotation is small for most stars.
        let obliquity_rad = 23.4393_f64.to_radians();
        let pm_lon = pm_ra * obliquity_rad.cos() + pm_dec * obliquity_rad.sin();
        let pm_lat = -pm_ra * obliquity_rad.sin() + pm_dec * obliquity_rad.cos();

        stars.push(CatalogStar {
            name,
            longitude_j2000: ecl_lon,
            latitude_j2000: ecl_lat,
            magnitude,
            pm_lon_mas_per_year: pm_lon,
            pm_lat_mas_per_year: pm_lat,
            ra_deg: Some(ra_deg),
            dec_deg: Some(dec_deg),
        });
    }

    Ok(stars)
}

fn parse_field(s: &str, line_number: usize, field_name: &str) -> Result<f64, CatalogError> {
    s.trim()
        .parse::<f64>()
        .map_err(|_| CatalogError::ParseError {
            line_number,
            message: format!("invalid {field_name}: '{}'", s.trim()),
        })
}

/// Merge stars loaded from an external catalog with the built-in catalog.
///
/// If a star in the external catalog has the same name (case-insensitive) as
/// a built-in star, the external entry takes precedence (allowing users to
/// override built-in positions with more precise data).
///
/// The returned vector contains `CatalogStar` entries for all stars.
pub fn merge_catalogs(builtin: &[super::FixedStar], external: &[CatalogStar]) -> Vec<CatalogStar> {
    // Build a set of external star names (lowercased) for dedup.
    let external_names: std::collections::HashSet<String> =
        external.iter().map(|s| s.name.to_lowercase()).collect();

    let mut merged: Vec<CatalogStar> = Vec::with_capacity(builtin.len() + external.len());

    // Add built-in stars that are NOT overridden by external entries.
    for star in builtin {
        if !external_names.contains(&star.name.to_lowercase()) {
            merged.push(CatalogStar {
                name: star.name.to_string(),
                longitude_j2000: star.longitude_j2000,
                latitude_j2000: star.latitude_j2000,
                magnitude: star.magnitude,
                pm_lon_mas_per_year: star.pm_lon_mas_per_year,
                pm_lat_mas_per_year: star.pm_lat_mas_per_year,
                ra_deg: None,
                dec_deg: None,
            });
        }
    }

    // Add all external stars.
    merged.extend_from_slice(external);

    merged
}

/// Find stars in a catalog by name (case-insensitive).
pub fn find_in_catalog<'a>(catalog: &'a [CatalogStar], name: &str) -> Option<&'a CatalogStar> {
    catalog.iter().find(|s| s.name.eq_ignore_ascii_case(name))
}

/// Find stars within a given orb of a longitude.
pub fn find_conjunctions_in_catalog(
    catalog: &[CatalogStar],
    planet_lon_deg: f64,
    orb_deg: f64,
    year: f64,
) -> Vec<(&CatalogStar, f64)> {
    catalog
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

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_CSV: &str = "\
name,ra_deg,dec_deg,magnitude,pm_ra,pm_dec,ecl_lon,ecl_lat
Sirius,101.287,−16.716,−1.46,−546.01,−1223.07,104.07,−39.60
Canopus,95.988,−52.696,−0.74,19.93,23.24,105.28,−75.75
Arcturus,213.915,19.182,−0.05,−1093.45,−1999.40,204.07,30.77
";

    // Replace the Unicode minus signs with ASCII for parsing
    fn normalize_csv(s: &str) -> String {
        s.replace('\u{2212}', "-")
    }

    #[test]
    fn load_catalog_basic() {
        let csv = normalize_csv(SAMPLE_CSV);
        let stars = load_catalog_from_str(&csv).unwrap();
        assert_eq!(stars.len(), 3, "Should load 3 stars from sample CSV");
        assert_eq!(stars[0].name, "Sirius");
        assert_eq!(stars[1].name, "Canopus");
        assert_eq!(stars[2].name, "Arcturus");
    }

    #[test]
    fn load_catalog_magnitudes() {
        let csv = normalize_csv(SAMPLE_CSV);
        let stars = load_catalog_from_str(&csv).unwrap();
        assert!((stars[0].magnitude - (-1.46)).abs() < 0.01, "Sirius mag");
        assert!((stars[1].magnitude - (-0.74)).abs() < 0.01, "Canopus mag");
    }

    #[test]
    fn load_catalog_ecliptic_coords() {
        let csv = normalize_csv(SAMPLE_CSV);
        let stars = load_catalog_from_str(&csv).unwrap();
        assert!(
            (stars[0].longitude_j2000 - 104.07).abs() < 0.01,
            "Sirius ecl lon"
        );
        assert!(
            (stars[0].latitude_j2000 - (-39.60)).abs() < 0.01,
            "Sirius ecl lat"
        );
    }

    #[test]
    fn load_catalog_with_comments() {
        let csv = "\
# This is a comment
name,ra_deg,dec_deg,magnitude,pm_ra,pm_dec,ecl_lon,ecl_lat
# Another comment
Vega,279.234,38.783,0.03,201.0,287.5,285.45,61.73
";
        let stars = load_catalog_from_str(csv).unwrap();
        assert_eq!(stars.len(), 1);
        assert_eq!(stars[0].name, "Vega");
    }

    #[test]
    fn load_catalog_no_header() {
        // If the first data line starts with a digit or minus, it should be parsed.
        let csv = "99999,100.0,-20.0,5.0,0.0,0.0,100.0,-20.0\n";
        let stars = load_catalog_from_str(csv).unwrap();
        assert_eq!(stars.len(), 1);
        assert_eq!(stars[0].name, "99999");
    }

    #[test]
    fn load_catalog_error_too_few_fields() {
        let csv = "name,ra,dec\nBadStar,100.0,-20.0\n";
        let result = load_catalog_from_str(csv);
        assert!(result.is_err());
        if let Err(CatalogError::ParseError { line_number, .. }) = result {
            assert_eq!(line_number, 2);
        }
    }

    #[test]
    fn load_catalog_error_bad_number() {
        let csv = "name,ra_deg,dec_deg,magnitude,pm_ra,pm_dec,ecl_lon,ecl_lat\n\
                   BadStar,abc,-20.0,5.0,0.0,0.0,100.0,-20.0\n";
        let result = load_catalog_from_str(csv);
        assert!(result.is_err());
    }

    #[test]
    fn precession_applied() {
        // On-ecliptic (lat = 0), zero-pm star: the rigorous IAU 2006/P03
        // rotation's longitude advance is within ~0.0003 deg of the
        // leading-order linear rate over a century, but it is the full rotation,
        // not the planar formula. (For off-ecliptic stars the two diverge much
        // more — that divergence is the bug the rotation fixes.)
        let csv = "name,ra_deg,dec_deg,magnitude,pm_ra,pm_dec,ecl_lon,ecl_lat\n\
                   TestStar,0.0,0.0,5.0,0.0,0.0,100.0,0.0\n";
        let stars = load_catalog_from_str(csv).unwrap();
        let star = &stars[0];
        let lon_2100 = star.longitude_at_epoch(2100.0);
        let expected = 100.0 + PRECESSION_DEG_PER_YEAR * 100.0;
        assert!(
            (lon_2100 - expected).abs() < 0.001,
            "100yr precession: expected ~{expected}, got {lon_2100}"
        );
    }

    #[test]
    fn merge_catalogs_basic() {
        let csv = normalize_csv(SAMPLE_CSV);
        let external = load_catalog_from_str(&csv).unwrap();
        let merged = merge_catalogs(&crate::CATALOG, &external);

        // Should have all built-in stars minus duplicates plus external.
        // Sirius, Canopus, Arcturus exist in built-in (108) and external (3).
        // So merged = 108 - 3 + 3 = 108.
        assert_eq!(
            merged.len(),
            108,
            "Merged should have 108 stars (3 replaced)"
        );
    }

    #[test]
    fn merge_catalogs_external_overrides() {
        let csv = "name,ra_deg,dec_deg,magnitude,pm_ra,pm_dec,ecl_lon,ecl_lat\n\
                   Sirius,101.0,-16.0,-1.50,0.0,0.0,999.0,0.0\n";
        let external = load_catalog_from_str(csv).unwrap();
        let merged = merge_catalogs(&crate::CATALOG, &external);

        let sirius = find_in_catalog(&merged, "Sirius").unwrap();
        // Should have the external value (999.0), not the built-in (104.07).
        assert!(
            (sirius.longitude_j2000 - 999.0).abs() < 0.01,
            "External should override built-in: got {}",
            sirius.longitude_j2000
        );
    }

    #[test]
    fn merge_catalogs_adds_new() {
        let csv = "name,ra_deg,dec_deg,magnitude,pm_ra,pm_dec,ecl_lon,ecl_lat\n\
                   NewStar,50.0,10.0,4.0,0.0,0.0,50.0,10.0\n";
        let external = load_catalog_from_str(csv).unwrap();
        let merged = merge_catalogs(&crate::CATALOG, &external);

        assert_eq!(merged.len(), 109, "Should have 108 + 1 new star");
        let new = find_in_catalog(&merged, "NewStar");
        assert!(new.is_some());
    }

    #[test]
    fn find_in_catalog_case_insensitive() {
        let csv = "name,ra_deg,dec_deg,magnitude,pm_ra,pm_dec,ecl_lon,ecl_lat\n\
                   TestStar,50.0,10.0,4.0,0.0,0.0,50.0,10.0\n";
        let stars = load_catalog_from_str(csv).unwrap();
        assert!(find_in_catalog(&stars, "teststar").is_some());
        assert!(find_in_catalog(&stars, "TESTSTAR").is_some());
        assert!(find_in_catalog(&stars, "TestStar").is_some());
        assert!(find_in_catalog(&stars, "nonexistent").is_none());
    }

    #[test]
    fn find_conjunctions_in_loaded_catalog() {
        let csv = "name,ra_deg,dec_deg,magnitude,pm_ra,pm_dec,ecl_lon,ecl_lat\n\
                   Star1,0.0,0.0,1.0,0.0,0.0,100.0,0.0\n\
                   Star2,0.0,0.0,2.0,0.0,0.0,200.0,0.0\n\
                   Star3,0.0,0.0,3.0,0.0,0.0,101.5,0.0\n";
        let stars = load_catalog_from_str(csv).unwrap();
        let hits = find_conjunctions_in_catalog(&stars, 100.5, 2.0, 2000.0);
        assert_eq!(hits.len(), 2, "Should find 2 stars near 100.5 deg");
        let names: Vec<&str> = hits.iter().map(|(s, _)| s.name.as_str()).collect();
        assert!(names.contains(&"Star1"));
        assert!(names.contains(&"Star3"));
    }

    #[test]
    fn empty_catalog() {
        let stars = load_catalog_from_str("").unwrap();
        assert!(stars.is_empty());
    }

    #[test]
    fn header_only_catalog() {
        let csv = "name,ra_deg,dec_deg,magnitude,pm_ra,pm_dec,ecl_lon,ecl_lat\n";
        let stars = load_catalog_from_str(csv).unwrap();
        assert!(stars.is_empty());
    }

    #[test]
    fn jd_based_longitude() {
        let csv = "name,ra_deg,dec_deg,magnitude,pm_ra,pm_dec,ecl_lon,ecl_lat\n\
                   TestStar,0.0,0.0,5.0,0.0,0.0,100.0,0.0\n";
        let stars = load_catalog_from_str(csv).unwrap();
        let star = &stars[0];
        // JD for J2000 = 2451545.0, epoch-based should be exactly 100.0
        let lon = star.longitude_at_jd(2_451_545.0);
        assert!(
            (lon - 100.0).abs() < 0.001,
            "At J2000 JD should give 100.0, got {lon}"
        );
    }
}
