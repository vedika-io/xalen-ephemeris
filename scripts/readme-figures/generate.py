#!/usr/bin/env python3
"""Generate the figures shown in README.md from real `xalen` runs.

Run from the repository root:

    python3 scripts/readme-figures/generate.py
    python3 scripts/readme-figures/generate.py --xalen /path/to/xalen   # skip the build

Without --xalen the script builds the CLI first (`cargo build --release -p
xalen-cli`) and uses the binary from the workspace target directory. It needs
Python 3.8 or newer and nothing outside the standard library. It needs network
access once, to download the Natural Earth 110m coastline (public domain) for
the map, pinned to natural-earth-vector commit ca96624 (VERSION 5.2.0-pre);
the coastline is simplified and embedded in the SVG, and the GeoJSON is not
kept.

Every SVG is written into docs/assets/:

    chart-wheel.svg, chart-wheel-dark.svg   birth chart wheel  (xalen chart --json)
    map.svg, map-dark.svg                   astrocartography   (xalen map --geojson --parans)
    accuracy.svg, accuracy-dark.svg         error per body, the figures of docs/ACCURACY.md
    terminal.svg                            one `xalen positions` run, its output verbatim

The logo is not generated here: docs/assets/xalen-ephemeris-logo.png is the
official asset and docs/assets/xalen-ephemeris-logo.svg a tracing of it.

The example birth is the moment of Indian independence, 15 August 1947 00:00
IST (UTC+05:30) at Delhi from the built-in city table, the same moment as the
Rust quick start in the README.
"""

import argparse
import json
import math
import os
import subprocess
import sys
import urllib.request
from xml.sax.saxutils import escape

REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
OUT = os.path.join(REPO, "docs", "assets")

# Pinned to one commit of nvkelso/natural-earth-vector (master of 2 June 2022,
# VERSION 5.2.0-pre) so the map is reproducible.
NATURAL_EARTH_COMMIT = "ca96624a56bd078437bca8184e78163e5039ad19"
COASTLINE_URL = (
    "https://raw.githubusercontent.com/nvkelso/natural-earth-vector/"
    f"{NATURAL_EARTH_COMMIT}/geojson/ne_110m_coastline.geojson"
)

# The example birth. The date carries its own offset; the CLI turns it into UT.
BIRTH_DATE = "1947-08-15T00:00:00+05:30"
BIRTH_PLACE = "delhi"
BIRTH_LABEL = "Indian independence, 15 August 1947, 00:00 IST (UTC+05:30), Delhi"
AYANAMSA = "Lahiri"
HOUSE_SYSTEM = "placidus"

CHART_BODIES = [
    "sun", "moon", "mercury", "venus", "mars", "jupiter", "saturn",
    "uranus", "neptune", "pluto", "rahu", "ketu",
]
CLASSICAL = ["sun", "moon", "mercury", "venus", "mars", "jupiter", "saturn"]
ABBR = {
    "sun": "Su", "moon": "Mo", "mercury": "Me", "venus": "Ve", "mars": "Ma",
    "jupiter": "Ju", "saturn": "Sa", "uranus": "Ur", "neptune": "Ne",
    "pluto": "Pl", "mean-node": "Ra", "ketu": "Ke",
}
SIGNS = ["Ari", "Tau", "Gem", "Can", "Leo", "Vir",
         "Lib", "Sco", "Sag", "Cap", "Aqu", "Pis"]

SANS = "system-ui, -apple-system, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif"
MONO = "ui-monospace, SFMono-Regular, Menlo, Consolas, 'Liberation Mono', 'DejaVu Sans Mono', monospace"

LIGHT = {
    "fg": "#1f2328", "page": "#ffffff", "muted": "#59636e", "faint": "#d1d9e0", "band": "#f6f8fa",
    "accent": "#0969da", "water": "#f3f7fb", "land": "#8c959f",
    "planet": {
        "sun": "#d97706", "moon": "#57606a", "mercury": "#047857",
        "venus": "#be185d", "mars": "#cf222e", "jupiter": "#6d28d9",
        "saturn": "#1d4ed8",
    },
    "other": "#59636e",
}
DARK = {
    "fg": "#e6edf3", "page": "#0d1117", "muted": "#9198a1", "faint": "#3d444d", "band": "#151b23",
    "accent": "#4493f8", "water": "#0f151d", "land": "#6e7681",
    "planet": {
        "sun": "#fbbf24", "moon": "#c9d1d9", "mercury": "#34d399",
        "venus": "#f472b6", "mars": "#ff7b72", "jupiter": "#c4b5fd",
        "saturn": "#79c0ff",
    },
    "other": "#9198a1",
}


def run_xalen(xalen, args):
    """Run the CLI and return its stdout."""
    result = subprocess.run([xalen] + args, check=True, capture_output=True, text=True)
    return result.stdout


