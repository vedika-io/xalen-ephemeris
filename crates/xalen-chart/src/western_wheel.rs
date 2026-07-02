//! Western wheel chart renderer.
//!
//! Produces a circular chart with concentric rings:
//! 1. Outer zodiac ring (12 sign segments with colored backgrounds)
//! 2. House cusp lines radiating from center
//! 3. Planet glyphs positioned at their ecliptic longitude

use crate::{ChartData, SIGN_ABBREV, safe_longitude, svg_escape};
use std::cmp::Ordering;
use std::fmt::Write;

const SIZE: f64 = 400.0;
const CENTER: f64 = SIZE / 2.0;
const OUTER_R: f64 = 180.0;
const ZODIAC_R: f64 = 160.0;
const INNER_R: f64 = 140.0;
const PLANET_R: f64 = 110.0;
const CUSP_INNER_R: f64 = 50.0;

const BG_FILL: &str = "#FFFEF5";
const STROKE: &str = "#2C1810";
const SIGN_LABEL_COLOR: &str = "#8B7355";
const PLANET_DOT: &str = "#C4783E";
const FAINT: &str = "#999";

// Element-based fill colors for zodiac ring segments
const FIRE_COLOR: &str = "#FFE4E1";
const EARTH_COLOR: &str = "#E8F5E9";
const AIR_COLOR: &str = "#E3F2FD";
const WATER_COLOR: &str = "#F3E5F5";

fn sign_color(sign_idx: usize) -> &'static str {
    match sign_idx % 4 {
        0 => FIRE_COLOR,
        1 => EARTH_COLOR,
        2 => AIR_COLOR,
        _ => WATER_COLOR,
    }
}

fn polar_to_xy(angle_deg: f64, radius: f64) -> (f64, f64) {
    let rad = angle_deg.to_radians();
    (CENTER + radius * rad.cos(), CENTER - radius * rad.sin())
}

/// Map an ecliptic longitude to the on-screen wheel angle.
///
/// The Ascendant (`asc_cusp` longitude) sits at screen-left (angle 180°) and
/// ecliptic longitude increases **counter-clockwise** — the standard Western
/// natal-wheel orientation, where the 2nd house / Asc+30° falls *below* the
/// Ascendant. With [`polar_to_xy`] (SVG y inverted) that means the screen angle
/// must increase with longitude: `180 + (lon − asc)`.
///
/// (Previously this was `180 + asc − lon`, which rendered the zodiac and houses
/// reversed — clockwise — placing the 2nd house above the Ascendant.)
fn lon_to_angle(lon_deg: f64, asc_cusp_deg: f64) -> f64 {
    180.0 - asc_cusp_deg + lon_deg
}

fn arc_segment(start_deg: f64, end_deg: f64, outer_r: f64, inner_r: f64, fill: &str) -> String {
    let (x1o, y1o) = polar_to_xy(start_deg, outer_r);
    let (x2o, y2o) = polar_to_xy(end_deg, outer_r);
    let (x1i, y1i) = polar_to_xy(end_deg, inner_r);
    let (x2i, y2i) = polar_to_xy(start_deg, inner_r);

    format!(
        "<path d=\"M {x1o:.1} {y1o:.1} A {outer_r} {outer_r} 0 0 0 {x2o:.1} {y2o:.1} L {x1i:.1} {y1i:.1} A {inner_r} {inner_r} 0 0 1 {x2i:.1} {y2i:.1} Z\" fill=\"{fill}\" stroke=\"{s}\" stroke-width=\"0.5\"/>",
        s = STROKE,
    )
}

