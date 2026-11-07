"""Acceptance tests for rise/set/transit, eclipse search and ``swe.Error`` in
the ``xalen.swe`` pyswisseph drop-in (issues #47 and #48).

Reference values and where they come from:

* ``ENGINE_*``: the Rust engine's own answer, printed by
  ``xalen rise-set --jd <start> --lat 28.6139 --lon 77.2090 --alt 0 --json``
  (24-hour window from the start instant). ``rise_trans`` must agree to 1 s.
* ``USNO_*``: the U.S. Naval Observatory one-day almanac
  (aa.usno.navy.mil/api/rstt/oneday, coords 28.6139,77.2090, tz 5.5), which
  rounds to the minute. ``rise_trans`` must agree to 2 minutes.
* Eclipse instants and magnitudes: NASA eclipse catalogues (2024-04-08 total,
  greatest eclipse about 18:17 UT; 2024-10-02 annular, about 18:45 UT;
  2022-11-08 total lunar, greatest about 10:59 UT, umbral magnitude 1.359;
  2024-03-25 penumbral, about 07:13 UT; Dallas totality on 2024-04-08 from
  about 18:40:43 to 18:44:35 UT).

Tuple lengths, return codes and bit values follow the pyswisseph 2.10 API.
"""

import math
import re
from pathlib import Path

import pytest

xalen = pytest.importorskip("xalen", reason="build the extension first (maturin develop)")
import xalen.swe as swe  # noqa: E402

DELHI = (77.2090, 28.6139, 0.0)  # geopos order: lon, lat, alt (m)
DALLAS = (-96.80, 32.78, 0.0)
SECOND = 1.0 / 86400.0
MINUTE = 1.0 / 1440.0

# Rust engine (CLI) results, JD UT, for the 24 h from 00:00 IST
# (`--jd 2460482.2708333333` and `--jd 2460665.2708333333`).
ENGINE_SUN_2024_06_21 = {"rise": 2460482.495832188, "set": 2460483.0778301693}
ENGINE_SUN_2024_12_21 = {"rise": 2460665.5692535527, "set": 2460665.999284885}


def jd_ist(y, m, d, hh, mm):
    """Julian Day (UT) of a wall-clock time in India Standard Time (UTC+5:30)."""
    return swe.julday(y, m, d, hh + mm / 60.0 - 5.5)


def rise(jd, body, rsmi, geopos=DELHI, **kw):
    res, tret = swe.rise_trans(jd, body, rsmi, geopos, **kw)
    assert res == 0, f"rise_trans returned {res}"
    assert len(tret) == 10
    return tret[0]


# ---------------------------------------------------------------------------
# swe.Error (#48)
# ---------------------------------------------------------------------------

def test_error_class_is_exported_and_is_a_runtime_error():
    assert hasattr(swe, "Error")
    assert issubclass(swe.Error, RuntimeError)
    assert issubclass(swe.Error, Exception)


def test_unavailable_body_raises_swe_error_in_calc_ut():
    with pytest.raises(swe.Error):
        swe.calc_ut(2451545.0, 999, swe.FLG_SWIEPH)
    # Downstreams that catch RuntimeError keep working too.
    with pytest.raises(RuntimeError):
        swe.calc_ut(2451545.0, 999, swe.FLG_SWIEPH)


def test_optional_body_pattern_from_issue_48():
    # `try: ... except swe.Error:` around an optional body must not crash.
    try:
        swe.calc_ut(2451545.0, 999)
        reached = True
    except swe.Error:
        reached = False
    assert reached is False


def test_unknown_star_and_body_raise_swe_error():
    with pytest.raises(swe.Error):
        swe.fixstar2_ut("NoSuchStarAnywhere", 2451545.0)
    with pytest.raises(swe.Error):
        swe.rise_trans(2460482.5, 999, swe.CALC_RISE, DELHI)


