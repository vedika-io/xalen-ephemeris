//! Validation fixtures for xalen-world calendar/astrology systems.
//!
//! ## Oracle provenance — READ BEFORE EDITING
//!
//! The Python BaZi/Saju oracle libraries `sxtwl` and `lunar-python` were **not
//! importable** in the harness that produced these fixtures, so no automated
//! third-party Saju calculator was available. No reference value here is
//! invented. Each is derived from an independently verifiable source:
//!
//! * **Saju day pillar** — Korea uses the *same* continuous sexagenary day
//!   count as Chinese BaZi (only the stem/branch names differ). The expected
//!   values were cross-checked against the public Chinese sexagenary formula at
//!   `ytliu0.github.io/ChineseCalendar/sexagenary.html`
//!   (`stem = (JDN + 9) % 10`, `branch = (JDN + 1) % 12`, noon JDN) and confirmed
//!   to land on the *same index* as XALEN's Korean-named day pillar across the
//!   anchor dates 1970-01-01 (Sin-Sa = Xin-Si) and 2000-01-01 (Mu-O = Wu-Wu).
//!
//! * **Mahabote weekday** — the Burmese day-sign is purely the civil weekday.
//!   Every weekday below was generated with **pyswisseph 2.10.03**
//!   (`swe.day_of_week`, Mon=0..Sun=6, remapped to Sun=0..Sat=6) and matched
//!   XALEN's `floor(jd + 1.5) % 7` formula exactly on 11 consecutive dates.
//!
//! * **Mayan** — Long Count / Tzolkin anchored to the GMT correlation
//!   (JD 584283 = 0.0.0.0.0 = 4 Ahau) and the universally published end-of-
//!   13th-baktun date 2012-12-21 = 13.0.0.0.0. JDs from pyswisseph.
//!
//! * **Aztec** — Tonalpohualli anchored to the Caso correlation (13 Aug 1521
//!   Julian = JD 2276827.5 = 1 Coatl, fall of Tenochtitlan). JDs from
//!   pyswisseph.
//!
//! ## Saju year pillar + Daeun (see the `saju_solar_term_rules` module)
//!
//! Two Saju computations that were previously placeholders are now resolved
//! with the shared `xalen_chinese` solar-term machinery and validated here:
//!   1. **Ipchun (입춘) year boundary** — `compute_saju` resolves the year
//!      pillar against the Li Chun boundary (Sun at 315°, ≈ 4 Feb). A January
//!      birth correctly rolls back to the previous solar year.
//!   2. **Daeun (대운) start age** — `daeun` now uses the distance (in days)
//!      to the adjacent Jie Qi ÷ 3, in the direction that drives the cycle.
//! The Saju month pillar still takes a civil month (not a Jie-Qi-resolved
//! month) and is therefore not cross-validated against the BaZi month oracle.

// ===========================================================================
// Saju (Korean Four Pillars) — day-pillar cross-validation
// ===========================================================================
mod saju_day_pillar {
    use xalen_world::saju::{EarthlyBranch, HeavenlyStem, compute_saju};

    // ytliu0 sexagenary oracle, Korean names:
    //   1970-01-01 (JDN 2440588 noon) = Sin(7)-Sa(5)   [Xin-Si in Chinese]
    //   2000-01-01 (JDN 2451545 noon) = Mu(4)-O(6)     [Wu-Wu  in Chinese]
    #[test]
    fn day_pillar_matches_sexagenary_oracle() {
        let a = compute_saju(1970, 1, 1, 12);
        assert_eq!(
            a.day_pillar.0,
            HeavenlyStem::Sin,
            "1970-01-01 day stem (idx 7)"
        );
        assert_eq!(
            a.day_pillar.1,
            EarthlyBranch::Sa,
            "1970-01-01 day branch (idx 5)"
        );

        let b = compute_saju(2000, 1, 1, 12);
        assert_eq!(
            b.day_pillar.0,
            HeavenlyStem::Mu,
            "2000-01-01 day stem (idx 4)"
        );
        assert_eq!(
            b.day_pillar.1,
            EarthlyBranch::O,
            "2000-01-01 day branch (idx 6)"
        );
    }

