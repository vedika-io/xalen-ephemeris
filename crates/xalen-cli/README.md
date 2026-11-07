# xalen-cli

The `xalen` command line for [XALEN Ephemeris](https://github.com/vedika-io/xalen-ephemeris):
planet positions, charts, panchang, dasha, fixed stars, eclipses, rise and set
times, aspects and time conversions from the terminal. Pure Rust, offline, no
data files. It is a thin layer over the library crates and contains no
astronomy of its own.

## Install

```bash
cargo install xalen-cli
xalen --version
```

`xalen-cli` is published to crates.io with each release of the workspace, so the
command above installs the latest released version. To try the tool before a
release, build it from a checkout with `cargo install --path crates/xalen-cli`.

## Conventions

- Time: `--date <ISO-8601>` (`2026-11-07`, `2026-11-07T00:00:00Z`,
  `2026-11-07T05:30:00+05:30`; no zone means UTC) or `--jd <Julian Day UT>`.
- Place: `--lat <deg> --lon <deg east> [--alt <m>]`, or `--place "<city>"` for a
  city in the built-in table (`xalen_houses::geocoding`).
- Output: a readable table by default, JSON with `--json`. Times are printed in
  UTC unless `--tz` is given: `UTC`, an offset such as `+05:30`, or a city from
  the library's timezone table (standard time only; IANA names and daylight
  saving are not supported).
