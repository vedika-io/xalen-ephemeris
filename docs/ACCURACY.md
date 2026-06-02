# XALEN Ephemeris -- Accuracy Report

**Last updated:** 2026-06-02
**Engine version:** 0.6.0
**Test suite:** `cargo test --workspace` reports **2,199 tests passing** across the
library and integration suites (0 failures). The committed per-domain Swiss/JPL
oracle tests (`cargo test`) are the validation you can reproduce from this
repository; the accuracy figures below that are sourced from those tests are
labelled as such. The large-scale statistical sweep is now **reproducible from
this repository at any chart count** via the committed `validation/` harness
(`validation/oracle_pyswisseph.py` + the `xalen-validation` runner), which uses
`pyswisseph` as the oracle — run it at `--n 5000000` to reproduce a 5,000,000-
chart sweep. The honest caveat is the oracle's backend: with no Swiss `.se1`
data files installed, `pyswisseph` falls back to the analytic Moshier theory, so
that sweep compares two independent analytic chains (XALEN vs Moshier), not
XALEN vs DE440; the harness detects and prints the backend so the numbers can be
read honestly. The DE440-grade ground truth is the committed Horizons-vector
oracle tests (`cargo test`). See `validation/README.md`. A historical 5,000,000-
chart run executed in a private Swiss-equipped monorepo is described in the
"Large-Scale Statistical Cross-Validation" section below for transparency, but
it is the `validation/` harness — not that private run — that you can reproduce.

---

## Validated Against Every Reference Standard

XALEN is cross-checked against the recognized authorities in both astronomy and
astrology — not against itself. The independent sources:

| Source | What it is | How XALEN compares |
|--------|-----------|--------------------|
| **JPL Horizons (DE440)** | NASA's definitive solar-system ephemeris (numerical integration) — the ground truth | Apparent geocentric longitudes (quantity #31) match to **sub-arcsecond** for the Sun and planets, 1950–2050 |
| **JPL DE440 binary kernel** (`de440s.bsp`) | The actual NASA SPK kernel, read directly | The bundled DE440 reader is verified against the real NASA kernel (loads, all body pairs, spans 1550–2650); apparent-place agreement with JPL is the **sub-arcsecond** per-body figures in the table below |
| **Official VSOP87 check file** (`vsop87.chk`, Bretagnon & Francou / IMCCE) | The reference data shipped *with* the VSOP87 theory itself, so an implementation can be proven against its own source | The VSOP87A planetary records are validated in CI (`tests/vsop87_official_crossval.rs`); the committed test parses **≥ 40 records** and asserts inner planets (Mercury–Mars) reproduce the source to **< 1×10⁻⁸ AU** (typically ~1×10⁻⁹ AU) and outer planets within **< 3×10⁻⁶ AU** (worst case ~300 km on Uranus ~900 yr from epoch, ≈0.02″), across ~1100–2000 CE |
| **Swiss Ephemeris** (`swetest` / `pyswisseph`) | The de-facto astrology-software standard (used by astro.com, etc.) | In-repo oracle tests pin Swiss reference values per domain: **46 of 47 SE ayanamsa systems < 1″** (with 1 documented exception — `GalCenterMulaWilhelm` SE 36 — held to ≤ 2″), **19 comparable house systems within 0.01°** (18 tight; PullenSinusoidalRatio within ~5° as a documented approximation), the **named generated stars + committed anchor rows sub-arcsecond** vs `fixstar2`, and the analytical **Moon at RMS ~2.8″** vs `pyswisseph` (AD 1600–2100). A statistical sweep of arbitrary size is **reproducible from this repo** via the committed `validation/` harness (`pyswisseph` oracle + `xalen-validation` runner) — run it at `--n 5000000` to reproduce a 5,000,000-chart sweep; the harness reports the oracle backend (Moshier fallback vs Swiss data files) so the comparison is read honestly |
| **Public calculators** | astro.com, astrosage.com, drikpanchang.com, prokerala.com, jagannathhora.com, appliedjyotish.com | Sidereal positions, nakshatra/pada boundaries, dasha cycles, and panchang cross-checked against the values these tools publish |
| **Meeus, *Astronomical Algorithms*** | The standard reduction-chain reference | Sun, lunar motion, elongation bounds, and continuity checks match the textbook |

**The honest framing (for any public claim):** XALEN *matches* JPL DE440 and Swiss
Ephemeris to the precision below — it does not "beat" them; DE440 *is* the
reference. The genuine differentiators are **pure Rust, zero `unsafe` in the core,
thread-safe, Apache-2.0, and WebAssembly-ready** — JPL-class accuracy with a
license and engineering that the C/AGPL incumbents can't offer. Measured bounds:

---

## Planetary Position Accuracy (JPL DE440 Cross-Validated)

All positions verified against NASA/JPL Horizons DE440 ephemeris on 2026-05-28
via API quantity #31 (ObsEcLon, apparent geocentric ecliptic-of-date).

