# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.0] - 2026-06-02

This entry consolidates Wave B (accuracy / breadth) and Wave C (bindings parity +
new computational surfaces) on top of the 0.4.3 licensing-hygiene cleanup. All
figures below are reproducible from this repo's test suite; nothing here is
published to crates.io / PyPI / npm yet (publishing is gated — see *Distribution*).

### Added

#### New astronomical / chart surfaces (`xalen-ephem`, `xalen-western`)
- **Optional automatic DE440 kernel provisioning** (`kernel-autodownload` cargo
  feature, **off by default**). New `kernel_cache` module with
  `ensure_de440s_kernel()` / `KernelFetch` and a one-call
  `De440Provider::from_auto_cache()`: on first use it fetches the public NASA
  NAIF `de440s.bsp` (~32 MB) into the per-OS cache directory, verifies it
  (structural DE440 provenance via the existing DAF/SPK parser, plus an optional
  SHA-256 when `XALEN_DE440S_SHA256` is set), and reuses the cached copy with no
  network access thereafter. With the feature enabled the apparent Moon — and
  every body the kernel covers — is sub-arcsecond out of the box. The feature is
  off by default so the base crate stays offline and crates.io-clean; without it
  the analytical Moon is the measured RMS ~2.9″ / max ~12″ vs pyswisseph 2.10.03
  over AD 1600–2100 (growing toward antiquity: RMS ~4.2″ by AD 1000, ~9.6″ by
  AD 1). Cache location is overridable with `XALEN_KERNEL_CACHE_DIR`.
- **Equatorial (RA/Dec), heliocentric, and rectangular (XYZ) output frames** on
  `Almanac` (new `output` module), mirroring Swiss `SEFLG_EQUATORIAL` /
  `SEFLG_HELCTR` / `SEFLG_XYZ`. `geocentric_equatorial[_tt]` rotates the apparent
  ecliptic-of-date place by the **true** obliquity (mean + nutation-in-obliquity)
  so RA/Dec carry the same nutation / aberration / light-time as the ecliptic
  place; `heliocentric_ecliptic[_tt]` / `*_rectangular` expose the provider's
  heliocentric place and the Cartesian forms of each spherical place. Validated
  against pyswisseph 2.10.03 at J2000 (Sun/Moon/Mars RA & Dec) — the equatorial
  rotation itself is exact (< 0.001″); residuals are the underlying
  analytical-place error only. Geometric points (nodes, Lilith) have no
  heliocentric place and return `BodyNotAvailable`.
- **True (osculating) Black Moon Lilith**: `lilith::true_lilith` (+ provider body
  `Body::OsculatingApogee` and compat `SE_OSCU_APOG` = 13), the apogee of the
  Moon's *instantaneous* orbit. Derived from the same geocentric Moon state vector
  used for the true node, via the Laplace–Runge–Lenz (eccentricity) vector with
  the Earth–Moon GM; the apogee is the +180° flip of the perigee direction.
  Validated vs pyswisseph `SE_OSCU_APOG` at committed spot fixtures (J2000 and
  the 1992-04-12 Meeus epoch), each within 0.5° — at the level of the osculating
  apogee's intrinsic, documented model spread. The existing mean Lilith is
  unchanged.
- **Exact return finders** (`returns` module): `find_return` / `find_return_tt`
  for `ReturnBody::{Sun, Moon, Mars, Jupiter, Saturn}`. Each roots the body's
  natal longitude on the **real `Almanac` longitude** — bracketing the first
  crossing forward from the search start (with ±180° wrap-discontinuity rejection
  so a body ~opposite its natal degree is never mistaken for a crossing), then a
  safeguarded Newton–Raphson (bisection on any out-of-bracket step or retrograde
  station) that pins the crossing on the engine's own longitude to ≈1e−7°.
  Previously only the **solar** return was exact (analytic Sun, in `xalen-western`);
  the others used a mean period that can be weeks off because real motion is far
  from uniform. Validated vs pyswisseph 2.10.03: a Saturn return to 294.0° lands at
  JD 2458871.555 UT (2020-01-23), matching the independent Swiss search to within
  the committed test bound of 0.1 day (the timing precision vs. an external
  ephemeris is set by the longitude-model agreement, not by the root-finder). The
  `solar_return` example now uses the exact finder + the almanac Sun.
  (`xalen-western::returns` mean-period helpers are retained, unchanged.)
- **Declination aspects (parallel / contraparallel)** in `xalen-western`: a new
  `declination` module computes a body's declination from ecliptic
  longitude/latitude and obliquity via the standard ecliptic→equatorial rotation
  (Meeus, *Astronomical Algorithms* 2nd ed., eq. 13.4, reusing
  `xalen-coords::ecliptic_to_equatorial`) and detects parallels (same-sign
  declination within orb, conjunction-like) and contraparallels (opposite-sign,
  opposition-like). Validated against pyswisseph equatorial output (< 0.002°).
- **Antiscia and contra-antiscia** in `xalen-western`: a new `antiscia` module
  reflects longitudes across the 0° Cancer / 0° Capricorn solstitial axis
  (`antiscion = 180° − λ`) and the equinoctial axis (`contra = 360° − λ`), and
  detects antiscion/contra-antiscion contacts. Pure longitude geometry; canonical
  sign pairs match Brennan (*Hellenistic Astrology*) and Lilly
  (*Christian Astrology*).
- **Chart patterns Grand Cross, Kite, and Mystic Rectangle** are now detected by
  `patterns::detect_patterns` (previously dead enum variants): Grand Cross = two
  oppositions with four squares; Kite = Grand Trine plus a body opposing one apex
  and sextile to the other two; Mystic Rectangle = two oppositions joined by two
  trines and two sextiles.

#### Houses (`xalen-houses`)
- **Auxiliary ascendant points** — every `HouseCusps` now carries the four Swiss
  `ascmc[4..8]` angles: `equatorial_ascendant` (East Point), `co_ascendant_koch`,
  `co_ascendant_munkasey`, and `polar_ascendant_munkasey`, also exposed standalone
  via `compute_auxiliary_ascmc` / the `AuxiliaryAscendants` struct. Pure spherical
  math from RAMC/ε/φ through a faithful Swiss `Asc1`/`Asc2` oblique-ascension
  primitive (the literal `atan(sin x / D)` + sign-of-`D` half-turn, distinct from
  the `atan2` form the quadrant cusps use). Validated bit-for-bit (worst Δ 0.0°)
  against pyswisseph 2.10.03 `swe.houses_armc(..., b'P')[1]` across 22
  latitude/date cases spanning −80°…+80°.
