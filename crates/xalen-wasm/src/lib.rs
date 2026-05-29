// WASM bindings for XALEN Ephemeris
// Compile with: wasm-pack build --target web

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use xalen_ayanamsa::Ayanamsa;
use xalen_coords::obliquity::mean_obliquity;
use xalen_ephem::{Almanac, Body};
use xalen_houses::{GeoLocation, HouseSystem, compute_houses};
use xalen_time::{DeltaTModel, JdUT1, JulianDay, delta_t};
use xalen_vedic::compatibility::ashtakoota_match;
use xalen_vedic::dasha::{DashaLevel, vimshottari_dasha};
use xalen_vedic::divisional::{VargaChart, compute_varga_sign};
use xalen_vedic::nakshatra::Nakshatra;
use xalen_vedic::panchang::compute_panchang;
use xalen_vedic::rashi::Rashi;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct XalenWasm {
    almanac: Almanac,
}

impl Default for XalenWasm {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl XalenWasm {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new() -> Self {
        Self {
            almanac: Almanac::default_vedic(),
        }
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "tropicalLongitude"))]
    pub fn tropical_longitude(&self, jd_ut1: f64, body_id: u8) -> Result<f64, String> {
        // Handle Ketu (id 13) as Rahu + 180
        if body_id == 13 {
            let rahu_lon = self
                .almanac
                .geocentric_longitude_deg(Body::MeanNode, JdUT1(jd_ut1))
                .map_err(|e| e.to_string())?;
            return Ok((rahu_lon + 180.0).rem_euclid(360.0));
        }
        let body = body_from_id(body_id)?;
        self.almanac
            .geocentric_longitude_deg(body, JdUT1(jd_ut1))
            .map_err(|e| e.to_string())
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "siderealLongitude"))]
    pub fn sidereal_longitude(
        &self,
        jd_ut1: f64,
        body_id: u8,
        ayanamsa_id: u8,
    ) -> Result<f64, String> {
        let aya = ayanamsa_from_id(ayanamsa_id)?;
        let jd = JdUT1(jd_ut1);
        let tt = jd.to_tt(&DeltaTModel::StephensonMorrisonHohenkerk2016);
        let aya_deg = aya.compute_deg(tt.as_f64());

        // Handle Ketu (id 13) as Rahu + 180
        if body_id == 13 {
            let rahu_lon = self
                .almanac
                .sidereal_longitude_deg(Body::MeanNode, jd, aya_deg)
                .map_err(|e| e.to_string())?;
            return Ok((rahu_lon + 180.0).rem_euclid(360.0));
        }

        let body = body_from_id(body_id)?;
        self.almanac
            .sidereal_longitude_deg(body, jd, aya_deg)
            .map_err(|e| e.to_string())
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "getNakshatra"))]
    pub fn nakshatra(&self, moon_sidereal_deg: f64) -> String {
        let nak = Nakshatra::from_longitude_deg(moon_sidereal_deg);
        let pada = Nakshatra::pada(moon_sidereal_deg);
        format!("{} Pada {}", nak, pada)
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "getRashi"))]
    pub fn rashi(&self, sidereal_deg: f64) -> String {
        Rashi::from_longitude_deg(sidereal_deg).to_string()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "panchangJson"))]
    pub fn panchang_json(&self, jd_ut1: f64, ayanamsa_id: u8) -> Result<String, String> {
        let aya = ayanamsa_from_id(ayanamsa_id)?;
        let jd = JdUT1(jd_ut1);
        let tt = jd.to_tt(&DeltaTModel::StephensonMorrisonHohenkerk2016);
        let aya_deg = aya.compute_deg(tt.as_f64());

        let sun_deg = self
            .almanac
            .sidereal_longitude_deg(Body::Sun, jd, aya_deg)
            .map_err(|e| e.to_string())?;
        let moon_deg = self
            .almanac
            .sidereal_longitude_deg(Body::Moon, jd, aya_deg)
            .map_err(|e| e.to_string())?;

        let panchang = compute_panchang(sun_deg, moon_deg, jd_ut1);
        serde_json::to_string(&panchang).map_err(|e| e.to_string())
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "housesJson"))]
    pub fn houses_json(
        &self,
        jd_ut1: f64,
        lat: f64,
        lon: f64,
        system_id: u8,
    ) -> Result<String, String> {
        let system = house_system_from_id(system_id)?;
        let loc = GeoLocation::new(lat, lon);
        let t = (jd_ut1 - 2_451_545.0) / 36525.0;
        let epsilon = mean_obliquity(t);
        let houses = compute_houses(jd_ut1, &loc, epsilon, system);
        serde_json::to_string(&houses).map_err(|e| e.to_string())
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "fullChartJson"))]
    pub fn full_chart_json(
        &self,
        jd_ut1: f64,
        lat: f64,
        lon: f64,
        ayanamsa_id: u8,
    ) -> Result<String, String> {
        let aya = ayanamsa_from_id(ayanamsa_id)?;
        let jd = JdUT1(jd_ut1);
        let tt = jd.to_tt(&DeltaTModel::StephensonMorrisonHohenkerk2016);
        let aya_deg = aya.compute_deg(tt.as_f64());

        let mut planets = serde_json::Map::new();
        for &body in Body::VEDIC_GRAHAS {
            match self.almanac.sidereal_longitude_deg(body, jd, aya_deg) {
                Ok(lon) => {
                    let nak = Nakshatra::from_longitude_deg(lon);
                    let rashi = Rashi::from_longitude_deg(lon);
                    let pada = Nakshatra::pada(lon);
                    let mut info = serde_json::Map::new();
                    info.insert("longitude".into(), serde_json::Value::from(lon));
                    info.insert("nakshatra".into(), serde_json::Value::from(nak.to_string()));
                    info.insert("pada".into(), serde_json::Value::from(pada));
                    info.insert("rashi".into(), serde_json::Value::from(rashi.to_string()));
                    planets.insert(body.to_string(), serde_json::Value::Object(info));
                }
                Err(e) => {
                    planets.insert(
                        body.to_string(),
                        serde_json::Value::from(format!("error: {e}")),
                    );
                }
            }
        }

        let loc = GeoLocation::new(lat, lon);
        let t = (jd_ut1 - 2_451_545.0) / 36525.0;
        let epsilon = mean_obliquity(t);
        let houses = compute_houses(jd_ut1, &loc, epsilon, HouseSystem::WholeSign);

        let mut chart = serde_json::Map::new();
        chart.insert("planets".into(), serde_json::Value::Object(planets));
        chart.insert(
            "ascendant_deg".into(),
            serde_json::Value::from(houses.ascendant.to_degrees().rem_euclid(360.0)),
        );
        chart.insert(
            "mc_deg".into(),
            serde_json::Value::from(houses.mc.to_degrees().rem_euclid(360.0)),
        );
        chart.insert("ayanamsa_deg".into(), serde_json::Value::from(aya_deg));

        Ok(serde_json::to_string_pretty(&serde_json::Value::Object(chart)).unwrap())
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "julianDay"))]
    pub fn julian_day(year: i32, month: u32, day: u32, hour: f64) -> f64 {
        xalen_time::calendar_to_jd(
            year,
            month,
            day,
            hour,
            xalen_time::CalendarSystem::ProlepticGregorian,
        )
        .as_f64()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "ayanamsaDeg"))]
    pub fn ayanamsa_deg(jd_ut1: f64, ayanamsa_id: u8) -> Result<f64, String> {
        let aya = ayanamsa_from_id(ayanamsa_id)?;
        let jd = JdUT1(jd_ut1);
        let tt = jd.to_tt(&DeltaTModel::StephensonMorrisonHohenkerk2016);
        Ok(aya.compute_deg(tt.as_f64()))
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "bodyName"))]
    pub fn body_name(body_id: u8) -> String {
        match body_from_id(body_id) {
            Ok(b) => b.to_string(),
            Err(e) => e,
        }
    }

    /// Delta T (TT - UT1) in seconds at a given Julian Day.
    /// Uses the Stephenson-Morrison-Hohenkerk 2016 model.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "deltaT"))]
    pub fn delta_t_seconds(jd: f64) -> f64 {
        delta_t(jd, &DeltaTModel::StephensonMorrisonHohenkerk2016)
    }

    /// Compute the Vimshottari Dasha periods from Moon longitude and birth JD.
    /// Returns JSON array of Mahadasha periods, each with Antardasha sub-periods.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "vimshottariDasha"))]
    pub fn vimshottari_dasha_json(moon_deg: f64, birth_jd: f64) -> String {
        let periods = vimshottari_dasha(moon_deg, birth_jd, DashaLevel::Antardasha);
        serde_json::to_string(&periods).unwrap_or_default()
    }

    /// Compute Ashta Koota (8-fold) compatibility from boy and girl nakshatra indices.
    /// Indices: 0=Ashwini .. 26=Revati. Returns JSON with all 8 koota scores.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "compatibility"))]
    pub fn compatibility(boy_nak: u8, girl_nak: u8) -> Result<String, String> {
        if boy_nak >= 27 || girl_nak >= 27 {
            return Err(format!(
                "Nakshatra index must be 0-26. Got boy={boy_nak}, girl={girl_nak}"
            ));
        }
        let boy = Nakshatra::ALL[boy_nak as usize];
        let girl = Nakshatra::ALL[girl_nak as usize];
        // Derive rashi index from nakshatra midpoint longitude
        let boy_rashi = (boy_nak as usize * 4 / 9) % 12;
        let girl_rashi = (girl_nak as usize * 4 / 9) % 12;
        let result = ashtakoota_match(boy, girl, boy_rashi, girl_rashi);
        serde_json::to_string(&result).map_err(|e| e.to_string())
    }

    /// Compute the divisional (Varga) chart sign for a given longitude.
    /// varga: 1=D1, 2=D2, 3=D3, 4=D4, 7=D7, 9=D9, 10=D10, 12=D12,
    ///        16=D16, 20=D20, 24=D24, 27=D27, 30=D30, 40=D40, 45=D45, 60=D60.
    /// Returns the rashi (sign) name in that divisional chart.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "divisionalChart"))]
    pub fn divisional_chart(lon_deg: f64, varga: u8) -> Result<String, String> {
        let chart = varga_from_id(varga)?;
        let rashi = compute_varga_sign(lon_deg, chart);
        Ok(rashi.to_string())
    }
}

