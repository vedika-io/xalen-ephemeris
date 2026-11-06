use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Algorithm used to compute delta-T (TT minus UT1) for a given epoch.
pub enum DeltaTModel {
    /// Stephenson, Morrison & Hohenkerk (2016), "Measurement of the Earth's
    /// Rotation: 720 BC to AD 2015" (Proc. R. Soc. A 472:20160404).
    ///
    /// This is the GENUINE published model, not an approximation:
    ///   * `[-720, 2016]` — the published 54-row cubic *regression* spline
    ///     (Table S15 of the supplement; coefficients read verbatim, evaluated
    ///     with the local normalized variable `t = (Y-K_i)/(K_{i+1}-K_i)` and
    ///     `ΔT = a0 + a1·t + a2·t² + a3·t³`).
    ///   * outside `[-720, 2016]` — the model's own long-term extrapolation:
    ///     the integral of the published length-of-day formula
    ///     `lod = 1.72·τ − 3.5·sin(2π·(τ+0.75)/14)` ms/day, `τ = (Y-1825)/100`,
    ///     with the integration constant chosen for continuity at the domain
    ///     boundary. (NOT the §4 discussion parabola, which does not match the
    ///     spline at the boundary and is only a background fit for plotting.)
    ///
    /// The spline's last fitted knot is AD 2016. Beyond it the value is an
    /// extrapolation and `delta_t_with_uncertainty` widens σ accordingly.
    /// The paper publishes no coefficient covariance matrix, so none is
    /// claimed; the scalar per-epoch σ envelope is XALEN's own, assembled from
    /// public-domain sources (see `delta_t_with_uncertainty`, CREDITS.md and
    /// docs/ACCURACY.md).
    StephensonMorrisonHohenkerk2016,
    /// Espenak & Meeus (2006) -- NASA polynomial expressions.
    EspenakMeeus2006,
    /// Morrison & Stephenson (2004) -- simple parabolic model.
    MorrisonStephenson2004,
    /// Always returns zero (no TT-UT1 correction).
    Zero,
}

/// Compute delta-T (TT minus UT1) in seconds for a given Julian Date.
pub fn delta_t(jd_ut: f64, model: &DeltaTModel) -> f64 {
    match model {
        DeltaTModel::StephensonMorrisonHohenkerk2016 => smh2016(jd_ut),
        DeltaTModel::EspenakMeeus2006 => espenak_meeus(jd_ut),
        DeltaTModel::MorrisonStephenson2004 => morrison_stephenson_2004(jd_ut),
        DeltaTModel::Zero => 0.0,
    }
}

/// First (−720.0) and last (2016.0) fitted knots of the SMH2016 cubic spline.
/// Inside this closed interval ΔT is the published regression spline; outside,
/// it is the long-term lod-integral extrapolation. The upper bound also marks
/// where the directly-fitted era ends and the uncertainty begins to grow.
const SMH_FIRST_KNOT_YEAR: f64 = -720.0;
const SMH_LAST_KNOT_YEAR: f64 = 2016.0;

/// Compute delta-T with an estimated 1-sigma uncertainty in seconds.
///
/// The σ returned is **model-specific** — it is NOT a single universal envelope:
///   * [`DeltaTModel::Zero`] — ΔT is identically 0 by construction, so σ = 0.
///     (Returning the SMH envelope here would falsely attribute uncertainty to a
///     value the caller asked to be exactly zero.)
///   * [`DeltaTModel::StephensonMorrisonHohenkerk2016`] — XALEN's own scalar
///     1-σ envelope, built only from public-domain sources (NASA eclipse
///     pages by F. Espenak and USNO Earth-orientation files; see
///     `historical_sigma` in the source for the segments and their citations)
///     and floored at 0.11 s. Past the last fitted knot it is the Huber
///     random-walk bound. This is the only model for which a σ is offered.
///   * [`DeltaTModel::EspenakMeeus2006`] / [`DeltaTModel::MorrisonStephenson2004`]
///     — these polynomial fits ship NO per-epoch σ. We do NOT borrow the SMH
///     envelope (the fits differ from SMH by far more than the SMH σ at many
///     epochs, so the SMH envelope would understate the real disagreement).
///     σ is returned as `f64::NAN` to signal "uncertainty unavailable for this
///     model" rather than a fabricated number.
pub fn delta_t_with_uncertainty(jd_ut: f64, model: &DeltaTModel) -> (f64, f64) {
    let dt = delta_t(jd_ut, model);
    // σ is only offered for the SMH2016 model. Zero has σ = 0 by construction;
    // the polynomial fits carry no per-epoch envelope.
    match model {
        DeltaTModel::Zero => return (dt, 0.0),
        DeltaTModel::EspenakMeeus2006 | DeltaTModel::MorrisonStephenson2004 => {
            return (dt, f64::NAN);
        }
        DeltaTModel::StephensonMorrisonHohenkerk2016 => {}
    }
    let year = jd_to_year(jd_ut);
    let sigma = if year > SMH_LAST_KNOT_YEAR {
        // Past the last fitted spline knot (AD 2016) ΔT is an extrapolation and
        // σ follows Huber's Brownian-motion-with-drift bound calibrated at
        // AD 2005 (≈15.5 s at 2050, ≈47.9 s at 2100), as given on NASA's
        // "Uncertainty in Delta T" page. It grows with the distance from the
        // last observed epoch, so it never understates a trend that fails to
        // hold. The jump from the fitted-era floor to this bound at 2016 is
        // deliberate: it is the step from fitted to extrapolated.
        huber_sigma(year, HUBER_FUTURE_CALIBRATION_YEAR).max(SIGMA_FLOOR_S)
    } else {
        historical_sigma(year)
    };
    (dt, sigma)
}

fn jd_to_year(jd: f64) -> f64 {
    2000.0 + (jd - 2_451_545.0) / 365.25
}