- **Sidereal house path** — `compute_houses_sidereal` and
  `HouseCusps::to_sidereal(ayanamsa_rad)` produce the
  `swe_houses_ex(SEFLG_SIDEREAL)` equivalent (the dominant Vedic path) by
  subtracting the ayanamsa from every cusp and every angle. Cross-checked against
  pyswisseph `swe.houses_ex(..., FLG_SIDEREAL)` with the Lahiri ayanamsa.
- **Gauquelin sectors — real 36-fold mundane division** (replaces the former
  placeholder that returned Placidus cusps). New `gauquelin_sectors(ramc, eps,
  phi) -> [f64; 36]` returns the 36 sector-boundary ecliptic longitudes counted
  clockwise from the Ascendant, and `gauquelin_position(ramc, eps, phi, lon,
  lat)` the continuous `[1, 37)` sector value of a body. Each boundary is the
  point whose mundane position equals its sector index: the semi-diurnal and
  semi-nocturnal arcs are each divided into nine, so every third boundary lands
  on a Placidus cusp (sector 1 = ASC, 10 = MC, 19 = DSC, 28 = IC). Validated
  against pyswisseph 2.10.03 `swe.house_pos(..., b'G')` — the continuous position
  matches to worst Δ = 0 sector over four epochs × ten latitudes × the full
  ecliptic, boundary longitudes to < 5e-5°, and the every-third boundary equals
  the Placidus cusp to < 1e-4°. The `HouseSystem::Gauquelin` 12-cusp output now
  returns those twelve every-third boundaries (genuine Gauquelin sub-boundaries,
  identical to the Placidus cusps); inside the polar circle it falls back to a
  nine-fold Porphyry sectoring so the 36-boundary contract never panics.

#### Panchang (`xalen-vedic`)
- **Panchang transition / end times**: a new `panchang_transitions` module
  computes when the current tithi, nakshatra, yoga, and karana begin and end —
  the single most-consumed panchang output (e.g. "Tithi: Dashami upto 05:11 AM"),
  previously absent. Each element's boundary is the next instant its driving angle
  (Moon−Sun elongation for tithi/karana, Moon longitude for nakshatra, Sun+Moon
  for yoga) reaches an integer multiple of its span; the crossing is bracketed
  forward (or backward for the optional start time) and bisected to sub-second
  precision over a caller-supplied `Fn(jd) -> (sun_sidereal_deg, moon_sidereal_deg)`
  ephemeris closure (no ephemeris is bundled in this crate). Re-exported from
  `panchang` as `tithi_transition`, `nakshatra_transition`, `yoga_transition`,
  `karana_transition`, and `compute_panchang_transitions`. Validated against an
  independent pyswisseph 2.10.03 computation: tithi-10 end on 25 May 2026
  reproduces pyswisseph's JD 2461186.4871 (26 May 2026 05:11 IST) to within a few
  minutes (limited by ephemeris differences, not the solver).

#### I Ching (`xalen-iching`)
- **All 384 per-line (yao ci / changing-line) texts** — the full line text for
  every hexagram (64 × 6 = 384 lines), verbatim from James Legge, *The Yî King*,
  Sacred Books of the East Vol. XVI (1882), public domain. Transcribed from the
  Internet Sacred Text Archive edition (`sacred-texts.com/ich/ic01.htm` …
  `ic64.htm`) and cross-checked against a second independent public-domain
  transcription; **384 / 384 fetched, 0 MISSING**. New API: `line_texts(n)`,
  `line_text(n, line)`, `use_line_text(n)` (Legge's two supplementary "use of the
  number" statements for Hexagrams 1 & 2), plus `Hexagram::line_texts` /
  `Hexagram::line_text` / `Hexagram::use_line_text` methods. `HexagramReading`
  gains a `changing_line_text` field carrying the moving line's text. Legge's
  parentheticals and romanisation diacritics (â, î, Î, Ž) are preserved; only
  page-break markers and a few obvious scan artifacts in the line *numbering* were
  corrected. The interpretive content is the verbatim public-domain Legge 1882 text.

#### Burmese Mahabote (`xalen-world`)
- **The deterministic 7-house square** (`mahabote_house_square`,
  `mahabote_house_square_from_jd`) — the positional cast that the existing
  weekday profile previously only described in a scope note. The seven houses
  `MahaboteHouse` (Binga, Ahtun, Yaza, Adipati, Marana, Thike, Puti) are seated
  by the deterministic rule: the birth-weekday lord opens the square in Binga and
  the remaining lords follow the fixed Burmese weekday sequence (Sun, Moon, Mars,
  Mercury, Jupiter, Venus, Saturn) clockwise — the lord of weekday
  `(birth + h) mod 7` occupies house `h`. The Wednesday-PM (Rahu) variant
  substitutes Rahu for the Wednesday lord. New `MahaboteSeat` /
  `MahaboteHouseSquare` types with `house_of_weekday`. The structural placement
  is the attested deterministic core; lineage-specific house *interpretation* is
  out of scope and the canonical short gloss only is returned (`meaning()`).

#### Qi Men Dun Jia (`xalen-chinese`)
- **Time-chart Ju determination by the San Yuan rule** (`qimen_ju`), replacing
  the former coarse civil-month bucketing. The Ju is read from the canonical San
  Yuan table at the instant's solar term (indexed from Dong Zhi via the Sun's
  longitude) and the upper/middle/lower yuan is chosen by the day pillar's
  Fu Tou (符头) — the 5-day run head's branch group. Yang Dun stores Ju 1–9,
  Yin Dun 10–18.
- **`compute_qimen` now casts the San Yuan time chart** (時家奇門) instead of the
  former experimental demo: the Earth Plate seats the six Yi + three Qi from the
  Jia palace along the Yang/Yin flight, and **Zhi Fu (值符) / Zhi Shi (值使) are
  anchored to the hour's Xun-head Yi (旬首)** — the palace where the Six-Yi stem
  hiding the hour's leading Jia sits — leading the Heaven-Plate stars, doors, and
  the eight Deities along the dun direction. `QiMenChart::zhi_fu_palace()` exposes
  the presiding palace; the type is no longer `#[doc(hidden)]`. Qi Men has genuine
  school variation, so this is documented as the San Yuan time-chart school
  implemented consistently, not a universal claim.