/// Canonical body ID mapping (shared across all bindings).
/// 0=Sun, 1=Moon, 2=Mercury, 3=Venus, 4=Mars, 5=Jupiter, 6=Saturn,
/// 7=Uranus, 8=Neptune, 9=Rahu/MeanNode, 10=TrueNode, 11=Pluto,
/// 12=Chiron, 13=Ketu (computed as Rahu+180, handled by callers)
fn body_from_id(id: u8) -> Result<Body, String> {
    match id {
        0 => Ok(Body::Sun),
        1 => Ok(Body::Moon),
        2 => Ok(Body::Mercury),
        3 => Ok(Body::Venus),
        4 => Ok(Body::Mars),
        5 => Ok(Body::Jupiter),
        6 => Ok(Body::Saturn),
        7 => Ok(Body::Uranus),
        8 => Ok(Body::Neptune),
        9 => Ok(Body::MeanNode),
        10 => Ok(Body::TrueNode),
        11 => Ok(Body::Pluto),
        12 => Ok(Body::Chiron),
        // 13 = Ketu handled specially by callers
        _ => Err(format!(
            "Invalid body ID: {id}. Valid: 0=Sun, 1=Moon, 2=Mercury, 3=Venus, \
             4=Mars, 5=Jupiter, 6=Saturn, 7=Uranus, 8=Neptune, 9=MeanNode(Rahu), \
             10=TrueNode, 11=Pluto, 12=Chiron, 13=Ketu"
        )),
    }
}

