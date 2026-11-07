# Type stubs for `xalen.swe` — the pyswisseph (`swisseph`) drop-in submodule.
#
# Mirrors the #[pyfunction] surface in src/swe_compat.rs. Function shapes match
# pyswisseph (argument order, tuple layout). Constants are exposed under BOTH
# the bare pyswisseph spellings (SUN, FLG_SWIEPH, SIDM_LAHIRI) and SE_-prefixed
# spellings (SE_SUN, SEFLG_SWIEPH, SE_SIDM_LAHIRI).
#
# Honesty notes (see the module docstring / README):
#  * speeds in calc_ut are 0.0 unless SEFLG_SPEED is set.
#  * houses_ex ascmc has length 8; all eight slots are populated:
#    [0..4] = asc, mc, armc, vertex; [4..8] = equatorial ascendant, co-ascendant
#    (Koch), co-ascendant (Munkasey), polar ascendant.
#  * houses_ex honors SEFLG_SIDEREAL in flags (cusps/angles shifted by ayanamsa).
#  * position-altering flags XALEN does not implement (HELCTR, TOPOCTR, J2000,
#    EQUATORIAL, BARYCTR, XYZ, RADIANS) raise ValueError when passed to calc_ut.
#  * set_ephe_path() and close() are no-ops (data is embedded at compile time).
#  * Every calculation failure raises swe.Error (a RuntimeError); TypeError and
#    ValueError are raised only for bad arguments.
#  * rise_trans / eclipse functions return pyswisseph's tuple shapes; slots the
#    engine does not compute are 0.0 (see each docstring).

from typing import Any, Optional, Sequence, Tuple, Union

_Tret = Tuple[float, float, float, float, float, float, float, float, float, float]
_Attr = Tuple[
    float, float, float, float, float, float, float, float, float, float,
    float, float, float, float, float, float, float, float, float, float,
]

class Error(RuntimeError):
    """Calculation failure (pyswisseph's swe.Error)."""
    ...

# --- functions -------------------------------------------------------------

def calc_ut(
    jd_ut: float, ipl: int, flags: int = ...
) -> Tuple[Tuple[float, float, float, float, float, float], int]:
    """((lon, lat, dist, lon_speed, lat_speed, dist_speed), ret_flag)."""
    ...

def calc(
    jd_et: float, ipl: int, flags: int = ...
) -> Tuple[Tuple[float, float, float, float, float, float], int]:
    """ET/TT-input sibling of calc_ut. Input is treated as TT and converted to
    UT1 via ΔT, so swe.calc(jd_tt) == swe.calc_ut(jd_tt - deltat(jd_tt))."""
    ...

def houses_ex(
    jd_ut: float, lat: float, lon: float, hsys: Union[str, bytes] = ..., flags: int = ...
) -> Tuple[
    Tuple[float, float, float, float, float, float, float, float, float, float, float, float],
    Tuple[float, float, float, float, float, float, float, float],
]:
    """(cusps[12], ascmc[8]); ascmc = (asc, mc, armc, vertex, equatorial_ascendant,
    co_ascendant_koch, co_ascendant_munkasey, polar_ascendant). flags may include
    SEFLG_SIDEREAL to return the sidereal frame (active-mode ayanamsa subtracted)."""
    ...

def houses(
    jd_ut: float, lat: float, lon: float, hsys: Union[str, bytes] = ...
) -> Tuple[
    Tuple[float, float, float, float, float, float, float, float, float, float, float, float],
    Tuple[float, float, float, float, float, float, float, float],
]: ...

def get_ayanamsa_ex_ut(jd_ut: float, flags: int = ...) -> Tuple[int, float]: ...
def get_ayanamsa_ut(jd_ut: float) -> float: ...
def set_sid_mode(sidmode: int, t0: float = ..., ayan_t0: float = ...) -> None: ...
def julday(year: int, month: int, day: int, hour: float = ..., cal: int = ...) -> float: ...
def revjul(jd: float, cal: int = ...) -> Tuple[int, int, int, float]: ...
def deltat(jd: float) -> float: ...

def fixstar2_ut(
    star: str, jd_ut: float, flags: int = ...
) -> Tuple[Tuple[float, float, float, float, float, float], str, int]:
    """((lon, lat, dist, lon_speed, lat_speed, dist_speed), name, ret_flag).
    Catalog has no parallax/proper-motion velocity, so distance + speeds are 0.0."""
    ...

def fixstar_ut(
    star: str, jd_ut: float, flags: int = ...
) -> Tuple[Tuple[float, float, float, float, float, float], str, int]: ...

def fixstar2_mag(star: str) -> Tuple[float, str]:
    """(visual_magnitude, name)."""
    ...

def set_ephe_path(path: Optional[str] = ...) -> None:
    """No-op. XALEN embeds all data at compile time."""
    ...

