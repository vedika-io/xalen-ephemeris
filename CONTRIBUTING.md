# Contributing to XALEN Ephemeris

Thanks for your interest in contributing. XALEN is a pure-Rust astronomical and
astrological computation library; correctness and accuracy are the priorities.

## Ground rules

- **Accuracy is verifiable, never asserted.** Any change to a position, house,
  ayanamsa, dasha, or other computed value must be backed by a test that
  cross-checks against an external reference (JPL Horizons / DE440, Swiss
  Ephemeris `swetest`, or a cited classical table). Do not change a constant or
  formula without a reference and a test.
- **No panics in library code.** Return `Result` (or `Option`) and validate
  inputs; do not `unwrap()`/`panic!()` on data that can come from a caller.
- **Cite your sources.** New algorithms must reference the theory or classical
  text they implement (add it to `CREDITS.md`).

## Toolchain

- Rust **1.85+** (edition 2024). Pinned MSRV: `rust-version = "1.85"`.
- `rustup component add clippy rustfmt`

## Project layout

- Core: `xalen-time`, `xalen-coords`, `xalen-ephem`, `xalen-houses`,
  `xalen-ayanamsa`, `xalen-stars`
- Systems: `xalen-vedic`, `xalen-western`, `xalen-chinese`, `xalen-numerology`,
  `xalen-iching`, `xalen-world`, `xalen-lalkitab`
- Output / bindings: `xalen-chart`, `xalen-ffi`, `xalen-wasm`,
  `xalen-python` (→ PyPI), `xalen-node` (→ npm)

Cross-validation tests live in `crates/xalen-ephem/tests/`; the accuracy report
is `docs/ACCURACY.md`. The numeric IDs the bindings take are listed in
`docs/BINDING_IDS.md`, which `tests/binding_ids.rs` keeps in step with the
source.

## From issue to merged pull request

### 1. Open an issue

