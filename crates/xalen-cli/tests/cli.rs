//! End-to-end tests: they run the built `xalen` binary and compare what it
//! prints with what the library returns for the same instant.

use std::process::Command;

use serde_json::Value;
use xalen_ephemeris::ayanamsa::Ayanamsa;
use xalen_ephemeris::coords::RAD_TO_DEG;
use xalen_ephemeris::ephem::compat::{SE_SIDM_LAHIRI, swe_houses_ex, swe_set_sid_mode};
use xalen_ephemeris::ephem::{Almanac, Body};
use xalen_ephemeris::time::{CalendarSystem, DeltaTModel, JdUT1, calendar_to_jd};
use xalen_ephemeris::vedic::nakshatra::Nakshatra;
use xalen_ephemeris::vedic::panchang::compute_panchang;
use xalen_ephemeris::vedic::rashi::Rashi;

struct Output {
    code: i32,
    stdout: String,
    stderr: String,
}

fn xalen(args: &[&str]) -> Output {
    let out = Command::new(env!("CARGO_BIN_EXE_xalen"))
        .args(args)
        .output()
        .expect("the xalen binary starts");
    Output {
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

/// Run a command that must succeed and parse its JSON.
fn json_of(args: &[&str]) -> Value {
    let out = xalen(args);
    assert_eq!(out.code, 0, "args {args:?} failed: {}", out.stderr);
    serde_json::from_str(&out.stdout).expect("stdout is valid JSON")
}

fn find<'a>(items: &'a Value, key: &str, id: &str) -> &'a Value {
    items
        .as_array()
        .expect("an array")
        .iter()
        .find(|item| item[key] == id)
        .unwrap_or_else(|| panic!("no item with {key} = {id}"))
}

fn num(value: &Value) -> f64 {
    value.as_f64().expect("a number")
}

fn angle_gap(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
}

#[test]
fn version_prints_the_crate_version() {
    let out = xalen(&["--version"]);
    assert_eq!(out.code, 0);
    assert!(
        out.stdout.contains(env!("CARGO_PKG_VERSION")),
        "got {:?}",
        out.stdout
    );
}

#[test]
fn help_exits_zero_and_every_subcommand_has_an_example() {
    let out = xalen(&["--help"]);
    assert_eq!(out.code, 0);
    assert!(out.stdout.contains("Example:"));
    for command in [
        "positions",
        "chart",
        "panchang",
        "dasha",
        "star",
        "eclipses",
        "rise-set",
        "aspects",
        "convert",
        "map",
        "synastry",
        "composite",
        "davison",
        "relocate",
    ] {
        assert!(
            out.stdout.contains(command),
            "{command} missing from --help"
        );
        let sub = xalen(&[command, "--help"]);
        assert_eq!(sub.code, 0, "{command} --help");
        assert!(
            sub.stdout.contains("Example:"),
            "{command} --help has no example"
        );
    }
    let positions = xalen(&["positions", "--help"]);
    assert!(positions.stdout.contains("Lahiri"));
    let chart = xalen(&["chart", "--help"]);
    assert!(chart.stdout.contains("Placidus") && chart.stdout.contains("WholeSign"));
}

#[test]
fn convert_reports_j2000() {
    let json = json_of(&["convert", "--date", "2000-01-01T12:00:00Z", "--json"]);
    assert!((num(&json["jd_ut"]) - 2_451_545.0).abs() < 1e-9);
    let tt_minus_ut = (num(&json["jd_tt"]) - num(&json["jd_ut"])) * 86_400.0;
    assert!((tt_minus_ut - num(&json["delta_t_seconds"])).abs() < 1e-3);
    assert!((60.0..70.0).contains(&tt_minus_ut), "delta T {tt_minus_ut}");
    // GMST at J2000.0 is 280.46061837 degrees.
    assert!((num(&json["gmst_deg"]) - 280.460_618_37).abs() < 1e-6);
    assert!(angle_gap(num(&json["gast_deg"]), num(&json["gmst_deg"])) < 0.01);
    let ayanamsas = json["ayanamsas"].as_array().expect("an array");
    assert_eq!(ayanamsas.len(), Ayanamsa::all_named().len());
    let lahiri = find(&json["ayanamsas"], "id", "Lahiri");
    assert!((23.0..24.5).contains(&num(&lahiri["value_deg"])));
}

#[test]
fn jd_input_matches_date_input() {
    let by_jd = json_of(&["convert", "--jd", "2451545.0", "--json"]);
    let by_date = json_of(&["convert", "--date", "2000-01-01T12:00:00Z", "--json"]);
    assert_eq!(by_jd["jd_ut"], by_date["jd_ut"]);
    assert_eq!(by_jd["gmst_deg"], by_date["gmst_deg"]);
}

#[test]
fn zone_offset_in_date_is_applied() {
    let utc = json_of(&["convert", "--date", "2026-11-07T00:00:00Z", "--json"]);
    let ist = json_of(&["convert", "--date", "2026-11-07T05:30:00+05:30", "--json"]);
    assert!((num(&utc["jd_ut"]) - num(&ist["jd_ut"])).abs() < 1e-9);
}

#[test]
fn positions_match_the_library() {
    let jd = calendar_to_jd(2026, 11, 7, 0.0, CalendarSystem::ProlepticGregorian);
    let almanac = Almanac::default_vedic();
    let json = json_of(&["positions", "--date", "2026-11-07T00:00:00Z", "--json"]);
    assert_eq!(json["zodiac"], "tropical");
    for (id, body) in [
        ("sun", Body::Sun),
        ("moon", Body::Moon),
        ("mercury", Body::Mercury),
        ("mars", Body::Mars),
        ("saturn", Body::Saturn),
        ("mean-node", Body::MeanNode),
    ] {
        let expected = almanac
            .geocentric_longitude_deg(body, jd)
            .expect("computes");
        let shown = num(&find(&json["bodies"], "id", id)["longitude"]);
        assert!(
            (shown - expected).abs() < 1e-6,
            "{id}: cli {shown} vs library {expected}"
        );
    }
    let sun = find(&json["bodies"], "id", "sun");
    let moon = find(&json["bodies"], "id", "moon");
    assert!((0.9..1.1).contains(&num(&sun["speed_deg_per_day"])));
    assert!((11.0..15.5).contains(&num(&moon["speed_deg_per_day"])));
    let ketu = num(&find(&json["bodies"], "id", "ketu")["longitude"]);
    let rahu = num(&find(&json["bodies"], "id", "mean-node")["longitude"]);
    assert!((angle_gap(ketu, rahu) - 180.0).abs() < 1e-9);
}

#[test]
fn positions_follow_the_readme_quick_start() {
    // README Quick Start: 15 August 1947, 00:00 IST, Lahiri, Sun and Moon.
    let jd = calendar_to_jd(1947, 8, 15, 0.0 - 5.5, CalendarSystem::default());
    let almanac = Almanac::default_vedic();
    let aya = Ayanamsa::Lahiri.compute_deg(jd.0);
    let json = json_of(&[
        "positions",
        "--date",
        "1947-08-15T00:00:00+05:30",
        "--sidereal",
        "--bodies",
        "sun,moon",
        "--json",
    ]);
    assert_eq!(json["zodiac"], "sidereal");
    for (id, body) in [("sun", Body::Sun), ("moon", Body::Moon)] {
        let pos = almanac.geocentric_ecliptic(body, jd).expect("computes");
        let sid = (pos.longitude * RAD_TO_DEG - aya).rem_euclid(360.0);
        let item = find(&json["bodies"], "id", id);
        assert!((num(&item["longitude"]) - sid).abs() < 1e-6, "{id}");
        assert_eq!(item["sign"], Rashi::from_longitude_deg(sid).to_string());
        assert_eq!(
            item["nakshatra"],
            Nakshatra::from_longitude_deg(sid).to_string()
        );
        assert_eq!(item["pada"], Nakshatra::pada(sid));
    }
}

#[test]
fn positions_can_select_bodies_and_print_a_table() {
    let json = json_of(&[
        "positions",
        "--date",
        "2026-11-07",
        "--bodies",
        "sun,Moon",
        "--json",
    ]);
    assert_eq!(json["bodies"].as_array().expect("an array").len(), 2);
    let table = xalen(&["positions", "--date", "2026-11-07", "--bodies", "sun"]);
    assert_eq!(table.code, 0);
    assert!(table.stdout.contains("Sun") && table.stdout.contains("Longitude"));
    assert!(!table.stdout.trim_start().starts_with('{'));
}

#[test]
fn sidereal_accepts_a_named_ayanamsa() {
    let lahiri = json_of(&["positions", "--date", "2026-11-07", "--sidereal", "--json"]);
    let named = json_of(&[
        "positions",
        "--date",
        "2026-11-07",
        "--sidereal",
        "Lahiri",
        "--json",
    ]);
    assert_eq!(lahiri["bodies"], named["bodies"]);
    let kp = json_of(&[
        "positions",
        "--date",
        "2026-11-07",
        "--sidereal",
        "KPKrishnamurti",
        "--json",
    ]);
    assert_eq!(kp["ayanamsa"]["id"], "KPKrishnamurti");
    assert_ne!(kp["ayanamsa"]["value_deg"], lahiri["ayanamsa"]["value_deg"]);
}

