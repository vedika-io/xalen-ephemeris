# Binding numeric IDs

The C, WebAssembly, Node and Python bindings take plain integers to pick a
body, an ayanamsa, a house system and (in WebAssembly) a divisional chart.
The Python `xalen.swe` module and the Rust `xalen_ephem::compat` layer take
Swiss Ephemeris numbers instead. This page lists every one of those numbers.

The tables between the `BEGIN GENERATED` and `END GENERATED` markers are
produced from the Rust sources by `tests/binding_ids.rs`: the `*_from_id`
match tables in each binding crate, the `SE_*` constants and mappings in
`crates/xalen-ephem/src/compat.rs`, `Ayanamsa::from_swiss_ephem_id`, and the
constant tables registered in `crates/xalen-python/src/swe_compat.rs`. The test
runs with `cargo test` and fails, printing the lines that differ, when this
file and the sources disagree. Do not edit the tables by hand; change the
source and run:

```sh
scripts/regen-binding-ids.sh
```

## Two numbering schemes

The binding IDs are XALEN's own numbering. They are not Swiss Ephemeris
numbers, and several of them collide with a different meaning there:

- Body `9` is the mean node (Rahu) in the bindings and Pluto in Swiss
  Ephemeris; body `11` is Pluto in the bindings and the true node in Swiss.
- Ayanamsa `0` is Lahiri in the bindings and Fagan-Bradley in Swiss
  (`SE_SIDM_FAGAN_BRADLEY`); Lahiri is Swiss `1`.
- House systems are integers in the bindings and single letters (`hsys`) in
  Swiss Ephemeris.

Use the Swiss numbers only with `xalen.swe` (Python) and
`xalen_ephem::compat` (Rust). Every other binding function takes the binding
IDs below. A "yes" in a binding column means that binding accepts the ID; any
other value is rejected with an error (an exception in Python and Node, an
`Err` string in WebAssembly, a negative status in C).

## Where each ID is used

| ID family | C (FFI) | WASM | Node | Python (`xalen`) |
|---|---|---|---|---|
| Body | `xalen_planet_position`, `xalen_planet_position_full`, `xalen_sidereal_longitude` (`body_id`) | `tropicalLongitude`, `siderealLongitude`, `planetPositionJson`, `bodyName` (`bodyId`) | `planetLongitudeById`, `siderealLongitudeById`, `planetPositionById` (`bodyId`) | `planet_longitude`, `planet_position` (`body`) |
| Ayanamsa | `xalen_sidereal_longitude`, `xalen_ayanamsa` (`ayanamsa_id`) | `siderealLongitude`, `planetPositionJson`, `panchangJson`, `fullChartJson`, `XalenWasm.ayanamsaDeg` (`ayanamsaId`) | `siderealLongitudeById`, `planetPositionById`, `fullChart`, `ayanamsaById` (`ayanamsaId`) | `planet_longitude`, `planet_position`, `all_planets`, `panchang`, `full_chart` (`ayanamsa`); `ayanamsa` (`system`) |
| House system | `xalen_houses` (`system_id`) | `housesJson` (`systemId`) | `housesById` (`systemId`) | `houses` (`system`) |
| Divisional chart | - | `divisionalChart` (`varga`) | - | - |

Node and Python also have functions that take names as strings
(`planetLongitude("sun", jd)`, `houses(jd, lat, lon, "placidus")`); those are
not covered here.

## Bodies

<!-- BEGIN GENERATED bodies -->
| ID | `Body` | Swiss number | Swiss constant | C (FFI) | WASM | Node | Python |
|---|---|---|---|---|---|---|---|
| 0 | `Sun` | 0 | `SE_SUN` | yes | yes | yes | yes |
| 1 | `Moon` | 1 | `SE_MOON` | yes | yes | yes | yes |
| 2 | `Mercury` | 2 | `SE_MERCURY` | yes | yes | yes | yes |
| 3 | `Venus` | 3 | `SE_VENUS` | yes | yes | yes | yes |
| 4 | `Mars` | 4 | `SE_MARS` | yes | yes | yes | yes |
| 5 | `Jupiter` | 5 | `SE_JUPITER` | yes | yes | yes | yes |
| 6 | `Saturn` | 6 | `SE_SATURN` | yes | yes | yes | yes |
| 7 | `Uranus` | 7 | `SE_URANUS` | yes | yes | yes | yes |
| 8 | `Neptune` | 8 | `SE_NEPTUNE` | yes | yes | yes | yes |
| 9 | `MeanNode` | 10 | `SE_MEAN_NODE` | yes | yes | yes | yes |
| 10 | `TrueNode` | 11 | `SE_TRUE_NODE` | yes | yes | yes | yes |
| 11 | `Pluto` | 9 | `SE_PLUTO` | yes | yes | yes | yes |
| 12 | `Chiron` | 15 | `SE_CHIRON` | yes | yes | yes | yes |
| 13 | Ketu (`MeanNode` + 180°) | - | - | yes | yes | yes | yes |
| - | `MeanApogee` | 12 | `SE_MEAN_APOG` | - | - | - | - |
| - | `OsculatingApogee` | 13 | `SE_OSCU_APOG` | - | - | - | - |
| - | `Earth` | 14 | `SE_EARTH` | - | - | - | - |
<!-- END GENERATED bodies -->

