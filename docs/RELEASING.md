# Releasing XALEN Ephemeris

A release is cut by a maintainer from the maintainers' AWS CI (any Linux host,
x86_64 or aarch64, with rustup, python3 with pip, node with npm and `file`
works the same way). Nothing in this repository publishes automatically: the
scripts build and check the artifacts, and the publish commands below are run
by hand with the maintainer's own registry credentials.

`X.Y.Z` below stands for the new version.

## 1. Prepare the release commit

1. Start from an up-to-date `main` on a release branch, for example
   `chore/release-X.Y.Z`.
2. Bump the version everywhere it is pinned:
   - `version` under `[workspace.package]` in the root `Cargo.toml`;
   - every intra-workspace dependency pin (`xalen-… = { version = "…", path =
     "…" }`) in the root `Cargo.toml`, `crates/*/Cargo.toml` and
     `validation/Cargo.toml`;
   - `version` in `crates/xalen-node/package.json` and
     `crates/xalen-wasm/package.json`, and the root package entries in
     `crates/xalen-node/package-lock.json`.

   `git grep -n '<old version>'` lists every place; the Python package takes
   its version from Cargo (`dynamic = ["version"]` in
   `crates/xalen-python/pyproject.toml`).
3. Regenerate the Node loader and types, which carry the version:
   `cd crates/xalen-node && npm ci && npx napi build --platform --release`, then
   commit the changed `index.js` and `index.d.ts`. The release build in section 2
   fails if the committed copies differ from what napi generates.
4. In `CHANGELOG.md`, rename `## [Unreleased]` to `## [X.Y.Z] - YYYY-MM-DD`
   (the release date) and add a new, empty `## [Unreleased]` above it.
5. Run the full gate on the release commit and open the release pull request:

   ```bash
   bash scripts/gate.sh
   ```

   Merge it once the `ci/gate` status is green.

## 2. Build and check the artifacts

On the merged release commit:

```bash
bash scripts/build-release-artifacts.sh
```

It writes, into `dist/`:

- the Python abi3 wheels (CPython 3.8 and newer) for Linux x86_64 and aarch64
  (manylinux2014, glibc 2.17) and macOS arm64 (11.0) and x86_64 (10.12);
- the Python source distribution `xalen-X.Y.Z.tar.gz`;
- `xalen-wasm-X.Y.Z.tgz`, the npm tarball of `@xalen/wasm`;
- `xalen-ephemeris-X.Y.Z.tgz`, the npm tarball of `@xalen/ephemeris` (the
  loader, no binary), and one tarball per prebuilt Node addon:
  `xalen-ephemeris-{linux-x64-gnu,linux-arm64-gnu,darwin-arm64,darwin-x64,win32-x64-msvc}-X.Y.Z.tgz`;
- `SHA256SUMS`.

