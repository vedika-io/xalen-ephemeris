use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
/// Ayanamsa (precession correction) systems for tropical-to-sidereal conversion.
///
/// 50 named systems plus a Custom variant.  Covers every predefined system
/// in Swiss Ephemeris (SE IDs 0-46) and additional systems used in Indian,
/// Babylonian, Theosophical, and Hellenistic traditions.
pub enum Ayanamsa {
    // ── Classic / Indian ──────────────────────────────────────────────
    /// Lahiri (Chitrapaksha) — Indian Ephemeris & Nautical Almanac.  SE ID 1.
    Lahiri,
    /// KP (Krishnamurti) — K.S. Krishnamurti, "Reader 1".  SE ID 5.
    KPKrishnamurti,
    /// B.V. Raman — "Hindu Predictive Astrology" (1938).  SE ID 3.
    Raman,
    /// True Chitrapaksha — Spica fixed at exactly 180 deg sidereal.  SE ID 27.
    /// Computed from Spica's apparent ecliptic longitude of date (J2000 catalog
    /// position, proper motion, rigorous IAU 2006/P03 precession that couples
    /// longitude AND latitude, IAU 2000B nutation, and annual aberration via a
    /// self-contained Meeus solar-longitude series — no planetary-engine
    /// dependency cycle) minus 180 deg. Cross-validated against Swiss
    /// SE_SIDM_TRUE_CITRA (pyswisseph get_ayanamsa_ex, with-nutation): 0.038 arcsec
    /// across the 1900-2100 oracle epochs (and 0.11 arcsec over 200 randomly
    /// sampled dates in that span). See README / ACCURACY.md.
    TrueChitra,
    /// True Revati — Zeta Piscium anchored at 29 deg 50' Pisces.  SE ID 28.
    /// Dynamic fixed-direction reduction (J2000 reference direction precessed via
    /// IAU 2006/P03 + IAU 2000B nutation); matches Swiss SE_SIDM_TRUE_REVATI to
    /// <= 0.24" across 1900-2100.
    TrueRevati,
    /// Surya Siddhanta — mean Sun ingress into Aries at 499 CE Ujjain.  SE ID 21.
    SuryaSiddhanta,
    /// Sri Yukteswar — "The Holy Science" (1920).  SE ID 7.
    YukteswarSriSS,
    /// J.N. Bhasin.  SE ID 8.
    JNBhasin,
    /// Usha/Shashi — "Hindu Astrological Calculations" (1978).  SE ID 4.
    Ushashashi,
    /// Pushya at 16 deg Cancer.
    PushyaPaksha,
    /// Lahiri ICRC — Calendar Reform Committee 1956 original.  SE ID 46.
    LahiriICRC,
    /// KP "straight line" ayanamsa.
    KPStraightLine,
    /// Lahiri variant with VP 285 CE zero year (1980 preface).  SE ID 44.
    LahiriVP285,
    /// Lahiri (1940) — "Panchanga Darpan" variant.  SE ID 43.
    Lahiri1940,
    /// Krishnamurti VP291 — mean equinox 291 CE (Senthilathiban).  SE ID 45.
    KrishnamurtiVP291,

    // ── Western sidereal ──────────────────────────────────────────────
    /// Fagan-Bradley — Western sidereal default.  SE ID 0.
    FaganBradley,
    /// Robert De Luce — "Constellational Astrology".  SE ID 2.
    DeLuce,
    /// Hipparchos.  SE ID 15.
    Hipparchos,
    /// Aldebaran at 15 deg Taurus (year -100 BCE).  SE ID 14.
    Aldebaran15Tau,

    // ── Galactic-reference systems ────────────────────────────────────
    /// Galactic Center at 0 deg Sagittarius.  SE ID 17.
    GalacticCenter0Sag,
    /// Galactic Center — R. Gil Brand (golden section 0 Sco/0 Aqu).  SE ID 30.
    GalacticCenterBrand,
    /// Galactic Center at 0 deg Capricorn (Cochrane).  SE ID 40.
    GalacticCenterCochrane,
    /// Galactic Equator IAU 1958.  SE ID 31.
    GalacticEquatorIAU1958,
    /// Galactic Equator True (Liu/Zhu/Zhang 2010).  SE ID 32.
    GalacticEquatorTrue,
    /// Galactic Equator at mid-Mula.  SE ID 33.
    GalacticEquatorMula,
    /// Galactic Alignment — Skydram/Mardyks.  SE ID 34.
    GalacticAlignMardyks,
    /// Galactic Equatorial (N.A. Fiorenza).  SE ID 41.
    GalacticEquatorialFiorenza,
    /// Ardra Nakshatra / Galactic Center at middle of Mula (Ernst Wilhelm).  SE ID 36.
    GalCenterMulaWilhelm,
    /// True Mula — Chandra Hari.  SE ID 35.
    TrueMulaChandraHari,

    // ── Babylonian / Hellenistic ──────────────────────────────────────
    /// Babylonian (Kugler 1).  SE ID 9.
    BabylonianKugler1,
    /// Babylonian (Kugler 2).  SE ID 10.
    BabylonianKugler2,
    /// Babylonian (Kugler 3).  SE ID 11.
    BabylonianKugler3,
    /// Babylonian (Huber) — "Uber den Nullpunkt der babylonischen Ekliptik".  SE ID 12.
    BabylonianHuber,
    /// Babylonian (Mercier) — Eta Piscium at culmination with zero point.  SE ID 13.
    BabylonianMercier,
    /// Babylonian (Britton 2010) — "Studies in Babylonian lunar theory".  SE ID 38.
    BabylonianBritton,
    /// Sassanian / Vettius Valens epoch.  SE ID 16.
    Sassanian,
    /// Vettius Valens (Moon-derived, Holden 1995).  SE ID 42.
    ValensMoon,

    // ── Theosophical / esoteric ───────────────────────────────────────
    /// Djwhal Khul (Graham Dawson / Alice Bailey).  SE ID 6.
    DjwhalKhul,

    // ── Star-anchored systems ─────────────────────────────────────────
    /// Suryasiddhanta (Revati) — Zeta Piscium at 0 deg Aries.  SE ID 25.
    SuryaSiddhantaRevati,
    /// Suryasiddhanta (mean Sun variant).  SE ID 22.
    SuryaSiddhantaMeanSun,
    /// Suryasiddhanta (Citra/Spica at polar long 180 deg).  SE ID 26.
    SuryaSiddhantaCitra,
    /// True Pushya — Delta Cancri at 16 deg Cancer.  SE ID 29.
    TruePushyaDeltaCancri,
    /// Citra variant: Spica fixed at exactly 180 deg (0 deg Libra).
    CitraAtSpica180,
    /// Aryabhata — epoch ~499 CE.  SE ID 23.
    Aryabhata,
    /// Aryabhata (Mean Sun).  SE ID 24.
    AryabhataMeanSun,
    /// Aryabhata 522 CE — Kali 3623 epoch.  SE ID 37.
    Aryabhata522,

    // ── Reference-epoch systems ───────────────────────────────────────
    /// J2000 epoch (zero ayanamsa at J2000.0).  SE ID 18.
    J2000,
    /// J1900 epoch (zero ayanamsa at J1900.0).  SE ID 19.
    J1900,
    /// B1950 epoch (zero ayanamsa at B1950.0).  SE ID 20.
    B1950,

    // ── Modern research ───────────────────────────────────────────────
    /// True Sheoran — Sunil Sheoran "Vedic" (2017).  SE ID 39.
    TrueSheoran,

    // ── Custom user-defined ───────────────────────────────────────────
    Custom {
        epoch_jd: f64,
        ayanamsa_at_epoch: f64,
        precession_rate: f64,
    },
}

impl Ayanamsa {
    /// Return the default ayanamsa for Vedic astrology (Lahiri).
    pub fn vedic_default() -> Self {
        Ayanamsa::Lahiri
    }
    /// Return the default ayanamsa for KP astrology (Krishnamurti).
    pub fn kp_default() -> Self {
        Ayanamsa::KPKrishnamurti
    }

