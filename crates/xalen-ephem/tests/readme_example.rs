//! Guards the headline example in the repo README so it can never silently
//! break (wrong import, wrong output) again. If this test changes, update the
//! README code block to match.
use xalen_ephem::{Almanac, Body};
use xalen_time::{calendar_to_jd, CalendarSystem, JulianDay};
use xalen_ayanamsa::Ayanamsa;
use xalen_vedic::nakshatra::Nakshatra;

#[test]
fn readme_headline_example_compiles_and_prints_swati() {
    let jd = calendar_to_jd(1990, 3, 15, 12.0 - 5.5, CalendarSystem::default());
    let almanac = Almanac::default_vedic();
    let pos = almanac.geocentric_ecliptic(Body::Moon, jd).unwrap();
    let sid = (pos.longitude.to_degrees() - Ayanamsa::Lahiri.compute_deg(jd.as_f64())).rem_euclid(360.0);
    let nak = Nakshatra::from_longitude_deg(sid);
    assert_eq!(format!("{nak}"), "Swati", "README example output drifted; got sid={sid:.4}");
}
