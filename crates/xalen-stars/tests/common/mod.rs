//! Swiss Ephemeris reference values shared by the integration tests of
//! `xalen-stars` and (through a `#[path]` include) `xalen-stars-hip-data`.
//!
//! Every reference longitude/latitude below is a REAL value produced by the
//! Swiss Ephemeris (`pyswisseph` 2.10.3.2, `libswe` 2.10.03). Each row is the
//! J2000.0 *mean* ecliptic place of the star, queried with the astrometric flag
//! set
//!
//!     SEFLG_SWIEPH | SEFLG_J2000 | SEFLG_NONUT | SEFLG_NOABERR | SEFLG_NOGDEFL
//!
//! at JD 2451545.0 (TT) using the official Swiss `sefstars.txt` star data file.
//!
//! To regenerate (requires the Swiss star file on disk):
//!
//! ```text
//!   curl -sL https://raw.githubusercontent.com/aloistr/swisseph/master/ephe/sefstars.txt \
//!     -o /tmp/ephe/sefstars.txt
//!   python3 - <<'PY'
//!   import swisseph as swe
//!   swe.set_ephe_path("/tmp/ephe")
//!   jd = 2451545.0
//!   fl = swe.FLG_SWIEPH|swe.FLG_J2000|swe.FLG_NONUT|swe.FLG_NOABERR|swe.FLG_NOGDEFL
//!   for n in ["Polaris","Sirius","Aldebaran","Regulus","Spica","Antares","Vega",
//!             "Deneb","Rigel","Betelgeuse","Arcturus","Fomalhaut","Canopus",
//!             "Capella","Procyon","Altair","Castor","Pollux","Algol","Bellatrix",
//!             "Alnilam","Elnath","Alphard","Denebola","Hamal","Achernar",
//!             "Alpheratz","Mirach","Acrux","Shaula","Nunki","Markab","Scheat",
//!             "Algenib","Dubhe","Alioth","Mizar","Diphda"]:
//!       xx,_,_ = swe.fixstar2(n, jd, fl)
//!       print(f"{n} {xx[0]:.6f} {xx[1]:.6f}")
//!   PY
//! ```
#![allow(dead_code)]

/// A Swiss-Ephemeris oracle row: (star name, J2000 ecliptic longitude deg,
/// J2000 ecliptic latitude deg). Every value is LIVE `swe.fixstar2` output —
/// see the module header for the exact flags, epoch and star-file provenance.
pub struct Oracle {
    pub name: &'static str,
    pub swiss_lon: f64,
    pub swiss_lat: f64,
}

/// 38 well-known named stars with their LIVE Swiss J2000 mean-ecliptic places.
pub const SWISS_ORACLE: &[Oracle] = &[
    Oracle {
        name: "Polaris",
        swiss_lon: 88.567582,
        swiss_lat: 66.101473,
    },
    Oracle {
        name: "Sirius",
        swiss_lon: 104.081678,
        swiss_lat: -39.605304,
    },
    Oracle {
        name: "Aldebaran",
        swiss_lon: 69.789184,
        swiss_lat: -5.467321,
    },
    Oracle {
        name: "Regulus",
        swiss_lon: 149.829145,
        swiss_lat: 0.464849,
    },
    Oracle {
        name: "Spica",
        swiss_lon: 203.841363,
        swiss_lat: -2.054487,
    },
    Oracle {
        name: "Antares",
        swiss_lon: 249.762295,
        swiss_lat: -4.569952,
    },
    Oracle {
        name: "Vega",
        swiss_lon: 285.316389,
        swiss_lat: 61.732822,
    },
    Oracle {
        name: "Deneb",
        swiss_lon: 335.329214,
        swiss_lat: 59.906140,
    },
    Oracle {
        name: "Rigel",
        swiss_lon: 76.829594,
        swiss_lat: -31.122760,
    },
    Oracle {
        name: "Betelgeuse",
        swiss_lon: 88.754602,
        swiss_lat: -16.027000,
    },
    Oracle {
        name: "Arcturus",
        swiss_lon: 204.233651,
        swiss_lat: 30.736231,
    },
    Oracle {
        name: "Fomalhaut",
        swiss_lon: 333.860228,
        swiss_lat: -21.135617,
    },
    Oracle {
        name: "Canopus",
        swiss_lon: 104.960545,
        swiss_lat: -75.823875,
    },
    Oracle {
        name: "Capella",
        swiss_lon: 81.857911,
        swiss_lat: 22.864282,
    },
    Oracle {
        name: "Procyon",
        swiss_lon: 115.785561,
        swiss_lat: -16.019629,
    },
    Oracle {
        name: "Altair",
        swiss_lon: 301.776395,
        swiss_lat: 29.303426,
    },
    Oracle {
        name: "Castor",
        swiss_lon: 110.240210,
        swiss_lat: 10.095768,
    },
    Oracle {
        name: "Pollux",
        swiss_lon: 113.215648,
        swiss_lat: 6.684202,
    },
    Oracle {
        name: "Algol",
        swiss_lon: 56.167575,
        swiss_lat: 22.428536,
    },
    Oracle {
        name: "Bellatrix",
        swiss_lon: 80.946467,
        swiss_lat: -16.816035,
    },
    Oracle {
        name: "Alnilam",
        swiss_lon: 83.463651,
        swiss_lat: -24.506375,
    },
    Oracle {
        name: "Elnath",
        swiss_lon: 82.574936,
        swiss_lat: 5.385117,
    },
    Oracle {
        name: "Alphard",
        swiss_lon: 147.279215,
        swiss_lat: -22.382383,
    },
    Oracle {
        name: "Denebola",
        swiss_lon: 171.617559,
        swiss_lat: 12.266884,
    },
    Oracle {
        name: "Hamal",
        swiss_lon: 37.662469,
        swiss_lat: 9.965113,
    },
    Oracle {
        name: "Achernar",
        swiss_lon: 345.311280,
        swiss_lat: -59.378150,
    },
    Oracle {
        name: "Alpheratz",
        swiss_lon: 14.308460,
        swiss_lat: 25.680439,
    },
    Oracle {
        name: "Mirach",
        swiss_lon: 30.405224,
        swiss_lat: 25.943362,
    },
    Oracle {
        name: "Acrux",
        swiss_lon: 221.869909,
        swiss_lat: -52.878864,
    },
    Oracle {
        name: "Shaula",
        swiss_lon: 264.585713,
        swiss_lat: -13.788463,
    },
    Oracle {
        name: "Nunki",
        swiss_lon: 282.385331,
        swiss_lat: -3.449530,
    },
    Oracle {
        name: "Markab",
        swiss_lon: 353.485597,
        swiss_lat: 19.406022,
    },
    Oracle {
        name: "Scheat",
        swiss_lon: 359.374173,
        swiss_lat: 31.140508,
    },
    Oracle {
        name: "Algenib",
        swiss_lon: 9.156034,
        swiss_lat: 12.599925,
    },
    Oracle {
        name: "Dubhe",
        swiss_lon: 135.197397,
        swiss_lat: 49.680317,
    },
    Oracle {
        name: "Alioth",
        swiss_lon: 158.933501,
        swiss_lat: 54.318808,
    },
    Oracle {
        name: "Mizar",
        swiss_lon: 165.700165,
        swiss_lat: 56.378929,
    },
    Oracle {
        name: "Diphda",
        swiss_lon: 2.583502,
        swiss_lat: -20.783547,
    },
];

