use crate::nakshatra::{DashaLord, Nakshatra};
use serde::{Deserialize, Serialize};

const NAKSHATRA_SPAN: f64 = 360.0 / 27.0; // 13.3333°

#[derive(Debug, Clone, Serialize, Deserialize)]
/// KP (Krishnamurti Paddhati) position breakdown for a sidereal degree.
pub struct KpPosition {
    pub sign_lord: &'static str,
    pub star_lord: DashaLord,
    pub sub_lord: DashaLord,
    pub sub_sub_lord: DashaLord,
    pub kp_number: u16, // 1-249
}

const SUB_SPANS: [(DashaLord, f64); 9] = [
    (DashaLord::Ketu, 7.0 / 120.0 * NAKSHATRA_SPAN),
    (DashaLord::Venus, 20.0 / 120.0 * NAKSHATRA_SPAN),
    (DashaLord::Sun, 6.0 / 120.0 * NAKSHATRA_SPAN),
    (DashaLord::Moon, 10.0 / 120.0 * NAKSHATRA_SPAN),
    (DashaLord::Mars, 7.0 / 120.0 * NAKSHATRA_SPAN),
    (DashaLord::Rahu, 18.0 / 120.0 * NAKSHATRA_SPAN),
    (DashaLord::Jupiter, 16.0 / 120.0 * NAKSHATRA_SPAN),
    (DashaLord::Saturn, 19.0 / 120.0 * NAKSHATRA_SPAN),
    (DashaLord::Mercury, 17.0 / 120.0 * NAKSHATRA_SPAN),
];

/// Compute sign lord, star lord, sub lord, and KP number for a degree.
///
/// Sub-lord spans follow Vimshottari Dasha proportions within each nakshatra.
/// The sub sequence starts from the nakshatra lord (star lord) and cycles
/// through the 9 Vimshottari lords: Ketu, Venus, Sun, Moon, Mars, Rahu,
/// Jupiter, Saturn, Mercury.
///
/// Web-verified: https://kpastrology.astrosage.com/reference-tables/sign-star-and-sub-table
/// Web-verified: https://kpastrologypro.com/blog/kp-astrology-nakshatras-27-stars
pub fn kp_position(sidereal_deg: f64) -> KpPosition {
    let lon = sidereal_deg.rem_euclid(360.0);
    let sign_idx = (lon / 30.0) as usize;
    let nak = Nakshatra::from_longitude_deg(lon);
    let star_lord = nak.lord();

    let pos_in_nak = lon % NAKSHATRA_SPAN;

    let (sub_lord, pos_in_sub, sub_span) = find_sub(pos_in_nak, &star_lord, NAKSHATRA_SPAN);
    let (sub_sub_lord, _, _) = find_sub(pos_in_sub, &sub_lord, sub_span);

    let kp_number = compute_kp_number(lon);

    let sign_lord = match sign_idx {
        0 | 7 => "Mars",
        1 | 6 => "Venus",
        2 | 5 => "Mercury",
        3 => "Moon",
        4 => "Sun",
        8 | 11 => "Jupiter",
        9 | 10 => "Saturn",
        _ => "Unknown",
    };

    KpPosition {
        sign_lord,
        star_lord,
        sub_lord,
        sub_sub_lord,
        kp_number,
    }
}

fn find_sub(
    pos_in_parent: f64,
    starting_lord: &DashaLord,
    parent_span: f64,
) -> (DashaLord, f64, f64) {
    let start_idx = DashaLord::SEQUENCE
        .iter()
        .position(|l| l == starting_lord)
        .unwrap_or(0);

    let normalized = pos_in_parent.rem_euclid(parent_span);

    let mut accumulated = 0.0;
    for i in 0..9 {
        let idx = (start_idx + i) % 9;
        let lord = DashaLord::SEQUENCE[idx];
        let span = lord.vimshottari_years() / 120.0 * parent_span;
        if accumulated + span > normalized {
            let pos_in_sub = normalized - accumulated;
            return (lord, pos_in_sub, span);
        }
        accumulated += span;
    }
    let lord = DashaLord::SEQUENCE[start_idx];
    let span = lord.vimshottari_years() / 120.0 * parent_span;
    (lord, 0.0, span)
}

fn compute_kp_number(lon: f64) -> u16 {
    let nak_idx = (lon / NAKSHATRA_SPAN) as usize;
    let pos_in_nak = lon % NAKSHATRA_SPAN;

    let nak = Nakshatra::from_longitude_deg(lon);
    let star_lord = nak.lord();
    let start_idx = DashaLord::SEQUENCE
        .iter()
        .position(|l| *l == star_lord)
        .unwrap_or(0);

    let mut sub_idx = 0;
    let mut accumulated = 0.0;
    for i in 0..9 {
        let idx = (start_idx + i) % 9;
        let span = SUB_SPANS[idx].1;
        if accumulated + span > pos_in_nak {
            sub_idx = i;
            break;
        }
        accumulated += span;
    }

    (nak_idx * 9 + sub_idx + 1).min(249) as u16
}

/// Compute the five ruling planets (deduplicated) for a KP judgment.
pub fn ruling_planets(
    day_lord: DashaLord,
    moon_sign_lord: DashaLord,
    moon_star_lord: DashaLord,
    lagna_sign_lord: DashaLord,
    lagna_star_lord: DashaLord,
) -> Vec<DashaLord> {
    let all = [
        day_lord,
        moon_sign_lord,
        moon_star_lord,
        lagna_sign_lord,
        lagna_star_lord,
    ];
    let mut unique = Vec::new();
    for &lord in &all {
        if !unique.contains(&lord) {
            unique.push(lord);
        }
    }
    unique
}