    // Cross-check additional dates whose pillars were computed via the ytliu0
    // oracle (Korean index == Chinese index):
    //   2024-06-15 → Gyeong(6)-Sul(10)   [Geng-Xu]
    //   1990-03-15 → Gi(5)-Myo(3)        [Ji-Mao]
    #[test]
    fn day_pillar_extra_oracle_dates() {
        let c = compute_saju(2024, 6, 15, 12);
        assert_eq!(c.day_pillar.0, HeavenlyStem::Gyeong, "2024-06-15 day stem");
        assert_eq!(
            c.day_pillar.1,
            EarthlyBranch::Sul,
            "2024-06-15 day branch (Dog)"
        );

        let d = compute_saju(1990, 3, 15, 12);
        assert_eq!(d.day_pillar.0, HeavenlyStem::Gi, "1990-03-15 day stem");
        assert_eq!(
            d.day_pillar.1,
            EarthlyBranch::Myo,
            "1990-03-15 day branch (Rabbit)"
        );
    }

    // Day master == day stem, and its element matches the stem's element.
    #[test]
    fn day_master_consistency() {
        let chart = compute_saju(2024, 6, 15, 12);
        assert_eq!(chart.day_master, chart.day_pillar.0);
        assert_eq!(chart.day_master_element, chart.day_pillar.0.element());
    }
}

// ===========================================================================
// Saju — solar-term rules (year pillar Ipchun boundary + Daeun start age).
//
// These tests encode the CORRECT astrological behaviour and now PASS against
// the implementation (previously they were `#[ignore]`d known bugs). Each
// carries the oracle-derived value, mirrored in Python against the same Meeus
// solar-longitude approximation that `xalen_chinese` uses.
// ===========================================================================
mod saju_solar_term_rules {
    use xalen_world::saju::{EarthlyBranch, HeavenlyStem, compute_saju};

    // FIXED — Ipchun (입춘) year boundary now applied.
    //
    // A birth on 1990-01-20 is BEFORE Ipchun 1990 (≈ 4 Feb), so its Saju solar
    // year is 1989 = Gi-Sa (Earth Snake) [Chinese Ji-Si]. `compute_saju` now
    // resolves the year pillar against the Li Chun boundary (Sun at 315°,
    // reusing `xalen_chinese::li_chun_jd`) instead of the raw Gregorian year.
    //
    // Correct (oracle): year pillar = Gi(5)-Sa(5).  [was: Gyeong(6)-O(6)]
    #[test]
    fn ipchun_year_boundary_january_birth() {
        let chart = compute_saju(1990, 1, 20, 10);
        assert_eq!(
            chart.year_pillar.0,
            HeavenlyStem::Gi,
            "pre-Ipchun Jan birth: year stem should roll back to 1989 (Gi)"
        );
        assert_eq!(
            chart.year_pillar.1,
            EarthlyBranch::Sa,
            "pre-Ipchun Jan birth: year branch should roll back to 1989 (Sa / Snake)"
        );
    }

    // Companion: a birth AFTER Ipchun keeps the current Gregorian year, and a
    // June birth (well past Li Chun) is unaffected by the boundary. This pins the
    // post-fix contract: only Jan..~Feb-4 births roll back, and 1990-06-15
    // remains 1990 = Gyeong-O (Metal Horse).
    #[test]
    fn ipchun_post_boundary_keeps_gregorian_year() {
        let jan = compute_saju(1990, 1, 20, 10); // before Ipchun → 1989 (Gi-Sa)
        let jun = compute_saju(1990, 6, 15, 10); // after  Ipchun → 1990 (Gyeong-O)
        assert_eq!(jan.year_pillar.0, HeavenlyStem::Gi);
        assert_eq!(jan.year_pillar.1, EarthlyBranch::Sa);
        assert_eq!(
            jun.year_pillar.0,
            HeavenlyStem::Gyeong,
            "post-Ipchun: 1990 raw = (1990-4)%60 = 6 → Gyeong"
        );
        assert_eq!(
            jun.year_pillar.1,
            EarthlyBranch::O,
            "post-Ipchun: 1990 → O (Horse)"
        );
        // The two halves of 1990 sit in DIFFERENT solar years — the whole point.
        assert_ne!(jan.year_pillar, jun.year_pillar);
    }