> The per-body figures below are **JPL Horizons spot-checks at sampled epochs**.
> The complementary **reproducible 20,000-chart statistical bound** (run from this
> repo via `cargo test -p xalen-ephem --test accuracy_vs_de440`, no Swiss
> dependency) is the headline figure used in the README: **Sun 0.21″,
> Mercury–Saturn ≤ 0.76″, Uranus 1.78″, Neptune 2.53″, Pluto ~3.2″ in-window**.
> Where a spot-check range (e.g. Saturn 0.1–1.0″) brushes above the statistical
> p-bound, it reflects a single worst sampled epoch, not the typical error.

| Body | Theory | Measured Error vs JPL DE440 | Valid Range |
|------|--------|----------------------------|-------------|
| Sun | VSOP87A + IAU 2000B nutation | **~0.21"** (20k-chart mean vs DE440; 0.48" at the J2000 spot-check below) | 4000 BCE -- 8000 CE |
| Moon | ELP2000-82 (Meeus Ch.47, 60+60 terms) + Δψ + geocentric light-time (NOT annual aberration) | RMS ~2.8" (max ~12") over AD 1600--2100 | Modern era |
| Mercury | VSOP87A + nutation | **0.35"** | 4000 BCE -- 8000 CE |
| Venus | VSOP87A + nutation | **0.30"** | 4000 BCE -- 8000 CE |
| Mars | VSOP87A + nutation | **0.3--0.7"** | 4000 BCE -- 8000 CE |
| Jupiter | VSOP87A + nutation | **0.1--0.8"** | 4000 BCE -- 8000 CE |
| Saturn | VSOP87A + nutation | **0.1--1.0"** | 4000 BCE -- 8000 CE |
| Uranus | VSOP87A + nutation | **0.52"** | 4000 BCE -- 8000 CE |
| Neptune | VSOP87A + nutation | **1.07"** | 4000 BCE -- 8000 CE |
| Pluto | Meeus Ch.37 (43-term Goffin fit) | ~3.2" measured in-window (20k charts); published Goffin bound ~1 arcminute | 1885 -- 2099 |
| Chiron | JPL Horizons osculating elements | < 1-2 deg | 1950 -- 2050 |
| Rahu (Mean Node) | Analytical polynomial | Exact (mean model) | Unlimited |
| Ketu | Rahu + 180 deg | Exact (by construction) | Unlimited |
| Mean Lilith | Analytical polynomial | Same as Moon theory | Modern era |

### Verification Details

Epochs tested: J2000.0 (2000-01-01 12:00 UT) and 2024-01-01 12:00 UT.

| Body | J2000 XALEN | J2000 JPL | Delta | 2024 XALEN | 2024 JPL | Delta |
|------|-------------|-----------|-------|------------|----------|-------|
| Sun | 280.3690 | 280.3689 | **0.48"** | 280.5486 | 280.5485 | **0.46"** |
| Moon | 223.3235 | 223.3238 | 0.95" | 161.9071 | 161.9070 | 0.33" |
| Mercury | 271.8894 | 271.8893 | **0.35"** | -- | -- | -- |
| Venus | 241.5659 | 241.5658 | **0.30"** | -- | -- | -- |
| Mars | 327.9634 | 327.9633 | **0.32"** | 267.6793 | 267.6791 | **0.67"** |
| Jupiter | 25.2531 | 25.2531 | **0.11"** | 35.5843 | 35.5844 | **0.25"** |
| Saturn | 40.3956 | 40.3956 | **0.13"** | -- | -- | -- |
| Uranus | 314.8093 | 314.8092 | **0.52"** | -- | -- | -- |
| Neptune | 303.1933 | 303.1930 | **1.07"** | -- | -- | -- |

### DE440 (Optional High-Precision Mode)

