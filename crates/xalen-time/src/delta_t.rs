use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Algorithm used to compute delta-T (TT minus UT1) for a given epoch.
pub enum DeltaTModel {
    /// Stephenson, Morrison & Hohenkerk (2016) -- best for broad historical range.
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

/// Compute delta-T with an estimated 1-sigma uncertainty in seconds.
pub fn delta_t_with_uncertainty(jd_ut: f64, model: &DeltaTModel) -> (f64, f64) {
    let dt = delta_t(jd_ut, model);
    let year = jd_to_year(jd_ut);
    let sigma = if year > 1955.0 {
        0.1
    } else if year > 1900.0 {
        1.0
    } else {
        let u = (year - 1820.0) / 100.0;
        0.8 * u * u
    };
    (dt, sigma)
}

fn jd_to_year(jd: f64) -> f64 {
    2000.0 + (jd - 2_451_545.0) / 365.25
}

fn smh2016(jd_ut: f64) -> f64 {
    let year = jd_to_year(jd_ut);

    if year >= 2019.0 {
        // IERS observed + extrapolation
        let t = year - 2019.0;
        69.36 + 0.3 * t + 0.004 * t * t
    } else if year >= 1972.0 {
        // Tabulated IERS values — cubic interpolation of key points
        iers_tabulated(year)
    } else if year >= 1900.0 {
        espenak_meeus_segment(year)
    } else if year >= -720.0 {
        // Stephenson 2016 cubic spline — approximated here with piecewise polynomials
        smh2016_spline(year)
    } else {
        // Parabolic extrapolation for ancient dates
        let u = (year - 1820.0) / 100.0;
        -20.0 + 32.0 * u * u
    }
}

fn iers_tabulated(year: f64) -> f64 {
    // Key IERS observed values (year, delta_T in seconds)
    const TABLE: &[(f64, f64)] = &[
        (1972.0, 42.23),
        (1975.0, 45.48),
        (1980.0, 50.54),
        (1985.0, 54.34),
        (1990.0, 56.86),
        (1995.0, 60.79),
        (2000.0, 63.83),
        (2005.0, 64.69),
        (2010.0, 66.07),
        (2015.0, 68.97),
        (2019.0, 69.36),
    ];

    // Linear interpolation between tabulated points
    if year <= TABLE[0].0 {
        return TABLE[0].1;
    }
    if year >= TABLE[TABLE.len() - 1].0 {
        return TABLE[TABLE.len() - 1].1;
    }

    for i in 0..TABLE.len() - 1 {
        if year >= TABLE[i].0 && year < TABLE[i + 1].0 {
            let frac = (year - TABLE[i].0) / (TABLE[i + 1].0 - TABLE[i].0);
            return TABLE[i].1 + frac * (TABLE[i + 1].1 - TABLE[i].1);
        }
    }
    TABLE[TABLE.len() - 1].1
}

fn smh2016_spline(year: f64) -> f64 {
    // Piecewise polynomial approximation of SMH2016 cubic spline
    // Key control points from the paper
    let u = (year - 1820.0) / 100.0;
    let base = -20.0 + 32.0 * u * u;

    // Corrections to the parabola based on SMH2016 spline deviations
    if year >= 1600.0 {
        espenak_meeus_segment(year)
    } else if year >= 1000.0 {
        let t = (year - 1000.0) / 100.0;
        1574.2 - 556.01 * t + 71.23472 * t * t + 0.319781 * t.powi(3)
            - 0.8503463 * t.powi(4)
            - 0.005050998 * t.powi(5)
            + 0.0083572073 * t.powi(6)
    } else if year >= 500.0 {
        let t = (year - 1000.0) / 100.0;
        1574.2 - 556.01 * t + 71.23472 * t * t + 0.319781 * t.powi(3) - 0.8503463 * t.powi(4)
    } else if year >= -500.0 {
        let t = year / 100.0;
        10583.6 - 1014.41 * t + 33.78311 * t * t - 5.952053 * t.powi(3) - 0.1798452 * t.powi(4)
            + 0.022174192 * t.powi(5)
            + 0.0090316521 * t.powi(6)
    } else {
        base
    }
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
}
