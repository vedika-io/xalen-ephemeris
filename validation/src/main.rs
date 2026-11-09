//! XALEN reproducible accuracy validation runner.
//!
//! Reads a pyswisseph oracle file (JSON Lines, produced by
//! `validation/oracle_pyswisseph.py`), recomputes the SAME quantities with the
//! pure-Rust XALEN crates, and prints per-body max / mean / RMS / p99 absolute delta
//! (p99 from a log histogram) in arcseconds plus the worst offender, an overall PASS/FAIL against a
//! configurable degree threshold (default 0.1deg legacy bound), and the tighter
//! <1" subset fraction.
//!
//! Usage:
//!     cargo run -p xalen-validation --release -- ORACLE.jsonl [--threshold-deg D]
//!
//! Honest interpretation: the oracle is only as accurate as the Swiss backend
//! the Python side actually used. When pyswisseph has no `.se1`/`.bsp` data
//! files installed it falls back to the analytic Moshier theory, so the deltas
//! reported here are XALEN-vs-Moshier (two independent analytic chains), NOT
//! XALEN-vs-DE440. The per-body backend the oracle reported is printed in the
//! header so the numbers can be read correctly. See `validation/README.md`.

mod oracle;
mod stats;

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};

use oracle::OracleRecord;
use stats::{DeltaStats, angle_sep_deg};

use xalen_ayanamsa::Ayanamsa;
use xalen_ephem::{Almanac, Body, EphemerisError};
use xalen_houses::{GeoLocation, HouseSystem, compute_houses};
use xalen_time::{DeltaTModel, JdUT1};

/// IAU 2006 mean obliquity of the ecliptic in radians from a Julian Day.
/// Byte-for-byte the polynomial used by the committed Swiss house oracle and by
/// `xalen_coords::obliquity::mean_obliquity`, so the house comparison here
/// reproduces the production caller's `epsilon` exactly.
fn mean_obliquity_rad(jd: f64) -> f64 {
    let t = (jd - 2_451_545.0) / 36525.0;
    let eps_arcsec = 84381.406 - 46.836769 * t - 0.0001831 * t * t + 0.00200340 * t * t * t
        - 0.000000576 * t.powi(4)
        - 0.0000000434 * t.powi(5);
    (eps_arcsec / 3600.0).to_radians()
}

/// Map an oracle body name to the corresponding `xalen_ephem::Body`.
fn body_for(name: &str) -> Option<Body> {
    Some(match name {
        "sun" => Body::Sun,
        "moon" => Body::Moon,
        "mercury" => Body::Mercury,
        "venus" => Body::Venus,
        "mars" => Body::Mars,
        "jupiter" => Body::Jupiter,
        "saturn" => Body::Saturn,
        "uranus" => Body::Uranus,
        "neptune" => Body::Neptune,
        "pluto" => Body::Pluto,
        "mean_node" => Body::MeanNode,
        "true_node" => Body::TrueNode,
        "mean_apogee" => Body::MeanApogee,
        "osculating_apogee" => Body::OsculatingApogee,
        _ => return None,
    })
}

/// Accumulates the overall PASS/FAIL verdict and worst-offender while printing
/// each quantity's row. A plain struct (rather than a capturing closure) keeps
/// the borrow checker happy when the verdict fields are read after the loop.
struct ReportAcc {
    threshold_arcsec: f64,
    overall_pass: bool,
    worst_max_arcsec: f64,
    worst_label: String,
}

impl ReportAcc {
    fn new(threshold_arcsec: f64) -> Self {
        Self {
            threshold_arcsec,
            overall_pass: true,
            worst_max_arcsec: 0.0,
            worst_label: String::new(),
        }
    }