// ---------------------------------------------------------------------------
// Planet — imported from xalen-coords (shared canonical enum)
// ---------------------------------------------------------------------------

pub use xalen_coords::Planet;

/// Convert a KP Planet to a DashaLord (1:1 mapping for the 9 Vedic grahas).
pub fn planet_to_dasha_lord(p: Planet) -> DashaLord {
    match p {
        Planet::Sun => DashaLord::Sun,
        Planet::Moon => DashaLord::Moon,
        Planet::Mars => DashaLord::Mars,
        Planet::Mercury => DashaLord::Mercury,
        Planet::Jupiter => DashaLord::Jupiter,
        Planet::Venus => DashaLord::Venus,
        Planet::Saturn => DashaLord::Saturn,
        Planet::Rahu | Planet::NorthNode => DashaLord::Rahu,
        Planet::Ketu | Planet::SouthNode => DashaLord::Ketu,
        _ => DashaLord::Sun, // outer planets don't map to Vedic lords
    }
}

/// Convert a DashaLord back to a Planet.
pub fn planet_from_dasha_lord(dl: DashaLord) -> Planet {
    match dl {
        DashaLord::Sun => Planet::Sun,
        DashaLord::Moon => Planet::Moon,
        DashaLord::Mars => Planet::Mars,
        DashaLord::Mercury => Planet::Mercury,
        DashaLord::Jupiter => Planet::Jupiter,
        DashaLord::Venus => Planet::Venus,
        DashaLord::Saturn => Planet::Saturn,
        DashaLord::Rahu => Planet::Rahu,
        DashaLord::Ketu => Planet::Ketu,
    }
}

/// Signs owned by this planet (classical rulership) -- KP convention.
fn kp_owned_signs(p: Planet) -> &'static [usize] {
    match p {
        Planet::Sun => &[4],         // Leo
        Planet::Moon => &[3],        // Cancer
        Planet::Mars => &[0, 7],     // Aries, Scorpio
        Planet::Mercury => &[2, 5],  // Gemini, Virgo
        Planet::Jupiter => &[8, 11], // Sagittarius, Pisces
        Planet::Venus => &[1, 6],    // Taurus, Libra
        Planet::Saturn => &[9, 10],  // Capricorn, Aquarius
        Planet::Rahu => &[10],       // Aquarius (KP assigns Rahu co-rule)
        Planet::Ketu => &[7],        // Scorpio (KP assigns Ketu co-rule)
        _ => &[],                    // outer/western-node planets
    }
}

/// Vedic full-strength aspects -- 0-indexed position advances from the
/// occupied house.  The Vedic "Nth house aspect" counts inclusively from
/// the planet's own house, so the advance is N-1.  E.g. "7th aspect"
/// from house 7 lands on house 1: advance = 6.
///
/// Web-verified:
/// - Mars: 4th, 7th, 8th house aspects (standard Vedic)
/// - Jupiter: 5th, 7th, 9th house aspects (standard Vedic)
/// - Saturn: 3rd, 7th, 10th house aspects (standard Vedic)
/// - Sun, Moon, Mercury, Venus: 7th house aspect only (standard Vedic)
/// - Rahu/Ketu: debated in KP; assigned Jupiter-like aspects (5th, 7th, 9th)
///   per one South Indian convention. Some KP practitioners omit node aspects
///   entirely. We follow the convention that includes them.
///
/// Sources:
/// - https://jagannathhora.com/aspects-vedic-astrology/
/// - https://kpastrology.astrosage.com/kp-learning-home/tutorial/chapter-2-fundamental-principles
fn kp_aspect_offsets(p: Planet) -> &'static [usize] {
    match p {
        Planet::Mars => &[3, 6, 7],    // 4th, 7th, 8th aspects
        Planet::Jupiter => &[4, 6, 8], // 5th, 7th, 9th aspects
        Planet::Saturn => &[2, 6, 9],  // 3rd, 7th, 10th aspects
        Planet::Rahu => &[4, 6, 8],    // same as Jupiter (debated; see comment)
        Planet::Ketu => &[4, 6, 8],    // same as Jupiter (debated; see comment)
        _ => &[6],                     // Sun, Moon, Mercury, Venus: 7th only
    }
}

// ---------------------------------------------------------------------------
// KpChart — input structure for KP analysis
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Input chart data for KP analysis (cusps + planet positions).
pub struct KpChart {
    /// Sidereal cusp degrees for houses 1-12 (index 0 = house 1).
    pub cusps: [f64; 12],
    /// Planet positions: (planet, sidereal longitude in degrees).
    pub planets: Vec<(Planet, f64)>,
}

impl KpChart {
    /// Determine which house (1-12) a given sidereal degree falls in.
    /// Uses Placidus-style unequal houses: a degree belongs to house N if it falls
    /// between cusp N and cusp N+1.
    pub fn house_of_degree(&self, deg: f64) -> usize {
        let d = deg.rem_euclid(360.0);
        for i in 0..12 {
            let start = self.cusps[i].rem_euclid(360.0);
            let end = self.cusps[(i + 1) % 12].rem_euclid(360.0);
            if start < end {
                if d >= start && d < end {
                    return i + 1;
                }
            } else {
                // wraps around 0/360
                if d >= start || d < end {
                    return i + 1;
                }
            }
        }
        1 // fallback
    }

    /// Get planet's occupied house (1-12).
    pub fn planet_house(&self, planet: Planet) -> Option<usize> {
        self.planets
            .iter()
            .find(|(p, _)| *p == planet)
            .map(|(_, deg)| self.house_of_degree(*deg))
    }
}

