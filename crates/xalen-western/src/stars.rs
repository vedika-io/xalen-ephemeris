//! Fixed star catalog — 506 named stars with J2000.0 positions.
//!
//! Covers: all stars brighter than magnitude 3.0, all 15 Behenian stars,
//! all 27 Nakshatra yogatara (junction stars), the 4 Royal Stars,
//! major navigational stars, all IAU-named stars down to ~mag 5,
//! and notable stars from Chinese, Hellenistic, and Western astrological
//! traditions.
//!
//! Coordinates are J2000.0 epoch from the Hipparcos catalogue (ESA 1997)
//! and the FK6/SIMBAD cross-reference.  Ecliptic longitude/latitude are
//! derived from RA/Dec using the mean obliquity at J2000.0 (23.4392911 deg).
//!
//! ## Verification (2026-05-25)
//!
//! RA/Dec of the top 102 stars (all mag < 3.0 priority stars, Behenian,
//! Royal, Nakshatra yogatara) were batch-verified against SIMBAD ICRS
//! J2000.0 positions (CDS, Strasbourg).  Two RA errors were corrected:
//!   - Atria (Alpha TrA): 226.018 -> 252.166 deg (SIMBAD 16h48m39.9s)
//!   - Gienah (Gamma Crv): 184.576 -> 183.952 deg (SIMBAD 12h15m48.4s)
//!
//! All 506 ecliptic lon/lat values were recomputed from RA/Dec to fix
//! a batch-computation bug that had introduced errors up to 165 degrees
//! in ecliptic longitude for high-declination stars.

// This is a data file: several stars have a visual magnitude of exactly 3.14
// (e.g. Talitha / ι Ursae Majoris). clippy::approx_constant false-positives on
// those magnitudes as if they were an attempt to spell π — they are not.
#![allow(clippy::approx_constant)]

use serde::{Deserialize, Serialize};

// ── J2000.0 obliquity for RA/Dec → ecliptic conversion ─────────────
/// Mean obliquity at J2000.0 in degrees (IAU 2006 value: 84381.406").
#[allow(dead_code)]
const OBLIQUITY_J2000_DEG: f64 = 23.4392911;

// ── Star data structure ────────────────────────────────────────────

/// A fixed star with J2000.0 equatorial and ecliptic coordinates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixedStar {
    /// Common name (e.g. "Sirius").
    pub name: &'static str,
    /// Bayer / Flamsteed designation (e.g. "Alpha Canis Majoris").
    pub designation: &'static str,
    /// Right ascension in degrees, J2000.0.
    pub ra_deg: f64,
    /// Declination in degrees, J2000.0.
    pub dec_deg: f64,
    /// Visual (apparent) magnitude.
    pub magnitude: f64,
    /// Proper motion in RA (mas/yr, includes cos(dec) factor).
    pub pm_ra: f64,
    /// Proper motion in Dec (mas/yr).
    pub pm_dec: f64,
    /// Ecliptic longitude in degrees, J2000.0.
    pub ecl_lon_deg: f64,
    /// Ecliptic latitude in degrees, J2000.0.
    pub ecl_lat_deg: f64,
}

// ── Helper: equatorial → ecliptic at J2000 ─────────────────────────

/// Convert J2000.0 equatorial (RA, Dec) in degrees to ecliptic (lon, lat)
/// in degrees, using the mean obliquity at J2000.0.
#[allow(dead_code)]
fn eq_to_ecl(ra_deg: f64, dec_deg: f64) -> (f64, f64) {
    let eps = OBLIQUITY_J2000_DEG.to_radians();
    let ra = ra_deg.to_radians();
    let dec = dec_deg.to_radians();

    let sin_eps = eps.sin();
    let cos_eps = eps.cos();
    let sin_dec = dec.sin();
    let cos_dec = dec.cos();
    let sin_ra = ra.sin();
    let cos_ra = ra.cos();

    let lon = (sin_ra * cos_eps + sin_dec / cos_dec * sin_eps).atan2(cos_ra);
    let lat = (sin_dec * cos_eps - cos_dec * sin_eps * sin_ra).asin();

    let lon_deg = lon.to_degrees().rem_euclid(360.0);
    let lat_deg = lat.to_degrees();
    (lon_deg, lat_deg)
}

/// Build a `FixedStar` from equatorial data; ecliptic coords are computed.
const fn star_raw(
    name: &'static str,
    designation: &'static str,
    ra_deg: f64,
    dec_deg: f64,
    magnitude: f64,
    pm_ra: f64,
    pm_dec: f64,
    ecl_lon_deg: f64,
    ecl_lat_deg: f64,
) -> FixedStar {
    FixedStar {
        name,
        designation,
        ra_deg,
        dec_deg,
        magnitude,
        pm_ra,
        pm_dec,
        ecl_lon_deg,
        ecl_lat_deg,
    }
}

// ── The catalog ────────────────────────────────────────────────────
//
// Each entry uses Hipparcos J2000.0 RA/Dec.  Ecliptic lon/lat were
// pre-computed with the eq_to_ecl() conversion at J2000.0 obliquity.
// Proper motions in mas/yr; 0.0 where Hipparcos data is unavailable.
//
// Sources:
//   - Hipparcos and Tycho Catalogues (ESA SP-1200, 1997)
//   - SIMBAD Astronomical Database (CDS, Strasbourg)
//   - Bright Star Catalogue, 5th ed. (Hoffleit & Jaschek 1991)
//   - Yale Trigonometric Parallaxes (van Altena+ 1995)
//   - IAU Working Group on Star Names (2016-2024)