def test_bad_argument_types_raise_type_error():
    with pytest.raises(TypeError):
        swe.rise_trans(2460482.5, swe.SUN, swe.CALC_RISE, "Delhi")
    with pytest.raises(TypeError):
        swe.rise_trans(2460482.5, swe.SUN, swe.CALC_RISE, (77.2, 28.6))
    with pytest.raises(TypeError):
        swe.sol_eclipse_when_loc(2460400.5, None)


# ---------------------------------------------------------------------------
# Constants (#47)
# ---------------------------------------------------------------------------

def test_rise_trans_constants_match_pyswisseph():
    assert (swe.CALC_RISE, swe.CALC_SET, swe.CALC_MTRANSIT, swe.CALC_ITRANSIT) == (1, 2, 4, 8)
    assert swe.BIT_DISC_CENTER == 256
    assert swe.BIT_DISC_BOTTOM == 8192
    assert swe.BIT_NO_REFRACTION == 512
    assert swe.BIT_CIVIL_TWILIGHT == 1024
    assert swe.BIT_NAUTIC_TWILIGHT == 2048
    assert swe.BIT_ASTRO_TWILIGHT == 4096
    assert swe.BIT_FIXED_DISC_SIZE == 16384
    assert swe.BIT_FORCE_SLOW_METHOD == 32768
    assert swe.BIT_HINDU_RISING == 896
    assert swe.SE_CALC_RISE == 1 and swe.SE_BIT_DISC_CENTER == 256


def test_eclipse_constants_match_pyswisseph():
    expected = {
        "ECL_NUT": -1,
        "ECL_CENTRAL": 1,
        "ECL_NONCENTRAL": 2,
        "ECL_TOTAL": 4,
        "ECL_ANNULAR": 8,
        "ECL_PARTIAL": 16,
        "ECL_ANNULAR_TOTAL": 32,
        "ECL_HYBRID": 32,
        "ECL_PENUMBRAL": 64,
        "ECL_ALLTYPES_SOLAR": 63,
        "ECL_ALLTYPES_LUNAR": 84,
        "ECL_VISIBLE": 128,
        "ECL_MAX_VISIBLE": 256,
        "ECL_1ST_VISIBLE": 512,
        "ECL_PARTBEG_VISIBLE": 512,
        "ECL_2ND_VISIBLE": 1024,
        "ECL_TOTBEG_VISIBLE": 1024,
        "ECL_3RD_VISIBLE": 2048,
        "ECL_TOTEND_VISIBLE": 2048,
        "ECL_4TH_VISIBLE": 4096,
        "ECL_PARTEND_VISIBLE": 4096,
        "ECL_PENUMBBEG_VISIBLE": 8192,
        "ECL_PENUMBEND_VISIBLE": 16384,
        "ECL_OCC_BEG_DAYLIGHT": 8192,
        "ECL_OCC_END_DAYLIGHT": 16384,
        "ECL_ONE_TRY": 32768,
    }
    for name, value in expected.items():
        assert getattr(swe, name) == value, name
        assert getattr(swe, "SE_" + name) == value, "SE_" + name
    assert swe.FLG_MOSEPH == 4 and swe.FLG_JPLEPH == 1


def test_ecl_nut_in_calc_ut_returns_obliquity_and_nutation():
    xx, _ = swe.calc_ut(2451545.0, swe.ECL_NUT)
    assert len(xx) == 6
    assert abs(xx[1] - 23.439279) < 1e-4  # IAU 2006 mean obliquity at J2000
    assert abs(xx[0] - (xx[1] + xx[3])) < 1e-12
    assert abs(xx[2]) < 20.0 / 3600.0


# ---------------------------------------------------------------------------
# rise_trans / rise_trans_true_hor (#47)
# ---------------------------------------------------------------------------

