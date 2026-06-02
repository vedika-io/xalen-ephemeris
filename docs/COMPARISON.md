# XALEN Ephemeris vs. the Field

An honest, source-grounded comparison of XALEN Ephemeris against the
ephemeris/astrology libraries it is most often weighed against:

- **Swiss Ephemeris** — the C library (`swetest` / `libswe`) from Astrodienst,
  the de-facto standard behind astro.com and most commercial astrology software.
- **pyswisseph** — the Python C-extension wrapper around Swiss Ephemeris
  (`import swisseph`). It is a binding, so it inherits Swiss Ephemeris's numbers,
  license, and `.se1` data-file requirement.
- **`astro`** (crates.io) — a small pure-Rust crate implementing selected
  *Astronomical Algorithms* (Meeus) routines: Sun/Moon, a few planets, time, and
  coordinate helpers.
- **`practical-astronomy-rust`** (crates.io) — a pure-Rust port of the worked
  examples in Duffett-Smith & Zwart, *Practical Astronomy with your Calculator or
  Spreadsheet*. A textbook-companion library, not an astrology engine.

> **Framing first — XALEN does not "beat" JPL or Swiss on raw geometry.**
> JPL DE440 *is* the reference, and Swiss Ephemeris reads the same JPL kernels.
> XALEN's claim is *matching* JPL/Swiss accuracy to the bounds measured below
> while shipping a license, packaging, and engineering profile the C/AGPL
> incumbents cannot offer (pure Rust, zero `unsafe` in the core, thread-safe,
> WASM-ready, Apache-2.0, zero data files). Every number here is reproducible
> from this repo's tests; where a competitor figure is from its own published
> docs rather than a test we ran, it is labelled.

---

## 1. The matrix at a glance

| Dimension | XALEN Ephemeris | Swiss Ephemeris (`swetest`/`libswe`) | pyswisseph | `astro` (Rust) | `practical-astronomy-rust` |
|---|---|---|---|---|---|
| **Language / core** | Pure Rust, zero `unsafe` in core¹ | C | C (wrapped for Python) | Pure Rust | Pure Rust |
| **License** | **Apache-2.0** | AGPL-3.0 **or** paid commercial | AGPL-3.0 (inherits Swiss) | MIT/Apache (permissive) | MIT |
| **Data files at runtime** | **None** — VSOP87/ELP/IAU theory + 8,870 stars compiled in; DE440 kernel optional | Requires `.se1` files (or Moshier fallback) for full precision | Same as Swiss | None | None |
| **Thread-safe** | Yes — `Almanac` is `Send + Sync`, zero global state | Process-global state (`swe_set_*`) — not thread-safe | Same as Swiss | n/a (stateless calls) | n/a (stateless calls) |
| **WASM** | Yes — core crates build `wasm32-unknown-unknown` | No (C, file I/O) | No | Likely (pure Rust) — untested here | Likely (pure Rust) — untested here |
| **Sun + Mercury–Saturn accuracy** | sub-arcsecond vs DE440 (0.1–1.1″, §3) | sub-arcsecond (reads JPL kernel) | same as Swiss | Meeus-grade (≈arcsec–arcmin, body-dependent) | textbook-grade (arcmin-class) |
| **Moon accuracy (analytical)** | RMS ~2.8″ / max ~12″ (truncated ELP); ~sub-arcsec with DE440 kernel | ~few arcsec (full ELP / Moshier); sub-arcsec with kernel | same as Swiss | Meeus low-order (arcmin-class) | textbook-grade |
| **House systems** | **23** (19 comparable systems within 0.01°, 18 tight; §5) | 23 | 23 (via Swiss) | None | A handful (textbook examples) |
| **Ayanamsa systems** | **50 named** (all 47 Swiss IDs 0–46 + 3 non-SE systems) + Custom, §4 | ~47 predefined + custom | same as Swiss | None | None |
| **Fixed stars (built-in)** | **8,870** compiled-in (Hipparcos Vmag ≤ 6.5) + 506 astrology catalog | 6,000+ via `sefstars.txt` data file | same as Swiss | None | None |
| **Vedic / Jyotish** | Deep: dasha, panchang, shadbala, KP, Jaimini, Tajaka, ashtakavarga, D-2…D-60, yogas/doshas (§6) | **None** (positions only) | None | None | None |
| **Western technique layer** | Aspects, dignities, 97 Arabic Lots, returns, progressions, harmonics, horary | None (positions only) | None | None | None |
| **World systems** | BaZi, Zi Wei, Saju, Mayan, Aztec, I-Ching, Tibetan, Nine Star Ki, Mahabote (day-sign profile), Qi Men (experimental), Persian, Egyptian, Celtic (§7) | None | None | None | None |
| **Bindings** | Rust + C-FFI + Python (PyO3) + Node (napi) + WASM, plus `swe_*` compat layer (§8) | C; many third-party wrappers | Python only | Rust only | Rust only |
| **Structured JSON output** | Yes — all public types derive `serde` Serialize/Deserialize | No (returns C arrays) | Python tuples/floats | Rust structs (no serde by default) | Rust structs |

