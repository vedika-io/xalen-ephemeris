mod calendar;
mod delta_t;
mod epoch;
mod julian;
pub mod leap_seconds;
mod timescale;
pub mod timezone;
mod utc;

pub use calendar::{CalendarSystem, calendar_to_jd, jd_to_calendar};
pub use delta_t::{DeltaTModel, delta_t, delta_t_with_uncertainty};
pub use epoch::Epoch;
pub use julian::{JdTDB, JdTT, JdUT1, JulianDay};
pub use leap_seconds::{
    FIRST_LEAP_SECOND_JD, month_ends_with_leap_second, tai_minus_utc_for_date,
    tai_minus_utc_seconds, utc_day_length_seconds,
};
pub use timescale::TimeScale;
pub use utc::{JdTAI, JdUTC, utc_calendar_to_jd};

/// Julian Date of the J2000.0 standard epoch (2000-01-01 12:00 TT).
pub const J2000_JD: f64 = 2_451_545.0;
/// Number of days in one Julian century (exactly 36525).
pub const DAYS_PER_JULIAN_CENTURY: f64 = 36_525.0;
/// Number of seconds in one day (exactly 86400).
pub const SECONDS_PER_DAY: f64 = 86_400.0;
