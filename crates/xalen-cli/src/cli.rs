//! Command-line definitions: the `clap` structures for every subcommand, and
//! the resolution of the shared options (time, place, zodiac) into values.

use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use xalen_ephemeris::ayanamsa::Ayanamsa;
use xalen_ephemeris::houses::GeoLocation;
use xalen_ephemeris::houses::geocoding::city_coordinates;

use crate::error::{CliError, Result};
use crate::input::{
    Instant, POINT_NAMES, Place, Tz, ayanamsa_names, house_system_names, parse_ayanamsa, parse_iso,
};

const ROOT_EXAMPLE: &str = "Example:\n  xalen positions --date 2026-11-07T00:00:00Z --json";

/// XALEN Ephemeris on the command line.
#[derive(Parser, Debug)]
#[command(
    name = "xalen",
    version,
    about = "XALEN Ephemeris on the command line: positions, charts, panchang, dasha, eclipses and more",
    long_about = "XALEN Ephemeris on the command line: positions, charts, panchang, dasha, \
eclipses and more. Pure Rust, offline, no data files.\n\n\
Every command takes --date <ISO-8601> or --jd <Julian Day UT>, prints a readable \
table by default and JSON with --json. Dates without a zone are UTC. Exit codes: \
0 success, 1 computation failed, 2 bad input.",
    after_help = ROOT_EXAMPLE,
    arg_required_else_help = true
)]
pub struct Cli {
    /// The command to run.
    #[command(subcommand)]
    pub command: Command,
}

/// The subcommands.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Longitudes, latitudes, distances and daily speeds of the planets and points
    Positions(PositionsArgs),
    /// Ascendant, midheaven, house cusps and the bodies placed in houses
    Chart(ChartArgs),
    /// Tithi, nakshatra, yoga, karana, vara, sunrise and sunset
    Panchang(PanchangArgs),
    /// Vimshottari mahadasha and antardasha periods from the Moon's nakshatra
    Dasha(DashaArgs),
    /// Position of a fixed star for a date
    Star(StarArgs),
    /// Solar and lunar eclipses in a date range
    Eclipses(EclipsesArgs),
    /// Rise, transit and set times of a body
    RiseSet(RiseSetArgs),
    /// Aspects between the bodies at an instant
    Aspects(AspectsArgs),
    /// Julian Days, delta T, sidereal time and every ayanamsa for a date
    Convert(ConvertArgs),
}

/// Output options shared by every command.
#[derive(Args, Debug, Clone)]
pub struct OutArgs {
    /// Print JSON instead of a table
    #[arg(long)]
    pub json: bool,

    /// Print times in this zone instead of UTC: UTC, an offset such as +05:30,
    /// or a city from the built-in table (standard time; no IANA names, no DST)
    #[arg(long, value_name = "ZONE", allow_hyphen_values = true)]
    pub tz: Option<String>,
}

impl OutArgs {
    /// The zone to print times in.
    pub fn zone(&self) -> Result<Tz> {
        match &self.tz {
            Some(text) => Tz::parse(text),
            None => Ok(Tz::utc()),
        }
    }
}

/// The moment to compute for.
#[derive(Args, Debug, Clone)]
pub struct TimeArgs {
    /// ISO-8601 instant, e.g. 2026-11-07T00:00:00Z or 2026-11-07T05:30:00+05:30
    /// (a date alone means midnight; no zone means UTC)
    #[arg(
        long,
        value_name = "ISO-8601",
        conflicts_with = "jd",
        allow_hyphen_values = true
    )]
    pub date: Option<String>,

    /// Julian Day in UT, e.g. 2451545.0
    #[arg(long, value_name = "JD", allow_negative_numbers = true)]
    pub jd: Option<f64>,
}

impl TimeArgs {
    /// Resolve `--date` or `--jd` into an instant.
    pub fn instant(&self, out: &OutArgs) -> Result<Instant> {
        let tz = out.zone()?;
        match (&self.date, self.jd) {
            (Some(text), _) => Instant::new(parse_iso(text)?, tz),
            (None, Some(jd)) => Instant::new(jd, tz),
            (None, None) => Err(CliError::usage(
                "missing --date (ISO-8601) or --jd (Julian Day UT)",
            )),
        }
    }
}

/// The observer's place.
#[derive(Args, Debug, Clone)]
pub struct LocArgs {
    /// Latitude in degrees, north positive
    #[arg(long, value_name = "DEG", allow_negative_numbers = true)]
    pub lat: Option<f64>,

    /// Longitude in degrees, east positive
    #[arg(long, value_name = "DEG", allow_negative_numbers = true)]
    pub lon: Option<f64>,

