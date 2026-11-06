#!/usr/bin/env bash
# Build and check the publishable release artifacts on one Linux host
# (x86_64 or aarch64). Nothing is published from here.
#
# Output, in dist/ at the repository root:
#   xalen-<version>-cp38-abi3-manylinux_2_17_x86_64.manylinux2014_x86_64.whl
#   xalen-<version>-cp38-abi3-manylinux_2_17_aarch64.manylinux2014_aarch64.whl
#   xalen-<version>-cp38-abi3-macosx_11_0_arm64.whl
#   xalen-<version>-cp38-abi3-macosx_10_12_x86_64.whl
#   xalen-<version>.tar.gz        Python source distribution (any other platform)
#   xalen-wasm-<version>.tgz      npm tarball of @xalen/wasm
#   SHA256SUMS
#
# How the cross builds work: every wheel is an abi3 extension module, so it
# needs no Python headers and never links libpython.
#   Linux   zig is the cross linker (`maturin build --zig`); it carries the
#           glibc 2.17 stubs for manylinux2014, so no container is needed.
#   macOS   linked with rust-lld, the LLVM linker inside the Rust toolchain,
#           against a stub SDK: zig's libSystem.tbd plus an empty libiconv
#           stub. zig's own Mach-O linker is not used: it leaves the __LINKEDIT
#           string pool unaligned and current macOS refuses to load the result
#           ("mis-aligned LINKEDIT string pool").
#
# Tools are installed into a private directory (XALEN_RELEASE_TOOLS, default: a
# fresh temp dir) and the host Python is left untouched:
#   maturin, ziglang, pytest   pip install --target
#   wasm-pack                  cargo install --root
# Needs on the host: bash, rustup + cargo (with rust-lld), python3 with pip,
# node + npm.
#
# Checks run before the script reports success:
#   - the wheel for this host's architecture is installed into a scratch
#     directory, smoke-tested, and the Python test suite runs against it;
#   - each macOS extension is checked for the __LINKEDIT alignment that macOS
#     enforces;
#   - the sdist is scanned so the unpublished non-commercial add-on crate
#     (xalen-stars-hip-data) can never ship in it;
#   - the WebAssembly package is loaded in Node and smoke-tested.
# Wheels for other architectures and operating systems cannot run here; import
# each one on its own platform before uploading it.
#
# Usage:
#   scripts/build-release-artifacts.sh            # everything
#   scripts/build-release-artifacts.sh python     # wheels + sdist only
#   scripts/build-release-artifacts.sh wasm       # WebAssembly package only

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$REPO_ROOT/dist"
PY_DIR="$REPO_ROOT/crates/xalen-python"
WASM_DIR="$REPO_ROOT/crates/xalen-wasm"
TOOLS="${XALEN_RELEASE_TOOLS:-$(mktemp -d)}"
WHAT="${1:-all}"

VERSION="$(sed -n 's/^version = "\(.*\)"$/\1/p' "$REPO_ROOT/Cargo.toml" | head -n 1)"
HOST_ARCH="$(uname -m)" # x86_64 | aarch64

LINUX_TARGETS=(x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu)
MACOS_TARGETS=(aarch64-apple-darwin x86_64-apple-darwin)

FAILED=()
log()  { printf '\n== %s ==\n' "$*"; }
fail() { printf '   [FAIL] %s\n' "$*" >&2; FAILED+=("$*"); }
ok()   { printf '   [ OK ] %s\n' "$*"; }

[ -n "$VERSION" ] || { echo "cannot read the workspace version from Cargo.toml" >&2; exit 2; }
[ "$(uname -s)" = "Linux" ] || { echo "this script builds on a Linux host" >&2; exit 2; }

mkdir -p "$DIST_DIR" "$TOOLS/py" "$TOOLS/cargo" "$TOOLS/bin"
export PYTHONPATH="$TOOLS/py"
export PATH="$TOOLS/py/bin:$TOOLS/py/ziglang:$TOOLS/cargo/bin:$PATH"

