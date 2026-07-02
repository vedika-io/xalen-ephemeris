//! # xalen-cloud — local-first interpretation shim
//!
//! The design principle is **local-first**: the astronomical chart is computed
//! entirely on-device by the XALEN engine (no network), and only the *optional*
//! interpretation step is delegated to a pluggable [`InterpretationProvider`].
//! A hosted deployment can point that provider at a remote service while the
//! ephemeris itself never leaves the machine.
//!
//! ```no_run
//! use xalen_cloud::{BirthData, compute_chart, OfflineInterpreter, InterpretationProvider};
//! use xalen_ayanamsa::Ayanamsa;
//!
//! // 1. Compute the chart locally (no network).
//! let birth = BirthData::from_calendar(1990, 1, 15, 9.0, 28.6139, 77.2090, Ayanamsa::Lahiri).unwrap();
//! let chart = compute_chart(&birth).unwrap();
//!
//! // 2. Interpret it (here, fully offline; swap in a RemoteInterpreter to call out).
//! let provider = OfflineInterpreter;
//! let interp = provider.interpret(&chart.to_request(None, "en")).unwrap();
//! println!("{}", interp.summary);
//! ```

use serde::{Deserialize, Serialize};
use xalen_ayanamsa::Ayanamsa;
use xalen_ephem::{Almanac, Body};
use xalen_houses::{GeoLocation, HouseSystem, compute_houses_sidereal};
use xalen_time::{DeltaTModel, JdUT1};
use xalen_vedic::nakshatra::Nakshatra;
use xalen_vedic::rashi::Rashi;

/// Errors from chart computation or interpretation.
#[derive(Debug, Clone)]
pub enum CloudError {
    /// Caller-supplied input was out of range or malformed.
    InvalidInput(String),
    /// The ephemeris engine failed to produce a position.
    Ephemeris(String),
    /// The interpretation provider failed (network, auth, decoding, …).
    Interpretation(String),
}

impl std::fmt::Display for CloudError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CloudError::InvalidInput(m) => write!(f, "invalid input: {m}"),
            CloudError::Ephemeris(m) => write!(f, "ephemeris error: {m}"),
            CloudError::Interpretation(m) => write!(f, "interpretation error: {m}"),
        }
    }
}

impl std::error::Error for CloudError {}

/// Birth inputs needed to compute a chart. Times are UT1 Julian Day; use
/// [`BirthData::from_calendar`] to build one from a civil date/time.
#[derive(Debug, Clone, Copy)]
pub struct BirthData {
    /// UT1 Julian Day of birth.
    pub jd_ut1: f64,
    /// Geographic latitude, degrees (north positive).
    pub lat: f64,
    /// Geographic longitude, degrees (east positive).
    pub lon: f64,
    /// Sidereal ayanamsa to apply.
    pub ayanamsa: Ayanamsa,
}

impl BirthData {
    /// Build from a proleptic-Gregorian civil date and **UT** hour-of-day
    /// (already converted from local time, e.g. IST 14:30 → 9.0 UT).
    ///
    /// The civil fields are validated: `month` must be 1–12, `day` must exist in
    /// that month (proleptic-Gregorian leap rule), and `ut_hour` must be finite
    /// and in `[0, 24)`. The geographic `lat`/`lon` and the resulting JD are
    /// additionally validated by [`compute_chart`].
    pub fn from_calendar(
        year: i32,
        month: u32,
        day: u32,
        ut_hour: f64,
        lat: f64,
        lon: f64,
        ayanamsa: Ayanamsa,
    ) -> Result<Self, CloudError> {
        if !(1..=12).contains(&month) {
            return Err(CloudError::InvalidInput(format!(
                "month must be 1-12, got {month}"
            )));
        }
        if day < 1 || day > days_in_month(year, month) {
            return Err(CloudError::InvalidInput(format!(
                "day must be 1-{} for {year}-{month:02}, got {day}",
                days_in_month(year, month)
            )));
        }
        if !ut_hour.is_finite() || !(0.0..24.0).contains(&ut_hour) {
            return Err(CloudError::InvalidInput(format!(
                "ut_hour must be finite and in [0, 24), got {ut_hour}"
            )));
        }
        let jd = xalen_time::calendar_to_jd(
            year,
            month,
            day,
            ut_hour,
            xalen_time::CalendarSystem::ProlepticGregorian,
        );
        Ok(Self {
            jd_ut1: jd.0,
            lat,
            lon,
            ayanamsa,
        })
    }
}

