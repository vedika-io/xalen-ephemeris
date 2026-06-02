"""End-to-end parity tests for the XALEN Python binding.

These run against the BUILT extension (``maturin develop`` /
``pip install xalen``); the Rust side is unit-tested by ``cargo test``. They are
skipped automatically when the compiled module is not importable, so they never
break a pure-``cargo test`` run.

They assert two things:

1. ``xalen.planet_position`` returns the full six-component state plus a
   retrograde flag (the pyswisseph ``calc_ut(..., FLG_SPEED)`` shape).
2. ``import xalen.swe as swe`` is a search-and-replace drop-in for
   ``import swisseph as swe`` — and, when pyswisseph itself is installed, the
   numbers agree to published Swiss tolerances.
"""

import math

import pytest

xalen = pytest.importorskip("xalen", reason="build the extension first (maturin develop)")

J2000 = 2451545.0


# ---------------------------------------------------------------------------
# planet_position — full 6-tuple + retrograde
# ---------------------------------------------------------------------------

def test_planet_position_full_tuple_keys():
    p = xalen.planet_position(J2000, 0)  # Sun, tropical
    for key in (
        "longitude",
        "latitude",
        "distance",
        "lon_speed",
        "lat_speed",
        "dist_speed",
        "is_retrograde",
    ):
        assert key in p, f"planet_position missing key {key}"
    assert 0.0 <= p["longitude"] < 360.0
    assert isinstance(p["is_retrograde"], bool)
    assert p["is_retrograde"] is False  # Sun is never retrograde


def test_planet_position_node_retrograde_and_ketu():
    rahu = xalen.planet_position(J2000, 9)  # Mean node
    assert rahu["is_retrograde"] is True
    assert rahu["lon_speed"] < 0.0

    ketu = xalen.planet_position(J2000, 13)  # Ketu = Rahu + 180
    expected = (rahu["longitude"] + 180.0) % 360.0
    assert abs(ketu["longitude"] - expected) < 1e-9
    assert ketu["is_retrograde"] == rahu["is_retrograde"]


def test_planet_position_sidereal_subtracts_ayanamsa():
    trop = xalen.planet_position(J2000, 0, False, 0)
    sid = xalen.planet_position(J2000, 0, True, 0)  # Lahiri
    offset = (trop["longitude"] - sid["longitude"]) % 360.0
    assert 23.0 < offset < 25.0
    # Sidereal speed is slightly slower than tropical but still ~1 deg/day.
    assert 0.9 < sid["lon_speed"] < trop["lon_speed"]


# ---------------------------------------------------------------------------
# xalen.swe drop-in submodule
# ---------------------------------------------------------------------------

def test_swe_submodule_importable_as_drop_in():
    import xalen.swe as swe

    # Constants exist under BOTH pyswisseph and SE_-prefixed spellings.
    assert swe.SUN == 0 == swe.SE_SUN
    assert swe.MOON == 1
    assert swe.FLG_SWIEPH == 2 == swe.SEFLG_SWIEPH
    assert swe.FLG_SPEED == 256
    assert swe.SIDM_LAHIRI == 1 == swe.SE_SIDM_LAHIRI
    # Osculating ("true") apogee / Black-Moon-Lilith id, matching pyswisseph.
    assert swe.OSCU_APOG == 13 == swe.SE_OSCU_APOG


def test_swe_oscu_apog_constant_usable_in_calc():
    import xalen.swe as swe

    # A pyswisseph caller doing calc_ut(jd, swe.OSCU_APOG, ...) must work.
    xx, _ = swe.calc_ut(J2000, swe.OSCU_APOG, swe.FLG_SWIEPH)
    assert 0.0 <= xx[0] < 360.0
    # It must differ from the MEAN apogee — that is the whole point of OSCU.
    mean, _ = swe.calc_ut(J2000, swe.MEAN_APOG, swe.FLG_SWIEPH)
    assert abs(xx[0] - mean[0]) > 1.0


