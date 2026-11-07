"""Regression guards for the cross-platform binding configuration.

These tests do NOT require the compiled extension — they parse the build
configuration files so that a future edit which silently drops the abi3 wiring
(reverting to one-wheel-per-Python-version) or breaks the release matrix is
caught by the release-config stage of scripts/gate.sh.

When the extension *is* built and importable, ``test_built_wheel_is_abi3``
additionally asserts the produced wheel is abi3-tagged. It is skipped when no
wheel is present, so a pure-source checkout still passes.
"""

import pathlib
import re

import pytest

# crates/xalen-python/
PKG_DIR = pathlib.Path(__file__).resolve().parent.parent
REPO_ROOT = PKG_DIR.parent.parent


def _read(path: pathlib.Path) -> str:
    return path.read_text(encoding="utf-8")


# ---------------------------------------------------------------------------
# abi3 wiring: pyo3 feature + maturin bindings.
# ---------------------------------------------------------------------------

def test_pyo3_has_abi3_feature():
    cargo = _read(PKG_DIR / "Cargo.toml")
    # The pyo3 dependency line must request an abi3-pyXX feature so that one
    # stable-ABI wheel serves all CPython >= that minor version.
    m = re.search(r'^pyo3\s*=\s*\{[^}]*\}', cargo, re.MULTILINE)
    assert m, "pyo3 dependency line not found in Cargo.toml"
    pyo3_line = m.group(0)
    assert "abi3-py38" in pyo3_line, (
        "pyo3 must enable the abi3-py38 feature so one wheel covers CPython "
        f">=3.8; got: {pyo3_line}"
    )


def test_pyproject_declares_pyo3_bindings():
    pyproject = _read(PKG_DIR / "pyproject.toml")
    assert 'bindings = "pyo3"' in pyproject, "maturin bindings must be pyo3"
    # extension-module must stay enabled for the published wheel.
    assert "extension-module" in pyproject


def test_requires_python_matches_abi3_floor():
    # The abi3 floor (abi3-py38) and the project's requires-python must agree:
    # a cp38-abi3 wheel must not be offered to a Python older than 3.8.
    pyproject = _read(PKG_DIR / "pyproject.toml")
    assert 'requires-python = ">=3.8"' in pyproject


# ---------------------------------------------------------------------------
# Release matrix: scripts/build-release-artifacts.sh builds the wheels and the
# npm packages; docs/RELEASING.md holds the publish commands.
# ---------------------------------------------------------------------------

RELEASE_SCRIPT = REPO_ROOT / "scripts" / "build-release-artifacts.sh"
RELEASING_DOC = REPO_ROOT / "docs" / "RELEASING.md"


def _bash_array(text, name):
    m = re.search(rf"^{name}=\(([^)]*)\)", text, re.MULTILINE)
    assert m, f"{name}=(...) not found in {RELEASE_SCRIPT.name}"
    return m.group(1).split()


def _not_yet_scripted(doc):
    m = re.search(r"^## Not yet scripted\n(.*?)(?=^## |\Z)", doc, re.MULTILINE | re.DOTALL)
    assert m, "docs/RELEASING.md must keep a '## Not yet scripted' section"
    return m.group(1)


def test_release_script_covers_all_platforms():
    assert RELEASE_SCRIPT.exists(), "scripts/build-release-artifacts.sh builds the release"
    text = _read(RELEASE_SCRIPT)
    linux = _bash_array(text, "LINUX_TARGETS")
    macos = _bash_array(text, "MACOS_TARGETS")
    windows = _bash_array(text, "WINDOWS_TARGETS")
    # abi3 wheels: Linux (manylinux2014 through zig) and macOS.
    assert sorted(linux) == ["aarch64-unknown-linux-gnu", "x86_64-unknown-linux-gnu"]
    assert sorted(macos) == ["aarch64-apple-darwin", "x86_64-apple-darwin"]
    assert 'maturin build --release --zig --compatibility manylinux2014' in text
    assert '--target "$tgt" --out "$DIST_DIR"' in text, "wheels must be built per target"
    assert "maturin sdist" in text, "Python sdist build missing"
    # Windows x64 is built for the Node addon only.
    assert windows == ["x86_64-pc-windows-msvc"]
    # The wasm package is built with wasm-pack and renamed to its npm scope.
    assert "wasm-pack build --release --target web" in text, "wasm build step missing"
    assert 'npm pkg set name="@xalen/wasm"' in text, "wasm npm scope step missing"
    # The Node packages go through the napi prepublish flow, publishing nothing.
    assert "prepublish -t npm --skip-optional-publish" in text


