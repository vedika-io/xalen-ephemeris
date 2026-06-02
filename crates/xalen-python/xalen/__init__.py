"""XALEN — pure-Rust astronomical & astrology ephemeris (Python bindings).

The compiled extension is built into this package as ``xalen.xalen`` (the
``xalen/xalen.abi3.so`` shared object). This ``__init__`` re-exports its public
API at the top level so ``import xalen`` exposes the functions directly, and it
pulls in the ``swe`` submodule so the pyswisseph drop-in is reachable both as the
attribute ``xalen.swe`` and via ``import xalen.swe`` (the extension registers it
in ``sys.modules`` on initialization).
"""

from .xalen import *  # noqa: F401,F403  — julian_day, planet_position, full_chart, ...
from .xalen import swe  # noqa: F401     — pyswisseph drop-in submodule

__all__ = [name for name in dir() if not name.startswith("_")]
