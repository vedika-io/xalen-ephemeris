//! BaZi (八字, Four Pillars) validation fixtures.
//!
//! ## Oracle provenance — READ BEFORE EDITING
//!
//! The Python BaZi oracle libraries `sxtwl` and `lunar-python` were **not
//! importable** in the harness that produced these fixtures, so no automated
//! third-party calculator was available. Rather than fabricate reference
//! values, every expected pillar below is derived from one of two
//! independently verifiable sources:
//!
//! 1. **Day pillar** — the continuous sexagenary day count. Cross-checked
//!    against the public formula published at
//!    `ytliu0.github.io/ChineseCalendar/sexagenary.html`:
//!    for a Julian Day Number `JDN` taken at noon,
//!    `stem = (JDN + 9) % 10` (0 = Jia) and `branch = (JDN + 1) % 12`
//!    (0 = Zi). XALEN's `sexagenary_day` (anchored at JD 2451544.5 = Wu-Wu,
//!    1 Jan 2000) was verified to reproduce that formula exactly across the
//!    JDs used here.
//!
//! 2. **Year / month / hour pillars** — derived from the canonical
//!    sexagenary-cycle anchor (4 CE = Jia-Zi), the Five-Tigers-Escape month
//!    table, the Five-Rats hour table, and the solar-term (Jie Qi) month
//!    boundaries. The Julian Days themselves were generated with
//!    **pyswisseph 2.10.03** (`swe.julday(y, m, d, 12.0)` → noon-UT JD) and
//!    the Sun's apparent ecliptic longitude with `swe.calc_ut(jd, SUN,
//!    FLG_SWIEPH)`, which confirms the BaZi solar month for each chart.
//!
//! 1984 is the textbook **Jia-Zi cycle restart** year (Wood Rat), so Chart C
//! pins the year-pillar phase against a date every practitioner can check.
//!
//! All charts use **noon UT** as the instant. XALEN's `compute_bazi` does NOT
//! apply a longitude/timezone or true-solar-time correction — these fixtures
//! validate the pillar arithmetic at the given UT instant, not local-mean-time
//! handling. That is an honest scope limit, not a hidden assumption.

use xalen_chinese::{EarthlyBranch, HeavenlyStem, compute_bazi};

// --- Chart A: 2024-06-15 12:00 UT ------------------------------------------
// JD (pyswisseph swe.julday(2024,6,15,12.0)) = 2460477.0
// Sun apparent longitude = 84.876° → BaZi month 5 (Wu / Horse).
// Year 2024 (after Li Chun) = Jia-Chen.
// Day pillar (ytliu0 oracle, JDN 2460477) = Geng-Xu.
// Month: Jia year → base 0, month 5 → stem (0+5+1)%10 = 6 = Geng; branch Wu.
// Hour: noon → branch Wu (idx 6); day stem Geng(6) → base (6%5)*2=2 → stem (2+6)%10=8 = Ren.
#[test]
fn chart_a_2024_06_15_noon() {
    let chart = compute_bazi(2024, 2_460_477.0, 12.0);

    assert_eq!(chart.year.stem, HeavenlyStem::Jia, "year stem");
    assert_eq!(
        chart.year.branch,
        EarthlyBranch::Chen,
        "year branch (Dragon)"
    );

    assert_eq!(chart.month.stem, HeavenlyStem::Geng, "month stem");
    assert_eq!(
        chart.month.branch,
        EarthlyBranch::Wu,
        "month branch (Horse)"
    );

    assert_eq!(chart.day.stem, HeavenlyStem::Geng, "day stem");
    assert_eq!(chart.day.branch, EarthlyBranch::Xu, "day branch (Dog)");

    assert_eq!(chart.hour.stem, HeavenlyStem::Ren, "hour stem");
    assert_eq!(chart.hour.branch, EarthlyBranch::Wu, "hour branch (Horse)");

    assert_eq!(chart.day_master, chart.day.stem, "day master == day stem");
}

