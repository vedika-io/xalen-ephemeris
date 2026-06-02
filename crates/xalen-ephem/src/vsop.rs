use crate::body::Body;
use crate::provider::{EphemerisError, EphemerisProvider};
use xalen_coords::EclipticPosition;
use xalen_time::{J2000_JD, JdTT, JulianDay};

/// Speed of light in AU per day (exact IAU 2012 value).
/// c = 299_792_458 m/s, 1 AU = 149_597_870_700 m, 1 day = 86400 s.
const LIGHT_SPEED_AU_PER_DAY: f64 = 173.14463267;

/// Apply IAU 2000B nutation in longitude to ecliptic coordinates.
/// Converts from mean-equinox-of-date to true-equinox-of-date (ecliptic-of-date).
fn apply_nutation(mut pos: EclipticPosition, t: f64) -> EclipticPosition {
    let nut = xalen_coords::nutation_2000b(t);
    pos.longitude += nut.delta_psi;
    pos
}

/// Precess an ecliptic position from the dynamical mean equinox of J2000 to the
/// mean equinox of date using the SOFA-validated IAU 2006/P03 (bias-free)
/// rotation matrix. VSOP87 / ELP2000 / Meeus-Pluto / Keplerian-asteroid output
/// is referred to the dynamical J2000 mean equinox (NOT the ICRF/GCRS pole), so
/// the bias-free matrix is the correct rotation. Replaces the prior scalar
/// `longitude += general_precession_longitude(t)` approximation; latitude is now
/// precessed consistently and the radius is preserved. The caller still applies
/// nutation in longitude afterwards for the true (apparent) equinox of date.
fn precess_dynamical_j2000(pos: EclipticPosition, t: f64) -> EclipticPosition {
    let prec = xalen_coords::precession_matrix_p03_nobias(t);
    xalen_coords::precess_ecliptic_to_of_date(pos, prec, t)
}

/// Apparent geocentric ecliptic position of the Moon (ecliptic of date).
///
/// Builds the apparent place from the mean-of-date Meeus Ch.47 series WITHOUT
/// the annual aberration term that planets receive (see the call site for why):
///   1. mean-of-date series longitude/latitude (`geocentric_moon`)
///   2. + nutation in longitude Δψ (IAU 2000B) → true equinox of date
///   3. − geocentric light-time (planetary aberration): the Moon is observed
///      where it was τ = ρ/c ago, so the longitude/latitude it travels during
///      its own light-time is subtracted. The rate is taken from a central
///      finite difference of the series (captures the true ±10 % anomalistic
///      variation of the lunar speed), which is exact to far below the series-
///      truncation floor.
///
/// Validated vs pyswisseph 2.10.03 (AD 1600–2100): residual RMS ≈ 2.82″ (now
/// limited by the 60-term truncation, not aberration); the prior full-annual-
/// aberration path was RMS ≈ 19.1″ / max 44″.
fn apparent_moon(jd_tt: JdTT) -> Result<EclipticPosition, EphemerisError> {
    let mut pos = crate::moon::geocentric_moon(jd_tt)?;

    // (2) nutation in longitude → true (apparent) equinox of date
    let t = jd_tt.julian_centuries_from_j2000();
    pos.longitude += xalen_coords::nutation_2000b(t).delta_psi;

    // (3) geocentric light-time / planetary aberration of the Moon.
    // τ = ρ / c, with ρ in AU and c in AU/day → τ in days.
    let tau = pos.distance / LIGHT_SPEED_AU_PER_DAY;

    // Central finite difference of the mean-of-date series for dλ/dt, dβ/dt
    // (rad/day). h ≈ 0.01 day (~14.4 min) is well inside the series' smoothness
    // and far from any catastrophic cancellation.
    const H: f64 = 0.01;
    let before = crate::moon::geocentric_moon(JdTT(jd_tt.as_f64() - H))?;
    let after = crate::moon::geocentric_moon(JdTT(jd_tt.as_f64() + H))?;
    let mut dlon = after.longitude - before.longitude;
    // unwrap a possible 2π wrap across the sample
    if dlon > std::f64::consts::PI {
        dlon -= std::f64::consts::TAU;
    } else if dlon < -std::f64::consts::PI {
        dlon += std::f64::consts::TAU;
    }
    let dlon_dt = dlon / (2.0 * H);
    let dlat_dt = (after.latitude - before.latitude) / (2.0 * H);

    pos.longitude -= tau * dlon_dt;
    pos.latitude -= tau * dlat_dt;

    Ok(pos)
}

/// VSOP87-based analytical ephemeris provider (Tier 0). Planetary apparent
/// longitudes sit at the ~1-3" level near the modern era; the post-fix analytical
/// Moon validates at RMS ~2.8" / max ~12" vs pyswisseph (AD 1600-2100), no longer
/// the soft spot (an earlier build's ~74" 5M-chart Moon max was the
/// annual-aberration bug, since removed — see `apparent_moon`). The worst PHYSICAL
/// body is now the long-period analytical Pluto (Meeus Ch.37, ~1 arcminute over
/// its 1885-2099 window), so [`accuracy_arcsec`](EphemerisProvider::accuracy_arcsec)
/// reports a conservative physical-body worst-case of 75" — bounding Pluto, never
/// overstating the sub-arcsecond planets/Moon. The derived lunar nodes are
/// characterised separately (see ACCURACY.md). See that method for the full
/// rationale and scope.
pub struct Vsop87Provider;

impl Default for Vsop87Provider {
    fn default() -> Self {
        Self::new()
    }
}

impl Vsop87Provider {
    /// Create a new VSOP87 provider instance.
    pub fn new() -> Self {
        Self
    }

    fn helio_rect(&self, body: Body, jde: f64) -> Result<(f64, f64, f64), EphemerisError> {
        // vsop87 v3.0 API: vsop87::vsop87a::{planet}(jde) -> RectangularCoordinates
        let c = match body {
            Body::Mercury => vsop87::vsop87a::mercury(jde),
            Body::Venus => vsop87::vsop87a::venus(jde),
            Body::Earth => vsop87::vsop87a::earth(jde),
            Body::Mars => vsop87::vsop87a::mars(jde),
            Body::Jupiter => vsop87::vsop87a::jupiter(jde),
            Body::Saturn => vsop87::vsop87a::saturn(jde),
            Body::Uranus => vsop87::vsop87a::uranus(jde),
            Body::Neptune => vsop87::vsop87a::neptune(jde),
            _ => return Err(EphemerisError::BodyNotAvailable(body)),
        };
        Ok((c.x, c.y, c.z))
    }

