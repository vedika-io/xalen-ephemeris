//! Parsing and validation of everything typed on the command line: dates,
//! time zones, places, bodies, ayanamsas and house systems.
//!
//! This module only turns text into library types. It does no astronomy.

use xalen_ephemeris::ayanamsa::Ayanamsa;
use xalen_ephemeris::ephem::Body;
use xalen_ephemeris::houses::HouseSystem;
use xalen_ephemeris::time::timezone::timezone_offset_hours;
use xalen_ephemeris::time::{CalendarSystem, DeltaTModel, JdUT1, calendar_to_jd, jd_to_calendar};

use crate::error::{CliError, Result};

/// Lower-case a name and drop everything but letters and digits, so that
/// `Whole-Sign`, `whole_sign` and `WholeSign` all compare equal.
pub fn normalize(text: &str) -> String {
    text.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// The delta-T model used everywhere (the same one `Almanac::default_vedic`
/// uses internally).
pub fn delta_t_model() -> DeltaTModel {
    DeltaTModel::StephensonMorrisonHohenkerk2016
}

// ---------------------------------------------------------------------------
// Dates
// ---------------------------------------------------------------------------

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ => {
            let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
            if leap { 29 } else { 28 }
        }
    }
}

/// Parse a UTC offset such as `+05:30`, `-0800` or `+02` into hours.
fn parse_offset(text: &str) -> Option<f64> {
    if !text.is_ascii() {
        return None;
    }
    let sign = match text.as_bytes().first()? {
        b'+' => 1.0,
        b'-' => -1.0,
        _ => return None,
    };
    let rest = &text[1..];
    let (hours, minutes) = if let Some((h, m)) = rest.split_once(':') {
        (h, m)
    } else if rest.len() == 4 {
        rest.split_at(2)
    } else {
        (rest, "0")
    };
    let all_digits = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit());
    if !all_digits(hours) || !all_digits(minutes) {
        return None;
    }
    let h: u32 = hours.parse().ok()?;
    let m: u32 = minutes.parse().ok()?;
    if h > 14 || m > 59 {
        return None;
    }
    Some(sign * (f64::from(h) + f64::from(m) / 60.0))
}

/// Split `12:30:00-08:00` into the clock part and the zone offset in hours.
/// No zone suffix means UTC.
fn split_zone(time: &str) -> Option<(&str, f64)> {
    if let Some(clock) = time.strip_suffix(['Z', 'z']) {
        return Some((clock, 0.0));
    }
    match time.rfind(['+', '-']) {
        Some(i) if i > 0 => parse_offset(&time[i..]).map(|offset| (&time[..i], offset)),
        _ => Some((time, 0.0)),
    }
}

