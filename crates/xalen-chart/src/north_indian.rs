//! North Indian diamond chart renderer.
//!
//! The classic North Indian chart is a diamond rotated 45 degrees inside a square.
//! House 1 (Ascendant) is always the top-center diamond. Houses are numbered
//! counter-clockwise.

use crate::{ChartData, planets_in_house, sign_for_house, svg_escape};
use std::fmt::Write;

const SIZE: f64 = 400.0;
const HALF: f64 = SIZE / 2.0;
const PAD: f64 = 10.0;
const INNER: f64 = (SIZE - 2.0 * PAD) / 2.0;

// Colors as constants to avoid # in format strings
const BG_FILL: &str = "#FFFEF5";
const STROKE: &str = "#2C1810";
const SIGN_COLOR: &str = "#8B7355";
const ASC_COLOR: &str = "#C4783E";
const FAINT: &str = "#999";

/// Center coordinates for planet text in each house (1-indexed).
fn house_center(house: usize) -> (f64, f64) {
    let cx = HALF;
    let cy = HALF;
    let q = INNER * 0.45;
    match house {
        1 => (cx, cy - q * 1.55),
        12 => (cx + q, cy - q * 1.1),
        11 => (cx + q * 1.55, cy - q * 0.3),
        10 => (cx + q * 1.55, cy + q * 0.3),
        9 => (cx + q, cy + q * 1.1),
        8 => (cx, cy + q * 1.55),
        7 => (cx, cy + q * 1.55 + q * 0.55),
        6 => (cx - q, cy + q * 1.1),
        5 => (cx - q * 1.55, cy + q * 0.3),
        4 => (cx - q * 1.55, cy - q * 0.3),
        3 => (cx - q, cy - q * 1.1),
        2 => (cx - q, cy - q * 0.15),
        _ => (cx, cy),
    }
}

pub(crate) fn render(data: &ChartData) -> String {
    let mut svg = String::with_capacity(4096);

    // SVG header
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

    let l = PAD;
    let r = SIZE - PAD;
    let t = PAD;
    let b = SIZE - PAD;
    let cx = HALF;
    let cy = HALF;
    let w = r - l;
    let h = b - t;

    // Outer square
    write!(svg,
        "<rect x=\"{l}\" y=\"{t}\" width=\"{w}\" height=\"{h}\" fill=\"none\" stroke=\"{c}\" stroke-width=\"2\"/>",
        c = STROKE,
    ).unwrap();

    // Inner diamond lines
    let diamond_lines = [
        (cx, t, r, cy), // top to right
        (r, cy, cx, b), // right to bottom
        (cx, b, l, cy), // bottom to left
        (l, cy, cx, t), // left to top
    ];
    for (x1, y1, x2, y2) in diamond_lines {
        write!(svg,
            "<line x1=\"{x1}\" y1=\"{y1}\" x2=\"{x2}\" y2=\"{y2}\" stroke=\"{c}\" stroke-width=\"1.5\"/>",
            c = STROKE,
        ).unwrap();
    }

    // Horizontal and vertical bisectors
    write!(
        svg,
        "<line x1=\"{l}\" y1=\"{cy}\" x2=\"{r}\" y2=\"{cy}\" stroke=\"{c}\" stroke-width=\"1\"/>",
        c = STROKE,
    )
    .unwrap();
    write!(
        svg,
        "<line x1=\"{cx}\" y1=\"{t}\" x2=\"{cx}\" y2=\"{b}\" stroke=\"{c}\" stroke-width=\"1\"/>",
        c = STROKE,
    )
    .unwrap();

    // House sign labels + planets
    for house in 1..=12 {
        let (hx, hy) = house_center(house);
        let sign = sign_for_house(data.ascendant_sign_index, house);
        let planets = planets_in_house(data, house);

        let sy = hy - 12.0;
        write!(svg,
            "<text x=\"{hx}\" y=\"{sy}\" text-anchor=\"middle\" font-size=\"10\" fill=\"{c}\">{sign}</text>",
            c = SIGN_COLOR,
        ).unwrap();

        if !planets.is_empty() {
            let planet_str = svg_escape(&planets.join(" "));
            let py = hy + 4.0;
            write!(svg,
                "<text x=\"{hx}\" y=\"{py}\" text-anchor=\"middle\" font-size=\"12\" font-weight=\"bold\" fill=\"{c}\">{planet_str}</text>",
                c = STROKE,
            ).unwrap();
        }
    }

    // Ascendant marker
    let ay = t + 15.0;
    write!(svg,
        "<text x=\"{cx}\" y=\"{ay}\" text-anchor=\"middle\" font-size=\"9\" fill=\"{c}\" font-style=\"italic\">Asc</text>",
        c = ASC_COLOR,
    ).unwrap();

    // Ayanamsa annotation
    let ax = r - 2.0;
    let aby = b - 3.0;
    write!(svg,
        "<text x=\"{ax}\" y=\"{aby}\" text-anchor=\"end\" font-size=\"8\" fill=\"{c}\">Ayan: {ayan:.2}</text>",
        c = FAINT, ayan = data.ayanamsa_deg,
    ).unwrap();

    svg.push_str("</svg>");
    svg
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ChartData;

    #[test]
    fn house_centers_within_bounds() {
        for h in 1..=12 {
            let (x, y) = house_center(h);
            assert!(x > 0.0 && x < SIZE, "house {h} x={x} out of bounds");
            assert!(y > 0.0 && y < SIZE, "house {h} y={y} out of bounds");
        }
    }

    #[test]
    fn svg_contains_all_12_sign_labels() {
        let chart = ChartData {
            planet_positions: vec![],
            house_cusps_deg: [0.0; 12],
            ascendant_sign_index: 0,
            ayanamsa_deg: 24.0,
        };
        let svg = render(&chart);
        for abbr in &[
            "Ar", "Ta", "Ge", "Cn", "Le", "Vi", "Li", "Sc", "Sg", "Cp", "Aq", "Pi",
        ] {
            assert!(svg.contains(abbr), "missing sign {abbr}");
        }
    }

    #[test]
    fn ascendant_marker_present() {
        let chart = ChartData {
            planet_positions: vec![],
            house_cusps_deg: [0.0; 12],
            ascendant_sign_index: 0,
            ayanamsa_deg: 24.0,
        };
        let svg = render(&chart);
        assert!(svg.contains("Asc"));
    }
}