### Changed

- **True Chitrapaksha ayanamsa (SE 27) now < 1″ vs Swiss across 1900–2100.** Its
  Spica apparent-place reduction now precesses Spica's J2000 (longitude, latitude)
  with the rigorous IAU 2006/P03 Cartesian rotation
  (`precess_ecliptic_to_of_date` + `precession_matrix_p03_nobias`) instead of the
  scalar pure-longitude `general_precession_longitude` term. Spica lies ~2.05° off
  the ecliptic, so its longitude precession is latitude-coupled; the scalar term
  could not represent that coupling and left a ~1.5″/century span residual
  (1.514″ at 1900/2100). The Cartesian rotation removes it: cross-validated
  against pyswisseph 2.10.03 `SE_SIDM_TRUE_CITRA` to **0.038″** at the 1900–2100
  oracle epochs and **≤ 0.04″** at off-grid (non-Jan-1) dates. The ayanamsa Swiss
  oracle now holds SE 27 under a 0.5″ guard and a dedicated off-grid guard, and
  removes it from the documented-2″ exception list. **46 of 47 SE systems are now
  < 1″ vs Swiss**; the single remaining documented exception is
  `GalCenterMulaWilhelm` (SE 36, ≤ 1.42″ — its Swiss-internal reduction has a
  ~52.5″/yr precession rate combined with a moderate-latitude annual aberration
  that no single fixed celestial direction reproduces; this is documented in code
  and `docs/ACCURACY.md`, not hidden by tolerance widening).
- **Krusinski-Pisa-Goelzer house cusps (Swiss `U`) — see Fixed below** for the
  corrected great-circle construction.
- **Empty-content honesty contract** — `xalen-numerology::number_meaning` now
  returns `Option<&'static str>` (`None` for every number, since no interpretive
  prose is bundled) instead of `&'static str` returning `""`; and
  `xalen-lalkitab`'s `LalKitabRemedy::remedies` / `items` are now
  `Option<Vec<String>>`, reporting absent (stripped, not-yet-backfilled) content
  as `None` rather than `Some(vec![])`. Callers can now distinguish "no content
  bundled" from genuinely empty content. No numerology/Lal Kitab interpretive
  prose is bundled.
- **Cosmobiology empty-API honesty contract**: `cosmobiology::lookup_midpoint_key`
  now returns `None` for stripped (empty-text) interpretation entries instead of
  `Some("")`, and planetary pictures built by `cosmobiology_chart` carry `None`
  for stripped entries. No fabricated interpretive text is ever surfaced — an
  absent entry is reported as absent.
- **Nadi empty-API honesty contract**: `nadi::NadiRule.indication` is now
  `Option<&'static str>` and `nadi_indications` returns `None` for the unbundled
  (copyrighted, not-shipped) BNN interpretive readings instead of `Some("")`. An
  absent reading is reported as genuinely absent, never as a misleading empty
  string; the life-`domain` classification remains the only interpretive data
  this crate provides.

### Fixed

- **Krusinski-Pisa-Goelzer house cusps (Swiss `U`) now match Swiss Ephemeris.**
  The previous projection divided the inclined house circle into equal 30° arcs
  and mapped them back through a `swe_cotrans` rotation chain; cusp 10 no longer
  landed on the MC and the intermediate cusps were 13–37° off Swiss. Replaced
  with the correct ASC-centred great-circle construction — the great circle
  through the Ascendant and the local Zenith, stepped 30° toward the MC, each
  division's hour circle intersected with the ecliptic — so cusp 1 = ASC,
  cusp 4 = IC, cusp 7 = DSC, cusp 10 = MC exactly, and all twelve cusps match
  pyswisseph 2.10.03 `swe.houses_armc(..., b'U')` bit-for-bit (worst Δ 0.0°)
  across −66°…+66° and both J2000.0 and JD 2460000.5. The `KrusinskiPisa` row of
  the `swiss_houses_oracle` integration test is re-enabled at the tight 0.01°
  tolerance (the prior `continue` skip is removed).

### Distribution / availability

- **Not yet published.** The Rust crates on crates.io are at **0.3.1** (leaf
  crates); the 0.4.x / 0.5.x / this candidate are **not** published — publishing
  is founder-gated and pending. The Node.js (`xalen` on npm) and Python
  (`xalen` / `xalen-python` on PyPI) packages are **not yet published** either;
  the `npm install xalen` / `pip install xalen` / "prebuilt binaries" lines in
  the docs describe the *intended* distribution and are labelled **forthcoming /
  alpha** until those packages are actually live. Build the bindings from source
  in the meantime (`maturin develop`, `wasm-pack build`, `napi build`).

## [0.5.0] - 2026-06-01

### Added
- **Fixed-star catalog expanded to 8,870 stars** generated directly from the Hipparcos
  Main Catalogue (CDS I/239, Vmag ≤ 6.5): J2000 ecliptic positions, magnitudes, and
  proper motions, with traditional names joined via the IAU Catalog of Star Names (WGSN).
- **Eclipse local circumstances** (per-observer C1–C4 contacts, magnitude, obscuration)
  layered on the global Besselian engine.
- **Swiss-grade rise/set/transit + twilight**, validated against a committed pyswisseph oracle.
- **I-Ching judgment/image text** now bundled from **James Legge, *The Yî King*, Sacred
  Books of the East Vol. XVI (1882)** — public domain.

### Changed
- **True Chitrapaksha ayanamsa is now a full dynamic apparent-place reduction** (Spica J2000
  catalog + proper motion + IAU 2006 precession + IAU 2000B nutation + annual aberration),
  replacing the prior linear approximation. Cross-validated vs Swiss `SE_SIDM_TRUE_CITRA`
  (pyswisseph): **0.015″ at the J2000 anchor, ≤ 1.6″ across 1900–2100**.
- **Spica catalog entry corrected** — ecliptic longitude to full precision (203.841355°) and
  proper-motion components rotated into the ecliptic frame (were mislabeled equatorial).
