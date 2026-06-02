#!/usr/bin/env bash
# Build the full cross-platform binding matrix LOCALLY.
#
# Produces, into the repository:
#   dist/                              — Python abi3 wheels + sdist (one wheel
#                                        per platform, valid for CPython >= 3.8)
#   crates/xalen-node/npm/<triple>/    — per-platform Node N-API .node addons
#   crates/xalen-wasm/pkg/             — platform-independent WebAssembly package
#
# Targets attempted:
#   macOS   arm64   (native)
#   macOS   x86_64  (rustup cross, same host linker)
#   Linux   x86_64  (cargo-zigbuild, GNU)
#   Linux   aarch64 (cargo-zigbuild, GNU)
#   Windows x86_64  (cargo-zigbuild / cargo-xwin, GNU)
#
# What CANNOT be cross-built reliably from a macOS host even with zig/docker is
# noted at each step and is instead covered by .github/workflows/release.yml on
# native runners (the durable, per-release mechanism). This script is for local
# matrix verification and for producing artifacts on demand; CI is the source of
# truth for published artifacts.
#
# Prerequisites the orchestrator must install first (printed by --check):
#   rustup target add x86_64-apple-darwin \
#                     x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu \
#                     x86_64-pc-windows-gnu wasm32-unknown-unknown
#   pip install cargo-zigbuild        # provides `cargo zigbuild` (uses zig as linker)
#   cargo install wasm-pack           # or: npm i -g wasm-pack
#   npm install                       # inside crates/xalen-node (napi CLI)
#
# Usage:
#   scripts/build-all-platforms.sh            # build everything possible on this host
#   scripts/build-all-platforms.sh --check    # print prerequisites, do not build
#   scripts/build-all-platforms.sh python     # only the Python wheel matrix
#   scripts/build-all-platforms.sh node       # only the Node addon matrix
#   scripts/build-all-platforms.sh wasm       # only the WebAssembly package
#
# The script never fails the whole run when a single cross-target is unavailable
# on the host: it reports SKIP and continues, so a partial local matrix still
# yields the targets that this host can produce. Exit code is non-zero only when
# a target that SHOULD work on this host fails.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$REPO_ROOT/dist"
NODE_DIR="$REPO_ROOT/crates/xalen-node"
WASM_DIR="$REPO_ROOT/crates/xalen-wasm"
PY_MANIFEST="$REPO_ROOT/crates/xalen-python/Cargo.toml"

HOST_OS="$(uname -s)"   # Darwin | Linux
HOST_ARCH="$(uname -m)" # arm64 | x86_64 | aarch64

# Python abi3 wheels: ONE wheel per platform serves all CPython >= 3.8.
PY_TARGETS=(
  "aarch64-apple-darwin"
  "x86_64-apple-darwin"
  "x86_64-unknown-linux-gnu"
  "aarch64-unknown-linux-gnu"
  "x86_64-pc-windows-gnu"
)

# Node N-API addons. The first four triples match package.json `napi.targets`
# and are the publishable set. The Windows local cross-build targets the *-gnu
# triple (zig can produce it; cargo-xwin/MSVC cannot run on a macOS host), which
# is a LOCAL VERIFICATION artifact only — the published Windows addon is the
# MSVC build produced by .github/workflows/release.yml on a native windows
# runner (package.json lists x86_64-pc-windows-msvc).
NODE_TARGETS=(
  "aarch64-apple-darwin"
  "x86_64-apple-darwin"
  "x86_64-unknown-linux-gnu"
  "aarch64-unknown-linux-gnu"
  "x86_64-pc-windows-gnu"
)

log()  { printf '\n\033[1m== %s ==\033[0m\n' "$*"; }
ok()   { printf '   [ OK ]  %s\n' "$*"; }
skip() { printf '   [SKIP]  %s\n' "$*"; }
err()  { printf '   [FAIL]  %s\n' "$*" >&2; }

OVERALL_RC=0

have_target() { rustup target list --installed 2>/dev/null | grep -qx "$1"; }

print_check() {
  cat <<'EOF'
Run these once on the build host before invoking this script:

  rustup target add x86_64-apple-darwin \
                    x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu \
                    x86_64-pc-windows-gnu wasm32-unknown-unknown
  pip install cargo-zigbuild
  cargo install wasm-pack          # or npm i -g wasm-pack
  ( cd crates/xalen-node && npm install )

zig itself is used by cargo-zigbuild as the cross-linker (no separate install
beyond `pip install cargo-zigbuild`, which expects `zig` on PATH).
EOF
}