#[test]
fn chart_for_new_delhi_has_ordered_cusps_and_ascendant_in_house_one() {
    let json = json_of(&[
        "chart",
        "--date",
        "2026-11-07T06:00:00Z",
        "--lat",
        "28.6139",
        "--lon",
        "77.2090",
        "--json",
    ]);
    assert_eq!(json["house_system"], "Placidus");
    let cusps: Vec<f64> = json["cusps"]
        .as_array()
        .expect("an array")
        .iter()
        .map(|c| num(&c["longitude"]))
        .collect();
    assert_eq!(cusps.len(), 12);
    let mut total = 0.0;
    for i in 0..12 {
        let step = (cusps[(i + 1) % 12] - cusps[i]).rem_euclid(360.0);
        assert!(step > 0.0 && step < 180.0, "cusp {i} step {step}");
        total += step;
    }
    assert!((total - 360.0).abs() < 1e-9, "cusps wrap once, got {total}");

    let asc = num(&json["ascendant"]["longitude"]);
    let into_house_one = (asc - cusps[0]).rem_euclid(360.0);
    let house_one = (cusps[1] - cusps[0]).rem_euclid(360.0);
    assert!(into_house_one < house_one, "ascendant outside house 1");
    let mc = num(&json["mc"]["longitude"]);
    assert!(angle_gap(mc, cusps[9]) < 1e-6, "MC is the tenth cusp");

    for body in json["bodies"].as_array().expect("an array") {
        let house = body["house"].as_u64().expect("a house number");
        assert!((1..=12).contains(&house));
    }
}

#[test]
fn chart_defaults_to_whole_sign_when_sidereal_and_accepts_place_names() {
    let json = json_of(&[
        "chart",
        "--date",
        "1990-03-15T06:30:00Z",
        "--place",
        "delhi",
        "--sidereal",
        "--json",
    ]);
    assert_eq!(json["house_system"], "WholeSign");
    let by_name = json_of(&[
        "chart",
        "--date",
        "1990-03-15T06:30:00Z",
        "--place",
        "Delhi",
        "--sidereal",
        "--house-system",
        "whole-sign",
        "--json",
    ]);
    assert_eq!(by_name["ascendant"], json["ascendant"]);
}

#[test]
fn eclipses_list_the_april_2024_total_solar_eclipse() {
    let json = json_of(&[
        "eclipses",
        "--from",
        "2024-04-01",
        "--to",
        "2024-04-30",
        "--json",
    ]);
    let list = json["eclipses"].as_array().expect("an array");
    let april_8 = list
        .iter()
        .find(|e| {
            e["kind"] == "solar"
                && e["maximum"]
                    .as_str()
                    .is_some_and(|d| d.starts_with("2024-04-08"))
        })
        .expect("the 8 April 2024 solar eclipse is listed");
    assert_eq!(april_8["type"], "Total");
    assert_eq!(json["count"], list.len());

    let lunar_only = json_of(&[
        "eclipses",
        "--from",
        "2024-01-01",
        "--to",
        "2024-12-31",
        "--type",
        "lunar",
        "--json",
    ]);
    assert!(
        lunar_only["eclipses"]
            .as_array()
            .expect("an array")
            .iter()
            .all(|e| e["kind"] == "lunar")
    );
}

#[test]
fn panchang_for_delhi_is_complete_and_ordered() {
    let json = json_of(&[
        "panchang",
        "--date",
        "2026-11-07T00:00:00Z",
        "--place",
        "delhi",
        "--json",
    ]);
    let tithi = num(&json["tithi"]["number"]);
    assert!((1.0..=30.0).contains(&tithi));
    assert!(json["tithi"]["ends_jd_ut"].as_f64() > json["tithi"]["starts_jd_ut"].as_f64());
    for limb in ["nakshatra", "yoga", "karana", "vara"] {
        assert!(json[limb]["name"].is_string(), "{limb}");
    }
    let rise = num(&json["sunrise_jd_ut"]);
    let set = num(&json["sunset_jd_ut"]);
    assert!(set > rise && set - rise < 0.6, "day length {}", set - rise);
}

#[test]
fn dasha_has_nine_contiguous_mahadashas_with_nine_antardashas_each() {
    let json = json_of(&["dasha", "--date", "1990-03-15T06:30:00Z", "--json"]);
    let periods = json["periods"].as_array().expect("an array");
    assert_eq!(periods.len(), 9);
    for pair in periods.windows(2) {
        assert!((num(&pair[0]["end_jd_ut"]) - num(&pair[1]["start_jd_ut"])).abs() < 1e-9);
    }
    for period in periods {
        let subs = period["sub_periods"].as_array().expect("an array");
        assert_eq!(subs.len(), 9);
        assert!((num(&subs[0]["start_jd_ut"]) - num(&period["start_jd_ut"])).abs() < 1e-9);
        assert!((num(&subs[8]["end_jd_ut"]) - num(&period["end_jd_ut"])).abs() < 1e-6);
    }
    let one = json_of(&[
        "dasha",
        "--date",
        "1990-03-15T06:30:00Z",
        "--levels",
        "1",
        "--json",
    ]);
    assert!(
        one["periods"][0]["sub_periods"]
            .as_array()
            .expect("an array")
            .is_empty()
    );
}

#[test]
fn star_spica_sits_near_twenty_four_degrees_of_libra() {
    let json = json_of(&["star", "Spica", "--date", "2026-11-07T00:00:00Z", "--json"]);
    let lon = num(&json["longitude"]);
    assert!((203.0..205.5).contains(&lon), "Spica at {lon}");
    assert_eq!(json["sign"], "Libra");
    assert!(num(&json["latitude"]) < 0.0);
    assert!((num(&json["ra_hours"]) - 13.42).abs() < 0.1);
    assert!((num(&json["dec_deg"]) + 11.2).abs() < 0.5);
}

#[test]
fn unknown_star_is_a_usage_error_with_advice() {
    let out = xalen(&["star", "Notastar", "--date", "2026-11-07"]);
    assert_eq!(out.code, 2);
    assert!(out.stdout.is_empty());
    assert!(out.stderr.contains("unknown star"));
    assert_eq!(out.stderr.trim().lines().count(), 1);
}

#[test]
fn rise_set_orders_rise_transit_set() {
    let json = json_of(&[
        "rise-set",
        "--date",
        "2026-11-07T00:00:00Z",
        "--place",
        "delhi",
        "--body",
        "sun",
        "--json",
    ]);
    let rise = num(&json["rise_jd_ut"]);
    let transit = num(&json["transit_jd_ut"]);
    let set = num(&json["set_jd_ut"]);
    assert!(rise < transit && transit < set);
}

#[test]
fn rise_set_prints_in_the_requested_zone() {
    let out = xalen(&[
        "rise-set",
        "--date",
        "2026-11-07T00:00:00Z",
        "--lat",
        "28.6139",
        "--lon",
        "77.2090",
        "--body",
        "moon",
        "--tz",
        "+05:30",
    ]);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert!(out.stdout.contains("+05:30"));
}

#[test]
fn aspects_respect_default_and_custom_orbs() {
    let json = json_of(&["aspects", "--date", "2026-11-07T00:00:00Z", "--json"]);
    for aspect in json["aspects"].as_array().expect("an array") {
        assert!(num(&aspect["orb_deg"]) <= num(&aspect["default_orb_deg"]) + 1e-9);
        let gap = (num(&aspect["separation_deg"]) - num(&aspect["angle_deg"])).abs();
        assert!((gap - num(&aspect["orb_deg"])).abs() < 1e-6);
    }
    let tight = json_of(&[
        "aspects",
        "--date",
        "2026-11-07T00:00:00Z",
        "--orb",
        "1",
        "--json",
    ]);
    let tight_list = tight["aspects"].as_array().expect("an array");
    assert!(tight_list.len() <= json["aspects"].as_array().expect("an array").len());
    for aspect in tight_list {
        assert!(num(&aspect["orb_deg"]) <= 1.0 + 1e-9);
    }
}

#[test]
fn output_is_deterministic() {
    let args = ["positions", "--date", "2026-11-07T00:00:00Z", "--json"];
    assert_eq!(xalen(&args).stdout, xalen(&args).stdout);
}

#[test]
fn bad_dates_exit_two_with_one_line_on_stderr() {
    for bad in [
        "not-a-date",
        "2026-02-30",
        "2026-13-01",
        "2026-11-07T25:00:00Z",
        "2026-11",
    ] {
        let out = xalen(&["positions", "--date", bad]);
        assert_eq!(out.code, 2, "{bad}");
        assert!(out.stdout.is_empty(), "{bad}");
        assert!(out.stderr.contains("bad date"), "{bad}: {}", out.stderr);
        assert_eq!(out.stderr.trim().lines().count(), 1, "{bad}");
    }
}