    // FIXED — Daeun start age now uses the solar-term distance.
    //
    // The first-Daeun start age is the count of days from birth to the ADJACENT
    // Jie Qi (next term when the cycle runs forward, previous term when it runs
    // backward), divided by three (the classical "3 days = 1 year" rule). The
    // old `birth_year % 3 + 1` placeholder is gone.
    //
    // 1989 and 1992 share (year % 3 == 0), so the OLD code gave both the same
    // start age. They differ now because their year stems have opposite polarity
    // — 1989 = Gi (Yin) so a Male counts BACKWARD to the previous term (~9.7 d →
    // age 3); 1992 = Im (Yang) so a Male counts FORWARD to the next term
    // (~21.4 d → age 7). Direction is the same flag that drives the cycle.
    #[test]
    fn daeun_start_age_depends_on_solar_term_distance() {
        use xalen_world::saju::{Gender, daeun};
        let a = compute_saju(1989, 6, 15, 10);
        let b = compute_saju(1992, 6, 15, 10);
        let da = daeun(&a, Gender::Male, 1989);
        let db = daeun(&b, Gender::Male, 1992);
        // Oracle (Meeus solar-longitude root-find, mirrored in Python):
        //   1989 Male → backward 9.71 d → round(9.71/3) = 3
        //   1992 Male → forward 21.44 d → round(21.44/3) = 7
        assert_eq!(da[0].start_age, 3, "1989 Male: backward ~9.7 d → age 3");
        assert_eq!(db[0].start_age, 7, "1992 Male: forward ~21.4 d → age 7");
        assert_ne!(
            da[0].start_age, db[0].start_age,
            "a real Daeun start age differs between these births"
        );
        // Every start age is a real 1-10 year value, never the old %3 placeholder.
        for (chart, gender, by) in [(&a, Gender::Male, 1989), (&b, Gender::Male, 1992)] {
            let first = daeun(chart, gender, by)[0].start_age;
            assert!((1..=10).contains(&first), "start age {first} out of 1..=10");
        }
    }

    // Pin the current Daeun contract that IS sound: 8 consecutive 10-year
    // periods, and the forward/backward direction rule
    // (Male+Yang or Female+Yin → forward). This part is correct and stays green.
    #[test]
    fn daeun_structure_is_sound() {
        use xalen_world::saju::{Gender, daeun};
        let chart = compute_saju(2024, 6, 15, 10); // 2024 = Gap (Yang) year
        let periods = daeun(&chart, Gender::Male, 2024);
        assert_eq!(periods.len(), 8, "8 Daeun periods");
        for i in 1..periods.len() {
            assert_eq!(
                periods[i].start_age,
                periods[i - 1].end_age,
                "Daeun periods must be consecutive"
            );
        }
        // Male + Yang year → forward: first period stem is the next stem after
        // the month-pillar stem.
        let expected = (chart.month_pillar.0.index() + 1) % 10;
        assert_eq!(
            periods[0].stem.index(),
            expected,
            "forward Daeun first stem"
        );
    }
}

// ===========================================================================
// Mahabote (Burmese) — weekday day-sign, oracle = pyswisseph swe.day_of_week
// ===========================================================================
mod mahabote {
    use xalen_world::mahabote::{MahaboteDay, mahabote_from_jd};

    // pyswisseph weekday (Sun=0..Sat=6) for noon-midnight JDs:
    //   2024-01-01 JD 2460310.5 = Monday
    //   2012-12-21 JD 2456282.5 = Friday
    //   2000-01-01 JD 2451544.5 = Saturday
    //   1970-01-01 JD 2440587.5 = Thursday
    //   1990-01-15 JD 2447906.5 = Monday
    #[test]
    fn weekday_matches_pyswisseph() {
        let cases = [
            (2_460_310.5_f64, MahaboteDay::Monday),
            (2_456_282.5, MahaboteDay::Friday),
            (2_451_544.5, MahaboteDay::Saturday),
            (2_440_587.5, MahaboteDay::Thursday),
            (2_447_906.5, MahaboteDay::Monday),
        ];
        for (jd, expected) in cases {
            assert_eq!(
                mahabote_from_jd(jd).birth_day,
                expected,
                "JD {jd} weekday (pyswisseph oracle)"
            );
        }
    }

    // The ruling planet must track the weekday (Monday → Moon).
    #[test]
    fn ruling_planet_follows_weekday() {
        assert_eq!(mahabote_from_jd(2_460_310.5).ruling_planet, "Moon"); // Monday
        assert_eq!(mahabote_from_jd(2_451_544.5).ruling_planet, "Saturn"); // Saturday
    }
}

// ===========================================================================
// Mayan — Long Count / Tzolkin / Haab, anchored to GMT correlation
// ===========================================================================
mod mayan {
    use xalen_world::mayan::{TzolkinDayName, long_count_from_jd, tzolkin_from_jd};

    const JD_2012_DEC_21: f64 = 2_456_282.5; // pyswisseph
    const JD_CREATION: f64 = 584_283.0; // GMT correlation
    const JD_2024_JAN_01: f64 = 2_460_310.5; // pyswisseph