- Audit hardening across the workspace: model-specific ΔT σ (Zero→0, non-SMH→unavailable),
  UTC≈UT1 documented, Julian-cutover gap-dates rejected, DE440 crafted-kernel guards + per-body
  Moon accuracy tier, topocentric true-obliquity + apparent sidereal time, event-scan range
  clamp, horary future-perfection VOC, D9 fire-sign navamsa from Aries, binding NaN guards +
  FFI `catch_unwind`, honest naming/renames, and licensing/attribution fixes
  (ERFA BSD-3-Clause notice, Meeus→ELP-2000/82B + Goffin primary attribution).

### Notes
- Sabian symbol text remains intentionally unbundled (no clean public-domain source); the
  degree/sign/decanate classification API is unchanged and functional.
- `cargo test --workspace`: 2,018 tests pass.

## [0.4.3] - 2026-06-01

### Changed
- **Licensing hygiene — all bundled interpretive text is now public-domain-sourced or
  original.** Removed third-party interpretive text whose wording derived from
  in-copyright works: Sabian symbol images + keynotes, midpoint interpretations,
  I-Ching judgment/image renderings, Nadi predictive lines, and numerology + Lal Kitab
  descriptions. The computational logic, degree/sign classifications, names, formulas,
  and structural data are **unchanged** — only the prose payload was affected.
- Removed non-primary citation URLs (blogs/wikis) and unverified chapter attributions
  from doc-comments; retained primary/classical references (Ptolemy, Valens, Lilly,
  Hipparcos, SIMBAD, Meeus, JPL).

### Notes
- Affected interpretive fields now expose empty strings until the public-domain /
  original backfill lands (Legge 1882 I-Ching, Charubel 1898 degree symbols planned).
  All engines, lookups, and tests remain functional — workspace builds green, 1960 tests pass.

## [0.4.2] - 2026-06-01

### Changed
- **Precession**: the SOFA-validated IAU 2006/P03 rotation matrix is now wired into
  the production position pipeline (VSOP / Sun / Pluto / Chiron / asteroids / DE440) —
  latitude is precessed (was frozen), radius preserved; replaces the scalar
  longitude-only shift.
- **README accuracy figures rewritten to MEASURED values** (vs JPL DE440, 20k charts,
  reproducible via `cargo test -p xalen-ephem --test accuracy_vs_de440`): Sun 0.21",
  Mercury–Saturn ≤0.76", Uranus/Neptune 1.78"/2.53", Moon ~31" analytical (sub-arcsec
  with DE440 kernel), Pluto ~3.2" in-window. Removed the overstated "~2" Moon",
  "~15" Pluto", "most comprehensive", and the false "DE440 light-time not iterated"
  note (a 2-pass light-time loop exists).

### Added
- Reproducible in-repo accuracy harness `accuracy_vs_de440.rs` (analytical engine vs
  the bundled NASA DE440 reader; no Swiss dependency; SKIPs cleanly without a kernel).
- `AccuracyTier` surfacing with-kernel (DE440) vs analytical-fallback provenance.

### Fixed
- DE440 load failures are now loud (return `EphemerisError` + log) instead of a silent
  VSOP fallback while appearing DE440-capable.
- Pluto resolves full-range (1550–2650) via DE440 when a kernel is loaded.
- `compat.rs` is a true Swiss drop-in: real ARMC (was 0.0), `SEFLG_SPEED` wired,
  lat/lon validation, fixed-star Swiss `xx` layout + `swe_fixstar_mag`.
- Morinus house cusps: obliquity inversion corrected (tan λ = tan A · cos ε).
- DE440 parser hardening: NaN `intlen` guard, negative-i32 IPT, checked arithmetic.
- FFI version string 0.1.0 → 0.4.2; ayanamsa doc count 17 → 50.

## [0.4.1] - 2026-06-01

### Added
- `JdUT1::to_tt_with_sigma` / `Epoch::jd_tt_with_sigma`: surface the ΔT 1σ
  uncertainty envelope (seconds) via an additive parallel API. Existing
  `to_tt` / `JdTT` and the FFI ABI are unchanged. The value is the
  time-conversion (TT−UT1) envelope, NOT a position or output covariance.
- Pluto is served from a loaded DE440 kernel across its full 1550–2650 span:
  the almanac now falls through `EpochOutOfRange` to the next provider, so
  Pluto requests outside the analytic 1885–2099 window resolve from DE440 when
  a kernel is present (the final error is still surfaced if all providers miss).

### Changed
- DE440 cross-validation asserts an independent JPL Horizons geometric state
  vector (Sun/SSB) instead of a parser self-comparison.

## [0.4.0] - 2026-06-01

### Fixed (pre-public honesty audit — 42-cell adversarial sweep, 9 confirmed)
- **Moon docstring overclaim** (`moon.rs` `geocentric_moon`): "~2″ longitude / ~1″
  latitude" → the honest Meeus-truncation bound (~10″ long / ~4″ lat; ~2-3″ near
  J2000), matching the corrected module header.
- **Pullen Sinusoidal Delta / Ratio house docs** (`cusps.rs`): the docstrings
  claimed the exact Swiss Ephemeris 'L'/'Q' algorithms with formulas that did not
  match (and in the SD case were mathematically broken). Relabelled honestly as
  sinusoidal APPROXIMATIONS (not bit-for-bit Swiss, not validated against Swiss
  reference cusps), with the docstring formulas corrected to match the code.
- **Ayanamsa system count** (`xalen-ayanamsa` README + lib.rs): "47 named" → "50
  named" (the 47 Swiss Ephemeris predefined IDs 0–46 plus 3 additional variants).
- **Western fixed-star count** (`xalen-western/lib.rs`): "110 named stars" → "506".
- **Fixed-star precession scope** (`xalen-stars` README): clarified that linear
  precession is applied to ecliptic LONGITUDE only; latitude changes solely via
  proper motion.
- **Fagan-Bradley test comment** (`xalen-western/sidereal.rs`): 24.042 is the
  B1950 value, not the J2000 ayanamsa (~24.74).

### Added
- **F6 — ΔT validation against the raw lunar-occultation record.** New
  `xalen-time` cross-validation test confirming the SMH2016 ΔT spline is the
  UNBIASED fit to the actual Earth-rotation observations (not just the paper's
  published spline values): over the full SMH2016 figshare dataset of **120,908**
  reduced lunar-occultation ΔT observations (1621–2015) the spline gives a
  weighted-mean residual of **+0.000 s** and weighted RMS **1.564 s** (0.78 s in
  the 2000s). Committed as a regression test over a faithful 186-point
  proportional decimation (the 13 MB dataset cannot be vendored).
