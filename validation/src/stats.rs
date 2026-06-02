//! Per-quantity delta accumulation: max / mean / RMS absolute error in arcsec,
//! the worst-offending chart, and counts against threshold buckets.

/// Smallest unsigned separation between two angles in degrees, robust across
/// the 0deg/360deg wrap.
pub fn angle_sep_deg(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
}

/// Running accumulator for one quantity's absolute angular error.
#[derive(Debug, Clone, Default)]
pub struct DeltaStats {
    pub count: u64,
    pub sum_arcsec: f64,
    pub sum_sq_arcsec: f64,
    pub max_arcsec: f64,
    /// (jd, lat, lon) of the worst chart seen so far.
    pub worst_chart: Option<(f64, f64, f64)>,
    /// number of samples whose error exceeded the legacy 0.1deg (360") bound.
    pub over_legacy: u64,
    /// number of samples within the tight 1" subset.
    pub within_1_arcsec: u64,
}

impl DeltaStats {
    /// Record one delta given in degrees together with the chart that produced it.
    pub fn record_deg(&mut self, delta_deg: f64, jd: f64, lat: f64, lon: f64) {
        let arcsec = delta_deg * 3600.0;
        self.count += 1;
        self.sum_arcsec += arcsec;
        self.sum_sq_arcsec += arcsec * arcsec;
        if arcsec > self.max_arcsec {
            self.max_arcsec = arcsec;
            self.worst_chart = Some((jd, lat, lon));
        }
        if delta_deg > 0.1 {
            self.over_legacy += 1;
        }
        if arcsec <= 1.0 {
            self.within_1_arcsec += 1;
        }
    }

    pub fn mean_arcsec(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum_arcsec / self.count as f64
        }
    }

    pub fn rms_arcsec(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            (self.sum_sq_arcsec / self.count as f64).sqrt()
        }
    }

    pub fn within_1_arcsec_frac(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.within_1_arcsec as f64 / self.count as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angle_sep_wraps() {
        assert!((angle_sep_deg(359.9, 0.1) - 0.2).abs() < 1e-12);
        assert!((angle_sep_deg(10.0, 10.0)).abs() < 1e-12);
        assert!((angle_sep_deg(0.0, 180.0) - 180.0).abs() < 1e-12);
        // separation is always <= 180
        assert!(angle_sep_deg(0.0, 270.0) <= 180.0 + 1e-9);
    }

    #[test]
    fn stats_accumulate() {
        let mut s = DeltaStats::default();
        // 1 arcsec = 1/3600 deg
        s.record_deg(1.0 / 3600.0, 2451545.0, 0.0, 0.0);
        s.record_deg(3.0 / 3600.0, 2451546.0, 1.0, 1.0);
        assert_eq!(s.count, 2);
        assert!((s.mean_arcsec() - 2.0).abs() < 1e-9);
        assert!((s.max_arcsec - 3.0).abs() < 1e-9);
        assert_eq!(s.worst_chart.unwrap().0, 2451546.0);
        // rms = sqrt((1+9)/2) = sqrt(5)
        assert!((s.rms_arcsec() - 5.0_f64.sqrt()).abs() < 1e-9);
        assert_eq!(s.over_legacy, 0);
        assert_eq!(s.within_1_arcsec, 1);
        assert!((s.within_1_arcsec_frac() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn over_legacy_counts_tenth_degree() {
        let mut s = DeltaStats::default();
        s.record_deg(0.2, 0.0, 0.0, 0.0); // 720" > 360" => over legacy
        s.record_deg(0.05, 0.0, 0.0, 0.0); // 180" < 360" => not over
        assert_eq!(s.over_legacy, 1);
    }
}
