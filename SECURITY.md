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

Include: affected crate(s) and version, a description, the impact you expect,
and a reproduction if possible (an input file or a short program is ideal).

### What you can expect

- **Acknowledgement** within 72 hours.
- **Triage and severity assessment** within 7 days, including whether we can
  reproduce the issue and which versions it affects.
- **A fix or mitigation plan with a target date** once the issue is confirmed.
  We keep you updated if the date moves.
- **Credit** in the published advisory and in `CHANGELOG.md`, unless you prefer
  to stay anonymous. Tell us how you want to be named.

We follow coordinated disclosure. The default window is 90 days from your
report: we aim to ship a fix and publish an advisory within it, and we ask that
you do not disclose details publicly before then. If a fix needs longer, or the
issue is already being exploited, we will agree a different date with you.

Published advisories are listed at
<https://github.com/vedika-io/xalen-ephemeris/security/advisories>.

### Dependencies

Dependencies are checked against the [RustSec advisory database](https://rustsec.org/)
with `cargo audit` as part of the maintainers' pre-merge checks (the `audit`
stage of `scripts/gate.sh`), and Dependabot security updates are enabled on the
repository. Coordinated-disclosure embargoes from RustSec are honored.

## Scope

XALEN performs offline numerical computation and ships no auth code. By
default the crates perform no network I/O and read only the files the caller
names. Two opt-in cargo features add network access and are off by default:
`kernel-autodownload` in `xalen-ephem` (fetches the public NASA NAIF DE440s
kernel) and `remote-http` in `xalen-cloud`.

The most relevant classes of issue are: parsing untrusted input (notably JPL
`.bsp` kernel files in the optional DE440 reader), integer/float edge cases,
and any `unsafe` in the FFI/WASM bindings. Reports in these areas are
especially welcome.

### Out of scope

- Errors or inaccuracies in third-party ephemeris data files themselves (report
  those to the data provider). A defect in how XALEN reads such a file is in
  scope.
- Denial of service caused by very large inputs on the user's own machine,
  where the caller chose the input.
- Reports from automated scanners with no reproduction or demonstrated impact.
- Accuracy discrepancies against a reference tool. These are ordinary bugs:
  use the bug report form.

## Supply chain

- Releases are built by the maintainers from tagged commits of this repository.
- The `commercial-clean` check verifies that no published crate, built with its
  default features, depends on the non-commercial data crate, so published
  crates carry no non-commercial data.
- Release tags matching `v*` are protected: they cannot be deleted, moved or
  rewritten once created.
- Dependency updates arrive as reviewed pull requests from Dependabot.

We do not currently publish signed releases or build provenance attestations.