/// Calibration year of Huber's bound for targets after the last fitted knot
/// (NASA, "Uncertainty in Delta T": "the calibration year is 2005 CE for target
/// years in the future").
const HUBER_FUTURE_CALIBRATION_YEAR: f64 = 2005.0;

/// Calibration year of Huber's bound for targets before 500 BCE (same page:
/// "The calibration year is taken as -500 for target years before 500 BCE").
const HUBER_PAST_CALIBRATION_YEAR: f64 = -500.0;

/// Huber's Brownian-motion-with-drift 1-σ uncertainty of ΔT outside the
/// directly observed era, as given on NASA's "Uncertainty in Delta T" page by
/// F. Espenak (<https://eclipse.gsfc.nasa.gov/SEhelp/uncertainty2004.html>,
/// from P. J. Huber, 2000):
///
/// ```text
/// σ = 365.25 · N · sqrt[ (N·Q/3)·(1 + N/M) ] / 1000   seconds
/// ```
///
/// with `N = |target_year − calibration_year|`, `M = 2500` yr and
/// `Q = 0.058 ms²/yr`. NASA's Table 3 lists the results at 500-year steps
/// (e.g. 622 s at −1000, 612 s at 2500, 1885 s at 3000); the tests pin them.
fn huber_sigma(year: f64, calibration_year: f64) -> f64 {
    const M: f64 = 2500.0;
    const Q: f64 = 0.058; // ms² / yr
    let n = (year - calibration_year).abs();
    365.25 * n * ((n * Q / 3.0) * (1.0 + n / M)).sqrt() / 1000.0
}

// ===========================================================================
// Stephenson–Morrison–Hohenkerk (2016) ΔT model
// ===========================================================================
//
// Spline coefficients are taken verbatim from the supplement of
//   Stephenson, Morrison & Hohenkerk (2016), Proc. R. Soc. A 472:20160404,
//   "Table S15: The Polynomial Coefficients for DT -720.0 to 2016.0"
// (figshare dataset 4290866, CC-BY-4.0). They form a least-squares cubic
// *regression* spline: continuity of value / 1st / 2nd derivative at interior
// knots is built into the coefficients, so they are read in mechanically — we
// do NOT re-solve a tridiagonal system. The 54 spline rows are transcribed and
// checked directly against the SMH2016 Table-S15 supplement.

/// Start year `K_i` of each spline interval `[K_i, K_{i+1}]` (row 1 starts at
/// −720; the implied end of the last interval is [`SMH_LAST_KNOT_YEAR`]).
const SMH_KNOTS: [f64; 54] = [
    -720.0, 400.0, 1000.0, 1500.0, 1600.0, 1650.0, 1720.0, 1800.0, 1810.0, 1820.0, 1830.0, 1840.0,
    1850.0, 1855.0, 1860.0, 1865.0, 1870.0, 1875.0, 1880.0, 1885.0, 1890.0, 1895.0, 1900.0, 1905.0,
    1910.0, 1915.0, 1920.0, 1925.0, 1930.0, 1935.0, 1940.0, 1945.0, 1950.0, 1953.0, 1956.0, 1959.0,
    1962.0, 1965.0, 1968.0, 1971.0, 1974.0, 1977.0, 1980.0, 1983.0, 1986.0, 1989.0, 1992.0, 1995.0,
    1998.0, 2001.0, 2004.0, 2007.0, 2010.0, 2013.0,
];

/// End year `K_{i+1}` of each spline interval (parallel to [`SMH_KNOTS`]).
const SMH_KNOTS_END: [f64; 54] = [
    400.0, 1000.0, 1500.0, 1600.0, 1650.0, 1720.0, 1800.0, 1810.0, 1820.0, 1830.0, 1840.0, 1850.0,
    1855.0, 1860.0, 1865.0, 1870.0, 1875.0, 1880.0, 1885.0, 1890.0, 1895.0, 1900.0, 1905.0, 1910.0,
    1915.0, 1920.0, 1925.0, 1930.0, 1935.0, 1940.0, 1945.0, 1950.0, 1953.0, 1956.0, 1959.0, 1962.0,
    1965.0, 1968.0, 1971.0, 1974.0, 1977.0, 1980.0, 1983.0, 1986.0, 1989.0, 1992.0, 1995.0, 1998.0,
    2001.0, 2004.0, 2007.0, 2010.0, 2013.0, 2016.0,
];

