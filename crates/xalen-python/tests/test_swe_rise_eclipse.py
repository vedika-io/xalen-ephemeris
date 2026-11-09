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
  about 18:40:43 to 18:44:35 UT; New York partial on 2024-04-08; 2025-03-29
  partial, greatest about 10:47 UT).
* ``SWISS_ORACLE``: pyswisseph 2.10.03 rise/transit/set rows already in the
  repository (crates/xalen-ephem/tests/data/swiss_rise_set_oracle.json, the
  file the Rust engine tests use); ``rise_trans`` must agree to 15 s.

Tuple lengths, return codes and bit values follow the pyswisseph 2.10 API.
"""

import json
import math
import re
from pathlib import Path

import pytest

xalen = pytest.importorskip("xalen", reason="build the extension first (maturin develop)")
import xalen.swe as swe  # noqa: E402

DELHI = (77.2090, 28.6139, 0.0)  # geopos order: lon, lat, alt (m)
DALLAS = (-96.80, 32.78, 0.0)
TOKYO = (139.69, 35.69, 0.0)
NEW_YORK = (-74.01, 40.71, 0.0)
SWISS_ORACLE = (
    Path(__file__).resolve().parents[2]
    / "xalen-ephem"
    / "tests"
    / "data"
    / "swiss_rise_set_oracle.json"
)
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


def test_bad_house_arguments_raise_value_error():
    jd = 2451545.0
    cusps, ascmc = swe.houses(jd, 28.6139, 77.2090, b"P")
    assert len(cusps) == 12 and len(ascmc) == 8
    for lat, lon, hsys in (
        (91.0, 77.2, b"P"),
        (-90.5, 77.2, b"P"),
        (28.6, 400.0, b"P"),
        (28.6, -181.0, b"P"),
        (28.6, 77.2, b"Z"),
    ):
        with pytest.raises(ValueError):
            swe.houses(jd, lat, lon, hsys)
        with pytest.raises(ValueError):
            swe.houses_ex(jd, lat, lon, hsys, 0)
    with pytest.raises(ValueError):
        swe.houses(float("nan"), 28.6, 77.2, b"P")


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


def test_twilight_bits_apply_to_the_sun_only():
    # Swiss defines twilight for the Sun; for the Moon the bits change nothing.
    start = jd_ist(2024, 6, 21, 0, 0)
    for event in (swe.CALC_RISE, swe.CALC_SET):
        plain = rise(start, swe.MOON, event)
        for bit in (swe.BIT_CIVIL_TWILIGHT, swe.BIT_NAUTIC_TWILIGHT, swe.BIT_ASTRO_TWILIGHT):
            assert rise(start, swe.MOON, event | bit) == plain
    # The Sun does move: civil dawn comes well before sunrise.
    sunrise = rise(start, swe.SUN, swe.CALC_RISE)
    dawn = rise(start, swe.SUN, swe.CALC_RISE | swe.BIT_CIVIL_TWILIGHT)
    assert 15 * MINUTE < sunrise - dawn < 40 * MINUTE


def test_moon_rise_transit_set_match_the_swiss_oracle():
    # pyswisseph 2.10.03 rows, forward search from 00:00 UT with 1013.25 hPa
    # and 15 deg C. The Moon rows the Rust engine test (rise_set.rs ORACLE)
    # pins: Delhi in June, December and March; London and Quito in June and
    # December.
    pinned = {
        ("Delhi", 6),
        ("Delhi", 12),
        ("Delhi", 3),
        ("London", 6),
        ("London", 12),
        ("Quito", 6),
        ("Quito", 12),
    }
    data = json.loads(SWISS_ORACLE.read_text())
    assert (data["atpress"], data["attemp"]) == (1013.25, 15.0)
    rows = [r for r in data["rows"] if r["body"] == "Moon" and (r["loc"], r["m"]) in pinned]
    assert len(rows) == len(pinned)
    for r in rows:
        geopos = (r["lon"], r["lat"], r["elev"])
        for event, key in (
            (swe.CALC_RISE, "rise"),
            (swe.CALC_SET, "set"),
            (swe.CALC_MTRANSIT, "transit"),
        ):
            got = rise(r["jd_start"], swe.MOON, event, geopos, atpress=1013.25, attemp=15.0)
            off = abs(got - r[key]) * 86400.0
            assert off < 15.0, f"{r['loc']} {r['m']}: Moon {key} off by {off:.1f} s"


def test_moon_hindu_rising_differs_from_topocentric_centre_by_minutes():
    # BIT_HINDU_RISING puts the Moon's geocentric position on the ecliptic:
    # no parallax (about 1 deg for the Moon) and no ecliptic latitude (up to
    # 5 deg), so its rise moves by minutes against the topocentric disc centre.
    centre = swe.BIT_DISC_CENTER | swe.BIT_NO_REFRACTION
    gaps = []
    for start in (2460482.5, 2460665.5, 2460389.5):
        hindu = rise(start, swe.MOON, swe.CALC_RISE | swe.BIT_HINDU_RISING)
        topo = rise(start, swe.MOON, swe.CALC_RISE | centre)
        gap = abs(hindu - topo)
        assert gap < 60 * MINUTE, gap * 1440
        gaps.append(gap)
    assert max(gaps) > 2 * MINUTE, [g * 1440 for g in gaps]


def test_rise_trans_true_hor_dip_of_the_horizon():
    start = jd_ist(2024, 6, 21, 0, 0)
    # At sea level the dip is zero: -100 gives the true-horizon answer.
    _, flat = swe.rise_trans_true_hor(start, swe.SUN, swe.CALC_RISE, DELHI, 0.0, 15.0, 0.0)
    res, dip = swe.rise_trans_true_hor(start, swe.SUN, swe.CALC_RISE, DELHI, 0.0, 15.0, -100.0)
    assert res == 0
    assert abs(dip[0] - flat[0]) < 0.01 * SECOND
    # From 1000 m the sea horizon is about 0.93 deg down: the Sun is seen
    # rising several minutes earlier.
    high = (77.2090, 28.6139, 1000.0)
    _, flat = swe.rise_trans_true_hor(start, swe.SUN, swe.CALC_RISE, high, 0.0, 15.0, 0.0)
    res, dip = swe.rise_trans_true_hor(start, swe.SUN, swe.CALC_RISE, high, 0.0, 15.0, -100.0)
    assert res == 0
    assert 3 * MINUTE < flat[0] - dip[0] < 10 * MINUTE, (flat[0] - dip[0]) * 1440
    # Any other value below the horizon limit is a bad argument, not a -2.
    for bad in (-95.0, -90.5, 91.0):
        with pytest.raises(ValueError):
            swe.rise_trans_true_hor(start, swe.SUN, swe.CALC_RISE, DELHI, 0.0, 15.0, bad)


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


def test_sol_eclipse_when_glob_partial_passes_the_centrality_filter():
    # ECL_CENTRAL | ECL_PARTIAL: a partial eclipse has no central path, so the
    # centrality bit must not reject it. From 2024 the first partial is
    # 2025-03-29 (NASA greatest eclipse about 10:47 UT).
    retflag, tret = swe.sol_eclipse_when_glob(
        swe.julday(2024, 1, 1, 0.0), swe.FLG_SWIEPH, swe.ECL_CENTRAL | swe.ECL_PARTIAL
    )
    assert retflag == swe.ECL_PARTIAL
    assert swe.revjul(tret[0])[:3] == (2025, 3, 29)
    assert abs(tret[0] - swe.julday(2025, 3, 29, 10 + 47 / 60)) < 15 * MINUTE
    retflag, tret = swe.sol_eclipse_when_glob(
        swe.julday(2024, 1, 1, 0.0), swe.FLG_SWIEPH, swe.ECL_NONCENTRAL | swe.ECL_PARTIAL
    )
    assert retflag == swe.ECL_PARTIAL
    assert swe.revjul(tret[0])[:3] == (2025, 3, 29)


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
    assert 1.0 < attr[1] < 1.1  # Moon/Sun diameter ratio (NASA 1.0566 at greatest)
    assert attr[8] == attr[1]  # NASA magnitude of a total eclipse is the ratio
    assert attr[2] > 0.999  # obscuration
    assert attr[5] > 50.0  # Sun high over Dallas at maximum


def test_sol_eclipse_when_loc_new_york_partial_magnitude():
    # New York saw 2024-04-08 as a deep partial eclipse (about 0.9).
    retflag, tret, attr = swe.sol_eclipse_when_loc(swe.julday(2024, 4, 1, 0.0), NEW_YORK)
    assert retflag & swe.ECL_PARTIAL and retflag & swe.ECL_VISIBLE
    assert swe.revjul(tret[0])[:3] == (2024, 4, 8)
    assert 0.8 < attr[0] < 1.0
    assert 1.0 < attr[1] < 1.1  # the same large Moon as at Dallas
    assert attr[8] == attr[0]  # NASA magnitude of a partial eclipse


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


def test_lun_eclipse_how_2022_november_at_tokyo_and_delhi():
    _, tret = swe.lun_eclipse_when(swe.julday(2022, 10, 1, 0.0))
    # Tokyo, about 20:00 local time: the Moon is up in the east.
    retflag, attr = swe.lun_eclipse_how(tret[0], TOKYO)
    assert len(attr) == 20
    assert retflag == swe.ECL_TOTAL
    assert abs(attr[0] - 1.36) < 0.02  # umbral magnitude (NASA 1.359)
    assert 2.3 < attr[1] < 2.5  # penumbral magnitude (NASA 2.416)
    assert attr[8] == attr[0]
    assert attr[5] > 10.0  # true altitude
    assert attr[5] < attr[6] < attr[5] + 0.2  # refraction lifts it a little
    # Azimuth from the south point through west: east of the meridian is
    # between 180 and 360.
    assert 180.0 < attr[4] < 360.0
    # Delhi, 16:29 IST: the Moon has not risen, so Swiss returns 0, with the
    # shadow values still filled in.
    retflag, attr = swe.lun_eclipse_how(tret[0], DELHI)
    assert retflag == 0
    assert attr[6] <= 0.0
    assert attr[5] < attr[6]
    assert abs(attr[0] - 1.36) < 0.02
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
    moonrise = tret[8]
    assert moonrise > 0.0 and tret[9] == 0.0
    # As in Swiss, the contacts before moonrise are 0.0: penumbral and umbral
    # begin and totality begin all came before the maximum, which was not seen.
    assert tret[6] == tret[2] == tret[4] == 0.0
    assert moonrise < tret[3] < tret[7]  # partial and penumbral end were seen
    assert tret[0] > 0.0  # the maximum is kept
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
