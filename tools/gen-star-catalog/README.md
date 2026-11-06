# gen-star-catalog

Generators for the fixed-star data.

| Script | Writes | Source data |
|---|---|---|
| `gen_bright_catalog.py` | `crates/xalen-stars/src/bright_generated.rs` (9,096 stars, the default list) | Yale Bright Star Catalogue + IAU-CSN |
| `regen_western_stars.py` | numeric columns of `crates/xalen-western/src/stars.rs` (503 stars) | Yale Bright Star Catalogue + IAU-CSN; CNS3 and Harris for four objects outside the Bright Star Catalogue |
| `gen_catalog.py` | `crates/xalen-stars-hip-data/src/catalog_generated.rs` (optional, unpublished, non-commercial add-on, 8,870 stars) | Hipparcos Main Catalogue + IAU-CSN |
| `validate_vs_swiss.py` | nothing; compares a generated file with `pyswisseph` | Swiss `sefstars.txt` |

## Bright Star Catalogue list (the default)

Sources, neither with a non-commercial restriction:

- **Yale Bright Star Catalogue, 5th Revised Ed. (Preliminary Version)**,
  Hoffleit D., Warren Jr W.H., Astronomical Data Center, NSSDC/ADC (1991),
  CDS catalogue V/50, file `catalog` (9,110 fixed-width records, 14 of them novae
  or extragalactic objects without a position, so 9,096 stars).
- **IAU Catalog of Star Names (IAU-CSN)**, IAU WGSN, Creative Commons
  Attribution. `IAU-CSN.txt` is committed in this directory.

### Reproduce

```bash
# 1. Download and unpack the Bright Star Catalogue (the raw file is not committed).
mkdir -p /tmp/bsc5 && cd /tmp/bsc5
curl -sSL -o catalog.gz https://vizier.cfa.harvard.edu/ftp/cats/V/50/catalog.gz
gunzip -k catalog.gz
shasum -a 256 catalog
# 69797549cc1605aad7ff94e9325e29a1661f2a253917faaa056d9bf20b809afd  catalog

# 2. Generate (the scripts refuse a file with a different sha256).
cd <repo>
python3 tools/gen-star-catalog/gen_bright_catalog.py --bsc /tmp/bsc5/catalog

# 3. Only for regen_western_stars.py: two more small tables (not committed).
curl -sSL -o /tmp/cns3.dat.gz https://vizier.cfa.harvard.edu/ftp/cats/V/70A/catalog.dat.gz
gunzip -f /tmp/cns3.dat.gz && shasum -a 256 /tmp/cns3.dat
# d12d64aba1f3a0d53cc0a1332f421dc2499675ecbaae566ea339477b9cec9d25  /tmp/cns3.dat
curl -sSL -o /tmp/harris.gz https://vizier.cfa.harvard.edu/ftp/cats/VII/195/catalog.gz
gunzip -f /tmp/harris.gz && shasum -a 256 /tmp/harris
# ebd1f2a4bfbe0da03ec9c026ae846ed5e39178a9daa25d8c34fca6caf323d71b  /tmp/harris
python3 tools/gen-star-catalog/regen_western_stars.py --bsc /tmp/bsc5/catalog \
  --cns3 /tmp/cns3.dat --harris /tmp/harris
```

### Pipeline (`gen_bright_catalog.py`)

1. Parse every record that has a J2000 position: HR, name (Bayer/Flamsteed), HD,
   RA/Dec (equinox J2000, epoch 2000.0, FK5), Vmag, pmRA (already times cos Dec)
   and pmDE in arcsec/yr, converted to mas/yr.
2. Join the IAU-CSN rows to the catalogue on the HD number, then on the
   `HR nnn` designation. If the two name different stars (Alrescha, Izar,
   Tegmine, resolved doubles), the star nearer the IAU position is taken, and the
   IAU's own `HR nnn` when the two are within 1″. 338 IAU rows join (113 IAU rows
   are fainter stars that are not in the catalogue). A join is refused when the
   two positions are more than 120″ apart; none is.
3. A joined star takes the proper name, the constellation and the IAU RA/Dec
   (6 decimals of a degree). Proper motion and magnitude always come from the
   Bright Star Catalogue. Every other star keeps the catalogue position (RA to
   0.1 s, Dec to 1″).
4. Convert RA/Dec to ecliptic longitude/latitude at the IAU 2006 J2000 mean
   obliquity (84381.406″), and rotate the proper motion into ecliptic components
   (mas/yr), as in `gen_catalog.py`.
5. Carry the curated traditional name, constellation and Ptolemaic `nature` from
   `CATALOG` in `crates/xalen-stars/src/lib.rs` to the star it names: curated
   name, or its spelling alias in `CURATED_TO_IAU`, to an IAU-CSN row, to the
   catalogue star. `Al Jabhah` (η Leonis) has no IAU name and is joined by its
   Bayer designation in the catalogue (`CURATED_TO_BSC_BAYER`). 106 of the 108
   curated names join; Pleiades and Praesepe are open clusters.
6. Emit the table sorted by HR.

`designation` is the Bayer or Flamsteed designation (`Alpha Canis Majoris`,
`33 Piscium`) and `HR nnnn` when the catalogue gives the star neither.
`constellation` is filled for the 3,154 stars that have an IAU name or a
Bayer/Flamsteed designation and is empty for the rest (no constellation
boundary data is used).

### `regen_western_stars.py`