/// Parse an ISO-8601 instant into a Julian Day (UT, taking UTC as UT1).
///
/// Accepted: `YYYY-MM-DD` (midnight UTC), `YYYY-MM-DDTHH:MM[:SS[.fff]]` with a
/// `Z` or `+HH:MM` suffix; a missing suffix means UTC. The calendar is the
/// proleptic Gregorian one, as ISO-8601 defines.
pub fn parse_iso(input: &str) -> Result<f64> {
    let bad = |why: &str| {
        CliError::usage(format!(
            "bad date '{input}': {why} (use ISO-8601, e.g. 2026-11-07T00:00:00Z \
             or 2026-11-07T05:30:00+05:30)"
        ))
    };
    let text = input.trim();
    let (date_part, time_part) = match text.find(['T', 't', ' ']) {
        Some(i) => (&text[..i], Some(text[i + 1..].trim())),
        None => (text, None),
    };

    let (negative, digits) = match date_part.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, date_part),
    };
    let mut fields = digits.split('-');
    let (year_text, month_text, day_text) =
        match (fields.next(), fields.next(), fields.next(), fields.next()) {
            (Some(y), Some(m), Some(d), None) => (y, m, d),
            _ => return Err(bad("the date part must be YYYY-MM-DD")),
        };
    let numeric = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit());
    if !numeric(year_text) || !numeric(month_text) || !numeric(day_text) {
        return Err(bad("the date part must be YYYY-MM-DD"));
    }
    let year: i32 = year_text
        .parse()
        .map_err(|_| bad("the year is too large"))?;
    let year = if negative { -year } else { year };
    let month: u32 = month_text
        .parse()
        .map_err(|_| bad("the month is not a number"))?;
    let day: u32 = day_text
        .parse()
        .map_err(|_| bad("the day is not a number"))?;
    if !(1..=12).contains(&month) {
        return Err(bad("the month must be 01 to 12"));
    }
    if day == 0 || day > days_in_month(year, month) {
        return Err(bad("the day does not exist in that month"));
    }

    let mut hours = 0.0;
    let mut offset = 0.0;
    if let Some(time) = time_part.filter(|t| !t.is_empty()) {
        let (clock, zone) =
            split_zone(time).ok_or_else(|| bad("the zone must be Z or +HH:MM or -HH:MM"))?;
        offset = zone;
        let parts: Vec<&str> = clock.split(':').collect();
        if parts.is_empty() || parts.len() > 3 {
            return Err(bad("the time must be HH[:MM[:SS]]"));
        }
        let hour_text = parts[0];
        let minute_text = parts.get(1).copied().unwrap_or("0");
        let second_text = parts.get(2).copied().unwrap_or("0");
        let decimal = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit() || c == '.');
        if !decimal(hour_text) || !decimal(minute_text) || !decimal(second_text) {
            return Err(bad("the time must be HH[:MM[:SS]]"));
        }
        let hour: f64 = hour_text
            .parse()
            .map_err(|_| bad("the hour is not a number"))?;
        let minute: f64 = minute_text
            .parse()
            .map_err(|_| bad("the minute is not a number"))?;
        let second: f64 = second_text
            .parse()
            .map_err(|_| bad("the second is not a number"))?;
        if hour >= 24.0 || minute >= 60.0 || second >= 60.0 {
            return Err(bad("the time is out of range"));
        }
        hours = hour + minute / 60.0 + second / 3600.0;
    }

    let jd = calendar_to_jd(
        year,
        month,
        day,
        hours - offset,
        CalendarSystem::ProlepticGregorian,
    );
    Ok(jd.0)
}

/// The Julian Day (UT) of local midnight that starts the civil day containing
/// `jd_ut`, for a zone `offset_hours` ahead of UTC.
pub fn local_day_start(jd_ut: f64, offset_hours: f64) -> f64 {
    let (year, month, day, _) = jd_to_calendar(
        jd_ut + offset_hours / 24.0,
        CalendarSystem::ProlepticGregorian,
    );
    calendar_to_jd(
        year,
        month,
        day,
        -offset_hours,
        CalendarSystem::ProlepticGregorian,
    )
    .0
}

// ---------------------------------------------------------------------------
// Time zones and instants
// ---------------------------------------------------------------------------

/// A fixed UTC offset used to print times.
#[derive(Debug, Clone)]
pub struct Tz {
    /// Hours ahead of UTC.
    pub offset_hours: f64,
    /// `UTC` or a numeric offset such as `+05:30`.
    pub label: String,
    /// True when the user passed `--tz`.
    pub explicit: bool,
}

/// Format an offset in hours as `+05:30`.
fn format_offset(hours: f64) -> String {
    let total = (hours.abs() * 60.0).round() as u32;
    let sign = if hours < 0.0 { '-' } else { '+' };
    format!("{sign}{:02}:{:02}", total / 60, total % 60)
}

impl Tz {
    /// Plain UTC.
    pub fn utc() -> Self {
        Tz {
            offset_hours: 0.0,
            label: "UTC".to_string(),
            explicit: false,
        }
    }

    /// Parse a `--tz` value: `UTC`, a numeric offset, or a city from the
    /// library's timezone table (standard time, no daylight saving).
    pub fn parse(text: &str) -> Result<Self> {
        let trimmed = text.trim();
        let lower = trimmed.to_ascii_lowercase();
        if matches!(lower.as_str(), "utc" | "z" | "gmt") {
            return Ok(Tz {
                explicit: true,
                ..Tz::utc()
            });
        }
        let offset = if trimmed.starts_with(['+', '-']) {
            parse_offset(trimmed)
        } else {
            timezone_offset_hours(trimmed)
        };
        match offset {
            Some(offset_hours) => Ok(Tz {
                offset_hours,
                label: format_offset(offset_hours),
                explicit: true,
            }),
            None => Err(CliError::usage(format!(
                "unknown time zone '{trimmed}': use UTC, an offset such as +05:30, or a city \
                 from the built-in table (standard time only; IANA names are not supported)"
            ))),
        }
    }

