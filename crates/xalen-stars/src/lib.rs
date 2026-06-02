use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

pub mod catalog;
#[rustfmt::skip]
pub mod catalog_generated;

pub use catalog_generated::{GENERATED_CATALOG, GENERATED_STAR_COUNT};

/// General precession in longitude — IAU 2006 J2000 linear rate
/// (5028.796195″/century → 50.28796″/yr).
///
/// HISTORICAL NOTE / now used only for documentation and the legacy
/// regression guards: the catalog NO LONGER precesses stars by adding this
/// scalar to ecliptic longitude while holding latitude fixed. That planar
/// approximation FROZE ecliptic latitude, so it misplaced high-ecliptic-latitude
/// stars — for Vega (+61.7°) by ~0.06°/1000 yr vs Swiss, almost all of it in the
/// latitude it refused to move; for low-latitude stars the longitude term was
/// nearly right but latitude still drifted. Positions are now precessed with the
/// SOFA-validated IAU 2006/P03 rotation via [`precessed_ecliptic_of_date`] (see
/// that function), which couples longitude and latitude and matches Swiss to
/// < 0.1′. The rate is kept as a named constant because it remains the
/// leading-order longitude term and the precession-rate regression tests assert
/// against it. (Only referenced from `#[cfg(test)]` now, hence the allow.)
#[allow(dead_code)]
const PRECESSION_DEG_PER_YEAR: f64 = 50.28796 / 3600.0;

/// Julian Date of the J2000.0 epoch (2000-01-01 12:00 TT).
const J2000_JD: f64 = 2_451_545.0;

/// Rigorously precess a J2000.0 ecliptic star position (with proper motion) to
/// the **mean ecliptic of date** for a given decimal `year`.
///
/// Pipeline (matches `xalen-ayanamsa`'s `fixed_point_apparent_longitude` and the
/// VSOP/asteroid/Chiron position paths):
///
/// 1. Apply proper motion linearly in the J2000 ecliptic frame (mas/yr → deg).
/// 2. Build the IAU 2006/P03 precession rotation WITHOUT frame bias
///    ([`xalen_coords::precession_matrix_p03_nobias`]) — the catalogue equinox
///    is the dynamical mean equinox of J2000, so the bias must NOT be applied.
/// 3. Rotate the J2000 ecliptic direction to the ecliptic of date through the
///    rigorous Cartesian pipeline ([`xalen_coords::precess_ecliptic_to_of_date`]),
///    which couples longitude AND latitude (the missing coupling was the old
///    planar bug).
///
/// The Julian-century argument `t` is taken as `(year − 2000)/100`, consistent
/// with this module's `year ↔ jd` convention (`year = 2000 + (jd − 2451545)/365.25`),
/// so `*_at_epoch` and `*_at_jd` round-trip exactly. The returned longitude is
/// wrapped to `[0, 360)`; latitude is signed.
///
/// Validated against `pyswisseph` `swe.fixstar2` (mean ecliptic of date,
/// `FLG_SWIEPH | FLG_NONUT | FLG_NOABERR | FLG_NOGDEFL`) at 1000 CE and 3000 CE
/// for Polaris, Aldebaran, Spica, Antares, Regulus, Sirius and Pollux: worst
/// total separation 0.074′ (≈4.4″), all under 0.1′. See the
/// `precession_*_multi_epoch_*` tests.
fn precessed_ecliptic_of_date(
    longitude_j2000_deg: f64,
    latitude_j2000_deg: f64,
    pm_lon_mas_per_year: f64,
    pm_lat_mas_per_year: f64,
    year: f64,
) -> (f64, f64) {
    let dt = year - 2000.0;
    // Step 1: proper motion in the J2000 ecliptic frame (mas/yr → deg).
    let lon_pm = longitude_j2000_deg + pm_lon_mas_per_year * dt / 3_600_000.0;
    let lat_pm = latitude_j2000_deg + pm_lat_mas_per_year * dt / 3_600_000.0;

    // Step 2+3: rigorous IAU 2006/P03 rotation to the ecliptic of date.
    let t = dt / 100.0; // Julian centuries from J2000 (year convention).
    let pos = xalen_coords::EclipticPosition {
        longitude: lon_pm.to_radians(),
        latitude: lat_pm.to_radians(),
        distance: 1.0,
    };
    let prec = xalen_coords::precession_matrix_p03_nobias(t);
    let of_date = xalen_coords::precess_ecliptic_to_of_date(pos, prec, t);

    (
        of_date.longitude.to_degrees().rem_euclid(360.0),
        of_date.latitude.to_degrees(),
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixedStar {
    pub name: &'static str,
    pub constellation: &'static str,
    pub longitude_j2000: f64, // degrees ecliptic
    pub latitude_j2000: f64,  // degrees ecliptic
    pub magnitude: f64,
    pub nature: &'static str,
    /// Proper motion in ecliptic longitude (milliarcseconds per year).
    pub pm_lon_mas_per_year: f64,
    /// Proper motion in ecliptic latitude (milliarcseconds per year).
    pub pm_lat_mas_per_year: f64,
}

impl FixedStar {
    /// Ecliptic longitude (degrees, `[0, 360)`) at a given decimal `year`,
    /// rigorously precessed from J2000 with the IAU 2006/P03 rotation plus
    /// proper motion. See [`precessed_ecliptic_of_date`] for the method and its
    /// Swiss-validated accuracy (the previous planar `longitude += rate·dt`
    /// model held ecliptic latitude FIXED, misplacing high-latitude stars by
    /// up to ~0.06°/1000 yr vs Swiss for Vega).
    pub fn longitude_at_epoch(&self, year: f64) -> f64 {
        self.ecliptic_at_epoch(year).0
    }

    /// Ecliptic latitude (degrees, signed) at a given decimal `year`. Now
    /// precession-coupled (not proper-motion-only): the rigorous rotation
    /// changes latitude with epoch, matching Swiss Ephemeris.
    pub fn latitude_at_epoch(&self, year: f64) -> f64 {
        self.ecliptic_at_epoch(year).1
    }

    /// Both ecliptic coordinates at once (degrees): `(longitude, latitude)`.
    /// Computing them together avoids precessing the same star twice.
    pub fn ecliptic_at_epoch(&self, year: f64) -> (f64, f64) {
        precessed_ecliptic_of_date(
            self.longitude_j2000,
            self.latitude_j2000,
            self.pm_lon_mas_per_year,
            self.pm_lat_mas_per_year,
            year,
        )
    }

    pub fn longitude_at_jd(&self, jd: f64) -> f64 {
        let year = 2000.0 + (jd - J2000_JD) / 365.25;
        self.longitude_at_epoch(year)
    }
}

/// A fixed star generated from the Hipparcos Main Catalogue (CDS I/239).
///
/// Every field is derived from a real `hip_main.dat` record (see
/// `catalog_generated.rs` header for the full provenance). `name`, `constellation`
/// and `nature` are populated ONLY where a curated traditional name joined to
/// this HIP via the IAU Catalog of Star Names; otherwise they are `None`/`""`.
///
/// Positions are J2000.0 ecliptic (the Hipparcos J1991.25 epoch is propagated
/// forward 8.75 yr by proper motion before the equatorial→ecliptic rotation).
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

impl GeneratedStar {
    /// Ecliptic longitude (degrees, `[0, 360)`) at a given decimal year,
    /// rigorously precessed from J2000 with the IAU 2006/P03 rotation plus
    /// proper motion. See [`precessed_ecliptic_of_date`].
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
        precessed_ecliptic_of_date(
            self.longitude_j2000,
            self.latitude_j2000,
            self.pm_lon_mas_per_year,
            self.pm_lat_mas_per_year,
            year,
        )
    }

    /// Ecliptic longitude at a given Julian Date.
    pub fn longitude_at_jd(&self, jd: f64) -> f64 {
        let year = 2000.0 + (jd - J2000_JD) / 365.25;
        self.longitude_at_epoch(year)
    }
}

/// Total stars available across both layers: the curated `CATALOG` plus the
/// HIP-derived `GENERATED_CATALOG`. (The two overlap on the ~100 curated names
/// that joined to a HIP; this is the raw sum, not a de-duplicated count.)
pub fn expanded_star_count() -> usize {
    CATALOG.len() + GENERATED_CATALOG.len()
}

/// Find a generated (HIP) star by its Hipparcos identifier.
pub fn find_generated_by_hip(hip: u32) -> Option<&'static GeneratedStar> {
    GENERATED_CATALOG.iter().find(|s| s.hip == hip)
}

/// Find a generated star by traditional name (case-insensitive). Only the
/// IAU-joined stars carry a name.
pub fn find_generated_by_name(name: &str) -> Option<&'static GeneratedStar> {
    GENERATED_CATALOG
        .iter()
        .find(|s| s.name.is_some_and(|n| n.eq_ignore_ascii_case(name)))
}

