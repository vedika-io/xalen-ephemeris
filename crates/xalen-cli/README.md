# xalen-cli

The `xalen` command line for [XALEN Ephemeris](https://github.com/vedika-io/xalen-ephemeris):
planet positions, charts, panchang, dasha, fixed stars, eclipses, rise and set
times, aspects, time conversions, astrocartography maps and relationship charts
(synastry, composite, Davison, relocation) from the terminal. Pure Rust, offline, no
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

### map

Astrocartography lines (after Lewis, 1976): where each body is on the MC, IC,
rising (ASC) or setting (DSC) at the moment. The table gives each body's right
ascension, declination, zenith point (where it stands overhead) and the MC and
IC longitudes; with `--lat` it adds the longitudes where the body rises and sets
on that latitude. `--bodies` defaults to Sun to Pluto; `--lat-min`, `--lat-max`
and `--lat-step` (default -66, 66, 1) set how the lines are sampled, and
`--altitude` sets the horizon altitude (default 0). Longitudes are east-positive,
-180 to 180. `--json` prints the library's full `AstroMap`; the map has no
zodiac.

```console
$ xalen map --date 2024-06-21T00:00:00Z --bodies sun,moon,mars --lat 40
Astrocartography for 2024-06-21 00:00:00 UTC
JD UT 2460482.500000   JD TT 2460482.500790   GAST 269.6831 deg
Latitudes -66.00 to 66.00, step 1.00, horizon altitude 0.0000 deg

Body  RA        Dec       Zenith lat  Zenith lon  MC lon     IC lon    ASC lon   DSC lon
----  --------  --------  ----------  ----------  ---------  --------  --------  ---------
Sun   90.1364   23.4382   23.4382     -179.5467   -179.5467  0.4533    69.1211   -68.2145
...
```

`--parans` adds the parans (Brady): the places where two bodies are on angles
at the same moment, which are the crossings of one body's lines with another's,
within `--lat-min` and `--lat-max`. Each row gives the latitude and longitude
and the two bodies with their angles; `--json` adds a `parans` array (the
library's `Paran` values), `parans_skipped` (line pairs that do not cross in the
range, with the reason) and `paran_options` (the options of that full list).
With `--lat` the table lists only the parans within `--orb` degrees (default 1,
at least 0 and under 90) of that latitude, and `--json` adds them as
`parans_near`. That search uses the band `--lat` plus or minus `--orb` in place
of `--lat-min` and `--lat-max`, so it also works outside them. `--orb` needs
both `--parans` and `--lat`. The Moon's horizon parans are geocentric and can be
off by up to about one degree.

```console
$ xalen map --date 2024-06-21T00:00:00Z --bodies sun,moon,mars,jupiter --parans --lat 28.61 --orb 2
```

`--geojson` prints a GeoJSON FeatureCollection instead: one `LineString` feature
per line segment (properties `body` and `kind`, which is `MC`, `IC`, `ASC` or
`DSC`) and one `Point` feature per zenith (`kind` is `ZENITH`), with coordinates
as `[longitude, latitude]`. With `--parans` it also has one `Point` per paran,
with `kind` `PARAN` and properties `body_a`, `kind_a`, `body_b` and `kind_b`.
Segments never cross the antimeridian. `--geojson`
cannot be combined with `--lat` or `--json`, and it leaves out bodies the
ephemeris cannot reach at the date (Pluto outside 1885-2099 in the default set),
which the table and `--json` list as skipped. Load the file
in any GIS tool or on geojson.io.

```console
$ xalen map --date 2024-06-21T00:00:00Z --bodies sun --geojson > sun.geojson
```

### synastry

The aspects between two births and the house overlays. Person A is given with
`--a-date` (or `--a-jd`) and `--a-place` (or `--a-lat` and `--a-lon`, optionally
`--a-alt`); person B the same with `--b-`. `--house-system` applies to both
charts (default Placidus), `--aspects` takes `major` (default), `all` or names
such as `trine,square`, `--orb-mult` scales the default orbs, and `--angles` adds
the Ascendant, Midheaven, Descendant and Imum Coeli. The zodiac is tropical. The
header notes a chart whose houses fell back to Porphyry at a polar latitude
(`fallback_used` in the JSON).

```console
$ xalen synastry --a-date 1990-03-15T06:30:00Z --a-place delhi --b-date 1992-08-02T14:10:00Z --b-place mumbai --bodies sun,moon,venus,mars --angles
Synastry (tropical zodiac)
A: 1990-03-15 06:30:00 UTC  delhi (lat 28.6139, lon 77.2090)
B: 1992-08-02 14:10:00 UTC  mumbai (lat 19.0760, lon 72.8777)
House system: Placidus   orb multiplier: 1.00   14 aspects found

Aspects (rows are A, columns are B; each cell is the aspect and its orb in degrees)
A \ B  Su        Mo        Ve        Ma        Asc  MC        Dsc  IC
-----  --------  --------  --------  --------  ---  --------  ---  --------
Su     -         -         -         -         -    Tri 4.61  -    Sxt 4.61
Mo     Sqr 3.50  -         -         -         -    -         -    -
...
```

The output goes on with "B's points in A's houses" and "A's points in B's
houses". `--json` prints the library's `Synastry` (`rows`, `cols`, row-major
`cells`, `b_in_a`, `a_in_b`) with the two births and the options added.

