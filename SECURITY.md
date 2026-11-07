# Security Policy

## Supported versions

XALEN Ephemeris is pre-1.0. Security fixes are applied to the latest published
`0.x` release on crates.io.

| Version | Supported |
|---------|-----------|
| latest 0.x | ✅ |
| older | ❌ |

## Reporting a vulnerability

**Do not open a public issue for security reports.**

Please report privately via GitHub's
[private vulnerability reporting](https://github.com/vedika-io/xalen-ephemeris/security/advisories/new)
or by email to **security@vedika.io**.

Include: affected crate(s) and version, a description, and a reproduction if
possible. We aim to acknowledge within 72 hours and to provide a remediation
timeline after triage.

For dependency vulnerabilities, the `audit` stage of `scripts/gate.sh` (`cargo audit`) tracks the
[RustSec advisory database](https://rustsec.org/); coordinated-disclosure
embargoes from RustSec are honored.

## Scope

XALEN performs offline numerical computation and ships no network or auth code.
The most relevant classes are: parsing untrusted input (notably JPL `.bsp`
kernel files in the optional DE440 reader), integer/float edge cases, and any
`unsafe` in the FFI/WASM bindings. Reports in these areas are especially
welcome.