def test_swe_calc_tt_path_is_deltat_shifted_calc_ut():
    import xalen.swe as swe

    # swe.calc takes a TT/ET epoch; the correct UT1 equivalent is jd - deltat(jd).
    # So swe.calc(jd_tt) must equal swe.calc_ut(jd_tt - deltat(jd_tt)).
    jd_tt = J2000
    dt = swe.deltat(jd_tt)  # days
    assert dt > 0.0
    et_xx, _ = swe.calc(jd_tt, swe.MOON, swe.FLG_SWIEPH)
    ut_xx, _ = swe.calc_ut(jd_tt - dt, swe.MOON, swe.FLG_SWIEPH)
    assert abs((et_xx[0] - ut_xx[0] + 180.0) % 360.0 - 180.0) < 1e-6
    # And calc differs from calc_ut(same jd) by the Moon's motion over ΔT
    # (~13 deg/day * ~0.0008 day ~ 0.01 deg) — i.e. ΔT is actually applied.
    same_jd, _ = swe.calc_ut(jd_tt, swe.MOON, swe.FLG_SWIEPH)
    assert abs((et_xx[0] - same_jd[0] + 180.0) % 360.0 - 180.0) > 1e-4


def test_swe_calc_ut_shape_and_speed():
    import xalen.swe as swe

    xx, retflag = swe.calc_ut(J2000, swe.SUN, swe.FLG_SWIEPH | swe.FLG_SPEED)
    assert len(xx) == 6
    assert 0.0 <= xx[0] < 360.0
    # Sun ~1 deg/day, ~1 AU (validated vs pyswisseph 2.10.03: 1.019432, 0.98332764).
    assert abs(xx[3] - 1.019432) < 0.01
    assert abs(xx[2] - 0.98332764) < 1e-3
    assert retflag & swe.FLG_SWIEPH


def test_swe_calc_ut_sidereal_subtracts_ayanamsa():
    import xalen.swe as swe

    swe.set_sid_mode(swe.SIDM_LAHIRI, 0.0, 0.0)
    trop, _ = swe.calc_ut(J2000, swe.SUN, swe.FLG_SWIEPH)
    sid, _ = swe.calc_ut(J2000, swe.SUN, swe.FLG_SWIEPH | swe.FLG_SIDEREAL)
    offset = (trop[0] - sid[0]) % 360.0
    assert 23.0 < offset < 25.0


def test_swe_houses_ex_shape():
    import xalen.swe as swe

    # pyswisseph passes hsys as BYTES (b"P"); the drop-in must accept that.
    cusps, ascmc = swe.houses_ex(J2000, 18.52, 73.85, b"P")
    assert len(cusps) == 12
    assert len(ascmc) == 8
    # All eight ascmc slots are populated and in range — including the four
    # auxiliary ascendants [4..8] that used to be hard-coded 0.0.
    for v in ascmc:
        assert 0.0 <= v < 360.0
    # The Koch co-ascendant (ascmc[5]) and Munkasey polar ascendant (ascmc[7])
    # are exactly 180° apart (a Swiss property) — proves they are computed.
    diff = (ascmc[5] - ascmc[7]) % 360.0
    assert abs(diff - 180.0) < 1e-6, f"co-asc/polar-asc should be 180 apart, got {diff}"


def test_swe_houses_accepts_str_and_bytes_hsys():
    import xalen.swe as swe

    by_bytes = swe.houses_ex(J2000, 18.52, 73.85, b"P")
    by_str = swe.houses_ex(J2000, 18.52, 73.85, "P")
    # str and bytes house codes must produce identical results.
    assert by_bytes[0] == by_str[0]
    assert by_bytes[1] == by_str[1]
    # houses() (no flags arg) also accepts bytes.
    cusps, _ = swe.houses(J2000, 18.52, 73.85, b"K")
    assert len(cusps) == 12


