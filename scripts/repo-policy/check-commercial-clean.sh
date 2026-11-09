#!/usr/bin/env bash
# Repository policy: no published crate, built with its DEFAULT features, links
# the non-commercial Hipparcos data (ESA, CC BY-NC 3.0 IGO). That data lives only
# in the optional, unpublished `xalen-stars-hip-data` add-on crate. The default
# star list (`xalen-stars` BRIGHT_CATALOG) is built from the Yale Bright Star
# Catalogue and the IAU Catalog of Star Names. See NOTICE and
# crates/xalen-stars-hip-data.
#
# Checks, in order (every check runs; the script exits non-zero if any failed):
#   1  no publishable workspace crate depends on xalen-stars-hip-data with its
#      default features, and nothing in the workspace depends on it at all
#   2  the default xalen-ffi build carries no Hipparcos symbols or catalogue
#      marker strings
#   3  the default star list is present and the lookups work without default
#      features
#   4  the core sidereal crates do not depend on the star catalogue
#
# Usage: scripts/repo-policy/check-commercial-clean.sh   (from anywhere in the repo)
# Needs: cargo, python3 (to read `cargo metadata`), nm and strings (binutils on
# Linux, the Xcode command line tools on macOS).
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT" || exit 2

FAILED=()
fail() { echo "commercial-clean: FAIL: $*" >&2; FAILED+=("$*"); }

meta="$(cargo metadata --format-version 1 --no-deps)" || { echo "cargo metadata failed" >&2; exit 1; }
# Every workspace package that may be published (`publish` is not set to false).
pkgs="$(python3 -c 'import json,sys
for p in json.load(sys.stdin)["packages"]:
    if p.get("publish") is None:
        print(p["name"])' <<<"$meta")" || { echo "cannot read cargo metadata" >&2; exit 1; }
target_dir="$(python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])' <<<"$meta")" \
  || { echo "cannot read the target directory from cargo metadata" >&2; exit 1; }
[ -n "$pkgs" ] || { echo "no publishable workspace packages found" >&2; exit 1; }

echo "== 1. No published crate depends on the NC data crate (default features)"
for p in $pkgs; do
  if ! tree="$(cargo tree -p "$p" 2>/dev/null)"; then
    fail "cargo tree -p $p"
    continue
  fi
  n="$(grep -c 'xalen-stars-hip-data' <<<"$tree" || true)"
  echo "$p (default features): xalen-stars-hip-data count = $n"
  [ "$n" -eq 0 ] || fail "$p links the NC data crate"
done
# The add-on crate is a leaf: nothing in the workspace depends on it.
if rtree="$(cargo tree --workspace -i xalen-stars-hip-data 2>/dev/null)"; then
  rev="$(grep -c 'xalen-stars-hip-data' <<<"$rtree" || true)"
  echo "reverse-dependency lines for xalen-stars-hip-data = $rev (1 = itself only)"
  [ "$rev" -le 1 ] || fail "a crate depends on the NC data crate"
else
  fail "cargo tree --workspace -i xalen-stars-hip-data"
fi

echo "== 2. Published binding builds contain no Hipparcos symbols"
check_ffi_binary() {
  local lib="" f syms
  cargo build -p xalen-ffi || return 1
  for f in libxalen_ffi.a libxalen_ffi.so libxalen_ffi.dylib; do
    if [ -f "$target_dir/debug/$f" ]; then lib="$target_dir/debug/$f"; break; fi
  done
  [ -n "$lib" ] || { echo "no libxalen_ffi library found in $target_dir/debug" >&2; return 1; }
  command -v nm >/dev/null 2>&1 || { echo "nm is not installed" >&2; return 1; }
  command -v strings >/dev/null 2>&1 || { echo "strings is not installed" >&2; return 1; }
  echo "scanning $lib"
  syms="$(nm "$lib" 2>/dev/null)"
  [ -n "$syms" ] || { echo "nm printed no symbols for $lib" >&2; return 1; }
  if grep -iE 'GENERATED_CATALOG|stars_hip_data' <<<"$syms"; then
    echo "NC HIP symbols found in the default binary" >&2
    return 1
  fi
  if strings "$lib" | grep -iE 'hip_main|Hipparcos Main Catalogue'; then
    echo "NC catalogue marker strings found in the default binary" >&2
    return 1
  fi
  echo "clean: no NC symbols/strings"
}
check_ffi_binary || fail "xalen-ffi default build"

echo "== 3. Default star list is present and the lookups work without defaults"
cargo test -p xalen-stars || fail "cargo test -p xalen-stars"
cargo test -p xalen-stars --no-default-features || fail "cargo test -p xalen-stars --no-default-features"
cargo test -p xalen-ephem --no-default-features fixstar || fail "cargo test -p xalen-ephem --no-default-features fixstar"

echo "== 4. Core sidereal output does not depend on the star catalogue"
cargo test -p xalen-star-anchors || fail "cargo test -p xalen-star-anchors"
cargo test -p xalen-ayanamsa || fail "cargo test -p xalen-ayanamsa"

if [ "${#FAILED[@]}" -ne 0 ]; then
  printf 'commercial-clean: %d check(s) failed:\n' "${#FAILED[@]}" >&2
  printf '  - %s\n' "${FAILED[@]}" >&2
  exit 1
fi
echo "commercial-clean: all checks passed"