# --- macOS link setup: rust-lld + a stub SDK --------------------------------
# Prints nothing; leaves $TOOLS/bin/macos-lld and $TOOLS/macsdk ready.
setup_macos_linker() {
  local host lld system_tbd
  host="$(rustc -vV | sed -n 's/^host: //p')"
  lld="$(rustc --print sysroot)/lib/rustlib/$host/bin/rust-lld"
  [ -x "$lld" ] || { echo "rust-lld not found at $lld" >&2; return 1; }
  system_tbd="$(find "$TOOLS/py/ziglang" -name libSystem.tbd | head -n 1)"
  [ -n "$system_tbd" ] || { echo "libSystem.tbd not found in the zig package" >&2; return 1; }

  mkdir -p "$TOOLS/macsdk/usr/lib"
  # Newer SDK stubs list the arm64e.x1 architecture, which the LLVM inside the
  # Rust toolchain cannot parse yet. No wheel targets it, so drop it.
  python3 - "$system_tbd" "$TOOLS/macsdk/usr/lib/libSystem.tbd" <<'PY' || return 1
import re
import sys

text = open(sys.argv[1], encoding="utf-8").read()
text = re.sub(r",\s*arm64e\.x1-[a-z]+", "", text)
text = re.sub(r"arm64e\.x1-[a-z]+\s*,\s*", "", text)
if "arm64e.x1" in text:
    raise SystemExit("libSystem.tbd still names arm64e.x1 after filtering")
open(sys.argv[2], "w", encoding="utf-8").write(text)
PY
  ln -sf libSystem.tbd "$TOOLS/macsdk/usr/lib/libc.tbd"
  ln -sf libSystem.tbd "$TOOLS/macsdk/usr/lib/libm.tbd"
  # Rust's libc crate asks for libiconv on Apple targets; nothing here calls it.
  cat > "$TOOLS/macsdk/usr/lib/libiconv.tbd" <<'TBD'
--- !tapi-tbd
tbd-version:     4
targets:         [ x86_64-macos, arm64-macos ]
install-name:    '/usr/lib/libiconv.2.dylib'
current-version: 7
compatibility-version: 7
exports:
  - targets:         [ x86_64-macos, arm64-macos ]
    symbols:         [ _iconv, _iconv_close, _iconv_open ]
...
TBD

  # maturin hands one linker argument over in compiler-driver form
  # (-Wl,-install_name,...); rust-lld takes plain linker arguments.
  cat > "$TOOLS/bin/macos-lld" <<WRAPPER
#!/usr/bin/env bash
args=()
for a in "\$@"; do
  case "\$a" in
    -Wl,*) IFS=, read -r -a parts <<< "\${a#-Wl,}"; args+=("\${parts[@]}") ;;
    *)     args+=("\$a") ;;
  esac
done
exec "$lld" "\${args[@]}"
WRAPPER
  chmod +x "$TOOLS/bin/macos-lld"
}

# Fails when a macOS extension inside a wheel has a symbol table or string pool
# that is not 8-byte aligned in __LINKEDIT (dyld rejects such a file).
check_macho_alignment() {
  python3 - "$@" <<'PY'
import struct
import sys
import zipfile

LC_SYMTAB = 0x2
for wheel in sys.argv[1:]:
    with zipfile.ZipFile(wheel) as archive:
        libs = [n for n in archive.namelist() if n.endswith(".so")]
        if not libs:
            raise SystemExit(f"{wheel}: no extension module inside")
        for name in libs:
            blob = archive.read(name)
            magic, _cpu, _sub, _type, ncmds = struct.unpack_from("<IiiII", blob, 0)
            if magic != 0xFEEDFACF:
                raise SystemExit(f"{wheel}: {name} is not a 64-bit Mach-O file")
            offset, found = 32, False
            for _ in range(ncmds):
                cmd, size = struct.unpack_from("<II", blob, offset)
                if cmd == LC_SYMTAB:
                    symoff, _nsyms, stroff, _strsize = struct.unpack_from("<IIII", blob, offset + 8)
                    if symoff % 8 or stroff % 8:
                        raise SystemExit(
                            f"{wheel}: {name}: __LINKEDIT is not 8-byte aligned "
                            f"(symoff={symoff:#x}, stroff={stroff:#x})"
                        )
                    found = True
                offset += size
            if not found:
                raise SystemExit(f"{wheel}: {name}: no symbol table command")
            print(f"   aligned: {wheel.rsplit('/', 1)[-1]} {name}")
PY
}

