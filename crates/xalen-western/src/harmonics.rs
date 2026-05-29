use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarmonicChart {
    pub harmonic: u32,
    pub positions: Vec<HarmonicPosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarmonicPosition {
    pub body: String,
    pub natal_longitude: f64,
    pub harmonic_longitude: f64,
}

pub fn harmonic_longitude(natal_lon: f64, harmonic: u32) -> f64 {
    (natal_lon * harmonic as f64).rem_euclid(360.0)
}

pub fn compute_harmonic_chart(positions: &[(&str, f64)], harmonic: u32) -> HarmonicChart {
    HarmonicChart {
        harmonic,
        positions: positions
            .iter()
            .map(|(name, lon)| HarmonicPosition {
                body: name.to_string(),
                natal_longitude: *lon,
                harmonic_longitude: harmonic_longitude(*lon, harmonic),
            })
            .collect(),
    }
}

pub fn harmonic_aspect(natal_lon_a: f64, natal_lon_b: f64, harmonic: u32, orb: f64) -> bool {
    let ha = harmonic_longitude(natal_lon_a, harmonic);
    let hb = harmonic_longitude(natal_lon_b, harmonic);
    let diff = (ha - hb).rem_euclid(360.0);
    let dist = if diff > 180.0 { 360.0 - diff } else { diff };
    dist <= orb
}

pub fn age_harmonic(natal_positions: &[(&str, f64)], age: f64) -> HarmonicChart {
    let h = age.ceil() as u32;
    compute_harmonic_chart(natal_positions, h.max(1))
}

pub fn common_harmonics() -> Vec<(u32, &'static str)> {
    vec![
        (1, "Natal (radical)"),
        (2, "Opposition / polarity"),
        (3, "Trine / creativity"),
        (4, "Square / challenge"),
        (5, "Quintile / talent"),
        (7, "Septile / fate / inspiration"),
        (8, "Semi-square / effort"),
        (9, "Novile / completion"),
        (12, "Semi-sextile / growth"),
        (16, "16th harmonic"),
        (32, "32nd harmonic"),
    ]
}

pub fn harmonic_midpoint(lon_a: f64, lon_b: f64) -> f64 {
    let sum = lon_a + lon_b;
    let mid = sum / 2.0;
    let diff = (lon_a - lon_b).abs();
    if diff > 180.0 {
        (mid + 180.0).rem_euclid(360.0)
    } else {
        mid.rem_euclid(360.0)
    }
}

pub fn midpoint_tree(positions: &[(&str, f64)]) -> Vec<(String, String, f64)> {
    let mut tree = Vec::new();
    for i in 0..positions.len() {
        for j in (i + 1)..positions.len() {
            let mp = harmonic_midpoint(positions[i].1, positions[j].1);
            tree.push((positions[i].0.to_string(), positions[j].0.to_string(), mp));
        }
    }
    tree
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harmonic_1_is_natal() {
        assert_eq!(harmonic_longitude(120.0, 1), 120.0);
    }

    #[test]
    fn harmonic_2_doubles() {
        assert_eq!(harmonic_longitude(100.0, 2), 200.0);
        assert_eq!(harmonic_longitude(200.0, 2), 40.0); // 400 % 360
    }

    #[test]
    fn harmonic_wraps() {
        let h = harmonic_longitude(350.0, 5);
        assert!((h - 310.0).abs() < 0.01, "350*5=1750, mod360=310, got {h}");
    }

    #[test]
    fn chart_has_all_bodies() {
        let positions = vec![("Sun", 100.0), ("Moon", 200.0), ("Mars", 300.0)];
        let chart = compute_harmonic_chart(&positions, 5);
        assert_eq!(chart.positions.len(), 3);
        assert_eq!(chart.harmonic, 5);
    }

    #[test]
    fn harmonic_aspect_conjunction() {
        assert!(harmonic_aspect(0.0, 120.0, 3, 5.0));
        assert!(!harmonic_aspect(0.0, 119.0, 3, 2.0));
    }

    #[test]
    fn age_harmonic_at_30() {
        let positions = vec![("Sun", 100.0)];
        let chart = age_harmonic(&positions, 30.0);
        assert_eq!(chart.harmonic, 30);
    }

    #[test]
    fn common_harmonics_count() {
        assert!(common_harmonics().len() >= 10);
    }

    #[test]
    fn midpoint_basic() {
        let mp = harmonic_midpoint(0.0, 90.0);
        assert!((mp - 45.0).abs() < 0.01);
    }

    #[test]
    fn midpoint_wrapping() {
        let mp = harmonic_midpoint(350.0, 10.0);
        assert!(
            (mp - 0.0).abs() < 0.5 || (mp - 360.0).abs() < 0.5,
            "Midpoint of 350° and 10° should be near 0°, got {mp}"
        );
    }

    #[test]
    fn midpoint_tree_n_choose_2() {
        let positions = vec![
            ("Sun", 0.0),
            ("Moon", 90.0),
            ("Mars", 180.0),
            ("Jupiter", 270.0),
        ];
        let tree = midpoint_tree(&positions);
        assert_eq!(tree.len(), 6); // 4C2 = 6
    }
}
