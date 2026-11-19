#!/usr/bin/env python3
"""Tests for the code paths of generate.py that the weekly public-repo scan flagged.

Run from the repository root with the standard library only:

    python3 -m unittest discover -s scripts/readme-figures -p "test_*.py"

generate.py only writes SVG text and downloads one pinned GeoJSON file. It never parses XML, so
it must not import an xml module (semgrep use-defused-xml). It fetches the coastline over https
only, through an opener that has no file:// or ftp:// handler, and not through
urllib.request.urlopen (semgrep dynamic-urllib-use-detected) or an http.client connection
(semgrep httpsconnection-detected).
"""
import ast
import importlib.util
import json
import os
import unittest
import urllib.error
import urllib.request
from unittest import mock

HERE = os.path.dirname(os.path.abspath(__file__))
SOURCE = os.path.join(HERE, "generate.py")


def load_generate():
    spec = importlib.util.spec_from_file_location("readme_figures_generate", SOURCE)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class FakeResponse:
    """A canned HTTP response usable as a context manager, like the one OpenerDirector.open returns."""

    def __init__(self, body):
        self._body = body

    def read(self):
        return self._body

    def __enter__(self):
        return self

    def __exit__(self, *exc):
        return False


class GenerateTests(unittest.TestCase):
    def setUp(self):
        self.gen = load_generate()
        with open(SOURCE, encoding="utf-8") as f:
            self.tree = ast.parse(f.read())

    def test_imports_no_xml_module(self):
        for node in ast.walk(self.tree):
            if isinstance(node, ast.Import):
                names = [alias.name for alias in node.names]
            elif isinstance(node, ast.ImportFrom):
                names = [node.module or ""]
            else:
                continue
            for name in names:
                self.assertNotEqual(name.split(".")[0], "xml", f"generate.py imports {name}")

    def test_never_calls_urlopen(self):
        for node in ast.walk(self.tree):
            if isinstance(node, ast.Attribute):
                self.assertNotEqual(node.attr, "urlopen", "generate.py calls urlopen")

    def test_escape_handles_markup_characters_and_leaves_quotes(self):
        self.assertEqual(self.gen.escape("a & b < c > d"), "a &amp; b &lt; c &gt; d")
        self.assertEqual(self.gen.escape("\"quoted\" 'text'"), "\"quoted\" 'text'")
        self.assertEqual(self.gen.escape("&amp;"), "&amp;amp;")
        self.assertEqual(self.gen.escape("Whole Sign · Lahiri"), "Whole Sign · Lahiri")

    def test_opener_handles_https_only(self):
        opener = self.gen.https_only_opener()
        kinds = {type(handler).__name__ for handler in opener.handlers}
        self.assertIn("HTTPSHandler", kinds)
        for forbidden in ("FileHandler", "FTPHandler", "DataHandler", "HTTPHandler"):
            self.assertNotIn(forbidden, kinds)
        # A non-https URL cannot be opened even when it reaches the opener directly.
        with self.assertRaises(urllib.error.URLError):
            opener.open("file:///etc/hostname")
        # A non-2xx answer must raise HTTPError, so the error processor has to be present.
        self.assertIn("HTTPErrorProcessor", kinds)

    def test_fetch_coastline_refuses_other_schemes(self):
        for url in ("file:///etc/passwd", "http://example.com/x.geojson", "ftp://example.com/x", "//example.com/x"):
            with self.assertRaises(ValueError, msg=url):
                self.gen.fetch_coastline(url)

    def test_fetch_coastline_reads_pinned_https_url(self):
        body = json.dumps({"type": "FeatureCollection", "features": []}).encode("utf-8")
        with mock.patch.object(urllib.request.OpenerDirector, "open", return_value=FakeResponse(body)) as opened:
            coast = self.gen.fetch_coastline()
        self.assertEqual(coast, {"type": "FeatureCollection", "features": []})
        opened.assert_called_once_with(
            "https://raw.githubusercontent.com/nvkelso/natural-earth-vector/"
            "ca96624a56bd078437bca8184e78163e5039ad19/geojson/ne_110m_coastline.geojson",
            timeout=60,
        )


if __name__ == "__main__":
    unittest.main()