/// Days in a proleptic-Gregorian month (`month` must already be 1–12).
fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ => {
            let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
            if leap { 29 } else { 28 }
        }
    }
}

/// A single body's placement in the locally-computed chart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetSummary {
    /// Graha name (e.g. "Sun", "Rahu", "Ketu").
    pub name: String,
    /// Sidereal ecliptic longitude, degrees `[0, 360)`.
    pub longitude_deg: f64,
    /// Rashi (sign) name.
    pub rashi: String,
    /// Nakshatra name.
    pub nakshatra: String,
    /// Nakshatra pada (1-4).
    pub pada: u8,
    /// Whole-sign house from the ascendant (1-12).
    pub house: usize,
}

/// The full locally-computed chart. This is the ONLY payload that an
/// [`InterpretationProvider`] receives — raw birth data never leaves the device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSummary {
    /// Each graha's placement.
    pub planets: Vec<PlanetSummary>,
    /// Ascendant sidereal longitude, degrees.
    pub ascendant_deg: f64,
    /// Ascendant rashi name.
    pub ascendant_rashi: String,
    /// Applied ayanamsa, degrees.
    pub ayanamsa_deg: f64,
}

impl ChartSummary {
    /// Wrap this chart in an [`InterpretationRequest`].
    pub fn to_request(&self, query: Option<String>, locale: &str) -> InterpretationRequest {
        InterpretationRequest {
            chart: self.clone(),
            query,
            locale: locale.to_string(),
        }
    }
}

const fn graha_name(body: Body) -> &'static str {
    match body {
        Body::Sun => "Sun",
        Body::Moon => "Moon",
        Body::Mars => "Mars",
        Body::Mercury => "Mercury",
        Body::Jupiter => "Jupiter",
        Body::Venus => "Venus",
        Body::Saturn => "Saturn",
        Body::MeanNode => "Rahu",
        _ => "?",
    }
}

fn summarize(name: &str, lon: f64, asc_sign: usize) -> PlanetSummary {
    let lon = lon.rem_euclid(360.0);
    let psign = Rashi::from_longitude_deg(lon).index();
    PlanetSummary {
        name: name.to_string(),
        longitude_deg: lon,
        rashi: Rashi::from_longitude_deg(lon).to_string(),
        nakshatra: Nakshatra::from_longitude_deg(lon).to_string(),
        pada: Nakshatra::pada(lon),
        house: (psign + 12 - asc_sign) % 12 + 1,
    }
}

