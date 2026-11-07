#!/usr/bin/env bash
# The full pre-merge gate in one script. Maintainers run it on their own
# AWS CI; contributors can run it locally. It works on any Linux or macOS host
# that has rustup, python3 with pip, and node.
#
# Steps, in order. A failing step does not stop the later ones, so one run shows
# every problem; the script exits non-zero if any step failed.
#   1  cargo fmt --all --check
#   2  cargo clippy --workspace --exclude xalen-python -- -D warnings
#   3  cargo test --workspace --exclude xalen-python
#   4  cargo doc --no-deps --workspace --exclude xalen-python
#   5  release-config tests: crates/xalen-python/tests/test_build_config.py
#      (pytest) and crates/xalen-node/__test__/platform-matrix.test.mjs (node)
#   6  validation accuracy harness: pyswisseph generates an oracle of N charts,
#      xalen-validation compares XALEN against it and fails past 0.1 deg
#
# The oracle (pyswisseph, AGPL) is a run-time test dependency only. It is
# installed into a private directory under /tmp for this run; nothing from it is
# committed or shipped.
#
# Usage:
#   bash scripts/gate.sh                     # harness at 2000 charts (default)
#   bash scripts/gate.sh --harness-n 20000   # larger harness run
#   GATE_HARNESS_N=20000 bash scripts/gate.sh
#
# Settings (environment; --harness-n overrides GATE_HARNESS_N):
#   GATE_HARNESS_N      charts for the harness (default 2000)
#   GATE_HARNESS_SEED   oracle seed (default 42)
#   GATE_TOOLS          directory for the Python tools (default: a fresh /tmp dir)
set -u -o pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT" || exit 2

N="${GATE_HARNESS_N:-2000}"
SEED="${GATE_HARNESS_SEED:-42}"
while [ $# -gt 0 ]; do
  case "$1" in
    --harness-n)
      N="${2:-}"
      shift 2 || { echo "gate: --harness-n needs a number" >&2; exit 2; }
      ;;
    -h|--help)
      sed -n 2,30p "$0"
      exit 0
      ;;
    *)
      echo "gate: unknown argument: $1" >&2
      exit 2
      ;;
  esac
done
case "$N" in
  ''|*[!0-9]*) echo "gate: harness N must be a positive integer, got: $N" >&2; exit 2 ;;
esac
TOOLS="${GATE_TOOLS:-$(mktemp -d /tmp/xalen-gate.XXXXXX)}"
mkdir -p "$TOOLS/py"

PASSED=()
FAILED=()

step() { # step "name" command...
  local name="$1"
  shift
  printf '\n==== STEP: %s ====\n' "$name"
  if "$@"; then
    PASSED+=("$name")
    printf '==== PASS: %s ====\n' "$name"
  else
    FAILED+=("$name")
    printf '==== FAIL: %s ====\n' "$name"
  fi
}

release_config_tests() {
  local rc=0
  python3 -m pip install --quiet --disable-pip-version-check --target "$TOOLS/py" pytest \
    || return 1
  PYTHONPATH="$TOOLS/py" python3 -m pytest -q \
    crates/xalen-python/tests/test_build_config.py || rc=1
  node --test crates/xalen-node/__test__/platform-matrix.test.mjs || rc=1
  return $rc
}

# Installs pyswisseph and prints "<python> <site-dir>" for running the oracle.
# pyswisseph ships wheels for few platforms (none for Linux aarch64), and
# building it from source needs Python.h. When the plain install fails, fetch a
# self-contained Python with headers through uv (into $TOOLS) and build against
# that instead.
oracle_python() {
  if python3 -m pip install --quiet --disable-pip-version-check --target "$TOOLS/py" \
      pyswisseph >&2; then
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

validation_harness() {
  local spec oracle_py oracle_path target_dir
  spec="$(oracle_python)" || return 1
  oracle_py="${spec%% *}"
  oracle_path="${spec#* }"
  cargo build -p xalen-validation --release --bin xalen-validation || return 1
  target_dir="$(cargo metadata --format-version 1 --no-deps \
    | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')" \
    || return 1
  echo "harness: N=$N seed=$SEED oracle python: $oracle_py"
  PYTHONPATH="$oracle_path" "$oracle_py" validation/oracle_pyswisseph.py --n "$N" --seed "$SEED" \
    | "$target_dir/release/xalen-validation" -
}

step "cargo fmt --check" cargo fmt --all --check
step "cargo clippy (-D warnings)" cargo clippy --workspace --exclude xalen-python -- -D warnings
step "cargo test" cargo test --workspace --exclude xalen-python
step "cargo doc" cargo doc --no-deps --workspace --exclude xalen-python
step "release-config tests (pytest + node)" release_config_tests
step "validation harness (N=$N)" validation_harness

printf '\n==== GATE SUMMARY ====\n'
printf 'passed: %d\n' "${#PASSED[@]}"
for s in "${PASSED[@]}"; do printf '  ok    %s\n' "$s"; done
printf 'failed: %d\n' "${#FAILED[@]}"
for s in "${FAILED[@]}"; do printf '  FAIL  %s\n' "$s"; done
if [ "${#FAILED[@]}" -ne 0 ]; then
  echo "GATE: FAIL"
  exit 1
fi
echo "GATE: PASS"