// --- Chart B: 2000-06-15 12:00 UT ------------------------------------------
// JD (pyswisseph) = 2451711.0. Sun apparent longitude = 84.68° → month 5.
// Year 2000 (after Li Chun) = Geng-Chen.
// Day pillar (ytliu0 oracle, JDN 2451711) = Jia-Chen.
// Month: Geng year(idx 6) → base 2, month 5 → stem (2+5+1)%10 = 8 = Ren; branch Wu.
// Hour: noon → branch Wu; day stem Jia(0) → base 0 → stem (0+6)%10 = 6 = Geng.
#[test]
fn chart_b_2000_06_15_noon() {
    let chart = compute_bazi(2000, 2_451_711.0, 12.0);

    assert_eq!(chart.year.stem, HeavenlyStem::Geng, "year stem");
    assert_eq!(
        chart.year.branch,
        EarthlyBranch::Chen,
        "year branch (Dragon)"
    );

    assert_eq!(chart.month.stem, HeavenlyStem::Ren, "month stem");
    assert_eq!(
        chart.month.branch,
        EarthlyBranch::Wu,
        "month branch (Horse)"
    );

    assert_eq!(chart.day.stem, HeavenlyStem::Jia, "day stem");
    assert_eq!(chart.day.branch, EarthlyBranch::Chen, "day branch (Dragon)");

    assert_eq!(chart.hour.stem, HeavenlyStem::Geng, "hour stem");
    assert_eq!(chart.hour.branch, EarthlyBranch::Wu, "hour branch (Horse)");
}

// --- Chart C: 1984-06-15 12:00 UT (Jia-Zi cycle-restart year) ---------------
// JD (pyswisseph) = 2445867.0. Sun apparent longitude = 84.55° → month 5.
// 1984 is the canonical start of a new 60-year sexagenary cycle = Jia-Zi.
// Day pillar (ytliu0 oracle, JDN 2445867) = Geng-Chen.
// Month: Jia year → base 0, month 5 → stem 6 = Geng; branch Wu.
// Hour: noon → branch Wu; day stem Geng(6) → base 2 → stem 8 = Ren.
#[test]
fn chart_c_1984_jia_zi_cycle_restart() {
    let chart = compute_bazi(1984, 2_445_867.0, 12.0);

    assert_eq!(
        chart.year.stem,
        HeavenlyStem::Jia,
        "1984 must be the Jia-Zi cycle restart"
    );
    assert_eq!(
        chart.year.branch,
        EarthlyBranch::Zi,
        "1984 year branch (Rat)"
    );

    assert_eq!(chart.month.stem, HeavenlyStem::Geng, "month stem");
    assert_eq!(
        chart.month.branch,
        EarthlyBranch::Wu,
        "month branch (Horse)"
    );

    assert_eq!(chart.day.stem, HeavenlyStem::Geng, "day stem");
    assert_eq!(chart.day.branch, EarthlyBranch::Chen, "day branch (Dragon)");

    assert_eq!(chart.hour.stem, HeavenlyStem::Ren, "hour stem");
    assert_eq!(chart.hour.branch, EarthlyBranch::Wu, "hour branch (Horse)");
}

// --- Day-pillar anchor cross-check -----------------------------------------
// Pins the two documented sexagenary day anchors used by the ytliu0 oracle:
//   2000-01-01 (JD 2451544.5, midnight) = Wu-Wu
//   1970-01-01 (JD 2440587.5, midnight) = Xin-Si
// Both are .5 (midnight) JDs, exercising the `floor(jd - 2451544.5)` path.
#[test]
fn day_pillar_documented_anchors() {
    // 2000-01-01 midnight
    let chart_2000 = compute_bazi(2000, 2_451_544.5, 0.0);
    assert_eq!(
        chart_2000.day.stem,
        HeavenlyStem::Wu,
        "2000-01-01 day stem = Wu"
    );
    assert_eq!(
        chart_2000.day.branch,
        EarthlyBranch::Wu,
        "2000-01-01 day branch = Wu"
    );

    // 1970-01-01 midnight
    let chart_1970 = compute_bazi(1970, 2_440_587.5, 0.0);
    assert_eq!(
        chart_1970.day.stem,
        HeavenlyStem::Xin,
        "1970-01-01 day stem = Xin"
    );
    assert_eq!(
        chart_1970.day.branch,
        EarthlyBranch::Si,
        "1970-01-01 day branch = Si"
    );
}