    // The single most-published Maya date: 21 Dec 2012 = 13.0.0.0.0.
    #[test]
    fn long_count_2012_end_of_baktun_13() {
        let lc = long_count_from_jd(JD_2012_DEC_21);
        assert_eq!(
            (lc.baktun, lc.katun, lc.tun, lc.uinal, lc.kin),
            (13, 0, 0, 0, 0)
        );
    }

    // Creation date Tzolkin = 4 Ahau (definitional anchor).
    #[test]
    fn creation_is_4_ahau() {
        let tz = tzolkin_from_jd(JD_CREATION);
        assert_eq!(tz.number, 4);
        assert_eq!(tz.day_name, TzolkinDayName::Ahau);
    }

    // Independently computed (GMT formula) reference for a neutral modern date.
    //   day_count(2460310.5) = floor(2460310.5+0.5) - 584283 = 1876028
    //   1876028 / 144000 = 13 r 4028 ; 4028/7200=0 ; 4028/360=11 r 68 ;
    //   68/20=3 r 8 → 13.0.11.3.8 ; Tzolkin: num=((1876028+3)%13)+1=2,
    //   name idx=(1876028+19)%20=7 → Lamat.
    #[test]
    fn modern_date_long_count_and_tzolkin() {
        let lc = long_count_from_jd(JD_2024_JAN_01);
        assert_eq!(
            (lc.baktun, lc.katun, lc.tun, lc.uinal, lc.kin),
            (13, 0, 11, 3, 8),
            "2024-01-01 Long Count"
        );
        let tz = tzolkin_from_jd(JD_2024_JAN_01);
        assert_eq!(tz.number, 2, "2024-01-01 Tzolkin number");
        assert_eq!(
            tz.day_name,
            TzolkinDayName::Lamat,
            "2024-01-01 Tzolkin day name"
        );
    }
}

// ===========================================================================
// Aztec — Tonalpohualli, anchored to the Caso (1521) correlation
// ===========================================================================
mod aztec {
    use xalen_world::aztec::tonalpohualli_from_jd;

    const ANCHOR_JD: f64 = 2_276_827.5; // 13 Aug 1521 Julian = 1 Coatl

    // Definitional anchor: fall of Tenochtitlan = 1 Coatl (Serpent), sign idx 4.
    #[test]
    fn anchor_is_1_coatl() {
        let fate = tonalpohualli_from_jd(ANCHOR_JD);
        assert_eq!(fate.day_sign, "Coatl");
        assert_eq!(fate.day_sign_english, "Serpent");
        assert_eq!(fate.day_number, 1);
        assert_eq!(fate.day_sign_index, 4);
    }

    // Independently computed reference for 2024-01-01 (JD 2460310.5):
    //   day_diff = floor(2460310.5) - floor(2276827.5) = 2460310 - 2276827 = 183483
    //   sign idx = (4 + 183483) % 20 = 7 → Tochtli (Rabbit)
    //   number   = ((1 - 1 + 183483) % 13) + 1 = (183483 % 13) + 1 = 1 + 1 = 2
    #[test]
    fn modern_date_2024_01_01_is_2_tochtli() {
        let fate = tonalpohualli_from_jd(2_460_310.5);
        assert_eq!(fate.day_sign, "Tochtli", "2024-01-01 day sign");
        assert_eq!(fate.day_sign_english, "Rabbit");
        assert_eq!(fate.day_number, 2, "2024-01-01 day number");
        assert_eq!(fate.day_sign_index, 7);
    }
}

// ===========================================================================
// Nine Star Ki (Japanese) — year/month star tables.
// Oracle: well-published year-star values (these are the same anchors the
// crate's own unit tests use; reproduced here as integration-level fixtures).
// No external library was needed — the year star is closed-form digit math.
// ===========================================================================
mod nine_star_ki {
    use xalen_world::nine_star_ki::{month_star, year_star};

    // Published reference year stars (widely tabulated):
    //   2020 = 7 (Metal), 2023 = 4, 2024 = 3, 2025 = 2.
    #[test]
    fn published_year_stars() {
        assert_eq!(year_star(2020), 7);
        assert_eq!(year_star(2023), 4);
        assert_eq!(year_star(2024), 3);
        assert_eq!(year_star(2025), 2);
    }

    // Group-A (year star 1/4/7) February month star = 8 (table head).
    // 2020 has year star 7 → group A → Feb (month 2) = 8.
    #[test]
    fn february_month_star_group_a() {
        assert_eq!(year_star(2020), 7);
        assert_eq!(month_star(2020, 2), 8);
    }
}
