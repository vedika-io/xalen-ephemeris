//! Western chart: planets, aspects, essential dignities, Arabic lots.
//!
//! Run with: cargo run --example western_chart

use xalen_coords::{RAD_TO_DEG, obliquity::mean_obliquity};
use xalen_ephem::{Almanac, Body};
use xalen_houses::{GeoLocation, HouseSystem, compute_houses};
use xalen_time::{CalendarSystem, JulianDay, calendar_to_jd};
use xalen_western::aspects::{AspectType, find_all_aspects};
use xalen_western::dignity::essential_dignity_score;
use xalen_western::lots::{Lot, is_day_chart, part_of_fortune, part_of_spirit};

fn main() {
    // Birth data: 21 June 2000, 14:30 UTC — London (51.51N, -0.13E)
    let jd_ut1 = calendar_to_jd(2000, 6, 21, 14.5, CalendarSystem::default());
    let location = GeoLocation::new(51.51, -0.13);

    let almanac = Almanac::default_vedic();

    // Obliquity for house computation
    let t = (jd_ut1.as_f64() - 2_451_545.0) / 36525.0;
    let epsilon = mean_obliquity(t);

    // Compute Placidus houses (Western default)
    let houses = compute_houses(jd_ut1.as_f64(), &location, epsilon, HouseSystem::Placidus);
    let asc_deg = houses.ascendant.to_degrees().rem_euclid(360.0);
    let mc_deg = houses.mc.to_degrees().rem_euclid(360.0);

    println!("=== Western Chart ===");
    println!("Date: 21 June 2000, 14:30 UTC");
    println!("Location: London (51.51N, 0.13W)");
    println!("House system: Placidus");
    println!("Ascendant: {:.2} deg", asc_deg);
    println!("MC:        {:.2} deg", mc_deg);
    println!();

    // House cusps
    println!("--- House Cusps ---");
    for i in 0..12 {
        println!("  House {:>2}: {:>7.2} deg", i + 1, houses.cusp_deg(i));
    }
    println!();

    // Planet positions (tropical, since this is Western)
    println!("--- Planetary Positions (Tropical) ---");
    let bodies = [
        (Body::Sun, "Sun"),
        (Body::Moon, "Moon"),
        (Body::Mercury, "Mercury"),
        (Body::Venus, "Venus"),
        (Body::Mars, "Mars"),
        (Body::Jupiter, "Jupiter"),
        (Body::Saturn, "Saturn"),
        (Body::Uranus, "Uranus"),
        (Body::Neptune, "Neptune"),
        (Body::Pluto, "Pluto"),
    ];

    let sign_names = [
        "Aries",
        "Taurus",
        "Gemini",
        "Cancer",
        "Leo",
        "Virgo",
        "Libra",
        "Scorpio",
        "Sagittarius",
        "Capricorn",
        "Aquarius",
        "Pisces",
    ];

    let mut positions: Vec<(String, f64, f64)> = Vec::new();
    let mut sun_deg = 0.0_f64;
    let mut moon_deg = 0.0_f64;

    for &(body, name) in &bodies {
        let pos = almanac.geocentric_ecliptic(body, jd_ut1).unwrap();
        let lon_deg = pos.longitude * RAD_TO_DEG;
        let sign_idx = (lon_deg / 30.0) as usize % 12;
        let deg_in_sign = lon_deg % 30.0;
        let house = houses.planet_in_house(pos.longitude);

        // Essential dignity
        let dignity = essential_dignity_score(name, sign_idx, deg_in_sign);
        let dignity_str = match dignity {
            d if d >= 5 => "Domicile",
            d if d >= 4 => "Exaltation",
            d if d >= 3 => "Triplicity",
            d if d > 0 => "minor+",
            d if d <= -5 => "Peregrine",
            d if d <= -4 => "Fall",
            _ => "debil.",
        };

        if body == Body::Sun {
            sun_deg = lon_deg;
        }
        if body == Body::Moon {
            moon_deg = lon_deg;
        }

        // Speed placeholder (1.0 for Sun, varies by planet)
        let speed = match body {
            Body::Moon => 13.0,
            Body::Sun => 1.0,
            Body::Mercury => 1.2,
            Body::Venus => 1.1,
            _ => 0.5,
        };
        positions.push((name.to_string(), lon_deg, speed));

        println!(
            "  {:<9} {:>7.2} deg  {:>13} {:>5.1} deg  House {:>2}  dignity: {} ({})",
            name, lon_deg, sign_names[sign_idx], deg_in_sign, house, dignity_str, dignity
        );
    }
    println!();

    // Aspects (major only)
    let aspects = find_all_aspects(&positions, AspectType::MAJOR);
    println!("--- Aspects ---");
    for asp in &aspects {
        println!(
            "  {} {:?} {} (orb {:.1} deg, {:?})",
            asp.body1, asp.aspect_type, asp.body2, asp.orb_deg, asp.direction
        );
    }
    println!();

    // Arabic Lots
    let is_day = is_day_chart(sun_deg, asc_deg);
    let fortune = part_of_fortune(asc_deg, sun_deg, moon_deg, is_day);
    let spirit = part_of_spirit(asc_deg, sun_deg, moon_deg, is_day);
    let marriage = Lot::Marriage.compute(asc_deg, sun_deg, moon_deg, is_day);

    println!("--- Arabic Lots ---");
    println!(
        "  Chart sect:      {}",
        if is_day { "Day" } else { "Night" }
    );
    println!("  Part of Fortune: {:.2} deg", fortune);
    println!("  Part of Spirit:  {:.2} deg", spirit);
    println!("  Lot of Marriage: {:.2} deg", marriage);
}