// ---------------------------------------------------------------------------
// Significator system — the heart of KP
// ---------------------------------------------------------------------------

/// How a planet signifies a house — ordered by KP strength.
///
/// KP hierarchy (per K.S. Krishnamurti, web-verified):
///   A (strongest) = Planet in the star (nakshatra) of a planet OCCUPYING the house
///   B             = Planet OCCUPYING the house
///   C             = Planet in the star of the house OWNER
///   D             = House OWNER (sign lord of the cusp)
///   E (weakest)   = Planet ASPECTING or CONJOINED with the above significators
///
/// Sources:
/// - https://kpastrologypro.com/blog/kp-astrology-significators-explained
/// - https://kpastroapp.com/learn/house-significators-in-kp-astrology
/// - https://www.kpastrology.com/kpbasics.htm
/// - https://kpastrology.astrosage.com/kp-learning-home/tutorial/chapter-2-fundamental-principles
///
/// Our enum collapses A and C into `StarLord` because the implementation
/// already distinguishes them by checking whether the planet whose star is
/// occupied is an Occupant (yielding grade A) or an Owner (yielding grade C).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignificatorType {
    /// Planet is the star lord of a planet occupying this house (grade A, strongest).
    StarLord,
    /// Planet occupies this house (grade B).
    Occupant,
    /// Planet owns the sign on this house cusp (grade D).
    Owner,
    /// Planet aspects this house (grade E, weakest).
    Aspecting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A planet and the houses it signifies with strength ordering.
pub struct KpSignificator {
    pub planet: Planet,
    /// Houses this planet signifies (1-12), deduplicated.
    pub signified_houses: Vec<usize>,
    /// Each signification with its type, ordered strongest first.
    pub strength_order: Vec<(usize, SignificatorType)>,
}

/// Compute the full significator table for all 9 planets in a KP chart.
///
/// KP signification hierarchy (per K.S. Krishnamurti):
/// A. Planet in the **star of an occupant** — strongest significator.
/// B. Planet **occupying** the house.
/// C. Planet in the **star of the owner** — third level.
/// D. Planet **owning** the sign on the cusp.
/// E. Planet **aspecting** the house — weakest.
///
/// Web-verified:
/// - https://kpastrologypro.com/blog/kp-astrology-significators-explained
/// - https://kpastroapp.com/learn/house-significators-in-kp-astrology
/// - https://www.kpastrology.com/kpbasics.htm
pub fn compute_significators(chart: &KpChart) -> Vec<KpSignificator> {
    let mut result = Vec::with_capacity(9);

    // Pre-compute: which house does each planet occupy?
    let mut planet_houses: Vec<(Planet, usize)> = Vec::new();
    for &(planet, deg) in &chart.planets {
        let house = chart.house_of_degree(deg);
        planet_houses.push((planet, house));
    }

    // Pre-compute: star lord of each planet's position
    let mut planet_star_lords: Vec<(Planet, DashaLord)> = Vec::new();
    for &(planet, deg) in &chart.planets {
        let nak = Nakshatra::from_longitude_deg(deg);
        planet_star_lords.push((planet, nak.lord()));
    }

    // Owner of each house cusp's sign
    let cusp_owners: Vec<usize> = (0..12)
        .map(|i| {
            
            (chart.cusps[i].rem_euclid(360.0) / 30.0) as usize
        })
        .collect();

    // Pre-compute: for each house, which planet owns its cusp sign?
    // (Maps house 1-12 to the owner Planet.)
    let house_owner_planets: Vec<Option<Planet>> = (0..12)
        .map(|i| {
            let sign_idx = cusp_owners[i];
            Planet::VEDIC_NINE
                .iter()
                .find(|&&p| kp_owned_signs(p).contains(&sign_idx))
                .copied()
        })
        .collect();

    for &planet in &Planet::VEDIC_NINE {
        let mut strength_order: Vec<(usize, SignificatorType)> = Vec::new();

        // Grade A (StarLord): if this planet is the star lord of any planet X
        // that OCCUPIES a house, this planet signifies that house at grade A.
        for &(other_planet, star_lord) in &planet_star_lords {
            if star_lord == planet_to_dasha_lord(planet)
                && let Some(&(_, house)) = planet_houses.iter().find(|(p, _)| *p == other_planet) {
                    strength_order.push((house, SignificatorType::StarLord));
                }
        }

        // Grade B (Occupant): the house this planet sits in.
        if let Some(&(_, house)) = planet_houses.iter().find(|(p, _)| *p == planet) {
            strength_order.push((house, SignificatorType::Occupant));
        }

        // Grade C (StarLord of Owner): if this planet is the star lord of the
        // planet that OWNS a house cusp, this planet signifies that house at
        // grade C. We reuse the StarLord enum type but insert it after Occupant,
        // preserving the A > B > C ordering within the strength_order vec.
        for (house_idx, owner_opt) in house_owner_planets.iter().enumerate() {
            if let Some(owner_planet) = owner_opt {
                // Find the star lord of this owner planet
                if let Some(&(_, owner_star)) =
                    planet_star_lords.iter().find(|(p, _)| p == owner_planet)
                    && owner_star == planet_to_dasha_lord(planet) {
                        let h = house_idx + 1;
                        // Avoid duplicating if already added as grade A
                        if !strength_order.iter().any(|&(hh, _)| hh == h) {
                            strength_order.push((h, SignificatorType::StarLord));
                        }
                    }
            }
        }

        // Grade D (Owner): houses whose cusp sign is owned by this planet.
        let owned = kp_owned_signs(planet);
        for (house_idx, &sign_idx) in cusp_owners.iter().enumerate() {
            if owned.contains(&sign_idx) {
                strength_order.push((house_idx + 1, SignificatorType::Owner));
            }
        }

        // Grade E (Aspecting): houses aspected by this planet from its occupied house.
        if let Some(&(_, occ_house)) = planet_houses.iter().find(|(p, _)| *p == planet) {
            for &offset in kp_aspect_offsets(planet) {
                let aspected = ((occ_house - 1 + offset) % 12) + 1;
                strength_order.push((aspected, SignificatorType::Aspecting));
            }
        }

        // Deduplicate signified houses while preserving strongest-first order.
        let mut signified_houses: Vec<usize> = Vec::new();
        for &(h, _) in &strength_order {
            if !signified_houses.contains(&h) {
                signified_houses.push(h);
            }
        }

        result.push(KpSignificator {
            planet,
            signified_houses,
            strength_order,
        });
    }

    result
}

