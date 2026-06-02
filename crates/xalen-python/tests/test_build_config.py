"""Regression guards for the cross-platform binding configuration.

These tests do NOT require the compiled extension — they parse the build
configuration files so that a future edit which silently drops the abi3 wiring
(reverting to one-wheel-per-Python-version) or breaks the publish matrix is
caught in CI.

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
# Publish matrix: release.yml must cover the documented platforms.
# ---------------------------------------------------------------------------

def test_release_workflow_covers_all_platforms():
    wf = REPO_ROOT / ".github" / "workflows" / "release.yml"
    assert wf.exists(), "release.yml must exist (the durable publish mechanism)"
    text = _read(wf)
    # PyPI abi3 wheel matrix — the five platform targets.
    for target in (
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
        "x86_64-apple-darwin",
        "aarch64-apple-darwin",
        "x86_64-pc-windows-msvc",
    ):
        assert target in text, f"release.yml missing platform target {target}"
    # All three publish channels must be present.
    assert "maturin upload" in text, "PyPI publish step missing"
    assert "napi prepublish" in text, "npm (node) publish step missing"
    assert "wasm-pack publish" in text, "wasm publish step missing"
    assert "cargo publish" in text, "crates.io publish step missing"
    # Tokens must come from secrets, never be hardcoded.
    assert "secrets.PYPI_API_TOKEN" in text
    assert "secrets.NPM_TOKEN" in text
    assert "secrets.CARGO_REGISTRY_TOKEN" in text


def test_crates_publish_order_is_topological():
    """The crates.io publish order in release.yml must list every crate's
    intra-workspace dependencies before the crate itself."""
    wf_text = _read(REPO_ROOT / ".github" / "workflows" / "release.yml")
    m = re.search(r"order=\(([^)]*)\)", wf_text)
    assert m, "publish order array not found in release.yml"
    order = m.group(1).split()
    position = {name: i for i, name in enumerate(order)}

    crates_dir = REPO_ROOT / "crates"
    dep_re = re.compile(r"^xalen-([a-z]+)\s*=", re.MULTILINE)
    for name in order:
        cargo = _read(crates_dir / f"xalen-{name}" / "Cargo.toml")
        for dep in dep_re.findall(cargo):
            if dep in position:  # only intra-published-set deps
                assert position[dep] < position[name], (
                    f"xalen-{name} is published before its dependency "
                    f"xalen-{dep}; fix the order in release.yml"
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