¹ The only `unsafe` in the whole workspace lives in the C-binding crate
`xalen-ffi` (required for the `extern "C"` surface). Every other crate is
`unsafe`-free. The 14 computation crates (`xalen-time`, `-coords`, `-ephem`, `-houses`,
`-ayanamsa`, `-stars`, `-vedic`, `-western`, `-chinese`, `-numerology`,
`-world`, `-lalkitab`, `-iching`, `-chart`) contain **no `unsafe`** — verified by
grep across `crates/*/src`.

---

## 2. License — the clearest differentiator

This is where the comparison is least ambiguous.

- **Swiss Ephemeris** is dual-licensed: **AGPL-3.0** (copyleft — the network-use
  clause means a hosted service must release its full source) **or** a **paid
  commercial license** from Astrodienst. **pyswisseph inherits this**: using it in
  a closed-source SaaS requires the commercial Swiss license.
- **XALEN is Apache-2.0** (see `LICENSE`): permissive, patent-grant included, no
  copyleft, no per-deployment fee, usable in closed-source and commercial
  products with attribution only.
- `astro` and `practical-astronomy-rust` are also permissive (MIT/Apache), but
  they do not offer XALEN's accuracy, breadth, or astrology layers.

**Net:** if you need JPL-class accuracy *and* a permissive license in one
package, XALEN is currently the only option. The accuracy alternatives (Swiss)
are AGPL/commercial; the permissive alternatives (`astro`,
`practical-astronomy-rust`) are textbook-accuracy and astronomy-only.

---

## 3. Planetary accuracy, per body (measured)

All XALEN figures below come from `docs/ACCURACY.md` and are reproducible from
the in-repo tests cited in §9. They are *apparent geocentric ecliptic longitude*
errors versus **JPL Horizons DE440**.

| Body | XALEN analytical engine (no data files) | XALEN with DE440 kernel | Swiss Ephemeris |
|---|---|---|---|
| Sun | 0.4–1.1″ (VSOP87A + IAU 2000B) | sub-arcsecond | sub-arcsecond (kernel) |
| Mercury | ~0.35″ | sub-arcsecond | sub-arcsecond |
| Venus | ~0.30″ | sub-arcsecond | sub-arcsecond |
| Mars | 0.3–0.7″ | sub-arcsecond | sub-arcsecond |
| Jupiter | 0.1–0.8″ | sub-arcsecond | sub-arcsecond |
| Saturn | 0.1–1.0″ | sub-arcsecond | sub-arcsecond |
| Uranus | ~0.52″ | sub-arcsecond | sub-arcsecond |
| Neptune | ~1.07″ | sub-arcsecond | sub-arcsecond |
| Pluto | ~1′ (Goffin DE200 fit, valid **1885–2099** only) | full-range, sub-arcsecond | full-range (kernel) |
| **Moon** | **RMS ~2.8″ / max ~12″** (truncated 60+60-term ELP) | **~sub-arcsecond** | ~few arcsec (analytical) / sub-arcsec (kernel) |
| Mean Node (Rahu) | exact by mean model | — | mean model |
| True Node | ~differs from Swiss by algorithm, not error | — | reference |

