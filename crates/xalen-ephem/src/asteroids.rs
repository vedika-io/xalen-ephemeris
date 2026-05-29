//! Simplified asteroid ephemeris for 17 astrologically significant asteroids.
//!
//! Uses Keplerian orbital elements at epoch J2000.0, solving Kepler's
//! equation via Newton-Raphson (same approach as the chiron module).
//! Geocentric positions are obtained by subtracting Earth's VSOP87
//! heliocentric position.
//!
//! # Coverage
//!
//! **Main belt (big 4):** Ceres, Pallas, Juno, Vesta
//! **Main belt (additional):** Hygeia, Astraea, Psyche, Eros, Lilith (1181)
//! **Centaurs:** Pholus, Nessus
//! **Trans-Neptunian:** Eris, Sedna, Makemake, Haumea
//!
//! Chiron is a centaur but has its own dedicated module (`chiron.rs`) with
//! multi-epoch osculating elements for better accuracy.
//!
//! # Accuracy
//!
//! Main belt: a few arcminutes over a decade from epoch, degrading further
//! out due to neglected perturbations (primarily Jupiter).
//!
//! Centaurs (Pholus, Nessus): ~0.5-2° over a decade. Their orbits cross
//! giant planet orbits and accumulate perturbation errors faster.
//!
//! TNOs (Eris, Sedna, etc.): ~0.1-0.5° over decades near epoch due to
//! very slow motion, but they drift on century timescales.
//!
//! # Sources
//!
//! Orbital elements sourced from JPL Small-Body Database (sbdb.jpl.nasa.gov),
//! osculating elements at epoch J2000.0 (JD 2451545.0) or nearest available
//! epoch, with mean daily motion computed from the semi-major axis via
//! Kepler's third law: n = 0.9856076686 / a^(3/2) deg/day.

use crate::provider::EphemerisError;
use serde::{Deserialize, Serialize};
use xalen_coords::EclipticPosition;
use xalen_time::{JdTT, JulianDay};

// ── Asteroid enum ─────────────────────────────────────────────────────

/// Astrologically significant asteroids and minor planets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Asteroid {
    // ── Main belt (big 4) ──
    /// 1 Ceres (dwarf planet). Period ~4.60 yr.
    Ceres,
    /// 2 Pallas. Period ~4.61 yr. Highly inclined orbit (34.83°).
    Pallas,
    /// 3 Juno. Period ~4.36 yr.
    Juno,
    /// 4 Vesta. Period ~3.63 yr. Brightest asteroid.
    Vesta,

    // ── Main belt (additional) ──
    /// 10 Hygeia. Period ~5.56 yr. Fourth-largest asteroid.
    Hygeia,
    /// 5 Astraea. Period ~4.13 yr. First asteroid found after the big 4.
    Astraea,
    /// 16 Psyche. Period ~5.00 yr. Metallic M-type asteroid.
    Psyche,
    /// 433 Eros. Period ~1.76 yr. Near-Earth asteroid, first visited by spacecraft.
    Eros,
    /// 1181 Lilith. Period ~4.36 yr. Not to be confused with the lunar apogee "Black Moon Lilith".
    LilithAsteroid,

    // ── Centaurs ──
    /// 5145 Pholus. Period ~92 yr. Orbit crosses Saturn and Neptune.
    Pholus,
    /// 7066 Nessus. Period ~122 yr. Orbit crosses Saturn, Uranus, and Neptune.
    Nessus,

    // ── Trans-Neptunian Objects ──
    /// 136199 Eris (dwarf planet). Period ~559 yr. Most massive known TNO.
    Eris,
    /// 90377 Sedna. Period ~12,900 yr. Extreme distant orbit (a ~550 AU).
    Sedna,
    /// 136472 Makemake (dwarf planet). Period ~306 yr.
    Makemake,
    /// 136108 Haumea (dwarf planet). Period ~284 yr. Rapid rotation.
    Haumea,
}

impl Asteroid {
    /// The original four major asteroids (backward compatibility).
    pub const BIG_FOUR: &[Asteroid] = &[
        Asteroid::Ceres,
        Asteroid::Pallas,
        Asteroid::Juno,
        Asteroid::Vesta,
    ];

    /// All four major asteroids (alias for backward compatibility).
    pub const ALL: &[Asteroid] = Self::BIG_FOUR;

    /// All main belt asteroids (big 4 + additional).
    pub const MAIN_BELT: &[Asteroid] = &[
        Asteroid::Ceres,
        Asteroid::Pallas,
        Asteroid::Juno,
        Asteroid::Vesta,
        Asteroid::Hygeia,
        Asteroid::Astraea,
        Asteroid::Psyche,
        Asteroid::Eros,
        Asteroid::LilithAsteroid,
    ];

    /// Centaur asteroids (excluding Chiron, which has its own module).
    pub const CENTAURS: &[Asteroid] = &[Asteroid::Pholus, Asteroid::Nessus];

    /// Trans-Neptunian Objects (dwarf planets and detached objects).
    pub const TNOS: &[Asteroid] = &[
        Asteroid::Eris,
        Asteroid::Sedna,
        Asteroid::Makemake,
        Asteroid::Haumea,
    ];

    /// All 15 asteroids in this module.
    pub const ALL_EXTENDED: &[Asteroid] = &[
        // Main belt
        Asteroid::Ceres,
        Asteroid::Pallas,
        Asteroid::Juno,
        Asteroid::Vesta,
        Asteroid::Hygeia,
        Asteroid::Astraea,
        Asteroid::Psyche,
        Asteroid::Eros,
        Asteroid::LilithAsteroid,
        // Centaurs
        Asteroid::Pholus,
        Asteroid::Nessus,
        // TNOs
        Asteroid::Eris,
        Asteroid::Sedna,
        Asteroid::Makemake,
        Asteroid::Haumea,
    ];