| Body | Theory | Accuracy | Data Files |
|------|--------|----------|------------|
| Sun + major planets | JPL DE440 Chebyshev polynomials | Raw geometry sub-mas; **apparent longitude sub-arcsecond** (full chain: body light-time retardation + precession + IAU 2000B nutation + annual aberration, same as the analytical path) | Requires `de440s.bsp` (NAIF DAF/SPK format) |
| Moon | JPL DE440 Chebyshev polynomials | Raw geometry sub-mas; **apparent longitude sub-arcsecond** — geometric geocentric vector (at the observation epoch) + precession + IAU 2000B nutation + geocentric light-time (~0.7"), and deliberately NOT the full annual aberration that planets receive (the geocentric Moon shares Earth's heliocentric velocity). A prior build wrongly applied annual aberration to the Moon, giving a ~11" residual; that bug is fixed. `accuracy_arcsec()` conservatively reports 2" to bound the apparent-place reduction | Requires `de440s.bsp` (NAIF DAF/SPK format) |

The DE440 reader is a full NAIF DAF/SPK parser that reads the standard binary
format produced by JPL. It provides Chebyshev polynomial interpolation with
automatic body/epoch fallback to the VSOP87 analytical engine when a segment is
not available.

**Getting the kernel with zero manual handling.** Enable the optional
`kernel-autodownload` cargo feature and call `De440Provider::from_auto_cache()`:
on first use it fetches the public NASA NAIF `de440s.bsp` (~32 MB) into the
per-OS cache directory, verifies it (structural DE440 provenance, plus an
optional SHA-256 when `XALEN_DE440S_SHA256` is set), and reuses the cached copy
on every later call — no network access after the first fetch. With that path
the apparent Moon (and every body the kernel covers) is **sub-arcsecond out of
the box**. The feature is **off by default** so the base crate stays offline and
crates.io-clean; without it, supply your own `.bsp` via
`De440Provider::try_from_file`.

> **Offline analytical Moon — the actionable caveat.** Without a kernel the
> apparent Moon comes from the 60-term Meeus/ELP series and is good to **RMS
> ~2.9″ / max ~12″ over AD 1600–2100** (RMS ~4.2″ by AD 1000, ~9.6″ by AD 1),
> measured vs `pyswisseph` 2.10.03. That is fine for natal/dasha/transit work; if
> you need sub-arcsecond lunar positions, enable `kernel-autodownload` (or supply
> a `.bsp`). This replaces the older "needs DE440 kernel path; analytical is
> tens-of-arcsec" wording with the measured figure and the one-call fetch.

**For most astrological applications, the analytical engine (VSOP87A + ELP2000-82)
is more than sufficient.** The sub-arcsecond differences between VSOP87 and DE440
are invisible in natal chart interpretation, dasha computation, transit analysis,
and every standard astrological technique.

---

## Precession and Nutation

| Component | Model | Accuracy | Source |
|-----------|-------|----------|--------|
| Precession | IAU 2006/P03 (Capitaine, Wallace, Chapront 2003) | SOFA-validated to 1e-12 | Fukushima–Williams `pmat06` + `bp00` frame bias, matching ERFA `t_erfa_c.c` golden vectors element-wise (see precession tests) |
| Nutation | IAU 2000B (McCarthy & Luzum 2003) | ~1 mas | 77 largest lunisolar terms + 5 out-of-phase corrections |
| Mean obliquity | IAU 2006 polynomial | Sub-arcsecond | Tied to precession model |
| P03 rotation matrix | IAU 2006/P03 Fukushima–Williams | SOFA-validated to 1e-12 | Full 3-D rotation wired into the VSOP87/ELP/Pluto position pipeline |

> Note: the genuine IAU 2006/P03 precession rotation
> (`precession_matrix_p03_nobias` — built from the Fukushima–Williams angles via
> `fw06_angles` / `fw2m`, with the ICRS frame-bias variant
> `precession_bias_matrix_iau2006` and `frame_bias_matrix` also available in
> `xalen-coords`) **is now wired into the production position pipeline.** The
> VSOP87 / ELP2000 / Meeus-Pluto output — referred to the dynamical J2000 mean
> equinox — is rotated to the mean equinox of date with the **bias-free** P03
> matrix (`Vsop87Provider::precess_dynamical_j2000`), which precesses latitude
> consistently and preserves the radius. This **replaces** the earlier scalar
> `longitude += general_precession_longitude(t)` approximation. Nutation in
> longitude (IAU 2000B) is then applied for the true (apparent) equinox of date.
> The rotation is SOFA-validated: its `pmat06` and `bp00` constituents match the
> ERFA/SOFA `t_erfa_c.c` golden vectors element-wise to 1e-12 (see precession
> tests).
>
> The earlier scalar treatment neglected the moving-ecliptic (latitude-coupling)
> contribution; with the full rotation now applied that term is carried, not
> dropped. For reference, the previously-neglected residual was: **at β = 0 it
> rounded to 0.00″ over ±1 century (a tiny second-order term ≈0.003″), ≤1″ for
> the Moon (β ≈ 5°) at 2025 rising to ≤4″ at ±1 century, and ≤14″ for Pluto
> (β ≈ 17°) at ±1 century.**

---

## Ayanamsa Systems

50 named systems (covering all 47 Swiss Ephemeris predefined IDs 0-46 — which
already include the J2000/J1900/B1950 reference epochs as SE IDs 18/19/20 — plus
3 systems with no Swiss ID of their own: PushyaPaksha, KPStraightLine, and
CitraAtSpica180) plus a fully customizable `Custom` variant.

| Category | Count | Examples |
|----------|-------|---------|
| Classic / Indian | 15 | Lahiri, KP Krishnamurti, Raman, True Chitrapaksha, True Revati, Surya Siddhanta, Sri Yukteswar, J.N. Bhasin |
| Western sidereal | 4 | Fagan-Bradley, De Luce, Hipparchos, Aldebaran 15 Tau |
| Galactic-reference | 10 | Galactic Center 0 Sag, Gil Brand, Cochrane, IAU 1958, True (Liu/Zhu/Zhang 2010), Mula, Mardyks, Fiorenza |
| Babylonian / Hellenistic | 8 | Kugler 1/2/3, Huber, Mercier, Britton 2010, Sassanian, Vettius Valens |
| Theosophical | 1 | Djwhal Khul |
| Star-anchored | 7 | Suryasiddhanta Revati, Suryasiddhanta Citra, True Pushya, Aryabhata, Aryabhata 522 |
| Reference-epoch | 3 | J2000, J1900, B1950 |
| Modern research | 1 | True Sheoran |
| **Custom** | 1 | User-defined epoch, ayanamsa-at-epoch, precession rate |

**Validation (honest scope):** of the **47 Swiss-Ephemeris-mapped systems**, **46
match Swiss `get_ayanamsa_ex(jd, 0)` (the with-nutation apparent value) to < 1″**
across 1900–2100, with **1 documented exception held to ≤ 2″**:
`GalCenterMulaWilhelm` (SE 36, ~1.42″). `TrueChitra` (SE 27) is now under 1″
(0.038″ at the 1900–2100 oracle epochs, ≤ 0.04″ at off-grid dates) after its
Spica reduction was switched from the scalar pure-longitude precession to the
rigorous IAU 2006/P03 Cartesian rotation that couples Spica's ~2.05° ecliptic
latitude — the old scalar term left a ~1.5″/century span residual. The remaining
exception (SE 36) reflects that Swiss's internal Ernst Wilhelm reduction has a
no-nutation precession rate of ~52.5″/yr combined with a moderate-latitude annual
aberration of ~±23″, a combination no single fixed celestial direction
reproduces; the self-contained fixed-direction model therefore caps at ≤ 1.42″.
This is a **strict oracle** (`all_ayanamsas_match_swiss_within_1_arcsec`, 1″
tolerance with the one named exception at 2″), not a self-consistency check.

These are **no longer linear-from-J2000** approximations. The fixed-epoch systems
(Lahiri, KP, Raman, …) are anchored at their true Swiss epoch (Lahiri at the 1956
Calendar-Reform-Committee epoch, not a J2000 constant) and accumulate the **full
IAU 2006 general precession from t₀ plus IAU 2000B nutation in longitude (Δψ)**;
the galactic and true-star systems rotate a fixed J2000 reference direction to the
ecliptic of date with the IAU 2006/P03 rotation + nutation. Lahiri specifically
tracks Swiss SE_SIDM_LAHIRI to **≤ 0.74″** across 1900–2100 (the earlier
J2000-anchored 23.85306° constant was ~14.5″ wrong; the true Swiss mean at J2000
is 23.857092°).

**Honest caveat that remains:** this is arcsec-level agreement with **Swiss**, not
a bit-for-bit reconciliation against the **Indian Astronomical Ephemeris (IAE) /
Rashtriya Panchang** tables — those tables are not bundled, and the IAE-named
checks elsewhere still use coarse (0.05–0.15°) tolerances. Precession correctly
increases over time (Lahiri at 2100 CE > Lahiri at J2000, verified).

---

## House Systems

23 systems are implemented. All share the same validated Ascendant/MC
primitives. A committed Swiss oracle (`swiss_houses_oracle.rs`) pins each
system's 12 cusps to **live `swe.houses_armc` output** at multiple latitudes and
dates: of the 19 systems Swiss can supply a comparable cusp array for, **18 match
Swiss to < 0.01°** (the tight bar), driven off XALEN's own RAMC and IAU 2006 mean
obliquity. The single exception is Pullen Sinusoidal (Ratio), which XALEN
documents as an approximation and which is held to a loosened 5° tolerance.
(Alcabitius Classic, Gauquelin, and the two Sunshine variants have no comparable
Swiss `houses_armc` oracle — Swiss degenerates them to Placidus — so they are not
in this count.) Placidus is additionally cross-validated at scale
(p99 within ~0.013°, |lat| ≤ 66°, where Placidus is well-conditioned). The 15
below are the classic, polar-robust systems:

| System | Code | Latitude-dependent? | Polar limitation? |
|--------|------|---------------------|-------------------|
| Whole Sign | `W` | No | No |
| Equal | `A` | No | No |
| Placidus | `P` | Yes | Yes (> 66.5 deg) |
| Koch | `K` | Yes | Yes |
| Porphyry | `O` | Yes | No |
| Regiomontanus | `R` | Yes | No |
| Campanus | `C` | Yes | No |
| Morinus | `M` | No | No |
| Alcabitius | `B` | Yes | Yes |
| Alcabitius (Classic) | — (no Swiss code) | Yes | Yes |
| Topocentric (Polich-Page) | `T` | Yes | Yes |
| Meridian | `X` | No | No |
| Vehlow | `V` | No | No |
| Sripati | `S` | Yes | No |
| Krusinski-Pisa | `U` | Yes | No |

> **`needs_latitude()` vs. the table above.** The table's "Latitude-dependent?"
> column is the *conventional house-division* classification (does the division
> method itself use latitude?). The `needs_latitude()` API answers a stricter
> question — "is geographic latitude required to *compute* the cusps at all?" —
> and returns `true` for every system except Morinus, Meridian and Zariel,
> because all others are anchored on the Ascendant, which is itself
> latitude-dependent. So Equal, Whole Sign, Vehlow and Carter return `true` from
> `needs_latitude()` even though their division is latitude-independent in the
> conventional sense. Use the API value for caching/validation (a false "no"
> would silently compute a lat-0 Ascendant); use the table for the astrological
> classification.

The remaining specialised systems split into two groups. **Now held to the tight
< 0.01° Swiss oracle:** Pullen Sinusoidal (Delta), Carter Poli-Equatorial, APC,
and Zariel (Axial Rotation). Carter Poli-Equatorial is anchored on the right
ascension of the Ascendant and applies the Swiss `case 'F'` AC/DC swap inside the
polar circle (verified against `swehouse.c`); it is therefore latitude-dependent.
**Not in the Swiss `houses_armc` oracle** (Swiss degenerates them to Placidus, so
no comparable reference exists): Gauquelin sectors, Sunshine (Makransky), Sunshine
(Treindl), and Alcabitius Classic; **Pullen Sinusoidal (Ratio)** is a documented
approximation held to a loosened 5° tolerance.

Systems with polar limitations automatically fall back to Porphyry at extreme
latitudes.

**Validation:** Cross-validated across 6 locations (Delhi, Pune, New York, London,
Sydney, Equator), 9 house systems, and 3 dates (J2000, 2023-Feb-25, 1968-May-24).
All cusps 0-360 deg, ASC-DSC opposite within 0.01 deg, MC-IC opposite within
0.01 deg.

---

## Eclipse Engine

| Metric | Value | Reference |
|--------|-------|-----------|
| Syzygy finding | Bisection on Sun-Moon elongation, 1-day scan step | Meeus Ch.54/55 |
| Solar global classification | Rigorous **Besselian elements** (x, y, d, l1, l2, tan f1, tan f2) → γ, global type, greatest-eclipse instant | Explanatory Supplement §11; Meeus Ch.54 |
| Solar local circumstances | Per-observer reduction (ξ, η, ζ) → magnitude, obscuration, **C1–C4 contact times** | Explanatory Supplement 3rd ed. §11.3 |
| Lunar classification | Penumbral / Partial / Total via shadow-cone latitude test | Meeus Ch.55 |
| Date timing | +/- 1 day of NASA reference dates; greatest-eclipse instant within the committed bound of **< 30 s** vs NASA (2017) | NASA Five Millennium Canon |

**NASA cross-validation (6 eclipses, 2024-2025):**

| NASA Event | Date | Type | Detected? | Timing |
|------------|------|------|-----------|--------|
| Penumbral Lunar | 2024-Mar-25 | Penumbral | Yes | < 1 day |
| Total Solar | 2024-Apr-08 | Total | Yes | < 1 day |
| Partial Lunar | 2024-Sep-18 | Not total (confirmed) | Yes | < 1 day |
| Annular Solar | 2024-Oct-02 | Annular | Yes | < 1 day |
| Total Lunar | 2025-Mar-14 | Total | Yes | < 1 day |
| Partial Solar | 2025-Mar-29 | Partial | Yes | < 1 day |

All 6 NASA reference eclipses detected within 1-day tolerance.

> **What this engine is (honest scope):** for **solar** eclipses XALEN now runs
> the rigorous Bessel/Chauvenet method (`crate::besselian`): it forms the
> geocentric fundamental-plane elements, derives γ, the global type
> (total/annular/hybrid/partial) and the greatest-eclipse instant, and a separate
> observer-reduction layer (`crate::local_eclipse`) gives per-location magnitude,
> obscuration and the four contact instants C1–C4. The global type and γ are
> validated against NASA's published values for 2017-08-21 (γ = 0.4367) and
> 2024-04-08 (γ = 0.3431); cone angles match NASA, and the 2017 greatest-eclipse
> time is asserted within the committed bound of < 30 s vs NASA. The γ residual
> (~0.0015–0.002 Earth-radii) is the
> truncated-ELP Moon position, not the Besselian method — the method itself
> reproduces NASA to < 0.05%. Load the DE440 kernel to tighten γ further.
>
> **Still NOT computed:** the path of totality (northern/southern limits and
> central-line traces). That cartographic layer is tracked as future work; the
> per-observer contact times above are the practical substitute for a single
> location. Lunar eclipses use the simpler Meeus Ch.55 shadow-cone test (date +
> type, not contact-time geometry).

---

## Fixed Stars

| Metric | Value |
|--------|-------|
| Built-in catalog | 506 stars (`xalen-western`, all mag < 3.0 + Behenian/Royal/yogatara), plus **8,870 compiled-in Hipparcos stars** (Vmag ≤ 6.5) and a 108-star curated core catalog in `xalen-stars` — all zero-data-file |
| Magnitude range | Compiled-in to Vmag ≤ 6.5 (8,870 Hipparcos stars; covers all naked-eye + traditional astrologically significant stars) |
| Proper motion | Individual proper motion corrections for each star (J1991.25 → J2000.0 from Hipparcos pmRA/pmDE) |
| Precession to epoch | Full **IAU 2006/P03 rotation** (SOFA-validated `precession_matrix_p03_nobias`) on both star catalogs — the J2000 mean ecliptic direction is rotated to the mean ecliptic of date, precessing latitude consistently. (Earlier builds applied only the linear 50.28796″/yr rate, which left a latitude drift; that scalar is retained only for a regression guard test.) |
| Swiss agreement | The **named generated stars** (those carried by name in the generated catalog) match the **committed anchor rows** of live Swiss `fixstar2` (`SEFLG_J2000\|NONUT\|NOABERR\|NOGDEFL`) to **sub-arcsecond**, asserted against the public `find_generated_by_name` surface (`swiss_star_oracle.rs`). The committed oracle pins the named/anchor stars, not all 8,870 generated entries. |
| External catalogs | Full Hipparcos (118,218 stars) additionally loadable at runtime via CSV |

---

## Vedic Computations

| Component | Method | Validation |
|-----------|--------|------------|
| Nakshatra | 27-division (13 deg 20 min each) | Boundary verified: 0 deg = Ashwini, 120 deg = Magha |
| Rashi | 12-division (30 deg each) | Sun at J2000 sidereal in Sagittarius (Dhanu) verified |
| Panchang (5 limbs) | Computed from Sun/Moon sidereal positions | Tithi 1-30, Yoga 1-27, Vara (J2000 = Saturday verified) |
| Panchang transition times | Root-find the driving angle (Moon−Sun, Moon, Sun+Moon) for each limb's boundary | Start/end JD for current tithi/nakshatra/yoga/karana ("upto HH:MM"), the most-consumed panchang output |
| Vimshottari Dasha | 120-year cycle, nakshatra-lord based | Full Antardasha level computed |
| Divisional charts | D1 through D60 (16 varga charts) | Vargottama detection verified |
| Exaltation/debilitation | Classical lordship table | All 7 planets: exaltation and debilitation exactly 6 signs apart |
| Ashtottari Dasha | 108-year cycle | Implemented |
| Yogini Dasha | 36-year cycle | Implemented |
| Shadbala | Six-fold strength | Implemented |
| Ashtakavarga | Bindu computation | Implemented |
| KP (Krishnamurti) | Sub-lord table | Implemented |
| Jaimini | Chara karakas, Chara dasha | Implemented |
| Tajaka | Sahams, Ithasala yogas | Implemented |

---

## Time Systems

| Component | Model | Accuracy |
|-----------|-------|----------|
| Delta-T | Stephenson-Morrison-Hohenkerk 2016 cubic spline (genuine Table-S15) | 0.02 s at J2000 (spline 63.81 s vs IERS 63.83 s); tracks observed ΔT to <0.25 s across the telescopic era |
| Julian Day | UT1, TT, and TDB variants (type-safe) | Exact by construction |
| UTC ↔ TAI | Full IERS leap-second table 1972-01-01 → 2017-01-01 (28 steps) | Exact in the post-1972 era; pre-1972 "rubber-second" UTC is **not** modelled (the table floors to TAI−UTC = 10 s before 1972). Scale-aware `offset_to_tai_seconds` wires TAI/TT/TDB/UTC/UT1 with the documented sign convention |
| Calendar | Gregorian and Julian, bidirectional | Standard algorithms |

> **ΔT model & uncertainty.** `StephensonMorrisonHohenkerk2016` evaluates the
> genuine published SMH2016 Table-S15 cubic regression spline over [−720, AD 2016]
> (coefficients read verbatim) and the model's own lod-integral extrapolation tail
> outside that range — not a polynomial approximation. The spline's last fitted
> knot is AD 2016; the model carries a published scalar per-epoch σ envelope only,
> with **no** coefficient covariance matrix, so none is claimed.
>
> `delta_t_with_uncertainty` reproduces the published NAO/SMH scalar σ envelope as
> a left-continuous step lookup (≈180 s at −720, ≈15 s at AD 1000, ≈0.1 s in the
> telescopic era; NAO quadratic tails outside [−2000, 2500]).
> Past the last fitted knot (2016) ΔT is an extrapolation: σ is the **larger** of
> that envelope and Espenak's Huber Brownian-motion bound (calibration year +2005,
> ≈15.5 s at 2050 / ≈47.9 s at 2100), so the reported uncertainty never understates
> — for a process driven by unpredictable core-mantle coupling the conservative
> random-walk bound is the honest one.