/// Compute the chart **entirely locally** — no network access.
pub fn compute_chart(birth: &BirthData) -> Result<ChartSummary, CloudError> {
    // Validate inputs before touching the engine (no bad/NaN states downstream).
    if !birth.jd_ut1.is_finite() {
        return Err(CloudError::Ephemeris("birth jd_ut1 is not finite".into()));
    }
    let loc = GeoLocation::try_new(birth.lat, birth.lon).ok_or_else(|| {
        CloudError::Ephemeris(format!(
            "invalid location: lat={}, lon={} (expected lat -90..=90, lon -180..=180, finite)",
            birth.lat, birth.lon
        ))
    })?;

    let jd_ut1 = JdUT1(birth.jd_ut1);
    let jd_tt = jd_ut1.to_tt(&DeltaTModel::StephensonMorrisonHohenkerk2016);
    let aya_deg = birth.ayanamsa.compute_deg(jd_tt.0);

    // SIDEREAL ascendant + whole-sign houses, consistent with the sidereal
    // planet positions below: `compute_houses_sidereal` shifts the tropical
    // ascendant by the ayanamsa. (Subtracting the ayanamsa is essential — the
    // planets are sidereal, so a tropical ascendant would put them in the wrong
    // houses.)
    let t = (birth.jd_ut1 - 2_451_545.0) / 36525.0;
    let epsilon = xalen_coords::obliquity::mean_obliquity(t);
    let houses = compute_houses_sidereal(
        birth.jd_ut1,
        &loc,
        epsilon,
        aya_deg.to_radians(),
        HouseSystem::WholeSign,
    );
    let asc_deg = houses.ascendant.to_degrees().rem_euclid(360.0);
    let asc_sign = Rashi::from_longitude_deg(asc_deg).index();

    let almanac = Almanac::default_vedic();
    let mut planets = Vec::with_capacity(9);
    let mut rahu = 0.0;
    for &body in Body::VEDIC_GRAHAS {
        let sid = almanac
            .sidereal_longitude_deg(body, jd_ut1, aya_deg)
            .map_err(|e| CloudError::Ephemeris(e.to_string()))?;
        if body == Body::MeanNode {
            rahu = sid;
        }
        planets.push(summarize(graha_name(body), sid, asc_sign));
    }
    // Ketu is exactly opposite Rahu.
    planets.push(summarize(
        "Ketu",
        (rahu + 180.0).rem_euclid(360.0),
        asc_sign,
    ));

    Ok(ChartSummary {
        planets,
        ascendant_deg: asc_deg,
        ascendant_rashi: Rashi::from_longitude_deg(asc_deg).to_string(),
        ayanamsa_deg: aya_deg,
    })
}

/// A request to interpret a locally-computed chart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterpretationRequest {
    /// The locally-computed chart (the only data sent to a provider).
    pub chart: ChartSummary,
    /// Optional free-text question to focus the interpretation. Not validated
    /// or length-limited here; providers should enforce their own limits.
    pub query: Option<String>,
    /// Locale tag, e.g. "en", "hi". Expected to be BCP-47 but passed through to
    /// the provider unvalidated.
    pub locale: String,
}

/// A titled block of interpretation text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterpretationSection {
    /// Section heading (e.g. "Career", "Relationships").
    pub title: String,
    /// Section body text.
    pub body: String,
}

/// An interpretation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interpretation {
    /// Human-readable summary.
    pub summary: String,
    /// Optional structured sections.
    #[serde(default)]
    pub sections: Vec<InterpretationSection>,
    /// Provider identifier (e.g. "offline", "vedika").
    pub provider: String,
}

/// Something that turns a chart into an interpretation. The chart is computed
/// locally; only this step may reach the network (and only if the concrete
/// provider does so).
pub trait InterpretationProvider {
    fn interpret(&self, request: &InterpretationRequest) -> Result<Interpretation, CloudError>;
}

/// A fully **offline** provider: it states the factual placements without any
/// network call or predictive claims. Useful as a default, for tests, and for
/// privacy-strict deployments.
pub struct OfflineInterpreter;

impl InterpretationProvider for OfflineInterpreter {
    fn interpret(&self, request: &InterpretationRequest) -> Result<Interpretation, CloudError> {
        let c = &request.chart;
        let mut summary = format!(
            "Ascendant {} ({:.2}°), ayanamsa {:.3}°. ",
            c.ascendant_rashi, c.ascendant_deg, c.ayanamsa_deg
        );
        for p in &c.planets {
            summary.push_str(&format!(
                "{} in {} (house {}, {} pada {}); ",
                p.name, p.rashi, p.house, p.nakshatra, p.pada
            ));
        }
        Ok(Interpretation {
            summary: summary.trim_end().to_string(),
            sections: Vec::new(),
            provider: "offline".into(),
        })
    }
}

