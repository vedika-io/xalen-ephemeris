/// Ashtakavarga point computation and analysis.
pub mod ashtakavarga;
/// Ashtottari dasha system (108-year cycle).
pub mod ashtottari;
/// Jaimini Chara Dasha system.
pub mod chara_dasha;
pub mod compatibility;
pub mod dasha;
pub mod dignity;
/// Divisional (Varga) chart computation (D-2 through D-60).
pub mod divisional;
/// Dosha detection (Mangal/Kuja Dosha, Kaal Sarp, etc.).
pub mod dosha;
/// Gandanta (water-fire sign junction) detection and severity.
pub mod gandanta;
/// Multi-language names (Hindi, Sanskrit, Tamil, Telugu) for planets,
/// rashis, nakshatras, tithis, yogas, karanas, and varas.
pub mod i18n;
/// Jaimini astrology (Chara Karakas, Pada, Arudha).
pub mod jaimini;
pub mod kp;
/// Mrityu Bhaga (death degrees) — inauspicious planetary degrees per sign.
pub mod mrityu_bhaga;
/// Muhurta (electional) timing analysis.
pub mod muhurta;
/// Nadi astrology techniques and significations.
pub mod nadi;
pub mod nakshatra;
pub mod panchang;
/// Panchang TRANSITION/END times — when the current tithi, nakshatra, yoga, and
/// karana begin and end (root-finding to ~second precision over a caller-supplied
/// Sun/Moon ephemeris closure).
pub mod panchang_transitions;
/// Prashna (horary) astrology techniques.
pub mod prashna;
/// Pushkara Navamsa and Pushkara Bhaga (auspicious degrees/navamsas).
pub mod pushkara;
pub mod rashi;
/// Retrograde detection and motion status classification.
pub mod retrograde;
pub mod shadbala;
/// Sudarshana Chakra Dasha system.
pub mod sudarshana;
/// Tajaka (annual horoscopy) yogas and techniques.
pub mod tajaka;
/// Transit analysis and Gochara (transit effects).
pub mod transit;
/// Sub-planetary bodies (Gulika, Mandi, Dhuma, etc.).
pub mod upagraha;
/// Varshaphal (annual chart/solar return) analysis.
pub mod varshaphal;
/// Classical planetary yoga detection (Pancha Mahapurusha, Raja, etc.).
pub mod yoga;
/// Yogini dasha system (36-year cycle).
pub mod yogini;
