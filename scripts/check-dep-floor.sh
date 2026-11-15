#!/usr/bin/env bash
# Dependency floor check (NEW-AUDIT-U1-16-1). Fails when pyo3, sha2 or ureq sit
# below the floor in the manifests, or when the workspace Cargo.lock (which
# covers the cdylib crates xalen-ffi, xalen-python, xalen-wasm and xalen-node)
# is missing from git or pins an older version. Pure text checks, no compile.
# Usage: bash scripts/check-dep-floor.sh   (run from anywhere inside the repo)
set -u
cd "$(git rev-parse --show-toplevel)" || exit 2
FLOOR_PYO3=0.30 FLOOR_SHA2=0.11 FLOOR_UREQ=3
fail=0
ge() { [ "$(printf '%s\n%s\n' "$2" "$1" | sort -V | head -n1)" = "$2" ]; }  # $1 >= $2
bad() { echo "FAIL: $*"; fail=1; }

check_manifest() { # crate-name floor manifest...
  local dep=$1 floor=$2; shift 2
  local f v
  for f in "$@"; do
    v=$(grep -E "^$dep *= *[{\"]" "$f" | grep -oE 'version *= *"[0-9.]+"|= *"[0-9.]+"$' | grep -oE '[0-9][0-9.]*' | head -n1)
    [ -n "$v" ] || { bad "$f: no version found for $dep"; continue; }
    ge "$v" "$floor" && echo "ok:   $f $dep $v >= $floor" || bad "$f: $dep $v is below floor $floor"
  done
}
check_manifest pyo3 "$FLOOR_PYO3" crates/xalen-python/Cargo.toml
check_manifest sha2 "$FLOOR_SHA2" crates/xalen-ephem/Cargo.toml
check_manifest ureq "$FLOOR_UREQ" crates/xalen-ephem/Cargo.toml crates/xalen-cloud/Cargo.toml

if ! git ls-files --error-unmatch Cargo.lock >/dev/null 2>&1; then
  bad "Cargo.lock is not committed (cdylib and node crates build without a pinned lock)"
elif git check-ignore -q Cargo.lock 2>/dev/null; then
  bad "Cargo.lock is committed but still listed in .gitignore"
else
  echo "ok:   Cargo.lock is tracked"
  lock_ver() { awk -v n="$1" '$0=="name = \""n"\""{getline; gsub(/[^0-9.]/,"",$0); print; exit}' Cargo.lock; }
  for pair in "pyo3 $FLOOR_PYO3" "sha2 $FLOOR_SHA2" "ureq $FLOOR_UREQ"; do
    set -- $pair; v=$(lock_ver "$1")
    [ -n "$v" ] || { bad "Cargo.lock has no $1"; continue; }
    ge "$v" "$2" && echo "ok:   Cargo.lock $1 $v >= $2" || bad "Cargo.lock pins $1 $v, below floor $2"
  done
fi
[ $fail -eq 0 ] && echo "dep floor: PASS" || echo "dep floor: FAIL"
exit $fail