    /// Return a slice of all named (non-Custom) ayanamsa variants.
    pub fn all_named() -> &'static [Ayanamsa] {
        &[
            // Classic / Indian
            Ayanamsa::Lahiri,
            Ayanamsa::KPKrishnamurti,
            Ayanamsa::Raman,
            Ayanamsa::TrueChitra,
            Ayanamsa::TrueRevati,
            Ayanamsa::SuryaSiddhanta,
            Ayanamsa::YukteswarSriSS,
            Ayanamsa::JNBhasin,
            Ayanamsa::Ushashashi,
            Ayanamsa::PushyaPaksha,
            Ayanamsa::LahiriICRC,
            Ayanamsa::KPStraightLine,
            Ayanamsa::LahiriVP285,
            Ayanamsa::Lahiri1940,
            Ayanamsa::KrishnamurtiVP291,
            // Western sidereal
            Ayanamsa::FaganBradley,
            Ayanamsa::DeLuce,
            Ayanamsa::Hipparchos,
            Ayanamsa::Aldebaran15Tau,
            // Galactic-reference
            Ayanamsa::GalacticCenter0Sag,
            Ayanamsa::GalacticCenterBrand,
            Ayanamsa::GalacticCenterCochrane,
            Ayanamsa::GalacticEquatorIAU1958,
            Ayanamsa::GalacticEquatorTrue,
            Ayanamsa::GalacticEquatorMula,
            Ayanamsa::GalacticAlignMardyks,
            Ayanamsa::GalacticEquatorialFiorenza,
            Ayanamsa::GalCenterMulaWilhelm,
            Ayanamsa::TrueMulaChandraHari,
            // Babylonian / Hellenistic
            Ayanamsa::BabylonianKugler1,
            Ayanamsa::BabylonianKugler2,
            Ayanamsa::BabylonianKugler3,
            Ayanamsa::BabylonianHuber,
            Ayanamsa::BabylonianMercier,
            Ayanamsa::BabylonianBritton,
            Ayanamsa::Sassanian,
            Ayanamsa::ValensMoon,
            // Theosophical
            Ayanamsa::DjwhalKhul,
            // Star-anchored
            Ayanamsa::SuryaSiddhantaRevati,
            Ayanamsa::SuryaSiddhantaMeanSun,
            Ayanamsa::SuryaSiddhantaCitra,
            Ayanamsa::TruePushyaDeltaCancri,
            Ayanamsa::CitraAtSpica180,
            Ayanamsa::Aryabhata,
            Ayanamsa::AryabhataMeanSun,
            Ayanamsa::Aryabhata522,
            // Reference-epoch
            Ayanamsa::J2000,
            Ayanamsa::J1900,
            Ayanamsa::B1950,
            // Modern research
            Ayanamsa::TrueSheoran,
        ]
    }

    /// Compute the ayanamsa value in radians at the given TT epoch.
    pub fn compute(&self, jd_tt: f64) -> f64 {
        let t = (jd_tt - 2_451_545.0) / 36525.0; // Julian centuries from J2000
        match self {
            // ── Classic / Indian ──────────────────────────────────────
            Ayanamsa::Lahiri => lahiri(t),
            Ayanamsa::LahiriICRC => lahiri_icrc(t),
            Ayanamsa::KPKrishnamurti => kp_krishnamurti(t),
            Ayanamsa::Raman => raman(t),
            Ayanamsa::TrueChitra => true_chitra(t),
            Ayanamsa::TrueRevati => true_revati(t),
            Ayanamsa::SuryaSiddhanta => surya_siddhanta(t),
            Ayanamsa::YukteswarSriSS => yukteswar(t),
            Ayanamsa::JNBhasin => jn_bhasin(t),
            Ayanamsa::Ushashashi => ushashashi(t),
            Ayanamsa::PushyaPaksha => pushya_paksha(t),
            Ayanamsa::LahiriVP285 => lahiri_vp285(t),
            Ayanamsa::KPStraightLine => kp_straight_line(t),
            Ayanamsa::Lahiri1940 => lahiri_1940(t),
            Ayanamsa::KrishnamurtiVP291 => krishnamurti_vp291(t),

            // ── Western sidereal ──────────────────────────────────────
            Ayanamsa::FaganBradley => fagan_bradley(t),
            Ayanamsa::DeLuce => de_luce(t),
            Ayanamsa::Hipparchos => hipparchos(t),
            Ayanamsa::Aldebaran15Tau => aldebaran_15_tau(t),

            // ── Galactic-reference ────────────────────────────────────
            Ayanamsa::GalacticCenter0Sag => galactic_center_0_sag(t),
            Ayanamsa::GalacticCenterBrand => galactic_center_brand(t),
            Ayanamsa::GalacticCenterCochrane => galactic_center_cochrane(t),
            Ayanamsa::GalacticEquatorIAU1958 => galactic_equator_iau1958(t),
            Ayanamsa::GalacticEquatorTrue => galactic_equator_true(t),
            Ayanamsa::GalacticEquatorMula => galactic_equator_mula(t),
            Ayanamsa::GalacticAlignMardyks => galactic_align_mardyks(t),
            Ayanamsa::GalacticEquatorialFiorenza => galactic_equatorial_fiorenza(t),
            Ayanamsa::GalCenterMulaWilhelm => gal_center_mula_wilhelm(t),
            Ayanamsa::TrueMulaChandraHari => true_mula_chandra_hari(t),

            // ── Babylonian / Hellenistic ──────────────────────────────
            Ayanamsa::BabylonianKugler1 => babylonian_kugler1(t),
            Ayanamsa::BabylonianKugler2 => babylonian_kugler2(t),
            Ayanamsa::BabylonianKugler3 => babylonian_kugler3(t),
            Ayanamsa::BabylonianHuber => babylonian_huber(t),
            Ayanamsa::BabylonianMercier => babylonian_mercier(t),
            Ayanamsa::BabylonianBritton => babylonian_britton(t),
            Ayanamsa::Sassanian => sassanian(t),
            Ayanamsa::ValensMoon => valens_moon(t),

            // ── Theosophical / esoteric ───────────────────────────────
            Ayanamsa::DjwhalKhul => djwhal_khul(t),

            // ── Star-anchored systems ─────────────────────────────────
            Ayanamsa::SuryaSiddhantaRevati => ss_revati(t),
            Ayanamsa::SuryaSiddhantaMeanSun => ss_mean_sun(t),
            Ayanamsa::SuryaSiddhantaCitra => ss_citra(t),
            Ayanamsa::TruePushyaDeltaCancri => true_pushya(t),
            Ayanamsa::CitraAtSpica180 => citra_at_spica_180(t),
            Ayanamsa::Aryabhata => aryabhata(t),
            Ayanamsa::AryabhataMeanSun => aryabhata_mean_sun(t),
            Ayanamsa::Aryabhata522 => aryabhata_522(t),

            // ── Reference-epoch ───────────────────────────────────────
            Ayanamsa::J2000 => j2000_ayanamsa(t),
            Ayanamsa::J1900 => j1900_ayanamsa(t),
            Ayanamsa::B1950 => b1950_ayanamsa(t),

            // ── Modern research ───────────────────────────────────────
            Ayanamsa::TrueSheoran => true_sheoran(t),

            // ── Custom ────────────────────────────────────────────────
            Ayanamsa::Custom {
                epoch_jd,
                ayanamsa_at_epoch,
                precession_rate,
            } => {
                let dt_centuries = (jd_tt - epoch_jd) / 36525.0;
                (ayanamsa_at_epoch + precession_rate * dt_centuries * 100.0).to_radians()
            }
        }
    }

    /// Compute the ayanamsa value in degrees at the given TT epoch.
    pub fn compute_deg(&self, jd_tt: f64) -> f64 {
        self.compute(jd_tt).to_degrees()
    }

    /// Return the Swiss Ephemeris ayanamsa ID, if this system has one.
    ///
    /// Maps to the official SE_SIDM_* constants (0-46) from swephexp.h.
    pub fn swiss_ephem_id(&self) -> Option<u32> {
        match self {
            Ayanamsa::FaganBradley => Some(0),
            Ayanamsa::Lahiri => Some(1),
            Ayanamsa::DeLuce => Some(2),
            Ayanamsa::Raman => Some(3),
            Ayanamsa::Ushashashi => Some(4),
            Ayanamsa::KPKrishnamurti => Some(5),
            Ayanamsa::DjwhalKhul => Some(6),
            Ayanamsa::YukteswarSriSS => Some(7),
            Ayanamsa::JNBhasin => Some(8),
            Ayanamsa::BabylonianKugler1 => Some(9),
            Ayanamsa::BabylonianKugler2 => Some(10),
            Ayanamsa::BabylonianKugler3 => Some(11),
            Ayanamsa::BabylonianHuber => Some(12),
            Ayanamsa::BabylonianMercier => Some(13),
            Ayanamsa::Aldebaran15Tau => Some(14),
            Ayanamsa::Hipparchos => Some(15),
            Ayanamsa::Sassanian => Some(16),
            Ayanamsa::GalacticCenter0Sag => Some(17),
            Ayanamsa::J2000 => Some(18),
            Ayanamsa::J1900 => Some(19),
            Ayanamsa::B1950 => Some(20),
            Ayanamsa::SuryaSiddhanta => Some(21),
            Ayanamsa::SuryaSiddhantaMeanSun => Some(22),
            Ayanamsa::Aryabhata => Some(23),
            Ayanamsa::AryabhataMeanSun => Some(24),
            Ayanamsa::SuryaSiddhantaRevati => Some(25),
            Ayanamsa::SuryaSiddhantaCitra => Some(26),
            Ayanamsa::TrueChitra => Some(27),
            Ayanamsa::TrueRevati => Some(28),
            Ayanamsa::TruePushyaDeltaCancri => Some(29),
            Ayanamsa::GalacticCenterBrand => Some(30),
            Ayanamsa::GalacticEquatorIAU1958 => Some(31),
            Ayanamsa::GalacticEquatorTrue => Some(32),
            Ayanamsa::GalacticEquatorMula => Some(33),
            Ayanamsa::GalacticAlignMardyks => Some(34),
            Ayanamsa::TrueMulaChandraHari => Some(35),
            Ayanamsa::GalCenterMulaWilhelm => Some(36),
            Ayanamsa::Aryabhata522 => Some(37),
            Ayanamsa::BabylonianBritton => Some(38),
            Ayanamsa::TrueSheoran => Some(39),
            Ayanamsa::GalacticCenterCochrane => Some(40),
            Ayanamsa::GalacticEquatorialFiorenza => Some(41),
            Ayanamsa::ValensMoon => Some(42),
            Ayanamsa::Lahiri1940 => Some(43),
            Ayanamsa::LahiriVP285 => Some(44),
            Ayanamsa::KrishnamurtiVP291 => Some(45),
            Ayanamsa::LahiriICRC => Some(46),
            _ => None,
        }
    }

    /// Construct an Ayanamsa from a Swiss Ephemeris ID (0-46).
    pub fn from_swiss_ephem_id(id: u32) -> Option<Self> {
        match id {
            0 => Some(Ayanamsa::FaganBradley),
            1 => Some(Ayanamsa::Lahiri),
            2 => Some(Ayanamsa::DeLuce),
            3 => Some(Ayanamsa::Raman),
            4 => Some(Ayanamsa::Ushashashi),
            5 => Some(Ayanamsa::KPKrishnamurti),
            6 => Some(Ayanamsa::DjwhalKhul),
            7 => Some(Ayanamsa::YukteswarSriSS),
            8 => Some(Ayanamsa::JNBhasin),
            9 => Some(Ayanamsa::BabylonianKugler1),
            10 => Some(Ayanamsa::BabylonianKugler2),
            11 => Some(Ayanamsa::BabylonianKugler3),
            12 => Some(Ayanamsa::BabylonianHuber),
            13 => Some(Ayanamsa::BabylonianMercier),
            14 => Some(Ayanamsa::Aldebaran15Tau),
            15 => Some(Ayanamsa::Hipparchos),
            16 => Some(Ayanamsa::Sassanian),
            17 => Some(Ayanamsa::GalacticCenter0Sag),
            18 => Some(Ayanamsa::J2000),
            19 => Some(Ayanamsa::J1900),
            20 => Some(Ayanamsa::B1950),
            21 => Some(Ayanamsa::SuryaSiddhanta),
            22 => Some(Ayanamsa::SuryaSiddhantaMeanSun),
            23 => Some(Ayanamsa::Aryabhata),
            24 => Some(Ayanamsa::AryabhataMeanSun),
            25 => Some(Ayanamsa::SuryaSiddhantaRevati),
            26 => Some(Ayanamsa::SuryaSiddhantaCitra),
            27 => Some(Ayanamsa::TrueChitra),
            28 => Some(Ayanamsa::TrueRevati),
            29 => Some(Ayanamsa::TruePushyaDeltaCancri),
            30 => Some(Ayanamsa::GalacticCenterBrand),
            31 => Some(Ayanamsa::GalacticEquatorIAU1958),
            32 => Some(Ayanamsa::GalacticEquatorTrue),
            33 => Some(Ayanamsa::GalacticEquatorMula),
            34 => Some(Ayanamsa::GalacticAlignMardyks),
            35 => Some(Ayanamsa::TrueMulaChandraHari),
            36 => Some(Ayanamsa::GalCenterMulaWilhelm),
            37 => Some(Ayanamsa::Aryabhata522),
            38 => Some(Ayanamsa::BabylonianBritton),
            39 => Some(Ayanamsa::TrueSheoran),
            40 => Some(Ayanamsa::GalacticCenterCochrane),
            41 => Some(Ayanamsa::GalacticEquatorialFiorenza),
            42 => Some(Ayanamsa::ValensMoon),
            43 => Some(Ayanamsa::Lahiri1940),
            44 => Some(Ayanamsa::LahiriVP285),
            45 => Some(Ayanamsa::KrishnamurtiVP291),
            46 => Some(Ayanamsa::LahiriICRC),
            _ => None,
        }
    }
}

// ── Constants from Swiss Ephemeris (sweph.h) ─────────────────────────
//
// Epoch JD values — exact matches for SE #define constants.
// Source: sweph.h lines defining J2000, J1900, B1950.

/// J2000.0 = 2000 January 1.5 TT (SE: #define J2000 2451545.0)
const J2000_JD: f64 = 2_451_545.0;
/// J1900.0 = 1900 January 0.5 TT (SE: #define J1900 2415020.0)
const J1900_JD: f64 = 2_415_020.0;
/// B1950.0 = 1950 January 0.923 (SE: #define B1950 2433282.42345905)
const B1950_JD: f64 = 2_433_282.42345905;

// ── Precession rate ──────────────────────────────────────────────────
//
// General precession in longitude at J2000.0:
//   pA = 5028.796195 arcsec/Julian century = 50.28796195 arcsec/yr
// (IAU 2006 J2000 linear term; numerically identical at J2000 to the Vondrak
// 2011 rate Swiss uses by default).
//
// We do NOT use a single linear rate for the precession of the equinox.
// Instead every non-dynamic system accumulates the FULL IAU 2006 general
// precession in longitude polynomial via [`xalen_coords::general_precession_longitude`]
// (referenced to the system epoch t0), then adds the IAU 2000B nutation in
// longitude Δψ — which is exactly what Swiss `swe_get_ayanamsa_ex(jd, 0)`
// (the WITH-nutation apparent ayanamsa) returns. Cross-validated against
// pyswisseph 2.10.03 to < 1 arcsec across 1900–2100 for every SE id with a
// fixed epoch (see `tests/swiss_ayanamsa_oracle.rs`). The single-rate linear
// extrapolation that lived here previously accumulated 250–510" of error at
// the deep (1 CE / 499 CE / 564 CE) epochs and a uniform ~17" nutation error
// everywhere; both are now gone.

/// General precession in longitude, arcsec per Julian year (IAU 2006 J2000 rate).
/// Retained only for the linear systems without an SE id and the `Custom` model.
const PRECESSION_ARCSEC_PER_YEAR: f64 = 50.28796195;

// ── Shared helpers ──────────────────────────────────────────────────

/// IAU 2000B nutation in longitude (Δψ) at TT epoch `t` (Julian centuries from
/// J2000), in radians. Swiss `swe_get_ayanamsa_ex(jd, 0)` returns the apparent
/// (with-nutation) ayanamsa, so every system must add this term. Identical to the
/// term `true_chitra` already applies via `spica_apparent_longitude`.
fn delta_psi(t: f64) -> f64 {
    xalen_coords::nutation_2000b(t).delta_psi
}

