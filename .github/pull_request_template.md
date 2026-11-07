<!-- Title: type(scope): subject, for example fix(houses): ... (see CONTRIBUTING.md) -->

## Summary

<!-- What changes and why, in a few sentences. -->

Closes #

## Verification

<!-- What you ran and what it printed (passed / failed counts). -->

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --exclude xalen-python -- -D warnings`
- [ ] `cargo test --workspace --exclude xalen-python`
- [ ] `cargo doc --no-deps --workspace --exclude xalen-python`
- [ ] or the complete gate, which runs all of the above plus the release-config tests, the attribution and commercial-clean policy checks, `cargo audit`, the DE440 cross-validation, the WASM, Python and Node binding builds and the accuracy harness: `bash scripts/gate.sh` (paste its summary)
- [ ] New or changed behaviour has a test; a fix has a test that fails without it
- [ ] Numeric changes are checked against a reference (Swiss Ephemeris, JPL DE440, IAU SOFA or the cited text) with the measured difference stated; no claim beyond what a committed test shows

## Breaking changes

<!-- None, or what callers must change. -->

## Docs updated

- [ ] rustdoc for new or changed public items
- [ ] `CHANGELOG.md` under `[Unreleased]`
- [ ] README or `docs/` where relevant (`scripts/regen-binding-ids.sh` if a binding ID changed)

<!-- Outside contributors: sign the CLA by posting the comment in CLA.md section 6 on this pull request. -->