pub(crate) fn render(data: &ChartData) -> String {
    let mut svg = String::with_capacity(8192);

    write!(svg,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {s} {s}\" width=\"{s}\" height=\"{s}\" font-family=\"sans-serif\">",
        s = SIZE,
    ).unwrap();

    // Background
    write!(
        svg,
        "<rect x=\"0\" y=\"0\" width=\"{s}\" height=\"{s}\" fill=\"{bg}\" stroke=\"none\"/>",
        s = SIZE,
        bg = BG_FILL,
    )
    .unwrap();

    // Circles
    for (r, w) in [(OUTER_R, "2"), (ZODIAC_R, "1"), (INNER_R, "1")] {
        write!(svg,
            "<circle cx=\"{c}\" cy=\"{c}\" r=\"{r}\" fill=\"none\" stroke=\"{s}\" stroke-width=\"{w}\"/>",
            c = CENTER, s = STROKE,
        ).unwrap();
    }

    // Center circle
    write!(svg,
        "<circle cx=\"{c}\" cy=\"{c}\" r=\"{r}\" fill=\"{bg}\" stroke=\"{s}\" stroke-width=\"0.5\"/>",
        c = CENTER, r = CUSP_INNER_R, bg = BG_FILL, s = STROKE,
    ).unwrap();

    // Zodiac segments
    let asc_cusp = safe_longitude(data.house_cusps_deg[0]);

    for i in 0..12usize {
        let sign_start = (i as f64) * 30.0;
        let offset_start = lon_to_angle(sign_start, asc_cusp);
        let offset_end = offset_start + 30.0;

        let color = sign_color(i);
        svg.push_str(&arc_segment(
            offset_start,
            offset_end,
            OUTER_R,
            ZODIAC_R,
            color,
        ));

        // Sign label at midpoint
        let mid_angle = offset_start + 15.0;
        let label_r = (OUTER_R + ZODIAC_R) / 2.0;
        let (lx, ly) = polar_to_xy(mid_angle, label_r);
        let abbr = SIGN_ABBREV[i];
        write!(svg,
            "<text x=\"{lx:.1}\" y=\"{ly:.1}\" text-anchor=\"middle\" dominant-baseline=\"central\" font-size=\"9\" fill=\"{c}\">{abbr}</text>",
            c = STROKE,
        ).unwrap();
    }

    // House cusp lines
    for i in 0..12usize {
        let cusp_deg = safe_longitude(data.house_cusps_deg[i]);
        let angle = lon_to_angle(cusp_deg, asc_cusp);
        let (x1, y1) = polar_to_xy(angle, CUSP_INNER_R);
        let (x2, y2) = polar_to_xy(angle, ZODIAC_R);

        let w = if i % 3 == 0 { "1.5" } else { "0.7" };
        write!(svg,
            "<line x1=\"{x1:.1}\" y1=\"{y1:.1}\" x2=\"{x2:.1}\" y2=\"{y2:.1}\" stroke=\"{s}\" stroke-width=\"{w}\"/>",
            s = STROKE,
        ).unwrap();

        // House number
        let next = safe_longitude(data.house_cusps_deg[(i + 1) % 12]);
        let mid_cusp = if next > cusp_deg {
            (cusp_deg + next) / 2.0
        } else {
            ((cusp_deg + next + 360.0) / 2.0) % 360.0
        };
        let num_angle = lon_to_angle(mid_cusp, asc_cusp);
        let (nx, ny) = polar_to_xy(num_angle, (INNER_R + CUSP_INNER_R) / 2.0);
        let num = i + 1;
        write!(svg,
            "<text x=\"{nx:.1}\" y=\"{ny:.1}\" text-anchor=\"middle\" dominant-baseline=\"central\" font-size=\"8\" fill=\"{c}\">{num}</text>",
            c = SIGN_LABEL_COLOR,
        ).unwrap();
    }

    // Planet positions with simple collision avoidance
    let mut planet_angles: Vec<(f64, String)> = data
        .planet_positions
        .iter()
        .map(|pp| {
            let lon = safe_longitude(pp.longitude_deg);
            let angle = lon_to_angle(lon, asc_cusp);
            (angle, svg_escape(&pp.abbreviation))
        })
        .collect();
    planet_angles.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));

    for i in 1..planet_angles.len() {
        let prev = planet_angles[i - 1].0;
        let curr = planet_angles[i].0;
        if (curr - prev).abs() < 10.0 {
            planet_angles[i].0 = prev + 10.0;
        }
    }

    for (angle, abbr) in &planet_angles {
        let (px, py) = polar_to_xy(*angle, PLANET_R);
        write!(svg,
            "<text x=\"{px:.1}\" y=\"{py:.1}\" text-anchor=\"middle\" dominant-baseline=\"central\" font-size=\"11\" font-weight=\"bold\" fill=\"{c}\">{abbr}</text>",
            c = STROKE,
        ).unwrap();

        let (dx, dy) = polar_to_xy(*angle, INNER_R - 3.0);
        write!(
            svg,
            "<circle cx=\"{dx:.1}\" cy=\"{dy:.1}\" r=\"2\" fill=\"{c}\"/>",
            c = PLANET_DOT,
        )
        .unwrap();
    }

    // Ayanamsa annotation in center
    let cy = CENTER + 5.0;
    write!(svg,
        "<text x=\"{c}\" y=\"{cy}\" text-anchor=\"middle\" font-size=\"8\" fill=\"{f}\">Ayan: {ayan:.2}</text>",
        c = CENTER, f = FAINT, ayan = safe_longitude(data.ayanamsa_deg),
    ).unwrap();

    svg.push_str("</svg>");
    svg
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ChartData;

    #[test]
    fn polar_to_xy_center_plus_radius() {
        let (x, y) = polar_to_xy(0.0, 100.0);
        assert!((x - (CENTER + 100.0)).abs() < 0.01);
        assert!((y - CENTER).abs() < 0.01);
    }

    #[test]
    fn polar_to_xy_90deg() {
        let (x, y) = polar_to_xy(90.0, 100.0);
        assert!((x - CENTER).abs() < 0.01);
        assert!((y - (CENTER - 100.0)).abs() < 0.01);
    }

    #[test]
    fn ascendant_renders_on_the_left() {
        // A point on the Ascendant longitude maps to screen angle 180° (left).
        let asc = 100.0;
        let (x, y) = polar_to_xy(lon_to_angle(asc, asc), 100.0);
        assert!(
            (x - (CENTER - 100.0)).abs() < 0.01,
            "Asc must be at screen-left"
        );
        assert!((y - CENTER).abs() < 0.01);
    }

    #[test]
    fn zodiac_advances_counter_clockwise() {
        // Standard Western wheel: longitude increases counter-clockwise, so the
        // point 30° past the Ascendant sits BELOW it (screen y increases), not
        // above. Guards against the previous reversed (clockwise) mapping.
        let asc = 0.0;
        let (_, y_asc) = polar_to_xy(lon_to_angle(asc, asc), 100.0);
        let (_, y_next) = polar_to_xy(lon_to_angle(asc + 30.0, asc), 100.0);
        assert!(
            y_next > y_asc,
            "Asc+30° (2nd house) must fall below the Ascendant, not above"
        );
    }

    #[test]
    fn sign_colors_cycle() {
        assert_eq!(sign_color(0), FIRE_COLOR);
        assert_eq!(sign_color(1), EARTH_COLOR);
        assert_eq!(sign_color(2), AIR_COLOR);
        assert_eq!(sign_color(3), WATER_COLOR);
        assert_eq!(sign_color(4), FIRE_COLOR);
    }

    #[test]
    fn renders_all_sign_abbreviations() {
        let chart = ChartData {
            planet_positions: vec![],
            house_cusps_deg: [
                0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0, 210.0, 240.0, 270.0, 300.0, 330.0,
            ],
            ascendant_sign_index: 0,
            ayanamsa_deg: 24.0,
        };
        let svg = render(&chart);
        for abbr in &SIGN_ABBREV {
            assert!(svg.contains(abbr), "missing sign {abbr}");
        }
    }

    #[test]
    fn renders_house_numbers() {
        let chart = ChartData {
            planet_positions: vec![],
            house_cusps_deg: [
                0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0, 210.0, 240.0, 270.0, 300.0, 330.0,
            ],
            ascendant_sign_index: 0,
            ayanamsa_deg: 24.0,
        };
        let svg = render(&chart);
        for i in 1..=12 {
            assert!(svg.contains(&format!(">{i}<")), "missing house number {i}");
        }
    }
}