- **F4 — DE440 external validation runs in CI.** New `de440-validation` CI job
  downloads the real NASA/NAIF DE440s SPK and runs `de440_real_crossval`, so the
  DE440 reader + apparent-place reduction are validated against the actual kernel
  on every push (previously these tests were kernel-gated and always skipped in
  CI). DE441 long-span (AD 400–1800) noted as a future cached-kernel step (~3 GB,
  impractical per-run).

### Fixed
- **DE440 Moon cross-validation had a wrong reference.** `de440_real_crossval`'s
  Moon-at-J2000 test asserted against a stale 218.601° value; corrected to the
  independent JPL Horizons apparent longitude 223.3238° (the value
  `swiss_eph_crossval::jpl_moon_j2000` validates). The bug was invisible because
  the test was kernel-gated and never ran. DE440 now matches JPL to ~11"; the
  comment no longer overclaims sub-arcsecond and honestly notes the DE440 lunar
  apparent-place reduction is marginally behind the VSOP path at this epoch.
- **Besselian eclipse classifier — external-review hardening.** Signed γ
  (`hypot(x,y).copysign(y)`; was unsigned — verified +0.4382 for the northern
  2017 path, −0.3935 for the southern 2023 Ningaloo path). Total/annular/hybrid
  now judged at the greatest-eclipse SUB-POINT (`L2_sub = l2 − √(1−γ²)·tan f2`),
  not the geocentric l2 sign — this correctly classifies the 2023-04-20 **hybrid**
  eclipse (new regression test) that the geocentric criterion mislabelled annular.
  Penumbral grazing limit is now event-specific (|γ| < 1 + l1) instead of a fixed
  1.55. `find_solar_eclipses` now reports the Besselian **greatest-eclipse** time
  (not the New Moon). Magnitude documented as approximate (not a Besselian local
  magnitude); the γ-residual note softened to "consistent with ephemeris/reduction
  residuals."

