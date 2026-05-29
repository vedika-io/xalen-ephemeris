use criterion::{Criterion, criterion_group, criterion_main};
use xalen_ephemeris::ayanamsa::Ayanamsa;
use xalen_ephemeris::coords::obliquity::mean_obliquity;
use xalen_ephemeris::ephem::{Almanac, Body};
use xalen_ephemeris::houses::{GeoLocation, HouseSystem, compute_houses};
use xalen_ephemeris::time::{DeltaTModel, JdUT1, JulianDay};
use xalen_ephemeris::vedic::dasha::{DashaLevel, vimshottari_dasha};
use xalen_ephemeris::vedic::nakshatra::Nakshatra;
use xalen_ephemeris::vedic::panchang::compute_panchang;

const J2000: f64 = 2_451_545.0;
// Pune, India
const PUNE_LAT: f64 = 18.52;
const PUNE_LON: f64 = 73.85;

fn bench_planet_longitude(c: &mut Criterion) {
    let almanac = Almanac::default_vedic();
    let jd = JdUT1(J2000);
    c.bench_function("planet_longitude_sun", |b| {
        b.iter(|| almanac.geocentric_longitude_deg(Body::Sun, jd).unwrap());
    });
}

fn bench_moon_longitude(c: &mut Criterion) {
    let almanac = Almanac::default_vedic();
    let jd = JdUT1(J2000);
    c.bench_function("moon_longitude", |b| {
        b.iter(|| almanac.geocentric_longitude_deg(Body::Moon, jd).unwrap());
    });
}

fn bench_full_chart(c: &mut Criterion) {
    let almanac = Almanac::default_vedic();
    let jd = JdUT1(J2000);
    let aya = Ayanamsa::Lahiri;
    let tt = jd.to_tt(&DeltaTModel::StephensonMorrisonHohenkerk2016);
    let aya_deg = aya.compute_deg(tt.as_f64());
    let loc = GeoLocation::new(PUNE_LAT, PUNE_LON);
    let t = (J2000 - 2_451_545.0) / 36525.0;
    let epsilon = mean_obliquity(t);

    c.bench_function("full_chart_9_planets_houses_ayanamsa", |b| {
        b.iter(|| {
            // All 9 Vedic planets sidereal
            for &body in Body::VEDIC_GRAHAS {
                let lon = almanac.sidereal_longitude_deg(body, jd, aya_deg).unwrap();
                let _nak = Nakshatra::from_longitude_deg(lon);
                let _pada = Nakshatra::pada(lon);
            }
            // Houses
            let _houses = compute_houses(J2000, &loc, epsilon, HouseSystem::WholeSign);
        });
    });
}

fn bench_house_cusps(c: &mut Criterion) {
    let loc = GeoLocation::new(PUNE_LAT, PUNE_LON);
    let t = (J2000 - 2_451_545.0) / 36525.0;
    let epsilon = mean_obliquity(t);

    c.bench_function("house_cusps_placidus_pune", |b| {
        b.iter(|| compute_houses(J2000, &loc, epsilon, HouseSystem::Placidus));
    });
}

fn bench_ayanamsa(c: &mut Criterion) {
    let aya = Ayanamsa::Lahiri;
    let jd = JdUT1(J2000);

    c.bench_function("ayanamsa_lahiri", |b| {
        b.iter(|| {
            let tt = jd.to_tt(&DeltaTModel::StephensonMorrisonHohenkerk2016);
            aya.compute_deg(tt.as_f64())
        });
    });
}

fn bench_nakshatra(c: &mut Criterion) {
    c.bench_function("nakshatra_from_moon_lon", |b| {
        b.iter(|| {
            let nak = Nakshatra::from_longitude_deg(100.0);
            let _pada = Nakshatra::pada(100.0);
            let _lord = nak.lord();
            let _deity = nak.deity();
        });
    });
}

fn bench_panchang(c: &mut Criterion) {
    let almanac = Almanac::default_vedic();
    let jd = JdUT1(J2000);
    let aya = Ayanamsa::Lahiri;
    let tt = jd.to_tt(&DeltaTModel::StephensonMorrisonHohenkerk2016);
    let aya_deg = aya.compute_deg(tt.as_f64());

    c.bench_function("panchang_full_5_limbs", |b| {
        b.iter(|| {
            let sun_deg = almanac
                .sidereal_longitude_deg(Body::Sun, jd, aya_deg)
                .unwrap();
            let moon_deg = almanac
                .sidereal_longitude_deg(Body::Moon, jd, aya_deg)
                .unwrap();
            compute_panchang(sun_deg, moon_deg, J2000)
        });
    });
}

fn bench_vimshottari_dasha(c: &mut Criterion) {
    c.bench_function("vimshottari_dasha_120yr_antardasha", |b| {
        b.iter(|| vimshottari_dasha(100.0, J2000, DashaLevel::Antardasha));
    });
}

criterion_group!(
    benches,
    bench_planet_longitude,
    bench_moon_longitude,
    bench_full_chart,
    bench_house_cusps,
    bench_ayanamsa,
    bench_nakshatra,
    bench_panchang,
    bench_vimshottari_dasha,
);
criterion_main!(benches);
