#!/usr/bin/env bash
# The complete pre-merge gate in one script. The maintainers run it on their own
# AWS CI and post the result on the pull request head as the `ci/gate` commit
# status; contributors can run it locally. It works on any Linux or macOS host
# that has rustup, python3 with pip, node with npm, curl, nm and strings.
#
# Stages, in order. A failing stage does not stop the later ones, so one run
# shows every problem. Each stage prints its name and OK, FAIL or SKIPPED (with
# the reason); the summary lists every stage, and the script exits non-zero if
# any stage failed.
#   fmt               cargo fmt --all --check
#   clippy            cargo clippy --workspace --exclude xalen-python -- -D warnings
#   test              cargo test --workspace --exclude xalen-python
#   doc               cargo doc --no-deps --workspace --exclude xalen-python
#   release-config    crates/xalen-python/tests/test_build_config.py (pytest) and
#                     crates/xalen-node/__test__/platform-matrix.test.mjs (node)
#   attribution       scripts/repo-policy/check-attribution.sh over the commits in
#                     <base>..HEAD (base: origin/main); skipped when that range is
#                     empty or the base is not in this checkout. With --pr-text
#                     FILE it also checks the pull request title and body.
#   commercial-clean  scripts/repo-policy/check-commercial-clean.sh: no published
#                     crate links the non-commercial Hipparcos data
#   audit             cargo audit (RustSec advisories)            [heavy]
#   de440             downloads the NASA/NAIF de440s.bsp kernel into the cache
#                     and runs the xalen-ephem DE440 cross-validation tests
#                     (de440_real_crossval, accuracy_vs_de440)    [heavy]
#   wasm              wasm-pack build --target web in crates/xalen-wasm [heavy]
#   python-binding    maturin build in crates/xalen-python        [heavy]
#   node-binding      npm ci, npm run build:debug and npm test in
#                     crates/xalen-node                           [heavy]
#   harness           validation accuracy harness: pyswisseph generates an oracle
#                     of N charts, xalen-validation compares XALEN against it and
#                     fails past 0.1 deg                          [heavy]
#
# Tools the gate needs and the host lacks (pytest, cargo-audit, wasm-pack,
# maturin, pyswisseph) are installed into a private tools directory; the host's
# Python and cargo installs are left untouched. The oracle (pyswisseph, AGPL) is
# a run-time test dependency only; nothing from it is committed or shipped.
#
# Usage:
#   bash scripts/gate.sh                     # every stage, harness at 2000 charts
#   bash scripts/gate.sh --quick             # fmt, clippy, test, doc, release-config,
#                                            # attribution, commercial-clean only
#   bash scripts/gate.sh --skip wasm --skip harness   # leave out named stages
#   bash scripts/gate.sh --skip wasm,harness          # same thing
#   bash scripts/gate.sh --harness-n 20000   # larger harness run
#   bash scripts/gate.sh --base upstream/main         # attribution range base
#   bash scripts/gate.sh --pr-text pr.txt    # also check a PR title and body
#   bash scripts/gate.sh --list              # print the stage names
#
# Settings (environment; the flags override them):
#   GATE_HARNESS_N      charts for the harness (default 2000)
#   GATE_HARNESS_SEED   oracle seed (default 42)
#   GATE_BASE           base revision for the attribution range (default origin/main)
#   GATE_TOOLS          directory for installed tools (default: a fresh temp dir;
#                       set it to a fixed path to reuse the tools between runs)
#   GATE_CACHE          download cache for the DE440 kernel
#                       (default: ${XDG_CACHE_HOME:-$HOME/.cache}/xalen-gate)
set -u -o pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT" || exit 2

STAGES=(fmt clippy test doc release-config attribution commercial-clean
  audit de440 wasm python-binding node-binding harness)
HEAVY=" audit de440 wasm python-binding node-binding harness "

usage() {
  awk 'NR == 1 { next } /^#/ { sub(/^# ?/, ""); print; next } { exit }' "$0"
}