### Added
- **Besselian-elements solar-eclipse engine** (`xalen-ephem::besselian`): the
  rigorous Bessel/Chauvenet geocentric shadow geometry (Explanatory Supplement
  §11 / Meeus Ch. 54) — `besselian_elements` (x, y, d, l1, l2, f1, f2 from the
  Almanac's apparent Sun/Moon) and `classify_solar_eclipse` (γ = min shadow-axis
  distance, global type total/annular/hybrid/partial, greatest-eclipse instant).
  **NASA-validated**: 2017-08-21 (γ 0.4367) and 2024-04-08 (γ 0.3431) reproduced
  to ~0.002 Earth-radii with correct type + greatest-eclipse time within 30 s;
  cone angles tan f1/f2 match NASA to <0.05%. The γ residual (~0.002) is the
  truncated-ELP Moon position (~10"), not the method — documented, not hidden.
  This is the real geocentric global-classification engine; per-observer local
  circumstances (μ + ξ/η/ζ → magnitude, C1–C4 contact times) are a separate
  layer.

### Changed
- **Solar-eclipse classification now uses the Besselian engine.**
  `eclipse::find_solar_eclipses` no longer classifies by a `|Moon ecliptic
  latitude|`-vs-threshold heuristic; the `|latitude|` test is now only a cheap
  candidate pre-filter, and `besselian::classify_solar_eclipse` determines γ, the
  global type (total/annular/hybrid/partial) and confirms whether an eclipse
  actually occurs. `SolarEclipse` gains a `gamma` field and `SolarEclipseType`
  gains a `Hybrid` variant. Lunar eclipses still use the latitude-threshold
  classifier (a Besselian lunar treatment is not yet implemented).
- **Genuine SOFA-validated IAU 2006/P03 precession + ICRS frame-bias rotation**
  in `xalen-coords::precession`: `fw06_angles` (Fukushima–Williams γ̄/φ̄/ψ̄/εₐ,
  coefficients verbatim from ERFA `pfw06.c`), `fw2m`, `precession_bias_matrix_iau2006`
  (`pmat06`, GCRS→mean-of-date incl. frame bias), and `frame_bias_matrix` (`bp00`).
  Validated element-wise to **1e-12** against the ERFA/SOFA `t_erfa_c.c` golden
  vectors (`pfw06`, `pmat06`, `bp00`). NOTE: an externally-sourced "golden" set of
  `pfw06` angle values turned out to be incorrect — the discrepancy was caught by
  the end-to-end `pmat06` matrix golden (which the code matches) and resolved by
  verifying the source coefficients directly against ERFA `pfw06.c`. The matrix is
  not yet wired into the position pipeline (which uses general-precession-in-
  longitude; measured latitude-coupling residual 0″ on the ecliptic, ≤4″ Moon,
  ≤14″ Pluto at ±1 century — below each body's series-truncation error). Docs
  (CREDITS.md, ACCURACY.md) updated.
- VSOP87 primary-source cross-validation against the official IMCCE `vsop87.chk`
  check file (`xalen-ephem/tests/vsop87_official_crossval.rs`): inner planets
  reproduce the source theory to ~1e-9 AU (meters), all bodies within 3e-6 AU.
- `Vsop87Provider::heliocentric_rectangular_j2000` — public accessor for raw
  VSOP87A heliocentric rectangular ecliptic-J2000 coordinates.
- IAU SOFA reference cross-validation of the reduction chain
  (`xalen-coords/tests/sofa_reference_crossval.rs`): IAU 2006 mean obliquity
  matches SOFA `iauObl06` to machine precision; IAU 2000B nutation within
  0.12 mas of SOFA `iauNut00b`.

### Changed
- **ΔT — genuine SMH2016 cubic spline (was an approximation).** The
  `StephensonMorrisonHohenkerk2016` model now evaluates the actual published
  Stephenson–Morrison–Hohenkerk (2016) Table-S15 cubic regression spline
  (coefficients verbatim) over [−720, AD 2016], plus the model's own long-term
  lod-integral extrapolation tail (continuity constants c1/c2 re-derived for the
  2016 endpoint) outside that range — replacing the previous parabola + Espenak-
  Meeus-segment hybrid that only *approximated* the SMH range. Reproduces the
  paper's published ΔT values (0.02 s at J2000, tracks IERS to <0.25 s in the
  telescopic era). σ now reproduces the published NAO/SMH scalar envelope as a
  left-continuous step lookup (≈180 s at −720, ≈15 s at AD 1000; NAO quadratic
  tails outside [−2000, 2500]) within the fitted era, and `max(envelope, Huber
  random-walk)` past the 2016 knot — so it never understates; **no** coefficient
  covariance is claimed (none is published). Docs (CREDITS.md, ACCURACY.md)
  corrected accordingly.
- Statistical Swiss-Ephemeris cross-validation scaled to **5,000,000 charts**
  (0 of 5,000,000 over 0.1° for any planet or node; ascendant/cusp p99 < 0.013°,
  |lat| ≤ 66°; worst-case ascendant 0.172° sits at the polar Placidus boundary).
- Docs (audit-driven honesty corrections): fixed the README Western example
  (real function/module names), the fixed-star precession note (linear
  50.28796″/yr, the IAU 2006 J2000 rate — not the full IAU 2006 polynomial), and
  the house-systems cross-validation claim
  (Placidus validated vs Swiss at scale; per-system tables pending).

### Fixed
- **P1 correctness/safety batch across 11 crates** (external SME audit; each verified by `cargo test`):
  - **KP**: full 249-division sub-lord table (was capped at 243); `planet_to_dasha_lord` returns `Option` instead of silently mapping outer planets to Sun.
  - **Shadbala**: Drik Bala now applies the BPHS `/4` normalization before summing.
  - **Divisional**: D4 computed as classical Chaturthamsa (kendra offsets) instead of generic sequential division.
  - **Dosha**: Mangal cancellation evaluated by nakshatra, not a rashi proxy (removes false cancellations).
  - **Panchang**: `Tithi`/`Yoga` reject `0`/out-of-range instead of underflow-panicking in `name()`.
  - **Houses**: Carter Poli-Equatorial uses RA(Ascendant) (was East Point); `is_polar_fallback` no longer falsely flags Regiomontanus/Campanus/Krusinski-Pisa; added `GeoLocation::try_new` rejecting NaN/out-of-range (additive).
  - **DE440 reader**: guarded negative word-address overflow, summary-chain cycle (infinite-loop/DoS), last-in-file segment priority, and VSOP fallback for `geocentric_ecliptic`.
  - **Coords/Time**: zero-vector latitude guard (`0/0` NaN); SMH2016 ΔT continuity at the 500 CE boundary (dropped t⁵/t⁶ terms restored, ~146 s step removed); future ΔT uncertainty grows past the observed era.
  - **Chinese**: QiMen day-stem calendar fix; Zi Wei Zuo Fu/You Bi made month-only (were hour-dependent). **Saju**: `month=0` underflow guard.
  - **Fixed stars**: precession constant corrected to the IAU value (50.28796″/yr). **WASM/Python/FFI**: rashi-from-nakshatra start-sign fix, error propagation instead of silent empty/`Sun`, NaN/Inf input guards.
- **SVG chart rendering** hardened against malformed input (external SME audit):
  `south_indian` `sign_index_for_house` clamped (house `0`/`>12` underflowed
  `usize`); NaN/Inf longitudes, cusps and ayanamsa are now run through
  `safe_longitude` in all three renderers so no `NaN`/`inf` token can leak into
  the SVG (regression `xalen-chart/tests/robustness_regression.rs`).
- **House Vertex** was computed with the negated latitude instead of the
  co-latitude (90° − φ); the previous value could be tens of degrees off. Now
  matches Swiss Ephemeris `ascmc[3]` to < 0.01° (regression test
  `xalen-houses/tests/vertex_swiss_regression.rs`). Independently confirmed by a
  an independent external SME review.
- Honesty: the README now marks the 7 specialised house systems (Gauquelin,
  Sunshine, Pullen, Carter Poli-Equatorial, APC, Zariel, Alcabitius Classic) as
  **experimental / not yet Swiss-validated**, and drops the blanket "automatic
  Porphyry fallback at polar latitudes" claim (only Placidus's fallback is
  verified).