- Calendar and time scale: dates are proleptic Gregorian, years -13000 to 17000
  and Julian Days -3000000 to 10000000 are accepted, and UTC is taken as UT1
  (no DUT1 correction, the library's convention). TT comes from the library's
  delta T model.
- Zodiac: tropical by default. `positions` and `chart` take `--sidereal
  [AYANAMSA]` (Lahiri unless named). `panchang` and `dasha` are always
  sidereal and take `--ayanamsa <NAME>` (Lahiri by default). `xalen positions
  --help` lists every ayanamsa; a name or a Swiss Ephemeris number works.
- Exit codes: `0` success, `1` the computation failed, `2` bad input (one line on
  stderr).

## Commands

### positions

Longitudes, latitudes, distances and daily speeds. `--bodies sun,moon,...`
selects bodies; sidereal output adds nakshatra and pada.

```console
$ xalen positions --date 2026-11-07T00:00:00Z --sidereal --bodies sun,moon,mars
Positions for 2026-11-07 00:00:00 UTC
JD UT 2461351.500000   JD TT 2461351.500792
Zodiac: sidereal, Lahiri (Chitrapaksha) ayanamsa 24.234165 deg

Body  Longitude  Sign            Deg    Nakshatra  Pada  Latitude  Dist (AU)  Speed/day  R
----  ---------  --------------  -----  ---------  ----  --------  ---------  ---------  -
Sun   200.3533   Tula (Libra)    20.35  Vishakha   1     -0.0001   0.991201   1.0034
Moon  173.7469   Kanya (Virgo)   23.75  Chitra     1     -3.9172   0.002610   12.8097
Mars  117.3483   Karka (Cancer)  27.35  Ashlesha   4     1.7288    1.383221   0.4829
```

### chart

Ascendant, midheaven, house cusps (`--house-system`, default Placidus, or Whole
Sign with `--sidereal`) and each body with sign, degree and house. The house
system is a name (`Placidus`, `WholeSign`, `Koch`, ...) or a Swiss Ephemeris
letter (`P`, `W`, `K`, ...); `xalen chart --help` lists them all.

```console
$ xalen chart --date 1990-03-15T06:30:00Z --place delhi --sidereal --bodies sun,moon
Chart for 1990-03-15 06:30:00 UTC
Zodiac: sidereal, Lahiri (Chitrapaksha) ayanamsa 23.723402 deg

Place: delhi (lat 28.6139, lon 77.2090)
House system: Whole Sign

Angle       Longitude  Sign                 Deg
----------  ---------  -------------------  -----
Ascendant   67.1905    Mithuna (Gemini)     7.19
Midheaven   322.4921   Kumbha (Aquarius)    22.49
Descendant  247.1905   Dhanu (Sagittarius)  7.19
Imum Coeli  142.4921   Simha (Leo)          22.49
Vertex      202.6252   Tula (Libra)         22.63

House  Cusp      Sign                  Deg
-----  --------  --------------------  ----
1      60.0000   Vrishabha (Taurus)    0.00
2      90.0000   Karka (Cancer)        0.00
...
```

Sidereal Whole Sign cusps sit on sign boundaries: the ascendant (67.19) is in
house 1, which starts at 60.

### panchang

Tithi, nakshatra, yoga and karana with their start and end times, the vara, and
sunrise and sunset for the place. The vara runs from sunrise to sunrise.

```console
$ xalen panchang --date 2026-11-07T00:00:00Z --place delhi --tz +05:30
Limb       Value                    Starts                      Ends
---------  -----------------------  --------------------------  --------------------------
Tithi      Krishna Trayodashi (28)  2026-11-06 10:31:07 +05:30  2026-11-07 10:48:26 +05:30
Nakshatra  Chitra (pada 1)          2026-11-07 04:43:31 +05:30  2026-11-08 05:52:41 +05:30
Yoga       Preeti                   2026-11-07 04:10:05 +05:30  2026-11-08 03:27:47 +05:30
Karana     Vanija                   2026-11-06 22:36:59 +05:30  2026-11-07 10:48:26 +05:30

Vara:    Shukravara (lord Venus)
Sunrise: 2026-11-07 06:37:22 +05:30
Sunset:  2026-11-07 17:31:58 +05:30
```

### dasha

Vimshottari mahadasha periods from the Moon's nakshatra at the given instant;
`--levels 2` (the default) adds the antardashas.

```console
$ xalen dasha --date 1990-03-15T06:30:00Z --levels 1
Moon 193.2923 deg in Tula (Libra), Swati pada 2 (lord Rahu)

Lord     Level      Start       End         Years
-------  ---------  ----------  ----------  -----
Rahu     Mahadasha  1990-03-15  1999-04-04  9.06
Jupiter  Mahadasha  1999-04-04  2015-04-04  16.00
...
```

### star

A fixed star's position for the date. The name is an IAU proper name or a
traditional name, matched ignoring case; quote names with spaces.

```console
$ xalen star Spica --date 2026-11-07T00:00:00Z
Constellation: Virgo
Magnitude:     0.98
Ecliptic lon:  204.2161 deg (Libra 24.22 deg)
Ecliptic lat:  -2.0565 deg
Right asc.:    201.6528 deg (13.4435 h)
Declination:   -11.3006 deg
```

### eclipses

Eclipses between `--from` and `--to` (at most 100 years), `--type
solar|lunar|all`.

```console
$ xalen eclipses --from 2024-01-01 --to 2024-12-31 --type solar
Maximum                  Kind   Type     Detail
-----------------------  -----  -------  -------------
2024-04-08 18:17:26 UTC  solar  Total    gamma 0.3433
2024-10-02 18:45:03 UTC  solar  Annular  gamma -0.3507
```

### rise-set

Rise, transit and set of the Sun, the Moon or a planet over the 24 hours that
start at the given instant.

```console
$ xalen rise-set --date 2026-11-07T00:00:00Z --place delhi --body moon --tz +05:30
Rise:     -
Transit:  2026-11-07 10:25:24 +05:30
Set:      2026-11-07 16:06:01 +05:30
Altitude at transit: 48.97 deg
```

A `-` means the event does not fall inside the window.

### aspects

Tropical aspects between the bodies with the default orbs of the aspects module,
or one flat `--orb <deg>`; `--minor` adds the minor aspects.

```console
$ xalen aspects --date 2026-11-07T00:00:00Z --orb 3
Body     Aspect      Body       Angle  Orb    Motion
-------  ----------  ---------  -----  -----  ----------
Jupiter  Opposition  Mean Node  180    0.744  Applying
Venus    Sextile     Jupiter    60     1.141  Separating
...
```

### convert

JD UT, JD TT, delta T, Greenwich mean and apparent sidereal time, and the value
of every ayanamsa.

```console
$ xalen convert --date 2000-01-01T12:00:00Z
JD UT:                  2451545.000000
JD TT:                  2451545.000739
Delta T (TT - UT):      63.810 s
GMST (mean sidereal):   280.460618 deg = 18h 41m 50.548s
GAST (apparent):        280.457067 deg = 18h 41m 49.696s
...
```

## Errors

```console
$ xalen positions --date 2026-02-30
error: bad date '2026-02-30': the day does not exist in that month (use ISO-8601, ...)
$ echo $?
2
```

## Licence

Apache-2.0.
