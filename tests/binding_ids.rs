//! Drift test for `docs/BINDING_IDS.md`.
//!
//! The numeric IDs the C, WebAssembly, Node and Python bindings accept are the
//! `*_from_id` match tables in each binding crate. The Swiss Ephemeris numbers
//! come from `xalen_ephem::compat` (the `SE_*` constants and the planet and
//! house-letter mappings), from `Ayanamsa::from_swiss_ephem_id`, and from the
//! constant tables the `xalen.swe` Python module registers. This test reads
//! those sources, renders one Markdown table per ID family and compares each
//! with the matching generated block in the committed document. It fails with
//! a line diff when they differ.
//!
//! Regenerate the document with `scripts/regen-binding-ids.sh`, which runs this
//! test with `XALEN_REGEN_BINDING_IDS=1` so it rewrites the blocks in place.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use xalen_ayanamsa::Ayanamsa;
use xalen_ephem::compat;

const FFI: &str = include_str!("../crates/xalen-ffi/src/lib.rs");
const WASM: &str = include_str!("../crates/xalen-wasm/src/lib.rs");
const NODE: &str = include_str!("../crates/xalen-node/src/lib.rs");
const PYTHON: &str = include_str!("../crates/xalen-python/src/lib.rs");
const SWE_SHIM: &str = include_str!("../crates/xalen-python/src/swe_compat.rs");
const COMPAT: &str = include_str!("../crates/xalen-ephem/src/compat.rs");

/// Set this variable to rewrite the generated blocks instead of failing.
const REGEN_ENV: &str = "XALEN_REGEN_BINDING_IDS";
const DOC: &str = "docs/BINDING_IDS.md";
const J2000: f64 = 2_451_545.0;

/// The bindings with their own numeric IDs, in the column order of the tables.
const BINDINGS: [(&str, &str); 4] = [
    ("C (FFI)", FFI),
    ("WASM", WASM),
    ("Node", NODE),
    ("Python", PYTHON),
];

// ---------------------------------------------------------------------------
// Source readers
// ---------------------------------------------------------------------------

/// Text of the top-level function `name`, from its signature to the closing
/// brace in column 0.
fn fn_body<'a>(src: &'a str, name: &str) -> &'a str {
    let sig = format!("fn {name}(");
    let Some(start) = src.find(&sig) else {
        panic!("function `{name}` not found");
    };
    let rest = &src[start..];
    let Some(end) = rest.find("\n}") else {
        panic!("end of function `{name}` not found");
    };
    &rest[..end]
}

/// Leading identifier characters of `text`.
fn ident(text: &str) -> String {
    text.chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect()
}

/// `(pattern, variant)` for every arm `PATTERN => Wrapper(Enum::Variant)` of
/// the function `name`, skipping comments and the catch-all arm.
fn match_arms(src: &str, name: &str) -> Vec<(String, String)> {
    let mut arms = Vec::new();
    for line in fn_body(src, name).lines() {
        let line = line.trim();
        if line.starts_with("//") {
            continue;
        }
        let Some((pattern, target)) = line.split_once("=>") else {
            continue;
        };
        let pattern = pattern.trim();
        if pattern == "_" {
            continue;
        }
        let Some((_, variant)) = target.rsplit_once("::") else {
            continue;
        };
        arms.push((pattern.to_string(), ident(variant)));
    }
    assert!(!arms.is_empty(), "no match arms found in `{name}`");
    arms
}

/// ID to variant name for an integer match table such as `body_from_id`.
fn id_arms(src: &str, name: &str) -> BTreeMap<u32, String> {
    let mut map = BTreeMap::new();
    for (pattern, variant) in match_arms(src, name) {
        let Ok(id) = pattern.parse::<u32>() else {
            panic!("`{name}`: pattern `{pattern}` is not an integer");
        };
        if map.insert(id, variant).is_some() {
            panic!("`{name}`: ID {id} appears twice");
        }
    }
    map
}

/// Value of `pub const NAME: i32 = EXPR;` in `compat.rs`, where EXPR is an
/// integer or a product of integers such as `64 * 1024`.
fn compat_const(name: &str) -> i32 {
    let decl = format!("pub const {name}: i32 = ");
    let start = match COMPAT.find(&decl) {
        Some(pos) => pos + decl.len(),
        None => panic!("constant `{name}` not found in compat.rs"),
    };
    let len = COMPAT[start..].find(';').expect("`;` after the constant");
    COMPAT[start..start + len]
        .split('*')
        .map(|factor| factor.trim().parse::<i32>().expect("integer factor"))
        .product()
}

