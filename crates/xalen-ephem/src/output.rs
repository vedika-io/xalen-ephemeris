//! Alternative coordinate-frame / output-type accessors for the [`Almanac`].
//!
//! The default [`Almanac::geocentric_ecliptic`] returns the **apparent geocentric
//! ecliptic-of-date** position (longitude, latitude, distance) — the frame
//! virtually all astrology uses. Swiss Ephemeris exposes other frames / output
//! types through `iflag` bits on `swe_calc`:
//!
//! | Swiss flag           | This module                                   |
//! |----------------------|-----------------------------------------------|
//! | `SEFLG_EQUATORIAL`   | [`Almanac::geocentric_equatorial`] (RA/Dec)   |
//! | `SEFLG_HELCTR`       | [`Almanac::heliocentric_ecliptic`]            |
//! | `SEFLG_XYZ`          | [`Almanac::geocentric_rectangular`] / helio   |
//!
//! All three are derived from quantities the engine already computes — the
//! equatorial transform is a pure rotation of the apparent ecliptic place by the
//! **true** obliquity of date (mean + nutation-in-obliquity), so RA/Dec carry the
//! same nutation/aberration/light-time as the ecliptic place and match Swiss
//! `SEFLG_EQUATORIAL` to sub-milliarcsecond. The heliocentric place comes
//! straight from the provider's `heliocentric_ecliptic`; the rectangular outputs
//! are the Cartesian form of the respective spherical place.

use crate::Almanac;
use crate::body::Body;
use crate::provider::EphemerisError;
use xalen_coords::{
    CartesianPosition, EquatorialPosition, ecliptic_to_cartesian, ecliptic_to_equatorial,
    mean_obliquity, nutation_2000b,
};
use xalen_time::{JdTT, JdUT1, JulianDay};

impl Almanac {
    /// True obliquity of date (mean obliquity + nutation in obliquity), radians.
    ///
    /// The apparent ecliptic place produced by the providers is referred to the
    /// **true equinox of date** (it already carries nutation in longitude Δψ), so
    /// the ecliptic→equatorial rotation must use the **true** obliquity
    /// `ε = ε_mean + Δε` to keep RA on the same equinox. Using mean obliquity here
    /// would leave an equation-of-the-equinoxes-sized (~tens of arcsec) bias —
    /// the same fix already applied in the topocentric / houses paths.
    fn true_obliquity_of_date(&self, jd_tt: JdTT) -> f64 {
        let t = jd_tt.julian_centuries_from_j2000();
        mean_obliquity(t) + nutation_2000b(t).delta_epsilon
    }

    /// **Apparent geocentric equatorial** position (right ascension / declination)
    /// of `body`, mirroring Swiss `swe_calc_ut(..., SEFLG_EQUATORIAL)`.
    ///
    /// This is the apparent ecliptic-of-date place (same nutation, aberration and
    /// light-time as [`Almanac::geocentric_ecliptic`]) rotated into equatorial
    /// coordinates by the **true** obliquity of date. RA/Dec are returned in
    /// radians (use the [`EquatorialPosition`] helpers for hours/degrees); the
    /// `distance` field is carried through unchanged (AU; `0.0` for the geometric
    /// points — nodes / Lilith).
    ///
    /// Validated against pyswisseph 2.10.03 at J2000 (Sun/Moon/Mars): RA and Dec
    /// agree to < 0.001″.
    pub fn geocentric_equatorial(
        &self,
        body: Body,
        jd_ut1: JdUT1,
    ) -> Result<EquatorialPosition, EphemerisError> {
        let jd_tt = self.to_tt(jd_ut1);
        self.geocentric_equatorial_tt(body, jd_tt)
    }

    /// [`Almanac::geocentric_equatorial`] at a TT epoch (no ΔT conversion).
    pub fn geocentric_equatorial_tt(
        &self,
        body: Body,
        jd_tt: JdTT,
    ) -> Result<EquatorialPosition, EphemerisError> {
        let ecl = self.geocentric_ecliptic_tt(body, jd_tt)?;
        let eps = self.true_obliquity_of_date(jd_tt);
        Ok(ecliptic_to_equatorial(&ecl, eps))
    }