    /// Common name.
    pub fn name(&self) -> &'static str {
        match self {
            Asteroid::Ceres => "Ceres",
            Asteroid::Pallas => "Pallas",
            Asteroid::Juno => "Juno",
            Asteroid::Vesta => "Vesta",
            Asteroid::Hygeia => "Hygeia",
            Asteroid::Astraea => "Astraea",
            Asteroid::Psyche => "Psyche",
            Asteroid::Eros => "Eros",
            Asteroid::LilithAsteroid => "Lilith",
            Asteroid::Pholus => "Pholus",
            Asteroid::Nessus => "Nessus",
            Asteroid::Eris => "Eris",
            Asteroid::Sedna => "Sedna",
            Asteroid::Makemake => "Makemake",
            Asteroid::Haumea => "Haumea",
        }
    }

    /// Minor planet number.
    pub fn number(&self) -> u32 {
        match self {
            Asteroid::Ceres => 1,
            Asteroid::Pallas => 2,
            Asteroid::Juno => 3,
            Asteroid::Vesta => 4,
            Asteroid::Hygeia => 10,
            Asteroid::Astraea => 5,
            Asteroid::Psyche => 16,
            Asteroid::Eros => 433,
            Asteroid::LilithAsteroid => 1181,
            Asteroid::Pholus => 5145,
            Asteroid::Nessus => 7066,
            Asteroid::Eris => 136199,
            Asteroid::Sedna => 90377,
            Asteroid::Makemake => 136472,
            Asteroid::Haumea => 136108,
        }
    }

    /// Orbital category for this asteroid.
    pub fn category(&self) -> &'static str {
        match self {
            Asteroid::Ceres
            | Asteroid::Pallas
            | Asteroid::Juno
            | Asteroid::Vesta
            | Asteroid::Hygeia
            | Asteroid::Astraea
            | Asteroid::Psyche
            | Asteroid::LilithAsteroid => "Main Belt",
            Asteroid::Eros => "Near-Earth",
            Asteroid::Pholus | Asteroid::Nessus => "Centaur",
            Asteroid::Eris | Asteroid::Sedna | Asteroid::Makemake | Asteroid::Haumea => {
                "Trans-Neptunian"
            }
        }
    }
}

impl std::fmt::Display for Asteroid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}) {}", self.number(), self.name())
    }
}

// ── Orbital elements ──────────────────────────────────────────────────

/// Keplerian orbital elements at a fixed epoch.
struct OrbitalElements {
    /// Semi-major axis (AU).
    a: f64,
    /// Eccentricity.
    e: f64,
    /// Inclination (degrees).
    i_deg: f64,
    /// Longitude of ascending node (degrees).
    omega_node_deg: f64,
    /// Argument of perihelion (degrees).
    omega_peri_deg: f64,
    /// Mean anomaly at epoch J2000.0 (degrees).
    m0_deg: f64,
    /// Mean daily motion (degrees/day).
    n_deg_per_day: f64,
}

/// Compute mean daily motion from semi-major axis using Kepler's third law.
///
/// n = k / a^(3/2) where k = 0.9856076686 deg/day (Gaussian gravitational constant).
fn mean_motion(a: f64) -> f64 {
    0.9856076686 / a.powf(1.5)
}

/// J2000.0 osculating orbital elements for each asteroid.
///
/// Sources:
/// - JPL Small-Body Database (sbdb.jpl.nasa.gov)
/// - JPL Horizons System (ssd.jpl.nasa.gov/horizons/)
/// - Meeus, *Astronomical Algorithms*, Table 40.a (big 4)
///
/// Mean daily motion computed via Kepler's third law: n = 0.9856076686 / a^(3/2)
fn elements(asteroid: Asteroid) -> OrbitalElements {
    match asteroid {
        // ── Main belt (big 4) — same values as before ──
        Asteroid::Ceres => OrbitalElements {
            a: 2.7675,
            e: 0.0758,
            i_deg: 10.59,
            omega_node_deg: 80.33,
            omega_peri_deg: 72.58,
            m0_deg: 77.37,
            n_deg_per_day: mean_motion(2.7675), // ~0.2142
        },
        Asteroid::Pallas => OrbitalElements {
            a: 2.7721,
            e: 0.2305,
            i_deg: 34.83,
            omega_node_deg: 173.09,
            omega_peri_deg: 310.15,
            m0_deg: 259.82,
            n_deg_per_day: mean_motion(2.7721), // ~0.2139
        },
        Asteroid::Juno => OrbitalElements {
            a: 2.6700,
            e: 0.2562,
            i_deg: 12.97,
            omega_node_deg: 169.91,
            omega_peri_deg: 248.41,
            m0_deg: 115.42,
            n_deg_per_day: mean_motion(2.6700), // ~0.2259
        },
        Asteroid::Vesta => OrbitalElements {
            a: 2.3615,
            e: 0.0887,
            i_deg: 7.14,
            omega_node_deg: 103.85,
            omega_peri_deg: 149.83,
            m0_deg: 20.86,
            n_deg_per_day: mean_motion(2.3615), // ~0.2717
        },

        // ── Main belt (additional) ──
        //
        // 10 Hygeia — JPL SBDB, elements propagated to J2000.0
        Asteroid::Hygeia => OrbitalElements {
            a: 3.1476,
            e: 0.1082,
            i_deg: 3.83,
            omega_node_deg: 283.12,
            omega_peri_deg: 312.61,
            m0_deg: 347.83,
            n_deg_per_day: mean_motion(3.1476), // ~0.1769
        },
        // 5 Astraea — JPL SBDB, elements propagated to J2000.0
        Asteroid::Astraea => OrbitalElements {
            a: 2.5769,
            e: 0.1875,
            i_deg: 5.36,
            omega_node_deg: 141.45,
            omega_peri_deg: 359.35,
            m0_deg: 40.92,
            n_deg_per_day: mean_motion(2.5769), // ~0.2383
        },
        // 16 Psyche — JPL SBDB
        Asteroid::Psyche => OrbitalElements {
            a: 2.9227,
            e: 0.1339,
            i_deg: 3.10,
            omega_node_deg: 150.35,
            omega_peri_deg: 228.05,
            m0_deg: 318.95,
            n_deg_per_day: mean_motion(2.9227), // ~0.1973
        },
        // 433 Eros — JPL SBDB (near-Earth, higher eccentricity)
        Asteroid::Eros => OrbitalElements {
            a: 1.4583,
            e: 0.2229,
            i_deg: 10.83,
            omega_node_deg: 304.32,
            omega_peri_deg: 178.64,
            m0_deg: 208.12,
            n_deg_per_day: mean_motion(1.4583), // ~0.5597
        },
        // 1181 Lilith — JPL SBDB, elements propagated to J2000.0
        Asteroid::LilithAsteroid => OrbitalElements {
            a: 2.6646,
            e: 0.1933,
            i_deg: 5.59,
            omega_node_deg: 260.56,
            omega_peri_deg: 156.62,
            m0_deg: 329.40,
            n_deg_per_day: mean_motion(2.6646), // ~0.2266
        },

        // ── Centaurs ──
        //
        // 5145 Pholus — JPL SBDB (Saturn/Neptune crosser)
        Asteroid::Pholus => OrbitalElements {
            a: 20.3297,
            e: 0.5728,
            i_deg: 24.69,
            omega_node_deg: 119.35,
            omega_peri_deg: 354.87,
            m0_deg: 72.89,
            n_deg_per_day: mean_motion(20.3297), // ~0.01075
        },
        // 7066 Nessus — JPL SBDB, elements propagated to J2000.0
        Asteroid::Nessus => OrbitalElements {
            a: 24.5166,
            e: 0.5177,
            i_deg: 15.64,
            omega_node_deg: 31.29,
            omega_peri_deg: 170.37,
            m0_deg: 23.75,
            n_deg_per_day: mean_motion(24.5166), // ~0.00812
        },

        // ── Trans-Neptunian Objects ──
        //
        // 136199 Eris — JPL SBDB (most massive known dwarf planet)
        Asteroid::Eris => OrbitalElements {
            a: 67.781,
            e: 0.4407,
            i_deg: 44.04,
            omega_node_deg: 35.87,
            omega_peri_deg: 151.43,
            m0_deg: 205.99,
            n_deg_per_day: mean_motion(67.781), // ~0.001765
        },
        // 90377 Sedna — JPL SBDB latest solution, elements propagated to J2000.0
        Asteroid::Sedna => OrbitalElements {
            a: 549.54,
            e: 0.8613,
            i_deg: 11.93,
            omega_node_deg: 144.48,
            omega_peri_deg: 311.01,
            m0_deg: 357.88,
            n_deg_per_day: mean_motion(549.54), // ~0.0000765
        },
        // 136472 Makemake — JPL SBDB
        Asteroid::Makemake => OrbitalElements {
            a: 45.430,
            e: 0.1613,
            i_deg: 28.98,
            omega_node_deg: 79.38,
            omega_peri_deg: 296.53,
            m0_deg: 155.12,
            n_deg_per_day: mean_motion(45.430), // ~0.003218
        },
        // 136108 Haumea — JPL SBDB
        Asteroid::Haumea => OrbitalElements {
            a: 43.218,
            e: 0.1952,
            i_deg: 28.19,
            omega_node_deg: 121.90,
            omega_peri_deg: 239.18,
            m0_deg: 218.45,
            n_deg_per_day: mean_motion(43.218), // ~0.003468
        },
    }
}