#[test]
fn unknown_body_exits_two() {
    let out = xalen(&[
        "positions",
        "--date",
        "2026-11-07",
        "--bodies",
        "sun,vulcan",
    ]);
    assert_eq!(out.code, 2);
    assert!(out.stderr.contains("unknown body"));
    let out = xalen(&[
        "rise-set",
        "--date",
        "2026-11-07",
        "--place",
        "delhi",
        "--body",
        "vulcan",
    ]);
    assert_eq!(out.code, 2);
    assert!(out.stderr.contains("unknown body"));
}

#[test]
fn missing_inputs_exit_two() {
    let no_place = xalen(&["chart", "--date", "2026-11-07"]);
    assert_eq!(no_place.code, 2);
    assert!(no_place.stderr.contains("--lat"));
    let no_lon = xalen(&["chart", "--date", "2026-11-07", "--lat", "28.6"]);
    assert_eq!(no_lon.code, 2);
    assert!(no_lon.stderr.contains("--lon"));
    let no_date = xalen(&["positions"]);
    assert_eq!(no_date.code, 2);
    assert!(no_date.stderr.contains("--date"));
    let bad_place = xalen(&["chart", "--date", "2026-11-07", "--place", "Atlantis"]);
    assert_eq!(bad_place.code, 2);
    let bad_lat = xalen(&[
        "chart",
        "--date",
        "2026-11-07",
        "--lat",
        "95",
        "--lon",
        "10",
    ]);
    assert_eq!(bad_lat.code, 2);
    let bad_range = xalen(&["eclipses", "--from", "2024-05-01", "--to", "2024-04-01"]);
    assert_eq!(bad_range.code, 2);
    let bad_aya = xalen(&["positions", "--date", "2026-11-07", "--sidereal", "nope"]);
    assert_eq!(bad_aya.code, 2);
    let bad_house = xalen(&[
        "chart",
        "--date",
        "2026-11-07",
        "--lat",
        "28",
        "--lon",
        "77",
        "--house-system",
        "nope",
    ]);
    assert_eq!(bad_house.code, 2);
    let bad_tz = xalen(&["positions", "--date", "2026-11-07", "--tz", "Mars/Olympus"]);
    assert_eq!(bad_tz.code, 2);
}

/// The house (1-12) that a longitude falls in, given 12 cusp longitudes.
fn house_in(cusps: &[f64], longitude: f64) -> u64 {
    for i in 0..12 {
        let start = cusps[i];
        let width = (cusps[(i + 1) % 12] - start).rem_euclid(360.0);
        if (longitude - start).rem_euclid(360.0) < width {
            return i as u64 + 1;
        }
    }
    panic!("longitude {longitude} is in no house");
}

#[test]
fn chart_matches_the_library_houses_for_new_delhi() {
    let jd = calendar_to_jd(2026, 11, 7, 0.0, CalendarSystem::ProlepticGregorian).0;
    swe_set_sid_mode(SE_SIDM_LAHIRI, 0.0, 0.0);
    for (sidereal, system_arg, letter) in [
        (true, "WholeSign", 'W'),
        (true, "Equal", 'A'),
        (false, "Placidus", 'P'),
    ] {
        let library = swe_houses_ex(jd, 28.6139, 77.2090, letter, sidereal).expect("computes");
        let mut args = vec![
            "chart",
            "--date",
            "2026-11-07T00:00:00Z",
            "--lat",
            "28.6139",
            "--lon",
            "77.2090",
            "--house-system",
            system_arg,
            "--json",
        ];
        if sidereal {
            args.push("--sidereal");
        }
        let json = json_of(&args);
        let cusps: Vec<f64> = json["cusps"]
            .as_array()
            .expect("an array")
            .iter()
            .map(|c| num(&c["longitude"]))
            .collect();
        for i in 0..12 {
            assert!(
                angle_gap(cusps[i], library.cusps[i]) < 1e-6,
                "{system_arg} sidereal={sidereal} cusp {}: cli {} vs library {}",
                i + 1,
                cusps[i],
                library.cusps[i]
            );
        }
        assert!(angle_gap(num(&json["ascendant"]["longitude"]), library.ascendant) < 1e-6);
        assert!(angle_gap(num(&json["mc"]["longitude"]), library.mc) < 1e-6);
        for body in json["bodies"].as_array().expect("an array") {
            let expected = house_in(&library.cusps, num(&body["longitude"]));
            assert_eq!(
                body["house"].as_u64(),
                Some(expected),
                "{} in {system_arg}",
                body["id"]
            );
        }
        if system_arg == "WholeSign" {
            for cusp in json["cusps"].as_array().expect("an array") {
                assert_eq!(num(&cusp["degree_in_sign"]), 0.0, "cusp shown at {cusp}");
            }
            for cusp in &cusps {
                let signs = cusp / 30.0;
                assert!(
                    (signs - signs.round()).abs() * 30.0 < 1e-6,
                    "{cusp} is not a sign boundary"
                );
            }
            let asc_sign = (num(&json["ascendant"]["longitude"]) / 30.0).floor();
            for body in json["bodies"].as_array().expect("an array") {
                let in_asc_sign = (num(&body["longitude"]) / 30.0).floor() == asc_sign;
                assert_eq!(body["house"] == 1, in_asc_sign, "{}", body["id"]);
            }
        }
    }
}

#[test]
fn panchang_matches_the_library() {
    let jd = calendar_to_jd(2026, 11, 7, 0.0, CalendarSystem::ProlepticGregorian);
    let jd_tt = JdUT1(jd.0)
        .to_tt(&DeltaTModel::StephensonMorrisonHohenkerk2016)
        .0;
    let aya = Ayanamsa::Lahiri.compute_deg(jd_tt);
    let almanac = Almanac::default_vedic();
    let sun = almanac
        .sidereal_longitude_deg(Body::Sun, jd, aya)
        .expect("computes");
    let moon = almanac
        .sidereal_longitude_deg(Body::Moon, jd, aya)
        .expect("computes");
    let expected = compute_panchang(sun, moon, jd.0);
    let json = json_of(&[
        "panchang",
        "--date",
        "2026-11-07T00:00:00Z",
        "--place",
        "delhi",
        "--json",
    ]);
    assert_eq!(
        json["tithi"]["number"].as_u64(),
        Some(u64::from(expected.tithi.number))
    );
    assert_eq!(json["nakshatra"]["name"], expected.nakshatra.to_string());
    assert_eq!(
        json["yoga"]["number"].as_u64(),
        Some(u64::from(expected.yoga.number))
    );
    assert_eq!(json["karana"]["name"], expected.karana.name());
    assert!((num(&json["moon_longitude"]) - moon).abs() < 1e-9);
}

#[test]
fn rise_set_matches_the_library_within_a_second() {
    let jd = calendar_to_jd(2026, 11, 7, 0.0, CalendarSystem::ProlepticGregorian);
    let almanac = Almanac::default_vedic();
    let json = json_of(&[
        "rise-set",
        "--date",
        "2026-11-07T00:00:00Z",
        "--lat",
        "28.6139",
        "--lon",
        "77.2090",
        "--body",
        "sun",
        "--json",
    ]);
    let expected = almanac
        .rise_transit_set(Body::Sun, jd, 28.6139, 77.2090, 0.0)
        .expect("computes");
    let second = 1.0 / 86_400.0;
    for (key, value) in [
        ("rise_jd_ut", expected.rise),
        ("transit_jd_ut", expected.transit),
        ("set_jd_ut", expected.set),
    ] {
        let want = value.expect("the Sun rises, transits and sets in Delhi").0;
        assert!((num(&json[key]) - want).abs() < second, "{key}");
    }
}

#[test]
fn absurd_dates_and_julian_days_exit_two() {
    for args in [
        vec!["convert", "--jd", "99999999"],
        vec!["convert", "--jd", "-5000000"],
        vec!["convert", "--date", "20000-01-01"],
        vec!["convert", "--date", "-14000-01-01"],
    ] {
        let out = xalen(&args);
        assert_eq!(out.code, 2, "{args:?}");
        assert!(out.stdout.is_empty(), "{args:?}");
        assert_eq!(out.stderr.trim().lines().count(), 1, "{args:?}");
    }
    assert_eq!(xalen(&["convert", "--date", "-4712-01-01"]).code, 0);
}

// ---------------------------------------------------------------------------
// map, synastry, composite, davison, relocate
// ---------------------------------------------------------------------------

use xalen_ephemeris::maps::{
    AstroMap, MapOptions, Paran, ParanOptions, astro_map, parans, parans_near,
};

fn with_flags(base: &[&str], more: &[&str]) -> Vec<String> {
    base.iter().chain(more).map(|s| (*s).to_string()).collect()
}

fn json_of_owned(args: &[String]) -> Value {
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    json_of(&refs)
}

/// The angle (ascendant, mc, ...) of a chart-style JSON as a longitude.
fn angle_of(json: &Value, key: &str) -> f64 {
    num(&json[key]["longitude"])
}