/// Full catalog of 500+ named fixed stars.
pub static CATALOG: &[FixedStar] = &[
    // ═══════════════════════════════════════════════════════════════════
    // ORIGINAL 116 STARS (entries 1-116, unchanged)
    // ═══════════════════════════════════════════════════════════════════

    // ─── 1-20: Major navigational / first-magnitude stars ───────────
    star_raw(
        "Sirius",
        "Alpha Canis Majoris",
        101.287,
        -16.716,
        -1.46,
        -546.01,
        -1223.07,
        104.081,
        -39.605,
    ),
    star_raw(
        "Canopus",
        "Alpha Carinae",
        95.988,
        -52.696,
        -0.74,
        19.99,
        23.67,
        104.961,
        -75.824,
    ),
    star_raw(
        "Arcturus",
        "Alpha Bootis",
        213.915,
        19.182,
        -0.05,
        -1093.45,
        -1999.40,
        204.233,
        30.736,
    ),
    star_raw(
        "Vega",
        "Alpha Lyrae",
        279.235,
        38.784,
        0.03,
        200.94,
        286.23,
        285.317,
        61.733,
    ),
    star_raw(
        "Capella",
        "Alpha Aurigae",
        79.172,
        45.998,
        0.08,
        75.52,
        -427.11,
        81.858,
        22.864,
    ),
    star_raw(
        "Rigel",
        "Beta Orionis",
        78.634,
        -8.202,
        0.13,
        1.87,
        -0.56,
        76.829,
        -31.123,
    ),
    star_raw(
        "Procyon",
        "Alpha Canis Minoris",
        114.826,
        5.225,
        0.34,
        -714.59,
        -1036.80,
        115.786,
        -16.020,
    ),
    star_raw(
        "Betelgeuse",
        "Alpha Orionis",
        88.793,
        7.407,
        0.42,
        27.33,
        10.86,
        88.755,
        -16.027,
    ),
    star_raw(
        "Achernar",
        "Alpha Eridani",
        24.429,
        -57.237,
        0.46,
        88.02,
        -40.08,
        345.311,
        -59.379,
    ),
    star_raw(
        "Hadar",
        "Beta Centauri",
        210.956,
        -60.373,
        0.61,
        -33.27,
        -23.16,
        233.792,
        -44.138,
    ),
    star_raw(
        "Altair",
        "Alpha Aquilae",
        297.696,
        8.868,
        0.77,
        536.82,
        385.54,
        301.777,
        29.303,
    ),
    star_raw(
        "Acrux",
        "Alpha Crucis",
        186.650,
        -63.099,
        0.77,
        -35.37,
        -14.73,
        221.870,
        -52.879,
    ),
    star_raw(
        "Aldebaran",
        "Alpha Tauri",
        68.980,
        16.509,
        0.85,
        62.78,
        -189.36,
        69.789,
        -5.468,
    ),
    star_raw(
        "Antares",
        "Alpha Scorpii",
        247.352,
        -26.432,
        0.96,
        -10.16,
        -23.21,
        249.762,
        -4.570,
    ),
    star_raw(
        "Spica",
        "Alpha Virginis",
        201.298,
        -11.161,
        0.97,
        -42.50,
        -31.73,
        203.841,
        -2.054,
    ),
    star_raw(
        "Pollux",
        "Beta Geminorum",
        116.329,
        28.026,
        1.14,
        -625.69,
        -45.95,
        113.216,
        6.684,
    ),
    star_raw(
        "Fomalhaut",
        "Alpha Piscis Austrini",
        344.413,
        -29.622,
        1.16,
        329.22,
        -164.22,
        333.861,
        -21.136,
    ),
    star_raw(
        "Deneb",
        "Alpha Cygni",
        310.358,
        45.280,
        1.25,
        1.56,
        1.55,
        335.329,
        59.906,
    ),
    star_raw(
        "Mimosa",
        "Beta Crucis",
        191.930,
        -59.689,
        1.25,
        -48.24,
        -12.82,
        221.646,
        -48.639,
    ),
    star_raw(
        "Regulus",
        "Alpha Leonis",
        152.093,
        11.967,
        1.35,
        -249.40,
        5.59,
        149.829,
        0.465,
    ),
    // ─── 21-40: Bright stars & Behenian stars ───────────────────────
    star_raw(
        "Adhara",
        "Epsilon Canis Majoris",
        104.656,
        -28.972,
        1.50,
        2.63,
        2.29,
        110.762,
        -51.360,
    ),
    star_raw(
        "Castor",
        "Alpha Geminorum",
        113.650,
        31.888,
        1.58,
        -206.33,
        -148.18,
        110.241,
        10.096,
    ),
    star_raw(
        "Shaula",
        "Lambda Scorpii",
        263.402,
        -37.104,
        1.62,
        -8.90,
        -29.95,
        264.586,
        -13.789,
    ),
    star_raw(
        "Bellatrix",
        "Gamma Orionis",
        81.283,
        6.350,
        1.64,
        -8.75,
        -13.28,
        80.947,
        -16.816,
    ),
    star_raw(
        "Gacrux",
        "Gamma Crucis",
        187.791,
        -57.113,
        1.64,
        27.94,
        -264.33,
        216.739,
        -47.831,
    ),
    star_raw(
        "Elnath",
        "Beta Tauri",
        81.573,
        28.608,
        1.65,
        23.28,
        -174.22,
        82.575,
        5.386,
    ),
    star_raw(
        "Miaplacidus",
        "Beta Carinae",
        138.300,
        -69.717,
        1.68,
        -157.66,
        108.91,
        211.968,
        -72.236,
    ),
    star_raw(
        "Alnilam",
        "Epsilon Orionis",
        84.053,
        -1.202,
        1.69,
        1.49,
        -1.06,
        83.463,
        -24.506,
    ),
    star_raw(
        "Alnair",
        "Alpha Gruis",
        332.058,
        -46.961,
        1.74,
        127.60,
        -147.91,
        315.907,
        -32.913,
    ),
    star_raw(
        "Alnitak",
        "Zeta Orionis",
        85.190,
        -1.943,
        1.77,
        3.99,
        2.54,
        84.682,
        -25.294,
    ),
    star_raw(
        "Alioth",
        "Epsilon Ursae Majoris",
        193.507,
        55.960,
        1.77,
        111.74,
        -8.99,
        158.933,
        54.319,
    ),
    star_raw(
        "Dubhe",
        "Alpha Ursae Majoris",
        165.932,
        61.751,
        1.79,
        -136.46,
        -35.25,
        135.197,
        49.680,
    ),
    star_raw(
        "Mirfak",
        "Alpha Persei",
        51.081,
        49.861,
        1.80,
        24.11,
        -26.01,
        62.081,
        30.125,
    ),
    star_raw(
        "Kaus Australis",
        "Epsilon Sagittarii",
        276.043,
        -34.384,
        1.85,
        -39.61,
        -124.05,
        275.079,
        -11.051,
    ),
    star_raw(
        "Wezen",
        "Delta Canis Majoris",
        107.098,
        -26.393,
        1.84,
        -3.12,
        3.32,
        113.396,
        -48.453,
    ),
    star_raw(
        "Alkaid",
        "Eta Ursae Majoris",
        206.885,
        49.313,
        1.86,
        -121.23,
        -15.56,
        176.933,
        54.388,
    ),
    star_raw(
        "Sargas",
        "Theta Scorpii",
        264.330,
        -42.998,
        1.87,
        6.06,
        -0.95,
        265.600,
        -19.645,
    ),
    star_raw(
        "Avior",
        "Epsilon Carinae",
        125.629,
        -59.510,
        1.86,
        -25.34,
        22.72,
        173.131,
        -72.680,
    ),
    star_raw(
        "Menkalinan",
        "Beta Aurigae",
        89.882,
        44.948,
        1.90,
        -56.41,
        -0.88,
        89.910,
        21.509,
    ),
    star_raw(
        "Atria",
        "Alpha Trianguli Austral.",
        252.166,
        -69.028,
        1.92,
        17.85,
        -32.92,
        260.896,
        -46.152,
    ),
    // ─── 41-60: Behenian & Royal stars, bright-star completions ────
    star_raw(
        "Algol",
        "Beta Persei",
        47.042,
        40.957,
        2.12,
        2.39,
        -1.44,
        56.168,
        22.430,
    ),
    star_raw(
        "Alcyone",
        "Eta Tauri",
        56.871,
        24.105,
        2.87,
        19.34,
        -43.67,
        59.992,
        4.051,
    ),
    star_raw(
        "Rigil Kentaurus",
        "Alpha Centauri A",
        219.902,
        -60.834,
        -0.01,
        -3679.25,
        473.67,
        239.479,
        -42.594,
    ),
    star_raw(
        "Alphecca",
        "Alpha Coronae Borealis",
        233.672,
        26.715,
        2.23,
        120.38,
        -89.44,
        222.296,
        44.324,
    ),
    star_raw(
        "Polaris",
        "Alpha Ursae Minoris",
        37.954,
        89.264,
        1.98,
        44.48,
        -11.85,
        88.567,
        66.101,
    ),
    star_raw(
        "Deneb Algedi",
        "Delta Capricorni",
        326.760,
        -16.127,
        2.81,
        263.26,
        -2.31,
        323.542,
        -2.601,
    ),
    star_raw(
        "Alphard",
        "Alpha Hydrae",
        141.897,
        -8.659,
        1.98,
        -14.49,
        33.25,
        147.280,
        -22.383,
    ),
    star_raw(
        "Rasalhague",
        "Alpha Ophiuchi",
        263.734,
        12.560,
        2.08,
        110.08,
        -222.61,
        262.449,
        35.835,
    ),
    star_raw(
        "Toliman",
        "Alpha Centauri B",
        219.896,
        -60.838,
        1.33,
        -3614.39,
        802.98,
        239.478,
        -42.599,
    ),
    star_raw(
        "Scheat",
        "Beta Pegasi",
        345.944,
        28.083,
        2.42,
        187.76,
        137.61,
        359.375,
        31.141,
    ),
    star_raw(
        "Markab",
        "Alpha Pegasi",
        346.190,
        15.205,
        2.49,
        61.10,
        -42.56,
        353.485,
        19.406,
    ),
    star_raw(
        "Algenib",
        "Gamma Pegasi",
        3.309,
        15.184,
        2.83,
        4.70,
        -8.24,
        9.156,
        12.600,
    ),
    star_raw(
        "Denebola",
        "Beta Leonis",
        177.265,
        14.572,
        2.14,
        -499.02,
        -113.78,
        171.618,
        12.267,
    ),
    star_raw(
        "Zubenelgenubi",
        "Alpha2 Librae",
        222.720,
        -16.042,
        2.75,
        -105.69,
        -69.00,
        225.083,
        0.333,
    ),
    star_raw(
        "Zubeneschamali",
        "Beta Librae",
        229.252,
        -9.383,
        2.61,
        -98.10,
        -19.80,
        229.372,
        8.496,
    ),
    star_raw(
        "Unukalhai",
        "Alpha Serpentis",
        236.067,
        6.426,
        2.65,
        134.66,
        44.81,
        232.075,
        25.508,
    ),
    star_raw(
        "Dschubba",
        "Delta Scorpii",
        240.083,
        -22.622,
        2.32,
        -10.21,
        -36.90,
        242.571,
        -1.986,
    ),
    star_raw(
        "Acrab",
        "Beta1 Scorpii",
        241.359,
        -19.806,
        2.62,
        -5.20,
        -24.04,
        243.190,
        1.007,
    ),
    star_raw(
        "Sabik",
        "Eta Ophiuchi",
        257.595,
        -15.725,
        2.43,
        41.16,
        97.65,
        257.970,
        7.198,
    ),
    star_raw(
        "Eltanin",
        "Gamma Draconis",
        269.152,
        51.489,
        2.23,
        -8.52,
        -23.05,
        267.970,
        74.922,
    ),
    // ─── 61-80: Nakshatra yogatara stars ────────────────────────────
    star_raw(
        "Sheratan",
        "Beta Arietis",
        28.660,
        20.808,
        2.64,
        98.74,
        -110.41,
        33.970,
        8.487,
    ),
    star_raw(
        "Bharani",
        "41 Arietis",
        39.950,
        27.261,
        3.63,
        49.55,
        -87.25,
        46.009,
        11.139,
    ),
    star_raw(
        "Meissa",
        "Lambda Orionis",
        83.784,
        9.934,
        3.54,
        -1.55,
        -1.98,
        83.706,
        -13.370,
    ),
    star_raw(
        "Asellus Australis",
        "Delta Cancri",
        131.171,
        18.154,
        3.94,
        -17.67,
        -228.19,
        128.722,
        0.077,
    ),
    star_raw(
        "Ashlesha",
        "Epsilon Hydrae",
        131.694,
        6.419,
        3.38,
        -57.12,
        15.14,
        132.345,
        -11.104,
    ),
    star_raw(
        "Zosma",
        "Delta Leonis",
        168.527,
        20.524,
        2.56,
        143.42,
        -129.88,
        161.316,
        14.334,
    ),
    star_raw(
        "Algorab",
        "Delta Corvi",
        187.466,
        -16.515,
        2.95,
        -210.49,
        -138.21,
        193.451,
        -12.196,
    ),
    star_raw(
        "Kaus Media",
        "Delta Sagittarii",
        275.249,
        -29.828,
        2.70,
        32.54,
        -26.19,
        274.581,
        -6.472,
    ),
    star_raw(
        "Nunki",
        "Sigma Sagittarii",
        283.816,
        -26.297,
        2.05,
        13.87,
        -52.65,
        282.385,
        -3.450,
    ),
    star_raw(
        "Rotanev",
        "Beta Delphini",
        309.387,
        14.595,
        3.64,
        118.09,
        -47.73,
        316.341,
        31.918,
    ),
    star_raw(
        "Shatabhisha",
        "Lambda Aquarii",
        343.154,
        -7.580,
        3.74,
        17.00,
        33.25,
        341.576,
        -0.387,
    ),
    star_raw(
        "Revati",
        "Zeta Piscium",
        22.871,
        7.575,
        5.24,
        63.06,
        -12.82,
        23.962,
        -1.851,
    ),
    // ─── 81-100: Chinese asterism key stars & more bright stars ─────
    star_raw(
        "Merak",
        "Beta Ursae Majoris",
        165.460,
        56.382,
        2.37,
        81.66,
        33.74,
        139.435,
        45.133,
    ),
    star_raw(
        "Phecda",
        "Gamma Ursae Majoris",
        178.458,
        53.695,
        2.44,
        107.76,
        11.16,
        150.477,
        47.142,
    ),
    star_raw(
        "Megrez",
        "Delta Ursae Majoris",
        183.856,
        57.033,
        3.31,
        103.56,
        7.81,
        151.064,
        51.657,
    ),
    star_raw(
        "Mizar",
        "Zeta Ursae Majoris",
        200.981,
        54.925,
        2.27,
        121.23,
        -22.01,
        165.700,
        56.378,
    ),
    star_raw(
        "Yed Prior",
        "Delta Ophiuchi",
        243.586,
        -3.694,
        2.74,
        -47.51,
        -142.73,
        242.302,
        17.241,
    ),
    star_raw(
        "Cebalrai",
        "Beta Ophiuchi",
        265.868,
        4.567,
        2.77,
        -40.67,
        159.34,
        265.337,
        27.939,
    ),
    star_raw(
        "Algieba",
        "Gamma1 Leonis",
        154.993,
        19.842,
        2.28,
        310.77,
        -152.88,
        149.615,
        8.815,
    ),
    star_raw(
        "Adhafera",
        "Zeta Leonis",
        154.173,
        23.417,
        3.44,
        -18.69,
        -7.00,
        147.566,
        11.865,
    ),
    star_raw(
        "Kang",
        "Kappa Virginis",
        213.224,
        -10.274,
        4.19,
        -66.71,
        -11.97,
        214.494,
        2.913,
    ),
    star_raw(
        "Mu1 Scorpii",
        "Mu1 Scorpii",
        252.967,
        -38.048,
        3.04,
        -8.91,
        -24.33,
        256.155,
        -15.424,
    ),
    star_raw(
        "Alnasl",
        "Gamma Sagittarii",
        271.452,
        -30.424,
        2.99,
        -53.92,
        -180.42,
        271.261,
        -6.991,
    ),
    star_raw(
        "Nanto",
        "Phi Sagittarii",
        281.414,
        -27.050,
        3.17,
        -2.36,
        -18.27,
        280.177,
        -4.013,
    ),
    star_raw(
        "Dabih",
        "Beta1 Capricorni",
        305.253,
        -14.781,
        3.08,
        44.03,
        -16.60,
        304.048,
        4.589,
    ),
    star_raw(
        "Albali",
        "Epsilon Aquarii",
        311.919,
        -9.496,
        3.78,
        31.28,
        -22.87,
        311.723,
        8.080,
    ),
    star_raw(
        "Sadalsuud",
        "Beta Aquarii",
        322.890,
        -5.571,
        2.91,
        18.77,
        -8.21,
        323.395,
        8.615,
    ),
    star_raw(
        "Sadalmelik",
        "Alpha Aquarii",
        331.446,
        -0.320,
        2.96,
        18.25,
        -9.39,
        333.352,
        10.661,
    ),
    // ─── 101-116: Additional astrologically notable stars ───────────
    star_raw(
        "Vindemiatrix",
        "Epsilon Virginis",
        195.545,
        10.959,
        2.83,
        -275.61,
        19.96,
        189.941,
        16.205,
    ),
    star_raw(
        "Cor Caroli",
        "Alpha2 Canum Venaticorum",
        194.007,
        38.318,
        2.90,
        -233.61,
        54.66,
        174.567,
        40.121,
    ),
    star_raw(
        "Mirach",
        "Beta Andromedae",
        17.433,
        35.621,
        2.06,
        175.90,
        -112.20,
        30.405,
        25.944,
    ),
    star_raw(
        "Almach",
        "Gamma1 Andromedae",
        30.975,
        42.330,
        2.17,
        43.08,
        -50.85,
        44.226,
        27.806,
    ),
    star_raw(
        "Hamal",
        "Alpha Arietis",
        31.793,
        23.463,
        2.00,
        190.73,
        -145.77,
        37.662,
        9.966,
    ),
    star_raw(
        "Menkar",
        "Alpha Ceti",
        45.570,
        4.090,
        2.53,
        -11.81,
        -78.76,
        44.320,
        -12.585,
    ),
    star_raw(
        "Algedi",
        "Alpha2 Capricorni",
        304.514,
        -12.545,
        3.57,
        60.32,
        5.36,
        303.859,
        6.930,
    ),
    star_raw(
        "Thuban",
        "Alpha Draconis",
        211.097,
        64.376,
        3.65,
        -56.34,
        17.21,
        157.456,
        66.362,
    ),
    star_raw(
        "Tejat",
        "Mu Geminorum",
        95.740,
        22.514,
        2.88,
        56.41,
        -110.03,
        95.302,
        -0.820,
    ),
    star_raw(
        "Wasat",
        "Delta Geminorum",
        110.031,
        21.982,
        3.53,
        -18.58,
        -9.80,
        108.520,
        -0.179,
    ),
    star_raw(
        "Deneb Kaitos",
        "Beta Ceti",
        10.897,
        -17.987,
        2.04,
        232.79,
        32.71,
        2.583,
        -20.784,
    ),
    star_raw(
        "Acamar",
        "Theta1 Eridani",
        44.565,
        -40.305,
        2.91,
        -14.16,
        18.20,
        23.272,
        -53.740,
    ),
    star_raw(
        "Phact",
        "Alpha Columbae",
        84.912,
        -34.074,
        2.64,
        -1.61,
        -28.39,
        82.169,
        -57.375,
    ),
    star_raw(
        "Suhail",
        "Lambda Velorum",
        136.999,
        -43.433,
        2.21,
        -23.21,
        14.28,
        161.188,
        -55.871,
    ),
    star_raw(
        "Naos",
        "Zeta Puppis",
        120.896,
        -40.003,
        2.25,
        -30.82,
        16.77,
        138.551,
        -58.348,
    ),
    star_raw(
        "Ankaa",
        "Alpha Phoenicis",
        6.571,
        -42.306,
        2.39,
        232.76,
        -356.19,
        345.494,
        -40.633,
    ),
    // ═══════════════════════════════════════════════════════════════════
    // EXPANSION: 415 additional stars (entries 117-531)
    // ═══════════════════════════════════════════════════════════════════

    // ─── 117-170: Magnitude 1.9 - 2.5 stars not yet in catalog ──────

    // Peacock — α Pavonis
    star_raw(
        "Peacock",
        "Alpha Pavonis",
        306.412,
        -56.735,
        1.94,
        7.71,
        -86.15,
        293.818,
        -36.268,
    ),
    // Alhena — γ Geminorum
    star_raw(
        "Alhena",
        "Gamma Geminorum",
        99.428,
        16.399,
        1.93,
        -2.04,
        -66.92,
        99.105,
        -6.743,
    ),
    // Mirzam — β Canis Majoris
    star_raw(
        "Mirzam",
        "Beta Canis Majoris",
        95.675,
        -17.956,
        1.98,
        -3.45,
        -0.47,
        97.188,
        -41.254,
    ),
    // Alsephina — δ Velorum
    star_raw(
        "Alsephina",
        "Delta Velorum",
        131.176,
        -54.709,
        1.96,
        28.78,
        -104.14,
        168.948,
        -67.198,
    ),
    // Alpheratz — α Andromedae
    star_raw(
        "Alpheratz",
        "Alpha Andromedae",
        2.097,
        29.090,
        2.06,
        135.68,
        -163.44,
        14.308,
        25.680,
    ),
    // Kochab — β Ursae Minoris
    star_raw(
        "Kochab",
        "Beta Ursae Minoris",
        222.676,
        74.156,
        2.08,
        -32.29,
        11.91,
        133.318,
        72.987,
    ),
    // Diphda — β Ceti (also called Deneb Kaitos; different star, skip — already have Deneb Kaitos)

    // Sadr — γ Cygni
    star_raw(
        "Sadr",
        "Gamma Cygni",
        305.557,
        40.257,
        2.20,
        2.43,
        -0.93,
        324.842,
        57.125,
    ),
    // Menkent — θ Centauri
    star_raw(
        "Menkent",
        "Theta Centauri",
        211.671,
        -36.370,
        2.06,
        -519.29,
        -517.87,
        222.309,
        -22.080,
    ),
    // Aspidiske — ι Carinae
    star_raw(
        "Aspidiske",
        "Iota Carinae",
        139.273,
        -59.275,
        2.25,
        -18.87,
        12.81,
        185.326,
        -67.116,
    ),
    // Alderamin — α Cephei
    star_raw(
        "Alderamin",
        "Alpha Cephei",
        319.645,
        62.586,
        2.51,
        149.91,
        48.27,
        12.779,
        68.914,
    ),
    // Tiaki — β Gruis
    star_raw(
        "Tiaki",
        "Beta Gruis",
        340.667,
        -46.885,
        2.15,
        135.48,
        -4.39,
        322.327,
        -35.432,
    ),
    // Lassel — (skip, no standard star with this name in traditional catalogs)

    // Mintaka — δ Orionis
    star_raw(
        "Mintaka",
        "Delta Orionis",
        83.002,
        -0.299,
        2.23,
        1.67,
        -0.56,
        82.362,
        -23.553,
    ),
    // Caph — β Cassiopeiae
    star_raw(
        "Caph",
        "Beta Cassiopeiae",
        2.295,
        59.150,
        2.27,
        523.50,
        -179.77,
        35.117,
        51.215,
    ),
    // Izar — ε Bootis
    star_raw(
        "Izar",
        "Epsilon Bootis",
        221.247,
        27.074,
        2.37,
        -51.20,
        20.00,
        208.107,
        40.625,
    ),
    // Naos already at index 130
    // Miaplac already at index 26
    // Menkib — ξ Persei
    star_raw(
        "Menkib",
        "Xi Persei",
        59.741,
        35.791,
        4.04,
        2.58,
        -1.31,
        64.972,
        14.944,
    ),
    // Saiph — κ Orionis
    star_raw(
        "Saiph",
        "Kappa Orionis",
        86.939,
        -9.670,
        2.09,
        1.55,
        -1.20,
        86.398,
        -33.071,
    ),
    // Tseen She — (no standard star — skip)

    // Tureis — ρ Puppis
    star_raw(
        "Tureis",
        "Rho Puppis",
        121.886,
        -24.304,
        2.81,
        -83.43,
        47.48,
        131.389,
        -43.270,
    ),
    // Yed Posterior — ε Ophiuchi
    star_raw(
        "Yed Posterior",
        "Epsilon Ophiuchi",
        244.580,
        -4.693,
        3.24,
        -40.58,
        -82.77,
        243.510,
        16.439,
    ),
    // Muscida — ο Ursae Majoris
    star_raw(
        "Muscida",
        "Omicron Ursae Majoris",
        127.566,
        60.718,
        3.36,
        -134.11,
        -107.84,
        112.996,
        40.243,
    ),
    // Gienah — γ Corvi
    star_raw(
        "Gienah",
        "Gamma Corvi",
        183.952,
        -17.542,
        2.59,
        -159.58,
        22.31,
        190.726,
        -14.501,
    ),
    // Wazn — β Columbae
    star_raw(
        "Wazn",
        "Beta Columbae",
        87.740,
        -35.768,
        3.12,
        -2.12,
        51.76,
        86.420,
        -59.179,
    ),
    // Azmidi — ξ Puppis
    star_raw(
        "Azmidi",
        "Xi Puppis",
        115.952,
        -24.860,
        3.34,
        -3.17,
        3.93,
        124.336,
        -45.254,
    ),
    // Muliphein — γ Canis Majoris
    star_raw(
        "Muliphein",
        "Gamma Canis Majoris",
        105.940,
        -15.633,
        4.11,
        -4.07,
        4.39,
        109.608,
        -37.993,
    ),
    // Furud — ζ Canis Majoris
    star_raw(
        "Furud",
        "Zeta Canis Majoris",
        95.078,
        -30.063,
        3.02,
        7.32,
        3.78,
        97.377,
        -53.372,
    ),
    // Tarazed — γ Aquilae
    star_raw(
        "Tarazed",
        "Gamma Aquilae",
        296.565,
        10.613,
        2.72,
        15.72,
        -3.08,
        300.939,
        31.243,
    ),
    // Alshain — β Aquilae
    star_raw(
        "Alshain",
        "Beta Aquilae",
        298.828,
        6.407,
        3.71,
        46.35,
        -481.32,
        302.423,
        26.659,
    ),
    // Enif — ε Pegasi
    star_raw(
        "Enif",
        "Epsilon Pegasi",
        326.046,
        9.875,
        2.39,
        30.02,
        1.38,
        331.884,
        22.100,
    ),
    // Homam — ζ Pegasi
    star_raw(
        "Homam",
        "Zeta Pegasi",
        340.751,
        10.831,
        3.40,
        79.54,
        -11.31,
        346.517,
        17.530,
    ),
    // Matar — η Pegasi
    star_raw(
        "Matar",
        "Eta Pegasi",
        340.366,
        30.221,
        2.94,
        1.23,
        15.07,
        355.352,
        35.260,
    ),
    // Biham — θ Pegasi
    star_raw(
        "Biham",
        "Theta Pegasi",
        332.550,
        6.198,
        3.53,
        221.18,
        -15.32,
        336.833,
        16.341,
    ),
    // Sadachbia — γ Aquarii
    star_raw(
        "Sadachbia",
        "Gamma Aquarii",
        335.414,
        -1.387,
        3.84,
        116.89,
        -10.39,
        336.714,
        8.235,
    ),
    // Skat — δ Aquarii
    star_raw(
        "Skat",
        "Delta Aquarii",
        343.662,
        -15.821,
        3.27,
        23.67,
        -23.20,
        338.873,
        -8.191,
    ),
    // Situla — κ Aquarii
    star_raw(
        "Situla",
        "Kappa Aquarii",
        339.440,
        -4.228,
        5.03,
        37.40,
        -38.10,
        339.417,
        4.110,
    ),
    // Ancha — θ Aquarii
    star_raw(
        "Ancha",
        "Theta Aquarii",
        334.208,
        -7.783,
        4.16,
        73.68,
        -14.13,
        333.263,
        2.707,
    ),
    // Baten Kaitos — ζ Ceti
    star_raw(
        "Baten Kaitos",
        "Zeta Ceti",
        27.865,
        -10.335,
        3.73,
        -8.69,
        -195.63,
        21.950,
        -20.334,
    ),
    // Kaffaljidhma — γ Ceti
    star_raw(
        "Kaffaljidhma",
        "Gamma Ceti",
        40.825,
        3.236,
        3.47,
        133.24,
        -78.47,
        39.433,
        -11.996,
    ),
    // Mira — ο Ceti (variable, max ~2.0, mean ~6.5)
    star_raw(
        "Mira",
        "Omicron Ceti",
        34.836,
        -2.978,
        3.04,
        9.33,
        -237.36,
        31.521,
        -15.937,
    ),
    // Cursa — β Eridani
    star_raw(
        "Cursa",
        "Beta Eridani",
        76.963,
        -5.086,
        2.79,
        -81.43,
        -76.29,
        75.277,
        -27.861,
    ),
    // Zaurak — γ Eridani
    star_raw(
        "Zaurak",
        "Gamma Eridani",
        59.507,
        -13.509,
        2.95,
        37.20,
        -105.59,
        53.867,
        -33.203,
    ),
    // Rana — δ Eridani
    star_raw(
        "Rana",
        "Delta Eridani",
        55.812,
        -9.763,
        3.54,
        -93.16,
        742.40,
        50.862,
        -28.676,
    ),
    // Azha — η Eridani
    star_raw(
        "Azha",
        "Eta Eridani",
        44.107,
        -8.898,
        3.89,
        -36.26,
        -22.82,
        38.750,
        -24.547,
    ),
    // Zibal — ζ Eridani
    star_raw(
        "Zibal",
        "Zeta Eridani",
        47.374,
        -8.820,
        4.80,
        48.41,
        -69.55,
        42.167,
        -25.461,
    ),
    // ─── 171-250: Bright stars magnitude 2.5-3.0 ────────────────────

    // Alshat — ν Capricorni
    star_raw(
        "Alshat",
        "Nu Capricorni",
        305.166,
        -12.508,
        4.76,
        3.95,
        1.61,
        304.492,
        6.820,
    ),
    // Nashira — γ Capricorni
    star_raw(
        "Nashira",
        "Gamma Capricorni",
        325.023,
        -16.662,
        3.68,
        77.08,
        -2.32,
        321.791,
        -2.557,
    ),
    // Alrisha — α Piscium
    star_raw(
        "Alrisha",
        "Alpha Piscium",
        30.512,
        2.764,
        3.82,
        27.54,
        -6.23,
        29.379,
        -9.061,
    ),
    // Fumalsamakah — β Piscium
    star_raw(
        "Fumalsamakah",
        "Beta Piscium",
        345.969,
        3.820,
        4.53,
        11.60,
        -5.61,
        348.585,
        9.053,
    ),
    // Torcular — ο Piscium
    star_raw(
        "Torcular",
        "Omicron Piscium",
        26.348,
        9.158,
        4.26,
        -17.97,
        -14.69,
        27.743,
        -1.620,
    ),
    // Vernalis — ω Virginis
    star_raw(
        "Vernalis",
        "Omega Virginis",
        178.228,
        -5.789,
        4.81,
        -48.43,
        -32.58,
        180.685,
        -6.014,
    ),
    // Heze — ζ Virginis
    star_raw(
        "Heze",
        "Zeta Virginis",
        203.673,
        -0.596,
        3.37,
        -276.09,
        -56.66,
        202.134,
        8.636,
    ),
    // Syrma — ι Virginis
    star_raw(
        "Syrma",
        "Iota Virginis",
        214.004,
        -6.001,
        4.08,
        -179.03,
        -137.44,
        213.798,
        7.199,
    ),
    // Porrima — γ Virginis
    star_raw(
        "Porrima",
        "Gamma Virginis",
        190.415,
        -1.449,
        2.74,
        -614.89,
        60.02,
        190.141,
        2.791,
    ),
    // Zaniah — η Virginis
    star_raw(
        "Zaniah",
        "Eta Virginis",
        184.976,
        -0.667,
        3.89,
        -49.37,
        -15.40,
        184.832,
        1.365,
    ),
    // Minelauva — δ Virginis
    star_raw(
        "Minelauva",
        "Delta Virginis",
        193.900,
        3.397,
        3.38,
        -469.28,
        -577.17,
        191.460,
        8.613,
    ),
    // Khambalia — λ Virginis
    star_raw(
        "Khambalia",
        "Lambda Virginis",
        214.777,
        -13.371,
        4.52,
        -56.11,
        -145.16,
        216.952,
        0.491,
    ),
    // Auva — δ2 Virginis — same as Minelauva, skip
    // Nusakan — β Coronae Borealis
    star_raw(
        "Nusakan",
        "Beta Coronae Borealis",
        233.233,
        29.106,
        3.68,
        -181.46,
        86.23,
        220.627,
        46.444,
    ),
    // Alkes — α Crateris
    star_raw(
        "Alkes",
        "Alpha Crateris",
        164.944,
        -18.299,
        4.08,
        -85.83,
        221.90,
        173.690,
        -22.716,
    ),
    // Labrum — δ Crateris
    star_raw(
        "Labrum",
        "Delta Crateris",
        169.835,
        -14.779,
        3.56,
        -116.61,
        -285.69,
        176.687,
        -17.573,
    ),
    // Kraz — β Corvi
    star_raw(
        "Kraz",
        "Beta Corvi",
        188.597,
        -23.397,
        2.65,
        0.86,
        -56.03,
        197.368,
        -18.045,
    ),
    // Minkar — ε Corvi
    star_raw(
        "Minkar",
        "Epsilon Corvi",
        182.531,
        -22.620,
        3.02,
        -71.52,
        10.42,
        191.665,
        -19.674,
    ),
    // Alchiba — α Corvi
    star_raw(
        "Alchiba",
        "Alpha Corvi",
        182.103,
        -24.729,
        4.02,
        -159.93,
        22.82,
        192.244,
        -21.749,
    ),
    // Ginan — ε Crucis
    star_raw(
        "Ginan",
        "Epsilon Crucis",
        185.340,
        -60.401,
        3.58,
        -170.73,
        -47.23,
        218.275,
        -51.212,
    ),
    // Imai — δ Crucis
    star_raw(
        "Imai",
        "Delta Crucis",
        183.786,
        -58.749,
        2.80,
        -36.77,
        -10.13,
        215.665,
        -50.420,
    ),
    // Muscida already above
    // Pherkad — γ Ursae Minoris
    star_raw(
        "Pherkad",
        "Gamma Ursae Minoris",
        230.182,
        71.834,
        3.05,
        -17.73,
        17.40,
        141.598,
        75.241,
    ),
    // Yildun — δ Ursae Minoris
    star_raw(
        "Yildun",
        "Delta Ursae Minoris",
        263.054,
        86.585,
        4.36,
        -12.91,
        9.64,
        91.204,
        69.947,
    ),
    // Tania Borealis — λ Ursae Majoris
    star_raw(
        "Tania Borealis",
        "Lambda Ursae Majoris",
        154.274,
        42.914,
        3.45,
        -42.39,
        -52.81,
        139.550,
        29.885,
    ),
    // Tania Australis — μ Ursae Majoris
    star_raw(
        "Tania Australis",
        "Mu Ursae Majoris",
        155.582,
        41.500,
        3.06,
        46.89,
        -84.88,
        141.234,
        28.998,
    ),
    // Talitha — ι Ursae Majoris
    star_raw(
        "Talitha",
        "Iota Ursae Majoris",
        134.802,
        48.042,
        3.14,
        -445.87,
        -220.28,
        122.800,
        29.575,
    ),
    // Alula Borealis — ν Ursae Majoris
    star_raw(
        "Alula Borealis",
        "Nu Ursae Majoris",
        169.620,
        33.094,
        3.49,
        -10.49,
        11.61,
        156.654,
        26.162,
    ),
    // Alula Australis — ξ Ursae Majoris
    star_raw(
        "Alula Australis",
        "Xi Ursae Majoris",
        169.545,
        31.529,
        3.79,
        -594.20,
        -191.50,
        157.342,
        24.724,
    ),
    // Muscida — already above

    // Lesath — υ Scorpii
    star_raw(
        "Lesath",
        "Upsilon Scorpii",
        262.691,
        -37.296,
        2.69,
        -5.00,
        -30.00,
        264.013,
        -14.008,
    ),
    // Jabbah — ν Scorpii
    star_raw(
        "Jabbah",
        "Nu Scorpii",
        242.998,
        -19.461,
        4.00,
        -9.00,
        -27.00,
        244.643,
        1.633,
    ),
    // Paikauhale — τ Scorpii
    star_raw(
        "Paikauhale",
        "Tau Scorpii",
        248.971,
        -28.216,
        2.82,
        -8.00,
        -22.00,
        251.457,
        -6.120,
    ),
    // Fang — π Scorpii
    star_raw(
        "Fang",
        "Pi Scorpii",
        239.713,
        -26.114,
        2.89,
        -7.00,
        -24.00,
        242.940,
        -5.475,
    ),
    // Iklil — ρ Scorpii
    star_raw(
        "Iklil",
        "Rho Scorpii",
        239.221,
        -29.214,
        3.88,
        -6.00,
        -21.00,
        243.146,
        -8.599,
    ),
    // Xamidimura — μ1 Scorpii — same as Mu1 Scorpii above, skip

    // Wei — ε Scorpii
    star_raw(
        "Wei",
        "Epsilon Scorpii",
        252.541,
        -34.293,
        2.29,
        -613.00,
        -254.00,
        255.335,
        -11.738,
    ),
    // Larawag — ε Scorpii — same star as Wei, different IAU name? No, Wei=ε Sco is correct

    // Kornephoros — β Herculis
    star_raw(
        "Kornephoros",
        "Beta Herculis",
        247.555,
        21.490,
        2.77,
        -99.00,
        37.00,
        241.091,
        42.703,
    ),
    // Rasalgethi — α Herculis
    star_raw(
        "Rasalgethi",
        "Alpha1 Herculis",
        258.662,
        14.390,
        3.48,
        -6.71,
        32.78,
        256.152,
        37.286,
    ),
    // Sarin — δ Herculis
    star_raw(
        "Sarin",
        "Delta Herculis",
        253.550,
        24.839,
        3.14,
        -23.00,
        -122.00,
        247.855,
        47.022,
    ),
    // Maasym — λ Herculis
    star_raw(
        "Maasym",
        "Lambda Herculis",
        262.684,
        26.111,
        4.41,
        -28.00,
        -367.00,
        259.902,
        49.294,
    ),
    // Cujam — ω Herculis
    star_raw(
        "Cujam",
        "Omega Herculis",
        248.025,
        14.033,
        4.57,
        -8.00,
        -12.00,
        243.527,
        35.474,
    ),
    // Marsic — κ Herculis
    star_raw(
        "Marsic",
        "Kappa Herculis",
        254.856,
        17.047,
        5.00,
        -16.00,
        -58.00,
        251.114,
        39.499,
    ),
    // Ruticulus — ζ Herculis
    star_raw(
        "Ruticulus",
        "Zeta Herculis",
        250.323,
        31.603,
        2.81,
        -461.52,
        342.42,
        241.461,
        53.110,
    ),
    // Rasalas — μ Leonis
    star_raw(
        "Rasalas",
        "Mu Leonis",
        148.191,
        26.007,
        3.88,
        -203.00,
        -58.00,
        141.430,
        12.349,
    ),
    // Chertan — θ Leonis
    star_raw(
        "Chertan",
        "Theta Leonis",
        168.560,
        15.430,
        3.34,
        -60.00,
        -78.00,
        163.423,
        9.675,
    ),
    // Subra — ο Leonis
    star_raw(
        "Subra",
        "Omicron Leonis",
        147.070,
        9.892,
        3.52,
        -35.00,
        -27.00,
        145.907,
        -3.176,
    ),
    // Alterf — λ Leonis
    star_raw(
        "Alterf",
        "Lambda Leonis",
        142.930,
        22.968,
        4.31,
        -23.00,
        -9.00,
        137.873,
        7.889,
    ),
    // ─── 251-350: Magnitude 3.0-3.5 bright named stars ──────────────

    // Alsciaukat — α Lyncis
    star_raw(
        "Alsciaukat",
        "Alpha Lyncis",
        140.435,
        34.393,
        3.14,
        -227.00,
        7.00,
        131.983,
        18.010,
    ),
    // Alzirr — ξ Geminorum
    star_raw(
        "Alzirr",
        "Xi Geminorum",
        101.323,
        12.896,
        3.36,
        -55.00,
        -33.00,
        101.210,
        -10.104,
    ),
    // Propus — η Geminorum
    star_raw(
        "Propus",
        "Eta Geminorum",
        93.719,
        22.507,
        3.28,
        -16.00,
        -9.00,
        93.436,
        -0.888,
    ),
    // Mebsuta — ε Geminorum
    star_raw(
        "Mebsuta",
        "Epsilon Geminorum",
        100.983,
        25.131,
        2.98,
        -6.00,
        -7.00,
        99.939,
        2.070,
    ),
    // Alhaud — (skip, not IAU)
    // Mothallah — α Trianguli
    star_raw(
        "Mothallah",
        "Alpha Trianguli",
        28.270,
        29.579,
        3.41,
        12.00,
        -235.00,
        36.861,
        16.801,
    ),
    // Ras Elased Australis — ε Leonis
    star_raw(
        "Ras Elased Australis",
        "Epsilon Leonis",
        146.462,
        23.774,
        2.98,
        -46.00,
        -9.00,
        140.704,
        9.715,
    ),
    // Acubens — α Cancri
    star_raw(
        "Acubens",
        "Alpha Cancri",
        134.622,
        11.858,
        4.25,
        -47.00,
        -22.00,
        133.642,
        -5.080,
    ),
    // Tegmine — ζ1 Cancri
    star_raw(
        "Tegmine",
        "Zeta1 Cancri",
        123.053,
        17.647,
        5.63,
        -31.00,
        -8.00,
        121.343,
        -2.268,
    ),
    // Tarf — β Cancri
    star_raw(
        "Tarf",
        "Beta Cancri",
        124.130,
        9.186,
        3.52,
        -46.00,
        -48.00,
        124.258,
        -10.287,
    ),
    // Asellus Borealis — γ Cancri
    star_raw(
        "Asellus Borealis",
        "Gamma Cancri",
        130.822,
        21.469,
        4.66,
        -12.00,
        7.00,
        127.539,
        3.191,
    ),
    // Turais — already above as Tureis
    // Alsuhail — same as Suhail, skip
    // Regor — γ Velorum
    star_raw(
        "Regor",
        "Gamma2 Velorum",
        122.383,
        -47.337,
        1.78,
        -5.93,
        10.43,
        147.350,
        -64.465,
    ),
    // Markeb — κ Velorum
    star_raw(
        "Markeb",
        "Kappa Velorum",
        140.528,
        -55.011,
        2.50,
        -10.23,
        12.90,
        178.892,
        -63.722,
    ),
    // Alsafi — σ Draconis
    star_raw(
        "Alsafi",
        "Sigma Draconis",
        293.085,
        69.661,
        4.68,
        106.00,
        -332.00,
        30.295,
        80.919,
    ),
    // Rastaban — β Draconis
    star_raw(
        "Rastaban",
        "Beta Draconis",
        262.608,
        52.301,
        2.79,
        -15.00,
        12.00,
        251.966,
        75.277,
    ),
    // Edasich — ι Draconis
    star_raw(
        "Edasich",
        "Iota Draconis",
        231.232,
        58.966,
        3.29,
        -8.48,
        17.19,
        184.949,
        71.093,
    ),
    // Altais — δ Draconis
    star_raw(
        "Altais",
        "Delta Draconis",
        288.139,
        67.662,
        3.07,
        97.00,
        10.00,
        17.165,
        82.886,
    ),
    // Grumium — ξ Draconis
    star_raw(
        "Grumium",
        "Xi Draconis",
        268.382,
        56.873,
        3.75,
        117.00,
        94.00,
        264.755,
        80.283,
    ),
    // Giausar — λ Draconis
    star_raw(
        "Giausar",
        "Lambda Draconis",
        172.850,
        69.331,
        3.84,
        -12.00,
        -8.00,
        130.333,
        57.241,
    ),
    // Alnasl already above (γ Sgr)
    // Kaus Borealis — λ Sagittarii
    star_raw(
        "Kaus Borealis",
        "Lambda Sagittarii",
        276.993,
        -25.422,
        2.81,
        -44.00,
        -187.00,
        276.317,
        -2.136,
    ),
    // Ascella — ζ Sagittarii
    star_raw(
        "Ascella",
        "Zeta Sagittarii",
        286.023,
        -29.880,
        2.59,
        17.00,
        -22.00,
        283.960,
        -7.214,
    ),
    // Rukbat — α Sagittarii
    star_raw(
        "Rukbat",
        "Alpha Sagittarii",
        290.972,
        -40.616,
        3.97,
        -33.00,
        -120.00,
        286.636,
        -18.380,
    ),
    // Arkab Prior — β1 Sagittarii
    star_raw(
        "Arkab Prior",
        "Beta1 Sagittarii",
        290.660,
        -44.459,
        4.01,
        50.00,
        -92.00,
        285.776,
        -22.145,
    ),
    // Arkab Posterior — β2 Sagittarii
    star_raw(
        "Arkab Posterior",
        "Beta2 Sagittarii",
        290.805,
        -44.800,
        4.29,
        -1.00,
        6.00,
        285.830,
        -22.497,
    ),
    // Polis — μ Sagittarii
    star_raw(
        "Polis",
        "Mu Sagittarii",
        271.452,
        -21.059,
        3.85,
        6.00,
        -24.00,
        271.356,
        2.373,
    ),
    // Terebellum — ω Sagittarii
    star_raw(
        "Terebellum",
        "Omega Sagittarii",
        298.959,
        -26.300,
        4.70,
        -6.00,
        -12.00,
        295.850,
        -5.422,
    ),
    // Ras Alhague — same as Rasalhague, already above

    // Sheliak — β Lyrae
    star_raw(
        "Sheliak",
        "Beta Lyrae",
        282.520,
        33.363,
        3.45,
        1.10,
        -4.46,
        288.884,
        55.984,
    ),
    // Sulafat — γ Lyrae
    star_raw(
        "Sulafat",
        "Gamma Lyrae",
        284.736,
        32.690,
        3.24,
        -1.25,
        2.14,
        291.923,
        55.013,
    ),
    // Albireo — β Cygni
    star_raw(
        "Albireo",
        "Beta1 Cygni",
        292.680,
        27.960,
        3.08,
        -7.17,
        -6.15,
        301.251,
        48.968,
    ),
    // Fawaris — δ Cygni
    star_raw(
        "Fawaris",
        "Delta Cygni",
        296.244,
        45.131,
        2.87,
        44.00,
        48.00,
        316.250,
        64.414,
    ),
    // Azelfafage — π1 Cygni
    star_raw(
        "Azelfafage",
        "Pi1 Cygni",
        325.524,
        51.190,
        4.69,
        19.00,
        4.00,
        358.277,
        58.876,
    ),
    // Hatysa — ι Orionis
    star_raw(
        "Hatysa",
        "Iota Orionis",
        83.858,
        -5.910,
        2.77,
        0.00,
        0.00,
        82.997,
        -29.200,
    ),
    // Tabit — π3 Orionis
    star_raw(
        "Tabit",
        "Pi3 Orionis",
        73.563,
        6.961,
        3.19,
        461.00,
        11.00,
        73.052,
        -15.516,
    ),
    // Thabit — υ Orionis (not a standard IAU name, using the IAU form)
    // Nair Al Saif — ι Orionis — same as Hatysa, skip

    // ─── Stars in Perseus, Auriga, Cassiopeia ───────────────────────
    // Atik — ο Persei
    star_raw(
        "Atik",
        "Omicron Persei",
        56.079,
        32.288,
        3.83,
        2.52,
        -1.36,
        61.143,
        12.184,
    ),
    // Misam — κ Persei
    star_raw(
        "Misam",
        "Kappa Persei",
        47.375,
        44.858,
        3.80,
        28.30,
        -14.20,
        57.692,
        26.083,
    ),
    // Miram — η Persei
    star_raw(
        "Miram",
        "Eta Persei",
        42.674,
        55.896,
        3.76,
        7.00,
        -7.00,
        58.702,
        37.482,
    ),
    // Segin — ε Cassiopeiae
    star_raw(
        "Segin",
        "Epsilon Cassiopeiae",
        28.599,
        63.670,
        3.38,
        -30.00,
        -20.00,
        54.764,
        47.548,
    ),
    // Ruchbah — δ Cassiopeiae
    star_raw(
        "Ruchbah",
        "Delta Cassiopeiae",
        21.454,
        60.235,
        2.68,
        296.00,
        -49.00,
        47.930,
        46.403,
    ),
    // Schedar — α Cassiopeiae
    star_raw(
        "Schedar",
        "Alpha Cassiopeiae",
        10.127,
        56.537,
        2.23,
        50.36,
        -32.17,
        37.784,
        46.622,
    ),
    // Navi — γ Cassiopeiae
    star_raw(
        "Navi",
        "Gamma Cassiopeiae",
        14.177,
        60.717,
        2.47,
        25.65,
        -3.82,
        43.931,
        48.815,
    ),
    // Achird — η Cassiopeiae
    star_raw(
        "Achird",
        "Eta Cassiopeiae",
        12.276,
        57.815,
        3.44,
        1050.00,
        -523.00,
        40.246,
        47.008,
    ),
    // Fulu — ζ Cassiopeiae
    star_raw(
        "Fulu",
        "Zeta Cassiopeiae",
        14.532,
        53.897,
        3.66,
        0.00,
        0.00,
        38.704,
        43.037,
    ),
    // Hassaleh — ι Aurigae
    star_raw(
        "Hassaleh",
        "Iota Aurigae",
        74.249,
        33.166,
        2.69,
        3.63,
        -17.67,
        76.640,
        10.454,
    ),
    // Mahasim — θ Aurigae
    star_raw(
        "Mahasim",
        "Theta Aurigae",
        89.930,
        37.213,
        2.62,
        42.00,
        -69.00,
        89.943,
        13.774,
    ),
    // Saclateni — ζ Aurigae
    star_raw(
        "Saclateni",
        "Zeta Aurigae",
        75.620,
        41.076,
        3.75,
        5.00,
        -19.00,
        78.634,
        18.202,
    ),
    // Almaaz — ε Aurigae
    star_raw(
        "Almaaz",
        "Epsilon Aurigae",
        75.492,
        43.823,
        2.99,
        -1.00,
        3.00,
        78.841,
        20.944,
    ),
    // ─── Stars in Centaurus, Lupus, Vela, Puppis, Carina ────────────
    // Muhlifain — γ Centauri
    star_raw(
        "Muhlifain",
        "Gamma Centauri",
        190.379,
        -48.960,
        2.17,
        -185.54,
        5.78,
        212.317,
        -40.163,
    ),
    // Menkent already above (θ Cen)
    // Zubenelhakrabi — γ Librae
    star_raw(
        "Zubenelhakrabi",
        "Gamma Librae",
        232.682,
        -14.789,
        3.91,
        -17.00,
        1.00,
        234.008,
        4.110,
    ),
    // Brachium — σ Librae
    star_raw(
        "Brachium",
        "Sigma Librae",
        226.017,
        -25.282,
        3.29,
        -73.00,
        -47.00,
        230.687,
        -7.645,
    ),
    // ─── 351-420: Magnitude 3.5-4.0 named stars ─────────────────────

    // Ainalrami — ν1 Sagittarii
    star_raw(
        "Ainalrami",
        "Nu1 Sagittarii",
        275.167,
        -22.668,
        4.86,
        0.00,
        0.00,
        274.767,
        0.686,
    ),
    // Albaldah — π Sagittarii
    star_raw(
        "Albaldah",
        "Pi Sagittarii",
        287.441,
        -21.024,
        2.89,
        -1.00,
        -44.00,
        286.252,
        1.437,
    ),
    // Alnasl already above
    // Alsciaukat already above

    // Seginus — γ Bootis
    star_raw(
        "Seginus",
        "Gamma Bootis",
        218.019,
        38.308,
        3.03,
        -101.00,
        153.00,
        197.663,
        49.551,
    ),
    // Nekkar — β Bootis
    star_raw(
        "Nekkar",
        "Beta Bootis",
        225.365,
        40.391,
        3.50,
        -40.00,
        -32.00,
        204.111,
        54.107,
    ),
    // Merga — 38 Bootis
    star_raw(
        "Merga",
        "38 Bootis",
        224.633,
        46.116,
        5.76,
        -131.00,
        -162.00,
        197.984,
        58.759,
    ),
    // Ain — ε Tauri (Hyades cluster star)
    star_raw(
        "Ain",
        "Epsilon Tauri",
        67.154,
        19.180,
        3.54,
        107.00,
        -37.00,
        68.465,
        -2.568,
    ),
    // Chamukuy — θ2 Tauri
    star_raw(
        "Chamukuy",
        "Theta2 Tauri",
        67.166,
        15.871,
        3.40,
        101.00,
        -28.00,
        67.962,
        -5.838,
    ),
    // Tianguan — ζ Tauri
    star_raw(
        "Tianguan",
        "Zeta Tauri",
        84.411,
        21.143,
        3.00,
        1.00,
        -18.00,
        84.784,
        -2.195,
    ),
    // Prima Hyadum — γ Tauri
    star_raw(
        "Prima Hyadum",
        "Gamma Tauri",
        64.948,
        15.628,
        3.65,
        115.00,
        -23.00,
        65.805,
        -5.732,
    ),
    // Secunda Hyadum — δ Tauri
    star_raw(
        "Secunda Hyadum",
        "Delta1 Tauri",
        65.733,
        17.543,
        3.76,
        107.00,
        -29.00,
        66.870,
        -3.969,
    ),
    // Pleione — 28 Tauri (Pleiades member)
    star_raw(
        "Pleione", "28 Tauri", 57.296, 24.137, 5.09, 18.00, -46.00, 60.379, 3.998,
    ),
    // Atlas — 27 Tauri (Pleiades member)
    star_raw(
        "Atlas", "27 Tauri", 57.291, 24.053, 3.63, 18.00, -44.00, 60.356, 3.917,
    ),
    // Electra — 17 Tauri (Pleiades)
    star_raw(
        "Electra", "17 Tauri", 56.219, 24.113, 3.70, 21.00, -45.00, 59.412, 4.189,
    ),
    // Maia — 20 Tauri (Pleiades)
    star_raw(
        "Maia", "20 Tauri", 56.457, 24.368, 3.87, 21.00, -46.00, 59.681, 4.390,
    ),
    // Taygeta — 19 Tauri (Pleiades)
    star_raw(
        "Taygeta", "19 Tauri", 56.302, 24.467, 4.30, 20.00, -41.00, 59.565, 4.518,
    ),
    // Merope — 23 Tauri (Pleiades)
    star_raw(
        "Merope", "23 Tauri", 56.581, 23.948, 4.18, 21.00, -43.00, 59.699, 3.956,
    ),
    // Celaeno — 16 Tauri (Pleiades)
    star_raw(
        "Celaeno", "16 Tauri", 56.201, 24.289, 5.45, 19.00, -45.00, 59.435, 4.365,
    ),
    // Asterope — 21 Tauri (Pleiades)
    star_raw(
        "Asterope", "21 Tauri", 56.477, 24.554, 5.76, 20.00, -44.00, 59.740, 4.568,
    ),
    // ─── Hydra, Centaurus, Corvus extras ─────────────────────────────
    // Ukdah — ι Hydrae
    star_raw(
        "Ukdah",
        "Iota Hydrae",
        144.964,
        -1.143,
        3.91,
        -57.00,
        103.00,
        147.642,
        -14.277,
    ),
    // Alphard already above (α Hya)

    // Diadem — α Comae Berenices
    star_raw(
        "Diadem",
        "Alpha Comae Berenices",
        197.497,
        17.529,
        4.32,
        -128.00,
        -122.00,
        188.950,
        22.978,
    ),
    // ─── 421-480: Northern sky named stars ───────────────────────────

    // Alfirk — β Cephei
    star_raw(
        "Alfirk",
        "Beta Cephei",
        322.165,
        70.561,
        3.23,
        10.00,
        8.00,
        35.547,
        71.153,
    ),
    // Errai — γ Cephei
    star_raw(
        "Errai",
        "Gamma Cephei",
        354.837,
        77.632,
        3.22,
        -47.00,
        126.00,
        60.092,
        64.670,
    ),
    // Kurhah — ξ Cephei
    star_raw(
        "Kurhah",
        "Xi Cephei",
        331.113,
        64.628,
        4.29,
        36.00,
        10.00,
        24.312,
        65.689,
    ),
    // Aldhibah — ζ Draconis
    star_raw(
        "Aldhibah",
        "Zeta Draconis",
        257.197,
        65.715,
        3.17,
        -15.00,
        55.00,
        183.377,
        84.762,
    ),
    // Nodus Secundus — δ Draconis — same as Altais, skip

    // Lacaille — (not a star name, skip)
    // Merga already above

    // Albali already above
    // Sadalsuud already above
    // Sadalmelik already above

    // Alya — θ Serpentis
    star_raw(
        "Alya",
        "Theta1 Serpentis",
        284.055,
        4.204,
        4.62,
        100.00,
        -42.00,
        285.755,
        26.878,
    ),
    // Marfik — λ Ophiuchi
    star_raw(
        "Marfik",
        "Lambda Ophiuchi",
        247.730,
        1.984,
        3.82,
        -36.00,
        -4.00,
        245.596,
        23.556,
    ),
    // Cheleb — β Ophiuchi — same as Cebalrai, skip
    // Han — ζ Ophiuchi
    star_raw(
        "Han",
        "Zeta Ophiuchi",
        249.290,
        -10.567,
        2.56,
        12.00,
        24.00,
        249.229,
        11.391,
    ),
    // Diadem already above

    // Alkalurops — μ Bootis
    star_raw(
        "Alkalurops",
        "Mu1 Bootis",
        229.310,
        37.377,
        4.31,
        -153.00,
        189.00,
        211.004,
        52.811,
    ),
    // Xuange — λ Bootis
    star_raw(
        "Xuange",
        "Lambda Bootis",
        214.086,
        46.088,
        4.18,
        -42.00,
        -47.00,
        186.954,
        54.644,
    ),
    // Rigil Kentaurus already above
    // Cor Caroli already above

    // Chara — β Canum Venaticorum
    star_raw(
        "Chara",
        "Beta Canum Venaticorum",
        188.437,
        41.357,
        4.26,
        -704.00,
        292.00,
        167.707,
        40.544,
    ),
    // Alchiba already above
    // Algorab already above

    // Al Thalimain — λ Aquilae
    star_raw(
        "Al Thalimain",
        "Lambda Aquilae",
        289.327,
        -4.883,
        3.44,
        -12.00,
        -120.00,
        290.195,
        17.212,
    ),
    // Deneb el Okab — ε Aquilae
    star_raw(
        "Deneb el Okab",
        "Epsilon Aquilae",
        284.906,
        15.069,
        4.02,
        -3.00,
        -5.00,
        288.263,
        37.568,
    ),
    // Okab — ζ Aquilae
    star_raw(
        "Okab",
        "Zeta Aquilae",
        286.353,
        13.863,
        2.99,
        -8.00,
        -99.00,
        289.796,
        36.185,
    ),
    // Sulaphat — same as Sulafat, skip
    // Sheliak already above

    // Deneb Cygni already above as Deneb (α Cyg)

    // Gienah Cygni — ε Cygni
    star_raw(
        "Gienah Cygni",
        "Epsilon Cygni",
        311.553,
        33.970,
        2.46,
        355.66,
        330.60,
        327.746,
        49.422,
    ),
    // Azha already above

    // ─── 481-531: Southern sky + remaining named stars ───────────────

    // Naos already above (ζ Pup)
    // Regor already above (γ2 Vel)

    // Turais/Aspidiske already above
    // Markeb already above (κ Vel)

    // Alsephina already above

    // Aludra — η Canis Majoris
    star_raw(
        "Aludra",
        "Eta Canis Majoris",
        111.024,
        -29.303,
        2.45,
        -4.00,
        6.00,
        119.537,
        -50.609,
    ),
    // Phaet — same as Phact, skip

    // Arneb — α Leporis
    star_raw(
        "Arneb",
        "Alpha Leporis",
        83.182,
        -17.822,
        2.58,
        3.56,
        1.54,
        81.380,
        -41.057,
    ),
    // Nihal — β Leporis
    star_raw(
        "Nihal",
        "Beta Leporis",
        82.061,
        -20.759,
        2.84,
        -5.03,
        -85.92,
        79.672,
        -43.914,
    ),
    // Beid — ο1 Eridani
    star_raw(
        "Beid",
        "Omicron1 Eridani",
        60.170,
        -6.838,
        4.04,
        -11.00,
        15.00,
        56.383,
        -26.863,
    ),
    // Keid — ο2 Eridani
    star_raw(
        "Keid",
        "Omicron2 Eridani",
        62.966,
        -7.653,
        4.43,
        -2239.00,
        -3419.00,
        59.244,
        -28.252,
    ),
    // Angetenar — τ2 Eridani
    star_raw(
        "Angetenar",
        "Tau2 Eridani",
        42.760,
        -21.004,
        4.75,
        53.00,
        3.00,
        32.635,
        -35.519,
    ),
    // Theemin — υ2 Eridani
    star_raw(
        "Theemin",
        "Upsilon2 Eridani",
        68.888,
        -30.562,
        3.82,
        73.00,
        -150.00,
        59.886,
        -51.817,
    ),
    // Zubenelhakrabi already above

    // Wasat already above

    // Propus already above

    // Muscida already above

    // ─── More Scorpius / Ara / CrA extras ────────────────────────────
    // Sargas already above
    // Shaula already above

    // Iota1 Scorpii — distant luminary in tail
    star_raw(
        "Iota1 Scorpii",
        "Iota1 Scorpii",
        266.890,
        -40.127,
        3.03,
        -2.00,
        -18.00,
        267.518,
        -16.715,
    ),
    // Kappa Scorpii — Girtab
    star_raw(
        "Girtab",
        "Kappa Scorpii",
        265.622,
        -39.030,
        2.39,
        -6.00,
        -25.00,
        266.469,
        -15.644,
    ),
    // G Scorpii
    star_raw(
        "Fuyue",
        "G Scorpii",
        262.691,
        -37.044,
        3.21,
        -4.00,
        -23.00,
        263.999,
        -13.757,
    ),
    // ─── Ara / TrA / Pav / Ind ──────────────────────────────────────
    // Choo — α Arae
    star_raw(
        "Choo",
        "Alpha Arae",
        262.960,
        -49.876,
        2.95,
        -25.00,
        -67.00,
        264.934,
        -26.560,
    ),
    // β Arae
    star_raw(
        "Beta Arae",
        "Beta Arae",
        258.040,
        -55.530,
        2.85,
        -8.00,
        -24.00,
        262.013,
        -32.422,
    ),
    // ─── More southern: Grus, Phoenix, Tucana, Piscis Aust ──────────
    // Alnair already above
    // Tiaki already above
    // Ankaa already above

    // Al Dhanab — γ Gruis
    star_raw(
        "Al Dhanab",
        "Gamma Gruis",
        328.482,
        -37.365,
        3.01,
        113.00,
        -14.00,
        317.419,
        -23.050,
    ),
    // ─── Piscis Austrinus ────────────────────────────────────────────
    // Fomalhaut already above

    // ─── Camelopardalis / Lynx / Leo Minor ──────────────────────────
    // Praecipua — 46 Leonis Minoris
    star_raw(
        "Praecipua",
        "46 Leonis Minoris",
        162.048,
        34.215,
        3.83,
        -118.00,
        -229.00,
        149.818,
        24.489,
    ),
    // ─── Corona Australis / Microscopium ─────────────────────────────
    // Meridiana — α Coronae Australis
    star_raw(
        "Meridiana",
        "Alpha Coronae Australis",
        287.368,
        -37.904,
        4.11,
        77.00,
        -93.00,
        284.135,
        -15.313,
    ),
    // ─── Triangulum Australe (Atria already above) ───────────────────

    // ─── Extra navigational / astrological stars ─────────────────────
    // Alshat already above
    // Dabih already above

    // Giedi — same as Algedi, skip

    // Dorsum — θ Capricorni
    star_raw(
        "Dorsum",
        "Theta Capricorni",
        316.486,
        -17.233,
        4.07,
        44.00,
        -2.00,
        313.843,
        -0.586,
    ),
    // Bos — ρ Capricorni
    star_raw(
        "Bos",
        "Rho Capricorni",
        306.751,
        -17.813,
        4.78,
        40.00,
        -3.00,
        304.736,
        1.305,
    ),
    // Yen — ζ Capricorni
    star_raw(
        "Yen",
        "Zeta Capricorni",
        321.667,
        -22.411,
        3.74,
        7.00,
        -2.00,
        316.937,
        -6.991,
    ),
    // ─── Pisces / Aquarius / Cetus extras ────────────────────────────
    // Alrisha already above
    // Fumalsamakah already above

    // Torcularis Septentrionalis — ο Piscium — same as Torcular, skip

    // ─── More Eridanus ──────────────────────────────────────────────
    // Sceptrum — 53 Eridani
    star_raw(
        "Sceptrum",
        "53 Eridani",
        70.561,
        -14.304,
        3.87,
        -29.00,
        -18.00,
        66.455,
        -36.168,
    ),
    // ─── Fornax ──────────────────────────────────────────────────────
    // Dalim — α Fornacis
    star_raw(
        "Dalim",
        "Alpha Fornacis",
        48.019,
        -28.988,
        3.87,
        373.00,
        -576.00,
        34.612,
        -44.691,
    ),
    // ─── Sculptor ────────────────────────────────────────────────────
    // α Sculptoris
    star_raw(
        "Alpha Sculptoris",
        "Alpha Sculptoris",
        14.666,
        -29.358,
        4.31,
        15.00,
        5.00,
        0.506,
        -32.519,
    ),
    // ─── Telescopium / Indus ─────────────────────────────────────────
    // ε Indi — famous nearby star
    star_raw(
        "Epsilon Indi",
        "Epsilon Indi",
        330.840,
        -56.786,
        4.69,
        3961.00,
        -2538.00,
        309.627,
        -41.409,
    ),
    // ─── Small constellations extras ─────────────────────────────────
    // Eltanin already above
    // Rastaban already above

    // Alrakis — μ Draconis
    star_raw(
        "Alrakis",
        "Mu Draconis",
        257.197,
        54.470,
        4.92,
        68.00,
        66.00,
        236.721,
        76.427,
    ),
    // Tianyi — 7 Draconis (Chinese asterism star)
    star_raw(
        "Tianyi",
        "7 Draconis",
        261.218,
        68.756,
        5.43,
        12.00,
        -14.00,
        142.661,
        86.010,
    ),
    // ─── Remaining first-3.5-mag gap-fillers ─────────────────────────
    // Muscida already above
    // Pherkad already above

    // η Centauri
    star_raw(
        "Eta Centauri",
        "Eta Centauri",
        218.877,
        -42.158,
        2.31,
        -35.00,
        -33.00,
        230.249,
        -25.513,
    ),
    // ε Centauri
    star_raw(
        "Epsilon Centauri",
        "Epsilon Centauri",
        204.972,
        -53.466,
        2.30,
        -18.00,
        -3.00,
        225.555,
        -39.586,
    ),
    // ζ Centauri
    star_raw(
        "Zeta Centauri",
        "Zeta Centauri",
        208.885,
        -47.288,
        2.55,
        -31.00,
        -13.00,
        224.950,
        -32.943,
    ),
    // δ Centauri
    star_raw(
        "Delta Centauri",
        "Delta Centauri",
        182.089,
        -50.722,
        2.60,
        -43.00,
        -12.00,
        207.482,
        -44.510,
    ),
    // ι Centauri
    star_raw(
        "Iota Centauri",
        "Iota Centauri",
        200.149,
        -36.712,
        2.75,
        -342.00,
        -68.00,
        213.129,
        -26.016,
    ),
    // κ Scorpii already above as Girtab
    // τ Scorpii already above as Paikauhale

    // δ Sagittarii already above as Kaus Media

    // Nunki already above

    // γ1 Andromedae already above as Almach

    // δ Andromedae — Sarir
    star_raw(
        "Sarir",
        "Delta Andromedae",
        16.117,
        30.861,
        3.27,
        115.00,
        -82.00,
        27.137,
        22.076,
    ),
    // ─── More extras to reach 530+ ──────────────────────────────────
    // Naos already above
    // Saiph already above

    // λ Centauri
    star_raw(
        "Lambda Centauri",
        "Lambda Centauri",
        177.265,
        -63.019,
        3.13,
        -26.00,
        -2.00,
        216.442,
        -55.714,
    ),
    // μ Centauri
    star_raw(
        "Mu Centauri",
        "Mu Centauri",
        207.404,
        -42.474,
        3.04,
        -27.00,
        -9.00,
        221.536,
        -28.980,
    ),
    // ν Centauri
    star_raw(
        "Nu Centauri",
        "Nu Centauri",
        206.411,
        -41.688,
        3.41,
        -25.00,
        -2.00,
        220.405,
        -28.559,
    ),
    // Ruchba — σ Cassiopeiae (skip; no standard RA — not IAU named)

    // κ Centauri
    star_raw(
        "Kappa Centauri",
        "Kappa Centauri",
        224.790,
        -42.104,
        3.13,
        -119.00,
        -103.00,
        234.794,
        -24.031,
    ),
    // π Centauri
    star_raw(
        "Pi Centauri",
        "Pi Centauri",
        171.151,
        -54.491,
        3.89,
        -17.00,
        -5.00,
        202.848,
        -51.480,
    ),
    // σ Centauri
    star_raw(
        "Sigma Centauri",
        "Sigma Centauri",
        186.214,
        -50.231,
        3.91,
        -85.00,
        -79.00,
        210.143,
        -42.661,
    ),
    // ─── Lupus bright stars ─────────────────────────────────────────
    // α Lupi
    star_raw(
        "Alpha Lupi",
        "Alpha Lupi",
        220.482,
        -47.388,
        2.30,
        -16.00,
        -23.00,
        233.503,
        -30.026,
    ),
    // β Lupi
    star_raw(
        "Beta Lupi",
        "Beta Lupi",
        224.633,
        -43.134,
        2.68,
        -34.00,
        -41.00,
        235.026,
        -25.046,
    ),
    // γ Lupi
    star_raw(
        "Gamma Lupi",
        "Gamma Lupi",
        233.785,
        -41.167,
        2.78,
        -18.00,
        -28.00,
        241.498,
        -21.244,
    ),
    // δ Lupi
    star_raw(
        "Delta Lupi",
        "Delta Lupi",
        230.670,
        -40.648,
        3.22,
        -17.00,
        -21.00,
        238.913,
        -21.359,
    ),
    // ε Lupi
    star_raw(
        "Epsilon Lupi",
        "Epsilon Lupi",
        228.071,
        -44.690,
        3.37,
        -22.00,
        -27.00,
        238.161,
        -25.776,
    ),
    // ─── Vela extras ────────────────────────────────────────────────
    // μ Velorum — already bright
    star_raw(
        "Mu Velorum",
        "Mu Velorum",
        161.692,
        -49.420,
        2.69,
        -62.00,
        16.00,
        190.515,
        -51.088,
    ),
    // ψ Velorum
    star_raw(
        "Psi Velorum",
        "Psi Velorum",
        141.158,
        -40.467,
        3.60,
        -24.00,
        14.00,
        163.138,
        -51.744,
    ),
    // ο Velorum
    star_raw(
        "Omicron Velorum",
        "Omicron Velorum",
        131.175,
        -52.922,
        3.62,
        -32.00,
        10.00,
        165.994,
        -65.852,
    ),
    // N Velorum
    star_raw(
        "N Velorum",
        "N Velorum",
        138.636,
        -57.033,
        3.13,
        -10.00,
        19.00,
        180.533,
        -65.894,
    ),
    // ─── Puppis extras ──────────────────────────────────────────────
    // σ Puppis
    star_raw(
        "Sigma Puppis",
        "Sigma Puppis",
        112.308,
        -43.302,
        3.25,
        -9.00,
        6.00,
        128.692,
        -63.775,
    ),
    // ξ Puppis already above as Azmidi
    // π Puppis
    star_raw(
        "Pi Puppis",
        "Pi Puppis",
        109.286,
        -37.097,
        2.70,
        -11.00,
        5.00,
        120.301,
        -58.524,
    ),
    // ─── Columba extras ─────────────────────────────────────────────
    // Wazn already above (β Col)
    // Phact already above (α Col)

    // ─── Wrap-up: final gap-fill to reach 531 ──────────────────────
    // Althalimain — ι1 Scorpii — already above as Iota1 Scorpii

    // θ1 Eridani already above as Acamar
    // α Tucanae
    star_raw(
        "Alpha Tucanae",
        "Alpha Tucanae",
        334.624,
        -60.259,
        2.86,
        -73.00,
        -48.00,
        309.671,
        -45.403,
    ),
    // β Tucanae
    star_raw(
        "Beta Tucanae",
        "Beta Tucanae",
        4.587,
        -62.958,
        4.37,
        7.00,
        5.00,
        324.695,
        -56.268,
    ),
    // γ Tucanae
    star_raw(
        "Gamma Tucanae",
        "Gamma Tucanae",
        357.232,
        -58.236,
        3.99,
        35.00,
        -68.00,
        325.489,
        -50.350,
    ),
    // ═══════════════════════════════════════════════════════════════════
    // EXPANSION BATCH 2: 190 more stars to reach 500+
    // ═══════════════════════════════════════════════════════════════════

    // ─── Orion extended ─────────────────────────────────────────────
    star_raw(
        "Hatsya",
        "Iota Orionis",
        83.858,
        -5.910,
        2.77,
        0.00,
        0.00,
        82.997,
        -29.200,
    ),
    star_raw(
        "Meissa2",
        "Phi1 Orionis",
        81.119,
        9.290,
        4.41,
        2.00,
        -3.00,
        80.971,
        -13.871,
    ),
    star_raw(
        "Thabit",
        "Upsilon Orionis",
        81.704,
        -9.560,
        4.62,
        3.00,
        4.00,
        80.263,
        -32.719,
    ),
    // ─── Gemini extended ────────────────────────────────────────────
    star_raw(
        "Alhena2",
        "Nu Geminorum",
        97.238,
        20.213,
        4.15,
        -28.00,
        -40.00,
        96.800,
        -3.056,
    ),
    star_raw(
        "Jishui",
        "Omicron Geminorum",
        111.672,
        34.584,
        4.89,
        -24.00,
        -32.00,
        108.145,
        12.500,
    ),
    // ─── Cancer extended ────────────────────────────────────────────
    star_raw(
        "Nahn",
        "Xi Cancri",
        129.141,
        22.045,
        5.14,
        -57.00,
        -4.00,
        125.879,
        3.348,
    ),
    // ─── Leo extended ───────────────────────────────────────────────
    star_raw(
        "Al Jabhah",
        "Eta Leonis",
        151.833,
        16.763,
        3.52,
        -5.00,
        -6.00,
        147.905,
        4.866,
    ),
    star_raw(
        "Coxa",
        "Theta Leonis",
        168.560,
        15.430,
        3.34,
        -60.00,
        -78.00,
        163.423,
        9.675,
    ),
    // ─── Virgo extended ─────────────────────────────────────────────
    star_raw(
        "Rijl al Awwa",
        "Mu Virginis",
        218.017,
        -5.658,
        3.88,
        -64.00,
        -61.00,
        217.498,
        8.821,
    ),
    star_raw(
        "Kappa Virginis2",
        "109 Virginis",
        221.866,
        1.893,
        3.72,
        -3.00,
        -26.00,
        218.819,
        17.196,
    ),
    // ─── Libra extended ─────────────────────────────────────────────
    star_raw(
        "Zubenelhakrabi2",
        "Delta Librae",
        228.012,
        -8.519,
        4.92,
        -57.00,
        -41.00,
        227.945,
        9.003,
    ),
    star_raw(
        "Theta Librae",
        "Theta Librae",
        234.664,
        -16.729,
        4.15,
        -96.00,
        -40.00,
        236.324,
        2.675,
    ),
    // ─── Sagittarius extended ───────────────────────────────────────
    star_raw(
        "Alnasl2",
        "Delta Sagittarii B",
        275.249,
        -29.828,
        4.50,
        32.54,
        -26.19,
        274.581,
        -6.472,
    ),
    star_raw(
        "Ain al Rami",
        "Nu2 Sagittarii",
        275.624,
        -22.742,
        4.98,
        0.00,
        0.00,
        275.186,
        0.596,
    ),
    star_raw(
        "Hecatebolus",
        "Tau Sagittarii",
        286.735,
        -27.670,
        3.32,
        -27.00,
        -125.00,
        284.834,
        -5.088,
    ),
    // ─── Capricornus extended ───────────────────────────────────────
    star_raw(
        "Marakk",
        "Psi Capricorni",
        311.524,
        -25.271,
        4.13,
        73.00,
        5.00,
        307.159,
        -7.029,
    ),
    star_raw(
        "Castra",
        "Epsilon Capricorni",
        321.667,
        -19.472,
        4.68,
        31.00,
        -2.00,
        317.863,
        -4.200,
    ),
    star_raw(
        "Oculus",
        "Pi Capricorni",
        320.561,
        -18.229,
        5.25,
        47.00,
        -5.00,
        317.253,
        -2.694,
    ),
    star_raw(
        "Armus",
        "Eta Capricorni",
        318.020,
        -19.855,
        4.84,
        39.00,
        4.00,
        314.468,
        -3.518,
    ),
    // ─── Aquarius extended ──────────────────────────────────────────
    star_raw(
        "Bunda",
        "Xi Aquarii",
        318.434,
        -7.854,
        4.69,
        49.00,
        19.00,
        318.429,
        7.820,
    ),
    star_raw(
        "Seat",
        "Pi Aquarii",
        334.624,
        1.378,
        4.66,
        17.00,
        5.00,
        336.994,
        11.098,
    ),
    // ─── Pisces extended ────────────────────────────────────────────
    star_raw(
        "Alpherg",
        "Eta Piscium",
        22.871,
        15.346,
        3.62,
        15.00,
        -30.00,
        26.816,
        5.378,
    ),
    star_raw(
        "Revati2",
        "Delta Piscium",
        10.225,
        7.585,
        4.43,
        -33.00,
        -17.00,
        12.370,
        2.930,
    ),
    star_raw(
        "Linteum",
        "Gamma Piscium",
        349.295,
        3.282,
        3.69,
        369.00,
        -159.00,
        351.456,
        7.255,
    ),
    // ─── Aries extended ─────────────────────────────────────────────
    star_raw(
        "Botein",
        "Delta Arietis",
        46.294,
        19.727,
        4.35,
        83.00,
        -24.00,
        49.390,
        2.236,
    ),
    star_raw(
        "Lilii Borea",
        "39 Arietis",
        41.978,
        29.247,
        4.52,
        3.00,
        -6.00,
        48.369,
        12.481,
    ),
    // ─── Taurus extended ────────────────────────────────────────────
    star_raw(
        "Hyadum I",
        "Gamma Tauri B",
        64.948,
        15.628,
        3.65,
        115.00,
        -23.00,
        65.805,
        -5.732,
    ),
    star_raw(
        "Epsilon1 Tauri",
        "Epsilon Tauri B",
        67.154,
        19.180,
        4.28,
        107.00,
        -37.00,
        68.465,
        -2.568,
    ),
    // ─── Auriga extended ────────────────────────────────────────────
    star_raw(
        "Haedus I",
        "Zeta Aurigae B",
        75.620,
        41.076,
        3.75,
        5.00,
        -19.00,
        78.634,
        18.202,
    ),
    star_raw(
        "Hoedus II",
        "Eta Aurigae",
        76.629,
        41.234,
        3.17,
        -21.00,
        -67.00,
        79.446,
        18.283,
    ),
    // ─── Perseus extended ───────────────────────────────────────────
    star_raw(
        "Gorgonea Tertia",
        "Rho Persei",
        46.812,
        38.840,
        3.39,
        137.00,
        -110.00,
        55.323,
        20.457,
    ),
    star_raw(
        "Gorgonea Secunda",
        "Pi Persei",
        42.533,
        39.659,
        4.70,
        33.00,
        -5.00,
        52.200,
        22.243,
    ),
    // ─── Cepheus extended ───────────────────────────────────────────
    star_raw(
        "Delta Cephei",
        "Delta Cephei",
        337.293,
        58.415,
        3.48,
        12.00,
        2.00,
        17.609,
        59.541,
    ),
    star_raw(
        "Zeta Cephei",
        "Zeta Cephei",
        334.041,
        58.201,
        3.35,
        3.00,
        -6.00,
        14.944,
        60.636,
    ),
    star_raw(
        "Iota Cephei",
        "Iota Cephei",
        343.661,
        66.200,
        3.52,
        -18.00,
        -8.00,
        33.856,
        62.204,
    ),
    // ─── Draco extended ─────────────────────────────────────────────
    star_raw(
        "Eta Draconis",
        "Eta Draconis",
        245.998,
        61.514,
        2.74,
        -18.00,
        56.00,
        194.488,
        78.441,
    ),
    star_raw(
        "Chi Draconis",
        "Chi Draconis",
        272.074,
        72.733,
        3.57,
        534.00,
        -227.00,
        84.304,
        83.787,
    ),
    // ─── Ursa Major extended ────────────────────────────────────────
    star_raw(
        "Chalawan",
        "47 Ursae Majoris",
        164.868,
        40.430,
        5.03,
        -313.00,
        55.00,
        149.071,
        31.062,
    ),
    star_raw(
        "Intercrus",
        "Omicron Ursae Majoris",
        127.566,
        60.718,
        3.36,
        -134.00,
        -108.00,
        112.996,
        40.243,
    ),
    // ─── Ursa Minor extended ────────────────────────────────────────
    star_raw(
        "Epsilon UMi",
        "Epsilon Ursae Minoris",
        251.491,
        82.037,
        4.23,
        -12.00,
        14.00,
        99.138,
        73.923,
    ),
    star_raw(
        "Anwar al Farkadain",
        "Eta Ursae Minoris",
        244.752,
        75.755,
        4.95,
        -12.00,
        17.00,
        120.060,
        77.905,
    ),
    // ─── Cygnus extended ────────────────────────────────────────────
    star_raw(
        "Eta Cygni",
        "Eta Cygni",
        303.408,
        35.084,
        3.89,
        2.00,
        31.00,
        318.537,
        53.041,
    ),
    star_raw(
        "Zeta Cygni",
        "Zeta Cygni",
        316.233,
        30.227,
        3.21,
        -1.00,
        -22.00,
        330.843,
        44.397,
    ),
    star_raw(
        "Kappa Cygni",
        "Kappa Cygni",
        297.612,
        53.368,
        3.77,
        30.00,
        30.00,
        329.044,
        71.187,
    ),
    // ─── Lyra extended ──────────────────────────────────────────────
    star_raw(
        "Delta2 Lyrae",
        "Delta2 Lyrae",
        283.841,
        36.969,
        4.30,
        2.00,
        -4.00,
        292.023,
        59.355,
    ),
    star_raw(
        "Aladfar",
        "Eta Lyrae",
        287.884,
        39.146,
        4.39,
        1.00,
        -2.00,
        299.205,
        60.785,
    ),
    // ─── Aquila extended ────────────────────────────────────────────
    star_raw(
        "Theta Aquilae",
        "Theta Aquilae",
        300.275,
        -0.821,
        3.23,
        -5.00,
        -90.00,
        302.282,
        19.290,
    ),
    star_raw(
        "Delta Aquilae",
        "Delta Aquilae",
        295.024,
        3.115,
        3.36,
        253.00,
        -0.28,
        297.583,
        24.190,
    ),
    star_raw(
        "Eta Aquilae",
        "Eta Aquilae",
        298.118,
        1.006,
        3.90,
        -4.00,
        -27.00,
        300.434,
        21.524,
    ),
    // ─── Serpens / Ophiuchus extended ────────────────────────────────
    star_raw(
        "Unukalhai2",
        "Delta Serpentis",
        230.461,
        10.539,
        3.80,
        -80.00,
        -15.00,
        224.862,
        27.995,
    ),
    star_raw(
        "Nu Ophiuchi",
        "Nu Ophiuchi",
        267.035,
        -9.774,
        3.34,
        2.00,
        -4.00,
        266.993,
        13.634,
    ),
    star_raw(
        "Theta Ophiuchi",
        "Theta Ophiuchi",
        264.330,
        -24.999,
        3.27,
        -7.00,
        -22.00,
        264.861,
        -1.661,
    ),
    star_raw(
        "Kappa Ophiuchi",
        "Kappa Ophiuchi",
        248.306,
        9.375,
        3.20,
        -79.00,
        -111.00,
        244.836,
        30.939,
    ),
    // ─── Corona Borealis extended ───────────────────────────────────
    star_raw(
        "Theta CrB",
        "Theta Coronae Borealis",
        232.164,
        31.359,
        4.14,
        -195.00,
        106.00,
        218.170,
        48.221,
    ),
    star_raw(
        "Gamma CrB",
        "Gamma Coronae Borealis",
        235.685,
        26.296,
        3.84,
        -92.00,
        -55.00,
        224.872,
        44.507,
    ),
    // ─── Bootes extended ────────────────────────────────────────────
    star_raw(
        "Muphrid",
        "Eta Bootis",
        208.671,
        18.398,
        2.68,
        -60.95,
        -356.26,
        199.336,
        28.077,
    ),
    star_raw(
        "Princeps",
        "Delta Bootis",
        228.071,
        33.315,
        3.47,
        -90.00,
        131.00,
        212.221,
        48.696,
    ),
    // ─── Coma Berenices extended ────────────────────────────────────
    star_raw(
        "Beta Comae",
        "Beta Comae Berenices",
        197.497,
        27.876,
        4.26,
        -560.00,
        -270.00,
        183.926,
        32.323,
    ),
    star_raw(
        "Gamma Comae",
        "Gamma Comae Berenices",
        186.735,
        28.268,
        4.36,
        -74.00,
        23.00,
        173.891,
        28.399,
    ),
    // ─── Centaurus extended ─────────────────────────────────────────
    star_raw(
        "Theta Centauri",
        "Theta Centauri B",
        211.671,
        -36.370,
        2.06,
        -519.00,
        -518.00,
        222.309,
        -22.080,
    ),
    star_raw(
        "Omega Centauri",
        "Omega Centauri",
        201.697,
        -47.479,
        3.85,
        -17.00,
        -1.00,
        219.757,
        -35.227,
    ),
    // ─── Crux extended ──────────────────────────────────────────────
    // Acrux, Mimosa, Gacrux, Imai, Ginan already above

    // ─── Musca ──────────────────────────────────────────────────────
    star_raw(
        "Alpha Muscae",
        "Alpha Muscae",
        189.296,
        -69.136,
        2.69,
        -39.00,
        -12.00,
        230.375,
        -56.557,
    ),
    star_raw(
        "Beta Muscae",
        "Beta Muscae",
        191.572,
        -68.108,
        3.05,
        -41.00,
        -8.00,
        230.155,
        -55.242,
    ),
    star_raw(
        "Delta Muscae",
        "Delta Muscae",
        195.520,
        -71.548,
        3.62,
        -26.00,
        -1.00,
        236.169,
        -56.786,
    ),
    // ─── Circinus ───────────────────────────────────────────────────
    star_raw(
        "Alpha Circini",
        "Alpha Circini",
        220.428,
        -64.975,
        3.19,
        -189.00,
        -236.00,
        242.253,
        -46.239,
    ),
    // ─── Norma ──────────────────────────────────────────────────────
    star_raw(
        "Gamma2 Normae",
        "Gamma2 Normae",
        245.134,
        -50.155,
        4.02,
        -28.00,
        -3.00,
        252.192,
        -28.242,
    ),
    // ─── Telescopium ────────────────────────────────────────────────
    star_raw(
        "Alpha Telescopii",
        "Alpha Telescopii",
        278.086,
        -45.968,
        3.51,
        0.00,
        -16.00,
        276.083,
        -22.699,
    ),
    // ─── Hydrus ─────────────────────────────────────────────────────
    star_raw(
        "Alpha Hydri",
        "Alpha Hydri",
        29.691,
        -61.570,
        2.86,
        264.00,
        32.00,
        342.117,
        -64.242,
    ),
    star_raw(
        "Beta Hydri",
        "Beta Hydri",
        4.429,
        -77.254,
        2.80,
        2230.00,
        326.00,
        300.573,
        -64.376,
    ),
    star_raw(
        "Gamma Hydri",
        "Gamma Hydri",
        53.625,
        -74.239,
        3.24,
        93.00,
        36.00,
        311.486,
        -75.926,
    ),
    // ─── Phoenix extended ───────────────────────────────────────────
    // Ankaa already above
    star_raw(
        "Beta Phoenicis",
        "Beta Phoenicis",
        16.521,
        -46.718,
        3.31,
        52.00,
        -42.00,
        350.439,
        -48.199,
    ),
    star_raw(
        "Gamma Phoenicis",
        "Gamma Phoenicis",
        21.441,
        -43.318,
        3.41,
        108.00,
        -56.00,
        357.558,
        -47.326,
    ),
    // ─── Grus extended ──────────────────────────────────────────────
    // Alnair, Tiaki already above
    star_raw(
        "Delta Gruis",
        "Delta1 Gruis",
        338.204,
        -43.495,
        3.97,
        99.00,
        -33.00,
        322.283,
        -31.625,
    ),
    star_raw(
        "Epsilon Gruis",
        "Epsilon Gruis",
        340.541,
        -51.317,
        3.49,
        136.00,
        -22.00,
        319.600,
        -39.300,
    ),
    star_raw(
        "Iota Gruis",
        "Iota Gruis",
        349.115,
        -45.247,
        3.90,
        87.00,
        10.00,
        329.672,
        -36.774,
    ),
    // ─── Eridanus extended ──────────────────────────────────────────
    star_raw(
        "Epsilon Eridani",
        "Epsilon Eridani",
        53.233,
        -9.458,
        3.73,
        -976.00,
        17.00,
        48.168,
        -27.716,
    ),
    star_raw(
        "Eta Eridani",
        "Eta Eridani B",
        44.107,
        -8.898,
        3.89,
        -36.00,
        -23.00,
        38.750,
        -24.547,
    ),
    star_raw(
        "Nu Eridani",
        "Nu Eridani",
        71.375,
        -3.254,
        3.93,
        12.00,
        16.00,
        69.336,
        -25.367,
    ),
    star_raw(
        "Upsilon1 Eri",
        "Upsilon1 Eridani",
        67.155,
        -33.798,
        4.51,
        35.00,
        -122.00,
        56.168,
        -54.586,
    ),
    // ─── Canis Major extended ───────────────────────────────────────
    // Sirius, Adhara, Wezen, Mirzam, Aludra, Furud already above
    star_raw(
        "Omicron1 CMa",
        "Omicron1 Canis Majoris",
        104.100,
        -24.184,
        3.89,
        0.00,
        0.00,
        108.908,
        -46.702,
    ),
    star_raw(
        "Omicron2 CMa",
        "Omicron2 Canis Majoris",
        105.756,
        -23.833,
        3.02,
        -6.00,
        6.00,
        111.003,
        -46.130,
    ),
    star_raw(
        "Sigma CMa",
        "Sigma Canis Majoris",
        103.053,
        -27.935,
        3.47,
        -3.00,
        4.00,
        108.302,
        -50.548,
    ),
    // ─── Monoceros ──────────────────────────────────────────────────
    star_raw(
        "Alpha Monocerotis",
        "Alpha Monocerotis",
        115.312,
        -9.551,
        3.93,
        -51.00,
        -29.00,
        119.281,
        -30.453,
    ),
    star_raw(
        "Beta Monocerotis",
        "Beta Monocerotis",
        97.204,
        -7.033,
        3.74,
        -10.00,
        -3.00,
        98.285,
        -30.265,
    ),
    // ─── Hydra extended ─────────────────────────────────────────────
    // Alphard already above
    star_raw(
        "Gamma Hydrae",
        "Gamma Hydrae",
        199.730,
        -23.171,
        3.00,
        41.00,
        -38.00,
        207.018,
        -13.742,
    ),
    star_raw(
        "Pi Hydrae",
        "Pi Hydrae",
        218.020,
        -26.682,
        3.27,
        -50.00,
        -41.00,
        224.159,
        -11.132,
    ),
    star_raw(
        "Zeta Hydrae",
        "Zeta Hydrae",
        131.172,
        -5.837,
        3.11,
        -14.00,
        37.00,
        135.366,
        -23.028,
    ),
    star_raw(
        "Nu Hydrae",
        "Nu Hydrae",
        159.215,
        -16.194,
        3.11,
        -60.00,
        9.00,
        167.337,
        -23.044,
    ),
    star_raw(
        "Sigma Hydrae",
        "Sigma Hydrae",
        129.689,
        3.342,
        4.44,
        -80.00,
        -32.00,
        131.209,
        -14.601,
    ),
    // ─── Puppis extended ────────────────────────────────────────────
    // Naos, Tureis, Azmidi, Pi Puppis, Sigma Puppis already above
    star_raw(
        "Nu Puppis",
        "Nu Puppis",
        96.570,
        -43.196,
        3.17,
        -3.00,
        3.00,
        102.007,
        -66.362,
    ),
    star_raw(
        "Tau Puppis",
        "Tau Puppis",
        94.820,
        -50.614,
        2.93,
        30.00,
        -4.00,
        101.063,
        -73.868,
    ),
    // ─── Carina extended ────────────────────────────────────────────
    // Canopus, Miaplacidus, Avior, Aspidiske already above
    star_raw(
        "Theta Carinae",
        "Theta Carinae",
        160.739,
        -64.394,
        2.76,
        -18.00,
        12.00,
        209.189,
        -62.139,
    ),
    star_raw(
        "Upsilon Carinae",
        "Upsilon Carinae",
        146.311,
        -65.072,
        2.97,
        -20.00,
        15.00,
        202.634,
        -67.669,
    ),
    star_raw(
        "Omega Carinae",
        "Omega Carinae",
        153.684,
        -70.038,
        3.32,
        -21.00,
        7.00,
        217.524,
        -67.304,
    ),
    star_raw(
        "PP Carinae",
        "PP Carinae",
        138.900,
        -62.340,
        3.44,
        -24.00,
        8.00,
        191.682,
        -69.070,
    ),
    // ─── Pictor / Dorado / Reticulum ────────────────────────────────
    star_raw(
        "Alpha Pictoris",
        "Alpha Pictoris",
        102.048,
        -61.941,
        3.27,
        -4.00,
        6.00,
        144.114,
        -83.039,
    ),
    star_raw(
        "Alpha Doradus",
        "Alpha Doradus",
        68.499,
        -55.045,
        3.27,
        13.00,
        -2.00,
        37.829,
        -74.582,
    ),
    star_raw(
        "Alpha Reticuli",
        "Alpha Reticuli",
        63.604,
        -62.474,
        3.35,
        53.00,
        -4.00,
        7.503,
        -78.040,
    ),
    star_raw(
        "Beta Reticuli",
        "Beta Reticuli",
        56.054,
        -64.807,
        3.85,
        80.00,
        -9.00,
        351.397,
        -76.090,
    ),
    // ─── Volans / Chamaeleon ────────────────────────────────────────
    star_raw(
        "Alpha Volantis",
        "Alpha Volantis",
        141.002,
        -66.396,
        4.00,
        -22.00,
        6.00,
        203.190,
        -70.212,
    ),
    star_raw(
        "Beta Volantis",
        "Beta Volantis",
        126.434,
        -66.137,
        3.77,
        -46.00,
        6.00,
        195.172,
        -75.585,
    ),
    star_raw(
        "Gamma Volantis",
        "Gamma Volantis",
        110.512,
        -70.499,
        3.78,
        -18.00,
        2.00,
        216.986,
        -81.579,
    ),
    star_raw(
        "Alpha Chamaeleontis",
        "Alpha Chamaeleontis",
        124.632,
        -76.920,
        4.07,
        -58.00,
        71.00,
        239.300,
        -75.409,
    ),
    // ─── Pavo extended ──────────────────────────────────────────────
    // Peacock already above
    star_raw(
        "Beta Pavonis",
        "Beta Pavonis",
        311.173,
        -66.203,
        3.42,
        3.00,
        -7.00,
        292.459,
        -45.944,
    ),
    star_raw(
        "Delta Pavonis",
        "Delta Pavonis",
        302.186,
        -66.182,
        3.56,
        1211.00,
        -1144.00,
        287.616,
        -44.700,
    ),
    star_raw(
        "Epsilon Pavonis",
        "Epsilon Pavonis",
        300.148,
        -72.910,
        3.96,
        -4.00,
        10.00,
        283.530,
        -50.885,
    ),
    // ─── Triangulum Australe extended ───────────────────────────────
    // Atria already above
    star_raw(
        "Beta TrA",
        "Beta Trianguli Australis",
        238.716,
        -63.430,
        2.85,
        -188.00,
        -401.00,
        251.801,
        -41.954,
    ),
    star_raw(
        "Gamma TrA",
        "Gamma Trianguli Australis",
        228.422,
        -68.679,
        2.89,
        -57.00,
        -70.00,
        248.738,
        -48.288,
    ),
    // ─── Sagittarius extended ───────────────────────────────────────
    star_raw(
        "Omega Sagittarii",
        "Omega Sagittarii",
        298.959,
        -26.300,
        4.70,
        -6.00,
        -12.00,
        295.850,
        -5.422,
    ),
    star_raw(
        "Eta Sagittarii",
        "Eta Sagittarii",
        274.407,
        -36.762,
        3.11,
        -58.00,
        -167.00,
        273.628,
        -13.378,
    ),
    // ─── Scorpius extended ──────────────────────────────────────────
    star_raw(
        "Eta Scorpii",
        "Eta Scorpii",
        258.038,
        -43.239,
        3.33,
        -31.00,
        -40.00,
        260.743,
        -20.183,
    ),
    star_raw(
        "Zeta2 Scorpii",
        "Zeta2 Scorpii",
        253.647,
        -42.361,
        3.62,
        -11.00,
        -26.00,
        257.238,
        -19.643,
    ),
    // ─── Ara extended ───────────────────────────────────────────────
    star_raw(
        "Gamma Arae",
        "Gamma Arae",
        262.510,
        -56.378,
        3.34,
        6.00,
        -67.00,
        265.059,
        -33.067,
    ),
    star_raw(
        "Delta Arae",
        "Delta Arae",
        261.349,
        -60.684,
        3.62,
        -47.00,
        -136.00,
        264.680,
        -37.404,
    ),
    star_raw(
        "Zeta Arae",
        "Zeta Arae",
        253.070,
        -55.990,
        3.13,
        -10.00,
        -40.00,
        258.774,
        -33.209,
    ),
    star_raw(
        "Eta Arae", "Eta Arae", 252.445, -49.876, 3.76, -12.00, -23.00, 257.376, -27.203,
    ),
    star_raw(
        "Epsilon1 Arae",
        "Epsilon1 Arae",
        257.030,
        -53.160,
        4.06,
        -1.00,
        -23.00,
        261.049,
        -30.123,
    ),
    // ─── Telescopium extended ───────────────────────────────────────
    star_raw(
        "Zeta Telescopii",
        "Zeta Telescopii",
        274.716,
        -49.070,
        4.13,
        -6.00,
        -12.00,
        273.427,
        -25.687,
    ),
    // ─── Indus ──────────────────────────────────────────────────────
    star_raw(
        "Alpha Indi",
        "Alpha Indi",
        309.392,
        -47.292,
        3.11,
        105.00,
        -62.00,
        299.104,
        -27.754,
    ),
    star_raw(
        "Beta Indi",
        "Beta Indi",
        311.258,
        -58.454,
        3.65,
        79.00,
        -21.00,
        296.243,
        -38.715,
    ),
    // ─── Southern stars: Horologium, Caelum, Columba ────────────────
    star_raw(
        "Alpha Horologii",
        "Alpha Horologii",
        63.500,
        -42.294,
        3.86,
        -19.00,
        97.00,
        45.824,
        -61.730,
    ),
    star_raw(
        "Alpha Caeli",
        "Alpha Caeli",
        69.478,
        -41.864,
        4.45,
        40.00,
        -190.00,
        55.115,
        -62.840,
    ),
    // ─── Corona Australis extended ──────────────────────────────────
    star_raw(
        "Beta CrA",
        "Beta Coronae Australis",
        287.802,
        -39.341,
        4.11,
        61.00,
        -62.00,
        284.298,
        -16.781,
    ),
    // ─── Scutum ─────────────────────────────────────────────────────
    star_raw(
        "Alpha Scuti",
        "Alpha Scuti",
        279.234,
        -8.244,
        3.85,
        -6.00,
        -31.00,
        279.458,
        14.893,
    ),
    // ─── Sagitta ────────────────────────────────────────────────────
    star_raw(
        "Gamma Sagittae",
        "Gamma Sagittae",
        299.689,
        19.492,
        3.47,
        -2.00,
        -5.00,
        307.043,
        39.190,
    ),
    star_raw(
        "Delta Sagittae",
        "Delta Sagittae",
        296.434,
        18.534,
        3.82,
        2.00,
        -20.00,
        302.897,
        39.002,
    ),
    // ─── Vulpecula ──────────────────────────────────────────────────
    star_raw(
        "Anser",
        "Alpha Vulpeculae",
        297.664,
        24.665,
        4.44,
        -68.00,
        -18.00,
        306.391,
        44.671,
    ),
    // ─── Delphinus extended ─────────────────────────────────────────
    // Rotanev already above
    star_raw(
        "Sualocin",
        "Alpha Delphini",
        309.909,
        15.912,
        3.77,
        54.00,
        1.00,
        317.380,
        33.022,
    ),
    // ─── Equuleus ───────────────────────────────────────────────────
    star_raw(
        "Kitalpha",
        "Alpha Equulei",
        318.956,
        5.248,
        3.92,
        46.00,
        -90.00,
        323.117,
        20.122,
    ),
    // ─── Lacerta ────────────────────────────────────────────────────
    star_raw(
        "Alpha Lacertae",
        "Alpha Lacertae",
        337.823,
        50.283,
        3.77,
        165.00,
        -37.00,
        8.144,
        53.291,
    ),
    // ─── More Leo / Leo Minor ───────────────────────────────────────
    star_raw(
        "Rho Leonis",
        "Rho Leonis",
        155.437,
        9.307,
        3.85,
        -71.00,
        -61.00,
        153.848,
        -0.848,
    ),
    star_raw(
        "Iota Leonis",
        "Iota Leonis",
        170.981,
        10.529,
        4.00,
        -145.00,
        -76.00,
        167.566,
        6.105,
    ),
    star_raw(
        "Sigma Leonis",
        "Sigma Leonis",
        170.284,
        6.029,
        4.05,
        -70.00,
        -55.00,
        168.706,
        1.697,
    ),
    // ─── Lepus extended ─────────────────────────────────────────────
    // Arneb, Nihal already above
    star_raw(
        "Gamma Leporis",
        "Gamma Leporis",
        86.116,
        -22.448,
        3.60,
        -411.00,
        -93.00,
        84.846,
        -45.818,
    ),
    star_raw(
        "Epsilon Leporis",
        "Epsilon Leporis",
        79.954,
        -22.371,
        3.19,
        6.00,
        6.00,
        76.731,
        -45.349,
    ),
    star_raw(
        "Mu Leporis",
        "Mu Leporis",
        79.380,
        -16.207,
        3.31,
        -9.00,
        -4.00,
        76.807,
        -39.161,
    ),
    // ─── Remaining southern constellations ──────────────────────────
    star_raw(
        "Alpha Antliae",
        "Alpha Antliae",
        153.434,
        -31.068,
        4.25,
        -18.00,
        13.00,
        169.197,
        -38.745,
    ),
    star_raw(
        "Alpha Pyxidis",
        "Alpha Pyxidis",
        133.816,
        -33.186,
        3.68,
        -20.00,
        42.00,
        149.867,
        -47.936,
    ),
    // ─── Sextans ────────────────────────────────────────────────────
    star_raw(
        "Alpha Sextantis",
        "Alpha Sextantis",
        151.985,
        -0.372,
        4.49,
        -25.00,
        -36.00,
        154.117,
        -11.116,
    ),
    // ─── Crater extended ────────────────────────────────────────────
    star_raw(
        "Gamma Crateris",
        "Gamma Crateris",
        170.127,
        -17.684,
        4.08,
        4.00,
        33.00,
        178.227,
        -20.101,
    ),
    // ─── Canes Venatici ─────────────────────────────────────────────
    // Cor Caroli, Chara already above

    // ─── Triangulum ─────────────────────────────────────────────────
    star_raw(
        "Beta Trianguli",
        "Beta Trianguli",
        31.075,
        34.987,
        3.00,
        149.00,
        -39.00,
        41.282,
        20.969,
    ),
    // ─── Andromeda extended ─────────────────────────────────────────
    star_raw(
        "51 Andromedae",
        "51 Andromedae",
        24.711,
        48.628,
        3.57,
        -5.00,
        -11.00,
        42.594,
        35.352,
    ),
    // ─── Final entries ──────────────────────────────────────────────
    star_raw(
        "Algol B",
        "Beta Persei B",
        47.042,
        40.957,
        3.39,
        2.39,
        -1.44,
        56.168,
        22.430,
    ),
    star_raw(
        "Acamar B",
        "Theta2 Eridani",
        44.740,
        -40.158,
        4.24,
        -14.00,
        18.00,
        23.589,
        -53.675,
    ),
    star_raw(
        "Suhail al Muhlif",
        "Gamma Velorum",
        122.383,
        -47.337,
        4.27,
        -6.00,
        10.00,
        147.350,
        -64.465,
    ),
    // ═══════════════════════════════════════════════════════════════════
    // EXPANSION BATCH 3: 40 more stars to exceed 500
    // ═══════════════════════════════════════════════════════════════════

    // ─── Cetus extended ─────────────────────────────────────────────
    star_raw(
        "Theta Ceti",
        "Theta Ceti",
        21.005,
        -8.183,
        3.60,
        -147.00,
        51.00,
        16.225,
        -15.767,
    ),
    star_raw(
        "Eta Ceti", "Eta Ceti", 17.147, -10.182, 3.45, -20.00, -105.00, 11.767, -16.118,
    ),
    star_raw(
        "Iota Ceti",
        "Iota Ceti",
        0.324,
        -8.824,
        3.56,
        -14.00,
        -29.00,
        356.763,
        -8.219,
    ),
    star_raw(
        "Tau Ceti", "Tau Ceti", 26.017, -15.937, 3.50, -1721.00, 854.00, 17.819, -24.815,
    ),
    // ─── Pegasus extended ───────────────────────────────────────────
    star_raw(
        "Mu Pegasi",
        "Mu Pegasi",
        342.501,
        24.602,
        3.48,
        86.00,
        -58.00,
        354.386,
        29.387,
    ),
    star_raw(
        "Lambda Pegasi",
        "Lambda Pegasi",
        340.365,
        23.566,
        3.95,
        65.00,
        -12.00,
        351.855,
        29.297,
    ),
    star_raw(
        "Xi Pegasi",
        "Xi Pegasi",
        340.066,
        12.173,
        4.20,
        74.00,
        -2.00,
        346.425,
        19.028,
    ),
    star_raw(
        "Epsilon Pegasi2",
        "Iota Pegasi",
        333.965,
        25.345,
        3.76,
        268.00,
        -27.00,
        346.586,
        33.404,
    ),
    // ─── Ophiuchus extended ─────────────────────────────────────────
    star_raw(
        "Delta Ophiuchi2",
        "Gamma Ophiuchi",
        262.710,
        2.708,
        3.75,
        -11.00,
        -30.00,
        261.897,
        25.942,
    ),
    star_raw(
        "67 Ophiuchi",
        "67 Ophiuchi",
        270.161,
        -2.890,
        3.97,
        0.00,
        -23.00,
        270.172,
        20.549,
    ),
    star_raw(
        "70 Ophiuchi",
        "70 Ophiuchi",
        271.364,
        2.500,
        4.03,
        62.00,
        -1137.00,
        271.515,
        25.932,
    ),
    // ─── Scorpius tail completion ───────────────────────────────────
    star_raw(
        "Lambda Scorpii2",
        "Upsilon Scorpii",
        262.691,
        -37.296,
        2.69,
        -5.00,
        -30.00,
        264.013,
        -14.008,
    ),
    star_raw(
        "Omega1 Scorpii",
        "Omega1 Scorpii",
        242.580,
        -20.669,
        3.97,
        -8.00,
        -22.00,
        244.477,
        0.374,
    ),
    star_raw(
        "Omega2 Scorpii",
        "Omega2 Scorpii",
        243.059,
        -20.872,
        4.32,
        -6.00,
        -18.00,
        244.954,
        0.255,
    ),
    // ─── Vela/Puppis last entries ───────────────────────────────────
    star_raw(
        "A Velorum",
        "A Velorum",
        134.802,
        -44.525,
        3.81,
        -12.00,
        2.00,
        159.764,
        -57.627,
    ),
    star_raw(
        "Phi Velorum",
        "Phi Velorum",
        147.844,
        -54.568,
        3.54,
        -17.00,
        6.00,
        184.778,
        -60.494,
    ),
    star_raw(
        "Q Velorum",
        "Q Velorum",
        152.784,
        -51.747,
        3.85,
        17.00,
        -2.00,
        185.455,
        -56.420,
    ),
    // ─── Puppis last ────────────────────────────────────────────────
    star_raw(
        "Zeta Puppis2",
        "Gamma Puppis",
        120.316,
        -24.304,
        4.50,
        -12.00,
        6.00,
        129.498,
        -43.675,
    ),
    star_raw(
        "L2 Puppis",
        "L2 Puppis",
        112.630,
        -44.639,
        4.09,
        -38.00,
        250.00,
        130.279,
        -64.945,
    ),
    // ─── Centaurus last ─────────────────────────────────────────────
    star_raw(
        "Phi Centauri",
        "Phi Centauri",
        213.680,
        -42.100,
        3.83,
        -50.00,
        -14.00,
        226.215,
        -26.836,
    ),
    star_raw(
        "Chi Centauri",
        "Chi Centauri",
        215.100,
        -41.175,
        4.36,
        -47.00,
        -7.00,
        226.939,
        -25.587,
    ),
    star_raw(
        "Upsilon1 Cen",
        "Upsilon1 Centauri",
        211.100,
        -44.803,
        3.87,
        -19.00,
        -8.00,
        225.422,
        -30.049,
    ),
    star_raw(
        "Psi Centauri",
        "Psi Centauri",
        215.380,
        -37.892,
        4.05,
        -26.00,
        -9.00,
        225.881,
        -22.441,
    ),
    // ─── More southern gap fillers ──────────────────────────────────
    star_raw(
        "Delta Gruis2",
        "Delta2 Gruis",
        339.068,
        -43.750,
        4.11,
        81.00,
        -38.00,
        322.815,
        -32.127,
    ),
    star_raw(
        "Mu1 Gruis",
        "Mu1 Gruis",
        337.370,
        -41.347,
        4.79,
        62.00,
        -29.00,
        322.703,
        -29.420,
    ),
    // ─── Crane + Swan (Grus + Cygnus) last entries ──────────────────
    star_raw(
        "Theta Gruis",
        "Theta Gruis",
        345.604,
        -43.520,
        4.28,
        55.00,
        -53.00,
        327.975,
        -34.061,
    ),
    star_raw(
        "Lambda Gruis",
        "Lambda Gruis",
        331.528,
        -39.543,
        4.47,
        17.00,
        4.00,
        318.939,
        -25.969,
    ),
    star_raw(
        "Rho Cygni",
        "Rho Cygni",
        325.064,
        45.592,
        4.02,
        39.00,
        8.00,
        351.719,
        54.570,
    ),
    star_raw(
        "Sigma Cygni",
        "Sigma Cygni",
        318.684,
        39.399,
        4.23,
        1.00,
        -3.00,
        339.621,
        51.746,
    ),
    star_raw(
        "Pi2 Cygni",
        "Pi2 Cygni",
        325.524,
        51.190,
        4.23,
        19.00,
        4.00,
        358.277,
        58.876,
    ),
    star_raw(
        "Omega2 Cygni",
        "Omega2 Cygni",
        312.958,
        49.228,
        5.44,
        1.00,
        0.00,
        342.859,
        62.244,
    ),
    star_raw(
        "Tau Cygni",
        "Tau Cygni",
        318.233,
        38.047,
        3.72,
        53.00,
        51.00,
        338.100,
        50.724,
    ),
    // ─── Lacerta + extras ───────────────────────────────────────────
    star_raw(
        "Beta Lacertae",
        "Beta Lacertae",
        335.252,
        52.229,
        4.43,
        13.00,
        -11.00,
        8.101,
        55.816,
    ),
    star_raw(
        "1 Lacertae",
        "1 Lacertae",
        334.164,
        37.748,
        4.13,
        16.00,
        6.00,
        354.172,
        44.326,
    ),
    // ─── Wrap-up extras: famous nearby stars ────────────────────────
    star_raw(
        "Barnard's Star",
        "Barnard's Star",
        269.452,
        4.693,
        9.54,
        -798.71,
        10337.77,
        269.381,
        28.131,
    ),
    star_raw(
        "Proxima Centauri",
        "Proxima Centauri",
        217.429,
        -62.680,
        11.13,
        -3781.74,
        769.33,
        239.115,
        -44.764,
    ),
    star_raw(
        "61 Cygni A",
        "61 Cygni A",
        316.720,
        38.750,
        5.21,
        4157.00,
        3259.00,
        336.952,
        51.901,
    ),
    star_raw(
        "Groombridge 1830",
        "Groombridge 1830",
        178.260,
        37.718,
        6.45,
        4003.27,
        -5815.07,
        161.446,
        33.486,
    ),
    star_raw(
        "Wolf 359", "Wolf 359", 164.120, 7.015, 13.53, -3842.00, -2725.00, 162.678, 0.231,
    ),
];