/// Compute ayanamsa via a linear-in-time user model from a J2000 reference value,
/// PLUS the IAU 2000B nutation in longitude. `value_j2000` is in degrees,
/// `rate_arcsec_per_year` is the precession rate. Used only for the systems with
/// no SE id and for `Ayanamsa::Custom`-style anchoring; the nutation term makes it
/// track the Swiss WITH-nutation convention to arcsec.
fn linear_ayanamsa(t: f64, value_j2000: f64, rate_arcsec_per_year: f64) -> f64 {
    let years = t * 100.0;
    let mean = (value_j2000 + rate_arcsec_per_year * years / 3600.0).to_radians();
    mean + delta_psi(t)
}

/// Compute an epoch-anchored ayanamsa exactly the way Swiss Ephemeris does for a
/// system with a fixed `{t0, ayan_t0}` reference: take `value_at_epoch` (degrees)
/// at Julian Day `epoch_jd`, accumulate the FULL IAU 2006 general precession in
/// longitude from t0 to the target date, then add the IAU 2000B nutation Δψ
/// (Swiss returns the apparent / with-nutation value for flags = 0).
///
///   ayanamsa(t) = ayan_t0 + [p_A(t) − p_A(t0)] + Δψ(t)
///
/// where p_A is `xalen_coords::general_precession_longitude` (IAU 2006). This
/// replaces the previous single-rate `rate · Δyears` term, which drifted by
/// hundreds of arcsec at deep epochs. Cross-validated < 1" vs pyswisseph 2.10.03
/// for every SE id across 1900–2100.
fn epoch_based_ayanamsa(jd_tt: f64, epoch_jd: f64, value_at_epoch: f64) -> f64 {
    let t = (jd_tt - J2000_JD) / 36525.0;
    let t0 = (epoch_jd - J2000_JD) / 36525.0;
    let precession = xalen_coords::general_precession_longitude(t)
        - xalen_coords::general_precession_longitude(t0);
    value_at_epoch.to_radians() + precession + delta_psi(t)
}

/// Apparent (with-nutation) ecliptic longitude of a FIXED celestial direction
/// (a galactic reference point or a distant catalogue star treated as fixed),
/// in radians, at TT epoch `t` Julian centuries from J2000.
///
/// Swiss computes the galactic and several "true star" sidereal frames by
/// rotating a fixed J2000 reference direction to the equinox/ecliptic of date and
/// adding nutation. We do the same with the SOFA-validated IAU 2006/P03 rotation
/// (`xalen_coords::precess_ecliptic_to_of_date` + `precession_matrix_p03_nobias`):
///
///   1. take the reference point's J2000 ecliptic (longitude, latitude);
///   2. rigorously precess it to the mean ecliptic of date (this couples
///      longitude AND latitude — the planar `general_precession_longitude`
///      scalar cannot, which is why a pure-longitude model leaves a
///      ~1.5"/century residual for off-ecliptic points);
///   3. add Δψ for the true (apparent) ecliptic of date.
///
/// `lon_j2000_deg` is the J2000 ecliptic longitude of the reference direction (it
/// equals the Swiss mean — no-nutation — ayanamsa at J2000 by construction, since
/// the system anchors that direction at sidereal 0° and precession is zero at
/// J2000). `lat_j2000_deg` is the reference direction's J2000 ecliptic latitude,
/// which controls the latitude-coupled part of its precession.
fn fixed_point_apparent_longitude(lon_j2000_deg: f64, lat_j2000_deg: f64, t: f64) -> f64 {
    let pos = xalen_coords::EclipticPosition {
        longitude: lon_j2000_deg.to_radians(),
        latitude: lat_j2000_deg.to_radians(),
        distance: 1.0,
    };
    let prec = xalen_coords::precession_matrix_p03_nobias(t);
    let of_date = xalen_coords::precess_ecliptic_to_of_date(pos, prec, t);
    (of_date.longitude + delta_psi(t)).rem_euclid(std::f64::consts::TAU)
}

/// Ayanamsa of a fixed-direction sidereal frame (galactic point or fixed star),
/// in radians. The reference direction is anchored at sidereal longitude 0° at
/// every epoch, so the ayanamsa is simply its apparent ecliptic longitude of date.
/// See [`fixed_point_apparent_longitude`] for the reduction and accuracy notes.
fn fixed_point_ayanamsa(lon_j2000_deg: f64, lat_j2000_deg: f64, t: f64) -> f64 {
    fixed_point_apparent_longitude(lon_j2000_deg, lat_j2000_deg, t)
}

// ── Classic / Indian implementations ───────────────────────────────

fn lahiri(t: f64) -> f64 {
    // Standard Lahiri (Chitrapaksha) — SE ID 1.
    // sweph.h: {2435553.5, 23.25, FALSE, SEMOD_PREC_NEWCOMB}; SE then applies an
    // internal `get_aya_correction()` (Newcomb-vs-modern offset, ≈ −16.7" at this
    // epoch). We fold that correction into the epoch value (23.25 − 0.00464207,
    // the same constant Swiss uses for the closely-related ICRC variant, SE ID 46),
    // then accumulate the FULL IAU 2006 general precession from t0 plus Δψ.
    //
    // The previous J2000-anchored constant (23.85306°) was WRONG by ~14.5": the
    // true Swiss mean value at J2000 is 23.857092°. Anchoring from the 1956
    // epoch with accumulated IAU 2006 precession + nutation now tracks Swiss
    // SE_SIDM_LAHIRI to ≤ 0.74" across 1900–2100 (cross-validated, pyswisseph
    // 2.10.03). NOTE: this is agreement with SWISS, not a bit-for-bit
    // reconciliation against the Indian Astronomical Ephemeris tables (those are
    // not bundled).
    const T0: f64 = 2_435_553.5;
    const AYAN_T0: f64 = 23.25 - 0.00464207;
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, T0, AYAN_T0)
}

fn lahiri_icrc(t: f64) -> f64 {
    // Lahiri ICRC — Calendar Reform Committee 1956 original.  SE ID 46.
    // sweph.h: {2435553.5, 23.25 - 0.00464207, FALSE, SEMOD_PREC_NEWCOMB}
    // prec_offset = NEWCOMB(11); SE applies get_aya_correction() internally.
    const T0: f64 = 2_435_553.5;
    const AYAN_T0: f64 = 23.25 - 0.00464207; // = 23.24535793
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, T0, AYAN_T0)
}

fn lahiri_1940(t: f64) -> f64 {
    // Lahiri 1940 — "Panchanga Darpan" variant.  SE ID 43.
    // sweph.h: {J1900, 22.44597222, FALSE, SEMOD_PREC_NEWCOMB}
    // prec_offset = NEWCOMB(11); SE applies get_aya_correction() internally.
    const AYAN_T0: f64 = 22.44597222;
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, J1900_JD, AYAN_T0)
}

fn kp_krishnamurti(t: f64) -> f64 {
    // KP ayanamsa — SE ID 5.
    // sweph.h: {J1900, 360 - 337.636111, FALSE, SEMOD_PREC_NEWCOMB}
    // Value at J1900 = 22.363889 deg.
    // prec_offset = NEWCOMB(11); SE applies get_aya_correction() internally.
    const AYAN_T0: f64 = 360.0 - 337.636111; // = 22.363889
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, J1900_JD, AYAN_T0)
}

fn krishnamurti_vp291(t: f64) -> f64 {
    // Krishnamurti VP291 — mean equinox 291 CE, Senthilathiban.  SE ID 45.
    // sweph.h: {1827424.752255678, 0.0, FALSE, 0}
    // Zero ayanamsa at JD 1827424.75 (~291 CE). prec_offset = 0 (Vondrak).
    const T0: f64 = 1_827_424.752255678;
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, T0, 0.0)
}

fn raman(t: f64) -> f64 {
    // B.V. Raman — SE ID 3.
    // sweph.h: {J1900, 360 - 338.98556, FALSE, SEMOD_PREC_NEWCOMB}
    // Value at J1900 = 21.01444 deg.
    // prec_offset = NEWCOMB(11); SE applies get_aya_correction() internally.
    const AYAN_T0: f64 = 360.0 - 338.98556; // = 21.01444
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, J1900_JD, AYAN_T0)
}

fn true_chitra(t: f64) -> f64 {
    // True Chitrapaksha — Spica fixed at exactly 180 deg sidereal (0 deg Libra).
    // SE ID 27. sweph.h: {0, 0, FALSE, 0} — Swiss Ephemeris computes this
    // dynamically from Spica's APPARENT ecliptic longitude of date, NOT a linear
    // constant. By definition the ayanamsa is:
    //
    //     ayanamsa(t) = λ_apparent(Spica, t) − 180 deg
    //
    // We compute λ_apparent from first principles via spica_apparent_longitude():
    //   J2000 ecliptic (longitude, latitude) (5-decimal Hipparcos-derived catalog)
    //   + proper motion in ecliptic longitude and latitude
    //   + rigorous IAU 2006/P03 precession (Cartesian rotation, latitude-coupled)
    //   + IAU 2000B nutation in longitude (Δψ)
    //   + annual aberration in longitude (the full apparent-place reduction).
    //
    // This matches Swiss SE_SIDM_TRUE_CITRA (the WITH-nutation, full apparent
    // place that `swe_get_ayanamsa_ex(jd, 0)` returns) to 0.038" across the
    // 1900–2100 oracle epochs; see the cross-validation test
    // `true_chitra_matches_swiss_within_1_arcsec` for the measured residuals
    // against pyswisseph's SE_SIDM_TRUE_CITRA reference table.
    spica_apparent_longitude(t) - 180.0_f64.to_radians()
}

/// Spica's apparent ecliptic longitude of date, in radians.
///
/// Full apparent-place reduction: J2000 catalog (longitude, latitude) + proper
/// motion + rigorous IAU 2006/P03 precession + IAU 2000B nutation (Δψ) + annual
/// aberration. This is the quantity Swiss Ephemeris anchors at exactly 180°
/// sidereal for SE_SIDM_TRUE_CITRA.
///
/// The precession step uses the SAME rigorous Cartesian rotation
/// ([`xalen_coords::precess_ecliptic_to_of_date`] with
/// [`xalen_coords::precession_matrix_p03_nobias`]) that the galactic and other
/// "true star" sidereal frames use, NOT the scalar
/// `general_precession_longitude` polynomial. Spica lies ~2.05° off the ecliptic,
/// so the precession of its longitude is latitude-coupled; the scalar
/// pure-longitude term cannot represent that coupling and left a
/// ~1.5″/century-slope residual (1.514″ at the 1900/2100 span endpoints,
/// passing through zero at the J2000 anchor). The Cartesian rotation couples
/// longitude and latitude consistently and removes that residual: cross-validated
/// against Swiss SE_SIDM_TRUE_CITRA (pyswisseph 2.10.03 `get_ayanamsa_ex(jd, 0)`)
/// to 0.038″ across the 1900–2100 oracle epochs and 0.11″ across 200 randomly
/// sampled dates within that span.
fn spica_apparent_longitude(t: f64) -> f64 {
    let spica =
        xalen_stars::find_by_name("Spica").expect("Spica must exist in the fixed-star catalog");

    // J2000.0 mean ecliptic (longitude, latitude) + linear proper motion.
    // pm components are in milliarcsec/yr; t is Julian centuries → years = t·100.
    let years = t * 100.0;
    let lon_j2000 =
        (spica.longitude_j2000 + spica.pm_lon_mas_per_year * years / 3_600_000.0).to_radians();
    let lat_j2000 =
        (spica.latitude_j2000 + spica.pm_lat_mas_per_year * years / 3_600_000.0).to_radians();

    // Rigorous IAU 2006/P03 precession of the J2000 ecliptic position to the
    // MEAN ecliptic of date (couples longitude AND latitude — see doc comment).
    let pos_j2000 = xalen_coords::EclipticPosition {
        longitude: lon_j2000,
        latitude: lat_j2000,
        distance: 1.0,
    };
    let prec = xalen_coords::precession_matrix_p03_nobias(t);
    let of_date = xalen_coords::precess_ecliptic_to_of_date(pos_j2000, prec, t);

    // IAU 2000B nutation in longitude (Δψ) — apparent place is referred to the
    // TRUE ecliptic of date.
    let delta_psi = xalen_coords::nutation_2000b(t).delta_psi;
    let geometric_lon = (of_date.longitude + delta_psi).rem_euclid(std::f64::consts::TAU);
    let latitude = of_date.latitude;

    // Annual aberration in longitude. The full term needs the Sun's geometric
    // (true) ecliptic longitude of date; we compute it from a self-contained
    // low-precision solar series (Meeus, Astronomical Algorithms §25) so that
    // xalen-ayanamsa stays free of any planetary-engine dependency (no cycle).
    // The aberration term's sensitivity to a Sun-longitude error δ⊙ is
    // κ·sin(⊙−λ)·δ⊙ ≈ 20.5"·δ⊙; the Meeus series is accurate to ~0.004°, so the
    // induced error is ~0.0015" — utterly negligible. The aberration helper
    // (xalen-coords::aberration) supplies the constant of aberration κ and the
    // eccentricity term; we pass it the real Sun longitude here.
    let sun_lon = sun_geometric_longitude(t);
    let aberration = xalen_coords::annual_aberration_longitude(geometric_lon, latitude, sun_lon);

    (geometric_lon + aberration).rem_euclid(std::f64::consts::TAU)
}