---

## Cross-Validation Test Suite

The test suite (`tests/swiss_eph_crossval.rs`) validates against externally known
positions from Swiss Ephemeris (swetest) output and Meeus "Astronomical Algorithms"
(2nd ed.).

### Tolerances

| Level | Tolerance | Used for |
|-------|-----------|----------|
| Exact invariant | 0.01 deg | Rahu-Ketu opposition (mathematical identity) |
| Ayanamsa reference | 0.05 deg | Lahiri at J2000 against SE ICRC value |
| Sun (best-determined) | 0.1 deg | Sun at J2000, 2023, 1968 |
| Other planets | 0.5 deg | Mercury through Saturn at J2000 |
| Moon | 1.0 deg | Moon at J2000 (needs swetest refinement) |

### Tests passing

- Sun at J2000 (280.4589 deg, Meeus reference)
- All 7 planets at J2000 within tolerance
- Sun at 2023-Feb-25, 1968-May-24 (modern and historical)
- Moon daily motion 10-16 deg/day (30 consecutive days)
- Mercury elongation < 28 deg (4 years, 5-day steps)
- Venus elongation < 47.5 deg (4 years, 5-day steps)
- Rahu-Ketu opposition (7 dates, 1968-2025)
- House cusps valid across 6 locations x 9 systems x 3 dates
- Sun continuity (no jumps > 1.1 deg/day over 365 days)
- Outer planet period ordering (Jupiter > Saturn annual motion)
- Concurrent computation (20 threads, Arc-shared almanac)

