use crate::body::Body;
use crate::provider::{EphemerisError, EphemerisProvider};
use crate::vsop::Vsop87Provider;
use std::sync::Arc;
use xalen_coords::{
    EclipticPosition, EclipticSpeed, ecliptic_to_equatorial, mean_obliquity, nutation_2000b,
};
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

    /// Convert a UT1 Julian Day to TT using this almanac's ΔT model. Shared by
    /// the alternative-output accessors (`output.rs`) and return finders so they
    /// use the identical ΔT the geocentric path uses.
    pub(crate) fn to_tt(&self, jd_ut1: JdUT1) -> JdTT {
        jd_ut1.to_tt(&self.delta_t_model)
    }

    /// Borrow the provider chain (for output/frame accessors that need the same
    /// coverage-aware fall-through as the geocentric paths).
    pub(crate) fn providers_slice(&self) -> &[Arc<dyn EphemerisProvider>] {
        &self.providers
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
        // Tracks the most informative provider error so that, once every provider
        // is exhausted, we surface *why* it failed rather than a generic
        // "not available". An EpochOutOfRange (e.g. Pluto's analytic series only
        // covers 1885-2099) is treated the same as BodyNotAvailable — keep trying
        // the next provider, since a loaded DE440 kernel may cover the epoch the
        // analytical model cannot. Only genuinely unexpected errors short-circuit.
        let mut last_err: Option<EphemerisError> = None;
        for provider in &self.providers {
            let (start, end) = provider.coverage();
            if jd_tt.as_f64() >= start && jd_tt.as_f64() <= end {
                match provider.geocentric_ecliptic(body, jd_tt) {
                    Ok(pos) => return Ok(pos),
                    // Both "this provider can't do this body" and "this provider's
                    // model doesn't reach this epoch" mean: try the next provider.
                    Err(e @ EphemerisError::BodyNotAvailable(_)) => last_err = Some(e),
                    Err(e @ EphemerisError::EpochOutOfRange(_)) => last_err = Some(e),
                    // A corrupt segment / ComputationFailed / IO error is a real
                    // fault, not a coverage miss — surface it immediately.
                    Err(e) => return Err(e),
                }
            }
        }
        Err(last_err.unwrap_or(EphemerisError::BodyNotAvailable(body)))
    }

    /// Compute geocentric ecliptic speed (apparent daily motion), converting
    /// UT1 to TT internally. Longitude rate < 0 means retrograde.
    pub fn geocentric_speed(
        &self,
        body: Body,
        jd_ut1: JdUT1,
    ) -> Result<EclipticSpeed, EphemerisError> {
        let jd_tt = jd_ut1.to_tt(&self.delta_t_model);
        self.geocentric_speed_tt(body, jd_tt)
    }

    /// Compute geocentric ecliptic speed at a TT epoch (no delta-T conversion).
    pub fn geocentric_speed_tt(
        &self,
        body: Body,
        jd_tt: JdTT,
    ) -> Result<EclipticSpeed, EphemerisError> {
        // Same fall-through policy as geocentric_ecliptic_tt: an EpochOutOfRange
        // is a coverage miss (the analytic Pluto series is only valid 1885-2099,
        // and its ±0.5-day finite-difference samples must also stay in window), so
        // keep trying providers and only surface the last coverage error once all
        // are exhausted. Real faults short-circuit.
        let mut last_err: Option<EphemerisError> = None;
        for provider in &self.providers {
            let (start, end) = provider.coverage();
            // The finite difference samples jd ± 0.5 day, so require that margin
            // to stay inside the provider's coverage.
            if jd_tt.as_f64() - 0.5 >= start && jd_tt.as_f64() + 0.5 <= end {
                match provider.geocentric_ecliptic_speed(body, jd_tt) {
                    Ok(s) => return Ok(s),
                    Err(e @ EphemerisError::BodyNotAvailable(_)) => last_err = Some(e),
                    Err(e @ EphemerisError::EpochOutOfRange(_)) => last_err = Some(e),
                    Err(e) => return Err(e),
                }
            }
        }
        Err(last_err.unwrap_or(EphemerisError::BodyNotAvailable(body)))
    }

    /// Compute the **topocentric** (observer-centered) ecliptic position for an
    /// observer at `lat_deg`/`lon_deg` (east positive) and `elevation_m` metres,
    /// converting UT1 to TT internally. Applies diurnal parallax (Meeus Ch.40):
    /// ~8.8″ for the Sun, up to ~1° for the Moon.
    pub fn topocentric_ecliptic(
        &self,
        body: Body,
        jd_ut1: JdUT1,
        lat_deg: f64,
        lon_deg: f64,
        elevation_m: f64,
    ) -> Result<EclipticPosition, EphemerisError> {
        let geo = self.geocentric_ecliptic(body, jd_ut1)?;
        let jd_tt = jd_ut1.to_tt(&self.delta_t_model);
        let t = jd_tt.julian_centuries_from_j2000();
        Ok(crate::topocentric::topocentric_ecliptic(
            &geo,
            jd_ut1.as_f64(),
            t,
            lat_deg,
            lon_deg,
            elevation_m,
        ))
    }

    /// The body's **topocentric altitude** above the horizon (degrees) for an
    /// observer, after diurnal parallax. Negative = below the horizon.
    pub fn topocentric_altitude_deg(
        &self,
        body: Body,
        jd_ut1: JdUT1,
        lat_deg: f64,
        lon_deg: f64,
        elevation_m: f64,
    ) -> Result<f64, EphemerisError> {
        let topo = self.topocentric_ecliptic(body, jd_ut1, lat_deg, lon_deg, elevation_m)?;
        let jd_tt = jd_ut1.to_tt(&self.delta_t_model);
        // `topo` is on the TRUE equinox of date (apparent ecliptic-of-date), so
        // use the TRUE obliquity (mean + delta_epsilon) for ecliptic→equatorial and
        // APPARENT sidereal time (GMST + equation of equinoxes) for the hour angle —
        // keeping RA and sidereal time on the same equinox. (Matches
        // topocentric::topocentric_ecliptic and besselian.rs.)
        let t = jd_tt.julian_centuries_from_j2000();
        let nut = nutation_2000b(t);
        let eps = mean_obliquity(t) + nut.delta_epsilon; // true obliquity of date
        let eq = ecliptic_to_equatorial(&topo, eps);
        let eq_of_equinoxes = nut.delta_psi * eps.cos();
        let last = (crate::topocentric::gmst_deg(jd_ut1.as_f64()) + lon_deg).to_radians()
            + eq_of_equinoxes;
        let h = last - eq.right_ascension;
        let phi = lat_deg.to_radians();
        let alt =
            (phi.sin() * eq.declination.sin() + phi.cos() * eq.declination.cos() * h.cos()).asin();
        Ok(alt.to_degrees())
    }

    /// The body's **topocentric local hour angle** in degrees, wrapped to
    /// `(−180°, +180°]`. Zero at upper meridian transit (culmination); negative east
    /// of the meridian (rising), positive west (setting). Used by the rise/set engine
    /// to locate meridian transits the way `swe_rise_trans` does (hour-angle zero),
    /// which for the fast-moving Moon differs by minutes from the altitude maximum.
    pub fn topocentric_hour_angle_deg(
        &self,
        body: Body,
        jd_ut1: JdUT1,
        lat_deg: f64,
        lon_deg: f64,
        elevation_m: f64,
    ) -> Result<f64, EphemerisError> {
        let topo = self.topocentric_ecliptic(body, jd_ut1, lat_deg, lon_deg, elevation_m)?;
        let jd_tt = jd_ut1.to_tt(&self.delta_t_model);
        // `topo` is on the TRUE equinox of date (apparent ecliptic-of-date), so the
        // ecliptic→equatorial rotation must use the TRUE obliquity (mean + Δε) and
        // the meridian sidereal time must be APPARENT (GAST = GMST + Δψ·cos ε_true),
        // keeping RA and sidereal time on the same equinox. This mirrors the
        // sibling altitude path above (and topocentric::topocentric_ecliptic); the
        // earlier mean-obliquity + bare-GMST form left a Δε-/equation-of-equinoxes-
        // sized (~tens-of-arcsec) inconsistency that biased rise/set/transit times.
        let t = jd_tt.julian_centuries_from_j2000();
        let eps = mean_obliquity(t) + nutation_2000b(t).delta_epsilon; // true obliquity of date
        let eq = ecliptic_to_equatorial(&topo, eps);
        let last_deg = xalen_coords::gast_deg(jd_ut1.as_f64(), t) + lon_deg;
        let ha = last_deg - eq.right_ascension.to_degrees();
        // Wrap to (−180, 180].
        let mut h = ha.rem_euclid(360.0);
        if h > 180.0 {
            h -= 360.0;
        }
        Ok(h)
    }

    /// Rise, transit (upper culmination), and set times over the 24 hours from
    /// `jd_start` (UT1) for an observer at `lat_deg`/`lon_deg`/`elevation_m`.
    /// See [`crate::rise_set::RiseTransitSet`].
    pub fn rise_transit_set(
        &self,
        body: Body,
        jd_start: JdUT1,
        lat_deg: f64,
        lon_deg: f64,
        elevation_m: f64,
    ) -> Result<crate::rise_set::RiseTransitSet, EphemerisError> {
        crate::rise_set::compute(self, body, jd_start, lat_deg, lon_deg, elevation_m)
    }

    /// Swiss-style forward search for the **next** rise, transit, and set after
    /// `jd_start` (UT1) for an observer at `lat_deg`/`lon_deg`/`elevation_m`. Each
    /// event is found independently by searching strictly forward and refined to
    /// sub-second precision — matching `swe_rise_trans` (default flags: upper limb +
    /// standard refraction). Use this (not [`Almanac::rise_transit_set`]) when you
    /// want "the next sunrise from this instant" rather than "the rise inside this
    /// calendar day's window".
    pub fn rise_transit_set_next(
        &self,
        body: Body,
        jd_start: JdUT1,
        lat_deg: f64,
        lon_deg: f64,
        elevation_m: f64,
    ) -> Result<crate::rise_set::RiseTransitSet, EphemerisError> {
        crate::rise_set::next_events(self, body, jd_start, lat_deg, lon_deg, elevation_m)
    }

    /// Next dawn and dusk for a twilight boundary (civil −6°, nautical −12°,
    /// astronomical −18°), searching forward from `jd_start` (UT1). Dawn = Sun
    /// centre rising through the boundary; dusk = sinking through it. Returns `None`
    /// for a leg that does not occur within ~1.2 days (polar day / night).
    pub fn twilight(
        &self,
        twilight: crate::rise_set::Twilight,
        jd_start: JdUT1,
        lat_deg: f64,
        lon_deg: f64,
        elevation_m: f64,
    ) -> Result<crate::rise_set::TwilightTimes, EphemerisError> {
        crate::rise_set::twilight(self, twilight, jd_start, lat_deg, lon_deg, elevation_m)
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
    use crate::provider::EphemerisProvider;
    use xalen_coords::EclipticPosition;

    /// A provider that ALWAYS reports `EpochOutOfRange` (mimics the analytic
    /// Pluto series being asked for a year outside 1885-2099, but with broad
    /// `coverage()` so the Almanac actually invokes it). Used to prove the
    /// Almanac falls THROUGH an out-of-window provider to the next one rather
    /// than hard-erroring.
    struct AlwaysOutOfRange;
    impl EphemerisProvider for AlwaysOutOfRange {
        fn heliocentric_ecliptic(
            &self,
            _body: Body,
            jd_tt: JdTT,
        ) -> Result<EclipticPosition, EphemerisError> {
            Err(EphemerisError::EpochOutOfRange(jd_tt.as_f64()))
        }
        fn geocentric_ecliptic(
            &self,
            _body: Body,
            jd_tt: JdTT,
        ) -> Result<EclipticPosition, EphemerisError> {
            Err(EphemerisError::EpochOutOfRange(jd_tt.as_f64()))
        }
        fn coverage(&self) -> (f64, f64) {
            (f64::MIN, f64::MAX)
        }
        fn accuracy_arcsec(&self) -> f64 {
            f64::INFINITY
        }
        fn name(&self) -> &str {
            "always-out-of-range (test)"
        }
    }

    /// A provider that ALWAYS fails with a real fault (`ComputationFailed`).
    /// Used to prove that genuine errors are NOT swallowed by the
    /// EpochOutOfRange fall-through path.
    struct AlwaysComputationFailed;
    impl EphemerisProvider for AlwaysComputationFailed {
        fn heliocentric_ecliptic(
            &self,
            _body: Body,
            _jd_tt: JdTT,
        ) -> Result<EclipticPosition, EphemerisError> {
            Err(EphemerisError::ComputationFailed("boom".into()))
        }
        fn geocentric_ecliptic(
            &self,
            _body: Body,
            _jd_tt: JdTT,
        ) -> Result<EclipticPosition, EphemerisError> {
            Err(EphemerisError::ComputationFailed("boom".into()))
        }
        fn coverage(&self) -> (f64, f64) {
            (f64::MIN, f64::MAX)
        }
        fn accuracy_arcsec(&self) -> f64 {
            f64::INFINITY
        }
        fn name(&self) -> &str {
            "always-computation-failed (test)"
        }
    }

    #[test]
    fn epoch_out_of_range_falls_through_to_next_provider() {
        // Front provider is out-of-window for every epoch; the default VSOP87
        // provider behind it must still answer. This mirrors the real Pluto
        // case: the analytic series throws EpochOutOfRange and a downstream
        // (DE440) provider picks it up. Mars at J2000 is firmly in VSOP87 range.
        let a = Almanac::default_vedic().with_provider(Arc::new(AlwaysOutOfRange));
        let jd = JdUT1(2451545.0);
        let pos = a.geocentric_ecliptic(Body::Mars, jd);
        assert!(
            pos.is_ok(),
            "EpochOutOfRange from the front provider must fall through to VSOP87, got {:?}",
            pos.err()
        );
        // The speed path uses the same fall-through policy.
        let speed = a.geocentric_speed(Body::Mars, jd);
        assert!(
            speed.is_ok(),
            "speed: EpochOutOfRange must fall through to VSOP87, got {:?}",
            speed.err()
        );
    }

    #[test]
    fn epoch_out_of_range_surfaced_when_no_provider_covers() {
        // Only an out-of-range provider in the chain → the final error must be
        // the informative EpochOutOfRange, NOT a generic BodyNotAvailable.
        let mut a = Almanac::default_vedic();
        a.providers.clear();
        a.providers.push(Arc::new(AlwaysOutOfRange));
        let err = a
            .geocentric_ecliptic(Body::Mars, JdUT1(2451545.0))
            .unwrap_err();
        assert!(
            matches!(err, EphemerisError::EpochOutOfRange(_)),
            "exhausted chain must surface the last coverage error (EpochOutOfRange), got {err:?}"
        );
    }

    #[test]
    fn genuine_errors_are_not_swallowed_by_fall_through() {
        // A real fault (ComputationFailed) must short-circuit immediately and
        // NOT be silently converted into a try-next-provider — even though a
        // working VSOP87 provider sits behind it.
        let a = Almanac::default_vedic().with_provider(Arc::new(AlwaysComputationFailed));
        let err = a
            .geocentric_ecliptic(Body::Mars, JdUT1(2451545.0))
            .unwrap_err();
        assert!(
            matches!(err, EphemerisError::ComputationFailed(_)),
            "a genuine fault must surface, not fall through to VSOP87, got {err:?}"
        );
    }

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
    fn geocentric_speed_matches_known_daily_motion() {
        let a = Almanac::default_vedic();
        let jd = JdUT1(2451545.0); // J2000 (early January)

        // Sun: ~0.95-1.02 deg/day (fastest near perihelion, early January); never retrograde.
        let sun = a.geocentric_speed(Body::Sun, jd).unwrap();
        let sun_dpd = sun.longitude_deg_per_day();
        assert!(
            (0.9..=1.1).contains(&sun_dpd),
            "Sun daily motion should be ~1 deg/day, got {sun_dpd}"
        );
        assert!(!sun.is_retrograde(), "Sun is never retrograde");

        // Moon: ~11-15 deg/day; never retrograde.
        let moon = a.geocentric_speed(Body::Moon, jd).unwrap();
        let moon_dpd = moon.longitude_deg_per_day();
        assert!(
            (10.0..=16.0).contains(&moon_dpd),
            "Moon daily motion should be ~13 deg/day, got {moon_dpd}"
        );
        assert!(!moon.is_retrograde(), "Moon is never retrograde");

        // Mean lunar node: ALWAYS retrograde (regresses ~0.053 deg/day).
        let node = a.geocentric_speed(Body::MeanNode, jd).unwrap();
        assert!(
            node.is_retrograde(),
            "Mean node is always retrograde, got {} deg/day",
            node.longitude_deg_per_day()
        );
    }

    #[test]
    fn topocentric_lunar_parallax_dominates_solar() {
        let a = Almanac::default_vedic();
        let jd = JdUT1(2451545.0);
        let (lat, lon, elev) = (28.6139, 77.2090, 216.0); // Delhi
        let ang = |x: f64| {
            let d = x.abs() % 360.0;
            d.min(360.0 - d)
        };

        let sun_geo = a.geocentric_longitude_deg(Body::Sun, jd).unwrap();
        let sun_topo = a
            .topocentric_ecliptic(Body::Sun, jd, lat, lon, elev)
            .unwrap()
            .longitude
            .to_degrees();
        let moon_geo = a.geocentric_longitude_deg(Body::Moon, jd).unwrap();
        let moon_topo = a
            .topocentric_ecliptic(Body::Moon, jd, lat, lon, elev)
            .unwrap()
            .longitude
            .to_degrees();

        let sun_par = ang(sun_topo - sun_geo);
        let moon_par = ang(moon_topo - moon_geo);

        // Solar parallax is always tiny (~8.8"); lunar parallax is up to ~1 deg.
        assert!(
            sun_par < 0.02,
            "Sun parallax should be < 0.02 deg, got {sun_par}"
        );
        assert!(
            moon_par < 1.2,
            "Moon parallax bounded by ~1 deg, got {moon_par}"
        );
        // The Moon is ~400x closer than the Sun, so its parallax always dominates.
        assert!(
            moon_par > sun_par,
            "Lunar parallax must exceed solar: moon={moon_par} sun={sun_par}"
        );
    }

    #[test]
    fn sun_rise_transit_set_at_delhi() {
        let a = Almanac::default_vedic();
        let jd = JdUT1(2460482.5); // 2024-06-21 00:00 UT
        let r = a
            .rise_transit_set(Body::Sun, jd, 28.6139, 77.2090, 216.0)
            .unwrap();
        assert!(r.rise.is_some(), "Sun rises at Delhi");
        assert!(r.set.is_some(), "Sun sets at Delhi");
        assert!(r.transit.is_some());
        assert!(!r.always_above && !r.always_below, "Delhi is not polar");
        // Delhi (lat 28.6°): the Sun's culmination altitude ranges ~38° (winter)
        // to ~85° (summer) — always well above the horizon at transit.
        assert!(
            r.transit_altitude_deg > 25.0 && r.transit_altitude_deg <= 90.0,
            "Sun transit altitude at Delhi should be high, got {}",
            r.transit_altitude_deg
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
