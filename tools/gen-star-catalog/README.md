# gen-star-catalog

Generates `crates/xalen-stars/src/catalog_generated.rs` — the expanded fixed-star
catalog (Vmag ≤ 6.5, ~8,870 stars) — directly from the real **Hipparcos Main
Catalogue** (CDS catalogue I/239, `hip_main.dat`, 118,218 records).

Every coordinate, magnitude, and proper-motion value in the generated file comes
from a real `hip_main.dat` record. **No value is invented or interpolated.**

## Reproduce

```bash
# 1. Download the real Hipparcos Main Catalogue (53 MB, 118,218 records).
curl -sSL -o /tmp/hip_main.dat \
  "https://cdsarc.cds.unistra.fr/ftp/cats/I/239/hip_main.dat"

# 2. Extract the curated star table from lib.rs (for the name overlay).
#    (Produces /tmp/curated_stars.json — see the snippet in this dir's git log,
#     or re-run the small regex extractor used in the original generation.)

# 3. Generate. IAU-CSN.txt (the name authority, shipped in this dir) is used
#    by default for the traditional-name join.
python3 gen_catalog.py \
  --hip /tmp/hip_main.dat \
  --curated /tmp/curated_stars.json \
  --out ../../crates/xalen-stars/src/catalog_generated.rs
```

## Pipeline

1. Parse `HIP`, `Vmag`, `RAdeg`, `DEdeg`, `pmRA` (= μα·cos δ), `pmDE` from the
   fixed-width records (byte positions from the official I/239 `ReadMe`).
2. Filter `Vmag ≤ 6.5` → 8,870 stars.
3. Propagate the J1991.25 catalogue epoch forward **8.75 yr to J2000.0** using
   pmRA/pmDE (Hipparcos is NOT a J2000 catalogue).
4. Convert RA/Dec → ecliptic at the IAU 2006 J2000 mean obliquity
   (23.4392794444°), matching `xalen-coords::equatorial_to_ecliptic`.
5. Derive ecliptic-frame proper motion (mas/yr) by rotating the equatorial
   tangential-velocity vector — so the Rust struct's `pm_lon`/`pm_lat` are
   populated from the measured pmRA/pmDE.
6. **LEFT-JOIN traditional names** via the **IAU Catalog of Star Names (IAU-CSN)**
   authority: curated name → HIP (from IAU-CSN) → generated star. A real,
   sourced crossmatch — never an invented HIP↔name table. 106/108 curated names
   join. A few curated names use a traditional spelling that differs from the
   IAU-WGSN name (e.g. `Zuben Elgenubi`→`Zubenelgenubi`, `Lambda Orionis`→`Meissa`,
   `Hyadum II`→`Secunda Hyadum`, `Bharani 41`→`Bharani`); these are resolved via
   the sourced `CURATED_TO_IAU` alias map. `Al Jabhah` (η Leonis) has no IAU-WGSN
   proper name, so it is joined by its documented `CURATED_TO_HIP` designation
   (HIP 49583). The only two remaining misses are the open clusters **Pleiades**
   and **Praesepe**, which have no single HIP star.

## Validation

`validate_vs_swiss.py` cross-checks the generated positions against the **Swiss
Ephemeris** (`pyswisseph`, `swe_fixstar2`) at J2000.0 with astrometric flags
(`J2000 | NONUT | NOABERR | NOGDEFL` — the catalog place, not the apparent place):

```bash
SE_EPHE_PATH=/path/to/sefstars.txt python3 validate_vs_swiss.py
```

Result (pyswisseph 2.10.03): all **106 named stars** validate, **median 0.031″**,
max 5.965″. The only stars over 1″ are documented multiple-star Swiss
name-component artifacts (Rigil Kentaurus = α Cen, Algieba = γ Leo). Where the
catalog's traditional spelling is not the name Swiss indexes, the validator
queries the Bayer/Flamsteed designation instead (`BAYER_OVERRIDE` — same body,
e.g. Menkar = α Cet `,alCet`, Bharani 41 = 41 Arietis `,41Ari`, Lambda Orionis =
λ Ori `,laOri`, Al Jabhah = η Leo `,etLeo`). A baked-in subset of these
Swiss-verified anchors is enforced at build time by
`tests/star_catalog_swiss_crossval.rs`.

## Data & license

- **Hipparcos**: The Hipparcos and Tycho Catalogues, © ESA 1997 (ESA SP-1200),
  CDS I/239. See repo `NOTICE` / `CREDITS.md`.
- **IAU-CSN**: IAU WGSN, CC-BY. `IAU-CSN.txt` is committed alongside this tool.
- A *comprehensive* COMMERCIAL claim over the Hipparcos-derived catalog still
  requires CDS/ESA redistribution clearance — see the gate note in `NOTICE`.
