// Smoke tests for the XALEN Node.js binding.
//
// Run after building the addon:
//   cd crates/xalen-node && npm install && npm run build:debug && npm test
//
// These exercise the published N-API surface (the Rust side is covered by
// `cargo test`; this verifies the addon loads and marshals values correctly).
import test from 'node:test';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
// The napi build emits index.js next to package.json.
const xalen = require('../index.js');

const J2000 = 2451545.0;

test('planet longitude is in range', () => {
  const lon = xalen.planetLongitude('sun', J2000);
  assert.ok(lon >= 0 && lon < 360, `Sun longitude out of range: ${lon}`);
  // Sun at J2000 is ~280.37 deg tropical.
  assert.ok(Math.abs(lon - 280.37) < 0.1, `Sun longitude off: ${lon}`);
});

test('houses() returns angles, not just cusps', () => {
  const h = xalen.houses(J2000, 18.52, 73.85, 'placidus');
  assert.ok(Array.isArray(h.cusps) && h.cusps.length === 12, 'expected 12 cusps');
  for (const key of ['ascendant', 'mc', 'ic', 'descendant', 'vertex']) {
    assert.equal(typeof h[key], 'number', `missing angle: ${key}`);
    assert.ok(h[key] >= 0 && h[key] < 360, `${key} out of range: ${h[key]}`);
  }
  // MC and IC are opposite.
  const sep = Math.abs(((h.mc - h.ic) % 360 + 360) % 360 - 180);
  assert.ok(sep < 0.01, `MC and IC should be opposite, got sep ${sep}`);
});

test('numerology life path is a single digit or master number', () => {
  const lp = xalen.lifePath(1990, 3, 15);
  assert.ok([1, 2, 3, 4, 5, 6, 7, 8, 9, 11, 22, 33].includes(lp), `unexpected life path: ${lp}`);
});