    /// Raw VSOP87A heliocentric rectangular coordinates (AU) in the ecliptic
    /// frame of equinox J2000 — exactly the quantities tabulated in the official
    /// VSOP87 check file (`vsop87.chk`, Bretagnon & Francou / IMCCE). Exposed so
    /// the analytical theory layer can be validated against the source reference
    /// directly. Returns `None` for bodies not covered by VSOP87 (Sun, Moon,
    /// Pluto, lunar nodes).
    pub fn heliocentric_rectangular_j2000(&self, body: Body, jde: f64) -> Option<(f64, f64, f64)> {
        self.helio_rect(body, jde).ok()
    }
}

impl EphemerisProvider for Vsop87Provider {
    fn heliocentric_ecliptic(
        &self,
        body: Body,
        jd_tt: JdTT,
    ) -> Result<EclipticPosition, EphemerisError> {
        if body == Body::Sun {
            return Ok(EclipticPosition {
                longitude: 0.0,
                latitude: 0.0,
                distance: 0.0,
            });
        }
        // Pluto is not in VSOP87 — use the Meeus Ch.37 series (already heliocentric,
        // referred to the dynamical mean equinox of J2000). Precess + nutate to the
        // ecliptic of date, exactly as the other heliocentric planets below.
        if body == Body::Pluto {
            let t = jd_tt.julian_centuries_from_j2000();
            let pos = crate::pluto::pluto_position(jd_tt)?;
            let pos = precess_dynamical_j2000(pos, t);
            let pos = apply_nutation(pos, t);
            return Ok(pos.normalize());
        }
        let (x, y, z) = self.helio_rect(body, jd_tt.as_f64())?;
        let pos = xalen_coords::cartesian_to_ecliptic(&xalen_coords::CartesianPosition { x, y, z });
        // VSOP87A J2000 → precess + nutate to ecliptic-of-date.
        let t = jd_tt.julian_centuries_from_j2000();
        let pos = precess_dynamical_j2000(pos, t);
        let pos = apply_nutation(pos, t);
        Ok(pos.normalize())
    }

