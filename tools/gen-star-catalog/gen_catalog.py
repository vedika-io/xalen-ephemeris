#!/usr/bin/env python3
"""Generate crates/xalen-stars/src/catalog_generated.rs from the real
Hipparcos Main Catalogue (CDS I/239, hip_main.dat, 118,218 records).

EVERY emitted star comes from a record in hip_main.dat. No coordinate, magnitude
or proper-motion value is invented or interpolated. The only data NOT from the
HIP record is the *traditional name*, which is carried over by a positional
crossmatch against the curated catalog already shipped in lib.rs.

Pipeline (per LEVER spec):
  1. Parse HIP, RAdeg, DEdeg, Vmag, pmRA(=mu_alpha*cos delta), pmDE from the
     fixed-width records (byte positions from the official ReadMe).
  2. Filter Vmag <= 6.5.
  3. Propagate J1991.25 -> J2000.0 (8.75 yr) by applying pmRA/pmDE.
  4. Convert RA/Dec -> ecliptic at the J2000 mean obliquity (IAU 2006).
  5. Derive ecliptic-longitude/latitude proper motion (mas/yr) by the analytic
     Jacobian of the equatorial->ecliptic rotation, so the Rust struct's
     pm_lon/pm_lat fields stay populated from real measured pmRA/pmDE.
  6. LEFT-JOIN the curated names by nearest J2000 ecliptic position
     (orb <= 0.02 deg AND magnitude within 0.6 mag) — a real crossmatch, never
     an invented HIP<->name table.
  7. Emit catalog_generated.rs.

Run:
  python3 tools/gen-star-catalog/gen_catalog.py \
      --hip /tmp/hip_main.dat \
      --curated /tmp/curated_stars.json \
      --out crates/xalen-stars/src/catalog_generated.rs
"""
import argparse
import json
import math

# --- Constants -------------------------------------------------------------
# IAU 2006 mean obliquity at J2000.0 = 84381.406" = 23.4392794444... deg.
# Source: xalen-coords/src/obliquity.rs OBLIQUITY_J2000_ARCSEC (84381.406).
OBLIQUITY_J2000_DEG = 84381.406 / 3600.0
EPS = math.radians(OBLIQUITY_J2000_DEG)

# Hipparcos catalogue epoch is J1991.25; J2000.0 is 8.75 years later.
DT_YEARS = 2000.0 - 1991.25  # = 8.75

VMAG_LIMIT = 6.5

# Traditional-name join is done through the IAU Catalog of Star Names (IAU-CSN),
# the official authority of the IAU Working Group on Star Names. It gives a real
# proper-name -> HIP designation, so the curated name is carried onto the exact
# HIP record by HIP number — NOT by an invented table and NOT by an unreliable
# positional crossmatch against the coarse curated coordinates. The IAU-CSN also
# supplies J2000 RA/Dec, which we use only as a sanity check that the named star
# and the HIP record are the same body (logged, not a hard fabrication gate).
IAU_POS_SANITY_DEG = 0.1   # IAU RA/Dec vs HIP-derived position must agree this well

# Curated-spelling -> IAU-CSN-spelling alias map.
#
# A handful of curated catalog names use a traditional spelling that differs
# from the official IAU-WGSN ASCII name in IAU-CSN.txt, so the exact-name lookup
# `iau.get(curated_name)` misses and the star silently falls back to the coarse
# curated coordinates. Each alias below resolves the curated spelling to the
# authoritative IAU-CSN entry; the HIP designation (and therefore every emitted
# coordinate) still comes entirely from IAU-CSN -> Hipparcos, never invented.
#
#   "Zuben Elgenubi" (Bayer alpha2 Lib)  -> IAU "Zubenelgenubi"   (HIP 72622)
#   "Zuben Eschamali" (Bayer beta Lib)   -> IAU "Zubeneschamali"  (HIP 74785)
#   "Bharani 41" (Flamsteed 41 Ari)      -> IAU "Bharani"         (HIP 13209)
#   "Lambda Orionis" (Bayer lambda Ori)  -> IAU "Meissa"          (HIP 26207)
#   "Hyadum II" (Bayer delta1 Tau)       -> IAU "Secunda Hyadum"  (HIP 20455)
CURATED_TO_IAU = {
    "Zuben Elgenubi": "Zubenelgenubi",
    "Zuben Eschamali": "Zubeneschamali",
    "Bharani 41": "Bharani",
    "Lambda Orionis": "Meissa",
    "Hyadum II": "Secunda Hyadum",
}