/// Curated traditional name → Hipparcos (HIP) identifier, for the handful of
/// curated names whose spelling differs from the IAU-WGSN name in
/// `catalog_generated.rs` (so an exact-name join would miss).
///
/// This is the runtime counterpart to `tools/gen-star-catalog/gen_catalog.py`'s
/// `CURATED_TO_IAU` / `CURATED_TO_HIP` maps: it lets [`find_generated_for_curated`]
/// resolve these names to their validated Hipparcos row **by HIP number** even
/// if the generated catalog was produced without the alias join. Each HIP is the
/// documented identification of the curated star, sourced as follows:
///
/// | Curated name      | Star                | IAU-WGSN name    | HIP   |
/// |-------------------|---------------------|------------------|-------|
/// | `Zuben Elgenubi`  | α² Librae           | Zubenelgenubi    | 72622 |
/// | `Zuben Eschamali` | β Librae            | Zubeneschamali   | 74785 |
/// | `Bharani 41`      | 41 Arietis          | Bharani          | 13209 |
/// | `Lambda Orionis`  | λ Orionis           | Meissa           | 26207 |
/// | `Hyadum II`       | δ¹ Tauri            | Secunda Hyadum   | 20455 |
/// | `Al Jabhah`       | η Leonis (HR 4031)  | *(none — WGSN)*  | 49583 |
///
/// η Leonis has no IAU-WGSN proper name (WGSN named ζ Leonis "Adhafera" and
/// γ¹ Leonis "Algieba"; η Leonis is a distinct, unnamed star), so "Al Jabhah" is
/// a traditional designation resolved by HIP only. The position, magnitude and
/// proper motion still come entirely from the Hipparcos record for that HIP.
const CURATED_NAME_HIP: &[(&str, u32)] = &[
    ("zuben elgenubi", 72622),
    ("zuben eschamali", 74785),
    ("bharani 41", 13209),
    ("lambda orionis", 26207),
    ("hyadum ii", 20455),
    ("al jabhah", 49583),
];

/// Resolve a curated catalog entry to its validated Hipparcos-derived row.
///
/// Tries an exact (case-insensitive) name join first — this succeeds for the
/// ~100 names whose curated spelling matches the generated catalog (and, after
/// regeneration with the alias map, for the aliased names too). For the handful
/// of curated names whose spelling differs from the IAU-WGSN name, it falls back
/// to a HIP-number lookup via [`CURATED_NAME_HIP`], so the reconciliation is
/// correct regardless of how the generated catalog was produced. Returns `None`
/// only for genuine clusters (Pleiades, Praesepe) with no single HIP star.
fn find_generated_for_curated(name: &str) -> Option<&'static GeneratedStar> {
    if let Some(s) = find_generated_by_name(name) {
        return Some(s);
    }
    let lower = name.to_ascii_lowercase();
    CURATED_NAME_HIP
        .iter()
        .find(|(n, _)| *n == lower.as_str())
        .and_then(|&(_, hip)| find_generated_by_hip(hip))
}

impl GeneratedStar {
    /// Adapt a named HIP-derived star into the legacy [`FixedStar`] shape.
    ///
    /// Used to back the public [`find_by_name`] lookup with the validated
    /// Hipparcos catalog (sub-arcsecond vs Swiss) while preserving the
    /// `FixedStar` API that downstream callers depend on. The `name` argument
    /// supplies the canonical `'static` name string (the generated catalog's
    /// own `name` is also `'static`, but the curated spelling is authoritative
    /// for the public surface). Position, latitude, magnitude, nature,
    /// constellation and the ecliptic proper-motion components are taken from
    /// the validated generated record.
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
}

/// The public fixed-star catalog, reconciled against the validated Hipparcos
/// catalog.
///
/// For every curated [`CATALOG`] entry whose traditional name resolves to a
/// named star in the sub-arcsecond-accurate [`GENERATED_CATALOG`], the position,
/// latitude, magnitude, nature and ecliptic proper-motion components are
/// REPLACED by the validated Hipparcos-derived values; only the canonical name
/// (a `'static str`) is carried over from the curated entry. The name join is
/// resolved by [`find_generated_for_curated`], which tries an exact name match
/// and then a documented HIP-number alias ([`CURATED_NAME_HIP`]) for the handful
/// of curated names whose spelling differs from the IAU-WGSN name (e.g.
/// "Bharani 41" → Bharani/HIP 13209, "Lambda Orionis" → Meissa/HIP 26207).
///
/// The ONLY curated entries kept verbatim as a fallback are the two open
/// clusters — Pleiades and Praesepe — which have no single Hipparcos star.
///
/// This is the data the public [`find_by_name`] / [`find_conjunctions`] /
/// [`nakshatra_yogatara`] surface reads. The raw curated [`CATALOG`] previously
/// backed those lookups directly and carried gross J2000 longitude errors
/// (Polaris ~60°, Deneb ~30°, Dubhe ~31°); see the `swiss_star_oracle.rs`
/// integration test for the Swiss-Ephemeris ground truth this reconciliation
/// now matches.
static RECONCILED_CATALOG: LazyLock<Vec<FixedStar>> = LazyLock::new(|| {
    CATALOG
        .iter()
        .map(|curated| {
            match find_generated_for_curated(curated.name) {
                // Validated HIP data exists for this name — prefer it.
                Some(g) => g.to_fixed_star(curated.name),
                // No HIP match (an open cluster) — keep the curated entry.
                None => curated.clone(),
            }
        })
        .collect()
});

/// Case-insensitive name → index map into [`RECONCILED_CATALOG`].
static NAME_INDEX: LazyLock<HashMap<String, usize>> = LazyLock::new(|| {
    let mut idx = HashMap::with_capacity(RECONCILED_CATALOG.len());
    for (i, star) in RECONCILED_CATALOG.iter().enumerate() {
        // First spelling wins; the catalog has no duplicate names (asserted by
        // the `no_duplicate_names` test).
        idx.entry(star.name.to_ascii_lowercase()).or_insert(i);
    }
    idx
});

/// Find conjunctions in the **expanded** HIP catalog (all stars Vmag ≤ 6.5).
///
/// Returns `(star, separation_deg)` for every generated star within `orb_deg`
/// of `planet_lon_deg` at the given epoch year. Use this for breadth; use
/// [`find_conjunctions_at_epoch`] for the curated astrologically-named set only.
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

pub fn find_conjunctions(planet_lon_deg: f64, orb_deg: f64) -> Vec<(&'static FixedStar, f64)> {
    find_conjunctions_at_epoch(planet_lon_deg, orb_deg, 2000.0)
}

/// Conjunctions against the astrologically-named set, using validated
/// Hipparcos-derived positions (the reconciled catalog; see
/// [`RECONCILED_CATALOG`]). For breadth across all 8,870 HIP stars, use
/// [`find_conjunctions_expanded`].
pub fn find_conjunctions_at_epoch(
    planet_lon_deg: f64,
    orb_deg: f64,
    year: f64,
) -> Vec<(&'static FixedStar, f64)> {
    RECONCILED_CATALOG
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

/// Find a fixed star by traditional name (case-insensitive).
///
/// Returns positions backed by the validated Hipparcos [`GENERATED_CATALOG`]
/// (sub-arcsecond vs Swiss Ephemeris) for every named star — including the
/// curated names whose spelling differs from the IAU-WGSN name, resolved via
/// [`find_generated_for_curated`]. The only names that fall back to the curated
/// [`CATALOG`] coordinates are the two open clusters (Pleiades, Praesepe), which
/// have no single Hipparcos star. See [`RECONCILED_CATALOG`] for the rules.
pub fn find_by_name(name: &str) -> Option<&'static FixedStar> {
    NAME_INDEX
        .get(&name.to_ascii_lowercase())
        .map(|&i| &RECONCILED_CATALOG[i])
}

/// Maps a nakshatra index (0 = Ashwini, 26 = Revati) to its primary
/// reference star (yogatara). Returns `None` for invalid indices.
///
/// Star identifications follow the traditional Surya Siddhanta / Burgess
/// mapping used in formal Jyotish certification curricula. Where the
/// traditional yogatara is a faint star not in our catalog, the brightest
/// astrologically-used star in the same nakshatra zone is returned.
pub fn nakshatra_yogatara(nakshatra_index: usize) -> Option<&'static FixedStar> {
    // The mapping table uses the most widely accepted yogatara star for each
    // nakshatra. Names must match entries in CATALOG (case-insensitive).
    let star_name: &str = match nakshatra_index {
        0 => "Sheratan",          // Ashwini — beta Arietis
        1 => "Bharani 41",        // Bharani — 41 Arietis
        2 => "Alcyone",           // Krittika — eta Tauri (Pleiades brightest)
        3 => "Aldebaran",         // Rohini — alpha Tauri
        4 => "Lambda Orionis",    // Mrigashira — lambda Orionis (Meissa)
        5 => "Betelgeuse",        // Ardra — alpha Orionis
        6 => "Pollux",            // Punarvasu — beta Geminorum
        7 => "Asellus Australis", // Pushya — delta Cancri
        8 => "Alphard",           // Ashlesha — alpha Hydrae
        9 => "Regulus",           // Magha — alpha Leonis
        10 => "Zosma",            // Purva Phalguni — delta Leonis
        11 => "Denebola",         // Uttara Phalguni — beta Leonis
        12 => "Porrima",          // Hasta — gamma Virginis
        13 => "Spica",            // Chitra — alpha Virginis
        14 => "Arcturus",         // Swati — alpha Bootis
        15 => "Zuben Eschamali",  // Vishakha — beta Librae
        16 => "Dschubba",         // Anuradha — delta Scorpii
        17 => "Antares",          // Jyeshtha — alpha Scorpii
        18 => "Shaula",           // Mula — lambda Scorpii
        19 => "Kaus Australis",   // Purva Ashadha — epsilon Sagittarii (proxy)
        20 => "Nunki",            // Uttara Ashadha — sigma Sagittarii
        21 => "Altair",           // Shravana — alpha Aquilae
        22 => "Sadalsuud",        // Dhanishta — beta Aquarii (proxy for Delphini)
        23 => "Sadalmelik",       // Shatabhisha — alpha Aquarii (proxy for lambda)
        24 => "Markab",           // Purva Bhadrapada — alpha Pegasi
        25 => "Scheat",           // Uttara Bhadrapada — beta Pegasi
        26 => "Revati",           // Revati — zeta Piscium
        _ => return None,
    };
    find_by_name(star_name)
}

