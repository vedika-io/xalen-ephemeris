use crate::aspects::angular_distance;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AspectPattern {
    GrandTrine,
    TSquare,
    GrandCross,
    Yod,
    Kite,
    Stellium,
    MysticRectangle,
}

pub fn detect_patterns(positions_deg: &[f64], orb: f64) -> Vec<(AspectPattern, Vec<usize>)> {
    let mut patterns = Vec::new();
    let n = positions_deg.len();

    // Grand Trine: 3 planets each ~120° apart
    for i in 0..n {
        for j in (i + 1)..n {
            for k in (j + 1)..n {
                let d_ij = angular_distance(positions_deg[i], positions_deg[j]);
                let d_jk = angular_distance(positions_deg[j], positions_deg[k]);
                let d_ik = angular_distance(positions_deg[i], positions_deg[k]);
                if (d_ij - 120.0).abs() < orb
                    && (d_jk - 120.0).abs() < orb
                    && (d_ik - 120.0).abs() < orb
                {
                    patterns.push((AspectPattern::GrandTrine, vec![i, j, k]));
                }
            }
        }
    }

    // T-Square: 2 in opposition, both square a 3rd
    for i in 0..n {
        for j in (i + 1)..n {
            let d_ij = angular_distance(positions_deg[i], positions_deg[j]);
            if (d_ij - 180.0).abs() < orb {
                for k in 0..n {
                    if k == i || k == j {
                        continue;
                    }
                    let d_ik = angular_distance(positions_deg[i], positions_deg[k]);
                    let d_jk = angular_distance(positions_deg[j], positions_deg[k]);
                    if (d_ik - 90.0).abs() < orb && (d_jk - 90.0).abs() < orb {
                        patterns.push((AspectPattern::TSquare, vec![i, j, k]));
                    }
                }
            }
        }
    }

    // Yod: 2 in sextile, both quincunx to apex
    for i in 0..n {
        for j in (i + 1)..n {
            let d_ij = angular_distance(positions_deg[i], positions_deg[j]);
            if (d_ij - 60.0).abs() < orb {
                for k in 0..n {
                    if k == i || k == j {
                        continue;
                    }
                    let d_ik = angular_distance(positions_deg[i], positions_deg[k]);
                    let d_jk = angular_distance(positions_deg[j], positions_deg[k]);
                    if (d_ik - 150.0).abs() < orb && (d_jk - 150.0).abs() < orb {
                        patterns.push((AspectPattern::Yod, vec![i, j, k]));
                    }
                }
            }
        }
    }

    // Stellium: 3+ planets within 30° arc
    for i in 0..n {
        let mut cluster = vec![i];
        for j in 0..n {
            if j == i {
                continue;
            }
            if angular_distance(positions_deg[i], positions_deg[j]) < 30.0 {
                cluster.push(j);
            }
        }
        if cluster.len() >= 3 {
            cluster.sort();
            cluster.dedup();
            let key = cluster.clone();
            if !patterns
                .iter()
                .any(|(p, v)| *p == AspectPattern::Stellium && *v == key)
            {
                patterns.push((AspectPattern::Stellium, key));
            }
        }
    }

    patterns
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grand_trine_detected() {
        let positions = vec![10.0, 130.0, 250.0]; // 120° apart
        let patterns = detect_patterns(&positions, 8.0);
        assert!(
            patterns
                .iter()
                .any(|(p, _)| *p == AspectPattern::GrandTrine)
        );
    }

    #[test]
    fn t_square_detected() {
        // Opposition at 10° and 190°, square focal at 100°
        let positions = vec![10.0, 190.0, 100.0];
        let patterns = detect_patterns(&positions, 8.0);
        assert!(patterns.iter().any(|(p, _)| *p == AspectPattern::TSquare));
    }

    #[test]
    fn yod_detected() {
        // Sextile at 10° and 70°, both quincunx to 220°
        let positions = vec![10.0, 70.0, 220.0];
        let patterns = detect_patterns(&positions, 5.0);
        assert!(
            patterns.iter().any(|(p, _)| *p == AspectPattern::Yod),
            "Yod should be detected: 10-70 (sextile 60°), 10-220 (quincunx ~150°), 70-220 (quincunx ~150°)"
        );
    }

    #[test]
    fn stellium_detected() {
        let positions = vec![100.0, 105.0, 110.0, 115.0, 250.0];
        let patterns = detect_patterns(&positions, 8.0);
        assert!(patterns.iter().any(|(p, _)| *p == AspectPattern::Stellium));
    }

    #[test]
    fn no_patterns_when_scattered() {
        let positions = vec![10.0, 80.0, 160.0, 230.0]; // no clean aspects
        let patterns = detect_patterns(&positions, 5.0);
        let major = patterns
            .iter()
            .filter(|(p, _)| *p != AspectPattern::Stellium)
            .count();
        assert_eq!(
            major, 0,
            "Scattered positions shouldn't form major patterns"
        );
    }
}