# Curated-spelling -> explicit HIP designation, for traditional star names that
# have NO IAU-WGSN proper name at all (so they cannot be resolved through
# IAU-CSN). These bodies are still real Hipparcos stars; the HIP number is the
# documented Bright Star / Bayer-Flamsteed identification, and every coordinate
# still comes from the Hipparcos record for that HIP — only the *name* is the
# traditional one.
#
#   "Al Jabhah" (Bayer eta Leonis, HR 4031 = HD 87737) -> HIP 49583.
#     WGSN has not assigned eta Leonis a proper name (it named the nearby
#     zeta Leonis "Adhafera" and gamma1 Leonis "Algieba"; eta Leonis is a
#     distinct, unnamed star), so this is an explicit HIP designation, not an
#     IAU-CSN alias.
CURATED_TO_HIP = {
    "Al Jabhah": 49583,
}


def field(line, a, b):
    """1-indexed inclusive byte slice, stripped."""
    return line[a - 1:b].strip()


def parse_hip(path):
    """Yield dicts for every HIP record that has Vmag, RAdeg, DEdeg present."""
    out = []
    total = 0
    with open(path, encoding="latin-1") as f:
        for line in f:
            total += 1
            hip = field(line, 9, 14)
            vmag = field(line, 42, 46)
            radeg = field(line, 52, 63)
            dedeg = field(line, 65, 76)
            pmra = field(line, 88, 95)
            pmde = field(line, 97, 104)
            if not (hip and vmag and radeg and dedeg):
                continue
            try:
                rec = dict(
                    hip=int(hip),
                    vmag=float(vmag),
                    radeg=float(radeg),
                    dedeg=float(dedeg),
                    # pmRA is mu_alpha * cos(delta) per ReadMe (H12); pmDE is mu_delta.
                    pmra=float(pmra) if pmra else 0.0,
                    pmde=float(pmde) if pmde else 0.0,
                )
            except ValueError:
                continue
            out.append(rec)
    return out, total


def propagate_to_j2000(rec):
    """Apply 8.75 yr of proper motion to move J1991.25 -> J2000.0.

    pmRA = mu_alpha * cos(delta) in mas/yr, so the change in alpha itself is
    pmRA / cos(delta).  Working in degrees:
        d_alpha = (pmRA / cos(delta)) * DT / 3.6e6
        d_delta = pmDE * DT / 3.6e6
    """
    dec_rad = math.radians(rec["dedeg"])
    cos_dec = math.cos(dec_rad)
    # mas -> deg : /3_600_000
    d_alpha_deg = (rec["pmra"] / cos_dec) * DT_YEARS / 3_600_000.0 if abs(cos_dec) > 1e-9 else 0.0
    d_delta_deg = rec["pmde"] * DT_YEARS / 3_600_000.0
    ra2000 = (rec["radeg"] + d_alpha_deg) % 360.0
    de2000 = rec["dedeg"] + d_delta_deg
    return ra2000, de2000


def eq_to_ecl(ra_deg, dec_deg):
    """Equatorial (RA/Dec, deg) -> ecliptic (lon/lat, deg) at J2000 obliquity.

    Matches xalen-coords::equatorial_to_ecliptic exactly:
      lon = atan2(sin a cos e + tan d sin e, cos a)
      lat = asin(sin d cos e - cos d sin e sin a)
    """
    a = math.radians(ra_deg)
    d = math.radians(dec_deg)
    sin_e, cos_e = math.sin(EPS), math.cos(EPS)
    sin_a, cos_a = math.sin(a), math.cos(a)
    sin_d, cos_d = math.sin(d), math.cos(d)
    lon = math.atan2(sin_a * cos_e + (sin_d / cos_d) * sin_e, cos_a)
    lat = math.asin(sin_d * cos_e - cos_d * sin_e * sin_a)
    return math.degrees(lon) % 360.0, math.degrees(lat)