    fn parts(&self, jd_ut: f64) -> (i32, u32, u32, u32, u32, u32) {
        let local = jd_ut + self.offset_hours / 24.0 + 0.5 / 86_400.0;
        let (year, month, day, hour) = jd_to_calendar(local, CalendarSystem::ProlepticGregorian);
        let seconds = (hour * 3600.0).floor().clamp(0.0, 86_399.0) as u32;
        (
            year,
            month,
            day,
            seconds / 3600,
            seconds % 3600 / 60,
            seconds % 60,
        )
    }

    /// `2024-04-08 18:17:00 UTC`
    pub fn stamp(&self, jd_ut: f64) -> String {
        let (y, mo, d, h, mi, s) = self.parts(jd_ut);
        format!("{y:04}-{mo:02}-{d:02} {h:02}:{mi:02}:{s:02} {}", self.label)
    }

    /// `2024-04-08T18:17:00Z` or `2024-04-08T23:47:00+05:30`
    pub fn iso(&self, jd_ut: f64) -> String {
        let (y, mo, d, h, mi, s) = self.parts(jd_ut);
        let zone = if self.label == "UTC" {
            "Z"
        } else {
            self.label.as_str()
        };
        format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}{zone}")
    }

    /// A time stamp, or `-` when there is no event.
    pub fn stamp_opt(&self, jd_ut: Option<f64>) -> String {
        jd_ut.map_or_else(|| "-".to_string(), |jd| self.stamp(jd))
    }

    /// An ISO-8601 JSON string, or `null` when there is no event.
    pub fn iso_opt(&self, jd_ut: Option<f64>) -> serde_json::Value {
        jd_ut.map_or(serde_json::Value::Null, |jd| {
            serde_json::Value::String(self.iso(jd))
        })
    }

    /// `2024-04-08`
    pub fn date(&self, jd_ut: f64) -> String {
        let (y, mo, d, _, _, _) = self.parts(jd_ut);
        format!("{y:04}-{mo:02}-{d:02}")
    }
}

/// One moment in time in the scales the library needs.
#[derive(Debug, Clone)]
pub struct Instant {
    /// Julian Day, UT (civil UTC taken as UT1).
    pub jd_ut: f64,
    /// Julian Day, TT.
    pub jd_tt: f64,
    /// How to print times.
    pub tz: Tz,
}

impl Instant {
    /// Build an instant from a UT Julian Day.
    pub fn new(jd_ut: f64, tz: Tz) -> Result<Self> {
        if !jd_ut.is_finite() {
            return Err(CliError::usage("the Julian Day must be a finite number"));
        }
        let jd_tt = JdUT1(jd_ut).to_tt(&delta_t_model()).0;
        Ok(Instant { jd_ut, jd_tt, tz })
    }

    /// The instant as the library's UT1 type.
    pub fn ut1(&self) -> JdUT1 {
        JdUT1(self.jd_ut)
    }

    /// Printable time stamp.
    pub fn stamp(&self) -> String {
        self.tz.stamp(self.jd_ut)
    }

    /// ISO-8601 time stamp.
    pub fn iso(&self) -> String {
        self.tz.iso(self.jd_ut)
    }
}

// ---------------------------------------------------------------------------
// Places
// ---------------------------------------------------------------------------

/// An observer on Earth.
#[derive(Debug, Clone)]
pub struct Place {
    /// Latitude in degrees, north positive.
    pub lat: f64,
    /// Longitude in degrees, east positive.
    pub lon: f64,
    /// Height above sea level in metres.
    pub alt: f64,
    /// The city name when `--place` was used.
    pub name: Option<String>,
}

// ---------------------------------------------------------------------------
// Bodies
// ---------------------------------------------------------------------------

/// A point that can be listed: a library body, or Ketu (Rahu + 180 degrees).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Point {
    /// A body the ephemeris computes.
    Body(Body),
    /// The south node, derived from the mean node.
    Ketu,
}