/// Canonical ayanamsa ID mapping (shared across all bindings).
/// 0=Lahiri, 1=KP, 2=Raman, 3=FaganBradley, 4=TrueChitra, 5=TrueRevati,
/// 6=SuryaSiddhanta, 7=Yukteswar, 8=JNBhasin, 9=DeLuce, 10=Ushashashi,
/// 11=PushyaPaksha, 12=GalacticCenter, 13=LahiriICRC, 14=KPStraightLine,
/// 15=Hipparchos, 16=LahiriVP285
fn ayanamsa_from_id(id: u8) -> Result<Ayanamsa, String> {
    match id {
        0 => Ok(Ayanamsa::Lahiri),
        1 => Ok(Ayanamsa::KPKrishnamurti),
        2 => Ok(Ayanamsa::Raman),
        3 => Ok(Ayanamsa::FaganBradley),
        4 => Ok(Ayanamsa::TrueChitra),
        5 => Ok(Ayanamsa::TrueRevati),
        6 => Ok(Ayanamsa::SuryaSiddhanta),
        7 => Ok(Ayanamsa::YukteswarSriSS),
        8 => Ok(Ayanamsa::JNBhasin),
        9 => Ok(Ayanamsa::DeLuce),
        10 => Ok(Ayanamsa::Ushashashi),
        11 => Ok(Ayanamsa::PushyaPaksha),
        12 => Ok(Ayanamsa::GalacticCenter0Sag),
        13 => Ok(Ayanamsa::LahiriICRC),
        14 => Ok(Ayanamsa::KPStraightLine),
        15 => Ok(Ayanamsa::Hipparchos),
        16 => Ok(Ayanamsa::LahiriVP285),
        _ => Err(format!(
            "Invalid ayanamsa ID: {id}. Valid: 0=Lahiri, 1=KP, 2=Raman, \
             3=FaganBradley, 4=TrueChitra, 5=TrueRevati, 6=SuryaSiddhanta, \
             7=Yukteswar, 8=JNBhasin, 9=DeLuce, 10=Ushashashi, 11=PushyaPaksha, \
             12=GalacticCenter, 13=LahiriICRC, 14=KPStraightLine, 15=Hipparchos, \
             16=LahiriVP285"
        )),
    }
}

