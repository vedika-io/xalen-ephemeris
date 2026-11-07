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
<!-- END GENERATED ayanamsas -->

A `-` in the Swiss column means that system has no Swiss Ephemeris number.

Swiss numbers, accepted by `xalen.swe.set_sid_mode` (which selects the
ayanamsa that sidereal `calc_ut`, `houses_ex` and `get_ayanamsa_*` use), by
Rust `compat::swe_set_sid_mode` and `compat::swe_get_ayanamsa_ut_ex`, and by
`Ayanamsa::from_swiss_ephem_id`. A number without a named `xalen.swe`
constant can be passed as an integer, for example `swe.set_sid_mode(46)`.

<!-- BEGIN GENERATED swiss-ayanamsas -->
<!-- END GENERATED swiss-ayanamsas -->

## House systems

<!-- BEGIN GENERATED houses -->
<!-- END GENERATED houses -->

The Swiss letters are what `xalen.swe.houses` / `houses_ex` (as `bytes` or
`str`, for example `b'P'`) and Rust `compat::swe_houses` accept. Rows with ID
`-` are reachable only through their letter. `HouseSystem::Zariel` and
`HouseSystem::AlcabitiusClassic` have neither a binding ID nor a Swiss letter;
use them through the Rust `xalen_houses` API.

## Divisional charts

WebAssembly `divisionalChart(lonDeg, varga)` takes the division number itself:

<!-- BEGIN GENERATED vargas -->
<!-- END GENERATED vargas -->

## `xalen.swe` flags

The `flags` argument of `xalen.swe.calc_ut` / `calc` is a bit set. Flags that
would change the frame, centre or units of the result in Swiss Ephemeris are
rejected with a `ValueError` instead of being ignored.

<!-- BEGIN GENERATED swe-flags -->
<!-- END GENERATED swe-flags -->

## `xalen.swe` calendars

The `cal` argument of `xalen.swe.julday` and `revjul`:

<!-- BEGIN GENERATED swe-calendars -->
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