def test_swe_houses_ex_sidereal_subtracts_ayanamsa():
    import xalen.swe as swe

    swe.set_sid_mode(swe.SIDM_LAHIRI, 0.0, 0.0)
    trop_c, trop_a = swe.houses_ex(J2000, 18.52, 73.85, b"P", 0)
    sid_c, sid_a = swe.houses_ex(J2000, 18.52, 73.85, b"P", swe.FLG_SIDEREAL)
    aya = swe.get_ayanamsa_ut(J2000)
    # Ascendant shifts by exactly the ayanamsa.
    asc_offset = (trop_a[0] - sid_a[0]) % 360.0
    assert abs(asc_offset - aya) < 1e-6, f"sidereal ASC offset {asc_offset} != ayanamsa {aya}"
    # Every cusp shifts by the same ayanamsa.
    for i in range(12):
        off = (trop_c[i] - sid_c[i]) % 360.0
        assert abs(off - aya) < 1e-6, f"cusp {i+1} offset {off} != ayanamsa {aya}"
    # ARMC (ascmc[2]) is a sidereal-time angle and is NOT shifted by ayanamsa.
    assert abs(trop_a[2] - sid_a[2]) < 1e-9


def test_swe_julday_revjul_roundtrip():
    import xalen.swe as swe

    jd = swe.julday(1990, 6, 15, 10.5)
    y, m, d, h = swe.revjul(jd)
    assert (y, m, d) == (1990, 6, 15)
    assert abs(h - 10.5) < 1e-3


def test_swe_fixstar2_ut_shape():
    import xalen.swe as swe

    xx, name, retflag = swe.fixstar2_ut("Spica", J2000)
    assert len(xx) == 6
    # Spica ecliptic longitude ~203.8 deg at J2000.
    assert 200.0 < xx[0] < 210.0
    assert "Spica" in name


def test_swe_get_ayanamsa():
    import xalen.swe as swe

    swe.set_sid_mode(swe.SIDM_LAHIRI, 0.0, 0.0)
    aya = swe.get_ayanamsa_ut(J2000)
    assert 23.0 < aya < 25.0  # Lahiri ~23.85 deg at J2000
    retflag, aya2 = swe.get_ayanamsa_ex_ut(J2000, swe.FLG_SWIEPH)
    assert abs(aya - aya2) < 1e-9


def test_swe_unsupported_flag_errors_not_silent():
    import xalen.swe as swe

    # A drop-in that silently ignores a frame-changing flag is worse than one
    # that errors. Heliocentric is not modelled => ValueError, never a wrong
    # geocentric position.
    with pytest.raises(ValueError):
        swe.calc_ut(J2000, swe.SUN, swe.FLG_SWIEPH | swe.FLG_HELCTR)


# ---------------------------------------------------------------------------
# Cross-check vs pyswisseph itself, when available.
# ---------------------------------------------------------------------------

def test_speed_agrees_with_pyswisseph():
    pyswe = pytest.importorskip("swisseph", reason="pyswisseph not installed")
    import xalen.swe as swe

    pyswe.set_ephe_path(None)
    for ipl in (pyswe.SUN, pyswe.MOON, pyswe.MARS, pyswe.MERCURY):
        ref, _ = pyswe.calc_ut(J2000, ipl, pyswe.FLG_SWIEPH | pyswe.FLG_SPEED)
        got, _ = swe.calc_ut(J2000, ipl, swe.FLG_SWIEPH | swe.FLG_SPEED)
        # Longitude agreement within the suite's documented sidereal-grade band.
        dlon = abs((got[0] - ref[0] + 180.0) % 360.0 - 180.0)
        assert dlon < 0.02, f"ipl={ipl} lon delta {dlon} deg"
        # Longitude speed within 0.05 deg/day (finite-difference vs Swiss).
        assert abs(got[3] - ref[3]) < 0.05, f"ipl={ipl} lon_speed delta"
        # Retrograde flag (sign of speed) must agree.
        assert math.copysign(1.0, got[3]) == math.copysign(1.0, ref[3])