@pytest.mark.parametrize(
    "start, engine, usno_rise, usno_set",
    [
        (
            jd_ist(2024, 6, 21, 0, 0),
            ENGINE_SUN_2024_06_21,
            jd_ist(2024, 6, 21, 5, 24),
            jd_ist(2024, 6, 21, 19, 22),
        ),
        (
            jd_ist(2024, 12, 21, 0, 0),
            ENGINE_SUN_2024_12_21,
            jd_ist(2024, 12, 21, 7, 10),
            jd_ist(2024, 12, 21, 17, 29),
        ),
    ],
)
def test_sun_rise_and_set_at_new_delhi(start, engine, usno_rise, usno_set):
    # atpress/attemp at the standard values make the refraction exactly the
    # engine's 34', so the result must be the CLI's to the second.
    r = rise(start, swe.SUN, swe.CALC_RISE, atpress=1013.25, attemp=15.0)
    s = rise(start, swe.SUN, swe.CALC_SET, atpress=1013.25, attemp=15.0)
    assert abs(r - engine["rise"]) < SECOND, (r - engine["rise"]) * 86400
    assert abs(s - engine["set"]) < SECOND, (s - engine["set"]) * 86400
    assert abs(r - usno_rise) < 2 * MINUTE, (r - usno_rise) * 1440
    assert abs(s - usno_set) < 2 * MINUTE, (s - usno_set) * 1440
    # Default pressure/temperature (0, 0) only nudges refraction: seconds.
    r0 = rise(start, swe.SUN, swe.CALC_RISE)
    assert abs(r0 - r) < 30 * SECOND


def test_sun_transit_at_new_delhi():
    t = rise(jd_ist(2024, 6, 21, 0, 0), swe.SUN, swe.CALC_MTRANSIT)
    assert abs(t - jd_ist(2024, 6, 21, 12, 23)) < 2 * MINUTE


@pytest.mark.parametrize(
    "start, usno",
    [
        # USNO: Moon upper transit 00:08 IST on 22 June, 05:03 IST on 21 Dec.
        (jd_ist(2024, 6, 21, 17, 30), jd_ist(2024, 6, 22, 0, 8)),
        (jd_ist(2024, 12, 21, 0, 0), jd_ist(2024, 12, 21, 5, 3)),
    ],
)
def test_moon_transit_at_new_delhi(start, usno):
    t = rise(start, swe.MOON, swe.CALC_MTRANSIT)
    assert abs(t - usno) < 2 * MINUTE, (t - usno) * 1440


def test_lower_transit_is_half_a_day_after_upper():
    start = jd_ist(2024, 6, 21, 0, 0)
    upper = rise(start, swe.SUN, swe.CALC_MTRANSIT)
    lower = rise(upper, swe.SUN, swe.CALC_ITRANSIT)
    assert abs((lower - upper) * 24.0 - 12.0) < 0.05


def test_circumpolar_sun_returns_minus_two():
    # 80 N in June: the Sun neither rises nor sets. pyswisseph returns -2.
    start = swe.julday(2024, 6, 21, 0.0)
    for rsmi in (swe.CALC_RISE, swe.CALC_SET):
        res, tret = swe.rise_trans(start, swe.SUN, rsmi, (15.0, 80.0, 0.0))
        assert res == -2
        assert len(tret) == 10
    # A transit still happens every day.
    res, tret = swe.rise_trans(start, swe.SUN, swe.CALC_MTRANSIT, (15.0, 80.0, 0.0))
    assert res == 0 and tret[0] > start