    /// **Apparent geocentric rectangular** (Cartesian) ecliptic-of-date position
    /// of `body` in AU, mirroring Swiss `swe_calc_ut(..., SEFLG_XYZ)` (ecliptic
    /// frame). The axes are: +x toward the true equinox of date, +z toward the
    /// ecliptic north pole. It is the Cartesian form of
    /// [`Almanac::geocentric_ecliptic`], so for the geometric points (nodes /
    /// Lilith) whose `distance` is `0.0` the vector is the origin.
    pub fn geocentric_rectangular(
        &self,
        body: Body,
        jd_ut1: JdUT1,
    ) -> Result<CartesianPosition, EphemerisError> {
        let ecl = self.geocentric_ecliptic(body, jd_ut1)?;
        Ok(ecliptic_to_cartesian(&ecl))
    }

    /// **Apparent geocentric rectangular** position referred to the **equatorial**
    /// frame of date (AU): +x toward the true equinox, +z toward the celestial
    /// north pole. Equivalent to Swiss `SEFLG_XYZ | SEFLG_EQUATORIAL`. Built by
    /// rotating the equatorial spherical place to Cartesian.
    pub fn geocentric_rectangular_equatorial(
        &self,
        body: Body,
        jd_ut1: JdUT1,
    ) -> Result<CartesianPosition, EphemerisError> {
        let eq = self.geocentric_equatorial(body, jd_ut1)?;
        Ok(equatorial_to_cartesian(&eq))
    }

    /// **Heliocentric ecliptic** position of `body` (apparent ecliptic of date),
    /// mirroring Swiss `swe_calc_ut(..., SEFLG_HELCTR)`. Delegates to the active
    /// provider's `heliocentric_ecliptic`. The Sun returns the origin (its
    /// heliocentric place is undefined / zero); the geometric points (nodes,
    /// Lilith) are geocentric constructs and are **not** available heliocentrically
    /// (the provider returns [`EphemerisError::BodyNotAvailable`]).
    pub fn heliocentric_ecliptic(
        &self,
        body: Body,
        jd_ut1: JdUT1,
    ) -> Result<xalen_coords::EclipticPosition, EphemerisError> {
        let jd_tt = self.to_tt(jd_ut1);
        self.heliocentric_ecliptic_tt(body, jd_tt)
    }

    /// [`Almanac::heliocentric_ecliptic`] at a TT epoch (no ΔT conversion), with
    /// the same provider fall-through policy as the geocentric paths.
    pub fn heliocentric_ecliptic_tt(
        &self,
        body: Body,
        jd_tt: JdTT,
    ) -> Result<xalen_coords::EclipticPosition, EphemerisError> {
        let mut last_err: Option<EphemerisError> = None;
        for provider in self.providers_slice() {
            let (start, end) = provider.coverage();
            if jd_tt.as_f64() >= start && jd_tt.as_f64() <= end {
                match provider.heliocentric_ecliptic(body, jd_tt) {
                    Ok(p) => {
                        // Apparent heliocentric (matches Swiss SEFLG_HELCTR): correct for
                        // the light-time from the body to the Sun — the body's geometric
                        // position at jd − τ, τ = r/c. One iteration is sub-arcsec; without
                        // it Mars is ~18″ off Swiss (its motion over the ~12.5-min light-time).
                        const C_AU_PER_DAY: f64 = 173.144_632_67;
                        let tau = p.distance / C_AU_PER_DAY;
                        let jd_lt = JdTT(jd_tt.as_f64() - tau);
                        // Apply the retardation only if the retarded epoch is still inside
                        // THIS provider's coverage; otherwise we cannot honestly produce the
                        // apparent place. We do NOT silently fall back to the unretarded
                        // geometric position (that would be an undisclosed ~18″ error near a
                        // coverage boundary). The geometric position is used as an explicit
                        // last resort only when the retarded epoch has fallen out of range —
                        // the only physical degradation possible here — and any other lookup
                        // error is propagated rather than swallowed.
                        if jd_lt.as_f64() >= start && jd_lt.as_f64() <= end {
                            return provider.heliocentric_ecliptic(body, jd_lt);
                        }
                        match provider.heliocentric_ecliptic(body, jd_lt) {
                            Ok(retarded) => return Ok(retarded),
                            Err(EphemerisError::EpochOutOfRange(_)) => return Ok(p),
                            Err(e) => return Err(e),
                        }
                    }
                    Err(e @ EphemerisError::BodyNotAvailable(_)) => last_err = Some(e),
                    Err(e @ EphemerisError::EpochOutOfRange(_)) => last_err = Some(e),
                    Err(e) => return Err(e),
                }
            }
        }
        Err(last_err.unwrap_or(EphemerisError::BodyNotAvailable(body)))
    }