def ecliptic_pm(ra_deg, dec_deg, pmra_mas, pmde_mas):
    """Convert equatorial proper motion (mas/yr) to ecliptic lon/lat PM (mas/yr).

    pmra_mas is mu_alpha*cos(delta).  Build the equatorial velocity unit-sphere
    tangent vector, then rotate by -EPS about the x-axis (the standard
    equatorial->ecliptic rotation) and read off the ecliptic tangential rates.
    pm_lon returned is mu_lambda * cos(beta) converted back to plain mu_lambda
    so the Rust model (which multiplies pm_lon * dt directly into longitude)
    receives mu_lambda, consistent with the curated catalog's convention.
    """
    a = math.radians(ra_deg)
    d = math.radians(dec_deg)
    # tangential rates in rad/yr on the sky (small)
    mas_to_rad = math.pi / (180.0 * 3600.0 * 1000.0)
    mua_cosd = pmra_mas * mas_to_rad   # mu_alpha * cos(delta)
    mud = pmde_mas * mas_to_rad        # mu_delta
    # Equatorial tangent basis: e_alpha (east), e_delta (north).
    # velocity vector v = mua_cosd * e_alpha + mud * e_delta  (rad/yr, on unit sphere)
    e_alpha = (-math.sin(a), math.cos(a), 0.0)
    e_delta = (-math.cos(a) * math.sin(d), -math.sin(a) * math.sin(d), math.cos(d))
    v = tuple(mua_cosd * e_alpha[i] + mud * e_delta[i] for i in range(3))
    # Rotate vector from equatorial to ecliptic frame: R_x(+EPS) maps eq->ecl.
    # (ecl = R_x(eps) * eq, standard for J2000 with eps>0)
    ce, se = math.cos(EPS), math.sin(EPS)
    vx = v[0]
    vy = ce * v[1] + se * v[2]
    vz = -se * v[1] + ce * v[2]
    # ecliptic position
    lam_deg, beta_deg = eq_to_ecl(ra_deg, dec_deg)
    lam = math.radians(lam_deg)
    beta = math.radians(beta_deg)
    # ecliptic tangent basis
    el_lon = (-math.sin(lam), math.cos(lam), 0.0)              # east (lambda)
    el_lat = (-math.cos(lam) * math.sin(beta), -math.sin(lam) * math.sin(beta), math.cos(beta))
    mu_lon_cosb = vx * el_lon[0] + vy * el_lon[1] + vz * el_lon[2]   # mu_lambda*cos(beta)
    mu_lat = vx * el_lat[0] + vy * el_lat[1] + vz * el_lat[2]         # mu_beta
    cos_b = math.cos(beta)
    mu_lon = mu_lon_cosb / cos_b if abs(cos_b) > 1e-9 else 0.0
    rad_to_mas = 1.0 / mas_to_rad
    return mu_lon * rad_to_mas, mu_lat * rad_to_mas


def parse_iau_csn(path):
    """Parse the IAU-CSN authority file -> {ascii_name: (hip|None, ra_deg, dec_deg)}.

    The data rows are whitespace-delimited; HIP, HD, RA(J2000), Dec(J2000) form
    the trailing numeric block immediately before the YYYY-MM-DD approval date.
    """
    import re
    out = {}
    with open(path, encoding="latin-1") as f:
        for line in f:
            if line.startswith("#") or line.startswith("$") or not line.strip():
                continue
            # Name/ASCII is a fixed-width field (header column 1-18). Multi-word
            # proper names such as "Polaris Australis" / "Deneb Algedi" live
            # ENTIRELY in this column, so a naive whitespace split would clip
            # them to "Polaris"/"Deneb" and collide with the real Polaris/Deneb.
            name = line[0:18].strip()
            if not name:
                continue
            # The trailing numeric block (HIP HD RA Dec) precedes the YYYY-MM-DD
            # approval date; split the remainder of the line on whitespace.
            rest = line[18:].split()
            didx = None
            for i, t in enumerate(rest):
                if re.match(r"^\d{4}-\d{2}-\d{2}$", t):
                    didx = i
                    break
            if didx is None or didx < 4:
                continue
            ra, dec, hip = rest[didx - 2], rest[didx - 1], rest[didx - 4]
            try:
                ra = float(ra)
                dec = float(dec)
            except ValueError:
                continue
            # First occurrence wins; never let a later multi-word name overwrite
            # a single-word entry (both are unique ASCII names anyway).
            out.setdefault(name, (int(hip) if hip.isdigit() else None, ra, dec))
    return out