#[test]
fn map_json_matches_the_library() {
    let jd = calendar_to_jd(2024, 6, 21, 0.0, CalendarSystem::ProlepticGregorian);
    let almanac = Almanac::default_vedic();
    let expected = astro_map(
        &almanac,
        jd,
        &[Body::Sun, Body::Moon, Body::Jupiter],
        &MapOptions::default(),
    )
    .expect("the library maps the bodies");
    let json = json_of(&[
        "map",
        "--date",
        "2024-06-21T00:00:00Z",
        "--bodies",
        "sun,moon,jupiter",
        "--json",
    ]);
    assert_eq!(json["command"], "map");
    assert!((num(&json["gast_deg"]) - expected.gast_deg).abs() < 1e-9);
    let bodies = json["bodies"].as_array().expect("an array");
    assert_eq!(bodies.len(), expected.bodies.len());
    for (item, want) in bodies.iter().zip(&expected.bodies) {
        assert_eq!(item["body"], format!("{:?}", want.body));
        assert!((num(&item["ra_deg"]) - want.ra_deg).abs() < 1e-9);
        assert!((num(&item["dec_deg"]) - want.dec_deg).abs() < 1e-9);
        let lines = item["lines"].as_array().expect("lines");
        assert_eq!(lines[0]["kind"], "Mc");
        let mc = num(&lines[0]["segments"][0][0][1]);
        assert!(
            (mc - want.zenith.1).abs() < 1e-9,
            "MC {mc} vs {}",
            want.zenith.1
        );
        assert!((num(&item["zenith"][0]) - want.zenith.0).abs() < 1e-9);
    }
}

#[test]
fn map_table_lists_the_meridians_and_the_horizon_at_a_latitude() {
    let out = xalen(&[
        "map",
        "--date",
        "2024-06-21T00:00:00Z",
        "--bodies",
        "sun,moon",
        "--lat",
        "40",
    ]);
    assert_eq!(out.code, 0, "{}", out.stderr);
    for word in [
        "Sun",
        "Moon",
        "MC lon",
        "IC lon",
        "ASC lon",
        "DSC lon",
        "Zenith lat",
    ] {
        assert!(out.stdout.contains(word), "{word} missing:\n{}", out.stdout);
    }
    // The Sun is circumpolar at 80 north around the June solstice.
    let polar = xalen(&[
        "map",
        "--date",
        "2024-06-21T00:00:00Z",
        "--bodies",
        "sun",
        "--lat",
        "80",
    ]);
    assert_eq!(polar.code, 0, "{}", polar.stderr);
    assert!(polar.stdout.contains("always above the horizon"));
}

#[test]
fn map_horizon_at_a_latitude_agrees_with_the_json_lines() {
    let json = json_of(&[
        "map",
        "--date",
        "2024-06-21T00:00:00Z",
        "--bodies",
        "sun",
        "--lat",
        "40",
        "--lat-step",
        "0.5",
        "--json",
    ]);
    let asc = &json["bodies"][0]["lines"][2];
    assert_eq!(asc["kind"], "Asc");
    let want = num(&json["at_latitude"]["bodies"][0]["asc_longitude"]);
    let found = asc["segments"]
        .as_array()
        .expect("segments")
        .iter()
        .flat_map(|s| s.as_array().expect("points").iter())
        .find(|p| (num(&p[0]) - 40.0).abs() < 1e-9)
        .expect("a vertex at latitude 40");
    assert!((num(&found[1]) - want).abs() < 1e-6, "{found} vs {want}");
}

#[test]
fn map_geojson_is_a_valid_feature_collection() {
    let out = xalen(&[
        "map",
        "--date",
        "2024-06-21T00:00:00Z",
        "--bodies",
        "sun,moon",
        "--geojson",
    ]);
    assert_eq!(out.code, 0, "{}", out.stderr);
    let json: Value = serde_json::from_str(&out.stdout).expect("stdout is valid JSON");
    assert_eq!(json["type"], "FeatureCollection");
    let features = json["features"].as_array().expect("features");
    assert!(!features.is_empty());
    let (mut points, mut lines) = (0, 0);
    for feature in features {
        assert_eq!(feature["type"], "Feature");
        for key in ["body", "kind"] {
            assert!(feature["properties"][key].is_string(), "{key} in {feature}");
        }
        let kind = feature["geometry"]["type"].as_str().expect("a type");
        let coordinates = &feature["geometry"]["coordinates"];
        let positions: Vec<&Value> = match kind {
            "Point" => {
                points += 1;
                vec![coordinates]
            }
            "LineString" => {
                lines += 1;
                let all: Vec<&Value> = coordinates.as_array().expect("positions").iter().collect();
                assert!(all.len() >= 2, "a LineString needs two positions");
                all
            }
            other => panic!("unexpected geometry {other}"),
        };
        for position in positions {
            let lon = num(&position[0]);
            let lat = num(&position[1]);
            assert!((-180.0..=180.0).contains(&lon), "lon {lon}");
            assert!((-90.0..=90.0).contains(&lat), "lat {lat}");
        }
    }
    assert_eq!(points, 2, "one zenith per body");
    assert!(
        lines >= 8,
        "at least the MC and IC line of each body: {lines}"
    );
}

#[test]
fn synastry_of_a_chart_with_itself_is_a_conjunction_on_the_diagonal() {
    let same = [
        "synastry",
        "--a-date",
        "1990-03-15T06:30:00Z",
        "--a-place",
        "delhi",
        "--b-date",
        "1990-03-15T06:30:00Z",
        "--b-place",
        "delhi",
        "--angles",
    ];
    let json = json_of_owned(&with_flags(&same, &["--json"]));
    assert_eq!(json["command"], "synastry");
    let n = json["rows"].as_array().expect("rows").len();
    assert_eq!(n, json["cols"].as_array().expect("cols").len());
    assert_eq!(n, 15, "eleven bodies and four angles");
    let cells = json["cells"].as_array().expect("cells");
    assert_eq!(cells.len(), n * n);
    for i in 0..n {
        let cell = &cells[i * n + i];
        assert_eq!(cell["aspect_type"], "Conjunction", "diagonal {i}");
        assert!(num(&cell["orb_deg"]) < 1e-12, "diagonal {i}");
    }
    assert_eq!(json["b_in_a"].as_array().expect("overlay").len(), n);
    assert_eq!(json["a_in_b"].as_array().expect("overlay").len(), n);
    for (a, b) in json["a_in_b"]
        .as_array()
        .expect("overlay")
        .iter()
        .zip(json["b_in_a"].as_array().expect("overlay"))
    {
        assert_eq!(a["house"], b["house"], "identical charts share houses");
    }

    let table = xalen(&same);
    assert_eq!(table.code, 0, "{}", table.stderr);
    for word in [
        "Su", "Mo", "Me", "Ve", "Ma", "Ju", "Sa", "Ur", "Ne", "Pl", "Ra", "Asc", "MC",
    ] {
        assert!(
            table.stdout.contains(word),
            "{word} missing:\n{}",
            table.stdout
        );
    }
    assert!(table.stdout.contains("Cnj 0.00"));
    assert!(table.stdout.contains("B's points in A's houses"));
    assert!(table.stdout.contains("A's points in B's houses"));
}

#[test]
fn synastry_options_change_the_grid() {
    let narrow = json_of_owned(&with_flags(
        &["synastry"],
        &[
            "--a-date",
            "1990-03-15T06:30:00Z",
            "--a-place",
            "delhi",
            "--b-date",
            "1992-08-02T14:10:00Z",
            "--b-place",
            "mumbai",
            "--aspects",
            "conjunction",
            "--bodies",
            "sun,moon",
            "--json",
        ],
    ));
    assert_eq!(narrow["rows"].as_array().expect("rows").len(), 2);
    assert_eq!(narrow["aspect_types"], serde_json::json!(["Conjunction"]));
    for cell in narrow["cells"].as_array().expect("cells") {
        assert!(cell.is_null() || cell["aspect_type"] == "Conjunction");
    }
}

#[test]
fn composite_of_identical_births_equals_the_chart() {
    let chart = json_of(&[
        "chart",
        "--date",
        "1990-03-15T06:30:00Z",
        "--place",
        "delhi",
        "--json",
    ]);
    let composite = json_of(&[
        "composite",
        "--a-date",
        "1990-03-15T06:30:00Z",
        "--a-place",
        "delhi",
        "--b-date",
        "1990-03-15T06:30:00Z",
        "--b-place",
        "delhi",
        "--json",
    ]);
    assert_eq!(composite["command"], "composite");
    let bodies = composite["bodies"].as_array().expect("bodies");
    assert_eq!(bodies.len(), 11);
    for body in bodies {
        let id = body["id"].as_str().expect("an id");
        let want = find(&chart["bodies"], "id", id);
        let gap = angle_gap(num(&body["longitude"]), num(&want["longitude"]));
        assert!(gap < 1e-6, "{id} differs by {gap}");
    }
    for key in ["ascendant", "mc"] {
        let gap = angle_gap(angle_of(&composite, key), angle_of(&chart, key));
        assert!(gap < 1e-6, "{key} differs by {gap}");
    }
}

