//! # xalen-lalkitab
//!
//! Lal Kitab remedial astrology engine.
//!
//! Implements the unique house-based system from Pt. Roop Chand Joshi's
//! five volumes (1939-1952). Unlike classical BPHS-based Jyotish, Lal Kitab
//! judges planets by **house occupation** rather than sign placement, and
//! prescribes concrete material remedies (donations, items, feeding animals)
//! for each of the 108 planet-house combinations.
//!
//! ## Modules
//!
//! - [`houses`] -- chart representation, natural house rulers
//! - [`debts`]  -- five categories of planetary debt (Rin)
//! - [`effects`] -- good / neutral / troubled classification per planet-house
//! - [`dormant`] -- dormant planet detection (Soya Hua Graha)
//! - [`remedies`] -- 108-entry remedy lookup table
//! - [`varshphal`] -- Lal Kitab annual chart construction

pub mod debts;
pub mod dormant;
pub mod effects;
pub mod houses;
pub mod remedies;
pub mod varshphal;

// Re-export core types at crate root for convenience.
pub use debts::{DebtKind, PlanetaryDebt, detect_debts};
pub use dormant::is_dormant;
pub use effects::{LalKitabEffect, planet_effect};
pub use houses::{LalKitabChart, Planet};
pub use remedies::{LalKitabRemedy, remedy_lookup};
pub use varshphal::{AnnualChart, build_annual_chart};