// ── Kepler solver ─────────────────────────────────────────────────────

/// Solve Kepler's equation E - e*sin(E) = M via Newton-Raphson.
/// Returns eccentric anomaly E in radians.
fn solve_kepler(m: f64, e: f64) -> Result<f64, EphemerisError> {
    let mut ea = if e < 0.8 { m } else { std::f64::consts::PI };

    for _ in 0..50 {
        let delta = ea - e * ea.sin() - m;
        let deriv = 1.0 - e * ea.cos();
        if deriv.abs() < 1e-15 {
            return Err(EphemerisError::ComputationFailed(
                "Kepler equation: derivative vanished (asteroid)".into(),
            ));
        }
        let correction = delta / deriv;
        ea -= correction;
        if correction.abs() < 1e-12 {
            return Ok(ea);
        }
    }

    Err(EphemerisError::ComputationFailed(
        "Kepler equation did not converge in 50 iterations (asteroid)".into(),
    ))
}

// ── Position computation ──────────────────────────────────────────────

/// Compute the geocentric ecliptic position of an asteroid at a given Julian Date (TT).
///
/// Uses simplified Keplerian orbital elements at J2000.0 epoch.
/// Geocentric correction via VSOP87 Earth position.
///
/// Returns position with longitude in radians [0, TAU), latitude in radians,
/// distance in AU.
pub fn asteroid_position(
    asteroid: Asteroid,
    jd_tt: JdTT,
) -> Result<EclipticPosition, EphemerisError> {
    let d2r = std::f64::consts::PI / 180.0;
    let el = elements(asteroid);

    // Convert elements to radians
    let i = el.i_deg * d2r;
    let omega_node = el.omega_node_deg * d2r;
    let omega_peri = el.omega_peri_deg * d2r;
    let m0 = el.m0_deg * d2r;
    let n = el.n_deg_per_day * d2r;

    // Time since J2000.0 in days
    let dt_days = jd_tt.as_f64() - 2_451_545.0;

    // Mean anomaly at requested epoch
    let mean_anom = (m0 + n * dt_days).rem_euclid(std::f64::consts::TAU);

    // Solve Kepler's equation
    let ecc_anom = solve_kepler(mean_anom, el.e)?;

    // True anomaly from eccentric anomaly
    let cos_e = ecc_anom.cos();
    let sin_e = ecc_anom.sin();
    let true_anom = ((1.0 - el.e * el.e).sqrt() * sin_e).atan2(cos_e - el.e);

    // Heliocentric distance
    let r = el.a * (1.0 - el.e * cos_e);

    // Position in the orbital plane -> heliocentric ecliptic
    let cos_om = omega_node.cos();
    let sin_om = omega_node.sin();
    let cos_i = i.cos();
    let sin_i = i.sin();

    let u = omega_peri + true_anom;
    let cos_u = u.cos();
    let sin_u = u.sin();

    // Heliocentric ecliptic rectangular coordinates (J2000)
    let x_h = r * (cos_om * cos_u - sin_om * sin_u * cos_i);
    let y_h = r * (sin_om * cos_u + cos_om * sin_u * cos_i);
    let z_h = r * (sin_u * sin_i);

    // Geocentric: subtract Earth's heliocentric position (VSOP87)
    let earth = vsop87::vsop87a::earth(jd_tt.as_f64());

    let x_geo = x_h - earth.x;
    let y_geo = y_h - earth.y;
    let z_geo = z_h - earth.z;

    let mut pos = xalen_coords::cartesian_to_ecliptic(&xalen_coords::CartesianPosition {
        x: x_geo,
        y: y_geo,
        z: z_geo,
    });

    // Apply general precession (J2000 -> equinox-of-date)
    let t = jd_tt.julian_centuries_from_j2000();
    pos.longitude += xalen_coords::general_precession_longitude(t);

    Ok(pos.normalize())
}

/// Compute positions for all four major asteroids at a given epoch.
///
/// This is the original function for backward compatibility. For the
/// extended set, use [`all_extended_positions`].
pub fn all_asteroid_positions(
    jd_tt: JdTT,
) -> Result<Vec<(Asteroid, EclipticPosition)>, EphemerisError> {
    Asteroid::ALL
        .iter()
        .map(|&ast| {
            let pos = asteroid_position(ast, jd_tt)?;
            Ok((ast, pos))
        })
        .collect()
}