/// `("NAME", compat::CONST)` on one line, as in the `xalen.swe` tables.
fn quoted_pair(line: &str) -> Option<(String, String)> {
    let (_, rest) = line.split_once('"')?;
    let (name, rest) = rest.split_once('"')?;
    let (_, konst) = rest.split_once("compat::")?;
    Some((name.to_string(), ident(konst)))
}

/// `(python_name, compat_const)` pairs of the `let {list}: &[(&str, i32)]`
/// table that `swe_compat.rs` registers on the `xalen.swe` module.
fn shim_list(list: &str) -> Vec<(String, String)> {
    let head = format!("let {list}: &[(&str, i32)] = &[");
    let Some(start) = SWE_SHIM.find(&head) else {
        panic!("`{head}` not found in swe_compat.rs");
    };
    let body = &SWE_SHIM[start + head.len()..];
    let body = &body[..body.find("];").expect("end of the constant table")];
    body.lines().filter_map(quoted_pair).collect()
}

/// Whether a binding derives Ketu (body ID 13) as the mean node plus 180°.
fn accepts_ketu(src: &str) -> bool {
    src.contains("body_id == 13") || src.contains("body == 13")
}

/// Every ID accepted by any binding's `fn_name` with its variant name, and the
/// set of IDs each binding accepts. A binding without the function accepts
/// none. Two bindings mapping one ID to different variants is an error.
fn binding_ids(fn_name: &str) -> (BTreeMap<u32, String>, Vec<BTreeSet<u32>>) {
    let mut merged: BTreeMap<u32, String> = BTreeMap::new();
    let mut accepted = Vec::new();
    for (label, src) in BINDINGS {
        let mut ids = BTreeSet::new();
        if src.contains(&format!("fn {fn_name}(")) {
            for (id, variant) in id_arms(src, fn_name) {
                if let Some(seen) = merged.get(&id) {
                    assert!(
                        *seen == variant,
                        "{label}: {fn_name}({id}) is {variant}, elsewhere {seen}"
                    );
                }
                merged.insert(id, variant);
                ids.insert(id);
            }
        }
        accepted.push(ids);
    }
    (merged, accepted)
}

// ---------------------------------------------------------------------------
// Tables
// ---------------------------------------------------------------------------

fn yes(flag: bool) -> String {
    let cell = if flag { "yes" } else { "-" };
    cell.to_string()
}

fn code(text: &str) -> String {
    format!("`{text}`")
}

fn table(header: &[&str], rows: &[Vec<String>]) -> String {
    let mut out = format!("| {} |\n", header.join(" | "));
    out.push_str(&format!("|{}\n", "---|".repeat(header.len())));
    for row in rows {
        out.push_str(&format!("| {} |\n", row.join(" | ")));
    }
    out
}

/// The `yes`/`-` cells of one ID across the bindings.
fn accept_cells(accepted: &[BTreeSet<u32>], id: u32) -> Vec<String> {
    accepted.iter().map(|ids| yes(ids.contains(&id))).collect()
}

fn binding_header<'a>(leading: &[&'a str]) -> Vec<&'a str> {
    let mut header = leading.to_vec();
    header.extend(BINDINGS.iter().map(|(label, _)| *label));
    header
}

fn bodies_table() -> String {
    let (ids, accepted) = binding_ids("body_from_id");
    let ketu: Vec<bool> = BINDINGS.iter().map(|(_, src)| accepts_ketu(src)).collect();
    let exported: BTreeSet<String> = shim_list("planets").into_iter().map(|p| p.1).collect();

    // Body variant -> (Swiss constant, Swiss number), from compat's mapping.
    let mut swiss: BTreeMap<String, (String, i32)> = BTreeMap::new();
    for (konst, variant) in match_arms(COMPAT, "se_planet_to_body") {
        let number = compat_const(&konst);
        assert!(
            compat::swe_calc_ut(J2000, number, 0).is_ok(),
            "compat::swe_calc_ut rejects {konst} = {number}"
        );
        swiss.insert(variant, (konst, number));
    }
    let swiss_cells = |variant: &str| {
        let Some((konst, number)) = swiss.get(variant) else {
            return vec!["-".to_string(), "-".to_string()];
        };
        let mut constant = code(konst);
        if !exported.contains(konst) {
            constant.push_str(" (not in `xalen.swe`)");
        }
        vec![number.to_string(), constant]
    };

    let mut rows = Vec::new();
    for (&id, variant) in &ids {
        let mut row = vec![id.to_string(), code(variant)];
        row.extend(swiss_cells(variant.as_str()));
        row.extend(accept_cells(&accepted, id));
        rows.push(row);
    }
    if ketu.iter().any(|&k| k) {
        assert!(!ids.contains_key(&13), "body ID 13 is both Ketu and a Body");
        let mut row = vec!["13".to_string(), "Ketu (`MeanNode` + 180°)".to_string()];
        row.extend(["-".to_string(), "-".to_string()]);
        row.extend(ketu.iter().map(|&k| yes(k)));
        rows.push(row);
    }
    let mut swiss_only: Vec<(&String, &(String, i32))> = swiss
        .iter()
        .filter(|(variant, _)| !ids.values().any(|v| v == *variant))
        .collect();
    swiss_only.sort_by_key(|(_, (_, number))| *number);
    for (variant, _) in swiss_only {
        let mut row = vec!["-".to_string(), code(variant)];
        row.extend(swiss_cells(variant.as_str()));
        row.extend(BINDINGS.iter().map(|_| "-".to_string()));
        rows.push(row);
    }

    let header = binding_header(&["ID", "`Body`", "Swiss number", "Swiss constant"]);
    table(&header, &rows)
}

