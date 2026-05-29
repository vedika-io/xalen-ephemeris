use serde::{Deserialize, Serialize};

pub mod catalog;

const PRECESSION_DEG_PER_YEAR: f64 = 50.2564 / 3600.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixedStar {
    pub name: &'static str,
    pub constellation: &'static str,
    pub longitude_j2000: f64, // degrees ecliptic
    pub latitude_j2000: f64,  // degrees ecliptic
    pub magnitude: f64,
    pub nature: &'static str,
    /// Proper motion in ecliptic longitude (milliarcseconds per year).
    pub pm_lon_mas_per_year: f64,
    /// Proper motion in ecliptic latitude (milliarcseconds per year).
    pub pm_lat_mas_per_year: f64,
}

impl FixedStar {
    pub fn longitude_at_epoch(&self, year: f64) -> f64 {
        let dt = year - 2000.0;
        let precession = PRECESSION_DEG_PER_YEAR * dt;
        let pm = self.pm_lon_mas_per_year * dt / 3_600_000.0; // mas to degrees
        (self.longitude_j2000 + precession + pm).rem_euclid(360.0)
    }

    pub fn latitude_at_epoch(&self, year: f64) -> f64 {
        let dt = year - 2000.0;
        let pm = self.pm_lat_mas_per_year * dt / 3_600_000.0;
        self.latitude_j2000 + pm
    }

    pub fn longitude_at_jd(&self, jd: f64) -> f64 {
        let year = 2000.0 + (jd - 2_451_545.0) / 365.25;
        self.longitude_at_epoch(year)
    }
}

pub fn find_conjunctions(planet_lon_deg: f64, orb_deg: f64) -> Vec<(&'static FixedStar, f64)> {
    find_conjunctions_at_epoch(planet_lon_deg, orb_deg, 2000.0)
}

pub fn find_conjunctions_at_epoch(
    planet_lon_deg: f64,
    orb_deg: f64,
    year: f64,
) -> Vec<(&'static FixedStar, f64)> {
    CATALOG
        .iter()
        .filter_map(|star| {
            let star_lon = star.longitude_at_epoch(year);
            let diff = (planet_lon_deg - star_lon).rem_euclid(360.0);
            let dist = if diff > 180.0 { 360.0 - diff } else { diff };
            if dist <= orb_deg {
                Some((star, dist))
            } else {
                None
            }
        })
        .collect()
}

pub fn find_by_name(name: &str) -> Option<&'static FixedStar> {
    CATALOG.iter().find(|s| s.name.eq_ignore_ascii_case(name))
}

/// Maps a nakshatra index (0 = Ashwini, 26 = Revati) to its primary
/// reference star (yogatara). Returns `None` for invalid indices.
///
/// Star identifications follow the traditional Surya Siddhanta / Burgess
/// mapping used in formal Jyotish certification curricula. Where the
/// traditional yogatara is a faint star not in our catalog, the brightest
/// astrologically-used star in the same nakshatra zone is returned.
pub fn nakshatra_yogatara(nakshatra_index: usize) -> Option<&'static FixedStar> {
    // The mapping table uses the most widely accepted yogatara star for each
    // nakshatra. Names must match entries in CATALOG (case-insensitive).
    let star_name: &str = match nakshatra_index {
        0 => "Sheratan",          // Ashwini — beta Arietis
        1 => "Bharani 41",        // Bharani — 41 Arietis
        2 => "Alcyone",           // Krittika — eta Tauri (Pleiades brightest)
        3 => "Aldebaran",         // Rohini — alpha Tauri
        4 => "Lambda Orionis",    // Mrigashira — lambda Orionis (Meissa)
        5 => "Betelgeuse",        // Ardra — alpha Orionis
        6 => "Pollux",            // Punarvasu — beta Geminorum
        7 => "Asellus Australis", // Pushya — delta Cancri
        8 => "Alphard",           // Ashlesha — alpha Hydrae
        9 => "Regulus",           // Magha — alpha Leonis
        10 => "Zosma",            // Purva Phalguni — delta Leonis
        11 => "Denebola",         // Uttara Phalguni — beta Leonis
        12 => "Porrima",          // Hasta — gamma Virginis
        13 => "Spica",            // Chitra — alpha Virginis
        14 => "Arcturus",         // Swati — alpha Bootis
        15 => "Zuben Eschamali",  // Vishakha — beta Librae
        16 => "Dschubba",         // Anuradha — delta Scorpii
        17 => "Antares",          // Jyeshtha — alpha Scorpii
        18 => "Shaula",           // Mula — lambda Scorpii
        19 => "Kaus Australis",   // Purva Ashadha — epsilon Sagittarii (proxy)
        20 => "Nunki",            // Uttara Ashadha — sigma Sagittarii
        21 => "Altair",           // Shravana — alpha Aquilae
        22 => "Sadalsuud",        // Dhanishta — beta Aquarii (proxy for Delphini)
        23 => "Sadalmelik",       // Shatabhisha — alpha Aquarii (proxy for lambda)
        24 => "Markab",           // Purva Bhadrapada — alpha Pegasi
        25 => "Scheat",           // Uttara Bhadrapada — beta Pegasi
        26 => "Revati",           // Revati — zeta Piscium
        _ => return None,
    };
    find_by_name(star_name)
}

