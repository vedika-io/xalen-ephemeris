//! South Indian box chart renderer.
//!
//! The South Indian chart uses a 4x4 grid where **signs are fixed in position**.
//! Pisces always occupies the top-left corner. The sign sequence runs clockwise
//! around the outer ring. Planets are placed in the cell matching their sign.

use crate::{ChartData, SIGN_ABBREV, svg_escape};
use std::fmt::Write;

const SIZE: f64 = 400.0;
const HALF: f64 = SIZE / 2.0;
const PAD: f64 = 10.0;
const CELL: f64 = (SIZE - 2.0 * PAD) / 4.0;

const BG_FILL: &str = "#FFFEF5";
const STROKE: &str = "#2C1810";
const SIGN_COLOR: &str = "#8B7355";
const ASC_COLOR: &str = "#C4783E";
const FAINT: &str = "#999";
const LABEL_COLOR: &str = "#666";

/// Fixed grid positions for each sign (0=Aries..11=Pisces).
/// Returns (col, row) in a 4x4 grid.
fn sign_grid_pos(sign_idx: usize) -> (usize, usize) {
    match sign_idx {
        0 => (1, 0),  // Aries
        1 => (2, 0),  // Taurus
        2 => (3, 0),  // Gemini
        3 => (3, 1),  // Cancer
        4 => (3, 2),  // Leo
        5 => (3, 3),  // Virgo
        6 => (2, 3),  // Libra
        7 => (1, 3),  // Scorpio
        8 => (0, 3),  // Sagittarius
        9 => (0, 2),  // Capricorn
        10 => (0, 1), // Aquarius
        11 => (0, 0), // Pisces
        _ => (0, 0),
    }
}

/// Determine which sign a planet is in, given ascendant sign and house number.
fn sign_index_for_house(asc_sign: usize, house: usize) -> usize {
    (asc_sign + house - 1) % 12
}

pub(crate) fn render(data: &ChartData) -> String {
    let mut svg = String::with_capacity(4096);

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

    // Draw outer cells of 4x4 grid (skip inner 2x2)
    for row in 0..4 {
        for col in 0..4 {
            if (row == 1 || row == 2) && (col == 1 || col == 2) {
                continue;
            }
            let x = PAD + col as f64 * CELL;
            let y = PAD + row as f64 * CELL;
            write!(svg,
                "<rect x=\"{x}\" y=\"{y}\" width=\"{c}\" height=\"{c}\" fill=\"none\" stroke=\"{s}\" stroke-width=\"1.5\"/>",
                c = CELL, s = STROKE,
            ).unwrap();
        }
    }

    // Outer border
    let full = 4.0 * CELL;
    write!(svg,
        "<rect x=\"{p}\" y=\"{p}\" width=\"{f}\" height=\"{f}\" fill=\"none\" stroke=\"{s}\" stroke-width=\"2\"/>",
        p = PAD, f = full, s = STROKE,
    ).unwrap();

    // Center rectangle (inner 2x2 merge)
    let ix = PAD + CELL;
    let iy = PAD + CELL;
    let iw = 2.0 * CELL;
    write!(svg,
        "<rect x=\"{ix}\" y=\"{iy}\" width=\"{iw}\" height=\"{iw}\" fill=\"none\" stroke=\"{s}\" stroke-width=\"1.5\"/>",
        s = STROKE,
    ).unwrap();

    // Build sign -> planet map
    let mut sign_planets: [Vec<&str>; 12] = Default::default();
    for pp in &data.planet_positions {
        let si = sign_index_for_house(data.ascendant_sign_index, pp.house);
        sign_planets[si].push(&pp.abbreviation);
    }

    let asc_sign = data.ascendant_sign_index;

    // Draw sign labels, ascendant marker, planets
    for sign_idx in 0..12usize {
        let (col, row) = sign_grid_pos(sign_idx);
        let x = PAD + col as f64 * CELL;
        let y = PAD + row as f64 * CELL;
        let cell_cx = x + CELL / 2.0;
        let cell_cy = y + CELL / 2.0;

        // Sign abbreviation (top-left of cell)
        let tx = x + 4.0;
        let ty = y + 13.0;
        let abbr = SIGN_ABBREV[sign_idx];
        write!(
            svg,
            "<text x=\"{tx}\" y=\"{ty}\" font-size=\"10\" fill=\"{c}\">{abbr}</text>",
            c = SIGN_COLOR,
        )
        .unwrap();

        // Ascendant marker: diagonal in top-left corner
        if sign_idx == asc_sign {
            let lx1 = x;
            let ly1 = y + 20.0;
            let lx2 = x + 20.0;
            let ly2 = y;
            write!(svg,
                "<line x1=\"{lx1}\" y1=\"{ly1}\" x2=\"{lx2}\" y2=\"{ly2}\" stroke=\"{c}\" stroke-width=\"2\"/>",
                c = ASC_COLOR,
            ).unwrap();
        }

        // Planet abbreviations
        let planets = &sign_planets[sign_idx];
        if !planets.is_empty() {
            let text = svg_escape(&planets.join(" "));
            let py = cell_cy + 5.0;
            write!(svg,
                "<text x=\"{cell_cx}\" y=\"{py}\" text-anchor=\"middle\" font-size=\"12\" font-weight=\"bold\" fill=\"{c}\">{text}</text>",
                c = STROKE,
            ).unwrap();
        }
    }

    // Center label
    let cy1 = HALF - 5.0;
    write!(svg,
        "<text x=\"{h}\" y=\"{cy1}\" text-anchor=\"middle\" font-size=\"11\" fill=\"{c}\">Rashi Chart</text>",
        h = HALF, c = LABEL_COLOR,
    ).unwrap();
    let cy2 = HALF + 10.0;
    write!(svg,
        "<text x=\"{h}\" y=\"{cy2}\" text-anchor=\"middle\" font-size=\"9\" fill=\"{c}\">Ayan: {ayan:.2}</text>",
        h = HALF, c = FAINT, ayan = data.ayanamsa_deg,
    ).unwrap();

    svg.push_str("</svg>");
    svg
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ChartData, PlanetPosition};

    #[test]
    fn grid_positions_unique() {
        let mut seen = std::collections::HashSet::new();
        for i in 0..12 {
            let pos = sign_grid_pos(i);
            assert!(
                seen.insert(pos),
                "duplicate grid pos for sign {i}: {:?}",
                pos
            );
        }
    }

    #[test]
    fn grid_positions_within_4x4() {
        for i in 0..12 {
            let (col, row) = sign_grid_pos(i);
            assert!(
                col < 4 && row < 4,
                "sign {i} at ({col},{row}) out of 4x4 grid"
            );
        }
    }

    #[test]
    fn pisces_top_left() {
        assert_eq!(sign_grid_pos(11), (0, 0), "Pisces should be at (0,0)");
    }

    #[test]
    fn renders_rashi_chart_label() {
        let chart = ChartData {
            planet_positions: vec![],
            house_cusps_deg: [0.0; 12],
            ascendant_sign_index: 0,
            ayanamsa_deg: 24.0,
        };
        let svg = render(&chart);
        assert!(svg.contains("Rashi Chart"));
    }

    #[test]
    fn ascendant_line_present() {
        let chart = ChartData {
            planet_positions: vec![PlanetPosition {
                abbreviation: "Su".into(),
                house: 1,
                longitude_deg: 15.0,
            }],
            house_cusps_deg: [0.0; 12],
            ascendant_sign_index: 3,
            ayanamsa_deg: 24.0,
        };
        let svg = render(&chart);
        assert!(svg.contains(ASC_COLOR), "ascendant marker color missing");
    }
}
