// Regression guard for the Node binding's cross-platform publish matrix.
//
// Does NOT require the compiled addon — it reads package.json, the release build
// script and docs/RELEASING.md so a future edit that drops a target (shipping a
// Node addon for only one platform) is caught by the release-config stage of
// scripts/gate.sh.

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

// The release build script builds the addon for every target in its three
// target arrays; those must be exactly package.json's napi.targets.
function bashArray(text, name) {
  const m = text.match(new RegExp(`^${name}=\\(([^)]*)\\)`, 'm'));
  assert.ok(m, `${name}=(...) not found in scripts/build-release-artifacts.sh`);
  return m[1].trim().split(/\s+/);
}

test('release build script builds every package.json napi target', () => {
  const script = readFileSync(join(REPO_ROOT, 'scripts', 'build-release-artifacts.sh'), 'utf8');
  const built = [
    ...bashArray(script, 'LINUX_TARGETS'),
    ...bashArray(script, 'MACOS_TARGETS'),
    ...bashArray(script, 'WINDOWS_TARGETS'),
  ].sort();
  assert.deepEqual(built, [...(pkg.napi.targets ?? [])].sort(),
    'build-release-artifacts.sh targets differ from package.json napi.targets');
  // The packages are assembled through the napi prepublish flow without
  // publishing anything from the build script.
  assert.ok(script.includes('prepublish -t npm --skip-optional-publish'),
    'build-release-artifacts.sh missing napi prepublish');
});

test('RELEASING.md publishes every platform package, then the root', () => {
  const doc = readFileSync(join(REPO_ROOT, 'docs', 'RELEASING.md'), 'utf8');
  const loop = doc.match(/^for p in ([^;]+); do\n\s*npm publish --access public "dist\/xalen-ephemeris-\$p-X\.Y\.Z\.tgz"/m);
  assert.ok(loop, 'RELEASING.md missing the per-platform npm publish loop');
  const loader = readFileSync(join(NODE_DIR, 'index.js'), 'utf8');
  const platforms = loop[1].trim().split(/\s+/);
  assert.equal(platforms.length, EXPECTED.length, 'publish loop must name all five platform packages');
  for (const p of platforms) {
    assert.ok(loader.includes(`@xalen/ephemeris-${p}`), `index.js does not load @xalen/ephemeris-${p}`);
  }
  const rootAt = doc.indexOf('npm publish --access public dist/xalen-ephemeris-X.Y.Z.tgz');
  assert.ok(rootAt > loop.index, 'the root package must be published after the platform packages');
  assert.ok(doc.includes('NODE_AUTH_TOKEN'), 'RELEASING.md must take the npm token from NODE_AUTH_TOKEN');
});
