//! `xalen panchang`: tithi, nakshatra, yoga, karana, vara, sunrise and sunset.

use serde_json::{Value, json};
use xalen_ephemeris::ayanamsa::Ayanamsa;
use xalen_ephemeris::ephem::{Almanac, Body, EphemerisError};
use xalen_ephemeris::time::{CalendarSystem, JdUT1, calendar_to_jd, jd_to_calendar};
use xalen_ephemeris::vedic::nakshatra::Nakshatra;
use xalen_ephemeris::vedic::panchang::{compute_panchang, compute_panchang_transitions};

use crate::cli::PanchangArgs;
use crate::common::{Frame, base_json, header, place_json};
use crate::error::{CliError, Result};
use crate::input::{delta_t_model, local_day_start, parse_ayanamsa};
use crate::output::{Report, num, table};

/// Sidereal Sun and Moon longitudes in degrees at a UT Julian Day.
fn sun_moon(
    almanac: &Almanac,
    ayanamsa: Ayanamsa,
    jd_ut: f64,
) -> std::result::Result<(f64, f64), EphemerisError> {
    let jd_tt = JdUT1(jd_ut).to_tt(&delta_t_model()).0;
    let aya = ayanamsa.compute_deg(jd_tt);
    let sun = almanac.sidereal_longitude_deg(Body::Sun, JdUT1(jd_ut), aya)?;
    let moon = almanac.sidereal_longitude_deg(Body::Moon, JdUT1(jd_ut), aya)?;
    Ok((sun, moon))
}

/// A Julian Day at noon UT on the calendar date that `jd_ut` has in a zone
/// `offset_hours` ahead of UTC (the library derives the weekday from it).
fn weekday_jd(jd_ut: f64, offset_hours: f64) -> f64 {
    let (year, month, day, _) = jd_to_calendar(
        jd_ut + offset_hours / 24.0,
        CalendarSystem::ProlepticGregorian,
    );
    calendar_to_jd(year, month, day, 12.0, CalendarSystem::ProlepticGregorian).0
}