The script lists what it checks in its header (the host-architecture wheel and
the Python test suite against it, the macOS `__LINKEDIT` alignment, the sdist
free of the non-commercial star add-on, the WebAssembly package in Node, the
Node addons' glibc floor, Windows DLL imports and the Node smoke test). It exits
non-zero if any check failed; do not publish from a failed run.

Wheels and addons for other operating systems cannot run on the build host.
Before uploading, import each macOS wheel on a Mac and run the Node smoke test
with the macOS and Windows addon tarballs on those systems (see the list at the
end).

`scripts/build-all-platforms.sh` builds the same matrix on a developer machine
for local checks; the published artifacts come from
`scripts/build-release-artifacts.sh`.

## 3. Tag

```bash
git tag -a vX.Y.Z -m "vX.Y.Z"
git push origin vX.Y.Z
```

## 4. Publish

Credentials come from the environment and are never written into the
repository: `CARGO_REGISTRY_TOKEN` for crates.io, `MATURIN_PYPI_TOKEN` for PyPI
and `NODE_AUTH_TOKEN` for npm, read through a user-level `~/.npmrc` line
`//registry.npmjs.org/:_authToken=${NODE_AUTH_TOKEN}`.

### crates.io

Publish the source crates in dependency order: every crate's intra-workspace
dependencies come before it (`ephemeris` is the umbrella crate at the
repository root, and `cli` depends on it). `cargo publish` waits until each
crate is in the index before it returns, so the next one can resolve it.
`crates/xalen-python/tests/test_build_config.py` checks this order against the
manifests.

```bash
order=(time coords star-anchors ayanamsa houses chart chinese stars vedic ephem cloud western relationship maps numerology lalkitab iching world ephemeris ffi cli)
for c in "${order[@]}"; do
  cargo publish -p "xalen-$c" --no-verify
done
```

`xalen-stars-hip-data` (non-commercial data), `xalen-python`, `xalen-node`,
`xalen-wasm` and `xalen-validation` are not published to crates.io.

### PyPI

```bash
python3 -m pip install --upgrade maturin
maturin upload --skip-existing dist/xalen-X.Y.Z-*.whl dist/xalen-X.Y.Z.tar.gz
```

(`python3 -m twine upload dist/xalen-X.Y.Z-*.whl dist/xalen-X.Y.Z.tar.gz` with
`TWINE_USERNAME=__token__` and `TWINE_PASSWORD` set to the PyPI token works the
same way.)

### npm: Node addon

Publish the five platform packages first, then the root package that names them
as `optionalDependencies`:

```bash
for p in linux-x64-gnu linux-arm64-gnu darwin-arm64 darwin-x64 win32-x64-msvc; do
  npm publish --access public "dist/xalen-ephemeris-$p-X.Y.Z.tgz"
done
npm publish --access public dist/xalen-ephemeris-X.Y.Z.tgz
```

### npm: WebAssembly

```bash
npm publish --access public dist/xalen-wasm-X.Y.Z.tgz
```

## 5. After publishing

Check that each registry shows the new version (`cargo search xalen-ephemeris`,
`pip index versions xalen`, `npm view @xalen/ephemeris version`,
`npm view @xalen/wasm version`), then install each package once in a clean
environment and run a one-line smoke call.

## Not yet scripted

The former GitHub Actions release workflow did the following, and no script in
this repository covers it yet. Until one does, each item is either done by hand
as described or not done:

- **Windows Python wheel** (`x86_64-pc-windows-msvc`): no script builds it.
  Windows users install from the sdist, which needs a Rust toolchain. The Node
  addon for Windows x64 is built.
- **Native smoke tests on macOS and Windows**: the workflow installed and tested
  the macOS arm64 and Windows x64 wheels and ran `npm test` against the macOS
  arm64 and Windows x64 Node addons on those systems. The build host runs only
  its own architecture's wheel and addon; the others are checked by hand
  before upload (section 2).
- **The other Linux architecture**: the workflow tested the Linux x86_64 wheel
  and addon natively. The build host tests only its own architecture, so on an
  aarch64 host the x86_64 wheel and addon are built and inspected but not run.
- **Publishing**: the crates.io, PyPI and npm uploads in step 4 are manual
  commands, not a script.
- **GitHub Packages mirror**: the workflow also published the WebAssembly
  package to the repository's GitHub Packages npm registry, so that it shows in
  the repository's Packages panel. GitHub Packages requires the npm scope to
  match the repository owner, so the mirror is `@vedika-io/xalen-wasm`; npmjs.org
  (`@xalen/wasm`) stays the canonical home. The mirror is kept, by hand, with
  the same steps the workflow ran, applied to the released tarball. It needs a
  GitHub token with the `write:packages` scope in `GITHUB_PACKAGES_TOKEN`, read
  through a user-level `~/.npmrc` line
  `//npm.pkg.github.com/:_authToken=${GITHUB_PACKAGES_TOKEN}`:

  ```bash
  mirror="$(mktemp -d)"
  tar -xzf dist/xalen-wasm-X.Y.Z.tgz -C "$mirror"
  cd "$mirror/package"
  npm pkg set name="@vedika-io/xalen-wasm"
  npm pkg set publishConfig.registry="https://npm.pkg.github.com"
  npm publish
  ```
