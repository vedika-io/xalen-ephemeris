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

from typing import Optional, Tuple, Union

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