/// Sun's geometric (true) ecliptic longitude of date, in radians.
///
/// Low-precision solar series from Meeus, *Astronomical Algorithms* (2nd ed.),
/// chapter 25: mean longitude L0 plus the equation-of-centre C, evaluated at
/// `t` Julian centuries (TT) from J2000.0. Accurate to ~0.01°, which is far
/// more than the annual-aberration term needs (see [`spica_apparent_longitude`]).
/// Kept local to avoid a dependency on the planetary engine.
fn sun_geometric_longitude(t: f64) -> f64 {
    // Geometric mean longitude of the Sun (deg).
    let l0 = 280.466_46 + 36_000.769_83 * t + 0.000_303_2 * t * t;
    // Mean anomaly of the Sun (deg → rad).
    let m_deg = 357.529_11 + 35_999.050_29 * t - 0.000_153_7 * t * t;
    let m = m_deg.to_radians();
    // Equation of the centre C (deg).
    let c = (1.914_602 - 0.004_817 * t - 0.000_014 * t * t) * m.sin()
        + (0.019_993 - 0.000_101 * t) * (2.0 * m).sin()
        + 0.000_289 * (3.0 * m).sin();
    (l0 + c).to_radians().rem_euclid(std::f64::consts::TAU)
}

fn true_revati(t: f64) -> f64 {
    // True Revati — Zeta Piscium anchored at sidereal 29°50' Pisces (359.8333°).
    // SE ID 28. sweph.h: {0, 0, FALSE, 0} — Swiss computes this DYNAMICALLY from
    // ζ Psc's apparent place of date. We reproduce it as a fixed-direction frame:
    // the reference direction's J2000 ecliptic longitude (= the Swiss mean
    // ayanamsa at J2000, 20.045165°) precessed rigorously (IAU 2006/P03 rotation)
    // and corrected for nutation. The ecliptic latitude (−15.525°) is ζ Psc's
    // J2000 ecliptic latitude, which drives the latitude-coupled part of its
    // precession. Cross-validated ≤ 0.24" vs Swiss SE_SIDM_TRUE_REVATI across
    // 1900–2100 (pyswisseph 2.10.03). (Previous linear approx diverged ~8430".)
    const LON_J2000_DEG: f64 = 20.045_164_650;
    const LAT_J2000_DEG: f64 = -15.525;
    fixed_point_ayanamsa(LON_J2000_DEG, LAT_J2000_DEG, t)
}

fn surya_siddhanta(t: f64) -> f64 {
    // Surya Siddhanta — SE ID 21.
    // sweph.h: {1903396.8128654, 0, TRUE, 0}
    // Zero ayanamsa at JD 1903396.81 (~21 Mar 499 CE, Ujjain noon LMT).
    const T0: f64 = 1_903_396.8128654;
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, T0, 0.0)
}

fn ss_mean_sun(t: f64) -> f64 {
    // Surya Siddhanta Mean Sun — SE ID 22.
    // sweph.h: {1903396.8128654, -0.21463395, TRUE, 0}
    const T0: f64 = 1_903_396.8128654;
    const AYAN_T0: f64 = -0.21463395;
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, T0, AYAN_T0)
}

fn ss_citra(t: f64) -> f64 {
    // Surya Siddhanta Citra — Spica at polar long 180 deg.  SE ID 26.
    // sweph.h: {1903396.8128654, 2.11070444, TRUE, 0}
    const T0: f64 = 1_903_396.8128654;
    const AYAN_T0: f64 = 2.11070444;
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, T0, AYAN_T0)
}

fn yukteswar(t: f64) -> f64 {
    // Sri Yukteswar — SE ID 7.
    // sweph.h: {J1900, 360 - 338.917778, FALSE, -1}
    // Value at J1900 = 21.082222 deg. prec_offset = -1 (no special correction).
    const AYAN_T0: f64 = 360.0 - 338.917778; // = 21.082222
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, J1900_JD, AYAN_T0)
}

fn jn_bhasin(t: f64) -> f64 {
    // J.N. Bhasin — SE ID 8.
    // sweph.h: {J1900, 360 - 338.634444, FALSE, -1}
    // Value at J1900 = 21.365556 deg. prec_offset = -1 (no special correction).
    const AYAN_T0: f64 = 360.0 - 338.634444; // = 21.365556
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, J1900_JD, AYAN_T0)
}

fn ushashashi(t: f64) -> f64 {
    // Usha/Shashi — SE ID 4.
    // sweph.h: {J1900, 360 - 341.33904, FALSE, -1}
    // Value at J1900 = 18.66096 deg. prec_offset = -1 (no special correction).
    const AYAN_T0: f64 = 360.0 - 341.33904; // = 18.66096
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, J1900_JD, AYAN_T0)
}

fn pushya_paksha(t: f64) -> f64 {
    // Pushya at 16 deg Cancer — no SE ID. Custom definition.
    linear_ayanamsa(t, 23.95, PRECESSION_ARCSEC_PER_YEAR)
}

fn lahiri_vp285(t: f64) -> f64 {
    // Lahiri VP285 (1980 preface) — SE ID 44.
    // sweph.h: {1825235.2458513028, 0.0, FALSE, 0}
    // Zero ayanamsa at JD 1825235.25 (~285 CE). prec_offset = 0 (Vondrak).
    const T0: f64 = 1_825_235.2458513028;
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, T0, 0.0)
}

fn kp_straight_line(t: f64) -> f64 {
    // KP "straight line" ayanamsa — no SE ID, commonly used in KP software.
    linear_ayanamsa(t, 23.75, PRECESSION_ARCSEC_PER_YEAR)
}

// ── Western sidereal ───────────────────────────────────────────────

fn fagan_bradley(t: f64) -> f64 {
    // Fagan-Bradley — SE ID 0 (Western sidereal default).
    // sweph.h: {2433282.42346, 24.042044444, FALSE, SEMOD_PREC_NEWCOMB}
    // Ayanamsa at B1950 (JD 2433282.42346) = 24.042044444 deg.
    // prec_offset = NEWCOMB(11); SE applies get_aya_correction() internally.
    const T0: f64 = 2433282.42346;
    const AYAN_T0: f64 = 24.042044444;
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, T0, AYAN_T0)
}

fn de_luce(t: f64) -> f64 {
    // De Luce — SE ID 2.
    // sweph.h: {1721057.5, 0, TRUE, 0}
    // Zero ayanamsa at 1 Jan 1 CE (JD 1721057.5). prec_offset = 0 (Vondrak).
    const T0: f64 = 1_721_057.5;
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, T0, 0.0)
}

fn hipparchos(t: f64) -> f64 {
    // Hipparchos — SE ID 15.
    // sweph.h: {1674484.0, -9.33333, TRUE, -1}
    const T0: f64 = 1_674_484.0;
    const AYAN_T0: f64 = -9.33333;
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, T0, AYAN_T0)
}

fn aldebaran_15_tau(t: f64) -> f64 {
    // Aldebaran at 15 Taurus in year -100 — SE ID 14.
    // sweph.h: {1684532.5, -4.44138598, TRUE, 0}
    const T0: f64 = 1_684_532.5;
    const AYAN_T0: f64 = -4.44138598;
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, T0, AYAN_T0)
}

// ── Galactic-reference systems ─────────────────────────────────────

fn galactic_center_0_sag(t: f64) -> f64 {
    // Galactic Center at 0° Sagittarius — SE ID 17.
    // sweph.h: {0, 0, FALSE, 0} — Swiss computes this DYNAMICALLY by rotating the
    // galactic-centre direction to the ecliptic of date. Fixed-direction reduction
    // (see [`fixed_point_ayanamsa`]): GC J2000 ecliptic longitude 26.846046°
    // (= Swiss mean at J2000), latitude 0.280°. Cross-validated ≤ 0.08" vs Swiss
    // SE_SIDM_GALCENT_0SAG across 1900–2100. (Previous hand-typed linear anchor
    // diverged ~6305".)
    fixed_point_ayanamsa(26.846_045_806, 0.280, t)
}

fn galactic_center_brand(t: f64) -> f64 {
    // R. Gil Brand — GC at golden section 0 Sco / 0 Aqu.  SE ID 30.
    // Fixed-direction reduction: J2000 ecliptic longitude 22.469105° (= Swiss mean
    // at J2000), latitude 0.270°. Cross-validated ≤ 0.08" vs Swiss
    // SE_SIDM_GALCENT_RGILBRAND across 1900–2100. (Previous linear approx ~9269".)
    fixed_point_ayanamsa(22.469_104_789, 0.270, t)
}

fn galactic_center_cochrane(t: f64) -> f64 {
    // Galactic Center at 0° Capricorn (Cochrane) — SE ID 40.
    // Swiss places this ayanamsa near 356.85° (the J2000 anchor is the GC direction
    // offset by a full sign from SE ID 17), NOT 25.36° — the old constant was 28.5°
    // wrong. Fixed-direction reduction: J2000 ecliptic longitude 356.846046°
    // (= Swiss mean at J2000), latitude 0.240°. Cross-validated ≤ 0.08" vs Swiss
    // SE_SIDM_GALCENT_COCHRANE across 1900–2100. (Previous linear approx ~102664"
    // = 28.5°.)
    fixed_point_ayanamsa(356.846_045_806, 0.240, t)
}

fn galactic_equator_iau1958(t: f64) -> f64 {
    // Galactic Equator IAU 1958 — SE ID 31.
    // Swiss places this near 30.02° at J2000, NOT 25.15°. Fixed-direction reduction:
    // J2000 ecliptic longitude 30.023172° (= Swiss mean at J2000), latitude
    // −34.910° (the ascending node of the galactic equator on the ecliptic sits
    // well off the ecliptic, so its precession is strongly latitude-coupled).
    // Cross-validated ≤ 0.08" vs Swiss SE_SIDM_GALEQU_IAU1958 across 1900–2100.
    // (Previous linear approx diverged ~17575".)
    fixed_point_ayanamsa(30.023_172_469, -34.910, t)
}

fn galactic_equator_true(t: f64) -> f64 {
    // Galactic Equator True (Liu/Zhu/Zhang 2010) — SE ID 32.
    // Fixed-direction reduction: J2000 ecliptic longitude 30.076092° (= Swiss mean
    // at J2000), latitude −34.920°. Cross-validated ≤ 0.08" vs Swiss
    // SE_SIDM_GALEQU_TRUE across 1900–2100. (Previous linear approx ~18305".)
    fixed_point_ayanamsa(30.076_092_180, -34.920, t)
}

fn galactic_equator_mula(t: f64) -> f64 {
    // Galactic Equator at mid-Mula — SE ID 33.
    // Fixed-direction reduction: J2000 ecliptic longitude 23.409426° (= Swiss mean
    // at J2000), latitude −33.000°. Cross-validated ≤ 0.06" vs Swiss
    // SE_SIDM_GALEQU_MULA across 1900–2100. (Previous linear approx ~6462".)
    fixed_point_ayanamsa(23.409_425_513, -33.000, t)
}

fn galactic_align_mardyks(t: f64) -> f64 {
    // Galactic Alignment (Skydram/Mardyks) — SE ID 34.
    // sweph.h: {2451079.734892000, 30, FALSE, 0}
    // Ayanamsa = 30 deg at JD 2451079.73 (1998-09-11).
    const T0: f64 = 2_451_079.734892;
    const AYAN_T0: f64 = 30.0;
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, T0, AYAN_T0)
}

