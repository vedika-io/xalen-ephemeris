#!/usr/bin/env python3
"""Generate crates/xalen-stars/src/bright_generated.rs, the in-house fixed-star
list, from two sources that carry no non-commercial restriction:

  1. Yale Bright Star Catalogue, 5th rev. ed. (Hoffleit & Warren 1991,
     CDS V/50, file `catalog`, 9,110 fixed-width records, 9,096 of them stars).
     Used for: identification (HR, HD, Bayer/Flamsteed name), magnitude, proper
     motion, and the position of every star that has no IAU name.
  2. IAU Catalog of Star Names (IAU-CSN, tools/gen-star-catalog/IAU-CSN.txt,
     released by the IAU under Creative Commons Attribution).
     Used for: the proper name, the constellation and the higher precision
     J2000 position (6 decimals of a degree) of every named star that also
     appears in the Bright Star Catalogue (joined on HD number, then on
     the "HR nnn" designation).

No value is read from the Hipparcos, Tycho, Gaia, SIMBAD, HYG or Swiss
Ephemeris data. The only other input is the curated traditional-name table in
crates/xalen-stars/src/lib.rs (`CATALOG`), from which the traditional name,
constellation and Ptolemaic `nature` are carried over to the star it names.

Pipeline:
  * Parse every BSC record that has a J2000 position (9,096 of 9,110).
  * Replace the position by the IAU-CSN one where the star has an IAU row.
  * Convert RA/Dec to ecliptic longitude/latitude at the IAU 2006 J2000 mean
    obliquity (84381.406") and rotate the proper motion into ecliptic
    components in mas/yr (same math as gen_catalog.py).
  * Join curated names: curated name -> IAU-CSN row (directly or through the
    CURATED_TO_IAU spelling alias) -> HD / HR -> BSC star.
  * Emit the table sorted by HR.

Run:
  python3 tools/gen-star-catalog/gen_bright_catalog.py \
      --bsc /path/to/catalog \
      --out crates/xalen-stars/src/bright_generated.rs
"""
import argparse
import hashlib
import math
import os
import re
import sys

OBLIQUITY_J2000_DEG = 84381.406 / 3600.0
EPS = math.radians(OBLIQUITY_J2000_DEG)

BSC_SHA256 = "69797549cc1605aad7ff94e9325e29a1661f2a253917faaa056d9bf20b809afd"
BSC_URL = "https://vizier.cfa.harvard.edu/ftp/cats/V/50/catalog.gz"

# IAU-CSN position vs BSC position of the same star: anything farther apart
# than this is reported (an unresolved binary component or a bad join).
JOIN_SEP_REPORT_ARCSEC = 30.0
# A join is refused outright when the two positions disagree by more than this.
JOIN_SEP_REFUSE_ARCSEC = 120.0

# Curated spelling -> IAU-CSN spelling, for the curated names whose traditional
# spelling differs from the IAU-WGSN ASCII name.
CURATED_TO_IAU = {
    "Zuben Elgenubi": "Zubenelgenubi",
    "Zuben Eschamali": "Zubeneschamali",
    "Bharani 41": "Bharani",
    "Lambda Orionis": "Meissa",
    "Hyadum II": "Secunda Hyadum",
}

# Curated names that have no IAU-WGSN proper name at all. They are joined to
# the Bright Star Catalogue by their documented Bayer designation
# (BSC `Name` field, "Gamma Con" style: Greek abbreviation + constellation).
#   Al Jabhah = eta Leonis (17 Leo), BSC Name "17Eta Leo".
CURATED_TO_BSC_BAYER = {
    "Al Jabhah": ("Eta", "Leo"),
}

