---
name: Bug Report
about: Report incorrect calculations, unexpected behavior, or other issues
title: "[BUG] "
labels: bug
assignees: ''
---

## Expected behavior
<!-- What did you expect to happen? -->


## Actual behavior
<!-- What actually happened? Include error messages or incorrect output. -->


## Astrology system
<!-- Which system were you using? Check all that apply. -->

- [ ] Vedic / Jyotish
- [ ] Western / Tropical
- [ ] KP (Krishnamurti Paddhati)
- [ ] Chinese (BaZi / Four Pillars)
- [ ] Lal Kitab
- [ ] I Ching
- [ ] Numerology
- [ ] Mayan / Tibetan / Korean Saju / Other world system
- [ ] Not system-specific (core ephemeris, coordinates, time, houses)

## Input values
<!-- Provide the exact inputs that reproduce the issue. -->

- **Date/Time:** <!-- e.g., 1990-03-15 14:30:00 UTC -->
- **Location:** <!-- e.g., 18.5204 N, 73.8567 E (Pune, India) -->
- **Ayanamsa (if Vedic):** <!-- e.g., Lahiri, Krishnamurti, Raman -->
- **House system (if applicable):** <!-- e.g., Placidus, Whole Sign, Equal -->
- **Other parameters:** <!-- Any other relevant settings -->

## Reference source
<!-- Which calculator, book, or ephemeris table disagrees with our output? -->

- **Source name:** <!-- e.g., Swiss Ephemeris, Jagannatha Hora, astro.com, BPHS Ch.X -->
- **Source value:** <!-- The value the reference source gives -->
- **XALEN value:** <!-- The value xalen-ephemeris gives -->
- **Difference:** <!-- e.g., 0.5 degrees, wrong nakshatra, off by 1 day -->

## Reproduction
<!-- Minimal code to reproduce. -->

```rust
// Paste minimal code here
```

## Environment
- **xalen-ephemeris version:** <!-- e.g., 0.1.0 -->
- **Rust version:** <!-- output of `rustc --version` -->
- **OS:** <!-- e.g., Ubuntu 24.04, macOS 15, Windows 11 -->
- **Target:** <!-- e.g., native, wasm32-unknown-unknown, Python binding -->

## Additional context
<!-- Any other information that might help diagnose the issue. -->