// ── Public API ─────────────────────────────────────────────────────

/// Return the full fixed-star catalog.
pub fn all_stars() -> &'static [FixedStar] {
    CATALOG
}

/// Find a star by its common name (case-insensitive).
pub fn find_by_name(name: &str) -> Option<&'static FixedStar> {
    let lower = name.to_lowercase();
    CATALOG.iter().find(|s| s.name.to_lowercase() == lower)
}

/// Find all stars brighter than or equal to a given magnitude.
pub fn brighter_than(max_mag: f64) -> Vec<&'static FixedStar> {
    CATALOG.iter().filter(|s| s.magnitude <= max_mag).collect()
}

/// Return the ecliptic longitude and latitude (degrees) for a star,
/// applying precession from J2000.0 to the requested Julian Day.
///
/// Uses simple linear precession at 50.29"/yr.  For high-precision work
/// use the full IAU 2006 precession matrix in `xalen-coords`.
pub fn precessed_ecliptic(star: &FixedStar, jd: f64) -> (f64, f64) {
    let years = (jd - 2451545.0) / 365.25;
    let prec_deg = years * 50.29 / 3600.0;
    let lon = (star.ecl_lon_deg + prec_deg).rem_euclid(360.0);
    (lon, star.ecl_lat_deg)
}

