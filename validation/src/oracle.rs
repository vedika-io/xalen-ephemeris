//! Oracle-file parsing: the JSON Lines records emitted by
//! `validation/oracle_pyswisseph.py`.

use serde::Deserialize;
use std::collections::BTreeMap;

/// One chart record from the pyswisseph oracle.
#[derive(Debug, Deserialize)]
pub struct OracleRecord {
    pub jd: f64,
    pub lat: f64,
    pub lon: f64,
    /// body name -> [ecliptic longitude deg, ecliptic latitude deg]
    pub bodies: BTreeMap<String, [f64; 2]>,
    pub houses: OracleHouses,
    /// ayanamsa name -> degrees (with-nutation apparent value)
    pub ayanamsa: BTreeMap<String, f64>,
    /// Present only on the first line.
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<OracleMeta>,
}

#[derive(Debug, Deserialize)]
pub struct OracleHouses {
    pub asc: f64,
    pub mc: f64,
    pub cusps: Vec<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OracleMeta {
    #[serde(default)]
    pub swisseph_version: String,
    #[serde(default)]
    pub n: u64,
    #[serde(default)]
    pub seed: u64,
    #[serde(default)]
    pub start_jd: f64,
    #[serde(default)]
    pub end_jd: f64,
    #[serde(default)]
    pub ephe_path: Option<String>,
    /// body name -> backend label ("moshier" / "swieph" / "jpleph") observed
    /// on the first record. Honest disclosure of which engine the oracle used.
    #[serde(default)]
    pub body_backend: BTreeMap<String, String>,
}
