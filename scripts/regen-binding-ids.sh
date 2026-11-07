#!/usr/bin/env bash
# Regenerate the generated tables in docs/BINDING_IDS.md from the Rust sources.
#
# The drift test tests/binding_ids.rs renders the tables and, with
# XALEN_REGEN_BINDING_IDS set, writes them into the document instead of
# failing. Review the diff and commit it with the source change.
set -euo pipefail

cd "$(dirname "$0")/.."
XALEN_REGEN_BINDING_IDS=1 cargo test -p xalen-ephemeris --test binding_ids