    /// Height above sea level in metres
    #[arg(
        long,
        value_name = "M",
        default_value_t = 0.0,
        allow_negative_numbers = true
    )]
    pub alt: f64,

    /// A city from the built-in table (instead of --lat and --lon)
    #[arg(long, value_name = "CITY", conflicts_with_all = ["lat", "lon"])]
    pub place: Option<String>,
}

impl LocArgs {
    /// Resolve the place. With `required` unset, no place at all is fine.
    pub fn resolve(&self, required: bool) -> Result<Option<Place>> {
        if !self.alt.is_finite() {
            return Err(CliError::usage("--alt must be a number of metres"));
        }
        if let Some(name) = &self.place {
            let city = name.trim();
            let found = city_coordinates(city).ok_or_else(|| {
                CliError::usage(format!(
                    "unknown place '{city}': it is not in the built-in city table; \
                     give --lat and --lon instead"
                ))
            })?;
            return Ok(Some(Place {
                lat: found.lat_deg(),
                lon: found.lon_deg(),
                alt: self.alt,
                name: Some(city.to_string()),
            }));
        }
        match (self.lat, self.lon) {
            (Some(lat), Some(lon)) => {
                if GeoLocation::try_new(lat, lon).is_none() {
                    return Err(CliError::usage(
                        "--lat must be within -90 to 90 and --lon within -180 to 180 degrees",
                    ));
                }
                Ok(Some(Place {
                    lat,
                    lon,
                    alt: self.alt,
                    name: None,
                }))
            }
            (None, None) if required => Err(CliError::usage(
                "missing --lat and --lon (or use --place \"<city>\")",
            )),
            (None, None) => Ok(None),
            (None, Some(_)) => Err(CliError::usage(
                "missing --lat (it is needed together with --lon, or use --place)",
            )),
            (Some(_), None) => Err(CliError::usage(
                "missing --lon (it is needed together with --lat, or use --place)",
            )),
        }
    }
}

/// Tropical or sidereal zodiac.
#[derive(Args, Debug, Clone)]
pub struct ZodiacArgs {
    /// Use the sidereal zodiac, optionally naming the ayanamsa (default Lahiri)
    #[arg(
        long,
        value_name = "AYANAMSA",
        num_args = 0..=1,
        default_missing_value = "Lahiri"
    )]
    pub sidereal: Option<String>,
}

impl ZodiacArgs {
    /// The chosen ayanamsa, or `None` for the tropical zodiac.
    pub fn resolve(&self) -> Result<Option<Ayanamsa>> {
        self.sidereal.as_deref().map(parse_ayanamsa).transpose()
    }
}

/// Arguments of `positions`.
#[derive(Args, Debug)]
pub struct PositionsArgs {
    #[command(flatten)]
    pub time: TimeArgs,
    #[command(flatten)]
    pub zodiac: ZodiacArgs,
    /// Comma-separated bodies to show (default: all)
    #[arg(long, value_delimiter = ',', value_name = "LIST")]
    pub bodies: Vec<String>,
    #[command(flatten)]
    pub out: OutArgs,
}

/// Arguments of `chart`.
#[derive(Args, Debug)]
pub struct ChartArgs {
    #[command(flatten)]
    pub time: TimeArgs,
    #[command(flatten)]
    pub loc: LocArgs,
    #[command(flatten)]
    pub zodiac: ZodiacArgs,
    /// House system (default Placidus; WholeSign with --sidereal)
    #[arg(long, value_name = "SYSTEM")]
    pub house_system: Option<String>,
    /// Comma-separated bodies to place (default: all)
    #[arg(long, value_delimiter = ',', value_name = "LIST")]
    pub bodies: Vec<String>,
    #[command(flatten)]
    pub out: OutArgs,
}

/// Arguments of `panchang`.
#[derive(Args, Debug)]
pub struct PanchangArgs {
    #[command(flatten)]
    pub time: TimeArgs,
    #[command(flatten)]
    pub loc: LocArgs,
    /// Ayanamsa (default Lahiri)
    #[arg(long, value_name = "AYANAMSA")]
    pub ayanamsa: Option<String>,
    #[command(flatten)]
    pub out: OutArgs,
}

/// Arguments of `dasha`.
#[derive(Args, Debug)]
pub struct DashaArgs {
    #[command(flatten)]
    pub time: TimeArgs,
    /// Ayanamsa (default Lahiri)
    #[arg(long, value_name = "AYANAMSA")]
    pub ayanamsa: Option<String>,
    /// 1 for mahadashas only, 2 to add antardashas
    #[arg(
        long,
        value_name = "N",
        default_value_t = 2,
        value_parser = clap::value_parser!(u8).range(1..=2)
    )]
    pub levels: u8,
    #[command(flatten)]
    pub out: OutArgs,
}