def close() -> None:
    """No-op. XALEN holds no resources to release."""
    ...

# --- rise, set and transit -------------------------------------------------

def rise_trans(
    tjdut: float,
    body: Union[int, str],
    rsmi: int,
    geopos: Sequence[float],
    atpress: float = ...,
    attemp: float = ...,
    flags: int = ...,
) -> Tuple[int, _Tret]:
    """(res, tret): res 0 = found (tret[0] = JD UT), -2 = circumpolar.
    geopos = (lon, lat, alt_m). rsmi = CALC_* | BIT_*; twilight bits apply to
    the Sun only. Star names raise Error."""
    ...

def rise_trans_true_hor(
    tjdut: float,
    body: Union[int, str],
    rsmi: int,
    geopos: Sequence[float],
    atpress: float = ...,
    attemp: float = ...,
    horhgt: float = ...,
    flags: int = ...,
) -> Tuple[int, _Tret]:
    """rise_trans with the local horizon at horhgt degrees (-90..90);
    horhgt=-100 = dip of the sea horizon from the altitude in geopos."""
    ...

# --- eclipses --------------------------------------------------------------

def sol_eclipse_when_glob(
    tjdut: float, flags: int = ..., ecltype: int = ..., backwards: Any = ...
) -> Tuple[int, _Tret]:
    """(retflag, tret); tret[0] = greatest eclipse (JD UT), tret[1:] = 0.0."""
    ...

def sol_eclipse_when_loc(
    tjdut: float, geopos: Sequence[float], flags: int = ..., backwards: Any = ...
) -> Tuple[int, _Tret, _Attr]:
    """(retflag, tret, attr): tret = max, C1..C4, sunrise, sunset;
    attr[0] magnitude, [1] Moon/Sun diameter ratio, [2] obscuration,
    [5] Sun altitude, [8] NASA magnitude ([0] if partial, else [1])."""
    ...

def lun_eclipse_when(
    tjdut: float, flags: int = ..., ecltype: int = ..., backwards: Any = ...
) -> Tuple[int, _Tret]:
    """(retflag, tret): max, -, U1, U4, U2, U3, P1, P4, -, -."""
    ...

def lun_eclipse_when_loc(
    tjdut: float, geopos: Sequence[float], flags: int = ..., backwards: Any = ...
) -> Tuple[int, _Tret, _Attr]:
    """(retflag, tret, attr); tret[8]/[9] = moonrise/moonset during the eclipse;
    contacts before moonrise or after moonset are 0.0."""
    ...

def lun_eclipse_how(
    tjdut: float, geopos: Sequence[float], flags: int = ...
) -> Tuple[int, _Attr]:
    """(retflag, attr): attr[0] umbral mag, [1] penumbral mag, [4] Moon azimuth
    (from south through west), [5] true and [6] apparent Moon altitude,
    [7] distance from the shadow axis, [8] umbral mag. retflag 0 when the
    Moon is not in the penumbra or its apparent altitude is <= 0."""
    ...

# --- planet constants (both spellings) -------------------------------------

SUN: int
MOON: int
MERCURY: int
VENUS: int
MARS: int
JUPITER: int
SATURN: int
URANUS: int
NEPTUNE: int
PLUTO: int
MEAN_NODE: int
TRUE_NODE: int
MEAN_APOG: int
OSCU_APOG: int
CHIRON: int
EARTH: int

SE_SUN: int
SE_MOON: int
SE_MERCURY: int
SE_VENUS: int
SE_MARS: int
SE_JUPITER: int
SE_SATURN: int
SE_URANUS: int
SE_NEPTUNE: int
SE_PLUTO: int
SE_MEAN_NODE: int
SE_TRUE_NODE: int
SE_MEAN_APOG: int
SE_OSCU_APOG: int
SE_CHIRON: int
SE_EARTH: int

# --- flags (both FLG_* and SEFLG_* spellings) ------------------------------

FLG_SWIEPH: int
FLG_SPEED: int
FLG_SIDEREAL: int
FLG_HELCTR: int
FLG_J2000: int
FLG_EQUATORIAL: int
FLG_BARYCTR: int
FLG_TOPOCTR: int
FLG_XYZ: int
FLG_RADIANS: int

SEFLG_SWIEPH: int
SEFLG_SPEED: int
SEFLG_SIDEREAL: int
SEFLG_HELCTR: int
SEFLG_J2000: int
SEFLG_EQUATORIAL: int
SEFLG_BARYCTR: int
SEFLG_TOPOCTR: int
SEFLG_XYZ: int
SEFLG_RADIANS: int

# --- sidereal-mode constants (both spellings) ------------------------------