GREEK = {
    "Alp": "Alpha", "Bet": "Beta", "Gam": "Gamma", "Del": "Delta",
    "Eps": "Epsilon", "Zet": "Zeta", "Eta": "Eta", "The": "Theta",
    "Iot": "Iota", "Kap": "Kappa", "Lam": "Lambda", "Mu": "Mu", "Nu": "Nu",
    "Xi": "Xi", "Omi": "Omicron", "Pi": "Pi", "Rho": "Rho", "Sig": "Sigma",
    "Tau": "Tau", "Ups": "Upsilon", "Phi": "Phi", "Chi": "Chi", "Psi": "Psi",
    "Ome": "Omega",
}

# abbreviation: (nominative name, genitive name)
CONSTELLATIONS = {
    "And": ("Andromeda", "Andromedae"), "Ant": ("Antlia", "Antliae"),
    "Aps": ("Apus", "Apodis"), "Aqr": ("Aquarius", "Aquarii"),
    "Aql": ("Aquila", "Aquilae"), "Ara": ("Ara", "Arae"),
    "Ari": ("Aries", "Arietis"), "Aur": ("Auriga", "Aurigae"),
    "Boo": ("Bootes", "Bootis"), "Cae": ("Caelum", "Caeli"),
    "Cam": ("Camelopardalis", "Camelopardalis"), "Cnc": ("Cancer", "Cancri"),
    "CVn": ("Canes Venatici", "Canum Venaticorum"),
    "CMa": ("Canis Major", "Canis Majoris"),
    "CMi": ("Canis Minor", "Canis Minoris"),
    "Cap": ("Capricornus", "Capricorni"), "Car": ("Carina", "Carinae"),
    "Cas": ("Cassiopeia", "Cassiopeiae"), "Cen": ("Centaurus", "Centauri"),
    "Cep": ("Cepheus", "Cephei"), "Cet": ("Cetus", "Ceti"),
    "Cha": ("Chamaeleon", "Chamaeleontis"), "Cir": ("Circinus", "Circini"),
    "Col": ("Columba", "Columbae"), "Com": ("Coma Berenices", "Comae Berenices"),
    "CrA": ("Corona Australis", "Coronae Australis"),
    "CrB": ("Corona Borealis", "Coronae Borealis"),
    "Crv": ("Corvus", "Corvi"), "Crt": ("Crater", "Crateris"),
    "Cru": ("Crux", "Crucis"), "Cyg": ("Cygnus", "Cygni"),
    "Del": ("Delphinus", "Delphini"), "Dor": ("Dorado", "Doradus"),
    "Dra": ("Draco", "Draconis"), "Equ": ("Equuleus", "Equulei"),
    "Eri": ("Eridanus", "Eridani"), "For": ("Fornax", "Fornacis"),
    "Gem": ("Gemini", "Geminorum"), "Gru": ("Grus", "Gruis"),
    "Her": ("Hercules", "Herculis"), "Hor": ("Horologium", "Horologii"),
    "Hya": ("Hydra", "Hydrae"), "Hyi": ("Hydrus", "Hydri"),
    "Ind": ("Indus", "Indi"), "Lac": ("Lacerta", "Lacertae"),
    "Leo": ("Leo", "Leonis"), "LMi": ("Leo Minor", "Leonis Minoris"),
    "Lep": ("Lepus", "Leporis"), "Lib": ("Libra", "Librae"),
    "Lup": ("Lupus", "Lupi"), "Lyn": ("Lynx", "Lyncis"),
    "Lyr": ("Lyra", "Lyrae"), "Men": ("Mensa", "Mensae"),
    "Mic": ("Microscopium", "Microscopii"), "Mon": ("Monoceros", "Monocerotis"),
    "Mus": ("Musca", "Muscae"), "Nor": ("Norma", "Normae"),
    "Oct": ("Octans", "Octantis"), "Oph": ("Ophiuchus", "Ophiuchi"),
    "Ori": ("Orion", "Orionis"), "Pav": ("Pavo", "Pavonis"),
    "Peg": ("Pegasus", "Pegasi"), "Per": ("Perseus", "Persei"),
    "Phe": ("Phoenix", "Phoenicis"), "Pic": ("Pictor", "Pictoris"),
    "Psc": ("Pisces", "Piscium"), "PsA": ("Piscis Austrinus", "Piscis Austrini"),
    "Pup": ("Puppis", "Puppis"), "Pyx": ("Pyxis", "Pyxidis"),
    "Ret": ("Reticulum", "Reticuli"), "Sge": ("Sagitta", "Sagittae"),
    "Sgr": ("Sagittarius", "Sagittarii"), "Sco": ("Scorpius", "Scorpii"),
    "Scl": ("Sculptor", "Sculptoris"), "Sct": ("Scutum", "Scuti"),
    "Ser": ("Serpens", "Serpentis"), "Sex": ("Sextans", "Sextantis"),
    "Tau": ("Taurus", "Tauri"), "Tel": ("Telescopium", "Telescopii"),
    "Tri": ("Triangulum", "Trianguli"),
    "TrA": ("Triangulum Australe", "Trianguli Australis"),
    "Tuc": ("Tucana", "Tucanae"), "UMa": ("Ursa Major", "Ursae Majoris"),
    "UMi": ("Ursa Minor", "Ursae Minoris"), "Vel": ("Vela", "Velorum"),
    "Vir": ("Virgo", "Virginis"), "Vol": ("Volans", "Volantis"),
    "Vul": ("Vulpecula", "Vulpeculae"),
}


