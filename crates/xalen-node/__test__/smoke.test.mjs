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

test('planetPosition returns the full 6-tuple + retrograde flag', () => {
  const p = xalen.planetPosition('sun', J2000);
  for (const key of ['longitude', 'latitude', 'distance', 'lonSpeed', 'latSpeed', 'distSpeed']) {
    assert.equal(typeof p[key], 'number', `missing/!number field: ${key}`);
  }
  assert.equal(typeof p.isRetrograde, 'boolean', 'isRetrograde should be boolean');
  assert.ok(p.longitude >= 0 && p.longitude < 360, `lon out of range: ${p.longitude}`);
  // Validated vs pyswisseph swe.calc_ut(J2000, SUN, FLG_SWIEPH|FLG_SPEED):
  //   lonSpeed = 1.019432 deg/day, distance = 0.98332764 AU.
  assert.ok(Math.abs(p.lonSpeed - 1.019432) < 0.01, `Sun lonSpeed off: ${p.lonSpeed}`);
  assert.ok(Math.abs(p.distance - 0.98332764) < 1e-3, `Sun distance off: ${p.distance}`);
  assert.equal(p.isRetrograde, false, 'Sun is never retrograde');
});

test('planetPositionById: mean node is retrograde, Ketu = Rahu+180', () => {
  // Mean node (id 9): pyswisseph lonSpeed = -0.052952 deg/day, always retrograde.
  const rahu = xalen.planetPositionById(9, J2000);
  assert.equal(rahu.isRetrograde, true, 'mean node is always retrograde');
  assert.ok(rahu.lonSpeed < 0, `node lonSpeed should be < 0: ${rahu.lonSpeed}`);

  // Ketu (id 13) = Rahu + 180, sharing Rahu's retrograde state.
  const ketu = xalen.planetPositionById(13, J2000);
  const expected = ((rahu.longitude + 180) % 360 + 360) % 360;
  assert.ok(Math.abs(ketu.longitude - expected) < 1e-9, `Ketu lon ${ketu.longitude} != Rahu+180 ${expected}`);
  assert.equal(ketu.isRetrograde, rahu.isRetrograde, 'Ketu shares Rahu retrograde state');
});

test('planetPositionById sidereal subtracts ayanamsa (id arg)', () => {
  const trop = xalen.planetPositionById(0, J2000, undefined); // Sun tropical
  const sid = xalen.planetPositionById(0, J2000, 0); // Sun, Lahiri
  const offset = ((trop.longitude - sid.longitude) % 360 + 360) % 360;
  assert.ok(offset > 23 && offset < 25, `Lahiri offset off: ${offset}`);
  // Retrograde is frame-independent.
  assert.equal(trop.isRetrograde, sid.isRetrograde);
});

test('nakshatraInfo returns the unified structured shape', () => {
  const n = xalen.nakshatraInfo(0.0); // 0 deg => Ashwini, pada 1, lord Ketu
  for (const key of ['name', 'lord', 'deity']) {
    assert.equal(typeof n[key], 'string', `field ${key} should be a string`);
  }
  assert.equal(typeof n.pada, 'number', 'pada should be a number');
  assert.equal(typeof n.index, 'number', 'index should be a number');
  assert.equal(n.index, 0, '0 deg => index 0');
  assert.equal(n.pada, 1, '0 deg => pada 1');
  assert.ok(n.name.includes('Ashwini'), `0 deg => Ashwini, got ${n.name}`);
  // The legacy string function still works.
  assert.equal(typeof xalen.nakshatra(0.0), 'string');
});

test('fullChart returns planets, angles, ayanamsa and 12 cusps (Ketu present)', () => {
  const c = xalen.fullChart(J2000, 18.52, 73.85, 0);
  assert.ok(c.planets && typeof c.planets === 'object', 'planets object present');
  for (const body of ['Sun', 'Moon', 'Mars', 'Rahu', 'Ketu']) {
    assert.ok(c.planets[body], `chart should include ${body}`);
    assert.equal(typeof c.planets[body].longitude, 'number', `${body}.longitude`);
  }
  assert.equal(typeof c.ascendant, 'number', 'ascendant');
  assert.equal(typeof c.ayanamsaDeg, 'number', 'ayanamsaDeg');
  assert.ok(Array.isArray(c.cusps) && c.cusps.length === 12, 'expected 12 cusps');
  // Ketu is exactly opposite Rahu.
  const expected = ((c.planets.Rahu.longitude + 180) % 360 + 360) % 360;
  assert.ok(Math.abs(c.planets.Ketu.longitude - expected) < 1e-9, 'Ketu = Rahu+180');
});
