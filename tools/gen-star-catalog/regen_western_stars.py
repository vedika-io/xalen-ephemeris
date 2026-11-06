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

Before those rules, an entry named in HR_OVERRIDES is pointed at the Bright
Star Catalogue record given there, and an entry named in EXTERNAL_SOURCES is
filled from the pre-Hipparcos source given there (see the two tables below; each
row says why). The old position of an entry is only ever used as a matching key;
no number of the old table is copied into the new one.

An entry that none of these rules can place is NOT kept: its old values came from
sources this repository no longer uses, and none is invented. It is removed from
the table and listed in the report (STILL_DROPPED explains the ones known).

Run:
  python3 tools/gen-star-catalog/regen_western_stars.py \\
      --bsc /path/to/catalog --cns3 /path/to/catalog.dat --harris /path/to/catalog
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

CNS3_SHA256 = "d12d64aba1f3a0d53cc0a1332f421dc2499675ecbaae566ea339477b9cec9d25"
HARRIS_SHA256 = "ebd1f2a4bfbe0da03ec9c026ae846ed5e39178a9daa25d8c34fca6caf323d71b"

# Entries that name a Bright Star Catalogue star by something other than the
# Bayer/Flamsteed name the generic matcher reads. Key: entry name. Value: (HR
# number, why). Nothing here comes from the old table except the entry name.
HR_OVERRIDES = {
    # The catalogue's variable-star column (bytes 52-60) carries these Latin-letter
    # designations; the designation, not the position, identifies the star.
    "Omicron Velorum": (3447, "BSC variable-star column 'o Vel' (HD 74195), V 3.62"),
    "N Velorum": (3803, "BSC variable-star column 'N Vel' (HD 82668), V 3.13"),
    "PP Carinae": (4140, "BSC variable-star column 'PP Car' (HD 91465), V 3.32"),
    "L2 Puppis": (2748, "BSC variable-star column 'L2 Pup' (HD 56096), V 5.10 (semi-regular variable)"),
    # Second components of stars the catalogue lists as ONE record. The entry is
    # kept and points at that record (so it carries the combined magnitude).
    "Alnasl2": (6859, "component B of delta Sgr; the catalogue has one record, HR 6859 (same as Kaus Media)"),
    "Epsilon1 Tauri": (1409, "component B of epsilon Tau; the catalogue has one record, HR 1409 (same as Ain)"),
    "Algol B": (936, "component B of beta Per; the catalogue has one record, HR 936 (same as Algol)"),
}

# Objects that are not in the Bright Star Catalogue. Key: entry name. Value:
# (source, record key, why).
#   cns3   : Gliese & Jahreiss, Preliminary Third Catalogue of Nearby Stars (CNS3),
#            CDS V/70A, 1991 (before Hipparcos). Positions are B1950 at epoch
#            1950, given to 1 s of RA and 0.1 arcmin of Dec; the total proper
#            motion and position angle are B1950 too. Converted to J2000 here.
#   harris : Harris, Catalog of Parameters for Milky Way Globular Clusters,
#            AJ 112, 1487 (1996), CDS VII/195: J2000 position, integrated V.
# IAU_EPOCH: epoch of the IAU-CSN position when the entry has an IAU-CSN row.
EXTERNAL_SOURCES = {
    "Barnard's Star": ("cns3", "Gl 699", "IAU-CSN row GJ 699"),
    "Proxima Centauri": ("cns3", "Gl 551", "IAU-CSN row GJ 551"),
    "Wolf 359": ("cns3", "Gl 406", "no IAU-CSN row; CNS3 position, about 10 arcsec resolution"),
    "Omega Centauri": ("harris", "NGC 5139", "globular cluster NGC 5139; zero proper motion"),
}
# The IAU-CSN position of Barnard's Star is not for J2000.0: it is 90 arcsec
# from the J2000.0 place, which is the proper motion of about 8.75 years, i.e.
# the epoch of the Hipparcos catalogue (1991.25). The CNS3 place agrees with the
# IAU-CSN place at that epoch within the CNS3 resolution (checked at run time).
# Proxima Centauri's IAU-CSN position is for J2000.0 (same check).
IAU_EPOCH = {"Barnard's Star": 1991.25, "Proxima Centauri": 2000.0}

# Entries that are dropped because no star can be identified from the allowed
# sources. Key: entry name. Value: why.
STILL_DROPPED = {
    "A Velorum": "Lacaille letter designation; the Bright Star Catalogue lists no 'A Vel' (name or variable-star column) and the old position and magnitude match no catalogue star",
    "Q Velorum": "Lacaille letter designation; the Bright Star Catalogue lists no 'Q Vel' (name or variable-star column) and the old position and magnitude match no catalogue star",
    "Zeta Puppis2": "designation 'Gamma Puppis' does not exist (no gamma Pup in the Bright Star Catalogue); Zeta Puppis (Naos) is a separate entry and no catalogue star matches the old position and magnitude",
}