Statistical confirmation: across a **5,000,000-chart** diff vs Swiss Ephemeris
(reading its real DE431 `.se1` data), **0 of 5,000,000 charts exceeded 0.1°** for
any planet or node (`docs/ACCURACY.md` §"Large-Scale Statistical
Cross-Validation"). VSOP87A is additionally proven against the *official* IMCCE
`vsop87.chk` check file to < 1×10⁻⁹ AU for the inner planets
(`crates/xalen-ephem/tests/vsop87_official_crossval.rs`).

### Where XALEN does NOT win on accuracy — stated plainly

- **Analytical Moon.** XALEN's no-data-file Moon is a *truncated* ELP series
  (~2.8″ RMS, max ~12″). Swiss Ephemeris's analytical Moon (full ELP / Moshier)
  is tighter, and either engine is sub-arcsecond only when a kernel is loaded. If
  you need sub-arcsecond Moon, **load the DE440 kernel** — the analytical path is
  deliberately a compactness/no-files trade-off.
- **Analytical outer planets** (Uranus ~1.8″, Neptune ~2.5″ at the worst-case
  bound in the README table) are slightly looser than the inner planets. For the
  tightest outer-body figures, use the DE440 kernel.
- **Pluto without a kernel** is valid **only 1885–2099** (Goffin fit). Outside
  that window the analytical Pluto is not usable; the DE440 kernel covers the full
  span. Swiss computes Pluto across millennia without this caveat.
- **`astro` / `practical-astronomy-rust`** are not accuracy competitors — they
  target Meeus / Duffett-Smith textbook precision (arcsecond-to-arcminute,
  body-dependent), with no DE440 path. XALEN is roughly one to three orders of
  magnitude tighter on the planets, but they are smaller, simpler crates and were
  never trying to be JPL-class.

---

## 4. Ayanamsa — 50 named systems, Swiss-matched

XALEN's `Ayanamsa` enum (`crates/xalen-ayanamsa/src/lib.rs`) has **51 variants**:
**50 fixed, named systems plus a `Custom` variant** (user-defined epoch,
ayanamsa-at-epoch, and precession rate). The 50 named systems **cover all 47
Swiss Ephemeris predefined IDs (`SE_SIDM_*`, 0–46)** — those 47 already include
the J2000 / J1900 / B1950 reference epochs as SE IDs 18 / 19 / 20 — and add **3
systems that have no Swiss ID of their own**: PushyaPaksha, KPStraightLine, and
CitraAtSpica180.

So both numbers you may see are correct and mean different things:
- **47** = the count of Swiss-Ephemeris-predefined ayanamsa IDs XALEN matches
  (J2000/J1900/B1950 are among them, as SE 18/19/20).
- **50** = total *named* systems (the 47 SE IDs + the 3 non-SE systems
  PushyaPaksha / KPStraightLine / CitraAtSpica180).
- **51** = enum variants including `Custom`.

**Honest validation scope** (`docs/ACCURACY.md` §"Ayanamsa"): of the 47
Swiss-mapped systems, **46 match Swiss `get_ayanamsa_ex` to < 1″** across
1900–2100, with 1 documented exception held to ≤ 2″ (`GalCenterMulaWilhelm`
SE 36). `TrueChitra` (SE 27) is now within 1″ (0.038″) after its Spica reduction
moved to the rigorous IAU 2006/P03 Cartesian (latitude-coupled) precession.
Lahiri specifically tracks Swiss `SE_SIDM_LAHIRI`
to **≤ 0.74″** across that span (a true epoch-anchored reduction, not a linear
J2000 approximation). There is, however, **no arcsecond-level reconciliation
against the Indian Astronomical Ephemeris / Rashtriya Panchang** — no IAE
reference data is bundled, and IAE-named checks use coarse (0.05–0.15°)
tolerances. Swiss, pyswisseph (via Swiss), `astro`, and
`practical-astronomy-rust` offer **no** ayanamsa framework at all
(`astro`/`practical-astronomy-rust` are tropical-only astronomy crates).

---

## 5. House systems — 23 implemented, 19 comparable to Swiss (18 tight)

XALEN's `HouseSystem` enum (`crates/xalen-houses/src/systems.rs`) has **23
variants**, matching Swiss Ephemeris's count: WholeSign, Equal, Placidus, Koch,
Porphyry, Regiomontanus, Campanus, Morinus, Alcabitius, AlcabitiusClassic,
Topocentric, Meridian, Vehlow, Sripati, KrusinskiPisa, Gauquelin,
SunshineMakransky, SunshineTreindl, PullenSinusoidalDelta, PullenSinusoidalRatio,
CarterPoliEquatorial, APC, Zariel.

**Honest scope** (`docs/ACCURACY.md` §"House Systems"): Swiss can supply a
comparable `houses_armc` cusp array for **19** of the 23 systems. **18 of those 19
match Swiss to < 0.01°** (the tight bar) — including Placidus (also verified at
scale: ascendant/cusp p99 within ~0.013° for |lat| ≤ 66°), Pullen Sinusoidal
(Delta), Carter Poli-Equatorial, APC, and Zariel. The 19th, **Pullen Sinusoidal
(Ratio)**, is a documented approximation held to a loosened ~5° tolerance. The
remaining **4** — Gauquelin sectors, both Sunshine variants (Makransky, Treindl),
and Alcabitius Classic — have **no comparable Swiss oracle** (Swiss degenerates
them to Placidus), so they are not Swiss-comparable and are excluded from the
count. Polar latitudes fall back to Porphyry automatically.

`astro` and `practical-astronomy-rust` implement **no house systems**.

---

## 6. Vedic / Jyotish depth — where Swiss has nothing

Swiss Ephemeris and pyswisseph return *positions only*. They have **no** dasha,
panchang, shadbala, ashtakavarga, KP, Jaimini, Tajaka, varga, or yoga layer — you
build all of it yourself on top of their longitudes. `astro` and
`practical-astronomy-rust` have none of this either.

XALEN ships it as first-class, serde-serializable computations
(`crates/xalen-vedic`, verified by 602 in-crate tests):

- **Dashas:** Vimshottari (full antardasha), Ashtottari, Yogini, Jaimini Chara,
  Sudarshana Chakra.
- **Panchang:** all five limbs (tithi, vara, nakshatra, yoga, karana).
- **Divisional charts:** D-2 through D-60 (16 vargas), with vargottama detection.
- **Strength:** Shadbala (six-fold), Ashtakavarga (bindus), planetary dignity,
  Mrityu Bhaga.
- **Branches:** KP (Krishnamurti Paddhati) sub-lords, Jaimini karakas/arudha,
  Tajaka/Varshaphal, Prashna, Muhurta, Gochara transits, Upagraha (Gulika/Mandi).
- **Yogas & doshas:** Pancha Mahapurusha, Raja yogas, Mangal/Kuja Dosha, Kaal
  Sarp, Gandanta.

**Honest gap:** the **Nadi** (Bhrigu Nandi Nadi) module ships the 48 planet/sign
rule *slots* tagged by life-domain, but the **interpretive reading text is not
bundled** in this open-source release (each slot's `indication` field is
`Option<&'static str>` and returns `None`, not an empty string) — see
`crates/xalen-vedic/README.md`. The *computation* is present; the *interpretive
content* is not, by design (interpretive prose must be public-domain or
human-authored).

---

## 7. World systems — breadth no astronomy library offers

None of Swiss, pyswisseph, `astro`, or `practical-astronomy-rust` implement any
of these. XALEN ships them across `xalen-chinese`, `xalen-world`,
`xalen-iching`, `xalen-numerology`, and `xalen-lalkitab`, all serde-serializable:

- **Chinese** (`xalen-chinese`): BaZi Four Pillars (Li Chun year boundary), Zi Wei
  Dou Shu, Feng Shui (Flying Stars + Eight Mansions), 24 solar terms, and **Qi Men
  Dun Jia (experimental)** — the Stars/Doors/Deities/Lo-Shu reference data and the
  day/hour stem pillars are correct, but the Ju/chart assembly is simplified and
  the output is **not an authoritative reading**.
- **East/South-East Asian** (`xalen-world`): Korean **Saju**, Japanese **Nine Star
  Ki**, Burmese **Mahabote** (day-sign / ruling-planet profile only — *not* the
  full Mahabote 7-house square).
- **Mesoamerican** (`xalen-world`): **Mayan** (Long Count / Tzolkin / Haab,
  GMT 584283), **Aztec** Tonalpohualli (Caso correlation).
- **Other traditions** (`xalen-world`): Tibetan Rabjung, Persian Jarbakhtar/Tasyir,
  Egyptian decans, Celtic tree calendar.
- **I-Ching** (`xalen-iching`): 64 hexagrams, 8 trigrams, date casting.
- **Numerology** (`xalen-numerology`): Pythagorean + Chaldean.
- **Lal Kitab** (`xalen-lalkitab`): planet-house effects, debts, dormancy,
  remedies.

**Honest caveats:** several of these are *deliberately approximate* and say so in
their own READMEs — e.g. Tibetan Losar is an *approximate* Julian Day for a
Western year, and the Persian **Tasyir arc is a first-order ecliptic-longitude
approximation, not a true right-ascension primary direction**
(`crates/xalen-world/README.md`). Chinese solar-term boundaries use the Meeus
low-accuracy solar-longitude formula (~0.01°). These are correct for calendar/
profile use, not for sub-arcsecond timing. As with Nadi, these systems ship the
**computation + life-domain tags only** — no interpretive reading prose is
bundled (the unbundled fields surface as `None`/empty, never as fabricated text).

---

## 8. Bindings and the `swe_*` drop-in

XALEN compiles once and exposes itself to five surfaces — Swiss requires a
separate third-party wrapper per language, and pyswisseph is Python-only:

| Binding | Crate | Mechanism | Status |
|---|---|---|---|
| Rust | `xalen-ephemeris` (umbrella) | native | Published on crates.io (0.3.1 line; 0.4.x+ not yet) |
| C / C++ | `xalen-ffi` | `extern "C"` + `repr(C)` | Source-stable; crates.io 0.3.1 line |
| Python | `xalen-python` | PyO3 — `maturin develop` | Alpha; **not yet on PyPI** (`pip install xalen` forthcoming) |
| Node.js | `xalen-node` | napi-rs native addon — `napi build` | Alpha; **not yet on npm** (`npm install xalen` forthcoming) |
| Browser / WASM | `xalen-wasm` | wasm-bindgen — `wasm-pack build` | Alpha; build from source |

**Migration path.** XALEN provides a Swiss-Ephemeris-compatible shim so existing
code can move with minimal edits:

- Rust: `xalen_ephem::compat` exposes `swe_calc_ut(jd, SE_SUN, SEFLG_SWIEPH)`
  with the familiar `xx[0..6]` array semantics
  (`docs/MIGRATING_FROM_SWISS_EPH.md`).
- Node: `npm install xalen` as a drop-in for the `sweph` package — no `node-gyp`,
  no C compiler, no `.se1` files (`docs/SWEPH_NPM_REPLACEMENT.md`). **Forthcoming:**
  the npm package is not published yet; build the napi-rs addon from source for now.

**Honest scope of the C-FFI surface:** the stable `extern "C"` layer
(`crates/xalen-ffi`) intentionally exposes a *subset* of the full Rust API — **17
ayanamsa IDs and 14 house-system IDs** are reachable over C-FFI today (vs 50 and
23 in native Rust). The full catalogs are available through the Rust, WASM, and
napi/PyO3 surfaces; the C-FFI subset will be widened over time. The Python/Node/
WASM bindings are **Alpha** (the `pip`/`npm` publishes are pending, tasks #35),
while Rust and C-FFI are Stable.

**Structured output.** Every public XALEN type derives `serde`
`Serialize`/`Deserialize`, so positions, charts, dashas, panchang, BaZi pillars,
etc. serialize straight to JSON. Swiss Ephemeris returns C `double[6]` arrays and
status codes; pyswisseph returns Python tuples/floats — JSON shaping is the
caller's job.

---

## 9. Evidence — reproduce it yourself

XALEN's claims are backed by in-repo tests (run with `cargo test`; the workspace
default-members exclude the PyO3 crate, which needs a Python toolchain):

| Claim | Test file |
|---|---|
| VSOP87A vs official IMCCE check file | `crates/xalen-ephem/tests/vsop87_official_crossval.rs` |
| Apparent positions vs real DE440 kernel | `crates/xalen-ephem/tests/de440_real_crossval.rs`, `accuracy_vs_de440.rs` |
| Positions vs Swiss Ephemeris epochs | `crates/xalen-ephem/tests/swiss_eph_crossval.rs` |
| Positions vs public online calculators | `crates/xalen-ephem/tests/online_calculator_crossval.rs` |
| Precession/nutation vs ERFA/SOFA golden vectors | `crates/xalen-coords/tests/sofa_reference_crossval.rs` |
| House cusps vs Swiss oracle | `crates/xalen-houses/tests/swiss_houses_oracle.rs` |
| Ayanamsa vs Swiss oracle | `crates/xalen-ayanamsa/tests/swiss_ayanamsa_oracle.rs` |
| Fixed stars vs Swiss oracle | `crates/xalen-stars/tests/swiss_star_oracle.rs` |
| ΔT vs occultation/polynomial references | `crates/xalen-time/tests/delta_t_*_crossval.rs` |

Aggregate at the time of writing: `cargo test --workspace` reports **2,199 tests
passing** across the library and integration suites, 0 failures (`README.md`,
`docs/ACCURACY.md`). The external 5,000,000-chart statistical run vs Swiss
Ephemeris is reported for transparency but is **not reproducible from this public
repo** (it ran in a private Swiss-equipped tree).

---

## 10. When to pick which

- **Pick Swiss Ephemeris / pyswisseph** if you are already deeply invested in the
  Swiss ecosystem, need its exact historical-millennia Pluto/Moon without loading
  a kernel into XALEN, and the AGPL/commercial license is acceptable to you.
- **Pick `astro` or `practical-astronomy-rust`** if you want a tiny permissive
  Rust crate for textbook-grade Sun/Moon/planet positions and don't need
  astrology layers, ayanamsa, houses, or sub-arcsecond accuracy.
- **Pick XALEN** if you want **JPL-class planet accuracy under a permissive
  Apache-2.0 license, with zero data files to ship, real thread-safety, WASM
  support, and the deepest Vedic + Western + world-systems layer of any of these
  libraries** — and you are comfortable loading the optional DE440 kernel when you
  need sub-arcsecond Moon/outer-body/full-range-Pluto precision, and aware that a
  few world/Nadi interpretive tables are intentionally left empty pending
  public-domain or human-authored content.

*Last reviewed against repo state 2026-06-02. Competitor figures for Swiss
Ephemeris / pyswisseph / `astro` / `practical-astronomy-rust` are from those
projects' own documentation and are labelled as such; all XALEN figures are
reproducible from the tests in §9.*
