//! These fixtures test verdict integrity, not astronomical accuracy. Their zero
//! reference values intentionally pass only under a very loose 360-degree bound.
use serde_json::{Value, json};
use std::io::Write;
use std::process::{Command, Output, Stdio};

fn record() -> Value {
    let mut bodies = serde_json::Map::new();
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
        bodies.insert(name.into(), json!([0.0, 0.0]));
    }
    json!({"jd":2451545.0,"lat":0.0,"lon":0.0,"bodies":bodies,
        "houses":{"asc":0.0,"mc":0.0,"cusps":vec![0.0;12]},
        "ayanamsa":{"lahiri":0.0,"fagan_bradley":0.0,"krishnamurti":0.0,"raman":0.0}})
}

fn run(input: &str, args: &[&str]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_xalen-validation"))
        .arg("-")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // Invalid CLI arguments can close stdin before it is written.
    let _ = child.stdin.take().unwrap().write_all(input.as_bytes());
    child.wait_with_output().unwrap()
}

fn invalid(input: &str, args: &[&str]) {
    let out = run(input, args);
    assert_eq!(
        out.status.code(),
        Some(2),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!String::from_utf8_lossy(&out.stdout).contains("RESULT      : PASS"));
    assert!(!out.stderr.is_empty());
}

#[test]
fn empty_and_malformed_oracles_never_pass() {
    for input in ["", " \n\t\n", "not-json\n", "{}\n", "{\"jd\":NaN}\n"] {
        invalid(input, &[]);
    }
    invalid(
        &format!("{}\nnot-json\n", record()),
        &["--threshold-deg", "360"],
    );
}

#[test]
fn threshold_and_unknown_arguments_are_validated() {
    // A valid chart prevents the empty-input guard from masking CLI bugs.
    let input = record().to_string();
    for value in ["NaN", "inf", "-inf", "-0.1", "1e308", "bad"] {
        invalid(&input, &["--threshold-deg", value]);
    }
    invalid(&input, &["--threshold-deg"]);
    invalid(&input, &["--typo", "360"]);
}

#[test]
fn incomplete_or_unknown_quantities_are_rejected() {
    let mut samples = vec![];
    let mut r = record();
    r["bodies"].as_object_mut().unwrap().remove("sun");
    samples.push(r);
    let mut r = record();
    r["bodies"].as_object_mut().unwrap().remove("sun");
    r["bodies"]["unknown"] = json!([0.0, 0.0]);
    samples.push(r);
    let mut r = record();
    r["ayanamsa"] = json!({});
    samples.push(r);
    let mut r = record();
    r["ayanamsa"].as_object_mut().unwrap().remove("lahiri");
    r["ayanamsa"]["unknown"] = json!(0.0);
    samples.push(r);
    let mut r = record();
    r["bodies"]["sun"] = json!([0.0, 91.0]);
    samples.push(r);
    let mut r = record();
    r["houses"]["cusps"] = json!(vec![0.0; 11]);
    samples.push(r);
    let mut r = record();
    r["houses"]["cusps"] = json!(vec![0.0; 13]);
    samples.push(r);
    let mut r = record();
    r["lat"] = json!(91.0);
    samples.push(r);
    let mut r = record();
    r["lon"] = json!(181.0);
    samples.push(r);
    let mut r = record();
    r["jd"] = json!(1e308);
    samples.push(r);
    for r in samples {
        invalid(&r.to_string(), &["--threshold-deg", "360"]);
    }
}

#[test]
fn metadata_count_mismatch_and_repeated_metadata_are_rejected() {
    let mut r = record();
    r["_meta"] = json!({"n":2});
    invalid(&r.to_string(), &["--threshold-deg", "360"]);
    r["_meta"] = json!({"n":1});
    invalid(&format!("{r}\n{r}\n"), &["--threshold-deg", "360"]);
    invalid(&format!("{}\n{r}\n", record()), &["--threshold-deg", "360"]);
    r["_meta"] = json!({"n":0});
    invalid(&r.to_string(), &["--threshold-deg", "360"]);
}

#[test]
fn valid_records_can_pass_and_actual_comparison_failures_still_fail() {
    let input = record().to_string();
    let out = run(&input, &["--threshold-deg", "360"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("RESULT      : PASS"));
    let out = run(&input, &["--threshold-deg", "0"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stdout).contains("RESULT      : FAIL"));
}

#[test]
fn complete_metadata_count_is_accepted() {
    let mut r = record();
    r["_meta"] = json!({"n":2});
    let out = run(
        &format!("{r}\n\n{}\n", record()),
        &["--threshold-deg", "360"],
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn expected_coverage_exclusions_are_visible_and_need_comparable_samples() {
    let mut old = record();
    old["jd"] = json!(2341972.5); // 1700-01-01, outside analytic Pluto coverage.
    invalid(&old.to_string(), &["--threshold-deg", "360"]);
    let out = run(
        &format!("{}\n{old}\n", record()),
        &["--threshold-deg", "360"],
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let report = String::from_utf8_lossy(&out.stdout);
    assert!(report.contains("pluto: 1 epochs outside calculation coverage"));
    assert!(report.contains("every evaluated gated sample"));
}

#[test]
fn house_latitude_exclusions_are_visible() {
    let mut r = record();
    r["lat"] = json!(65.0);
    let out = run(&r.to_string(), &["--threshold-deg", "360"]);
    assert_eq!(out.status.code(), Some(0));
    assert!(
        String::from_utf8_lossy(&out.stdout)
            .contains("houses: 1 charts outside |latitude| <= 60 deg")
    );
}

#[test]
fn unreadable_input_is_an_error() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_xalen-validation"))
        .arg(env!("CARGO_MANIFEST_DIR"))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // On Unix a directory can be opened as a file, but reads fail. The old
    // runner retried this error forever; bound the regression test itself.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while child.try_wait().unwrap().is_none() {
        if std::time::Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("runner did not terminate after an input read error");
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(!String::from_utf8_lossy(&out.stdout).contains("RESULT      : PASS"));
}