    /// Print one row. `gating` controls whether this quantity participates in
    /// the overall PASS/FAIL verdict and the worst-offender tracking. Body
    /// latitude is reported but NOT gating: XALEN models the lunar nodes and
    /// both apogees as ecliptic points (latitude = 0 by construction), so their
    /// latitude cannot match Swiss's osculating-orbit latitude — that is a
    /// documented convention difference, not a position error.
    fn emit_inner(&mut self, label: &str, s: &DeltaStats, gating: bool) {
        if s.count == 0 {
            return;
        }
        let worst = s
            .worst_chart
            .map(|(j, la, lo)| format!("{j:.2},{la:.1},{lo:.1}"))
            .unwrap_or_default();
        let flag = if gating && s.over_legacy > 0 {
            "  <-- OVER"
        } else if !gating {
            "  (info)"
        } else {
            ""
        };
        println!(
            "{:<20} {:>10} {:>12.4} {:>12.4} {:>12.4} {:>12.4} {:>8.1}% {:>22}{}",
            label,
            s.count,
            s.mean_arcsec(),
            s.rms_arcsec(),
            s.p99_arcsec(),
            s.max_arcsec,
            s.within_1_arcsec_frac() * 100.0,
            worst,
            flag,
        );
        if gating {
            if s.max_arcsec > self.worst_max_arcsec {
                self.worst_max_arcsec = s.max_arcsec;
                self.worst_label = label.to_string();
            }
            if s.max_arcsec > self.threshold_arcsec {
                self.overall_pass = false;
            }
        }
    }

    /// Gating quantity (longitudes, houses, ayanamsa).
    fn emit(&mut self, label: &str, s: &DeltaStats) {
        self.emit_inner(label, s, true);
    }

    /// Informational-only quantity (body latitude).
    fn emit_info(&mut self, label: &str, s: &DeltaStats) {
        self.emit_inner(label, s, false);
    }
}

/// Map an oracle ayanamsa name to its Swiss id, then to `Ayanamsa`.
fn ayanamsa_for(name: &str) -> Option<Ayanamsa> {
    let sid = match name {
        "lahiri" => 1,
        "fagan_bradley" => 0,
        "krishnamurti" => 5,
        "raman" => 3,
        _ => return None,
    };
    Ayanamsa::from_swiss_ephem_id(sid)
}

