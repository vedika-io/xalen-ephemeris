//! Cross-validation harness: prints next_events vs Swiss oracle deltas in seconds.
use xalen_ephem::{Almanac, Body};
use xalen_time::{JdUT1, JulianDay};

type RiseSetCase = (&'static str, f64, f64, f64, f64, Body, f64, f64, f64);

fn main() {
    let a = Almanac::default_vedic();
    // (loc, lon, lat, elev, jd_start, body, swiss_rise, swiss_transit, swiss_set)
    let cases: &[RiseSetCase] = &[
        (
            "Delhi-Jun-Sun",
            77.2090,
            28.6139,
            216.0,
            2460482.5,
            Body::Sun,
            2460483.496030195,
            2460482.786833312,
            2460483.0777908363,
        ),
        (
            "Delhi-Jun-Moon",
            77.2090,
            28.6139,
            216.0,
            2460482.5,
            Body::Moon,
            2460483.065541837,
            2460483.276299793,
            2460483.4865981843,
        ),
        (
            "Delhi-Dec-Sun",
            77.2090,
            28.6139,
            216.0,
            2460665.5,
            Body::Sun,
            2460665.569261926,
            2460665.7842695178,
            2460665.99927599,
        ),
        (
            "Delhi-Dec-Moon",
            77.2090,
            28.6139,
            216.0,
            2460665.5,
            Body::Moon,
            2460666.243577112,
            2460666.5097756856,
            2460665.752418408,
        ),
        (
            "Delhi-Mar-Sun",
            77.2090,
            28.6139,
            216.0,
            2460389.5,
            Body::Sun,
            2460389.5381450094,
            2460389.7906495747,
            2460390.0434560175,
        ),
        (
            "Delhi-Mar-Moon",
            77.2090,
            28.6139,
            216.0,
            2460389.5,
            Body::Moon,
            2460389.864726836,
            2460390.1616545147,
            2460390.4542521285,
        ),
        (
            "London-Jun-Sun",
            -0.1276,
            51.5074,
            11.0,
            2460482.5,
            Body::Sun,
            2460482.6550674746,
            2460483.001689346,
            2460483.348295645,
        ),
        (
            "London-Jun-Moon",
            -0.1276,
            51.5074,
            11.0,
            2460482.5,
            Body::Moon,
            2460483.363964687,
            2460483.5000201254,
            2460482.6002419777,
        ),
        (
            "London-Dec-Sun",
            -0.1276,
            51.5074,
            11.0,
            2460665.5,
            Body::Sun,
            2460665.836117957,
            2460665.9991674577,
            2460666.1622180315,
        ),
        (
            "London-Dec-Moon",
            -0.1276,
            51.5074,
            11.0,
            2460665.5,
            Body::Moon,
            2460666.4562710966,
            2460665.7025849028,
            2460665.9867603537,
        ),
        (
            "Quito-Jun-Sun",
            -78.4678,
            -0.1807,
            2850.0,
            2460482.5,
            Body::Sun,
            2460482.9670312866,
            2460483.2193337195,
            2460483.4716355144,
        ),
        (
            "Quito-Jun-Moon",
            -78.4678,
            -0.1807,
            2850.0,
            2460482.5,
            Body::Moon,
            2460483.4664642387,
            2460482.6856126394,
            2460482.945551798,
        ),
        (
            "Quito-Dec-Sun",
            -78.4678,
            -0.1807,
            2850.0,
            2460665.5,
            Body::Sun,
            2460665.9640411884,
            2460666.2168543357,
            2460666.469666951,
        ),
        (
            "Quito-Dec-Moon",
            -78.4678,
            -0.1807,
            2850.0,
            2460665.5,
            Body::Moon,
            2460665.66966709,
            2460665.9263858595,
            2460666.183046383,
        ),
    ];
    let s = |x: Option<JdUT1>, sw: f64| match x {
        Some(j) => (j.as_f64() - sw).abs() * 86400.0,
        None => f64::NAN,
    };
    println!(
        "{:<18} {:>10} {:>10} {:>10}",
        "case", "rise_s", "transit_s", "set_s"
    );
    let mut maxr = 0.0f64;
    let mut maxt = 0.0f64;
    let mut maxs = 0.0f64;
    for (name, lon, lat, el, jd, body, sr, st, ss) in cases {
        let r = a
            .rise_transit_set_next(*body, JdUT1(*jd), *lat, *lon, *el)
            .unwrap();
        let dr = s(r.rise, *sr);
        let dt = s(r.transit, *st);
        let ds = s(r.set, *ss);
        if dr.is_finite() {
            maxr = maxr.max(dr);
        }
        if dt.is_finite() {
            maxt = maxt.max(dt);
        }
        if ds.is_finite() {
            maxs = maxs.max(ds);
        }
        println!("{:<18} {:>10.2} {:>10.2} {:>10.2}", name, dr, dt, ds);
    }
    println!(
        "MAX                {:>10.2} {:>10.2} {:>10.2}",
        maxr, maxt, maxs
    );
}
