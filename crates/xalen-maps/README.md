# xalen-maps
> Astrocartography: where on Earth each body stands on the MC, IC, Ascendant or Descendant.

Part of the [XALEN Ephemeris](https://github.com/vedika-io/xalen-ephemeris) suite — pure-Rust, thread-safe, Apache-2.0.

## What it computes

For one moment and a list of bodies, `astro_map` returns for each body

- the **MC** and **IC** lines (meridians where the body culminates, above and below),
- the **ASC** and **DSC** lines (where it is rising and setting), as polylines sampled in latitude, split at the antimeridian and broken where the body is circumpolar or never rises, with the exact limit-latitude vertex,
- the **zenith** point where the body is straight overhead.

The pieces are also public: `mc_longitude_deg`, `ic_longitude_deg`, `horizon_half_arc` and `horizon_lines`.

### Parans

A paran (Brady, *The Eagle and the Lark*) is a place where two bodies are on angles at the same moment: one rising or setting while the other culminates or anti-culminates, or both on the horizon together. On the map these are the crossings of one body's lines with another's. `parans(&map, &ParanOptions)` returns every crossing within the latitude limits (default -66 to 66), ordered by latitude, plus a `skipped` list of the line pairs that do not cross there and why (outside the latitudes, never meeting, or the same line). `parans_near(&map, lat, orb, &ParanOptions)` returns the parans within an orb of one latitude, such as a birthplace's (Brady works with about one degree).

Each crossing is solved in closed form, not read off the drawn polylines:

- **Horizon with meridian.** Along body B's MC or IC meridian at longitude `λ`, body A's hour angle `H = λ - λ_MC(A)` is fixed, so A's horizon line crosses it where `sin φ sin δ + cos φ cos δ cos H = sin h0`. That is `R cos(φ - ψ) = sin h0` with `R = hypot(sin δ, cos δ cos H)` and `ψ = atan2(sin δ, cos δ cos H)`, so `φ = ψ ± acos(sin h0 / R)`, keeping the roots that lie on this half of the meridian. A is rising (ASC) when `H < 0` and setting (DSC) when `H > 0`. With `h0 = 0` this is `tan φ = -cos H / tan δ`, one crossing per line pair.
- **Horizon with horizon.** Both bodies are at altitude `h0`: the intersection of two circles on the sphere centred on the points where each body is overhead (unit vectors `za`, `zb`, `g = za·zb`): `P = a (za + zb) ± c (za × zb)` with `a = sin h0 / (1 + g)` and `c² = (1 - 2 sin² h0 / (1 + g)) / (1 - g²)`. At most two points; with `h0 = 0` they are antipodal.

```rust
use xalen_maps::{astro_map, parans, parans_near, MapOptions, ParanOptions};
use xalen_ephem::{Almanac, Body};
use xalen_time::JdUT1;

let map = astro_map(&Almanac::default_vedic(), JdUT1(2_460_482.5),
                    &[Body::Sun, Body::Moon, Body::Jupiter], &MapOptions::default()).unwrap();
for p in parans(&map, &ParanOptions::default()).unwrap().parans {
    println!("{:?} {:?} with {:?} {:?} at {:.2}, {:.2}",
             p.body_a, p.kind_a, p.body_b, p.kind_b, p.latitude_deg, p.longitude_deg);
}
let at_delhi = parans_near(&map, 28.61, 1.0, &ParanOptions::default()).unwrap();
println!("{} parans within 1 degree of New Delhi's latitude", at_delhi.len());
```

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
- Parans use the same geocentric positions; set `ParanOptions::horizon_altitude_deg` to the value used for the map so the crossings lie on the drawn lines.

## Accuracy & sources
The lines are validated against the independent topocentric altitude and hour-angle code in `xalen-ephem` for the Sun, Mars and Jupiter (altitude within 0.01 degrees of the horizon altitude at every vertex); see `tests/maps.rs`. Every paran is checked the same way in `tests/parans.rs`: a body on the horizon has topocentric altitude within 0.01 degrees of the horizon altitude, and a body on the MC or IC has hour angle within 0.01 degrees of 0 or 180 (the Moon's horizon crossings are held to its parallax, about one degree). For the accuracy of the underlying positions see [ACCURACY.md](../../docs/ACCURACY.md) and [CREDITS.md](../../CREDITS.md).
