#!/usr/bin/env python3
"""Rewrite the numeric columns of crates/xalen-western/src/stars.rs from the two
sources that carry no non-commercial restriction:

  * IAU Catalog of Star Names (tools/gen-star-catalog/IAU-CSN.txt) for the J2000
    position of every star that has an IAU row;
  * Yale Bright Star Catalogue, 5th rev. ed. (CDS V/50) for proper motion,
    magnitude, and the position of every star without an IAU row.

For every `star_raw(name, designation, ra, dec, mag, pm_ra, pm_dec, lon, lat)`
entry the columns ra_deg, dec_deg, magnitude, pm_ra, pm_dec, ecl_lon_deg and
ecl_lat_deg are replaced; name, designation and order are kept.

Matching (first hit wins):
  1. the entry name equals an IAU-CSN ASCII name whose row joins to a Bright
     Star Catalogue star (HD number, then "HR nnn" designation);
  2. the entry designation equals a Bayer / Flamsteed designation of the
     Bright Star Catalogue (nearest to the old position when several stars
     share it, e.g. Alpha1/Alpha2 Centauri);
  3. the Bright Star Catalogue star nearest to the old position, within 60
     arcsec and 0.5 mag.
When rule 1 and the entry's own designation name different stars (the old
table has a few such rows), the one nearer the old position wins; those rows
are listed in the report.

An entry that none of the three rules can place is NOT kept: its old values
came from sources this repository no longer uses, and none is invented. It is
removed from the table and listed in the report.

Run:
  python3 tools/gen-star-catalog/regen_western_stars.py --bsc /path/to/catalog
  (add --report to print the match report without writing the file)
"""
import argparse
import importlib.util
import math
import os
import re
import sys

here = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("gen_bright_catalog", os.path.join(here, "gen_bright_catalog.py"))
gb = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gb)

ENTRY = re.compile(
    r'(?P<indent>[ \t]*)star_raw\(\s*"(?P<name>[^"]*)",\s*"(?P<desig>[^"]*)",\s*'
    r"(?P<nums>(?:[-\d.]+,\s*){6}[-\d.]+),?\s*\),?[ \t]*\n"
)

POS_TOL_ARCSEC = 60.0
MAG_TOL = 0.5


def norm_desig(s):
    """'Alpha1 Centauri' / 'Alpha Centauri' -> 'alpha centauri' (drop the digit
    that tells Greek-letter components apart)."""
    return re.sub(r"^([A-Za-z]+)\d\b", r"\1", s).lower()