#[test]
fn composite_midpoint_ascendant_needs_equal_or_whole_sign() {
    let two_births = [
        "composite",
        "--a-date",
        "1990-03-15T06:30:00Z",
        "--a-place",
        "delhi",
        "--b-date",
        "1992-08-02T14:10:00Z",
        "--b-place",
        "mumbai",
        "--ascendant",
        "midpoint",
    ];
    let ok = json_of_owned(&with_flags(&two_births, &["--json"]));
    assert_eq!(ok["ascendant_method"], "midpoint");
    assert_eq!(ok["house_system"], "Equal");
    let out = xalen(
        &two_births
            .iter()
            .copied()
            .chain(["--house-system", "placidus"])
            .collect::<Vec<_>>(),
    );
    assert_eq!(out.code, 2);
    assert_eq!(out.stderr.trim().lines().count(), 1);
}

#[test]
fn davison_of_identical_births_equals_the_chart() {
    let chart = json_of(&[
        "chart",
        "--date",
        "1990-03-15T06:30:00Z",
        "--place",
        "delhi",
        "--json",
    ]);
    for space in ["arithmetic", "great-circle"] {
        let davison = json_of(&[
            "davison",
            "--a-date",
            "1990-03-15T06:30:00Z",
            "--a-place",
            "delhi",
            "--b-date",
            "1990-03-15T06:30:00Z",
            "--b-place",
            "delhi",
            "--space",
            space,
            "--json",
        ]);
        assert_eq!(davison["command"], "davison");
        assert_eq!(davison["space"], space);
        for body in davison["bodies"].as_array().expect("bodies") {
            let id = body["id"].as_str().expect("an id");
            let want = find(&chart["bodies"], "id", id);
            let gap = angle_gap(num(&body["longitude"]), num(&want["longitude"]));
            assert!(gap < 1e-6, "{space} {id} differs by {gap}");
        }
        for key in ["ascendant", "mc"] {
            let gap = angle_gap(angle_of(&davison, key), angle_of(&chart, key));
            assert!(gap < 1e-6, "{space} {key} differs by {gap}");
        }
    }
}

#[test]
fn davison_reports_the_mean_date_and_place() {
    let davison = json_of(&[
        "davison",
        "--a-date",
        "1990-03-15T06:30:00Z",
        "--a-place",
        "delhi",
        "--b-date",
        "1992-08-02T14:10:00Z",
        "--b-place",
        "mumbai",
        "--json",
    ]);
    let a = num(&davison["a"]["jd_ut"]);
    let b = num(&davison["b"]["jd_ut"]);
    assert!((num(&davison["jd_ut"]) - (a + b) / 2.0).abs() < 1e-9);
    let lat = num(&davison["place"]["latitude"]);
    let want =
        (num(&davison["a"]["place"]["latitude"]) + num(&davison["b"]["place"]["latitude"])) / 2.0;
    assert!((lat - want).abs() < 1e-9);
}

#[test]
fn relocate_keeps_the_bodies_and_changes_the_houses() {
    let natal = json_of(&[
        "chart",
        "--date",
        "1990-03-15T06:30:00Z",
        "--place",
        "delhi",
        "--json",
    ]);
    let london = json_of(&[
        "chart",
        "--date",
        "1990-03-15T06:30:00Z",
        "--place",
        "london",
        "--json",
    ]);
    let moved = json_of(&[
        "relocate",
        "--date",
        "1990-03-15T06:30:00Z",
        "--place",
        "delhi",
        "--to-place",
        "london",
        "--json",
    ]);
    assert_eq!(moved["command"], "relocate");
    for body in moved["bodies"].as_array().expect("bodies") {
        let id = body["id"].as_str().expect("an id");
        let want = find(&natal["bodies"], "id", id);
        assert!(
            (num(&body["longitude"]) - num(&want["longitude"])).abs() < 1e-9,
            "{id} moved"
        );
    }
    let natal_asc = angle_of(&natal, "ascendant");
    let moved_asc = angle_of(&moved, "ascendant");
    assert!(
        angle_gap(natal_asc, moved_asc) > 1.0,
        "the ascendant must change"
    );
    assert!(angle_gap(moved_asc, angle_of(&london, "ascendant")) < 1e-6);
    assert!(angle_gap(num(&moved["natal_ascendant"]["longitude"]), natal_asc) < 1e-6);

    // The same place changes nothing.
    let same = json_of(&[
        "relocate",
        "--date",
        "1990-03-15T06:30:00Z",
        "--place",
        "delhi",
        "--to-place",
        "delhi",
        "--json",
    ]);
    assert!(angle_gap(angle_of(&same, "ascendant"), natal_asc) < 1e-9);
}

#[test]
fn sidereal_only_converts_the_printed_longitudes() {
    let args = [
        "relocate",
        "--date",
        "1990-03-15T06:30:00Z",
        "--place",
        "delhi",
        "--to-place",
        "london",
        "--house-system",
        "equal",
    ];
    let tropical = json_of_owned(&with_flags(&args, &["--json"]));
    let sidereal = json_of_owned(&with_flags(&args, &["--sidereal", "--json"]));
    assert_eq!(sidereal["zodiac"], "sidereal");
    let shift = num(&sidereal["ayanamsa"]["value_deg"]);
    assert!((20.0..26.0).contains(&shift), "ayanamsa {shift}");
    for body in sidereal["bodies"].as_array().expect("bodies") {
        let id = body["id"].as_str().expect("an id");
        let want = find(&tropical["bodies"], "id", id);
        let gap = angle_gap(num(&body["longitude"]) + shift, num(&want["longitude"]));
        assert!(gap < 1e-6, "{id} differs by {gap}");
        assert_eq!(body["house"], want["house"], "{id} house");
    }
    let gap = angle_gap(
        angle_of(&sidereal, "ascendant") + shift,
        angle_of(&tropical, "ascendant"),
    );
    assert!(gap < 1e-6, "ascendant differs by {gap}");
    // Synastry has no zodiac option.
    let out = xalen(&[
        "synastry",
        "--a-date",
        "1990-03-15",
        "--a-place",
        "delhi",
        "--b-date",
        "1992-08-02",
        "--b-place",
        "mumbai",
        "--sidereal",
    ]);
    assert!(out.stderr.contains("--sidereal"), "{}", out.stderr);
    assert_eq!(out.code, 2);
}

#[test]
fn the_relationship_and_map_commands_exit_two_on_bad_input() {
    let cases: Vec<Vec<&str>> = vec![
        // Missing --b-date.
        vec![
            "synastry",
            "--a-date",
            "1990-03-15",
            "--a-place",
            "delhi",
            "--b-place",
            "mumbai",
        ],
        // Missing the second birthplace.
        vec![
            "composite",
            "--a-date",
            "1990-03-15",
            "--a-place",
            "delhi",
            "--b-date",
            "1992-08-02",
        ],
        // Bad --ascendant and --space values.
        vec![
            "composite",
            "--a-date",
            "1990-03-15",
            "--a-place",
            "delhi",
            "--b-date",
            "1992-08-02",
            "--b-place",
            "mumbai",
            "--ascendant",
            "sideways",
        ],
        vec![
            "davison",
            "--a-date",
            "1990-03-15",
            "--a-place",
            "delhi",
            "--b-date",
            "1992-08-02",
            "--b-place",
            "mumbai",
            "--space",
            "flat",
        ],
        // Both --a-date and --a-jd.
        vec![
            "synastry",
            "--a-date",
            "1990-03-15",
            "--a-jd",
            "2451545",
            "--a-place",
            "delhi",
            "--b-date",
            "1992-08-02",
            "--b-place",
            "mumbai",
        ],
        // A place and coordinates together.
        vec![
            "synastry",
            "--a-date",
            "1990-03-15",
            "--a-place",
            "delhi",
            "--a-lat",
            "10",
            "--b-date",
            "1992-08-02",
            "--b-place",
            "mumbai",
        ],
        // Unknown aspect, bad orb multiplier, unknown house system, ketu.
        vec![
            "synastry",
            "--a-date",
            "1990-03-15",
            "--a-place",
            "delhi",
            "--b-date",
            "1992-08-02",
            "--b-place",
            "mumbai",
            "--aspects",
            "blue",
        ],
        vec![
            "synastry",
            "--a-date",
            "1990-03-15",
            "--a-place",
            "delhi",
            "--b-date",
            "1992-08-02",
            "--b-place",
            "mumbai",
            "--orb-mult",
            "0",
        ],
        vec![
            "synastry",
            "--a-date",
            "1990-03-15",
            "--a-place",
            "delhi",
            "--b-date",
            "1992-08-02",
            "--b-place",
            "mumbai",
            "--house-system",
            "round",
        ],
        vec![
            "composite",
            "--a-date",
            "1990-03-15",
            "--a-place",
            "delhi",
            "--b-date",
            "1992-08-02",
            "--b-place",
            "mumbai",
            "--bodies",
            "ketu",
        ],
        // Bad birth date and unknown city for B.
        vec![
            "synastry",
            "--a-date",
            "nope",
            "--a-place",
            "delhi",
            "--b-date",
            "1992-08-02",
            "--b-place",
            "mumbai",
        ],
        vec![
            "synastry",
            "--a-date",
            "1990-03-15",
            "--a-place",
            "delhi",
            "--b-date",
            "1992-08-02",
            "--b-place",
            "atlantis",
        ],
        // relocate without a destination.
        vec!["relocate", "--date", "1990-03-15", "--place", "delhi"],
        // map: a zero step, an empty range, bad altitude, bad latitude, json with geojson.
        vec!["map", "--date", "2024-06-21", "--lat-step", "0"],
        vec![
            "map",
            "--date",
            "2024-06-21",
            "--lat-min",
            "10",
            "--lat-max",
            "-10",
        ],
        vec!["map", "--date", "2024-06-21", "--altitude", "95"],
        vec!["map", "--date", "2024-06-21", "--lat", "90"],
        vec!["map", "--date", "2024-06-21", "--json", "--geojson"],
        vec!["map", "--date", "2024-06-21", "--geojson", "--lat", "40"],
        vec!["map", "--date", "2024-06-21", "--bodies", "earth"],
        // map: a negative or non-finite orb, and an orb without --parans.
        vec!["map", "--date", "2024-06-21", "--parans", "--lat", "40", "--orb", "-1"],
        vec!["map", "--date", "2024-06-21", "--parans", "--lat", "40", "--orb", "nan"],
        vec!["map", "--date", "2024-06-21", "--lat", "40", "--orb", "1"],
        vec!["map"],
    ];
    for args in cases {
        let out = xalen(&args);
        assert_eq!(out.code, 2, "{args:?}: {}", out.stderr);
        assert!(out.stdout.is_empty(), "{args:?} printed {}", out.stdout);
        assert_eq!(
            out.stderr.trim().lines().count(),
            1,
            "{args:?}: {}",
            out.stderr
        );
    }
    let missing = xalen(&[
        "synastry",
        "--a-date",
        "1990-03-15",
        "--a-place",
        "delhi",
        "--b-place",
        "mumbai",
    ]);
    assert!(missing.stderr.contains("--b-date"), "{}", missing.stderr);
}

