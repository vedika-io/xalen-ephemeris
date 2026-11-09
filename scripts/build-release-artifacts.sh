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
#   xalen-ephemeris-<version>.tgz npm tarball of @xalen/ephemeris (Node, loader only)
#   xalen-ephemeris-{linux-x64-gnu,linux-arm64-gnu,darwin-arm64,darwin-x64,win32-x64-msvc}-<version>.tgz
#                                 npm tarballs of the five prebuilt Node addons
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
# The Node addon (crates/xalen-node, npm package @xalen/ephemeris) is built the
# same way, with cargo directly instead of maturin: cargo-zigbuild with a glibc
# 2.17 floor for the two Linux targets, rust-lld against the stub SDK for the two
# macOS targets, and cargo-xwin for Windows x64 (x86_64-pc-windows-msvc): it
# downloads the Microsoft CRT and Windows SDK import libraries into
# $TOOLS/xwin and links with lld-link (the host's, or rust-lld under that name
# when the host has none). The Windows addon is
# built with a static C runtime (+crt-static), as the napi-rs package template
# does for this target, so it needs no Visual C++ Redistributable
# (vcruntime140.dll) on the user's machine. Each library is copied to its napi-rs
# name (xalen.<platform>.node) and the @napi-rs/cli turns the five files into one
# npm package per platform plus the root package that names them as
# optionalDependencies.
#
# Microsoft licence: the CRT and SDK files come under Microsoft's licence
# (https://go.microsoft.com/fwlink/?LinkId=2086102). cargo-xwin takes running it
# as acceptance of those terms; the project accepted them for these builds on
# 7 November 2026. The downloaded files stay in $TOOLS/xwin; no Microsoft file
# is committed or packed. The addon does contain statically linked Microsoft
# C-runtime code, as every Rust binary for this target does, redistributed under
# the distributable-code terms of the Visual Studio licence (the licence accepted
# on 7 November 2026). At run time it links only to Windows system DLLs.
#
# The CRT and SDK come from the Visual Studio 17 manifest (--xwin-version 17,
# pinned below); inside it, cargo-xwin picks the newest CRT and SDK, so those
# float with Microsoft's manifest. The build prints the payloads cargo-xwin
# downloaded, with their versions, so each rebuild can be traced.
#
# Tools are installed into a private directory (XALEN_RELEASE_TOOLS, default: a
# fresh temp dir) and the host Python is left untouched:
#   maturin, ziglang, pytest               pip install --target
#   wasm-pack, cargo-zigbuild, cargo-xwin  cargo install --root
#   @napi-rs/cli                           npm ci inside crates/xalen-node (its own node_modules)
#   llvm-readobj (Windows addon checks)    rustup component llvm-tools
# Needs on the host: bash, rustup + cargo (with rust-lld), python3 with pip,
# node + npm, file.
#
# Checks run before the script reports success:
#   - the wheel for this host's architecture is installed into a scratch
#     directory, smoke-tested, and the Python test suite runs against it;
#   - each macOS extension is checked for the __LINKEDIT alignment that macOS
#     enforces;
#   - the sdist is scanned so the unpublished non-commercial add-on crate
#     (xalen-stars-hip-data) can never ship in it;
#   - the WebAssembly package is loaded in Node and smoke-tested.
#   - the Node addons: the committed index.js / index.d.ts equal what napi generates,
#     both Linux addons need glibc 2.17 or older, both macOS addons pass the
#     __LINKEDIT alignment check, the Windows addon is a PE32+ x86-64 DLL under
#     20 MB that exports napi_register_module_v1 and imports only Windows system
#     DLLs (no vcruntime140.dll, no dynamic UCRT), the packed root and platform tarballs for this
#     host are installed into a scratch project and the Node smoke test runs there.
# Wheels for other architectures and operating systems cannot run here; import
# each one on its own platform before uploading it.
#
# Usage:
#   scripts/build-release-artifacts.sh            # everything
#   scripts/build-release-artifacts.sh python     # wheels + sdist only
#   scripts/build-release-artifacts.sh wasm       # WebAssembly package only
#   scripts/build-release-artifacts.sh node       # Node addons (5 platforms) only

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$REPO_ROOT/dist"
PY_DIR="$REPO_ROOT/crates/xalen-python"
WASM_DIR="$REPO_ROOT/crates/xalen-wasm"
NODE_DIR="$REPO_ROOT/crates/xalen-node"
TOOLS="${XALEN_RELEASE_TOOLS:-$(mktemp -d)}"
WHAT="${1:-all}"