/// Swiss `SE_SIDM_*` number to `Ayanamsa` variant name, for every number
/// `Ayanamsa::from_swiss_ephem_id` accepts.
fn swiss_ayanamsas() -> BTreeMap<u32, String> {
    let mut map = BTreeMap::new();
    for number in 0..=255 {
        let Some(aya) = Ayanamsa::from_swiss_ephem_id(number) else {
            continue;
        };
        assert_eq!(
            aya.swiss_ephem_id(),
            Some(number),
            "{aya:?} does not round-trip"
        );
        assert!(
            compat::swe_get_ayanamsa_ut_ex(J2000, number as i32).is_ok(),
            "compat::swe_get_ayanamsa_ut_ex rejects SE_SIDM {number}"
        );
        map.insert(number, format!("{aya:?}"));
    }
    map
}

fn ayanamsas_table() -> String {
    let (ids, accepted) = binding_ids("ayanamsa_from_id");
    let swiss: BTreeMap<String, u32> = swiss_ayanamsas()
        .into_iter()
        .map(|(number, variant)| (variant, number))
        .collect();
    let mut rows = Vec::new();
    for (&id, variant) in &ids {
        let number = swiss.get(variant).map_or("-".to_string(), u32::to_string);
        let mut row = vec![id.to_string(), code(variant), number];
        row.extend(accept_cells(&accepted, id));
        rows.push(row);
    }
    let header = binding_header(&["ID", "`Ayanamsa`", "Swiss `SE_SIDM` number"]);
    table(&header, &rows)
}

fn swiss_ayanamsas_table() -> String {
    let (ids, _) = binding_ids("ayanamsa_from_id");
    let mut constants: BTreeMap<i32, String> = BTreeMap::new();
    for (name, konst) in shim_list("sidm") {
        let names = format!("{} / {}", code(&name), code(&format!("SE_{name}")));
        constants.insert(compat_const(&konst), names);
    }
    let mut rows = Vec::new();
    for (number, variant) in swiss_ayanamsas() {
        let constant = constants
            .remove(&(number as i32))
            .unwrap_or_else(|| "-".to_string());
        let binding_id = ids
            .iter()
            .find(|(_, v)| **v == variant)
            .map_or("-".to_string(), |(id, _)| id.to_string());
        rows.push(vec![
            number.to_string(),
            code(&variant),
            constant,
            binding_id,
        ]);
    }
    assert!(
        constants.is_empty(),
        "xalen.swe exports unknown SIDM numbers: {constants:?}"
    );
    let header = [
        "Swiss number",
        "`Ayanamsa`",
        "`xalen.swe` constant",
        "Binding ID",
    ];
    table(&header, &rows)
}

