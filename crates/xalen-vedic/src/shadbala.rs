use crate::dignity::{
    exaltation_sign, moolatrikona_range, natural_enemies, natural_friends, own_signs,
};
use crate::divisional::{VargaChart, compute_varga_sign};
use crate::panchang::Vara;
use crate::rashi::Rashi;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// Six-fold planetary strength (Shadbala) in virupas per BPHS.
pub struct Shadbala {
    pub sthana_bala: SthanaBala,
    pub dig_bala: f64,
    pub kala_bala: KalaBala,
    pub cheshta_bala: f64,
    pub naisargika_bala: f64,
    pub drik_bala: f64,
    pub total: f64,
    pub required_minimum: f64,
    pub ratio: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// Positional strength sub-components of Shadbala.
pub struct SthanaBala {
    pub uchcha_bala: f64,
    pub saptavargaja_bala: f64,
    pub ojhayugma_bala: f64,
    pub kendra_bala: f64,
    pub drekkana_bala: f64,
    pub total: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// Temporal strength sub-components per BPHS Ch.27, all nine parts.
pub struct KalaBala {
    pub nathonnatha_bala: f64,
    pub paksha_bala: f64,
    pub tribhaga_bala: f64,
    pub abda_bala: f64,
    pub masa_bala: f64,
    pub vara_bala: f64,
    pub hora_bala: f64,
    /// Ayana Bala (declination strength), 0 to 60 virupas, 0 to 120 for the
    /// Sun. See [`Shadbala::compute_full`].
    pub ayana_bala: f64,
    /// Yuddha Bala (planetary war). Positive for the winner, negative for the
    /// loser, 0 for a planet not in a war. See [`Shadbala::compute_full`].
    pub yuddha_bala: f64,
    pub total: f64,
}

/// Position and metadata for a single planet, used as input to Drik Bala,
/// Ayana Bala and Yuddha Bala.
#[derive(Debug, Clone)]
pub struct PlanetPosition {
    pub name: &'static str,
    /// Sidereal longitude in degrees.
    pub longitude: f64,
    /// Daily motion in degrees per day.
    pub speed: f64,
    /// Declination in degrees, positive north of the celestial equator.
    /// Geocentric apparent declination is best. When no latitude is at hand,
    /// [`kranti_deg`] gives the classical kranti from the tropical longitude.
    pub declination: f64,
    /// House occupied, 1 to 12, with the same meaning as the `house` argument
    /// of [`Shadbala::compute_full`]. Needed for the Sthana and Dig Bala that
    /// enter Yuddha Bala.
    pub house: usize,
}

/// Classical kranti (declination) of a point on the ecliptic with zero
/// latitude: `sin(declination) = sin(obliquity) * sin(longitude)`.
///
/// `tropical_lon_deg` is the tropical (sayana) longitude, not the sidereal
/// one. `obliquity_deg` is the obliquity of the ecliptic in degrees, for
/// example `xalen_coords::mean_obliquity(t).to_degrees()`. The result is in
/// degrees, positive north. This ignores the planet's own ecliptic latitude,
/// which can change the declination of the Moon by up to about five degrees
/// and of the other planets by a few degrees; pass the true declination in
/// [`PlanetPosition::declination`] when it is known.
pub fn kranti_deg(tropical_lon_deg: f64, obliquity_deg: f64) -> f64 {
    (obliquity_deg.to_radians().sin() * tropical_lon_deg.to_radians().sin())
        .asin()
        .to_degrees()
}

/// Extended input for full Shadbala computation.
#[derive(Debug, Clone)]
pub struct ShadBalaInput {
    /// Julian date (UT1) of the moment.
    pub jd: f64,
    /// Sun sidereal longitude in degrees (needed for Paksha Bala).
    pub sun_lon: f64,
    /// Moon sidereal longitude in degrees (needed for Paksha Bala).
    pub moon_lon: f64,
    /// Sidereal time fraction of day at birth location (0.0 = midnight,
    /// 0.25 = 6am, 0.5 = noon, 0.75 = 6pm). Used for Nathonnatha and
    /// Tribhaga Bala.
    pub day_fraction: f64,
    /// Positions of all 7 planets (Sun through Saturn) for Drik Bala, Ayana
    /// Bala and Yuddha Bala.
    pub all_planets: Vec<PlanetPosition>,
}

impl Shadbala {
    /// Simplified Shadbala: Kala=30, Drik=0, Saptavargaja=0.
    /// **Warning**: produces non-BPHS totals. Use `compute_full()` for the
    /// complete six-fold computation.
    #[deprecated(note = "Use compute_full() for the complete six-fold Shadbala")]
    pub fn compute(planet: &str, lon_deg: f64, house: usize, speed: f64) -> Self {
        let sthana = compute_sthana_bala(planet, lon_deg, house);
        let dig = compute_dig_bala(planet, house);
        let naisargika = naisargika_bala(planet);
        let cheshta = compute_cheshta_bala(planet, speed);
        let kala = KalaBala {
            total: 30.0,
            ..Default::default()
        }; // simplified placeholder
        let drik = 0.0;

        let total = sthana.total + dig + kala.total + cheshta + naisargika + drik;
        let required = required_minimum(planet);

        Shadbala {
            sthana_bala: sthana,
            dig_bala: dig,
            kala_bala: kala,
            cheshta_bala: cheshta,
            naisargika_bala: naisargika,
            drik_bala: drik,
            total,
            required_minimum: required,
            ratio: total / required,
        }
    }

    /// Compute all six Shadbala strengths per Brihat Parashara Hora Shastra
    /// Ch. 27 — Sthana, Dig, Kala (all nine parts), Cheshta, Naisargika and
    /// Drik Bala — with Saptavargaja Bala included in Sthana.
    ///
    /// # Ayana Bala
    ///
    /// Ayana Bala comes from the declination `d` of the planet (BPHS Ch. 27,
    /// Santhanam's translation vv. 23-24; B.V. Raman, *Graha and Bhava Balas*).
    /// With the signed declination `k`, the strength is
    /// `(24 + k) * 60 / 48` virupas. For the Sun, Mars, Jupiter and Venus `k`
    /// is `d` (north is strong). For the Moon and Saturn `k` is `-d` (south is
    /// strong). For Mercury `k` is `|d|` (either side is strong). The Sun's
    /// result is doubled. `k` is clamped to -24..24 degrees, because the
    /// Moon and the planets with latitude can pass 24 degrees of declination,
    /// so the result is 0 to 60 virupas, and 0 to 120 for the Sun.
    ///
    /// # Yuddha Bala
    ///
    /// Only Mars, Mercury, Jupiter, Venus and Saturn fight; the Sun, the
    /// Moon and the nodes never do. Two of them are at war when their
    /// longitudes differ by less than one degree (Surya Siddhanta Ch. 7).
    /// The winner is the planet with the greater declination, the northern
    /// one (Surya Siddhanta Ch. 7, the rule that the northern planet wins).
    /// Surya Siddhanta judges the north by ecliptic latitude; this crate uses
    /// declination because that is what [`PlanetPosition`] carries. If the
    /// declinations are equal, the planet with the larger partial sum wins.
    ///
    /// The magnitude follows Sripati: the difference of the two planets'
    /// partial sums (Sthana, Dig, Nathonnatha, Paksha, Tribhaga and Hora
    /// Bala), divided by the difference of their disc diameters in seconds of
    /// arc (Mars 9.4, Mercury 6.6, Jupiter 190.4, Venus 16.6, Saturn 158.0:
    /// the table of the modern Shadbala literature, B.V. Raman, *Graha and
    /// Bhava Balas*, which PyJHora also uses; the Surya Siddhanta's own Ch. 7
    /// figures are in yojanas on the Moon's orbit and are not used here).
    /// The result is added to the winner and subtracted from the loser, so
    /// the Yuddha Bala of all planets sums to zero. A planet within one
    /// degree of two others is treated pair by pair. The Sun and Moon never
    /// form a pair, so they do not cancel a war between other planets.
    ///
    /// The alternative literal reading of BPHS uses the difference of the
    /// full Shadbala totals instead of the six partial sums. It is not
    /// implemented: it is circular, because Yuddha Bala is itself part of
    /// Kala Bala, and it is larger in order of magnitude, because full
    /// totals run to several hundred virupas while the six partial sums
    /// are roughly half of that.
    ///
    /// # Inputs
    ///
    /// `input.all_planets` must contain the planet being computed, found by
    /// name. Its `declination`, `house` and `longitude` are used for Ayana
    /// and Yuddha Bala. If the planet is absent, both are 0 and nothing
    /// panics.
    ///
    /// NOTE: the Drik Bala component uses the graded-aspect model (see
    /// [`compute_drik_bala`]) — classically-correct special aspects and graded
    /// virupas with the BPHS `/4` normalization, but NOT the continuous Sphuta
    /// Drishti interpolation of BPHS Ch. 27-28. All other balas follow BPHS.
    pub fn compute_full(
        planet: &str,
        lon_deg: f64,
        house: usize,
        speed: f64,
        input: &ShadBalaInput,
    ) -> Self {
        let sthana = compute_sthana_bala_full(planet, lon_deg, house);
        let dig = compute_dig_bala(planet, house);
        let naisargika = naisargika_bala(planet);
        let cheshta = compute_cheshta_bala(planet, speed);
        let kala = compute_kala_bala(planet, input);
        let drik = compute_drik_bala(planet, lon_deg, &input.all_planets, input.moon_lon);

        let total = sthana.total + dig + kala.total + cheshta + naisargika + drik;
        let required = required_minimum(planet);

        Shadbala {
            sthana_bala: sthana,
            dig_bala: dig,
            kala_bala: kala,
            cheshta_bala: cheshta,
            naisargika_bala: naisargika,
            drik_bala: drik,
            total,
            required_minimum: required,
            ratio: total / required,
        }
    }