def test_windows_wheel_gap_is_documented():
    # The former release workflow built a Windows x64 wheel; no script does
    # now. Until one does, RELEASING.md must say so.
    gap = _not_yet_scripted(_read(RELEASING_DOC))
    assert "x86_64-pc-windows-msvc" in gap and "wheel" in gap.lower()


def test_releasing_doc_covers_every_channel():
    doc = _read(RELEASING_DOC)
    assert "scripts/build-release-artifacts.sh" in doc
    assert "maturin upload" in doc, "PyPI publish step missing"
    assert "cargo publish" in doc, "crates.io publish step missing"
    assert 'npm publish --access public "dist/xalen-ephemeris-$p-X.Y.Z.tgz"' in doc, (
        "per-platform Node package publish missing"
    )
    assert "npm publish --access public dist/xalen-ephemeris-X.Y.Z.tgz" in doc
    assert "npm publish --access public dist/xalen-wasm-X.Y.Z.tgz" in doc
    # Credentials come from the environment, never from the repository.
    for var in ("CARGO_REGISTRY_TOKEN", "MATURIN_PYPI_TOKEN", "NODE_AUTH_TOKEN"):
        assert var in doc, f"RELEASING.md must name {var}"
    for path in (RELEASING_DOC, RELEASE_SCRIPT):
        body = _read(path)
        assert not re.search(r"pypi-[A-Za-z0-9_-]{20,}", body), f"PyPI token in {path}"
        assert not re.search(r"npm_[A-Za-z0-9]{30,}", body), f"npm token in {path}"
        assert not re.search(r"\bcio[A-Za-z0-9]{30,}", body), f"crates.io token in {path}"


def test_crates_publish_order_is_topological():
    """The crates.io publish order in docs/RELEASING.md must list every crate's
    intra-workspace dependencies before the crate itself."""
    doc = _read(RELEASING_DOC)
    m = re.search(r"order=\(([^)]*)\)", doc)
    assert m, "publish order array not found in docs/RELEASING.md"
    order = m.group(1).split()
    position = {name: i for i, name in enumerate(order)}
    assert len(position) == len(order), "a crate is listed twice in the publish order"

    crates_dir = REPO_ROOT / "crates"
    dep_re = re.compile(r"^xalen-([a-z-]+)\s*=", re.MULTILINE)
    for name in order:
        # `ephemeris` is the umbrella crate at the repository root.
        manifest = (
            REPO_ROOT / "Cargo.toml"
            if name == "ephemeris"
            else crates_dir / f"xalen-{name}" / "Cargo.toml"
        )
        cargo = _read(manifest)
        for dep in dep_re.findall(cargo):
            if dep in position:  # only intra-published-set deps
                assert position[dep] < position[name], (
                    f"xalen-{name} is published before its dependency "
                    f"xalen-{dep}; fix the order in docs/RELEASING.md"
                )


# ---------------------------------------------------------------------------
# When a wheel is actually built, it must be abi3-tagged.
# ---------------------------------------------------------------------------

def test_built_wheel_is_abi3():
    # Look for a built wheel in the usual maturin output locations.
    search = [
        REPO_ROOT / "dist",
        REPO_ROOT / "target" / "wheels",
        PKG_DIR / "target" / "wheels",
    ]
    wheels: list[pathlib.Path] = []
    for d in search:
        if d.is_dir():
            wheels.extend(d.glob("xalen-*.whl"))
    if not wheels:
        pytest.skip("no built wheel found (build with `maturin build --release`)")
    # An abi3 wheel is tagged like `...-cp38-abi3-<platform>.whl`.
    assert any("abi3" in w.name for w in wheels), (
        "no abi3 wheel found among built wheels: "
        + ", ".join(w.name for w in wheels)
    )