fn houses_table() -> String {
    let (ids, accepted) = binding_ids("house_system_from_id");

    // HouseSystem variant -> the Swiss `hsys` letters compat accepts for it.
    let mut letters: BTreeMap<String, Vec<char>> = BTreeMap::new();
    for (pattern, variant) in match_arms(COMPAT, "se_hsys_to_system") {
        for alternative in pattern.split('|') {
            let letter = alternative.trim().trim_matches('\'');
            let mut chars = letter.chars();
            let (Some(ch), None) = (chars.next(), chars.next()) else {
                panic!("se_hsys_to_system: `{alternative}` is not one letter");
            };
            assert!(
                compat::swe_houses(J2000, 28.6, 77.2, ch).is_ok(),
                "compat::swe_houses rejects '{ch}'"
            );
            letters.entry(variant.clone()).or_default().push(ch);
        }
    }
    let letter_cell = |variant: &str| match letters.get(variant) {
        Some(chars) => chars
            .iter()
            .map(|ch| code(&ch.to_string()))
            .collect::<Vec<_>>()
            .join(", "),
        None => "-".to_string(),
    };

    let mut rows = Vec::new();
    for (&id, variant) in &ids {
        let mut row = vec![id.to_string(), code(variant), letter_cell(variant.as_str())];
        row.extend(accept_cells(&accepted, id));
        rows.push(row);
    }
    let mut swiss_only: Vec<(&String, &Vec<char>)> = letters
        .iter()
        .filter(|(variant, _)| !ids.values().any(|v| v == *variant))
        .collect();
    swiss_only.sort_by_key(|(_, chars)| chars[0]);
    for (variant, _) in swiss_only {
        let mut row = vec![
            "-".to_string(),
            code(variant),
            letter_cell(variant.as_str()),
        ];
        row.extend(BINDINGS.iter().map(|_| "-".to_string()));
        rows.push(row);
    }
    let header = binding_header(&["ID", "`HouseSystem`", "Swiss `hsys` letter"]);
    table(&header, &rows)
}

fn vargas_table() -> String {
    let (ids, accepted) = binding_ids("varga_from_id");
    let mut rows = Vec::new();
    for (&id, variant) in &ids {
        let mut row = vec![id.to_string(), code(variant)];
        row.extend(accept_cells(&accepted, id));
        rows.push(row);
    }
    table(&binding_header(&["ID", "`VargaChart`"]), &rows)
}

fn flags_table() -> String {
    // swe_compat.rs registers each flag as `FLG_X` and `SE` + `FLG_X`.
    let mut flags: Vec<(i32, String)> = shim_list("flags")
        .into_iter()
        .map(|(name, konst)| (compat_const(&konst), name))
        .collect();
    flags.sort();
    let mut rows = Vec::new();
    for (value, name) in flags {
        let names = format!("{} / {}", code(&name), code(&format!("SE{name}")));
        let calc_ut = match compat::swe_calc_ut(J2000, compat::SE_SUN, value) {
            Ok(_) => "accepted".to_string(),
            Err(_) => "rejected (`ValueError`)".to_string(),
        };
        rows.push(vec![value.to_string(), names, calc_ut]);
    }
    table(&["Value", "`xalen.swe` names", "`calc_ut`"], &rows)
}

fn calendars_table() -> String {
    let mut calendars: BTreeMap<i32, Vec<String>> = BTreeMap::new();
    for line in SWE_SHIM.lines() {
        if !line.trim_start().starts_with("m.add(\"") {
            continue;
        }
        if let Some((name, konst)) = quoted_pair(line) {
            calendars
                .entry(compat_const(&konst))
                .or_default()
                .push(code(&name));
        }
    }
    let rows: Vec<Vec<String>> = calendars
        .into_iter()
        .map(|(value, names)| vec![value.to_string(), names.join(" / ")])
        .collect();
    table(&["Value", "`xalen.swe` names"], &rows)
}

fn blocks() -> Vec<(&'static str, String)> {
    vec![
        ("bodies", bodies_table()),
        ("ayanamsas", ayanamsas_table()),
        ("swiss-ayanamsas", swiss_ayanamsas_table()),
        ("houses", houses_table()),
        ("vargas", vargas_table()),
        ("swe-flags", flags_table()),
        ("swe-calendars", calendars_table()),
    ]
}

// ---------------------------------------------------------------------------
// Document handling
// ---------------------------------------------------------------------------

/// Replace the text between the `name` markers of `doc` with `table`.
fn splice(doc: &str, name: &str, table: &str) -> String {
    let begin = format!("<!-- BEGIN GENERATED {name} -->\n");
    let end = format!("<!-- END GENERATED {name} -->");
    let Some(start) = doc.find(&begin) else {
        panic!("{DOC} lacks the marker `{}`", begin.trim_end());
    };
    let start = start + begin.len();
    let Some(len) = doc[start..].find(&end) else {
        panic!("{DOC} lacks the marker `{end}`");
    };
    format!("{}{}{}", &doc[..start], table, &doc[start + len..])
}