// ---------------------------------------------------------------------------
// Distinct births and exact positions, compared with direct library calls
// ---------------------------------------------------------------------------

use serde_json::json;
use xalen_ephemeris::houses::geocoding::city_coordinates;
use xalen_ephemeris::houses::{GeoLocation, HouseCusps, HouseSystem};
use xalen_ephemeris::relationship::{
    BirthMoment, ChartBody, CompositeAscendant, CompositeOptions, DavisonOptions, NatalChart,
    SpaceMidpoint, SynastryOptions, composite_chart, davison_chart, default_bodies, synastry,
};

const DISTINCT_A: [&str; 4] = ["--a-date", "1990-05-10T04:30:00Z", "--a-place", "delhi"];
const DISTINCT_B: [&str; 4] = ["--b-date", "1992-11-02T19:15:00Z", "--b-place", "mumbai"];

fn city(name: &str) -> GeoLocation {
    city_coordinates(name).expect("the city is in the table")
}

fn birth_a() -> JdUT1 {
    calendar_to_jd(1990, 5, 10, 4.5, CalendarSystem::ProlepticGregorian)
}

fn birth_b() -> JdUT1 {
    calendar_to_jd(1992, 11, 2, 19.25, CalendarSystem::ProlepticGregorian)
}

fn natal(almanac: &Almanac, jd: JdUT1, place: &GeoLocation, system: HouseSystem) -> NatalChart {
    NatalChart::compute(almanac, jd, place, system, &default_bodies()).expect("the chart computes")
}

fn distinct_args(command: &str, more: &[&str]) -> Vec<String> {
    std::iter::once(command)
        .chain(DISTINCT_A)
        .chain(DISTINCT_B)
        .chain(more.iter().copied())
        .map(str::to_string)
        .collect()
}

/// Every body longitude, the ascendant, the midheaven and every cusp of a
/// chart-style JSON against the library's values.
fn assert_chart_json(json: &Value, bodies: &[ChartBody], houses: &HouseCusps, what: &str) {
    let items = json["bodies"].as_array().expect("bodies");
    assert_eq!(items.len(), bodies.len(), "{what}: body count");
    for (item, want) in items.iter().zip(bodies) {
        let gap = angle_gap(num(&item["longitude"]), want.longitude_deg);
        assert!(gap < 1e-6, "{what}: {} differs by {gap}", item["id"]);
    }
    let deg = |radians: f64| radians.to_degrees().rem_euclid(360.0);
    assert!(
        angle_gap(angle_of(json, "ascendant"), deg(houses.ascendant)) < 1e-6,
        "{what}: ASC"
    );
    assert!(
        angle_gap(angle_of(json, "mc"), deg(houses.mc)) < 1e-6,
        "{what}: MC"
    );
    let cusps = json["cusps"].as_array().expect("cusps");
    assert_eq!(cusps.len(), 12);
    for (i, cusp) in cusps.iter().enumerate() {
        let gap = angle_gap(num(&cusp["longitude"]), houses.cusp_deg(i));
        assert!(gap < 1e-6, "{what}: cusp {} differs by {gap}", i + 1);
    }
}

#[test]
fn synastry_of_different_births_matches_the_library_cell_by_cell() {
    let almanac = Almanac::default_vedic();
    let a = natal(&almanac, birth_a(), &city("delhi"), HouseSystem::Placidus);
    let b = natal(&almanac, birth_b(), &city("mumbai"), HouseSystem::Placidus);
    let expected = synastry(
        &a,
        &b,
        &SynastryOptions {
            include_angles: true,
            ..SynastryOptions::default()
        },
    )
    .expect("the library computes the synastry");
    let json = json_of_owned(&distinct_args("synastry", &["--angles", "--json"]));

    assert_eq!(json["rows"], serde_json::to_value(&expected.rows).unwrap());
    assert_eq!(json["cols"], serde_json::to_value(&expected.cols).unwrap());
    let (rows, cols) = (expected.rows.len(), expected.cols.len());
    let cells = json["cells"].as_array().expect("cells");
    assert_eq!(cells.len(), rows * cols);
    let mut found = 0;
    for i in 0..rows {
        for j in 0..cols {
            let cell = &cells[i * cols + j];
            match expected.cell(i, j) {
                None => assert!(cell.is_null(), "cell ({i},{j}) should be empty: {cell}"),
                Some(hit) => {
                    found += 1;
                    assert_eq!(
                        cell["aspect_type"],
                        format!("{:?}", hit.aspect_type),
                        "({i},{j})"
                    );
                    assert!(
                        (num(&cell["orb_deg"]) - hit.orb_deg).abs() < 1e-9,
                        "({i},{j}) orb"
                    );
                    assert!((num(&cell["separation_deg"]) - hit.separation_deg).abs() < 1e-9);
                    assert_eq!(
                        cell["a"],
                        serde_json::to_value(hit.a).unwrap(),
                        "({i},{j}) a"
                    );
                    assert_eq!(
                        cell["b"],
                        serde_json::to_value(hit.b).unwrap(),
                        "({i},{j}) b"
                    );
                }
            }
        }
    }
    assert!(found > 5, "a meaningful grid, got {found} aspects");

    // Each overlay lands in the other chart's houses.
    for (key, items, host, guest) in [
        ("b_in_a", &expected.b_in_a, &a, &b),
        ("a_in_b", &expected.a_in_b, &b, &a),
    ] {
        let got = json[key].as_array().expect("overlay");
        let guest_points = guest.points(true);
        assert_eq!(got.len(), items.len(), "{key}");
        for (k, (item, want)) in got.iter().zip(items).enumerate() {
            assert_eq!(
                item["point"],
                serde_json::to_value(want.point).unwrap(),
                "{key} {k}"
            );
            assert_eq!(item["house"], json!(want.house), "{key} {k}");
            assert!(angle_gap(num(&item["longitude_deg"]), guest_points[k].1) < 1e-9);
            let house = host.houses.planet_in_house(guest_points[k].1.to_radians());
            assert_eq!(
                item["house"],
                json!(house),
                "{key} {k} lands in the other chart"
            );
        }
    }
    assert_ne!(
        json["b_in_a"], json["a_in_b"],
        "the two overlays differ for these births"
    );
}

#[test]
fn synastry_reports_a_polar_house_fallback() {
    let out = xalen(&[
        "synastry",
        "--a-date",
        "1990-05-10T04:30:00Z",
        "--a-lat",
        "80",
        "--a-lon",
        "0",
        "--b-date",
        "1992-11-02T19:15:00Z",
        "--b-place",
        "mumbai",
        "--bodies",
        "sun,moon",
    ]);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert!(
        out.stdout.contains("chart A fell back to Porphyry"),
        "{}",
        out.stdout
    );
    assert!(!out.stdout.contains("chart B fell back"));
    let json = json_of(&[
        "synastry",
        "--a-date",
        "1990-05-10T04:30:00Z",
        "--a-lat",
        "80",
        "--a-lon",
        "0",
        "--b-date",
        "1992-11-02T19:15:00Z",
        "--b-place",
        "mumbai",
        "--bodies",
        "sun,moon",
        "--json",
    ]);
    assert_eq!(json["a"]["fallback_used"], true);
    assert_eq!(json["b"]["fallback_used"], false);
}