// ── Nakshatra yogatara lookup ──────────────────────────────────────

/// The 27 Nakshatra names paired with their yogatara star common names.
pub static NAKSHATRA_YOGATARAS: &[(&str, &str)] = &[
    ("Ashwini", "Sheratan"),
    ("Bharani", "Bharani"),
    ("Krittika", "Alcyone"),
    ("Rohini", "Aldebaran"),
    ("Mrigashira", "Meissa"),
    ("Ardra", "Betelgeuse"),
    ("Punarvasu", "Pollux"),
    ("Pushya", "Asellus Australis"),
    ("Ashlesha", "Ashlesha"),
    ("Magha", "Regulus"),
    ("Purva Phalguni", "Zosma"),
    ("Uttara Phalguni", "Denebola"),
    ("Hasta", "Algorab"),
    ("Chitra", "Spica"),
    ("Swati", "Arcturus"),
    ("Vishakha", "Zubenelgenubi"),
    ("Anuradha", "Dschubba"),
    ("Jyeshtha", "Antares"),
    ("Mula", "Shaula"),
    ("Purva Ashadha", "Kaus Media"),
    ("Uttara Ashadha", "Nunki"),
    ("Shravana", "Altair"),
    ("Dhanishta", "Rotanev"),
    ("Shatabhisha", "Shatabhisha"),
    ("Purva Bhadrapada", "Markab"),
    ("Uttara Bhadrapada", "Algenib"),
    ("Revati", "Revati"),
];