- **P1 refinements (an independent external SME review follow-up; each verified by `cargo test`
  + the cited reference):**
  - **ΔT future uncertainty**: `delta_t_with_uncertainty` beyond the last
    observed year now uses Espenak's published Huber Brownian-motion model
    (calibration year +2005, ≈47.9 s at 2100) instead of the historical
    Morrison–Stephenson parabola (≈6.3 s at 2100), which understated the genuine
    future uncertainty ~8×. The 500–1000 CE polynomial comment is corrected to
    its true Espenak/Meeus attribution (it is not the SMH2016 spline). Test pins
    the Huber values at 2050/2100. Ref: EclipseWise "Uncertainty in ΔT".
  - **Carter Poli-Equatorial polar correctness**: ports the Swiss `swehouse.c`
    `case 'F'` AC/DC swap — inside the polar circle (signed AC−MC < 0) the anchor
    becomes AC+180° before deriving its right ascension (regression test at
    75°N). `needs_latitude()` corrected: Equal and Carter are anchored on the
    Ascendant, so they (and Whole Sign, Vehlow) require latitude; only Morinus,
    Meridian and Zariel are latitude-independent.
  - **DE440 provenance honesty**: the engine name and the ~1″ accuracy figure are
    now gated on the DAF comment area actually confirming `DE440` provenance, not
    on "any SPK parsed". A synthetic or non-DE kernel reports a generic
    "JPL SPK kernel" label and the analytical fallback accuracy. New
    `De440Reader::kernel_id()` accessor; `is_de440_loaded()` means *confirmed*
    DE440. The comment scanner was verified against the real `de440.bsp` /
    `de440s.bsp` bytes (modern "DE440" text + legacy "DE-0440LE-0440" token) and
    hardened with a left word-boundary check and full zero-pad consumption, so
    "NODE440" is rejected and "DE-00440" normalizes to "DE440" rather than
    truncating.
  - **DE440 corruption handling**: a structurally-corrupt kernel (summary-chain
    cycle, non-positive/out-of-range word address, invalid Type 2 directory) is
    now rejected wholesale as `InvalidFormat` — never silently partial-loaded —
    and `n·rsize·8` size math is fully `checked_*`. `with_de440` turns the
    rejection into a clean VSOP87 fallback. The `geocentric`/`heliocentric`
    fallback is narrowed to genuine coverage gaps (`EpochOutOfRange`); a real
    computation failure now surfaces instead of silently degrading to VSOP87.
  - **Docs**: the `Vsop87Provider` rustdoc now states the physical-body
    worst-case figure (Moon) instead of the optimistic planetary-only ~1″.
  - **Saju (Korean)**: out-of-range `month`/`day`/`hour` are now clamped ONCE at
    the `compute_saju` entry, so the month pillar and day pillar always use the
    same month. Previously `month_pillar` clamped internally while `day_pillar`
    received the raw month, desyncing the two pillars for an invalid month.
    Regression test asserts the day pillar (not just the month pillar) matches.
  - **Dosha**: documented `detect_mangal_dosha`'s 4th argument as the 0-based
    nakshatra index (0..=26), NOT a rashi index, to prevent the silent
    positional-argument mismatch flagged in review.
  - **Fixed stars**: the IAU 2006 rate (50.28796″/yr) the P1 batch CHANGELOG
    claimed was never committed (commit `0673a21` had no xalen-stars diff); the
    precession constant is now genuinely corrected from Newcomb's 1900-epoch
    50.2564″/yr in `lib.rs` + `catalog.rs`, with README/ACCURACY reconciled.
  - **Coords**: `cartesian_to_ecliptic` now propagates a NaN latitude for a
    non-finite input vector instead of silently reporting latitude 0 (only an
    exact null vector returns 0).
  - **FFI**: `xalen_ayanamsa()` now rejects non-finite `jd_ut1` with the -1.0
    sentinel, matching the other FFI entrypoints.
  - **Python (PyO3)**: the `extension-module` feature is now optional (default
    off) with `rlib` added and `doctest = false`, so `cargo test -p xalen-python`
    links libpython and runs the suite (previously a hard link failure). A
    `pyproject.toml` (`[tool.maturin] features = ["extension-module"]`) keeps the
    published wheel correct. Added GIL-based tests for `all_planets`/`full_chart`,
    and the Ketu paths now reuse the already-computed Rahu longitude instead of
    recomputing it inside a silent `if let Ok(...)` that would drop Ketu on error.
  - **Accuracy honesty (2nd external SME review):** `Vsop87Provider::accuracy_arcsec`
    corrected from **20″ to 75″** — 20″ understated the worst physical body (the
    analytical Moon hits 74″ in the committed 5M-chart Swiss validation), i.e. it
    overclaimed accuracy. The DE440 provider's figure stays **1″**, now justified
    by adding the body light-time correction (3rd-review item below) so the
    apparent place is computed in full. Both figures are now explicitly scoped to
    physical bodies; the
    lunar nodes (mean ~19″, true ~111″ vs Swiss) are documented as a node-
    algorithm difference, not folded into the figure. Test now asserts the figure
    bounds the 5M Moon max (74″), not just the single-epoch 17″.
  - **DE440 corruption (2nd external SME review):** two remaining silent partial-load
    `break`s now return `InvalidFormat` — a NEXT pointer past end-of-file and an
    NSUM that overruns the record/file. Regression tests added for both.
  - **Carter polar (doc):** documented that `HouseCusps.ascendant` stays the
    astronomically-correct rising degree while `cusp[0]` carries the Swiss
    `case 'F'` AC/DC swap — a deliberate divergence from Swiss overwriting
    `ascmc[0]` for Carter (the rising degree must not depend on house system).
  - **Chinese (Zi Wei) silent input:** `compute_chart` now clamps
    `lunar_month`/`lunar_day` ONCE at the entry (documented; out-of-range →
    panic-free clamped chart, no internal desync), with an invalid-month
    regression test. (`panchang` Tithi/Yoga/Karana name clamps were already
    documented + exhaustively tested over the full `u8` range; `qimen`'s
    `gregorian_to_jd` already documents its lenient out-of-range behaviour.)
  - **WASM:** `vimshottariDasha` now rejects non-finite `moon_deg`/`birth_jd`
    with an `Err` (matching the other entrypoints, which already guard). Fixed
    the README's stale `compatibility(boy_nak, girl_nak)` signature to the
    current `compatibility(boy_moon_deg, girl_moon_deg)` degrees contract
    (the code already takes sidereal Moon longitudes and resolves both nakshatra
    and rashi from them). Verified `cargo check --target wasm32-unknown-unknown`.
  - **Shadbala Drik Bala (honesty + citation):** clarified in the rustdoc that
    the Drik Bala is a graded-aspect model — the special aspects (Mars 4th/8th,
    Jupiter 5th/9th, Saturn 3rd/10th, all 7th = full) and graded virupas
    (¼/½/¾/full) plus the BPHS `/4` normalization are classically correct and
    TESTED — but it is NOT the continuous, degree-interpolated BPHS Ch.27-28
    Sphuta Drishti curve. The exact Sphuta Drishti is tracked as a future
    enhancement gated on the canonical Santhanam Ch.27-28 formula and a
    verification oracle; no classical formula is committed from secondary
    sources (Rule 22 / no-overclaim).
  - **DE440 light-time (3rd external SME review):** the DE440 geocentric path now
    applies the body light-time correction (2-pass retardation: the target is
    evaluated at `t − τ`, the observer at `t`) before precession/nutation/annual
    aberration — the SAME full apparent-place chain the VSOP87 path already used.
    Previously DE440 used same-epoch geometric positions, leaving inner planets
    tens of arcseconds off (worse than the analytical path) and making the 1″
    claim an overclaim. New `De440Reader::geocentric_position_au_split`.
  - **DE440 NSUM record bound (3rd external SME review):** the summary-count check now
    enforces the NAIF per-record maximum (≤25 for SPK; summaries never cross a
    1024-byte record), so an `NSUM=26` in a long file is rejected instead of
    parsing the next record's bytes.
  - **Honesty (3rd external SME review):** `Shadbala::compute_full`/`compute` docs no
    longer say "full/BPHS-accurate" without qualification (Drik Bala is the
    graded model, all other balas per BPHS Ch.27); the `EphemerisProvider::
    accuracy_arcsec` trait doc is now scoped to physical bodies with nodes
    characterised separately.
  - **Drik Bala aspect DIRECTION (completeness-audit HIGH — real bug):** the
    Drishti Kendra separation was computed as aspecting−aspected; it must be
    aspected−aspecting (the arc counted forward FROM the aspecting graha). The
    reversed direction mirrored the asymmetric special aspects (Mars 4th/8th,
    Jupiter 5th/9th, Saturn 3rd/10th) onto the wrong planet. Fixed, and the
    `drik_bala_mars_special_aspect` test (which had encoded the bug — Mars at 90°
    is the aspected's 10th, not Mars's 4th) corrected, with a direction-guard
    assertion. (This corrects an earlier doc claim that the special aspects were
    "classically correct"; they are now, after this fix.)
  - **Panchang Karana underflow (completeness-audit HIGH):**
    `Karana::from_tithi_and_elongation` underflowed/panicked under debug on
    `Tithi { number: 0 }` (`number as u16 - 1`); now clamps before subtracting,
    matching the name-lookup guards.
  - **Dosha contract test:** `mangal_cancellation_uses_exact_nakshatra_not_rashi`
    now also pins the backward-compat hazard (a legacy caller passing a rashi
    index mis-fires), making the nakshatra-index contract explicit.