/// Six curated stars whose spelling differs from the IAU-WGSN name (formerly
/// served from the coarse curated coordinates). These reference values come
/// from the original oracle test (tolerance 0.05 deg) and disagree with BOTH the
/// Bright Star Catalogue list and the Hipparcos-derived data by 2" to 27" (see
/// `crates/xalen-stars/tests/swiss_star_oracle.rs`).
pub const SWISS_ORACLE_ALIASED: &[Oracle] = &[
    Oracle {
        name: "Bharani 41",
        swiss_lon: 48.203266,
        swiss_lat: 10.450526,
    },
    Oracle {
        name: "Lambda Orionis",
        swiss_lon: 83.708661,
        swiss_lat: -13.369833,
    },
    Oracle {
        name: "Hyadum II",
        swiss_lon: 66.871684,
        swiss_lat: -3.969725,
    },
    Oracle {
        name: "Zuben Elgenubi",
        swiss_lon: 225.075512,
        swiss_lat: 0.333043,
    },
    Oracle {
        name: "Zuben Eschamali",
        swiss_lon: 229.364142,
        swiss_lat: 8.495271,
    },
    Oracle {
        name: "Al Jabhah",
        swiss_lon: 147.905168,
        swiss_lat: 4.865355,
    },
];

/// Signed shortest angular separation in longitude (handles the 0/360 wrap).
pub fn lon_delta(a: f64, b: f64) -> f64 {
    ((a - b + 180.0).rem_euclid(360.0)) - 180.0
}

/// Great-circle separation in arcseconds between two ecliptic places (degrees).
pub fn sep_arcsec(lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> f64 {
    let dlon = lon_delta(lon1, lon2) * 3600.0;
    let dlat = (lat1 - lat2) * 3600.0;
    let cos_lat = ((lat1 + lat2) / 2.0).to_radians().cos();
    (dlon * cos_lat).hypot(dlat)
}

/// Swiss `fixstar2` J2000 places at full double precision for 16 stars (same
/// flags as above): `(name, ecliptic longitude deg, ecliptic latitude deg)`.
/// These are the recorded values of the original Hipparcos cross-validation
/// test, not hand-typed.
pub const SWISS_J2000_FULL: &[(&str, f64, f64)] = &[
    ("Aldebaran", 69.789_183_983_576_72, -5.467_321_051_715_642),
    ("Sirius", 104.081_676_762_978_83, -39.605_303_481_139_92),
    ("Spica", 203.841_362_760_432_73, -2.054_487_223_354_432_2),
    ("Antares", 249.762_295_314_691_33, -4.569_951_746_392_631),
    ("Regulus", 149.829_144_459_570_6, 0.464_848_871_981_076_44),
    ("Vega", 285.316_389_479_313_05, 61.732_822_493_854_55),
    ("Rigel", 76.829_593_803_689_52, -31.122_759_674_056_027),
    ("Betelgeuse", 88.754_602_446_198_43, -16.026_999_842_154_822),
    ("Polaris", 88.567_581_471_252_32, 66.101_473_335_247_82),
    ("Deneb", 335.329_213_925_250_94, 59.906_139_787_280_84),
    ("Fomalhaut", 333.860_228_480_759_1, -21.135_617_059_800_44),
    ("Procyon", 115.785_560_008_261_94, -16.019_629_083_402_492),
    ("Arcturus", 204.233_651_167_495_4, 30.736_231_546_598_702),
    ("Capella", 81.857_911_200_265_93, 22.864_282_246_491_822),
    ("Pollux", 113.215_647_947_393_07, 6.684_202_172_931_602),
    // Menkar = alpha Ceti; Swiss ",alCet" place (NOT lambda Ceti).
    ("Menkar", 44.320_143_953_175_81, -12.585_573_744_354_766),
];