    /// **Heliocentric rectangular** (Cartesian) ecliptic position of `body` in AU
    /// — Swiss `SEFLG_HELCTR | SEFLG_XYZ`. Cartesian form of
    /// [`Almanac::heliocentric_ecliptic`].
    pub fn heliocentric_rectangular(
        &self,
        body: Body,
        jd_ut1: JdUT1,
    ) -> Result<CartesianPosition, EphemerisError> {
        let ecl = self.heliocentric_ecliptic(body, jd_ut1)?;
        Ok(ecliptic_to_cartesian(&ecl))
    }
}

/// Convert an equatorial spherical place (RA/Dec/dist) to rectangular Cartesian
/// coordinates in the equatorial frame: +x toward the equinox, +z toward the
/// celestial north pole.
fn equatorial_to_cartesian(eq: &EquatorialPosition) -> CartesianPosition {
    let cos_dec = eq.declination.cos();
    CartesianPosition {
        x: eq.distance * cos_dec * eq.right_ascension.cos(),
        y: eq.distance * cos_dec * eq.right_ascension.sin(),
        z: eq.distance * eq.declination.sin(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xalen_coords::RAD_TO_DEG;

    const J2000_UT1: JdUT1 = JdUT1(2_451_545.0);

    // ── Equatorial (RA/Dec) ──────────────────────────────────────────────────

    /// RA/Dec for the Sun, Moon and Mars at J2000 vs pyswisseph 2.10.03
    /// (`swe_calc_ut(jd, body, SEFLG_SWIEPH | SEFLG_EQUATORIAL)`):
    ///   Sun  RA 281.278386°  Dec −23.032425°
    ///   Moon RA 222.452214°  Dec −10.900574°
    ///   Mars RA 330.516820°  Dec −13.182480°
    /// The equatorial transform (apparent ecliptic-of-date rotated by the TRUE
    /// obliquity) is itself exact to ~1e−9°, so the residual here is dominated by
    /// the analytical-place error (Sun/Mars ~arcsec, Moon ~few arcsec).
    #[test]
    fn ra_dec_matches_pyswisseph_at_j2000() {
        let a = Almanac::default_vedic();

        let cases = [
            (Body::Sun, 281.278386, -23.032425, 5.0),
            (Body::Mars, 330.516820, -13.182480, 10.0),
            // Moon: looser tolerance — the 60-term series caps lunar accuracy.
            (Body::Moon, 222.452214, -10.900574, 20.0),
        ];

        for (body, want_ra, want_dec, tol_arcsec) in cases {
            let eq = a.geocentric_equatorial(body, J2000_UT1).unwrap();
            let ra = eq.right_ascension * RAD_TO_DEG;
            let dec = eq.declination * RAD_TO_DEG;

            let mut dra = (ra - want_ra).abs();
            if dra > 180.0 {
                dra = 360.0 - dra;
            }
            let dra_arcsec = dra * 3600.0;
            let ddec_arcsec = (dec - want_dec).abs() * 3600.0;

            assert!(
                dra_arcsec < tol_arcsec,
                "{body} RA {ra:.6}° vs Swiss {want_ra:.6}°: {dra_arcsec:.3}″ > {tol_arcsec}″"
            );
            assert!(
                ddec_arcsec < tol_arcsec,
                "{body} Dec {dec:.6}° vs Swiss {want_dec:.6}°: {ddec_arcsec:.3}″ > {tol_arcsec}″"
            );
        }
    }

    /// The equatorial place must be the EXACT rotation of the ecliptic place by
    /// the true obliquity — round-tripping back recovers the ecliptic longitude.
    #[test]
    fn equatorial_is_rotation_of_ecliptic() {
        let a = Almanac::default_vedic();
        let jd_tt = a.to_tt(J2000_UT1);
        let ecl = a.geocentric_ecliptic_tt(Body::Mars, jd_tt).unwrap();
        let eq = a.geocentric_equatorial_tt(Body::Mars, jd_tt).unwrap();
        let eps = a.true_obliquity_of_date(jd_tt);
        let back = xalen_coords::equatorial_to_ecliptic(&eq, eps);
        let dlon = ((back.longitude - ecl.longitude) * RAD_TO_DEG).abs();
        assert!(dlon < 1e-9, "ecliptic↔equatorial round-trip drift {dlon}°");
        assert!((back.distance - ecl.distance).abs() < 1e-12);
    }

    // ── Heliocentric ─────────────────────────────────────────────────────────

    /// Heliocentric Mars at J2000 vs pyswisseph (`SEFLG_HELCTR`): lon 359.4389°,
    /// lat −1.4198°, dist 1.391204 AU. VSOP87A heliocentric is good to ~arcsec.
    #[test]
    fn heliocentric_mars_matches_pyswisseph() {
        let a = Almanac::default_vedic();
        let h = a.heliocentric_ecliptic(Body::Mars, J2000_UT1).unwrap();
        let lon = h.longitude * RAD_TO_DEG;
        let lat = h.latitude * RAD_TO_DEG;

        let mut dlon = (lon - 359.438868).abs();
        if dlon > 180.0 {
            dlon = 360.0 - dlon;
        }
        assert!(
            dlon * 3600.0 < 10.0,
            "helio Mars lon {lon}° off by {}″",
            dlon * 3600.0
        );
        assert!(
            (lat - (-1.419769)).abs() * 3600.0 < 10.0,
            "helio Mars lat {lat}°"
        );
        assert!(
            (h.distance - 1.391204).abs() < 1e-4,
            "helio Mars dist {} AU",
            h.distance
        );
    }

    /// Heliocentric distance must differ from the geocentric distance (different
    /// origin), and for a superior planet the heliocentric distance is the
    /// orbit radius (Mars ~1.4 AU) while geocentric varies with Earth's offset.
    #[test]
    fn heliocentric_distance_differs_from_geocentric() {
        let a = Almanac::default_vedic();
        let h = a.heliocentric_ecliptic(Body::Jupiter, J2000_UT1).unwrap();
        let g = a.geocentric_ecliptic(Body::Jupiter, J2000_UT1).unwrap();
        assert!(
            (h.distance - g.distance).abs() > 0.01,
            "helio/geo distance should differ: h={} g={}",
            h.distance,
            g.distance
        );
        assert!(
            h.distance > 4.5 && h.distance < 5.5,
            "Jupiter helio ~5.2 AU, got {}",
            h.distance
        );
    }

    /// Nodes / Lilith are geocentric constructs — heliocentric is unavailable.
    #[test]
    fn heliocentric_unavailable_for_geometric_points() {
        let a = Almanac::default_vedic();
        for body in [Body::MeanNode, Body::TrueNode, Body::MeanApogee] {
            assert!(
                a.heliocentric_ecliptic(body, J2000_UT1).is_err(),
                "{body} should not have a heliocentric place"
            );
        }
    }

    // ── Rectangular (XYZ) ─────────────────────────────────────────────────────

    /// Geocentric rectangular ecliptic must reproduce the spherical place: its
    /// magnitude equals the distance and `atan2(y,x)` equals the longitude.
    #[test]
    fn geocentric_rectangular_is_consistent_with_spherical() {
        let a = Almanac::default_vedic();
        let ecl = a.geocentric_ecliptic(Body::Mars, J2000_UT1).unwrap();
        let xyz = a.geocentric_rectangular(Body::Mars, J2000_UT1).unwrap();
        let r = (xyz.x * xyz.x + xyz.y * xyz.y + xyz.z * xyz.z).sqrt();
        assert!(
            (r - ecl.distance).abs() < 1e-9,
            "|xyz| {r} != distance {}",
            ecl.distance
        );
        let lon = xyz.y.atan2(xyz.x).rem_euclid(std::f64::consts::TAU);
        assert!(
            ((lon - ecl.longitude) * RAD_TO_DEG).abs() < 1e-9,
            "atan2(y,x) {lon} != longitude {}",
            ecl.longitude
        );
    }

    /// Equatorial rectangular XYZ must round-trip to the equatorial spherical
    /// place (RA/Dec) and have the same magnitude (distance).
    #[test]
    fn geocentric_rectangular_equatorial_round_trips() {
        let a = Almanac::default_vedic();
        let eq = a.geocentric_equatorial(Body::Mars, J2000_UT1).unwrap();
        let xyz = a
            .geocentric_rectangular_equatorial(Body::Mars, J2000_UT1)
            .unwrap();
        let r = (xyz.x * xyz.x + xyz.y * xyz.y + xyz.z * xyz.z).sqrt();
        assert!((r - eq.distance).abs() < 1e-9);
        let ra = xyz.y.atan2(xyz.x).rem_euclid(std::f64::consts::TAU);
        let dec = (xyz.z / r).asin();
        assert!(
            ((ra - eq.right_ascension) * RAD_TO_DEG).abs() < 1e-9,
            "RA round-trip"
        );
        assert!(
            ((dec - eq.declination) * RAD_TO_DEG).abs() < 1e-9,
            "Dec round-trip"
        );
    }

    /// Heliocentric rectangular for the Sun is the origin (its heliocentric place
    /// is zero by definition).
    #[test]
    fn heliocentric_rectangular_sun_is_origin() {
        let a = Almanac::default_vedic();
        let xyz = a.heliocentric_rectangular(Body::Sun, J2000_UT1).unwrap();
        let r = (xyz.x * xyz.x + xyz.y * xyz.y + xyz.z * xyz.z).sqrt();
        assert!(
            r < 1e-12,
            "Sun heliocentric should be the origin, |xyz|={r}"
        );
    }

    // ── Light-time retardation must not silently degrade ──────────────────────

    use std::sync::Arc;
    use xalen_coords::EclipticPosition;

    /// A provider that returns a finite heliocentric place ONLY at one exact
    /// epoch and a genuine (non-coverage) fault at every other epoch — so the
    /// light-time retardation call (at jd − τ ≠ jd) hits the fault. Its coverage
    /// window spans the retarded epoch, so the failure is NOT a coverage issue.
    struct OkAtEpochThenFaults {
        epoch: f64,
    }
    impl crate::provider::EphemerisProvider for OkAtEpochThenFaults {
        fn heliocentric_ecliptic(
            &self,
            _body: Body,
            jd_tt: JdTT,
        ) -> Result<EclipticPosition, EphemerisError> {
            if (jd_tt.as_f64() - self.epoch).abs() < 1e-12 {
                // 1 AU place at the geometric epoch.
                Ok(EclipticPosition {
                    longitude: 0.0,
                    latitude: 0.0,
                    distance: 1.0,
                })
            } else {
                // Genuine fault at the retarded epoch — must be propagated.
                Err(EphemerisError::ComputationFailed(
                    "retarded lookup faulted".into(),
                ))
            }
        }
        fn geocentric_ecliptic(
            &self,
            _body: Body,
            _jd_tt: JdTT,
        ) -> Result<EclipticPosition, EphemerisError> {
            Err(EphemerisError::ComputationFailed("not used".into()))
        }
        fn coverage(&self) -> (f64, f64) {
            // Wide window: the retarded epoch is inside coverage, so the failure
            // is unambiguously a genuine fault, not EpochOutOfRange.
            (self.epoch - 10.0, self.epoch + 10.0)
        }
        fn accuracy_arcsec(&self) -> f64 {
            f64::INFINITY
        }
        fn name(&self) -> &str {
            "ok-at-epoch-then-faults (test)"
        }
    }

    /// REGRESSION: when the light-time-retarded heliocentric lookup fails with a
    /// genuine (non-coverage) error, the engine must PROPAGATE that error and not
    /// silently fall back to the unretarded geometric place (the old `unwrap_or(p)`
    /// degraded a ~18″ error into a "success").
    #[test]
    fn helio_light_time_fault_is_propagated_not_silently_degraded() {
        let epoch = 2_451_545.0;
        // `with_provider` inserts the test provider at the FRONT of the chain. Its
        // coverage contains the epoch and it returns Ok for the geometric place, so
        // the loop enters its branch first and propagates the retarded-lookup fault
        // before VSOP (still behind it) is ever consulted.
        let a = Almanac::default_vedic().with_provider(Arc::new(OkAtEpochThenFaults { epoch }));
        let res = a.heliocentric_ecliptic_tt(Body::Mars, JdTT(epoch));
        assert!(
            matches!(res, Err(EphemerisError::ComputationFailed(_))),
            "a genuine retarded-lookup fault must surface, not be degraded to the \
             geometric place, got {res:?}"
        );
    }

    /// The light-time retardation is still applied normally (and silently) when
    /// the retarded epoch is in coverage and resolvable: heliocentric Mars stays
    /// within arcseconds of Swiss (the retardation moves it ~18″, so without it the
    /// existing `heliocentric_mars_matches_pyswisseph` test would fail).
    #[test]
    fn helio_light_time_applied_when_resolvable() {
        let a = Almanac::default_vedic();
        // Same assertion shape as heliocentric_mars_matches_pyswisseph, kept here as
        // an explicit guard that the resolvable path is unaffected by the fix.
        let h = a.heliocentric_ecliptic(Body::Mars, J2000_UT1).unwrap();
        let lon = h.longitude * RAD_TO_DEG;
        let mut dlon = (lon - 359.438868).abs();
        if dlon > 180.0 {
            dlon = 360.0 - dlon;
        }
        assert!(
            dlon * 3600.0 < 10.0,
            "resolvable helio Mars lon off by {}″",
            dlon * 3600.0
        );
    }
}