/// Compute positions for all 15 asteroids (main belt + centaurs + TNOs).
pub fn all_extended_positions(
    jd_tt: JdTT,
) -> Result<Vec<(Asteroid, EclipticPosition)>, EphemerisError> {
    Asteroid::ALL_EXTENDED
        .iter()
        .map(|&ast| {
            let pos = asteroid_position(ast, jd_tt)?;
            Ok((ast, pos))
        })
        .collect()
}

/// Compute positions for a specific category of asteroids.
pub fn positions_by_category(
    asteroids: &[Asteroid],
    jd_tt: JdTT,
) -> Result<Vec<(Asteroid, EclipticPosition)>, EphemerisError> {
    asteroids
        .iter()
        .map(|&ast| {
            let pos = asteroid_position(ast, jd_tt)?;
            Ok((ast, pos))
        })
        .collect()
}

/// Approximate orbital period in years for an asteroid.
pub fn orbital_period_years(asteroid: Asteroid) -> f64 {
    let el = elements(asteroid);
    360.0 / (el.n_deg_per_day * 365.25)
}

/// Check if an asteroid is likely in retrograde at a given epoch by
/// comparing positions one day apart.
///
/// Returns `true` if the geocentric ecliptic longitude decreased
/// (retrograde motion).
pub fn is_retrograde(asteroid: Asteroid, jd_tt: JdTT) -> Result<bool, EphemerisError> {
    let p1 = asteroid_position(asteroid, jd_tt)?;
    let p2 = asteroid_position(asteroid, JdTT(jd_tt.as_f64() + 1.0))?;
    let mut diff = p2.longitude - p1.longitude;
    if diff > std::f64::consts::PI {
        diff -= std::f64::consts::TAU;
    }
    if diff < -std::f64::consts::PI {
        diff += std::f64::consts::TAU;
    }
    Ok(diff < 0.0)
}

// ── External catalog loader ───────────────────────────────────────────

/// An asteroid loaded from an external orbital elements file.
///
/// Unlike the built-in `Asteroid` enum (which has hard-coded elements),
/// this struct carries its own orbital elements loaded at runtime.
#[derive(Debug, Clone)]
pub struct ExternalAsteroid {
    /// Name or designation.
    pub name: String,
    /// Epoch of the osculating elements (Julian Date, TT).
    pub epoch_jd: f64,
    /// Semi-major axis (AU).
    pub a: f64,
    /// Eccentricity.
    pub e: f64,
    /// Inclination (degrees).
    pub i_deg: f64,
    /// Argument of perihelion (degrees).
    pub omega_peri_deg: f64,
    /// Longitude of ascending node (degrees).
    pub omega_node_deg: f64,
    /// Mean anomaly at epoch (degrees).
    pub m0_deg: f64,
    /// Mean daily motion (degrees/day).  If zero or absent in the file,
    /// computed from `a` via Kepler's third law.
    pub n_deg_per_day: f64,
}

/// Errors that can occur when loading an asteroid catalog.
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

/// Load asteroid orbital elements from a CSV file.
///
/// The expected format is:
/// ```text
/// name,epoch_jd,a,e,i,omega,Omega,M0,n
/// ```
///
/// Where:
/// - `name`: Asteroid name or designation
/// - `epoch_jd`: Epoch of osculating elements (Julian Date)
/// - `a`: Semi-major axis (AU)
/// - `e`: Eccentricity
/// - `i`: Inclination (degrees)
/// - `omega`: Argument of perihelion (degrees)
/// - `Omega`: Longitude of ascending node (degrees)
/// - `M0`: Mean anomaly at epoch (degrees)
/// - `n`: Mean daily motion (degrees/day); if 0 or missing, computed from `a`
///
/// Lines starting with `#` are skipped.  The first non-comment line is
/// skipped if it appears to be a header.
pub fn load_asteroid_elements(
    path: &std::path::Path,
) -> Result<Vec<ExternalAsteroid>, CatalogError> {
    let content = std::fs::read_to_string(path)?;
    load_asteroid_elements_from_str(&content)
}

/// Load asteroid orbital elements from a CSV string.
pub fn load_asteroid_elements_from_str(
    content: &str,
) -> Result<Vec<ExternalAsteroid>, CatalogError> {
    let mut asteroids = Vec::new();
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
                message: format!("expected at least 8 fields, got {}", fields.len()),
            });
        }

        let name = fields[0].trim().trim_matches('"').to_string();

        let epoch_jd = parse_ast_field(fields[1], line_number, "epoch_jd")?;
        let a = parse_ast_field(fields[2], line_number, "a")?;
        let e = parse_ast_field(fields[3], line_number, "e")?;
        let i_deg = parse_ast_field(fields[4], line_number, "i")?;
        let omega_peri_deg = parse_ast_field(fields[5], line_number, "omega")?;
        let omega_node_deg = parse_ast_field(fields[6], line_number, "Omega")?;
        let m0_deg = parse_ast_field(fields[7], line_number, "M0")?;

        // n is optional (9th field); if absent or zero, compute from a.
        let n_deg_per_day = if fields.len() >= 9 {
            let n = parse_ast_field(fields[8], line_number, "n")?;
            if n.abs() < 1e-15 { mean_motion(a) } else { n }
        } else {
            mean_motion(a)
        };

        // Validate orbital elements.
        if a <= 0.0 {
            return Err(CatalogError::ParseError {
                line_number,
                message: format!("semi-major axis must be positive, got {a}"),
            });
        }
        if !(0.0..1.0).contains(&e) {
            return Err(CatalogError::ParseError {
                line_number,
                message: format!("eccentricity must be in [0, 1), got {e}"),
            });
        }

        asteroids.push(ExternalAsteroid {
            name,
            epoch_jd,
            a,
            e,
            i_deg,
            omega_peri_deg,
            omega_node_deg,
            m0_deg,
            n_deg_per_day,
        });
    }

    Ok(asteroids)
}

fn parse_ast_field(s: &str, line_number: usize, field_name: &str) -> Result<f64, CatalogError> {
    s.trim()
        .parse::<f64>()
        .map_err(|_| CatalogError::ParseError {
            line_number,
            message: format!("invalid {field_name}: '{}'", s.trim()),
        })
}