    fn geocentric_ecliptic(
        &self,
        body: Body,
        jd_tt: JdTT,
    ) -> Result<EclipticPosition, EphemerisError> {
        if body == Body::Sun {
            let (ex, ey, ez) = self.helio_rect(Body::Earth, jd_tt.as_f64())?;
            let pos = xalen_coords::cartesian_to_ecliptic(&xalen_coords::CartesianPosition {
                x: -ex,
                y: -ey,
                z: -ez,
            });
            // VSOP87A returns J2000 ecliptic — precess + nutate to ecliptic-of-date
            let t = jd_tt.julian_centuries_from_j2000();
            let pos = precess_dynamical_j2000(pos, t);
            let pos = apply_nutation(pos, t);
            let pos = aberration_correction(pos, jd_tt);
            return Ok(pos.normalize());
        }

        if body == Body::Earth {
            return Ok(EclipticPosition {
                longitude: 0.0,
                latitude: 0.0,
                distance: 0.0,
            });
        }

        if body == Body::MeanNode {
            let t = jd_tt.julian_centuries_from_j2000();
            let omega = mean_node_longitude(t);
            return Ok(EclipticPosition {
                longitude: omega,
                latitude: 0.0,
                distance: 0.0,
            });
        }

        if body == Body::MeanApogee {
            let lon = crate::lilith::mean_lilith(jd_tt);
            return Ok(EclipticPosition {
                longitude: lon,
                latitude: 0.0,
                distance: 0.0,
            });
        }

        if body == Body::OsculatingApogee {
            // True (osculating) Black Moon Lilith — derived from the Moon's
            // instantaneous orbit (see `lilith::true_lilith`). Geometric point:
            // latitude/distance 0, no aberration.
            let lon = crate::lilith::osculating_apogee_longitude(jd_tt)?;
            return Ok(EclipticPosition {
                longitude: lon,
                latitude: 0.0,
                distance: 0.0,
            });
        }

        if body == Body::Moon {
            // The Moon must NOT receive the full annual aberration term
            // (κ = 20.49552″) that `aberration_correction` applies. Annual
            // aberration is the displacement caused by Earth's HELIOCENTRIC
            // velocity, and the geocentric Moon shares that velocity almost
            // exactly — so for a geocentric lunar place it does not apply.
            // Treating the Moon like a planet here injected up to ~44″ of
            // spurious longitude error (validated vs pyswisseph 2.10 over
            // AD 1600–2100: RMS 19.1″, max 44″).
            //
            // The correct apparent geocentric Moon =
            //   mean-of-date series longitude (Meeus Ch.47)
            //   + nutation-in-longitude (Δψ, IAU 2000B)
            //   + geocentric light-time / planetary aberration: the Moon is
            //     seen where it was τ = ρ/c seconds ago, so subtract the small
            //     amount of longitude/latitude it travels during its own
            //     light-time (ρ ≈ 0.0026 AU ⇒ τ ≈ 1.3 s ⇒ ~0.66–0.8″).
            // After this fix the residual vs pyswisseph drops to RMS 2.82″,
            // which is now dominated by the 60-term Meeus SERIES TRUNCATION
            // (not aberration); for sub-arcsecond apparent lunar positions use
            // the DE440 provider.
            return Ok(apparent_moon(jd_tt)?.normalize());
        }

        if body == Body::TrueNode {
            let lon = crate::true_node::true_node_longitude(jd_tt)?;
            return Ok(EclipticPosition {
                longitude: lon,
                latitude: 0.0,
                distance: 0.0,
            });
        }

        if body == Body::Pluto {
            // Meeus Ch.37 gives heliocentric ecliptic referred to the dynamical
            // mean equinox of J2000 — convert to geocentric by subtracting
            // Earth's heliocentric position, same pattern as other planets. The
            // ENTIRE light-time iteration runs in the J2000 frame; the single
            // precession rotation to the equinox of date is applied ONCE at the
            // end (no per-epoch strip-and-readd). `pluto_position` now returns the
            // raw J2000 position, so no precession needs stripping.
            let t = jd_tt.julian_centuries_from_j2000();
            let (ex, ey, ez) = self.helio_rect(Body::Earth, jd_tt.as_f64())?;

            // --- Light-time iteration (1 pass), all in J2000 ecliptic ---
            // Step 1: geometric distance from initial heliocentric position
            let cart_0 = xalen_coords::ecliptic_to_cartesian(&crate::pluto::pluto_position(jd_tt)?);
            let dx = cart_0.x - ex;
            let dy = cart_0.y - ey;
            let dz = cart_0.z - ez;
            let dist_0 = (dx * dx + dy * dy + dz * dz).sqrt();
            let tau_0 = dist_0 / LIGHT_SPEED_AU_PER_DAY; // days

            // Step 2: re-evaluate Pluto at retarded time (t - tau), recompute distance
            let jd_retarded = JdTT(jd_tt.as_f64() - tau_0);
            let cart_1 =
                xalen_coords::ecliptic_to_cartesian(&crate::pluto::pluto_position(jd_retarded)?);
            let dx1 = cart_1.x - ex;
            let dy1 = cart_1.y - ey;
            let dz1 = cart_1.z - ez;
            let dist_1 = (dx1 * dx1 + dy1 * dy1 + dz1 * dz1).sqrt();
            let tau_1 = dist_1 / LIGHT_SPEED_AU_PER_DAY;

            // Step 3: final retarded position
            let jd_final = JdTT(jd_tt.as_f64() - tau_1);
            let cart_final =
                xalen_coords::ecliptic_to_cartesian(&crate::pluto::pluto_position(jd_final)?);

            let geo_cart = xalen_coords::CartesianPosition {
                x: cart_final.x - ex,
                y: cart_final.y - ey,
                z: cart_final.z - ez,
            };
            // J2000 ecliptic geocentric → precess (rigorous rotation, ONCE) →
            // nutate → aberrate = apparent ecliptic-of-date.
            let pos = xalen_coords::cartesian_to_ecliptic(&geo_cart);
            let pos = precess_dynamical_j2000(pos, t);
            let pos = apply_nutation(pos, t);
            let pos = aberration_correction(pos, jd_tt);
            return Ok(pos.normalize());
        }

        if body == Body::Chiron {
            // --- Light-time iteration (1 pass) for Chiron, all in J2000 ---
            // chiron_helio_rect returns J2000-ecliptic rectangular coordinates;
            // the precession rotation to the equinox of date is applied ONCE at
            // the end, after light-time, not as a scalar longitude add.
            let (ex, ey, ez) = self.helio_rect(Body::Earth, jd_tt.as_f64())?;
            let t = jd_tt.julian_centuries_from_j2000();

            // Step 1: geometric distance
            let (cx0, cy0, cz0) = crate::chiron::chiron_helio_rect(jd_tt)?;
            let dx = cx0 - ex;
            let dy = cy0 - ey;
            let dz = cz0 - ez;
            let dist_0 = (dx * dx + dy * dy + dz * dz).sqrt();
            let tau_0 = dist_0 / LIGHT_SPEED_AU_PER_DAY;

            // Step 2: retarded position, recompute distance
            let jd_retarded = JdTT(jd_tt.as_f64() - tau_0);
            let (cx1, cy1, cz1) = crate::chiron::chiron_helio_rect(jd_retarded)?;
            let dx1 = cx1 - ex;
            let dy1 = cy1 - ey;
            let dz1 = cz1 - ez;
            let dist_1 = (dx1 * dx1 + dy1 * dy1 + dz1 * dz1).sqrt();
            let tau_1 = dist_1 / LIGHT_SPEED_AU_PER_DAY;

            // Step 3: final retarded position
            let jd_final = JdTT(jd_tt.as_f64() - tau_1);
            let (cxf, cyf, czf) = crate::chiron::chiron_helio_rect(jd_final)?;
            let geo_cart = xalen_coords::CartesianPosition {
                x: cxf - ex,
                y: cyf - ey,
                z: czf - ez,
            };
            // J2000 ecliptic geocentric → precess (rigorous rotation, ONCE) →
            // nutate → aberrate = apparent ecliptic-of-date.
            let pos = xalen_coords::cartesian_to_ecliptic(&geo_cart);
            let pos = precess_dynamical_j2000(pos, t);
            let pos = apply_nutation(pos, t);
            let pos = aberration_correction(pos, jd_tt);
            return Ok(pos.normalize());
        }

        // Geocentric = planet_helio - earth_helio, with light-time correction.
        // Earth position is always evaluated at the observation epoch.
        let (ex, ey, ez) = self.helio_rect(Body::Earth, jd_tt.as_f64())?;

        // --- Light-time iteration (1 pass) ---
        // Step 1: geometric distance from instantaneous positions
        let (px0, py0, pz0) = self.helio_rect(body, jd_tt.as_f64())?;
        let dx = px0 - ex;
        let dy = py0 - ey;
        let dz = pz0 - ez;
        let dist_0 = (dx * dx + dy * dy + dz * dz).sqrt();
        let tau_0 = dist_0 / LIGHT_SPEED_AU_PER_DAY; // days

        // Step 2: re-evaluate planet at retarded time, recompute distance
        let jde_retarded = jd_tt.as_f64() - tau_0;
        let (px1, py1, pz1) = self.helio_rect(body, jde_retarded)?;
        let dx1 = px1 - ex;
        let dy1 = py1 - ey;
        let dz1 = pz1 - ez;
        let dist_1 = (dx1 * dx1 + dy1 * dy1 + dz1 * dz1).sqrt();
        let tau_1 = dist_1 / LIGHT_SPEED_AU_PER_DAY;

        // Step 3: final retarded position
        let jde_final = jd_tt.as_f64() - tau_1;
        let (pxf, pyf, pzf) = self.helio_rect(body, jde_final)?;

        let geo = xalen_coords::CartesianPosition {
            x: pxf - ex,
            y: pyf - ey,
            z: pzf - ez,
        };

        let pos = xalen_coords::cartesian_to_ecliptic(&geo);
        // VSOP87A J2000 → precess → nutate → aberrate = ecliptic-of-date apparent
        let t = jd_tt.julian_centuries_from_j2000();
        let pos = precess_dynamical_j2000(pos, t);
        let pos = apply_nutation(pos, t);
        let pos = aberration_correction(pos, jd_tt);
        Ok(pos.normalize())
    }