# --- math (same as gen_catalog.py) ------------------------------------------
def eq_to_ecl(ra_deg, dec_deg):
    a = math.radians(ra_deg)
    d = math.radians(dec_deg)
    sin_e, cos_e = math.sin(EPS), math.cos(EPS)
    sin_a, cos_a = math.sin(a), math.cos(a)
    sin_d, cos_d = math.sin(d), math.cos(d)
    lon = math.atan2(sin_a * cos_e + (sin_d / cos_d) * sin_e, cos_a)
    lat = math.asin(sin_d * cos_e - cos_d * sin_e * sin_a)
    return math.degrees(lon) % 360.0, math.degrees(lat)


def ecliptic_pm(ra_deg, dec_deg, pmra_mas, pmde_mas):
    """Equatorial proper motion (pmRA includes cos dec, mas/yr) -> ecliptic
    (mu_lambda, mu_beta) in mas/yr; mu_lambda is the plain rate of longitude."""
    a = math.radians(ra_deg)
    d = math.radians(dec_deg)
    mas_to_rad = math.pi / (180.0 * 3600.0 * 1000.0)
    mua_cosd = pmra_mas * mas_to_rad
    mud = pmde_mas * mas_to_rad
    e_alpha = (-math.sin(a), math.cos(a), 0.0)
    e_delta = (-math.cos(a) * math.sin(d), -math.sin(a) * math.sin(d), math.cos(d))
    v = tuple(mua_cosd * e_alpha[i] + mud * e_delta[i] for i in range(3))
    ce, se = math.cos(EPS), math.sin(EPS)
    vx = v[0]
    vy = ce * v[1] + se * v[2]
    vz = -se * v[1] + ce * v[2]
    lam_deg, beta_deg = eq_to_ecl(ra_deg, dec_deg)
    lam = math.radians(lam_deg)
    beta = math.radians(beta_deg)
    el_lon = (-math.sin(lam), math.cos(lam), 0.0)
    el_lat = (-math.cos(lam) * math.sin(beta), -math.sin(lam) * math.sin(beta), math.cos(beta))
    mu_lon_cosb = vx * el_lon[0] + vy * el_lon[1] + vz * el_lon[2]
    mu_lat = vx * el_lat[0] + vy * el_lat[1] + vz * el_lat[2]
    cos_b = math.cos(beta)
    mu_lon = mu_lon_cosb / cos_b if abs(cos_b) > 1e-9 else 0.0
    rad_to_mas = 1.0 / mas_to_rad
    return mu_lon * rad_to_mas, mu_lat * rad_to_mas