    /// Returns `true` if the planet meets its minimum required strength.
    pub fn is_strong(&self) -> bool {
        self.ratio >= 1.0
    }
}

// ---------------------------------------------------------------------------
// Sthana Bala (Positional Strength)
// ---------------------------------------------------------------------------

fn compute_sthana_bala(planet: &str, lon_deg: f64, house: usize) -> SthanaBala {
    let uchcha = uchcha_bala(planet, lon_deg);
    let kendra = kendra_bala(house);
    let drekkana = drekkana_bala(planet, lon_deg);
    let ojhayugma = ojhayugma_bala(planet, lon_deg);

    SthanaBala {
        uchcha_bala: uchcha,
        saptavargaja_bala: 0.0,
        ojhayugma_bala: ojhayugma,
        kendra_bala: kendra,
        drekkana_bala: drekkana,
        total: uchcha + kendra + drekkana + ojhayugma,
    }
}

fn compute_sthana_bala_full(planet: &str, lon_deg: f64, house: usize) -> SthanaBala {
    let uchcha = uchcha_bala(planet, lon_deg);
    let kendra = kendra_bala(house);
    let drekkana = drekkana_bala(planet, lon_deg);
    let ojhayugma = ojhayugma_bala(planet, lon_deg);
    let saptavargaja = compute_saptavargaja_bala(planet, lon_deg);

    SthanaBala {
        uchcha_bala: uchcha,
        saptavargaja_bala: saptavargaja,
        ojhayugma_bala: ojhayugma,
        kendra_bala: kendra,
        drekkana_bala: drekkana,
        total: uchcha + kendra + drekkana + ojhayugma + saptavargaja,
    }
}

fn uchcha_bala(planet: &str, lon_deg: f64) -> f64 {
    let exalt_deg = match planet {
        "Sun" => 10.0,
        "Moon" => 33.0,
        "Mars" => 298.0,
        "Mercury" => 165.0,
        "Jupiter" => 95.0,
        "Venus" => 357.0,
        "Saturn" => 200.0,
        _ => return 0.0,
    };
    let diff = (lon_deg - exalt_deg).abs();
    let diff = if diff > 180.0 { 360.0 - diff } else { diff };
    60.0 * (1.0 - diff / 180.0)
}

fn kendra_bala(house: usize) -> f64 {
    match house {
        1 | 4 | 7 | 10 => 60.0,
        2 | 5 | 8 | 11 => 30.0,
        _ => 15.0,
    }
}

fn drekkana_bala(planet: &str, lon_deg: f64) -> f64 {
    let deg_in_sign = lon_deg % 30.0;
    let decan = (deg_in_sign / 10.0) as usize;
    let is_male = matches!(planet, "Sun" | "Mars" | "Jupiter");
    let is_female = matches!(planet, "Moon" | "Venus");

    match (decan, is_male, is_female) {
        (0, true, _) => 15.0,
        (2, _, true) => 15.0,
        (1, _, _) if !is_male && !is_female => 15.0, // Mercury, Saturn = neutral
        _ => 0.0,
    }
}

fn ojhayugma_bala(planet: &str, lon_deg: f64) -> f64 {
    let sign_idx = (lon_deg / 30.0) as usize % 12;
    let is_even_sign = sign_idx % 2 == 1;
    let prefers_even = matches!(planet, "Moon" | "Venus");

    if (prefers_even && is_even_sign) || (!prefers_even && !is_even_sign) {
        15.0
    } else {
        0.0
    }
}

// ---------------------------------------------------------------------------
// Saptavargaja Bala — Strength from 7 divisional charts
// Per BPHS Ch.27 vv.15-18: In each of the 7 varga charts (D1, D2, D3,
// D7, D9, D12, D30), a planet receives virupas based on its relationship
// to the sign lord of that varga position.
// ---------------------------------------------------------------------------

/// Virupas assigned per dignity classification in a single varga chart.
/// BPHS values: Moolatrikona=45, Own=30, Great Friend=22.5,
/// Friend=15, Neutral=7.5, Enemy=3.75, Great Enemy=1.875.
fn saptavarga_unit_strength(planet: &str, varga_rashi: Rashi) -> f64 {
    // Check exaltation first (treated as own sign level per some authorities,
    // but BPHS gives exalted placement a boost equivalent to own sign).
    if exaltation_sign(planet) == Some(varga_rashi) {
        return 30.0;
    }

    // Check moolatrikona — we only check sign, not degree within varga.
    if let Some((mt_rashi, _, _)) = moolatrikona_range(planet)
        && mt_rashi == varga_rashi
    {
        return 45.0;
    }

    // Check own sign.
    if own_signs(planet).contains(&varga_rashi) {
        return 30.0;
    }

    // Determine relationship to the sign lord.
    let lord = varga_rashi.lord();
    if lord == planet {
        // Planet is the lord of this sign — own sign (already caught above for
        // most cases, but defensive).
        return 30.0;
    }

    let is_friend = natural_friends(planet).contains(&lord);
    let is_enemy = natural_enemies(planet).contains(&lord);

    if is_friend {
        15.0
    } else if is_enemy {
        3.75
    } else {
        // Neutral
        7.5
    }
}

/// Compute Saptavargaja Bala: sum of virupas across D1, D2, D3, D7, D9, D12, D30.
fn compute_saptavargaja_bala(planet: &str, lon_deg: f64) -> f64 {
    let vargas = [
        VargaChart::D1,
        VargaChart::D2,
        VargaChart::D3,
        VargaChart::D7,
        VargaChart::D9,
        VargaChart::D12,
        VargaChart::D30,
    ];

    vargas
        .iter()
        .map(|v| {
            let varga_rashi = compute_varga_sign(lon_deg, *v);
            saptavarga_unit_strength(planet, varga_rashi)
        })
        .sum()
}

// ---------------------------------------------------------------------------
// Kala Bala (Temporal Strength)
// Per BPHS Ch.27 vv.19-28
// ---------------------------------------------------------------------------

fn compute_kala_bala(planet: &str, input: &ShadBalaInput) -> KalaBala {
    let nathonnatha = nathonnatha_bala(planet, input.day_fraction);
    let paksha = paksha_bala(planet, input.sun_lon, input.moon_lon);
    let tribhaga = tribhaga_bala(planet, input.day_fraction);
    let vara = Vara::from_jd(input.jd);
    let vara_b = vara_bala(planet, &vara);
    let abda_b = abda_bala(planet, input.jd);
    let masa_b = masa_bala(planet, input.jd);
    let hora_b = hora_bala(planet, input.jd, input.day_fraction);
    let ayana = input
        .all_planets
        .iter()
        .find(|p| p.name == planet)
        .map_or(0.0, |p| ayana_bala(planet, p.declination));
    let yuddha = yuddha_bala(planet, input);

    let total =
        nathonnatha + paksha + tribhaga + vara_b + abda_b + masa_b + hora_b + ayana + yuddha;

    KalaBala {
        nathonnatha_bala: nathonnatha,
        paksha_bala: paksha,
        tribhaga_bala: tribhaga,
        abda_bala: abda_b,
        masa_bala: masa_b,
        vara_bala: vara_b,
        hora_bala: hora_b,
        ayana_bala: ayana,
        yuddha_bala: yuddha,
        total,
    }
}

/// Ayana Bala from the signed declination in degrees (BPHS Ch. 27,
/// Santhanam vv. 23-24). 0 to 60 virupas, doubled for the Sun.
fn ayana_bala(planet: &str, declination_deg: f64) -> f64 {
    let k = match planet {
        "Sun" | "Mars" | "Jupiter" | "Venus" => declination_deg,
        "Moon" | "Saturn" => -declination_deg,
        "Mercury" => declination_deg.abs(),
        _ => return 0.0,
    };
    let k = k.clamp(-24.0, 24.0);
    let b = (24.0 + k) * 60.0 / 48.0;
    if planet == "Sun" { 2.0 * b } else { b }
}

/// Disc diameter in seconds of arc used by Yuddha Bala, for the five planets
/// that can fight (B.V. Raman's table, see [`Shadbala::compute_full`]).
fn disc_diameter(planet: &str) -> Option<f64> {
    match planet {
        "Mars" => Some(9.4),
        "Mercury" => Some(6.6),
        "Jupiter" => Some(190.4),
        "Venus" => Some(16.6),
        "Saturn" => Some(158.0),
        _ => None,
    }
}

/// Sum of the six Sthana, Dig, Nathonnatha, Paksha, Tribhaga and Hora Bala.
fn yuddha_partial_sum(p: &PlanetPosition, input: &ShadBalaInput) -> f64 {
    compute_sthana_bala_full(p.name, p.longitude, p.house).total
        + compute_dig_bala(p.name, p.house)
        + nathonnatha_bala(p.name, input.day_fraction)
        + paksha_bala(p.name, input.sun_lon, input.moon_lon)
        + tribhaga_bala(p.name, input.day_fraction)
        + hora_bala(p.name, input.jd, input.day_fraction)
}

/// Yuddha Bala (planetary war), Sripati's formulation. See
/// [`Shadbala::compute_full`] for the convention.
fn yuddha_bala(planet: &str, input: &ShadBalaInput) -> f64 {
    if disc_diameter(planet).is_none() {
        return 0.0;
    }
    let Some(me) = input.all_planets.iter().find(|p| p.name == planet) else {
        return 0.0;
    };
    let my_sum = yuddha_partial_sum(me, input);
    let my_diameter = disc_diameter(planet).unwrap_or(0.0);

    let mut total = 0.0;
    for other in &input.all_planets {
        let Some(other_diameter) = disc_diameter(other.name) else {
            continue;
        };
        if other.name == planet {
            continue;
        }
        let gap = (me.longitude - other.longitude).rem_euclid(360.0);
        if gap.min(360.0 - gap) >= 1.0 {
            continue;
        }
        let diameter_diff = (my_diameter - other_diameter).abs();
        if diameter_diff < 1e-9 {
            continue;
        }
        let other_sum = yuddha_partial_sum(other, input);
        let magnitude = (my_sum - other_sum).abs() / diameter_diff;
        let i_win = if (me.declination - other.declination).abs() > 1e-12 {
            me.declination > other.declination
        } else {
            my_sum > other_sum
        };
        total += if i_win { magnitude } else { -magnitude };
    }
    total
}

/// Nathonnatha Bala (Day/Night strength).
/// Sun, Jupiter, Venus are strong by day (diurnal planets).
/// Moon, Mars, Saturn are strong by night (nocturnal planets).
/// Mercury is always strong (ubhayachari).
/// Day fraction: 0.25 = 6am (sunrise approx), 0.75 = 6pm (sunset approx).
/// Strength is proportional to distance from the weakest point.
/// Max 60 virupas.
fn nathonnatha_bala(planet: &str, day_fraction: f64) -> f64 {
    // Normalize day_fraction to 0..1
    let df = day_fraction.rem_euclid(1.0);

    // Mercury is always strong
    if planet == "Mercury" {
        return 60.0;
    }

    // is_day: true if between ~6am (0.25) and ~6pm (0.75)
    let _is_day = (0.25..0.75).contains(&df);
    let diurnal = matches!(planet, "Sun" | "Jupiter" | "Venus");

    // Distance from noon (0.5) or midnight (0.0/1.0) determines proportional
    // strength. For diurnal planets, noon = max strength, midnight = 0.
    // For nocturnal planets, midnight = max, noon = 0.
    if diurnal {
        // Distance from midnight (0.0 or 1.0), normalized to 0-0.5
        let dist_from_midnight = if df <= 0.5 { df } else { 1.0 - df };
        // At noon (0.5 from midnight) = 60, at midnight (0 from midnight) = 0
        60.0 * (dist_from_midnight / 0.5)
    } else {
        // Nocturnal: Moon, Mars, Saturn
        // Distance from noon (0.5), normalized to 0-0.5
        let dist_from_noon = (df - 0.5).abs();
        // At midnight (0.5 from noon) = 60, at noon (0 from noon) = 0
        60.0 * (dist_from_noon / 0.5)
    }
}

/// Paksha Bala (Lunar phase strength).
/// Benefics (Jupiter, Venus, waxing Moon, well-associated Mercury) gain
/// strength in Shukla Paksha (waxing). Malefics (Saturn, Mars, waning Moon)
/// gain in Krishna Paksha (waning).
/// Formula: elongation = (moon_lon - sun_lon) mod 360.
/// For benefics: strength = elongation / 3 (max 60 at full moon, 0 at new).
/// For malefics: strength = (360 - elongation) / 3, but capped at 60.
/// BPHS gives max 60 virupas.
fn paksha_bala(planet: &str, sun_lon: f64, moon_lon: f64) -> f64 {
    let elongation = (moon_lon - sun_lon).rem_euclid(360.0);

    // Shukla paksha: elongation 0-180 (waxing)
    // Krishna paksha: elongation 180-360 (waning)

    // For BPHS: benefic strength = elongation/6 for Shukla, (360-elongation)/6 for Krishna
    // This gives max 30 virupas. Some authorities use /3 for max 60.
    // We use the standard BPHS formulation:
    // Benefic Paksha Bala = |elongation mapped to 0-180| / 3
    // Malefic Paksha Bala = 60 - benefic_value

    let benefic_value = if elongation <= 180.0 {
        elongation / 3.0
    } else {
        (360.0 - elongation) / 3.0
    };

    let is_benefic = is_natural_benefic(planet, moon_lon, sun_lon);

    if is_benefic {
        benefic_value
    } else {
        60.0 - benefic_value
    }
}

/// Tribhaga Bala (Strength from 1/3 portions of day/night).
/// Day is divided into 3 parts, night into 3 parts.
/// First 1/3 of day = Mercury strong. 2nd 1/3 of day = Sun strong.
/// 3rd 1/3 of day = Saturn strong.
/// First 1/3 of night = Moon strong. 2nd 1/3 of night = Venus strong.
/// 3rd 1/3 of night = Mars strong.
/// Jupiter always gets Tribhaga Bala.
/// Strength = 60 virupas when applicable, 0 otherwise.
fn tribhaga_bala(planet: &str, day_fraction: f64) -> f64 {
    // Jupiter always gets Tribhaga Bala.
    if planet == "Jupiter" {
        return 60.0;
    }

    let df = day_fraction.rem_euclid(1.0);
    // Day: 0.25 (6am) to 0.75 (6pm) = 0.5 span, each third = ~0.1667
    // Night: 0.75 to 1.25 (next day 6am) = 0.5 span, each third = ~0.1667

    let is_day = (0.25..0.75).contains(&df);
    if is_day {
        let day_elapsed = df - 0.25; // 0 to 0.5
        let third = (day_elapsed / (0.5 / 3.0)) as usize;
        match (third.min(2), planet) {
            (0, "Mercury") => 60.0,
            (1, "Sun") => 60.0,
            (2, "Saturn") => 60.0,
            _ => 0.0,
        }
    } else {
        // Night: 0.75-1.0 and 0.0-0.25
        let night_elapsed = if df >= 0.75 { df - 0.75 } else { df + 0.25 };
        let third = (night_elapsed / (0.5 / 3.0)) as usize;
        match (third.min(2), planet) {
            (0, "Moon") => 60.0,
            (1, "Venus") => 60.0,
            (2, "Mars") => 60.0,
            _ => 0.0,
        }
    }
}

/// Vara Bala: The lord of the weekday gets 45 virupas.
fn vara_bala(planet: &str, vara: &Vara) -> f64 {
    if vara.lord() == planet { 45.0 } else { 0.0 }
}

/// Abda Bala: The lord of the year gets 15 virupas.
/// The Vedic year lord follows the weekday-lord cycle based on the Jovian
/// (Barhaspatya) year. We use a simplified calculation based on JD.
/// The planetary year lords cycle in the Kaldina order:
/// Sun, Venus, Mercury, Moon, Saturn, Jupiter, Mars (repeating).
fn abda_bala(planet: &str, jd: f64) -> f64 {
    let year_lord = year_lord_from_jd(jd);
    if year_lord == planet { 15.0 } else { 0.0 }
}

/// Masa Bala: The lord of the lunar month gets 30 virupas.
/// Uses a simplified month lord based on the synodic month count.
fn masa_bala(planet: &str, jd: f64) -> f64 {
    let month_lord = month_lord_from_jd(jd);
    if month_lord == planet { 30.0 } else { 0.0 }
}

/// Hora Bala: The lord of the current planetary hour gets 60 virupas.
/// Planetary hours cycle in the Chaldean order starting from the weekday lord.
/// Chaldean order: Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon.
fn hora_bala(planet: &str, jd: f64, day_fraction: f64) -> f64 {
    let hora_lord = hora_lord_from_jd(jd, day_fraction);
    if hora_lord == planet { 60.0 } else { 0.0 }
}

// ---------------------------------------------------------------------------
// Kala Bala helper functions
// ---------------------------------------------------------------------------

/// Chaldean order of planets (descending orbital period).
const CHALDEAN_ORDER: [&str; 7] = [
    "Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon",
];

/// Vedic year lord cycle (Kaldina order).
/// Starts from a known epoch and cycles every 7 years.
fn year_lord_from_jd(jd: f64) -> &'static str {
    // Reference: J2000 (2451545.0) = 2000 CE. The year lord for 2000 CE
    // is computed from the cycle. The lords cycle in weekday order:
    // Sun(0), Moon(1), Mars(2), Mercury(3), Jupiter(4), Venus(5), Saturn(6).
    // Using the Julian year number relative to an epoch.
    let year = 2000.0 + (jd - 2451545.0) / 365.25;
    let year_int = year.floor() as i64;
    // The cycle repeats every 7 years in the order:
    // Sun, Venus, Mercury, Moon, Saturn, Jupiter, Mars
    let cycle_order: [&str; 7] = [
        "Sun", "Venus", "Mercury", "Moon", "Saturn", "Jupiter", "Mars",
    ];
    let idx = year_int.rem_euclid(7) as usize;
    cycle_order[idx]
}

/// Month lord from JD — uses synodic month count from a known new moon epoch.
fn month_lord_from_jd(jd: f64) -> &'static str {
    // Known new moon: 2000-01-06 (JD ~2451550.1)
    let synodic_month = 29.530588853;
    let months_since = ((jd - 2451550.1) / synodic_month).floor() as i64;
    // Month lords cycle in the same weekday-lord order.
    let cycle_order: [&str; 7] = [
        "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn",
    ];
    let idx = months_since.rem_euclid(7) as usize;
    cycle_order[idx]
}