### composite

The midpoint chart of two births (Hand, 1975), printed like `chart`. The
Ascendant is `--ascendant derived` (default, from the midpoint Midheaven at the
mean latitude) or `midpoint` (the midpoint of the two Ascendants, Equal and Whole
Sign houses only; Equal is the default then).

```console
$ xalen composite --a-date 1990-03-15T06:30:00Z --a-place delhi --b-date 1992-08-02T14:10:00Z --b-place mumbai --bodies sun,moon
Composite chart for 1991-05-24 22:20:00 UTC
...
Reference place (mean birthplace): lat 23.8450, lon 75.0433
Ascendant: derived from the midpoint Midheaven at the reference latitude
House system: Placidus

Angle       Longitude  Sign       Deg
----------  ---------  ---------  -----
Ascendant   32.0831    Taurus     2.08
Midheaven   292.6174   Capricorn  22.62
...
```

### davison

The chart for the midpoint in time and place of two births (Davison, 1977),
printed like `chart` with the mean date and place. `--space arithmetic`
(default) averages latitude and longitude; `--space great-circle` takes the
midpoint on the great circle between the places.

```console
$ xalen davison --a-date 1990-03-15T06:30:00Z --a-place delhi --b-date 1992-08-02T14:10:00Z --b-place mumbai --bodies sun,moon
Davison chart for 1991-05-24 22:20:00 UTC
...
Mean place (arithmetic): lat 23.8450, lon 75.0433
House system: Placidus
...
```

### relocate

One birth with the houses recomputed for another place; the bodies do not move.
The destination is `--to-place` or `--to-lat` and `--to-lon`. The JSON adds the
birth Ascendant as `natal_ascendant`.

```console
$ xalen relocate --date 1990-03-15T06:30:00Z --place delhi --to-place london --bodies sun,moon
Relocated chart for 1990-03-15 06:30:00 UTC
...
Born at: delhi (lat 28.6139, lon 77.2090)
Relocated to: london (lat 51.5074, lon -0.1278); the bodies are unchanged, the houses are recomputed

Angle       Longitude  Sign         Deg
----------  ---------  -----------  -----
Ascendant   359.9476   Pisces       29.95
...
```

Zodiac of the four relationship commands: `synastry` is always tropical (its
aspects and overlays are differences of tropical longitudes). `composite`,
`davison` and `relocate` take `--sidereal [AYANAMSA]`, which only converts the
printed longitudes, signs and houses of the tropical chart by the ayanamsa at
the chart's date (the mean date for composite and Davison); nothing is
recomputed. Cusps go through the library's `HouseCusps::to_sidereal`, so Whole
Sign cusps are rebuilt from the sidereal Ascendant (the default house system
with `--sidereal`, as in `chart`). Without `--sidereal` the output is tropical.

## Errors

```console
$ xalen positions --date 2026-02-30
error: bad date '2026-02-30': the day does not exist in that month (use ISO-8601, ...)
$ echo $?
2
```

## Licence

Apache-2.0.