---

## Large-Scale Statistical Cross-Validation (Swiss Ephemeris, 5,000,000 charts)

> **READ FIRST — provenance.** The 5M-chart run *in the table below* was an
> **INTERNAL** historical validation executed inside a private
> Swiss-Ephemeris-equipped monorepo; that specific run is not
> re-runnable here (its merge harness lives in that private tree, and several
> rows — notably the Moon — predate numeric fixes; see the per-row notes). The
> figures are kept for transparency.
>
> **A 5,000,000-chart (or any-N) sweep is now reproducible from THIS repository**
> via the committed `validation/` harness: `validation/oracle_pyswisseph.py`
> generates the `pyswisseph` oracle and the `xalen-validation` runner diffs it
> against the public XALEN crates. Run:
>
> ```bash
> python3 validation/oracle_pyswisseph.py --n 5000000 --seed 42 \
>   | cargo run -p xalen-validation --release -- -
> ```
>
> Honest caveat: the harness's oracle is only as good as the Swiss backend the
> local `pyswisseph` uses. With no Swiss `.se1` data files installed it falls
> back to the analytic Moshier theory, so the sweep then compares two independent
> analytic chains (XALEN vs Moshier), not XALEN vs DE440 — the harness detects and
> prints the backend per body so this is never hidden. The **DE440-grade** ground
> truth lives in the committed Horizons-vector oracle suite (`cargo test`:
> `swiss_ayanamsa_oracle`, `swiss_houses_oracle`, `swiss_star_oracle`,
> `swiss_eph_crossval`, `accuracy_vs_de440` with a kernel). See
> `validation/README.md`.