    fn coverage(&self) -> (f64, f64) {
        // VSOP87 valid roughly +-4000 years from J2000
        (J2000_JD - 4000.0 * 365.25, J2000_JD + 4000.0 * 365.25)
    }

    fn accuracy_arcsec(&self) -> f64 {
        // Worst-case apparent-longitude error for the PHYSICAL bodies this
        // provider serves (Sun, Moon, planets). The Moon used to dominate this
        // figure (5M-chart MAX ~74"), but that was the annual-aberration bug:
        // it wrongly applied the full κ=20.49552" term — meant for planets/Sun
        // that do NOT share Earth's heliocentric velocity — to the geocentric
        // Moon. With that removed (see `apparent_moon`) the Moon validates at
        // RMS ~2.8" / max ~12" vs pyswisseph (AD 1600-2100), so the worst
        // physical body is now the long-period analytical Pluto (Meeus Ch.37,
        // ~1 arcminute over its 1885-2099 window). We retain a conservative 75"
        // so the figure never understates the worst physical body (Pluto), and
        // never overstates a precision the analytical models do not achieve.
        //
        // SCOPE: this is the physical-body ephemeris figure. The derived lunar
        // NODES are auxiliary points whose deviation vs Swiss (mean node ~19",
        // true node ~111" max) reflects differing node ALGORITHMS, not ephemeris
        // error, and is characterised separately in docs/ACCURACY.md. Load DE440
        // for sub-arcsecond planets/Moon.
        75.0
    }
    fn name(&self) -> &str {
        "VSOP87 Analytical (Tier 0)"
    }
}

/// Apply first-order annual aberration correction to geocentric ecliptic coordinates.
///
/// The dominant term of the Bradley aberration formula:
///   Δλ ≈ −κ · cos(Sun_lon − λ) / cos(β)
///   Δβ ≈ −κ · sin(β) · sin(Sun_lon − λ)
///
/// where κ = 20.49552″ is the constant of aberration, Sun_lon is the geometric
/// longitude of the Sun, λ is the planet's geocentric longitude, and β is its
/// geocentric latitude.  This corrects for the finite speed of light combined
/// with Earth's orbital velocity — typically ~20″ in longitude.
///
/// `pub(crate)` so the DE440 provider applies the identical apparent-place
/// correction as the analytical path.
pub(crate) fn aberration_correction(
    pos: xalen_coords::EclipticPosition,
    jd_tt: JdTT,
) -> xalen_coords::EclipticPosition {
    use xalen_coords::ARCSEC_TO_RAD;

    // Constant of aberration (Bradley), in radians
    let kappa = 20.49552 * ARCSEC_TO_RAD;

    // Compute Sun's geometric longitude (equinox-of-date) from Earth's VSOP87
    // heliocentric position. This is only the REFERENCE DIRECTION that scales the
    // ~20.5" aberration term, not a body output, so the scalar general-precession
    // approximation is retained here deliberately: the Sun is on the ecliptic
    // (β≈0), where the scalar and rigorous longitudes agree to sub-arcsecond, and
    // a sub-arcsec error in this reference direction perturbs a 20" correction by
    // far less than a milliarcsecond. The body-output sites use the rigorous
    // rotation (precess_dynamical_j2000).
    let earth = vsop87::vsop87a::earth(jd_tt.as_f64());
    let sun_j2000 = xalen_coords::cartesian_to_ecliptic(&xalen_coords::CartesianPosition {
        x: -earth.x,
        y: -earth.y,
        z: -earth.z,
    });
    let t = jd_tt.julian_centuries_from_j2000();
    let sun_lon = sun_j2000.longitude + xalen_coords::general_precession_longitude(t);

    let cos_beta = pos.latitude.cos();
    // Guard against division by zero at the ecliptic poles (|β| ≈ 90°)
    if cos_beta.abs() < 1e-12 {
        return pos;
    }

    let diff = sun_lon - pos.longitude;
    let delta_lon = -kappa * diff.cos() / cos_beta;
    let delta_lat = -kappa * pos.latitude.sin() * diff.sin();

    xalen_coords::EclipticPosition {
        longitude: pos.longitude + delta_lon,
        latitude: pos.latitude + delta_lat,
        distance: pos.distance,
    }
}

fn mean_node_longitude(t: f64) -> f64 {
    // Mean longitude of ascending node (Rahu) — IAU expression
    let omega = (450160.280 - 6962890.539 * t + 7.455 * t * t + 0.008 * t * t * t)
        * xalen_coords::ARCSEC_TO_RAD;
    omega.rem_euclid(std::f64::consts::TAU)
}

#[cfg(test)]
mod tests {
    use super::*;
    use xalen_coords::RAD_TO_DEG;

    #[test]
    fn sun_geocentric_at_j2000() {
        let p = Vsop87Provider::new();
        let sun = p.geocentric_ecliptic(Body::Sun, JdTT::J2000).unwrap();
        let lon_deg = sun.longitude * RAD_TO_DEG;
        // Sun at J2000 (2000-01-01 12:00 TT) should be ~280.5° ecliptic longitude
        assert!(
            (lon_deg - 280.5).abs() < 1.0,
            "Sun at J2000 should be ~280.5°, got {lon_deg}°"
        );
    }

    #[test]
    fn mars_geocentric_reasonable() {
        let p = Vsop87Provider::new();
        let mars = p.geocentric_ecliptic(Body::Mars, JdTT::J2000).unwrap();
        let lon_deg = mars.longitude * RAD_TO_DEG;
        assert!(
            lon_deg >= 0.0 && lon_deg < 360.0,
            "Mars longitude should be 0-360°, got {lon_deg}°"
        );
        assert!(
            mars.distance > 0.3 && mars.distance < 3.0,
            "Mars distance should be 0.3-3.0 AU, got {} AU",
            mars.distance
        );
    }