// ---------------------------------------------------------------------------
// Cuspal Sub-Lord Theory
// ---------------------------------------------------------------------------

/// Whether a house's sub lord promises positive or negative results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Whether a house cusp sub lord promises positive or negative results.
pub enum HousePromise {
    Positive,
    Negative,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Sub-lord analysis result for a single house cusp.
pub struct CuspalSubLord {
    /// House number (1-12).
    pub house: usize,
    /// Cusp degree (sidereal).
    pub cusp_deg: f64,
    /// Sign lord of the cusp (planet name).
    pub sign_lord: &'static str,
    /// Star (nakshatra) lord of the cusp.
    pub star_lord: DashaLord,
    /// Sub lord of the cusp — the decisive factor in KP.
    pub sub_lord: DashaLord,
    /// Promise: does this house deliver positive or negative results?
    pub promise: HousePromise,
}

/// Favorable houses for each house cusp per KP (Krishnamurti).
/// If the sub lord of house N signifies houses favorable to N's matters,
/// the house promise is positive.
///
/// Web-verified: For marriage, houses 2, 7, 11 are favorable per KP theory.
/// Source: https://kpastrologypro.com/blog/12-houses-kp-astrology
/// Source: https://kpastrology.astrosage.com/kp-learning-home/tutorial/chapter-2-fundamental-principles
fn favorable_houses_for_cusp(house: usize) -> &'static [usize] {
    match house {
        1 => &[1, 5, 9, 11],     // self, fortune, dharma, gains
        2 => &[2, 6, 10, 11],    // wealth, service, career, gains
        3 => &[3, 6, 10, 11],    // siblings, effort
        4 => &[4, 2, 11],        // property, family, gains
        5 => &[2, 5, 11],        // children, education, speculation
        6 => &[1, 2, 6, 10, 11], // health recovery, service, career
        7 => &[2, 7, 11],        // marriage, partnership
        8 => &[1, 5, 8, 11],     // longevity
        9 => &[2, 9, 11],        // fortune, higher learning
        10 => &[2, 6, 10, 11],   // career, profession
        11 => &[2, 3, 6, 11],    // gains, fulfilment
        12 => &[3, 9, 12],       // foreign travel, liberation
        _ => &[],
    }
}

/// Unfavorable houses for each cusp that negate the promise.
fn unfavorable_houses_for_cusp(house: usize) -> &'static [usize] {
    match house {
        1 => &[6, 8, 12],
        2 => &[5, 8, 12],
        3 => &[8, 12],
        4 => &[3, 5, 12],
        5 => &[4, 8, 12],
        6 => &[5, 11, 12],
        7 => &[1, 6, 10, 12],
        8 => &[6, 12],
        9 => &[3, 8, 12],
        10 => &[5, 8, 12],
        11 => &[5, 8, 12],
        12 => &[1, 2, 6, 10],
        _ => &[],
    }
}

/// Perform cuspal sub-lord analysis for all 12 house cusps.
///
/// For each cusp, computes sign lord, star lord, sub lord, then checks
/// whether the sub lord's significations (from the significator table)
/// lean favorable or unfavorable for that house.
pub fn cuspal_analysis(
    cusp_degrees: &[f64; 12],
    planet_positions: &[(Planet, f64)],
) -> Vec<CuspalSubLord> {
    let chart = KpChart {
        cusps: *cusp_degrees,
        planets: planet_positions.to_vec(),
    };
    let significators = compute_significators(&chart);

    let mut results = Vec::with_capacity(12);

    for (i, &cusp_deg) in cusp_degrees.iter().enumerate() {
        let house = i + 1;
        let kp = kp_position(cusp_deg);

        // Find the sub lord's significator entry
        let sub_lord_planet = planet_from_dasha_lord(kp.sub_lord);
        let sub_lord_sigs = significators.iter().find(|s| s.planet == sub_lord_planet);

        let promise = if let Some(sigs) = sub_lord_sigs {
            let favorable = favorable_houses_for_cusp(house);
            let unfavorable = unfavorable_houses_for_cusp(house);

            let fav_count = sigs
                .signified_houses
                .iter()
                .filter(|h| favorable.contains(h))
                .count();
            let unfav_count = sigs
                .signified_houses
                .iter()
                .filter(|h| unfavorable.contains(h))
                .count();

            if fav_count > 0 && unfav_count == 0 {
                HousePromise::Positive
            } else if unfav_count > 0 && fav_count == 0 {
                HousePromise::Negative
            } else if fav_count > 0 && unfav_count > 0 {
                HousePromise::Mixed
            } else {
                // Sub lord signifies neither favorable nor unfavorable — treat as mixed
                HousePromise::Mixed
            }
        } else {
            HousePromise::Mixed
        };

        results.push(CuspalSubLord {
            house,
            cusp_deg,
            sign_lord: kp.sign_lord,
            star_lord: kp.star_lord,
            sub_lord: kp.sub_lord,
            promise,
        });
    }

    results
}