// ---------------------------------------------------------------------------
// CATALOG — 110 astrologically significant fixed stars
//
// Every entry carries J2000.0 ecliptic longitude / latitude, visual magnitude,
// Ptolemaic planetary nature, and proper-motion components in mas/yr.
//
// High proper motion values (> ~100 mas/yr) are set for Sirius, Arcturus,
// Procyon, Pollux, Aldebaran, and other well-measured stars. Stars where
// proper motion is astrologically negligible (< 50 mas/yr) use 0.0.
// ---------------------------------------------------------------------------
pub static CATALOG: &[FixedStar] = &[
    // ===== ORIGINAL 25 BRIGHT / ROYAL STARS ==============================
    FixedStar {
        name: "Aldebaran",
        constellation: "Taurus",
        longitude_j2000: 69.95,
        latitude_j2000: -5.47,
        magnitude: 0.87,
        nature: "Mars",
        pm_lon_mas_per_year: 62.8,
        pm_lat_mas_per_year: -189.4,
    },
    FixedStar {
        name: "Regulus",
        constellation: "Leo",
        longitude_j2000: 149.83,
        latitude_j2000: 0.47,
        magnitude: 1.36,
        nature: "Mars-Jupiter",
        pm_lon_mas_per_year: -248.7,
        pm_lat_mas_per_year: 5.6,
    },
    FixedStar {
        // Spica (alpha Virginis, HIP 65474). Ecliptic position and proper-motion
        // components derived from the Hipparcos/FK5 J2000.0 equatorial place
        // RA = 201.2982474 deg, Dec = -11.1613125 deg, pmRA* = -42.50 mas/yr,
        // pmDec = -31.73 mas/yr, rotated into the J2000 ecliptic frame with the
        // IAU 2006 mean obliquity (84381.406"). Carried to 5+ decimals so the
        // True Chitra ayanamsa (which anchors Spica at exactly 180 deg) is not
        // capped at the ~36" floor that the old 2-decimal longitude (203.83)
        // imposed. The previous pm components were the un-rotated equatorial
        // values mislabelled as ecliptic; they are now correctly rotated.
        name: "Spica",
        constellation: "Virgo",
        longitude_j2000: 203.841355,
        latitude_j2000: -2.054487,
        magnitude: 0.98,
        nature: "Venus-Mars",
        pm_lon_mas_per_year: -27.72,
        pm_lat_mas_per_year: -45.23,
    },
    FixedStar {
        name: "Antares",
        constellation: "Scorpio",
        longitude_j2000: 249.78,
        latitude_j2000: -4.57,
        magnitude: 1.06,
        nature: "Mars-Jupiter",
        pm_lon_mas_per_year: -10.2,
        pm_lat_mas_per_year: -23.2,
    },
    FixedStar {
        name: "Fomalhaut",
        constellation: "Pisces Australis",
        longitude_j2000: 334.17,
        latitude_j2000: -21.08,
        magnitude: 1.17,
        nature: "Venus-Mercury",
        pm_lon_mas_per_year: 329.2,
        pm_lat_mas_per_year: -164.2,
    },
    FixedStar {
        name: "Sirius",
        constellation: "Canis Major",
        longitude_j2000: 104.07,
        latitude_j2000: -39.60,
        magnitude: -1.44,
        nature: "Jupiter-Mars",
        pm_lon_mas_per_year: -546.0,
        pm_lat_mas_per_year: -1223.1,
    },
    FixedStar {
        name: "Canopus",
        constellation: "Carina",
        longitude_j2000: 105.28,
        latitude_j2000: -75.75,
        magnitude: -0.62,
        nature: "Saturn-Jupiter",
        pm_lon_mas_per_year: 19.9,
        pm_lat_mas_per_year: 23.2,
    },
    FixedStar {
        name: "Arcturus",
        constellation: "Bootes",
        longitude_j2000: 204.07,
        latitude_j2000: 30.77,
        magnitude: -0.05,
        nature: "Mars-Jupiter",
        pm_lon_mas_per_year: -1093.4,
        pm_lat_mas_per_year: -1999.4,
    },
    FixedStar {
        name: "Vega",
        constellation: "Lyra",
        longitude_j2000: 285.45,
        latitude_j2000: 61.73,
        magnitude: 0.03,
        nature: "Venus-Mercury",
        pm_lon_mas_per_year: 201.0,
        pm_lat_mas_per_year: 287.5,
    },
    FixedStar {
        name: "Capella",
        constellation: "Auriga",
        longitude_j2000: 81.73,
        latitude_j2000: 22.87,
        magnitude: 0.08,
        nature: "Mars-Mercury",
        pm_lon_mas_per_year: 75.5,
        pm_lat_mas_per_year: -427.1,
    },
    FixedStar {
        name: "Rigel",
        constellation: "Orion",
        longitude_j2000: 78.63,
        latitude_j2000: -31.10,
        magnitude: 0.18,
        nature: "Jupiter-Mars",
        pm_lon_mas_per_year: 1.3,
        pm_lat_mas_per_year: -0.6,
    },
    FixedStar {
        name: "Procyon",
        constellation: "Canis Minor",
        longitude_j2000: 115.62,
        latitude_j2000: -16.03,
        magnitude: 0.40,
        nature: "Mercury-Mars",
        pm_lon_mas_per_year: -714.6,
        pm_lat_mas_per_year: -1036.8,
    },
    FixedStar {
        name: "Betelgeuse",
        constellation: "Orion",
        longitude_j2000: 88.79,
        latitude_j2000: -16.03,
        magnitude: 0.45,
        nature: "Mars-Mercury",
        pm_lon_mas_per_year: 27.3,
        pm_lat_mas_per_year: 10.9,
    },
    FixedStar {
        name: "Altair",
        constellation: "Aquila",
        longitude_j2000: 301.82,
        latitude_j2000: 29.31,
        magnitude: 0.76,
        nature: "Mars-Jupiter",
        pm_lon_mas_per_year: 536.2,
        pm_lat_mas_per_year: 385.3,
    },
    FixedStar {
        name: "Polaris",
        constellation: "Ursa Minor",
        longitude_j2000: 28.60,
        latitude_j2000: 66.10,
        magnitude: 1.97,
        nature: "Saturn-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Algol",
        constellation: "Perseus",
        longitude_j2000: 56.17,
        latitude_j2000: 22.42,
        magnitude: 2.09,
        nature: "Saturn-Jupiter",
        pm_lon_mas_per_year: 2.4,
        pm_lat_mas_per_year: -1.5,
    },
    FixedStar {
        name: "Deneb",
        constellation: "Cygnus",
        longitude_j2000: 305.17,
        latitude_j2000: 59.88,
        magnitude: 1.25,
        nature: "Venus-Mercury",
        pm_lon_mas_per_year: 1.6,
        pm_lat_mas_per_year: 1.5,
    },
    FixedStar {
        name: "Castor",
        constellation: "Gemini",
        longitude_j2000: 110.15,
        latitude_j2000: 10.09,
        magnitude: 1.58,
        nature: "Mercury",
        pm_lon_mas_per_year: -191.5,
        pm_lat_mas_per_year: -145.2,
    },
    FixedStar {
        name: "Pollux",
        constellation: "Gemini",
        longitude_j2000: 113.22,
        latitude_j2000: 6.68,
        magnitude: 1.16,
        nature: "Mars",
        pm_lon_mas_per_year: -625.7,
        pm_lat_mas_per_year: -45.8,
    },
    FixedStar {
        name: "Pleiades",
        constellation: "Taurus",
        longitude_j2000: 60.00,
        latitude_j2000: 4.07,
        magnitude: 1.60,
        nature: "Moon-Mars",
        pm_lon_mas_per_year: 19.3,
        pm_lat_mas_per_year: -43.7,
    },
    FixedStar {
        name: "Vindemiatrix",
        constellation: "Virgo",
        longitude_j2000: 189.93,
        latitude_j2000: 16.20,
        magnitude: 2.85,
        nature: "Saturn-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Zuben Elgenubi",
        constellation: "Libra",
        longitude_j2000: 225.10,
        latitude_j2000: 0.33,
        magnitude: 2.75,
        nature: "Saturn-Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Zuben Eschamali",
        constellation: "Libra",
        longitude_j2000: 229.32,
        latitude_j2000: 8.87,
        magnitude: 2.61,
        nature: "Jupiter-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Achernar",
        constellation: "Eridanus",
        longitude_j2000: 345.33,
        latitude_j2000: -59.38,
        magnitude: 0.45,
        nature: "Jupiter",
        pm_lon_mas_per_year: 88.0,
        pm_lat_mas_per_year: -40.1,
    },
    FixedStar {
        name: "Hamal",
        constellation: "Aries",
        longitude_j2000: 37.73,
        latitude_j2000: 9.93,
        magnitude: 2.01,
        nature: "Mars-Saturn",
        pm_lon_mas_per_year: 190.7,
        pm_lat_mas_per_year: -148.1,
    },
    // ===== ORIGINAL EXPANSION (25 stars, formerly bringing total to 55) ===
    FixedStar {
        name: "Alpheratz",
        constellation: "Andromeda",
        longitude_j2000: 14.18,
        latitude_j2000: 25.68,
        magnitude: 2.07,
        nature: "Venus-Jupiter",
        pm_lon_mas_per_year: 135.7,
        pm_lat_mas_per_year: -162.9,
    },
    FixedStar {
        name: "Mirach",
        constellation: "Andromeda",
        longitude_j2000: 30.42,
        latitude_j2000: 25.55,
        magnitude: 2.07,
        nature: "Venus",
        pm_lon_mas_per_year: 175.6,
        pm_lat_mas_per_year: -112.2,
    },
    FixedStar {
        name: "Almach",
        constellation: "Andromeda",
        longitude_j2000: 44.17,
        latitude_j2000: 21.50,
        magnitude: 2.10,
        nature: "Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Sheratan",
        constellation: "Aries",
        longitude_j2000: 33.93,
        latitude_j2000: 8.49,
        magnitude: 2.64,
        nature: "Mars-Saturn",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Menkar",
        constellation: "Cetus",
        longitude_j2000: 44.33,
        latitude_j2000: -12.59,
        magnitude: 2.54,
        nature: "Saturn",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Mirfak",
        constellation: "Perseus",
        longitude_j2000: 62.33,
        latitude_j2000: 30.21,
        magnitude: 1.79,
        nature: "Jupiter-Saturn",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Bellatrix",
        constellation: "Orion",
        longitude_j2000: 80.95,
        latitude_j2000: -16.77,
        magnitude: 1.64,
        nature: "Mars-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Alnilam",
        constellation: "Orion",
        longitude_j2000: 83.42,
        latitude_j2000: -24.22,
        magnitude: 1.69,
        nature: "Jupiter-Saturn",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Elnath",
        constellation: "Taurus",
        longitude_j2000: 82.35,
        latitude_j2000: 5.23,
        magnitude: 1.65,
        nature: "Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Menkalinan",
        constellation: "Auriga",
        longitude_j2000: 89.90,
        latitude_j2000: 17.00,
        magnitude: 1.90,
        nature: "Mars-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Alhena",
        constellation: "Gemini",
        longitude_j2000: 99.07,
        latitude_j2000: -6.38,
        magnitude: 1.93,
        nature: "Mercury-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Wezen",
        constellation: "Canis Major",
        longitude_j2000: 113.42,
        latitude_j2000: -28.95,
        magnitude: 1.84,
        nature: "Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Adhara",
        constellation: "Canis Major",
        longitude_j2000: 110.72,
        latitude_j2000: -42.47,
        magnitude: 1.50,
        nature: "Venus-Jupiter",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Praesepe",
        constellation: "Cancer",
        longitude_j2000: 127.17,
        latitude_j2000: 1.50,
        magnitude: 3.70,
        nature: "Mars-Moon",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Alphard",
        constellation: "Hydra",
        longitude_j2000: 147.28,
        latitude_j2000: -22.38,
        magnitude: 1.99,
        nature: "Saturn-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Zosma",
        constellation: "Leo",
        longitude_j2000: 161.33,
        latitude_j2000: 14.43,
        magnitude: 2.56,
        nature: "Saturn-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Denebola",
        constellation: "Leo",
        longitude_j2000: 171.55,
        latitude_j2000: 12.17,
        magnitude: 2.14,
        nature: "Saturn-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Algorab",
        constellation: "Corvus",
        longitude_j2000: 193.60,
        latitude_j2000: -12.62,
        magnitude: 2.94,
        nature: "Mars-Saturn",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Cor Caroli",
        constellation: "Canes Venatici",
        longitude_j2000: 174.73,
        latitude_j2000: 39.53,
        magnitude: 2.89,
        nature: "Sun-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Acrux",
        constellation: "Crux",
        longitude_j2000: 222.00,
        latitude_j2000: -52.68,
        magnitude: 0.77,
        nature: "Jupiter",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Alphecca",
        constellation: "Corona Borealis",
        longitude_j2000: 222.10,
        latitude_j2000: 44.32,
        magnitude: 2.22,
        nature: "Venus-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Unukalhai",
        constellation: "Serpens",
        longitude_j2000: 232.05,
        latitude_j2000: 25.62,
        magnitude: 2.63,
        nature: "Saturn-Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Dschubba",
        constellation: "Scorpius",
        longitude_j2000: 242.58,
        latitude_j2000: -1.98,
        magnitude: 2.29,
        nature: "Mars-Saturn",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Rasalhague",
        constellation: "Ophiuchus",
        longitude_j2000: 262.40,
        latitude_j2000: 35.83,
        magnitude: 2.08,
        nature: "Saturn-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Shaula",
        constellation: "Scorpius",
        longitude_j2000: 264.58,
        latitude_j2000: -13.85,
        magnitude: 1.62,
        nature: "Mercury-Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Kaus Australis",
        constellation: "Sagittarius",
        longitude_j2000: 275.00,
        latitude_j2000: -11.07,
        magnitude: 1.79,
        nature: "Jupiter-Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Nunki",
        constellation: "Sagittarius",
        longitude_j2000: 282.30,
        latitude_j2000: -3.45,
        magnitude: 2.05,
        nature: "Jupiter-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Deneb Algedi",
        constellation: "Capricornus",
        longitude_j2000: 323.45,
        latitude_j2000: 2.58,
        magnitude: 2.85,
        nature: "Saturn-Jupiter",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Markab",
        constellation: "Pegasus",
        longitude_j2000: 353.47,
        latitude_j2000: 19.40,
        magnitude: 2.49,
        nature: "Mars-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Scheat",
        constellation: "Pegasus",
        longitude_j2000: 359.32,
        latitude_j2000: 31.08,
        magnitude: 2.44,
        nature: "Mars-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // ===== NEW — NAKSHATRA YOGATARAS & VEDIC/JYOTISH STARS ===============
    // Stars needed as yogataras for the 27 nakshatras that were missing above.

    // Bharani — 41 Arietis
    FixedStar {
        name: "Bharani 41",
        constellation: "Aries",
        longitude_j2000: 41.08,
        latitude_j2000: -10.32,
        magnitude: 3.63,
        nature: "Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Krittika — Alcyone (eta Tauri, brightest Pleiad)
    FixedStar {
        name: "Alcyone",
        constellation: "Taurus",
        longitude_j2000: 60.06,
        latitude_j2000: 4.07,
        magnitude: 2.85,
        nature: "Moon-Mars",
        pm_lon_mas_per_year: 19.3,
        pm_lat_mas_per_year: -43.7,
    },
    // Mrigashira — Lambda Orionis (Meissa)
    FixedStar {
        name: "Lambda Orionis",
        constellation: "Orion",
        longitude_j2000: 83.78,
        latitude_j2000: -13.38,
        magnitude: 3.39,
        nature: "Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Pushya — Asellus Australis (delta Cancri)
    FixedStar {
        name: "Asellus Australis",
        constellation: "Cancer",
        longitude_j2000: 128.70,
        latitude_j2000: -0.07,
        magnitude: 3.94,
        nature: "Saturn-Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Pushya area — Asellus Borealis (gamma Cancri)
    FixedStar {
        name: "Asellus Borealis",
        constellation: "Cancer",
        longitude_j2000: 127.52,
        latitude_j2000: 3.12,
        magnitude: 4.66,
        nature: "Saturn-Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Hasta — Porrima (gamma Virginis)
    FixedStar {
        name: "Porrima",
        constellation: "Virgo",
        longitude_j2000: 190.23,
        latitude_j2000: 2.78,
        magnitude: 2.74,
        nature: "Mercury-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Shatabhisha — Sadalmelik (alpha Aquarii, proxy)
    FixedStar {
        name: "Sadalmelik",
        constellation: "Aquarius",
        longitude_j2000: 333.38,
        latitude_j2000: 8.80,
        magnitude: 2.95,
        nature: "Saturn-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Dhanishta — Sadalsuud (beta Aquarii, proxy for Sravishtha)
    FixedStar {
        name: "Sadalsuud",
        constellation: "Aquarius",
        longitude_j2000: 323.40,
        latitude_j2000: 8.52,
        magnitude: 2.90,
        nature: "Saturn-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Revati — zeta Piscium
    FixedStar {
        name: "Revati",
        constellation: "Pisces",
        longitude_j2000: 19.87,
        latitude_j2000: -0.22,
        magnitude: 5.21,
        nature: "Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // ===== NEW — HYADES CLUSTER & TAURUS STARS ===========================
    // Ain (epsilon Tauri) — Hyades member, V-shape eye
    FixedStar {
        name: "Ain",
        constellation: "Taurus",
        longitude_j2000: 68.09,
        latitude_j2000: -3.02,
        magnitude: 3.54,
        nature: "Mars",
        pm_lon_mas_per_year: 107.0,
        pm_lat_mas_per_year: -37.8,
    },
    // Prima Hyadum (gamma Tauri) — tip of the V
    FixedStar {
        name: "Prima Hyadum",
        constellation: "Taurus",
        longitude_j2000: 69.00,
        latitude_j2000: -5.66,
        magnitude: 3.65,
        nature: "Saturn-Mercury",
        pm_lon_mas_per_year: 115.3,
        pm_lat_mas_per_year: -23.9,
    },
    // Hyadum II (delta Tauri)
    FixedStar {
        name: "Hyadum II",
        constellation: "Taurus",
        longitude_j2000: 68.80,
        latitude_j2000: -3.63,
        magnitude: 3.76,
        nature: "Saturn-Mercury",
        pm_lon_mas_per_year: 107.8,
        pm_lat_mas_per_year: -29.1,
    },
    // ===== NEW — SOUTHERN CROSS (CRUX) ===================================
    // Acrux already included above; here are the remaining Crux stars.
    FixedStar {
        name: "Mimosa",
        constellation: "Crux",
        longitude_j2000: 222.40,
        latitude_j2000: -47.53,
        magnitude: 1.25,
        nature: "Jupiter-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Gacrux",
        constellation: "Crux",
        longitude_j2000: 222.03,
        latitude_j2000: -45.52,
        magnitude: 1.59,
        nature: "Jupiter",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Imai",
        constellation: "Crux",
        longitude_j2000: 218.08,
        latitude_j2000: -49.22,
        magnitude: 2.79,
        nature: "Jupiter",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // ===== NEW — NAVIGATOR STARS MISSING FROM ORIGINAL CATALOG ===========
    // Peacock (alpha Pavonis)
    FixedStar {
        name: "Peacock",
        constellation: "Pavo",
        longitude_j2000: 293.35,
        latitude_j2000: -36.34,
        magnitude: 1.94,
        nature: "Mercury-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Diphda / Deneb Kaitos (beta Ceti) — important navigator star
    FixedStar {
        name: "Diphda",
        constellation: "Cetus",
        longitude_j2000: 2.33,
        latitude_j2000: -20.84,
        magnitude: 2.04,
        nature: "Saturn",
        pm_lon_mas_per_year: 232.8,
        pm_lat_mas_per_year: 32.7,
    },
    // Ankaa (alpha Phoenicis)
    FixedStar {
        name: "Ankaa",
        constellation: "Phoenix",
        longitude_j2000: 355.48,
        latitude_j2000: -41.98,
        magnitude: 2.40,
        nature: "Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Alnair (alpha Gruis)
    FixedStar {
        name: "Alnair",
        constellation: "Grus",
        longitude_j2000: 327.95,
        latitude_j2000: -35.18,
        magnitude: 1.73,
        nature: "Jupiter-Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Kochab (beta Ursae Minoris) — historical pole navigator
    FixedStar {
        name: "Kochab",
        constellation: "Ursa Minor",
        longitude_j2000: 222.87,
        latitude_j2000: 72.83,
        magnitude: 2.07,
        nature: "Saturn-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Dubhe (alpha Ursae Majoris) — pointer to Polaris
    FixedStar {
        name: "Dubhe",
        constellation: "Ursa Major",
        longitude_j2000: 166.63,
        latitude_j2000: 49.33,
        magnitude: 1.81,
        nature: "Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Alioth (epsilon Ursae Majoris) — navigator
    FixedStar {
        name: "Alioth",
        constellation: "Ursa Major",
        longitude_j2000: 177.33,
        latitude_j2000: 53.73,
        magnitude: 1.76,
        nature: "Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Alkaid / Benetnash (eta Ursae Majoris)
    FixedStar {
        name: "Alkaid",
        constellation: "Ursa Major",
        longitude_j2000: 190.68,
        latitude_j2000: 54.38,
        magnitude: 1.85,
        nature: "Mars-Moon",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Miaplacidus (beta Carinae) — southern navigator
    FixedStar {
        name: "Miaplacidus",
        constellation: "Carina",
        longitude_j2000: 131.15,
        latitude_j2000: -69.58,
        magnitude: 1.67,
        nature: "Saturn-Jupiter",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Avior (epsilon Carinae)
    FixedStar {
        name: "Avior",
        constellation: "Carina",
        longitude_j2000: 124.70,
        latitude_j2000: -59.05,
        magnitude: 1.86,
        nature: "Jupiter-Saturn",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Suhail (lambda Velorum)
    FixedStar {
        name: "Suhail",
        constellation: "Vela",
        longitude_j2000: 132.63,
        latitude_j2000: -43.18,
        magnitude: 2.23,
        nature: "Saturn-Jupiter",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Atria (alpha Trianguli Australis)
    FixedStar {
        name: "Atria",
        constellation: "Triangulum Australe",
        longitude_j2000: 252.33,
        latitude_j2000: -28.88,
        magnitude: 1.91,
        nature: "Jupiter",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // ===== NEW — ADDITIONAL ASTROLOGICALLY SIGNIFICANT STARS ==============

    // Algenib (gamma Pegasi) — Ptolemaic
    FixedStar {
        name: "Algenib",
        constellation: "Pegasus",
        longitude_j2000: 9.08,
        latitude_j2000: 12.62,
        magnitude: 2.83,
        nature: "Mars-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Baten Kaitos (zeta Ceti) — belly of the whale
    FixedStar {
        name: "Baten Kaitos",
        constellation: "Cetus",
        longitude_j2000: 21.82,
        latitude_j2000: -20.59,
        magnitude: 3.74,
        nature: "Saturn",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Kaffaljidhma (gamma Ceti)
    FixedStar {
        name: "Kaffaljidhma",
        constellation: "Cetus",
        longitude_j2000: 32.35,
        latitude_j2000: -12.47,
        magnitude: 3.47,
        nature: "Saturn",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Alcyone already above as Krittika yogatara

    // Phact (alpha Columbae)
    FixedStar {
        name: "Phact",
        constellation: "Columba",
        longitude_j2000: 91.83,
        latitude_j2000: -41.88,
        magnitude: 2.65,
        nature: "Mercury-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Tejat (mu Geminorum)
    FixedStar {
        name: "Tejat",
        constellation: "Gemini",
        longitude_j2000: 96.58,
        latitude_j2000: -0.87,
        magnitude: 2.87,
        nature: "Mercury-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Acubens (alpha Cancri)
    FixedStar {
        name: "Acubens",
        constellation: "Cancer",
        longitude_j2000: 133.38,
        latitude_j2000: -5.08,
        magnitude: 4.26,
        nature: "Saturn-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Algieba (gamma Leonis)
    FixedStar {
        name: "Algieba",
        constellation: "Leo",
        longitude_j2000: 149.47,
        latitude_j2000: 4.63,
        magnitude: 2.01,
        nature: "Saturn-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Al Jabhah (eta Leonis)
    FixedStar {
        name: "Al Jabhah",
        constellation: "Leo",
        longitude_j2000: 147.78,
        latitude_j2000: 4.07,
        magnitude: 3.52,
        nature: "Saturn-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Zavijava (beta Virginis)
    FixedStar {
        name: "Zavijava",
        constellation: "Virgo",
        longitude_j2000: 177.17,
        latitude_j2000: 0.67,
        magnitude: 3.59,
        nature: "Mercury-Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Mizar (zeta Ursae Majoris) — famous double star
    FixedStar {
        name: "Mizar",
        constellation: "Ursa Major",
        longitude_j2000: 185.58,
        latitude_j2000: 54.88,
        magnitude: 2.23,
        nature: "Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Alcor (80 Ursae Majoris) — eyesight test star, near Mizar
    FixedStar {
        name: "Alcor",
        constellation: "Ursa Major",
        longitude_j2000: 186.00,
        latitude_j2000: 55.00,
        magnitude: 3.99,
        nature: "Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Kraz (beta Corvi)
    FixedStar {
        name: "Kraz",
        constellation: "Corvus",
        longitude_j2000: 197.35,
        latitude_j2000: -14.55,
        magnitude: 2.65,
        nature: "Mars-Saturn",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Menkent (theta Centauri)
    FixedStar {
        name: "Menkent",
        constellation: "Centaurus",
        longitude_j2000: 222.87,
        latitude_j2000: -22.65,
        magnitude: 2.06,
        nature: "Jupiter-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Rigil Kentaurus (alpha Centauri A) — closest star system
    FixedStar {
        name: "Rigil Kentaurus",
        constellation: "Centaurus",
        longitude_j2000: 239.58,
        latitude_j2000: -42.58,
        magnitude: -0.01,
        nature: "Jupiter-Venus",
        pm_lon_mas_per_year: -3679.0,
        pm_lat_mas_per_year: 481.8,
    },
    // Hadar (beta Centauri)
    FixedStar {
        name: "Hadar",
        constellation: "Centaurus",
        longitude_j2000: 233.68,
        latitude_j2000: -44.32,
        magnitude: 0.61,
        nature: "Jupiter-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Yed Prior (delta Ophiuchi)
    FixedStar {
        name: "Yed Prior",
        constellation: "Ophiuchus",
        longitude_j2000: 242.35,
        latitude_j2000: 17.13,
        magnitude: 2.73,
        nature: "Saturn-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Sabik (eta Ophiuchi)
    FixedStar {
        name: "Sabik",
        constellation: "Ophiuchus",
        longitude_j2000: 257.63,
        latitude_j2000: 7.28,
        magnitude: 2.43,
        nature: "Saturn-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Kaus Borealis (lambda Sagittarii) — top of the bow
    FixedStar {
        name: "Kaus Borealis",
        constellation: "Sagittarius",
        longitude_j2000: 276.05,
        latitude_j2000: 2.10,
        magnitude: 2.82,
        nature: "Jupiter-Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Kaus Media (delta Sagittarii) — middle of the bow
    FixedStar {
        name: "Kaus Media",
        constellation: "Sagittarius",
        longitude_j2000: 274.53,
        latitude_j2000: -6.45,
        magnitude: 2.72,
        nature: "Jupiter-Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Ascella (zeta Sagittarii)
    FixedStar {
        name: "Ascella",
        constellation: "Sagittarius",
        longitude_j2000: 283.43,
        latitude_j2000: -7.20,
        magnitude: 2.60,
        nature: "Jupiter-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Dabih (beta Capricorni)
    FixedStar {
        name: "Dabih",
        constellation: "Capricornus",
        longitude_j2000: 304.07,
        latitude_j2000: 4.58,
        magnitude: 3.05,
        nature: "Saturn-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Nashira (gamma Capricorni)
    FixedStar {
        name: "Nashira",
        constellation: "Capricornus",
        longitude_j2000: 322.17,
        latitude_j2000: -2.53,
        magnitude: 3.68,
        nature: "Saturn-Jupiter",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Skat (delta Aquarii) — important Arabic-tradition star
    FixedStar {
        name: "Skat",
        constellation: "Aquarius",
        longitude_j2000: 338.95,
        latitude_j2000: -8.22,
        magnitude: 3.27,
        nature: "Saturn-Jupiter",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Alrescha (alpha Piscium) — the knot of Pisces
    FixedStar {
        name: "Alrescha",
        constellation: "Pisces",
        longitude_j2000: 29.35,
        latitude_j2000: -9.15,
        magnitude: 3.82,
        nature: "Mercury-Saturn",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Acamar (theta Eridani) — navigator star, river's end
    FixedStar {
        name: "Acamar",
        constellation: "Eridanus",
        longitude_j2000: 23.30,
        latitude_j2000: -53.40,
        magnitude: 2.88,
        nature: "Jupiter",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Wazn (beta Columbae)
    FixedStar {
        name: "Wazn",
        constellation: "Columba",
        longitude_j2000: 97.07,
        latitude_j2000: -40.58,
        magnitude: 3.12,
        nature: "Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_108_stars() {
        assert_eq!(
            CATALOG.len(),
            108,
            "Expected 108 stars in catalog, found {}",
            CATALOG.len()
        );
    }

    #[test]
    fn no_duplicate_names() {
        let mut names: Vec<&str> = CATALOG.iter().map(|s| s.name).collect();
        names.sort();
        for w in names.windows(2) {
            assert_ne!(w[0], w[1], "Duplicate star name: {}", w[0]);
        }
    }

    #[test]
    fn spica_at_j2000() {
        let spica = find_by_name("Spica").unwrap();
        assert!((spica.longitude_j2000 - 203.841355).abs() < 0.01);
    }

    #[test]
    fn precession_moves_stars() {
        let spica = find_by_name("Spica").unwrap();
        let lon_2000 = spica.longitude_at_epoch(2000.0);
        let lon_2100 = spica.longitude_at_epoch(2100.0);
        let diff = lon_2100 - lon_2000;
        // Spica has ecliptic pm_lon = -27.72 mas/yr, so over 100 yr that adds
        // -27.72 * 100 / 3_600_000 = -0.00077 deg. Combined with precession
        // of ~1.39689 deg (50.28796″/yr × 100) the total is ~1.39612 deg.
        assert!(
            (diff - 1.39612).abs() < 0.01,
            "100yr precession+pm should be ~1.39612 deg, got {diff}"
        );
    }

    #[test]
    fn precession_rate_is_iau2006_not_newcomb() {
        // Guard against a regression to Newcomb's 1900-epoch 50.2564″/yr: the
        // fixed-star catalog uses the IAU 2006 J2000 general-precession linear
        // rate (50.28796″/yr).
        let arcsec_per_year = PRECESSION_DEG_PER_YEAR * 3600.0;
        assert!(
            (arcsec_per_year - 50.28796).abs() < 1e-9,
            "fixed-star precession must be the IAU 2006 rate 50.28796″/yr, got {arcsec_per_year}"
        );
    }

    #[test]
    fn proper_motion_sirius_significant_over_1000yr() {
        let sirius = find_by_name("Sirius").unwrap();
        assert!(
            sirius.pm_lon_mas_per_year.abs() > 500.0,
            "Sirius should have large proper motion"
        );

        let lon_2000 = sirius.longitude_at_epoch(2000.0);
        let lon_3000 = sirius.longitude_at_epoch(3000.0);
        let diff = (lon_3000 - lon_2000).rem_euclid(360.0);
        // Over 1000 years: precession adds ~13.96 deg, pm_lon adds
        // -546 * 1000 / 3_600_000 = -0.1517 deg. Net ~13.81 deg.
        assert!(
            diff > 13.0 && diff < 15.0,
            "Sirius 1000yr lon shift should be ~13.8 deg, got {diff}"
        );
    }

    #[test]
    fn proper_motion_arcturus_significant_over_1000yr() {
        let arc = find_by_name("Arcturus").unwrap();
        // Arcturus's ECLIPTIC longitude proper motion is ~-281 mas/yr. (Its
        // famous ~1100 mas/yr figure is the EQUATORIAL pmRA*cos(dec); the
        // curated catalog used to mislabel that equatorial value as ecliptic.
        // The validated Hipparcos-derived ecliptic component now backs this
        // lookup — cross-checked against Swiss `fixstar2` at FLG_J2000.)
        assert!(
            arc.pm_lon_mas_per_year.abs() > 250.0,
            "Arcturus ecliptic pm_lon should be ~281 mas/yr, got {}",
            arc.pm_lon_mas_per_year
        );

        let lon_2000 = arc.longitude_at_epoch(2000.0);
        let lon_3000 = arc.longitude_at_epoch(3000.0);
        let diff = (lon_3000 - lon_2000).rem_euclid(360.0);
        // Precession +13.969, pm_lon -281.237*1000/3.6e6 = -0.078 deg. Net ~13.89
        assert!(
            diff > 13.0 && diff < 15.0,
            "Arcturus 1000yr lon shift should be ~13.89 deg, got {diff}"
        );
    }

    #[test]
    fn proper_motion_zero_stars_track_pure_rotation_precession() {
        // Praesepe (the Beehive cluster, M44) has no single Hipparcos star, so
        // it is kept as a curated-catalog fallback with pm = 0.0 — its motion
        // is therefore pure precession, no proper motion.
        //
        // The longitude advance is CLOSE to the planar scalar
        // (PRECESSION_DEG_PER_YEAR · dt) but NOT identical: the rigorous IAU
        // 2006/P03 rotation couples latitude into longitude, so over a century
        // it differs from the planar value by ~0.0005° (≈2″) for this
        // low-latitude (+1.5°) cluster. We assert the rotation result is in the
        // right neighbourhood of the leading-order rate yet provably distinct
        // from the old planar model (which is the whole point of the fix).
        let star = find_by_name("Praesepe").unwrap();
        assert_eq!(
            star.pm_lon_mas_per_year, 0.0,
            "Praesepe is a cluster fallback with zero proper motion"
        );
        assert_eq!(star.pm_lat_mas_per_year, 0.0);

        let lon_2100 = star.longitude_at_epoch(2100.0);
        let planar = (star.longitude_j2000 + PRECESSION_DEG_PER_YEAR * 100.0).rem_euclid(360.0);
        // Within ~0.01° of the leading-order linear rate …
        assert!(
            (lon_2100 - planar).abs() < 0.01,
            "rotation longitude {lon_2100} should be within ~0.01 deg of the \
             leading-order planar rate {planar}"
        );
        // … but NOT bit-identical: the latitude coupling makes them differ.
        assert!(
            (lon_2100 - planar).abs() > 1e-6,
            "rigorous rotation MUST differ from the old planar model (got \
             identical {lon_2100} vs {planar}) — the planar bug is back"
        );
        // Latitude is no longer frozen: precession changes it even with zero pm.
        let lat_2100 = star.latitude_at_epoch(2100.0);
        assert!(
            (lat_2100 - star.latitude_j2000).abs() > 1e-4,
            "precession should move ecliptic latitude (zero-pm star): \
             {lat_2100} vs J2000 {}",
            star.latitude_j2000
        );
    }

    #[test]
    fn latitude_changes_with_precession_and_proper_motion() {
        let sirius = find_by_name("Sirius").unwrap();
        let lat_2000 = sirius.latitude_at_epoch(2000.0);
        let lat_3000 = sirius.latitude_at_epoch(3000.0);
        let diff = lat_3000 - lat_2000;
        // Sirius ecliptic pm_lat = -1271.994 mas/yr → -0.35333 deg PM-only over
        // 1000 yr. But the rigorous IAU 2006/P03 rotation also precesses
        // latitude (the planar model wrongly held it fixed), and that coupling
        // partly cancels the proper motion here: the net Δlat is ~-0.2327 deg.
        // pyswisseph `swe.fixstar2` (mean ecliptic of date) gives Sirius lat
        // -39.6052 (2000 CE) → -39.8385 (3000 CE), Δ = -0.2333 deg — our
        // -0.2327 matches to <0.04′.
        assert!(
            (diff - (-0.23271)).abs() < 0.01,
            "Sirius lat shift over 1000yr should be ~-0.2327 deg (precession + \
             proper motion), got {diff}"
        );
        // The point of the fix: latitude is NOT proper-motion-only any more.
        let pm_only = sirius.pm_lat_mas_per_year * 1000.0 / 3_600_000.0; // ≈ -0.35333
        assert!(
            (diff - pm_only).abs() > 0.05,
            "latitude must now include the precession coupling, not just pm: \
             net {diff} vs pm-only {pm_only}"
        );
    }

    // -----------------------------------------------------------------------
    // Multi-epoch star precession vs pyswisseph (the regression this fix closes)
    // -----------------------------------------------------------------------
    //
    // Reference longitudes/latitudes below are LIVE `pyswisseph` 2.10.3.2
    // (`libswe` 2.10.03) `swe.fixstar2` output for the MEAN ecliptic of date,
    // flags `FLG_SWIEPH | FLG_NONUT | FLG_NOABERR | FLG_NOGDEFL`, at the
    // Gregorian-noon Julian Dates JD(1000-01-01 12:00) = 2086303.0 and
    // JD(3000-01-01 12:00) = 2816788.0. Swiss applies proper motion, so the
    // catalog's ecliptic PM components participate — exactly what our pipeline
    // also does. Regenerate with the snippet in `tests/swiss_star_oracle.rs`,
    // substituting those JDs and the date flags above.
    //
    // The OLD planar model (longitude += rate·dt, latitude frozen) misplaces
    // high-ecliptic-latitude stars because it never moves their latitude
    // (e.g. Vega, ~0.06°/1000 yr error vs Swiss). The IAU 2006/P03 rotation
    // couples longitude and latitude and matches Swiss to < 0.1′ across the
    // stars below.
    struct EpochOracle {
        name: &'static str,
        year: f64,
        swiss_lon: f64,
        swiss_lat: f64,
    }

    // Polaris, Aldebaran, Spica, Antares, Regulus, Sirius, Pollux at 1000 & 3000 CE.
    const EPOCH_ORACLE: &[EpochOracle] = &[
        EpochOracle {
            name: "Polaris",
            year: 1000.0,
            swiss_lon: 74.610263,
            swiss_lat: 65.981708,
        },
        EpochOracle {
            name: "Polaris",
            year: 3000.0,
            swiss_lon: 102.611184,
            swiss_lat: 66.218680,
        },
        EpochOracle {
            name: "Aldebaran",
            year: 1000.0,
            swiss_lon: 55.837618,
            swiss_lat: -5.538059,
        },
        EpochOracle {
            name: "Aldebaran",
            year: 3000.0,
            swiss_lon: 83.801123,
            swiss_lat: -5.395195,
        },
        EpochOracle {
            name: "Spica",
            year: 1000.0,
            swiss_lon: 189.915486,
            swiss_lat: -1.983434,
        },
        EpochOracle {
            name: "Spica",
            year: 3000.0,
            swiss_lon: 217.828814,
            swiss_lat: -2.134218,
        },
        EpochOracle {
            name: "Antares",
            year: 1000.0,
            swiss_lon: 235.829777,
            swiss_lat: -4.437693,
        },
        EpochOracle {
            name: "Antares",
            year: 3000.0,
            swiss_lon: 263.756925,
            swiss_lat: -4.703289,
        },
        EpochOracle {
            name: "Regulus",
            year: 1000.0,
            swiss_lon: 135.955486,
            swiss_lat: 0.427207,
        },
        EpochOracle {
            name: "Regulus",
            year: 3000.0,
            swiss_lon: 163.764189,
            swiss_lat: 0.492036,
        },
        EpochOracle {
            name: "Sirius",
            year: 1000.0,
            swiss_lon: 90.325449,
            swiss_lat: -39.378372,
        },
        EpochOracle {
            name: "Sirius",
            year: 3000.0,
            swiss_lon: 117.888987,
            swiss_lat: -39.838647,
        },
        EpochOracle {
            name: "Pollux",
            year: 1000.0,
            swiss_lon: 99.441148,
            swiss_lat: 6.609573,
        },
        EpochOracle {
            name: "Pollux",
            year: 3000.0,
            swiss_lon: 127.052579,
            swiss_lat: 6.752322,
        },
    ];

    /// Signed shortest longitude separation (handles the 0/360 wrap), degrees.
    fn lon_delta(a: f64, b: f64) -> f64 {
        ((a - b + 180.0).rem_euclid(360.0)) - 180.0
    }

    #[test]
    fn precession_multi_epoch_matches_pyswisseph() {
        // 1 arcmin = 1/60 deg. The task target is < 1'; we measure < 0.1'.
        const TOL_ARCMIN: f64 = 1.0;
        let mut worst = 0.0_f64;
        let mut failures: Vec<String> = Vec::new();

        for o in EPOCH_ORACLE {
            let star = find_by_name(o.name).expect("oracle star in catalog");
            let (lon, lat) = star.ecliptic_at_epoch(o.year);
            // Great-circle-ish separation: scale the longitude error by cos(lat).
            let dlon = lon_delta(lon, o.swiss_lon) * lat.to_radians().cos();
            let dlat = lat - o.swiss_lat;
            let sep_arcmin = (dlon * dlon + dlat * dlat).sqrt() * 60.0;
            worst = worst.max(sep_arcmin);
            if sep_arcmin > TOL_ARCMIN {
                failures.push(format!(
                    "{:<9} {:.0}: model {:.5}/{:.5} vs Swiss {:.5}/{:.5} = {:.3}'",
                    o.name, o.year, lon, lat, o.swiss_lon, o.swiss_lat, sep_arcmin
                ));
            }
        }

        assert!(
            failures.is_empty(),
            "{} star/epoch pairs disagree with pyswisseph by > {TOL_ARCMIN}' \
             (worst {worst:.3}'):\n  {}",
            failures.len(),
            failures.join("\n  ")
        );
    }

    #[test]
    fn rotation_beats_planar_model_for_high_latitude_vega() {
        // Documents WHY the rotation was needed and proves it is strictly
        // better than the discarded planar model. Vega sits at ecliptic
        // latitude ~+61.7°, the regime where the planar model (longitude +=
        // rate·dt, with latitude FROZEN) is worst: by holding latitude fixed it
        // misplaces the star. The dominant symptom is a latitude error — over
        // 1000 yr Vega's true ecliptic latitude moves ~0.048° by precession,
        // which the planar model entirely misses.
        //
        // LIVE pyswisseph `swe.fixstar2` (mean ecliptic of date) for Vega at
        // JD(3000-01-01 12:00) = 2816788.0: lon 299.36177, lat 61.68486.
        const SWISS_VEGA_3000_LON: f64 = 299.36177;
        const SWISS_VEGA_3000_LAT: f64 = 61.68486;

        let vega = find_by_name("Vega").expect("Vega in catalog");
        let (rot_lon, rot_lat) = vega.ecliptic_at_epoch(3000.0);

        // Planar model: longitude advanced by the linear rate, latitude frozen.
        let planar_lon =
            (vega.longitude_j2000 + PRECESSION_DEG_PER_YEAR * 1000.0).rem_euclid(360.0);
        let planar_lat = vega.latitude_j2000;

        let sep = |l1: f64, b1: f64, l2: f64, b2: f64| {
            let dlon = lon_delta(l1, l2) * ((b1 + b2) / 2.0).to_radians().cos();
            ((dlon * dlon) + (b1 - b2) * (b1 - b2)).sqrt()
        };
        let rot_err = sep(rot_lon, rot_lat, SWISS_VEGA_3000_LON, SWISS_VEGA_3000_LAT);
        let planar_err = sep(
            planar_lon,
            planar_lat,
            SWISS_VEGA_3000_LON,
            SWISS_VEGA_3000_LAT,
        );

        // The rigorous rotation matches Swiss to < 1′; the planar model is off
        // by an order of magnitude more (mostly its frozen latitude).
        assert!(
            rot_err < 1.0 / 60.0,
            "rotation should match Swiss to < 1' for Vega, got {:.4}' \
             (rot {rot_lon:.5}/{rot_lat:.5} vs Swiss {SWISS_VEGA_3000_LON}/{SWISS_VEGA_3000_LAT})",
            rot_err * 60.0,
        );
        assert!(
            planar_err > 5.0 * rot_err,
            "the planar (frozen-latitude) model must be markedly worse than the \
             rotation: planar {:.4}' vs rotation {:.4}'",
            planar_err * 60.0,
            rot_err * 60.0,
        );

        // Latitude must move with precession (the planar model never moved it).
        assert!(
            (rot_lat - vega.latitude_j2000).abs() > 1e-3,
            "Vega ecliptic latitude must move with precession: {rot_lat} vs \
             J2000 {}",
            vega.latitude_j2000
        );
    }

    #[test]
    fn find_conjunction_with_aldebaran() {
        let matches = find_conjunctions(70.0, 2.0);
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|(s, _)| s.name == "Aldebaran"));
    }

    #[test]
    fn no_conjunction_when_far() {
        let matches = find_conjunctions(180.0, 1.0);
        // Should find nothing near 180 deg with 1 deg orb
        assert!(matches.is_empty() || matches.iter().all(|(_, d)| *d <= 1.0));
    }

    #[test]
    fn find_by_name_case_insensitive() {
        assert!(find_by_name("sirius").is_some());
        assert!(find_by_name("SIRIUS").is_some());
        assert!(find_by_name("nonexistent").is_none());
    }

    #[test]
    fn royal_stars_present() {
        assert!(find_by_name("Aldebaran").is_some());
        assert!(find_by_name("Regulus").is_some());
        assert!(find_by_name("Antares").is_some());
        assert!(find_by_name("Fomalhaut").is_some());
    }

    #[test]
    fn southern_cross_complete() {
        assert!(find_by_name("Acrux").is_some());
        assert!(find_by_name("Mimosa").is_some());
        assert!(find_by_name("Gacrux").is_some());
        assert!(find_by_name("Imai").is_some());
    }

    #[test]
    fn hyades_stars_present() {
        assert!(find_by_name("Aldebaran").is_some()); // alpha Tauri
        assert!(find_by_name("Ain").is_some()); // epsilon Tauri
        assert!(find_by_name("Prima Hyadum").is_some()); // gamma Tauri
        assert!(find_by_name("Hyadum II").is_some()); // delta Tauri
    }

    #[test]
    fn navigator_stars_present() {
        for name in &[
            "Polaris",
            "Canopus",
            "Vega",
            "Capella",
            "Sirius",
            "Procyon",
            "Peacock",
            "Diphda",
            "Ankaa",
            "Alnair",
            "Kochab",
            "Dubhe",
            "Miaplacidus",
            "Avior",
            "Suhail",
            "Atria",
            "Achernar",
        ] {
            assert!(
                find_by_name(name).is_some(),
                "Navigator star {name} missing"
            );
        }
    }

    // ----- Nakshatra yogatara tests -----

    #[test]
    fn nakshatra_yogatara_all_27_mapped() {
        for i in 0..27 {
            let star = nakshatra_yogatara(i);
            assert!(
                star.is_some(),
                "Nakshatra index {i} has no yogatara mapping"
            );
        }
    }

    #[test]
    fn nakshatra_yogatara_invalid_index_returns_none() {
        assert!(nakshatra_yogatara(27).is_none());
        assert!(nakshatra_yogatara(100).is_none());
    }

    #[test]
    fn nakshatra_yogatara_spot_checks() {
        // Rohini = Aldebaran
        assert_eq!(nakshatra_yogatara(3).unwrap().name, "Aldebaran");
        // Magha = Regulus
        assert_eq!(nakshatra_yogatara(9).unwrap().name, "Regulus");
        // Chitra = Spica
        assert_eq!(nakshatra_yogatara(13).unwrap().name, "Spica");
        // Jyeshtha = Antares
        assert_eq!(nakshatra_yogatara(17).unwrap().name, "Antares");
        // Swati = Arcturus
        assert_eq!(nakshatra_yogatara(14).unwrap().name, "Arcturus");
        // Shravana = Altair
        assert_eq!(nakshatra_yogatara(21).unwrap().name, "Altair");
        // Punarvasu = Pollux
        assert_eq!(nakshatra_yogatara(6).unwrap().name, "Pollux");
        // Ardra = Betelgeuse
        assert_eq!(nakshatra_yogatara(5).unwrap().name, "Betelgeuse");
    }

    // ----- Generated (Hipparcos) catalog tests -----

    #[test]
    fn generated_catalog_count_matches_constant() {
        assert_eq!(GENERATED_CATALOG.len(), GENERATED_STAR_COUNT);
        // The real hip_main.dat (118,218 records) yields 8,870 stars at Vmag<=6.5.
        assert_eq!(
            GENERATED_CATALOG.len(),
            8870,
            "expected 8870 HIP stars at Vmag<=6.5, found {}",
            GENERATED_CATALOG.len()
        );
    }

    #[test]
    fn generated_catalog_all_within_vmag_limit() {
        for s in GENERATED_CATALOG {
            assert!(
                s.magnitude <= 6.5,
                "HIP {} magnitude {} exceeds Vmag<=6.5 filter",
                s.hip,
                s.magnitude
            );
        }
    }

    #[test]
    fn generated_catalog_sorted_by_hip_no_dupes() {
        for w in GENERATED_CATALOG.windows(2) {
            assert!(
                w[0].hip < w[1].hip,
                "GENERATED_CATALOG must be strictly sorted by HIP; {} !< {}",
                w[0].hip,
                w[1].hip
            );
        }
    }

    #[test]
    fn generated_positions_are_finite_and_in_range() {
        for s in GENERATED_CATALOG {
            assert!(
                (0.0..360.0).contains(&s.longitude_j2000),
                "HIP {} lon {} out of [0,360)",
                s.hip,
                s.longitude_j2000
            );
            assert!(
                (-90.0..=90.0).contains(&s.latitude_j2000),
                "HIP {} lat {} out of [-90,90]",
                s.hip,
                s.latitude_j2000
            );
            assert!(s.pm_lon_mas_per_year.is_finite());
            assert!(s.pm_lat_mas_per_year.is_finite());
        }
    }

    #[test]
    fn generated_named_bright_stars_present_by_hip() {
        // HIP numbers for these bright stars come from the IAU-CSN authority
        // file; positions are cross-checked against pyswisseph in the
        // tests/swiss_crossval.rs oracle test.
        let aldebaran = find_generated_by_hip(21421).expect("HIP 21421 Aldebaran");
        assert_eq!(aldebaran.name, Some("Aldebaran"));
        let sirius = find_generated_by_hip(32349).expect("HIP 32349 Sirius");
        assert_eq!(sirius.name, Some("Sirius"));
        let polaris = find_generated_by_hip(11767).expect("HIP 11767 Polaris");
        assert_eq!(polaris.name, Some("Polaris"));
        // Polaris is a high-ecliptic-latitude star; guard the name->HIP join did
        // NOT collide with "Polaris Australis" (a southern star).
        assert!(
            polaris.latitude_j2000 > 60.0,
            "Polaris ecliptic latitude should be ~+66 deg, got {}",
            polaris.latitude_j2000
        );
    }

    #[test]
    fn generated_find_by_name_case_insensitive() {
        assert!(find_generated_by_name("vega").is_some());
        assert!(find_generated_by_name("VEGA").is_some());
        assert!(find_generated_by_name("definitely-not-a-star").is_none());
    }

    #[test]
    fn formerly_fallback_names_resolve_to_generated_catalog() {
        // These six curated names use a traditional spelling that differs from
        // the IAU-WGSN name in the generated catalog (or, for Al Jabhah, have no
        // WGSN name at all). The exact-name reconciliation used to MISS them, so
        // `find_by_name` returned the coarse curated coordinates instead of the
        // validated Hipparcos row. After the alias fix every one resolves to its
        // generated row. (curated name, expected HIP).
        let cases = [
            ("Zuben Elgenubi", 72622_u32),
            ("Zuben Eschamali", 74785),
            ("Bharani 41", 13209),
            ("Lambda Orionis", 26207),
            ("Hyadum II", 20455),
            ("Al Jabhah", 49583),
        ];
        for (name, hip) in cases {
            let g = find_generated_by_hip(hip)
                .unwrap_or_else(|| panic!("HIP {hip} missing from generated catalog"));
            let star =
                find_by_name(name).unwrap_or_else(|| panic!("{name} not found via find_by_name"));
            // The public lookup must now return the validated Hipparcos position,
            // not the curated fallback. Compare against the generated HIP row.
            assert!(
                (star.longitude_j2000 - g.longitude_j2000).abs() < 1e-6,
                "{name} longitude {} should equal generated HIP {hip} longitude {}",
                star.longitude_j2000,
                g.longitude_j2000
            );
            assert!(
                (star.latitude_j2000 - g.latitude_j2000).abs() < 1e-6,
                "{name} latitude {} should equal generated HIP {hip} latitude {}",
                star.latitude_j2000,
                g.latitude_j2000
            );
            // Proper motion must be the real measured Hipparcos PM, not the
            // zeroed curated value — these stars carried pm = 0.0 when falling
            // back. (Every one of the six has a nonzero ecliptic PM.)
            assert!(
                star.pm_lon_mas_per_year.abs() + star.pm_lat_mas_per_year.abs() > 1.0,
                "{name} should carry real Hipparcos proper motion, got pm_lon={} pm_lat={}",
                star.pm_lon_mas_per_year,
                star.pm_lat_mas_per_year
            );
        }
    }

    #[test]
    fn bharani_41_resolves_to_validated_longitude_not_curated() {
        // Sharpest single proof of the alias bug: the curated "Bharani 41" entry
        // carried longitude 41.08 deg (off by ~7.1 deg / 25600 arcsec vs Swiss).
        // 41 Arietis (HIP 13209) is actually at ~48.2 deg J2000 ecliptic. After
        // the alias fix, find_by_name returns the validated ~48.2 deg.
        let bharani = find_by_name("Bharani 41").expect("Bharani 41 present");
        assert!(
            (bharani.longitude_j2000 - 48.2).abs() < 0.5,
            "Bharani 41 longitude should be ~48.2 deg (HIP 13209), got {} \
             (the old curated fallback was the wrong 41.08)",
            bharani.longitude_j2000
        );
        // And it must NOT be the stale curated 41.08.
        assert!(
            (bharani.longitude_j2000 - 41.08).abs() > 1.0,
            "Bharani 41 must no longer return the wrong curated 41.08, got {}",
            bharani.longitude_j2000
        );
    }

    #[test]
    fn only_clusters_remain_unreconciled() {
        // Every curated name must now resolve to a validated generated row,
        // EXCEPT the two open clusters (Pleiades, Praesepe) which have no single
        // Hipparcos star. This guards against a future curated name silently
        // regressing to the coarse fallback.
        let unreconciled: Vec<&str> = CATALOG
            .iter()
            .filter(|c| find_generated_for_curated(c.name).is_none())
            .map(|c| c.name)
            .collect();
        assert_eq!(
            unreconciled,
            vec!["Pleiades", "Praesepe"],
            "only the two open clusters should fall back to curated coordinates"
        );
    }

    #[test]
    fn expanded_conjunction_finds_more_than_curated() {
        // At Aldebaran's longitude with a 1-deg orb, the expanded HIP catalog
        // should find at least the curated count (it is a strict superset of
        // bright stars in that zone).
        let curated = find_conjunctions_at_epoch(69.79, 1.0, 2000.0).len();
        let expanded = find_conjunctions_expanded(69.79, 1.0, 2000.0).len();
        assert!(
            expanded >= curated && expanded >= 1,
            "expanded ({expanded}) should be >= curated ({curated}) and nonzero"
        );
    }

    #[test]
    fn expanded_star_count_is_sum() {
        assert_eq!(
            expanded_star_count(),
            CATALOG.len() + GENERATED_CATALOG.len()
        );
        assert!(expanded_star_count() > 6000, "breadth target: 6000+ stars");
    }

    #[test]
    fn nakshatra_yogatara_boundary_nakshatras() {
        // First: Ashwini = Sheratan
        assert_eq!(nakshatra_yogatara(0).unwrap().name, "Sheratan");
        // Last: Revati = Revati (zeta Piscium)
        assert_eq!(nakshatra_yogatara(26).unwrap().name, "Revati");
    }
}