def test_disc_center_no_refraction_moves_rise_later_and_set_earlier():
    start = jd_ist(2024, 6, 21, 0, 0)
    flags = swe.BIT_DISC_CENTER | swe.BIT_NO_REFRACTION
    r_std = rise(start, swe.SUN, swe.CALC_RISE)
    s_std = rise(start, swe.SUN, swe.CALC_SET)
    r_geo = rise(start, swe.SUN, swe.CALC_RISE | flags)
    s_geo = rise(start, swe.SUN, swe.CALC_SET | flags)
    # The rise altitude goes from about -0.83 deg to 0 deg: 3 to 5 minutes.
    assert 2.5 * MINUTE < r_geo - r_std < 5.0 * MINUTE
    assert 2.5 * MINUTE < s_std - s_geo < 5.0 * MINUTE
    # Lower limb rises after the centre, which rises after the upper limb.
    r_low = rise(start, swe.SUN, swe.CALC_RISE | swe.BIT_DISC_BOTTOM)
    r_mid = rise(start, swe.SUN, swe.CALC_RISE | swe.BIT_DISC_CENTER)
    assert r_std < r_mid < r_low


def test_civil_twilight_bit_gives_dawn():
    # USNO: civil twilight begins 04:57 IST on 22 June 2024 at Delhi.
    dawn = rise(jd_ist(2024, 6, 22, 0, 0), swe.SUN, swe.CALC_RISE | swe.BIT_CIVIL_TWILIGHT)
    assert abs(dawn - jd_ist(2024, 6, 22, 4, 57)) < 2 * MINUTE


def test_rise_trans_true_hor_with_raised_horizon():
    start = jd_ist(2024, 6, 21, 0, 0)
    base = rise(start, swe.SUN, swe.CALC_RISE, atpress=1013.25, attemp=15.0)
    res, tret = swe.rise_trans_true_hor(
        start, swe.SUN, swe.CALC_RISE, DELHI, 1013.25, 15.0, 0.0
    )
    assert res == 0 and len(tret) == 10
    assert abs(tret[0] - base) < 0.01 * SECOND  # zero horizon == rise_trans
    res, tret = swe.rise_trans_true_hor(
        start, swe.SUN, swe.CALC_RISE, DELHI, 1013.25, 15.0, 2.0
    )
    assert res == 0
    # A 2 deg ridge hides the Sun for roughly 10 more minutes.
    assert 5 * MINUTE < tret[0] - base < 20 * MINUTE
    # Keyword form, as pyswisseph names the argument.
    res, tret2 = swe.rise_trans_true_hor(
        start, swe.SUN, swe.CALC_SET, DELHI, horhgt=2.0
    )
    assert res == 0
    assert tret2[0] < rise(start, swe.SUN, swe.CALC_SET)


def test_rise_trans_docstring():
    assert swe.rise_trans.__doc__ and "rise_trans" in swe.rise_trans.__doc__
    assert swe.lun_eclipse_how.__doc__


# ---------------------------------------------------------------------------
# Solar eclipses (#47)
# ---------------------------------------------------------------------------

def test_sol_eclipse_when_glob_finds_2024_april_total():
    retflag, tret = swe.sol_eclipse_when_glob(swe.julday(2024, 1, 1, 0.0))
    assert len(tret) == 10
    assert retflag == swe.ECL_TOTAL | swe.ECL_CENTRAL
    assert abs(tret[0] - swe.julday(2024, 4, 8, 18 + 17 / 60)) < 2 * MINUTE


def test_sol_eclipse_when_glob_finds_2024_october_annular():
    retflag, tret = swe.sol_eclipse_when_glob(swe.julday(2024, 5, 1, 0.0))
    assert retflag == swe.ECL_ANNULAR | swe.ECL_CENTRAL
    assert abs(tret[0] - swe.julday(2024, 10, 2, 18 + 45 / 60)) < 5 * MINUTE


def test_sol_eclipse_when_glob_type_filter_and_backward():
    # Asking for an annular eclipse from 2024-01-01 skips the April total.
    retflag, tret = swe.sol_eclipse_when_glob(
        swe.julday(2024, 1, 1, 0.0), swe.FLG_SWIEPH, swe.ECL_ANNULAR
    )
    assert retflag & swe.ECL_ANNULAR
    assert swe.revjul(tret[0])[:3] == (2024, 10, 2)
    # Backward from 2024-05-01 finds 2024-04-08 again.
    retflag, tret = swe.sol_eclipse_when_glob(swe.julday(2024, 5, 1, 0.0), backwards=True)
    assert retflag & swe.ECL_TOTAL
    assert swe.revjul(tret[0])[:3] == (2024, 4, 8)