/// Cubic coefficients `[a0, a1, a2, a3]` for each interval (parallel to
/// [`SMH_KNOTS`]). `ΔT = a0 + a1·t + a2·t² + a3·t³`, `t = (Y−K_i)/(K_{i+1}−K_i)`.
#[rustfmt::skip]
const SMH_COEFFS: [[f64; 4]; 54] = [
    [20550.593, -21268.478, 11863.418, -4541.129],
    [ 6604.404,  -5981.266,  -505.093,  1349.609],
    [ 1467.654,  -2452.187,  2460.927, -1183.759],
    [  292.635,   -216.322,   -43.614,    56.681],
    [   89.380,    -66.754,    31.607,   -10.497],
    [   43.736,    -49.043,     0.227,    15.811],
    [   10.730,     -1.321,    62.250,   -52.946],
    [   18.714,     -4.457,    -1.509,     2.507],
    [   15.255,      0.046,     6.012,    -4.634],
    [   16.679,     -1.831,    -7.889,     3.799],
    [   10.758,     -6.211,     3.509,    -0.388],
    [    7.668,     -0.357,     2.345,    -0.338],
    [    9.317,      1.659,     0.332,    -0.932],
    [   10.376,     -0.472,    -2.463,     1.596],
    [    9.038,     -0.610,     2.325,    -2.497],
    [    8.256,     -3.450,    -5.166,     2.729],
    [    2.369,     -5.596,     3.020,    -0.919],
    [   -1.126,     -2.312,     0.264,    -0.037],
    [   -3.211,     -1.894,     0.154,     0.562],
    [   -4.388,      0.101,     1.841,    -1.438],
    [   -3.884,     -0.531,    -2.473,     1.870],
    [   -5.017,      0.134,     3.138,    -0.232],
    [   -1.977,      5.715,     2.443,    -1.257],
    [    4.923,      6.828,    -1.329,     0.720],
    [   11.142,      6.330,     0.831,    -0.825],
    [   17.479,      5.518,    -1.643,     0.262],
    [   21.617,      3.020,    -0.856,     0.008],
    [   23.789,      1.333,    -0.831,     0.127],
    [   24.418,      0.052,    -0.449,     0.142],
    [   24.164,     -0.419,    -0.022,     0.702],
    [   24.426,      1.645,     2.086,    -1.106],
    [   27.050,      2.499,    -1.232,     0.614],
    [   28.932,      1.127,     0.220,    -0.277],
    [   30.002,      0.737,    -0.610,     0.631],
    [   30.760,      1.409,     1.282,    -0.799],
    [   32.652,      1.577,    -1.115,     0.507],
    [   33.621,      0.868,     0.406,     0.199],
    [   35.093,      2.275,     1.002,    -0.414],
    [   37.956,      3.035,    -0.242,     0.202],
    [   40.951,      3.157,     0.364,    -0.229],
    [   44.244,      3.198,    -0.323,     0.172],
    [   47.291,      3.069,     0.193,    -0.192],
    [   50.361,      2.878,    -0.384,     0.081],
    [   52.936,      2.354,    -0.140,    -0.166],
    [   54.984,      1.577,    -0.637,     0.448],
    [   56.373,      1.649,     0.709,    -0.277],
    [   58.453,      2.235,    -0.122,     0.111],
    [   60.677,      2.324,     0.212,    -0.315],
    [   62.899,      1.804,    -0.732,     0.112],
    [   64.082,      0.675,    -0.396,     0.193],
    [   64.555,      0.463,     0.184,    -0.008],
    [   65.194,      0.809,     0.161,    -0.101],
    [   66.063,      0.828,    -0.142,     0.168],
    [   66.917,      1.046,     0.360,    -0.282],
];

/// Integration constant making the lod-integral tail continuous with the spline
/// at the lower domain boundary (Y = −720). Equals `spline(−720) − lod(−720)`.
/// Re-derived for the 2016 spline (`spline(−720) = 20550.593`); guarded by
/// `lod_tail_is_continuous_with_spline`.
const SMH_TAIL_C1: f64 = 179.752_739_546_147_5;

/// Integration constant making the lod-integral tail continuous with the spline
/// at the upper domain boundary (Y = 2016). Equals `spline(2016) − lod(2016)`.
const SMH_TAIL_C2: f64 = -151.409_208_815_920_07;

fn smh2016(jd_ut: f64) -> f64 {
    let year = jd_to_year(jd_ut);
    if year < SMH_FIRST_KNOT_YEAR {
        smh_lod_tail(year, SMH_TAIL_C1)
    } else if year > SMH_LAST_KNOT_YEAR {
        smh_lod_tail(year, SMH_TAIL_C2)
    } else {
        smh_spline(year)
    }
}

/// Evaluate the SMH2016 cubic regression spline at `year ∈ [−720, 2016]`.
fn smh_spline(year: f64) -> f64 {
    // Largest interval start `K_i <= year`. Linear scan over 54 short intervals
    // is cheaper than a branchy binary search and keeps the data contiguous.
    let mut i = 0usize;
    while i + 1 < SMH_KNOTS.len() && SMH_KNOTS[i + 1] <= year {
        i += 1;
    }
    let t = (year - SMH_KNOTS[i]) / (SMH_KNOTS_END[i] - SMH_KNOTS[i]);
    let [a0, a1, a2, a3] = SMH_COEFFS[i];
    a0 + t * (a1 + t * (a2 + t * a3)) // Horner
}

/// The SMH long-term extrapolation tail: the analytic integral of the published
/// length-of-day formula `lod = 1.72·τ − 3.5·sin(2π·(τ+0.75)/14)` ms/day, with
/// `τ = (Y − 1825)/100`. Integrating (1 ms = 1e-3 s, 1 Julian yr = 365.25 d):
///
/// ```text
/// ΔT(Y) = C + 31.4115·τ² + (894.8625/π)·cos(2π·(τ+0.75)/14)   seconds
/// ```
///
/// `C` is the integration constant fixing continuity at the relevant domain
/// boundary (`SMH_TAIL_C1` below −720, `SMH_TAIL_C2` above 2016).
fn smh_lod_tail(year: f64, c: f64) -> f64 {
    let tau = 0.01 * (year - 1825.0);
    c + 31.4115 * tau * tau + 284.843_580_525_142_4 * (0.448_798_950_512_827_6 * (tau + 0.75)).cos()
}

/// Smallest 1-σ ΔT uncertainty XALEN reports anywhere (seconds).
///
/// From the start of the atomic-time era the USNO table below quotes
/// measurement errors of a few milliseconds, but the SMH2016 spline is a smooth
/// regression fit and does not follow every wiggle of the measured ΔT. Taking
/// the monthly TT−UT1 values published by USNO (`deltat.data`, 1973–2016) as
/// truth, the spline departs from them with an RMS of 0.1075 s (largest single
/// departure 0.27 s, in 1978). The floor is that RMS rounded up to 0.01 s, so
/// the reported σ describes the model rather than the observations, and it is
/// applied at every epoch so the envelope stays monotone towards the present.
const SIGMA_FLOOR_S: f64 = 0.11;

/// Year at which the parabolic σ of Morrison & Stephenson (2004) is centred:
/// `σ = 0.8·t²` seconds with `t = (year − 1820)/100`.
const MS2004_SIGMA_CENTRE_YEAR: f64 = 1820.0;