Rewrites `ra_deg`, `dec_deg`, `magnitude`, `pm_ra`, `pm_dec`, `ecl_lon_deg`,
`ecl_lat_deg` of every `star_raw(...)` entry; names, designations and order are
kept. An entry is placed by, in order:

1. `HR_OVERRIDES`: an explicit Bright Star Catalogue record (HR number) with the
   reason in the script. Four are Latin-letter designations that the catalogue
   carries only in its variable-star column (Omicron Velorum = `o Vel`, N Velorum
   = `N Vel`, PP Carinae = `PP Car`, L2 Puppis = `L2 Pup`); three are second
   components of stars the catalogue lists as one record (Alnasl2, Epsilon1
   Tauri, Algol B), which keep their entry and point at the primary's record.
2. `EXTERNAL_SOURCES`: objects that are not in the Bright Star Catalogue.
   Barnard's Star (CNS3 `Gl 699`), Proxima Centauri (`Gl 551`) and Wolf 359
   (`Gl 406`) from the Gliese & Jahreiss Preliminary Third Catalogue of Nearby
   Stars (CDS V/70A, 1991; positions B1950 to 1 s of RA and 0.1′ of Dec, total
   proper motion and position angle), converted from B1950 to J2000 with the
   standard FK4 to FK5 rotation matrix (the E-terms, at most 0.34″, are not
   applied); Omega Centauri (NGC 5139) from Harris (1996, CDS VII/195): J2000
   position and integrated V, zero proper motion. When the object has an IAU-CSN
   row (Barnard's Star, Proxima Centauri) the IAU-CSN position is used, moved to
   epoch 2000.0 with the CNS3 proper motion; the IAU-CSN place of Barnard's Star
   is for epoch 1991.25, not 2000.0 (the script checks both against CNS3: they
   agree within 4.0″ at 1991.25 and 2.9″ for Proxima Centauri at 2000.0, against
   91″ if Barnard's Star were taken as 2000.0).
3. The IAU name, then the Bayer/Flamsteed designation, then the nearest catalogue
   star within 60″ and 0.5 mag of the old entry.

The old position of an entry is only a matching key for rule 3; no number of the
old table is copied. Three entries are removed because no star can be identified
(`STILL_DROPPED`, with the reasons): A Velorum and Q Velorum (Lacaille letters
that the catalogue does not list) and Zeta Puppis2 (designation "Gamma Puppis"
does not exist). The run on the committed table: 506 entries, 503 kept (306 with
an IAU-CSN position, 193 with a catalogue position, 4 from CNS3 / Harris).
`--report` prints the match report without writing.

## Hipparcos add-on (optional, non-commercial)

`gen_catalog.py` builds the Hipparcos-derived table of the unpublished
`xalen-stars-hip-data` crate from `hip_main.dat` (CDS I/239, © ESA 1997, CC BY-NC
3.0 IGO). No published crate uses it.

```bash
curl -sSL -o /tmp/hip_main.dat https://cdsarc.cds.unistra.fr/ftp/cats/I/239/hip_main.dat
python3 tools/gen-star-catalog/gen_catalog.py \
  --hip /tmp/hip_main.dat \
  --curated /tmp/curated_stars.json \
  --out crates/xalen-stars-hip-data/src/catalog_generated.rs
```

## Validation

`validate_vs_swiss.py` compares the named stars of a generated file with the
Swiss Ephemeris (`pyswisseph`, `swe_fixstar2`) at J2000.0 with astrometric flags
(`J2000 | NONUT | NOABERR | NOGDEFL`, the catalogue place, not the apparent
place):

```bash
SE_EPHE_PATH=/path/to/sefstars.txt python3 validate_vs_swiss.py
```

The recorded Swiss places that the Rust tests use are in
`crates/xalen-stars/tests/common/mod.rs`. Measured with the Bright Star
Catalogue list: the 43 IAU-named oracle stars are within 0.245″ (Sirius), median
0.026″; Al Jabhah (no IAU name, catalogue position) is 0.79″ away; seven stars
at 1000, 1600, 1900, 2000, 2100, 2400 and 3000 are within 20.6″ (median 0.72″).
Six rows that were once recorded with a 0.05° tolerance were found to be 1.3″ to
27″ from live Swiss output and were re-recorded from it. The Hipparcos
comparison tests (every bright star, and the epoch propagation) are in
`crates/xalen-stars-hip-data/tests/`.

How the live values were taken: `libswe` 2.10.03 (the library inside
`pyswisseph` 2.10.3.2) was built in a throwaway directory on the build machine
from the `pyswisseph` source distribution, with the official `sefstars.txt` from
the Swiss Ephemeris repository, and `swe_fixstar2` was called with the flags
above. Only the resulting numbers are committed, never Swiss files or code.

## Data & licence

- **Yale Bright Star Catalogue**: Hoffleit & Warren (1991), ADC/CDS V/50. See
  repo `NOTICE` / `CREDITS.md`.
- **IAU-CSN**: IAU WGSN, Creative Commons Attribution. `IAU-CSN.txt` is committed
  alongside this tool.
- **CNS3**: Gliese & Jahreiss (1991), CDS V/70A. **Harris (1996)**: AJ 112, 1487,
  CDS VII/195. Both only for `regen_western_stars.py`.
- **Hipparcos** (add-on crate only): The Hipparcos and Tycho Catalogues, © ESA
  1997 (ESA SP-1200), CC BY-NC 3.0 IGO.