def test_sol_eclipse_when_loc_dallas_totality():
    retflag, tret, attr = swe.sol_eclipse_when_loc(swe.julday(2024, 4, 1, 0.0), DALLAS)
    assert len(tret) == 10 and len(attr) == 20
    assert retflag & swe.ECL_TOTAL
    assert retflag & swe.ECL_VISIBLE
    assert retflag & swe.ECL_MAX_VISIBLE
    for bit in (swe.ECL_1ST_VISIBLE, swe.ECL_2ND_VISIBLE, swe.ECL_3RD_VISIBLE, swe.ECL_4TH_VISIBLE):
        assert retflag & bit
    c_max, c1, c2, c3, c4 = tret[0], tret[1], tret[2], tret[3], tret[4]
    assert c1 < c2 < c_max < c3 < c4
    # NASA: totality 18:40:43 - 18:44:35 UT, partial 17:23 - 20:02 UT.
    assert abs(c2 - swe.julday(2024, 4, 8, 18 + 40 / 60 + 43 / 3600)) < 3 * MINUTE
    assert abs(c3 - swe.julday(2024, 4, 8, 18 + 44 / 60 + 35 / 3600)) < 3 * MINUTE
    assert abs(c1 - swe.julday(2024, 4, 8, 17 + 23 / 60)) < 5 * MINUTE
    assert abs(c4 - swe.julday(2024, 4, 8, 20 + 2 / 60)) < 5 * MINUTE
    assert 3.0 * MINUTE < c3 - c2 < 4.5 * MINUTE
    assert attr[0] >= 1.0  # magnitude
    assert attr[2] > 0.999  # obscuration
    assert attr[5] > 50.0  # Sun high over Dallas at maximum


# ---------------------------------------------------------------------------
# Lunar eclipses (#47)
# ---------------------------------------------------------------------------

def test_lun_eclipse_when_finds_2022_november_total():
    retflag, tret = swe.lun_eclipse_when(swe.julday(2022, 10, 1, 0.0))
    assert len(tret) == 10
    assert retflag == swe.ECL_TOTAL
    assert abs(tret[0] - swe.julday(2022, 11, 8, 10 + 59 / 60)) < 3 * MINUTE
    p1, u1, u2, mx, u3, u4, p4 = tret[6], tret[2], tret[4], tret[0], tret[5], tret[3], tret[7]
    assert p1 < u1 < u2 < mx < u3 < u4 < p4


def test_lun_eclipse_when_finds_2024_march_penumbral():
    retflag, tret = swe.lun_eclipse_when(swe.julday(2024, 3, 1, 0.0))
    assert retflag == swe.ECL_PENUMBRAL
    assert abs(tret[0] - swe.julday(2024, 3, 25, 7 + 13 / 60)) < 5 * MINUTE
    # No umbral phase: those contacts are 0, the penumbral ones are set.
    assert tret[2] == tret[3] == tret[4] == tret[5] == 0.0
    assert tret[6] < tret[0] < tret[7]


def test_lun_eclipse_when_backward_and_type_filter():
    retflag, tret = swe.lun_eclipse_when(swe.julday(2022, 10, 1, 0.0), swe.FLG_SWIEPH, 0, True)
    assert retflag == swe.ECL_TOTAL
    assert swe.revjul(tret[0])[:3] == (2022, 5, 16)
    # Only total eclipses: from 2023 the next one is 2025-03-14.
    retflag, tret = swe.lun_eclipse_when(swe.julday(2023, 1, 1, 0.0), ecltype=swe.ECL_TOTAL)
    assert retflag == swe.ECL_TOTAL
    assert swe.revjul(tret[0])[:3] == (2025, 3, 14)