def sep_arcsec(ra1, de1, ra2, de2):
    p1, p2 = math.radians(de1), math.radians(de2)
    dl = math.radians(ra1 - ra2)
    a = math.sin((p1 - p2) / 2) ** 2 + math.cos(p1) * math.cos(p2) * math.sin(dl / 2) ** 2
    return math.degrees(2 * math.asin(min(1.0, math.sqrt(a)))) * 3600.0


# --- parsers ----------------------------------------------------------------
def parse_bsc(path):
    """Return (stars, n_records). Only records with a J2000 position are kept."""
    stars = []
    n = 0
    with open(path, encoding="latin-1") as f:
        for line in f:
            line = line.rstrip("\n")
            if len(line) < 107:
                continue
            n += 1
            ra_h = line[75:77].strip()
            if not ra_h:
                continue  # novae / extragalactic objects kept only for numbering
            hr = int(line[0:4])
            hd_s = line[25:31].strip()
            ra = int(ra_h) * 15.0 + int(line[77:79]) * 0.25 + float(line[79:83]) * 15.0 / 3600.0
            sign = -1.0 if line[83] == "-" else 1.0
            dec = sign * (int(line[84:86]) + int(line[86:88]) / 60.0 + int(line[88:90]) / 3600.0)
            vmag = float(line[102:107])
            pmra = float(line[148:154]) * 1000.0 if line[148:154].strip() else 0.0
            pmde = float(line[154:160]) * 1000.0 if line[154:160].strip() else 0.0
            flam = line[4:7].strip()
            greek = line[7:10].strip()
            gdig = line[10:11].strip()
            con = line[11:14].strip()
            stars.append(dict(
                hr=hr, hd=int(hd_s) if hd_s else None, ra=ra, dec=dec, vmag=vmag,
                pmra=pmra, pmde=pmde, flam=flam, greek=greek, gdig=gdig, con=con,
            ))
    return stars, n


def parse_iau(path):
    """IAU-CSN rows: dict(name, desig, con, mag, hip, hd, ra, dec)."""
    rows = []
    with open(path, encoding="utf-8") as f:
        for line in f:
            if line.startswith(("#", "$")) or not line.strip():
                continue
            name = line[0:18].strip()
            if not name:
                continue
            rest = line[18:].split()
            didx = None
            for i, t in enumerate(rest):
                if re.match(r"^\d{4}-\d{2}-\d{2}$", t):
                    didx = i
                    break
            if didx is None or didx < 4:
                continue
            try:
                ra = float(rest[didx - 2])
                dec = float(rest[didx - 1])
            except ValueError:
                continue
            hd = rest[didx - 3]
            hip = rest[didx - 4]
            desig = line[36:49].strip()
            con = line[61:64].strip()
            mag = None
            for t in rest[: didx - 4][::-1]:
                try:
                    mag = float(t)
                    break
                except ValueError:
                    continue
            rows.append(dict(
                name=name, desig=desig, con=con, mag=mag,
                hip=int(hip) if hip.isdigit() else None,
                hd=int(hd) if hd.isdigit() else None, ra=ra, dec=dec,
            ))
    return rows


def parse_curated(lib_rs):
    """Curated (name, constellation, nature) from the `CATALOG` static."""
    src = open(lib_rs, encoding="utf-8").read()
    start = src.index("pub static CATALOG: &[FixedStar] = &[")
    end = src.index("#[cfg(test)]", start)
    body = src[start:end]
    pat = re.compile(
        r'name:\s*"([^"]*)",\s*constellation:\s*"([^"]*)",.*?nature:\s*"([^"]*)"', re.S
    )
    return [dict(name=a, constellation=b, nature=c) for a, b, c in pat.findall(body)]


def designation(s):
    """Bayer / Flamsteed designation, e.g. 'Alpha Andromedae', '33 Piscium',
    'Alpha1 Centauri'; 'HR nnnn' when the BSC gives the star no such name."""
    con = CONSTELLATIONS.get(s["con"])
    if not con or not (s["greek"] or s["flam"]):
        return f"HR {s['hr']}"
    if s["greek"]:
        g = GREEK.get(s["greek"], s["greek"])
        return f"{g}{s['gdig']} {con[1]}"
    return f"{s['flam']} {con[1]}"