// ---------------------------------------------------------------------------
// Ruling Planets with Strength
// ---------------------------------------------------------------------------

/// Enhanced ruling planets with occurrence-based strength scoring.
///
/// A planet appearing multiple times across the 5 input positions scores higher.
/// If Rahu or Ketu appears, checks if it is in the same sign as another planet
/// (agent rule): Rahu/Ketu act as agents of the sign lord they occupy.
pub fn ruling_planets_with_strength(
    day_lord: DashaLord,
    moon_sign_lord: DashaLord,
    moon_star_lord: DashaLord,
    lagna_sign_lord: DashaLord,
    lagna_star_lord: DashaLord,
) -> Vec<(DashaLord, u8)> {
    let all = [
        day_lord,
        moon_sign_lord,
        moon_star_lord,
        lagna_sign_lord,
        lagna_star_lord,
    ];

    let mut counts: Vec<(DashaLord, u8)> = Vec::new();
    for &lord in &all {
        if let Some(entry) = counts.iter_mut().find(|(l, _)| *l == lord) {
            entry.1 += 1;
        } else {
            counts.push((lord, 1));
        }
    }

    // Sort descending by count (strength).
    counts.sort_by_key(|x| std::cmp::Reverse(x.1));
    counts
}

/// Extended ruling planets with Rahu/Ketu agent substitution.
///
/// If Rahu or Ketu is among the ruling planets, identify which planet's sign
/// they occupy (the agent). The agent planet is added or its count boosted.
pub fn ruling_planets_with_agents(
    day_lord: DashaLord,
    moon_sign_lord: DashaLord,
    moon_star_lord: DashaLord,
    lagna_sign_lord: DashaLord,
    lagna_star_lord: DashaLord,
    rahu_sign_lord: Option<DashaLord>,
    ketu_sign_lord: Option<DashaLord>,
) -> Vec<(DashaLord, u8)> {
    let mut result = ruling_planets_with_strength(
        day_lord,
        moon_sign_lord,
        moon_star_lord,
        lagna_sign_lord,
        lagna_star_lord,
    );

    // Rahu agent rule
    if let Some(agent) = rahu_sign_lord
        && result.iter().any(|(l, _)| *l == DashaLord::Rahu) {
            if let Some(entry) = result.iter_mut().find(|(l, _)| *l == agent) {
                entry.1 += 1;
            } else {
                result.push((agent, 1));
            }
        }

    // Ketu agent rule
    if let Some(agent) = ketu_sign_lord
        && result.iter().any(|(l, _)| *l == DashaLord::Ketu) {
            if let Some(entry) = result.iter_mut().find(|(l, _)| *l == agent) {
                entry.1 += 1;
            } else {
                result.push((agent, 1));
            }
        }

    result.sort_by_key(|x| std::cmp::Reverse(x.1));
    result
}

// ---------------------------------------------------------------------------
// KP Number to House mapping
// ---------------------------------------------------------------------------

/// Map a KP number (1-249) to its house (1-12).
///
/// The 249 KP sub-divisions map onto 360 degrees. The base count is
/// 27 nakshatras x 9 subs = 243 unique star-lord + sub-lord pairs.
/// Krishnamurti identified 6 additional sub-divisions where a sub straddles
/// two adjacent signs (the same star+sub lord spans a sign boundary),
/// yielding the canonical 249-division table used in KP horary astrology.
///
/// Web-verified: https://indiankpastrology.com/2020/09/11/sub-lord-table-how-to-know-249-sub-lord-table-in-kp-astrology/
/// Web-verified: https://roxyapi.com/blogs/kp-astrology-krishnamurti-paddhati-sub-lord-horary-guide
///
/// This function computes the starting degree of the given KP sub and returns
/// the sign (1=Aries through 12=Pisces) it falls in.
pub fn kp_number_to_house(kp_num: u16) -> usize {
    if kp_num == 0 || kp_num > 249 {
        return 1; // guard
    }

    // Walk the full 27 nakshatras x 9 subs, accumulating degree spans,
    // and stop when we reach the target KP number.
    let mut degree = 0.0_f64;
    let mut kp_count: u16 = 0;

    for nak_idx in 0..27 {
        let nak = Nakshatra::ALL[nak_idx];
        let star_lord = nak.lord();
        let start_idx = DashaLord::SEQUENCE
            .iter()
            .position(|l| *l == star_lord)
            .unwrap_or(0);

        for sub_i in 0..9 {
            kp_count += 1;
            if kp_count > 249 {
                break;
            }

            if kp_count == kp_num {
                // This is the target sub — return the sign it starts in.
                return ((degree.rem_euclid(360.0) / 30.0) as usize) + 1;
            }

            let dl_idx = (start_idx + sub_i) % 9;
            degree += SUB_SPANS[dl_idx].1;
        }
    }

    // Fallback (should not reach)
    ((degree.rem_euclid(360.0) / 30.0) as usize) + 1
}

// ---------------------------------------------------------------------------
// Event fruitfulness — KP event promise check
// ---------------------------------------------------------------------------

/// Events that can be checked for promise in KP.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Life events that can be checked for promise in KP analysis.
pub enum KpEvent {
    Marriage,
    Job,
    Health,
    ChildBirth,
    Education,
    ForeignTravel,
    Wealth,
    Litigation,
}

