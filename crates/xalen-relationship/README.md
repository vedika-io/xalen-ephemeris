# xalen-relationship
> Synastry, composite and Davison charts for two people, with a relocated-chart helper.

Part of the [XALEN Ephemeris](https://github.com/vedika-io/xalen-ephemeris) suite — pure-Rust, thread-safe, Apache-2.0.

## What it computes

- **`NatalChart`** — tropical geocentric body positions (with daily motion) and house cusps for one moment and place, all on the true equinox of date. `relocated` recasts the houses for another place; `to_chart_data` hands the chart to `xalen-chart` for rendering.
- **Synastry** — the aspect grid between every point of chart A and every point of chart B, and where each chart's points fall in the other's houses (`synastry`, `inter_aspects`, `house_overlay`).
- **Composite chart** — each body at the shorter-arc midpoint of the two natal longitudes; houses from the midpoint of the two Midheavens (Ascendant derived at the mean latitude), or alternatively the midpoint of the two Ascendants.
- **Davison chart** — a real chart for the mean of the two birth moments at the midpoint of the two birthplaces (arithmetic or great-circle).

## Method and sources

| Part | Source |
|------|--------|
| Synastry grid, house overlays | Robert Hand, *Horoscope Symbols* (1981). Orbs are the `xalen-western` defaults times a multiplier. |
| Composite midpoints and houses | Robert Hand, *Planets in Composite* (1975). The composite MC is the exact midpoint of the MCs; RAMC is recovered with `tan RAMC = tan MC / cos ε` and fed to the house engine at the mean latitude. |
| Davison chart | Ronald Davison, *Synastry* (1977). |
| Sidereal time, obliquity | Meeus, *Astronomical Algorithms*, 2nd ed., ch. 12 and 13; houses use `xalen_houses::compute_houses_true` (apparent sidereal time, true obliquity). |

## Example

```rust
use xalen_relationship::{
    CompositeOptions, NatalChart, SynastryOptions, composite_chart, default_bodies, synastry,
};
use xalen_ephem::Almanac;
use xalen_houses::{GeoLocation, HouseSystem};
use xalen_time::JdUT1;

let almanac = Almanac::default_vedic();
let chart = |jd: f64, lat: f64, lon: f64| {
    NatalChart::compute(&almanac, JdUT1(jd), &GeoLocation::new(lat, lon),
        HouseSystem::Placidus, &default_bodies()).unwrap()
};
let a = chart(2_448_000.5, 51.5, -0.1);
let b = chart(2_449_500.5, 40.7, -74.0);

let syn = synastry(&a, &b, &SynastryOptions::default()).unwrap();
for asp in syn.aspects() {
    println!("{:?} {:?} {:?} orb {:.2}", asp.a, asp.aspect_type, asp.b, asp.orb_deg);
}
let comp = composite_chart(&a, &b, &CompositeOptions::default()).unwrap();
println!("composite MC {:.2}", comp.houses.mc.to_degrees());
```

## Limits

- Geocentric positions only; lunar parallax (up to about 1 degree) is ignored.
- The built-in Pluto series covers 1885-2099; outside it `NatalChart::compute` returns `RelationshipError::Ephemeris`.
- Two points exactly 180 degrees apart have no unique midpoint. `xalen_western::midpoints::compute_midpoint` picks `lon_a + 90`, so a composite of such a pair depends on the order of the charts.
- The midpoint-of-ascendants composite supports only Equal and Whole Sign houses, and its Ascendant and Midheaven are not geometrically linked.
- Aspects are tested with default orbs (no per-body orb policy) and with zero speeds, so applying/separating is not reported.

## Accuracy & sources
Positions and angles come from `xalen-ephem` and `xalen-houses`; see [ACCURACY.md](../../docs/ACCURACY.md) and [CREDITS.md](../../CREDITS.md). The midpoint arithmetic is checked by hand-computed cases and self-consistency tests in `tests/relationship.rs`.