#[test]
fn composite_of_different_births_matches_the_library() {
    let almanac = Almanac::default_vedic();
    // Derived Ascendant, Placidus.
    let a = natal(&almanac, birth_a(), &city("delhi"), HouseSystem::Placidus);
    let b = natal(&almanac, birth_b(), &city("mumbai"), HouseSystem::Placidus);
    let expected = composite_chart(&a, &b, &CompositeOptions::default()).expect("composite");
    let json = json_of_owned(&distinct_args("composite", &["--json"]));
    assert_eq!(json["ascendant_method"], "derived");
    assert_chart_json(
        &json,
        &expected.bodies,
        &expected.houses,
        "composite derived",
    );

    // Midpoint of the Ascendants, Whole Sign.
    let a = natal(&almanac, birth_a(), &city("delhi"), HouseSystem::WholeSign);
    let b = natal(&almanac, birth_b(), &city("mumbai"), HouseSystem::WholeSign);
    let options = CompositeOptions {
        house_system: HouseSystem::WholeSign,
        ascendant: CompositeAscendant::MidpointOfAscendants,
    };
    let expected = composite_chart(&a, &b, &options).expect("composite");
    let json = json_of_owned(&distinct_args(
        "composite",
        &[
            "--ascendant",
            "midpoint",
            "--house-system",
            "wholesign",
            "--json",
        ],
    ));
    assert_eq!(json["ascendant_method"], "midpoint");
    assert_chart_json(
        &json,
        &expected.bodies,
        &expected.houses,
        "composite midpoint",
    );
    let derived = json_of_owned(&distinct_args(
        "composite",
        &["--house-system", "wholesign", "--json"],
    ));
    assert!(
        angle_gap(
            angle_of(&json, "ascendant"),
            angle_of(&derived, "ascendant")
        ) > 1e-3,
        "the two Ascendant methods give different Ascendants for these births"
    );
}

#[test]
fn davison_of_different_births_matches_the_library() {
    let almanac = Almanac::default_vedic();
    for (space, how) in [
        ("arithmetic", SpaceMidpoint::Arithmetic),
        ("great-circle", SpaceMidpoint::GreatCircle),
    ] {
        for (place_a, place_b) in [("delhi", "mumbai"), ("london", "sydney")] {
            let moment = |jd: JdUT1, place: &str| BirthMoment {
                jd_ut1: jd.0,
                location: city(place),
            };
            let expected = davison_chart(
                &almanac,
                &moment(birth_a(), place_a),
                &moment(birth_b(), place_b),
                &DavisonOptions {
                    space: how,
                    ..DavisonOptions::default()
                },
            )
            .expect("davison");
            let json = json_of(&[
                "davison",
                "--a-date",
                "1990-05-10T04:30:00Z",
                "--a-place",
                place_a,
                "--b-date",
                "1992-11-02T19:15:00Z",
                "--b-place",
                place_b,
                "--space",
                space,
                "--json",
            ]);
            let what = format!("davison {space} {place_a}/{place_b}");
            assert_chart_json(&json, &expected.bodies, &expected.houses, &what);
            assert!(
                (num(&json["jd_ut"]) - expected.jd_ut1).abs() < 1e-9,
                "{what}: date"
            );
            assert!((num(&json["place"]["latitude"]) - expected.location.lat_deg()).abs() < 1e-9);
            assert!((num(&json["place"]["longitude"]) - expected.location.lon_deg()).abs() < 1e-9);
        }
    }
    // London and Sydney are far enough apart for the two midpoints to differ.
    let place = |space: &str| {
        let json = json_of(&[
            "davison",
            "--a-date",
            "1990-05-10T04:30:00Z",
            "--a-place",
            "london",
            "--b-date",
            "1992-11-02T19:15:00Z",
            "--b-place",
            "sydney",
            "--space",
            space,
            "--json",
        ]);
        num(&json["place"]["latitude"])
    };
    assert!((place("arithmetic") - place("great-circle")).abs() > 0.1);
}

#[test]
fn davison_great_circle_of_antipodal_places_exits_two() {
    let antipodes = [
        "davison",
        "--a-date",
        "1990-05-10",
        "--a-lat",
        "10",
        "--a-lon",
        "20",
        "--b-date",
        "1992-11-02",
        "--b-lat",
        "-10",
        "--b-lon",
        "-160",
    ];
    let out = xalen(
        &antipodes
            .iter()
            .copied()
            .chain(["--space", "great-circle"])
            .collect::<Vec<_>>(),
    );
    assert_eq!(out.code, 2, "{}", out.stderr);
    assert!(out.stdout.is_empty());
    assert_eq!(out.stderr.trim().lines().count(), 1, "{}", out.stderr);
    assert!(out.stderr.contains("antipodal"), "{}", out.stderr);
    // The arithmetic midpoint still exists.
    assert_eq!(xalen(&antipodes).code, 0);
}

#[test]
fn sidereal_whole_sign_cusps_sit_on_sign_boundaries() {
    for command in ["composite", "davison"] {
        let json = json_of_owned(&distinct_args(command, &["--sidereal", "--json"]));
        assert_eq!(json["zodiac"], "sidereal", "{command}");
        assert_eq!(json["house_system"], "WholeSign", "{command}");
        let cusps = json["cusps"].as_array().expect("cusps");
        assert_eq!(cusps.len(), 12);
        for cusp in cusps {
            let within = num(&cusp["longitude"]).rem_euclid(30.0);
            assert!(
                within < 1e-6 || 30.0 - within < 1e-6,
                "{command}: cusp {cusp} is not on a sign boundary"
            );
        }
    }
}

#[test]
fn map_geojson_positions_follow_the_library() {
    let almanac = Almanac::default_vedic();
    let expected = astro_map(
        &almanac,
        calendar_to_jd(2024, 6, 21, 0.0, CalendarSystem::ProlepticGregorian),
        &[Body::Sun, Body::Moon],
        &MapOptions::default(),
    )
    .expect("the library maps the bodies");
    let json = json_of(&[
        "map",
        "--date",
        "2024-06-21T00:00:00Z",
        "--bodies",
        "sun,moon",
        "--geojson",
    ]);
    let features = json["features"].as_array().expect("features");
    for body in &expected.bodies {
        let id = format!("{:?}", body.body).to_lowercase();
        let of_kind = |kind: &str| -> Vec<&Value> {
            features
                .iter()
                .filter(|f| f["properties"]["body"] == id && f["properties"]["kind"] == kind)
                .collect()
        };
        // The zenith is [mc_lon, dec].
        let zeniths = of_kind("ZENITH");
        assert_eq!(zeniths.len(), 1, "{id}");
        let at = &zeniths[0]["geometry"]["coordinates"];
        let mc_lon = body.lines[0].segments[0][0].1;
        assert!((num(&at[0]) - mc_lon).abs() < 1e-9, "{id} zenith lon");
        assert!((num(&at[1]) - body.dec_deg).abs() < 1e-9, "{id} zenith lat");
        // MC and IC are meridians at the library's longitude.
        for (kind, index) in [("MC", 0), ("IC", 1)] {
            let lines = of_kind(kind);
            assert_eq!(lines.len(), 1, "{id} {kind}");
            let want = body.lines[index].segments[0][0].1;
            for position in lines[0]["geometry"]["coordinates"]
                .as_array()
                .expect("positions")
            {
                assert!(
                    (num(&position[0]) - want).abs() < 1e-9,
                    "{id} {kind} longitude"
                );
                assert!(
                    (-90.0..=90.0).contains(&num(&position[1])),
                    "{id} {kind} latitude"
                );
            }
        }
        // Every ASC and DSC position is a library vertex, as [lon, lat].
        for (kind, index) in [("ASC", 2), ("DSC", 3)] {
            let vertices: Vec<(f64, f64)> = body.lines[index]
                .segments
                .iter()
                .flatten()
                .copied()
                .collect();
            let mut count = 0;
            for line in of_kind(kind) {
                for position in line["geometry"]["coordinates"]
                    .as_array()
                    .expect("positions")
                {
                    count += 1;
                    let (lon, lat) = (num(&position[0]), num(&position[1]));
                    assert!(
                        vertices
                            .iter()
                            .any(|v| (v.1 - lon).abs() < 1e-9 && (v.0 - lat).abs() < 1e-9),
                        "{id} {kind}: [{lon}, {lat}] is not a library vertex"
                    );
                }
            }
            assert_eq!(count, vertices.len(), "{id} {kind} vertex count");
        }
    }
}