### Honesty / capability-statement relabeling

  An independent capability audit found several headline claims overstated vs.
  the code. Corrected so the docs survive a clone-and-inspect (no behaviour
  change — wording + a test only):
  - **ΔT model labeling:** the `StephensonMorrisonHohenkerk2016` variant doc and
    CREDITS.md now state plainly that it is an Espenak–Meeus 2006 + IERS-table
    HYBRID approximating the SMH2016 historical *range* — the published SMH2016
    cubic spline is NOT implemented (it is the cited reference, not the method).
  - **Eclipse engine scope:** ACCURACY.md now states the engine is a geocentric
    syzygy + |latitude|-threshold *classifier* (date + type), NOT a Besselian
    local-circumstances engine — no shadow cones, path of totality, or
    topocentric C1–C4 contact times.
  - **Precession:** ACCURACY.md clarifies the position pipeline uses the scalar
    general-precession-in-longitude shift; the full IAU 2006/P03 rotation matrix
    is provided but NOT wired into the position path and not SOFA-validated.
  - **Lahiri ayanamsa:** the "<1" at J2000" figure is documented as a
    self-consistency check against the Swiss-anchored constant (the in-code
    "genuine external cross-validation" comment was false); real agreement is
    ~2" vs Swiss, and there is NO arcsec reconciliation against the Indian
    Astronomical Ephemeris (no IAE data bundled).
  - **DE440 km cross-validation:** the test header + reference docstring now state
    the km-tolerance assertions are self-consistency (reference vectors are this
    parser's own output) and are CI-skipped (gated on a local `.bsp`); an
    external JPL-Horizons-vector assertion is the remaining step. README's DE440
    accuracy row no longer says "body light-time not iterated" (it now is).
  - **ΔT observed table:** documented that the IERS table is frozen at the 2019
    anchor though IERS publishes later values (conservative — σ grows past it).
  - **Stale-doc + test gaps (completeness audit):** coords README no longer
    claims the geometric transforms apply nutation; the FFI README `-2` status
    now notes non-finite/out-of-range numeric inputs; the FFI houses test now
    covers ±Inf longitude; a stale "no aberration" diagnostic line corrected.

## [0.3.1] - 2026-05-30

### Fixed
- **Packaging:** the umbrella `xalen-ephemeris` crate (the workspace root) was
  publishing internal development files in its `.crate` archive. Added an
  `exclude` so only library sources, public docs, examples, and standard metadata
  ship. The 0.1.0–0.3.0 umbrella versions are **yanked**; use 0.3.1+. (Leaf
  crates were never affected — they package only `src/`.)

## [0.3.0] - 2026-05-30

### Added
- **Rise / transit / set times** — `Almanac::rise_transit_set` (and
  `Almanac::topocentric_altitude_deg`). Scans topocentric altitude over 24h and
  bisects the horizon crossings, with the standard per-body rise altitude (Meeus
  Ch.15): Sun −0.833° (refraction + semidiameter), Moon parallax-corrected,
  planets/stars −0.567°. Handles circumpolar / never-rises.
- **`docs/ACCURACY.md`: "Validated Against Every Reference Standard"** — a
  consolidated cross-validation summary against JPL Horizons DE440 (including the
  real `de440s.bsp` binary kernel), Swiss Ephemeris (per-epoch + 2,000-chart
  statistical), and the public calculators (astro.com, AstroSage, Drik Panchang,
  Prokerala, Jagannatha Hora) + Meeus.

### Fixed
- `xalen-world` `nine_star_ki::month_star` no longer panics on an out-of-range
  `month` (0 underflowed the index; > 13 ran off the table) — input is now clamped
  to 1..=12.
- `xalen-wasm` `full_chart_json` no longer panics across the WASM boundary when a
  degenerate input yields a non-finite `f64`; it propagates the serialization
  error instead.

## [0.2.0] - 2026-05-30

### Added
- **Velocity / daily motion** — `EphemerisProvider::geocentric_ecliptic_speed`
  (a default trait method, so every provider gets it) and
  `Almanac::geocentric_speed`, returning `EclipticSpeed { longitude, latitude,
  distance }` with `is_retrograde()` and `*_deg_per_day()` helpers. Central
  finite difference over ±0.5 day = the body's daily motion.
- **Topocentric positions** — `Almanac::topocentric_ecliptic` applies diurnal
  parallax (Meeus Ch.40) for an observer's latitude/longitude/elevation
  (~8.8″ for the Sun, up to ~1° for the Moon).
- `README.md` for every leaf crate (crates.io landing pages).
- `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, and `CREDITS.md`
  (algorithm and data-source attribution).
- Pinned MSRV via `rust-version = "1.85"`.
- `docs/ACCURACY.md`: a large-scale statistical cross-validation section
  (2,000 deterministic charts vs Swiss Ephemeris, per-body max/p99/rms).

### Changed
- Removed an unused `xalen-vedic` dependency from `xalen-ffi`.

### Notes
- Apparent positions, houses, ayanamsa, and the Vedic/Western/Chinese/world
  surfaces are unchanged from 0.1.0; this release is additive.

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