Search the [issues](https://github.com/vedika-io/xalen-ephemeris/issues)
first; if yours is new, open one with the matching form (bug report, feature
request or documentation). For a wrong value, give the exact input (date,
time and time zone, place, ayanamsa or house system), the value XALEN returns,
the value you expected and where that value comes from (Swiss Ephemeris
`swetest`, JPL Horizons, or the classical text and page). Security problems go
to the private channel in [SECURITY.md](SECURITY.md), not to an issue.

For anything larger than a small fix, wait until a maintainer agrees on the
approach in the issue before you start, so the work is not wasted.

### 2. Branch

Branch from the current `main` with a prefix that matches the change and a
short name in lower case with hyphens:

- `feat/` for a new feature, for example `feat/cli-maps`
- `fix/` for a bug fix
- `docs/` for documentation only
- `test/`, `ci/`, `chore/`, `refactor/`, `perf/` for the rest

### 3. Commit

Write each commit message as `type(scope): subject`, then a blank line and a
body:

- `type` is one of `feat`, `fix`, `docs`, `test`, `ci`, `chore`, `refactor`
  or `perf`.
- `scope` is the crate or area without the `xalen-` prefix (`vedic`, `houses`,
  `node`, `cli`); leave it out for a change across the whole workspace.
- Put `!` after the scope for a breaking change (`fix(vedic)!: ...`) and say in
  the body what callers must change.
- The subject is in the imperative, lower case, without a full stop.
- The body says what changed and why, names the reference behind any numeric
  change, and says how you checked it. Wrap it at about 72 columns.
- End the body with `Refs #N` for the issue the commit belongs to.

```text
fix(houses): put sidereal Whole Sign cusps on sign boundaries

The cusps were the tropical sign boundaries shifted by the ayanamsa, so a
planet near the ascendant could land in the wrong house. They are now built
from the sidereal ascendant, as Swiss Ephemeris does; checked against
swe_houses_ex with SEFLG_SIDEREAL for Lahiri and Fagan-Bradley.

Refs #123
```

No AI provider, AI coding tool or AI co-author attribution in commits or pull
requests: no `Co-authored-by:` trailers for AI tools or bots, no "Generated
with …" footers, no tool links or bot author addresses. The committer is the
author of record. `scripts/repo-policy/check-attribution.sh` enforces this in
the `commit-msg` hook (enable once per clone with
`git config core.hooksPath .githooks`) and in the `attribution` workflow.

### 4. Check

Run the four checks CI runs on every pull request. All four must pass:

```bash
cargo fmt --all --check
cargo clippy --workspace --exclude xalen-python -- -D warnings
cargo test  --workspace --exclude xalen-python
cargo doc   --no-deps --workspace --exclude xalen-python
```

`scripts/gate.sh` runs all of these in one go, plus the release-configuration
tests (`crates/xalen-python/tests/test_build_config.py` and
`crates/xalen-node/__test__/platform-matrix.test.mjs`) and the accuracy harness
in `validation/`. It prints each step, then a summary, and exits non-zero if
any step failed. The harness runs on 2000 charts by default; for a larger run
use `bash scripts/gate.sh --harness-n 20000` or `GATE_HARNESS_N=20000 bash
scripts/gate.sh`. It installs `pyswisseph`, the Swiss Ephemeris bindings it
compares against, into a temporary directory for the run; that is a test-time
tool only and nothing from it is committed or shipped.

CI runs on the maintainers' own AWS infrastructure. Every pull request receives
a `ci/gate` commit status once the maintainers have run `scripts/gate.sh`
against its head; contributors can run the same script locally or in their
fork's CI.

Clippy runs with warnings as errors. CI also runs the tests on Linux, macOS
and Windows, `cargo audit` (RustSec advisories), and the WebAssembly, Python
and Node binding builds.

Every change in behaviour comes with a test, and a bug fix with a test that
fails without the fix. New public items need rustdoc (`///`) comments. Add a
line to `CHANGELOG.md` under `[Unreleased]`, and update the README or the
files in `docs/` your change affects. If you change a binding ID, run
`scripts/regen-binding-ids.sh` and commit the updated `docs/BINDING_IDS.md`.

### 5. Open the pull request

Open one pull request per change; split unrelated changes. Give it a title in
the same `type(scope): subject` form and fill in the
[template](.github/pull_request_template.md): a summary, `Closes #N` for the
issue it resolves (GitHub then closes the issue on merge), what you ran and
the results, any breaking change, and the documentation you updated.

If you are not a maintainer or employee, sign the Contributor License
Agreement on your first pull request by posting the comment in
[CLA.md §6](CLA.md); see [below](#license-and-contributor-license-agreement-cla).
A pull request without a signed CLA cannot be merged.

### 6. Review and merge

A maintainer reviews the pull request once CI is green. Answer review comments
with new commits on the same branch; you do not need to rebase or force-push.
Maintainers squash-merge: the pull request becomes one commit on `main` whose
message is the pull request title with its number, for example
`fix(vedic)!: compute Cheshta Bala from the Cheshta Kendra per BPHS (#69)`.

## License and Contributor License Agreement (CLA)

XALEN Ephemeris is **dual-licensed**: the open-source [Apache-2.0](LICENSE)
license *and* a separate paid [commercial license](COMMERCIAL_LICENSE.md)
offered by XALEN Technology Pvt Ltd. For the commercial option to remain
possible, the maintainer must hold the right to distribute **every** merged
contribution under *both* licenses.

A DCO `Signed-off-by` line is **not sufficient** on its own. The DCO only
certifies that you wrote the change and may submit it under the project's
open-source license — it does **not** grant the right to relicense your
contribution under the commercial license.

Therefore, before an outside (non-employee) contribution can be merged, the
contributor must sign the project's **Contributor License Agreement (CLA)**. The
CLA grants XALEN Technology Pvt Ltd a copyright license to use, modify, and
sublicense the contribution under both the Apache-2.0 and the commercial
license. You keep the copyright in your contribution; the CLA only grants the
licensing rights the dual-license model requires.

The full CLA text is in **[CLA.md](CLA.md)**, together with the signing flow:
post the signing comment from [CLA.md §6](CLA.md) on your first pull request,
and a maintainer records it before merge. Pull requests from external
contributors who have not signed the CLA **must not be merged**.
