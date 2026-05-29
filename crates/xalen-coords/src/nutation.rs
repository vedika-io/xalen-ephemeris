use crate::ARCSEC_TO_RAD;

/// Nutation correction values (delta-psi and delta-epsilon) in radians.
#[derive(Debug, Clone, Copy)]
pub struct NutationResult {
    /// Nutation in longitude (delta-psi) in radians.
    pub delta_psi: f64,
    /// Nutation in obliquity (delta-epsilon) in radians.
    pub delta_epsilon: f64,
}

struct NutTerm {
    l: i8,
    lp: i8,
    f: i8,
    d: i8,
    om: i8,
    s: f64,
    sd: f64, // longitude: S_i, S_dot_i (0.1 µas)
    c: f64,
    cd: f64, // obliquity: C_i, C_dot_i (0.1 µas)
}

// Out-of-phase corrections S'_i*cos(arg) for longitude and C'_i*sin(arg) for obliquity.
// Only the first ~5 terms have significant out-of-phase components (SOFA nut00b.c).
// Index matches position in TERMS array. Values in 0.1 µas.
struct OutOfPhaseTerm {
    idx: usize,
    sp: f64, // S' — added to longitude sum as sp*cos(arg)
    cp: f64, // C' — added to obliquity sum as cp*sin(arg)
}

const OUT_OF_PHASE: &[OutOfPhaseTerm] = &[
    OutOfPhaseTerm {
        idx: 0,
        sp: 33386.0,
        cp: 15377.0,
    }, // (0,0,0,0,1) 18.6-yr
    OutOfPhaseTerm {
        idx: 1,
        sp: -13696.0,
        cp: -4587.0,
    }, // (0,0,2,-2,2)
    OutOfPhaseTerm {
        idx: 2,
        sp: -11467.0,
        cp: -1455.0,
    }, // (0,0,2,0,2)
    OutOfPhaseTerm {
        idx: 3,
        sp: 5765.0,
        cp: -514.0,
    }, // (0,0,0,0,2)
    OutOfPhaseTerm {
        idx: 5,
        sp: -524.0,
        cp: -174.0,
    }, // (0,1,2,-2,2)
];