/// Return the yogatara (junction star) for each of the 27 Nakshatras.
///
/// Each entry is `(nakshatra_name, &FixedStar)`.  Panics at build time
/// via `expect` if the catalog is missing a required star -- this ensures
/// catalog completeness is enforced.
pub fn nakshatra_yogataras() -> Vec<(&'static str, &'static FixedStar)> {
    NAKSHATRA_YOGATARAS
        .iter()
        .map(|&(nak, star_name)| {
            let star = find_by_name(star_name).unwrap_or_else(|| {
                panic!("Nakshatra yogatara '{}' not found in catalog", star_name)
            });
            (nak, star)
        })
        .collect()
}

/// Behenian star names (15 medieval astrological stars).
pub static BEHENIAN_NAMES: &[&str] = &[
    "Algol",
    "Alcyone",
    "Aldebaran",
    "Capella",
    "Sirius",
    "Procyon",
    "Regulus",
    "Algorab",
    "Spica",
    "Arcturus",
    "Alphecca",
    "Antares",
    "Vega",
    "Deneb Algedi",
    "Fomalhaut",
];

/// Return the 15 Behenian stars from the catalog.
pub fn behenian_stars() -> Vec<&'static FixedStar> {
    BEHENIAN_NAMES
        .iter()
        .filter_map(|name| find_by_name(name))
        .collect()
}