N="${GATE_HARNESS_N:-2000}"
SEED="${GATE_HARNESS_SEED:-42}"
BASE="${GATE_BASE:-origin/main}"
PR_TEXT=""
QUICK=0
SKIP=" "
while [ $# -gt 0 ]; do
  case "$1" in
    --harness-n)
      [ $# -ge 2 ] || { echo "gate: --harness-n needs a number" >&2; exit 2; }
      N="$2"
      shift 2
      ;;
    --skip)
      [ $# -ge 2 ] || { echo "gate: --skip needs a stage name" >&2; exit 2; }
      for s in ${2//,/ }; do
        case " ${STAGES[*]} " in
          *" $s "*) SKIP="$SKIP$s " ;;
          *) echo "gate: unknown stage: $s (stages: ${STAGES[*]})" >&2; exit 2 ;;
        esac
      done
      shift 2
      ;;
    --quick)
      QUICK=1
      shift
      ;;
    --base)
      [ $# -ge 2 ] || { echo "gate: --base needs a revision" >&2; exit 2; }
      BASE="$2"
      shift 2
      ;;
    --pr-text)
      [ $# -ge 2 ] || { echo "gate: --pr-text needs a file" >&2; exit 2; }
      PR_TEXT="$2"
      [ -r "$PR_TEXT" ] || { echo "gate: cannot read $PR_TEXT" >&2; exit 2; }
      shift 2
      ;;
    --list)
      printf '%s\n' "${STAGES[@]}"
      exit 0
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "gate: unknown argument: $1 (see --help)" >&2
      exit 2
      ;;
  esac
done
case "$N" in
  ''|*[!0-9]*) echo "gate: harness N must be a positive integer, got: $N" >&2; exit 2 ;;
esac

TOOLS="${GATE_TOOLS:-$(mktemp -d "${TMPDIR:-/tmp}/xalen-gate.XXXXXX")}"
CACHE="${GATE_CACHE:-${XDG_CACHE_HOME:-$HOME/.cache}/xalen-gate}"
mkdir -p "$TOOLS/py" "$TOOLS/cargo/bin" "$CACHE" || exit 2
export PATH="$TOOLS/cargo/bin:$TOOLS/py/bin:$PATH"

# --- helpers ------------------------------------------------------------------

# A stage function calls `skipped "reason"` and returns 0 when it cannot run on
# this host or has nothing to check; the stage is then reported as SKIPPED.
SKIP_REASON=""
skipped() { SKIP_REASON="$*"; echo "skipped: $*"; }

pip_tools() { # pip_tools <package>... installs into $TOOLS/py
  python3 -m pip install --quiet --disable-pip-version-check --target "$TOOLS/py" "$@"
}

cargo_tool() { # cargo_tool <binary> <crate> installs a cargo tool unless present
  local bin="$1" crate="$2"
  command -v "$bin" >/dev/null 2>&1 && return 0
  echo "installing $crate into $TOOLS/cargo"
  if ! cargo install "$crate" --locked --root "$TOOLS/cargo" >"$TOOLS/$crate-install.log" 2>&1; then
    tail -n 30 "$TOOLS/$crate-install.log" >&2
    echo "cargo install $crate failed" >&2
    return 1
  fi
}

target_dir() {
  cargo metadata --format-version 1 --no-deps \
    | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])'
}

# --- stages -------------------------------------------------------------------

stage_fmt() { cargo fmt --all --check; }
stage_clippy() { cargo clippy --workspace --exclude xalen-python -- -D warnings; }
stage_test() { cargo test --workspace --exclude xalen-python; }
stage_doc() { cargo doc --no-deps --workspace --exclude xalen-python; }

stage_release_config() {
  local rc=0
  pip_tools pytest || return 1
  PYTHONPATH="$TOOLS/py" python3 -m pytest -q \
    crates/xalen-python/tests/test_build_config.py || rc=1
  node --test crates/xalen-node/__test__/platform-matrix.test.mjs || rc=1
  return $rc
}