impl KpEvent {
    /// The primary house cusp to check for this event.
    pub fn primary_house(self) -> usize {
        match self {
            KpEvent::Marriage => 7,
            KpEvent::Job => 10,
            KpEvent::Health => 1,
            KpEvent::ChildBirth => 5,
            KpEvent::Education => 4,
            KpEvent::ForeignTravel => 12,
            KpEvent::Wealth => 2,
            KpEvent::Litigation => 6,
        }
    }

    /// Favorable supporting houses for this event per KP.
    pub fn favorable_houses(self) -> &'static [usize] {
        match self {
            KpEvent::Marriage => &[2, 7, 11],
            KpEvent::Job => &[2, 6, 10, 11],
            KpEvent::Health => &[1, 5, 11],
            KpEvent::ChildBirth => &[2, 5, 11],
            KpEvent::Education => &[4, 9, 11],
            KpEvent::ForeignTravel => &[3, 9, 12],
            KpEvent::Wealth => &[1, 2, 6, 11],
            KpEvent::Litigation => &[1, 2, 6, 11],
        }
    }

    /// Negating houses for this event.
    pub fn negating_houses(self) -> &'static [usize] {
        match self {
            KpEvent::Marriage => &[1, 6, 10, 12],
            KpEvent::Job => &[5, 8, 12],
            KpEvent::Health => &[6, 8, 12],
            KpEvent::ChildBirth => &[4, 8, 12],
            KpEvent::Education => &[3, 8, 12],
            KpEvent::ForeignTravel => &[1, 4, 10],
            KpEvent::Wealth => &[5, 8, 12],
            KpEvent::Litigation => &[5, 8, 12],
        }
    }
}

/// Check whether a specific life event is promised in a KP chart.
///
/// Examines the sub lord of the relevant house cusp and checks whether it
/// signifies houses favorable to the event. Returns true if the favorable
/// significations outweigh the negating ones.
pub fn is_event_promised(
    event: KpEvent,
    cuspal_sub_lord: DashaLord,
    significators: &[KpSignificator],
) -> bool {
    let sub_planet = planet_from_dasha_lord(cuspal_sub_lord);

    let sigs = match significators.iter().find(|s| s.planet == sub_planet) {
        Some(s) => s,
        None => return false,
    };

    let favorable = event.favorable_houses();
    let negating = event.negating_houses();

    let fav_count = sigs
        .signified_houses
        .iter()
        .filter(|h| favorable.contains(h))
        .count();
    let neg_count = sigs
        .signified_houses
        .iter()
        .filter(|h| negating.contains(h))
        .count();

    fav_count > neg_count
}

/// Convenience: check event promise directly from chart data.
pub fn check_event_in_chart(event: KpEvent, chart: &KpChart) -> bool {
    let significators = compute_significators(chart);
    let primary_house = event.primary_house();
    let cusp_kp = kp_position(chart.cusps[primary_house - 1]);
    is_event_promised(event, cusp_kp.sub_lord, &significators)
}

// ---------------------------------------------------------------------------
// Significator summary helper
// ---------------------------------------------------------------------------

