//! Full Vedic chart: planets, houses, Vimshottari dasha, Ashtakavarga, Shadbala.
//!
//! Run with: cargo run --example vedic_chart

use xalen_ayanamsa::Ayanamsa;
use xalen_coords::{RAD_TO_DEG, obliquity::mean_obliquity};
use xalen_ephem::{Almanac, Body};
use xalen_houses::{GeoLocation, HouseSystem, compute_houses};
use xalen_time::{CalendarSystem, JulianDay, calendar_to_jd};
use xalen_vedic::dasha::{DashaLevel, vimshottari_dasha};
use xalen_vedic::nakshatra::Nakshatra;
use xalen_vedic::panchang::compute_panchang;
use xalen_vedic::rashi::Rashi;
use xalen_vedic::shadbala::Shadbala;

fn main() {
    // Birth data: 1 January 1990, 10:30 IST — Pune, India (18.52N, 73.86E)
    let hour_utc = 10.5 - 5.5; // IST to UTC
    let jd_ut1 = calendar_to_jd(1990, 1, 1, hour_utc, CalendarSystem::default());
    let location = GeoLocation::new(18.52, 73.86);

    let almanac = Almanac::default_vedic();
    let ayanamsa = Ayanamsa::Lahiri;
    let aya_deg = ayanamsa.compute_deg(jd_ut1.as_f64());

    // Obliquity for house computation
    let t = (jd_ut1.as_f64() - 2_451_545.0) / 36525.0;
    let epsilon = mean_obliquity(t);

    // Compute houses (Whole Sign for Vedic)
    let houses = compute_houses(jd_ut1.as_f64(), &location, epsilon, HouseSystem::WholeSign);
    let asc_deg = houses.ascendant.to_degrees().rem_euclid(360.0);
    let asc_sid = (asc_deg - aya_deg).rem_euclid(360.0);

    println!("=== Vedic Chart ===");
    println!("Date: 1 January 1990, 10:30 IST");
    println!("Location: Pune (18.52N, 73.86E)");
    println!("Ayanamsa (Lahiri): {:.4} deg", aya_deg);
    println!(
        "Lagna: {} ({:.2} deg sidereal)",
        Rashi::from_longitude_deg(asc_sid),
        asc_sid
    );
    println!();

    // Planet positions
    println!("--- Planetary Positions ---");
    let grahas = [
        (Body::Sun, "Sun"),
        (Body::Moon, "Moon"),
        (Body::Mars, "Mars"),
        (Body::Mercury, "Mercury"),
        (Body::Jupiter, "Jupiter"),
        (Body::Venus, "Venus"),
        (Body::Saturn, "Saturn"),
        (Body::MeanNode, "Rahu"),
    ];

    let mut moon_sid = 0.0_f64;
    let mut sun_sid = 0.0_f64;

    for &(body, name) in &grahas {
        let pos = almanac.geocentric_ecliptic(body, jd_ut1).unwrap();
        let tropical_deg = pos.longitude * RAD_TO_DEG;
        let sidereal_deg = (tropical_deg - aya_deg).rem_euclid(360.0);
        let rashi = Rashi::from_longitude_deg(sidereal_deg);
        let house = houses.planet_in_house(pos.longitude);
        let nak = Nakshatra::from_longitude_deg(sidereal_deg);

        if body == Body::Moon {
            moon_sid = sidereal_deg;
        }
        if body == Body::Sun {
            sun_sid = sidereal_deg;
        }

        println!(
            "  {:<9} {:>7.2} deg  {:20}  House {:>2}  {}",
            name,
            sidereal_deg,
            rashi.to_string(),
            house,
            nak
        );
    }

    // Ketu (always 180 deg from Rahu)
    let rahu_pos = almanac.geocentric_ecliptic(Body::MeanNode, jd_ut1).unwrap();
    let ketu_lon = Body::ketu_longitude(rahu_pos.longitude);
    let ketu_sid = (ketu_lon * RAD_TO_DEG - aya_deg).rem_euclid(360.0);
    println!(
        "  {:<9} {:>7.2} deg  {:20}  House {:>2}  {}",
        "Ketu",
        ketu_sid,
        Rashi::from_longitude_deg(ketu_sid).to_string(),
        houses.planet_in_house(ketu_lon),
        Nakshatra::from_longitude_deg(ketu_sid)
    );
    println!();

    // Panchang
    let panchang = compute_panchang(sun_sid, moon_sid, jd_ut1.as_f64());
    println!("--- Panchang ---");
    println!(
        "  Tithi:     {} ({:?} Paksha)",
        panchang.tithi.name(),
        panchang.tithi.paksha
    );
    println!("  Nakshatra: {}", panchang.nakshatra);
    println!("  Yoga:      {}", panchang.yoga.name());
    println!("  Karana:    {}", panchang.karana.name());
    println!("  Vara:      {:?}", panchang.vara);
    println!();

    // Vimshottari Dasha (Mahadasha + Antardasha)
    let dashas = vimshottari_dasha(moon_sid, jd_ut1.as_f64(), DashaLevel::Antardasha);
    println!("--- Vimshottari Dasha (first 3 Mahadashas) ---");
    for md in dashas.iter().take(3) {
        let years = (md.end_jd - md.start_jd) / 365.25;
        println!("  {} Mahadasha ({:.1} years)", md.lord, years);
        for ad in md.sub_periods.iter().take(3) {
            let months = (ad.end_jd - ad.start_jd) / 30.44;
            println!("    - {}/{} ({:.1} months)", md.lord, ad.lord, months);
        }
        if md.sub_periods.len() > 3 {
            println!("    ... and {} more", md.sub_periods.len() - 3);
        }
    }
    println!();

    // Shadbala (simplified — for Sun and Moon)
    println!("--- Shadbala ---");
    for &(body, name) in &[
        (Body::Sun, "Sun"),
        (Body::Moon, "Moon"),
        (Body::Jupiter, "Jupiter"),
        (Body::Saturn, "Saturn"),
    ] {
        let pos = almanac.geocentric_ecliptic(body, jd_ut1).unwrap();
        let sid = (pos.longitude * RAD_TO_DEG - aya_deg).rem_euclid(360.0);
        let house = houses.planet_in_house(pos.longitude);
        let sb = Shadbala::compute(name, sid, house, 1.0);
        println!(
            "  {:<9} total={:>6.1}  ratio={:.2}  {}",
            name,
            sb.total,
            sb.ratio,
            if sb.is_strong() { "STRONG" } else { "weak" }
        );
    }
}
