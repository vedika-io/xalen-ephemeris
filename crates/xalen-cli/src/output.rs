//! Table and JSON printing shared by every command.

use std::io::{self, Write};

use serde_json::Value;

/// What a command produces: the same facts as JSON and as readable text.
pub struct Report {
    /// Machine-readable form, printed with `--json`.
    pub json: Value,
    /// Human-readable form, printed by default.
    pub text: String,
}

/// Format a number with a fixed count of decimals.
pub fn num(value: f64, places: usize) -> String {
    format!("{value:.places$}")
}

fn push_row<'a>(out: &mut String, cells: impl Iterator<Item = &'a str>, widths: &[usize]) {
    let mut line = String::new();
    for (i, cell) in cells.enumerate() {
        if i > 0 {
            line.push_str("  ");
        }
        line.push_str(cell);
        let width = widths.get(i).copied().unwrap_or(0);
        let pad = width.saturating_sub(cell.chars().count());
        line.push_str(&" ".repeat(pad));
    }
    out.push_str(line.trim_end());
    out.push('\n');
}

/// Render a left-aligned text table with a header underline.
pub fn table(headers: &[&str], rows: &[Vec<String>]) -> String {
    let mut widths: Vec<usize> = headers.iter().map(|h| h.chars().count()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if let Some(width) = widths.get_mut(i) {
                *width = (*width).max(cell.chars().count());
            }
        }
    }
    let mut out = String::new();
    push_row(&mut out, headers.iter().copied(), &widths);
    let rule: Vec<String> = widths.iter().map(|w| "-".repeat(*w)).collect();
    push_row(&mut out, rule.iter().map(String::as_str), &widths);
    for row in rows {
        push_row(&mut out, row.iter().map(String::as_str), &widths);
    }
    out
}

/// Print a report to stdout, as JSON when `json` is set.
pub fn emit(report: &Report, json: bool) -> io::Result<()> {
    let mut out = io::stdout().lock();
    if json {
        let text = serde_json::to_string_pretty(&report.json).map_err(io::Error::other)?;
        writeln!(out, "{text}")?;
    } else {
        write!(out, "{}", report.text)?;
    }
    out.flush()
}