def ang_sep_deg(lon1, lat1, lon2, lat2):
    """Great-circle separation (deg) between two ecliptic positions."""
    p1, p2 = math.radians(lat1), math.radians(lat2)
    dl = math.radians(lon1 - lon2)
    a = (math.sin((p1 - p2) / 2) ** 2
         + math.cos(p1) * math.cos(p2) * math.sin(dl / 2) ** 2)
    return math.degrees(2 * math.asin(min(1.0, math.sqrt(a))))


def main():
    ap = argparse.ArgumentParser()
    import os
    here = os.path.dirname(os.path.abspath(__file__))
    ap.add_argument("--hip", default="/tmp/hip_main.dat")
    ap.add_argument("--curated", default="/tmp/curated_stars.json")
    ap.add_argument("--iau", default=os.path.join(here, "IAU-CSN.txt"))
    ap.add_argument("--out", default="crates/xalen-stars/src/catalog_generated.rs")
    args = ap.parse_args()

    records, total = parse_hip(args.hip)
    bright = [r for r in records if r["vmag"] <= VMAG_LIMIT]
    bright.sort(key=lambda r: r["hip"])

    stars = []
    star_by_hip = {}
    for r in bright:
        ra2000, de2000 = propagate_to_j2000(r)
        lon, lat = eq_to_ecl(ra2000, de2000)
        pm_lon, pm_lat = ecliptic_pm(ra2000, de2000, r["pmra"], r["pmde"])
        s = dict(
            hip=r["hip"], lon=lon, lat=lat, mag=r["vmag"],
            pm_lon=pm_lon, pm_lat=pm_lat,
            ra2000=ra2000, de2000=de2000,
        )
        stars.append(s)
        star_by_hip[r["hip"]] = s

    # LEFT-JOIN curated traditional names via the IAU-CSN authority:
    #   curated name --(IAU-CSN)--> HIP --(this catalog)--> generated star.
    curated = json.load(open(args.curated))
    iau = parse_iau_csn(args.iau)
    name_by_hip = {}
    used_names = set()
    matched = 0
    aliased = 0
    sanity_warnings = []
    for c in curated:
        # 1. Direct IAU-CSN lookup, then 2. curated->IAU alias, both of which
        #    still resolve a real IAU-CSN (proper-name -> HIP) row.
        rec = iau.get(c["name"]) or iau.get(CURATED_TO_IAU.get(c["name"], ""))
        if rec:
            hip, ira, idec = rec
            if hip is None or hip not in star_by_hip:
                continue
            s = star_by_hip[hip]
            # Sanity: the IAU RA/Dec and the HIP record must be the same star.
            # Compare in equatorial space at J2000 (cos-dec weighted).
            dra = (s["ra2000"] - ira + 180.0) % 360.0 - 180.0
            ddec = s["de2000"] - idec
            sep = math.hypot(dra * math.cos(math.radians(idec)), ddec)
            if sep > IAU_POS_SANITY_DEG:
                sanity_warnings.append(f"{c['name']} HIP{hip} IAU-vs-HIP sep={sep:.3f} deg")
                continue
            if hip not in name_by_hip:
                name_by_hip[hip] = (c["name"], c["constellation"], c["nature"])
                used_names.add(c["name"])
                matched += 1
                if c["name"] in CURATED_TO_IAU:
                    aliased += 1
            continue

        # 3. No IAU-WGSN name exists for this body — use the documented explicit
        #    HIP designation (still a real Hipparcos record; only the traditional
        #    name is carried over, no coordinate is invented).
        hip = CURATED_TO_HIP.get(c["name"])
        if hip is not None and hip in star_by_hip and hip not in name_by_hip:
            name_by_hip[hip] = (c["name"], c["constellation"], c["nature"])
            used_names.add(c["name"])
            matched += 1
            aliased += 1

    unmatched = [c["name"] for c in curated if c["name"] not in used_names]

    # Emit Rust.
    lines = []
    lines.append("// @generated by tools/gen-star-catalog/gen_catalog.py — DO NOT EDIT BY HAND.")
    lines.append("//")
    lines.append("// Some HIP-derived magnitudes/coordinates (e.g. 6.28, 3.14) numerically")
    lines.append("// resemble TAU/PI but are real catalog data, not math constants.")
    lines.append("#![allow(clippy::approx_constant)]")
    lines.append("//")
    lines.append(f"// Source: Hipparcos Main Catalogue (CDS I/239, hip_main.dat, {total} records).")
    lines.append(f"// Filter: Vmag <= {VMAG_LIMIT}  ->  {len(stars)} stars.")
    lines.append("// Epoch: positions propagated J1991.25 -> J2000.0 (8.75 yr) using pmRA/pmDE,")
    lines.append(f"//        then RA/Dec -> ecliptic at the IAU 2006 J2000 mean obliquity ({OBLIQUITY_J2000_DEG:.10f} deg).")
    lines.append(f"// Curated traditional names joined to HIP: {matched} matched ({aliased} via")
    lines.append("// curated-spelling alias / explicit HIP designation; the rest by exact IAU-CSN name).")
    lines.append("//")
    lines.append("// Every coordinate, magnitude, and proper-motion value below is derived from a")
    lines.append("// real hip_main.dat record. No value is invented or interpolated.")
    lines.append("//")
    lines.append("// Hipparcos data: (C) ESA 1997, from the Hipparcos and Tycho Catalogues.")
    lines.append("// See CREDITS.md / NOTICE for attribution.")
    lines.append("")
    lines.append("use crate::GeneratedStar;")
    lines.append("")
    lines.append(f"/// Number of HIP stars with Vmag <= {VMAG_LIMIT} in this generated catalog.")
    lines.append(f"pub const GENERATED_STAR_COUNT: usize = {len(stars)};")
    lines.append("")
    lines.append("/// Fixed stars generated from the Hipparcos Main Catalogue (Vmag <= 6.5).")
    lines.append("///")
    lines.append("/// Sorted by HIP number. `name` is `Some(..)` only where a curated")
    lines.append("/// traditional name joined to this HIP via the IAU-CSN authority.")
    lines.append("pub static GENERATED_CATALOG: &[GeneratedStar] = &[")
    for s in stars:
        nm = name_by_hip.get(s["hip"])
        if nm:
            name_lit = f'Some("{nm[0]}")'
            const_lit = f'"{nm[1]}"'
            nature_lit = f'"{nm[2]}"'
        else:
            name_lit = "None"
            const_lit = '""'
            nature_lit = '""'
        lines.append(
            "    GeneratedStar { "
            f"hip: {s['hip']}, name: {name_lit}, constellation: {const_lit}, "
            f"longitude_j2000: {s['lon']:.6f}, latitude_j2000: {s['lat']:.6f}, "
            f"magnitude: {s['mag']:.2f}, nature: {nature_lit}, "
            f"pm_lon_mas_per_year: {s['pm_lon']:.3f}, pm_lat_mas_per_year: {s['pm_lat']:.3f} "
            "},"
        )
    lines.append("];")
    lines.append("")

    with open(args.out, "w") as f:
        f.write("\n".join(lines))

    print(f"HIP records read         : {total}")
    print(f"records with Vmag/RA/Dec : {len(records)}")
    print(f"Vmag <= {VMAG_LIMIT} (emitted)    : {len(stars)}")
    print(f"curated names matched    : {matched} / {len(curated)} (via IAU-CSN authority)")
    print(f"   of which via alias/HIP : {aliased} (curated-spelling alias or explicit HIP)")
    if sanity_warnings:
        print(f"IAU<->HIP sanity skips   : {len(sanity_warnings)}")
        for w in sanity_warnings:
            print(f"    {w}")
    if unmatched:
        print(f"curated UNMATCHED ({len(unmatched)}): {', '.join(unmatched)}")
    print(f"wrote                    : {args.out}")


if __name__ == "__main__":
    main()
