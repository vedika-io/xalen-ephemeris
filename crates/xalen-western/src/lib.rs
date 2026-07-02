/// Antiscia and contra-antiscia — reflections across the solstitial axis.
pub mod antiscia;
/// Planetary aspects (conjunction, opposition, trine, square, sextile, etc.).
pub mod aspects;
/// Cosmobiology (Ebertin) methods and analysis.
pub mod cosmobiology;
/// Declination aspects — parallel and contraparallel.
pub mod declination;
/// Essential and accidental dignities for Western astrology.
pub mod dignity;
/// Electional astrology timing rules.
pub mod electional;
/// Harmonic charts and harmonic aspects.
pub mod harmonics;
/// Heliacal rise and set computation for planets and stars.
pub mod heliacal;
/// Hellenistic techniques (sect, bounds, decans, etc.).
pub mod hellenistic;
/// Horary astrology judgment rules.
pub mod horary;
/// Arabic Parts / Lots (Part of Fortune, Spirit, etc.).
pub mod lots;
/// Lunar phases, eclipses, and void-of-course Moon.
pub mod lunar;
/// Midpoint structures and trees.
pub mod midpoints;
/// Chart patterns (T-square, Grand Trine, Yod, etc.).
pub mod patterns;
/// Secondary progressions, solar arc directions.
pub mod progressions;
/// Solar and lunar return charts.
pub mod returns;
/// Sabian symbol lookup for each zodiac degree.
pub mod sabian;
/// Western sidereal astrology (Fagan-Bradley system).
pub mod sidereal;
/// Fixed star catalog — 506 named stars with J2000.0 positions.
///
/// Gated behind the default-on `hip-catalog` feature: its coordinates derive
/// from the Hipparcos catalogue (ESA 1997) + SIMBAD, which is non-commercial,
/// so commercial builds (`--no-default-features`) exclude this module entirely.
#[cfg(feature = "hip-catalog")]
pub mod stars;
/// Uranian astrology and Transneptunian Points.
pub mod uranian;