/// Earliest year to which the Morrison–Stephenson parabola is applied;
/// earlier epochs use Huber's bound (and never less than the parabola).
const MS2004_SIGMA_FIRST_YEAR: f64 = -1000.0;

/// Plateau σ (seconds) from the "decade fluctuations" in ΔT over 1300–1600 CE
/// (NASA: "approximately 20 seconds"). The parabola falls to exactly this value
/// at `1820 − 500 = 1320`, so a `max` of the two joins continuously.
const DECADE_FLUCTUATION_SIGMA_S: f64 = 20.0;

/// First epoch of the USNO telescopic-era error table, [`USNO_SIGMA_STEPS`].
/// The plateau above is held up to here.
const TELESCOPIC_ERA_START_YEAR: f64 = 1657.0;

/// Morrison & Stephenson (2004) standard error of ΔT, as quoted on NASA's
/// "Uncertainty in Delta T" page (Table 1: 636 s at −1000, 265 s at 0,
/// 54 s at 1000, 31 s at 1200): `σ = 0.8·((year − 1820)/100)²` seconds.
fn parabolic_sigma(year: f64) -> f64 {
    let t = (year - MS2004_SIGMA_CENTRE_YEAR) / 100.0;
    0.8 * t * t
}

/// Telescopic-era ΔT errors from USNO's "Historic Delta T and LOD values from
/// McCarthy and Babcock (1986)" (`historic_deltat.data`, column "error" of
/// TDT−UT1, half-yearly rows 1657.0 to 1984.5), reduced to a non-increasing
/// step function: for any year `y`, the entry `(row_year, error)` is the first
/// one with `row_year ≥ y`, and `error` is the largest error in the USNO file
/// at every row dated `y` or later. Entries are those rows whose error strictly
/// exceeds the error of every later row (the last such row where values tie),
/// copied verbatim; the unit test `usno_steps_pin_the_source_rows` keeps them
/// tied to the source lines. The envelope can therefore only shrink towards
/// the present, never understates any later USNO row, and the first entry
/// (19 s) is the file's overall maximum.
#[rustfmt::skip]
const USNO_SIGMA_STEPS: [(f64, f64); 58] = [
    (1694.0, 19.0), (1695.0, 18.0), (1696.0, 17.0), (1696.5, 16.0),
    (1697.5, 15.0), (1698.5, 14.0), (1700.5, 12.0), (1704.5, 11.0),
    (1716.5, 10.0), (1726.5, 9.0), (1728.5, 6.3), (1742.5, 5.9),
    (1743.0, 5.8), (1743.5, 5.6), (1744.0, 5.2), (1744.5, 4.8),
    (1750.5, 4.6), (1751.0, 4.5), (1751.5, 4.3), (1752.0, 4.0),
    (1752.5, 3.8), (1761.0, 3.6), (1816.0, 3.5), (1817.0, 3.0),
    (1818.0, 2.9), (1819.0, 2.8), (1819.5, 2.2), (1820.0, 2.15),
    (1821.0, 1.86), (1833.0, 1.59), (1834.0, 1.56), (1835.0, 1.27),
    (1835.5, 0.97), (1853.0, 0.87), (1854.0, 0.84), (1855.0, 0.81),
    (1856.0, 0.74), (1859.0, 0.71), (1860.0, 0.70), (1860.5, 0.51),
    (1861.0, 0.49), (1861.5, 0.44), (1873.5, 0.38), (1874.0, 0.31),
    (1876.5, 0.25), (1920.5, 0.22), (1921.0, 0.17), (1921.5, 0.15),
    (1922.0, 0.11), (1926.5, 0.09), (1927.0, 0.07), (1927.5, 0.05),
    (1939.5, 0.04), (1940.0, 0.03), (1945.5, 0.02), (1955.5, 0.01),
    (1961.5, 0.003), (1984.5, 0.001),
];

/// XALEN's 1-σ ΔT uncertainty envelope (seconds) for `year ≤ 2016`, with every
/// segment taken from a source that allows reuse:
///
/// | years                | σ                                                     | source |
/// |----------------------|-------------------------------------------------------|--------|
/// | below −1000          | max(Huber bound, parabola) (calibration year −500)    | NASA, Uncertainty in Delta T (Huber 2000; Morrison & Stephenson 2004) |
/// | −1000 to 1657        | parabola `0.8·((Y−1820)/100)²`, at least 20 s         | same page: Table 1 and the 1300–1600 "approximately 20 seconds" |
/// | 1657 to 1984.5       | non-increasing step of the USNO error column          | USNO `historic_deltat.data` |
/// | after 1984.5         | the floor                                             | derived against USNO `deltat.data` |
///
/// and never below [`SIGMA_FLOOR_S`]. The pieces join continuously or step
/// down towards the present, so the envelope never rises with the year. Years
/// after the last fitted knot use [`huber_sigma`] directly, see
/// [`delta_t_with_uncertainty`].
///
/// The Huber and parabolic expressions are published mathematics; the NASA
/// page is a US Government work (<https://eclipse.gsfc.nasa.gov/SEpubs/copyright.html>:
/// "NASA material is not protected by copyright unless noted") and the USNO
/// files carry "Distribution Statement A. Approved for public release:
/// distribution unlimited." (<https://maia.usno.navy.mil/products/deltaT>).
fn historical_sigma(year: f64) -> f64 {
    let envelope = if year < MS2004_SIGMA_FIRST_YEAR {
        huber_sigma(year, HUBER_PAST_CALIBRATION_YEAR).max(parabolic_sigma(year))
    } else if year < TELESCOPIC_ERA_START_YEAR {
        parabolic_sigma(year).max(DECADE_FLUCTUATION_SIGMA_S)
    } else {
        USNO_SIGMA_STEPS
            .iter()
            .find(|&&(row_year, _)| row_year >= year)
            .map_or(0.0, |&(_, error)| error)
    };
    envelope.max(SIGMA_FLOOR_S)
}