stage_attribution() {
  local rc=0 count
  if [ -n "$PR_TEXT" ]; then
    echo "checking the pull request title and body in $PR_TEXT"
    scripts/repo-policy/check-attribution.sh --stdin <"$PR_TEXT" || rc=1
  fi
  if ! git rev-parse --verify --quiet HEAD >/dev/null 2>&1; then
    [ -n "$PR_TEXT" ] && return $rc
    skipped "no git history in this checkout"
    return 0
  fi
  if ! git rev-parse --verify --quiet "$BASE^{commit}" >/dev/null 2>&1; then
    [ -n "$PR_TEXT" ] && { echo "base $BASE not found; commit messages not checked"; return $rc; }
    skipped "base $BASE is not in this checkout (fetch it or pass --base <rev>)"
    return 0
  fi
  count="$(git rev-list --count --no-merges "$BASE..HEAD")" || return 1
  if [ "$count" -eq 0 ]; then
    [ -n "$PR_TEXT" ] && { echo "no commits in $BASE..HEAD"; return $rc; }
    skipped "no commits in $BASE..HEAD"
    return 0
  fi
  echo "checking $count commit message(s) in $BASE..HEAD"
  scripts/repo-policy/check-attribution.sh --range "$BASE..HEAD" || rc=1
  return $rc
}

stage_commercial_clean() { bash scripts/repo-policy/check-commercial-clean.sh; }

stage_audit() {
  cargo_tool cargo-audit cargo-audit || return 1
  cargo audit --version || return 1
  # Cargo.lock is not committed; the earlier stages normally leave one behind.
  [ -f Cargo.lock ] || cargo generate-lockfile || return 1
  cargo audit
}

# The NASA/NAIF de440s.bsp kernel (1550-2650 CE, about 31 MB). The tests read it
# from /tmp/de440s.bsp; the download is kept in $CACHE and linked there.
DE440S_URL="https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440s.bsp"
DE440S_LINK="/tmp/de440s.bsp"
DE440S_MIN_BYTES=30000000
stage_de440() {
  local kernel="$CACHE/de440s.bsp" size log="$TOOLS/de440.log" rc
  if [ ! -s "$kernel" ]; then
    echo "downloading $DE440S_URL"
    curl -sSL --fail --retry 3 -o "$kernel.part" "$DE440S_URL" || return 1
    mv "$kernel.part" "$kernel" || return 1
  fi
  if [ -L "$DE440S_LINK" ] || [ ! -e "$DE440S_LINK" ]; then
    ln -sf "$kernel" "$DE440S_LINK" || return 1
  else
    echo "using the existing $DE440S_LINK"
  fi
  size="$(wc -c <"$DE440S_LINK" | tr -d ' ')"
  echo "kernel: $DE440S_LINK ($size bytes)"
  [ "$size" -ge "$DE440S_MIN_BYTES" ] || { echo "kernel is too small; delete $kernel and rerun" >&2; return 1; }
  cargo test -p xalen-ephem --test de440_real_crossval --test accuracy_vs_de440 -- --nocapture 2>&1 \
    | tee "$log"
  rc="${PIPESTATUS[0]}"
  [ "$rc" -eq 0 ] || return 1
  # The tests pass by skipping when they cannot read the kernel; that is a failure here.
  if grep -q "SKIP: $DE440S_LINK not found" "$log"; then
    echo "the DE440 tests did not find the kernel" >&2
    return 1
  fi
}

stage_wasm() {
  if command -v rustup >/dev/null 2>&1; then
    rustup target add wasm32-unknown-unknown || return 1
  fi
  cargo_tool wasm-pack wasm-pack || return 1
  wasm-pack --version || return 1
  ( cd crates/xalen-wasm && wasm-pack build --target web )
}