VERSION="$(sed -n 's/^version = "\(.*\)"$/\1/p' "$REPO_ROOT/Cargo.toml" | head -n 1)"
HOST_ARCH="$(uname -m)" # x86_64 | aarch64

LINUX_TARGETS=(x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu)
MACOS_TARGETS=(aarch64-apple-darwin x86_64-apple-darwin)
WINDOWS_TARGETS=(x86_64-pc-windows-msvc) # Node addon only; no Windows wheel here
CARGO_XWIN_VERSION=0.23.1

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
  # (-Wl,-install_name,...); rust-lld takes plain linker arguments. napi-build
  # emits a bare `-Wl` (nothing after it) followed by `-undefined` and
  # `dynamic_lookup`: the bare `-Wl` carries no argument and is dropped, the
  # other two pass through unchanged.
  cat > "$TOOLS/bin/macos-lld" <<WRAPPER
#!/usr/bin/env bash
args=()
for a in "\$@"; do
  case "\$a" in
    -Wl)   ;;
    -Wl,*) IFS=, read -r -a parts <<< "\${a#-Wl,}"; args+=("\${parts[@]}") ;;
    *)     args+=("\$a") ;;
  esac
done
exec "$lld" "\${args[@]}"
WRAPPER
  chmod +x "$TOOLS/bin/macos-lld"
}

# Fails when a macOS extension has a symbol table or string pool that is not
# 8-byte aligned in __LINKEDIT (dyld rejects such a file). Each argument is a
# wheel (every .so inside is checked) or a raw Mach-O file such as a .node addon.
check_macho_alignment() {
  python3 - "$@" <<'PY'
import struct
import sys
import zipfile

LC_SYMTAB = 0x2


def members(path):
    """Yield (name, bytes) for each Mach-O candidate in a wheel or a raw file."""
    if path.endswith(".whl"):
        with zipfile.ZipFile(path) as archive:
            libs = [n for n in archive.namelist() if n.endswith(".so")]
            if not libs:
                raise SystemExit(f"{path}: no extension module inside")
            for name in libs:
                yield name, archive.read(name)
    else:
        with open(path, "rb") as handle:
            yield path.rsplit("/", 1)[-1], handle.read()


for source in sys.argv[1:]:
    for name, blob in members(source):
        magic, _cpu, _sub, _type, ncmds = struct.unpack_from("<IiiII", blob, 0)
        if magic != 0xFEEDFACF:
            raise SystemExit(f"{source}: {name} is not a 64-bit Mach-O file")
        offset, found = 32, False
        for _ in range(ncmds):
            cmd, size = struct.unpack_from("<II", blob, offset)
            if cmd == LC_SYMTAB:
                symoff, _nsyms, stroff, _strsize = struct.unpack_from("<IIII", blob, offset + 8)
                if symoff % 8 or stroff % 8:
                    raise SystemExit(
                        f"{source}: {name}: __LINKEDIT is not 8-byte aligned "
                        f"(symoff={symoff:#x}, stroff={stroff:#x})"
                    )
                found = True
            offset += size
        if not found:
            raise SystemExit(f"{source}: {name}: no symbol table command")
        print(f"   aligned: {source.rsplit('/', 1)[-1]} {name}")
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

# --- Node: five prebuilt N-API addons as npm packages -------------------------
# rust target -> the platform suffix napi-rs puts in the file and package name
node_platform() {
  case "$1" in
    x86_64-unknown-linux-gnu)  echo linux-x64-gnu ;;
    aarch64-unknown-linux-gnu) echo linux-arm64-gnu ;;
    aarch64-apple-darwin)      echo darwin-arm64 ;;
    x86_64-apple-darwin)       echo darwin-x64 ;;
    x86_64-pc-windows-msvc)    echo win32-x64-msvc ;;
    *) return 1 ;;
  esac
}

