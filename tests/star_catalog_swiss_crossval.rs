//! Fixed-star catalog (Hipparcos-derived) cross-validation vs Swiss Ephemeris.
//!
//! Requires the HIP catalog (`hip-catalog` feature); skipped in commercial
//! (`--no-default-features`) test runs where the non-commercial data is absent.
#![cfg(feature = "hip-catalog")]
//!
//! Reference: Swiss Ephemeris `swe_fixstar2` at J2000.0 (JD 2451545.0) with the
//! ASTROMETRIC flag set `SEFLG_J2000 | SEFLG_NONUT | SEFLG_NOABERR | SEFLG_NOGDEFL`
//! — the mean-J2000 ecliptic CATALOG place, the same quantity `catalog_generated.rs`
//! emits. (Swiss's default apparent place adds ~20" of annual aberration, which is
//! an apparent-position effect, not a catalog property.)
//!
//! Oracle reproduced by:
//!   SE_EPHE_PATH=<dir with sefstars.txt> \
//!     python3 tools/gen-star-catalog/validate_vs_swiss.py
//!
//! Verified with pyswisseph 2.10.03: across all 106 named stars, median
//! separation 0.031", max 5.965". The two stars >1" are documented multiple-star
//! Swiss-component artifacts (Rigil Kentaurus = alpha Cen, Algieba = gamma Leo).
//! Where the catalog's traditional spelling is not the name Swiss indexes, the
//! validator queries the Bayer/Flamsteed designation (e.g. Menkar = alpha Cet
//! ",alCet"; Bharani 41 = 41 Arietis ",41Ari"; Al Jabhah = eta Leonis ",etLeo").

use xalen_stars::{GENERATED_CATALOG, GENERATED_STAR_COUNT, GeneratedStar, find_generated_by_hip};

/// Great-circle separation in arcseconds between a generated star and a
/// reference ecliptic (lon, lat) in degrees.
fn sep_arcsec(star: &GeneratedStar, ref_lon: f64, ref_lat: f64) -> f64 {
    let dlon = {
        let d = (star.longitude_j2000 - ref_lon + 180.0).rem_euclid(360.0) - 180.0;
        d * 3600.0
    };
    let dlat = (star.latitude_j2000 - ref_lat) * 3600.0;
    let cos_lat = ref_lat.to_radians().cos();
    (dlon * cos_lat).hypot(dlat)
}

/// (HIP, name, Swiss J2000 ecliptic lon, lat) — values straight from the
/// pyswisseph oracle (swe.fixstar2, astrometric J2000 flags). Anti-fabrication:
/// these are NOT typed from memory; they are the recorded oracle output.
const SWISS_J2000: &[(u32, &str, f64, f64)] = &[
    (
        21421,
        "Aldebaran",
        69.789_183_983_576_72,
        -5.467_321_051_715_642,
    ),
    (
        32349,
        "Sirius",
        104.081_676_762_978_83,
        -39.605_303_481_139_92,
    ),
    (
        65474,
        "Spica",
        203.841_362_760_432_73,
        -2.054_487_223_354_432_2,
    ),
    (
        80763,
        "Antares",
        249.762_295_314_691_33,
        -4.569_951_746_392_631,
    ),
    (
        49669,
        "Regulus",
        149.829_144_459_570_6,
        0.464_848_871_981_076_44,
    ),
    (91262, "Vega", 285.316_389_479_313_05, 61.732_822_493_854_55),
    (
        24436,
        "Rigel",
        76.829_593_803_689_52,
        -31.122_759_674_056_027,
    ),
    (
        27989,
        "Betelgeuse",
        88.754_602_446_198_43,
        -16.026_999_842_154_822,
    ),
    (
        11767,
        "Polaris",
        88.567_581_471_252_32,
        66.101_473_335_247_82,
    ),
    (
        102098,
        "Deneb",
        335.329_213_925_250_94,
        59.906_139_787_280_84,
    ),
    (
        113368,
        "Fomalhaut",
        333.860_228_480_759_1,
        -21.135_617_059_800_44,
    ),
    (
        37279,
        "Procyon",
        115.785_560_008_261_94,
        -16.019_629_083_402_492,
    ),
    (
        69673,
        "Arcturus",
        204.233_651_167_495_4,
        30.736_231_546_598_702,
    ),
    (
        24608,
        "Capella",
        81.857_911_200_265_93,
        22.864_282_246_491_822,
    ),
    (
        37826,
        "Pollux",
        113.215_647_947_393_07,
        6.684_202_172_931_602,
    ),
    // Menkar = alpha Cet (HIP 14135); Swiss ",alCet" place (NOT lambda Cet).
    (
        14135,
        "Menkar",
        44.320_143_953_175_81,
        -12.585_573_744_354_766,
    ),
];

#[test]
fn generated_catalog_count_is_real_hipparcos_8870() {
    // 118,218 hip_main.dat records filtered to Vmag<=6.5 -> 8,870 stars.
    assert_eq!(GENERATED_CATALOG.len(), GENERATED_STAR_COUNT);
    assert_eq!(GENERATED_CATALOG.len(), 8870);
}

#[test]
fn named_stars_match_swiss_subarcsecond() {
    let mut worst = 0.0_f64;
    let mut worst_name = "";
    for &(hip, name, slon, slat) in SWISS_J2000 {
        let star = find_generated_by_hip(hip)
            .unwrap_or_else(|| panic!("HIP {hip} ({name}) missing from generated catalog"));
        assert_eq!(star.name, Some(name), "HIP {hip} name mismatch");
        let sep = sep_arcsec(star, slon, slat);
        if sep > worst {
            worst = sep;
            worst_name = name;
        }
        assert!(
            sep < 1.0,
            "{name} (HIP {hip}) is {sep:.3}\" from Swiss J2000 — expected sub-arcsecond"
        );
    }
    // Tightest practical bound: every anchor here is a single star, so the
    // worst should be well under half an arcsecond (Sirius ~0.24" is the max).
    assert!(
        worst < 0.5,
        "worst anchor separation {worst:.3}\" ({worst_name}) exceeds 0.5\""
    );
}

#[test]
fn breadth_target_met() {
    // The lever's breadth goal: 6,000+ stars from Hipparcos.
    assert!(
        GENERATED_CATALOG.len() >= 6000,
        "breadth target 6000+, got {}",
        GENERATED_CATALOG.len()
    );
}