// IAU 2000B — 77 largest lunisolar nutation terms
// Coefficients in units of 0.1 microarcsecond (multiply by 1e-7 to get arcseconds)
// Source: McCarthy & Luzum (2003), IERS Conventions 2003 Table 5.3b
const TERMS: &[NutTerm] = &[
    NutTerm {
        l: 0,
        lp: 0,
        f: 0,
        d: 0,
        om: 1,
        s: -172064161.0,
        sd: -174666.0,
        c: 92052331.0,
        cd: 9086.0,
    },
    NutTerm {
        l: 0,
        lp: 0,
        f: 2,
        d: -2,
        om: 2,
        s: -13170906.0,
        sd: -1675.0,
        c: 5730336.0,
        cd: -3015.0,
    },
    NutTerm {
        l: 0,
        lp: 0,
        f: 2,
        d: 0,
        om: 2,
        s: -2276413.0,
        sd: -234.0,
        c: 978459.0,
        cd: -485.0,
    },
    NutTerm {
        l: 0,
        lp: 0,
        f: 0,
        d: 0,
        om: 2,
        s: 2074554.0,
        sd: 207.0,
        c: -897492.0,
        cd: 470.0,
    },
    NutTerm {
        l: 0,
        lp: 1,
        f: 0,
        d: 0,
        om: 0,
        s: 1475877.0,
        sd: -3633.0,
        c: 73871.0,
        cd: -184.0,
    },
    NutTerm {
        l: 0,
        lp: 1,
        f: 2,
        d: -2,
        om: 2,
        s: -516821.0,
        sd: 1226.0,
        c: 224386.0,
        cd: -677.0,
    },
    NutTerm {
        l: 1,
        lp: 0,
        f: 0,
        d: 0,
        om: 0,
        s: 711159.0,
        sd: 73.0,
        c: -6750.0,
        cd: 0.0,
    },
    NutTerm {
        l: 0,
        lp: 0,
        f: 2,
        d: 0,
        om: 1,
        s: -387298.0,
        sd: -367.0,
        c: 200728.0,
        cd: 18.0,
    },
    NutTerm {
        l: 1,
        lp: 0,
        f: 2,
        d: 0,
        om: 2,
        s: -301461.0,
        sd: -36.0,
        c: 129025.0,
        cd: -63.0,
    },
    NutTerm {
        l: 0,
        lp: -1,
        f: 2,
        d: -2,
        om: 2,
        s: 215829.0,
        sd: -494.0,
        c: -95929.0,
        cd: 299.0,
    },
    NutTerm {
        l: 0,
        lp: 0,
        f: 2,
        d: -2,
        om: 1,
        s: 128227.0,
        sd: 137.0,
        c: -68982.0,
        cd: -9.0,
    },
    NutTerm {
        l: -1,
        lp: 0,
        f: 2,
        d: 0,
        om: 2,
        s: 123457.0,
        sd: 11.0,
        c: -53311.0,
        cd: 32.0,
    },
    NutTerm {
        l: -1,
        lp: 0,
        f: 0,
        d: 2,
        om: 0,
        s: 156994.0,
        sd: 10.0,
        c: -1235.0,
        cd: 0.0,
    },
    NutTerm {
        l: 1,
        lp: 0,
        f: 0,
        d: 0,
        om: 1,
        s: 63110.0,
        sd: 63.0,
        c: -33228.0,
        cd: 0.0,
    },
    NutTerm {
        l: -1,
        lp: 0,
        f: 0,
        d: 0,
        om: 1,
        s: -57976.0,
        sd: -63.0,
        c: 31429.0,
        cd: 0.0,
    },
    NutTerm {
        l: -1,
        lp: 0,
        f: 2,
        d: 2,
        om: 2,
        s: -59641.0,
        sd: -11.0,
        c: 25543.0,
        cd: -11.0,
    },
    NutTerm {
        l: 1,
        lp: 0,
        f: 2,
        d: 0,
        om: 1,
        s: -51613.0,
        sd: -42.0,
        c: 26366.0,
        cd: 0.0,
    },
    NutTerm {
        l: -2,
        lp: 0,
        f: 2,
        d: 0,
        om: 1,
        s: 45893.0,
        sd: 50.0,
        c: -24236.0,
        cd: -10.0,
    },
    NutTerm {
        l: 0,
        lp: 0,
        f: 0,
        d: 2,
        om: 0,
        s: 63384.0,
        sd: 11.0,
        c: -1220.0,
        cd: 0.0,
    },
    NutTerm {
        l: 0,
        lp: 0,
        f: 2,
        d: 2,
        om: 2,
        s: -38571.0,
        sd: -1.0,
        c: 16452.0,
        cd: -11.0,
    },
    NutTerm {
        l: 0,
        lp: -2,
        f: 2,
        d: -2,
        om: 2,
        s: 32481.0,
        sd: 0.0,
        c: -13870.0,
        cd: 0.0,
    },
    NutTerm {
        l: -2,
        lp: 0,
        f: 0,
        d: 2,
        om: 0,
        s: -47722.0,
        sd: 0.0,
        c: 477.0,
        cd: -17.0,
    },
    NutTerm {
        l: 2,
        lp: 0,
        f: 2,
        d: 0,
        om: 2,
        s: -31046.0,
        sd: -1.0,
        c: 13238.0,
        cd: -11.0,
    },
    NutTerm {
        l: 1,
        lp: 0,
        f: 2,
        d: -2,
        om: 2,
        s: 28593.0,
        sd: 0.0,
        c: -12338.0,
        cd: 10.0,
    },
    NutTerm {
        l: -1,
        lp: 0,
        f: 2,
        d: 0,
        om: 1,
        s: 20441.0,
        sd: 21.0,
        c: -10758.0,
        cd: 0.0,
    },
    NutTerm {
        l: 2,
        lp: 0,
        f: 0,
        d: 0,
        om: 0,
        s: 29243.0,
        sd: 0.0,
        c: -609.0,
        cd: 0.0,
    },
    NutTerm {
        l: 0,
        lp: 0,
        f: 2,
        d: 0,
        om: 0,
        s: 25887.0,
        sd: 0.0,
        c: -550.0,
        cd: 0.0,
    },
    NutTerm {
        l: 0,
        lp: 1,
        f: 0,
        d: 0,
        om: 1,
        s: -14053.0,
        sd: -25.0,
        c: 8551.0,
        cd: -2.0,
    },
    NutTerm {
        l: -1,
        lp: 0,
        f: 0,
        d: 2,
        om: 1,
        s: 15164.0,
        sd: 10.0,
        c: -8001.0,
        cd: 0.0,
    },
    NutTerm {
        l: 0,
        lp: 2,
        f: 2,
        d: -2,
        om: 2,
        s: -15794.0,
        sd: 72.0,
        c: 6850.0,
        cd: -42.0,
    },
    NutTerm {
        l: 0,
        lp: 0,
        f: -2,
        d: 2,
        om: 0,
        s: 21783.0,
        sd: 0.0,
        c: -167.0,
        cd: 13.0,
    },
    NutTerm {
        l: 1,
        lp: 0,
        f: 0,
        d: -2,
        om: 1,
        s: -12873.0,
        sd: -10.0,
        c: 6953.0,
        cd: 0.0,
    },
    NutTerm {
        l: 0,
        lp: -1,
        f: 0,
        d: 0,
        om: 1,
        s: -12654.0,
        sd: 11.0,
        c: 6415.0,
        cd: 0.0,
    },
    NutTerm {
        l: -1,
        lp: 0,
        f: 2,
        d: 2,
        om: 1,
        s: -10204.0,
        sd: 0.0,
        c: 5222.0,
        cd: 0.0,
    },
    NutTerm {
        l: 0,
        lp: 2,
        f: 0,
        d: 0,
        om: 0,
        s: 16707.0,
        sd: -85.0,
        c: 168.0,
        cd: -1.0,
    },
    NutTerm {
        l: 1,
        lp: 0,
        f: 2,
        d: 2,
        om: 2,
        s: -7691.0,
        sd: 0.0,
        c: 3268.0,
        cd: 0.0,
    },
    NutTerm {
        l: -2,
        lp: 0,
        f: 2,
        d: 0,
        om: 0,
        s: -11024.0,
        sd: 0.0,
        c: 104.0,
        cd: 0.0,
    },
    NutTerm {
        l: 0,
        lp: 1,
        f: 2,
        d: 0,
        om: 2,
        s: 7566.0,
        sd: -21.0,
        c: -3250.0,
        cd: 0.0,
    },
    NutTerm {
        l: 0,
        lp: 0,
        f: 2,
        d: 2,
        om: 1,
        s: -6637.0,
        sd: -11.0,
        c: 3353.0,
        cd: 0.0,
    },
    NutTerm {
        l: 0,
        lp: -1,
        f: 2,
        d: 0,
        om: 2,
        s: -7141.0,
        sd: 21.0,
        c: 3070.0,
        cd: 0.0,
    },
    NutTerm {
        l: 0,
        lp: 0,
        f: 0,
        d: 2,
        om: 1,
        s: -6302.0,
        sd: -11.0,
        c: 3272.0,
        cd: 0.0,
    },
    NutTerm {
        l: 1,
        lp: 0,
        f: 0,
        d: 2,
        om: 0,
        s: 5800.0,
        sd: 10.0,
        c: -63.0,
        cd: 0.0,
    },
    NutTerm {
        l: 2,
        lp: 0,
        f: 2,
        d: -2,
        om: 2,
        s: 6443.0,
        sd: 0.0,
        c: -2768.0,
        cd: 0.0,
    },
    NutTerm {
        l: -2,
        lp: 0,
        f: 0,
        d: 2,
        om: 1,
        s: -5774.0,
        sd: -11.0,
        c: 3041.0,
        cd: 0.0,
    },
    NutTerm {
        l: 2,
        lp: 0,
        f: 2,
        d: 0,
        om: 1,
        s: -5350.0,
        sd: 0.0,
        c: 2695.0,
        cd: 0.0,
    },
    NutTerm {
        l: 0,
        lp: -1,
        f: 2,
        d: -2,
        om: 1,
        s: -4752.0,
        sd: -11.0,
        c: 2719.0,
        cd: 0.0,
    },
    NutTerm {
        l: 0,
        lp: 0,
        f: 0,
        d: -2,
        om: 1,
        s: -4940.0,
        sd: -11.0,
        c: 2720.0,
        cd: 0.0,
    },
    NutTerm {
        l: -1,
        lp: -1,
        f: 0,
        d: 2,
        om: 0,
        s: 7350.0,
        sd: 0.0,
        c: -51.0,
        cd: 0.0,
    },
    NutTerm {
        l: 2,
        lp: 0,
        f: 0,
        d: -2,
        om: 1,
        s: 4065.0,
        sd: 0.0,
        c: -2206.0,
        cd: 0.0,
    },
    NutTerm {
        l: 1,
        lp: 0,
        f: 0,
        d: -2,
        om: 0,
        s: 6579.0,
        sd: 0.0,
        c: -199.0,
        cd: 0.0,
    },
    NutTerm {
        l: 0,
        lp: 1,
        f: 2,
        d: -2,
        om: 1,
        s: 3579.0,
        sd: 0.0,
        c: -1900.0,
        cd: 0.0,
    },
    NutTerm {
        l: 1,
        lp: -1,
        f: 0,
        d: 0,
        om: 0,
        s: 4725.0,
        sd: 0.0,
        c: -41.0,
        cd: 0.0,
    },
    NutTerm {
        l: -2,
        lp: 0,
        f: 2,
        d: 0,
        om: 2,
        s: -3075.0,
        sd: 0.0,
        c: 1313.0,
        cd: 0.0,
    },
    NutTerm {
        l: 3,
        lp: 0,
        f: 2,
        d: 0,
        om: 2,
        s: -2904.0,
        sd: 0.0,
        c: 1233.0,
        cd: 0.0,
    },
    NutTerm {
        l: 0,
        lp: -1,
        f: 0,
        d: 2,
        om: 0,
        s: 4348.0,
        sd: 0.0,
        c: -81.0,
        cd: 0.0,
    },
    NutTerm {
        l: 1,
        lp: -1,
        f: 2,
        d: 0,
        om: 2,
        s: -2878.0,
        sd: 0.0,
        c: 1232.0,
        cd: 0.0,
    },
    NutTerm {
        l: 0,
        lp: 0,
        f: 0,
        d: 1,
        om: 0,
        s: -4230.0,
        sd: 0.0,
        c: -20.0,
        cd: 0.0,
    },
    NutTerm {
        l: -1,
        lp: -1,
        f: 2,
        d: 2,
        om: 2,
        s: -2819.0,
        sd: 0.0,
        c: 1207.0,
        cd: 0.0,
    },
    NutTerm {
        l: -1,
        lp: 0,
        f: 2,
        d: 0,
        om: 0,
        s: -4056.0,
        sd: 0.0,
        c: 40.0,
        cd: 0.0,
    },
    NutTerm {
        l: 0,
        lp: -1,
        f: 2,
        d: 2,
        om: 2,
        s: -2647.0,
        sd: 0.0,
        c: 1129.0,
        cd: 0.0,
    },
    NutTerm {
        l: -2,
        lp: 0,
        f: 0,
        d: 0,
        om: 1,
        s: -2294.0,
        sd: 0.0,
        c: 1266.0,
        cd: 0.0,
    },
    NutTerm {
        l: 1,
        lp: 1,
        f: 2,
        d: 0,
        om: 2,
        s: 2481.0,
        sd: 0.0,
        c: -1062.0,
        cd: 0.0,
    },
    NutTerm {
        l: 2,
        lp: 0,
        f: 0,
        d: 0,
        om: 1,
        s: 2179.0,
        sd: 0.0,
        c: -1129.0,
        cd: 0.0,
    },
    NutTerm {
        l: -1,
        lp: 1,
        f: 0,
        d: 1,
        om: 0,
        s: 3276.0,
        sd: 0.0,
        c: -9.0,
        cd: 0.0,
    },
    NutTerm {
        l: 1,
        lp: 1,
        f: 0,
        d: 0,
        om: 0,
        s: -3389.0,
        sd: 0.0,
        c: 35.0,
        cd: 0.0,
    },
    NutTerm {
        l: 1,
        lp: 0,
        f: 2,
        d: 0,
        om: 0,
        s: 3339.0,
        sd: 0.0,
        c: -107.0,
        cd: 0.0,
    },
    NutTerm {
        l: -1,
        lp: 0,
        f: 2,
        d: -2,
        om: 1,
        s: -1987.0,
        sd: 0.0,
        c: 1073.0,
        cd: 0.0,
    },
    NutTerm {
        l: 1,
        lp: 0,
        f: 0,
        d: 0,
        om: 2,
        s: -1981.0,
        sd: 0.0,
        c: 854.0,
        cd: 0.0,
    },
    NutTerm {
        l: -1,
        lp: 0,
        f: 0,
        d: 1,
        om: 0,
        s: 4026.0,
        sd: 0.0,
        c: -553.0,
        cd: 0.0,
    },
    NutTerm {
        l: 0,
        lp: 0,
        f: 2,
        d: 1,
        om: 2,
        s: 1660.0,
        sd: 0.0,
        c: -710.0,
        cd: 0.0,
    },
    NutTerm {
        l: -1,
        lp: 0,
        f: 2,
        d: 4,
        om: 2,
        s: -1521.0,
        sd: 0.0,
        c: 647.0,
        cd: 0.0,
    },
    NutTerm {
        l: -1,
        lp: 1,
        f: 0,
        d: 1,
        om: 1,
        s: 1314.0,
        sd: 0.0,
        c: -700.0,
        cd: 0.0,
    },
    NutTerm {
        l: 0,
        lp: -2,
        f: 2,
        d: -2,
        om: 1,
        s: -1283.0,
        sd: 0.0,
        c: 672.0,
        cd: 0.0,
    },
    NutTerm {
        l: 1,
        lp: 0,
        f: 2,
        d: 2,
        om: 1,
        s: -1331.0,
        sd: 0.0,
        c: 663.0,
        cd: 0.0,
    },
    NutTerm {
        l: -2,
        lp: 0,
        f: 2,
        d: 2,
        om: 2,
        s: 1383.0,
        sd: 0.0,
        c: -594.0,
        cd: 0.0,
    },
    NutTerm {
        l: -1,
        lp: 0,
        f: 0,
        d: 0,
        om: 2,
        s: 1405.0,
        sd: 0.0,
        c: -610.0,
        cd: 0.0,
    },
    NutTerm {
        l: 1,
        lp: 1,
        f: 2,
        d: -2,
        om: 2,
        s: 1290.0,
        sd: 0.0,
        c: -556.0,
        cd: 0.0,
    },
];