fn espenak_meeus(jd_ut: f64) -> f64 {
    let year = jd_to_year(jd_ut);
    espenak_meeus_segment(year)
}

fn espenak_meeus_segment(year: f64) -> f64 {
    if year >= 2015.0 {
        let t = year - 2015.0;
        67.62 + 0.3645 * t + 0.0039755 * t * t
    } else if year >= 2005.0 {
        let t = year - 2005.0;
        64.69 + 0.2930 * t
    } else if year >= 1986.0 {
        let t = year - 2000.0;
        63.86 + 0.3345 * t - 0.060374 * t * t
            + 0.0017275 * t.powi(3)
            + 0.000651814 * t.powi(4)
            + 0.00002373599 * t.powi(5)
    } else if year >= 1961.0 {
        let t = year - 1975.0;
        45.45 + 1.067 * t - t * t / 260.0 - t.powi(3) / 718.0
    } else if year >= 1941.0 {
        let t = year - 1950.0;
        29.07 + 0.407 * t - t * t / 233.0 + t.powi(3) / 2547.0
    } else if year >= 1920.0 {
        let t = year - 1920.0;
        21.20 + 0.84493 * t - 0.076100 * t * t + 0.0020936 * t.powi(3)
    } else if year >= 1900.0 {
        let t = year - 1900.0;
        -2.79 + 1.494119 * t - 0.0598939 * t * t + 0.0061966 * t.powi(3) - 0.000197 * t.powi(4)
    } else if year >= 1860.0 {
        let t = year - 1860.0;
        7.62 + 0.5737 * t - 0.251754 * t * t + 0.01680668 * t.powi(3) - 0.0004473624 * t.powi(4)
            + t.powi(5) / 233174.0
    } else if year >= 1800.0 {
        let t = year - 1800.0;
        13.72 - 0.332447 * t + 0.0068612 * t * t + 0.0041116 * t.powi(3) - 0.00037436 * t.powi(4)
            + 0.0000121272 * t.powi(5)
            - 0.0000001699 * t.powi(6)
            + 0.000000000875 * t.powi(7)
    } else if year >= 1700.0 {
        let t = year - 1700.0;
        8.83 + 0.1603 * t - 0.0059285 * t * t + 0.00013336 * t.powi(3) - t.powi(4) / 1174000.0
    } else if year >= 1600.0 {
        let t = year - 1600.0;
        120.0 - 0.9808 * t - 0.01532 * t * t + t.powi(3) / 7129.0
    } else if year >= 500.0 {
        let t = (year - 1000.0) / 100.0;
        1574.2 - 556.01 * t + 71.23472 * t * t + 0.319781 * t.powi(3)
            - 0.8503463 * t.powi(4)
            - 0.005050998 * t.powi(5)
            + 0.0083572073 * t.powi(6)
    } else if year >= -500.0 {
        let t = year / 100.0;
        10583.6 - 1014.41 * t + 33.78311 * t * t - 5.952053 * t.powi(3) - 0.1798452 * t.powi(4)
            + 0.022174192 * t.powi(5)
            + 0.0090316521 * t.powi(6)
    } else {
        let u = (year - 1820.0) / 100.0;
        -20.0 + 32.0 * u * u
    }
}