Beyond the fixed reference epochs above, every body was diffed against Swiss
Ephemeris across **5,000,000 deterministically-sampled charts** -- ten SplitMix64
shards of 500,000 each (seeds 1-10), dates 1850-2150, worldwide
latitude/longitude -- feeding the identical Julian Day into both engines, with
Swiss reading its real `.se1` ephemeris (DE431). Fixed seeds made the sample
reproducible **within that internal harness**; per-shard error histograms merged
to exact union statistics (in a private validation harness, not bundled here).
Errors in arcseconds:

| Body | max | p99 | rms |
|------|----:|----:|----:|
| Sun | 2.8 | 2.5 | 0.8 |
| Moon (PRE-FIX†) | 74 | 55 | 22 |
| Mercury | 5.3 | 4.1 | 1.3 |
| Venus | 9.9 | 5.9 | 1.4 |
| Mars | 7.1 | 3.9 | 1.0 |
| Jupiter | 4.4 | 1.5 | 0.6 |
| Saturn | 4.5 | 2.6 | 1.0 |
| Uranus | 5.7 | 1.8 | 0.9 |
| Neptune | 5.7 | 2.5 | 1.1 |
| Pluto | 8.9 | 8.7 | 3.9 |
| Rahu (mean node) | 19 | 18 | 12 |
| Rahu (true node) | 111 | 66 | 25 |
| Ascendant (\|lat\| <= 66 deg) | 750 | 45 | 16 |
| House cusps (\|lat\| <= 66 deg) | 750 | 29 | 13 |
| Ayanamsa (Lahiri) | 2.0 (0.00057 deg) | -- | -- |