# --- Python (maturin, abi3) ------------------------------------------------
build_python() {
  log "Python — abi3 wheels (cp38-abi3) + sdist"
  command -v maturin >/dev/null 2>&1 || { err "maturin not found; pip install maturin"; OVERALL_RC=1; return; }
  mkdir -p "$DIST_DIR"

  # Source distribution: installable on ANY platform with a Rust toolchain.
  if maturin sdist --manifest-path "$PY_MANIFEST" --out "$DIST_DIR" >/dev/null 2>&1; then
    ok "sdist -> dist/"
  else
    err "maturin sdist failed"
    OVERALL_RC=1
  fi

  local use_zig=0
  command -v cargo-zigbuild >/dev/null 2>&1 && use_zig=1

  for tgt in "${PY_TARGETS[@]}"; do
    if ! have_target "$tgt"; then
      skip "wheel $tgt (run: rustup target add $tgt)"
      continue
    fi
    # macOS targets link natively on a macOS host; everything else needs a
    # cross linker. maturin --zig delegates to cargo-zigbuild.
    local zig_flag=""
    case "$tgt" in
      *-apple-darwin)
        if [ "$HOST_OS" != "Darwin" ]; then
          skip "wheel $tgt (Apple targets need a macOS host or CI macOS runner)"
          continue
        fi
        ;;
      *)
        if [ "$use_zig" -eq 1 ]; then
          zig_flag="--zig"
        else
          skip "wheel $tgt (install cargo-zigbuild for cross-compile, or use CI)"
          continue
        fi
        ;;
    esac
    if maturin build --release $zig_flag --target "$tgt" \
         --manifest-path "$PY_MANIFEST" --out "$DIST_DIR" >/dev/null 2>&1; then
      ok "wheel $tgt -> dist/"
    else
      err "wheel $tgt build failed"
      OVERALL_RC=1
    fi
  done
  printf '   wheels now in dist/:\n'
  ls -1 "$DIST_DIR" 2>/dev/null | sed 's/^/     /'
}

# --- Node (napi-rs) --------------------------------------------------------
build_node() {
  log "Node — N-API addons (.node) per platform"
  ( cd "$NODE_DIR" && [ -d node_modules ] || npm install >/dev/null 2>&1 )

  # Generate per-platform npm package dirs under crates/xalen-node/npm/<triple>/
  ( cd "$NODE_DIR" && npx napi create-npm-dirs >/dev/null 2>&1 ) \
    && ok "npm/<triple>/ package dirs created" \
    || skip "create-npm-dirs (napi CLI) — non-fatal"

  for tgt in "${NODE_TARGETS[@]}"; do
    if ! have_target "$tgt"; then
      skip "addon $tgt (run: rustup target add $tgt)"
      continue
    fi
    local x_flag=""
    case "$tgt" in
      *-apple-darwin)
        if [ "$HOST_OS" != "Darwin" ]; then
          skip "addon $tgt (Apple targets need a macOS host or CI macOS runner)"
          continue
        fi
        ;;
      x86_64-unknown-linux-gnu|aarch64-unknown-linux-gnu|x86_64-pc-windows-gnu)
        # napi --cross-compile uses cargo-zigbuild (non-Windows hosts) / cargo-xwin.
        if command -v cargo-zigbuild >/dev/null 2>&1; then
          x_flag="--cross-compile"
        else
          skip "addon $tgt (install cargo-zigbuild for --cross-compile, or use CI)"
          continue
        fi
        ;;
    esac
    if ( cd "$NODE_DIR" && npx napi build --platform --release $x_flag \
           --target "$tgt" --output-dir "." >/dev/null 2>&1 ); then
      ok "addon $tgt"
    else
      err "addon $tgt build failed"
      OVERALL_RC=1
    fi
  done

  # Collect any built .node files into the per-triple npm dirs.
  ( cd "$NODE_DIR" && npx napi artifacts --output-dir . --npm-dir npm >/dev/null 2>&1 ) \
    && ok "addons collected into npm/<triple>/" \
    || skip "napi artifacts (nothing to collect, or CI handles it)"

  printf '   .node files in %s:\n' "$NODE_DIR"
  ls -1 "$NODE_DIR"/*.node 2>/dev/null | sed 's/^/     /' || printf '     (none on this host)\n'
}

# --- WASM (platform-independent) -------------------------------------------
build_wasm() {
  log "WebAssembly — platform-independent package"
  if ! command -v wasm-pack >/dev/null 2>&1; then
    skip "wasm-pack not found (cargo install wasm-pack OR npm i -g wasm-pack)"
    return
  fi
  if ! have_target "wasm32-unknown-unknown"; then
    skip "wasm32-unknown-unknown target (rustup target add wasm32-unknown-unknown)"
    return
  fi
  # One wasm artifact runs on every platform. Build the web target into pkg/.
  if ( cd "$WASM_DIR" && wasm-pack build --release --target web --out-dir pkg >/dev/null 2>&1 ); then
    ok "wasm -> crates/xalen-wasm/pkg/"
    ls -1 "$WASM_DIR/pkg" 2>/dev/null | sed 's/^/     /'
  else
    err "wasm-pack build failed"
    OVERALL_RC=1
  fi
}

main() {
  local what="${1:-all}"
  case "$what" in
    --check|-c) print_check; exit 0 ;;
    python) build_python ;;
    node)   build_node ;;
    wasm)   build_wasm ;;
    all)    build_python; build_node; build_wasm ;;
    *) echo "usage: $0 [all|python|node|wasm|--check]" >&2; exit 2 ;;
  esac
  log "Summary"
  printf '   host: %s/%s\n' "$HOST_OS" "$HOST_ARCH"
  if [ "$OVERALL_RC" -eq 0 ]; then
    ok "all targets that this host supports were produced"
  else
    err "one or more host-supported targets failed (see above)"
  fi
  printf '   Targets that cannot be produced on this host are covered by\n'
  printf '   .github/workflows/release.yml on native GitHub runners.\n'
  exit "$OVERALL_RC"
}

main "$@"
