# xalen-maps
> Astrocartography: where on Earth each body stands on the MC, IC, Ascendant or Descendant.

Part of the [XALEN Ephemeris](https://github.com/vedika-io/xalen-ephemeris) suite — pure-Rust, thread-safe, Apache-2.0.

## What it computes

For one moment and a list of bodies, `astro_map` returns for each body

- the **MC** and **IC** lines (meridians where the body culminates, above and below),
- the **ASC** and **DSC** lines (where it is rising and setting), as polylines sampled in latitude, split at the antimeridian and broken where the body is circumpolar or never rises, with the exact limit-latitude vertex,
- the **zenith** point where the body is straight overhead.

The pieces are also public: `mc_longitude_deg`, `ic_longitude_deg`, `horizon_half_arc` and `horizon_lines`.

## Method and sources

Jim Lewis, *Astro\*Carto\*Graphy* (1976), with the spherical astronomy of Meeus, *Astronomical Algorithms*, 2nd ed.: sidereal time (ch. 12), coordinate transformation (ch. 13) and rising, transit and setting (ch. 15).

With hour angle `H = GST + longitude - RA`: the MC line is `H = 0`, the IC line `H = 180`, rising is `H = -H0` and setting `H = +H0`, where `cos H0 = (sin h0 - sin φ sin δ) / (cos φ cos δ)`. Right ascension and declination are the apparent place on the true equinox of date, paired with Greenwich apparent sidereal time. Longitudes are east-positive (Meeus counts west-positive; his signs are not used).

## Example

```rust
use xalen_maps::{LineKind, MapOptions, astro_map};
use xalen_ephem::{Almanac, Body};
use xalen_time::JdUT1;

let almanac = Almanac::default_vedic();
let map = astro_map(&almanac, JdUT1(2_460_482.5), &[Body::Sun, Body::Jupiter],
                    &MapOptions::default()).unwrap();
let sun = &map.bodies[0];
let asc = sun.lines.iter().find(|l| l.kind == LineKind::Asc).unwrap();
for segment in &asc.segments {
    println!("{} points from {:?} to {:?}", segment.len(), segment[0], segment[segment.len() - 1]);
}
println!("Sun overhead at {:?}", sun.zenith);
```

## Limits

- Geocentric only. Lunar parallax (up to about 1 degree) is ignored, so Moon lines are only that accurate; for the Sun and planets the parallax is a few arc seconds at most.
- No refraction unless you set `MapOptions::horizon_altitude_deg` (for example `-0.5667`).
- Pluto is available 1885-2099 on the built-in series; outside it `astro_map` returns `MapError::Ephemeris`.
- Latitudes are sampled (default one degree, -66 to 66), so very close turning points are approximated by the limit vertex only.
- **Not yet included:** parans (latitudes where two bodies are on angles at once; Brady). This is the planned follow-up.

## Accuracy & sources
The lines are validated against the independent topocentric altitude and hour-angle code in `xalen-ephem` for the Sun, Mars and Jupiter (altitude within 0.01 degrees of the horizon altitude at every vertex); see `tests/maps.rs`. For the accuracy of the underlying positions see [ACCURACY.md](../../docs/ACCURACY.md) and [CREDITS.md](../../CREDITS.md).