# Structural checks of a Windows addon; there is no Windows machine to load it
# on. $1 is the .node file, $2 the llvm-readobj to read it with. It must be a
# PE32+ x86-64 DLL under 20 MB that exports the N-API entry point
# (napi_register_module_v1) and imports nothing but Windows system DLLs. The
# addon is linked with +crt-static, so neither the Visual C++ runtime
# (vcruntime140.dll) nor the dynamic UCRT (api-ms-win-crt-*.dll) may appear: an
# import of either means the static C runtime was lost.
check_windows_addon() {
  local f="$1" readobj="$2" name desc size dll imports
  name="$(basename "$f")"
  desc="$(file -b "$f")"
  echo "   file: $name: $desc"
  case "$desc" in
    *PE32+*DLL*x86-64* | *PE32+*x86-64*DLL*) ok "$name is a PE32+ x86-64 DLL" ;;
    *) fail "$name is not a PE32+ x86-64 DLL" ;;
  esac

  echo "   exports: $("$readobj" --coff-exports "$f" | sed -n 's/^ *Name: //p' | tr '\n' ' ')"
  if "$readobj" --coff-exports "$f" | grep -Eq '^ *Name: napi_register_module_v1$'; then
    ok "$name exports napi_register_module_v1"
  else
    fail "$name does not export napi_register_module_v1"
  fi

  # Both the normal and the delay-load import tables list their DLLs as `Name:`.
  imports="$("$readobj" --coff-imports "$f" | sed -n 's/^ *Name: //p' | sort -uf)"
  echo "   imports: $(echo "$imports" | tr '\n' ' ')"
  if [ -z "$imports" ]; then
    fail "$name imports no DLL at all (not a linked Windows DLL?)"
  else
    local foreign=()
    while IFS= read -r dll; do
      case "$(echo "$dll" | tr 'A-Z' 'a-z')" in
        kernel32.dll | ntdll.dll | advapi32.dll | bcrypt.dll | bcryptprimitives.dll \
          | ws2_32.dll | userenv.dll | api-ms-win-core-*.dll) ;;
        *) foreign+=("$dll") ;;
      esac
    done <<< "$imports"
    if [ "${#foreign[@]}" -eq 0 ]; then
      ok "$name imports only Windows system DLLs"
    else
      fail "$name imports DLLs outside the allowed Windows system set: ${foreign[*]}"
    fi
  fi

  size="$(stat -c %s "$f")"
  if [ "$size" -lt $((20 * 1024 * 1024)) ]; then
    ok "$name is $size bytes (limit 20 MB)"
  else
    fail "$name is $size bytes, over the 20 MB limit"
  fi
}

# Highest glibc version a Linux ELF file needs (empty when it needs none).
# readelf reads ELF files of any architecture; objdump on this host may not.
glibc_floor() {
  readelf -W --dyn-syms "$1" | grep -o 'GLIBC_[0-9][0-9.]*' | sed 's/^GLIBC_//' | sort -uV | tail -n 1
}

