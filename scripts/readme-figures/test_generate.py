#!/usr/bin/env python3
"""Tests for the two code paths of generate.py that the weekly public-repo scan (semgrep) flagged.

Run from the repository root with the standard library only:

    python3 -m unittest discover -s scripts/readme-figures -p "test_*.py"

generate.py only writes SVG text and downloads one pinned GeoJSON file. It never parses XML, so
it must not import an xml module (semgrep use-defused-xml), and it must fetch the coastline over
https only, never through urllib.request.urlopen, which also opens file:// and ftp:// URLs
(semgrep dynamic-urllib-use-detected).
"""
import ast
import importlib.util
import json
import os
import unittest
from unittest import mock

HERE = os.path.dirname(os.path.abspath(__file__))
SOURCE = os.path.join(HERE, "generate.py")


def load_generate():
    spec = importlib.util.spec_from_file_location("readme_figures_generate", SOURCE)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class FakeResponse:
    def __init__(self, status, body):
        self.status = status
        self._body = body

    def read(self):
        return self._body


class FakeConnection:
    """Stands in for http.client.HTTPSConnection: records the request, returns a canned response."""

    instances = []

    def __init__(self, host, timeout=None, **kwargs):
        self.host = host
        self.timeout = timeout
        self.requests = []
        self.closed = False
        self.response = FakeResponse(200, json.dumps({"type": "FeatureCollection", "features": []}).encode("utf-8"))
        FakeConnection.instances.append(self)

    def request(self, method, path, **kwargs):
        self.requests.append((method, path))

    def getresponse(self):
        return self.response

    def close(self):
        self.closed = True


class GenerateTests(unittest.TestCase):
    def setUp(self):
        self.gen = load_generate()
        FakeConnection.instances = []
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

    def test_fetch_coastline_refuses_other_schemes(self):
        for url in ("file:///etc/passwd", "http://example.com/x.geojson", "ftp://example.com/x", "//example.com/x"):
            with self.assertRaises(ValueError, msg=url):
                self.gen.fetch_coastline(url)

    def test_fetch_coastline_reads_pinned_https_url(self):
        with mock.patch.object(self.gen.http.client, "HTTPSConnection", FakeConnection):
            coast = self.gen.fetch_coastline()
        self.assertEqual(coast, {"type": "FeatureCollection", "features": []})
        conn = FakeConnection.instances[0]
        self.assertEqual(conn.host, "raw.githubusercontent.com")
        self.assertEqual(conn.timeout, 60)
        self.assertEqual(
            conn.requests,
            [("GET", "/nvkelso/natural-earth-vector/ca96624a56bd078437bca8184e78163e5039ad19/geojson/ne_110m_coastline.geojson")],
        )
        self.assertTrue(conn.closed)

    def test_fetch_coastline_fails_on_http_error(self):
        with mock.patch.object(self.gen.http.client, "HTTPSConnection", FakeConnection):
            original = FakeConnection.__init__

            def init_404(conn, host, timeout=None, **kwargs):
                original(conn, host, timeout=timeout, **kwargs)
                conn.response = FakeResponse(404, b"Not Found")

            with mock.patch.object(FakeConnection, "__init__", init_404):
                with self.assertRaises(RuntimeError):
                    self.gen.fetch_coastline()
        self.assertTrue(FakeConnection.instances[0].closed)


if __name__ == "__main__":
    unittest.main()