fn morrison_stephenson_2004(jd_ut: f64) -> f64 {
    let year = jd_to_year(jd_ut);
    let u = (year - 1820.0) / 100.0;
    -20.0 + 32.0 * u * u
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Inverse of `jd_to_year` for constructing test epochs from a calendar year.
    fn year_to_jd(year: f64) -> f64 {
        2_451_545.0 + (year - 2000.0) * 365.25
    }

    #[test]
    fn delta_t_j2000_is_about_64_seconds() {
        let jd = 2_451_545.0;
        let dt = delta_t(jd, &DeltaTModel::StephensonMorrisonHohenkerk2016);
        assert!(
            (dt - 63.83).abs() < 2.0,
            "delta-T at J2000 should be ~63.83s, got {dt}"
        );
    }

    #[test]
    fn delta_t_ancient_is_large() {
        let jd = 2_451_545.0 - 365.25 * 3000.0; // ~1000 BCE
        let dt = delta_t(jd, &DeltaTModel::StephensonMorrisonHohenkerk2016);
        assert!(
            dt > 20000.0,
            "delta-T at 1000 BCE should be > 20000s, got {dt}"
        );
    }

    #[test]
    fn delta_t_with_uncertainty_grows_for_ancient() {
        let modern = 2_451_545.0;
        let ancient = 2_451_545.0 - 365.25 * 3000.0;
        let (_, sigma_modern) =
            delta_t_with_uncertainty(modern, &DeltaTModel::StephensonMorrisonHohenkerk2016);
        let (_, sigma_ancient) =
            delta_t_with_uncertainty(ancient, &DeltaTModel::StephensonMorrisonHohenkerk2016);
        assert!(
            sigma_ancient > sigma_modern * 100.0,
            "Ancient uncertainty should be >> modern"
        );
    }

    #[test]
    fn zero_model_returns_zero() {
        let dt = delta_t(2_451_545.0, &DeltaTModel::Zero);
        assert_eq!(dt, 0.0);
    }

    /// C10: σ must follow the SELECTED model, not silently borrow the SMH
    /// envelope. `Zero` is exactly zero (σ = 0); the polynomial fits publish no
    /// per-epoch envelope (σ = NaN = "unavailable"); only SMH2016 has a real σ.
    #[test]
    fn uncertainty_is_model_specific_not_borrowed_from_smh() {
        // Probe an ancient epoch and a modern epoch — both must obey the rule.
        for jd in [2_451_545.0, 2_451_545.0 - 365.25 * 3000.0] {
            // Zero: ΔT == 0 AND σ == 0 (NOT the SMH envelope of ~516 s at −720).
            let (dt0, s0) = delta_t_with_uncertainty(jd, &DeltaTModel::Zero);
            assert_eq!(dt0, 0.0, "Zero ΔT must be 0 at JD {jd}");
            assert_eq!(
                s0, 0.0,
                "Zero σ must be 0, not the SMH envelope, at JD {jd}"
            );

            // Espenak–Meeus and Morrison–Stephenson: no per-epoch σ offered → NaN.
            for model in [
                DeltaTModel::EspenakMeeus2006,
                DeltaTModel::MorrisonStephenson2004,
            ] {
                let (_, s) = delta_t_with_uncertainty(jd, &model);
                assert!(
                    s.is_nan(),
                    "σ for {model:?} must be NaN (unavailable), got {s} at JD {jd}"
                );
            }

            // SMH2016 keeps a real, finite, positive σ.
            let (_, s_smh) =
                delta_t_with_uncertainty(jd, &DeltaTModel::StephensonMorrisonHohenkerk2016);
            assert!(
                s_smh.is_finite() && s_smh > 0.0,
                "SMH σ must be finite and positive, got {s_smh} at JD {jd}"
            );
        }
    }

    // -----------------------------------------------------------------------
    // SMH2016 spline: validation against the paper's own published ΔT values.
    // -----------------------------------------------------------------------

    /// The genuine SMH2016 spline must reproduce the published Table-S15 ΔT
    /// values (the spline's own evaluation; transcription + Horner check). These
    /// are the values the 2016 paper reports for the historical fitted range.
    #[test]
    fn smh_spline_matches_published_values() {
        let model = DeltaTModel::StephensonMorrisonHohenkerk2016;
        // (year, published ΔT seconds) — from the SMH2016 Table-S15 spline.
        let cases = [
            (-720.0, 20550.593),
            (-500.0, 16796.179),
            (0.0, 10574.295),
            (500.0, 5599.744),
            (1000.0, 1467.654),
            (1500.0, 292.635),
            (1600.0, 89.380),
            (1800.0, 18.714),
            (1900.0, -1.977),
            (2000.0, 63.810),
            (2016.0, 68.041),
        ];
        for (year, expected) in cases {
            let got = delta_t(year_to_jd(year), &model);
            assert!(
                (got - expected).abs() < 0.05,
                "ΔT(SMH spline) at {year}: expected {expected:.3} s, got {got:.3} s"
            );
        }
    }

    /// The SMH spline is fit to the same IERS data Swiss/JPL use, so in the
    /// telescopic era it must track observed ΔT to within the fit residual.
    #[test]
    fn smh_spline_tracks_iers_modern_era() {
        let model = DeltaTModel::StephensonMorrisonHohenkerk2016;
        // (year, IERS observed ΔT seconds, tolerance = fit residual).
        let cases = [
            (1950.0, 29.15, 0.4),
            (1980.0, 50.54, 0.4),
            (2000.0, 63.83, 0.1),
            (2010.0, 66.07, 0.1),
        ];
        for (year, observed, tol) in cases {
            let got = delta_t(year_to_jd(year), &model);
            assert!(
                (got - observed).abs() < tol,
                "ΔT at {year}: observed {observed:.2} s, spline {got:.3} s (tol {tol})"
            );
        }
    }

    /// The lod-integral extrapolation tail must join the spline continuously at
    /// both domain boundaries — this is what `SMH_TAIL_C1`/`SMH_TAIL_C2` encode,
    /// so the test guards those constants against silent drift.
    #[test]
    fn lod_tail_is_continuous_with_spline() {
        // Lower boundary, Y = −720.
        let spline_lo = smh_spline(SMH_FIRST_KNOT_YEAR);
        let tail_lo = smh_lod_tail(SMH_FIRST_KNOT_YEAR, SMH_TAIL_C1);
        assert!(
            (spline_lo - tail_lo).abs() < 1e-6,
            "lod tail discontinuous at −720: spline={spline_lo}, tail={tail_lo}"
        );
        // Upper boundary, Y = 2016.
        let spline_hi = smh_spline(SMH_LAST_KNOT_YEAR);
        let tail_hi = smh_lod_tail(SMH_LAST_KNOT_YEAR, SMH_TAIL_C2);
        assert!(
            (spline_hi - tail_hi).abs() < 1e-6,
            "lod tail discontinuous at 2016: spline={spline_hi}, tail={tail_hi}"
        );
    }

    /// The whole ΔT function must be continuous across every interior point,
    /// including the 500 CE region (interior to spline row [400, 1000]) which a
    /// previous polynomial-hybrid implementation discontinuously approximated.
    #[test]
    fn delta_t_continuous_across_domain() {
        let model = DeltaTModel::StephensonMorrisonHohenkerk2016;
        for &year in &[-720.0, -500.0, 500.0, 1000.0, 1500.0, 1900.0, 2016.0] {
            let below = delta_t(year_to_jd(year - 0.001), &model);
            let above = delta_t(year_to_jd(year + 0.001), &model);
            let jump = (above - below).abs();
            assert!(
                jump < 0.5,
                "ΔT must be continuous across {year}: jump of {jump} s \
                 (below={below}, above={above})"
            );
        }
    }

    // P1-17: past the last fitted knot (2016) ΔT uncertainty must grow rather
    // than stay pinned at the observed-era 0.1 s.
    #[test]
    fn future_uncertainty_grows_beyond_observed() {
        let model = DeltaTModel::StephensonMorrisonHohenkerk2016;
        let (_, sigma_2010) = delta_t_with_uncertainty(year_to_jd(2010.0), &model);
        let (_, sigma_2100) = delta_t_with_uncertainty(year_to_jd(2100.0), &model);
        assert_eq!(
            sigma_2010, SIGMA_FLOOR_S,
            "2010 is within the fitted era; σ sits at the floor, got {sigma_2010}"
        );
        // Beyond 2016 we report the Huber random-walk bound (calibration year
        // 2005): ≈47.9 s at 2100 and ≈15.5 s at 2050. A flat floor or the
        // Morrison-Stephenson parabola (≈6.3 s at 2100) would understate this.
        assert!(
            (sigma_2100 - 47.9).abs() < 1.0,
            "2100 σ should match the Huber bound ≈47.9 s, got {sigma_2100}"
        );
        let (_, sigma_2050) = delta_t_with_uncertainty(year_to_jd(2050.0), &model);
        assert!(
            (sigma_2050 - 15.5).abs() < 1.0,
            "2050 σ should match the Huber bound ≈15.5 s, got {sigma_2050}"
        );
    }

    fn sigma_at(year: f64) -> f64 {
        delta_t_with_uncertainty(
            year_to_jd(year),
            &DeltaTModel::StephensonMorrisonHohenkerk2016,
        )
        .1
    }

    /// The σ envelope must reproduce the numbers printed on NASA's
    /// "Uncertainty in Delta T" page (F. Espenak; US Government work), which
    /// are the source of the parabola and the Huber bound. Table values are
    /// rounded to whole seconds on the page, hence the half-second tolerance.
    #[test]
    fn sigma_matches_nasa_published_tables() {
        // Table 1 (−1000 to 1200): σ = 0.8·((year − 1820)/100)².
        for (year, nasa) in [
            (-1000.0, 636.0),
            (-500.0, 431.0),
            (0.0, 265.0),
            (500.0, 139.0),
            (1000.0, 54.0),
            (1200.0, 31.0),
        ] {
            let got = sigma_at(year);
            assert!(
                (got - nasa).abs() <= 0.5,
                "σ({year}) = {got}, NASA Table 1 prints {nasa}"
            );
        }
        // Table 3 (Huber bound, calibration −500 before 500 BCE, +2005 after
        // the last fitted knot). −1000 is printed as 622 s there; our envelope
        // takes the larger parabola (636 s) at that epoch, so it is checked
        // against the Huber function itself.
        assert!((huber_sigma(-1000.0, HUBER_PAST_CALIBRATION_YEAR) - 622.0).abs() <= 0.5);
        for (year, nasa) in [
            (-4000.0, 16291.0),
            (-3500.0, 12378.0),
            (-3000.0, 8978.0),
            (-2500.0, 6094.0),
            (-2000.0, 3732.0),
            (-1500.0, 1900.0),
            (2500.0, 612.0),
            (3000.0, 1885.0),
            (3500.0, 3711.0),
            (4000.0, 6068.0),
            (4500.0, 8946.0),
            (5000.0, 12341.0),
        ] {
            let got = sigma_at(year);
            assert!(
                (got - nasa).abs() <= 0.5,
                "σ({year}) = {got}, NASA Table 3 prints {nasa}"
            );
        }
        // Table 2 (Stephenson & Houlden 1986) is a lower reference for the
        // telescopic era: the envelope must not fall below it.
        for (year, nasa) in [(1700.0, 5.0), (1800.0, 1.0), (1900.0, 0.1)] {
            assert!(
                sigma_at(year) >= nasa,
                "σ({year}) = {} understates NASA Table 2 ({nasa})",
                sigma_at(year)
            );
        }
        // About 20 s over 1300–1600 ("approximately 20 seconds").
        for year in [1300.0, 1450.0, 1600.0] {
            assert!(
                (sigma_at(year) - 20.0).abs() <= 1.7,
                "σ({year}) = {}",
                sigma_at(year)
            );
        }
    }

    /// Every value in [`USNO_SIGMA_STEPS`] is pinned to the source line of
    /// USNO's `historic_deltat.data` it was copied from (columns: year,
    /// TDT−UT1, error, LOD, LOD error). The lines below are those rows
    /// verbatim, so editing the table without the source fails here.
    #[test]
    fn usno_steps_pin_the_source_rows() {
        const SOURCE_ROWS: [&str; 58] = [
            "1694.000     21       19        -1       74",
            "1695.000     21       18         0       71",
            "1696.000     20       17         0       67",
            "1696.500     20       16         0       65",
            "1697.500     20       15         0       59",
            "1698.500     20       14         0       55",
            "1700.500     21       12         0       38",
            "1704.500     19       11        -1       43",
            "1716.500     21       10         1       35",
            "1726.500     19.1      9.0      -2.1     33.9",
            "1728.500     17.4      6.3      -1.8     20.2",
            "1742.500     13.4      5.9      -0.2     22.2",
            "1743.000     13.3      5.8      -0.2     22.7",
            "1743.500     13.3      5.6      -0.2     22.6",
            "1744.000     13.2      5.2      -0.2     21.7",
            "1744.500     13.2      4.8      -0.2     20.3",
            "1750.500     13.8      4.6       0.4     17.8",
            "1751.000     13.9      4.5       0.4     17.9",
            "1751.500     14.0      4.3       0.3     17.4",
            "1752.000     14.0      4.0       0.3     16.6",
            "1752.500     14.1      3.8       0.2     15.6",
            "1761.000     14.9      3.6       0.3     13.3",
            "1816.000     11.6      3.5      -0.4     12.2",
            "1817.000     11.5      3.0      -0.3      9.3",
            "1818.000     11.4      2.9      -0.2      9.2",
            "1819.000     11.3      2.8      -0.4      8.9",
            "1819.500     11.3      2.2      -0.3     10.8",
            "1820.000     11.13     2.15     -0.48     8.61",
            "1821.000     10.94     1.86     -1.48     6.21",
            "1833.000      7.10     1.59     -0.64     4.93",
            "1834.000      6.89     1.56     -0.49     4.91",
            "1835.000      6.73     1.27     -0.80     4.84",
            "1835.500      6.64     0.97     -1.11     4.92",
            "1853.000      7.03     0.87      0.42     2.37",
            "1854.000      7.15     0.84      0.26     2.77",
            "1855.000      7.26     0.81      0.09     2.55",
            "1856.000      7.23     0.74     -0.14     2.50",
            "1859.000      7.19     0.71      0.73     1.08",
            "1860.000      7.35     0.70      0.12     2.36",
            "1860.500      7.39     0.51      0.19     2.70",
            "1861.000      7.41     0.49      0.07     1.97",
            "1861.500      7.45     0.44     -0.41     1.89",
            "1873.500     -2.08     0.38     -2.10     1.16",
            "1874.000     -2.48     0.31     -1.96     1.48",
            "1876.500     -4.14     0.25     -1.45     0.69",
            "1920.500     21.78     0.22      1.64     0.70",
            "1921.000     22.06     0.17      1.21     0.84",
            "1921.500     22.30     0.15      1.26     0.67",
            "1922.000     22.51     0.11      1.40     0.58",
            "1926.500     24.25     0.09      0.88     0.22",
            "1927.000     24.39     0.07      0.01     0.34",
            "1927.500     24.42     0.05     -0.27     0.29",
            "1939.500     24.20     0.04      0.95     0.11",
            "1940.000     24.35     0.03      1.29     0.08",
            "1945.500     27.04     0.02      1.29     0.85",
            "1955.500     31.24     0.01      1.37     0.01",
            "1961.500     33.804    0.003     1.29     0.10",
            "1984.500     54.087    0.001     1.45     0.04",
        ];
        assert_eq!(SOURCE_ROWS.len(), USNO_SIGMA_STEPS.len());
        for (line, &(year, error)) in SOURCE_ROWS.iter().zip(USNO_SIGMA_STEPS.iter()) {
            let cols: Vec<&str> = line.split_whitespace().collect();
            assert_eq!(cols.len(), 5, "malformed source row {line:?}");
            let src_year: f64 = cols[0].parse().unwrap();
            let src_error: f64 = cols[2].parse().unwrap();
            assert_eq!(year, src_year, "year differs from source row {line:?}");
            assert_eq!(error, src_error, "error differs from source row {line:?}");
        }
        // Shape of the reduction: rows on the half-year grid of the source,
        // later rows strictly smaller (so each kept row exceeds every later one).
        for pair in USNO_SIGMA_STEPS.windows(2) {
            assert!(pair[0].0 < pair[1].0 && pair[0].1 > pair[1].1);
        }
        for &(year, _) in &USNO_SIGMA_STEPS {
            assert!((TELESCOPIC_ERA_START_YEAR..=1984.5).contains(&year));
            assert_eq!(
                (year * 2.0).fract(),
                0.0,
                "{year} is not on the half-year grid"
            );
        }
        // The overall maximum of the file's error column is the first entry.
        assert_eq!(USNO_SIGMA_STEPS[0].1, 19.0);
    }

    /// Reference values of the envelope (seconds) at representative epochs, so
    /// a change to any segment is visible in review.
    #[test]
    fn sigma_envelope_reference_values() {
        for (year, expected) in [
            (-2000.0, 3731.99),
            (-1000.0, 636.19),
            (-720.0, 516.13),
            (0.0, 264.99),
            (500.0, 139.39),
            (1000.0, 53.79),
            (1600.0, 20.0),
            (1700.0, 12.0),
            (1800.0, 3.5),
            (1900.0, 0.22),
            (1962.0, SIGMA_FLOOR_S),
            (2000.0, SIGMA_FLOOR_S),
            (2015.999, SIGMA_FLOOR_S),
            (2026.0, 4.91),
            (2050.0, 15.47),
            (2100.0, 47.91),
            (2500.0, 612.18),
            (3000.0, 1884.66),
        ] {
            let got = sigma_at(year);
            assert!(
                (got - expected).abs() < 0.01,
                "σ({year}) = {got}, expected {expected}"
            );
        }
    }

    /// The envelope is finite and positive everywhere, never rises towards the
    /// present while the model is fitted to data (≤ 2016), never falls once it
    /// is extrapolated (> 2016), and joins its segments without an upward step.
    #[test]
    fn sigma_envelope_is_positive_monotone_and_joined() {
        let mut year = -50_000.0;
        let mut previous = f64::INFINITY;
        while year < 2016.0 {
            let sigma = sigma_at(year);
            assert!(
                sigma.is_finite() && sigma >= SIGMA_FLOOR_S,
                "σ({year}) = {sigma}"
            );
            assert!(
                sigma <= previous,
                "σ rises at {year}: {previous} -> {sigma}"
            );
            previous = sigma;
            year += 0.25;
        }
        let mut previous = sigma_at(2016.001);
        let mut year = 2016.25;
        while year <= 50_000.0 {
            let sigma = sigma_at(year);
            assert!(
                sigma.is_finite() && sigma >= previous,
                "σ({year}) = {sigma}"
            );
            previous = sigma;
            year += 7.25;
        }
        // Continuous where the formulas meet: −1000 (Huber/parabola) and 1320
        // (parabola/20 s plateau).
        for year in [MS2004_SIGMA_FIRST_YEAR, 1320.0] {
            let jump = (sigma_at(year - 1e-6) - sigma_at(year + 1e-6)).abs();
            assert!(jump < 1e-3, "σ jumps by {jump} at {year}");
        }
        // Stepwise-conservative where the USNO table starts (20 s -> 19 s) and
        // where it ends (0.001 s -> floor): never larger after the step.
        assert!(sigma_at(TELESCOPIC_ERA_START_YEAR - 1e-6) >= sigma_at(TELESCOPIC_ERA_START_YEAR));
        // Past the last fitted knot the bound takes over from the floor.
        assert!(sigma_at(SMH_LAST_KNOT_YEAR + 1e-3) > sigma_at(SMH_LAST_KNOT_YEAR - 1e-3));
        // Extremes stay finite.
        for year in [-1.0e9, -1.0e6, 1.0e6, 1.0e9] {
            let sigma = sigma_at(year);
            assert!(sigma.is_finite() && sigma > 0.0, "σ({year}) = {sigma}");
        }
    }
}
