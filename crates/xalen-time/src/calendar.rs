use crate::julian::JdUT1;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
/// Calendar system used for date-to-JD and JD-to-date conversions.
#[derive(Default)]
pub enum CalendarSystem {
    /// Gregorian calendar extended before 1582 (proleptic).
    #[default]
    ProlepticGregorian,
    /// Julian calendar extended forward indefinitely (proleptic).
    ProlepticJulian,
    /// Julian calendar before the given JD, Gregorian after.
    JulianWithCutover { cutover_jd: f64 },
}


/// Convert a calendar date (year, month, day, fractional hour) to a Julian Date.
pub fn calendar_to_jd(
    year: i32,
    month: u32,
    day: u32,
    hour: f64,
    calendar: CalendarSystem,
) -> JdUT1 {
    match calendar {
        CalendarSystem::ProlepticGregorian => JdUT1(gregorian_to_jd(year, month, day, hour)),
        CalendarSystem::ProlepticJulian => JdUT1(julian_to_jd(year, month, day, hour)),
        CalendarSystem::JulianWithCutover { cutover_jd } => {
            let jd_greg = gregorian_to_jd(year, month, day, hour);
            if jd_greg < cutover_jd {
                JdUT1(julian_to_jd(year, month, day, hour))
            } else {
                JdUT1(jd_greg)
            }
        }
    }
}

/// Convert a Julian Date to a calendar date (year, month, day, fractional hour).
pub fn jd_to_calendar(jd: f64, calendar: CalendarSystem) -> (i32, u32, u32, f64) {
    match calendar {
        CalendarSystem::ProlepticGregorian => jd_to_gregorian(jd),
        CalendarSystem::ProlepticJulian => jd_to_julian(jd),
        CalendarSystem::JulianWithCutover { cutover_jd } => {
            if jd < cutover_jd {
                jd_to_julian(jd)
            } else {
                jd_to_gregorian(jd)
            }
        }
    }
}

