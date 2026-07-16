## Summary

<!-- What does this PR change, and why? -->

## Related issue

Closes #

## Type of change

- [ ] Bug fix
- [ ] New feature / tradition / house system
- [ ] Accuracy / correctness fix
- [ ] Documentation
- [ ] Performance
- [ ] Refactor / internal

## Checklist

- [ ] `cargo test --workspace --exclude xalen-python` passes
- [ ] `cargo clippy --workspace --exclude xalen-python -- -D warnings` is clean
- [ ] `cargo fmt --all --check` is clean
- [ ] New or changed behaviour has tests
- [ ] For any astronomy/astrology math: cross-validated against an authoritative reference (Swiss Ephemeris, JPL DE440, IAU SOFA, or the cited classical text) with the measured error stated
- [ ] Docs updated (rustdoc + `ACCURACY.md` / README where relevant)
- [ ] **No accuracy overclaims** — every "matches X" / "accurate to Y" claim is backed by a committed test. The project's rule is that XALEN **matches** reference ephemerides, never claims to **beat** them.

## Accuracy / reference notes

<!-- If this touches positions, houses, ayanamsa, dasha, etc.: what did you validate
     against, and what was the measured error? -->