- Ketu (`13`) is not a `Body`: the bindings compute it as the mean node plus
  180°, with the latitude negated and the mean node's speed and retrograde
  state. WebAssembly `bodyName(13)` returns the "Invalid body ID" message,
  because it only names `Body` values.
- The Swiss number column is what `xalen.swe.calc_ut` / `calc` and Rust
  `compat::swe_calc_ut` accept; `xalen.swe` exposes each constant under both
  spellings (`PLUTO` and `SE_PLUTO`). Rows with ID `-` are reachable only
  through the Swiss numbers.

## Ayanamsas

Binding IDs:

<!-- BEGIN GENERATED ayanamsas -->
| ID | `Ayanamsa` | Swiss `SE_SIDM` number | C (FFI) | WASM | Node | Python |
|---|---|---|---|---|---|---|
| 0 | `Lahiri` | 1 | yes | yes | yes | yes |
| 1 | `KPKrishnamurti` | 5 | yes | yes | yes | yes |
| 2 | `Raman` | 3 | yes | yes | yes | yes |
| 3 | `FaganBradley` | 0 | yes | yes | yes | yes |
| 4 | `TrueChitra` | 27 | yes | yes | yes | yes |
| 5 | `TrueRevati` | 28 | yes | yes | yes | yes |
| 6 | `SuryaSiddhanta` | 21 | yes | yes | yes | yes |
| 7 | `YukteswarSriSS` | 7 | yes | yes | yes | yes |
| 8 | `JNBhasin` | 8 | yes | yes | yes | yes |
| 9 | `DeLuce` | 2 | yes | yes | yes | yes |
| 10 | `Ushashashi` | 4 | yes | yes | yes | yes |
| 11 | `PushyaPaksha` | - | yes | yes | yes | yes |
| 12 | `GalacticCenter0Sag` | 17 | yes | yes | yes | yes |
| 13 | `LahiriICRC` | 46 | yes | yes | yes | yes |
| 14 | `KPStraightLine` | - | yes | yes | yes | yes |
| 15 | `Hipparchos` | 15 | yes | yes | yes | yes |
| 16 | `LahiriVP285` | 44 | yes | yes | yes | yes |
<!-- END GENERATED ayanamsas -->

A `-` in the Swiss column means that system has no Swiss Ephemeris number.

Swiss numbers, accepted by `xalen.swe.set_sid_mode` (which selects the
ayanamsa that sidereal `calc_ut`, `houses_ex` and `get_ayanamsa_*` use), by
Rust `compat::swe_set_sid_mode` and `compat::swe_get_ayanamsa_ut_ex`, and by
`Ayanamsa::from_swiss_ephem_id`. A number without a named `xalen.swe`
constant can be passed as an integer, for example `swe.set_sid_mode(46)`.