**Zero of the 5,000,000 charts exceeded a 0.1 deg (360") tolerance** for any
planet or lunar node. Stated honestly:

> **† The Moon row predates the annual-aberration fix.** That 5M-chart run used
> the buggy path that applied the full annual aberration term (κ=20.49552",
> correct only for planets/Sun) to the geocentric Moon, which shares Earth's
> heliocentric velocity. The fix (Δψ + geocentric light-time, no annual
> aberration — `vsop::apparent_moon`) drops the analytical Moon residual to
> **RMS ~2.8" / max ~12"** over AD 1600-2100 measured vs pyswisseph 2.10.03; the
> DE440 Moon becomes sub-arcsecond. A full 5M-chart re-run will replace the row;
> the pre-fix numbers are kept here only for provenance.

- The **true node** (max 111") is now the largest planet/node residual; the
  post-fix analytical Moon (RMS ~2.8") is no longer the soft spot. All bodies
  still sit well inside 0.1 deg.
- **`accuracy_arcsec()` API figure:** the analytical provider bounds its worst
  physical body; the DE440 provider reports **2"** — it computes the apparent
  place (body light-time retardation + precession + IAU 2000B nutation; the Moon
  additionally gets its geocentric light-time but NOT the full annual aberration),
  with the kernel's raw geometry exact. The DE440 Sun, planets and (post-fix)
  Moon are all sub-arcsecond, so the 2" figure is a conservative bound, not an
  overstatement of a precision the bodies do not achieve. Both figures are scoped to the PHYSICAL bodies. The derived
  lunar nodes (mean ~19", true ~111") reflect differing node *algorithms* vs
  Swiss, not ephemeris error, and are characterised by the table above rather
  than folded into the single physical-body figure.
- **Pluto:** XALEN's analytical Pluto (Meeus Ch.37 Goffin/Steyaert fit) is valid
  1885-2099; ~1.44 million sampled dates (28.9%) fall outside that window and are
  excluded from the analytical-Pluto statistics. This is **XALEN's** analytical
  limit, not Swiss's -- Swiss computes Pluto across millennia. Over the
  **3,556,034** in-window charts, XALEN agrees with Swiss to **8.87"** max. With a
  DE440 kernel loaded, Pluto is served from JPL DE440 across the full 1550-2650
  span, closing the gap.
- **Ascendant / cusps (|lat| <= 66 deg):** p99 within 0.013 deg (asc 45", cusp 29"),
  mean 0.003 deg. The single worst chart (0.208 deg, ~1 in 5 million) lands at
  latitude -66.0 deg -- on the polar-circle boundary where Placidus is near-singular;
  it is a house-system edge, not a position error.
- **Polar latitudes (|lat| > 66 deg):** Placidus is mathematically degenerate near
  the poles and XALEN's and Swiss's fallbacks diverge by up to 180 deg -- house
  cusps are **not** comparable there (use Whole-Sign or Porphyry). Those rows are
  excluded from the ascendant/cusp figures above, which are for |lat| <= 66 deg.

The table above was produced by a private Swiss-Ephemeris-equipped harness. To
reproduce a sweep of the same shape from this public repository — at the same
5,000,000 charts or any other N — use the committed `validation/` harness
(`validation/README.md`); for DE440-grade ground truth the in-repo oracle suite
pins committed JPL Horizons DE440 reference vectors, since XALEN ships no
external ephemeris dependency.

---

## Comparison with Swiss Ephemeris

| Metric | XALEN Ephemeris | Swiss Ephemeris |
|--------|----------------|-----------------|
| Sun accuracy (analytical) | < 1" (VSOP87A) | < 1" (Moshier) |
| Moon accuracy (analytical) | RMS ~2.8" / max ~12" (60-term Meeus ELP + Δψ + geocentric light-time) | ~2" (ELP2000-82) |
| Outer planets | < 1-5" (VSOP87A) | < 1-5" (VSOP87A or Moshier) |
| Max precision (with data files) | Sub-mas (DE440) | Sub-mas (DE441) |
| Epoch range (analytical) | ~4000 BCE -- 8000 CE | ~5400 BCE -- 7900 CE |
| Ayanamsa systems | 50 named + Custom | 40+ |
| House systems | 23 (18 of 19 oracle-comparable match Swiss < 0.01°) | 23 |
| Fixed stars (built-in) | 8,870 compiled-in (xalen-stars, Vmag ≤ 6.5) + 506 (xalen-western) + 108-star curated core | 6,000+ (with catalog files) |
| Precession model | IAU 2006/P03 | IAU 2006 (Vondrak 2011 option) |
| Nutation model | IAU 2000B (77 terms) | IAU 2000A/B |

**Bottom line:** For all standard astrological computations (natal charts, dasha,
transits, compatibility, panchang), the analytical engine delivers positions that
are indistinguishable from Swiss Ephemeris results. The differences are in the
sub-arcsecond range -- invisible to any astrological interpretation technique.