/// Compute and format the panchang.
pub fn run(args: &PanchangArgs) -> Result<Report> {
    let inst = args.time.instant(&args.out)?;
    let place = args
        .loc
        .resolve(true)?
        .ok_or_else(|| CliError::usage("missing --lat and --lon (or use --place \"<city>\")"))?;
    let ayanamsa = match &args.ayanamsa {
        Some(text) => parse_ayanamsa(text)?,
        None => Ayanamsa::Lahiri,
    };
    let frame = Frame::new(Some(ayanamsa), inst.jd_tt);
    let almanac = Almanac::default_vedic();

    // The civil day that sunrise and the weekday belong to: the zone given with
    // --tz, otherwise the place's mean solar time.
    let offset = if inst.tz.explicit {
        inst.tz.offset_hours
    } else {
        place.lon / 15.0
    };
    let day_start = local_day_start(inst.jd_ut, offset);
    let sun_times = almanac.rise_transit_set_next(
        Body::Sun,
        JdUT1(day_start),
        place.lat,
        place.lon,
        place.alt,
    )?;
    let sunrise = sun_times.rise.map(|j| j.0);
    let sunset = sun_times.set.map(|j| j.0);

    // The vara runs from sunrise to sunrise: before today's sunrise it is
    // still yesterday's.
    let mut vara_day = weekday_jd(inst.jd_ut, offset);
    if sunrise.is_some_and(|rise| inst.jd_ut < rise) {
        vara_day -= 1.0;
    }

    let (sun, moon) = sun_moon(&almanac, ayanamsa, inst.jd_ut)?;
    let panchang = compute_panchang(sun, moon, vara_day);
    let positions = |jd: f64| sun_moon(&almanac, ayanamsa, jd).unwrap_or((f64::NAN, f64::NAN));
    let transitions = compute_panchang_transitions(&positions, inst.jd_ut, true)
        .ok_or_else(|| CliError::compute("could not find the end times of the panchang limbs"))?;
    let pada = Nakshatra::pada(moon);

    let tz = &inst.tz;
    let tithi_label = panchang.tithi.to_string();
    let rows = vec![
        vec![
            "Tithi".to_string(),
            tithi_label.clone(),
            tz.stamp_opt(transitions.tithi.start_jd),
            tz.stamp(transitions.tithi.end_jd),
        ],
        vec![
            "Nakshatra".to_string(),
            format!("{} (pada {pada})", panchang.nakshatra),
            tz.stamp_opt(transitions.nakshatra.start_jd),
            tz.stamp(transitions.nakshatra.end_jd),
        ],
        vec![
            "Yoga".to_string(),
            panchang.yoga.name().to_string(),
            tz.stamp_opt(transitions.yoga.start_jd),
            tz.stamp(transitions.yoga.end_jd),
        ],
        vec![
            "Karana".to_string(),
            panchang.karana.name().to_string(),
            tz.stamp_opt(transitions.karana.start_jd),
            tz.stamp(transitions.karana.end_jd),
        ],
    ];

    let place_label = place
        .name
        .clone()
        .unwrap_or_else(|| format!("{}, {}", num(place.lat, 4), num(place.lon, 4)));
    let mut text = header("Panchang", &inst, &frame);
    text.push_str(&format!(
        "Place: {place_label}\nSun {} deg, Moon {} deg (sidereal)\n\n",
        num(sun, 4),
        num(moon, 4)
    ));
    text.push_str(&table(&["Limb", "Value", "Starts", "Ends"], &rows));
    text.push_str(&format!(
        "\nVara:    {} (lord {})\nSunrise: {}\nSunset:  {}\n",
        panchang.vara.name(),
        panchang.vara.lord(),
        tz.stamp_opt(sunrise),
        tz.stamp_opt(sunset)
    ));

    let interval = |start: Option<f64>, end: f64| {
        json!({
            "starts": tz.iso_opt(start),
            "ends": tz.iso(end),
            "starts_jd_ut": start,
            "ends_jd_ut": end,
        })
    };
    let mut json = base_json("panchang", &inst, &frame);
    json["place"] = place_json(&place);
    json["sun_longitude"] = json!(sun);
    json["moon_longitude"] = json!(moon);
    let mut tithi = interval(transitions.tithi.start_jd, transitions.tithi.end_jd);
    tithi["number"] = json!(panchang.tithi.number);
    tithi["name"] = json!(panchang.tithi.name());
    tithi["paksha"] = json!(format!("{:?}", panchang.tithi.paksha));
    tithi["label"] = json!(tithi_label);
    json["tithi"] = tithi;
    let mut nakshatra = interval(transitions.nakshatra.start_jd, transitions.nakshatra.end_jd);
    nakshatra["name"] = json!(panchang.nakshatra.to_string());
    nakshatra["pada"] = json!(pada);
    nakshatra["lord"] = json!(panchang.nakshatra.lord().to_string());
    json["nakshatra"] = nakshatra;
    let mut yoga = interval(transitions.yoga.start_jd, transitions.yoga.end_jd);
    yoga["number"] = json!(panchang.yoga.number);
    yoga["name"] = json!(panchang.yoga.name());
    json["yoga"] = yoga;
    let mut karana = interval(transitions.karana.start_jd, transitions.karana.end_jd);
    karana["number"] = json!(panchang.karana.number);
    karana["name"] = json!(panchang.karana.name());
    json["karana"] = karana;
    json["vara"] = json!({
        "name": panchang.vara.name(),
        "weekday": format!("{:?}", panchang.vara),
        "lord": panchang.vara.lord(),
    });
    json["sunrise"] = tz.iso_opt(sunrise);
    json["sunset"] = tz.iso_opt(sunset);
    json["sunrise_jd_ut"] = sunrise.map_or(Value::Null, |j| json!(j));
    json["sunset_jd_ut"] = sunset.map_or(Value::Null, |j| json!(j));
    Ok(Report { json, text })
}
