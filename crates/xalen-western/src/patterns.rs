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

    // Grand Cross: four planets forming two oppositions, mutually squared.
    // A–C and B–D are oppositions (180°); the four "sides" A–B, B–C, C–D, D–A
    // are all squares (90°).  Iterate all 4-combinations and check the two
    // diagonals + four sides.  Members are returned in opposition-pair order
    // [A, C, B, D] so the two opposition axes are (members[0],members[1]) and
    // (members[2],members[3]).
    for a in 0..n {
        for b in (a + 1)..n {
            for c in (b + 1)..n {
                for d in (c + 1)..n {
                    let idx = [a, b, c, d];
                    if let Some(order) = grand_cross_order(positions_deg, idx, orb) {
                        patterns.push((AspectPattern::GrandCross, order));
                    }
                }
            }
        }
    }

    // Mystic Rectangle: two oppositions whose endpoints are joined by two
    // trines and two sextiles.  For bodies on opposition axes (p,q) and (r,s):
    // one of {p,q} trines one of {r,s} and sextiles the other, and the
    // remaining endpoint mirrors it.  Members returned as [p, q, r, s] (the two
    // opposition axes are (members[0],members[1]) and (members[2],members[3])).
    for a in 0..n {
        for b in (a + 1)..n {
            for c in (b + 1)..n {
                for d in (c + 1)..n {
                    let idx = [a, b, c, d];
                    if let Some(order) = mystic_rectangle_order(positions_deg, idx, orb) {
                        patterns.push((AspectPattern::MysticRectangle, order));
                    }
                }
            }
        }
    }

    // Kite: a Grand Trine (three planets 120° apart) plus a fourth body that
    // is in opposition to one trine member ("tail") and sextile to the other
    // two.  Members returned as [tail, apex, wing1, wing2] where `apex` is the
    // trine member opposed by the tail and wing1/wing2 are the sextile pair.
    for i in 0..n {
        for j in (i + 1)..n {
            for k in (j + 1)..n {
                // Must be a grand trine first.
                if !is_grand_trine(positions_deg, [i, j, k], orb) {
                    continue;
                }
                for tail in 0..n {
                    if tail == i || tail == j || tail == k {
                        continue;
                    }
                    if let Some(order) = kite_order(positions_deg, [i, j, k], tail, orb) {
                        patterns.push((AspectPattern::Kite, order));
                    }
                }
            }
        }
    }

    patterns
}

/// True when indices `[a,b,c]` form a Grand Trine (each pair ~120° apart).
fn is_grand_trine(pos: &[f64], idx: [usize; 3], orb: f64) -> bool {
    let [a, b, c] = idx;
    (angular_distance(pos[a], pos[b]) - 120.0).abs() < orb
        && (angular_distance(pos[b], pos[c]) - 120.0).abs() < orb
        && (angular_distance(pos[a], pos[c]) - 120.0).abs() < orb
}

/// Return true if the angular distance between two positions matches `target`
/// within `orb`.
fn is_aspect(pos: &[f64], i: usize, j: usize, target: f64, orb: f64) -> bool {
    (angular_distance(pos[i], pos[j]) - target).abs() < orb
}

/// If `[a,b,c,d]` form a Grand Cross, return the four indices ordered as two
/// opposition axes: `[A, C, B, D]` (axis 1 = A–C, axis 2 = B–D).
fn grand_cross_order(pos: &[f64], idx: [usize; 4], orb: f64) -> Option<Vec<usize>> {
    let [a, b, c, d] = idx;
    // Find a pairing of the 4 indices into two oppositions.
    // The three distinct pairings of 4 items into 2 pairs:
    //   (a,b)+(c,d), (a,c)+(b,d), (a,d)+(b,c)
    let pairings = [[(a, b), (c, d)], [(a, c), (b, d)], [(a, d), (b, c)]];
    for [(p1a, p1b), (p2a, p2b)] in pairings {
        let opp1 = is_aspect(pos, p1a, p1b, 180.0, orb);
        let opp2 = is_aspect(pos, p2a, p2b, 180.0, orb);
        if !(opp1 && opp2) {
            continue;
        }
        // All four cross-sides must be squares (90°): p1a–p2a, p1a–p2b,
        // p1b–p2a, p1b–p2b.
        let sides = is_aspect(pos, p1a, p2a, 90.0, orb)
            && is_aspect(pos, p1a, p2b, 90.0, orb)
            && is_aspect(pos, p1b, p2a, 90.0, orb)
            && is_aspect(pos, p1b, p2b, 90.0, orb);
        if sides {
            // [A, C, B, D] — opposition axes (0,1) and (2,3).
            return Some(vec![p1a, p1b, p2a, p2b]);
        }
    }
    None
}