def build_cli():
    subprocess.run(["cargo", "build", "--release", "-p", "xalen-cli"], cwd=REPO, check=True)
    meta = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        cwd=REPO, check=True, capture_output=True, text=True,
    )
    target = json.loads(meta.stdout)["target_directory"]
    return os.path.join(target, "release", "xalen")


def svg_open(width, height, title, desc):
    return [
        '<?xml version="1.0" encoding="UTF-8"?>',
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" '
        f'viewBox="0 0 {width} {height}" role="img" aria-labelledby="t d">',
        f'<title id="t">{escape(title)}</title>',
        f'<desc id="d">{escape(desc)}</desc>',
    ]


def write(name, lines):
    path = os.path.join(OUT, name)
    with open(path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")
    print(f"wrote {os.path.relpath(path, REPO)} ({os.path.getsize(path):,} bytes)")


def f1(x):
    return f"{x:.1f}".rstrip("0").rstrip(".")


def colour(pal, body):
    return pal["planet"].get(body, pal["other"])


# --------------------------------------------------------------------------
# Chart wheel
# --------------------------------------------------------------------------

def spread(angles, min_gap):
    """Move label angles apart (degrees, ascending order kept) so neighbours are
    at least `min_gap` apart, staying close to the true positions."""
    n = len(angles)
    if n < 2:
        return list(angles)
    order = sorted(range(n), key=lambda i: angles[i])
    # Start the circle after the widest gap so the sequence does not wrap.
    vals = [angles[i] for i in order]
    gaps = [(vals[(k + 1) % n] - vals[k]) % 360 for k in range(n)]
    start = (gaps.index(max(gaps)) + 1) % n
    order = order[start:] + order[:start]
    base = angles[order[0]]
    lin = [(angles[i] - base) % 360 for i in order]
    pos = list(lin)
    for _ in range(200):
        moved = False
        for k in range(1, n):
            if pos[k] - pos[k - 1] < min_gap:
                push = (min_gap - (pos[k] - pos[k - 1])) / 2 + 1e-6
                pos[k - 1] -= push
                pos[k] += push
                moved = True
        if not moved:
            break
    out = [0.0] * n
    for k, i in enumerate(order):
        out[i] = (pos[k] + base) % 360
    return out


def chart_wheel(chart, pal, dark):
    W, H = 700, 770
    cx, cy = 350, 340
    r_out, r_sign, r_lab, r_tick, r_house, r_num = 300, 262, 214, 252, 132, 112
    asc = chart["ascendant"]["longitude"]

    def ang(lon):
        return math.radians(180.0 + (lon - asc))

    def pt(lon, r):
        a = ang(lon)
        return cx + r * math.cos(a), cy - r * math.sin(a)

    place = chart["place"]
    aya = chart["ayanamsa"]
    lines = svg_open(
        W, H,
        "Birth chart wheel computed by xalen chart",
        f"{BIRTH_LABEL}. Sidereal zodiac, {aya['name']} ayanamsa, "
        f"{chart['house_system_name']} houses. Planets at their computed longitudes.",
    )
    lines.append(f'<g font-family="{SANS}" fill="none" stroke-linecap="round">')
    # Rings and sign band.
    lines.append(f'<circle cx="{cx}" cy="{cy}" r="{r_out}" fill="{pal["band"]}" stroke="{pal["muted"]}" stroke-width="1.5"/>')
    lines.append(f'<circle cx="{cx}" cy="{cy}" r="{r_sign}" fill="none" stroke="{pal["muted"]}" stroke-width="1"/>')
    lines.append(f'<circle cx="{cx}" cy="{cy}" r="{r_house}" stroke="{pal["muted"]}" stroke-width="1"/>')
    lines.append(f'<circle cx="{cx}" cy="{cy}" r="{r_num - 22}" stroke="{pal["faint"]}" stroke-width="1"/>')
    for s in range(12):
        x1, y1 = pt(s * 30, r_sign)
        x2, y2 = pt(s * 30, r_out)
        lines.append(f'<line x1="{f1(x1)}" y1="{f1(y1)}" x2="{f1(x2)}" y2="{f1(y2)}" stroke="{pal["muted"]}" stroke-width="1"/>')
        for d in range(5, 30, 5):
            x1, y1 = pt(s * 30 + d, r_sign)
            x2, y2 = pt(s * 30 + d, r_sign + (8 if d == 15 else 5))
            lines.append(f'<line x1="{f1(x1)}" y1="{f1(y1)}" x2="{f1(x2)}" y2="{f1(y2)}" stroke="{pal["muted"]}" stroke-width="0.8"/>')
        tx, ty = pt(s * 30 + 15, (r_sign + r_out) / 2 + 3)
        lines.append(f'<text x="{f1(tx)}" y="{f1(ty)}" fill="{pal["fg"]}" stroke="none" font-size="15" font-weight="600" text-anchor="middle" dominant-baseline="middle">{SIGNS[s]}</text>')
    # House cusps.
    cusps = [c["longitude"] for c in chart["cusps"]]
    for i, c in enumerate(cusps):
        angle_cusp = i in (0, 3, 6, 9)
        x1, y1 = pt(c, r_num - 22)
        x2, y2 = pt(c, r_sign)
        lines.append(
            f'<line x1="{f1(x1)}" y1="{f1(y1)}" x2="{f1(x2)}" y2="{f1(y2)}" '
            f'stroke="{pal["accent"] if angle_cusp else pal["faint"]}" '
            f'stroke-width="{2 if angle_cusp else 1.2}"/>'
        )
        nxt = cusps[(i + 1) % 12]
        mid = c + ((nxt - c) % 360) / 2
        tx, ty = pt(mid, r_num)
        lines.append(f'<text x="{f1(tx)}" y="{f1(ty)}" fill="{pal["muted"]}" stroke="none" font-size="13" text-anchor="middle" dominant-baseline="middle">{i + 1}</text>')
    # Angle labels outside the wheel.
    for key, label in (("ascendant", "ASC"), ("mc", "MC"), ("descendant", "DSC"), ("ic", "IC")):
        lon = chart[key]["longitude"]
        x1, y1 = pt(lon, r_out)
        x2, y2 = pt(lon, r_out + 6)
        lines.append(f'<line x1="{f1(x1)}" y1="{f1(y1)}" x2="{f1(x2)}" y2="{f1(y2)}" stroke="{pal["accent"]}" stroke-width="2"/>')
        tx, ty = pt(lon, r_out + 21)
        lines.append(f'<text x="{f1(tx)}" y="{f1(ty)}" fill="{pal["accent"]}" stroke="none" font-size="12" font-weight="700" text-anchor="middle" dominant-baseline="middle">{label}</text>')
    # Planets.
    bodies = [b for b in chart["bodies"] if b["id"] in ABBR]
    lons = [b["longitude"] for b in bodies]
    label_lons = spread(lons, 11.0)
    for b, lon, llon in zip(bodies, lons, label_lons):
        col = colour(pal, b["id"])
        x1, y1 = pt(lon, r_sign)
        x2, y2 = pt(lon, r_tick - 6)
        lines.append(f'<line x1="{f1(x1)}" y1="{f1(y1)}" x2="{f1(x2)}" y2="{f1(y2)}" stroke="{col}" stroke-width="2.5"/>')
        x3, y3 = pt(llon, r_lab + 16)
        lines.append(f'<line x1="{f1(x2)}" y1="{f1(y2)}" x2="{f1(x3)}" y2="{f1(y3)}" stroke="{col}" stroke-width="0.8" opacity="0.7"/>')
        tx, ty = pt(llon, r_lab)
        deg = int(b["degree_in_sign"])
        mins = int(round((b["degree_in_sign"] - deg) * 60))
        if mins == 60:
            deg, mins = deg + 1, 0
        retro = "℞" if b.get("retrograde") and b["id"] not in ("mean-node", "ketu") else ""
        lines.append(
            f'<text x="{f1(tx)}" y="{f1(ty - 7)}" fill="{col}" font-size="16" '
            f'font-weight="700" text-anchor="middle" dominant-baseline="middle" paint-order="stroke" stroke="{pal["band"]}" stroke-width="3">{ABBR[b["id"]]}{retro}</text>'
        )
        lines.append(
            f'<text x="{f1(tx)}" y="{f1(ty + 9)}" fill="{pal["muted"]}" font-size="11" '
            f'text-anchor="middle" dominant-baseline="middle" paint-order="stroke" stroke="{pal["band"]}" stroke-width="3">{deg}°{mins:02d}′</text>'
        )
    # Centre: the frame.
    lines.append(f'<text x="{cx}" y="{cy - 10}" fill="{pal["fg"]}" stroke="none" font-size="14" font-weight="600" text-anchor="middle">{escape(aya["name"])}</text>')
    lines.append(f'<text x="{cx}" y="{cy + 10}" fill="{pal["muted"]}" stroke="none" font-size="12" text-anchor="middle">{escape(chart["house_system_name"])} houses</text>')
    # Caption.
    lat, lon = place["latitude"], place["longitude"]
    cap1 = "15 Aug 1947, 00:00 IST (UTC+05:30) · Delhi"
    cap2 = (f"{abs(lat):.2f}°{'N' if lat >= 0 else 'S'} {abs(lon):.2f}°{'E' if lon >= 0 else 'W'}"
            f" · sidereal, {aya['name']} {aya['value_deg']:.4f}° · {chart['house_system_name']}")
    lines.append(f'<text x="{cx}" y="{H - 70}" fill="{pal["fg"]}" stroke="none" font-size="15" font-weight="600" text-anchor="middle">{escape(cap1)}</text>')
    lines.append(f'<text x="{cx}" y="{H - 48}" fill="{pal["muted"]}" stroke="none" font-size="13" text-anchor="middle">{escape(cap2)}</text>')
    lines.append(f'<text x="{cx}" y="{H - 28}" fill="{pal["muted"]}" stroke="none" font-size="11" text-anchor="middle" font-family="{MONO}">xalen chart --date {BIRTH_DATE} --place {BIRTH_PLACE} --sidereal --house-system {HOUSE_SYSTEM}</text>')
    lines.append(f'<text x="{cx}" y="{H - 10}" fill="{pal["muted"]}" stroke="none" font-size="11" text-anchor="middle" font-family="{MONO}">--bodies {",".join(CHART_BODIES)} --json</text>')
    lines.append("</g>")
    lines.append("</svg>")
    return lines


# --------------------------------------------------------------------------
# Astrocartography map
# --------------------------------------------------------------------------

MAP_W, MAP_LAT_MAX, MAP_LAT_MIN = 960.0, 84.0, -60.0
MAP_SCALE = MAP_W / 360.0
MAP_H = (MAP_LAT_MAX - MAP_LAT_MIN) * MAP_SCALE
MAP_X0, MAP_Y0 = 20.0, 56.0


def proj(lon, lat):
    return MAP_X0 + (lon + 180.0) * MAP_SCALE, MAP_Y0 + (MAP_LAT_MAX - lat) * MAP_SCALE


def simplify(points, tol):
    """Douglas-Peucker on a list of (x, y)."""
    if len(points) < 3:
        return points
    keep = [False] * len(points)
    keep[0] = keep[-1] = True
    stack = [(0, len(points) - 1)]
    while stack:
        a, b = stack.pop()
        ax, ay = points[a]
        bx, by = points[b]
        dx, dy = bx - ax, by - ay
        norm = math.hypot(dx, dy)
        best, idx = -1.0, -1
        for i in range(a + 1, b):
            px, py = points[i]
            if norm == 0:
                d = math.hypot(px - ax, py - ay)
            else:
                d = abs(dy * (px - ax) - dx * (py - ay)) / norm
            if d > best:
                best, idx = d, i
        if best > tol:
            keep[idx] = True
            stack.append((a, idx))
            stack.append((idx, b))
    return [p for p, k in zip(points, keep) if k]


def path_d(points):
    out = []
    for i, (x, y) in enumerate(points):
        out.append(("M" if i == 0 else "L") + f"{f1(x)} {f1(y)}")
    return "".join(out)


def coastline_path(geo):
    parts = []
    for feat in geo["features"]:
        g = feat["geometry"]
        rings = [g["coordinates"]] if g["type"] == "LineString" else g["coordinates"]
        for ring in rings:
            pts = simplify([proj(lon, lat) for lon, lat in ring], 0.45)
            if len(pts) >= 2:
                parts.append(path_d(pts))
    return "".join(parts)


def meridian_labels(lines_geo):
    """Place one label per MC or IC line (a meridian) in rows above the map so
    that labels in the same row do not overlap. Returns (labels, row count)."""
    items = []
    for feat in lines_geo["features"]:
        p = feat["properties"]
        if (feat["geometry"]["type"] == "LineString" and p.get("body") in CLASSICAL
                and p["kind"] in ("MC", "IC")):
            lon = feat["geometry"]["coordinates"][0][0]
            items.append((MAP_X0 + (lon + 180.0) * MAP_SCALE, f"{ABBR[p['body']]} {p['kind']}", p["body"]))
    placed = []
    for x, text, body in sorted(items):
        w = 6.8 * len(text)
        row = 0
        while any(r == row and abs(px - x) < (w + 6.8 * len(pt)) / 2 + 3 for px, pt, r, _ in placed):
            row += 1
        placed.append((x, text, row, body))
    return placed, 1 + max((r for _, _, r, _ in placed), default=0)


def astro_map(lines_geo, coast_d, delhi, pal, dark):
    W = int(MAP_W + 2 * MAP_X0)
    legend_y = MAP_Y0 + MAP_H + 34
    H = int(legend_y + 64)
    out = svg_open(
        W, H,
        "Astrocartography map computed by xalen map",
        f"{BIRTH_LABEL}. MC, IC, ascendant and descendant lines of the Sun, Moon, "
        "Mercury, Venus, Mars, Jupiter and Saturn over a world coastline, with three parans.",
    )
    out.append("<!-- Coastline: Natural Earth 110m coastline, public domain (naturalearthdata.com), "
               "simplified. Lines, zenith points and parans: the xalen map command with the geojson and parans options. -->")
    out.append(f'<defs><clipPath id="frame"><rect x="{MAP_X0}" y="{MAP_Y0}" width="{MAP_W}" height="{f1(MAP_H)}"/></clipPath></defs>')
    out.append(f'<g font-family="{SANS}">')
    out.append(f'<text x="{MAP_X0}" y="24" fill="{pal["fg"]}" font-size="18" font-weight="700">Astrocartography · 15 Aug 1947, 00:00 IST</text>')
    out.append(f'<text x="{MAP_X0}" y="44" fill="{pal["muted"]}" font-size="13">Where each classical planet was rising (ASC), setting (DSC), culminating (MC) or anti-culminating (IC) at that moment</text>')
    out.append(f'<rect x="{MAP_X0}" y="{MAP_Y0}" width="{MAP_W}" height="{f1(MAP_H)}" fill="{pal["water"]}" stroke="{pal["faint"]}"/>')
    out.append('<g clip-path="url(#frame)" fill="none">')
    # Graticule every 30 degrees.
    grid = []
    for lon in range(-150, 180, 30):
        x, _ = proj(lon, 0)
        grid.append(f"M{f1(x)} {f1(MAP_Y0)}V{f1(MAP_Y0 + MAP_H)}")
    for lat in range(-60, 90, 30):
        _, y = proj(0, lat)
        grid.append(f"M{f1(MAP_X0)} {f1(y)}H{f1(MAP_X0 + MAP_W)}")
    out.append(f'<path d="{"".join(grid)}" stroke="{pal["faint"]}" stroke-width="0.8"/>')
    out.append(f'<path d="{coast_d}" stroke="{pal["land"]}" stroke-width="0.9" stroke-linejoin="round"/>')
    # Planet lines.
    for feat in lines_geo["features"]:
        p = feat["properties"]
        if feat["geometry"]["type"] != "LineString" or p.get("body") not in CLASSICAL:
            continue
        pts = [proj(lon, lat) for lon, lat in feat["geometry"]["coordinates"]]
        dash = ' stroke-dasharray="6 4"' if p["kind"] in ("IC", "DSC") else ""
        col = colour(pal, p["body"])
        out.append(f'<path d="{path_d(pts)}" stroke="{col}" stroke-width="1.8"{dash} stroke-linejoin="round"/>')
    # Zenith points.
    for feat in lines_geo["features"]:
        p = feat["properties"]
        if p.get("kind") == "ZENITH" and p.get("body") in CLASSICAL:
            x, y = proj(*feat["geometry"]["coordinates"])
            out.append(f'<circle cx="{f1(x)}" cy="{f1(y)}" r="3.2" fill="{colour(pal, p["body"])}"/>')
    out.append("</g>")
    # MC and IC labels in the band above the map, with a guide down to the frame.
    placed, nrows = meridian_labels(lines_geo)
    for x, text, row, body in placed:
        col = colour(pal, body)
        y = MAP_Y0 - 6 - 13 * row
        if row:
            out.append(f'<line x1="{f1(x)}" y1="{f1(y + 2)}" x2="{f1(x)}" y2="{f1(MAP_Y0)}" stroke="{col}" stroke-width="0.8" opacity="0.6"/>')
        out.append(
            f'<text x="{f1(x)}" y="{f1(y)}" fill="{col}" font-size="11" font-weight="700" '
            f'text-anchor="middle">{text}</text>'
        )
    bx, by = proj(*delhi)
    # Parans: the three closest to the birth latitude.
    parans = [f for f in lines_geo["features"] if f["properties"].get("kind") == "PARAN"
              and f["properties"]["body_a"] in CLASSICAL and f["properties"]["body_b"] in CLASSICAL]
    parans.sort(key=lambda f: abs(f["geometry"]["coordinates"][1] - delhi[1]))
    chosen = parans[:3]
    for f in chosen:
        p = f["properties"]
        lon, lat = f["geometry"]["coordinates"]
        x, y = proj(lon, lat)
        out.append(f'<circle cx="{f1(x)}" cy="{f1(y)}" r="6" fill="none" stroke="{pal["fg"]}" stroke-width="1.8"/>')
        text = f"{ABBR[p['body_a']]} {p['kind_a']} × {ABBR[p['body_b']]} {p['kind_b']}"
        w = 6.6 * len(text)
        right_ok = x + 10 + w < MAP_X0 + MAP_W - 4
        hits_delhi = (x + 10 < bx + 22 and x + 10 + w > bx - 22 and abs(y - by) < 18)
        anchor, dx = ("start", 10) if right_ok and not hits_delhi else ("end", -10)
        out.append(
            f'<text x="{f1(x + dx)}" y="{f1(y + 4)}" fill="{pal["fg"]}" font-size="11" font-weight="600" '
            f'text-anchor="{anchor}" paint-order="stroke" stroke="{pal["water"]}" stroke-width="3">{escape(text)}</text>'
        )
    # Birthplace.
    out.append(f'<rect x="{f1(bx - 4)}" y="{f1(by - 4)}" width="8" height="8" fill="{pal["fg"]}" transform="rotate(45 {f1(bx)} {f1(by)})"/>')
    out.append(f'<text x="{f1(bx)}" y="{f1(by - 9)}" fill="{pal["fg"]}" font-size="11" font-weight="700" text-anchor="middle" paint-order="stroke" stroke="{pal["water"]}" stroke-width="3">Delhi</text>')
    # Graticule labels.
    for lat in range(-30, 90, 30):
        _, y = proj(0, lat)
        lab = "0°" if lat == 0 else f"{abs(lat)}°{'N' if lat > 0 else 'S'}"
        out.append(f'<text x="{f1(MAP_X0 + 4)}" y="{f1(y - 3)}" fill="{pal["muted"]}" font-size="10">{lab}</text>')
    for lon in range(-150, 180, 30):
        x, _ = proj(lon, 0)
        lab = "0°" if lon == 0 else f"{abs(lon)}°{'E' if lon > 0 else 'W'}"
        out.append(f'<text x="{f1(x)}" y="{f1(MAP_Y0 + MAP_H + 14)}" fill="{pal["muted"]}" font-size="10" text-anchor="middle">{lab}</text>')
    # Legend.
    x = MAP_X0
    for body in CLASSICAL:
        col = colour(pal, body)
        out.append(f'<line x1="{f1(x)}" y1="{f1(legend_y)}" x2="{f1(x + 22)}" y2="{f1(legend_y)}" stroke="{col}" stroke-width="3"/>')
        name = body.capitalize()
        out.append(f'<text x="{f1(x + 28)}" y="{f1(legend_y + 4)}" fill="{pal["fg"]}" font-size="12">{name} ({ABBR[body]})</text>')
        x += 28 + 8.0 * len(name) + 46
    y2 = legend_y + 24
    out.append(f'<line x1="{f1(MAP_X0)}" y1="{f1(y2)}" x2="{f1(MAP_X0 + 22)}" y2="{f1(y2)}" stroke="{pal["muted"]}" stroke-width="2"/>')
    out.append(f'<text x="{f1(MAP_X0 + 28)}" y="{f1(y2 + 4)}" fill="{pal["muted"]}" font-size="12">MC vertical, ASC curve</text>')
    out.append(f'<line x1="{f1(MAP_X0 + 180)}" y1="{f1(y2)}" x2="{f1(MAP_X0 + 202)}" y2="{f1(y2)}" stroke="{pal["muted"]}" stroke-width="2" stroke-dasharray="6 4"/>')
    out.append(f'<text x="{f1(MAP_X0 + 208)}" y="{f1(y2 + 4)}" fill="{pal["muted"]}" font-size="12">IC vertical, DSC curve</text>')
    out.append(f'<circle cx="{f1(MAP_X0 + 362)}" cy="{f1(y2)}" r="3.2" fill="{pal["muted"]}"/>')
    out.append(f'<text x="{f1(MAP_X0 + 370)}" y="{f1(y2 + 4)}" fill="{pal["muted"]}" font-size="12">zenith</text>')
    out.append(f'<circle cx="{f1(MAP_X0 + 432)}" cy="{f1(y2)}" r="6" fill="none" stroke="{pal["muted"]}" stroke-width="1.8"/>')
    out.append(f'<text x="{f1(MAP_X0 + 444)}" y="{f1(y2 + 4)}" fill="{pal["muted"]}" font-size="12">paran (two bodies on angles at once)</text>')
    out.append(f'<text x="{f1(MAP_X0 + MAP_W)}" y="{f1(y2 + 4)}" fill="{pal["muted"]}" font-size="11" text-anchor="end">Equirectangular · coastline: Natural Earth (public domain)</text>')
    out.append("</g>")
    out.append("</svg>")
    return out, chosen


# --------------------------------------------------------------------------
# Accuracy bars (figures from docs/ACCURACY.md)
# --------------------------------------------------------------------------

# docs/ACCURACY.md, "Planetary Position Accuracy (JPL DE440 Cross-Validated)":
# the reproducible 20,000-chart bound vs JPL DE440 (Sun 0.21", Mercury-Saturn
# <= 0.76", Uranus 1.78", Neptune 2.53", Pluto ~3.2" in-window), and for the
# Moon the analytical series vs pyswisseph 2.10.03 over AD 1600-2100
# (RMS ~2.8", max ~12"). Change these only together with that document.
ACCURACY = [
    ("Sun", 0.21, "0.21″", "vs JPL DE440"),
    ("Mercury – Saturn", 0.76, "≤ 0.76″", "vs JPL DE440"),
    ("Uranus", 1.78, "1.78″", "vs JPL DE440"),
    ("Neptune", 2.53, "2.53″", "vs JPL DE440"),
    ("Pluto", 3.2, "~3.2″", "vs JPL DE440, in-window 1885–2099"),
    ("Moon", 12.0, "max ~12″, RMS ~2.8″", "vs pyswisseph, AD 1600–2100"),
]


def accuracy_chart(pal, dark):
    W = 800
    left, right = 170, 40
    top, row = 88, 46
    H = top + row * len(ACCURACY) + 92
    lo, hi = 0.1, 20.0
    span = math.log10(hi) - math.log10(lo)

    def xs(v):
        return left + (math.log10(v) - math.log10(lo)) / span * (W - left - right)

    out = svg_open(
        W, H,
        "Accuracy of the analytical engine",
        "Measured error per body in arcseconds on a log scale, from docs/ACCURACY.md: "
        "Sun 0.21, Mercury to Saturn at most 0.76, Uranus 1.78, Neptune 2.53, Pluto about 3.2 "
        "arcseconds vs JPL DE440; Moon max about 12, RMS about 2.8 arcseconds vs pyswisseph.",
    )
    out.append(f'<g font-family="{SANS}">')
    out.append(f'<text x="24" y="30" fill="{pal["fg"]}" font-size="18" font-weight="700">Accuracy with no data files (analytical engine)</text>')
    out.append(f'<text x="24" y="52" fill="{pal["muted"]}" font-size="13">Error in arcseconds, log scale. Load the DE440 kernel for sub-arcsecond Sun, Moon and planets.</text>')
    axis_bottom = top + row * len(ACCURACY) - 8
    for v in (0.1, 0.2, 0.5, 1, 2, 5, 10, 20):
        x = xs(v)
        out.append(f'<line x1="{f1(x)}" y1="{top - 8}" x2="{f1(x)}" y2="{axis_bottom}" stroke="{pal["faint"]}" stroke-width="1"/>')
        out.append(f'<text x="{f1(x)}" y="{axis_bottom + 18}" fill="{pal["muted"]}" font-size="12" text-anchor="middle">{v:g}″</text>')
    x1 = xs(1.0)
    out.append(f'<line x1="{f1(x1)}" y1="{top - 14}" x2="{f1(x1)}" y2="{axis_bottom}" stroke="{pal["fg"]}" stroke-width="1.5" stroke-dasharray="5 3"/>')
    out.append(f'<text x="{f1(x1 + 6)}" y="{top - 16}" fill="{pal["fg"]}" font-size="12" font-weight="700">1 arcsecond</text>')
    for i, (name, v, label, ref) in enumerate(ACCURACY):
        y = top + i * row
        moon = name == "Moon"
        fill = pal["muted"] if moon else pal["accent"]
        out.append(f'<text x="{left - 12}" y="{y + 15}" fill="{pal["fg"]}" font-size="14" font-weight="600" text-anchor="end">{escape(name)}</text>')
        out.append(f'<rect x="{left}" y="{y}" width="{f1(xs(v) - left)}" height="22" rx="3" fill="{fill}" opacity="{0.9}"/>')
        if xs(v) > W - right - 200:
            on_bar = "#0d1117" if dark else "#ffffff"
            out.append(f'<text x="{f1(xs(v) - 10)}" y="{y + 12}" fill="{on_bar}" font-size="13" font-weight="700" text-anchor="end" dominant-baseline="middle">{escape(label)}</text>')
        else:
            out.append(f'<text x="{f1(xs(v) + 8)}" y="{y + 12}" fill="{pal["fg"]}" font-size="13" font-weight="700" dominant-baseline="middle" paint-order="stroke" stroke="{pal["page"]}" stroke-width="4">{escape(label)}</text>')
        out.append(f'<text x="{left}" y="{y + 34}" fill="{pal["muted"]}" font-size="10.5">{escape(ref)}</text>')
    out.append(f'<text x="24" y="{H - 34}" fill="{pal["muted"]}" font-size="11.5">Source: docs/ACCURACY.md, Planetary Position Accuracy: the reproducible 20,000-chart figures vs JPL DE440</text>')
    out.append(f'<text x="24" y="{H - 16}" fill="{pal["muted"]}" font-size="11.5">(cargo test -p xalen-ephem --test accuracy_vs_de440); the Moon vs pyswisseph 2.10.03, AD 1600–2100.</text>')
    out.append("</g>")
    out.append("</svg>")
    return out


# --------------------------------------------------------------------------
# Terminal
# --------------------------------------------------------------------------

TERMINAL_ARGS = [
    "positions", "--date", BIRTH_DATE, "--tz", "+05:30", "--sidereal",
    "--bodies", "sun,moon,mercury,venus,mars,jupiter,saturn,rahu,ketu",
]


def terminal(xalen):
    text = run_xalen(xalen, TERMINAL_ARGS)
    body = text.rstrip("\n").split("\n")
    cmd = [
        ("$ ", "xalen positions --date " + BIRTH_DATE + " --tz +05:30 \\"),
        ("", "    --sidereal --bodies sun,moon,mercury,venus,mars,jupiter,saturn,rahu,ketu"),
    ]
    fs, lh, cw = 13, 19, 7.85
    longest = max([len(p + c) for p, c in cmd] + [len(line) for line in body])
    W = int(max(720, longest * cw + 48))
    H = int(44 + 22 + lh * (len(cmd) + 1 + len(body)) + 18)
    bg, bar, fg, dim, green = "#0d1117", "#161b22", "#e6edf3", "#9198a1", "#3fb950"
    out = svg_open(
        W, H,
        "xalen positions in a terminal",
        "Sidereal Lahiri positions with nakshatra and pada for 15 August 1947 00:00 IST, "
        "printed by the xalen command-line tool.",
    )
    out.append(f'<rect width="{W}" height="{H}" rx="10" fill="{bg}"/>')
    out.append(f'<path d="M10 0H{W - 10}A10 10 0 0 1 {W} 10V36H0V10A10 10 0 0 1 10 0Z" fill="{bar}"/>')
    for i, c in enumerate(("#f85149", "#d29922", "#3fb950")):
        out.append(f'<circle cx="{22 + 20 * i}" cy="18" r="6" fill="{c}" opacity="0.85"/>')
    out.append(f'<text x="{W / 2:.0f}" y="23" fill="{dim}" font-family="{SANS}" font-size="12" text-anchor="middle">xalen — offline, no data files</text>')
    out.append(f'<g font-family="{MONO}" font-size="{fs}" xml:space="preserve" style="white-space:pre">')
    y = 44 + 22
    for prompt, c in cmd:
        out.append(f'<text x="24" y="{y}"><tspan fill="{green}">{escape(prompt)}</tspan><tspan fill="{fg}">{escape(c)}</tspan></text>')
        y += lh
    y += lh
    for line in body:
        is_rule = line.strip() != "" and set(line.replace(" ", "")) == {"-"}
        out.append(f'<text x="24" y="{y}" fill="{dim if is_rule else fg}">{escape(line)}</text>')
        y += lh
    out.append("</g>")
    out.append("</svg>")
    return out, text


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--xalen", help="path to a built xalen binary (default: build it)")
    parser.add_argument("--coastline", help="local copy of ne_110m_coastline.geojson (default: download)")
    args = parser.parse_args()
    xalen = args.xalen or build_cli()
    os.makedirs(OUT, exist_ok=True)

    chart = json.loads(run_xalen(xalen, [
        "chart", "--date", BIRTH_DATE, "--place", BIRTH_PLACE, "--sidereal",
        "--house-system", HOUSE_SYSTEM, "--bodies", ",".join(CHART_BODIES), "--json",
    ]))
    write("chart-wheel.svg", chart_wheel(chart, LIGHT, False))
    write("chart-wheel-dark.svg", chart_wheel(chart, DARK, True))

    lines_geo = json.loads(run_xalen(xalen, [
        "map", "--date", BIRTH_DATE, "--bodies", ",".join(CLASSICAL), "--geojson", "--parans",
    ]))
    if args.coastline:
        with open(args.coastline, encoding="utf-8") as f:
            coast = json.load(f)
    else:
        with urllib.request.urlopen(COASTLINE_URL, timeout=60) as resp:
            coast = json.loads(resp.read().decode("utf-8"))
    # Leave room above the map for the rows of MC and IC labels.
    global MAP_Y0
    MAP_Y0 = 62.0 + 13.0 * meridian_labels(lines_geo)[1]
    coast_d = coastline_path(coast)
    delhi = (chart["place"]["longitude"], chart["place"]["latitude"])
    light_map, chosen = astro_map(lines_geo, coast_d, delhi, LIGHT, False)
    dark_map, _ = astro_map(lines_geo, coast_d, delhi, DARK, True)
    write("map.svg", light_map)
    write("map-dark.svg", dark_map)
    for f in chosen:
        p = f["properties"]
        lon, lat = f["geometry"]["coordinates"]
        print(f"paran: {p['body_a']} {p['kind_a']} x {p['body_b']} {p['kind_b']} at {lat:.2f}, {lon:.2f}")

    write("accuracy.svg", accuracy_chart(LIGHT, False))
    write("accuracy-dark.svg", accuracy_chart(DARK, True))

    term, text = terminal(xalen)
    write("terminal.svg", term)
    print("--- xalen " + " ".join(TERMINAL_ARGS))
    print(text)
    print("--- chart summary")
    print(json.dumps({k: chart[k] for k in ("date", "place", "ayanamsa", "house_system_name", "ascendant", "mc")}, indent=1))
    for b in chart["bodies"]:
        print(f"{b['id']:10s} {b['longitude']:9.4f} {b['sign']:12s} {b['degree_in_sign']:6.2f} house {b.get('house')}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
