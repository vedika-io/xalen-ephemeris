// XALEN — Node.js quickstart (native N-API addon).
//
// Build the addon first (from the repo root):
//
//   cd crates/xalen-node && npm install && npm run build
//
// then run:
//
//   node examples/node/quickstart.mjs
//
// This uses the *native* `xalen` package. For the WebAssembly build (browser
// or Node), see crates/xalen-wasm and its README.

import { createRequire } from 'node:module';
const require = createRequire(import.meta.url);

// Use the built addon directly from the crate while it is unpublished.
// After `npm install xalen`, this becomes:  const xalen = require('xalen');
const xalen = require('../../crates/xalen-node/index.js');

const J2000 = 2451545.0;

// --- planetPosition: the full 6-tuple + retrograde flag --------------------
const sun = xalen.planetPosition('sun', J2000);
console.log('Sun:',
  `lon=${sun.longitude.toFixed(4)} deg`,
  `speed=${sun.lonSpeed.toFixed(4)} deg/day`,
  `dist=${sun.distance.toFixed(6)} AU`,
  `retrograde=${sun.isRetrograde}`,
);

// --- sidereal by id + Ketu = Rahu + 180 ------------------------------------
const moonSid = xalen.planetPositionById(1, J2000, 0); // Moon, Lahiri (id 0)
const rahu = xalen.planetPositionById(9, J2000);       // mean node (always retrograde)
const ketu = xalen.planetPositionById(13, J2000);      // Rahu + 180
console.log('Moon (sidereal):', moonSid.longitude.toFixed(4), 'deg');
console.log('Rahu retrograde:', rahu.isRetrograde, '| Ketu lon:', ketu.longitude.toFixed(4), 'deg');

// --- nakshatra (structured, unified shape) ---------------------------------
const n = xalen.nakshatraInfo(moonSid.longitude);
console.log(`Moon nakshatra: ${n.name} pada ${n.pada} (lord ${n.lord})`);

// --- full Vedic chart for Pune (18.52 N, 73.85 E) --------------------------
const chart = xalen.fullChart(J2000, 18.52, 73.85, 0);
console.log('Ascendant:', chart.ascendant.toFixed(2), 'deg | MC:', chart.mc.toFixed(2),
  'deg | ayanamsa:', chart.ayanamsaDeg.toFixed(4), 'deg');
for (const body of ['Sun', 'Moon', 'Mars']) {
  const p = chart.planets[body];
  console.log(`  ${body}: ${p.longitude.toFixed(2)} deg  ${p.rashi}  ${p.nakshatra} pada ${p.pada}`);
}

// --- houses (full object with angles) --------------------------------------
const h = xalen.houses(J2000, 18.52, 73.85, 'placidus');
console.log('Placidus ascendant:', h.ascendant.toFixed(2), 'deg, MC:', h.mc.toFixed(2), 'deg',
  h.fallbackUsed ? '(polar fallback used)' : '');

// --- panchang from sidereal Sun + Moon longitudes --------------------------
const pan = xalen.panchang(/*sunLon*/ 90.0, /*moonLon*/ 200.0, J2000);
console.log('Panchang:', pan.tithi_name, `(${pan.paksha})`, '/', pan.nakshatra, '/', pan.yoga_name);

// --- numerology ------------------------------------------------------------
console.log('Life path 1990-03-15:', xalen.lifePath(1990, 3, 15));
console.log('Expression "Ada Lovelace" (pythagorean):', xalen.expressionNumber('Ada Lovelace', 'pythagorean'));