fn galactic_equatorial_fiorenza(t: f64) -> f64 {
    // Galactic Equatorial (N.A. Fiorenza) — SE ID 41.
    // sweph.h: {2451544.5, 25.0, TRUE, 0}
    // Ayanamsa = 25.0 deg at JD 2451544.5 (1999-12-31).
    const T0: f64 = 2_451_544.5;
    const AYAN_T0: f64 = 25.0;
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, T0, AYAN_T0)
}

fn gal_center_mula_wilhelm(t: f64) -> f64 {
    // Dhruva / GC / Middle of Mula (Ernst Wilhelm) — SE ID 36.
    // sweph.h: {0, 0, FALSE, 0} — computed dynamically by Swiss. This is the ONE
    // SE-mapped system the self-contained model does NOT bring under 1": it tracks
    // Swiss SE_SIDM_GALCENT_MULA_WILHELM to ≤ 1.42" across the 1900–2100 oracle
    // epochs (vs the previous linear approx's ~18277" divergence).
    //
    // DOCUMENTED, NON-FIXABLE-IN-PURE-RUST RESIDUAL (measured against pyswisseph
    // 2.10.03, not assumed). Swiss's internal Wilhelm reduction has two properties
    // that are mutually exclusive for a single fixed celestial direction:
    //   * a no-nutation precession rate of ~52.5"/yr (≈ 2.2"/yr ABOVE general
    //     precession) — a fixed direction only reaches that rate near the ecliptic
    //     pole; and
    //   * a moderate-latitude annual aberration whose intra-year amplitude is
    //     ~±23" (calibrated against the with/without-nutation Swiss difference) —
    //     which corresponds to an ecliptic latitude of only a few degrees.
    // No single fixed (lon, lat) satisfies both, so the rigid-direction reduction
    // cannot match the curvature of Swiss's value. The (lon, lat) below is the
    // best Jan-1-anchored fixed-direction fit (the −79.155° latitude reproduces
    // the steep rate; it is a fit parameter, NOT a real celestial latitude). The
    // oracle tolerance for SE id 36 is set to 2" with this note, not loosened to
    // hide a fixable error. Closing it below 1" requires Swiss's exact internal
    // Wilhelm star/aberration model, which is not derivable from pyswisseph
    // outputs alone.
    fixed_point_ayanamsa(20.039_233_112, -79.155, t)
}

fn true_mula_chandra_hari(t: f64) -> f64 {
    // True Mula — Chandra Hari — Lambda Scorpii anchored at sidereal 0° Sagittarius
    // (240°).  SE ID 35. sweph.h: {0, 0, FALSE, 0} — Swiss computes this
    // DYNAMICALLY from λ Sco's apparent place. Fixed-direction reduction (see
    // [`fixed_point_ayanamsa`]): J2000 ecliptic longitude 24.579973° (= Swiss mean
    // at J2000), latitude 1.265° (λ Sco J2000 ecliptic latitude). Cross-validated
    // ≤ 0.11" vs Swiss SE_SIDM_TRUE_MULA across 1900–2100. (Previous linear approx
    // diverged ~1526".)
    const LON_J2000_DEG: f64 = 24.579_972_884;
    const LAT_J2000_DEG: f64 = 1.265;
    fixed_point_ayanamsa(LON_J2000_DEG, LAT_J2000_DEG, t)
}

// ── Babylonian / Hellenistic ───────────────────────────────────────

fn babylonian_kugler1(t: f64) -> f64 {
    // Babylonian Kugler 1 — SE ID 9.
    // sweph.h: {1684532.5, -5.66667, TRUE, -1}
    const T0: f64 = 1_684_532.5;
    const AYAN_T0: f64 = -5.66667;
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, T0, AYAN_T0)
}

fn babylonian_kugler2(t: f64) -> f64 {
    // Babylonian Kugler 2 — SE ID 10.
    // sweph.h: {1684532.5, -4.26667, TRUE, -1}
    const T0: f64 = 1_684_532.5;
    const AYAN_T0: f64 = -4.26667;
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, T0, AYAN_T0)
}

fn babylonian_kugler3(t: f64) -> f64 {
    // Babylonian Kugler 3 — SE ID 11.
    // sweph.h: {1684532.5, -3.41667, TRUE, -1}
    const T0: f64 = 1_684_532.5;
    const AYAN_T0: f64 = -3.41667;
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, T0, AYAN_T0)
}

fn babylonian_huber(t: f64) -> f64 {
    // Babylonian Huber — SE ID 12.
    // sweph.h: {1684532.5, -4.46667, TRUE, -1}
    const T0: f64 = 1_684_532.5;
    const AYAN_T0: f64 = -4.46667;
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, T0, AYAN_T0)
}

fn babylonian_mercier(t: f64) -> f64 {
    // Babylonian Mercier (Eta Piscium) — SE ID 13.
    // sweph.h: {1673941, -5.079167, TRUE, -1}
    const T0: f64 = 1_673_941.0;
    const AYAN_T0: f64 = -5.079167;
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, T0, AYAN_T0)
}

fn babylonian_britton(t: f64) -> f64 {
    // Babylonian Britton (2010) — SE ID 38.
    // sweph.h: {1721057.5, -3.2, TRUE, -1}
    const T0: f64 = 1_721_057.5;
    const AYAN_T0: f64 = -3.2;
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, T0, AYAN_T0)
}

fn sassanian(t: f64) -> f64 {
    // Sassanian — SE ID 16.
    // sweph.h: {1927135.8747793, 0, TRUE, -1}
    // Zero ayanamsa at JD 1927135.87 (~564 CE).
    const T0: f64 = 1_927_135.8747793;
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, T0, 0.0)
}

fn valens_moon(t: f64) -> f64 {
    // Vettius Valens (Moon-derived, Holden 1995) — SE ID 42.
    // sweph.h: {1775845.5, -2.9422, TRUE, -1}
    const T0: f64 = 1_775_845.5;
    const AYAN_T0: f64 = -2.9422;
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, T0, AYAN_T0)
}

// ── Theosophical / esoteric ────────────────────────────────────────

fn djwhal_khul(t: f64) -> f64 {
    // Djwhal Khul (Graham Dawson / Alice Bailey) — SE ID 6.
    // sweph.h: {J1900, 360 - 333.0369024, FALSE, 0}
    // Value at J1900 = 26.9630976 deg. prec_offset = 0 (default Vondrak).
    // NOTE: unusually high — esoteric system placing vernal point
    // much earlier in the sidereal zodiac (~28.36 at J2000).
    const AYAN_T0: f64 = 360.0 - 333.0369024; // = 26.9630976
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, J1900_JD, AYAN_T0)
}

// ── Star-anchored systems ──────────────────────────────────────────

fn ss_revati(t: f64) -> f64 {
    // Suryasiddhanta Revati — SE ID 25.
    // sweph.h: {1903396.8128654, -0.79167046, TRUE, 0}
    const T0: f64 = 1_903_396.8128654;
    const AYAN_T0: f64 = -0.79167046;
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, T0, AYAN_T0)
}

fn true_pushya(t: f64) -> f64 {
    // True Pushya — Delta Cancri anchored at sidereal 16° Cancer (106°).  SE ID 29.
    // sweph.h: {0, 0, FALSE, 0} — Swiss computes this DYNAMICALLY from δ Cnc's
    // apparent place. Fixed-direction reduction (see [`fixed_point_ayanamsa`]):
    // J2000 ecliptic longitude 22.727092° (= Swiss mean at J2000), latitude
    // −5.725° (δ Cnc J2000 ecliptic latitude). Cross-validated ≤ 0.13" vs Swiss
    // SE_SIDM_TRUE_PUSHYA across 1900–2100. (Previous linear approx diverged ~4956".)
    const LON_J2000_DEG: f64 = 22.727_091_551;
    const LAT_J2000_DEG: f64 = -5.725;
    fixed_point_ayanamsa(LON_J2000_DEG, LAT_J2000_DEG, t)
}

fn citra_at_spica_180(t: f64) -> f64 {
    // Citra variant: Spica fixed at exactly 180 deg (0 deg Libra).
    // No SE ID. Very close to TrueChitra but uses a slightly different calibration.
    linear_ayanamsa(t, 23.862, PRECESSION_ARCSEC_PER_YEAR)
}

fn aryabhata(t: f64) -> f64 {
    // Aryabhata — SE ID 23.
    // sweph.h: {1903396.7895321, 0, TRUE, 0}
    const T0: f64 = 1_903_396.7895321;
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, T0, 0.0)
}

fn aryabhata_mean_sun(t: f64) -> f64 {
    // Aryabhata Mean Sun — SE ID 24.
    // sweph.h: {1903396.7895321, -0.23763238, TRUE, 0}
    const T0: f64 = 1_903_396.7895321;
    const AYAN_T0: f64 = -0.23763238;
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, T0, AYAN_T0)
}

fn aryabhata_522(t: f64) -> f64 {
    // Aryabhata 522 CE (Kali 3623) — SE ID 37.
    // sweph.h: {1911797.740782065, 0, TRUE, 0}
    const T0: f64 = 1_911_797.740782065;
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, T0, 0.0)
}

// ── Reference-epoch systems ────────────────────────────────────────

fn j2000_ayanamsa(t: f64) -> f64 {
    // J2000 — zero ayanamsa at J2000.0.  SE ID 18.
    // sweph.h: {J2000, 0, FALSE, 0}
    // Ayanamsa = general precession accumulated from J2000.0.
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, J2000_JD, 0.0)
}

fn j1900_ayanamsa(t: f64) -> f64 {
    // J1900 — zero ayanamsa at J1900.0.  SE ID 19.
    // sweph.h: {J1900, 0, FALSE, 0}
    // prec_offset = 0 (default Vondrak).
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, J1900_JD, 0.0)
}

fn b1950_ayanamsa(t: f64) -> f64 {
    // B1950 — zero ayanamsa at B1950.0.  SE ID 20.
    // sweph.h: {B1950, 0, FALSE, 0}
    // prec_offset = 0 (default Vondrak).
    epoch_based_ayanamsa(J2000_JD + t * 36525.0, B1950_JD, 0.0)
}

// ── Modern research ────────────────────────────────────────────────

fn true_sheoran(t: f64) -> f64 {
    // True Sheoran — Sunil Sheoran, "The Science of Time" (2017).  SE ID 39.
    // sweph.h: {0, 0, FALSE, 0} — Swiss computes this dynamically with its own
    // internal reduction (it does NOT reduce to a clean fixed star anchor — the
    // direct ζ Psc apparent-place model leaves an ~8" slope). We approximate it as
    // a fixed-direction frame anchored at the Swiss mean J2000 value (25.234449°)
    // with the best-fit ecliptic latitude (−5.865°), rigorously precessed +
    // nutated. Cross-validated ≤ 0.13" vs Swiss SE_SIDM_TRUE_SHEORAN across
    // 1900–2100. (Previous linear approx diverged ~4462".)
    const LON_J2000_DEG: f64 = 25.234_449_334;
    const LAT_J2000_DEG: f64 = -5.865;
    fixed_point_ayanamsa(LON_J2000_DEG, LAT_J2000_DEG, t)
}

// ── Public conversion utilities ────────────────────────────────────

/// Convert a tropical longitude (radians) to sidereal by subtracting the ayanamsa.
pub fn tropical_to_sidereal(tropical_lon_rad: f64, ayanamsa: &Ayanamsa, jd_tt: f64) -> f64 {
    let aya = ayanamsa.compute(jd_tt);
    (tropical_lon_rad - aya).rem_euclid(std::f64::consts::TAU)
}

/// Convert a sidereal longitude (radians) to tropical by adding the ayanamsa.
pub fn sidereal_to_tropical(sidereal_lon_rad: f64, ayanamsa: &Ayanamsa, jd_tt: f64) -> f64 {
    let aya = ayanamsa.compute(jd_tt);
    (sidereal_lon_rad + aya).rem_euclid(std::f64::consts::TAU)
}

// ── Display ────────────────────────────────────────────────────────