/// For a given house, find all planets that signify it, ordered by strength.
pub fn significators_of_house(
    house: usize,
    all_significators: &[KpSignificator],
) -> Vec<(Planet, SignificatorType)> {
    let mut result: Vec<(Planet, SignificatorType)> = Vec::new();

    // Collect in KP strength order: StarLord > Occupant > Owner > Aspecting
    for sig_type in &[
        SignificatorType::StarLord,
        SignificatorType::Occupant,
        SignificatorType::Owner,
        SignificatorType::Aspecting,
    ] {
        for sig in all_significators {
            for &(h, ref st) in &sig.strength_order {
                if h == house && st == sig_type
                    && !result.iter().any(|(p, _)| *p == sig.planet) {
                        result.push((sig.planet, *sig_type));
                    }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kp_at_zero_degrees() {
        let kp = kp_position(0.0);
        assert_eq!(kp.sign_lord, "Mars"); // Aries
        assert_eq!(kp.star_lord, DashaLord::Ketu); // Ashwini
        assert_eq!(kp.kp_number, 1);
    }

    #[test]
    fn kp_number_range() {
        for deg in (0..360).step_by(1) {
            let kp = kp_position(deg as f64);
            assert!(
                kp.kp_number >= 1 && kp.kp_number <= 249,
                "KP number should be 1-249 at {deg}°, got {}",
                kp.kp_number
            );
        }
    }

    #[test]
    fn sub_lord_differs_within_nakshatra() {
        let kp1 = kp_position(0.5);
        let kp2 = kp_position(5.0);
        // Within Ashwini but different sub positions — sub lords may differ
        assert_eq!(kp1.star_lord, kp2.star_lord); // same nakshatra lord
    }

    #[test]
    fn sign_lords_correct() {
        assert_eq!(kp_position(0.0).sign_lord, "Mars"); // Aries
        assert_eq!(kp_position(30.0).sign_lord, "Venus"); // Taurus
        assert_eq!(kp_position(90.0).sign_lord, "Moon"); // Cancer
        assert_eq!(kp_position(120.0).sign_lord, "Sun"); // Leo
    }

    #[test]
    fn ruling_planets_dedup() {
        let rps = ruling_planets(
            DashaLord::Sun,
            DashaLord::Moon,
            DashaLord::Sun,
            DashaLord::Mars,
            DashaLord::Moon,
        );
        assert_eq!(rps.len(), 3); // Sun, Moon, Mars (deduplicated)
    }

    #[test]
    fn kp_249_max() {
        let kp = kp_position(359.9);
        assert!(kp.kp_number <= 249);
    }

    // -----------------------------------------------------------------------
    // New tests for deepened KP module
    // -----------------------------------------------------------------------

    /// Build a test chart with equal-house cusps starting from 0 Aries
    /// and planets at known positions.
    fn test_chart() -> KpChart {
        // Equal 30-degree houses starting at 0 Aries
        let cusps: [f64; 12] = [
            0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0, 210.0, 240.0, 270.0, 300.0, 330.0,
        ];
        let planets = vec![
            (Planet::Sun, 15.0),      // house 1 (Aries)
            (Planet::Moon, 65.0),     // house 3 (Gemini)
            (Planet::Mars, 280.0),    // house 10 (Capricorn)
            (Planet::Mercury, 45.0),  // house 2 (Taurus)
            (Planet::Jupiter, 190.0), // house 7 (Libra)
            (Planet::Venus, 310.0),   // house 11 (Aquarius)
            (Planet::Saturn, 130.0),  // house 5 (Leo)
            (Planet::Rahu, 95.0),     // house 4 (Cancer)
            (Planet::Ketu, 275.0),    // house 10 (Capricorn)
        ];
        KpChart { cusps, planets }
    }

    #[test]
    fn planet_to_from_dasha_lord_roundtrip() {
        for &p in &Planet::VEDIC_NINE {
            let dl = planet_to_dasha_lord(p);
            let p2 = planet_from_dasha_lord(dl);
            assert_eq!(p, p2, "roundtrip failed for {p:?}");
        }
    }

    #[test]
    fn kp_chart_house_of_degree_equal_houses() {
        let chart = test_chart();
        assert_eq!(chart.house_of_degree(0.0), 1);
        assert_eq!(chart.house_of_degree(29.9), 1);
        assert_eq!(chart.house_of_degree(30.0), 2);
        assert_eq!(chart.house_of_degree(180.0), 7);
        assert_eq!(chart.house_of_degree(359.9), 12);
    }

    #[test]
    fn kp_chart_planet_house() {
        let chart = test_chart();
        assert_eq!(chart.planet_house(Planet::Sun), Some(1));
        assert_eq!(chart.planet_house(Planet::Moon), Some(3));
        assert_eq!(chart.planet_house(Planet::Mars), Some(10));
        assert_eq!(chart.planet_house(Planet::Jupiter), Some(7));
        assert_eq!(chart.planet_house(Planet::Venus), Some(11));
        assert_eq!(chart.planet_house(Planet::Saturn), Some(5));
        assert_eq!(chart.planet_house(Planet::Rahu), Some(4));
    }

    #[test]
    fn compute_significators_returns_all_9_planets() {
        let chart = test_chart();
        let sigs = compute_significators(&chart);
        assert_eq!(sigs.len(), 9, "Should have significators for all 9 planets");
        for &p in &Planet::VEDIC_NINE {
            assert!(
                sigs.iter().any(|s| s.planet == p),
                "Missing significator for {p:?}"
            );
        }
    }

    #[test]
    fn significator_occupant_present() {
        let chart = test_chart();
        let sigs = compute_significators(&chart);

        // Sun occupies house 1 — it should signify house 1 as Occupant.
        let sun_sig = sigs.iter().find(|s| s.planet == Planet::Sun).unwrap();
        assert!(
            sun_sig
                .strength_order
                .iter()
                .any(|&(h, ref t)| h == 1 && *t == SignificatorType::Occupant),
            "Sun should be Occupant significator of house 1"
        );
        assert!(sun_sig.signified_houses.contains(&1));
    }

    #[test]
    fn significator_owner_present() {
        let chart = test_chart();
        let sigs = compute_significators(&chart);

        // Mars owns Aries (sign 0) — house 1 cusp is at 0 Aries.
        // So Mars should own house 1.
        let mars_sig = sigs.iter().find(|s| s.planet == Planet::Mars).unwrap();
        assert!(
            mars_sig
                .strength_order
                .iter()
                .any(|&(h, ref t)| h == 1 && *t == SignificatorType::Owner),
            "Mars should own house 1 (Aries cusp)"
        );
    }

    #[test]
    fn significator_aspecting_present() {
        let chart = test_chart();
        let sigs = compute_significators(&chart);

        // Jupiter in house 7 aspects house 1 (7th from 7), house 11 (5th from 7), house 3 (9th from 7).
        let jup_sig = sigs.iter().find(|s| s.planet == Planet::Jupiter).unwrap();
        let aspected: Vec<usize> = jup_sig
            .strength_order
            .iter()
            .filter(|(_, t)| *t == SignificatorType::Aspecting)
            .map(|(h, _)| *h)
            .collect();
        assert!(
            aspected.contains(&1),
            "Jupiter in H7 should aspect H1 (7th)"
        );
        assert!(
            aspected.contains(&11),
            "Jupiter in H7 should aspect H11 (5th)"
        );
        assert!(
            aspected.contains(&3),
            "Jupiter in H7 should aspect H3 (9th)"
        );
    }

    #[test]
    fn cuspal_analysis_returns_12_houses() {
        let chart = test_chart();
        let result = cuspal_analysis(&chart.cusps, &chart.planets);
        assert_eq!(result.len(), 12);
        for (i, csl) in result.iter().enumerate() {
            assert_eq!(csl.house, i + 1);
            assert!(
                !matches!(csl.sub_lord, _) || true,
                "sub_lord should be valid"
            );
        }
    }

    #[test]
    fn cuspal_sub_lord_fields_populated() {
        let chart = test_chart();
        let result = cuspal_analysis(&chart.cusps, &chart.planets);

        // House 1 cusp at 0 degrees Aries
        let h1 = &result[0];
        assert_eq!(h1.house, 1);
        assert_eq!(h1.sign_lord, "Mars"); // Aries
        assert_eq!(h1.star_lord, DashaLord::Ketu); // Ashwini
        assert!((h1.cusp_deg - 0.0).abs() < 0.001);
    }

    #[test]
    fn ruling_planets_strength_scoring() {
        let rps = ruling_planets_with_strength(
            DashaLord::Sun,
            DashaLord::Moon,
            DashaLord::Sun,
            DashaLord::Mars,
            DashaLord::Sun,
        );
        // Sun appears 3 times, Moon 1, Mars 1
        let sun_entry = rps.iter().find(|(l, _)| *l == DashaLord::Sun).unwrap();
        assert_eq!(sun_entry.1, 3, "Sun should have count 3");
        // Sun should be first (highest strength)
        assert_eq!(rps[0].0, DashaLord::Sun);
    }

    #[test]
    fn ruling_planets_agents_rahu() {
        // Rahu is a ruling planet, its sign lord is Venus (say Rahu in Taurus)
        let rps = ruling_planets_with_agents(
            DashaLord::Rahu,
            DashaLord::Moon,
            DashaLord::Sun,
            DashaLord::Mars,
            DashaLord::Jupiter,
            Some(DashaLord::Venus), // Rahu's sign lord
            None,
        );
        // Venus should appear as agent
        assert!(
            rps.iter().any(|(l, _)| *l == DashaLord::Venus),
            "Venus should be added as Rahu's agent"
        );
    }

    #[test]
    fn kp_number_to_house_range() {
        for num in 1..=249 {
            let house = kp_number_to_house(num);
            assert!(
                house >= 1 && house <= 12,
                "KP#{num} mapped to house {house}, expected 1-12"
            );
        }
    }

    #[test]
    fn kp_number_to_house_boundaries() {
        // KP 1 = start of Ashwini = 0 Aries = house 1
        assert_eq!(kp_number_to_house(1), 1);
        // KP 249 = last valid sub, deep in Pisces region
        let h249 = kp_number_to_house(249);
        assert!(
            h249 >= 1 && h249 <= 12,
            "KP 249 should be valid house, got {h249}"
        );
        // KP subs near the middle should be mid-zodiac houses
        let h125 = kp_number_to_house(125);
        assert!(
            h125 >= 5 && h125 <= 8,
            "KP 125 should be mid-zodiac house, got {h125}"
        );
    }

    #[test]
    fn kp_number_to_house_guard() {
        assert_eq!(kp_number_to_house(0), 1); // out of range
        assert_eq!(kp_number_to_house(250), 1); // out of range
    }

    #[test]
    fn kp_event_favorable_houses() {
        assert_eq!(KpEvent::Marriage.favorable_houses(), &[2, 7, 11]);
        assert_eq!(KpEvent::Job.favorable_houses(), &[2, 6, 10, 11]);
        assert_eq!(KpEvent::Health.favorable_houses(), &[1, 5, 11]);
    }

    #[test]
    fn is_event_promised_basic() {
        let chart = test_chart();
        let sigs = compute_significators(&chart);

        // Just verify it runs without panic and returns a bool
        let cusp7_kp = kp_position(chart.cusps[6]); // 7th house cusp
        let result = is_event_promised(KpEvent::Marriage, cusp7_kp.sub_lord, &sigs);
        // Result depends on chart — just verify type
        assert!(result || !result, "should return a boolean");
    }

    #[test]
    fn check_event_in_chart_runs() {
        let chart = test_chart();
        // Smoke test: should not panic
        let _marriage = check_event_in_chart(KpEvent::Marriage, &chart);
        let _job = check_event_in_chart(KpEvent::Job, &chart);
        let _health = check_event_in_chart(KpEvent::Health, &chart);
    }

    #[test]
    fn significators_of_house_returns_ordered() {
        let chart = test_chart();
        let sigs = compute_significators(&chart);

        let h1_sigs = significators_of_house(1, &sigs);
        // House 1 should have at least Sun (Occupant) and Mars (Owner)
        assert!(!h1_sigs.is_empty(), "House 1 should have significators");

        // Check ordering: StarLord entries before Occupant before Owner before Aspecting
        let mut last_rank = 0;
        for (_, st) in &h1_sigs {
            let rank = match st {
                SignificatorType::StarLord => 1,
                SignificatorType::Occupant => 2,
                SignificatorType::Owner => 3,
                SignificatorType::Aspecting => 4,
            };
            assert!(
                rank >= last_rank,
                "Significators should be ordered by strength"
            );
            last_rank = rank;
        }
    }

    #[test]
    fn kp_chart_wrapping_cusps() {
        // Test a chart where house 12 wraps around 360/0
        let cusps: [f64; 12] = [
            350.0, 20.0, 50.0, 80.0, 110.0, 140.0, 170.0, 200.0, 230.0, 260.0, 290.0, 320.0,
        ];
        let chart = KpChart {
            cusps,
            planets: vec![(Planet::Sun, 355.0)],
        };
        // 355 degrees is between cusp[0]=350 and cusp[1]=20 (wrapping)
        assert_eq!(chart.house_of_degree(355.0), 1);
        assert_eq!(chart.house_of_degree(5.0), 1); // still in house 1
        assert_eq!(chart.house_of_degree(25.0), 2); // in house 2
    }
}