def test_lun_eclipse_how_2022_november_at_delhi():
    _, tret = swe.lun_eclipse_when(swe.julday(2022, 10, 1, 0.0))
    retflag, attr = swe.lun_eclipse_how(tret[0], DELHI)
    assert len(attr) == 20
    assert retflag == swe.ECL_TOTAL
    assert abs(attr[0] - 1.36) < 0.02  # umbral magnitude (NASA 1.359)
    assert 2.3 < attr[1] < 2.5  # penumbral magnitude (NASA 2.416)
    assert attr[8] == attr[0]
    # A week earlier there is no eclipse: retflag 0.
    retflag, attr = swe.lun_eclipse_how(swe.julday(2022, 11, 1, 0.0), DELHI)
    assert retflag == 0
    assert len(attr) == 20


def test_lun_eclipse_when_loc_2022_november_rises_eclipsed_at_delhi():
    # The Moon rose at Delhi during the partial phase after totality.
    retflag, tret, attr = swe.lun_eclipse_when_loc(swe.julday(2022, 10, 1, 0.0), DELHI)
    assert len(tret) == 10 and len(attr) == 20
    assert retflag & swe.ECL_TOTAL
    assert retflag & swe.ECL_VISIBLE
    assert retflag & swe.ECL_PARTEND_VISIBLE
    assert not retflag & swe.ECL_MAX_VISIBLE
    assert swe.revjul(tret[0])[:3] == (2022, 11, 8)
    assert tret[6] < tret[8] < tret[7]  # moonrise during the eclipse
    assert abs(attr[0] - 1.36) < 0.02


# ---------------------------------------------------------------------------
# Stubs cover every public name
# ---------------------------------------------------------------------------

def test_stub_file_covers_every_public_name():
    stub = (Path(__file__).resolve().parents[1] / "xalen" / "swe.pyi").read_text()
    declared = set(re.findall(r"^(?:def |class )?([A-Za-z_][A-Za-z0-9_]*)\b", stub, re.M))
    missing = [n for n in dir(swe) if not n.startswith("_") and n not in declared]
    assert not missing, f"names missing from xalen/swe.pyi: {missing}"


# ---------------------------------------------------------------------------
# Cross-check against pyswisseph itself, when it is installed.
# ---------------------------------------------------------------------------

def test_agrees_with_pyswisseph_when_installed():
    pyswe = pytest.importorskip("swisseph", reason="pyswisseph not installed")
    pyswe.set_ephe_path(None)
    start = jd_ist(2024, 6, 21, 0, 0)
    for rsmi in (pyswe.CALC_RISE, pyswe.CALC_SET):
        ref = pyswe.rise_trans(start, pyswe.SUN, rsmi, DELHI, 1013.25, 15.0, pyswe.FLG_MOSEPH)
        got = swe.rise_trans(start, swe.SUN, rsmi, DELHI, 1013.25, 15.0)
        assert ref[0] == got[0]
        assert abs(ref[1][0] - got[1][0]) < 60 * SECOND
    ref = pyswe.sol_eclipse_when_glob(swe.julday(2024, 1, 1, 0.0), pyswe.FLG_MOSEPH)
    got = swe.sol_eclipse_when_glob(swe.julday(2024, 1, 1, 0.0))
    assert ref[0] == got[0]
    assert abs(ref[1][0] - got[1][0]) < 2 * MINUTE
    ref = pyswe.lun_eclipse_when(swe.julday(2022, 10, 1, 0.0), pyswe.FLG_MOSEPH)
    got = swe.lun_eclipse_when(swe.julday(2022, 10, 1, 0.0))
    assert ref[0] == got[0]
    assert abs(ref[1][0] - got[1][0]) < 2 * MINUTE
    assert math.isfinite(got[1][0])