/// Compute the geocentric ecliptic position of an externally loaded asteroid.
///
/// Uses the same Kepler solver and coordinate transform as the built-in
/// asteroids.
pub fn external_asteroid_position(
    ast: &ExternalAsteroid,
    jd_tt: JdTT,
) -> Result<EclipticPosition, EphemerisError> {
    let d2r = std::f64::consts::PI / 180.0;

    let i = ast.i_deg * d2r;
    let omega_node = ast.omega_node_deg * d2r;
    let omega_peri = ast.omega_peri_deg * d2r;
    let m0 = ast.m0_deg * d2r;
    let n = ast.n_deg_per_day * d2r;

    let dt_days = jd_tt.as_f64() - ast.epoch_jd;
    let mean_anom = (m0 + n * dt_days).rem_euclid(std::f64::consts::TAU);

    let ecc_anom = solve_kepler(mean_anom, ast.e)?;

    let cos_e = ecc_anom.cos();
    let sin_e = ecc_anom.sin();
    let true_anom = ((1.0 - ast.e * ast.e).sqrt() * sin_e).atan2(cos_e - ast.e);

    let r = ast.a * (1.0 - ast.e * cos_e);

    let cos_om = omega_node.cos();
    let sin_om = omega_node.sin();
    let cos_i = i.cos();
    let sin_i = i.sin();

    let u = omega_peri + true_anom;
    let cos_u = u.cos();
    let sin_u = u.sin();

    let x_h = r * (cos_om * cos_u - sin_om * sin_u * cos_i);
    let y_h = r * (sin_om * cos_u + cos_om * sin_u * cos_i);
    let z_h = r * (sin_u * sin_i);

    let earth = vsop87::vsop87a::earth(jd_tt.as_f64());

    let x_geo = x_h - earth.x;
    let y_geo = y_h - earth.y;
    let z_geo = z_h - earth.z;

    let mut pos = xalen_coords::cartesian_to_ecliptic(&xalen_coords::CartesianPosition {
        x: x_geo,
        y: y_geo,
        z: z_geo,
    });

    let t = jd_tt.julian_centuries_from_j2000();
    pos.longitude += xalen_coords::general_precession_longitude(t);

    Ok(pos.normalize())
}

/// Compute the orbital period in years for an external asteroid.
pub fn external_orbital_period_years(ast: &ExternalAsteroid) -> f64 {
    360.0 / (ast.n_deg_per_day * 365.25)
}

// ── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use xalen_coords::RAD_TO_DEG;

    // ── Kepler solver tests ──

    #[test]
    fn kepler_circular_asteroid() {
        let m = 1.5;
        let e_anom = solve_kepler(m, 0.0).unwrap();
        assert!(
            (e_anom - m).abs() < 1e-12,
            "Circular orbit: E should equal M"
        );
    }

    #[test]
    fn kepler_convergence_pallas() {
        // Pallas has highest eccentricity among big 4 (0.2305)
        let m = 2.0;
        let ea = solve_kepler(m, 0.2305).unwrap();
        let residual = (ea - 0.2305 * ea.sin() - m).abs();
        assert!(residual < 1e-12, "Kepler residual too large: {residual}");
    }

    #[test]
    fn kepler_convergence_high_eccentricity() {
        // Sedna has very high eccentricity (0.8496)
        let m = 1.0;
        let ea = solve_kepler(m, 0.8496).unwrap();
        let residual = (ea - 0.8496 * ea.sin() - m).abs();
        assert!(
            residual < 1e-12,
            "Kepler residual for e=0.85 too large: {residual}"
        );
    }

    // ── Big 4 tests (preserve original test coverage) ──

    #[test]
    fn ceres_at_j2000_reasonable() {
        let pos = asteroid_position(Asteroid::Ceres, JdTT::J2000).unwrap();
        let lon = pos.longitude * RAD_TO_DEG;
        let lat = pos.latitude * RAD_TO_DEG;
        assert!(
            lon >= 0.0 && lon < 360.0,
            "Longitude should be in [0,360), got {lon}°"
        );
        assert!(
            lat.abs() < 15.0,
            "Ceres lat should be < 15° (incl 10.59°), got {lat}°"
        );
        assert!(
            pos.distance > 1.0 && pos.distance < 5.0,
            "Ceres distance should be ~1-5 AU, got {} AU",
            pos.distance
        );
    }

    #[test]
    fn pallas_at_j2000_reasonable() {
        let pos = asteroid_position(Asteroid::Pallas, JdTT::J2000).unwrap();
        let lat = pos.latitude * RAD_TO_DEG;
        assert!(
            lat.abs() < 40.0,
            "Pallas lat should be < 40° (incl 34.83°), got {lat}°"
        );
        assert!(
            pos.distance > 0.5 && pos.distance < 6.0,
            "Pallas distance should be ~0.5-6 AU, got {} AU",
            pos.distance
        );
    }

    #[test]
    fn juno_at_j2000_reasonable() {
        let pos = asteroid_position(Asteroid::Juno, JdTT::J2000).unwrap();
        let lat = pos.latitude * RAD_TO_DEG;
        assert!(
            lat.abs() < 18.0,
            "Juno lat should be < 18° (incl 12.97°), got {lat}°"
        );
        assert!(
            pos.distance > 0.5 && pos.distance < 5.0,
            "Juno distance should be ~0.5-5 AU, got {} AU",
            pos.distance
        );
    }

    #[test]
    fn vesta_at_j2000_reasonable() {
        let pos = asteroid_position(Asteroid::Vesta, JdTT::J2000).unwrap();
        let lat = pos.latitude * RAD_TO_DEG;
        assert!(
            lat.abs() < 12.0,
            "Vesta lat should be < 12° (incl 7.14°), got {lat}°"
        );
        assert!(
            pos.distance > 0.5 && pos.distance < 5.0,
            "Vesta distance should be ~0.5-5 AU, got {} AU",
            pos.distance
        );
    }

    #[test]
    fn all_asteroids_reasonable_speed() {
        for &ast in Asteroid::ALL {
            let p1 = asteroid_position(ast, JdTT(2451545.0)).unwrap();
            let p2 = asteroid_position(ast, JdTT(2451545.0 + 365.25)).unwrap();
            let mut diff = (p2.longitude - p1.longitude) * RAD_TO_DEG;
            if diff < -180.0 {
                diff += 360.0;
            }
            if diff > 180.0 {
                diff -= 360.0;
            }
            assert!(
                diff.abs() < 150.0 && diff.abs() > 20.0,
                "{}: annual motion should be moderate (20-150°), got {diff:.1}°",
                ast.name()
            );
        }
    }

    #[test]
    fn orbital_periods_correct() {
        for &(ast, expected_min, expected_max) in &[
            (Asteroid::Ceres, 4.4, 4.8),
            (Asteroid::Pallas, 4.4, 4.8),
            (Asteroid::Juno, 4.2, 4.6),
            (Asteroid::Vesta, 3.4, 3.9),
        ] {
            let period = orbital_period_years(ast);
            assert!(
                period > expected_min && period < expected_max,
                "{}: period should be {}-{} yr, got {:.2} yr",
                ast.name(),
                expected_min,
                expected_max,
                period
            );
        }
    }

    #[test]
    fn all_asteroid_positions_returns_four() {
        let all = all_asteroid_positions(JdTT::J2000).unwrap();
        assert_eq!(all.len(), 4, "Should return all 4 asteroids");
    }

    #[test]
    fn distance_reasonable_range_across_epochs() {
        for &ast in Asteroid::ALL {
            for jd in [2451545.0, 2455000.0, 2458849.0, 2460000.0] {
                let pos = asteroid_position(ast, JdTT(jd)).unwrap();
                assert!(
                    pos.distance > 0.3 && pos.distance < 7.0,
                    "{} at JD {}: distance {} AU out of range",
                    ast.name(),
                    jd,
                    pos.distance
                );
            }
        }
    }

    #[test]
    fn retrograde_detection_works() {
        for &ast in Asteroid::ALL {
            let _retro = is_retrograde(ast, JdTT::J2000).unwrap();
        }
    }

    #[test]
    fn ceres_epoch_2020() {
        let pos = asteroid_position(Asteroid::Ceres, JdTT(2458849.0)).unwrap();
        let lon = pos.longitude * RAD_TO_DEG;
        assert!(
            lon > 200.0 && lon < 360.0,
            "Ceres in early 2020 should be in ~200-360° range, got {lon:.1}°"
        );
    }

    #[test]
    fn asteroid_display() {
        assert_eq!(format!("{}", Asteroid::Ceres), "(1) Ceres");
        assert_eq!(format!("{}", Asteroid::Pallas), "(2) Pallas");
        assert_eq!(format!("{}", Asteroid::Juno), "(3) Juno");
        assert_eq!(format!("{}", Asteroid::Vesta), "(4) Vesta");
    }

    // ── New asteroid tests ──

    #[test]
    fn all_extended_returns_fifteen() {
        let all = all_extended_positions(JdTT::J2000).unwrap();
        assert_eq!(
            all.len(),
            15,
            "Should return all 15 asteroids, got {}",
            all.len()
        );
    }

    #[test]
    fn new_main_belt_at_j2000_reasonable() {
        for &ast in &[
            Asteroid::Hygeia,
            Asteroid::Astraea,
            Asteroid::Psyche,
            Asteroid::LilithAsteroid,
        ] {
            let pos = asteroid_position(ast, JdTT::J2000).unwrap();
            let lon = pos.longitude * RAD_TO_DEG;
            let lat = pos.latitude * RAD_TO_DEG;
            assert!(
                lon >= 0.0 && lon < 360.0,
                "{}: lon out of range: {lon}°",
                ast.name()
            );
            // Main belt inclinations are all < 6° for these => max lat < 10°
            assert!(
                lat.abs() < 12.0,
                "{}: lat should be < 12°, got {lat}°",
                ast.name()
            );
            // Main belt geocentric distance: ~1-5 AU
            assert!(
                pos.distance > 0.5 && pos.distance < 6.0,
                "{}: distance {} AU out of range",
                ast.name(),
                pos.distance
            );
        }
    }

    #[test]
    fn eros_at_j2000_near_earth() {
        let pos = asteroid_position(Asteroid::Eros, JdTT::J2000).unwrap();
        let lon = pos.longitude * RAD_TO_DEG;
        let lat = pos.latitude * RAD_TO_DEG;
        assert!(lon >= 0.0 && lon < 360.0, "Eros lon out of range: {lon}°");
        assert!(lat.abs() < 20.0, "Eros lat too large: {lat}°");
        // Eros is a near-Earth asteroid, geocentric distance ~0.15-2.5 AU
        assert!(
            pos.distance > 0.05 && pos.distance < 4.0,
            "Eros distance {} AU out of range",
            pos.distance
        );
    }

    #[test]
    fn eros_period_correct() {
        let period = orbital_period_years(Asteroid::Eros);
        assert!(
            period > 1.5 && period < 2.0,
            "Eros period should be ~1.76 yr, got {period:.2} yr"
        );
    }

    #[test]
    fn centaurs_at_j2000_reasonable() {
        for &ast in Asteroid::CENTAURS {
            let pos = asteroid_position(ast, JdTT::J2000).unwrap();
            let lon = pos.longitude * RAD_TO_DEG;
            let lat = pos.latitude * RAD_TO_DEG;
            assert!(
                lon >= 0.0 && lon < 360.0,
                "{}: lon out of range: {lon}°",
                ast.name()
            );
            // Pholus incl 24.69°, Nessus incl 15.65° => max lat can be up to ~25°
            assert!(
                lat.abs() < 35.0,
                "{}: lat should be < 35°, got {lat}°",
                ast.name()
            );
            // Centaurs: heliocentric ~8-30 AU, geocentric ~7-31 AU
            assert!(
                pos.distance > 3.0 && pos.distance < 40.0,
                "{}: distance {} AU out of range",
                ast.name(),
                pos.distance
            );
        }
    }

    #[test]
    fn centaur_periods_correct() {
        let pholus_period = orbital_period_years(Asteroid::Pholus);
        assert!(
            pholus_period > 80.0 && pholus_period < 100.0,
            "Pholus period should be ~92 yr, got {pholus_period:.1} yr"
        );

        let nessus_period = orbital_period_years(Asteroid::Nessus);
        assert!(
            nessus_period > 110.0 && nessus_period < 135.0,
            "Nessus period should be ~122 yr, got {nessus_period:.1} yr"
        );
    }

    #[test]
    fn tnos_at_j2000_reasonable() {
        for &ast in Asteroid::TNOS {
            let pos = asteroid_position(ast, JdTT::J2000).unwrap();
            let lon = pos.longitude * RAD_TO_DEG;
            let lat = pos.latitude * RAD_TO_DEG;
            assert!(
                lon >= 0.0 && lon < 360.0,
                "{}: lon out of range: {lon}°",
                ast.name()
            );
            // TNOs have moderate-to-high inclinations (up to 44° for Eris)
            assert!(
                lat.abs() < 50.0,
                "{}: lat should be < 50°, got {lat}°",
                ast.name()
            );
            // TNOs: very distant, geocentric ~25-600 AU
            assert!(
                pos.distance > 15.0 && pos.distance < 700.0,
                "{}: distance {} AU out of range",
                ast.name(),
                pos.distance
            );
        }
    }

    #[test]
    fn tno_periods_correct() {
        let eris_period = orbital_period_years(Asteroid::Eris);
        assert!(
            eris_period > 500.0 && eris_period < 620.0,
            "Eris period should be ~559 yr, got {eris_period:.0} yr"
        );

        let makemake_period = orbital_period_years(Asteroid::Makemake);
        assert!(
            makemake_period > 280.0 && makemake_period < 330.0,
            "Makemake period should be ~306 yr, got {makemake_period:.0} yr"
        );

        let haumea_period = orbital_period_years(Asteroid::Haumea);
        assert!(
            haumea_period > 260.0 && haumea_period < 310.0,
            "Haumea period should be ~284 yr, got {haumea_period:.0} yr"
        );

        let sedna_period = orbital_period_years(Asteroid::Sedna);
        assert!(
            sedna_period > 11000.0 && sedna_period < 14000.0,
            "Sedna period should be ~12,900 yr, got {sedna_period:.0} yr"
        );
    }

    #[test]
    fn tno_slow_motion() {
        // TNOs move so slowly that over 1 year the position barely changes
        for &ast in Asteroid::TNOS {
            let p1 = asteroid_position(ast, JdTT(2451545.0)).unwrap();
            let p2 = asteroid_position(ast, JdTT(2451545.0 + 365.25)).unwrap();
            let mut diff = (p2.longitude - p1.longitude) * RAD_TO_DEG;
            if diff < -180.0 {
                diff += 360.0;
            }
            if diff > 180.0 {
                diff -= 360.0;
            }
            // TNOs move only ~0.01-1.5° per year (including retrograde)
            assert!(
                diff.abs() < 5.0,
                "{}: TNO annual motion should be very small, got {diff:.3}°",
                ast.name()
            );
        }
    }

    #[test]
    fn all_extended_distance_across_epochs() {
        for &ast in Asteroid::ALL_EXTENDED {
            let pos = asteroid_position(ast, JdTT::J2000).unwrap();
            let is_tno = matches!(
                ast,
                Asteroid::Eris | Asteroid::Sedna | Asteroid::Makemake | Asteroid::Haumea
            );
            let is_centaur = matches!(ast, Asteroid::Pholus | Asteroid::Nessus);

            if is_tno {
                assert!(
                    pos.distance > 15.0 && pos.distance < 700.0,
                    "{} (TNO): distance {} AU out of range",
                    ast.name(),
                    pos.distance
                );
            } else if is_centaur {
                assert!(
                    pos.distance > 3.0 && pos.distance < 40.0,
                    "{} (centaur): distance {} AU out of range",
                    ast.name(),
                    pos.distance
                );
            } else {
                assert!(
                    pos.distance > 0.05 && pos.distance < 7.0,
                    "{} (main belt): distance {} AU out of range",
                    ast.name(),
                    pos.distance
                );
            }
        }
    }

    #[test]
    fn category_assignment_correct() {
        assert_eq!(Asteroid::Ceres.category(), "Main Belt");
        assert_eq!(Asteroid::Eros.category(), "Near-Earth");
        assert_eq!(Asteroid::Pholus.category(), "Centaur");
        assert_eq!(Asteroid::Eris.category(), "Trans-Neptunian");
    }

    #[test]
    fn asteroid_numbers_correct() {
        assert_eq!(Asteroid::Ceres.number(), 1);
        assert_eq!(Asteroid::Eros.number(), 433);
        assert_eq!(Asteroid::LilithAsteroid.number(), 1181);
        assert_eq!(Asteroid::Pholus.number(), 5145);
        assert_eq!(Asteroid::Nessus.number(), 7066);
        assert_eq!(Asteroid::Eris.number(), 136199);
        assert_eq!(Asteroid::Sedna.number(), 90377);
    }

    #[test]
    fn extended_asteroid_display() {
        assert_eq!(format!("{}", Asteroid::Eros), "(433) Eros");
        assert_eq!(format!("{}", Asteroid::Pholus), "(5145) Pholus");
        assert_eq!(format!("{}", Asteroid::Eris), "(136199) Eris");
        assert_eq!(format!("{}", Asteroid::Sedna), "(90377) Sedna");
    }

    #[test]
    fn positions_by_category_works() {
        let centaur_positions = positions_by_category(Asteroid::CENTAURS, JdTT::J2000).unwrap();
        assert_eq!(centaur_positions.len(), 2);

        let tno_positions = positions_by_category(Asteroid::TNOS, JdTT::J2000).unwrap();
        assert_eq!(tno_positions.len(), 4);
    }

    #[test]
    fn mean_motion_kepler_third_law() {
        // Verify mean_motion matches Kepler's third law: P^2 = a^3
        // n (deg/day) = 360 / P (days)
        // P (days) = sqrt(a^3) * 365.25
        // n = 360 / (sqrt(a^3) * 365.25) = 0.9856076686 / a^1.5

        let a = 2.7675; // Ceres
        let n = mean_motion(a);
        let period_days = 360.0 / n;
        let period_years = period_days / 365.25;
        assert!(
            (period_years - a.powf(1.5)).abs() < 0.01,
            "Kepler's third law violated: period {period_years:.2} yr, a^1.5 = {:.2}",
            a.powf(1.5)
        );
    }

    #[test]
    fn retrograde_detection_extended() {
        // Just verify that retrograde detection works for all extended asteroids
        for &ast in Asteroid::ALL_EXTENDED {
            let _retro = is_retrograde(ast, JdTT::J2000).unwrap();
        }
    }

    // ── External catalog loader tests ──

    const SAMPLE_ASTEROID_CSV: &str = "\
name,epoch_jd,a,e,i,omega,Omega,M0,n
Ceres,2451545.0,2.7675,0.0758,10.59,72.58,80.33,77.37,0
Pallas,2451545.0,2.7721,0.2305,34.83,310.15,173.09,259.82,0
TestAsteroid,2451545.0,3.0,0.1,5.0,100.0,200.0,45.0,0.19
";

    #[test]
    fn load_asteroid_elements_basic() {
        let asteroids = load_asteroid_elements_from_str(SAMPLE_ASTEROID_CSV).unwrap();
        assert_eq!(asteroids.len(), 3, "Should load 3 asteroids");
        assert_eq!(asteroids[0].name, "Ceres");
        assert_eq!(asteroids[1].name, "Pallas");
        assert_eq!(asteroids[2].name, "TestAsteroid");
    }

    #[test]
    fn load_asteroid_elements_values() {
        let asteroids = load_asteroid_elements_from_str(SAMPLE_ASTEROID_CSV).unwrap();
        let ceres = &asteroids[0];
        assert!((ceres.a - 2.7675).abs() < 0.001);
        assert!((ceres.e - 0.0758).abs() < 0.001);
        assert!((ceres.i_deg - 10.59).abs() < 0.01);
        assert!((ceres.m0_deg - 77.37).abs() < 0.01);
    }

    #[test]
    fn load_asteroid_auto_mean_motion() {
        // When n=0, mean_motion should be computed from a.
        let asteroids = load_asteroid_elements_from_str(SAMPLE_ASTEROID_CSV).unwrap();
        let ceres = &asteroids[0];
        let expected_n = mean_motion(2.7675);
        assert!(
            (ceres.n_deg_per_day - expected_n).abs() < 1e-8,
            "Auto-computed n should be {expected_n}, got {}",
            ceres.n_deg_per_day
        );
    }

    #[test]
    fn load_asteroid_explicit_mean_motion() {
        let csv = "name,epoch_jd,a,e,i,omega,Omega,M0,n\n\
                   Fast,2451545.0,1.5,0.1,5.0,100.0,200.0,45.0,0.5\n";
        let asteroids = load_asteroid_elements_from_str(csv).unwrap();
        assert!((asteroids[0].n_deg_per_day - 0.5).abs() < 1e-10);
    }

    #[test]
    fn load_asteroid_no_n_column() {
        // 8 fields (no n) should still work.
        let csv = "name,epoch_jd,a,e,i,omega,Omega,M0\n\
                   NoN,2451545.0,2.5,0.1,5.0,100.0,200.0,45.0\n";
        let asteroids = load_asteroid_elements_from_str(csv).unwrap();
        assert_eq!(asteroids.len(), 1);
        let expected_n = mean_motion(2.5);
        assert!((asteroids[0].n_deg_per_day - expected_n).abs() < 1e-8);
    }

    #[test]
    fn load_asteroid_with_comments() {
        let csv = "# Asteroid elements file\n\
                   name,epoch_jd,a,e,i,omega,Omega,M0,n\n\
                   # Main belt\n\
                   Vesta,2451545.0,2.3615,0.0887,7.14,149.83,103.85,20.86,0\n";
        let asteroids = load_asteroid_elements_from_str(csv).unwrap();
        assert_eq!(asteroids.len(), 1);
        assert_eq!(asteroids[0].name, "Vesta");
    }

    #[test]
    fn load_asteroid_error_negative_a() {
        let csv = "name,epoch_jd,a,e,i,omega,Omega,M0,n\n\
                   Bad,2451545.0,-1.0,0.1,5.0,100.0,200.0,45.0,0\n";
        let result = load_asteroid_elements_from_str(csv);
        assert!(result.is_err());
    }

    #[test]
    fn load_asteroid_error_bad_eccentricity() {
        let csv = "name,epoch_jd,a,e,i,omega,Omega,M0,n\n\
                   Bad,2451545.0,2.0,1.5,5.0,100.0,200.0,45.0,0\n";
        let result = load_asteroid_elements_from_str(csv);
        assert!(result.is_err());
    }

    #[test]
    fn load_asteroid_error_too_few_fields() {
        let csv = "name,epoch_jd,a\nBad,2451545.0,2.0\n";
        let result = load_asteroid_elements_from_str(csv);
        assert!(result.is_err());
    }

    #[test]
    fn external_asteroid_position_ceres() {
        // Load Ceres with the same elements as the built-in, verify positions agree.
        let asteroids = load_asteroid_elements_from_str(SAMPLE_ASTEROID_CSV).unwrap();
        let ext_ceres = &asteroids[0];

        let ext_pos = external_asteroid_position(ext_ceres, JdTT::J2000).unwrap();
        let built_pos = asteroid_position(Asteroid::Ceres, JdTT::J2000).unwrap();

        let lon_ext = ext_pos.longitude * RAD_TO_DEG;
        let lon_built = built_pos.longitude * RAD_TO_DEG;

        // Should be very close since same elements.
        assert!(
            (lon_ext - lon_built).abs() < 1.0,
            "External Ceres lon {lon_ext:.2} should be close to built-in {lon_built:.2}"
        );
    }

    #[test]
    fn external_asteroid_reasonable_position() {
        let csv = "name,epoch_jd,a,e,i,omega,Omega,M0,n\n\
                   TestAst,2451545.0,3.0,0.1,5.0,100.0,200.0,45.0,0\n";
        let asteroids = load_asteroid_elements_from_str(csv).unwrap();
        let pos = external_asteroid_position(&asteroids[0], JdTT::J2000).unwrap();
        let lon = pos.longitude * RAD_TO_DEG;
        let lat = pos.latitude * RAD_TO_DEG;

        assert!(lon >= 0.0 && lon < 360.0, "Lon out of range: {lon}");
        assert!(lat.abs() < 15.0, "Lat too large for i=5 deg: {lat}");
        assert!(
            pos.distance > 0.5 && pos.distance < 7.0,
            "Distance out of range: {}",
            pos.distance
        );
    }

    #[test]
    fn external_orbital_period() {
        let csv = "name,epoch_jd,a,e,i,omega,Omega,M0,n\n\
                   Ceres,2451545.0,2.7675,0.0758,10.59,72.58,80.33,77.37,0\n";
        let asteroids = load_asteroid_elements_from_str(csv).unwrap();
        let period = external_orbital_period_years(&asteroids[0]);
        assert!(
            period > 4.4 && period < 4.8,
            "Ceres period should be ~4.6 yr, got {period:.2} yr"
        );
    }

    #[test]
    fn external_asteroid_empty_file() {
        let asteroids = load_asteroid_elements_from_str("").unwrap();
        assert!(asteroids.is_empty());
    }

    #[test]
    fn external_asteroid_header_only() {
        let csv = "name,epoch_jd,a,e,i,omega,Omega,M0,n\n";
        let asteroids = load_asteroid_elements_from_str(csv).unwrap();
        assert!(asteroids.is_empty());
    }
}