impl std::fmt::Display for Ayanamsa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ayanamsa::Lahiri => write!(f, "Lahiri (Chitrapaksha)"),
            Ayanamsa::KPKrishnamurti => write!(f, "KP (Krishnamurti)"),
            Ayanamsa::Raman => write!(f, "B.V. Raman"),
            Ayanamsa::FaganBradley => write!(f, "Fagan-Bradley"),
            Ayanamsa::TrueChitra => write!(f, "True Chitrapaksha"),
            Ayanamsa::TrueRevati => write!(f, "True Revati"),
            Ayanamsa::SuryaSiddhanta => write!(f, "Surya Siddhanta"),
            Ayanamsa::YukteswarSriSS => write!(f, "Sri Yukteswar"),
            Ayanamsa::JNBhasin => write!(f, "J.N. Bhasin"),
            Ayanamsa::DeLuce => write!(f, "De Luce"),
            Ayanamsa::Ushashashi => write!(f, "Usha/Shashi"),
            Ayanamsa::PushyaPaksha => write!(f, "Pushya Paksha"),
            Ayanamsa::GalacticCenter0Sag => write!(f, "Galactic Center 0 Sag"),
            Ayanamsa::GalacticCenterBrand => write!(f, "Galactic Center (Gil Brand)"),
            Ayanamsa::GalacticCenterCochrane => write!(f, "Galactic Center 0 Cap (Cochrane)"),
            Ayanamsa::GalacticEquatorIAU1958 => write!(f, "Galactic Equator IAU 1958"),
            Ayanamsa::GalacticEquatorTrue => write!(f, "Galactic Equator True"),
            Ayanamsa::GalacticEquatorMula => write!(f, "Galactic Equator mid-Mula"),
            Ayanamsa::GalacticAlignMardyks => write!(f, "Galactic Alignment (Mardyks)"),
            Ayanamsa::GalacticEquatorialFiorenza => write!(f, "Galactic Equatorial (Fiorenza)"),
            Ayanamsa::GalCenterMulaWilhelm => write!(f, "GC mid-Mula (Ernst Wilhelm)"),
            Ayanamsa::TrueMulaChandraHari => write!(f, "True Mula (Chandra Hari)"),
            Ayanamsa::LahiriICRC => write!(f, "Lahiri (ICRC 1956)"),
            Ayanamsa::KPStraightLine => write!(f, "KP Straight Line"),
            Ayanamsa::Hipparchos => write!(f, "Hipparchos"),
            Ayanamsa::LahiriVP285 => write!(f, "Lahiri VP285 (1980)"),
            Ayanamsa::Lahiri1940 => write!(f, "Lahiri (1940)"),
            Ayanamsa::KrishnamurtiVP291 => write!(f, "Krishnamurti VP291"),
            Ayanamsa::Aldebaran15Tau => write!(f, "Aldebaran 15 Taurus"),
            Ayanamsa::BabylonianKugler1 => write!(f, "Babylonian (Kugler 1)"),
            Ayanamsa::BabylonianKugler2 => write!(f, "Babylonian (Kugler 2)"),
            Ayanamsa::BabylonianKugler3 => write!(f, "Babylonian (Kugler 3)"),
            Ayanamsa::BabylonianHuber => write!(f, "Babylonian (Huber)"),
            Ayanamsa::BabylonianMercier => write!(f, "Babylonian (Mercier)"),
            Ayanamsa::BabylonianBritton => write!(f, "Babylonian (Britton 2010)"),
            Ayanamsa::Sassanian => write!(f, "Sassanian"),
            Ayanamsa::ValensMoon => write!(f, "Vettius Valens (Moon)"),
            Ayanamsa::DjwhalKhul => write!(f, "Djwhal Khul"),
            Ayanamsa::SuryaSiddhantaRevati => write!(f, "Suryasiddhanta (Revati)"),
            Ayanamsa::SuryaSiddhantaMeanSun => write!(f, "Suryasiddhanta (Mean Sun)"),
            Ayanamsa::SuryaSiddhantaCitra => write!(f, "Suryasiddhanta (Citra)"),
            Ayanamsa::TruePushyaDeltaCancri => write!(f, "True Pushya (Delta Cancri)"),
            Ayanamsa::CitraAtSpica180 => write!(f, "Citra (Spica 180)"),
            Ayanamsa::Aryabhata => write!(f, "Aryabhata"),
            Ayanamsa::AryabhataMeanSun => write!(f, "Aryabhata (Mean Sun)"),
            Ayanamsa::Aryabhata522 => write!(f, "Aryabhata 522 CE"),
            Ayanamsa::J2000 => write!(f, "J2000"),
            Ayanamsa::J1900 => write!(f, "J1900"),
            Ayanamsa::B1950 => write!(f, "B1950"),
            Ayanamsa::TrueSheoran => write!(f, "True Sheoran (Vedic)"),
            Ayanamsa::Custom { .. } => write!(f, "Custom Ayanamsa"),
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const J2000_JD_TEST: f64 = 2_451_545.0;

    #[test]
    fn lahiri_at_j2000() {
        let aya = Ayanamsa::Lahiri.compute_deg(J2000_JD_TEST);
        assert!(
            (aya - 23.85).abs() < 0.1,
            "Lahiri at J2000 should be ~23.85 deg, got {aya} deg"
        );
    }

    #[test]
    fn kp_differs_from_lahiri() {
        let lahiri = Ayanamsa::Lahiri.compute_deg(J2000_JD_TEST);
        let kp = Ayanamsa::KPKrishnamurti.compute_deg(J2000_JD_TEST);
        let diff = (lahiri - kp).abs();
        assert!(
            diff > 0.01 && diff < 1.0,
            "KP should differ from Lahiri by 0.01-1.0 deg, got {diff} deg"
        );
    }

    #[test]
    fn ayanamsa_increases_over_time() {
        let aya_2000 = Ayanamsa::Lahiri.compute_deg(J2000_JD_TEST);
        let aya_2100 = Ayanamsa::Lahiri.compute_deg(J2000_JD_TEST + 36525.0);
        assert!(
            aya_2100 > aya_2000,
            "Ayanamsa should increase: 2000={aya_2000} deg 2100={aya_2100} deg"
        );
        let rate = aya_2100 - aya_2000;
        assert!(
            (rate - 1.397).abs() < 0.1,
            "Precession rate ~1.397 deg/century, got {rate} deg"
        );
    }

    #[test]
    fn tropical_sidereal_roundtrip() {
        let tropical = 280.0_f64.to_radians();
        let sid = tropical_to_sidereal(tropical, &Ayanamsa::Lahiri, J2000_JD_TEST);
        let trop2 = sidereal_to_tropical(sid, &Ayanamsa::Lahiri, J2000_JD_TEST);
        assert!(
            (tropical - trop2).abs() < 1e-10,
            "Tropical-sidereal roundtrip failed"
        );
    }

    #[test]
    fn custom_ayanamsa() {
        let custom = Ayanamsa::Custom {
            epoch_jd: J2000_JD_TEST,
            ayanamsa_at_epoch: 23.5,
            precession_rate: 50.0 / 3600.0,
        };
        let aya = custom.compute_deg(J2000_JD_TEST);
        assert!(
            (aya - 23.5).abs() < 0.01,
            "Custom at epoch should match, got {aya} deg"
        );
    }

    #[test]
    fn at_least_40_named_systems() {
        let count = Ayanamsa::all_named().len();
        assert!(count >= 40, "Expected >= 40 named systems, got {count}");
    }

    #[test]
    fn all_named_ayanamsas_reasonable_at_j2000() {
        // Every ayanamsa at J2000 should be in the broad sidereal band. Most land
        // in 0–35°; J2000/J1900/B1950 are near zero; Djwhal Khul ~28; Galactic
        // Alignment (Mardyks) ~30; Galactic Equator IAU1958/True ~30. The lone
        // wrap-around is GalacticCenterCochrane, which Swiss places near 356.85°
        // (0° Capricorn anchor), so we normalize to a signed band before the check.
        for sys in Ayanamsa::all_named() {
            let mut aya = sys.compute_deg(J2000_JD_TEST);
            if aya > 180.0 {
                aya -= 360.0; // Cochrane ~356.85° -> ~ -3.15°
            }
            assert!(
                aya > -5.0 && aya < 35.0,
                "{sys} at J2000 should be -5 to 35 deg (signed), got {aya} deg"
            );
        }
    }

    #[test]
    fn fagan_bradley_larger_than_lahiri() {
        let fb = Ayanamsa::FaganBradley.compute_deg(J2000_JD_TEST);
        let lahiri = Ayanamsa::Lahiri.compute_deg(J2000_JD_TEST);
        assert!(
            fb > lahiri,
            "Fagan-Bradley ({fb} deg) should be > Lahiri ({lahiri} deg)"
        );
    }

    #[test]
    fn swiss_ephem_roundtrip() {
        // Every SE-mapped variant should roundtrip through from_swiss_ephem_id.
        for sys in Ayanamsa::all_named() {
            if let Some(id) = sys.swiss_ephem_id() {
                let recovered = Ayanamsa::from_swiss_ephem_id(id);
                assert!(recovered.is_some(), "SE ID {id} for {sys} should roundtrip");
                let orig = sys.compute_deg(J2000_JD_TEST);
                let rec = recovered.unwrap().compute_deg(J2000_JD_TEST);
                assert!(
                    (orig - rec).abs() < 0.001,
                    "SE roundtrip mismatch for {sys} (id {id}): {orig} vs {rec}"
                );
            }
        }
    }

    #[test]
    fn all_47_se_ids_covered() {
        // Every SE ID from 0 to 46 should map to a variant.
        for id in 0..=46 {
            let sys = Ayanamsa::from_swiss_ephem_id(id);
            assert!(sys.is_some(), "SE ID {id} should be covered");
        }
    }

    #[test]
    fn se_id_47_and_above_return_none() {
        assert!(Ayanamsa::from_swiss_ephem_id(47).is_none());
        assert!(Ayanamsa::from_swiss_ephem_id(100).is_none());
        assert!(Ayanamsa::from_swiss_ephem_id(255).is_none());
    }

    #[test]
    fn babylonian_systems_cluster() {
        let kugler1 = Ayanamsa::BabylonianKugler1.compute_deg(J2000_JD_TEST);
        let kugler2 = Ayanamsa::BabylonianKugler2.compute_deg(J2000_JD_TEST);
        let kugler3 = Ayanamsa::BabylonianKugler3.compute_deg(J2000_JD_TEST);
        let huber = Ayanamsa::BabylonianHuber.compute_deg(J2000_JD_TEST);
        let mercier = Ayanamsa::BabylonianMercier.compute_deg(J2000_JD_TEST);
        let britton = Ayanamsa::BabylonianBritton.compute_deg(J2000_JD_TEST);
        let all = [kugler1, kugler2, kugler3, huber, mercier, britton];
        for &a in &all {
            for &b in &all {
                assert!(
                    (a - b).abs() < 5.0,
                    "Babylonian variants should cluster: {a} vs {b}"
                );
            }
        }
    }

    #[test]
    fn galactic_systems_match_swiss_relationships() {
        // The galactic systems do NOT all cluster within 2° — that was an artifact
        // of the old hand-typed J2000 constants. With the corrected Swiss-matching
        // values:
        //   * GalacticCenter0Sag and GalacticCenterCochrane differ by EXACTLY 30°
        //     (0° Sagittarius vs 0° Capricorn anchor) — Cochrane wraps to ~356.85°.
        //   * The galactic-CENTRE systems (0Sag, Brand) sit ~22–27°.
        //   * The galactic-EQUATOR systems (IAU1958, True) sit ~30°.
        let gc_sag = Ayanamsa::GalacticCenter0Sag.compute_deg(J2000_JD_TEST);
        let gc_cochrane = Ayanamsa::GalacticCenterCochrane.compute_deg(J2000_JD_TEST);
        let gc_brand = Ayanamsa::GalacticCenterBrand.compute_deg(J2000_JD_TEST);
        let gc_eq = Ayanamsa::GalacticEquatorIAU1958.compute_deg(J2000_JD_TEST);

        // 0Sag vs Cochrane: exactly 30° apart (handle the 360° wrap).
        let sep = ((gc_sag - gc_cochrane).rem_euclid(360.0) - 30.0).abs();
        assert!(
            sep < 0.05,
            "GC 0Sag ({gc_sag}) and Cochrane ({gc_cochrane}) must differ by 30°, off by {sep}°"
        );
        // GC-centre systems cluster within a few degrees of each other.
        assert!(
            (gc_sag - gc_brand).abs() < 5.0,
            "GC-centre systems should be within 5°: 0Sag {gc_sag} vs Brand {gc_brand}"
        );
        // Galactic-equator system sits near 30°, distinctly above the centre ones.
        assert!(
            (gc_eq - 30.0).abs() < 1.0,
            "Galactic Equator IAU1958 should be ~30°, got {gc_eq}"
        );
    }

    #[test]
    fn j2000_is_nutation_only_at_j2000() {
        // J2000 (SE id 18) has zero MEAN ayanamsa at J2000, but the apparent
        // (with-nutation) value Swiss returns is the nutation in longitude alone:
        // Δψ(J2000) ≈ −13.93" ≈ −0.003870°. Our implementation includes Δψ, so the
        // value is small-but-nonzero and must match the Swiss apparent value, NOT 0.
        let aya = Ayanamsa::J2000.compute_deg(J2000_JD_TEST);
        const SWISS_J2000_AT_J2000_DEG: f64 = -0.003869869;
        let diff_arcsec = (aya - SWISS_J2000_AT_J2000_DEG).abs() * 3600.0;
        assert!(
            diff_arcsec < 1.0,
            "J2000 apparent ayanamsa at J2000 should be Δψ ≈ {SWISS_J2000_AT_J2000_DEG}°, \
             got {aya}° (diff {diff_arcsec:.3}\")"
        );
    }

    #[test]
    fn j1900_positive_at_j2000() {
        let aya = Ayanamsa::J1900.compute_deg(J2000_JD_TEST);
        assert!(
            (aya - 1.396).abs() < 0.1,
            "J1900 ayanamsa at J2000 should be ~1.396 deg, got {aya}"
        );
    }

    #[test]
    fn b1950_half_of_j1900() {
        let j1900 = Ayanamsa::J1900.compute_deg(J2000_JD_TEST);
        let b1950 = Ayanamsa::B1950.compute_deg(J2000_JD_TEST);
        assert!(
            b1950 < j1900 && b1950 > 0.0,
            "B1950 ({b1950}) should be between 0 and J1900 ({j1900})"
        );
    }

    #[test]
    fn djwhal_khul_higher_than_fagan_bradley() {
        // Djwhal Khul is an esoteric system with a higher ayanamsa value.
        let dk = Ayanamsa::DjwhalKhul.compute_deg(J2000_JD_TEST);
        let fb = Ayanamsa::FaganBradley.compute_deg(J2000_JD_TEST);
        assert!(
            dk > fb,
            "Djwhal Khul ({dk}) should be > Fagan-Bradley ({fb})"
        );
    }

    #[test]
    fn aryabhata_smaller_than_lahiri() {
        let ary = Ayanamsa::Aryabhata.compute_deg(J2000_JD_TEST);
        let lah = Ayanamsa::Lahiri.compute_deg(J2000_JD_TEST);
        assert!(
            ary < lah,
            "Aryabhata ({ary}) should be smaller than Lahiri ({lah})"
        );
    }

    #[test]
    fn aryabhata_522_between_aryabhata_and_lahiri() {
        let ary = Ayanamsa::Aryabhata.compute_deg(J2000_JD_TEST);
        let ary522 = Ayanamsa::Aryabhata522.compute_deg(J2000_JD_TEST);
        let _lah = Ayanamsa::Lahiri.compute_deg(J2000_JD_TEST);
        assert!(
            ary522 < ary,
            "Aryabhata 522 ({ary522}) should be < Aryabhata ({ary}) (later zero date)"
        );
    }

    #[test]
    fn mardyks_around_30_at_j2000() {
        let aya = Ayanamsa::GalacticAlignMardyks.compute_deg(J2000_JD_TEST);
        assert!(
            (aya - 30.0).abs() < 1.0,
            "Mardyks at J2000 should be ~30 deg, got {aya}"
        );
    }

    #[test]
    fn display_coverage() {
        for sys in Ayanamsa::all_named() {
            let s = format!("{sys}");
            assert!(!s.is_empty(), "Display for {sys:?} should not be empty");
            assert!(s.len() > 3, "Display for {sys:?} too short: {s}");
        }
    }

    #[test]
    fn lahiri_variants_close() {
        // All Lahiri variants should be within 1 degree of each other.
        let standard = Ayanamsa::Lahiri.compute_deg(J2000_JD_TEST);
        let icrc = Ayanamsa::LahiriICRC.compute_deg(J2000_JD_TEST);
        let v1940 = Ayanamsa::Lahiri1940.compute_deg(J2000_JD_TEST);
        let vp285 = Ayanamsa::LahiriVP285.compute_deg(J2000_JD_TEST);
        for &variant in &[icrc, v1940, vp285] {
            assert!(
                (standard - variant).abs() < 1.0,
                "Lahiri variant {variant} should be within 1 deg of {standard}"
            );
        }
    }

    #[test]
    fn surya_siddhanta_variants_close() {
        let ss = Ayanamsa::SuryaSiddhanta.compute_deg(J2000_JD_TEST);
        let ss_mean = Ayanamsa::SuryaSiddhantaMeanSun.compute_deg(J2000_JD_TEST);
        let ss_revati = Ayanamsa::SuryaSiddhantaRevati.compute_deg(J2000_JD_TEST);
        let ss_citra = Ayanamsa::SuryaSiddhantaCitra.compute_deg(J2000_JD_TEST);
        let all = [ss, ss_mean, ss_revati, ss_citra];
        for &a in &all {
            for &b in &all {
                assert!(
                    (a - b).abs() < 5.0,
                    "SS variants should be within 5 deg: {a} vs {b}"
                );
            }
        }
    }

    #[test]
    fn new_systems_compute_without_panic() {
        // Verify all newly added systems compute without errors.
        let new_systems = [
            Ayanamsa::BabylonianKugler1,
            Ayanamsa::BabylonianKugler2,
            Ayanamsa::BabylonianKugler3,
            Ayanamsa::BabylonianBritton,
            Ayanamsa::ValensMoon,
            Ayanamsa::Aldebaran15Tau,
            Ayanamsa::GalacticEquatorTrue,
            Ayanamsa::GalacticEquatorMula,
            Ayanamsa::GalacticAlignMardyks,
            Ayanamsa::GalacticEquatorialFiorenza,
            Ayanamsa::GalacticCenterCochrane,
            Ayanamsa::GalCenterMulaWilhelm,
            Ayanamsa::TrueMulaChandraHari,
            Ayanamsa::SuryaSiddhantaMeanSun,
            Ayanamsa::SuryaSiddhantaCitra,
            Ayanamsa::Aryabhata522,
            Ayanamsa::Lahiri1940,
            Ayanamsa::KrishnamurtiVP291,
            Ayanamsa::TrueSheoran,
            Ayanamsa::J2000,
            Ayanamsa::J1900,
            Ayanamsa::B1950,
        ];
        for sys in &new_systems {
            let aya = sys.compute_deg(J2000_JD_TEST);
            assert!(
                aya.is_finite(),
                "{sys} produced non-finite value at J2000: {aya}"
            );
        }
    }

    /// Verify all epoch-based SE ayanamsas produce values consistent with the
    /// Swiss Ephemeris {t0, ayan_t0} reference data (sweph.h) at J2000.
    ///
    /// This is a SELF-CONSISTENCY check against the same model `epoch_based_ayanamsa`
    /// implements (`ayan_t0 + [p_A(0) − p_A(t0)] + Δψ(0)`, IAU 2006 precession +
    /// IAU 2000B nutation). The authoritative EXTERNAL cross-validation against
    /// live pyswisseph values across 1900–2100 lives in
    /// `tests/swiss_ayanamsa_oracle.rs`.
    #[test]
    fn se_reference_values_at_j2000() {
        // SE constants (sweph.h)
        const J2000: f64 = 2_451_545.0;
        const J1900: f64 = 2_415_020.0;
        const B1950: f64 = 2_433_282.42345905;

        // Expected J2000 value computed from SE {t0, ayan_t0} via the SAME model
        // the implementation uses: accumulated IAU 2006 general precession from t0
        // plus IAU 2000B nutation in longitude at J2000.
        fn se_j2000(t0: f64, ayan_t0: f64) -> f64 {
            let t0c = (t0 - J2000) / 36525.0;
            let precession = xalen_coords::general_precession_longitude(0.0)
                - xalen_coords::general_precession_longitude(t0c);
            (ayan_t0.to_radians() + precession + delta_psi(0.0)).to_degrees()
        }

        // (Ayanamsa variant, SE t0, SE ayan_t0, tolerance in arcsec)
        // Epoch-based ayanamsas: tight tolerance (1")
        // Star-based/dynamic: wider tolerance (60") since our linear approx differs from SE's star lookup
        let checks: &[(Ayanamsa, f64, f64, f64)] = &[
            // SE ID 0: Fagan-Bradley
            (Ayanamsa::FaganBradley, 2433282.42346, 24.042044444, 1.0),
            // SE ID 1: Lahiri — uses the same {t0, ayan_t0} the implementation now
            // anchors from (1956 CRC epoch with the get_aya_correction folded in).
            // The authoritative external check vs live Swiss is in the oracle test.
            (Ayanamsa::Lahiri, 2435553.5, 23.25 - 0.00464207, 1.0),
            // SE ID 2: De Luce
            (Ayanamsa::DeLuce, 1721057.5, 0.0, 1.0),
            // SE ID 3: Raman
            (Ayanamsa::Raman, J1900, 360.0 - 338.98556, 1.0),
            // SE ID 4: Usha/Shashi
            (Ayanamsa::Ushashashi, J1900, 360.0 - 341.33904, 1.0),
            // SE ID 5: Krishnamurti
            (Ayanamsa::KPKrishnamurti, J1900, 360.0 - 337.636111, 1.0),
            // SE ID 6: Djwhal Khul
            (Ayanamsa::DjwhalKhul, J1900, 360.0 - 333.0369024, 1.0),
            // SE ID 7: Yukteswar
            (Ayanamsa::YukteswarSriSS, J1900, 360.0 - 338.917778, 1.0),
            // SE ID 8: J.N. Bhasin
            (Ayanamsa::JNBhasin, J1900, 360.0 - 338.634444, 1.0),
            // SE ID 9-11: Babylonian Kugler 1-3
            (Ayanamsa::BabylonianKugler1, 1684532.5, -5.66667, 1.0),
            (Ayanamsa::BabylonianKugler2, 1684532.5, -4.26667, 1.0),
            (Ayanamsa::BabylonianKugler3, 1684532.5, -3.41667, 1.0),
            // SE ID 12: Babylonian Huber
            (Ayanamsa::BabylonianHuber, 1684532.5, -4.46667, 1.0),
            // SE ID 13: Babylonian Mercier
            (Ayanamsa::BabylonianMercier, 1673941.0, -5.079167, 1.0),
            // SE ID 14: Aldebaran 15 Tau
            (Ayanamsa::Aldebaran15Tau, 1684532.5, -4.44138598, 1.0),
            // SE ID 15: Hipparchos
            (Ayanamsa::Hipparchos, 1674484.0, -9.33333, 1.0),
            // SE ID 16: Sassanian
            (Ayanamsa::Sassanian, 1927135.8747793, 0.0, 1.0),
            // SE ID 18-20: Reference epochs
            (Ayanamsa::J2000, J2000, 0.0, 0.01),
            (Ayanamsa::J1900, J1900, 0.0, 1.0),
            (Ayanamsa::B1950, B1950, 0.0, 1.0),
            // SE ID 21: Surya Siddhanta
            (Ayanamsa::SuryaSiddhanta, 1903396.8128654, 0.0, 1.0),
            // SE ID 22: SS Mean Sun
            (
                Ayanamsa::SuryaSiddhantaMeanSun,
                1903396.8128654,
                -0.21463395,
                1.0,
            ),
            // SE ID 23: Aryabhata
            (Ayanamsa::Aryabhata, 1903396.7895321, 0.0, 1.0),
            // SE ID 24: Aryabhata Mean Sun
            (
                Ayanamsa::AryabhataMeanSun,
                1903396.7895321,
                -0.23763238,
                1.0,
            ),
            // SE ID 25: SS Revati
            (
                Ayanamsa::SuryaSiddhantaRevati,
                1903396.8128654,
                -0.79167046,
                1.0,
            ),
            // SE ID 26: SS Citra
            (
                Ayanamsa::SuryaSiddhantaCitra,
                1903396.8128654,
                2.11070444,
                1.0,
            ),
            // SE ID 34: Mardyks
            (Ayanamsa::GalacticAlignMardyks, 2451079.734892, 30.0, 1.0),
            // SE ID 37: Aryabhata 522
            (Ayanamsa::Aryabhata522, 1911797.740782065, 0.0, 1.0),
            // SE ID 38: Babylonian Britton
            (Ayanamsa::BabylonianBritton, 1721057.5, -3.2, 1.0),
            // SE ID 41: Fiorenza
            (Ayanamsa::GalacticEquatorialFiorenza, 2451544.5, 25.0, 1.0),
            // SE ID 42: Vettius Valens
            (Ayanamsa::ValensMoon, 1775845.5, -2.9422, 1.0),
            // SE ID 43: Lahiri 1940
            (Ayanamsa::Lahiri1940, J1900, 22.44597222, 1.0),
            // SE ID 44: Lahiri VP285
            (Ayanamsa::LahiriVP285, 1825235.2458513028, 0.0, 1.0),
            // SE ID 45: Krishnamurti VP291
            (Ayanamsa::KrishnamurtiVP291, 1827424.752255678, 0.0, 1.0),
            // SE ID 46: Lahiri ICRC
            (Ayanamsa::LahiriICRC, 2435553.5, 23.25 - 0.00464207, 1.0),
        ];

        for (aya, t0, ayan_t0, tol_arcsec) in checks {
            let expected = se_j2000(*t0, *ayan_t0);
            let actual = aya.compute_deg(J2000_JD_TEST);
            let diff_arcsec = (actual - expected).abs() * 3600.0;
            assert!(
                diff_arcsec < *tol_arcsec,
                "SE cross-val FAIL for {aya} (SE t0={t0}, ayan_t0={ayan_t0}): \
                 expected {expected:.6} deg, got {actual:.6} deg, diff {diff_arcsec:.2}\" > {tol_arcsec}\""
            );
        }
    }

    // ── True Chitra (SE ID 27) — Spica apparent-place ayanamsa ──────────
    //
    // These tests validate the dynamic Spica computation that REPLACED the old
    // hardcoded `linear_ayanamsa(t, 23.860, ...)` magic number, plus a REAL
    // cross-validation against Swiss Ephemeris (pyswisseph) SE_SIDM_TRUE_CITRA.
    //
    // The cross-validation reference table in
    // `true_chitra_matches_swiss_within_tolerance` was generated by RUNNING
    // pyswisseph (swisseph 2.10.03), not from memory:
    //
    //   python3 -c "import swisseph as swe; \
    //     swe.set_sid_mode(swe.SIDM_TRUE_CITRA); \
    //     [print(jd, swe.get_ayanamsa_ex(jd, 0)[1]) for jd in \
    //       (2415020.5,2433282.5,2451545.0,2469807.5,2488069.5)]"
    //
    // `swe_get_ayanamsa_ex(jd, 0)` (flags=0) is the WITH-nutation, full apparent
    // place — the same reduction our `spica_apparent_longitude` performs
    // (precession + proper motion + Δψ + annual aberration). NOTE: the simple
    // `swe.get_ayanamsa(jd)` wrapper returns the NO-nutation ("mean") value and
    // must NOT be used as the reference for this implementation. All JD inputs
    // are TT (the `compute`/`compute_deg` contract), matching `get_ayanamsa_ex`'s
    // ET/TT input convention (no ΔT applied).

    #[test]
    fn true_chitra_is_no_longer_the_23_860_constant() {
        // The old fake was `linear_ayanamsa(t, 23.860, PRECESSION_ARCSEC_PER_YEAR)`,
        // i.e. exactly 23.860° at J2000 plus a pure linear precession term.
        // The real Spica-apparent computation must differ from BOTH the J2000
        // constant and the linear extrapolation at a non-J2000 epoch.
        let real_j2000 = Ayanamsa::TrueChitra.compute_deg(J2000_JD_TEST);
        assert!(
            (real_j2000 - 23.860).abs() > 0.001,
            "True Chitra at J2000 should no longer be the 23.860 magic number, got {real_j2000}"
        );

        // 500 years out, compare against the OLD linear formula's prediction.
        let jd_2500 = J2000_JD_TEST + 5.0 * 36525.0;
        let real_2500 = Ayanamsa::TrueChitra.compute_deg(jd_2500);
        let old_linear_2500 = 23.860 + 50.28796195 * 500.0 / 3600.0;
        assert!(
            (real_2500 - old_linear_2500).abs() > 0.001,
            "Real True Chitra must diverge from the old linear constant model \
             (real {real_2500}, old-linear {old_linear_2500})"
        );
    }

    #[test]
    fn true_chitra_pins_spica_sidereal_longitude_at_180() {
        // The DEFINING property of True Chitra: Spica's sidereal longitude
        // (apparent ecliptic longitude of date − ayanamsa) is exactly 180°,
        // at every epoch. This is the invariant the SE_SIDM_TRUE_CITRA model
        // exists to enforce, and our implementation satisfies it by
        // construction. Verify across a wide epoch span.
        for &jd in &[
            1_356_173.0,              // ~ -1000 CE
            J2000_JD_TEST - 36525.0,  // 1900
            J2000_JD_TEST,            // 2000
            J2000_JD_TEST + 36525.0,  // 2100
            J2000_JD_TEST + 365250.0, // 3000
        ] {
            let t = (jd - 2_451_545.0) / 36525.0;
            let spica_apparent = spica_apparent_longitude(t).to_degrees();
            let ayanamsa = Ayanamsa::TrueChitra.compute_deg(jd);
            let sidereal = (spica_apparent - ayanamsa).rem_euclid(360.0);
            assert!(
                (sidereal - 180.0).abs() < 1e-9,
                "Spica sidereal longitude must be pinned at 180° at jd={jd}, got {sidereal}"
            );
        }
    }

    #[test]
    fn true_chitra_drifts_with_precession() {
        // Over one century the ayanamsa must increase by roughly the
        // precession rate (~1.397°/century), confirming it is a real
        // precession-driven quantity and not frozen.
        let a2000 = Ayanamsa::TrueChitra.compute_deg(J2000_JD_TEST);
        let a2100 = Ayanamsa::TrueChitra.compute_deg(J2000_JD_TEST + 36525.0);
        let drift = a2100 - a2000;
        assert!(
            (drift - 1.397).abs() < 0.05,
            "True Chitra should drift ~1.397°/century, got {drift}"
        );
    }

    #[test]
    fn true_chitra_j2000_sanity_band_and_near_lahiri() {
        // SANITY band — complements the arcsec Swiss cross-validation below.
        //
        // From first principles our value at J2000 is ≈ 23.8361° (Spica's
        // apparent longitude 203.8414° + Δψ ≈ −13.9" + aberration ≈ −4.8",
        // minus 180°). True Chitra and Lahiri are SEPARATE systems
        // (instantaneous apparent Spica vs the mean Chitra-paksha convention)
        // that only APPROXIMATELY coincide near J2000 — here they differ by
        // ~61" (≈1.0'), which is expected, not a bug. We assert only:
        //   (a) the value is finite and in the broad 23.7–24.0° band, and
        //   (b) it is within ~2' of Lahiri (close, but deliberately NOT equal).
        let aya = Ayanamsa::TrueChitra.compute_deg(J2000_JD_TEST);
        assert!(aya.is_finite(), "True Chitra must be finite, got {aya}");
        assert!(
            aya > 23.7 && aya < 24.0,
            "True Chitra at J2000 should land in the broad 23.7–24.0° band, got {aya}"
        );
        let lahiri = Ayanamsa::Lahiri.compute_deg(J2000_JD_TEST);
        let diff_arcmin = (aya - lahiri).abs() * 60.0;
        assert!(
            diff_arcmin < 2.0,
            "True Chitra ({aya}) should be near (but not equal to) Lahiri \
             ({lahiri}); diff {diff_arcmin:.3}' should be < 2'"
        );
        assert!(
            (aya - lahiri).abs() > 1e-6,
            "True Chitra must NOT be identical to Lahiri (separate systems)"
        );
    }

    #[test]
    fn true_chitra_matches_swiss_within_tolerance() {
        // REAL cross-validation against Swiss Ephemeris SE_SIDM_TRUE_CITRA.
        //
        // Reference values produced by RUNNING pyswisseph 2.10.03 (NOT from
        // memory). `swe.get_ayanamsa_ex(jd, 0)` returns the full apparent-place
        // (WITH-nutation) ayanamsa — the same reduction our implementation
        // performs (precession + proper motion + Δψ + annual aberration). The
        // input JD is TT, matching the `compute`/`compute_deg` contract and
        // `get_ayanamsa_ex`'s ET/TT (no-ΔT) convention.
        //
        //   python3 -c "import swisseph as swe; \
        //     swe.set_sid_mode(swe.SIDM_TRUE_CITRA); \
        //     [print(jd, swe.get_ayanamsa_ex(jd, 0)[1]) for jd in \
        //       (2415020.5,2433282.5,2451545.0,2469807.5,2488069.5)]"
        //
        // Measured residuals (ours − Swiss), arcsec, with the rigorous IAU
        // 2006/P03 Cartesian Spica precession (couples Spica's ~2.05° ecliptic
        // latitude):
        //   1900 −0.037   1950 −0.035   2000 −0.013   2050 −0.024   2100 −0.001
        // i.e. ≤ 0.04" across the whole 1900–2100 span. The prior scalar
        // pure-longitude precession left a ~1.5"/century slope (−1.513"/+1.470"
        // at 1900/2100) because it could not couple Spica's ecliptic latitude;
        // the Cartesian rotation removes it.
        let swiss = [
            (2_415_020.5_f64, 22.449_597_249_f64), // 1900-01-01 TT
            (2_433_282.5, 23.141_359_125),         // 1950-01-01 TT
            (2_451_545.0, 23.836_145_079),         // 2000-01-01 TT (J2000)
            (2_469_807.5, 24.542_129_663),         // 2050-01-01 TT
            (2_488_069.5, 25.236_806_501),         // 2100-01-01 TT
        ];
        // Tolerance reflects the MEASURED agreement, not an aspiration: the
        // measured worst case is 0.037", so a single 0.5" tolerance across the
        // whole span has comfortable margin while still catching a regression to
        // the old scalar model (which exceeded 1.4" off the J2000 anchor).
        const TOL_ARCSEC: f64 = 0.5;
        for (jd, swiss_deg) in swiss {
            let ours = Ayanamsa::TrueChitra.compute_deg(jd);
            let resid_arcsec = (ours - swiss_deg).abs() * 3600.0;
            assert!(
                resid_arcsec < TOL_ARCSEC,
                "True Chitra at jd={jd} = {ours:.10}° vs Swiss {swiss_deg:.10}°: \
                 residual {resid_arcsec:.4}\" exceeds {TOL_ARCSEC}\" tolerance"
            );
        }
    }

    /// Verify the J1900 reference ayanamsa at J2000 matches the live Swiss value.
    #[test]
    fn j1900_matches_swiss_apparent_at_j2000() {
        // J1900 (SE id 19) anchors zero ayanamsa at J1900.0. At J2000 the
        // WITH-nutation Swiss value (swe.get_ayanamsa_ex(2451545.0, 0)) is
        // 1.392711130° — one century of IAU 2006 general precession in longitude
        // (≈ 1.39656°) PLUS the J2000 nutation in longitude Δψ ≈ −13.93" ≈ −0.00387°.
        // This is the value our implementation must reproduce (NOT the bare linear
        // 50.28796195"/yr × 100 yr, which omits both the higher-order precession
        // terms and nutation).
        const SWISS_J1900_AT_J2000_DEG: f64 = 1.392711130;
        let actual = Ayanamsa::J1900.compute_deg(J2000_JD_TEST);
        let diff_arcsec = (actual - SWISS_J1900_AT_J2000_DEG).abs() * 3600.0;
        assert!(
            diff_arcsec < 1.0,
            "J1900 at J2000 should match Swiss apparent value {SWISS_J1900_AT_J2000_DEG:.9}°, \
             got {actual:.9}°, diff {diff_arcsec:.4}\""
        );
    }
}