impl Point {
    /// Display name.
    pub fn name(self) -> &'static str {
        match self {
            Point::Body(Body::Sun) => "Sun",
            Point::Body(Body::Moon) => "Moon",
            Point::Body(Body::Mercury) => "Mercury",
            Point::Body(Body::Venus) => "Venus",
            Point::Body(Body::Earth) => "Earth",
            Point::Body(Body::Mars) => "Mars",
            Point::Body(Body::Jupiter) => "Jupiter",
            Point::Body(Body::Saturn) => "Saturn",
            Point::Body(Body::Uranus) => "Uranus",
            Point::Body(Body::Neptune) => "Neptune",
            Point::Body(Body::Pluto) => "Pluto",
            Point::Body(Body::MeanNode) => "Mean Node",
            Point::Body(Body::TrueNode) => "True Node",
            Point::Body(Body::MeanApogee) => "Lilith",
            Point::Body(Body::OsculatingApogee) => "True Lilith",
            Point::Body(Body::Chiron) => "Chiron",
            Point::Ketu => "Ketu",
        }
    }

    /// Stable lower-case identifier used in JSON and accepted by `--bodies`.
    pub fn id(self) -> &'static str {
        match self {
            Point::Body(Body::Sun) => "sun",
            Point::Body(Body::Moon) => "moon",
            Point::Body(Body::Mercury) => "mercury",
            Point::Body(Body::Venus) => "venus",
            Point::Body(Body::Earth) => "earth",
            Point::Body(Body::Mars) => "mars",
            Point::Body(Body::Jupiter) => "jupiter",
            Point::Body(Body::Saturn) => "saturn",
            Point::Body(Body::Uranus) => "uranus",
            Point::Body(Body::Neptune) => "neptune",
            Point::Body(Body::Pluto) => "pluto",
            Point::Body(Body::MeanNode) => "mean-node",
            Point::Body(Body::TrueNode) => "true-node",
            Point::Body(Body::MeanApogee) => "lilith",
            Point::Body(Body::OsculatingApogee) => "true-lilith",
            Point::Body(Body::Chiron) => "chiron",
            Point::Ketu => "ketu",
        }
    }
}

/// Bodies listed when `--bodies` is not given.
pub const DEFAULT_POINTS: [Point; 16] = [
    Point::Body(Body::Sun),
    Point::Body(Body::Moon),
    Point::Body(Body::Mercury),
    Point::Body(Body::Venus),
    Point::Body(Body::Mars),
    Point::Body(Body::Jupiter),
    Point::Body(Body::Saturn),
    Point::Body(Body::Uranus),
    Point::Body(Body::Neptune),
    Point::Body(Body::Pluto),
    Point::Body(Body::MeanNode),
    Point::Body(Body::TrueNode),
    Point::Ketu,
    Point::Body(Body::Chiron),
    Point::Body(Body::MeanApogee),
    Point::Body(Body::OsculatingApogee),
];

/// Bodies compared when `aspects` is run without `--bodies`.
pub const ASPECT_POINTS: [Point; 11] = [
    Point::Body(Body::Sun),
    Point::Body(Body::Moon),
    Point::Body(Body::Mercury),
    Point::Body(Body::Venus),
    Point::Body(Body::Mars),
    Point::Body(Body::Jupiter),
    Point::Body(Body::Saturn),
    Point::Body(Body::Uranus),
    Point::Body(Body::Neptune),
    Point::Body(Body::Pluto),
    Point::Body(Body::MeanNode),
];

/// The names accepted by `--bodies` and `--body`, for help and errors.
pub const POINT_NAMES: &str = "sun, moon, mercury, venus, mars, jupiter, saturn, uranus, \
neptune, pluto, rahu (mean node), true-node, ketu, chiron, lilith (mean), true-lilith";

/// Parse one body name.
pub fn parse_point(text: &str) -> Result<Point> {
    let body = match normalize(text).as_str() {
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
        "rahu" | "meannode" | "node" | "northnode" => Body::MeanNode,
        "truenode" | "truerahu" | "truenorthnode" => Body::TrueNode,
        "ketu" | "southnode" => return Ok(Point::Ketu),
        "chiron" => Body::Chiron,
        "lilith" | "meanlilith" | "meanapogee" | "blackmoon" => Body::MeanApogee,
        "truelilith" | "osculatinglilith" | "osculatingapogee" => Body::OsculatingApogee,
        "earth" => {
            return Err(CliError::usage(
                "Earth has no geocentric position; choose another body",
            ));
        }
        _ => {
            return Err(CliError::usage(format!(
                "unknown body '{}'; valid names: {POINT_NAMES}",
                text.trim()
            )));
        }
    };
    Ok(Point::Body(body))
}