    #[test]
    fn all_vsop87_planets_computable() {
        let p = Vsop87Provider::new();
        // VSOP87 covers Mercury-Neptune (not Pluto — reclassified 2006, never in VSOP87)
        let vsop_bodies = [
            Body::Sun,
            Body::Moon,
            Body::Mercury,
            Body::Venus,
            Body::Mars,
            Body::Jupiter,
            Body::Saturn,
            Body::Uranus,
            Body::Neptune,
            Body::MeanNode,
            Body::TrueNode,
            Body::Pluto,
            Body::Chiron,
            Body::MeanApogee,
        ];
        for body in vsop_bodies {
            let result = p.geocentric_ecliptic(body, JdTT::J2000);
            assert!(
                result.is_ok(),
                "Failed to compute {body}: {:?}",
                result.err()
            );
        }
    }

    #[test]
    fn pluto_computable_via_meeus() {
        let p = Vsop87Provider::new();
        let result = p.geocentric_ecliptic(Body::Pluto, JdTT::J2000);
        assert!(
            result.is_ok(),
            "Pluto should be computable via Meeus Ch.37: {:?}",
            result.err()
        );
        let lon = result.unwrap().longitude * RAD_TO_DEG;
        assert!(
            lon > 240.0 && lon < 260.0,
            "Pluto lon at J2000 should be ~251°, got {lon}°"
        );
    }

    #[test]
    fn true_node_computable() {
        let p = Vsop87Provider::new();
        let result = p.geocentric_ecliptic(Body::TrueNode, JdTT::J2000);
        assert!(
            result.is_ok(),
            "TrueNode should be computable: {:?}",
            result.err()
        );
        let lon = result.unwrap().longitude * RAD_TO_DEG;
        assert!(
            lon > 120.0 && lon < 130.0,
            "True node at J2000 should be ~125°, got {lon}°"
        );
    }

    #[test]
    fn chiron_computable() {
        let p = Vsop87Provider::new();
        let result = p.geocentric_ecliptic(Body::Chiron, JdTT::J2000);
        assert!(
            result.is_ok(),
            "Chiron should be computable: {:?}",
            result.err()
        );
        let lon = result.unwrap().longitude * RAD_TO_DEG;
        assert!(
            lon >= 0.0 && lon < 360.0,
            "Chiron lon should be 0-360°, got {lon}°"
        );
    }

    #[test]
    fn true_node_near_mean_node() {
        let p = Vsop87Provider::new();
        let mean = p.geocentric_ecliptic(Body::MeanNode, JdTT::J2000).unwrap();
        let true_n = p.geocentric_ecliptic(Body::TrueNode, JdTT::J2000).unwrap();
        let diff = ((true_n.longitude - mean.longitude) * RAD_TO_DEG).abs();
        let diff = if diff > 180.0 { 360.0 - diff } else { diff };
        assert!(
            diff < 2.5,
            "True node should be within 2.5° of mean node, got {diff}°"
        );
    }

    #[test]
    fn mean_node_at_j2000() {
        let p = Vsop87Provider::new();
        let rahu = p.geocentric_ecliptic(Body::MeanNode, JdTT::J2000).unwrap();
        let lon_deg = rahu.longitude * RAD_TO_DEG;
        // Mean node at J2000 should be ~125° (in Cancer-Leo area)
        assert!(
            lon_deg > 100.0 && lon_deg < 150.0,
            "Rahu at J2000 should be ~125°, got {lon_deg}°"
        );
    }

    #[test]
    fn ketu_opposite_rahu() {
        let rahu_lon = 125.0_f64.to_radians();
        let ketu_lon = Body::ketu_longitude(rahu_lon);
        let diff = (ketu_lon - rahu_lon).abs() * RAD_TO_DEG;
        assert!(
            (diff - 180.0).abs() < 0.01,
            "Ketu should be 180° from Rahu, got {diff}°"
        );
    }