/// Lines to remove (`-`) and add (`+`) to turn `old` into `new`.
fn line_diff(old: &str, new: &str) -> String {
    let a: Vec<&str> = old.lines().collect();
    let b: Vec<&str> = new.lines().collect();
    let mut lcs = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for i in (0..a.len()).rev() {
        for j in (0..b.len()).rev() {
            lcs[i][j] = if a[i] == b[j] {
                lcs[i + 1][j + 1] + 1
            } else {
                lcs[i + 1][j].max(lcs[i][j + 1])
            };
        }
    }
    let (mut i, mut j) = (0, 0);
    let mut out = String::new();
    while i < a.len() || j < b.len() {
        if i < a.len() && j < b.len() && a[i] == b[j] {
            i += 1;
            j += 1;
        } else if i < a.len() && (j == b.len() || lcs[i + 1][j] >= lcs[i][j + 1]) {
            out.push_str(&format!("- line {}: {}\n", i + 1, a[i]));
            i += 1;
        } else {
            out.push_str(&format!("+ line {}: {}\n", i + 1, b[j]));
            j += 1;
        }
    }
    out
}

#[test]
fn binding_id_tables_match_the_docs() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(DOC);
    let raw = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) => panic!("read {DOC}: {e}"),
    };
    // A Windows checkout may have CRLF line endings; compare the text itself.
    let committed = raw.replace("\r\n", "\n");
    let mut generated = committed.clone();
    for (name, table) in blocks() {
        generated = splice(&generated, name, &table);
    }
    if generated == committed {
        return;
    }
    if std::env::var_os(REGEN_ENV).is_some() {
        if let Err(e) = std::fs::write(&path, &generated) {
            panic!("write {DOC}: {e}");
        }
        return;
    }
    panic!(
        "{DOC} is out of date with the binding sources.\n\
         Run scripts/regen-binding-ids.sh and commit the result.\n\
         Changes (- committed, + generated):\n{}",
        line_diff(&committed, &generated)
    );
}

/// The source reader for `compat.rs` constants agrees with the compiled values,
/// so the tables cannot show a number the library does not use.
#[test]
fn parsed_compat_constants_match_compiled_values() {
    let compiled = [
        ("SE_SUN", compat::SE_SUN),
        ("SE_MOON", compat::SE_MOON),
        ("SE_MERCURY", compat::SE_MERCURY),
        ("SE_VENUS", compat::SE_VENUS),
        ("SE_MARS", compat::SE_MARS),
        ("SE_JUPITER", compat::SE_JUPITER),
        ("SE_SATURN", compat::SE_SATURN),
        ("SE_URANUS", compat::SE_URANUS),
        ("SE_NEPTUNE", compat::SE_NEPTUNE),
        ("SE_PLUTO", compat::SE_PLUTO),
        ("SE_MEAN_NODE", compat::SE_MEAN_NODE),
        ("SE_TRUE_NODE", compat::SE_TRUE_NODE),
        ("SE_MEAN_APOG", compat::SE_MEAN_APOG),
        ("SE_OSCU_APOG", compat::SE_OSCU_APOG),
        ("SE_CHIRON", compat::SE_CHIRON),
        ("SE_EARTH", compat::SE_EARTH),
        ("SEFLG_SWIEPH", compat::SEFLG_SWIEPH),
        ("SEFLG_SPEED", compat::SEFLG_SPEED),
        ("SEFLG_SIDEREAL", compat::SEFLG_SIDEREAL),
        ("SEFLG_HELCTR", compat::SEFLG_HELCTR),
        ("SEFLG_J2000", compat::SEFLG_J2000),
        ("SEFLG_EQUATORIAL", compat::SEFLG_EQUATORIAL),
        ("SEFLG_BARYCTR", compat::SEFLG_BARYCTR),
        ("SEFLG_TOPOCTR", compat::SEFLG_TOPOCTR),
        ("SEFLG_XYZ", compat::SEFLG_XYZ),
        ("SEFLG_RADIANS", compat::SEFLG_RADIANS),
        ("SE_SIDM_FAGAN_BRADLEY", compat::SE_SIDM_FAGAN_BRADLEY),
        ("SE_SIDM_LAHIRI", compat::SE_SIDM_LAHIRI),
        ("SE_SIDM_KRISHNAMURTI", compat::SE_SIDM_KRISHNAMURTI),
        ("SE_SIDM_TRUE_CITRA", compat::SE_SIDM_TRUE_CITRA),
        ("SE_SIDM_TRUE_REVATI", compat::SE_SIDM_TRUE_REVATI),
        ("SE_GREG_CAL", compat::SE_GREG_CAL),
        ("SE_JUL_CAL", compat::SE_JUL_CAL),
    ];
    for (name, value) in compiled {
        assert_eq!(compat_const(name), value, "{name}");
    }
}