/// Resolve `--bodies`. The flag is true when the user named the bodies, in
/// which case a body that cannot be computed is an error rather than skipped.
pub fn select_points(names: &[String], default: &[Point]) -> Result<(Vec<Point>, bool)> {
    if names.is_empty() {
        return Ok((default.to_vec(), false));
    }
    let mut points: Vec<Point> = Vec::new();
    for name in names {
        let point = parse_point(name)?;
        if !points.contains(&point) {
            points.push(point);
        }
    }
    Ok((points, true))
}

// ---------------------------------------------------------------------------
// Ayanamsas and house systems
// ---------------------------------------------------------------------------

/// Parse an ayanamsa by name (`Lahiri`, `KPKrishnamurti`, `fagan-bradley`, ...)
/// or by Swiss Ephemeris number.
pub fn parse_ayanamsa(text: &str) -> Result<Ayanamsa> {
    let trimmed = text.trim();
    if let Ok(id) = trimmed.parse::<u32>() {
        return Ayanamsa::from_swiss_ephem_id(id)
            .ok_or_else(|| CliError::usage(format!("no ayanamsa has Swiss Ephemeris id {id}")));
    }
    let wanted = normalize(trimmed);
    let all = Ayanamsa::all_named();
    all.iter()
        .copied()
        .find(|a| normalize(&format!("{a:?}")) == wanted)
        .or_else(|| {
            all.iter()
                .copied()
                .find(|a| normalize(&a.to_string()) == wanted)
        })
        .ok_or_else(|| {
            CliError::usage(format!(
                "unknown ayanamsa '{trimmed}'; run 'xalen convert --help' for the list"
            ))
        })
}

/// All ayanamsa names, comma separated, for help text.
pub fn ayanamsa_names() -> String {
    Ayanamsa::all_named()
        .iter()
        .map(|a| format!("{a:?}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Every house system the library offers.
pub const HOUSE_SYSTEMS: [HouseSystem; 23] = [
    HouseSystem::Placidus,
    HouseSystem::WholeSign,
    HouseSystem::Equal,
    HouseSystem::Koch,
    HouseSystem::Porphyry,
    HouseSystem::Regiomontanus,
    HouseSystem::Campanus,
    HouseSystem::Morinus,
    HouseSystem::Alcabitius,
    HouseSystem::Topocentric,
    HouseSystem::Meridian,
    HouseSystem::Vehlow,
    HouseSystem::Sripati,
    HouseSystem::KrusinskiPisa,
    HouseSystem::Gauquelin,
    HouseSystem::SunshineMakransky,
    HouseSystem::SunshineTreindl,
    HouseSystem::PullenSinusoidalDelta,
    HouseSystem::PullenSinusoidalRatio,
    HouseSystem::CarterPoliEquatorial,
    HouseSystem::APC,
    HouseSystem::Zariel,
    HouseSystem::AlcabitiusClassic,
];

/// Parse a house system by name or by its one-letter Swiss Ephemeris code.
pub fn parse_house_system(text: &str) -> Result<HouseSystem> {
    let trimmed = text.trim();
    let mut letters = trimmed.chars();
    if let (Some(letter), None) = (letters.next(), letters.next()) {
        if let Some(system) = HOUSE_SYSTEMS
            .iter()
            .copied()
            .find(|s| s.swiss_ephem_code() == Some(letter))
        {
            return Ok(system);
        }
    }
    let wanted = normalize(trimmed);
    HOUSE_SYSTEMS
        .iter()
        .copied()
        .find(|s| normalize(&format!("{s:?}")) == wanted)
        .or_else(|| {
            HOUSE_SYSTEMS
                .iter()
                .copied()
                .find(|s| normalize(&s.to_string()) == wanted)
        })
        .ok_or_else(|| {
            CliError::usage(format!(
                "unknown house system '{trimmed}'; run 'xalen chart --help' for the list"
            ))
        })
}

/// All house system names, comma separated, for help text.
pub fn house_system_names() -> String {
    HOUSE_SYSTEMS
        .iter()
        .map(|s| format!("{s:?}"))
        .collect::<Vec<_>>()
        .join(", ")
}