/// If `[a,b,c,d]` form a Mystic Rectangle, return the four indices ordered as
/// two opposition axes: `[p, q, r, s]` (axis 1 = p–q, axis 2 = r–s).  The
/// connecting sides are two trines and two sextiles.
fn mystic_rectangle_order(pos: &[f64], idx: [usize; 4], orb: f64) -> Option<Vec<usize>> {
    let [a, b, c, d] = idx;
    let pairings = [[(a, b), (c, d)], [(a, c), (b, d)], [(a, d), (b, c)]];
    for [(p, q), (r, s)] in pairings {
        // Both diagonals must be oppositions.
        if !(is_aspect(pos, p, q, 180.0, orb) && is_aspect(pos, r, s, 180.0, orb)) {
            continue;
        }
        // The four sides connecting the two axes must be exactly two trines
        // and two sextiles, arranged so each endpoint has one of each.
        // Side set: p–r, p–s, q–r, q–s.  Valid rectangle:
        //   (p–r trine, p–s sextile, q–r sextile, q–s trine)  OR
        //   (p–r sextile, p–s trine, q–r trine, q–s sextile)
        let pr_t = is_aspect(pos, p, r, 120.0, orb);
        let ps_s = is_aspect(pos, p, s, 60.0, orb);
        let qr_s = is_aspect(pos, q, r, 60.0, orb);
        let qs_t = is_aspect(pos, q, s, 120.0, orb);

        let pr_s = is_aspect(pos, p, r, 60.0, orb);
        let ps_t = is_aspect(pos, p, s, 120.0, orb);
        let qr_t = is_aspect(pos, q, r, 120.0, orb);
        let qs_s = is_aspect(pos, q, s, 60.0, orb);

        if (pr_t && ps_s && qr_s && qs_t) || (pr_s && ps_t && qr_t && qs_s) {
            return Some(vec![p, q, r, s]);
        }
    }
    None
}

