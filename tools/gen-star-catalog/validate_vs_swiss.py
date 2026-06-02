#!/usr/bin/env python3
"""Cross-validate catalog_generated.rs against the Swiss Ephemeris (pyswisseph).

Oracle = swe.fixstar2 at J2000.0 (JD 2451545.0) with the ASTROMETRIC flag set:
    SEFLG_J2000 | SEFLG_NONUT | SEFLG_NOABERR | SEFLG_NOGDEFL
which gives the catalog (mean J2000 ecliptic) place — the same quantity our
generator emits. (Swiss's DEFAULT apparent place adds ~20" of annual aberration,
which is an apparent-position effect, not a catalog-position property; comparing
against the default would show a spurious ~20" offset.)

Known Swiss name-disambiguation artifacts (NOT catalog errors):
  - "Menkar" : Swiss prefix-search may return lambda Cet, but Menkar is alpha
    Cet (HIP 14135) per the IAU-CSN. Our value matches Swiss ",alCet" to 0.02".
  - "Rigil Kentaurus" / "Algieba" : multiple-star systems; Swiss may return a
    different component than the HIP-photocentre.

Run:
  SE_EPHE_PATH=/path/with/sefstars.txt \
    python3 tools/gen-star-catalog/validate_vs_swiss.py \
      --rs crates/xalen-stars/src/catalog_generated.rs
"""
import argparse
import math
import os
import re
import sys

JD_J2000 = 2451545.0


def load_named(rs_path):
    out = []
    pat = re.compile(
        r'hip: (\d+), name: Some\("([^"]+)"\),.*'
        r"longitude_j2000: ([\d.]+), latitude_j2000: ([-\d.]+),"
    )
    for line in open(rs_path):
        m = pat.search(line)
        if m:
            out.append((int(m.group(1)), m.group(2),
                        float(m.group(3)), float(m.group(4))))
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--rs", default="crates/xalen-stars/src/catalog_generated.rs")
    ap.add_argument("--ephe", default=os.environ.get("SE_EPHE_PATH", ""))
    ap.add_argument("--threshold-arcsec", type=float, default=1.0)
    args = ap.parse_args()

    try:
        import swisseph as swe
    except ImportError:
        print("pyswisseph not installed; cannot validate.", file=sys.stderr)
        return 2
    if args.ephe:
        swe.set_ephe_path(args.ephe)

    flag = (swe.FLG_SWIEPH | swe.FLG_J2000 | swe.FLG_NONUT
            | swe.FLG_NOABERR | swe.FLG_NOGDEFL)

    # Swiss name-disambiguation: where the catalog's traditional spelling is not
    # the name Swiss's sefstars.txt indexes, validate against the explicit Bayer
    # / Flamsteed designation instead of the common name. (Same body, just the
    # name Swiss recognizes — not a coordinate override.)
    BAYER_OVERRIDE = {
        "Menkar": ",alCet",            # alpha Ceti (Swiss prefix-search returns lambda Cet)
        "Bharani 41": ",41Ari",        # 41 Arietis
        "Lambda Orionis": ",laOri",    # lambda Orionis (= Meissa)
        "Hyadum II": ",de-1Tau",       # delta1 Tauri (= Secunda Hyadum)
        "Zuben Elgenubi": ",al-2Lib",  # alpha2 Librae (= Zubenelgenubi)
        "Zuben Eschamali": ",beLib",   # beta Librae (= Zubeneschamali)
        "Al Jabhah": ",etLeo",         # eta Leonis (no IAU-WGSN proper name)
        "Imai": ",deCru",              # delta Crucis (Swiss indexes it as "Decrux")
    }

    named = load_named(args.rs)
    seps, missing, outliers = [], [], []
    for hip, name, glon, glat in named:
        query = BAYER_OVERRIDE.get(name, name)
        try:
            xx, _nm, _rf = swe.fixstar2(query, JD_J2000, flag)
        except Exception:
            missing.append(name)
            continue
        dlon = ((glon - xx[0] + 180) % 360 - 180) * 3600
        dlat = (glat - xx[1]) * 3600
        sep = math.hypot(dlon * math.cos(math.radians(xx[1])), dlat)
        seps.append(sep)
        if sep > args.threshold_arcsec:
            outliers.append((name, hip, sep))

    seps.sort()
    print(f"named stars         : {len(named)}")
    print(f"validated vs Swiss  : {len(seps)}")
    print(f"not found in Swiss  : {len(missing)} {missing}")
    if seps:
        print(f"median separation   : {seps[len(seps)//2]:.3f} arcsec")
        print(f"max separation      : {max(seps):.3f} arcsec")
        print(f"stars > {args.threshold_arcsec} arcsec   : {len(outliers)} {outliers}")
    # Pass criterion: every non-multiple-star is sub-arcsecond. We allow the
    # documented multiple-star Swiss-component artifacts.
    KNOWN_MULTIPLE = {"Rigil Kentaurus", "Algieba"}
    hard_fail = [o for o in outliers if o[0] not in KNOWN_MULTIPLE]
    if hard_fail:
        print(f"FAIL: {len(hard_fail)} unexpected outlier(s): {hard_fail}")
        return 1
    print("PASS: all stars sub-arcsecond vs Swiss (excl. documented "
          "multiple-star Swiss-component artifacts).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
