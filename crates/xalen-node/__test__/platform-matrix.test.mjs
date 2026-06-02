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

// The five publishable platform triples. (The Windows triple is MSVC: that is
// what the native windows CI runner builds and publishes; the local
// build-all-platforms.sh produces a *-gnu Windows addon for verification only.)
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