/// Given a Grand Trine `[i,j,k]` and a candidate `tail`, return the Kite
/// ordering `[tail, apex, wing1, wing2]` if `tail` is in opposition to exactly
/// one trine member (the apex) and sextile to the other two (the wings).
fn kite_order(pos: &[f64], trine: [usize; 3], tail: usize, orb: f64) -> Option<Vec<usize>> {
    let members = trine;
    // The apex is the trine member opposed by the tail.
    let apex = members
        .iter()
        .copied()
        .find(|&m| is_aspect(pos, tail, m, 180.0, orb))?;
    let wings: Vec<usize> = members.iter().copied().filter(|&m| m != apex).collect();
    if wings.len() != 2 {
        return None;
    }
    // The tail must sextile both wings.
    if is_aspect(pos, tail, wings[0], 60.0, orb) && is_aspect(pos, tail, wings[1], 60.0, orb) {
        Some(vec![tail, apex, wings[0], wings[1]])
    } else {
        None
    }
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

    // ── Grand Cross ──────────────────────────────────────────────────────

    #[test]
    fn grand_cross_detected() {
        // Two oppositions (0–180, 90–270), all four sides square.
        let positions = vec![0.0, 90.0, 180.0, 270.0];
        let patterns = detect_patterns(&positions, 6.0);
        let gc: Vec<_> = patterns
            .iter()
            .filter(|(p, _)| *p == AspectPattern::GrandCross)
            .collect();
        assert!(
            !gc.is_empty(),
            "Grand Cross must be detected, got {patterns:#?}"
        );
        // All four bodies participate.
        let mut members = gc[0].1.clone();
        members.sort();
        assert_eq!(members, vec![0, 1, 2, 3]);
        // Members are ordered as two opposition axes: (0,1) and (2,3).
        let m = &gc[0].1;
        assert!((angular_distance(positions[m[0]], positions[m[1]]) - 180.0).abs() < 6.0);
        assert!((angular_distance(positions[m[2]], positions[m[3]]) - 180.0).abs() < 6.0);
    }

    #[test]
    fn grand_cross_with_orb() {
        // Slightly off but within 6° orb.
        let positions = vec![1.0, 92.0, 179.0, 271.0];
        let patterns = detect_patterns(&positions, 6.0);
        assert!(
            patterns
                .iter()
                .any(|(p, _)| *p == AspectPattern::GrandCross)
        );
    }

    #[test]
    fn grand_cross_not_from_grand_trine() {
        // A grand trine alone (3 bodies) must NOT produce a Grand Cross.
        let positions = vec![10.0, 130.0, 250.0];
        let patterns = detect_patterns(&positions, 8.0);
        assert!(
            !patterns
                .iter()
                .any(|(p, _)| *p == AspectPattern::GrandCross)
        );
    }

    // ── Kite ─────────────────────────────────────────────────────────────

    #[test]
    fn kite_detected() {
        // Grand trine at 0,120,240; tail at 180 opposes apex 0 and sextiles
        // the two wings (120, 240).
        let positions = vec![0.0, 120.0, 240.0, 180.0];
        let patterns = detect_patterns(&positions, 6.0);
        let kites: Vec<_> = patterns
            .iter()
            .filter(|(p, _)| *p == AspectPattern::Kite)
            .collect();
        assert!(
            !kites.is_empty(),
            "Kite must be detected, got {patterns:#?}"
        );
        // members = [tail, apex, wing1, wing2]; tail opposes apex.
        let m = &kites[0].1;
        assert!(
            (angular_distance(positions[m[0]], positions[m[1]]) - 180.0).abs() < 6.0,
            "tail must oppose apex"
        );
        // tail sextiles both wings.
        assert!((angular_distance(positions[m[0]], positions[m[2]]) - 60.0).abs() < 6.0);
        assert!((angular_distance(positions[m[0]], positions[m[3]]) - 60.0).abs() < 6.0);
    }

    #[test]
    fn kite_requires_grand_trine() {
        // Two oppositions but NO 120° grand-trine spine → no kite. (0,30,180,210
        // has only 30/150/180° relations among its bodies — no trine triple.)
        let positions = vec![0.0, 30.0, 180.0, 210.0];
        let patterns = detect_patterns(&positions, 5.0);
        assert!(!patterns.iter().any(|(p, _)| *p == AspectPattern::Kite));
    }

    // ── Mystic Rectangle ──────────────────────────────────────────────────

    #[test]
    fn mystic_rectangle_detected() {
        // Two oppositions (0–180, 60–240) joined by two trines and two sextiles.
        let positions = vec![0.0, 60.0, 180.0, 240.0];
        let patterns = detect_patterns(&positions, 6.0);
        let mr: Vec<_> = patterns
            .iter()
            .filter(|(p, _)| *p == AspectPattern::MysticRectangle)
            .collect();
        assert!(
            !mr.is_empty(),
            "Mystic Rectangle must be detected, got {patterns:#?}"
        );
        // members ordered as two opposition axes (0,1) and (2,3).
        let m = &mr[0].1;
        assert!((angular_distance(positions[m[0]], positions[m[1]]) - 180.0).abs() < 6.0);
        assert!((angular_distance(positions[m[2]], positions[m[3]]) - 180.0).abs() < 6.0);
    }

    #[test]
    fn mystic_rectangle_not_grand_cross() {
        // The mystic rectangle (trine/sextile sides) must NOT register as a
        // Grand Cross (which needs square sides).
        let positions = vec![0.0, 60.0, 180.0, 240.0];
        let patterns = detect_patterns(&positions, 6.0);
        assert!(
            !patterns
                .iter()
                .any(|(p, _)| *p == AspectPattern::GrandCross),
            "Mystic Rectangle should not be a Grand Cross"
        );
    }

    #[test]
    fn grand_cross_not_mystic_rectangle() {
        // The Grand Cross (square sides) must NOT register as a Mystic
        // Rectangle (which needs trine/sextile sides).
        let positions = vec![0.0, 90.0, 180.0, 270.0];
        let patterns = detect_patterns(&positions, 6.0);
        assert!(
            !patterns
                .iter()
                .any(|(p, _)| *p == AspectPattern::MysticRectangle),
            "Grand Cross should not be a Mystic Rectangle"
        );
    }

    #[test]
    fn scattered_four_no_complex_patterns() {
        // No GrandCross / Kite / MysticRectangle from random spread.
        let positions = vec![5.0, 47.0, 158.0, 211.0];
        let patterns = detect_patterns(&positions, 5.0);
        assert!(!patterns.iter().any(|(p, _)| matches!(
            p,
            AspectPattern::GrandCross | AspectPattern::Kite | AspectPattern::MysticRectangle
        )));
    }
}
