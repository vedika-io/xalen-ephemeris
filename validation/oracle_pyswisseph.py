#!/usr/bin/env python3
"""Reproducible accuracy oracle for the XALEN ephemeris, generated with pyswisseph.

This script deterministically samples N random charts (Julian Day, latitude,
longitude) across a wide span of years and, for each chart, computes a fixed set
of quantities with the Swiss Ephemeris Python bindings:

  * geocentric apparent ecliptic longitude/latitude of every body
    Sun..Pluto plus the mean node, true node, mean apogee and osculating
    apogee (Swiss bodies 0..13);
  * the Placidus ascendant, midheaven and twelve house cusps (tropical);
  * the ayanamsa for several Swiss sidereal modes (Lahiri, Fagan-Bradley,
    Krishnamurti, Raman) at the same epoch.

The records are streamed to stdout (or a file) as one compact JSON object per
line (JSON Lines), which the companion Rust runner (`xalen-validation`) reads
back to compute per-body deltas against the pure-Rust XALEN crates.

HONESTY: this build of pyswisseph has no JPL/Swiss `.se1` data files installed,
so `swe.calc_ut` silently falls back to the built-in Moshier analytic theory.
The Moshier longitudes are themselves a model (sub-arcsecond vs DE for the modern
era, growing for distant epochs and the Moon), not DE440 ground truth. The
oracle therefore records, per body, the Swiss return-flag (`SEFLG_SWIEPH` bit set
=> Swiss data file used; `SEFLG_MOSEPH` bit => Moshier fallback) so the
downstream comparison can be interpreted honestly. The first record additionally
carries a `_meta` object describing the backend, the swisseph version, and the
generation parameters.

Usage:
    python3 oracle_pyswisseph.py [--n N] [--seed S] [--start-jd JD] [--end-jd JD]
                                 [--ephe-path DIR] [--out FILE]

    # Defaults: N=100000, seed=42, span = JD for years 1500..2400.
    python3 oracle_pyswisseph.py > oracle.jsonl
    python3 oracle_pyswisseph.py --n 5000000 --seed 42 --out oracle.jsonl
"""

import argparse
import json
import random
import sys

import swisseph as swe

# Swiss body ids 0..13 in a fixed order; the Rust runner maps these to the
# corresponding `xalen_ephem::Body` variants by the same name.
BODIES = [
    ("sun", swe.SUN),
    ("moon", swe.MOON),
    ("mercury", swe.MERCURY),
    ("venus", swe.VENUS),
    ("mars", swe.MARS),
    ("jupiter", swe.JUPITER),
    ("saturn", swe.SATURN),
    ("uranus", swe.URANUS),
    ("neptune", swe.NEPTUNE),
    ("pluto", swe.PLUTO),
    ("mean_node", swe.MEAN_NODE),
    ("true_node", swe.TRUE_NODE),
    ("mean_apogee", swe.MEAN_APOG),
    ("osculating_apogee", swe.OSCU_APOG),
]

# Swiss sidereal-mode ids; names match the xalen `Ayanamsa` variant names that
# the Rust runner resolves via `Ayanamsa::from_swiss_ephem_id`.
AYANAMSAS = [
    ("lahiri", swe.SIDM_LAHIRI),
    ("fagan_bradley", swe.SIDM_FAGAN_BRADLEY),
    ("krishnamurti", swe.SIDM_KRISHNAMURTI),
    ("raman", swe.SIDM_RAMAN),
]

# JD for 1500-01-01 and 2400-01-01 (proleptic Gregorian noon) — a deliberately
# wide span that exercises the analytic theories well outside the modern era.
DEFAULT_START_JD = 2_268_923.5  # 1500-01-01 00:00 UT
DEFAULT_END_JD = 2_597_641.5    # 2400-01-01 00:00 UT


def backend_name(retflag: int) -> str:
    """Map a Swiss `calc_ut` return flag to a human-readable backend label."""
    if retflag < 0:
        return "error"
    if retflag & swe.FLG_MOSEPH:
        return "moshier"
    if retflag & swe.FLG_SWIEPH:
        return "swieph"
    if retflag & swe.FLG_JPLEPH:
        return "jpleph"
    return "unknown"


