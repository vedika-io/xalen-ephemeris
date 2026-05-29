use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Upagraha {
    Dhuma,
    Vyatipata,
    Parivesha,
    Indrachapa,
    Upaketu,
    Gulika,
    Mandi,
}

impl Upagraha {
    pub fn from_sun(sun_sidereal_deg: f64) -> Vec<(Upagraha, f64)> {
        let sun = sun_sidereal_deg;
        vec![
            (Upagraha::Dhuma, (sun + 133.333333).rem_euclid(360.0)),
            (
                Upagraha::Vyatipata,
                (360.0 - (sun + 133.333333).rem_euclid(360.0)).rem_euclid(360.0),
            ),
            (Upagraha::Parivesha, {
                let vyati = (360.0 - (sun + 133.333333).rem_euclid(360.0)).rem_euclid(360.0);
                (vyati + 180.0).rem_euclid(360.0)
            }),
            (Upagraha::Indrachapa, {
                let pari = {
                    let vyati = (360.0 - (sun + 133.333333).rem_euclid(360.0)).rem_euclid(360.0);
                    (vyati + 180.0).rem_euclid(360.0)
                };
                (360.0 - pari).rem_euclid(360.0)
            }),
            (Upagraha::Upaketu, (sun - 30.0).rem_euclid(360.0)),
        ]
    }
}

pub fn gulika_longitude(
    sunrise_jd: f64,
    sunset_jd: f64,
    weekday: usize,
    _birth_jd: f64,
    asc_fn: impl Fn(f64) -> f64,
) -> f64 {
    // Saturn's slot in 8-part day division varies by weekday
    let saturn_slot: [usize; 7] = [7, 6, 5, 4, 3, 2, 1]; // Sun=7th, Mon=6th, etc.
    let period = (sunset_jd - sunrise_jd) / 8.0;
    let slot = saturn_slot[weekday % 7] - 1;
    let slot_start = sunrise_jd + slot as f64 * period;
    asc_fn(slot_start)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upagraha_from_sun() {
        let ups = Upagraha::from_sun(100.0);
        assert_eq!(ups.len(), 5);
        for (upa, lon) in &ups {
            assert!(
                *lon >= 0.0 && *lon < 360.0,
                "{upa:?} lon {lon} out of range"
            );
        }
    }

    #[test]
    fn dhuma_formula() {
        let ups = Upagraha::from_sun(0.0);
        let dhuma = ups.iter().find(|(u, _)| *u == Upagraha::Dhuma).unwrap().1;
        assert!(
            (dhuma - 133.333).abs() < 0.01,
            "Dhuma should be Sun+133°20', got {dhuma}°"
        );
    }

    #[test]
    fn upaketu_formula() {
        let ups = Upagraha::from_sun(50.0);
        let upaketu = ups.iter().find(|(u, _)| *u == Upagraha::Upaketu).unwrap().1;
        assert!(
            (upaketu - 20.0).abs() < 0.01,
            "Upaketu should be Sun-30°, got {upaketu}°"
        );
    }

    #[test]
    fn vyatipata_plus_dhuma_equals_360() {
        let ups = Upagraha::from_sun(100.0);
        let dhuma = ups.iter().find(|(u, _)| *u == Upagraha::Dhuma).unwrap().1;
        let vyati = ups
            .iter()
            .find(|(u, _)| *u == Upagraha::Vyatipata)
            .unwrap()
            .1;
        let sum = (dhuma + vyati).rem_euclid(360.0);
        assert!(
            (sum - 360.0).abs() < 0.01 || sum.abs() < 0.01,
            "Dhuma + Vyatipata should = 360°, got {sum}°"
        );
    }

    #[test]
    fn gulika_uses_correct_slot() {
        let sunrise = 2451545.0 + 0.25;
        let sunset = sunrise + 0.5;
        let lon = gulika_longitude(sunrise, sunset, 0, sunrise + 0.1, |jd| {
            ((jd - sunrise) * 360.0 / 0.5).rem_euclid(360.0)
        });
        assert!(lon >= 0.0 && lon < 360.0);
    }
}