# --- Python: abi3 wheels + sdist -------------------------------------------
build_python() {
  log "Python tools (maturin, zig) -> $TOOLS/py"
  python3 -m pip install --quiet --disable-pip-version-check --target "$TOOLS/py" \
    maturin ziglang pytest || { fail "pip install maturin ziglang pytest"; return; }
  maturin --version && zig version

  rustup target add "${LINUX_TARGETS[@]}" "${MACOS_TARGETS[@]}" >/dev/null \
    || { fail "rustup target add"; return; }

  # maturin reads crates/xalen-python/pyproject.toml (extension-module feature,
  # type stubs) only when it runs from that directory.
  cd "$PY_DIR" || { fail "cd $PY_DIR"; return; }

  log "Python wheels $VERSION"
  local tgt
  for tgt in "${LINUX_TARGETS[@]}"; do
    if maturin build --release --zig --compatibility manylinux2014 \
         --target "$tgt" --out "$DIST_DIR"; then
      ok "wheel $tgt"
    else
      fail "wheel $tgt"
    fi
  done
  if setup_macos_linker; then
    for tgt in "${MACOS_TARGETS[@]}"; do
      if SDKROOT="$TOOLS/macsdk" maturin build --release --target "$tgt" --out "$DIST_DIR" \
           -- -C linker="$TOOLS/bin/macos-lld" -C linker-flavor=ld64.lld; then
        ok "wheel $tgt"
      else
        fail "wheel $tgt"
      fi
    done
    if check_macho_alignment "$DIST_DIR"/xalen-"$VERSION"-*macosx*.whl; then
      ok "macOS extensions are aligned for dyld"
    else
      fail "macOS extension alignment"
    fi
  else
    fail "macOS linker setup"
  fi

  log "Python sdist $VERSION"
  if maturin sdist --out "$DIST_DIR"; then
    ok "sdist"
  else
    fail "sdist"
  fi
  cd "$REPO_ROOT" || return

  local sdist="$DIST_DIR/xalen-$VERSION.tar.gz"
  if [ -f "$sdist" ]; then
    if tar -tzf "$sdist" | grep -i -E 'hip-data|catalog_generated'; then
      fail "sdist contains the non-commercial star add-on"
    else
      ok "sdist carries no non-commercial star data"
    fi
  fi

  log "Python check on this host ($HOST_ARCH)"
  local wheel
  wheel="$(ls "$DIST_DIR"/xalen-"$VERSION"-*manylinux*"$HOST_ARCH"*.whl 2>/dev/null | head -n 1)"
  if [ -z "$wheel" ]; then
    fail "no wheel for $HOST_ARCH to check"
    return
  fi
  rm -rf "$TOOLS/site"
  python3 -m pip install --quiet --disable-pip-version-check --no-deps \
    --target "$TOOLS/site" "$wheel" || { fail "install $wheel"; return; }
  if PYTHONPATH="$TOOLS/site:$TOOLS/py" python3 - <<'PY'
import xalen
import xalen.swe as swe

jd = swe.julday(2000, 1, 1, 12.0)
assert jd == 2451545.0, jd
(lon, _lat, dist, lon_speed, *_), _flag = swe.calc_ut(jd, swe.SUN, swe.FLG_SWIEPH | swe.FLG_SPEED)
assert 280.0 < lon < 281.0, lon
assert 0.98 < dist < 0.99, dist
assert 1.0 < lon_speed < 1.03, lon_speed
star, name, _flag = swe.fixstar2_ut("Aldebaran", jd, swe.FLG_SWIEPH)
assert 69.0 < star[0] < 70.5, star
print("wheel smoke test passed: Sun %.4f deg, %s %.4f deg" % (lon, name, star[0]))
PY
  then
    ok "wheel smoke test"
  else
    fail "wheel smoke test"
  fi
  if PYTHONPATH="$TOOLS/site:$TOOLS/py" python3 -m pytest -q "$PY_DIR/tests"; then
    ok "Python test suite against the wheel"
  else
    fail "Python test suite against the wheel"
  fi
}