def main():
    root = os.path.abspath(os.path.join(here, "..", ".."))
    ap = argparse.ArgumentParser()
    ap.add_argument("--bsc", required=True)
    ap.add_argument("--iau", default=os.path.join(here, "IAU-CSN.txt"))
    ap.add_argument("--rs", default=os.path.join(root, "crates/xalen-western/src/stars.rs"))
    ap.add_argument("--report", action="store_true")
    ap.add_argument("--skip-sha", action="store_true")
    args = ap.parse_args()

    import hashlib
    digest = hashlib.sha256(open(args.bsc, "rb").read()).hexdigest()
    if digest != gb.BSC_SHA256 and not args.skip_sha:
        sys.exit(f"BSC catalog sha256 {digest} != expected {gb.BSC_SHA256}")

    bsc, _ = gb.parse_bsc(args.bsc)
    iau = gb.parse_iau(args.iau)
    by_hr = {s["hr"]: s for s in bsc}
    by_hd = {}
    for s in bsc:
        if s["hd"] is not None:
            by_hd.setdefault(s["hd"], []).append(s)
    desig_index = {}
    for s in bsc:
        d = gb.designation(s)
        if not d.startswith("HR "):
            desig_index.setdefault(norm_desig(d), []).append(s)

    # IAU name -> (IAU row, BSC star or None); BSC HR -> IAU row.
    iau_by_name = {}
    iau_by_hr = {}
    for r in iau:
        target = gb.join_iau_to_bsc(r, by_hr, by_hd)
        if target is not None and gb.sep_arcsec(r["ra"], r["dec"], target["ra"], target["dec"]) > gb.JOIN_SEP_REFUSE_ARCSEC:
            target = None
        iau_by_name.setdefault(r["name"].lower(), (r, target))
        if target is not None:
            iau_by_hr.setdefault(target["hr"], r)

    src = open(args.rs, encoding="utf-8").read()
    entries = list(ENTRY.finditer(src))
    print(f"parsed {len(entries)} star_raw entries")

    def sep_to(ra, dec, s):
        return gb.sep_arcsec(ra, dec, s["ra"], s["dec"])

    out = []
    pos = 0
    n_iau = n_bsc = 0
    dropped = []
    log = {"iau-name": 0, "designation": 0, "nearest": 0}
    warnings = []
    old_off = []
    placed = {}
    nearest_hits = []
    conflicts = []
    max_move = []
    for m in entries:
        nums = [float(x) for x in re.findall(r"[-\d.]+", m.group("nums"))]
        old_ra, old_dec, old_mag = nums[0], nums[1], nums[2]
        name, desig = m.group("name"), m.group("desig")

        star = None
        how = None
        hit = iau_by_name.get(name.lower())
        if hit is not None:
            iau_row, star = hit
            if star is None:
                # IAU row without a Bright Star Catalogue star (a fainter star):
                # no proper motion is available from the allowed sources.
                dropped.append((name, desig, "IAU-CSN row, star not in the Bright Star Catalogue"))
                out.append((m, None))
                continue
            how = "iau-name"
        cands = desig_index.get(norm_desig(desig), [])
        if cands:
            # Several stars can share a designation (Alpha1/Alpha2 Centauri,
            # resolved doubles): take the one nearest the old entry, magnitude
            # weighing in so a 2" pair is told apart.
            dstar = min(
                cands,
                key=lambda c: sep_to(old_ra, old_dec, c) + 100.0 * abs(c["vmag"] - old_mag),
            )
            if star is None:
                star, how = dstar, "designation"
            elif dstar is not star and norm_desig(desig) != norm_desig(gb.designation(star)):
                # The IAU name and the entry's own Bayer/Flamsteed designation
                # name different stars. The old position is the tie-break.
                if sep_to(old_ra, old_dec, dstar) < sep_to(old_ra, old_dec, star):
                    conflicts.append((name, desig, gb.designation(star), star["hr"], dstar["hr"]))
                    star, how = dstar, "designation"
        if star is not None:
            d = sep_to(old_ra, old_dec, star)
            if d > 300.0:
                old_off.append((d, name, how))
        if star is None:
            near = [
                c for c in bsc
                if abs(c["vmag"] - old_mag) <= MAG_TOL and sep_to(old_ra, old_dec, c) <= POS_TOL_ARCSEC
            ]
            if near:
                star = min(near, key=lambda c: sep_to(old_ra, old_dec, c))
                how = "nearest"
        if star is None:
            dropped.append((name, desig, "no Bright Star Catalogue star within 60\" and 0.5 mag; no IAU row"))
            out.append((m, None))
            continue

        # The IAU-CSN position (6 decimals) when the star has an IAU row,
        # else the Bright Star Catalogue position.
        row = iau_by_hr.get(star["hr"])
        if row is not None:
            ra, dec = row["ra"], row["dec"]
            n_iau += 1
        else:
            ra, dec = star["ra"], star["dec"]
            n_bsc += 1
        log[how] += 1
        placed.setdefault(star["hr"], []).append(name)
        if how == "nearest":
            nearest_hits.append((name, desig, star["hr"], sep_to(old_ra, old_dec, star)))
        lon, lat = gb.eq_to_ecl(ra, dec)
        move = gb.sep_arcsec(old_ra, old_dec, ra, dec)
        max_move.append((move, name))
        mag = star["vmag"]
        out.append((m, dict(
            ra=ra, dec=dec, mag=mag, pmra=star["pmra"], pmde=star["pmde"], lon=lon, lat=lat,
            hr=star["hr"], how=how,
        )))

    print(f"sourced from an IAU-CSN position : {n_iau}")
    print(f"sourced from the BSC position    : {n_bsc}")
    print(f"matched by: {log}")
    print(f"dropped (cannot be sourced)      : {len(dropped)}")
    for d in dropped:
        print(f"   {d[0]!r:34} {d[1]!r:34} {d[2]}")
    print(f"IAU name and designation named different stars; old position decided for the designation: {len(conflicts)}")
    for c in conflicts:
        print(f"   {c[0]!r:18} entry designation {c[1]!r:26} IAU name -> {c[2]!r} (HR {c[3]}); used HR {c[4]}")
    print(f"matched by nearest star: {nearest_hits}")
    shared = {hr: n for hr, n in placed.items() if len(n) > 1}
    print(f"entries that resolve to the same star (kept as separate rows): {shared}")
    old_off.sort(reverse=True)
    print(f"entries whose OLD position was more than 5 arcmin from the identified star: {len(old_off)}")
    for d, n, h in old_off[:12]:
        print(f"   {n!r:22} {d / 3600.0:7.2f} deg  (identified by {h})")
    max_move.sort(reverse=True)
    print("largest move from the old position (arcsec):", [(f"{a:.1f}", b) for a, b in max_move[:8]])

    if args.report:
        return

    buf = []
    last = 0
    for m, new in out:
        buf.append(src[last:m.start()])
        if new is not None:
            ind = m.group("indent")
            ind2 = ind + "    "
            buf.append(
                f'{ind}star_raw(\n'
                f'{ind2}"{m.group("name")}",\n'
                f'{ind2}"{m.group("desig")}",\n'
                f'{ind2}{new["ra"]:.6f},\n'
                f'{ind2}{new["dec"]:.6f},\n'
                f'{ind2}{new["mag"]:.2f},\n'
                f'{ind2}{new["pmra"]:.3f},\n'
                f'{ind2}{new["pmde"]:.3f},\n'
                f'{ind2}{new["lon"]:.6f},\n'
                f'{ind2}{new["lat"]:.6f},\n'
                f'{ind}),\n'
            )
        last = m.end()
    buf.append(src[last:])
    open(args.rs, "w", encoding="utf-8").write("".join(buf))
    print(f"rewrote {args.rs}: {len(entries) - len(dropped)} entries kept")


if __name__ == "__main__":
    main()