/// Hora (planetary hour) lord from JD and day fraction.
/// Each day has 24 planetary hours, cycling in the Chaldean order.
/// The first hour of each day is ruled by the weekday lord.
fn hora_lord_from_jd(jd: f64, day_fraction: f64) -> &'static str {
    let vara = Vara::from_jd(jd);
    let vara_lord = vara.lord();

    // Find the weekday lord's position in the Chaldean order
    let start_idx = CHALDEAN_ORDER
        .iter()
        .position(|&p| p == vara_lord)
        .unwrap_or(0);

    // Each planetary hour = 1/24 of a day
    // Day fraction 0.25 = sunrise = start of first hour
    let hours_since_sunrise = if day_fraction >= 0.25 {
        (day_fraction - 0.25) * 24.0
    } else {
        (day_fraction + 0.75) * 24.0
    };
    let hora_idx = hours_since_sunrise.floor() as usize % 24;

    // Each hora advances through the Chaldean order
    let planet_idx = (start_idx + hora_idx) % 7;
    CHALDEAN_ORDER[planet_idx]
}

// ---------------------------------------------------------------------------
// Drik Bala (Aspectual Strength)
//
// Grounded in the XALEN classical-text corpus (cleanroom translations):
//   • BPHS Ch.24 Sh.13 — "the sphuṭā dṛṣṭi (exact aspect) is produced; by its
//     force alone one should state the results of the bhavas." (Drik Bala derives
//     from the Sphuta Drishti / exact aspect value.)
//   • BPHS Ch.39 Sh.14 — "In full aspect (pūrṇa-dṛṣṭi) a full [result], in half
//     aspect (ardha-dṛṣṭi) half, in quarter aspect (pāda-dṛṣṭi) a quarter — to be
//     known progressively (kramāt)." (The graded full/half/quarter principle this
//     implementation uses.)
//   • Jataka Parijata Ch.1 Sh.31 — the special aspects: Saturn strongest by
//     quarter-aspect (3rd/10th), Jupiter by the triangular (5th/9th, ½), Mars by
//     three-quarter aspect (4th/8th).
// ---------------------------------------------------------------------------