// ---------------------------------------------------------------------------
// CATALOG — 110 astrologically significant fixed stars
//
// Every entry carries J2000.0 ecliptic longitude / latitude, visual magnitude,
// Ptolemaic planetary nature, and proper-motion components in mas/yr.
//
// High proper motion values (> ~100 mas/yr) are set for Sirius, Arcturus,
// Procyon, Pollux, Aldebaran, and other well-measured stars. Stars where
// proper motion is astrologically negligible (< 50 mas/yr) use 0.0.
// ---------------------------------------------------------------------------
pub static CATALOG: &[FixedStar] = &[
    // ===== ORIGINAL 25 BRIGHT / ROYAL STARS ==============================
    FixedStar {
        name: "Aldebaran",
        constellation: "Taurus",
        longitude_j2000: 69.95,
        latitude_j2000: -5.47,
        magnitude: 0.87,
        nature: "Mars",
        pm_lon_mas_per_year: 62.8,
        pm_lat_mas_per_year: -189.4,
    },
    FixedStar {
        name: "Regulus",
        constellation: "Leo",
        longitude_j2000: 149.83,
        latitude_j2000: 0.47,
        magnitude: 1.36,
        nature: "Mars-Jupiter",
        pm_lon_mas_per_year: -248.7,
        pm_lat_mas_per_year: 5.6,
    },
    FixedStar {
        name: "Spica",
        constellation: "Virgo",
        longitude_j2000: 203.83,
        latitude_j2000: -2.05,
        magnitude: 0.98,
        nature: "Venus-Mars",
        pm_lon_mas_per_year: -42.4,
        pm_lat_mas_per_year: -31.7,
    },
    FixedStar {
        name: "Antares",
        constellation: "Scorpio",
        longitude_j2000: 249.78,
        latitude_j2000: -4.57,
        magnitude: 1.06,
        nature: "Mars-Jupiter",
        pm_lon_mas_per_year: -10.2,
        pm_lat_mas_per_year: -23.2,
    },
    FixedStar {
        name: "Fomalhaut",
        constellation: "Pisces Australis",
        longitude_j2000: 334.17,
        latitude_j2000: -21.08,
        magnitude: 1.17,
        nature: "Venus-Mercury",
        pm_lon_mas_per_year: 329.2,
        pm_lat_mas_per_year: -164.2,
    },
    FixedStar {
        name: "Sirius",
        constellation: "Canis Major",
        longitude_j2000: 104.07,
        latitude_j2000: -39.60,
        magnitude: -1.44,
        nature: "Jupiter-Mars",
        pm_lon_mas_per_year: -546.0,
        pm_lat_mas_per_year: -1223.1,
    },
    FixedStar {
        name: "Canopus",
        constellation: "Carina",
        longitude_j2000: 105.28,
        latitude_j2000: -75.75,
        magnitude: -0.62,
        nature: "Saturn-Jupiter",
        pm_lon_mas_per_year: 19.9,
        pm_lat_mas_per_year: 23.2,
    },
    FixedStar {
        name: "Arcturus",
        constellation: "Bootes",
        longitude_j2000: 204.07,
        latitude_j2000: 30.77,
        magnitude: -0.05,
        nature: "Mars-Jupiter",
        pm_lon_mas_per_year: -1093.4,
        pm_lat_mas_per_year: -1999.4,
    },
    FixedStar {
        name: "Vega",
        constellation: "Lyra",
        longitude_j2000: 285.45,
        latitude_j2000: 61.73,
        magnitude: 0.03,
        nature: "Venus-Mercury",
        pm_lon_mas_per_year: 201.0,
        pm_lat_mas_per_year: 287.5,
    },
    FixedStar {
        name: "Capella",
        constellation: "Auriga",
        longitude_j2000: 81.73,
        latitude_j2000: 22.87,
        magnitude: 0.08,
        nature: "Mars-Mercury",
        pm_lon_mas_per_year: 75.5,
        pm_lat_mas_per_year: -427.1,
    },
    FixedStar {
        name: "Rigel",
        constellation: "Orion",
        longitude_j2000: 78.63,
        latitude_j2000: -31.10,
        magnitude: 0.18,
        nature: "Jupiter-Mars",
        pm_lon_mas_per_year: 1.3,
        pm_lat_mas_per_year: -0.6,
    },
    FixedStar {
        name: "Procyon",
        constellation: "Canis Minor",
        longitude_j2000: 115.62,
        latitude_j2000: -16.03,
        magnitude: 0.40,
        nature: "Mercury-Mars",
        pm_lon_mas_per_year: -714.6,
        pm_lat_mas_per_year: -1036.8,
    },
    FixedStar {
        name: "Betelgeuse",
        constellation: "Orion",
        longitude_j2000: 88.79,
        latitude_j2000: -16.03,
        magnitude: 0.45,
        nature: "Mars-Mercury",
        pm_lon_mas_per_year: 27.3,
        pm_lat_mas_per_year: 10.9,
    },
    FixedStar {
        name: "Altair",
        constellation: "Aquila",
        longitude_j2000: 301.82,
        latitude_j2000: 29.31,
        magnitude: 0.76,
        nature: "Mars-Jupiter",
        pm_lon_mas_per_year: 536.2,
        pm_lat_mas_per_year: 385.3,
    },
    FixedStar {
        name: "Polaris",
        constellation: "Ursa Minor",
        longitude_j2000: 28.60,
        latitude_j2000: 66.10,
        magnitude: 1.97,
        nature: "Saturn-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Algol",
        constellation: "Perseus",
        longitude_j2000: 56.17,
        latitude_j2000: 22.42,
        magnitude: 2.09,
        nature: "Saturn-Jupiter",
        pm_lon_mas_per_year: 2.4,
        pm_lat_mas_per_year: -1.5,
    },
    FixedStar {
        name: "Deneb",
        constellation: "Cygnus",
        longitude_j2000: 305.17,
        latitude_j2000: 59.88,
        magnitude: 1.25,
        nature: "Venus-Mercury",
        pm_lon_mas_per_year: 1.6,
        pm_lat_mas_per_year: 1.5,
    },
    FixedStar {
        name: "Castor",
        constellation: "Gemini",
        longitude_j2000: 110.15,
        latitude_j2000: 10.09,
        magnitude: 1.58,
        nature: "Mercury",
        pm_lon_mas_per_year: -191.5,
        pm_lat_mas_per_year: -145.2,
    },
    FixedStar {
        name: "Pollux",
        constellation: "Gemini",
        longitude_j2000: 113.22,
        latitude_j2000: 6.68,
        magnitude: 1.16,
        nature: "Mars",
        pm_lon_mas_per_year: -625.7,
        pm_lat_mas_per_year: -45.8,
    },
    FixedStar {
        name: "Pleiades",
        constellation: "Taurus",
        longitude_j2000: 60.00,
        latitude_j2000: 4.07,
        magnitude: 1.60,
        nature: "Moon-Mars",
        pm_lon_mas_per_year: 19.3,
        pm_lat_mas_per_year: -43.7,
    },
    FixedStar {
        name: "Vindemiatrix",
        constellation: "Virgo",
        longitude_j2000: 189.93,
        latitude_j2000: 16.20,
        magnitude: 2.85,
        nature: "Saturn-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Zuben Elgenubi",
        constellation: "Libra",
        longitude_j2000: 225.10,
        latitude_j2000: 0.33,
        magnitude: 2.75,
        nature: "Saturn-Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Zuben Eschamali",
        constellation: "Libra",
        longitude_j2000: 229.32,
        latitude_j2000: 8.87,
        magnitude: 2.61,
        nature: "Jupiter-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Achernar",
        constellation: "Eridanus",
        longitude_j2000: 345.33,
        latitude_j2000: -59.38,
        magnitude: 0.45,
        nature: "Jupiter",
        pm_lon_mas_per_year: 88.0,
        pm_lat_mas_per_year: -40.1,
    },
    FixedStar {
        name: "Hamal",
        constellation: "Aries",
        longitude_j2000: 37.73,
        latitude_j2000: 9.93,
        magnitude: 2.01,
        nature: "Mars-Saturn",
        pm_lon_mas_per_year: 190.7,
        pm_lat_mas_per_year: -148.1,
    },
    // ===== ORIGINAL EXPANSION (25 stars, formerly bringing total to 55) ===
    FixedStar {
        name: "Alpheratz",
        constellation: "Andromeda",
        longitude_j2000: 14.18,
        latitude_j2000: 25.68,
        magnitude: 2.07,
        nature: "Venus-Jupiter",
        pm_lon_mas_per_year: 135.7,
        pm_lat_mas_per_year: -162.9,
    },
    FixedStar {
        name: "Mirach",
        constellation: "Andromeda",
        longitude_j2000: 30.42,
        latitude_j2000: 25.55,
        magnitude: 2.07,
        nature: "Venus",
        pm_lon_mas_per_year: 175.6,
        pm_lat_mas_per_year: -112.2,
    },
    FixedStar {
        name: "Almach",
        constellation: "Andromeda",
        longitude_j2000: 44.17,
        latitude_j2000: 21.50,
        magnitude: 2.10,
        nature: "Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Sheratan",
        constellation: "Aries",
        longitude_j2000: 33.93,
        latitude_j2000: 8.49,
        magnitude: 2.64,
        nature: "Mars-Saturn",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Menkar",
        constellation: "Cetus",
        longitude_j2000: 44.33,
        latitude_j2000: -12.59,
        magnitude: 2.54,
        nature: "Saturn",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Mirfak",
        constellation: "Perseus",
        longitude_j2000: 62.33,
        latitude_j2000: 30.21,
        magnitude: 1.79,
        nature: "Jupiter-Saturn",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Bellatrix",
        constellation: "Orion",
        longitude_j2000: 80.95,
        latitude_j2000: -16.77,
        magnitude: 1.64,
        nature: "Mars-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Alnilam",
        constellation: "Orion",
        longitude_j2000: 83.42,
        latitude_j2000: -24.22,
        magnitude: 1.69,
        nature: "Jupiter-Saturn",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Elnath",
        constellation: "Taurus",
        longitude_j2000: 82.35,
        latitude_j2000: 5.23,
        magnitude: 1.65,
        nature: "Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Menkalinan",
        constellation: "Auriga",
        longitude_j2000: 89.90,
        latitude_j2000: 17.00,
        magnitude: 1.90,
        nature: "Mars-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Alhena",
        constellation: "Gemini",
        longitude_j2000: 99.07,
        latitude_j2000: -6.38,
        magnitude: 1.93,
        nature: "Mercury-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Wezen",
        constellation: "Canis Major",
        longitude_j2000: 113.42,
        latitude_j2000: -28.95,
        magnitude: 1.84,
        nature: "Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Adhara",
        constellation: "Canis Major",
        longitude_j2000: 110.72,
        latitude_j2000: -42.47,
        magnitude: 1.50,
        nature: "Venus-Jupiter",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Praesepe",
        constellation: "Cancer",
        longitude_j2000: 127.17,
        latitude_j2000: 1.50,
        magnitude: 3.70,
        nature: "Mars-Moon",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Alphard",
        constellation: "Hydra",
        longitude_j2000: 147.28,
        latitude_j2000: -22.38,
        magnitude: 1.99,
        nature: "Saturn-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Zosma",
        constellation: "Leo",
        longitude_j2000: 161.33,
        latitude_j2000: 14.43,
        magnitude: 2.56,
        nature: "Saturn-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Denebola",
        constellation: "Leo",
        longitude_j2000: 171.55,
        latitude_j2000: 12.17,
        magnitude: 2.14,
        nature: "Saturn-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Algorab",
        constellation: "Corvus",
        longitude_j2000: 193.60,
        latitude_j2000: -12.62,
        magnitude: 2.94,
        nature: "Mars-Saturn",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Cor Caroli",
        constellation: "Canes Venatici",
        longitude_j2000: 174.73,
        latitude_j2000: 39.53,
        magnitude: 2.89,
        nature: "Sun-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Acrux",
        constellation: "Crux",
        longitude_j2000: 222.00,
        latitude_j2000: -52.68,
        magnitude: 0.77,
        nature: "Jupiter",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Alphecca",
        constellation: "Corona Borealis",
        longitude_j2000: 222.10,
        latitude_j2000: 44.32,
        magnitude: 2.22,
        nature: "Venus-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Unukalhai",
        constellation: "Serpens",
        longitude_j2000: 232.05,
        latitude_j2000: 25.62,
        magnitude: 2.63,
        nature: "Saturn-Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Dschubba",
        constellation: "Scorpius",
        longitude_j2000: 242.58,
        latitude_j2000: -1.98,
        magnitude: 2.29,
        nature: "Mars-Saturn",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Rasalhague",
        constellation: "Ophiuchus",
        longitude_j2000: 262.40,
        latitude_j2000: 35.83,
        magnitude: 2.08,
        nature: "Saturn-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Shaula",
        constellation: "Scorpius",
        longitude_j2000: 264.58,
        latitude_j2000: -13.85,
        magnitude: 1.62,
        nature: "Mercury-Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Kaus Australis",
        constellation: "Sagittarius",
        longitude_j2000: 275.00,
        latitude_j2000: -11.07,
        magnitude: 1.79,
        nature: "Jupiter-Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Nunki",
        constellation: "Sagittarius",
        longitude_j2000: 282.30,
        latitude_j2000: -3.45,
        magnitude: 2.05,
        nature: "Jupiter-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Deneb Algedi",
        constellation: "Capricornus",
        longitude_j2000: 323.45,
        latitude_j2000: 2.58,
        magnitude: 2.85,
        nature: "Saturn-Jupiter",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Markab",
        constellation: "Pegasus",
        longitude_j2000: 353.47,
        latitude_j2000: 19.40,
        magnitude: 2.49,
        nature: "Mars-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Scheat",
        constellation: "Pegasus",
        longitude_j2000: 359.32,
        latitude_j2000: 31.08,
        magnitude: 2.44,
        nature: "Mars-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // ===== NEW — NAKSHATRA YOGATARAS & VEDIC/JYOTISH STARS ===============
    // Stars needed as yogataras for the 27 nakshatras that were missing above.

    // Bharani — 41 Arietis
    FixedStar {
        name: "Bharani 41",
        constellation: "Aries",
        longitude_j2000: 41.08,
        latitude_j2000: -10.32,
        magnitude: 3.63,
        nature: "Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Krittika — Alcyone (eta Tauri, brightest Pleiad)
    FixedStar {
        name: "Alcyone",
        constellation: "Taurus",
        longitude_j2000: 60.06,
        latitude_j2000: 4.07,
        magnitude: 2.85,
        nature: "Moon-Mars",
        pm_lon_mas_per_year: 19.3,
        pm_lat_mas_per_year: -43.7,
    },
    // Mrigashira — Lambda Orionis (Meissa)
    FixedStar {
        name: "Lambda Orionis",
        constellation: "Orion",
        longitude_j2000: 83.78,
        latitude_j2000: -13.38,
        magnitude: 3.39,
        nature: "Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Pushya — Asellus Australis (delta Cancri)
    FixedStar {
        name: "Asellus Australis",
        constellation: "Cancer",
        longitude_j2000: 128.70,
        latitude_j2000: -0.07,
        magnitude: 3.94,
        nature: "Saturn-Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Pushya area — Asellus Borealis (gamma Cancri)
    FixedStar {
        name: "Asellus Borealis",
        constellation: "Cancer",
        longitude_j2000: 127.52,
        latitude_j2000: 3.12,
        magnitude: 4.66,
        nature: "Saturn-Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Hasta — Porrima (gamma Virginis)
    FixedStar {
        name: "Porrima",
        constellation: "Virgo",
        longitude_j2000: 190.23,
        latitude_j2000: 2.78,
        magnitude: 2.74,
        nature: "Mercury-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Shatabhisha — Sadalmelik (alpha Aquarii, proxy)
    FixedStar {
        name: "Sadalmelik",
        constellation: "Aquarius",
        longitude_j2000: 333.38,
        latitude_j2000: 8.80,
        magnitude: 2.95,
        nature: "Saturn-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Dhanishta — Sadalsuud (beta Aquarii, proxy for Sravishtha)
    FixedStar {
        name: "Sadalsuud",
        constellation: "Aquarius",
        longitude_j2000: 323.40,
        latitude_j2000: 8.52,
        magnitude: 2.90,
        nature: "Saturn-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Revati — zeta Piscium
    FixedStar {
        name: "Revati",
        constellation: "Pisces",
        longitude_j2000: 19.87,
        latitude_j2000: -0.22,
        magnitude: 5.21,
        nature: "Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // ===== NEW — HYADES CLUSTER & TAURUS STARS ===========================
    // Ain (epsilon Tauri) — Hyades member, V-shape eye
    FixedStar {
        name: "Ain",
        constellation: "Taurus",
        longitude_j2000: 68.09,
        latitude_j2000: -3.02,
        magnitude: 3.54,
        nature: "Mars",
        pm_lon_mas_per_year: 107.0,
        pm_lat_mas_per_year: -37.8,
    },
    // Prima Hyadum (gamma Tauri) — tip of the V
    FixedStar {
        name: "Prima Hyadum",
        constellation: "Taurus",
        longitude_j2000: 69.00,
        latitude_j2000: -5.66,
        magnitude: 3.65,
        nature: "Saturn-Mercury",
        pm_lon_mas_per_year: 115.3,
        pm_lat_mas_per_year: -23.9,
    },
    // Hyadum II (delta Tauri)
    FixedStar {
        name: "Hyadum II",
        constellation: "Taurus",
        longitude_j2000: 68.80,
        latitude_j2000: -3.63,
        magnitude: 3.76,
        nature: "Saturn-Mercury",
        pm_lon_mas_per_year: 107.8,
        pm_lat_mas_per_year: -29.1,
    },
    // ===== NEW — SOUTHERN CROSS (CRUX) ===================================
    // Acrux already included above; here are the remaining Crux stars.
    FixedStar {
        name: "Mimosa",
        constellation: "Crux",
        longitude_j2000: 222.40,
        latitude_j2000: -47.53,
        magnitude: 1.25,
        nature: "Jupiter-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Gacrux",
        constellation: "Crux",
        longitude_j2000: 222.03,
        latitude_j2000: -45.52,
        magnitude: 1.59,
        nature: "Jupiter",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    FixedStar {
        name: "Imai",
        constellation: "Crux",
        longitude_j2000: 218.08,
        latitude_j2000: -49.22,
        magnitude: 2.79,
        nature: "Jupiter",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // ===== NEW — NAVIGATOR STARS MISSING FROM ORIGINAL CATALOG ===========
    // Peacock (alpha Pavonis)
    FixedStar {
        name: "Peacock",
        constellation: "Pavo",
        longitude_j2000: 293.35,
        latitude_j2000: -36.34,
        magnitude: 1.94,
        nature: "Mercury-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Diphda / Deneb Kaitos (beta Ceti) — important navigator star
    FixedStar {
        name: "Diphda",
        constellation: "Cetus",
        longitude_j2000: 2.33,
        latitude_j2000: -20.84,
        magnitude: 2.04,
        nature: "Saturn",
        pm_lon_mas_per_year: 232.8,
        pm_lat_mas_per_year: 32.7,
    },
    // Ankaa (alpha Phoenicis)
    FixedStar {
        name: "Ankaa",
        constellation: "Phoenix",
        longitude_j2000: 355.48,
        latitude_j2000: -41.98,
        magnitude: 2.40,
        nature: "Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Alnair (alpha Gruis)
    FixedStar {
        name: "Alnair",
        constellation: "Grus",
        longitude_j2000: 327.95,
        latitude_j2000: -35.18,
        magnitude: 1.73,
        nature: "Jupiter-Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Kochab (beta Ursae Minoris) — historical pole navigator
    FixedStar {
        name: "Kochab",
        constellation: "Ursa Minor",
        longitude_j2000: 222.87,
        latitude_j2000: 72.83,
        magnitude: 2.07,
        nature: "Saturn-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Dubhe (alpha Ursae Majoris) — pointer to Polaris
    FixedStar {
        name: "Dubhe",
        constellation: "Ursa Major",
        longitude_j2000: 166.63,
        latitude_j2000: 49.33,
        magnitude: 1.81,
        nature: "Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Alioth (epsilon Ursae Majoris) — navigator
    FixedStar {
        name: "Alioth",
        constellation: "Ursa Major",
        longitude_j2000: 177.33,
        latitude_j2000: 53.73,
        magnitude: 1.76,
        nature: "Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Alkaid / Benetnash (eta Ursae Majoris)
    FixedStar {
        name: "Alkaid",
        constellation: "Ursa Major",
        longitude_j2000: 190.68,
        latitude_j2000: 54.38,
        magnitude: 1.85,
        nature: "Mars-Moon",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Miaplacidus (beta Carinae) — southern navigator
    FixedStar {
        name: "Miaplacidus",
        constellation: "Carina",
        longitude_j2000: 131.15,
        latitude_j2000: -69.58,
        magnitude: 1.67,
        nature: "Saturn-Jupiter",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Avior (epsilon Carinae)
    FixedStar {
        name: "Avior",
        constellation: "Carina",
        longitude_j2000: 124.70,
        latitude_j2000: -59.05,
        magnitude: 1.86,
        nature: "Jupiter-Saturn",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Suhail (lambda Velorum)
    FixedStar {
        name: "Suhail",
        constellation: "Vela",
        longitude_j2000: 132.63,
        latitude_j2000: -43.18,
        magnitude: 2.23,
        nature: "Saturn-Jupiter",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Atria (alpha Trianguli Australis)
    FixedStar {
        name: "Atria",
        constellation: "Triangulum Australe",
        longitude_j2000: 252.33,
        latitude_j2000: -28.88,
        magnitude: 1.91,
        nature: "Jupiter",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // ===== NEW — ADDITIONAL ASTROLOGICALLY SIGNIFICANT STARS ==============

    // Algenib (gamma Pegasi) — Ptolemaic
    FixedStar {
        name: "Algenib",
        constellation: "Pegasus",
        longitude_j2000: 9.08,
        latitude_j2000: 12.62,
        magnitude: 2.83,
        nature: "Mars-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Baten Kaitos (zeta Ceti) — belly of the whale
    FixedStar {
        name: "Baten Kaitos",
        constellation: "Cetus",
        longitude_j2000: 21.82,
        latitude_j2000: -20.59,
        magnitude: 3.74,
        nature: "Saturn",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Kaffaljidhma (gamma Ceti)
    FixedStar {
        name: "Kaffaljidhma",
        constellation: "Cetus",
        longitude_j2000: 32.35,
        latitude_j2000: -12.47,
        magnitude: 3.47,
        nature: "Saturn",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Alcyone already above as Krittika yogatara

    // Phact (alpha Columbae)
    FixedStar {
        name: "Phact",
        constellation: "Columba",
        longitude_j2000: 91.83,
        latitude_j2000: -41.88,
        magnitude: 2.65,
        nature: "Mercury-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Tejat (mu Geminorum)
    FixedStar {
        name: "Tejat",
        constellation: "Gemini",
        longitude_j2000: 96.58,
        latitude_j2000: -0.87,
        magnitude: 2.87,
        nature: "Mercury-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Acubens (alpha Cancri)
    FixedStar {
        name: "Acubens",
        constellation: "Cancer",
        longitude_j2000: 133.38,
        latitude_j2000: -5.08,
        magnitude: 4.26,
        nature: "Saturn-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Algieba (gamma Leonis)
    FixedStar {
        name: "Algieba",
        constellation: "Leo",
        longitude_j2000: 149.47,
        latitude_j2000: 4.63,
        magnitude: 2.01,
        nature: "Saturn-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Al Jabhah (eta Leonis)
    FixedStar {
        name: "Al Jabhah",
        constellation: "Leo",
        longitude_j2000: 147.78,
        latitude_j2000: 4.07,
        magnitude: 3.52,
        nature: "Saturn-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Zavijava (beta Virginis)
    FixedStar {
        name: "Zavijava",
        constellation: "Virgo",
        longitude_j2000: 177.17,
        latitude_j2000: 0.67,
        magnitude: 3.59,
        nature: "Mercury-Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Mizar (zeta Ursae Majoris) — famous double star
    FixedStar {
        name: "Mizar",
        constellation: "Ursa Major",
        longitude_j2000: 185.58,
        latitude_j2000: 54.88,
        magnitude: 2.23,
        nature: "Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Alcor (80 Ursae Majoris) — eyesight test star, near Mizar
    FixedStar {
        name: "Alcor",
        constellation: "Ursa Major",
        longitude_j2000: 186.00,
        latitude_j2000: 55.00,
        magnitude: 3.99,
        nature: "Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Kraz (beta Corvi)
    FixedStar {
        name: "Kraz",
        constellation: "Corvus",
        longitude_j2000: 197.35,
        latitude_j2000: -14.55,
        magnitude: 2.65,
        nature: "Mars-Saturn",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Menkent (theta Centauri)
    FixedStar {
        name: "Menkent",
        constellation: "Centaurus",
        longitude_j2000: 222.87,
        latitude_j2000: -22.65,
        magnitude: 2.06,
        nature: "Jupiter-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Rigil Kentaurus (alpha Centauri A) — closest star system
    FixedStar {
        name: "Rigil Kentaurus",
        constellation: "Centaurus",
        longitude_j2000: 239.58,
        latitude_j2000: -42.58,
        magnitude: -0.01,
        nature: "Jupiter-Venus",
        pm_lon_mas_per_year: -3679.0,
        pm_lat_mas_per_year: 481.8,
    },
    // Hadar (beta Centauri)
    FixedStar {
        name: "Hadar",
        constellation: "Centaurus",
        longitude_j2000: 233.68,
        latitude_j2000: -44.32,
        magnitude: 0.61,
        nature: "Jupiter-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Yed Prior (delta Ophiuchi)
    FixedStar {
        name: "Yed Prior",
        constellation: "Ophiuchus",
        longitude_j2000: 242.35,
        latitude_j2000: 17.13,
        magnitude: 2.73,
        nature: "Saturn-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Sabik (eta Ophiuchi)
    FixedStar {
        name: "Sabik",
        constellation: "Ophiuchus",
        longitude_j2000: 257.63,
        latitude_j2000: 7.28,
        magnitude: 2.43,
        nature: "Saturn-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Kaus Borealis (lambda Sagittarii) — top of the bow
    FixedStar {
        name: "Kaus Borealis",
        constellation: "Sagittarius",
        longitude_j2000: 276.05,
        latitude_j2000: 2.10,
        magnitude: 2.82,
        nature: "Jupiter-Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Kaus Media (delta Sagittarii) — middle of the bow
    FixedStar {
        name: "Kaus Media",
        constellation: "Sagittarius",
        longitude_j2000: 274.53,
        latitude_j2000: -6.45,
        magnitude: 2.72,
        nature: "Jupiter-Mars",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Ascella (zeta Sagittarii)
    FixedStar {
        name: "Ascella",
        constellation: "Sagittarius",
        longitude_j2000: 283.43,
        latitude_j2000: -7.20,
        magnitude: 2.60,
        nature: "Jupiter-Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Dabih (beta Capricorni)
    FixedStar {
        name: "Dabih",
        constellation: "Capricornus",
        longitude_j2000: 304.07,
        latitude_j2000: 4.58,
        magnitude: 3.05,
        nature: "Saturn-Venus",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Nashira (gamma Capricorni)
    FixedStar {
        name: "Nashira",
        constellation: "Capricornus",
        longitude_j2000: 322.17,
        latitude_j2000: -2.53,
        magnitude: 3.68,
        nature: "Saturn-Jupiter",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Skat (delta Aquarii) — important Arabic-tradition star
    FixedStar {
        name: "Skat",
        constellation: "Aquarius",
        longitude_j2000: 338.95,
        latitude_j2000: -8.22,
        magnitude: 3.27,
        nature: "Saturn-Jupiter",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Alrescha (alpha Piscium) — the knot of Pisces
    FixedStar {
        name: "Alrescha",
        constellation: "Pisces",
        longitude_j2000: 29.35,
        latitude_j2000: -9.15,
        magnitude: 3.82,
        nature: "Mercury-Saturn",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Acamar (theta Eridani) — navigator star, river's end
    FixedStar {
        name: "Acamar",
        constellation: "Eridanus",
        longitude_j2000: 23.30,
        latitude_j2000: -53.40,
        magnitude: 2.88,
        nature: "Jupiter",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
    // Wazn (beta Columbae)
    FixedStar {
        name: "Wazn",
        constellation: "Columba",
        longitude_j2000: 97.07,
        latitude_j2000: -40.58,
        magnitude: 3.12,
        nature: "Mercury",
        pm_lon_mas_per_year: 0.0,
        pm_lat_mas_per_year: 0.0,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_108_stars() {
        assert_eq!(
            CATALOG.len(),
            108,
            "Expected 108 stars in catalog, found {}",
            CATALOG.len()
        );
    }

    #[test]
    fn no_duplicate_names() {
        let mut names: Vec<&str> = CATALOG.iter().map(|s| s.name).collect();
        names.sort();
        for w in names.windows(2) {
            assert_ne!(w[0], w[1], "Duplicate star name: {}", w[0]);
        }
    }

    #[test]
    fn spica_at_j2000() {
        let spica = find_by_name("Spica").unwrap();
        assert!((spica.longitude_j2000 - 203.83).abs() < 0.01);
    }

    #[test]
    fn precession_moves_stars() {
        let spica = find_by_name("Spica").unwrap();
        let lon_2000 = spica.longitude_at_epoch(2000.0);
        let lon_2100 = spica.longitude_at_epoch(2100.0);
        let diff = lon_2100 - lon_2000;
        // Spica has pm_lon = -42.4 mas/yr, so over 100 yr that adds
        // -42.4 * 100 / 3_600_000 = -0.00118 deg. Combined with precession
        // of ~1.3960 deg the total is ~1.3948 deg.
        assert!(
            (diff - 1.395).abs() < 0.01,
            "100yr precession+pm should be ~1.395 deg, got {diff}"
        );
    }

    #[test]
    fn proper_motion_sirius_significant_over_1000yr() {
        let sirius = find_by_name("Sirius").unwrap();
        assert!(
            sirius.pm_lon_mas_per_year.abs() > 500.0,
            "Sirius should have large proper motion"
        );

        let lon_2000 = sirius.longitude_at_epoch(2000.0);
        let lon_3000 = sirius.longitude_at_epoch(3000.0);
        let diff = (lon_3000 - lon_2000).rem_euclid(360.0);
        // Over 1000 years: precession adds ~13.96 deg, pm_lon adds
        // -546 * 1000 / 3_600_000 = -0.1517 deg. Net ~13.81 deg.
        assert!(
            diff > 13.0 && diff < 15.0,
            "Sirius 1000yr lon shift should be ~13.8 deg, got {diff}"
        );
    }

    #[test]
    fn proper_motion_arcturus_significant_over_1000yr() {
        let arc = find_by_name("Arcturus").unwrap();
        assert!(
            arc.pm_lon_mas_per_year.abs() > 1000.0,
            "Arcturus should have very large proper motion"
        );

        let lon_2000 = arc.longitude_at_epoch(2000.0);
        let lon_3000 = arc.longitude_at_epoch(3000.0);
        let diff = (lon_3000 - lon_2000).rem_euclid(360.0);
        // Precession +13.96, pm_lon -1093.4*1000/3.6e6 = -0.304 deg. Net ~13.66
        assert!(
            diff > 13.0 && diff < 15.0,
            "Arcturus 1000yr lon shift should be ~13.66 deg, got {diff}"
        );
    }

    #[test]
    fn proper_motion_zero_stars_match_pure_precession() {
        // Polaris has pm = 0.0, so should match pure precession exactly
        let pol = find_by_name("Polaris").unwrap();
        let lon_2100 = pol.longitude_at_epoch(2100.0);
        let expected = (pol.longitude_j2000 + PRECESSION_DEG_PER_YEAR * 100.0).rem_euclid(360.0);
        assert!(
            (lon_2100 - expected).abs() < 1e-10,
            "Zero-pm star should match pure precession"
        );
    }

    #[test]
    fn latitude_proper_motion_works() {
        let sirius = find_by_name("Sirius").unwrap();
        let lat_2000 = sirius.latitude_at_epoch(2000.0);
        let lat_3000 = sirius.latitude_at_epoch(3000.0);
        let diff = lat_3000 - lat_2000;
        // -1223.1 mas/yr * 1000 yr / 3_600_000 = -0.3398 deg
        assert!(
            (diff - (-0.3398)).abs() < 0.01,
            "Sirius lat shift over 1000yr should be ~-0.34 deg, got {diff}"
        );
    }

    #[test]
    fn find_conjunction_with_aldebaran() {
        let matches = find_conjunctions(70.0, 2.0);
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|(s, _)| s.name == "Aldebaran"));
    }

    #[test]
    fn no_conjunction_when_far() {
        let matches = find_conjunctions(180.0, 1.0);
        // Should find nothing near 180 deg with 1 deg orb
        assert!(matches.is_empty() || matches.iter().all(|(_, d)| *d <= 1.0));
    }

    #[test]
    fn find_by_name_case_insensitive() {
        assert!(find_by_name("sirius").is_some());
        assert!(find_by_name("SIRIUS").is_some());
        assert!(find_by_name("nonexistent").is_none());
    }

    #[test]
    fn royal_stars_present() {
        assert!(find_by_name("Aldebaran").is_some());
        assert!(find_by_name("Regulus").is_some());
        assert!(find_by_name("Antares").is_some());
        assert!(find_by_name("Fomalhaut").is_some());
    }

    #[test]
    fn southern_cross_complete() {
        assert!(find_by_name("Acrux").is_some());
        assert!(find_by_name("Mimosa").is_some());
        assert!(find_by_name("Gacrux").is_some());
        assert!(find_by_name("Imai").is_some());
    }

    #[test]
    fn hyades_stars_present() {
        assert!(find_by_name("Aldebaran").is_some()); // alpha Tauri
        assert!(find_by_name("Ain").is_some()); // epsilon Tauri
        assert!(find_by_name("Prima Hyadum").is_some()); // gamma Tauri
        assert!(find_by_name("Hyadum II").is_some()); // delta Tauri
    }

    #[test]
    fn navigator_stars_present() {
        for name in &[
            "Polaris",
            "Canopus",
            "Vega",
            "Capella",
            "Sirius",
            "Procyon",
            "Peacock",
            "Diphda",
            "Ankaa",
            "Alnair",
            "Kochab",
            "Dubhe",
            "Miaplacidus",
            "Avior",
            "Suhail",
            "Atria",
            "Achernar",
        ] {
            assert!(
                find_by_name(name).is_some(),
                "Navigator star {name} missing"
            );
        }
    }

    // ----- Nakshatra yogatara tests -----

    #[test]
    fn nakshatra_yogatara_all_27_mapped() {
        for i in 0..27 {
            let star = nakshatra_yogatara(i);
            assert!(
                star.is_some(),
                "Nakshatra index {i} has no yogatara mapping"
            );
        }
    }

    #[test]
    fn nakshatra_yogatara_invalid_index_returns_none() {
        assert!(nakshatra_yogatara(27).is_none());
        assert!(nakshatra_yogatara(100).is_none());
    }

    #[test]
    fn nakshatra_yogatara_spot_checks() {
        // Rohini = Aldebaran
        assert_eq!(nakshatra_yogatara(3).unwrap().name, "Aldebaran");
        // Magha = Regulus
        assert_eq!(nakshatra_yogatara(9).unwrap().name, "Regulus");
        // Chitra = Spica
        assert_eq!(nakshatra_yogatara(13).unwrap().name, "Spica");
        // Jyeshtha = Antares
        assert_eq!(nakshatra_yogatara(17).unwrap().name, "Antares");
        // Swati = Arcturus
        assert_eq!(nakshatra_yogatara(14).unwrap().name, "Arcturus");
        // Shravana = Altair
        assert_eq!(nakshatra_yogatara(21).unwrap().name, "Altair");
        // Punarvasu = Pollux
        assert_eq!(nakshatra_yogatara(6).unwrap().name, "Pollux");
        // Ardra = Betelgeuse
        assert_eq!(nakshatra_yogatara(5).unwrap().name, "Betelgeuse");
    }

    #[test]
    fn nakshatra_yogatara_boundary_nakshatras() {
        // First: Ashwini = Sheratan
        assert_eq!(nakshatra_yogatara(0).unwrap().name, "Sheratan");
        // Last: Revati = Revati (zeta Piscium)
        assert_eq!(nakshatra_yogatara(26).unwrap().name, "Revati");
    }
}