def compute_record(jd: float, lat: float, lon: float):
    """Compute all oracle quantities for one chart. Returns (record, backends)."""
    flag = swe.FLG_SWIEPH | swe.FLG_SPEED
    # Nodes and apogees are mean elements XALEN reports in the mean ecliptic of
    # date (no nutation in longitude). Swiss calc_ut applies Δψ by default, which
    # would inject a spurious ~Δψ (≈11" mean) offset. SEFLG_NONUT puts Swiss in
    # the same mean-of-date frame so the comparison is apples-to-apples. Real
    # bodies (Sun..Pluto) stay apparent (with nutation) — that is the frame XALEN
    # reports them in and they match Swiss to ~0.2-0.6".
    MEAN_FRAME = {"mean_node", "true_node", "mean_apogee", "osculating_apogee"}
    bodies = {}
    backends = {}
    for name, bid in BODIES:
        body_flag = flag | swe.FLG_NONUT if name in MEAN_FRAME else flag
        xx, retflag = swe.calc_ut(jd, bid, body_flag)
        # xx = (lon, lat, dist, lon_speed, lat_speed, dist_speed)
        bodies[name] = [xx[0], xx[1]]
        backends[name] = backend_name(retflag)

    # Tropical Placidus houses; cusps[0]=house 1 (ASC), ascmc[0]=ASC, [1]=MC.
    cusps, ascmc = swe.houses_ex(jd, lat, lon, b"P", 0)
    houses = {
        "asc": ascmc[0],
        "mc": ascmc[1],
        "cusps": list(cusps),  # 12 tropical cusps, degrees [0,360)
    }

    ayan = {}
    for name, sid in AYANAMSAS:
        swe.set_sid_mode(sid, 0, 0)
        # get_ayanamsa_ex_ut(jd, 0) is the APPARENT (with-nutation) ayanamsa —
        # the same quantity XALEN computes. The older get_ayanamsa_ut returns the
        # MEAN ayanamsa (no Δψ), which would show a spurious ~11" offset.
        _retflag, aya = swe.get_ayanamsa_ex_ut(jd, 0)
        ayan[name] = aya

    rec = {
        "jd": jd,
        "lat": lat,
        "lon": lon,
        "bodies": bodies,
        "houses": houses,
        "ayanamsa": ayan,
    }
    return rec, backends


def main() -> int:
    p = argparse.ArgumentParser(description="XALEN pyswisseph accuracy oracle")
    p.add_argument("--n", type=int, default=100_000, help="number of charts")
    p.add_argument("--seed", type=int, default=42, help="PRNG seed (deterministic)")
    p.add_argument("--start-jd", type=float, default=DEFAULT_START_JD)
    p.add_argument("--end-jd", type=float, default=DEFAULT_END_JD)
    p.add_argument(
        "--ephe-path",
        type=str,
        default=None,
        help="directory of Swiss .se1 data files; if omitted Moshier is used",
    )
    p.add_argument("--out", type=str, default=None, help="output file (default stdout)")
    args = p.parse_args()

    if args.ephe_path:
        swe.set_ephe_path(args.ephe_path)

    rng = random.Random(args.seed)
    out = open(args.out, "w") if args.out else sys.stdout

    # Probe the backend once so the meta line is honest even before the first
    # full record is built. Aggregate the per-body backend over the run too.
    seen_backends = {}

    try:
        for i in range(args.n):
            jd = rng.uniform(args.start_jd, args.end_jd)
            lat = rng.uniform(-66.0, 66.0)   # Placidus is well-conditioned here
            lon = rng.uniform(-180.0, 180.0)
            rec, backends = compute_record(jd, lat, lon)
            for b, name in backends.items():
                seen_backends.setdefault(b, set()).add(name)
            if i == 0:
                rec["_meta"] = {
                    "swisseph_version": swe.version,
                    "n": args.n,
                    "seed": args.seed,
                    "start_jd": args.start_jd,
                    "end_jd": args.end_jd,
                    "ephe_path": args.ephe_path,
                    "body_backend": {b: name for b, name in backends.items()},
                }
            out.write(json.dumps(rec, separators=(",", ":")))
            out.write("\n")
    finally:
        if out is not sys.stdout:
            out.close()
        swe.close()

    # Backend summary to stderr so stdout stays pure JSONL.
    summary = {b: sorted(s) for b, s in seen_backends.items()}
    sys.stderr.write(
        "oracle: %d charts, span JD %.1f..%.1f, seed %d, swisseph %s\n"
        % (args.n, args.start_jd, args.end_jd, args.seed, swe.version)
    )
    sys.stderr.write("oracle backend per body: %s\n" % json.dumps(summary))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