fn main() {
    if let Err(error) = run() {
        eprintln!("validation error: {error}");
        std::process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let path = match args.next() {
        Some(p) if !p.starts_with("--") => p,
        _ => {
            eprintln!(
                "usage: xalen-validation ORACLE.jsonl [--threshold-deg D]\n\
                 (generate ORACLE.jsonl with: python3 validation/oracle_pyswisseph.py > ORACLE.jsonl)"
            );
            std::process::exit(2);
        }
    };
    let mut threshold_deg = 0.1_f64;
    while let Some(a) = args.next() {
        if a != "--threshold-deg" {
            return Err(format!("unknown argument: {a}"));
        }
        let value = args.next().ok_or("--threshold-deg requires a value")?;
        threshold_deg = value
            .parse()
            .map_err(|_| format!("invalid threshold: {value}"))?;
        if !threshold_deg.is_finite()
            || threshold_deg < 0.0
            || !(threshold_deg * 3600.0).is_finite()
        {
            return Err(
                "threshold must be finite, non-negative, and representable in arcseconds".into(),
            );
        }
    }

    // "-" reads the oracle stream from stdin so the Python generator can be
    // piped straight in without materializing a multi-GB file at large N.
    let reader: Box<dyn BufRead> = if path == "-" {
        Box::new(BufReader::new(std::io::stdin()))
    } else {
        Box::new(BufReader::new(
            std::fs::File::open(&path)
                .map_err(|e| format!("cannot open oracle file {path}: {e}"))?,
        ))
    };

    let almanac = Almanac::default_vedic();
    let dt = DeltaTModel::StephensonMorrisonHohenkerk2016;

    // Per-quantity accumulators.
    let mut lon_stats: BTreeMap<String, DeltaStats> = BTreeMap::new();
    let mut lat_stats: BTreeMap<String, DeltaStats> = BTreeMap::new();
    let mut asc_stats = DeltaStats::default();
    let mut mc_stats = DeltaStats::default();
    let mut cusp_stats = DeltaStats::default();
    let mut ayan_stats: BTreeMap<String, DeltaStats> = BTreeMap::new();
    let mut coverage_skips: BTreeMap<String, u64> = BTreeMap::new();

    let mut meta: Option<oracle::OracleMeta> = None;
    let mut n_charts: u64 = 0;
    for (index, line) in reader.lines().enumerate() {
        let line_number = index + 1;
        let line = line.map_err(|e| format!("line {line_number}: cannot read oracle: {e}"))?;
        if line.trim().is_empty() {
            continue;
        }
        let rec: OracleRecord = serde_json::from_str(&line)
            .map_err(|e| format!("line {line_number}: invalid oracle record: {e}"))?;
        validate_record(&rec).map_err(|e| format!("line {line_number}: {e}"))?;
        if let Some(m) = &rec.meta {
            if n_charts != 0 || meta.is_some() {
                return Err(format!(
                    "line {line_number}: _meta is allowed only on the first chart"
                ));
            }
            if m.n == 0 {
                return Err(format!("line {line_number}: _meta.n must be positive"));
            }
            meta = Some(m.clone());
        }
        n_charts += 1;
        process_record(
            &rec,
            &almanac,
            &dt,
            &mut lon_stats,
            &mut lat_stats,
            &mut asc_stats,
            &mut mc_stats,
            &mut cusp_stats,
            &mut ayan_stats,
            &mut coverage_skips,
        )
        .map_err(|e| format!("line {line_number}: {e}"))?;
    }
    if n_charts == 0 {
        return Err("oracle contains no charts".into());
    }
    if let Some(m) = &meta
        && m.n != n_charts
    {
        return Err(format!(
            "oracle is incomplete: _meta.n declares {} charts, read {n_charts}",
            m.n
        ));
    }
    for name in coverage_skips.keys() {
        if !lon_stats.contains_key(name) {
            return Err(format!(
                "no comparable samples for {name}: every epoch is outside coverage"
            ));
        }
    }

    print_report(
        &path,
        threshold_deg,
        n_charts,
        meta.as_ref(),
        &lon_stats,
        &lat_stats,
        &asc_stats,
        &mc_stats,
        &cusp_stats,
        &ayan_stats,
        &coverage_skips,
    );
    Ok(())
}

/// Require the complete schema emitted by oracle_pyswisseph.py. A missing or
/// unknown quantity must not silently shrink the scope of an accuracy verdict.
fn validate_record(rec: &OracleRecord) -> Result<(), String> {
    if !rec.jd.is_finite() || !mean_obliquity_rad(rec.jd).is_finite() {
        return Err("jd cannot be evaluated with finite arithmetic".into());
    }
    if !(-90.0..=90.0).contains(&rec.lat) || !(-180.0..=180.0).contains(&rec.lon) {
        return Err("latitude/longitude are outside their geographic ranges".into());
    }
    if rec.bodies.len() != 14 || rec.bodies.keys().any(|name| body_for(name).is_none()) {
        return Err("bodies must contain all 14 supported oracle body names".into());
    }
    if rec.ayanamsa.len() != 4 || rec.ayanamsa.keys().any(|name| ayanamsa_for(name).is_none()) {
        return Err("ayanamsa must contain all four supported oracle names".into());
    }
    if rec.houses.cusps.len() != 12 {
        return Err("houses must contain exactly 12 cusps".into());
    }
    for (name, &[lon, lat]) in &rec.bodies {
        if !lon.is_finite() || !(-90.0..=90.0).contains(&lat) {
            return Err(format!("invalid oracle coordinates for {name}"));
        }
    }
    if !rec.houses.asc.is_finite()
        || !rec.houses.mc.is_finite()
        || rec
            .houses
            .cusps
            .iter()
            .chain(rec.ayanamsa.values())
            .any(|v| !v.is_finite())
    {
        return Err("oracle angles must be finite".into());
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn process_record(
    rec: &OracleRecord,
    almanac: &Almanac,
    dt: &DeltaTModel,
    lon_stats: &mut BTreeMap<String, DeltaStats>,
    lat_stats: &mut BTreeMap<String, DeltaStats>,
    asc_stats: &mut DeltaStats,
    mc_stats: &mut DeltaStats,
    cusp_stats: &mut DeltaStats,
    ayan_stats: &mut BTreeMap<String, DeltaStats>,
    coverage_skips: &mut BTreeMap<String, u64>,
) -> Result<(), String> {
    let jd = rec.jd;
    let jd_ut1 = JdUT1(jd);

    // ── Bodies: apparent geocentric ecliptic longitude (+ latitude) ──────
    for (name, &[o_lon, o_lat]) in &rec.bodies {
        let body = body_for(name).ok_or_else(|| format!("unknown body: {name}"))?;
        let pos = match almanac.geocentric_ecliptic(body, jd_ut1) {
            Ok(pos) => pos,
            // The default 1500–2400 oracle span intentionally exceeds the
            // analytic Pluto series' coverage. Disclose these exclusions, but
            // do not turn other computation failures into missing samples.
            Err(EphemerisError::EpochOutOfRange(_)) => {
                *coverage_skips.entry(name.clone()).or_default() += 1;
                continue;
            }
            Err(e) => return Err(format!("cannot compute {name}: {e}")),
        };
        if !pos.longitude_deg().is_finite() || !pos.latitude_deg().is_finite() {
            return Err(format!("non-finite computed coordinates for {name}"));
        }
        let x_lon = pos.longitude_deg().rem_euclid(360.0);
        let d_lon = angle_sep_deg(x_lon, o_lon);
        lon_stats
            .entry(name.clone())
            .or_default()
            .record_deg(d_lon, jd, rec.lat, rec.lon);

        // Nodes report latitude 0 by construction; compare latitude only
        // where the oracle gives a non-degenerate value (planets, Moon,
        // apogees). Latitude is a signed small angle, so a plain absolute
        // difference is the right metric (no 360deg wrap concern).
        let d_lat = (pos.latitude_deg() - o_lat).abs();
        lat_stats
            .entry(name.clone())
            .or_default()
            .record_deg(d_lat, jd, rec.lat, rec.lon);
    }

    // ── Houses: tropical Placidus. XALEN's compute_houses uses GMST-derived
    //    RAMC + mean obliquity (consistent across all house paths; cusp geometry
    //    validated to <0.01° vs Swiss at a given RAMC). The residual vs Swiss
    //    swe.houses_ex (which uses apparent sidereal time) is the equation of
    //    equinoxes, ~0.003° — well within the gate. Use mean obliquity to match
    //    XALEN's convention. ─────────────────────────────────────────────────
    let eps = mean_obliquity_rad(jd);
    // Placidus (and most quadrant systems) are classically unreliable toward the
    // polar circles: as |lat| rises the ascendant becomes hypersensitive to the
    // sidereal-time reference, so the ~0.003° equation-of-equinoxes difference
    // between XALEN's GMST-derived RAMC and Swiss swe.houses_ex's apparent RAMC
    // amplifies into ~0.1° in the ascendant near ±66°. The cusp geometry itself
    // matches Swiss to <0.01° at a shared RAMC (swiss_houses_oracle). Compare in
    // the |lat| ≤ 60° band where Placidus is well-conditioned (covering essentially
    // all inhabited latitudes); higher latitudes are out of scope, not an error.
    if rec.lat.abs() <= 60.0 {
        let loc = GeoLocation::new(rec.lat, rec.lon);
        let h = compute_houses(jd, &loc, eps, HouseSystem::Placidus);
        if !h.ascendant.is_finite()
            || !h.mc.is_finite()
            || (0..12).any(|i| !h.cusp_deg(i).is_finite())
        {
            return Err("non-finite computed houses".into());
        }
        asc_stats.record_deg(
            angle_sep_deg(h.ascendant.to_degrees(), rec.houses.asc),
            jd,
            rec.lat,
            rec.lon,
        );
        mc_stats.record_deg(
            angle_sep_deg(h.mc.to_degrees(), rec.houses.mc),
            jd,
            rec.lat,
            rec.lon,
        );
        for (i, &o_cusp) in rec.houses.cusps.iter().enumerate().take(12) {
            cusp_stats.record_deg(angle_sep_deg(h.cusp_deg(i), o_cusp), jd, rec.lat, rec.lon);
        }
    }

    // ── Ayanamsa: with-nutation apparent value. The oracle samples it at the
    //    UT epoch; XALEN's compute_deg takes a TT epoch, so convert with the
    //    same SMH2016 model the production path uses. (For the ayanamsa the
    //    UT/TT distinction is sub-milliarcsec, but we convert anyway to mirror
    //    the real pipeline rather than feed UT as TT.) ─────────────────────
    let jd_tt = jd_ut1.to_tt(dt).0;
    if !jd_tt.is_finite() {
        return Err("non-finite TT epoch".into());
    }
    for (name, &o_ayan) in &rec.ayanamsa {
        let a = ayanamsa_for(name).ok_or_else(|| format!("unknown ayanamsa: {name}"))?;
        let x_ayan = a.compute_deg(jd_tt);
        if !x_ayan.is_finite() {
            return Err(format!("non-finite computed ayanamsa: {name}"));
        }
        ayan_stats.entry(name.clone()).or_default().record_deg(
            angle_sep_deg(x_ayan, o_ayan),
            jd,
            rec.lat,
            rec.lon,
        );
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn print_report(
    path: &str,
    threshold_deg: f64,
    n_charts: u64,
    meta: Option<&oracle::OracleMeta>,
    lon_stats: &BTreeMap<String, DeltaStats>,
    lat_stats: &BTreeMap<String, DeltaStats>,
    asc_stats: &DeltaStats,
    mc_stats: &DeltaStats,
    cusp_stats: &DeltaStats,
    ayan_stats: &BTreeMap<String, DeltaStats>,
    coverage_skips: &BTreeMap<String, u64>,
) {
    println!("XALEN reproducible accuracy validation");
    println!("oracle file : {path}");
    println!("charts read : {n_charts}  (parse errors: 0)");
    for (name, count) in coverage_skips {
        println!("excluded    : {name}: {count} epochs outside calculation coverage");
    }
    if asc_stats.count < n_charts {
        println!(
            "excluded    : houses: {} charts outside |latitude| <= 60 deg",
            n_charts - asc_stats.count
        );
    }
    if let Some(m) = meta {
        println!(
            "oracle      : pyswisseph {}, n={}, seed={}, span JD {:.1}..{:.1}",
            m.swisseph_version, m.n, m.seed, m.start_jd, m.end_jd
        );
        println!(
            "ephe_path   : {}",
            m.ephe_path
                .as_deref()
                .unwrap_or("(none — analytic fallback)")
        );
        if !m.body_backend.is_empty() {
            let backends: Vec<String> = m
                .body_backend
                .iter()
                .map(|(b, e)| format!("{b}={e}"))
                .collect();
            println!("backend     : {}", backends.join(" "));
            let any_moshier = m.body_backend.values().any(|e| e == "moshier");
            if any_moshier {
                println!(
                    "NOTE        : oracle used the Moshier analytic fallback for one or more \
                     bodies (no Swiss/JPL data files). Deltas for those bodies are \
                     XALEN-vs-Moshier, two independent analytic theories, NOT XALEN-vs-DE440."
                );
            }
        }
    }
    let threshold_arcsec = threshold_deg * 3600.0;
    println!("threshold   : {threshold_deg} deg ({threshold_arcsec:.0}\") max per-body bound");
    println!();

    println!(
        "{:<20} {:>10} {:>12} {:>12} {:>12} {:>12} {:>9} {:>22}",
        "quantity",
        "count",
        "mean(\")",
        "rms(\")",
        "p99(\")",
        "max(\")",
        "<1\"%",
        "worst(jd,lat,lon)"
    );
    println!("{}", "-".repeat(113));

    let mut acc = ReportAcc::new(threshold_arcsec);

    println!("-- body longitude (apparent geocentric, deg-of-date) --");
    for (name, s) in lon_stats {
        // The lunar apogees ("Black Moon Lilith") and the osculating true node are
        // abstract derived points whose definition varies across ephemeris
        // software: mean apogee = Meeus mean-perigee+180° vs Swiss SE_MEAN_APOG's
        // mean theory (~0.07-0.12°); osculating apogee is derivative-sensitive
        // (~0.3° in the tail); the osculating true node likewise. They are
        // reported but NOT gated — the gated lunar node is the mean node, which
        // matches Swiss to <0.2".
        if matches!(
            name.as_str(),
            "mean_apogee" | "osculating_apogee" | "true_node"
        ) {
            acc.emit_info(&format!("lon:{name}"), s);
        } else {
            acc.emit(&format!("lon:{name}"), s);
        }
    }
    println!("-- body latitude (informational; nodes/apogees are modelled as ecliptic points) --");
    for (name, s) in lat_stats {
        acc.emit_info(&format!("lat:{name}"), s);
    }
    println!("-- houses (tropical Placidus) --");
    acc.emit("house:asc", asc_stats);
    acc.emit("house:mc", mc_stats);
    acc.emit("house:cusps", cusp_stats);
    println!("-- ayanamsa (with-nutation apparent) --");
    for (name, s) in ayan_stats {
        acc.emit(&format!("ayan:{name}"), s);
    }

    let ReportAcc {
        overall_pass,
        worst_max_arcsec,
        worst_label,
        ..
    } = acc;

    println!();
    println!(
        "worst quantity : {worst_label} at {worst_max_arcsec:.4}\" \
         ({:.6} deg)",
        worst_max_arcsec / 3600.0
    );
    if overall_pass {
        println!(
            "RESULT      : PASS — every evaluated gated sample within {threshold_deg} deg; {n_charts} charts read (see counts and exclusions)"
        );
    } else {
        println!("RESULT      : FAIL — at least one quantity exceeded {threshold_deg} deg");
    }

    if !overall_pass {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oracle::OracleHouses;

    #[test]
    fn body_name_mapping_is_complete() {
        // Every oracle body name resolves to a Body.
        for name in [
            "sun",
            "moon",
            "mercury",
            "venus",
            "mars",
            "jupiter",
            "saturn",
            "uranus",
            "neptune",
            "pluto",
            "mean_node",
            "true_node",
            "mean_apogee",
            "osculating_apogee",
        ] {
            assert!(body_for(name).is_some(), "unmapped body {name}");
        }
        assert!(body_for("nonsense").is_none());
    }

    #[test]
    fn ayanamsa_name_mapping_resolves() {
        assert_eq!(ayanamsa_for("lahiri"), Some(Ayanamsa::Lahiri));
        assert_eq!(ayanamsa_for("fagan_bradley"), Some(Ayanamsa::FaganBradley));
        assert_eq!(ayanamsa_for("krishnamurti"), Some(Ayanamsa::KPKrishnamurti));
        assert_eq!(ayanamsa_for("raman"), Some(Ayanamsa::Raman));
        assert!(ayanamsa_for("nonsense").is_none());
    }

    #[test]
    fn mean_obliquity_matches_coords_crate() {
        // The local helper must reproduce xalen_coords::mean_obliquity (which
        // takes Julian centuries from J2000) bit-for-bit at several epochs, so
        // the house comparison uses the production caller's epsilon.
        for jd in [2_268_923.5_f64, 2_451_545.0, 2_597_641.5] {
            let t = (jd - 2_451_545.0) / 36525.0;
            let coords = xalen_coords::mean_obliquity(t);
            let local = mean_obliquity_rad(jd);
            assert!(
                (coords - local).abs() < 1e-12,
                "obliquity drift at jd={jd}: coords={coords} local={local}"
            );
        }
    }

    #[test]
    fn report_acc_gating_and_info() {
        let mut acc = ReportAcc::new(0.1 * 3600.0); // 360"
        // A gating quantity over the bound flips overall_pass to false.
        let mut over = DeltaStats::default();
        over.record_deg(0.2, 0.0, 0.0, 0.0); // 720" > 360"
        acc.emit("lon:test", &over);
        assert!(!acc.overall_pass);
        assert_eq!(acc.worst_label, "lon:test");

        // An info-only quantity NEVER affects the verdict, even when huge.
        let mut acc2 = ReportAcc::new(0.1 * 3600.0);
        let mut big_lat = DeltaStats::default();
        big_lat.record_deg(5.0, 0.0, 0.0, 0.0); // 5deg latitude divergence
        acc2.emit_info("lat:mean_apogee", &big_lat);
        assert!(acc2.overall_pass, "info row must not gate the verdict");
        assert_eq!(acc2.worst_label, "", "info row must not set worst label");
    }

    /// End-to-end: feed one synthetic record and confirm the accumulators fill
    /// and that XALEN's J2000 Sun longitude lands near the apparent value Swiss
    /// reports (sanity that the wiring computes real positions, not zeros).
    #[test]
    fn process_record_populates_and_is_sane() {
        let almanac = Almanac::default_vedic();
        let dt = DeltaTModel::StephensonMorrisonHohenkerk2016;
        let mut lon = BTreeMap::new();
        let mut lat = BTreeMap::new();
        let mut asc = DeltaStats::default();
        let mut mc = DeltaStats::default();
        let mut cusp = DeltaStats::default();
        let mut ayan = BTreeMap::new();
        let mut coverage_skips = BTreeMap::new();

        let mut bodies = BTreeMap::new();
        // Swiss apparent geocentric Sun longitude at J2000 ~= 280.3689 deg.
        bodies.insert("sun".to_string(), [280.3689_f64, 0.0_f64]);
        let mut ay = BTreeMap::new();
        // Swiss APPARENT (with-nutation) Lahiri ayanamsa at J2000, per the
        // authoritative swiss_ayanamsa_oracle (get_ayanamsa_ex(jd,0)) — the
        // quantity XALEN's compute_deg returns. (The older get_ayanamsa_ut mean
        // value 23.857092 differs by the ~14" nutation in longitude.)
        ay.insert("lahiri".to_string(), 23.853222_f64);
        let rec = OracleRecord {
            jd: 2_451_545.0,
            lat: 28.6,
            lon: 77.2,
            bodies,
            houses: OracleHouses {
                asc: 100.0,
                mc: 10.0,
                cusps: vec![100.0; 12],
            },
            ayanamsa: ay,
            meta: None,
        };

        process_record(
            &rec,
            &almanac,
            &dt,
            &mut lon,
            &mut lat,
            &mut asc,
            &mut mc,
            &mut cusp,
            &mut ayan,
            &mut coverage_skips,
        )
        .unwrap();

        // The Sun longitude delta vs the Swiss value must be small (well under
        // the 0.1deg legacy bound) — this proves the pipeline computes a real
        // apparent place, not a stub.
        let sun = lon.get("sun").expect("sun longitude recorded");
        assert_eq!(sun.count, 1);
        assert!(
            sun.max_arcsec < 5.0,
            "J2000 Sun longitude delta {:.3}\" unexpectedly large",
            sun.max_arcsec
        );
        // Lahiri ayanamsa delta vs the Swiss J2000 value must be sub-arcsec.
        let lah = ayan.get("lahiri").expect("lahiri ayanamsa recorded");
        assert!(
            lah.max_arcsec < 2.0,
            "Lahiri ayanamsa delta {:.3}\" unexpectedly large",
            lah.max_arcsec
        );
        // Houses accumulator received all 12 cusps + asc + mc.
        assert_eq!(cusp.count, 12);
        assert_eq!(asc.count, 1);
        assert_eq!(mc.count, 1);
    }
}