# --- WebAssembly: one platform-independent npm package ----------------------
build_wasm() {
  log "wasm-pack -> $TOOLS/cargo"
  if ! command -v wasm-pack >/dev/null 2>&1; then
    cargo install wasm-pack --locked --root "$TOOLS/cargo" >/dev/null 2>&1 \
      || { fail "cargo install wasm-pack"; return; }
  fi
  wasm-pack --version
  rustup target add wasm32-unknown-unknown >/dev/null || { fail "rustup target add wasm32"; return; }

  log "WebAssembly package $VERSION"
  cd "$WASM_DIR" || { fail "cd $WASM_DIR"; return; }
  rm -rf pkg
  wasm-pack build --release --target web --out-dir pkg || { fail "wasm-pack build"; cd "$REPO_ROOT"; return; }
  cd pkg || { fail "cd pkg"; return; }
  # The crate is `xalen-wasm`; the npm package is published as `@xalen/wasm`.
  npm pkg set name="@xalen/wasm" || { fail "npm pkg set name"; cd "$REPO_ROOT"; return; }

  log "WebAssembly check in Node"
  if node --input-type=module - "$WASM_DIR/pkg" <<'JS'
import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";
import { join } from "node:path";

const pkg = process.argv[2];
const { default: init, XalenWasm } = await import(pathToFileURL(join(pkg, "xalen_wasm.js")).href);
await init({ module_or_path: readFileSync(join(pkg, "xalen_wasm_bg.wasm")) });

const jd = XalenWasm.julianDay(2000, 1, 1, 12.0);
if (jd !== 2451545.0) throw new Error(`julianDay ${jd}`);
const aya = XalenWasm.ayanamsaDeg(jd, 0);
if (!(aya > 23 && aya < 25)) throw new Error(`ayanamsa ${aya}`);
const chart = JSON.parse(new XalenWasm().fullChartJson(jd, 18.52, 73.85, 0));
if (!chart.planets || !chart.planets.Sun || !chart.planets.Moon) throw new Error("chart has no Sun/Moon");
console.log(`wasm smoke test passed: Lahiri ${aya.toFixed(4)} deg, Sun ${JSON.stringify(chart.planets.Sun)}`);
JS
  then
    ok "wasm smoke test"
  else
    fail "wasm smoke test"
  fi

  if npm pack --pack-destination "$DIST_DIR" >/dev/null; then
    ok "npm tarball"
  else
    fail "npm pack"
  fi
  cd "$REPO_ROOT" || return
}

case "$WHAT" in
  python) build_python ;;
  wasm)   build_wasm ;;
  all)    build_python; cd "$REPO_ROOT" || exit 2; build_wasm ;;
  *)      echo "usage: $0 [python|wasm|all]" >&2; exit 2 ;;
esac

log "Artifacts in dist/"
( cd "$DIST_DIR" && rm -f SHA256SUMS && sha256sum -- * > SHA256SUMS && cat SHA256SUMS )

if [ "${#FAILED[@]}" -gt 0 ]; then
  printf '\nFAILED (%d):\n' "${#FAILED[@]}" >&2
  printf '  - %s\n' "${FAILED[@]}" >&2
  exit 1
fi
printf '\nAll release artifacts for %s built and checked.\n' "$VERSION"
