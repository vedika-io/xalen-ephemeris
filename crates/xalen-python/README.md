# xalen

**Pure-Rust astronomical & astrology ephemeris for Python — VSOP87 planets, Vedic/Western/KP, house systems, ayanamsa, fixed stars, numerology. A drop-in for `pyswisseph`, with no AGPL and no data files to ship.**

[![PyPI](https://img.shields.io/pypi/v/xalen.svg)](https://pypi.org/project/xalen/)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://github.com/vedika-io/xalen-ephemeris/blob/main/LICENSE)

> Part of the [XALEN Ephemeris](https://github.com/vedika-io/xalen-ephemeris) suite — pure-Rust core, zero `unsafe` in the math, thread-safe, Apache-2.0. The Python wheel is the `xalen-python` crate (PyO3) wrapping that core.

---

## Install

```bash
pip install xalen
```

To build from the repository instead:

```bash
pip install maturin
git clone https://github.com/vedika-io/xalen-ephemeris.git
cd xalen-ephemeris/crates/xalen-python
maturin develop --release        # builds + installs into the active venv
# or build a wheel:  maturin build --release  ->  target/wheels/xalen-*.whl
```

`maturin` enables the `extension-module` Cargo feature automatically (it is set in `pyproject.toml`), so the wheel resolves Python symbols against the host interpreter — no `libpython` link needed.

Requires Python ≥ 3.8.

### Platform support

Wheels are built with the **abi3** stable ABI (`abi3-py38`), so a **single wheel
per platform** imports unchanged on CPython 3.8 and every later 3.x — there is no
per-interpreter-version wheel to pick.

| Platform | On PyPI |
|----------|---------|
| Linux x86_64, glibc ≥ 2.17 (manylinux2014) | `cp38-abi3` wheel |
| Linux aarch64, glibc ≥ 2.17 (manylinux2014) | `cp38-abi3` wheel |
| macOS arm64, 11.0 or later | `cp38-abi3` wheel |
| macOS x86_64, 10.12 or later | `cp38-abi3` wheel |
| **Windows, musl Linux and any other platform** | **sdist**: `pip install` builds from source (needs a Rust toolchain) |

The wheels and the sdist are built on one Linux host by
[`scripts/build-release-artifacts.sh`](https://github.com/vedika-io/xalen-ephemeris/blob/main/scripts/build-release-artifacts.sh),
which also smoke-tests them. A Windows wheel is not published yet.

---

## `import xalen.swe as swe` — the pyswisseph drop-in

The headline feature: migrating an existing `pyswisseph` (a.k.a. `swisseph`) codebase is a one-line search-and-replace.

```python
# before
import swisseph as swe
# after
import xalen.swe as swe
```

Every function mirrors the **shape** of the matching pyswisseph entry point — argument order, tuple layout, and the `SE_*` / `SEFLG_*` / `SIDM_*` constants (exposed under both the bare pyswisseph spellings and the `SE_`-prefixed ones).

```python
import xalen.swe as swe

jd = swe.julday(1990, 6, 15, 10.5)            # -> Julian Day (UT1)

# Sun position with speed. Returns ((lon, lat, dist, lon_speed, lat_speed,
# dist_speed), ret_flag) — exactly like pyswisseph.
xx, retflag = swe.calc_ut(jd, swe.SUN, swe.FLG_SWIEPH | swe.FLG_SPEED)
lon, lat, dist, lon_speed, *_ = xx

# Sidereal (Lahiri) longitude.
swe.set_sid_mode(swe.SIDM_LAHIRI, 0.0, 0.0)
sid, _ = swe.calc_ut(jd, swe.SUN, swe.FLG_SWIEPH | swe.FLG_SIDEREAL)

# Houses: (cusps[12], ascmc[8]). ascmc = (asc, mc, armc, vertex,
# equatorial_ascendant, co_ascendant_koch, co_ascendant_munkasey, polar_ascendant).
cusps, ascmc = swe.houses_ex(jd, 18.52, 73.85, b"P")   # Placidus (hsys as bytes)
asc, mc = ascmc[0], ascmc[1]
# Sidereal houses: pass SEFLG_SIDEREAL (active-mode ayanamsa subtracted).
sid_cusps, sid_ascmc = swe.houses_ex(jd, 18.52, 73.85, b"P", swe.FLG_SIDEREAL)

# Ayanamsa, ΔT, calendar round-trip, fixed stars.
aya  = swe.get_ayanamsa_ut(jd)
dt   = swe.deltat(jd)
y, m, d, h = swe.revjul(jd)
star_xx, star_name, _ = swe.fixstar2_ut("Aldebaran", jd, swe.FLG_SWIEPH)
mag, _ = swe.fixstar2_mag("Aldebaran")
```

### Honest compatibility scope

XALEN is a faithful *shape* drop-in, not a byte-for-byte clone. The differences are deliberate and documented:

- **Speeds** (`xx[3..6]`) are `0.0` unless `SEFLG_SPEED` is in the flags — matching Swiss.
- **`houses_ex`** returns `ascmc` of length 8; **all eight slots are populated** — `[0..4]` are `asc, mc, armc, vertex` and `[4..8]` are the equatorial ascendant, the Koch and Munkasey co-ascendants, and the polar ascendant (validated against pyswisseph's `swe.houses_armc` auxiliary points). `hsys` may be passed as bytes (`b"P"`, the pyswisseph form) or str (`"P"`). Passing `SEFLG_SIDEREAL` in `flags` returns the sidereal frame (active-mode ayanamsa subtracted from every cusp and angle; ARMC, a sidereal-time angle, is unchanged).
- **`swe.calc(jd_et, ...)`** takes a TT/ET epoch (like pyswisseph) and converts it to UT1 via ΔT before computing, so `swe.calc(jd_tt, ...)` equals `swe.calc_ut(jd_tt − deltat(jd_tt), ...)`. (The compat *core* still works in UT1; `calc` does the ΔT bridge for you.)
- **Position-altering flags XALEN does not yet implement** (`SEFLG_HELCTR`, `SEFLG_TOPOCTR`, `SEFLG_J2000`, `SEFLG_EQUATORIAL`, `SEFLG_BARYCTR`, `SEFLG_XYZ`, `SEFLG_RADIANS`) **raise `ValueError`** rather than silently returning a geocentric ecliptic position mislabeled as something else. A loud error beats a silent-wrong drop-in.
- **`set_ephe_path()` and `close()`** are no-ops — XALEN embeds all data at compile time, so there are no ephemeris files to point at or resources to release.

### Exceptions: `swe.Error`

`xalen.swe.Error` (a `RuntimeError`) for calculation failures; `TypeError`/`ValueError` only for bad arguments.

- **`swe.Error`** is raised for every failed calculation: an unknown body number (`calc_ut(jd, 999)`), an unknown star name, a body `rise_trans` cannot handle, an eclipse search that finds nothing in 100 years, an ephemeris error. Like pyswisseph's `swe.Error`, so `try: ... except swe.Error:` around an optional body keeps working, and `except RuntimeError` catches it too.
- **`TypeError`**: an argument of the wrong Python type (`geopos` that is not a sequence of three numbers, `hsys` that is neither bytes nor str).
- **`ValueError`**: an argument with a bad value (a non-finite Julian Day, a latitude outside ±90°, a house longitude outside -180°..360°, an unknown house letter, a negative pressure, a `horhgt` outside -90°..90° other than -100) and the position-altering `calc_ut` flags listed below, which XALEN does not implement.

Before 0.8 the shim raised `ValueError` for some calculation failures (unknown body, unknown star, a failed house or ayanamsa calculation); those are `swe.Error` now.

### Rise, set, transit and eclipses

```python
import xalen.swe as swe

geopos = (77.2090, 28.6139, 0.0)                  # lon, lat, altitude in metres
jd = swe.julday(2024, 6, 20, 18.5)                # 00:00 IST on 21 June 2024
res, tret = swe.rise_trans(jd, swe.SUN, swe.CALC_RISE, geopos)   # res 0, tret[0] = sunrise (JD UT)
res, tret = swe.rise_trans(jd, swe.MOON, swe.CALC_MTRANSIT, geopos)
res, tret = swe.rise_trans(jd, swe.SUN, swe.CALC_RISE | swe.BIT_DISC_CENTER | swe.BIT_NO_REFRACTION, geopos)
res, tret = swe.rise_trans_true_hor(jd, swe.SUN, swe.CALC_SET, geopos, 1013.25, 15.0, 2.0)  # 2° skyline
res, tret = swe.rise_trans_true_hor(jd, swe.SUN, swe.CALC_SET, (77.2, 28.6, 1000.0), 0.0, 15.0, -100.0)  # sea-horizon dip from 1000 m

retflag, tret = swe.sol_eclipse_when_glob(swe.julday(2024, 1, 1))     # 2024-04-08, ECL_TOTAL | ECL_CENTRAL
retflag, tret, attr = swe.sol_eclipse_when_loc(swe.julday(2024, 4, 1), (-96.80, 32.78, 0.0))  # Dallas
retflag, tret = swe.lun_eclipse_when(swe.julday(2022, 10, 1))         # 2022-11-08, ECL_TOTAL
retflag, attr = swe.lun_eclipse_how(tret[0], (139.69, 35.69, 0.0))     # Tokyo: ECL_TOTAL, attr[0] 1.36
retflag, attr = swe.lun_eclipse_how(tret[0], geopos)                  # Delhi: 0, the Moon has not risen
retflag, tret, attr = swe.lun_eclipse_when_loc(swe.julday(2022, 10, 1), geopos)
retflag, tret = swe.lun_eclipse_when(swe.julday(2022, 10, 1), swe.FLG_SWIEPH, 0, True)  # previous one
xx, _ = swe.calc_ut(jd, swe.ECL_NUT)              # true and mean obliquity, nutation in lon and obl
```

Signatures, argument order, tuple lengths (`tret` 10, `attr` 20), `retflag` bits and return codes follow pyswisseph 2.10, and the `CALC_*`, `BIT_*` and `ECL_*` constants have pyswisseph's values in both spellings (`CALC_RISE`, `SE_CALC_RISE`). The astronomy comes from the Rust rise/set and eclipse engines. What differs:

- **`rise_trans`** handles the Sun, Moon, planets and Chiron; a star name raises `swe.Error`. `res` is `-2` when the body does not rise or set within about a day (circumpolar). `atpress` (hPa, `0` = standard atmosphere at the altitude) and `attemp` (°C) scale the standard 34′ refraction; `BIT_FORCE_SLOW_METHOD` changes nothing. As in Swiss, the twilight bits apply to the Sun only and are ignored for other bodies. In `rise_trans_true_hor`, `horhgt=-100` gives the dip of the sea horizon seen from the altitude in `geopos` (1.76′·√metres, standard terrestrial refraction included), as in Swiss; any other `horhgt` outside -90°..90° raises `ValueError`. The `flags` argument (ephemeris choice) is accepted and ignored: XALEN always uses its own engine.
- **`sol_eclipse_when_glob`** returns greatest eclipse in `tret[0]`; the global contact times `tret[1:10]` are not computed and are `0.0`. Local contacts come from `sol_eclipse_when_loc`.
- **`sol_eclipse_when_loc`** fills `tret[0:7]` (maximum, C1–C4, sunrise and sunset during the eclipse) and `attr[0]` (magnitude), `attr[1]` (ratio of the Moon's to the Sun's apparent diameter), `attr[2]` (obscuration), `attr[5]` (true Sun altitude at maximum) and `attr[8]` (NASA's magnitude: `attr[0]` for a partial eclipse, `attr[1]` for a total or annular one); the shadow width, azimuth, apparent altitude and elongation slots are `0.0`.
- **`lun_eclipse_how`** fills `attr[0]`/`attr[8]` (umbral magnitude), `attr[1]` (penumbral magnitude), `attr[4]` (Moon azimuth, from the south point through west, as in Swiss), `attr[5]` (true Moon altitude), `attr[6]` (apparent altitude: Saemundsson refraction for the standard atmosphere at the altitude in `geopos` and 15 °C) and `attr[7]` (distance from the shadow axis). As in Swiss, its `retflag` is `0` when the apparent altitude is at or below 0, with `attr` still filled. `lun_eclipse_when_loc` sets contact times before moonrise (`tret[8]`) or after moonset (`tret[9]`) to `0.0`, as Swiss does; the `*_VISIBLE` bits say which contacts were above the horizon.
- Saros numbers are not computed: `attr[9]` and `attr[10]` are `-99999999`, pyswisseph's "unknown".

See [docs/MIGRATING_FROM_SWISS_EPH.md](https://github.com/vedika-io/xalen-ephemeris/blob/main/docs/MIGRATING_FROM_SWISS_EPH.md) for the full migration guide.

---

## Native `xalen` API (richer, structured returns)

When you are *not* porting Swiss code, the native `xalen` module gives you dicts instead of positional tuples.

### `planet_position` — the full 6-tuple + retrograde

The high-fidelity counterpart to `planet_longitude`. Equivalent to Swiss `calc_ut(..., FLG_SPEED)` plus a retrograde flag:

```python
import xalen

jd = xalen.julian_day(2000, 1, 1, 12.0)        # J2000.0
p = xalen.planet_position(jd, 0)               # body 0 = Sun, tropical
# {
#   "longitude": 280.37, "latitude": 0.0, "distance": 0.9833,   # AU
#   "lon_speed": 1.0194, "lat_speed": ..., "dist_speed": ...,   # per day
#   "is_retrograde": False,
# }

# Sidereal (Lahiri = ayanamsa 0): sidereal longitude AND the ayanamsa's own
# precession rate is removed from lon_speed (matches Swiss SIDEREAL|SPEED).
p_sid = xalen.planet_position(jd, 0, sidereal=True, ayanamsa=0)

# Ketu (id 13) = Rahu + 180°, sharing Rahu's speed/retrograde.
ketu = xalen.planet_position(jd, 13)
```

`longitude`, `latitude`, and the speeds are in degrees (speeds per day); `distance`/`dist_speed` are in AU. `longitude` is wrapped to `[0, 360)`; `is_retrograde` is taken from the tropical longitude rate regardless of the `sidereal` flag.

### Body IDs

`0`=Sun · `1`=Moon · `2`=Mercury · `3`=Venus · `4`=Mars · `5`=Jupiter · `6`=Saturn · `7`=Uranus · `8`=Neptune · `9`=MeanNode (Rahu) · `10`=TrueNode · `11`=Pluto · `12`=Chiron · `13`=Ketu (Rahu+180).

The ayanamsa and house-system IDs, and the Swiss Ephemeris numbers `xalen.swe` takes instead, are listed in [docs/BINDING_IDS.md](https://github.com/vedika-io/xalen-ephemeris/blob/main/docs/BINDING_IDS.md).

### Other native functions

```python
import xalen

jd = xalen.julian_day(1990, 6, 15, 10.5)

xalen.planet_longitude(jd, 1, sidereal=True, ayanamsa=0)  # just the longitude (float)
xalen.all_planets(jd, sidereal=True, ayanamsa=0)          # {"Sun":..,"Moon":.., "Ketu":..}

# Full Vedic chart: 9 grahas (+Ketu) with nakshatra/pada/rashi/lord, plus
# Whole-Sign ascendant, MC, ayanamsa_deg, and the 12 cusps.
chart = xalen.full_chart(jd, 18.52, 73.85, ayanamsa=0)
print(chart["planets"]["Sun"])     # {"longitude":.., "nakshatra":.., "pada":.., "rashi":.., "lord":..}
print(chart["ascendant"], chart["mc"], chart["ayanamsa_deg"])

# Panchang (five limbs) for a JD.
pan = xalen.panchang(jd, ayanamsa=0)
# {"tithi": {"number":..,"name":..,"paksha":"Shukla"|"Krishna"},
#  "nakshatra": "...", "yoga": {"number":..,"name":..}, "karana": "...", "vara": "..."}

# Nakshatra detail from a sidereal Moon longitude.
nak = xalen.nakshatra(123.45)      # {"name":.., "pada":.., "lord":.., "deity":.., "index":..}
xalen.rashi(123.45)                # e.g. "Simha (Leo)"

# Houses across 14 systems (0=WholeSign .. 13=Krusinski).
h = xalen.houses(jd, 18.52, 73.85, system=2)  # Placidus
# {"cusps":[12], "ascendant":.., "mc":.., "ic":.., "descendant":.., "vertex":..}

# Ayanamsa value (17 systems, 0=Lahiri).
xalen.ayanamsa(jd, system=0)
xalen.delta_t(jd)                  # ΔT (TT−UT1) seconds, SMH 2016 model
xalen.fixed_star_conjunctions(123.45, 1.0, 2000.0)

# Numerology.
xalen.life_path(1990, 6, 15)
xalen.expression_number("Ada Lovelace", "pythagorean")  # or "chaldean"

# String-named convenience variants.
xalen.planet_longitude_by_name("moon", jd)
xalen.sidereal_longitude("moon", jd, "lahiri")
xalen.houses_by_name(jd, 18.52, 73.85, "placidus")
xalen.ayanamsa_by_name(jd, "lahiri")
```

Accepted name strings: bodies `sun, moon, mercury, venus, earth, mars, jupiter, saturn, uranus, neptune, pluto, rahu`/`mean_node`, `true_node`, `chiron`, `lilith`/`mean_apogee`. Ayanamsas `lahiri, kp, raman, fagan-bradley, true-chitra, true-revati, surya-siddhanta, yukteswar, jn-bhasin, deluce, ushashashi, pushya-paksha, galactic-center, lahiri-icrc, kp-straight-line, hipparchos, lahiri-vp285`. House systems `whole-sign, equal, placidus, koch, porphyry, regiomontanus, campanus, morinus, alcabitius, topocentric, meridian, vehlow, sripati, krusinski-pisa` (dashes/spaces ignored, case-insensitive).

---

## Accuracy — what is and is not claimed

XALEN is cross-validated against JPL Horizons (DE440), the real DE440 binary kernel, the official VSOP87 check file, Swiss Ephemeris (`swetest`), and public calculators. The honest framing: XALEN **matches** these references to the bounds below — DE440 *is* the reference; we do not "beat" it. The differentiator is pure Rust, zero `unsafe` core, thread-safe, Apache-2.0, no data files.

| Body | Theory | Measured error vs JPL DE440 (analytical engine) |
|------|--------|--------------------------------------------------|
| Sun, Mercury–Saturn | VSOP87A + IAU 2000B nutation | **sub-arcsecond** (Sun 0.21″, Mercury–Saturn ≤ 0.76″; 20k-chart bound) |
| Uranus, Neptune | VSOP87A + nutation | Uranus 1.78″, Neptune 2.53″ (20k-chart bound) |
| Moon | ELP2000-82 (Meeus, 60+60 terms) | ~3″ RMS (max ~12″), AD 1600–2100 |
| Pluto | Meeus Goffin 43-term fit | ~1 arcminute, 1885–2099 |
| Rahu/Ketu (Mean Node) | Analytical mean model | exact (mean model) |

vs Swiss Ephemeris: **0 of 5,000,000** charts over **0.1°** for any planet or node (most < 0.02°), worldwide, 1850–2150. Loading the optional `de440s.bsp` kernel gives JPL-grade sub-arcsecond on the Sun + planets and full-range Pluto. Full report: [docs/ACCURACY.md](https://github.com/vedika-io/xalen-ephemeris/blob/main/docs/ACCURACY.md).

### Which Swiss flags are / are not supported

| Supported (shape + behavior) | **Not** supported — raises `ValueError` |
|------------------------------|------------------------------------------|
| `SEFLG_SWIEPH` (always the active backend) | `SEFLG_HELCTR` (heliocentric) |
| `SEFLG_SPEED` (daily motion) | `SEFLG_TOPOCTR` (topocentric) |
| `SEFLG_SIDEREAL` (with `set_sid_mode`) | `SEFLG_J2000` (J2000 frame) |
| | `SEFLG_EQUATORIAL` (RA/Dec) |
| | `SEFLG_BARYCTR` (barycentric) |
| | `SEFLG_XYZ` (cartesian) |
| | `SEFLG_RADIANS` |

All these constants are still *defined* on `xalen.swe` (so existing `import` lines and `flags = ...` arithmetic work); only their position-altering *effect* is unimplemented, and passing one to `calc_ut` raises rather than silently returning the wrong frame.

---

## Editor autocomplete

Type stubs ship with the wheel (`xalen.pyi`, `xalen/swe.pyi`), so `pyswisseph` and native users both get full autocomplete and type checking in editors and `mypy`.

---

## License

Apache-2.0. See [LICENSE](https://github.com/vedika-io/xalen-ephemeris/blob/main/LICENSE) and [CREDITS.md](https://github.com/vedika-io/xalen-ephemeris/blob/main/CREDITS.md).
