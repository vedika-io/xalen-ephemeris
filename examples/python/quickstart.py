#!/usr/bin/env python3
"""XALEN — Python quickstart.

Build + install the extension first (from the repo root):

    pip install maturin
    cd crates/xalen-python && maturin develop --release

then run:

    python examples/python/quickstart.py

This touches both APIs:
  1. the native `xalen` module (structured dicts), and
  2. `import xalen.swe as swe` — the pyswisseph drop-in.
"""

import xalen
import xalen.swe as swe


def native_api() -> None:
    print("=== native xalen API ===")

    # Calendar -> Julian Day (UT). 1990-06-15 10:30 UT.
    jd = xalen.julian_day(1990, 6, 15, 10.5)
    print(f"Julian Day: {jd:.5f}")

    # Full planetary state (Swiss calc_ut(..., FLG_SPEED) + retrograde flag).
    sun = xalen.planet_position(jd, 0)  # body 0 = Sun, tropical
    print(
        f"Sun: lon={sun['longitude']:.4f} deg  "
        f"speed={sun['lon_speed']:.4f} deg/day  "
        f"dist={sun['distance']:.6f} AU  "
        f"retrograde={sun['is_retrograde']}"
    )

    # Sidereal (Lahiri = ayanamsa 0). Ketu (id 13) = Rahu + 180.
    moon_sid = xalen.planet_position(jd, 1, sidereal=True, ayanamsa=0)
    ketu = xalen.planet_position(jd, 13, sidereal=True, ayanamsa=0)
    print(f"Moon (sidereal): {moon_sid['longitude']:.4f} deg")
    print(f"Ketu (sidereal): {ketu['longitude']:.4f} deg")

    # Nakshatra detail from the sidereal Moon longitude.
    nak = xalen.nakshatra(moon_sid["longitude"])
    print(f"Moon nakshatra: {nak['name']} pada {nak['pada']} (lord {nak['lord']})")

    # Panchang — the five limbs.
    pan = xalen.panchang(jd, ayanamsa=0)
    print(
        f"Panchang: tithi {pan['tithi']['name']} ({pan['tithi']['paksha']}), "
        f"nakshatra {pan['nakshatra']}, yoga {pan['yoga']['name']}, "
        f"karana {pan['karana']}, vara {pan['vara']}"
    )

    # Full Vedic chart for Pune (18.52 N, 73.85 E), Whole-Sign houses.
    chart = xalen.full_chart(jd, 18.52, 73.85, ayanamsa=0)
    asc, mc = chart["ascendant"], chart["mc"]
    print(f"Ascendant: {asc:.2f} deg   MC: {mc:.2f} deg   ayanamsa: {chart['ayanamsa_deg']:.4f} deg")
    for body in ("Sun", "Moon", "Mars"):
        p = chart["planets"][body]
        print(f"  {body}: {p['longitude']:.2f} deg  {p['rashi']}  {p['nakshatra']} pada {p['pada']}")

    # Houses across 14 systems (2 = Placidus).
    h = xalen.houses(jd, 18.52, 73.85, system=2)
    print(f"Placidus ascendant: {h['ascendant']:.2f} deg, MC: {h['mc']:.2f} deg")


def pyswisseph_drop_in() -> None:
    print("\n=== xalen.swe (pyswisseph drop-in) ===")

    # Identical call shapes to `import swisseph as swe`.
    jd = swe.julday(1990, 6, 15, 10.5)

    # calc_ut returns ((lon, lat, dist, lon_speed, lat_speed, dist_speed), ret).
    xx, retflag = swe.calc_ut(jd, swe.SUN, swe.FLG_SWIEPH | swe.FLG_SPEED)
    print(f"Sun tropical lon={xx[0]:.4f} deg  speed={xx[3]:.4f} deg/day  ret_flag={retflag}")

    # Sidereal mode (Lahiri).
    swe.set_sid_mode(swe.SIDM_LAHIRI, 0.0, 0.0)
    sid, _ = swe.calc_ut(jd, swe.SUN, swe.FLG_SWIEPH | swe.FLG_SIDEREAL)
    print(f"Sun sidereal lon={sid[0]:.4f} deg  ayanamsa={swe.get_ayanamsa_ut(jd):.4f} deg")

    # Houses: (cusps[12], ascmc[8]); ascmc[0..4] = asc, mc, armc, vertex.
    cusps, ascmc = swe.houses_ex(jd, 18.52, 73.85, b"P")
    print(f"Placidus asc={ascmc[0]:.2f} deg  mc={ascmc[1]:.2f} deg  cusp1={cusps[0]:.2f} deg")

    # ΔT and a calendar round-trip.
    print(f"deltaT={swe.deltat(jd):.2f} s   revjul={swe.revjul(jd)}")

    # Fixed star (catalog has no velocity, so distance + speeds are 0.0).
    star_xx, star_name, _ = swe.fixstar2_ut("Aldebaran", jd, swe.FLG_SWIEPH)
    mag, _ = swe.fixstar2_mag("Aldebaran")
    print(f"{star_name}: lon={star_xx[0]:.4f} deg  mag={mag:.2f}")

    # Honesty: position-altering flags XALEN does not implement raise ValueError
    # rather than silently mislabeling a geocentric ecliptic position.
    try:
        swe.calc_ut(jd, swe.SUN, swe.FLG_SWIEPH | swe.FLG_HELCTR)
    except ValueError as e:
        print(f"(expected) heliocentric not implemented -> ValueError: {e}")


if __name__ == "__main__":
    native_api()
    pyswisseph_drop_in()