stage_python_binding() {
  # maturin gets its own directory: pip --target does not add files to a bin/
  # that an earlier install (pytest) already created there.
  if ! command -v maturin >/dev/null 2>&1; then
    python3 -m pip install --quiet --disable-pip-version-check --target "$TOOLS/maturin" maturin \
      || return 1
    export PATH="$TOOLS/maturin/bin:$PATH"
  fi
  maturin --version || return 1
  # Run from the crate directory so maturin reads its pyproject.toml, which
  # enables the extension-module feature for the wheel build.
  ( cd crates/xalen-python && maturin build )
}

stage_node_binding() {
  # crates/xalen-node has a committed package-lock.json, so `npm ci` installs
  # exactly the locked @napi-rs/cli. build:debug is `napi build --platform`; the
  # tests load the freshly built .node addon.
  ( cd crates/xalen-node && npm ci && npm run build:debug && npm test )
}

# Installs pyswisseph and prints "<python> <site-dir>" for running the oracle.
# pyswisseph ships wheels for few platforms (none for Linux aarch64), and
# building it from source needs Python.h. When the plain install fails, fetch a
# self-contained Python with headers through uv (into $TOOLS) and build against
# that instead.
oracle_python() {
  if pip_tools pyswisseph >&2; then
    echo "python3 $TOOLS/py"
    return 0
  fi
  echo "gate: no pyswisseph wheel and no Python.h; using a uv-managed Python" >&2
  python3 -m pip install --quiet --disable-pip-version-check --target "$TOOLS/uv" uv >&2 \
    || return 1
  PYTHONPATH="$TOOLS/uv" python3 -m uv python install 3.12 --install-dir "$TOOLS/pys" >&2 \
    || return 1
  local py
  py="$(find "$TOOLS/pys" -maxdepth 4 -path '*/bin/python3.12' | head -n 1)"
  [ -x "$py" ] || return 1
  "$py" -m pip install --quiet --disable-pip-version-check --target "$TOOLS/py2" \
    pyswisseph >&2 || return 1
  echo "$py $TOOLS/py2"
}

stage_harness() {
  local spec oracle_py oracle_path tdir
  spec="$(oracle_python)" || return 1
  oracle_py="${spec%% *}"
  oracle_path="${spec#* }"
  cargo build -p xalen-validation --release --bin xalen-validation || return 1
  tdir="$(target_dir)" || return 1
  echo "harness: N=$N seed=$SEED oracle python: $oracle_py"
  PYTHONPATH="$oracle_path" "$oracle_py" validation/oracle_pyswisseph.py --n "$N" --seed "$SEED" \
    | "$tdir/release/xalen-validation" -
}

# --- run ----------------------------------------------------------------------

RESULTS=()
NFAIL=0
record() { # record <stage> <OK|FAIL|SKIPPED> [note]
  local line
  line="$(printf '%-8s %s' "$2" "$1")"
  [ -n "${3:-}" ] && line="$line ($3)"
  RESULTS+=("$line")
  [ "$2" = FAIL ] && NFAIL=$((NFAIL + 1))
  printf '==== %s: %s%s ====\n' "$1" "$2" "${3:+ ($3)}"
}

run_stage() {
  local name="$1" fn="stage_${1//-/_}"
  printf '\n==== STAGE: %s ====\n' "$name"
  case "$SKIP" in *" $name "*) record "$name" SKIPPED "--skip"; return ;; esac
  if [ "$QUICK" -eq 1 ]; then
    case "$HEAVY" in *" $name "*) record "$name" SKIPPED "--quick"; return ;; esac
  fi
  SKIP_REASON=""
  if "$fn"; then
    if [ -n "$SKIP_REASON" ]; then record "$name" SKIPPED "$SKIP_REASON"; else record "$name" OK; fi
  else
    record "$name" FAIL
  fi
}

for stage in "${STAGES[@]}"; do
  run_stage "$stage"
done

printf '\n==== GATE SUMMARY ====\n'
printf '  %s\n' "${RESULTS[@]}"
if [ "$NFAIL" -ne 0 ]; then
  echo "GATE: FAIL ($NFAIL stage(s) failed)"
  exit 1
fi
echo "GATE: PASS"