/// A remote provider that POSTs the locally-computed chart as JSON to an
/// interpretation endpoint and decodes the response. This is the ONLY network
/// path in the funnel: the ephemeris is computed locally; the chart (never the
/// raw birth data beyond what the chart already encodes) is sent for
/// interpretation. Available with the `remote-http` feature.
#[cfg(feature = "remote-http")]
pub struct RemoteInterpreter {
    /// Full URL of the interpretation endpoint.
    pub endpoint: String,
    /// Optional bearer token sent as `Authorization: Bearer <token>`.
    pub api_key: Option<String>,
    /// Request timeout.
    pub timeout: std::time::Duration,
}

#[cfg(feature = "remote-http")]
impl RemoteInterpreter {
    /// Create a remote interpreter for `endpoint` with a default 15s timeout.
    ///
    /// The endpoint must be a non-empty `http://` or `https://` URL. Note that
    /// [`interpret`](InterpretationProvider::interpret) additionally refuses to
    /// send a bearer token over plain `http://`.
    pub fn new(endpoint: impl Into<String>) -> Result<Self, CloudError> {
        let endpoint: String = endpoint.into();
        let trimmed = endpoint.trim();
        if trimmed.is_empty() {
            return Err(CloudError::InvalidInput(
                "endpoint must not be empty".into(),
            ));
        }
        if !(trimmed.starts_with("https://") || trimmed.starts_with("http://")) {
            return Err(CloudError::InvalidInput(format!(
                "endpoint must start with http:// or https://, got {trimmed:?}"
            )));
        }
        Ok(Self {
            endpoint: trimmed.to_string(),
            api_key: None,
            timeout: std::time::Duration::from_secs(15),
        })
    }

    /// Set a bearer token.
    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }
}

#[cfg(feature = "remote-http")]
impl InterpretationProvider for RemoteInterpreter {
    fn interpret(&self, request: &InterpretationRequest) -> Result<Interpretation, CloudError> {
        if self.api_key.is_some() && !self.endpoint.starts_with("https://") {
            return Err(CloudError::InvalidInput(
                "refusing to send a bearer token over non-https endpoint".into(),
            ));
        }
        let agent = ureq::AgentBuilder::new().timeout(self.timeout).build();
        let mut req = agent.post(&self.endpoint);
        if let Some(key) = &self.api_key {
            req = req.set("Authorization", &format!("Bearer {key}"));
        }
        let body = serde_json::to_value(request)
            .map_err(|e| CloudError::Interpretation(format!("serialize: {e}")))?;
        let resp = req
            .send_json(body)
            .map_err(|e| CloudError::Interpretation(format!("request: {e}")))?;
        resp.into_json::<Interpretation>()
            .map_err(|e| CloudError::Interpretation(format!("decode: {e}")))
    }
}

