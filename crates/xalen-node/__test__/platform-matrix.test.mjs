// Regression guard for the Node binding's cross-platform publish matrix.
//
// Does NOT require the compiled addon — it reads package.json and the release
// workflow so a future edit that drops a target (shipping a Node addon for only
// one platform) is caught in CI.

import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const NODE_DIR = join(__dirname, '..');
const REPO_ROOT = join(NODE_DIR, '..', '..');

const pkg = JSON.parse(readFileSync(join(NODE_DIR, 'package.json'), 'utf8'));

// The five platform triples published from 0.8, all built on one Linux host by
// scripts/build-release-artifacts.sh node (Windows x64 through cargo-xwin).
const EXPECTED = [
  'aarch64-apple-darwin',
  'x86_64-apple-darwin',
  'aarch64-unknown-linux-gnu',
  'x86_64-unknown-linux-gnu',
  'x86_64-pc-windows-msvc',
];

test('package.json declares the full napi target matrix', () => {
  assert.ok(pkg.napi, 'package.json must have a "napi" config block');
  const targets = pkg.napi.targets ?? [];
  for (const t of EXPECTED) {
    assert.ok(targets.includes(t), `napi.targets missing ${t}`);
  }
  assert.equal(targets.length, EXPECTED.length, 'napi.targets has targets beyond the published five');
});

test('committed loader names the Windows x64 package', () => {
  const loader = readFileSync(join(NODE_DIR, 'index.js'), 'utf8');
  assert.ok(
    loader.includes('@xalen/ephemeris-win32-x64-msvc'),
    'index.js does not load @xalen/ephemeris-win32-x64-msvc',
  );
});

test('release workflow builds every package.json napi target', () => {
  const wf = readFileSync(join(REPO_ROOT, '.github', 'workflows', 'release.yml'), 'utf8');
  for (const t of (pkg.napi.targets ?? [])) {
    assert.ok(wf.includes(t), `release.yml does not build napi target ${t}`);
  }
  // The npm publish path must use the napi prepublish flow + a secret token.
  assert.ok(wf.includes('napi prepublish'), 'release.yml missing napi prepublish');
  assert.ok(wf.includes('secrets.NPM_TOKEN'), 'release.yml missing NPM_TOKEN secret gate');
});
