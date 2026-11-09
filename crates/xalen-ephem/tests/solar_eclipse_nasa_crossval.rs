use xalen_ephem::Almanac;
use xalen_ephem::eclipse::SolarEclipseType::{self, Annular, Partial, Total};
use xalen_ephem::eclipse::find_solar_eclipses;

type Row = (&'static str, &'static str, SolarEclipseType, f64, f64);

const GAMMA_TOLERANCE: f64 = 0.0025;
const GREATEST_TOLERANCE_S: f64 = 60.0;

const NASA: &[Row] = &[
    ("1902-05-07", "P", Partial, 2415877.440463, -1.0831),
    ("1928-05-19", "T-", Total, 2425386.058287, -1.0048),
    ("1950-03-18", "A-", Annular, 2433359.146898, -0.9988),
    ("1957-04-30", "A+", Annular, 2435958.503426, 0.9992),
    ("1957-10-23", "T-", Total, 2436134.703819, -1.0022),
    ("1967-11-02", "T-", Total, 2439796.734931, -1.0007),
    ("2014-04-29", "A-", Annular, 2456776.752361, -1.0000),
    ("2024-04-08", "T", Total, 2460409.261979, 0.3431),
    ("2043-04-09", "T+", Total, 2467349.289144, 1.0031),
    ("2043-10-03", "A-", Annular, 2467525.625243, -1.0102),
];

#[test]
fn solar_eclipse_types_match_nasa_including_non_central() {
    let almanac = Almanac::default_vedic();
    for &(date, code, kind, jd_greatest, gamma) in NASA {
        let found = find_solar_eclipses(&almanac, jd_greatest - 2.0, jd_greatest + 2.0);
        assert_eq!(found.len(), 1, "{date}: eclipses found");
        let e = &found[0];
        assert_eq!(e.eclipse_type, kind, "{date}: NASA type {code}");
        let dg = (e.gamma - gamma).abs();
        assert!(dg < GAMMA_TOLERANCE, "{date}: gamma off by {dg}");
        let dt = (e.jd_maximum - jd_greatest).abs() * 86400.0;
        assert!(
            dt < GREATEST_TOLERANCE_S,
            "{date}: greatest eclipse off by {dt:.1}s"
        );
    }
}