def join_iau_to_bsc(r, by_hr, by_hd, conflicts=None):
    """BSC star for one IAU-CSN row: the HD number first, then the "HR nnn"
    designation. When both exist and name different stars (a resolved double:
    Izar, Tegmine, Alrescha), take the one nearer the IAU position, and the
    IAU's own "HR nnn" when the two are within 1 arcsec of each other."""
    by_hd_star = None
    if r["hd"] is not None and len(by_hd.get(r["hd"], [])) == 1:
        by_hd_star = by_hd[r["hd"]][0]
    by_hr_star = None
    m = re.match(r"^HR (\d+)$", r["desig"])
    if m and int(m.group(1)) in by_hr:
        by_hr_star = by_hr[int(m.group(1))]
    if by_hd_star is None or by_hr_star is None or by_hd_star is by_hr_star:
        return by_hd_star or by_hr_star
    d_hd = sep_arcsec(r["ra"], r["dec"], by_hd_star["ra"], by_hd_star["dec"])
    d_hr = sep_arcsec(r["ra"], r["dec"], by_hr_star["ra"], by_hr_star["dec"])
    pick = by_hr_star if d_hr <= d_hd + 1.0 else by_hd_star
    if conflicts is not None:
        conflicts.append((r["name"], by_hd_star["hr"], by_hr_star["hr"], pick["hr"]))
    return pick