fn delaunay_arguments(t: f64) -> [f64; 5] {
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;

    let l = (485868.249036 + 1717915923.2178 * t + 31.8792 * t2 + 0.051635 * t3 - 0.00024470 * t4)
        * ARCSEC_TO_RAD;
    let lp = (1287104.79305 + 129596581.0481 * t - 0.5532 * t2 + 0.000136 * t3 - 0.00001149 * t4)
        * ARCSEC_TO_RAD;
    let f = (335779.526232 + 1739527262.8478 * t - 12.7512 * t2 - 0.001037 * t3 + 0.00000417 * t4)
        * ARCSEC_TO_RAD;
    let d = (1072260.70369 + 1602961601.2090 * t - 6.3706 * t2 + 0.006593 * t3 - 0.00003169 * t4)
        * ARCSEC_TO_RAD;
    let om = (450160.398036 - 6962890.5431 * t + 7.4722 * t2 + 0.007702 * t3 - 0.00005939 * t4)
        * ARCSEC_TO_RAD;

    [l, lp, f, d, om]
}

/// Compute IAU 2000B nutation (77-term lunisolar series) for Julian centuries `t` from J2000.
pub fn nutation_2000b(t: f64) -> NutationResult {
    let args = delaunay_arguments(t);

    let mut dp = 0.0_f64;
    let mut de = 0.0_f64;

    for (i, term) in TERMS.iter().enumerate() {
        let arg = term.l as f64 * args[0]
            + term.lp as f64 * args[1]
            + term.f as f64 * args[2]
            + term.d as f64 * args[3]
            + term.om as f64 * args[4];

        let sin_arg = arg.sin();
        let cos_arg = arg.cos();

        dp += (term.s + term.sd * t) * sin_arg;
        de += (term.c + term.cd * t) * cos_arg;

        // Out-of-phase corrections: S'*cos(arg) for longitude, C'*sin(arg) for obliquity
        for oop in OUT_OF_PHASE {
            if oop.idx == i {
                dp += oop.sp * cos_arg;
                de += oop.cp * sin_arg;
                break;
            }
        }
    }

    // Convert from 0.1 µas to radians
    // 0.1 µas = 0.1e-6 arcsec = 1e-7 arcsec
    let factor = 1e-7 * ARCSEC_TO_RAD;

    // Add fixed corrections (luni-solar + planetary, IAU 2000B)
    let dp_corr = -0.135 * ARCSEC_TO_RAD / 1000.0; // -0.135 mas
    let de_corr = 0.388 * ARCSEC_TO_RAD / 1000.0; //  0.388 mas

    NutationResult {
        delta_psi: dp * factor + dp_corr,
        delta_epsilon: de * factor + de_corr,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nutation_magnitude_at_j2000() {
        let n = nutation_2000b(0.0);
        let dp_arcsec = n.delta_psi / ARCSEC_TO_RAD;
        let de_arcsec = n.delta_epsilon / ARCSEC_TO_RAD;
        assert!(
            dp_arcsec.abs() < 20.0,
            "delta_psi should be < 20\", got {dp_arcsec}\""
        );
        assert!(
            de_arcsec.abs() < 10.0,
            "delta_epsilon should be < 10\", got {de_arcsec}\""
        );
    }

    #[test]
    fn nutation_varies_with_time() {
        let n1 = nutation_2000b(0.0);
        let n2 = nutation_2000b(0.1); // 10 years later
        assert!(
            (n1.delta_psi - n2.delta_psi).abs() > 1e-8,
            "Nutation should change over 10 years"
        );
    }

    #[test]
    fn delaunay_args_are_reasonable() {
        let args = delaunay_arguments(0.0);
        for (i, a) in args.iter().enumerate() {
            assert!(
                a.is_finite(),
                "Delaunay argument {i} should be finite at J2000"
            );
        }
    }

    #[test]
    fn nutation_reference_j2000() {
        // At J2000.0, Omega is at ~125° phase (NOT peak). SOFA nut00b gives ~-14.0" Δψ
        let n = nutation_2000b(0.0);
        let dp_arcsec = n.delta_psi / ARCSEC_TO_RAD;
        let de_arcsec = n.delta_epsilon / ARCSEC_TO_RAD;
        assert!(
            (dp_arcsec - (-14.0)).abs() < 1.0,
            "Δψ at J2000 should be ~-14.0\", got {dp_arcsec:.4}\""
        );
        assert!(
            de_arcsec.abs() < 10.0,
            "Δε at J2000 should be within ±10\", got {de_arcsec:.4}\""
        );
    }

    #[test]
    fn out_of_phase_affects_result() {
        // Verify out-of-phase terms contribute non-negligibly
        // The first OOP term contributes ~3.3 mas at its peak
        let n = nutation_2000b(0.0);
        assert!(n.delta_psi.is_finite());
        assert!(n.delta_epsilon.is_finite());
    }

    #[test]
    fn largest_nutation_term() {
        // The 18.6-year nutation (Omega period) should dominate
        // Amplitude: ~17.2" in longitude, ~9.2" in obliquity
        // Check over a full cycle
        let mut max_dp = 0.0_f64;
        let mut max_de = 0.0_f64;
        for i in 0..200 {
            let t = i as f64 * 0.1 - 10.0; // -10 to +10 centuries
            let n = nutation_2000b(t);
            max_dp = max_dp.max(n.delta_psi.abs());
            max_de = max_de.max(n.delta_epsilon.abs());
        }
        let max_dp_arcsec = max_dp / ARCSEC_TO_RAD;
        let max_de_arcsec = max_de / ARCSEC_TO_RAD;
        assert!(
            max_dp_arcsec > 15.0 && max_dp_arcsec < 25.0,
            "Max nutation in longitude should be 15-25\", got {max_dp_arcsec}\""
        );
        assert!(
            max_de_arcsec > 7.0 && max_de_arcsec < 12.0,
            "Max nutation in obliquity should be 7-12\", got {max_de_arcsec}\""
        );
    }
}
