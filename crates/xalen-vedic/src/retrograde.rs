//! Retrograde detection and status classification.
//!
//! A planet is retrograde when its geocentric ecliptic longitude decreases
//! over time, i.e. its speed in degrees/day is negative. Near the points
//! where a planet switches between direct and retrograde motion it appears
//! to slow to near-zero speed — these brief intervals are called
//! "stationary" periods.
//!
//! The stationary threshold of 0.05 deg/day is a practical compromise:
//! - Inner planets (Mercury, Venus) slow below 0.1 for ~1 day around
//!   station, so 0.05 catches the core window.
//! - Outer planets (Mars through Saturn) have even smaller speeds at
//!   station.
//! - The Moon and Sun never retrograde; Rahu/Ketu are always retrograde
//!   (mean node) or oscillate (true node).

use serde::{Deserialize, Serialize};

/// Speed threshold (deg/day) below which a planet is considered
/// stationary rather than merely slow.
const STATIONARY_THRESHOLD: f64 = 0.05;

/// Whether a planet is retrograde given its daily speed in degrees.
///
/// A negative speed means the planet's ecliptic longitude is decreasing
/// (apparent backward motion against the fixed stars).
#[inline]
pub fn is_retrograde(speed_deg_per_day: f64) -> bool {
    speed_deg_per_day < 0.0
}

/// Motion status of a planet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MotionStatus {
    Direct,
    Retrograde,
    StationaryDirect,
    StationaryRetrograde,
}

impl MotionStatus {
    pub fn label(&self) -> &'static str {
        match self {
            MotionStatus::Direct => "Direct",
            MotionStatus::Retrograde => "Retrograde",
            MotionStatus::StationaryDirect => "Stationary Direct",
            MotionStatus::StationaryRetrograde => "Stationary Retrograde",
        }
    }

    pub fn is_retrograde(&self) -> bool {
        matches!(
            self,
            MotionStatus::Retrograde | MotionStatus::StationaryRetrograde
        )
    }

    pub fn is_stationary(&self) -> bool {
        matches!(
            self,
            MotionStatus::StationaryDirect | MotionStatus::StationaryRetrograde
        )
    }
}

/// Classify a planet's motion status from its speed in degrees per day.
///
/// Returns one of four states: Direct, Retrograde, Stationary Direct (about
/// to turn retrograde), or Stationary Retrograde (about to turn direct).
pub fn retrograde_status(speed_deg_per_day: f64) -> MotionStatus {
    let abs_speed = speed_deg_per_day.abs();
    if abs_speed < STATIONARY_THRESHOLD {
        if speed_deg_per_day < 0.0 {
            MotionStatus::StationaryRetrograde
        } else {
            MotionStatus::StationaryDirect
        }
    } else if speed_deg_per_day < 0.0 {
        MotionStatus::Retrograde
    } else {
        MotionStatus::Direct
    }
}

/// Convenience wrapper that accepts a planet name (unused for logic,
/// present for API symmetry with other modules) and a speed.
pub fn retrograde_status_for_planet(_planet: &str, speed: f64) -> MotionStatus {
    retrograde_status(speed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_motion() {
        assert!(!is_retrograde(0.5));
        assert_eq!(retrograde_status(0.5), MotionStatus::Direct);
        assert_eq!(retrograde_status(0.5).label(), "Direct");
    }

    #[test]
    fn retrograde_motion() {
        assert!(is_retrograde(-0.3));
        assert_eq!(retrograde_status(-0.3), MotionStatus::Retrograde);
        assert_eq!(retrograde_status(-0.3).label(), "Retrograde");
    }

    #[test]
    fn stationary_direct() {
        // Just barely positive, within threshold
        let status = retrograde_status(0.02);
        assert_eq!(status, MotionStatus::StationaryDirect);
        assert_eq!(status.label(), "Stationary Direct");
        assert!(status.is_stationary());
        assert!(!status.is_retrograde());
    }

    #[test]
    fn stationary_retrograde() {
        // Just barely negative, within threshold
        let status = retrograde_status(-0.01);
        assert_eq!(status, MotionStatus::StationaryRetrograde);
        assert_eq!(status.label(), "Stationary Retrograde");
        assert!(status.is_stationary());
        assert!(status.is_retrograde());
    }

    #[test]
    fn zero_speed_is_stationary_direct() {
        // Exactly zero = stationary, not negative → StationaryDirect
        assert_eq!(retrograde_status(0.0), MotionStatus::StationaryDirect);
    }

    #[test]
    fn threshold_boundary() {
        // At exactly the threshold, still stationary
        let at_thresh = retrograde_status(STATIONARY_THRESHOLD);
        assert_eq!(at_thresh, MotionStatus::Direct);

        let just_below = retrograde_status(STATIONARY_THRESHOLD - 0.001);
        assert_eq!(just_below, MotionStatus::StationaryDirect);
    }

    #[test]
    fn planet_name_variant() {
        assert_eq!(
            retrograde_status_for_planet("Mercury", -0.5),
            MotionStatus::Retrograde
        );
        assert_eq!(
            retrograde_status_for_planet("Jupiter", 0.1),
            MotionStatus::Direct
        );
    }

    #[test]
    fn typical_planet_speeds() {
        // Mercury fast direct
        assert_eq!(retrograde_status(1.4), MotionStatus::Direct);
        // Mercury retrograde
        assert_eq!(retrograde_status(-1.1), MotionStatus::Retrograde);
        // Saturn slow direct
        assert_eq!(retrograde_status(0.06), MotionStatus::Direct);
        // Saturn near station
        assert_eq!(retrograde_status(0.03), MotionStatus::StationaryDirect);
        // Saturn just turned retrograde
        assert_eq!(retrograde_status(-0.02), MotionStatus::StationaryRetrograde);
    }
}