/// Compute Drik Bala from aspects received by the planet.
/// Benefic aspects add strength, malefic aspects reduce it.
/// Full aspect (7th) = 60, 3/4 aspect = 45, 1/2 aspect = 30, 1/4 aspect = 15.
///
/// NOTE: this is a graded-aspect Drik Bala, NOT the continuous BPHS Sphuta
/// Drishti. It uses discrete house-based aspect fractions — the standard
/// Parashari graded virupas above (special aspects Mars 4th/8th, Jupiter
/// 5th/9th, Saturn 3rd/10th, all 7th = full; 1/4 = 15, 1/2 = 30, 3/4 = 45) —
/// rather than the continuous, degree-interpolated Sphuta Drishti curve of BPHS
/// Ch. 27-28. It DOES apply the BPHS `/4` (Shashtiamsha) normalization, so the
/// figure is on the standard Drik Bala virupa scale (a full benefic 7th aspect
/// contributes +15, a full malefic −15 — not ±60).
///
/// The SIGN, the special aspects and the relative ordering between planets —
/// what the Shadbala ranking depends on — are exact and tested; only the
/// continuous interpolation BETWEEN aspect points is approximated by the
/// nearest-house quantization.
///
/// On the exact continuous Sphuta Drishti: the XALEN BPHS corpus carries the
/// governing verses (Ch.24 Sh.13, Ch.39 Sh.14, and the Ch.27 strength list
/// "60, 55, 30, 20, 15, 8, 4, 2, 0") in transliterated Sanskrit, but the precise
/// angle→virupa piecewise mapping is NOT present in a translated, machine-usable
/// form. Committing the continuous curve would therefore mean decoding the
/// Sanskrit or copying a secondary source — neither is a verifiable primary
/// citation — so per the project's no-bluffing rule it is left as the graded
/// model above and flagged for a future pass once the numeric mapping is
/// available from a primary source with a cross-validation oracle (e.g.
/// Jagannatha Hora).
fn compute_drik_bala(
    planet: &str,
    planet_lon: f64,
    all_planets: &[PlanetPosition],
    moon_lon: f64,
) -> f64 {
    let mut drik: f64 = 0.0;

    // Sun longitude for determining Moon's benefic/malefic nature
    // (waxing Moon = benefic, waning Moon = malefic). We approximate by
    // checking if Moon is ahead of Sun within 180 degrees.
    // We need Sun's longitude from all_planets.
    let sun_lon = all_planets
        .iter()
        .find(|p| p.name == "Sun")
        .map(|p| p.longitude)
        .unwrap_or(0.0);

    for other in all_planets {
        if other.name == planet {
            continue;
        }

        // Drishti Kendra = the FORWARD arc FROM the aspecting planet (`other`)
        // TO the aspected point (`planet`), i.e. aspected − aspecting. Vedic
        // aspects are counted zodiacally forward from the aspecting graha, so
        // this direction is what selects the special-aspect house (Mars 4th/8th,
        // Jupiter 5th/9th, Saturn 3rd/10th). Computing it as aspecting − aspected
        // mirrors the asymmetric aspects onto the wrong planet.
        let sep = angular_separation(planet_lon, other.longitude);

        // Determine aspect strength based on house distance
        // In Vedic astrology, aspects are based on house-from relationships.
        // All planets aspect the 7th house (opposition) fully.
        // Mars additionally aspects 4th and 8th.
        // Jupiter additionally aspects 5th and 9th.
        // Saturn additionally aspects 3rd and 10th.
        let aspect_strength = vedic_aspect_strength(other.name, sep);

        if aspect_strength <= 0.0 {
            continue;
        }

        // Determine if aspecting planet is benefic or malefic
        let is_benefic_planet = is_natural_benefic(other.name, moon_lon, sun_lon);

        // Benefic aspects add, malefic aspects subtract
        let contribution = aspect_strength * if is_benefic_planet { 1.0 } else { -1.0 };
        drik += contribution;
    }

    // Apply the BPHS Drik Bala normalization: the summed graded-aspect virupas
    // are divided by 4 (Shashtiamsha) to land on the canonical Drik Bala scale.
    // Drik Bala can be negative (net malefic aspects).
    drik / 4.0
}

/// Calculate the angular separation between two longitudes (0-360).
fn angular_separation(lon1: f64, lon2: f64) -> f64 {
    (lon1 - lon2).rem_euclid(360.0)
}

/// Determine the Vedic aspect strength of a planet at a given angular
/// separation from the aspected point. Returns virupas (0-60).
///
/// All planets have full (60) aspect on the 7th house (180 degrees).
/// Special aspects:
/// - Mars: full aspect on 4th (90) and 8th (210-240) houses
/// - Jupiter: full aspect on 5th (120-150) and 9th (240-270) houses
/// - Saturn: full aspect on 3rd (60-90) and 10th (270-300) houses
///
/// For Drik Bala, we use the standard aspect fractions:
/// 3rd/10th house = 1/4 aspect (15 virupas) for generic planets
/// 4th/8th house = 3/4 aspect (45 virupas) for generic planets
/// 5th/9th house = 1/2 aspect (30 virupas) for generic planets
/// 7th house = full aspect (60 virupas) for all planets
fn vedic_aspect_strength(planet: &str, separation: f64) -> f64 {
    // Convert separation to approximate house (each house ~ 30 degrees)
    let house_from = ((separation / 30.0).round() as usize) % 12;

    // Tolerance for aspect matching (within 15 degrees of exact aspect point)
    // This allows orb-based partial strength.

    match planet {
        "Mars" => match house_from {
            // Mars special aspects: 4th, 7th, 8th
            3 => 60.0, // 4th house — full (special)
            6 => 60.0, // 7th house — full
            7 => 60.0, // 8th house — full (special)
            // Standard partial aspects for other houses
            2 | 9 => 15.0, // 3rd, 10th = 1/4
            4 | 8 => 30.0, // 5th, 9th = 1/2
            _ => 0.0,
        },
        "Jupiter" => match house_from {
            // Jupiter special aspects: 5th, 7th, 9th
            4 => 60.0, // 5th house — full (special)
            6 => 60.0, // 7th house — full
            8 => 60.0, // 9th house — full (special)
            // Standard partial aspects
            2 | 9 => 15.0,
            3 | 7 => 45.0, // 4th, 8th = 3/4
            _ => 0.0,
        },
        "Saturn" => match house_from {
            // Saturn special aspects: 3rd, 7th, 10th
            2 => 60.0, // 3rd house — full (special)
            6 => 60.0, // 7th house — full
            9 => 60.0, // 10th house — full (special)
            // Standard partial aspects
            3 | 7 => 45.0,
            4 | 8 => 30.0,
            _ => 0.0,
        },
        _ => {
            // Generic planets (Sun, Moon, Mercury, Venus): only 7th house full
            match house_from {
                6 => 60.0,     // 7th house — full
                2 | 9 => 15.0, // 3rd, 10th = 1/4
                3 | 7 => 45.0, // 4th, 8th = 3/4
                4 | 8 => 30.0, // 5th, 9th = 1/2
                _ => 0.0,
            }
        }
    }
}

/// Determine if a planet is a natural benefic.
/// Jupiter and Venus are always benefic.
/// Waxing Moon (Shukla Paksha) is benefic.
/// Mercury alone (not conjoined with malefics) is benefic.
/// For simplicity, we treat Mercury as benefic and Moon as benefic
/// when waxing (elongation from Sun 0-180).
fn is_natural_benefic(planet: &str, moon_lon: f64, sun_lon: f64) -> bool {
    match planet {
        "Jupiter" | "Venus" => true,
        "Moon" => {
            let elongation = (moon_lon - sun_lon).rem_euclid(360.0);
            elongation <= 180.0 // Waxing = benefic
        }
        "Mercury" => true, // Simplified: Mercury alone is benefic
        _ => false,        // Sun, Mars, Saturn, Rahu, Ketu = malefic
    }
}

// ---------------------------------------------------------------------------
// Existing components (unchanged)
// ---------------------------------------------------------------------------

fn compute_dig_bala(planet: &str, house: usize) -> f64 {
    let strongest_house = match planet {
        "Sun" | "Mars" => 10,
        "Moon" | "Venus" => 4,
        "Jupiter" | "Mercury" => 1,
        "Saturn" => 7,
        _ => return 0.0,
    };
    let diff = ((house as i32 - strongest_house).abs() % 12) as f64;
    let quadrant_diff = if diff > 6.0 { 12.0 - diff } else { diff };
    60.0 * (1.0 - quadrant_diff / 6.0)
}