<!-- BEGIN GENERATED swiss-ayanamsas -->
| Swiss number | `Ayanamsa` | `xalen.swe` constant | Binding ID |
|---|---|---|---|
| 0 | `FaganBradley` | `SIDM_FAGAN_BRADLEY` / `SE_SIDM_FAGAN_BRADLEY` | 3 |
| 1 | `Lahiri` | `SIDM_LAHIRI` / `SE_SIDM_LAHIRI` | 0 |
| 2 | `DeLuce` | `SIDM_DELUCE` / `SE_SIDM_DELUCE` | 9 |
| 3 | `Raman` | `SIDM_RAMAN` / `SE_SIDM_RAMAN` | 2 |
| 4 | `Ushashashi` | `SIDM_USHASHASHI` / `SE_SIDM_USHASHASHI` | 10 |
| 5 | `KPKrishnamurti` | `SIDM_KRISHNAMURTI` / `SE_SIDM_KRISHNAMURTI` | 1 |
| 6 | `DjwhalKhul` | `SIDM_DJWHAL_KHUL` / `SE_SIDM_DJWHAL_KHUL` | - |
| 7 | `YukteswarSriSS` | `SIDM_YUKTESWAR` / `SE_SIDM_YUKTESWAR` | 7 |
| 8 | `JNBhasin` | `SIDM_JN_BHASIN` / `SE_SIDM_JN_BHASIN` | 8 |
| 9 | `BabylonianKugler1` | - | - |
| 10 | `BabylonianKugler2` | - | - |
| 11 | `BabylonianKugler3` | - | - |
| 12 | `BabylonianHuber` | - | - |
| 13 | `BabylonianMercier` | - | - |
| 14 | `Aldebaran15Tau` | - | - |
| 15 | `Hipparchos` | - | 15 |
| 16 | `Sassanian` | - | - |
| 17 | `GalacticCenter0Sag` | - | 12 |
| 18 | `J2000` | - | - |
| 19 | `J1900` | - | - |
| 20 | `B1950` | - | - |
| 21 | `SuryaSiddhanta` | - | 6 |
| 22 | `SuryaSiddhantaMeanSun` | - | - |
| 23 | `Aryabhata` | - | - |
| 24 | `AryabhataMeanSun` | - | - |
| 25 | `SuryaSiddhantaRevati` | - | - |
| 26 | `SuryaSiddhantaCitra` | - | - |
| 27 | `TrueChitra` | `SIDM_TRUE_CITRA` / `SE_SIDM_TRUE_CITRA` | 4 |
| 28 | `TrueRevati` | `SIDM_TRUE_REVATI` / `SE_SIDM_TRUE_REVATI` | 5 |
| 29 | `TruePushyaDeltaCancri` | - | - |
| 30 | `GalacticCenterBrand` | - | - |
| 31 | `GalacticEquatorIAU1958` | - | - |
| 32 | `GalacticEquatorTrue` | - | - |
| 33 | `GalacticEquatorMula` | - | - |
| 34 | `GalacticAlignMardyks` | - | - |
| 35 | `TrueMulaChandraHari` | - | - |
| 36 | `GalCenterMulaWilhelm` | - | - |
| 37 | `Aryabhata522` | - | - |
| 38 | `BabylonianBritton` | - | - |
| 39 | `TrueSheoran` | - | - |
| 40 | `GalacticCenterCochrane` | - | - |
| 41 | `GalacticEquatorialFiorenza` | - | - |
| 42 | `ValensMoon` | - | - |
| 43 | `Lahiri1940` | - | - |
| 44 | `LahiriVP285` | - | 16 |
| 45 | `KrishnamurtiVP291` | - | - |
| 46 | `LahiriICRC` | - | 13 |
<!-- END GENERATED swiss-ayanamsas -->

## House systems

<!-- BEGIN GENERATED houses -->
| ID | `HouseSystem` | Swiss `hsys` letter | C (FFI) | WASM | Node | Python |
|---|---|---|---|---|---|---|
| 0 | `WholeSign` | `W` | yes | yes | yes | yes |
| 1 | `Equal` | `A`, `E` | yes | yes | yes | yes |
| 2 | `Placidus` | `P` | yes | yes | yes | yes |
| 3 | `Koch` | `K` | yes | yes | yes | yes |
| 4 | `Porphyry` | `O` | yes | yes | yes | yes |
| 5 | `Regiomontanus` | `R` | yes | yes | yes | yes |
| 6 | `Campanus` | `C` | yes | yes | yes | yes |
| 7 | `Morinus` | `M` | yes | yes | yes | yes |
| 8 | `Alcabitius` | `B` | yes | yes | yes | yes |
| 9 | `Topocentric` | `T` | yes | yes | yes | yes |
| 10 | `Sripati` | `S` | yes | yes | yes | yes |
| 11 | `Vehlow` | `V` | yes | yes | yes | yes |
| 12 | `Meridian` | `X` | yes | yes | yes | yes |
| 13 | `KrusinskiPisa` | `U` | yes | yes | yes | yes |
| - | `CarterPoliEquatorial` | `F` | - | - | - | - |
| - | `Gauquelin` | `G` | - | - | - | - |
| - | `SunshineTreindl` | `I` | - | - | - | - |
| - | `PullenSinusoidalDelta` | `L` | - | - | - | - |
| - | `PullenSinusoidalRatio` | `Q` | - | - | - | - |
| - | `APC` | `Y` | - | - | - | - |
| - | `SunshineMakransky` | `i` | - | - | - | - |
<!-- END GENERATED houses -->

