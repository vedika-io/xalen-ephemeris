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

## Before opening a PR

```bash
cargo fmt --all --check
cargo clippy --workspace --exclude xalen-python -- -D warnings
cargo test  --workspace --exclude xalen-python
cargo doc   --no-deps --workspace --exclude xalen-python
```

All four must pass. CI runs the same plus `cargo audit` (RustSec advisories) and
the WASM / Python / Node binding builds.

## Project layout

- Core: `xalen-time`, `xalen-coords`, `xalen-ephem`, `xalen-houses`,
  `xalen-ayanamsa`, `xalen-stars`
- Systems: `xalen-vedic`, `xalen-western`, `xalen-chinese`, `xalen-numerology`,
  `xalen-iching`, `xalen-world`, `xalen-lalkitab`
- Output / bindings: `xalen-chart`, `xalen-ffi`, `xalen-wasm`,
  `xalen-python` (→ PyPI), `xalen-node` (→ npm)

Cross-validation tests live in `crates/xalen-ephem/tests/`; the accuracy report
is `docs/ACCURACY.md`.

## Commit / PR conventions

- Conventional-commit prefixes (`feat:`, `fix:`, `test:`, `docs:`, `ci:`).
- Keep PRs focused; include the reference/source for any numeric change.
- New public items need rustdoc (`///`) comments.

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