fn naisargika_bala(planet: &str) -> f64 {
    // BPHS 27.14: "Shani, Mangal, Budha, Guru, Shukra, Chandra, Surya" (weakest→strongest)
    // = Sun > Moon > Venus > Jupiter > Mercury > Mars > Saturn
    match planet {
        "Sun" => 60.0,
        "Moon" => 51.43,
        "Venus" => 42.86,
        "Jupiter" => 34.29,
        "Mercury" => 25.71,
        "Mars" => 17.14,
        "Saturn" => 8.57,
        _ => 0.0,
    }
}

fn compute_cheshta_bala(planet: &str, speed_deg_day: f64) -> f64 {
    if matches!(planet, "Sun" | "Moon") {
        return 30.0; // simplified
    }
    if speed_deg_day < 0.0 {
        60.0 // retrograde = maximum cheshta bala
    } else if speed_deg_day.abs() < 0.01 {
        30.0 // stationary
    } else {
        15.0 // direct
    }
}

fn required_minimum(planet: &str) -> f64 {
    match planet {
        "Sun" => 390.0,
        "Moon" => 360.0,
        "Mars" => 300.0,
        "Mercury" => 420.0,
        "Jupiter" => 390.0,
        "Venus" => 330.0,
        "Saturn" => 300.0,
        _ => 300.0,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // === Existing tests (preserved) ===

    #[test]
    fn sun_exalted_in_aries_10deg() {
        let ub = uchcha_bala("Sun", 10.0);
        assert!(
            (ub - 60.0).abs() < 0.01,
            "Sun at 10deg (exact exaltation) should have 60 virupas, got {ub}"
        );
    }

    #[test]
    fn sun_debilitated_in_libra() {
        let ub = uchcha_bala("Sun", 190.0);
        assert!(
            ub < 5.0,
            "Sun at 190deg (near debilitation) should have ~0 virupas, got {ub}"
        );
    }

    #[test]
    fn kendra_bala_values() {
        assert_eq!(kendra_bala(1), 60.0);
        assert_eq!(kendra_bala(4), 60.0);
        assert_eq!(kendra_bala(2), 30.0);
        assert_eq!(kendra_bala(3), 15.0);
    }

    #[test]
    fn dig_bala_sun_in_10th() {
        let db = compute_dig_bala("Sun", 10);
        assert!(
            (db - 60.0).abs() < 0.01,
            "Sun in 10th should have max dig bala, got {db}"
        );
    }

    #[test]
    fn naisargika_order_bphs() {
        // BPHS 27.14: "Shani, Mangal, Budha, Guru, Shukra, Chandra, Surya" (weakest→strongest)
        // Reversed: Sun > Moon > Venus > Jupiter > Mercury > Mars > Saturn
        let s = naisargika_bala;
        assert!(s("Sun") > s("Moon"), "Sun > Moon");
        assert!(s("Moon") > s("Venus"), "Moon > Venus");
        assert!(s("Venus") > s("Jupiter"), "Venus > Jupiter");
        assert!(s("Jupiter") > s("Mercury"), "Jupiter > Mercury");
        assert!(s("Mercury") > s("Mars"), "Mercury > Mars");
        assert!(s("Mars") > s("Saturn"), "Mars > Saturn");
        assert_eq!(s("Sun"), 60.0);
        assert_eq!(s("Saturn"), 8.57);
    }

    #[test]
    fn retrograde_cheshta() {
        let cb = compute_cheshta_bala("Mars", -0.5);
        assert_eq!(cb, 60.0, "Retrograde should give max cheshta bala");
    }

    #[test]
    #[allow(deprecated)]
    fn full_shadbala_computation() {
        let sb = Shadbala::compute("Sun", 10.0, 10, 0.98);
        assert!(sb.total > 0.0, "Total shadbala should be positive");
        assert!(sb.ratio > 0.0, "Ratio should be positive");
        assert!(
            sb.sthana_bala.uchcha_bala > 50.0,
            "Sun exalted should have high uchcha"
        );
    }

    #[test]
    #[allow(deprecated)]
    fn shadbala_ratio_interpretation() {
        let strong = Shadbala::compute("Sun", 10.0, 10, 0.98);
        // Sun at exact exaltation in strongest house -- should be strong
        // (may or may not pass threshold depending on kala/drik placeholder values)
        assert!(
            strong.total > 100.0,
            "Well-placed Sun should have significant strength"
        );
    }

    // === Saptavargaja Bala tests ===

    #[test]
    fn saptavargaja_sun_in_aries() {
        // Sun at 10 degrees Aries (exalted in D1).
        // D1 = Aries, lord Mars is friend of Sun => friend (15) but
        // actually Sun is exalted in Aries => exaltation (30).
        let sv = compute_saptavargaja_bala("Sun", 10.0);
        // Should be positive and reasonably large (7 charts contributing)
        assert!(sv > 0.0, "Saptavargaja should be positive, got {sv}");
        // Maximum possible = 7 * 45 = 315. Minimum = 7 * 3.75 = 26.25
        assert!(
            sv >= 26.25 && sv <= 315.0,
            "Saptavargaja should be in valid range, got {sv}"
        );
    }

    #[test]
    fn saptavargaja_unit_own_sign() {
        // Sun in Leo (own sign) should get 30 virupas
        let strength = saptavarga_unit_strength("Sun", Rashi::Simha);
        // Leo is Sun's moolatrikona sign => 45
        assert!(
            (strength - 45.0).abs() < 0.01,
            "Sun in Leo (moolatrikona) should get 45 virupas, got {strength}"
        );
    }

    #[test]
    fn saptavargaja_unit_exaltation() {
        // Sun exalted in Aries
        let strength = saptavarga_unit_strength("Sun", Rashi::Mesha);
        assert!(
            (strength - 30.0).abs() < 0.01,
            "Sun exalted in Aries should get 30 virupas, got {strength}"
        );
    }

    #[test]
    fn saptavargaja_unit_enemy() {
        // Sun in Aquarius (Saturn's sign, Saturn is enemy of Sun)
        let strength = saptavarga_unit_strength("Sun", Rashi::Kumbha);
        assert!(
            (strength - 3.75).abs() < 0.01,
            "Sun in enemy sign should get 3.75 virupas, got {strength}"
        );
    }

    #[test]
    fn saptavargaja_unit_friend() {
        // Sun in Cancer (Moon's sign, Moon is friend of Sun)
        let strength = saptavarga_unit_strength("Sun", Rashi::Karka);
        assert!(
            (strength - 15.0).abs() < 0.01,
            "Sun in friend sign should get 15 virupas, got {strength}"
        );
    }

    #[test]
    fn saptavargaja_unit_neutral() {
        // Sun in Gemini (Mercury's sign). Mercury is neutral to Sun in BPHS.
        // Actually check: Sun's friends = Moon, Mars, Jupiter. Enemies = Venus, Saturn.
        // Mercury is neutral.
        let strength = saptavarga_unit_strength("Sun", Rashi::Mithuna);
        assert!(
            (strength - 7.5).abs() < 0.01,
            "Sun in neutral sign should get 7.5 virupas, got {strength}"
        );
    }

    #[test]
    fn saptavargaja_produces_nonzero_for_all_planets() {
        for planet in &[
            "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn",
        ] {
            let sv = compute_saptavargaja_bala(planet, 45.0); // 15 deg Taurus
            assert!(sv > 0.0, "{planet} saptavargaja should be > 0, got {sv}");
        }
    }

    #[test]
    fn sthana_bala_full_includes_saptavargaja() {
        let sb = compute_sthana_bala_full("Sun", 10.0, 10);
        assert!(
            sb.saptavargaja_bala > 0.0,
            "Full sthana bala should have nonzero saptavargaja"
        );
        // Total should be > old total (which excluded saptavargaja)
        let sb_old = compute_sthana_bala("Sun", 10.0, 10);
        assert!(
            sb.total > sb_old.total,
            "Full sthana total ({}) should exceed simplified ({})",
            sb.total,
            sb_old.total
        );
    }

    // === Kala Bala tests ===

    fn make_test_input() -> ShadBalaInput {
        ShadBalaInput {
            jd: 2451545.0, // J2000 = Saturday
            sun_lon: 280.0,
            moon_lon: 100.0,
            day_fraction: 0.5, // noon
            all_planets: vec![
                PlanetPosition {
                    name: "Sun",
                    longitude: 280.0,
                    speed: 1.0,
                    declination: 0.0,
                    house: 1,
                },
                PlanetPosition {
                    name: "Moon",
                    longitude: 100.0,
                    speed: 13.0,
                    declination: 0.0,
                    house: 1,
                },
                PlanetPosition {
                    name: "Mars",
                    longitude: 30.0,
                    speed: 0.5,
                    declination: 0.0,
                    house: 1,
                },
                PlanetPosition {
                    name: "Mercury",
                    longitude: 270.0,
                    speed: 1.2,
                    declination: 0.0,
                    house: 1,
                },
                PlanetPosition {
                    name: "Jupiter",
                    longitude: 50.0,
                    speed: 0.08,
                    declination: 0.0,
                    house: 1,
                },
                PlanetPosition {
                    name: "Venus",
                    longitude: 310.0,
                    speed: 1.1,
                    declination: 0.0,
                    house: 1,
                },
                PlanetPosition {
                    name: "Saturn",
                    longitude: 70.0,
                    speed: 0.03,
                    declination: 0.0,
                    house: 1,
                },
            ],
        }
    }

    #[test]
    fn nathonnatha_mercury_always_60() {
        assert_eq!(nathonnatha_bala("Mercury", 0.0), 60.0);
        assert_eq!(nathonnatha_bala("Mercury", 0.25), 60.0);
        assert_eq!(nathonnatha_bala("Mercury", 0.5), 60.0);
        assert_eq!(nathonnatha_bala("Mercury", 0.75), 60.0);
    }

    #[test]
    fn nathonnatha_sun_max_at_noon() {
        // Diurnal planet at noon = max 60
        let b = nathonnatha_bala("Sun", 0.5);
        assert!(
            (b - 60.0).abs() < 0.01,
            "Sun at noon should have 60 nathonnatha, got {b}"
        );
    }

    #[test]
    fn nathonnatha_sun_zero_at_midnight() {
        // Diurnal planet at midnight = 0
        let b = nathonnatha_bala("Sun", 0.0);
        assert!(
            b.abs() < 0.01,
            "Sun at midnight should have 0 nathonnatha, got {b}"
        );
    }

    #[test]
    fn nathonnatha_moon_max_at_midnight() {
        // Nocturnal planet at midnight = max 60
        let b = nathonnatha_bala("Moon", 0.0);
        assert!(
            (b - 60.0).abs() < 0.01,
            "Moon at midnight should have 60 nathonnatha, got {b}"
        );
    }

    #[test]
    fn nathonnatha_moon_zero_at_noon() {
        // Nocturnal planet at noon = 0
        let b = nathonnatha_bala("Moon", 0.5);
        assert!(
            b.abs() < 0.01,
            "Moon at noon should have 0 nathonnatha, got {b}"
        );
    }

    #[test]
    fn nathonnatha_mars_nocturnal() {
        // Mars is nocturnal, should be stronger at night
        let night = nathonnatha_bala("Mars", 0.0); // midnight
        let day = nathonnatha_bala("Mars", 0.5); // noon
        assert!(
            night > day,
            "Mars (nocturnal) should be stronger at midnight ({night}) than noon ({day})"
        );
    }

    #[test]
    fn paksha_bala_benefic_at_full_moon() {
        // Full moon: elongation ~180 degrees
        // Benefics should have max Paksha Bala
        let b = paksha_bala("Jupiter", 0.0, 180.0);
        assert!(
            (b - 60.0).abs() < 0.01,
            "Jupiter (benefic) at full moon should have 60 paksha bala, got {b}"
        );
    }

    #[test]
    fn paksha_bala_malefic_at_new_moon() {
        // New moon: elongation ~0 degrees
        // Malefics should have max Paksha Bala (60 - 0 = 60)
        let b = paksha_bala("Saturn", 0.0, 0.0);
        assert!(
            (b - 60.0).abs() < 0.01,
            "Saturn (malefic) at new moon should have 60 paksha bala, got {b}"
        );
    }

    #[test]
    fn paksha_bala_benefic_at_new_moon() {
        // New moon: elongation ~0 degrees
        // Benefics should have 0 Paksha Bala
        let b = paksha_bala("Jupiter", 0.0, 0.0);
        assert!(
            b.abs() < 0.01,
            "Jupiter (benefic) at new moon should have 0 paksha bala, got {b}"
        );
    }

    #[test]
    fn paksha_bala_range() {
        // Test that Paksha Bala is always in [0, 60]
        for elong in (0..360).step_by(30) {
            let b_benefic = paksha_bala("Jupiter", 0.0, elong as f64);
            let b_malefic = paksha_bala("Saturn", 0.0, elong as f64);
            assert!(
                b_benefic >= 0.0 && b_benefic <= 60.0,
                "Benefic paksha bala out of range at elongation {elong}: {b_benefic}"
            );
            assert!(
                b_malefic >= 0.0 && b_malefic <= 60.0,
                "Malefic paksha bala out of range at elongation {elong}: {b_malefic}"
            );
        }
    }

    #[test]
    fn tribhaga_jupiter_always_60() {
        for frac in [0.0, 0.1, 0.25, 0.4, 0.5, 0.6, 0.75, 0.9] {
            let b = tribhaga_bala("Jupiter", frac);
            assert_eq!(
                b, 60.0,
                "Jupiter should always get 60 tribhaga bala at frac {frac}"
            );
        }
    }

    #[test]
    fn tribhaga_mercury_first_third_day() {
        // First 1/3 of day = 0.25 to ~0.417
        let b = tribhaga_bala("Mercury", 0.3);
        assert_eq!(b, 60.0, "Mercury in first third of day should get 60");
        // Mercury NOT in first third
        let b2 = tribhaga_bala("Mercury", 0.5);
        assert_eq!(b2, 0.0, "Mercury in second third of day should get 0");
    }

    #[test]
    fn tribhaga_sun_second_third_day() {
        // Second 1/3 of day = ~0.417 to ~0.583
        let b = tribhaga_bala("Sun", 0.5);
        assert_eq!(b, 60.0, "Sun in second third of day should get 60");
    }

    #[test]
    fn tribhaga_saturn_third_third_day() {
        // Third 1/3 of day = ~0.583 to 0.75
        let b = tribhaga_bala("Saturn", 0.7);
        assert_eq!(b, 60.0, "Saturn in third third of day should get 60");
    }

    #[test]
    fn tribhaga_moon_first_third_night() {
        // First 1/3 of night = 0.75 to ~0.917
        let b = tribhaga_bala("Moon", 0.8);
        assert_eq!(b, 60.0, "Moon in first third of night should get 60");
    }

    #[test]
    fn tribhaga_venus_second_third_night() {
        // Second 1/3 of night = ~0.917 to ~1.083 = ~0.083
        let b = tribhaga_bala("Venus", 0.0); // midnight, in second third
        assert_eq!(b, 60.0, "Venus in second third of night should get 60");
    }

    #[test]
    fn tribhaga_mars_third_third_night() {
        // Third 1/3 of night = ~0.083 to 0.25
        let b = tribhaga_bala("Mars", 0.15);
        assert_eq!(b, 60.0, "Mars in third third of night should get 60");
    }

    #[test]
    fn vara_bala_saturday_saturn() {
        // J2000 = Saturday
        let vara = Vara::from_jd(2451545.0);
        assert_eq!(vara_bala("Saturn", &vara), 45.0);
        assert_eq!(vara_bala("Sun", &vara), 0.0);
    }

    #[test]
    fn hora_bala_first_hora_saturday() {
        // Saturday at sunrise (0.25). First hora = Saturn (weekday lord).
        let b = hora_bala("Saturn", 2451545.0, 0.25);
        assert_eq!(
            b, 60.0,
            "Saturn should get hora bala on first hora of Saturday"
        );
    }

    #[test]
    fn abda_bala_nonzero_for_some_planet() {
        // At least one planet should be year lord
        let mut found = false;
        for planet in &[
            "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn",
        ] {
            if abda_bala(planet, 2451545.0) > 0.0 {
                found = true;
            }
        }
        assert!(found, "At least one planet should have abda bala");
    }

    #[test]
    fn masa_bala_nonzero_for_some_planet() {
        let mut found = false;
        for planet in &[
            "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn",
        ] {
            if masa_bala(planet, 2451545.0) > 0.0 {
                found = true;
            }
        }
        assert!(found, "At least one planet should have masa bala");
    }

    #[test]
    fn kala_bala_has_subcomponents() {
        let input = make_test_input();
        let kala = compute_kala_bala("Sun", &input);
        // Sun at noon: nathonnatha = 60, tribhaga = 60 (second third)
        assert!(
            kala.nathonnatha_bala > 0.0,
            "Sun at noon should have positive nathonnatha"
        );
        assert!(kala.total > 0.0, "Kala bala total should be positive");
        // Total should equal sum of sub-components
        let expected = kala.nathonnatha_bala
            + kala.paksha_bala
            + kala.tribhaga_bala
            + kala.abda_bala
            + kala.masa_bala
            + kala.vara_bala
            + kala.hora_bala
            + kala.ayana_bala
            + kala.yuddha_bala;
        assert!(
            (kala.total - expected).abs() < 0.01,
            "Kala total ({}) should equal sum of subs ({expected})",
            kala.total
        );
    }

    // === Drik Bala tests ===

    #[test]
    fn drik_bala_no_aspects() {
        // Planet alone, no other planets
        let drik = compute_drik_bala("Sun", 0.0, &[], 100.0);
        assert_eq!(drik, 0.0, "No other planets = 0 drik bala");
    }

    #[test]
    fn drik_bala_benefic_opposition() {
        // Jupiter at 180 degrees from Sun (7th house aspect = full 60 virupas),
        // normalized by the BPHS /4 → +15 Drik Bala. Jupiter is benefic.
        let planets = vec![PlanetPosition {
            name: "Jupiter",
            longitude: 180.0,
            speed: 0.08,
            declination: 0.0,
            house: 1,
        }];
        let drik = compute_drik_bala("Sun", 0.0, &planets, 0.0);
        assert!(
            (drik - 15.0).abs() < 0.01,
            "Benefic full opposition should give +15 drik bala (60/4), got {drik}"
        );
    }

    #[test]
    fn drik_bala_malefic_opposition() {
        // Saturn at 180 degrees from Sun (7th house aspect = full 60 virupas),
        // normalized by the BPHS /4 → -15 Drik Bala. Saturn is malefic.
        let planets = vec![PlanetPosition {
            name: "Saturn",
            longitude: 180.0,
            speed: 0.03,
            declination: 0.0,
            house: 1,
        }];
        let drik = compute_drik_bala("Sun", 0.0, &planets, 0.0);
        assert!(
            (drik - (-15.0)).abs() < 0.01,
            "Malefic full opposition should give -15 drik bala (-60/4), got {drik}"
        );
    }

    #[test]
    fn drik_bala_within_bphs_virupa_scale() {
        // After the BPHS /4 normalization every single-aspect contribution is at
        // most ±15 virupas; even with several overlapping aspects the magnitude
        // stays on the Drik Bala scale (well within ±90).
        let planets = vec![
            PlanetPosition {
                name: "Jupiter",
                longitude: 180.0,
                speed: 0.08,
                declination: 0.0,
                house: 1,
            },
            PlanetPosition {
                name: "Venus",
                longitude: 90.0,
                speed: 1.2,
                declination: 0.0,
                house: 1,
            },
            PlanetPosition {
                name: "Saturn",
                longitude: 270.0,
                speed: 0.03,
                declination: 0.0,
                house: 1,
            },
            PlanetPosition {
                name: "Mars",
                longitude: 120.0,
                speed: 0.5,
                declination: 0.0,
                house: 1,
            },
            PlanetPosition {
                name: "Sun",
                longitude: 60.0,
                speed: 0.98,
                declination: 0.0,
                house: 1,
            },
        ];
        for target in [
            "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn",
        ] {
            let drik = compute_drik_bala(target, 0.0, &planets, 30.0);
            assert!(
                drik.abs() <= 90.0,
                "{target} Drik Bala {drik} should stay on the BPHS virupa scale (|x| <= 90)"
            );
        }
    }

    #[test]
    fn drik_bala_mars_special_aspect() {
        // The aspected planet must sit 90° FORWARD of Mars (Mars's 4th house) to
        // receive Mars's special 4th aspect (full 60). With Mars (aspecting) at
        // 0° and Mercury (aspected) at 90°, the forward arc aspected−aspecting =
        // 90° = the 4th house. Mars is malefic, so drik must be negative.
        // (Reversing to put Mercury at 0° and Mars at 90° would be Mars's 10th
        // house from Mercury — only a 1/4 aspect — which the old test wrongly
        // labeled the 4th.)
        let planets = vec![PlanetPosition {
            name: "Mars",
            longitude: 0.0,
            speed: 0.5,
            declination: 0.0,
            house: 1,
        }];
        let drik = compute_drik_bala("Mercury", 90.0, &planets, 0.0);
        assert!(
            drik < 0.0,
            "Mars's 4th-house special aspect (malefic) should give negative drik bala, got {drik}"
        );

        // Direction guard: the SAME geometry reversed (Mercury at 0°, Mars at
        // 90°) puts Mercury in Mars's 10th house (1/4 aspect, −15), NOT the 4th
        // (−60). Pin the magnitudes so the separation direction can't regress.
        let reversed = vec![PlanetPosition {
            name: "Mars",
            longitude: 90.0,
            speed: 0.5,
            declination: 0.0,
            house: 1,
        }];
        let drik_4th = compute_drik_bala("Mercury", 90.0, &planets, 0.0);
        let drik_10th = compute_drik_bala("Mercury", 0.0, &reversed, 0.0);
        assert!(
            drik_4th < drik_10th,
            "4th-aspect (−60/4) must be more negative than the 10th-aspect (−15/4): \
             4th={drik_4th}, 10th={drik_10th}"
        );
    }

    #[test]
    fn drik_bala_mixed_aspects() {
        // Jupiter at 180 (benefic full) + Saturn at 180 (malefic full)
        // Net should be ~0
        let planets = vec![
            PlanetPosition {
                name: "Jupiter",
                longitude: 180.0,
                speed: 0.08,
                declination: 0.0,
                house: 1,
            },
            PlanetPosition {
                name: "Saturn",
                longitude: 180.0,
                speed: 0.03,
                declination: 0.0,
                house: 1,
            },
        ];
        let drik = compute_drik_bala("Sun", 0.0, &planets, 0.0);
        assert!(
            drik.abs() < 0.01,
            "Equal benefic + malefic full aspects should net ~0, got {drik}"
        );
    }

    #[test]
    fn drik_bala_can_be_negative() {
        // Multiple malefic aspects
        let planets = vec![
            PlanetPosition {
                name: "Saturn",
                longitude: 180.0,
                speed: 0.03,
                declination: 0.0,
                house: 1,
            },
            PlanetPosition {
                name: "Mars",
                longitude: 180.0,
                speed: 0.5,
                declination: 0.0,
                house: 1,
            },
        ];
        let drik = compute_drik_bala("Venus", 0.0, &planets, 0.0);
        assert!(
            drik < 0.0,
            "Multiple malefic aspects should produce negative drik bala, got {drik}"
        );
    }

    #[test]
    fn vedic_aspect_all_planets_7th_house() {
        // All planets have full aspect on 7th house (separation ~180 degrees)
        for planet in &[
            "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn",
        ] {
            let strength = vedic_aspect_strength(planet, 180.0);
            assert_eq!(
                strength, 60.0,
                "{planet} should have full aspect at 180 degrees"
            );
        }
    }

    #[test]
    fn vedic_aspect_mars_special() {
        // Mars special: 4th house (90) and 8th house (210-240)
        assert_eq!(
            vedic_aspect_strength("Mars", 90.0),
            60.0,
            "Mars should have full aspect on 4th house"
        );
        assert_eq!(
            vedic_aspect_strength("Mars", 210.0),
            60.0,
            "Mars should have full aspect on 8th house"
        );
    }

    #[test]
    fn vedic_aspect_jupiter_special() {
        // Jupiter special: 5th house (4 signs = 120 deg) and 9th house (8 signs = 240 deg)
        assert_eq!(
            vedic_aspect_strength("Jupiter", 120.0),
            60.0,
            "Jupiter should have full aspect on 5th house"
        );
        assert_eq!(
            vedic_aspect_strength("Jupiter", 240.0),
            60.0,
            "Jupiter should have full aspect on 9th house"
        );
    }

    #[test]
    fn vedic_aspect_saturn_special() {
        // Saturn special: 3rd house (2 signs = 60 deg) and 10th house (9 signs = 270 deg)
        assert_eq!(
            vedic_aspect_strength("Saturn", 60.0),
            60.0,
            "Saturn should have full aspect on 3rd house"
        );
        assert_eq!(
            vedic_aspect_strength("Saturn", 270.0),
            60.0,
            "Saturn should have full aspect on 10th house"
        );
    }

    // === Full compute_full integration test ===

    #[test]
    fn compute_full_produces_valid_shadbala() {
        let input = make_test_input();
        let sb = Shadbala::compute_full("Sun", 280.0, 10, 1.0, &input);

        assert!(sb.total > 0.0, "Full shadbala total should be positive");
        assert!(sb.ratio > 0.0, "Full shadbala ratio should be positive");
        assert!(
            sb.sthana_bala.saptavargaja_bala > 0.0,
            "Full shadbala should have nonzero saptavargaja"
        );
        assert!(
            sb.kala_bala.total > 0.0,
            "Full shadbala should have positive kala bala"
        );

        // Total should equal the sum of all components
        let expected = sb.sthana_bala.total
            + sb.dig_bala
            + sb.kala_bala.total
            + sb.cheshta_bala
            + sb.naisargika_bala
            + sb.drik_bala;
        assert!(
            (sb.total - expected).abs() < 0.01,
            "Total ({}) should match component sum ({expected})",
            sb.total
        );
    }

    #[test]
    fn compute_full_all_seven_planets() {
        let input = make_test_input();
        for pp in &input.all_planets {
            let sb = Shadbala::compute_full(pp.name, pp.longitude, 1, pp.speed, &input);
            assert!(
                sb.total != 0.0,
                "{} should have nonzero total shadbala",
                pp.name
            );
            assert!(
                sb.required_minimum > 0.0,
                "{} should have positive required minimum",
                pp.name
            );
        }
    }

    #[test]
    fn compute_full_kala_bala_is_broken_down() {
        let input = make_test_input();
        let sb = Shadbala::compute_full("Jupiter", 50.0, 2, 0.08, &input);
        let k = &sb.kala_bala;
        // Jupiter: nathonnatha = 60 at noon (diurnal), tribhaga = 60 (always)
        assert_eq!(k.tribhaga_bala, 60.0, "Jupiter should always have tribhaga");
        // Verify total decomposition
        let sub_sum = k.nathonnatha_bala
            + k.paksha_bala
            + k.tribhaga_bala
            + k.abda_bala
            + k.masa_bala
            + k.vara_bala
            + k.hora_bala
            + k.ayana_bala
            + k.yuddha_bala;
        assert!(
            (k.total - sub_sum).abs() < 0.01,
            "Kala sub-sum ({sub_sum}) should match total ({})",
            k.total
        );
    }

    #[test]
    #[allow(deprecated)]
    fn backwards_compat_simplified_compute() {
        let sb = Shadbala::compute("Sun", 10.0, 10, 0.98);
        assert!(sb.total > 0.0);
        assert!(
            sb.kala_bala.total == 30.0,
            "Simplified compute should still use 30.0 for kala"
        );
        assert!(
            sb.drik_bala == 0.0,
            "Simplified compute should still use 0.0 for drik"
        );
        assert!(
            sb.sthana_bala.saptavargaja_bala == 0.0,
            "Simplified compute should still use 0.0 for saptavargaja"
        );
    }

    #[test]
    fn is_natural_benefic_checks() {
        assert!(is_natural_benefic("Jupiter", 0.0, 0.0));
        assert!(is_natural_benefic("Venus", 0.0, 0.0));
        assert!(is_natural_benefic("Mercury", 0.0, 0.0));
        // Waxing Moon (Moon ahead of Sun by <180)
        assert!(is_natural_benefic("Moon", 90.0, 0.0));
        // Waning Moon
        assert!(!is_natural_benefic("Moon", 270.0, 0.0));
        // Malefics
        assert!(!is_natural_benefic("Sun", 0.0, 0.0));
        assert!(!is_natural_benefic("Mars", 0.0, 0.0));
        assert!(!is_natural_benefic("Saturn", 0.0, 0.0));
    }

    // === Ayana Bala and Yuddha Bala (issue #46) ===

    fn pos(name: &'static str, longitude: f64, declination: f64, house: usize) -> PlanetPosition {
        PlanetPosition {
            name,
            longitude,
            speed: 1.0,
            declination,
            house,
        }
    }

    fn input_with(planets: Vec<PlanetPosition>) -> ShadBalaInput {
        ShadBalaInput {
            all_planets: planets,
            ..make_test_input()
        }
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.05
    }

    #[test]
    fn kranti_follows_the_classical_formula() {
        let eps = 23.44;
        assert!(close(kranti_deg(90.0, eps), 23.44));
        assert!(close(kranti_deg(270.0, eps), -23.44));
        assert!(kranti_deg(0.0, eps).abs() < 1e-9);
        assert!(kranti_deg(180.0, eps).abs() < 1e-9);
    }

    #[test]
    fn ayana_sun_is_doubled_and_follows_the_solstices() {
        let eps = 23.44;
        let summer = ayana_bala("Sun", kranti_deg(90.0, eps));
        let winter = ayana_bala("Sun", kranti_deg(270.0, eps));
        assert!(close(summer, 118.6), "summer solstice Sun {summer}");
        assert!(close(winter, 1.4), "winter solstice Sun {winter}");
        assert!(close(ayana_bala("Sun", 0.0), 60.0));
        assert!(close(ayana_bala("Mars", 0.0), 30.0));
    }

    #[test]
    fn ayana_sign_convention_per_planet() {
        assert!(close(ayana_bala("Moon", -20.0), 55.0));
        assert!(close(ayana_bala("Moon", 20.0), 5.0));
        assert!(close(ayana_bala("Saturn", 20.0), 5.0));
        assert!(close(ayana_bala("Saturn", -20.0), 55.0));
        for p in ["Mars", "Jupiter", "Venus"] {
            assert!(close(ayana_bala(p, 20.0), 55.0), "{p} north");
            assert!(close(ayana_bala(p, -20.0), 5.0), "{p} south");
        }
        assert!(close(ayana_bala("Mercury", 20.0), 55.0));
        assert!(close(ayana_bala("Mercury", -20.0), 55.0));
    }

    #[test]
    fn ayana_clamps_declination_beyond_24_degrees() {
        assert!(close(ayana_bala("Mars", 26.0), 60.0));
        assert!(close(ayana_bala("Mars", -26.0), 0.0));
        assert!(close(ayana_bala("Moon", -29.0), 60.0));
        assert!(close(ayana_bala("Mercury", -26.0), 60.0));
        assert!(close(ayana_bala("Sun", 26.0), 120.0));
    }

    #[test]
    fn ayana_stays_in_range_for_every_planet() {
        let mut d = -30.0;
        while d <= 30.0 {
            for p in ["Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"] {
                let b = ayana_bala(p, d);
                assert!((0.0..=60.0).contains(&b), "{p} at {d}: {b}");
            }
            let sun = ayana_bala("Sun", d);
            assert!((0.0..=120.0).contains(&sun), "Sun at {d}: {sun}");
            d += 0.5;
        }
    }

    #[test]
    fn ayana_matches_the_issue_46_sun_example() {
        // Sun about 20 degrees north gives about 110 virupas, the figure the
        // reporter of #46 expected for a 1983 Delhi chart.
        let sun = pos("Sun", 280.0, 20.0, 10);
        let mut planets = make_test_input().all_planets;
        planets[0] = sun;
        let input = input_with(planets);
        let sb = Shadbala::compute_full("Sun", 280.0, 10, 1.0, &input);
        assert!(
            close(sb.kala_bala.ayana_bala, 110.0),
            "{}",
            sb.kala_bala.ayana_bala
        );
    }

    #[test]
    fn compute_full_total_includes_ayana_and_yuddha() {
        let mut planets = make_test_input().all_planets;
        planets[0] = pos("Sun", 280.0, 20.0, 10);
        let input = input_with(planets);
        let sb = Shadbala::compute_full("Sun", 280.0, 10, 1.0, &input);
        let k = &sb.kala_bala;
        let expected = k.nathonnatha_bala
            + k.paksha_bala
            + k.tribhaga_bala
            + k.abda_bala
            + k.masa_bala
            + k.vara_bala
            + k.hora_bala
            + k.ayana_bala
            + k.yuddha_bala;
        assert!((k.total - expected).abs() < 1e-9);
        let six = sb.sthana_bala.total
            + sb.dig_bala
            + k.total
            + sb.cheshta_bala
            + sb.naisargika_bala
            + sb.drik_bala;
        assert!((sb.total - six).abs() < 1e-9);
        // Compared with the seven-part Kala Bala of 0.7.0 the total rose by
        // exactly the Ayana and Yuddha amounts.
        let seven = k.nathonnatha_bala
            + k.paksha_bala
            + k.tribhaga_bala
            + k.abda_bala
            + k.masa_bala
            + k.vara_bala
            + k.hora_bala;
        assert!((k.total - seven - k.ayana_bala - k.yuddha_bala).abs() < 1e-9);
        assert!(k.ayana_bala > 100.0);
    }

    #[test]
    fn planet_missing_from_all_planets_gets_zero_ayana_and_yuddha() {
        let input = input_with(vec![pos("Mars", 30.0, 10.0, 1)]);
        let sb = Shadbala::compute_full("Venus", 30.2, 1, 1.0, &input);
        assert_eq!(sb.kala_bala.ayana_bala, 0.0);
        assert_eq!(sb.kala_bala.yuddha_bala, 0.0);
    }

    fn yuddha_of(input: &ShadBalaInput, name: &str) -> f64 {
        let p = input.all_planets.iter().find(|p| p.name == name).unwrap();
        Shadbala::compute_full(name, p.longitude, p.house, p.speed, input)
            .kala_bala
            .yuddha_bala
    }

    #[test]
    fn yuddha_pair_inside_one_degree_gains_and_loses_equally() {
        let input = input_with(vec![
            pos("Sun", 280.0, -20.0, 10),
            pos("Moon", 100.0, 5.0, 4),
            pos("Mars", 150.0, 10.0, 6),
            pos("Mercury", 270.0, -22.0, 9),
            pos("Jupiter", 50.0, 15.0, 2),
            pos("Venus", 150.5, 8.0, 6),
            pos("Saturn", 70.0, 22.0, 3),
        ]);
        let mars = yuddha_of(&input, "Mars");
        let venus = yuddha_of(&input, "Venus");
        // Mars is further north, so it wins.
        assert!(mars > 0.0, "winner gains: {mars}");
        assert!(venus < 0.0, "loser loses: {venus}");
        assert!((mars + venus).abs() < 1e-9);
        for other in ["Sun", "Moon", "Mercury", "Jupiter", "Saturn"] {
            assert_eq!(yuddha_of(&input, other), 0.0, "{other}");
        }
    }

    #[test]
    fn yuddha_winner_is_the_northern_planet() {
        let mk = |mars_dec, venus_dec| {
            input_with(vec![
                pos("Mars", 150.0, mars_dec, 6),
                pos("Venus", 150.5, venus_dec, 6),
            ])
        };
        assert!(yuddha_of(&mk(10.0, 8.0), "Mars") > 0.0);
        assert!(yuddha_of(&mk(8.0, 10.0), "Mars") < 0.0);
        assert!(yuddha_of(&mk(8.0, 10.0), "Venus") > 0.0);
    }

    #[test]
    fn yuddha_magnitude_is_partial_sum_difference_over_diameter_difference() {
        let input = input_with(vec![
            pos("Mars", 150.0, 10.0, 6),
            pos("Venus", 150.5, 8.0, 6),
        ]);
        let partial = |n: &str| {
            let p = input.all_planets.iter().find(|p| p.name == n).unwrap();
            yuddha_partial_sum(p, &input)
        };
        let expected = (partial("Mars") - partial("Venus")).abs() / (16.6 - 9.4);
        assert!(expected > 0.0);
        assert!((yuddha_of(&input, "Mars") - expected).abs() < 1e-9);
    }

    #[test]
    fn yuddha_needs_less_than_one_degree() {
        let input = input_with(vec![
            pos("Mars", 150.0, 10.0, 6),
            pos("Venus", 151.5, 8.0, 6),
        ]);
        assert_eq!(yuddha_of(&input, "Mars"), 0.0);
        assert_eq!(yuddha_of(&input, "Venus"), 0.0);
        // wrap-around at 0/360
        let wrap = input_with(vec![pos("Mars", 359.8, 10.0, 6), pos("Venus", 0.3, 8.0, 6)]);
        assert!(yuddha_of(&wrap, "Mars") > 0.0);
    }

    #[test]
    fn yuddha_never_involves_sun_or_moon() {
        for lum in ["Sun", "Moon"] {
            let input = input_with(vec![pos("Mars", 150.0, 10.0, 6), pos(lum, 150.5, 12.0, 6)]);
            assert_eq!(yuddha_of(&input, "Mars"), 0.0, "{lum}");
            assert_eq!(yuddha_of(&input, lum), 0.0, "{lum}");
        }
    }

    #[test]
    fn yuddha_three_planets_are_handled_pair_by_pair() {
        let input = input_with(vec![
            pos("Mars", 150.0, 12.0, 6),
            pos("Mercury", 150.4, 10.0, 6),
            pos("Venus", 150.8, 8.0, 6),
        ]);
        let m = yuddha_of(&input, "Mars");
        let me = yuddha_of(&input, "Mercury");
        let v = yuddha_of(&input, "Venus");
        // Mars beats both, Venus loses to both, Mercury beats Venus and loses to Mars.
        assert!(m > 0.0 && v < 0.0);
        assert!((m + me + v).abs() < 1e-9);
    }

    #[test]
    fn yuddha_sums_to_zero_over_all_seven_planets() {
        let input = input_with(vec![
            pos("Sun", 280.0, -20.0, 10),
            pos("Moon", 280.4, -18.0, 10),
            pos("Mars", 150.0, 10.0, 6),
            pos("Mercury", 150.3, 9.0, 6),
            pos("Jupiter", 50.0, 15.0, 2),
            pos("Venus", 150.7, 8.0, 6),
            pos("Saturn", 50.6, 14.0, 2),
        ]);
        let sum: f64 = input
            .all_planets
            .iter()
            .map(|p| yuddha_of(&input, p.name))
            .sum();
        assert!(sum.abs() < 1e-9, "sum {sum}");
        assert!(yuddha_of(&input, "Jupiter") != 0.0);
    }
}