The Swiss letters are what `xalen.swe.houses` / `houses_ex` (as `bytes` or
`str`, for example `b'P'`) and Rust `compat::swe_houses` accept. Rows with ID
`-` are reachable only through their letter. `HouseSystem::Zariel` and
`HouseSystem::AlcabitiusClassic` have neither a binding ID nor a Swiss letter;
use them through the Rust `xalen_houses` API.

## Divisional charts

WebAssembly `divisionalChart(lonDeg, varga)` takes the division number itself:

<!-- BEGIN GENERATED vargas -->
| ID | `VargaChart` | C (FFI) | WASM | Node | Python |
|---|---|---|---|---|---|
| 1 | `D1` | - | yes | - | - |
| 2 | `D2` | - | yes | - | - |
| 3 | `D3` | - | yes | - | - |
| 4 | `D4` | - | yes | - | - |
| 7 | `D7` | - | yes | - | - |
| 9 | `D9` | - | yes | - | - |
| 10 | `D10` | - | yes | - | - |
| 12 | `D12` | - | yes | - | - |
| 16 | `D16` | - | yes | - | - |
| 20 | `D20` | - | yes | - | - |
| 24 | `D24` | - | yes | - | - |
| 27 | `D27` | - | yes | - | - |
| 30 | `D30` | - | yes | - | - |
| 40 | `D40` | - | yes | - | - |
| 45 | `D45` | - | yes | - | - |
| 60 | `D60` | - | yes | - | - |
<!-- END GENERATED vargas -->

## `xalen.swe` flags

The `flags` argument of `xalen.swe.calc_ut` / `calc` is a bit set. Flags that
would change the frame, centre or units of the result in Swiss Ephemeris are
rejected with a `ValueError` instead of being ignored.

<!-- BEGIN GENERATED swe-flags -->
| Value | `xalen.swe` names | `calc_ut` |
|---|---|---|
| 2 | `FLG_SWIEPH` / `SEFLG_SWIEPH` | accepted |
| 8 | `FLG_HELCTR` / `SEFLG_HELCTR` | rejected (`ValueError`) |
| 32 | `FLG_J2000` / `SEFLG_J2000` | rejected (`ValueError`) |
| 256 | `FLG_SPEED` / `SEFLG_SPEED` | accepted |
| 2048 | `FLG_EQUATORIAL` / `SEFLG_EQUATORIAL` | rejected (`ValueError`) |
| 4096 | `FLG_XYZ` / `SEFLG_XYZ` | rejected (`ValueError`) |
| 8192 | `FLG_RADIANS` / `SEFLG_RADIANS` | rejected (`ValueError`) |
| 16384 | `FLG_BARYCTR` / `SEFLG_BARYCTR` | rejected (`ValueError`) |
| 32768 | `FLG_TOPOCTR` / `SEFLG_TOPOCTR` | rejected (`ValueError`) |
| 65536 | `FLG_SIDEREAL` / `SEFLG_SIDEREAL` | accepted |
<!-- END GENERATED swe-flags -->

## `xalen.swe` calendars

The `cal` argument of `xalen.swe.julday` and `revjul`:

<!-- BEGIN GENERATED swe-calendars -->
| Value | `xalen.swe` names |
|---|---|
| 0 | `JUL_CAL` / `SE_JUL_CAL` |
| 1 | `GREG_CAL` / `SE_GREG_CAL` |
<!-- END GENERATED swe-calendars -->

## C status codes

Not generated; from `crates/xalen-ffi/src/lib.rs` and its tests.

| Return | Functions | Meaning |
|---|---|---|
| `0` | `int` functions | Success; the `status` field of the output struct is also `0`. |
| `-1` | `int` functions | The output pointer is null. |
| `-2` | `int` functions | Invalid body or house-system ID, or a non-finite or out-of-range input (NaN or infinite `jd`, `lat`, `lon`; latitude outside ±90°). |
| `-3` | `int` functions | The computation failed. |
| `-99` | `int` functions | An internal panic was caught at the C boundary. |
| `-1.0` | `xalen_sidereal_longitude` | Invalid body ID, or the computation failed. |
| `-2.0` | `xalen_sidereal_longitude` | Invalid ayanamsa ID, or a non-finite `jd_ut1`. |
| `-1.0` | `xalen_ayanamsa` | Invalid ayanamsa ID, or a non-finite `jd_ut1`. |
| `-99.0` | `double` functions | An internal panic was caught at the C boundary. |

The crate ships no C header; declare the functions from
[`crates/xalen-ffi/README.md`](../crates/xalen-ffi/README.md), with every ID
argument as an `int`.