fn paran_map() -> AstroMap {
    astro_map(
        &Almanac::default_vedic(),
        calendar_to_jd(2024, 6, 21, 0.0, CalendarSystem::ProlepticGregorian),
        &[Body::Sun, Body::Moon, Body::Mars, Body::Jupiter],
        &MapOptions::default(),
    )
    .expect("the library maps the bodies")
}

const PARAN_ARGS: [&str; 6] = [
    "map",
    "--date",
    "2024-06-21T00:00:00Z",
    "--bodies",
    "sun,moon,mars,jupiter",
    "--parans",
];

/// The JSON parans equal the library's, field by field, to 1e-9.
fn assert_parans_match(json: &Value, want: &[Paran]) {
    let got = json.as_array().expect("an array of parans");
    assert_eq!(got.len(), want.len(), "{json}");
    for (item, p) in got.iter().zip(want) {
        assert_eq!(item["body_a"], format!("{:?}", p.body_a));
        assert_eq!(item["kind_a"], format!("{:?}", p.kind_a));
        assert_eq!(item["body_b"], format!("{:?}", p.body_b));
        assert_eq!(item["kind_b"], format!("{:?}", p.kind_b));
        assert!((num(&item["latitude_deg"]) - p.latitude_deg).abs() < 1e-9);
        assert!((num(&item["longitude_deg"]) - p.longitude_deg).abs() < 1e-9);
    }
}

#[test]
fn map_parans_json_matches_the_library() {
    let want = parans(&paran_map(), &ParanOptions::default()).expect("the library finds parans");
    assert!(!want.parans.is_empty());
    let json = json_of_owned(&with_flags(&PARAN_ARGS, &["--json"]));
    assert_parans_match(&json["parans"], &want.parans);
    let skipped = json["parans_skipped"].as_array().expect("parans_skipped");
    assert_eq!(skipped.len(), want.skipped.len());
    assert_eq!(json["paran_options"]["include_horizon_pairs"], true);
    // Without --parans the map JSON has no parans.
    let plain = json_of(&PARAN_ARGS[..5]);
    assert!(plain.get("parans").is_none());
}

#[test]
fn map_parans_geojson_points_are_lon_lat() {
    let want = parans(&paran_map(), &ParanOptions::default()).expect("the library finds parans");
    assert!(!want.parans.is_empty());
    let json = json_of_owned(&with_flags(&PARAN_ARGS, &["--geojson"]));
    let points: Vec<&Value> = json["features"]
        .as_array()
        .expect("features")
        .iter()
        .filter(|f| f["properties"]["kind"] == "PARAN")
        .collect();
    assert_eq!(points.len(), want.parans.len());
    for (feature, p) in points.iter().zip(&want.parans) {
        assert_eq!(feature["geometry"]["type"], "Point");
        let at = &feature["geometry"]["coordinates"];
        assert!((num(&at[0]) - p.longitude_deg).abs() < 1e-9, "{feature}");
        assert!((num(&at[1]) - p.latitude_deg).abs() < 1e-9, "{feature}");
        let props = &feature["properties"];
        assert_eq!(props["body_a"], format!("{:?}", p.body_a).to_lowercase());
        assert_eq!(props["body_b"], format!("{:?}", p.body_b).to_lowercase());
        assert_eq!(props["kind_a"], format!("{:?}", p.kind_a).to_uppercase());
        assert_eq!(props["kind_b"], format!("{:?}", p.kind_b).to_uppercase());
    }
}

#[test]
fn map_parans_at_a_latitude_match_parans_near() {
    let map = paran_map();
    let options = ParanOptions::default();
    let all = parans(&map, &options).expect("the library finds parans").parans;
    assert!(!all.is_empty());
    // A latitude just off a known paran, so the band holds at least one.
    let lat = all[all.len() / 2].latitude_deg + 0.25;
    let want = parans_near(&map, lat, 2.0, &options).expect("parans near the latitude");
    assert!(!want.is_empty());
    let lat_arg = format!("{lat}");
    let near = ["--lat", lat_arg.as_str(), "--orb", "2"];
    let mut args = with_flags(&PARAN_ARGS, &near);
    args.push("--json".to_string());
    let json = json_of_owned(&args);
    assert!((num(&json["parans_near"]["latitude"]) - lat).abs() < 1e-12);
    assert_eq!(num(&json["parans_near"]["orb"]), 2.0);
    assert_parans_match(&json["parans_near"]["parans"], &want);
    // The full set is still in "parans".
    assert_parans_match(&json["parans"], &all);
    // The table lists exactly those parans.
    let args = with_flags(&PARAN_ARGS, &near);
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let out = xalen(&refs);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert!(out.stdout.contains("Parans within 2.00 deg of latitude"), "{}", out.stdout);
    let section = out.stdout.split("Parans within").nth(1).expect("the paran section");
    let rows: Vec<&str> = section
        .lines()
        .skip(3)
        .take_while(|l| {
            l.split_whitespace()
                .next()
                .is_some_and(|c| c.parse::<f64>().is_ok())
        })
        .collect();
    assert_eq!(rows.len(), want.len(), "{section}");
    for (row, p) in rows.iter().zip(&want) {
        let cells: Vec<&str> = row.split_whitespace().collect();
        assert_eq!(cells[0], format!("{:.4}", p.latitude_deg), "{row}");
        assert_eq!(cells[1], format!("{:.4}", p.longitude_deg), "{row}");
    }
}

fn vertex_lon(line: &Value, lat: f64) -> f64 {
    line["segments"]
        .as_array()
        .expect("segments")
        .iter()
        .flat_map(|s| s.as_array().expect("points").iter())
        .find(|p| (num(&p[0]) - lat).abs() < 1e-9)
        .map(|p| num(&p[1]))
        .unwrap_or_else(|| panic!("no vertex at latitude {lat}"))
}

#[test]
fn map_dsc_and_never_rises_at_a_latitude() {
    let json = json_of(&[
        "map",
        "--date",
        "2024-06-21T00:00:00Z",
        "--bodies",
        "sun",
        "--lat",
        "40",
        "--lat-step",
        "0.5",
        "--json",
    ]);
    let dsc = &json["bodies"][0]["lines"][3];
    assert_eq!(dsc["kind"], "Dsc");
    let want = num(&json["at_latitude"]["bodies"][0]["dsc_longitude"]);
    assert!((vertex_lon(dsc, 40.0) - want).abs() < 1e-6);

    // Near the June solstice the Sun never rises at 80 south.
    let south = json_of(&[
        "map",
        "--date",
        "2024-06-21T00:00:00Z",
        "--bodies",
        "sun",
        "--lat",
        "-80",
        "--json",
    ]);
    let entry = &south["at_latitude"]["bodies"][0];
    assert_eq!(entry["status"], "never rises");
    assert!(entry["asc_longitude"].is_null() && entry["dsc_longitude"].is_null());
    let text = xalen(&[
        "map",
        "--date",
        "2024-06-21T00:00:00Z",
        "--bodies",
        "sun",
        "--lat",
        "-80",
    ]);
    assert_eq!(text.code, 0, "{}", text.stderr);
    assert!(text.stdout.contains("never rises"), "{}", text.stdout);
}

#[test]
fn map_altitude_moves_the_horizon_lines() {
    let run = |altitude: &str| {
        json_of(&[
            "map",
            "--date",
            "2024-06-21T00:00:00Z",
            "--bodies",
            "sun",
            "--lat",
            "40",
            "--lat-step",
            "0.5",
            "--altitude",
            altitude,
            "--json",
        ])
    };
    let flat = run("0");
    let low = run("-0.5667");
    assert!((num(&low["options"]["horizon_altitude_deg"]) + 0.5667).abs() < 1e-12);
    let asc_flat = vertex_lon(&flat["bodies"][0]["lines"][2], 40.0);
    let asc_low = vertex_lon(&low["bodies"][0]["lines"][2], 40.0);
    assert!((asc_flat - asc_low).abs() > 1e-3, "{asc_flat} vs {asc_low}");

    let almanac = Almanac::default_vedic();
    let expected = astro_map(
        &almanac,
        calendar_to_jd(2024, 6, 21, 0.0, CalendarSystem::ProlepticGregorian),
        &[Body::Sun],
        &MapOptions {
            lat_step_deg: 0.5,
            horizon_altitude_deg: -0.5667,
            ..MapOptions::default()
        },
    )
    .expect("the library maps the Sun");
    for (index, key, line) in [(2, "asc_longitude", "Asc"), (3, "dsc_longitude", "Dsc")] {
        let want = expected.bodies[0].lines[index]
            .segments
            .iter()
            .flatten()
            .find(|v| (v.0 - 40.0).abs() < 1e-9)
            .expect("a vertex at 40 north")
            .1;
        assert_eq!(low["bodies"][0]["lines"][index]["kind"], line);
        assert!((vertex_lon(&low["bodies"][0]["lines"][index], 40.0) - want).abs() < 1e-9);
        let at = num(&low["at_latitude"]["bodies"][0][key]);
        assert!((at - want).abs() < 1e-6, "{key}: {at} vs {want}");
    }
}
