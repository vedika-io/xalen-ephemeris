# XALEN reproducible accuracy validation

This directory makes the XALEN accuracy validation **reproducible from this
repository at any chart count**, using [`pyswisseph`](https://pypi.org/project/pyswisseph/)
(the Python bindings for the Swiss Ephemeris) as the oracle. It replaces the
older "5,000,000-chart sweep" that lived in a private monorepo and could not be
re-run from the public source.

Two parts:

| File | Role |
|------|------|
| `oracle_pyswisseph.py` | Deterministically samples N random `(jd, lat, lon)` charts and writes one Swiss-Ephemeris reference record per line (JSON Lines). |
| `src/main.rs` (`xalen-validation` bin crate) | Reads the oracle file, recomputes the same quantities with the pure-Rust XALEN crates, and prints per-body max / mean / RMS absolute delta (arcsec), the worst offender, the `<1″` subset fraction, and an overall PASS/FAIL against a configurable degree threshold. |

## Quantities compared, per chart

* **Bodies** — apparent geocentric ecliptic longitude **and** latitude of
  Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto,
  plus the mean node, true node, mean apogee and osculating apogee (Swiss
  bodies 0–13).
* **Houses** — tropical **Placidus** ascendant, midheaven and the twelve house
  cusps. (Latitudes are sampled in `±66°`, the band where Placidus is
  well-conditioned and Swiss does not degenerate it.)
* **Ayanamsa** — Lahiri, Fagan-Bradley, Krishnamurti and Raman (with-nutation
  apparent values).

## Reproduce at any N

```bash
# Small smoke run (a few seconds):
python3 validation/oracle_pyswisseph.py --n 50000 --seed 42 > oracle.jsonl
cargo run -p xalen-validation --release -- oracle.jsonl

# The full 5,000,000-chart sweep:
python3 validation/oracle_pyswisseph.py --n 5000000 --seed 42 > oracle.jsonl
cargo run -p xalen-validation --release -- oracle.jsonl
```

To avoid materializing a multi-GB intermediate file at very large N, pipe the
oracle straight into the runner (`-` reads stdin):

```bash
python3 validation/oracle_pyswisseph.py --n 5000000 --seed 42 \
  | cargo run -p xalen-validation --release -- -
```

Both the generator and the runner are fully deterministic for a fixed
`--seed`, so a run is reproducible bit-for-bit on any machine with the same
`pyswisseph` build.

### Options

`oracle_pyswisseph.py`:

* `--n N` — number of charts (default `100000`).
* `--seed S` — PRNG seed (default `42`).
* `--start-jd` / `--end-jd` — sampling span. Default is JD `2268923.5`
  (1500-01-01) to `2597641.5` (2400-01-01), a deliberately wide, multi-century
  span well outside the modern era.
* `--ephe-path DIR` — directory of Swiss `.se1` data files. If omitted (or if
  the files are absent) Swiss falls back to the built-in Moshier theory — see
  the honesty note below.
* `--out FILE` — write to a file instead of stdout.

`xalen-validation`:

* positional `ORACLE.jsonl` (or `-` for stdin).
* `--threshold-deg D` — per-quantity max-error bound for the PASS/FAIL verdict
  (default `0.1`, the legacy bound). The report also always shows the tighter
  `<1″` subset fraction.

## Backend honesty (Moshier vs DE440)

`pyswisseph` is **only as accurate as the data files it can find**. With no
JPL/Swiss `.se1` (or `.bsp`) files installed, `swe.calc_ut` silently falls back
to the analytic **Moshier** theory. In that case the deltas reported by the
runner are **XALEN (VSOP87A/ELP) vs Moshier** — two *independent* analytic
theories — and **not** XALEN vs the DE440 numerical integration.

The harness makes this explicit rather than hiding it:

* The Python oracle reads the Swiss return flag for every body and records the
  backend (`swieph` / `moshier` / `jpleph`) in the first record's `_meta`
  block, and prints a per-body backend summary to stderr.
* The Rust runner prints the backend line in its header and, if any body used
  the Moshier fallback, prints a `NOTE` stating the comparison is
  XALEN-vs-Moshier, not XALEN-vs-DE440.

On the development machine used here (pyswisseph **2.10.03**, no data files)
the observed backend was **Moshier for the planets, true node and osculating
apogee**, while the **mean node and mean apogee** are pure analytic formulae and
report the `swieph` flag regardless of data files. To run the comparison
against the genuine Swiss/DE data, install the Swiss `.se1` files (or the
DE440/441 `.bsp`) and pass `--ephe-path` to the generator; the backend line in
the report will then read `swieph`/`jpleph` and the NOTE disappears.

For the **DE440-grade** comparison that is independent of any external Swiss
install, XALEN ships its own committed Horizons-vector oracle in
`tests/swiss_eph_crossval.rs` and `crates/xalen-ephem/tests/` (run with
`cargo test`). This `validation/` harness is the complementary **statistical
sweep at arbitrary N**.

## Runtime and footprint

Measured generator throughput on the development machine: ~4,000 charts/second
(≈0.49 s for 2,000 charts), so:

| N | Generation time (approx) | `oracle.jsonl` size (approx) |
|---|--------------------------|------------------------------|
| 50,000 | ~13 s | ~60 MB |
| 100,000 | ~25 s | ~120 MB |
| 5,000,000 | ~20 min | ~5.9 GB |

Each record is ~1.2 KB. For the 5M run, prefer the streaming (`| ... -`)
invocation so the 5.9 GB never has to hit disk. The Rust runner is streaming
and O(1) in memory — it accumulates per-quantity statistics line by line and
never holds the whole file. Expect the runner to be generation-bound when
streamed; standalone it processes well over 100k charts/second.

## Interpreting the output

The runner prints, per quantity, `count`, `mean(″)`, `rms(″)`, `max(″)`, the
`<1″%` subset, and the worst chart `(jd, lat, lon)`. A `<-- OVER` flag marks any
quantity with a sample beyond the 0.1° legacy bound. The final line is `PASS`
(exit 0) or `FAIL` (exit 1).

What the rows mean, and the honest expectation with a Moshier oracle over the
default 1500–2400 span:

* **Body longitude** is the gating quantity. Over the modern era the planets
  agree with the Moshier oracle to the few-arcsec level; both analytic theories
  (XALEN's VSOP87A/ELP and the oracle's Moshier) diverge from DE in the far-past
  and far-future tails of the span, so the largest longitude deltas land there.
  The **Moon** shows the widest spread (its analytic series is the hardest case).
* **Body latitude** is **informational only** (marked `(info)`, never gating):
  XALEN models the lunar nodes and both apogees as ecliptic points (latitude = 0
  by construction), which cannot match the oracle's osculating-orbit latitude —
  a convention difference, not a position error.
* **Ayanamsa** agrees to well under an arcsec.
* **House cusps / asc / mc** agree to roughly 0.01° in the well-conditioned
  `±66°` latitude band the oracle samples.

The PASS/FAIL verdict and the exact per-body numbers depend on **N**, the **span**,
and which **backend** the local `pyswisseph` actually used (Moshier vs Swiss data
files). Run the harness to reproduce them for your configuration rather than
relying on a quoted figure; the runner prints the backend and the worst chart so
any `<-- OVER` row can be inspected directly. For a verdict scoped to the modern
era (where the analytic theories are tightest), narrow the span, e.g.
`--start-jd 2415021 --end-jd 2488070` (1900–2050).
