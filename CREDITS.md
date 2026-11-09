# Credits & References

XALEN Ephemeris implements established astronomical theories and classical
astrological methods. This file attributes the algorithms, models, and data
sources the library is built on.

## Astronomical theories

| Component | Source |
|-----------|--------|
| Planetary positions (Sun, Mercury–Neptune) | **VSOP87A** — Bretagnon, P. & Francou, G. (1988), *Planetary theories in rectangular and spherical variables*, Astron. Astrophys. 202, 309. |
| Moon | **ELP-2000/82** — Chapront-Touzé, M. & Chapront, J. (1983, 1988), the original lunar theory. Coefficients used are the abridged 60+60-term tabulation reproduced as Meeus Tables 47.A/47.B (*Astronomical Algorithms*, 2nd ed., secondary source). |
| Pluto | **Goffin (1989)** analytical fit to JPL DE200, the original source of the 43-term periodic series. Coefficients used are the tabulation reproduced as Meeus Table 37.A (Ch. 37, secondary source). Valid ~1885–2099. |
| Precession | **IAU 2006/P03** — Capitaine, N., Wallace, P. T. & Chapront, J. (2003/2005); Wallace & Capitaine (2006), A&A 459, 981. Fukushima–Williams `pmat06` + ICRS `bp00` frame-bias matrix numerics ported from **ERFA** (liberfa, BSD-3-Clause — the numerical twin of IAU SOFA) and validated element-wise to 1e-12 against the ERFA/SOFA `t_erfa_c.c` golden vectors. The full bias-free P03 rotation matrix (`precession_matrix_p03_nobias`) is wired into the production VSOP87/ELP/Pluto position pipeline — confirmed to machine precision against ERFA/SOFA — superseding the earlier general-precession-in-longitude scalar; see docs/ACCURACY.md. |
| Nutation | **IAU 2000B** — McCarthy, D. D. & Luzum, B. J. (2003); IERS Conventions (2003/2010). The series layout follows the IAU SOFA routine `nut00b`; see the SOFA section of `NOTICE`. |
| ΔT (Terrestrial–Universal Time) | **Stephenson, F. R., Morrison, L. V. & Hohenkerk, C. Y. (2016)**, *Measurement of the Earth's rotation: 720 BC to AD 2015*, Proc. R. Soc. A 472:20160404 — the genuine published Table-S15 cubic *regression* spline (coefficients verbatim) over [−720, AD 2016], with the model's own long-term lod-integral extrapolation tail outside that range. The paper publishes no coefficient covariance matrix, so none is claimed. XALEN's scalar 1-σ ΔT envelope (`delta_t_with_uncertainty`) is our own and uses only public-domain material: the Morrison & Stephenson (2004) parabola and the Huber (2000) Brownian-motion bound as printed on NASA's *Uncertainty in Delta T* page (F. Espenak, NASA/GSFC, eclipse.gsfc.nasa.gov; "Eclipse map/figure/table/predictions courtesy of Fred Espenak, NASA/Goddard Space Flight Center, from eclipse.gsfc.nasa.gov"), and the ΔT error column of the U.S. Naval Observatory file `historic_deltat.data` (McCarthy & Babcock 1986, via maia.usno.navy.mil, Distribution Statement A, 1657–1984), with a floor measured against USNO `deltat.data`. Espenak & Meeus (2006) polynomials and Morrison & Stephenson (2004) are also selectable via `DeltaTModel`. |
| Practical reduction chain (apparent place: light-time, aberration, nutation) | Meeus, J. (1998), *Astronomical Algorithms*, 2nd ed., Willmann-Bell. |
| Lunar eclipses | **Espenak, F. & Meeus, J. (2009)**, *Five Millennium Canon of Lunar Eclipses: −1999 to +3000*, NASA/TP-2009-214172 — umbral and penumbral radii per **Danjon** (1951), `1.01·π☾ ± s☉ + π☉` (ch. 1, eqs. 1-5 and 1-6). Cross-validated against Swiss Ephemeris `swe_lun_eclipse_when` / `swe_lun_eclipse_how` (`crates/xalen-ephem/tests/data/swiss_lunar_eclipse_oracle.py`). |
| Optional high-precision kernel | **JPL DE440** — Park, R. S. et al. (2021), Astron. J. 161, 105. NASA/NAIF SPK (`.bsp`) format. |
| Cross-validation reference | NASA/JPL **Horizons** system (quantity #31, apparent geocentric ecliptic-of-date) and the Swiss Ephemeris `swetest` reference output. |
| Fixed stars (curated ~108) | Astrologically significant set; traditional names, constellations and Ptolemaic natures. Positions and proper motions are replaced by the Bright Star Catalogue values at run time for every star that joins to it. |
| Fixed stars (expanded 9,096) | **Hoffleit D., Warren Jr W.H., The Bright Star Catalogue, 5th Revised Ed. (Preliminary Version), Astronomical Data Center, NSSDC/ADC (1991)**, CDS catalogue **V/50**. Identification (HR, HD, Bayer/Flamsteed), magnitude, proper motion (FK5, J2000) and the position of every star without an IAU name; converted to ecliptic by `tools/gen-star-catalog/gen_bright_catalog.py`. Cross-validated vs Swiss Ephemeris `swe_fixstar2` (38 IAU-named stars within 0.245", median 0.026"). |
| Fixed-star names and positions | **IAU Catalog of Star Names (IAU-CSN)**, IAU WGSN (CC-BY): proper names, constellations and 6-decimal J2000 positions of the 338 named stars that are in the Bright Star Catalogue; the curated names are joined through it. |
| Western star table: nearby stars | **Gliese W., Jahreiss H., Preliminary Version of the Third Catalogue of Nearby Stars (CNS3)**, Astron. Rechen-Institut Heidelberg (1991), CDS catalogue **V/70A**. Magnitude and proper motion of Barnard's Star, Proxima Centauri and Wolf 359, and the position of Wolf 359; before Hipparcos. Used by `tools/gen-star-catalog/regen_western_stars.py`. |
| Western star table: globular cluster | **Harris W.E., A Catalog of Parameters for Globular Clusters in the Milky Way**, Astron. J. 112, 1487 (1996), CDS catalogue **VII/195**. Position and integrated V magnitude of Omega Centauri (NGC 5139). |
| Optional Hipparcos add-on (unpublished) | **The Hipparcos and Tycho Catalogues**, (C) ESA 1997 (ESA SP-1200), CDS **I/239**; CC BY-NC 3.0 IGO. Only in the unpublished `xalen-stars-hip-data` crate, used for cross-validation; no published crate depends on it. |

## Sidereal / Vedic methods

| Component | Source |
|-----------|--------|
| Lahiri (Chitrapaksha) ayanamsa | Calendar Reform Committee of India (1955); N. C. Lahiri, *Indian Ephemeris*. |
| Krishnamurti (KP) ayanamsa | K. S. Krishnamurti, *Krishnamurti Paddhati*. |
| Divisional charts, dashas, yogas, Shadbala, doshas | *Brihat Parashara Hora Shastra* (BPHS); *Phaladeepika* (Mantreswara); *Saravali* (Kalyana Varma); *Jataka Parijata*. |
| Jaimini methods | *Jaimini Sutras*. |
| Tajaka (annual) methods | *Tajaka Neelakanthi* (Neelakantha). |
| Lal Kitab | *Lal Kitab* (Arun Sanhita tradition). |

## Relationship and map methods

| Component | Source |
|-----------|--------|
| Synastry aspect grid, house overlays (`xalen-relationship`) | Hand, R. (1981), *Horoscope Symbols*. |
| Composite charts (`xalen-relationship`) | Hand, R. (1975), *Planets in Composite*. |
| Davison relationship chart (`xalen-relationship`) | Davison, R. C. (1977), *Synastry*. |
| Astrocartography lines (`xalen-maps`) | Lewis, J. (1976), *Astro\*Carto\*Graphy*. |
| Sidereal time, coordinate transformation, rising and setting used by both | Meeus, J., *Astronomical Algorithms*, 2nd ed., ch. 12, 13 and 15. |

## Other systems

Chinese (BaZi / Zi Wei Dou Shu), Korean Saju, Tibetan and Mesoamerican
calendars, numerology, and the I Ching are implemented from their respective
classical and standard references; see each crate's module documentation.

## Software

- The [`vsop87`](https://crates.io/crates/vsop87) crate provides the VSOP87
  series coefficients.
- Cross-checked against the [Swiss Ephemeris](https://www.astro.com/swisseph/)
  (used as an independent reference only — it is **not** a dependency).

## Contributors

Thank you to the people outside the core team whose fixes are in the engine:

- [Kunal Roy Choudhury](https://github.com/kunalgoesbyken) — lunar eclipses
  from the real shadow geometry, and the classification of non-central solar
  eclipses (0.7.0).
- [Vidy Atreya Uppaluru](https://github.com/vauppaluru-ctrl) — the D24
  (Chaturvimsamsa) fix (0.7.0).

If you believe an attribution is missing or incorrect, please open an issue.