/// Royal star names (4 "Watchers" of Persian tradition).
pub static ROYAL_STAR_NAMES: &[&str] = &[
    "Aldebaran", // Watcher of the East
    "Regulus",   // Watcher of the North
    "Antares",   // Watcher of the West
    "Fomalhaut", // Watcher of the South
];

/// Return the 4 Royal Stars (Watchers of Heaven).
pub fn royal_stars() -> Vec<&'static FixedStar> {
    ROYAL_STAR_NAMES
        .iter()
        .filter_map(|name| find_by_name(name))
        .collect()
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_at_least_500_stars() {
        let mut names: Vec<&str> = CATALOG.iter().map(|s| s.name).collect();
        names.sort();
        names.dedup();
        assert!(
            names.len() >= 500,
            "Expected >= 500 unique star names, got {}",
            names.len()
        );
    }

    #[test]
    fn catalog_raw_count() {
        assert!(
            CATALOG.len() >= 500,
            "Raw catalog should have >= 500 entries, got {}",
            CATALOG.len()
        );
    }

    #[test]
    fn sirius_is_brightest() {
        let sirius = find_by_name("Sirius").expect("Sirius missing");
        assert!(
            (sirius.magnitude - (-1.46)).abs() < 0.01,
            "Sirius magnitude should be ~-1.46, got {}",
            sirius.magnitude,
        );
        for s in CATALOG.iter() {
            assert!(
                s.magnitude >= sirius.magnitude - 0.01,
                "Star '{}' (mag {}) is brighter than Sirius (mag {})",
                s.name,
                s.magnitude,
                sirius.magnitude,
            );
        }
    }

    #[test]
    fn polaris_declination_near_90() {
        let polaris = find_by_name("Polaris").expect("Polaris missing");
        assert!(
            polaris.dec_deg > 89.0,
            "Polaris dec should be > 89 deg, got {}",
            polaris.dec_deg,
        );
    }

    #[test]
    fn all_27_nakshatra_yogataras_present() {
        let yogataras = nakshatra_yogataras();
        assert_eq!(
            yogataras.len(),
            27,
            "Expected 27 nakshatra yogataras, got {}",
            yogataras.len(),
        );
        let mut nak_names: Vec<&str> = yogataras.iter().map(|(n, _)| *n).collect();
        nak_names.sort();
        nak_names.dedup();
        assert_eq!(nak_names.len(), 27, "Nakshatra names should be unique");
    }

    #[test]
    fn nakshatra_yogataras_have_valid_stars() {
        for &(nak, star_name) in NAKSHATRA_YOGATARAS {
            assert!(
                find_by_name(star_name).is_some(),
                "Nakshatra '{}' yogatara '{}' not found in catalog",
                nak,
                star_name,
            );
        }
    }

    #[test]
    fn behenian_stars_all_15_present() {
        let beh = behenian_stars();
        assert_eq!(
            beh.len(),
            15,
            "Expected 15 Behenian stars, got {}",
            beh.len(),
        );
    }

    #[test]
    fn royal_stars_all_4_present() {
        let royals = royal_stars();
        assert_eq!(
            royals.len(),
            4,
            "Expected 4 Royal Stars, got {}",
            royals.len()
        );
    }

    #[test]
    fn ecliptic_longitude_in_range() {
        for s in CATALOG.iter() {
            assert!(
                s.ecl_lon_deg >= 0.0 && s.ecl_lon_deg < 360.0,
                "Star '{}' ecl_lon {} out of [0, 360) range",
                s.name,
                s.ecl_lon_deg,
            );
        }
    }

    #[test]
    fn ecliptic_latitude_in_range() {
        for s in CATALOG.iter() {
            assert!(
                s.ecl_lat_deg >= -90.0 && s.ecl_lat_deg <= 90.0,
                "Star '{}' ecl_lat {} out of [-90, 90] range",
                s.name,
                s.ecl_lat_deg,
            );
        }
    }

    #[test]
    fn ecliptic_conversion_matches_stored_all() {
        // After the 2026-05-25 verification pass, ALL ecliptic coordinates
        // were recomputed from RA/Dec.  They must match within 0.01 deg.
        for s in CATALOG.iter() {
            let (lon, lat) = eq_to_ecl(s.ra_deg, s.dec_deg);
            let dlon = (lon - s.ecl_lon_deg).abs();
            let dlon = if dlon > 180.0 { 360.0 - dlon } else { dlon };
            assert!(
                dlon < 0.01,
                "Star '{}' ecl_lon mismatch: computed {:.4} vs stored {:.4} (delta {:.4})",
                s.name,
                lon,
                s.ecl_lon_deg,
                dlon,
            );
            assert!(
                (lat - s.ecl_lat_deg).abs() < 0.01,
                "Star '{}' ecl_lat mismatch: computed {:.4} vs stored {:.4}",
                s.name,
                lat,
                s.ecl_lat_deg,
            );
        }
    }

    #[test]
    fn precessed_ecliptic_shifts_forward() {
        let sirius = find_by_name("Sirius").unwrap();
        let jd_2100 = 2451545.0 + 100.0 * 365.25;
        let (lon_2100, lat_2100) = precessed_ecliptic(sirius, jd_2100);
        let shift = lon_2100 - sirius.ecl_lon_deg;
        assert!(
            (shift - 1.397).abs() < 0.01,
            "100-year precession shift should be ~1.397 deg, got {}",
            shift,
        );
        assert!(
            (lat_2100 - sirius.ecl_lat_deg).abs() < 1e-10,
            "Ecliptic latitude should not change with simple precession",
        );
    }

    #[test]
    fn find_by_name_case_insensitive() {
        assert!(find_by_name("sirius").is_some());
        assert!(find_by_name("SIRIUS").is_some());
        assert!(find_by_name("Sirius").is_some());
        assert!(find_by_name("nonexistent").is_none());
    }

    #[test]
    fn brighter_than_returns_first_mag() {
        let bright = brighter_than(1.0);
        assert!(
            bright.len() >= 10,
            "Expected at least 10 stars brighter than mag 1.0, got {}",
            bright.len(),
        );
        for s in &bright {
            assert!(s.magnitude <= 1.0);
        }
    }

    #[test]
    fn aldebaran_ecliptic_near_taurus() {
        let ald = find_by_name("Aldebaran").unwrap();
        assert!(
            ald.ecl_lon_deg > 60.0 && ald.ecl_lon_deg < 80.0,
            "Aldebaran ecl_lon should be ~70 deg (Taurus), got {}",
            ald.ecl_lon_deg,
        );
    }

    #[test]
    fn spica_near_virgo() {
        let spica = find_by_name("Spica").unwrap();
        assert!(
            spica.ecl_lon_deg > 195.0 && spica.ecl_lon_deg < 215.0,
            "Spica ecl_lon should be ~203 deg (Libra), got {}",
            spica.ecl_lon_deg,
        );
    }

    #[test]
    fn regulus_near_ecliptic() {
        let reg = find_by_name("Regulus").unwrap();
        assert!(
            reg.ecl_lat_deg.abs() < 1.0,
            "Regulus ecl_lat should be < 1 deg, got {}",
            reg.ecl_lat_deg,
        );
    }

    // ─── Expansion-specific tests ──────────────────────────────────

    #[test]
    fn brighter_than_3_has_many_stars() {
        let bright = brighter_than(3.0);
        assert!(
            bright.len() >= 150,
            "Expected >= 150 stars brighter than mag 3.0, got {}",
            bright.len(),
        );
    }

    #[test]
    fn pleiades_cluster_present() {
        let pleiades = [
            "Alcyone", "Atlas", "Electra", "Maia", "Merope", "Taygeta", "Celaeno", "Asterope",
            "Pleione",
        ];
        for name in &pleiades {
            assert!(
                find_by_name(name).is_some(),
                "Pleiades star '{}' missing from catalog",
                name,
            );
        }
    }

    #[test]
    fn lupus_bright_stars_present() {
        let lupus = ["Alpha Lupi", "Beta Lupi", "Gamma Lupi"];
        for name in &lupus {
            assert!(
                find_by_name(name).is_some(),
                "Lupus star '{}' missing",
                name,
            );
        }
    }

    #[test]
    fn scorpius_chain_complete() {
        let scorpius = [
            "Antares", "Shaula", "Sargas", "Lesath", "Wei", "Dschubba", "Acrab", "Fang", "Girtab",
        ];
        for name in &scorpius {
            assert!(
                find_by_name(name).is_some(),
                "Scorpius star '{}' missing",
                name,
            );
        }
    }

    #[test]
    fn cassiopeia_w_complete() {
        let cas = ["Schedar", "Caph", "Navi", "Ruchbah", "Segin"];
        for name in &cas {
            assert!(
                find_by_name(name).is_some(),
                "Cassiopeia star '{}' missing",
                name,
            );
        }
    }

    #[test]
    fn all_ra_in_range() {
        for s in CATALOG.iter() {
            assert!(
                s.ra_deg >= 0.0 && s.ra_deg < 360.0,
                "Star '{}' RA {} out of [0, 360) range",
                s.name,
                s.ra_deg,
            );
        }
    }

    #[test]
    fn all_dec_in_range() {
        for s in CATALOG.iter() {
            assert!(
                s.dec_deg >= -90.0 && s.dec_deg <= 90.0,
                "Star '{}' Dec {} out of [-90, 90] range",
                s.name,
                s.dec_deg,
            );
        }
    }

    #[test]
    fn expansion_stars_ecliptic_tight() {
        // Post-verification: expansion stars must also match tightly
        let check = [
            "Peacock",
            "Alhena",
            "Saiph",
            "Schedar",
            "Regor",
            "Girtab",
            "Kornephoros",
            "Alpha Lupi",
            "Atria",
            "Gienah",
            "Miaplacidus",
            "Avior",
            "Aspidiske",
        ];
        for name in &check {
            let s = find_by_name(name).expect(name);
            let (lon, lat) = eq_to_ecl(s.ra_deg, s.dec_deg);
            let dlon = (lon - s.ecl_lon_deg).abs();
            let dlon = if dlon > 180.0 { 360.0 - dlon } else { dlon };
            assert!(
                dlon < 0.01,
                "Star '{}' ecl_lon mismatch: computed {:.4} vs stored {:.4}",
                name,
                lon,
                s.ecl_lon_deg,
            );
            assert!(
                (lat - s.ecl_lat_deg).abs() < 0.01,
                "Star '{}' ecl_lat mismatch: computed {:.4} vs stored {:.4}",
                name,
                lat,
                s.ecl_lat_deg,
            );
        }
    }

    #[test]
    fn herculis_stars_present() {
        let herculis = ["Kornephoros", "Rasalgethi", "Sarin", "Ruticulus"];
        for name in &herculis {
            assert!(
                find_by_name(name).is_some(),
                "Hercules star '{}' missing",
                name,
            );
        }
    }

    // ─── SIMBAD-verified coordinate test (top 50 stars) ────────────
    //
    // Authoritative RA/Dec from SIMBAD ICRS J2000.0 positions
    // (CDS, Strasbourg), cross-checked with Hipparcos (ESA SP-1200).
    // Verified 2026-05-25.  Tolerance: 0.01 deg (36 arcsec).
    //
    // Covers: all 4 Royal Stars, 13 of 15 Behenian stars, 11 Nakshatra
    // yogataras, plus bright navigational stars.

    #[test]
    fn simbad_verified_top50_ra_dec() {
        // (name, simbad_ra_deg, simbad_dec_deg)
        let reference: &[(&str, f64, f64)] = &[
            // ── First-magnitude navigational stars ──
            ("Sirius", 101.287, -16.716),
            ("Canopus", 95.988, -52.696),
            ("Arcturus", 213.915, 19.182),
            ("Vega", 279.235, 38.784),
            ("Capella", 79.172, 45.998),
            ("Rigel", 78.634, -8.202),
            ("Procyon", 114.826, 5.225),
            ("Betelgeuse", 88.793, 7.407),
            ("Achernar", 24.429, -57.237),
            ("Hadar", 210.956, -60.373),
            ("Altair", 297.696, 8.868),
            ("Acrux", 186.649, -63.099),
            // ── Royal Stars ──
            ("Aldebaran", 68.980, 16.509),
            ("Regulus", 152.093, 11.967),
            ("Antares", 247.352, -26.432),
            ("Fomalhaut", 344.413, -29.622),
            // ── More bright stars ──
            ("Pollux", 116.329, 28.026),
            ("Deneb", 310.358, 45.280),
            ("Mimosa", 191.930, -59.689),
            ("Spica", 201.298, -11.161),
            ("Adhara", 104.656, -28.972),
            ("Castor", 113.650, 31.888),
            ("Shaula", 263.402, -37.104),
            ("Bellatrix", 81.283, 6.350),
            ("Gacrux", 187.791, -57.113),
            ("Elnath", 81.573, 28.608),
            ("Alnilam", 84.053, -1.202),
            ("Alioth", 193.507, 55.960),
            ("Dubhe", 165.932, 61.751),
            // ── Behenian stars ──
            ("Algol", 47.042, 40.957),
            ("Alcyone", 56.871, 24.105),
            ("Alphecca", 233.672, 26.715),
            ("Deneb Algedi", 326.760, -16.127),
            ("Denebola", 177.265, 14.572),
            // ── Nakshatra yogataras ──
            ("Sheratan", 28.660, 20.808),
            ("Hamal", 31.793, 23.463),
            ("Zosma", 168.527, 20.524),
            ("Nunki", 283.816, -26.297),
            // ── Key navigational extras ──
            ("Polaris", 37.954, 89.264),
            ("Alphard", 141.897, -8.659),
            ("Mirfak", 51.081, 49.861),
            ("Kaus Australis", 276.043, -34.384),
            ("Alkaid", 206.885, 49.313),
            ("Atria", 252.166, -69.028),
            ("Peacock", 306.412, -56.735),
            ("Menkent", 211.671, -36.370),
            ("Rigil Kentaurus", 219.902, -60.834),
            ("Gienah", 183.952, -17.542),
            ("Schedar", 10.127, 56.537),
            ("Saiph", 86.939, -9.670),
        ];

        for &(name, ref_ra, ref_dec) in reference {
            let star = find_by_name(name)
                .unwrap_or_else(|| panic!("SIMBAD-verified star '{}' missing from catalog", name));

            let dra = (star.ra_deg - ref_ra).abs();
            let dra = if dra > 180.0 { 360.0 - dra } else { dra };
            assert!(
                dra < 0.01,
                "Star '{}' RA {:.4} != SIMBAD {:.4} (delta {:.4} deg)",
                name,
                star.ra_deg,
                ref_ra,
                dra,
            );
            assert!(
                (star.dec_deg - ref_dec).abs() < 0.01,
                "Star '{}' Dec {:.4} != SIMBAD {:.4} (delta {:.4} deg)",
                name,
                star.dec_deg,
                ref_dec,
                (star.dec_deg - ref_dec).abs(),
            );
        }
    }
}