// Meeus algorithm — Gregorian calendar to Julian Day Number
fn gregorian_to_jd(year: i32, month: u32, day: u32, hour: f64) -> f64 {
    let (y, m) = if month <= 2 {
        (year as f64 - 1.0, month as f64 + 12.0)
    } else {
        (year as f64, month as f64)
    };

    let a = (y / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();

    (365.25 * (y + 4716.0)).floor() + (30.6001 * (m + 1.0)).floor() + day as f64 + hour / 24.0 + b
        - 1524.5
}

fn julian_to_jd(year: i32, month: u32, day: u32, hour: f64) -> f64 {
    let (y, m) = if month <= 2 {
        (year as f64 - 1.0, month as f64 + 12.0)
    } else {
        (year as f64, month as f64)
    };

    (365.25 * (y + 4716.0)).floor() + (30.6001 * (m + 1.0)).floor() + day as f64 + hour / 24.0
        - 1524.5
}

fn jd_to_gregorian(jd: f64) -> (i32, u32, u32, f64) {
    let jd = jd + 0.5;
    let z = jd.floor() as i64;
    let f = jd - z as f64;

    // Proleptic Gregorian: ALWAYS apply the Gregorian correction,
    // regardless of whether z < 2299161 (Oct 15, 1582).
    // The previous code fell back to the Julian formula for dates
    // before the Gregorian adoption, which is wrong for proleptic Gregorian.
    let alpha = ((z as f64 - 1867216.25) / 36524.25).floor() as i64;
    let a = z + 1 + alpha - alpha / 4;

    let b = a + 1524;
    let c = ((b as f64 - 122.1) / 365.25).floor() as i64;
    let d = (365.25 * c as f64).floor() as i64;
    let e = ((b - d) as f64 / 30.6001).floor() as i64;

    let day = (b - d - (30.6001 * e as f64).floor() as i64) as u32;
    let month = if e < 14 { e - 1 } else { e - 13 } as u32;
    let year = if month > 2 { c - 4716 } else { c - 4715 } as i32;
    let hour = f * 24.0;

    (year, month, day, hour)
}

fn jd_to_julian(jd: f64) -> (i32, u32, u32, f64) {
    let jd = jd + 0.5;
    let z = jd.floor() as i64;
    let f = jd - z as f64;

    let a = z; // No Gregorian correction
    let b = a + 1524;
    let c = ((b as f64 - 122.1) / 365.25).floor() as i64;
    let d = (365.25 * c as f64).floor() as i64;
    let e = ((b - d) as f64 / 30.6001).floor() as i64;

    let day = (b - d - (30.6001 * e as f64).floor() as i64) as u32;
    let month = if e < 14 { e - 1 } else { e - 13 } as u32;
    let year = if month > 2 { c - 4716 } else { c - 4715 } as i32;
    let hour = f * 24.0;

    (year, month, day, hour)
}

#[allow(dead_code)]
/// Compute the Local Mean Time offset in hours for a given longitude.
pub fn lmt_offset_hours(longitude_deg: f64) -> f64 {
    longitude_deg / 15.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn j2000_epoch() {
        let jd = calendar_to_jd(2000, 1, 1, 12.0, CalendarSystem::ProlepticGregorian);
        assert!(
            (jd.0 - 2_451_545.0).abs() < 1e-6,
            "J2000 should be JD 2451545.0, got {}",
            jd.0
        );
    }

    #[test]
    fn gregorian_roundtrip() {
        let cases = [
            (2000, 1, 1, 12.0),
            (1990, 6, 15, 10.5),
            (1582, 10, 15, 0.0), // First Gregorian day
            (2026, 5, 24, 8.0),
        ];
        for (y, m, d, h) in cases {
            let jd = calendar_to_jd(y, m, d, h, CalendarSystem::ProlepticGregorian);
            let (y2, m2, d2, h2) = jd_to_calendar(jd.0, CalendarSystem::ProlepticGregorian);
            assert_eq!(y, y2, "Year mismatch for {y}-{m}-{d}");
            assert_eq!(m, m2, "Month mismatch for {y}-{m}-{d}");
            assert_eq!(d, d2, "Day mismatch for {y}-{m}-{d}");
            assert!(
                (h - h2).abs() < 0.001,
                "Hour mismatch for {y}-{m}-{d}: {h} vs {h2}"
            );
        }
    }

    #[test]
    fn julian_gregorian_differ() {
        // 1582-10-04 Julian = 1582-10-14 Gregorian (10-day gap)
        let jd_jul = calendar_to_jd(1582, 10, 4, 12.0, CalendarSystem::ProlepticJulian);
        let jd_greg = calendar_to_jd(1582, 10, 14, 12.0, CalendarSystem::ProlepticGregorian);
        assert!(
            (jd_jul.0 - jd_greg.0).abs() < 1.0,
            "Julian 1582-10-04 should be near Gregorian 1582-10-14"
        );
    }

    #[test]
    fn lmt_pune() {
        let offset = lmt_offset_hours(73.85); // Pune longitude
        assert!(
            (offset - 4.923).abs() < 0.01,
            "Pune LMT offset should be ~4.92h, got {offset}"
        );
    }

    #[test]
    fn proleptic_gregorian_roundtrip_year_1500() {
        // Year 1500 is before the Gregorian adoption (1582).
        // ProlepticGregorian must round-trip correctly at this date.
        let (y, m, d, h) = (1500, 6, 15, 12.0);
        let jd = calendar_to_jd(y, m, d, h, CalendarSystem::ProlepticGregorian);
        let (y2, m2, d2, h2) = jd_to_calendar(jd.0, CalendarSystem::ProlepticGregorian);
        assert_eq!(y, y2, "Year mismatch for proleptic Gregorian 1500-06-15");
        assert_eq!(m, m2, "Month mismatch for proleptic Gregorian 1500-06-15");
        assert_eq!(d, d2, "Day mismatch for proleptic Gregorian 1500-06-15");
        assert!(
            (h - h2).abs() < 0.001,
            "Hour mismatch for proleptic Gregorian 1500-06-15: {h} vs {h2}"
        );
    }

    #[test]
    fn proleptic_gregorian_roundtrip_at_jd_2299161_boundary() {
        // JD 2299161 = the Gregorian cutover boundary (Oct 15, 1582 Gregorian).
        // ProlepticGregorian must use the Gregorian formula on BOTH sides of this boundary.
        let boundary_jd = 2299161.0;

        // Just below the boundary
        let (y1, m1, d1, h1) =
            jd_to_calendar(boundary_jd - 1.0, CalendarSystem::ProlepticGregorian);
        let jd_back = calendar_to_jd(y1, m1, d1, h1, CalendarSystem::ProlepticGregorian);
        assert!(
            (jd_back.0 - (boundary_jd - 1.0)).abs() < 1e-6,
            "ProlepticGregorian round-trip below JD 2299161 failed: {} vs {}",
            jd_back.0,
            boundary_jd - 1.0
        );

        // Exactly at the boundary
        let (y2, m2, d2, h2) = jd_to_calendar(boundary_jd, CalendarSystem::ProlepticGregorian);
        let jd_back2 = calendar_to_jd(y2, m2, d2, h2, CalendarSystem::ProlepticGregorian);
        assert!(
            (jd_back2.0 - boundary_jd).abs() < 1e-6,
            "ProlepticGregorian round-trip at JD 2299161 failed: {} vs {}",
            jd_back2.0,
            boundary_jd
        );

        // Just above the boundary
        let (y3, m3, d3, h3) =
            jd_to_calendar(boundary_jd + 1.0, CalendarSystem::ProlepticGregorian);
        let jd_back3 = calendar_to_jd(y3, m3, d3, h3, CalendarSystem::ProlepticGregorian);
        assert!(
            (jd_back3.0 - (boundary_jd + 1.0)).abs() < 1e-6,
            "ProlepticGregorian round-trip above JD 2299161 failed: {} vs {}",
            jd_back3.0,
            boundary_jd + 1.0
        );
    }
}