    #[test]
    fn provider_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Vsop87Provider>();
    }

    // ========== Bug fix #1: Pluto geocentric vs heliocentric ==========

    #[test]
    fn pluto_geocentric_differs_from_heliocentric() {
        // The geocentric position must differ from the heliocentric position
        // by more than a trivial amount — Earth is ~1 AU from the Sun, and
        // Pluto is ~30 AU away, so the parallax in longitude is roughly
        // arctan(1/30) ≈ 1.9° plus/minus depending on geometry.
        let p = Vsop87Provider::new();
        let geo = p.geocentric_ecliptic(Body::Pluto, JdTT::J2000).unwrap();
        let helio = p.heliocentric_ecliptic(Body::Pluto, JdTT::J2000).unwrap();
        let lon_diff = ((geo.longitude - helio.longitude) * RAD_TO_DEG).abs();
        let lon_diff = if lon_diff > 180.0 {
            360.0 - lon_diff
        } else {
            lon_diff
        };
        // Must differ by at least 0.1° (before the fix it was 0.0° because
        // geocentric was returning heliocentric coordinates verbatim)
        assert!(
            lon_diff > 0.1,
            "Pluto geocentric and heliocentric longitudes should differ by >0.1°, got {lon_diff}°"
        );
        // But should not differ by more than ~5° (geometric parallax for a body at 30 AU)
        assert!(
            lon_diff < 5.0,
            "Pluto geo-helio difference should be <5°, got {lon_diff}° — something is wrong"
        );
    }

    #[test]
    fn pluto_geocentric_distance_differs_from_heliocentric() {
        // Pluto's heliocentric distance is ~30 AU; geocentric should be ~29-31 AU
        // (helio ± ~1 AU from Earth's position). The key assertion: they must NOT
        // be identical.
        let p = Vsop87Provider::new();
        let geo = p.geocentric_ecliptic(Body::Pluto, JdTT::J2000).unwrap();
        let helio = p.heliocentric_ecliptic(Body::Pluto, JdTT::J2000).unwrap();
        let dist_diff = (geo.distance - helio.distance).abs();
        assert!(
            dist_diff > 0.01,
            "Pluto geo/helio distances should differ (Earth offset), got diff = {dist_diff} AU"
        );
        assert!(
            geo.distance > 28.0 && geo.distance < 35.0,
            "Pluto geocentric distance should be ~29-33 AU, got {} AU",
            geo.distance
        );
    }

    #[test]
    fn pluto_geocentric_at_2020() {
        // Pluto in early 2020: geocentric lon ~293° (Sagittarius/Capricorn boundary).
        // The heliocentric is ~292.5°, geocentric offset is small due to large distance.
        let p = Vsop87Provider::new();
        let geo = p.geocentric_ecliptic(Body::Pluto, JdTT(2458849.0)).unwrap();
        let lon = geo.longitude * RAD_TO_DEG;
        assert!(
            lon > 285.0 && lon < 300.0,
            "Pluto geocentric lon in 2020 should be ~293°, got {lon}°"
        );
    }

    // ========== Bug fix #2: Aberration correction ==========

    #[test]
    fn aberration_shifts_sun_longitude() {
        // The Sun's aberration correction should shift its longitude by ~-20.5"
        // (the Sun is always 180° from the anti-Sun, so cos(Sun_lon - Sun_lon) = 1,
        // giving the maximum correction of exactly -kappa).
        let p = Vsop87Provider::new();
        let with_aberr = p.geocentric_ecliptic(Body::Sun, JdTT::J2000).unwrap();

        // Compute Sun without aberration (but with nutation) for comparison
        let earth = vsop87::vsop87a::earth(JdTT::J2000.as_f64());
        let mut sun_no_aberr =
            xalen_coords::cartesian_to_ecliptic(&xalen_coords::CartesianPosition {
                x: -earth.x,
                y: -earth.y,
                z: -earth.z,
            });
        let t = JdTT::J2000.julian_centuries_from_j2000();
        sun_no_aberr.longitude += xalen_coords::general_precession_longitude(t);
        let sun_no_aberr = apply_nutation(sun_no_aberr, t);

        let shift_arcsec = (with_aberr.longitude - sun_no_aberr.longitude) * RAD_TO_DEG * 3600.0;
        // Sun aberration should be approximately -20.5 arcseconds
        assert!(
            (shift_arcsec - (-20.5)).abs() < 1.0,
            "Sun aberration should be ~-20.5\", got {shift_arcsec}\""
        );
    }

    #[test]
    fn aberration_magnitude_bounded_for_planets() {
        // For any planet, the aberration correction in longitude should be
        // bounded by kappa/cos(beta), which is at most ~25" for typical
        // ecliptic latitudes. We verify the final position is still valid.
        let p = Vsop87Provider::new();
        let bodies = [
            Body::Mercury,
            Body::Venus,
            Body::Mars,
            Body::Jupiter,
            Body::Saturn,
            Body::Uranus,
            Body::Neptune,
        ];

        for body in bodies {
            let geo = p.geocentric_ecliptic(body, JdTT::J2000).unwrap();
            // Verify the longitude is valid and the lat is small
            let lon = geo.longitude * RAD_TO_DEG;
            assert!(
                lon >= 0.0 && lon < 360.0,
                "{body}: longitude {lon}° out of range after aberration"
            );
            // Latitude should remain small (aberration correction in lat is << 1°)
            let lat = geo.latitude * RAD_TO_DEG;
            assert!(
                lat.abs() < 15.0,
                "{body}: latitude {lat}° seems too large after aberration"
            );
        }
    }

    #[test]
    fn aberration_not_applied_to_nodes() {
        // Mean and True node are geometric points, not physical bodies.
        // Aberration should NOT be applied to them.
        let p = Vsop87Provider::new();
        let mean = p.geocentric_ecliptic(Body::MeanNode, JdTT::J2000).unwrap();
        let lon = mean.longitude * RAD_TO_DEG;
        // Mean node at J2000 is ~125°. If aberration were wrongly applied,
        // it would shift by up to 20" — we verify the value matches the
        // known analytical formula exactly (no aberration offset).
        let t = JdTT::J2000.julian_centuries_from_j2000();
        let expected = mean_node_longitude(t) * RAD_TO_DEG;
        assert!(
            (lon - expected).abs() < 1e-6,
            "MeanNode should have no aberration: expected {expected}°, got {lon}°"
        );
    }

    #[test]
    fn aberration_correction_function_at_known_angle() {
        // When the planet is at Sun_lon + 90°, cos(Sun_lon - lambda) = cos(-90°) = 0,
        // so the longitude correction should be ~0, and the latitude correction
        // should be at its maximum magnitude.
        let jd = JdTT::J2000;
        // Get the Sun's longitude first
        let earth = vsop87::vsop87a::earth(jd.as_f64());
        let sun_j2000 = xalen_coords::cartesian_to_ecliptic(&xalen_coords::CartesianPosition {
            x: -earth.x,
            y: -earth.y,
            z: -earth.z,
        });
        let t = jd.julian_centuries_from_j2000();
        let sun_lon = sun_j2000.longitude + xalen_coords::general_precession_longitude(t);

        // Place a test body 90° away from the Sun with some latitude
        let test_lat = 5.0_f64.to_radians();
        let test_pos = xalen_coords::EclipticPosition {
            longitude: sun_lon + std::f64::consts::FRAC_PI_2,
            latitude: test_lat,
            distance: 5.0,
        };

        let corrected = aberration_correction(test_pos, jd);
        let delta_lon_arcsec = (corrected.longitude - test_pos.longitude) * RAD_TO_DEG * 3600.0;
        let delta_lat_arcsec = (corrected.latitude - test_pos.latitude) * RAD_TO_DEG * 3600.0;

        // At 90° offset, cos(diff) = 0, so delta_lon should be ~0
        assert!(
            delta_lon_arcsec.abs() < 0.5,
            "At 90° from Sun, lon correction should be ~0\", got {delta_lon_arcsec}\""
        );

        // At 90° offset, sin(diff) = -1, so delta_lat = -kappa * sin(beta) * (-1) = +kappa*sin(beta)
        let expected_lat = 20.49552 * test_lat.sin();
        assert!(
            (delta_lat_arcsec - expected_lat).abs() < 0.5,
            "At 90° from Sun, lat correction should be ~{expected_lat}\", got {delta_lat_arcsec}\""
        );
    }

    // ========== Light-time correction tests ==========

    /// Helper: compute geocentric position WITHOUT light-time correction
    /// by using the geometric (instantaneous) planet position.
    fn geocentric_no_lighttime(body: Body, jd_tt: JdTT) -> EclipticPosition {
        let p = Vsop87Provider::new();
        // For VSOP87 planets: geometric position at same epoch
        let (px, py, pz) = p.helio_rect(body, jd_tt.as_f64()).unwrap();
        let (ex, ey, ez) = p.helio_rect(Body::Earth, jd_tt.as_f64()).unwrap();
        let geo = xalen_coords::CartesianPosition {
            x: px - ex,
            y: py - ey,
            z: pz - ez,
        };
        let mut pos = xalen_coords::cartesian_to_ecliptic(&geo);
        let t = jd_tt.julian_centuries_from_j2000();
        pos.longitude += xalen_coords::general_precession_longitude(t);
        aberration_correction(pos, jd_tt).normalize()
    }

    #[test]
    fn light_time_shifts_nonzero_for_planets() {
        // Both Mars and Jupiter should have nonzero light-time shifts.
        // Note: the longitude shift depends on BOTH light-time (distance) and
        // the planet's angular velocity. Mars moves faster than Jupiter
        // (~0.5 deg/day vs ~0.08 deg/day), so Mars can have a larger longitude
        // shift despite a shorter light-time. This is physically correct.
        let p = Vsop87Provider::new();
        let jd = JdTT::J2000;

        let mars_lt = p.geocentric_ecliptic(Body::Mars, jd).unwrap();
        let mars_no_lt = geocentric_no_lighttime(Body::Mars, jd);
        let mars_shift = ((mars_lt.longitude - mars_no_lt.longitude) * RAD_TO_DEG * 3600.0).abs();

        let jupiter_lt = p.geocentric_ecliptic(Body::Jupiter, jd).unwrap();
        let jupiter_no_lt = geocentric_no_lighttime(Body::Jupiter, jd);
        let jupiter_shift =
            ((jupiter_lt.longitude - jupiter_no_lt.longitude) * RAD_TO_DEG * 3600.0).abs();

        // Both shifts should be non-zero and in a plausible range (sub-arcminute)
        assert!(
            mars_shift > 0.01 && mars_shift < 120.0,
            "Mars light-time shift should be 0.01-120\", got {mars_shift}\""
        );
        assert!(
            jupiter_shift > 0.01 && jupiter_shift < 120.0,
            "Jupiter light-time shift should be 0.01-120\", got {jupiter_shift}\""
        );
    }

    #[test]
    fn jupiter_light_time_plausible() {
        // Jupiter is ~4-6 AU from Earth. Light-time = d/173.14 ~ 0.023-0.035 days
        // = 33-50 minutes. Jupiter moves ~0.083 deg/day, so the shift should be
        // roughly 0.002-0.003 deg = 7-11 arcseconds in longitude.
        let p = Vsop87Provider::new();
        let jd = JdTT::J2000;

        let with_lt = p.geocentric_ecliptic(Body::Jupiter, jd).unwrap();
        let without_lt = geocentric_no_lighttime(Body::Jupiter, jd);

        let shift_arcsec = ((with_lt.longitude - without_lt.longitude) * RAD_TO_DEG * 3600.0).abs();
        // Allow wide tolerance: 1-30 arcseconds (depends on geometry/retrograde)
        assert!(
            shift_arcsec > 0.5 && shift_arcsec < 60.0,
            "Jupiter light-time longitude shift should be ~5-15\", got {shift_arcsec}\""
        );

        // Verify the geocentric distance from the light-time-corrected position
        // is still physically reasonable (4-7 AU)
        assert!(
            with_lt.distance > 3.5 && with_lt.distance < 7.0,
            "Jupiter geocentric distance should be 3.5-7 AU, got {} AU",
            with_lt.distance
        );
    }

    #[test]
    fn saturn_light_time_nonzero() {
        // Saturn is farther than Jupiter, so its light-time shift should be nonzero.
        let p = Vsop87Provider::new();
        let jd = JdTT::J2000;

        let saturn_lt = p.geocentric_ecliptic(Body::Saturn, jd).unwrap();
        let saturn_no = geocentric_no_lighttime(Body::Saturn, jd);
        let sat_shift = ((saturn_lt.longitude - saturn_no.longitude) * RAD_TO_DEG * 3600.0).abs();

        assert!(
            sat_shift > 0.5,
            "Saturn light-time shift should be > 0.5\", got {sat_shift}\""
        );
    }

    #[test]
    fn sun_position_unaffected_by_light_time() {
        // Sun position uses a separate code path with no light-time.
        // Verify it returns the same position regardless (by convention,
        // Sun is geometric -- light-time is baked into aberration instead).
        let p = Vsop87Provider::new();
        let sun = p.geocentric_ecliptic(Body::Sun, JdTT::J2000).unwrap();
        let lon = sun.longitude * RAD_TO_DEG;
        // This should still be ~280.5 deg (same as existing test)
        assert!(
            (lon - 280.5).abs() < 1.0,
            "Sun position should be ~280.5 deg (no light-time), got {lon} deg"
        );
    }

    #[test]
    fn moon_position_unaffected_by_light_time() {
        // Moon has no light-time correction (light-time ~1.3 sec is negligible).
        // Verify the Moon path still works and gives valid coordinates.
        let p = Vsop87Provider::new();
        let moon = p.geocentric_ecliptic(Body::Moon, JdTT::J2000).unwrap();
        let lon = moon.longitude * RAD_TO_DEG;
        assert!(
            lon >= 0.0 && lon < 360.0,
            "Moon longitude should be 0-360 deg, got {lon} deg"
        );
        // Moon distance in AU should be very small (~0.0026 AU)
        assert!(
            moon.distance > 0.001 && moon.distance < 0.005,
            "Moon distance should be ~0.0026 AU, got {} AU",
            moon.distance
        );
    }

    #[test]
    fn node_positions_unaffected_by_light_time() {
        // Mean/True Nodes are mathematical points -- no light-time, no aberration.
        let p = Vsop87Provider::new();
        let mean = p.geocentric_ecliptic(Body::MeanNode, JdTT::J2000).unwrap();
        let true_n = p.geocentric_ecliptic(Body::TrueNode, JdTT::J2000).unwrap();

        // Verify they match the analytical formulas exactly (no light-time shift)
        let t = JdTT::J2000.julian_centuries_from_j2000();
        let expected_mean = mean_node_longitude(t) * RAD_TO_DEG;
        let mean_deg = mean.longitude * RAD_TO_DEG;
        assert!(
            (mean_deg - expected_mean).abs() < 1e-6,
            "MeanNode should be unaffected by light-time: expected {expected_mean} deg, got {mean_deg} deg"
        );

        // TrueNode uses its own series, but should be near MeanNode
        let true_deg = true_n.longitude * RAD_TO_DEG;
        let diff = (true_deg - expected_mean).abs();
        let diff = if diff > 180.0 { 360.0 - diff } else { diff };
        assert!(
            diff < 2.5,
            "TrueNode should be within 2.5 deg of MeanNode, got {diff} deg difference"
        );
    }

    #[test]
    fn pluto_light_time_applied() {
        // Pluto is ~30 AU from Earth. Light-time ~170 min. Pluto moves ~0.004 deg/day.
        // Expected shift is very small (~0.012 deg) but should be nonzero.
        let p = Vsop87Provider::new();
        let jd = JdTT::J2000;

        let with_lt = p.geocentric_ecliptic(Body::Pluto, jd).unwrap();
        // Verify Pluto is still in the right ballpark
        let lon = with_lt.longitude * RAD_TO_DEG;
        assert!(
            lon > 240.0 && lon < 260.0,
            "Pluto lon at J2000 should be ~251 deg, got {lon} deg"
        );
        assert!(
            with_lt.distance > 28.0 && with_lt.distance < 35.0,
            "Pluto distance should be ~30 AU, got {} AU",
            with_lt.distance
        );
    }

    #[test]
    fn chiron_light_time_applied() {
        // Chiron is ~10-12 AU from Earth at J2000. Light-time ~60-70 min.
        let p = Vsop87Provider::new();
        let jd = JdTT::J2000;
        let with_lt = p.geocentric_ecliptic(Body::Chiron, jd).unwrap();

        let lon = with_lt.longitude * RAD_TO_DEG;
        // Chiron at J2000 should still be in the Sagittarius region
        assert!(
            lon > 250.0 && lon < 310.0,
            "Chiron lon at J2000 should be ~270-290 deg, got {lon} deg"
        );
        assert!(
            with_lt.distance > 8.0 && with_lt.distance < 16.0,
            "Chiron distance should be ~8-16 AU, got {} AU",
            with_lt.distance
        );
    }

    #[test]
    fn light_time_all_vsop_planets_still_valid() {
        // Verify all VSOP87 planets still produce valid coordinates after light-time.
        let p = Vsop87Provider::new();
        let bodies = [
            Body::Mercury,
            Body::Venus,
            Body::Mars,
            Body::Jupiter,
            Body::Saturn,
            Body::Uranus,
            Body::Neptune,
        ];

        for body in bodies {
            let pos = p.geocentric_ecliptic(body, JdTT::J2000).unwrap();
            let lon = pos.longitude * RAD_TO_DEG;
            assert!(
                lon >= 0.0 && lon < 360.0,
                "{body}: longitude {lon} deg out of range after light-time"
            );
            let lat = pos.latitude * RAD_TO_DEG;
            assert!(
                lat.abs() < 15.0,
                "{body}: latitude {lat} deg out of range after light-time"
            );
            assert!(
                pos.distance > 0.0,
                "{body}: distance should be positive, got {}",
                pos.distance
            );
        }
    }

    // ========== Mean Apogee (Lilith) via provider ==========

    #[test]
    fn mean_apogee_computable() {
        let p = Vsop87Provider::new();
        let result = p.geocentric_ecliptic(Body::MeanApogee, JdTT::J2000);
        assert!(
            result.is_ok(),
            "MeanApogee should be computable: {:?}",
            result.err()
        );
        let lon = result.unwrap().longitude * RAD_TO_DEG;
        // Lilith at J2000 ≈ 263.35°
        assert!(
            lon > 260.0 && lon < 267.0,
            "Mean Apogee (Lilith) at J2000 should be ~263°, got {lon}°"
        );
    }

    #[test]
    fn mean_apogee_no_aberration() {
        // Mean Apogee is a geometric point — no aberration should be applied.
        let p = Vsop87Provider::new();
        let pos = p
            .geocentric_ecliptic(Body::MeanApogee, JdTT::J2000)
            .unwrap();
        let lon = pos.longitude * RAD_TO_DEG;
        // Compare against the raw analytical formula
        let expected = crate::lilith::mean_lilith(JdTT::J2000) * RAD_TO_DEG;
        assert!(
            (lon - expected).abs() < 1e-6,
            "MeanApogee should have no aberration: expected {expected}°, got {lon}°"
        );
    }

    #[test]
    fn mean_apogee_latitude_zero() {
        // Lilith is defined on the ecliptic plane — latitude must be zero.
        let p = Vsop87Provider::new();
        let pos = p
            .geocentric_ecliptic(Body::MeanApogee, JdTT::J2000)
            .unwrap();
        assert!(
            pos.latitude.abs() < 1e-15,
            "MeanApogee latitude should be exactly 0, got {}",
            pos.latitude
        );
    }

    #[test]
    fn mean_apogee_distance_zero() {
        // Lilith has no physical distance (it is a mathematical point).
        let p = Vsop87Provider::new();
        let pos = p
            .geocentric_ecliptic(Body::MeanApogee, JdTT::J2000)
            .unwrap();
        assert!(
            pos.distance.abs() < 1e-15,
            "MeanApogee distance should be 0, got {}",
            pos.distance
        );
    }

    #[test]
    fn mercury_light_time_smallest() {
        // Mercury, being closest to Earth, should have the smallest light-time
        // shift among the VSOP87 planets (at most epochs).
        let p = Vsop87Provider::new();
        let jd = JdTT::J2000;

        let merc_lt = p.geocentric_ecliptic(Body::Mercury, jd).unwrap();
        let merc_no = geocentric_no_lighttime(Body::Mercury, jd);
        let merc_shift = ((merc_lt.longitude - merc_no.longitude) * RAD_TO_DEG * 3600.0).abs();

        let nep_lt = p.geocentric_ecliptic(Body::Neptune, jd).unwrap();
        let nep_no = geocentric_no_lighttime(Body::Neptune, jd);
        let nep_shift = ((nep_lt.longitude - nep_no.longitude) * RAD_TO_DEG * 3600.0).abs();

        // Both should be nonzero
        assert!(
            merc_shift > 0.001,
            "Mercury light-time shift should be nonzero, got {merc_shift}\""
        );
        assert!(
            nep_shift > 0.001,
            "Neptune light-time shift should be nonzero, got {nep_shift}\""
        );
    }
}