/// Convenience: compute the chart locally and interpret it with `provider`.
pub fn chart_and_interpret(
    birth: &BirthData,
    provider: &dyn InterpretationProvider,
    query: Option<String>,
    locale: &str,
) -> Result<(ChartSummary, Interpretation), CloudError> {
    let chart = compute_chart(birth)?;
    let interp = provider.interpret(&chart.to_request(query, locale))?;
    Ok((chart, interp))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> BirthData {
        // 1990-01-15 14:30 IST (UT 9.0), New Delhi.
        BirthData::from_calendar(1990, 1, 15, 9.0, 28.6139, 77.2090, Ayanamsa::Lahiri).unwrap()
    }

    #[test]
    fn from_calendar_rejects_invalid_civil_input() {
        let ok = |y, m, d, h| BirthData::from_calendar(y, m, d, h, 0.0, 0.0, Ayanamsa::Lahiri);
        assert!(matches!(
            ok(1990, 0, 1, 0.0),
            Err(CloudError::InvalidInput(_))
        ));
        assert!(matches!(
            ok(1990, 13, 1, 0.0),
            Err(CloudError::InvalidInput(_))
        ));
        assert!(matches!(
            ok(1990, 2, 29, 0.0),
            Err(CloudError::InvalidInput(_))
        )); // 1990 not leap
        assert!(ok(2000, 2, 29, 0.0).is_ok()); // 2000 is leap (÷400)
        assert!(matches!(
            ok(1900, 2, 29, 0.0),
            Err(CloudError::InvalidInput(_))
        )); // 1900 not leap (÷100)
        assert!(matches!(
            ok(1990, 4, 31, 0.0),
            Err(CloudError::InvalidInput(_))
        ));
        assert!(matches!(
            ok(1990, 1, 1, 24.0),
            Err(CloudError::InvalidInput(_))
        ));
        assert!(matches!(
            ok(1990, 1, 1, f64::NAN),
            Err(CloudError::InvalidInput(_))
        ));
        assert!(ok(1990, 12, 31, 23.999).is_ok());
    }

    #[test]
    fn computes_chart_locally() {
        let c = compute_chart(&sample()).unwrap();
        assert_eq!(c.planets.len(), 9, "7 grahas + Rahu + Ketu");
        // Ascendant must be a valid sign; Sun in mid-January is sidereal Capricorn (Lahiri).
        assert!(c.ascendant_deg >= 0.0 && c.ascendant_deg < 360.0);
        let sun = c.planets.iter().find(|p| p.name == "Sun").unwrap();
        assert!(
            sun.rashi.contains("Capricorn"),
            "sidereal Sun mid-Jan is Capricorn, got {}",
            sun.rashi
        );
        // Rahu and Ketu are exactly opposite.
        let rahu = c.planets.iter().find(|p| p.name == "Rahu").unwrap();
        let ketu = c.planets.iter().find(|p| p.name == "Ketu").unwrap();
        let diff = (rahu.longitude_deg - ketu.longitude_deg).rem_euclid(360.0);
        assert!((diff - 180.0).abs() < 1e-6);
    }

    #[test]
    fn whole_sign_houses_are_consistent() {
        let c = compute_chart(&sample()).unwrap();
        for p in &c.planets {
            assert!((1..=12).contains(&p.house));
        }
    }

    #[test]
    fn offline_interpreter_is_deterministic_and_networkless() {
        let c = compute_chart(&sample()).unwrap();
        let req = c.to_request(None, "en");
        let a = OfflineInterpreter.interpret(&req).unwrap();
        let b = OfflineInterpreter.interpret(&req).unwrap();
        assert_eq!(a.summary, b.summary);
        assert_eq!(a.provider, "offline");
        assert!(a.summary.contains("Ascendant"));
    }

    #[test]
    fn chart_summary_serde_roundtrips() {
        let c = compute_chart(&sample()).unwrap();
        let json = serde_json::to_string(&c).unwrap();
        let back: ChartSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(back.planets.len(), c.planets.len());
        assert_eq!(back.ascendant_rashi, c.ascendant_rashi);
    }

    #[test]
    fn convenience_chart_and_interpret() {
        let (chart, interp) =
            chart_and_interpret(&sample(), &OfflineInterpreter, None, "en").unwrap();
        assert_eq!(chart.planets.len(), 9);
        assert!(!interp.summary.is_empty());
    }

    #[test]
    fn sidereal_ascendant_equals_tropical_minus_ayanamsa() {
        // The ascendant must be SIDEREAL (tropical shifted by the ayanamsa),
        // consistent with the sidereal planet longitudes. Guards the bug where a
        // tropical ascendant would put planets in the wrong whole-sign houses.
        use xalen_houses::{GeoLocation, HouseSystem, compute_houses};
        let b = sample();
        let c = compute_chart(&b).unwrap();
        let t = (b.jd_ut1 - 2_451_545.0) / 36525.0;
        let eps = xalen_coords::obliquity::mean_obliquity(t);
        let loc = GeoLocation::try_new(b.lat, b.lon).unwrap();
        let tropical = compute_houses(b.jd_ut1, &loc, eps, HouseSystem::WholeSign)
            .ascendant
            .to_degrees()
            .rem_euclid(360.0);
        let expected = (tropical - c.ayanamsa_deg).rem_euclid(360.0);
        let d = (c.ascendant_deg - expected).rem_euclid(360.0);
        let d = d.min(360.0 - d);
        assert!(
            d < 1e-6,
            "ascendant not sidereal: got {}, expected {}",
            c.ascendant_deg,
            expected
        );
        // ...and it must differ from the tropical ascendant by ~the ayanamsa.
        assert!((c.ascendant_deg - tropical).abs() > 1.0);
    }

    #[test]
    fn rejects_invalid_inputs() {
        let with = |jd: f64, lat: f64, lon: f64| BirthData {
            jd_ut1: jd,
            lat,
            lon,
            ayanamsa: Ayanamsa::Lahiri,
        };
        // out-of-range / non-finite latitude and longitude
        assert!(
            compute_chart(&with(2_447_907.0, 200.0, 0.0)).is_err(),
            "lat 200"
        );
        assert!(
            compute_chart(&with(2_447_907.0, -91.0, 0.0)).is_err(),
            "lat -91"
        );
        assert!(
            compute_chart(&with(2_447_907.0, 0.0, 200.0)).is_err(),
            "lon 200"
        );
        assert!(
            compute_chart(&with(2_447_907.0, f64::NAN, 0.0)).is_err(),
            "lat NaN"
        );
        assert!(
            compute_chart(&with(2_447_907.0, 0.0, f64::INFINITY)).is_err(),
            "lon inf"
        );
        // non-finite Julian Day
        assert!(compute_chart(&with(f64::NAN, 0.0, 0.0)).is_err(), "jd NaN");
        assert!(
            compute_chart(&with(f64::INFINITY, 0.0, 0.0)).is_err(),
            "jd inf"
        );
        // a valid input still succeeds
        assert!(compute_chart(&with(2_447_907.0, 28.6, 77.2)).is_ok());
    }

    #[test]
    fn interpretation_serde_roundtrips_with_sections() {
        let interp = Interpretation {
            summary: "s".into(),
            sections: vec![
                InterpretationSection {
                    title: "Career".into(),
                    body: "x".into(),
                },
                InterpretationSection {
                    title: "Health".into(),
                    body: "y".into(),
                },
            ],
            provider: "vedika".into(),
        };
        let back: Interpretation =
            serde_json::from_str(&serde_json::to_string(&interp).unwrap()).unwrap();
        assert_eq!(back.summary, "s");
        assert_eq!(back.provider, "vedika");
        assert_eq!(back.sections.len(), 2);
        assert_eq!(back.sections[0].title, "Career");
        assert_eq!(back.sections[0].body, "x");
        assert_eq!(back.sections[1].title, "Health");
        assert_eq!(back.sections[1].body, "y");

        // The full request (chart + query + locale) round-trips field-for-field.
        let chart = compute_chart(&sample()).unwrap();
        let req = chart.to_request(Some("career?".into()), "en");
        let rback: InterpretationRequest =
            serde_json::from_str(&serde_json::to_string(&req).unwrap()).unwrap();
        assert_eq!(rback.locale, "en");
        assert_eq!(rback.query.as_deref(), Some("career?"));
        assert_eq!(rback.chart.planets.len(), chart.planets.len());
        assert_eq!(rback.chart.ascendant_rashi, chart.ascendant_rashi);
        assert!((rback.chart.ascendant_deg - chart.ascendant_deg).abs() < 1e-9);
        assert!((rback.chart.ayanamsa_deg - chart.ayanamsa_deg).abs() < 1e-9);
        let (a, b) = (&rback.chart.planets[0], &chart.planets[0]);
        assert_eq!(a.name, b.name);
        assert!((a.longitude_deg - b.longitude_deg).abs() < 1e-9);
        assert_eq!(a.house, b.house);
    }
}
