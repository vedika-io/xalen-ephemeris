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
| `src/main.rs` (`xalen-validation` bin crate) | Reads the oracle file, recomputes the same quantities with the pure-Rust XALEN crates, and prints per-body max / p99 / mean / RMS absolute delta (arcsec), the worst offender, the `<1″` subset fraction, and an overall PASS/FAIL against a configurable degree threshold. |

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
  `<1″` subset fraction. D must be finite, non-negative, and representable in
  arcseconds; zero requests exact agreement.

### Input integrity and exit status

The runner requires at least one complete chart in the format emitted by the
generator: all 14 supported bodies, all four supported ayanamsas, and exactly
12 house cusps. Unknown quantity names, invalid geographic coordinates,
non-finite values or computed results, malformed JSON, and file read errors
stop validation with a diagnostic. Blank lines are allowed. When `_meta` is
present, it must be on the first chart and its positive `n` must match the
number of charts read. This detects a truncated generator stream, even if it
ends between otherwise valid records. Without `_meta`, only the supplied
records can be checked; completeness against an intended count is unknown.

* Exit **0**: every evaluated gated sample is within the requested bound.
* Exit **1**: a valid comparison exceeds the bound.
* Exit **2**: invalid arguments, input, or a computation failure; no PASS is
  printed for the partial run.

An epoch outside a body's calculation coverage is an explicit exclusion, not
a computation failure. The default 1500–2400 span exceeds the analytic Pluto
series' coverage. The report lists these excluded counts, and refuses a
verdict if a body has no comparable samples at all. The per-quantity counts
therefore matter: PASS does not mean every body was compared at every epoch.
Other computation errors abort validation.

This validates the comparison evidence; it does not change the astronomical
calculations or the informational-only quantities. House comparisons retain
the existing `|latitude| <= 60°` limit, with exclusions now reported explicitly.

## Run by the gate

The `harness` stage of `scripts/gate.sh`, which the maintainers run on their
AWS CI for every pull request, runs this harness as the last stage: it installs
`pyswisseph` into a temporary directory, generates an oracle of N charts (seed
42), pipes it into `xalen-validation`, and fails the gate if any gated quantity
is more than 0.1 degree from the oracle. N defaults to 2000. For a larger run:

```bash
bash scripts/gate.sh --harness-n 20000
GATE_HARNESS_N=20000 bash scripts/gate.sh     # same thing
GATE_HARNESS_SEED=7 bash scripts/gate.sh      # a different sample
bash scripts/gate.sh --skip harness          # every other stage
```

The oracle is a run-time test dependency only; nothing from `pyswisseph` is
committed.

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

The runner prints, per quantity, `count`, `mean(″)`, `rms(″)`, `p99(″)` (read from a log-spaced histogram, so it can sit up to about 2.3% above the true percentile but never above the max), `max(″)`, the
`<1″%` subset, and the worst chart `(jd, lat, lon)`. A `<-- OVER` flag marks any
quantity with a sample beyond the 0.1° legacy bound. A valid run's final line
is `PASS` (exit 0) or `FAIL` (exit 1); invalid evidence exits 2 as described above.

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