def main():
    here = os.path.dirname(os.path.abspath(__file__))
    root = os.path.abspath(os.path.join(here, "..", ".."))
    ap = argparse.ArgumentParser()
    ap.add_argument("--bsc", required=True, help="path to the CDS V/50 `catalog` file")
    ap.add_argument("--iau", default=os.path.join(here, "IAU-CSN.txt"))
    ap.add_argument("--lib", default=os.path.join(root, "crates/xalen-stars/src/lib.rs"))
    ap.add_argument("--out", default=os.path.join(root, "crates/xalen-stars/src/bright_generated.rs"))
    ap.add_argument("--skip-sha", action="store_true")
    args = ap.parse_args()

    digest = hashlib.sha256(open(args.bsc, "rb").read()).hexdigest()
    if digest != BSC_SHA256 and not args.skip_sha:
        sys.exit(f"BSC catalog sha256 {digest} != expected {BSC_SHA256}")

    bsc, n_records = parse_bsc(args.bsc)
    iau = parse_iau(args.iau)
    curated = parse_curated(args.lib)
    by_hr = {s["hr"]: s for s in bsc}
    by_hd = {}
    for s in bsc:
        if s["hd"] is not None:
            by_hd.setdefault(s["hd"], []).append(s)

    # --- IAU-CSN -> BSC join (HD number first, then "HR nnn" designation) -----
    iau_for_hr = {}   # hr -> [iau rows]
    unjoined_iau = 0
    conflicts = []
    for r in iau:
        target = join_iau_to_bsc(r, by_hr, by_hd, conflicts)
        if target is None:
            unjoined_iau += 1
            continue
        sep = sep_arcsec(r["ra"], r["dec"], target["ra"], target["dec"])
        r["sep_bsc"] = sep
        if sep > JOIN_SEP_REFUSE_ARCSEC:
            print(f"REFUSED join {r['name']} -> HR {target['hr']}: IAU vs BSC sep {sep:.1f}\"")
            unjoined_iau += 1
            continue
        iau_for_hr.setdefault(target["hr"], []).append(r)

    # One IAU row per star: prefer the row whose designation is "HR n" for this
    # star, then the brightest.
    iau_pick = {}
    multi = []
    for hr, rows in iau_for_hr.items():
        rows.sort(key=lambda r: (r["desig"] != f"HR {hr}", r["mag"] if r["mag"] is not None else 99.0))
        iau_pick[hr] = rows[0]
        if len(rows) > 1:
            multi.append((hr, [r["name"] for r in rows]))
    iau_by_name = {}
    for r in iau:
        iau_by_name.setdefault(r["name"], r)

    # --- curated traditional-name join ----------------------------------------
    curated_for_hr = {}
    curated_unjoined = []
    curated_alias = 0
    for c in curated:
        hr = None
        r = iau_by_name.get(c["name"]) or iau_by_name.get(CURATED_TO_IAU.get(c["name"], ""))
        if r is not None:
            for h, p in iau_pick.items():
                if p is r:
                    hr = h
                    break
            if hr is None:  # the IAU row lost the one-per-star pick; use its own join
                for h, rows in iau_for_hr.items():
                    if r in rows:
                        hr = h
                        break
            if c["name"] in CURATED_TO_IAU and hr is not None:
                curated_alias += 1
        elif c["name"] in CURATED_TO_BSC_BAYER:
            greek, con = CURATED_TO_BSC_BAYER[c["name"]]
            hits = [s for s in bsc if s["greek"] == greek and s["con"] == con and not s["gdig"]]
            if len(hits) == 1:
                hr = hits[0]["hr"]
                curated_alias += 1
        if hr is None or hr in curated_for_hr:
            curated_unjoined.append(c["name"])
            continue
        curated_for_hr[hr] = c

    # --- build records --------------------------------------------------------
    records = []
    sep_report = []
    for s in sorted(bsc, key=lambda s: s["hr"]):
        r = iau_pick.get(s["hr"])
        ra, dec = (r["ra"], r["dec"]) if r else (s["ra"], s["dec"])
        if r and r["sep_bsc"] > JOIN_SEP_REPORT_ARCSEC:
            sep_report.append((s["hr"], r["name"], r["sep_bsc"]))
        lon, lat = eq_to_ecl(ra, dec)
        pm_lon, pm_lat = ecliptic_pm(ra, dec, s["pmra"], s["pmde"])
        c = curated_for_hr.get(s["hr"])
        con_abbr = r["con"] if r and r["con"] in CONSTELLATIONS else s["con"]
        con_full = CONSTELLATIONS[con_abbr][0] if con_abbr in CONSTELLATIONS else ""
        if c:
            name, con_full, nature = c["name"], c["constellation"], c["nature"]
        elif r:
            name, nature = r["name"], ""
        else:
            name, nature = None, ""
        records.append(dict(
            hr=s["hr"], hd=s["hd"], desig=designation(s), name=name, con=con_full,
            lon=lon, lat=lat, mag=s["vmag"], nature=nature,
            pm_lon=pm_lon, pm_lat=pm_lat, iau=bool(r),
        ))

    # IAU spellings that differ from the carried name (curated spelling wins as
    # `name`): keep them findable by name.
    aliases = []
    for hr, c in curated_for_hr.items():
        r = iau_pick.get(hr)
        if r and r["name"].lower() != c["name"].lower():
            aliases.append((r["name"], hr))
    aliases.sort(key=lambda t: t[0].lower())

    n_iau = sum(1 for x in records if x["iau"])
    n_named = sum(1 for x in records if x["name"])
    n_curated = len(curated_for_hr)
    n_con = sum(1 for x in records if x["con"])

    def lit(v):
        return "None" if v is None else f'Some("{v}")'

    out = []
    out.append("// @generated by tools/gen-star-catalog/gen_bright_catalog.py - DO NOT EDIT BY HAND.")
    out.append("//")
    out.append("// Some magnitudes/coordinates (e.g. 6.28, 3.14) numerically resemble TAU/PI but")
    out.append("// are catalogue data, not math constants.")
    out.append("#![allow(clippy::approx_constant)]")
    out.append("//")
    out.append("// Sources (neither carries a non-commercial restriction):")
    out.append("//   * Yale Bright Star Catalogue, 5th Revised Ed. (Preliminary Version),")
    out.append("//     Hoffleit D., Warren Jr W.H., Astronomical Data Center, NSSDC/ADC (1991),")
    out.append(f"//     CDS V/50, {n_records} records, {len(bsc)} stars with a J2000 position.")
    out.append(f"//     File sha256 {BSC_SHA256}.")
    out.append("//   * IAU Catalog of Star Names (IAU WGSN, Creative Commons Attribution):")
    out.append("//     proper names, constellations and 6-decimal J2000 positions.")
    out.append("//")
    out.append(f"// {n_iau} stars take their position from the IAU-CSN row; the other")
    out.append(f"// {len(records) - n_iau} use the BSC J2000 position (RA to 0.1 s, Dec to 1\").")
    out.append("// Proper motion and magnitude always come from the BSC. Positions are")
    out.append("// equinox J2000, epoch 2000.0, converted to ecliptic longitude/latitude at the")
    out.append(f"// IAU 2006 J2000 mean obliquity ({OBLIQUITY_J2000_DEG:.10f} deg).")
    out.append(f"// {n_curated} curated traditional names are joined ({curated_alias} through a")
    out.append("// spelling alias or a Bayer designation); the rest of the curated table")
    out.append("// (open clusters) has no single star.")
    out.append("")
    out.append("use crate::BrightStar;")
    out.append("")
    out.append("/// Number of stars in [`BRIGHT_CATALOG`].")
    out.append(f"pub const BRIGHT_STAR_COUNT: usize = {len(records)};")
    out.append("")
    out.append("/// IAU-CSN spellings that differ from the curated name carried on the star:")
    out.append("/// `(IAU name, HR number)`, sorted by name.")
    out.append("pub static BRIGHT_NAME_ALIASES: &[(&str, u32)] = &[")
    for nm, hr in aliases:
        out.append(f'    ("{nm}", {hr}),')
    out.append("];")
    out.append("")
    out.append("/// Fixed stars of the Yale Bright Star Catalogue (9,096 stars, V up to about 6.5),")
    out.append("/// sorted by HR number.")
    out.append("pub static BRIGHT_CATALOG: &[BrightStar] = &[")
    for x in records:
        out.append(
            "    BrightStar { "
            f"hr: {x['hr']}, hd: {('Some(%d)' % x['hd']) if x['hd'] is not None else 'None'}, "
            f"designation: \"{x['desig']}\", name: {lit(x['name'])}, constellation: \"{x['con']}\", "
            f"longitude_j2000: {x['lon']:.6f}, latitude_j2000: {x['lat']:.6f}, "
            f"magnitude: {x['mag']:.2f}, nature: \"{x['nature']}\", "
            f"pm_lon_mas_per_year: {x['pm_lon']:.3f}, pm_lat_mas_per_year: {x['pm_lat']:.3f} "
            "},"
        )
    out.append("];")
    out.append("")
    with open(args.out, "w", encoding="utf-8") as f:
        f.write("\n".join(out))

    print(f"BSC records read          : {n_records}")
    print(f"BSC stars emitted         : {len(records)}")
    print(f"IAU-CSN rows              : {len(iau)} ({unjoined_iau} not in the BSC)")
    print(f"stars with IAU position   : {n_iau}")
    print(f"stars with a name         : {n_named}")
    print(f"stars with a constellation: {n_con}")
    print(f"curated names joined      : {n_curated} / {len(curated)} ({curated_alias} via alias/Bayer)")
    print(f"curated unjoined          : {curated_unjoined}")
    print(f"name aliases emitted      : {aliases}")
    print(f"several IAU rows -> 1 star: {multi}")
    print(f"HD join vs HR join differ  : {conflicts} (name, HD-join HR, HR-join HR, chosen)")
    print(f"IAU vs BSC sep > {JOIN_SEP_REPORT_ARCSEC}\"   : {sep_report}")
    print(f"wrote {args.out}")


if __name__ == "__main__":
    main()