SIDM_FAGAN_BRADLEY: int
SIDM_LAHIRI: int
SIDM_DELUCE: int
SIDM_RAMAN: int
SIDM_USHASHASHI: int
SIDM_KRISHNAMURTI: int
SIDM_DJWHAL_KHUL: int
SIDM_YUKTESWAR: int
SIDM_JN_BHASIN: int
SIDM_TRUE_CITRA: int
SIDM_TRUE_REVATI: int

SE_SIDM_FAGAN_BRADLEY: int
SE_SIDM_LAHIRI: int
SE_SIDM_DELUCE: int
SE_SIDM_RAMAN: int
SE_SIDM_USHASHASHI: int
SE_SIDM_KRISHNAMURTI: int
SE_SIDM_DJWHAL_KHUL: int
SE_SIDM_YUKTESWAR: int
SE_SIDM_JN_BHASIN: int
SE_SIDM_TRUE_CITRA: int
SE_SIDM_TRUE_REVATI: int

# --- calendar constants ----------------------------------------------------

GREG_CAL: int
SE_GREG_CAL: int
JUL_CAL: int
SE_JUL_CAL: int

# --- ephemeris-selection flags (accepted; XALEN always uses its engine) ----

FLG_JPLEPH: int
FLG_MOSEPH: int
SEFLG_JPLEPH: int
SEFLG_MOSEPH: int

# --- rise_trans selectors and modifiers (both spellings) -----------------

CALC_RISE: int
CALC_SET: int
CALC_MTRANSIT: int
CALC_ITRANSIT: int
BIT_DISC_CENTER: int
BIT_DISC_BOTTOM: int
BIT_NO_REFRACTION: int
BIT_CIVIL_TWILIGHT: int
BIT_NAUTIC_TWILIGHT: int
BIT_ASTRO_TWILIGHT: int
BIT_FIXED_DISC_SIZE: int
BIT_FORCE_SLOW_METHOD: int
BIT_HINDU_RISING: int

SE_CALC_RISE: int
SE_CALC_SET: int
SE_CALC_MTRANSIT: int
SE_CALC_ITRANSIT: int
SE_BIT_DISC_CENTER: int
SE_BIT_DISC_BOTTOM: int
SE_BIT_NO_REFRACTION: int
SE_BIT_CIVIL_TWILIGHT: int
SE_BIT_NAUTIC_TWILIGHT: int
SE_BIT_ASTRO_TWILIGHT: int
SE_BIT_FIXED_DISC_SIZE: int
SE_BIT_FORCE_SLOW_METHOD: int
SE_BIT_HINDU_RISING: int

# --- eclipse type and visibility bits (both spellings) -------------------

ECL_NUT: int
ECL_CENTRAL: int
ECL_NONCENTRAL: int
ECL_TOTAL: int
ECL_ANNULAR: int
ECL_PARTIAL: int
ECL_ANNULAR_TOTAL: int
ECL_HYBRID: int
ECL_PENUMBRAL: int
ECL_ALLTYPES_SOLAR: int
ECL_ALLTYPES_LUNAR: int
ECL_VISIBLE: int
ECL_MAX_VISIBLE: int
ECL_1ST_VISIBLE: int
ECL_PARTBEG_VISIBLE: int
ECL_2ND_VISIBLE: int
ECL_TOTBEG_VISIBLE: int
ECL_3RD_VISIBLE: int
ECL_TOTEND_VISIBLE: int
ECL_4TH_VISIBLE: int
ECL_PARTEND_VISIBLE: int
ECL_PENUMBBEG_VISIBLE: int
ECL_PENUMBEND_VISIBLE: int
ECL_OCC_BEG_DAYLIGHT: int
ECL_OCC_END_DAYLIGHT: int
ECL_ONE_TRY: int

SE_ECL_NUT: int
SE_ECL_CENTRAL: int
SE_ECL_NONCENTRAL: int
SE_ECL_TOTAL: int
SE_ECL_ANNULAR: int
SE_ECL_PARTIAL: int
SE_ECL_ANNULAR_TOTAL: int
SE_ECL_HYBRID: int
SE_ECL_PENUMBRAL: int
SE_ECL_ALLTYPES_SOLAR: int
SE_ECL_ALLTYPES_LUNAR: int
SE_ECL_VISIBLE: int
SE_ECL_MAX_VISIBLE: int
SE_ECL_1ST_VISIBLE: int
SE_ECL_PARTBEG_VISIBLE: int
SE_ECL_2ND_VISIBLE: int
SE_ECL_TOTBEG_VISIBLE: int
SE_ECL_3RD_VISIBLE: int
SE_ECL_TOTEND_VISIBLE: int
SE_ECL_4TH_VISIBLE: int
SE_ECL_PARTEND_VISIBLE: int
SE_ECL_PENUMBBEG_VISIBLE: int
SE_ECL_PENUMBEND_VISIBLE: int
SE_ECL_OCC_BEG_DAYLIGHT: int
SE_ECL_OCC_END_DAYLIGHT: int
SE_ECL_ONE_TRY: int
