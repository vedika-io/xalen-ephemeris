# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-05-25

Initial public release. 18 crates, ~71K lines of Rust, 1,847 tests passing.

### Added

#### Core Engine (`xalen-ephem`)
- VSOP87A analytical ephemeris for Mercury through Neptune (< 1" inner, ~1" outer)
- ELP2000-82 analytical lunar theory (~2" accuracy)
- Meeus Ch.37 Pluto position (valid 1885-2099, ~15")
- Bate-Mueller-White Chiron orbital elements
- True Node (osculating) and Mean Node (IAU expression) for Rahu/Ketu
- JPL DE440 binary Chebyshev reader for sub-milliarcsecond precision
- `Almanac` provider-stack with automatic body fallback
- `EphemerisProvider` trait for pluggable computation backends
- 17 asteroids: main belt (Ceres, Pallas, Juno, Vesta, Hygeia, Astraea, Psyche, Eros, Lilith-1181), centaurs (Pholus, Nessus), TNOs (Eris, Sedna, Makemake, Haumea), plus external element loader for custom asteroids
- Event search: sign ingress, station, and generic crossing finder

#### Time (`xalen-time`)
- Julian Day newtypes: `JdTT`, `JdUT1`, `JdTDB` with arithmetic and conversions
- Delta-T models: Stephenson-Morrison-Hohenkerk 2016, Espenak-Meeus 2006, Morrison-Stephenson 2004
- Calendar conversions: Proleptic Gregorian, Proleptic Julian, Julian with custom cutover
- `Epoch` type for named reference epochs (J2000, J1900, B1950, etc.)

#### Coordinates (`xalen-coords`)
- Ecliptic, equatorial, and Cartesian coordinate types
- Frame transforms: ecliptic-equatorial, Cartesian-ecliptic
- IAU 2006 precession (Capitaine et al., ~0.3 mas/century)
- IAU 2000B nutation (77-term truncated series, ~1 mas)
- Mean and true obliquity

#### Houses (`xalen-houses`)
- 23 house systems: Whole Sign, Equal, Placidus, Koch, Porphyry, Regiomontanus, Campanus, Morinus, Alcabitius, Topocentric (Polich-Page), Meridian, Vehlow, Sripati, Krusinski-Pisa, Gauquelin sectors, Sunshine Makransky, Sunshine Treindl, Pullen Sinusoidal Delta, Pullen Sinusoidal Ratio, Carter Poli-Equatorial, APC, Zariel (Axial Rotation), Alcabitius Classic
- Ascendant, MC, IC, Descendant, Vertex computation
- Polar region handling with automatic Porphyry fallback at > 66.5 deg latitude
- `planet_in_house()` search across cusp boundaries
- Swiss Ephemeris single-character code mapping for each system

#### Ayanamsa (`xalen-ayanamsa`)
- 48 systems (47 named + Custom): Lahiri, KP Krishnamurti, Raman, Fagan-Bradley, True Chitrapaksha, True Revati, Surya Siddhanta, Sri Yukteswar, J.N. Bhasin, De Luce, Ushashashi, Pushya Paksha, Lahiri ICRC, KP Straight Line, Lahiri VP285, Lahiri 1940, Krishnamurti VP291, Hipparchos, Aldebaran 15 Tau, Galactic Center (0 Sag, Brand, Cochrane, Mula, Wilhelm, Fiorenza, True Mula), Galactic Equator (IAU 1958, True, Mula), Galactic Alignment Mardyks, Babylonian (Kugler 1, 2, 3, Huber, Eta Piscium, Aldebaran), Sassanian, Mercier, and more -- all Swiss Ephemeris ayanamsa IDs covered
- `Custom` variant with user-defined epoch, value, and precession rate
- `tropical_to_sidereal()` and `sidereal_to_tropical()` conversion functions
- `compute()` (radians) and `compute_deg()` (degrees) for each system
- Swiss Ephemeris ayanamsa ID mapping for cross-validation

#### Fixed Stars (`xalen-stars`)
- 108-star built-in catalog with J2000 ecliptic coordinates
- Proper motion in longitude and latitude
- Precession-corrected positions at any epoch
- Conjunction search with configurable orb
- Nakshatra yogatara (reference star) mapping
- Runtime catalog loader for Hipparcos (118,218 stars) or custom CSV files

#### Vedic Astrology (`xalen-vedic`)
- **Nakshatra**: 27 nakshatras with pada, lord, deity, gana classification
- **Rashi**: 12 signs with lord, element, modality, Western name mapping
- **Vimshottari Dasha**: 5-level computation (Mahadasha through Pranadasha) with dasha balance
- **Ashtottari Dasha**: 8-planet 108-year system
- **Yogini Dasha**: 8-yogini 36-year system
- **Panchang**: Tithi, Nakshatra, Yoga (27), Karana (11), Vara
- **Ashtakavarga**: Bhinna (BAV) per BPHS Ch.66-72 with full bindu tables, Sarvashtakavarga (SAV)
- **Shadbala**: Sthana Bala (Uchcha, Kendra, Drekkana, Ojhayugma), Dig Bala, Kala Bala, Cheshta Bala, Naisargika Bala, Drik Bala
- **KP System**: sub-lord computation, significator tables, ruling planets
- **Jaimini**: Chara Karakas (7+1), Chara Dasha with sub-periods
- **Tajaka**: 16 yogas (Ikbaal through Tambira), Ithasala check, Sahams (day/night reversal), annual horoscopy
- **Prashna**: horary significations, Moon strength, planetary hour
- **Muhurta**: electional quality tables, Chaughadiya, Hora
- **Nadi**: Bhrigu Bindu, progression-based Nadi framework
- **Compatibility**: Ashta Kuta (8-factor) and Dasha Kuta (10-factor) matching
- **Divisional charts**: D1 through D60 (16 divisions including D2, D3, D4, D7, D9, D10, D12, D16, D20, D24, D27, D30, D40, D45, D60)
- **Dosha**: Mangal, Kaal Sarp, Pitru, Shani, Grahan, Kemdrum detection
- **Yoga**: classical yoga identification (Gajakesari, Budhaditya, Viparita, etc.)
- **Upagraha**: Dhuma, Vyatipata, Parivesha, Indrachapa, Upaketu, Gulika, Mandi
- **Transit**: Gochara analysis with Vedha and Ashtakavarga-based transit strength
- **Sudarshana Chakra**: triple overlay chart
- **Varshaphal**: annual horoscopy with Muntha
- **Narayana Dasha**: sign-based dasha with Sthira sub-periods

#### Lal Kitab (`xalen-lalkitab`)
- 108 planet-house effects with textual descriptions
- 5 debt types (Rin): Pitru, Matru, Stri, Kanya, Atma
- Dormant planet detection
- Remedy lookup per planet-house combination
- Varshphal (annual chart) support

#### Western Astrology (`xalen-western`)
- **Aspects**: 11 types (conjunction, opposition, trine, square, sextile, semi-sextile, quincunx, semi-square, sesquiquadrate, quintile, bi-quintile) with applying/separating/exact detection
- **Essential Dignities**: Ptolemaic 5-level scoring (domicile, exaltation, triplicity, term/bounds per Tetrabiblos I.21, face/decan)
- **Arabic Lots**: 97 lots with full day/night reversal formulas (Fortune, Spirit, Eros, Necessity, Marriage, etc.)
- **Hellenistic**: sect determination, planetary joys, Whole Sign profections, bounds
- **Sabian Symbols**: 360-degree symbol lookup
- **Uranian**: 8 transneptunian points (Cupido through Poseidon), midpoint trees
- **Cosmobiology**: midpoint analysis, 90-degree dial sort
- **Progressions**: secondary, solar arc, converse progressions
- **Returns**: solar, lunar, planetary return chart computation
- **Harmonics**: harmonic chart generation (H1-H180)
- **Horary**: essential/accidental dignity, receptions, void-of-course Moon, planetary hour
- **Electional**: planetary hours, Moon gardening calendar
- **Sidereal Western**: Fagan-Bradley-based sidereal positions
- **Lunar phases**: New Moon, Full Moon, quarter detection
- **Chart patterns**: Grand Trine, T-Square, Grand Cross, Stellium, Yod

#### Chinese Astrology (`xalen-chinese`)
- BaZi (Four Pillars): Year, Month, Day, Hour with Heavenly Stems and Earthly Branches
- Wu Xing (Five Elements): generating and overcoming cycles
- Sexagenary cycle: 60-year and 60-day continuous count
- Solar terms: 24 boundaries at 15-degree Sun intervals
- Hour branch and stem derivation from day stem
- Zi Wei Dou Shu: Ming Gong derivation, 14 main star placement, 12 palaces
- Feng Shui: Flying Stars (Xuan Kong), Ba Zhai (8 Mansions), annual/monthly star charts
- Qi Men Dun Jia: 9 stars, 8 doors, 8 deities, 3 Wonders (San Qi)

#### I Ching (`xalen-iching`)
- 64 hexagrams with King Wen sequence numbering
- 8 trigrams (Ba Gua) with element, direction, family associations
- Date-based hexagram derivation
- Nuclear hexagram computation
- Relating (changed) hexagram via moving lines
- Fu Xi (binary) sequence ordering

#### Numerology (`xalen-numerology`)
- Pythagorean and Chaldean letter-value systems
- Life Path, Expression, Soul Urge, Personality, Maturity, Birthday numbers
- Master number (11, 22, 33) preservation
- Full profile computation from name and birthdate

#### World Systems (`xalen-world`)
- **Mayan**: Tzolkin (260-day), Haab (365-day), Long Count, Calendar Round
- **Aztec**: Tonalpohualli 260-day calendar with 20 day signs and 13 numbers
- **Tibetan**: calendar with Mewa (9 numbers) and Parkha (8 trigrams)
- **Korean Saju**: Four Pillars Korean variant
- **Japanese Nine Star Ki**: annual, monthly, and daily star assignment
- **Burmese Mahabote**: planetary weekday system with life stages
- **Persian**: Zoroastrian Gahambar festivals, Yasna calendar
- **Egyptian**: decan system, Sothic cycle
- **Celtic**: tree calendar, Ogham letter associations

#### Chart Rendering (`xalen-chart`)
- SVG chart rendering with zero external dependencies
- North Indian diamond chart (Vedic standard)
- South Indian box chart
- Western wheel chart
- `ChartData` struct for passing planet positions and house cusps

#### Bindings
- C FFI (`xalen-ffi`): `extern "C"` exports with `repr(C)` structs
- Python (`xalen-python`): PyO3 bindings for position, panchang, nakshatra, houses
- Node.js (`xalen-node`): napi-rs native addon
- WASM (`xalen-wasm`): browser and Node.js via wasm-bindgen

#### Examples
- `basic_chart`: Sun/Moon positions with nakshatra and rashi (India Independence, 1947)
- `vedic_chart`: full Vedic chart with dasha, shadbala, panchang
- `western_chart`: Western chart with aspects, dignities, Arabic Lots
- `chinese_bazi`: BaZi Four Pillars with Wu Xing analysis

### Fixed
- Naisargika Bala ordering corrected to BPHS standard (Sun > Jupiter > Mars > Moon > Mercury > Venus > Saturn)
- Punya Saham (Lot of Fortune in Tajaka) day/night formula reversal
- ZWDS Ming Gong derivation from birth hour and month branch
- Ashtakavarga SAV total corrected to sum across all 8 contributors
- Ashtakavarga Lagna contributor row added to BAV computation
- Sripati house cusps: angular midpoint calculation
- Placidus polar threshold set to 66.5 degrees with automatic Porphyry fallback
- VSOP87A positions precessed from J2000 ecliptic to equinox-of-date
- Moon latitude sign (ELP2000-82 argument correction)
- Egyptian terms (bounds) entries aligned with Tetrabiblos I.21 Robbins translation
- Firdaria sub-period night-sect reversal
- Decennials minor period lord sequence
- Replaced panicking `assert!` with clamping in boundary conditions
- Removed `unwrap()` calls on user-controlled input paths