# Prints the minimum macOS version recorded in each Mach-O file.
macho_minos() {
  python3 - "$@" <<'PY'
import struct, sys
for path in sys.argv[1:]:
    blob = open(path, "rb").read()
    ncmds = struct.unpack_from("<I", blob, 16)[0]
    off, minos = 32, None
    for _ in range(ncmds):
        cmd, size = struct.unpack_from("<II", blob, off)
        if cmd == 0x32:  # LC_BUILD_VERSION
            minos = struct.unpack_from("<I", blob, off + 12)[0]
        elif cmd == 0x24 and minos is None:  # LC_VERSION_MIN_MACOSX
            minos = struct.unpack_from("<I", blob, off + 8)[0]
        off += size
    if minos is None:
        print(f"   minos: {path.rsplit('/', 1)[-1]}: none recorded")
    else:
        print(f"   minos: {path.rsplit('/', 1)[-1]}: macOS {minos >> 16}.{(minos >> 8) & 255}")
PY
}

build_node() {
  local target_dir="${CARGO_TARGET_DIR:-$REPO_ROOT/target}"
  local art="$TOOLS/node-addons" # the five xalen.<platform>.node files
  local native="$TOOLS/node-native"
  local tgt plat upper floor deps d tgz proj f pkgdir napi_bin target_os_min host readobj

  # build_node never edits the source tree (the npm work runs in a scratch copy),
  # but if it ever did, a killed run must not leave package.json changed.
  cp "$NODE_DIR/package.json" "$TOOLS/package.json.orig"
  restore_node_package_json() { cp "$TOOLS/package.json.orig" "$NODE_DIR/package.json" 2>/dev/null; }
  trap restore_node_package_json EXIT
  trap 'restore_node_package_json; exit 130' INT
  trap 'restore_node_package_json; exit 143' TERM

  log "Node tools (ziglang, cargo-zigbuild) -> $TOOLS"
  python3 -m pip install --quiet --disable-pip-version-check --target "$TOOLS/py" \
    ziglang || { fail "pip install ziglang"; return; }
  zig version
  if ! command -v cargo-zigbuild >/dev/null 2>&1; then
    cargo install cargo-zigbuild --locked --root "$TOOLS/cargo" >/dev/null 2>&1 \
      || { fail "cargo install cargo-zigbuild"; return; }
  fi
  command -v cargo-zigbuild
  rustup target add "${LINUX_TARGETS[@]}" "${MACOS_TARGETS[@]}" "${WINDOWS_TARGETS[@]}" >/dev/null \
    || { fail "rustup target add"; return; }
  setup_macos_linker || { fail "macOS linker setup"; return; }

  # cargo-xwin links with the host's lld-link, or with rust-lld under that name
  # when there is none, and uses llvm-lib for archives (llvm-ar from the
  # llvm-tools component when the host has no llvm-lib). llvm-tools also carries
  # the llvm-readobj the Windows checks use. No C source is compiled for the
  # addon, so clang (as clang-cl) is not needed; if a dependency ever adds one,
  # the build fails naming clang-cl.
  log "Windows tools (cargo-xwin $CARGO_XWIN_VERSION, llvm-tools) -> $TOOLS"
  if ! "$TOOLS/cargo/bin/cargo-xwin" --version 2>/dev/null | grep -q " $CARGO_XWIN_VERSION\$"; then
    cargo install cargo-xwin --version "$CARGO_XWIN_VERSION" --locked --root "$TOOLS/cargo" \
      >"$TOOLS/cargo-xwin-install.log" 2>&1 \
      || { tail -n 20 "$TOOLS/cargo-xwin-install.log" >&2; fail "cargo install cargo-xwin $CARGO_XWIN_VERSION"; return; }
  fi
  "$TOOLS/cargo/bin/cargo-xwin" --version
  rustup component add llvm-tools >/dev/null || { fail "rustup component add llvm-tools"; return; }
  host="$(rustc -vV | sed -n 's/^host: //p')"
  readobj="$(rustc --print sysroot)/lib/rustlib/$host/bin/llvm-readobj"
  [ -x "$readobj" ] || { fail "llvm-readobj not found at $readobj"; return; }
  "$readobj" --version | sed -n 's/^ *\(LLVM version .*\)/   llvm-readobj: \1/p'
  if command -v lld-link >/dev/null 2>&1; then
    echo "   lld-link: $(command -v lld-link): $(lld-link --version)"
  else
    echo "   lld-link: none on the host; cargo-xwin links with rust-lld"
  fi
  if command -v clang >/dev/null 2>&1; then
    echo "   clang: $(clang --version | head -n 1)"
  else
    echo "   clang: not installed (not needed: no C sources)"
  fi

  log "Node package tooling (@napi-rs/cli from package-lock.json)"
  cd "$NODE_DIR" || { fail "cd $NODE_DIR"; return; }
  npm ci --no-audit --no-fund || { fail "npm ci"; cd "$REPO_ROOT" || true; return; }
  npx napi --version

  # The loader (index.js) and the types (index.d.ts) are generated; the committed
  # copies must be exactly what the CLI writes for this checkout. This native
  # build is only used for that comparison and is never shipped.
  log "Node loader and types are current"
  rm -rf "$native" && mkdir -p "$native"
  if npx napi build --platform --release --output-dir "$native"; then
    for f in index.js index.d.ts; do
      if cmp -s "$NODE_DIR/$f" "$native/$f"; then
        ok "committed $f equals the generated one"
      else
        fail "committed $f differs from the generated one (run: npx napi build --platform --release)"
      fi
    done
  else
    fail "native napi build"
  fi

  log "Node addons $VERSION"
  rm -rf "$art" && mkdir -p "$art"
  for tgt in "${LINUX_TARGETS[@]}"; do
    plat="$(node_platform "$tgt")"
    # `.2.17` is cargo-zigbuild's glibc floor, the same as manylinux2014.
    if cargo zigbuild --release -p xalen-node --target "$tgt.2.17" \
       && cp "$target_dir/$tgt/release/libxalen_node.so" "$art/xalen.$plat.node"; then
      ok "addon $plat"
    else
      fail "addon $plat"
    fi
  done
  for tgt in "${MACOS_TARGETS[@]}"; do
    plat="$(node_platform "$tgt")"
    upper="$(echo "$tgt" | tr 'a-z-' 'A-Z_')"
    case "$tgt" in
      aarch64-apple-darwin) target_os_min=11.0 ;;
      *)                    target_os_min=10.13 ;;
    esac
    if env SDKROOT="$TOOLS/macsdk" MACOSX_DEPLOYMENT_TARGET="$target_os_min" \
         "CARGO_TARGET_${upper}_LINKER=$TOOLS/bin/macos-lld" \
         "CARGO_TARGET_${upper}_RUSTFLAGS=-C linker-flavor=ld64.lld" \
         cargo build --release -p xalen-node --target "$tgt" \
       && cp "$target_dir/$tgt/release/libxalen_node.dylib" "$art/xalen.$plat.node"; then
      ok "addon $plat"
    else
      fail "addon $plat"
    fi
  done
  # The Microsoft CRT and SDK files go to $TOOLS/xwin (x64 only). RUSTFLAGS
  # holds only +crt-static: with --target, cargo applies it to the addon and not
  # to build scripts, and cargo-xwin adds its own linker flags to it (static
  # UCRT included).
  for tgt in "${WINDOWS_TARGETS[@]}"; do
    plat="$(node_platform "$tgt")"
    if env XWIN_CACHE_DIR="$TOOLS/xwin" XWIN_ARCH=x86_64 \
         RUSTFLAGS="-C target-feature=+crt-static" \
         cargo xwin build --xwin-version 17 --release -p xalen-node --target "$tgt" \
       && cp "$target_dir/$tgt/release/xalen_node.dll" "$art/xalen.$plat.node"; then
      ok "addon $plat"
    else
      fail "addon $plat"
    fi
  done
  # cargo-xwin prints no versions itself; its record of the payloads it
  # downloaded ($TOOLS/xwin/DONE: the arches, then one file name per line) names
  # the CRT and SDK versions. Information only, never a failure.
  if [ -f "$TOOLS/xwin/DONE" ]; then
    echo "   xwin payloads (Visual Studio 17 manifest):"
    tail -n +2 "$TOOLS/xwin/DONE" | sort | sed 's/^/     /'
    echo "   Windows SDK: $(grep -oE '10\.0\.[0-9]+(\.[0-9]+)?' "$TOOLS/xwin/DONE" | sort -uV | tr '\n' ' ')"
    echo "   MSVC CRT: $(grep -oE 'VC\.[0-9]+\.[0-9]+(\.[0-9]+)*\.CRT' "$TOOLS/xwin/DONE" | sed 's/^VC\.//; s/\.CRT$//' | sort -uV | tr '\n' ' ')"
  else
    echo "   xwin payload record $TOOLS/xwin/DONE not found"
  fi
  for tgt in "${LINUX_TARGETS[@]}" "${MACOS_TARGETS[@]}" "${WINDOWS_TARGETS[@]}"; do
    plat="$(node_platform "$tgt")"
    [ -f "$art/xalen.$plat.node" ] || { fail "addon $plat is missing, stopping"; cd "$REPO_ROOT" || true; return; }
  done
  ( cd "$art" && sha256sum -- *.node )

  log "Node addon checks"
  for tgt in "${LINUX_TARGETS[@]}"; do
    plat="$(node_platform "$tgt")"
    floor="$(glibc_floor "$art/xalen.$plat.node")"
    if [ -z "$floor" ]; then
      fail "$plat: no glibc symbol versions found (not a gnu build?)"
    elif [ "$(printf '%s\n2.17\n' "$floor" | sort -V | tail -n 1)" = "2.17" ]; then
      ok "$plat needs glibc $floor at most (floor 2.17)"
    else
      fail "$plat needs glibc $floor, above the 2.17 floor"
    fi
  done
  macho_minos "$art"/xalen.darwin-*.node
  if check_macho_alignment "$art"/xalen.darwin-*.node; then
    ok "macOS addons are aligned for dyld"
  else
    fail "macOS addon alignment"
  fi
  for tgt in "${WINDOWS_TARGETS[@]}"; do
    check_windows_addon "$art/xalen.$(node_platform "$tgt").node" "$readobj"
  done

  log "Node npm packages"
  # create-npm-dirs, artifacts and prepublish write npm/, copies of the addons and
  # the optionalDependencies edit into the directory they run in. Run them in a
  # scratch copy of the files the root package ships, so nothing in the source
  # tree is created, edited or removed.
  pkgdir="$TOOLS/node-pkg"
  napi_bin="$NODE_DIR/node_modules/.bin/napi"
  rm -rf "$pkgdir" && mkdir -p "$pkgdir"
  cp package.json index.js index.d.ts README.md LICENSE "$pkgdir/" \
    || { fail "copy package files"; cd "$REPO_ROOT" || true; return; }
  cd "$pkgdir" || { fail "cd $pkgdir"; return; }
  if "$napi_bin" create-npm-dirs \
     && "$napi_bin" artifacts --output-dir "$art" --npm-dir npm \
     && "$napi_bin" prepublish -t npm --skip-optional-publish --no-gh-release; then
    ok "npm package directories prepared (nothing published)"
  else
    fail "napi create-npm-dirs / artifacts / prepublish"
    cd "$REPO_ROOT" || true
    return
  fi
  deps="$(node -e 'const p=require("./package.json");const o=p.optionalDependencies||{};const n=Object.keys(o);if(n.length!==5||n.some(k=>o[k]!==p.version))process.exit(1);console.log(n.join(" "))')" \
    && ok "root optionalDependencies pin all five addons to $VERSION: $deps" \
    || fail "root optionalDependencies are not the five addons at $VERSION"

  for d in npm/*/; do
    npm pack --pack-destination "$DIST_DIR" "./$d" >/dev/null || fail "npm pack $d"
  done
  npm pack --pack-destination "$DIST_DIR" >/dev/null || fail "npm pack root"
  # The loader must not have been rewritten on the way.
  cmp -s index.js "$native/index.js" || fail "index.js changed while packing"
  cd "$REPO_ROOT" || return

  tgz="$DIST_DIR/xalen-ephemeris-$VERSION.tgz"
  if [ -f "$tgz" ] && ! tar -tzf "$tgz" | grep -q '\.node$'; then
    ok "root package carries no binary"
  else
    fail "root package is missing or contains a .node file"
  fi
  for tgt in "${LINUX_TARGETS[@]}" "${MACOS_TARGETS[@]}" "${WINDOWS_TARGETS[@]}"; do
    plat="$(node_platform "$tgt")"
    tgz="$DIST_DIR/xalen-ephemeris-$plat-$VERSION.tgz"
    if [ -f "$tgz" ] && [ "$(tar -tzf "$tgz" | grep -c '\.node$')" = 1 ]; then
      ok "xalen-ephemeris-$plat-$VERSION.tgz carries exactly one addon"
    else
      fail "xalen-ephemeris-$plat-$VERSION.tgz is missing or does not carry exactly one addon"
    fi
  done

  # Install this host's platform package into a scratch project (npm checks its
  # os/cpu/libc fields), unpack the root package beside it, and run the smoke
  # test from there, the way a user would load the addon.
  log "Node smoke test on this host ($HOST_ARCH)"
  plat="$(node_platform "$HOST_ARCH-unknown-linux-gnu")"
  proj="$(mktemp -d)"
  mkdir -p "$proj/t"
  echo '{"name":"xalen-smoke","private":true,"version":"0.0.0"}' > "$proj/package.json"
  ( cd "$proj" && npm install --offline --ignore-scripts --no-audit --no-fund \
      "$DIST_DIR/xalen-ephemeris-$plat-$VERSION.tgz" ) \
    || { fail "npm install of the $plat package (os/cpu/libc fields?)"; return; }
  # The other addons are not on the registry yet, so unpack the root package
  # by hand instead of installing it.
  mkdir -p "$proj/node_modules/@xalen/ephemeris"
  tar -xzf "$DIST_DIR/xalen-ephemeris-$VERSION.tgz" -C "$proj/node_modules/@xalen/ephemeris" --strip-components=1 \
    || { fail "unpack root package"; return; }
  sed "s|require('../index.js')|require('@xalen/ephemeris')|" "$NODE_DIR/__test__/smoke.test.mjs" > "$proj/t/smoke.test.mjs"
  if ! grep -q "require('@xalen/ephemeris')" "$proj/t/smoke.test.mjs"; then
    fail "smoke test no longer requires ../index.js; update build_node"
    return
  fi
  if ( cd "$proj" && node --test t/smoke.test.mjs ); then
    ok "smoke test against the packed $plat addon"
  else
    fail "smoke test against the packed $plat addon"
  fi
  ( cd "$proj" && node -e 'const x=require("@xalen/ephemeris");console.log("node smoke: Sun longitude "+x.planetLongitude("sun",2451545.0).toFixed(6)+" deg")' )
}

case "$WHAT" in
  python) build_python ;;
  wasm)   build_wasm ;;
  node)   build_node ;;
  all)    build_python; cd "$REPO_ROOT" || exit 2; build_node; cd "$REPO_ROOT" || exit 2; build_wasm ;;
  *)      echo "usage: $0 [python|wasm|node|all]" >&2; exit 2 ;;
esac

log "Artifacts in dist/"
( cd "$DIST_DIR" && rm -f SHA256SUMS && sha256sum -- * > SHA256SUMS && cat SHA256SUMS )

if [ "${#FAILED[@]}" -gt 0 ]; then
  printf '\nFAILED (%d):\n' "${#FAILED[@]}" >&2
  printf '  - %s\n' "${FAILED[@]}" >&2
  exit 1
fi
printf '\nAll release artifacts for %s built and checked.\n' "$VERSION"