fn varga_from_id(id: u8) -> Result<VargaChart, String> {
    match id {
        1 => Ok(VargaChart::D1),
        2 => Ok(VargaChart::D2),
        3 => Ok(VargaChart::D3),
        4 => Ok(VargaChart::D4),
        7 => Ok(VargaChart::D7),
        9 => Ok(VargaChart::D9),
        10 => Ok(VargaChart::D10),
        12 => Ok(VargaChart::D12),
        16 => Ok(VargaChart::D16),
        20 => Ok(VargaChart::D20),
        24 => Ok(VargaChart::D24),
        27 => Ok(VargaChart::D27),
        30 => Ok(VargaChart::D30),
        40 => Ok(VargaChart::D40),
        45 => Ok(VargaChart::D45),
        60 => Ok(VargaChart::D60),
        _ => Err(format!(
            "Invalid varga: {id}. Valid: 1,2,3,4,7,9,10,12,16,20,24,27,30,40,45,60"
        )),
    }
}

/// Canonical house system ID mapping (shared across all bindings).
/// 0=WholeSign, 1=Equal, 2=Placidus, 3=Koch, 4=Porphyry, 5=Regiomontanus,
/// 6=Campanus, 7=Morinus, 8=Alcabitius, 9=Topocentric, 10=Sripati,
/// 11=Vehlow, 12=Meridian, 13=Krusinski
fn house_system_from_id(id: u8) -> Result<HouseSystem, String> {
    match id {
        0 => Ok(HouseSystem::WholeSign),
        1 => Ok(HouseSystem::Equal),
        2 => Ok(HouseSystem::Placidus),
        3 => Ok(HouseSystem::Koch),
        4 => Ok(HouseSystem::Porphyry),
        5 => Ok(HouseSystem::Regiomontanus),
        6 => Ok(HouseSystem::Campanus),
        7 => Ok(HouseSystem::Morinus),
        8 => Ok(HouseSystem::Alcabitius),
        9 => Ok(HouseSystem::Topocentric),
        10 => Ok(HouseSystem::Sripati),
        11 => Ok(HouseSystem::Vehlow),
        12 => Ok(HouseSystem::Meridian),
        13 => Ok(HouseSystem::KrusinskiPisa),
        _ => Err(format!(
            "Invalid house system ID: {id}. Valid: 0=WholeSign, 1=Equal, 2=Placidus, \
             3=Koch, 4=Porphyry, 5=Regiomontanus, 6=Campanus, 7=Morinus, \
             8=Alcabitius, 9=Topocentric, 10=Sripati, 11=Vehlow, 12=Meridian, \
             13=Krusinski"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wasm_tropical_sun() {
        let w = XalenWasm::new();
        let lon = w.tropical_longitude(2451545.0, 0).unwrap();
        assert!(lon >= 0.0 && lon < 360.0);
    }

    #[test]
    fn wasm_sidereal_sun() {
        let w = XalenWasm::new();
        let lon = w.sidereal_longitude(2451545.0, 0, 0).unwrap();
        assert!(lon >= 0.0 && lon < 360.0);
        let tropical = w.tropical_longitude(2451545.0, 0).unwrap();
        assert!(
            tropical > lon,
            "Tropical should be > sidereal (ayanamsa positive)"
        );
    }

    #[test]
    fn wasm_all_body_ids() {
        let w = XalenWasm::new();
        // 0=Sun through 12=Chiron
        for id in 0..=12u8 {
            let result = w.tropical_longitude(2451545.0, id);
            assert!(
                result.is_ok(),
                "Body {id} should compute: {:?}",
                result.err()
            );
            let lon = result.unwrap();
            assert!(
                lon >= 0.0 && lon < 360.0,
                "Body {id} lon out of range: {lon}"
            );
        }
    }

    #[test]
    fn wasm_ketu_body_id_13() {
        let w = XalenWasm::new();
        let rahu_lon = w.tropical_longitude(2451545.0, 9).unwrap(); // Rahu
        let ketu_lon = w.tropical_longitude(2451545.0, 13).unwrap(); // Ketu
        let expected = (rahu_lon + 180.0).rem_euclid(360.0);
        assert!(
            (ketu_lon - expected).abs() < 1e-10,
            "Ketu should be Rahu+180: expected {expected}, got {ketu_lon}"
        );
    }

    #[test]
    fn wasm_ketu_sidereal() {
        let w = XalenWasm::new();
        let rahu_lon = w.sidereal_longitude(2451545.0, 9, 0).unwrap();
        let ketu_lon = w.sidereal_longitude(2451545.0, 13, 0).unwrap();
        let expected = (rahu_lon + 180.0).rem_euclid(360.0);
        assert!(
            (ketu_lon - expected).abs() < 1e-10,
            "Ketu sidereal should be Rahu+180: expected {expected}, got {ketu_lon}"
        );
    }

    #[test]
    fn wasm_nakshatra() {
        let w = XalenWasm::new();
        let result = w.nakshatra(100.0);
        assert!(result.contains("Pada"), "Should contain Pada: '{result}'");
    }

    #[test]
    fn wasm_panchang() {
        let w = XalenWasm::new();
        let json = w.panchang_json(2451545.0, 0).unwrap();
        assert!(
            json.contains("tithi"),
            "Panchang should contain tithi: '{json}'"
        );
    }

    #[test]
    fn wasm_houses() {
        let w = XalenWasm::new();
        let json = w.houses_json(2451545.0, 18.52, 73.85, 0).unwrap();
        assert!(json.contains("cusps"), "Houses should contain cusps");
    }

    #[test]
    fn wasm_full_chart() {
        let w = XalenWasm::new();
        let json = w.full_chart_json(2451545.0, 18.52, 73.85, 0).unwrap();
        assert!(json.contains("planets"), "Chart should contain planets");
        assert!(json.contains("Sun"), "Chart should contain Sun");
        assert!(json.contains("Moon"), "Chart should contain Moon");
    }

    #[test]
    fn wasm_invalid_body() {
        let w = XalenWasm::new();
        assert!(w.tropical_longitude(2451545.0, 99).is_err());
    }

    #[test]
    fn wasm_invalid_ayanamsa() {
        let w = XalenWasm::new();
        assert!(w.sidereal_longitude(2451545.0, 0, 99).is_err());
        assert!(XalenWasm::ayanamsa_deg(2451545.0, 99).is_err());
    }

    #[test]
    fn wasm_invalid_house_system() {
        let w = XalenWasm::new();
        assert!(w.houses_json(2451545.0, 18.52, 73.85, 99).is_err());
    }

    #[test]
    fn wasm_all_vedic_grahas() {
        let w = XalenWasm::new();
        for id in 0..=12u8 {
            let result = w.sidereal_longitude(2451545.0, id, 0);
            assert!(
                result.is_ok(),
                "Body {id} should compute: {:?}",
                result.err()
            );
        }
    }

    #[test]
    fn wasm_julian_day() {
        let jd = XalenWasm::julian_day(2000, 1, 1, 12.0);
        assert!(
            (jd - 2451545.0).abs() < 0.01,
            "J2000 should be 2451545.0, got {jd}"
        );
    }

    #[test]
    fn wasm_ayanamsa() {
        let aya = XalenWasm::ayanamsa_deg(2451545.0, 0).unwrap();
        assert!(
            aya > 23.0 && aya < 25.0,
            "Lahiri at J2000 should be ~23.85, got {aya}"
        );
    }

    #[test]
    fn wasm_body_name() {
        assert_eq!(XalenWasm::body_name(0), "Sun");
        assert_eq!(XalenWasm::body_name(1), "Moon");
        assert!(XalenWasm::body_name(99).contains("Invalid"));
    }

    #[test]
    fn wasm_all_ayanamsas() {
        for id in 0..=16u8 {
            let aya = XalenWasm::ayanamsa_deg(2451545.0, id).unwrap();
            assert!(aya.is_finite(), "Ayanamsa {id} should be finite, got {aya}");
        }
    }

    #[test]
    fn wasm_all_house_systems() {
        let w = XalenWasm::new();
        for id in 0..=13u8 {
            let json = w.houses_json(2451545.0, 18.52, 73.85, id).unwrap();
            assert!(
                json.contains("cusps"),
                "House system {id} should produce cusps"
            );
        }
    }

    // ---- New function tests ----

    #[test]
    fn wasm_delta_t() {
        let dt = XalenWasm::delta_t_seconds(2451545.0);
        // Delta T at J2000 should be ~63.83 seconds
        assert!(
            dt > 60.0 && dt < 70.0,
            "Delta T at J2000 should be ~63.8s, got {dt}s"
        );
    }

    #[test]
    fn wasm_delta_t_historical() {
        let dt_modern = XalenWasm::delta_t_seconds(2451545.0);
        let dt_ancient = XalenWasm::delta_t_seconds(2000000.0);
        assert!(
            dt_ancient > dt_modern,
            "Ancient Delta T should be larger than modern"
        );
    }

    #[test]
    fn wasm_vimshottari_dasha() {
        let json = XalenWasm::vimshottari_dasha_json(100.0, 2451545.0);
        assert!(!json.is_empty(), "Dasha JSON should not be empty");
        assert!(
            json.contains("Ketu") || json.contains("Venus") || json.contains("Sun"),
            "Dasha JSON should contain dasha lord names"
        );
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 9, "Should have 9 Mahadasha periods");
        for period in &parsed {
            let subs = period["sub_periods"].as_array().unwrap();
            assert_eq!(
                subs.len(),
                9,
                "Each Mahadasha should have 9 Antardasha sub-periods"
            );
        }
    }

    #[test]
    fn wasm_vimshottari_dasha_covers_120_years() {
        let json = XalenWasm::vimshottari_dasha_json(0.0, 2451545.0);
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        let first_start = parsed[0]["start_jd"].as_f64().unwrap();
        let last_end = parsed[8]["end_jd"].as_f64().unwrap();
        let total_years = (last_end - first_start) / 365.25;
        assert!(
            (total_years - 120.0).abs() < 0.1,
            "Total dasha at boundary should be ~120 years, got {total_years}"
        );
    }

    #[test]
    fn wasm_compatibility_same_nakshatra() {
        let json = XalenWasm::compatibility(0, 0).unwrap();
        assert!(!json.is_empty(), "Compatibility JSON should not be empty");
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(
            parsed["total"].as_u64().unwrap() > 0,
            "Same nakshatra should score > 0"
        );
        assert!(
            parsed["total"].as_u64().unwrap() <= 36,
            "Score should be <= 36"
        );
        for koota in &[
            "varna",
            "vashya",
            "tara",
            "yoni",
            "graha_maitri",
            "gana",
            "bhakoot",
            "nadi",
        ] {
            assert!(parsed[koota].is_number(), "Missing koota: {koota}");
        }
    }

    #[test]
    fn wasm_compatibility_different_nakshatras() {
        let json = XalenWasm::compatibility(0, 1).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let total = parsed["total"].as_u64().unwrap();
        assert!(total <= 36, "Total should be <= 36, got {total}");
    }

    #[test]
    fn wasm_compatibility_invalid_nakshatra() {
        assert!(XalenWasm::compatibility(27, 0).is_err());
        assert!(XalenWasm::compatibility(0, 27).is_err());
        assert!(XalenWasm::compatibility(255, 255).is_err());
    }

    #[test]
    fn wasm_compatibility_all_pairs_valid_range() {
        for boy in 0..27u8 {
            for girl in 0..27u8 {
                let json = XalenWasm::compatibility(boy, girl).unwrap();
                let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
                let total = parsed["total"].as_u64().unwrap();
                assert!(total <= 36, "Boy={boy} Girl={girl}: total {total} > 36");
            }
        }
    }

    #[test]
    fn wasm_divisional_chart_d1() {
        let result = XalenWasm::divisional_chart(45.0, 1).unwrap();
        assert!(
            result.contains("Vrishabha"),
            "45 deg D1 should be Vrishabha, got '{result}'"
        );
    }

    #[test]
    fn wasm_divisional_chart_d9() {
        let result = XalenWasm::divisional_chart(0.0, 9).unwrap();
        assert!(
            result.contains("Mesha"),
            "0 deg D9 should be Mesha, got '{result}'"
        );
    }

    #[test]
    fn wasm_divisional_chart_invalid_varga() {
        assert!(XalenWasm::divisional_chart(100.0, 5).is_err());
        assert!(XalenWasm::divisional_chart(100.0, 0).is_err());
        assert!(XalenWasm::divisional_chart(100.0, 99).is_err());
    }

    #[test]
    fn wasm_divisional_chart_all_valid_vargas() {
        let valid_vargas: &[u8] = &[1, 2, 3, 4, 7, 9, 10, 12, 16, 20, 24, 27, 30, 40, 45, 60];
        for &varga in valid_vargas {
            let result = XalenWasm::divisional_chart(100.0, varga);
            assert!(
                result.is_ok(),
                "Varga D{varga} should succeed, got {:?}",
                result.err()
            );
            let name = result.unwrap();
            assert!(
                !name.is_empty(),
                "D{varga} should return a non-empty rashi name"
            );
        }
    }

    #[test]
    fn wasm_divisional_chart_normalized() {
        let r1 = XalenWasm::divisional_chart(-30.0, 1).unwrap();
        let r2 = XalenWasm::divisional_chart(330.0, 1).unwrap();
        assert_eq!(r1, r2, "-30 deg and 330 deg should give same D1 result");
    }
}