# B1950 (FK4) -> J2000 (FK5) rotation of the equatorial frame (Aoki et al. 1983;
# the matrix of the Explanatory Supplement, section 3.591). The E-terms of
# aberration (up to 0.34 arcsec) and the proper-motion-dependent terms of the full
# 6x6 matrix are not applied; both are below the CNS3 position resolution.
B1950_TO_J2000 = (
    (0.9999256794956877, -0.0111814832204662, -0.0048590038153592),
    (0.0111814832391717, 0.9999374848933135, -0.0000271625947142),
    (0.0048590037723143, -0.0000271702937440, 0.9999881946023742),
)


def _mat_vec(m, v):
    return tuple(sum(m[i][j] * v[j] for j in range(3)) for i in range(3))


def _tangent_basis(ra_rad, dec_rad):
    e_a = (-math.sin(ra_rad), math.cos(ra_rad), 0.0)
    e_d = (
        -math.sin(dec_rad) * math.cos(ra_rad),
        -math.sin(dec_rad) * math.sin(ra_rad),
        math.cos(dec_rad),
    )
    return e_a, e_d


def parse_cns3(path):
    """CNS3 records keyed by the stripped Name field (bytes 1-8). Position
    B1950 epoch 1950 (degrees), total proper motion (arcsec/yr), position angle
    (deg, north through east), V magnitude."""
    recs = {}
    for line in open(path, encoding="latin-1"):
        if len(line) < 75 or not line[12:14].strip():
            continue
        name = line[0:8].strip()
        comp = line[8:10].strip()
        try:
            ra = (int(line[12:14]) + int(line[15:17]) / 60.0 + int(line[18:20]) / 3600.0) * 15.0
            sign = -1.0 if line[21] == "-" else 1.0
            dec = sign * (int(line[22:24]) + float(line[25:29]) / 60.0)
            pm = float(line[30:36])
            pa = float(line[37:42])
            vmag = float(line[67:73])
        except ValueError:
            continue
        if not comp:
            recs[name] = dict(ra1950=ra, dec1950=dec, pm=pm, pa=pa, vmag=vmag)
    return recs


def cns3_to_j2000(rec, epoch):
    """J2000 equatorial place of a CNS3 record at `epoch` (decimal year) and the
    J2000 proper motion (mas/yr, pmRA includes cos dec)."""
    a, d = math.radians(rec["ra1950"]), math.radians(rec["dec1950"])
    r = (math.cos(d) * math.cos(a), math.cos(d) * math.sin(a), math.sin(d))
    e_a, e_d = _tangent_basis(a, d)
    mu = rec["pm"] * math.pi / (180.0 * 3600.0)  # rad/yr
    pa = math.radians(rec["pa"])
    v = tuple(mu * (math.sin(pa) * e_a[i] + math.cos(pa) * e_d[i]) for i in range(3))
    r2, v2 = _mat_vec(B1950_TO_J2000, r), _mat_vec(B1950_TO_J2000, v)
    dt = epoch - 1950.0
    p = tuple(r2[i] + v2[i] * dt for i in range(3))
    n = math.sqrt(sum(c * c for c in p))
    p = tuple(c / n for c in p)
    ra = math.degrees(math.atan2(p[1], p[0])) % 360.0
    dec = math.degrees(math.asin(p[2]))
    pa2, pd2 = _tangent_basis(math.radians(ra), math.radians(dec))
    to_mas = 180.0 * 3600.0 * 1000.0 / math.pi
    pmra = sum(v2[i] * pa2[i] for i in range(3)) * to_mas
    pmde = sum(v2[i] * pd2[i] for i in range(3)) * to_mas
    return ra, dec, pmra, pmde


def parse_harris(path):
    """Harris (1996) rows keyed by the ID field (bytes 2-10): J2000 position
    (degrees) and the integrated V magnitude Vt."""
    recs = {}
    for line in open(path, encoding="latin-1"):
        if len(line) < 115:
            continue
        key = " ".join(line[1:10].split())
        try:
            ra = (int(line[23:25]) + int(line[26:28]) / 60.0 + float(line[29:33]) / 3600.0) * 15.0
            sign = -1.0 if line[35] == "-" else 1.0
            dec = sign * (int(line[36:38]) + int(line[39:41]) / 60.0 + int(line[42:44]) / 3600.0)
            vt = float(line[109:114])
        except ValueError:
            continue
        recs[key] = dict(ra=ra, dec=dec, vmag=vt)
    return recs