/// Arguments of `star`.
#[derive(Args, Debug)]
pub struct StarArgs {
    /// Star name, e.g. Spica or "Alpha Centauri"
    #[arg(value_name = "NAME", required = true, num_args = 1..)]
    pub name: Vec<String>,
    #[command(flatten)]
    pub time: TimeArgs,
    #[command(flatten)]
    pub out: OutArgs,
}

/// Which eclipses to list.
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EclipseKind {
    /// Solar eclipses only
    Solar,
    /// Lunar eclipses only
    Lunar,
    /// Both
    All,
}

/// Arguments of `eclipses`.
#[derive(Args, Debug)]
pub struct EclipsesArgs {
    /// Start of the range (ISO-8601)
    #[arg(long, value_name = "ISO-8601", allow_hyphen_values = true)]
    pub from: String,
    /// End of the range (ISO-8601)
    #[arg(long, value_name = "ISO-8601", allow_hyphen_values = true)]
    pub to: String,
    /// Which eclipses to list
    #[arg(long = "type", value_enum, default_value_t = EclipseKind::All)]
    pub kind: EclipseKind,
    #[command(flatten)]
    pub out: OutArgs,
}

/// Arguments of `rise-set`.
#[derive(Args, Debug)]
pub struct RiseSetArgs {
    #[command(flatten)]
    pub time: TimeArgs,
    #[command(flatten)]
    pub loc: LocArgs,
    /// Sun, Moon or a planet
    #[arg(long, value_name = "BODY", default_value = "sun")]
    pub body: String,
    #[command(flatten)]
    pub out: OutArgs,
}

/// Arguments of `aspects`.
#[derive(Args, Debug)]
pub struct AspectsArgs {
    #[command(flatten)]
    pub time: TimeArgs,
    /// Comma-separated bodies to compare (default: Sun to Pluto and the mean node)
    #[arg(long, value_delimiter = ',', value_name = "LIST")]
    pub bodies: Vec<String>,
    /// Use this orb in degrees for every aspect instead of the library defaults
    #[arg(long, value_name = "DEG")]
    pub orb: Option<f64>,
    /// Include the minor aspects (semi-sextile, quintile, quincunx, ...)
    #[arg(long)]
    pub minor: bool,
    #[command(flatten)]
    pub out: OutArgs,
}

/// Arguments of `convert`.
#[derive(Args, Debug)]
pub struct ConvertArgs {
    #[command(flatten)]
    pub time: TimeArgs,
    #[command(flatten)]
    pub out: OutArgs,
}

fn example(command: &str) -> String {
    format!("Example:\n  xalen {command}")
}

/// Build the `clap` command, adding the help text that depends on the library
/// (the lists of ayanamsas and house systems).
pub fn build_command() -> clap::Command {
    let ayanamsas = format!(
        "Ayanamsas (name or Swiss Ephemeris number): {}",
        ayanamsa_names()
    );
    let houses = format!(
        "House systems (name or Swiss letter): {}",
        house_system_names()
    );
    let bodies = format!("Bodies: {POINT_NAMES}");
    Cli::command()
        .mut_subcommand("positions", |c| {
            c.after_help(format!(
                "{}\n\n{bodies}\n\n{ayanamsas}",
                example("positions --date 2026-11-07T00:00:00Z --sidereal --bodies sun,moon")
            ))
        })
        .mut_subcommand("chart", |c| {
            c.after_help(format!(
                "{}\n\n{houses}\n\n{ayanamsas}",
                example("chart --date 1990-03-15T06:30:00Z --place delhi --sidereal")
            ))
        })
        .mut_subcommand("panchang", |c| {
            c.after_help(format!(
                "{}\n\n{ayanamsas}",
                example("panchang --date 2026-11-07T00:00:00Z --place delhi --tz +05:30")
            ))
        })
        .mut_subcommand("dasha", |c| {
            c.after_help(format!(
                "{}\n\n{ayanamsas}",
                example("dasha --date 1990-03-15T06:30:00Z --levels 2")
            ))
        })
        .mut_subcommand("star", |c| {
            c.after_help(example("star Spica --date 2026-11-07T00:00:00Z"))
        })
        .mut_subcommand("eclipses", |c| {
            c.after_help(example(
                "eclipses --from 2024-01-01 --to 2024-12-31 --type solar",
            ))
        })
        .mut_subcommand("rise-set", |c| {
            c.after_help(format!(
                "{}\n\n{bodies}",
                example("rise-set --date 2026-11-07 --place delhi --body moon --tz +05:30")
            ))
        })
        .mut_subcommand("aspects", |c| {
            c.after_help(format!(
                "{}\n\n{bodies}",
                example("aspects --date 2026-11-07T00:00:00Z --orb 3")
            ))
        })
        .mut_subcommand("convert", |c| {
            c.after_help(format!(
                "{}\n\n{ayanamsas}",
                example("convert --date 2000-01-01T12:00:00Z")
            ))
        })
}
