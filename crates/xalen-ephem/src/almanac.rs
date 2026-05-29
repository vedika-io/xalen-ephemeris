use crate::body::Body;
use crate::provider::{EphemerisError, EphemerisProvider};
use crate::vsop::Vsop87Provider;
use std::sync::Arc;
use xalen_coords::EclipticPosition;
use xalen_time::DeltaTModel;
use xalen_time::{JdTT, JdUT1, JulianDay};

/// High-level facade chaining multiple ephemeris providers with automatic fallback.
pub struct Almanac {
    providers: Vec<Arc<dyn EphemerisProvider>>,
    delta_t_model: DeltaTModel,
}

impl Almanac {
    /// Create an almanac with the default VSOP87 provider and SMH2016 delta-T model.
    pub fn default_vedic() -> Self {
        Self {
            providers: vec![Arc::new(Vsop87Provider::new())],
            delta_t_model: DeltaTModel::StephensonMorrisonHohenkerk2016,
        }
    }

    /// Insert a provider at the front of the chain (highest priority).
    pub fn with_provider(mut self, provider: Arc<dyn EphemerisProvider>) -> Self {
        self.providers.insert(0, provider);
        self
    }

    /// Compute geocentric ecliptic position, converting UT1 to TT internally.
    pub fn geocentric_ecliptic(
        &self,
        body: Body,
        jd_ut1: JdUT1,
    ) -> Result<EclipticPosition, EphemerisError> {
        let jd_tt = jd_ut1.to_tt(&self.delta_t_model);
        self.geocentric_ecliptic_tt(body, jd_tt)
    }

    /// Compute geocentric ecliptic position at a TT epoch (no delta-T conversion).
    pub fn geocentric_ecliptic_tt(
        &self,
        body: Body,
        jd_tt: JdTT,
    ) -> Result<EclipticPosition, EphemerisError> {
        for provider in &self.providers {
            let (start, end) = provider.coverage();
            if jd_tt.as_f64() >= start && jd_tt.as_f64() <= end {
                match provider.geocentric_ecliptic(body, jd_tt) {
                    Ok(pos) => return Ok(pos),
                    Err(EphemerisError::BodyNotAvailable(_)) => continue,
                    Err(e) => return Err(e),
                }
            }
        }
        Err(EphemerisError::BodyNotAvailable(body))
    }

    /// Return geocentric tropical longitude in degrees [0, 360).
    pub fn geocentric_longitude_deg(
        &self,
        body: Body,
        jd_ut1: JdUT1,
    ) -> Result<f64, EphemerisError> {
        let pos = self.geocentric_ecliptic(body, jd_ut1)?;
        Ok(pos.longitude.to_degrees().rem_euclid(360.0))
    }

    /// Return sidereal longitude in degrees [0, 360) by subtracting the given ayanamsa.
    pub fn sidereal_longitude_deg(
        &self,
        body: Body,
        jd_ut1: JdUT1,
        ayanamsa_deg: f64,
    ) -> Result<f64, EphemerisError> {
        let tropical = self.geocentric_longitude_deg(body, jd_ut1)?;
        Ok((tropical - ayanamsa_deg).rem_euclid(360.0))
    }

    /// Compute geocentric ecliptic positions for multiple bodies at once.
    pub fn all_positions(
        &self,
        bodies: &[Body],
        jd_ut1: JdUT1,
    ) -> Vec<(Body, Result<EclipticPosition, EphemerisError>)> {
        bodies
            .iter()
            .map(|&b| (b, self.geocentric_ecliptic(b, jd_ut1)))
            .collect()
    }
}

// Thread safety: Almanac is automatically Send + Sync because:
// - providers: Vec<Arc<dyn EphemerisProvider>> where EphemerisProvider: Send + Sync
// - delta_t_model: DeltaTModel is a simple enum (Clone + Serialize)
// No unsafe impl needed — the compiler derives both traits.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn almanac_computes_all_vedic_grahas() {
        let a = Almanac::default_vedic();
        let jd = JdUT1(2451545.0);
        for body in Body::VEDIC_GRAHAS {
            let result = a.geocentric_longitude_deg(*body, jd);
            assert!(result.is_ok(), "Failed for {body}: {:?}", result.err());
            let lon = result.unwrap();
            assert!(lon >= 0.0 && lon < 360.0, "{body} lon out of range: {lon}°");
        }
    }

    #[test]
    fn sidereal_is_tropical_minus_ayanamsa() {
        let a = Almanac::default_vedic();
        let jd = JdUT1(2451545.0);
        let ayanamsa = 23.85;
        let tropical = a.geocentric_longitude_deg(Body::Sun, jd).unwrap();
        let sidereal = a.sidereal_longitude_deg(Body::Sun, jd, ayanamsa).unwrap();
        let diff = (tropical - sidereal - ayanamsa).abs();
        assert!(
            diff < 0.01 || (360.0 - diff).abs() < 0.01,
            "Sidereal should be tropical - ayanamsa: T={tropical}, S={sidereal}, A={ayanamsa}"
        );
    }

    #[test]
    fn almanac_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Almanac>();
    }

    #[test]
    fn concurrent_access() {
        let a = Arc::new(Almanac::default_vedic());
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let almanac = a.clone();
                std::thread::spawn(move || {
                    let jd = JdUT1(2451545.0 + i as f64 * 365.25);
                    almanac.geocentric_longitude_deg(Body::Mars, jd).unwrap()
                })
            })
            .collect();
        for h in handles {
            let lon = h.join().unwrap();
            assert!(lon >= 0.0 && lon < 360.0);
        }
    }
}