def norm_desig(s):
    """'Alpha1 Centauri' / 'Alpha Centauri' -> 'alpha centauri' (drop the digit
    that tells Greek-letter components apart)."""
    return re.sub(r"^([A-Za-z]+)\d\b", r"\1", s).lower()


def main():
    root = os.path.abspath(os.path.join(here, "..", ".."))
    ap = argparse.ArgumentParser()
    ap.add_argument("--bsc", required=True)
    ap.add_argument("--cns3", required=True, help="CDS V/70A catalog.dat")
    ap.add_argument("--harris", required=True, help="CDS VII/195 catalog")
    ap.add_argument("--iau", default=os.path.join(here, "IAU-CSN.txt"))
    ap.add_argument("--rs", default=os.path.join(root, "crates/xalen-western/src/stars.rs"))
    ap.add_argument("--report", action="store_true")
    ap.add_argument("--skip-sha", action="store_true")
    args = ap.parse_args()

    import hashlib
    digest = hashlib.sha256(open(args.bsc, "rb").read()).hexdigest()
    if digest != gb.BSC_SHA256 and not args.skip_sha:
        sys.exit(f"BSC catalog sha256 {digest} != expected {gb.BSC_SHA256}")

    for label, path, want in (("CNS3", args.cns3, CNS3_SHA256), ("Harris", args.harris, HARRIS_SHA256)):
        got = hashlib.sha256(open(path, "rb").read()).hexdigest()
        if got != want and not args.skip_sha:
            sys.exit(f"{label} sha256 {got} != expected {want}")
    cns3 = parse_cns3(args.cns3)
    harris = parse_harris(args.harris)

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
    log = {"iau-name": 0, "designation": 0, "nearest": 0, "override": 0}
    externals = []
    overrides = []
    n_ext = 0
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

        if name in STILL_DROPPED:
            dropped.append((name, desig, STILL_DROPPED[name]))
            out.append((m, None))
            continue

        if name in EXTERNAL_SOURCES:
            src_kind, key, why = EXTERNAL_SOURCES[name]
            if src_kind == "cns3":
                rec = cns3[key]
                ra, dec, pmra, pmde = cns3_to_j2000(rec, 2000.0)
                note = f"CNS3 {key}: place from CNS3 alone"
                hit = iau_by_name.get(name.lower())
                if hit is not None:
                    row = hit[0]
                    ep = IAU_EPOCH[name]
                    # advance the IAU-CSN place from its epoch to 2000.0 with the
                    # CNS3 J2000 proper motion, and check it against CNS3.
                    cosd = math.cos(math.radians(row["dec"]))
                    ira = row["ra"] + (pmra / 3.6e6) / cosd * (2000.0 - ep)
                    ide = row["dec"] + (pmde / 3.6e6) * (2000.0 - ep)
                    check = gb.sep_arcsec(ra, dec, ira, ide)
                    wrong = gb.sep_arcsec(ra, dec, row["ra"], row["dec"])
                    print(f"   external {name}: IAU-CSN place (epoch {ep}) agrees with CNS3 within {check:.1f} arcsec; "
                          f"IAU-CSN place taken as epoch 2000.0 would be {wrong:.1f} arcsec from CNS3")
                    if check > 30.0:
                        sys.exit(f"{name}: IAU-CSN and CNS3 disagree by {check:.1f} arcsec")
                    ra, dec = ira, ide
                    note = f"IAU-CSN place (epoch {ep}) advanced to 2000.0 with the CNS3 proper motion"
                mag = rec["vmag"]
            else:
                rec = harris[key]
                ra, dec, mag, pmra, pmde = rec["ra"], rec["dec"], rec["vmag"], 0.0, 0.0
                note = f"Harris {key}"
            lon, lat = gb.eq_to_ecl(ra, dec)
            log["external"] = log.get("external", 0) + 1
            externals.append((name, src_kind, key, note))
            out.append((m, dict(
                ra=ra, dec=dec, mag=mag, pmra=pmra, pmde=pmde, lon=lon, lat=lat,
                hr=0, how="external",
            )))
            n_ext += 1
            continue

        star = None
        how = None
        if name in HR_OVERRIDES:
            hr_over, why = HR_OVERRIDES[name]
            star, how = by_hr[hr_over], "override"
            overrides.append((name, hr_over, why))
            hit = None
        else:
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
        cands = [] if how == "override" else desig_index.get(norm_desig(desig), [])
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
        if star is None and how != "override":
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
    print(f"sourced outside the BSC (CNS3 / Harris): {n_ext}")
    for e in externals:
        print(f"   {e[0]!r:22} {e[1]:7} {e[2]:9} {e[3]}")
    print(f"pointed at a BSC record by HR_OVERRIDES: {len(overrides)}")
    for o in overrides:
        print(f"   {o[0]!r:20} HR {o[1]:5} {o[2]}")
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
