use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LunarPhase {
    NewMoon,
    WaxingCrescent,
    FirstQuarter,
    WaxingGibbous,
    FullMoon,
    WaningGibbous,
    LastQuarter,
    WaningCrescent,
}

impl LunarPhase {
    pub fn from_elongation_deg(elongation: f64) -> Self {
        let e = elongation.rem_euclid(360.0);
        match e as u32 {
            0..=22 => LunarPhase::NewMoon,
            23..=67 => LunarPhase::WaxingCrescent,
            68..=112 => LunarPhase::FirstQuarter,
            113..=157 => LunarPhase::WaxingGibbous,
            158..=202 => LunarPhase::FullMoon,
            203..=247 => LunarPhase::WaningGibbous,
            248..=292 => LunarPhase::LastQuarter,
            _ => LunarPhase::WaningCrescent,
        }
    }
}

pub fn lunar_illumination(elongation_deg: f64) -> f64 {
    let e_rad = elongation_deg.to_radians();
    (1.0 - e_rad.cos()) / 2.0
}

pub fn moon_elongation(sun_lon_deg: f64, moon_lon_deg: f64) -> f64 {
    (moon_lon_deg - sun_lon_deg).rem_euclid(360.0)
}

pub fn is_combust(planet_lon_deg: f64, sun_lon_deg: f64, orb_deg: f64) -> bool {
    let diff = (planet_lon_deg - sun_lon_deg).rem_euclid(360.0);
    let dist = if diff > 180.0 { 360.0 - diff } else { diff };
    dist < orb_deg
}

pub fn is_cazimi(planet_lon_deg: f64, sun_lon_deg: f64) -> bool {
    is_combust(planet_lon_deg, sun_lon_deg, 17.0 / 60.0) // 0°17'
}

pub fn combustion_status(planet: &str, planet_lon: f64, sun_lon: f64) -> &'static str {
    let orb = match planet {
        "Moon" => 12.0,
        "Mercury" => 14.0,
        "Venus" => 10.0,
        "Mars" => 17.0,
        "Jupiter" => 11.0,
        "Saturn" => 15.0,
        _ => 17.0,
    };

    if is_cazimi(planet_lon, sun_lon) {
        "Cazimi"
    } else if is_combust(planet_lon, sun_lon, orb) {
        "Combust"
    } else if is_combust(planet_lon, sun_lon, 17.0) {
        "Under Sunbeams"
    } else {
        "Free"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_moon_phase() {
        assert_eq!(LunarPhase::from_elongation_deg(0.0), LunarPhase::NewMoon);
        assert_eq!(LunarPhase::from_elongation_deg(5.0), LunarPhase::NewMoon);
    }

    #[test]
    fn full_moon_phase() {
        assert_eq!(LunarPhase::from_elongation_deg(180.0), LunarPhase::FullMoon);
    }

    #[test]
    fn illumination_new_moon() {
        let ill = lunar_illumination(0.0);
        assert!(ill < 0.01, "New moon illumination should be ~0, got {ill}");
    }

    #[test]
    fn illumination_full_moon() {
        let ill = lunar_illumination(180.0);
        assert!(
            (ill - 1.0).abs() < 0.01,
            "Full moon illumination should be ~1, got {ill}"
        );
    }

    #[test]
    fn cazimi_detection() {
        assert!(is_cazimi(100.0, 100.1));
        assert!(!is_cazimi(100.0, 101.0));
    }

    #[test]
    fn combustion_mars() {
        assert_eq!(combustion_status("Mars", 100.0, 110.0), "Combust");
        assert_eq!(combustion_status("Mars", 100.0, 100.2), "Cazimi");
        assert_eq!(combustion_status("Mars", 100.0, 150.0), "Free");
    }
}
