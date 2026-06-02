//! JPL DE440 / NAIF DAF/SPK binary ephemeris reader.
//!
//! Reads real NAIF DAF/SPK `.bsp` files (the standard format produced by
//! JPL for DE440, DE441, etc.) to provide sub-milliarcsecond planetary
//! positions via Chebyshev polynomial interpolation.
//!
//! **This module provides the full reader infrastructure.**  When no binary
//! file is loaded it transparently falls back to the crate's VSOP87 provider.
//!
//! # NAIF DAF/SPK file layout
//!
//! ```text
//! File Record (1024 bytes):
//!   LOCIDW[8]  = "DAF/SPK "
//!   ND (i32)   = 2  (number of double-precision components per summary)
//!   NI (i32)   = 6  (number of integer components per summary)
//!   LOCIFN[60] = internal filename
//!   FWARD (i32)= first summary record number
//!   BWARD (i32)= last summary record number
//!   ...
//!   LOCFMT[8]  = "LTL-IEEE" or "BIG-IEEE"
//!
//! Summary Records (at record_number * 1024):
//!   NEXT (f64), PREV (f64), NSUM (f64)
//!   Then NSUM summaries, each ND + ceil((NI+1)/2) = 5 f64 words
//!
//! SPK Type 2 segment data (Chebyshev position):
//!   N records of RSIZE doubles each
//!   Then 4 doubles at the segment end: INIT, INTLEN, RSIZE, N
//!   Each record: MID, RADIUS, then 3 sets of Chebyshev coefficients (X,Y,Z)
//! ```

use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::body::Body;
use crate::provider::{EphemerisError, EphemerisProvider};
use crate::vsop::Vsop87Provider;
use xalen_coords::{CartesianPosition, EclipticPosition};
use xalen_time::{JdTT, JulianDay};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// J2000.0 epoch as a Julian Date (TDB).
const J2000_JD: f64 = 2_451_545.0;

/// Seconds per day.
const SECONDS_PER_DAY: f64 = 86_400.0;

/// AU in km (IAU 2012 exact value, same as DE440).
const AU_KM: f64 = 149_597_870.700;

/// Speed of light in AU per day (IAU 2012 exact), for the apparent-place
/// light-time correction. Matches the value used by the VSOP87 provider.
const LIGHT_SPEED_AU_PER_DAY: f64 = 173.144_632_674_24;

/// Earth-Moon mass ratio (DE440 value).
const EMRAT: f64 = 81.300568;

/// Worst-body apparent-longitude residual quoted by the DE440 provider's
/// [`EphemerisProvider::accuracy_arcsec`].
///
/// The DE440 *raw* geometry is sub-milliarcsecond, and `accuracy_arcsec`
/// describes the worst PHYSICAL body the provider serves through its apparent-
/// place reduction (light-time + IAU 2006 precession + IAU 2000B nutation; the
/// Moon additionally gets its geocentric light-time but NOT annual aberration).
/// Previously the Moon was the worst body at ~11" — but that residual was a BUG:
/// the full annual aberration term (κ=20.49552", correct for planets/Sun that do
/// not share Earth's heliocentric velocity) was wrongly applied to the
/// geocentric Moon. With that term removed (see
/// `De440Provider::apparent_moon_de440`) the apparent Moon agrees with JPL
/// Horizons to sub-arcsecond, like the Sun and planets. We bound the worst body
/// conservatively at 2", which honestly covers the apparent-place reduction
/// (validated Moon residual is well below this) and still sits far inside the
/// 36" (0.01°) cross-validation tolerance the test enforces. Bodies the kernel
/// lacks (the lunar nodes especially) fall back to the analytical model and
/// carry its larger, separately-documented residual.
const DE440_APPARENT_WORST_ARCSEC: f64 = 2.0;

/// DAF record size in bytes (always 1024 for NAIF DAF files).
const DAF_RECORD_BYTES: usize = 1024;

// ---------------------------------------------------------------------------
// NAIF body ID mapping
// ---------------------------------------------------------------------------

/// NAIF integer body IDs used in SPK files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NaifId(pub i32);

impl NaifId {
    pub const MERCURY_BARYCENTER: Self = NaifId(1);
    pub const VENUS_BARYCENTER: Self = NaifId(2);
    pub const EARTH_MOON_BARYCENTER: Self = NaifId(3);
    pub const MARS_BARYCENTER: Self = NaifId(4);
    pub const JUPITER_BARYCENTER: Self = NaifId(5);
    pub const SATURN_BARYCENTER: Self = NaifId(6);
    pub const URANUS_BARYCENTER: Self = NaifId(7);
    pub const NEPTUNE_BARYCENTER: Self = NaifId(8);
    pub const PLUTO_BARYCENTER: Self = NaifId(9);
    pub const SUN: Self = NaifId(10);
    pub const MOON: Self = NaifId(301);
    pub const EARTH: Self = NaifId(399);
    pub const MERCURY: Self = NaifId(199);
    pub const VENUS: Self = NaifId(299);

    /// Map from our `Body` enum to NAIF target+center pairs.
    ///
    /// This mapping deliberately uses the planetary *barycenters* (NAIF 1-9)
    /// plus the Earth-Moon Barycenter (3), NOT the individual body centers:
    /// - Planets 1-9: barycenter relative to Solar System Barycenter (0)
    /// - Sun (10): relative to SSB (0)
    /// - Earth: the Earth-Moon Barycenter (3) relative to SSB (0)
    /// - Moon (301): relative to the Earth-Moon Barycenter (3)
    ///
    /// Note that the standard DE440/DE440s kernels DO contain body-center
    /// segments such as Earth (399), Mercury (199) and Venus (299); this mapping
    /// simply does not use them. A true geocenter is derived from the EMB and
    /// Moon segments by [`De440Reader::earth_ssb_au`] rather than by reading
    /// 399 directly, which keeps the mapping valid for the trimmed `de440s.bsp`
    /// kernel layout we target.
    pub fn body_to_naif(body: Body) -> Option<(NaifId, NaifId)> {
        match body {
            Body::Mercury => Some((NaifId(1), NaifId(0))),
            Body::Venus => Some((NaifId(2), NaifId(0))),
            Body::Earth => Some((NaifId(3), NaifId(0))),
            Body::Mars => Some((NaifId(4), NaifId(0))),
            Body::Jupiter => Some((NaifId(5), NaifId(0))),
            Body::Saturn => Some((NaifId(6), NaifId(0))),
            Body::Uranus => Some((NaifId(7), NaifId(0))),
            Body::Neptune => Some((NaifId(8), NaifId(0))),
            Body::Pluto => Some((NaifId(9), NaifId(0))),
            Body::Sun => Some((NaifId(10), NaifId(0))),
            Body::Moon => Some((NaifId(301), NaifId(3))),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// DE440 target enum (kept for backward compatibility with tests)
// ---------------------------------------------------------------------------

/// Maps our `Body` enum to the DE440 target concept.
///
/// This is maintained for backward compatibility. Internally the reader
/// now uses NAIF body IDs, but the legacy API surface is preserved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum De440Target {
    Mercury = 0,
    Venus = 1,
    EarthMoonBary = 2,
    Mars = 3,
    Jupiter = 4,
    Saturn = 5,
    Uranus = 6,
    Neptune = 7,
    Pluto = 8,
    MoonGeo = 9,
    Sun = 10,
    #[allow(dead_code)]
    Nutations = 11,
    #[allow(dead_code)]
    Librations = 12,
}

impl De440Target {
    fn from_body(body: Body) -> Option<Self> {
        match body {
            Body::Mercury => Some(Self::Mercury),
            Body::Venus => Some(Self::Venus),
            Body::Earth => Some(Self::EarthMoonBary),
            Body::Mars => Some(Self::Mars),
            Body::Jupiter => Some(Self::Jupiter),
            Body::Saturn => Some(Self::Saturn),
            Body::Uranus => Some(Self::Uranus),
            Body::Neptune => Some(Self::Neptune),
            Body::Pluto => Some(Self::Pluto),
            Body::Moon => Some(Self::MoonGeo),
            Body::Sun => Some(Self::Sun),
            _ => None,
        }
    }

    /// Convert legacy target to NAIF (target, center) pair.
    fn to_naif_pair(self) -> (NaifId, NaifId) {
        match self {
            Self::Mercury => (NaifId(1), NaifId(0)),
            Self::Venus => (NaifId(2), NaifId(0)),
            Self::EarthMoonBary => (NaifId(3), NaifId(0)),
            Self::Mars => (NaifId(4), NaifId(0)),
            Self::Jupiter => (NaifId(5), NaifId(0)),
            Self::Saturn => (NaifId(6), NaifId(0)),
            Self::Uranus => (NaifId(7), NaifId(0)),
            Self::Neptune => (NaifId(8), NaifId(0)),
            Self::Pluto => (NaifId(9), NaifId(0)),
            Self::Sun => (NaifId(10), NaifId(0)),
            Self::MoonGeo => (NaifId(301), NaifId(3)),
            Self::Nutations => (NaifId(0), NaifId(0)), // unused
            Self::Librations => (NaifId(0), NaifId(0)), // unused
        }
    }

    fn num_components(self) -> usize {
        3 // All SPK Type 2 segments are 3D position
    }
}

// ---------------------------------------------------------------------------
// Legacy types (kept for public API compatibility)
// ---------------------------------------------------------------------------

/// One row of the IPT table (legacy format).
/// Kept for backward compatibility with existing tests.
#[derive(Debug, Clone, Copy)]
pub struct IptEntry {
    /// 1-based offset into the data record.
    pub offset: usize,
    /// Number of Chebyshev coefficients per component per sub-interval.
    pub num_coeffs: usize,
    /// Number of sub-intervals within each block.
    pub num_sub_intervals: usize,
}

impl IptEntry {
    /// Total number of f64 words this body occupies per data record.
    pub fn words_per_record(&self, num_components: usize) -> usize {
        self.num_coeffs * num_components * self.num_sub_intervals
    }
}

/// Parsed header (legacy structure for API compatibility).
#[derive(Debug, Clone)]
pub struct De440Header {
    /// Record size in f64 words.
    pub ksize: usize,
    /// Number of coefficients per record.
    pub ncoeff: usize,
    /// Start Julian date (TDB).
    pub jd_start: f64,
    /// End Julian date (TDB).
    pub jd_end: f64,
    /// Block span in days.
    pub block_span: f64,
    /// Pointer table: 13 entries.
    pub ipt: [IptEntry; 13],
    /// Earth-Moon mass ratio.
    pub emrat: f64,
    /// AU in km.
    pub au_km: f64,
}

impl De440Header {
    /// Build a header with DE440 defaults (no file required).
    pub fn de440_defaults() -> Self {
        let ipt = [
            IptEntry {
                offset: 3,
                num_coeffs: 14,
                num_sub_intervals: 4,
            }, // Mercury
            IptEntry {
                offset: 171,
                num_coeffs: 10,
                num_sub_intervals: 2,
            }, // Venus
            IptEntry {
                offset: 231,
                num_coeffs: 13,
                num_sub_intervals: 2,
            }, // EMB
            IptEntry {
                offset: 309,
                num_coeffs: 11,
                num_sub_intervals: 1,
            }, // Mars
            IptEntry {
                offset: 342,
                num_coeffs: 8,
                num_sub_intervals: 1,
            }, // Jupiter
            IptEntry {
                offset: 366,
                num_coeffs: 7,
                num_sub_intervals: 1,
            }, // Saturn
            IptEntry {
                offset: 387,
                num_coeffs: 6,
                num_sub_intervals: 1,
            }, // Uranus
            IptEntry {
                offset: 405,
                num_coeffs: 6,
                num_sub_intervals: 1,
            }, // Neptune
            IptEntry {
                offset: 423,
                num_coeffs: 6,
                num_sub_intervals: 1,
            }, // Pluto
            IptEntry {
                offset: 441,
                num_coeffs: 13,
                num_sub_intervals: 8,
            }, // Moon (geo)
            IptEntry {
                offset: 753,
                num_coeffs: 11,
                num_sub_intervals: 2,
            }, // Sun
            IptEntry {
                offset: 819,
                num_coeffs: 10,
                num_sub_intervals: 4,
            }, // Nutations
            IptEntry {
                offset: 899,
                num_coeffs: 10,
                num_sub_intervals: 4,
            }, // Librations
        ];

        Self {
            ksize: 1652,
            ncoeff: 826,
            jd_start: 2_287_184.5,
            jd_end: 2_688_976.5,
            block_span: 32.0,
            ipt,
            emrat: EMRAT,
            au_km: AU_KM,
        }
    }
}

/// A legacy data record (kept for backward-compatible `with_records`).
#[derive(Debug, Clone)]
pub struct De440Record {
    pub jd_start: f64,
    pub jd_end: f64,
    pub coefficients: Vec<f64>,
}

impl De440Record {
    /// Extract Chebyshev coefficients for a specific target body, sub-interval, and component.
    pub fn body_coeffs(
        &self,
        ipt: &IptEntry,
        num_components: usize,
        sub_interval: usize,
        component: usize,
    ) -> Option<&[f64]> {
        if ipt.num_coeffs == 0 || ipt.num_sub_intervals == 0 {
            return None;
        }
        // All index arithmetic uses checked ops: a corrupt IPT entry must yield
        // `None` rather than wrap a `usize` and slice out of bounds (or panic).
        let base = ipt.offset.checked_sub(3)?;
        let sub_block_size = ipt.num_coeffs.checked_mul(num_components)?;
        let start = sub_interval
            .checked_mul(sub_block_size)?
            .checked_add(component.checked_mul(ipt.num_coeffs)?)?
            .checked_add(base)?;
        let end = start.checked_add(ipt.num_coeffs)?;
        if end <= self.coefficients.len() {
            Some(&self.coefficients[start..end])
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Chebyshev computation (Clenshaw's recurrence) -- PRESERVED
// ---------------------------------------------------------------------------

/// Compute a Chebyshev polynomial using Clenshaw's recurrence relation.
///
/// Given coefficients `[C0, C1, ..., Cn-1]` and a normalized argument
/// `t_norm` in [-1, 1], computes:
///
///   `sum_{k=0}^{n-1} Ck * Tk(t_norm)`
///
/// where `Tk` is the Chebyshev polynomial of the first kind of degree k.
pub fn chebyshev_compute(coeffs: &[f64], t_norm: f64) -> f64 {
    debug_assert!(
        (-1.0..=1.0).contains(&t_norm)
            || (t_norm - 1.0).abs() < 1e-12
            || (t_norm + 1.0).abs() < 1e-12,
        "t_norm must be in [-1, 1], got {t_norm}"
    );

    let n = coeffs.len();
    if n == 0 {
        return 0.0;
    }
    if n == 1 {
        return coeffs[0];
    }

    let two_t = 2.0 * t_norm;
    let mut b_k_plus_1 = 0.0_f64;
    let mut b_k_plus_2 = 0.0_f64;

    for k in (1..n).rev() {
        let b_k = two_t * b_k_plus_1 - b_k_plus_2 + coeffs[k];
        b_k_plus_2 = b_k_plus_1;
        b_k_plus_1 = b_k;
    }

    coeffs[0] + t_norm * b_k_plus_1 - b_k_plus_2
}

/// Compute the derivative of a Chebyshev polynomial (velocity).
pub fn chebyshev_derivative(coeffs: &[f64], t_norm: f64) -> f64 {
    let n = coeffs.len();
    if n <= 1 {
        return 0.0;
    }
    if n == 2 {
        return coeffs[1];
    }

    let mut d = vec![0.0; n];
    d[n - 1] = 0.0;
    if n >= 2 {
        d[n - 2] = 2.0 * (n as f64 - 1.0) * coeffs[n - 1];
    }
    for k in (0..n.saturating_sub(2)).rev() {
        d[k] = d[k + 2] + 2.0 * (k as f64 + 1.0) * coeffs[k + 1];
    }
    d[0] *= 0.5;

    chebyshev_compute(&d, t_norm)
}

// ---------------------------------------------------------------------------
// NAIF DAF/SPK structures
// ---------------------------------------------------------------------------

/// Endianness detected from the DAF file record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Endian {
    Little,
    Big,
}

impl Endian {
    fn read_f64(&self, buf: &[u8], offset: usize) -> f64 {
        let b: [u8; 8] = buf[offset..offset + 8].try_into().unwrap();
        match self {
            Endian::Little => f64::from_le_bytes(b),
            Endian::Big => f64::from_be_bytes(b),
        }
    }

    fn read_i32(&self, buf: &[u8], offset: usize) -> i32 {
        let b: [u8; 4] = buf[offset..offset + 4].try_into().unwrap();
        match self {
            Endian::Little => i32::from_le_bytes(b),
            Endian::Big => i32::from_be_bytes(b),
        }
    }
}

/// An SPK segment descriptor parsed from a DAF summary.
#[derive(Debug, Clone)]
struct SpkSegment {
    /// Start epoch in seconds past J2000 TDB.
    start_sec: f64,
    /// End epoch in seconds past J2000 TDB.
    end_sec: f64,
    /// NAIF target body ID.
    target: i32,
    /// NAIF center body ID.
    center: i32,
    /// Reference frame (1 = J2000).
    #[allow(dead_code)]
    frame: i32,
    /// SPK data type (2 = Chebyshev position, 3 = Chebyshev position+velocity).
    data_type: i32,
    /// Start word address in the file (1-based, in f64 words).
    start_word: i32,
    /// End word address in the file (1-based, in f64 words).
    end_word: i32,
    /// Order in which this segment was encountered while walking the SPK
    /// summary records (0-based, monotonically increasing). When two segments
    /// for the same (target, center) pair both bracket a requested epoch, the
    /// one with the LARGER `file_order` (i.e. appended later in the file) wins,
    /// matching SPK/SPICE "last-loaded segment takes priority" semantics.
    file_order: usize,
}

/// Type 2 segment metadata read from the segment's directory area.
#[derive(Debug, Clone)]
struct Type2Directory {
    /// Initial epoch of the first record (seconds past J2000 TDB).
    init: f64,
    /// Length of each time interval in seconds.
    intlen: f64,
    /// Record size in f64 words.
    rsize: usize,
    /// Number of Chebyshev records in the segment.
    n: usize,
    /// Number of Chebyshev coefficients per component.
    ncoeffs: usize,
}

/// A fully loaded SPK Type 2 segment with all coefficient data in memory.
#[derive(Debug, Clone)]
struct LoadedSegment {
    /// Segment descriptor.
    descriptor: SpkSegment,
    /// Type 2 directory.
    directory: Type2Directory,
    /// All coefficient data for this segment (flat array of f64).
    /// Records are stored contiguously: record 0 at offset 0,
    /// record 1 at offset rsize, etc.
    data: Vec<f64>,
}

impl LoadedSegment {
    /// Evaluate the position (x, y, z) in km at the given epoch
    /// (seconds past J2000 TDB).
    fn position_km(&self, t_sec: f64) -> Result<(f64, f64, f64), EphemerisError> {
        let dir = &self.directory;

        // Find which record covers this epoch. The directory covers
        // [init, init + n*intlen]; `offset` is the fractional record position.
        // Previously an out-of-coverage epoch was silently CLAMPED (negative
        // offset -> `floor() as usize` saturates to 0; an epoch past the last
        // record -> `.min(n-1)`), which returns a position for the WRONG record
        // and masks a coverage gap. Reject instead. `find_segment` already bounds
        // queries to the segment descriptor's [start_sec, end_sec]; this guards
        // the case where a (crafted) kernel's directory coverage is narrower than
        // its summary descriptor, so an epoch can pass `find_segment` yet fall
        // outside the records the directory actually holds.
        //
        // `init`/`intlen` were validated finite with intlen > 0 at load time, so
        // `offset` is finite. Allow a half-record epsilon past the last record so
        // the legitimate exact-end epoch (`find_segment` admits `t_sec ==
        // end_sec`) maps to the final record rather than being rejected.
        let offset = (t_sec - dir.init) / dir.intlen;
        if offset < 0.0 || offset >= dir.n as f64 + 0.5 {
            return Err(EphemerisError::EpochOutOfRange(
                J2000_JD + t_sec / SECONDS_PER_DAY,
            ));
        }
        let record_idx = (offset.floor() as usize).min(dir.n.saturating_sub(1));

        // Read the record.
        let rec_start = record_idx * dir.rsize;
        if rec_start + dir.rsize > self.data.len() {
            return Err(EphemerisError::ComputationFailed(format!(
                "record index {} out of bounds (data len {}, rsize {})",
                record_idx,
                self.data.len(),
                dir.rsize
            )));
        }
        let record = &self.data[rec_start..rec_start + dir.rsize];

        // record[0] = MID (midpoint epoch in seconds past J2000)
        // record[1] = RADIUS (half-interval in seconds)
        let mid = record[0];
        let radius = record[1];

        // Normalize time to [-1, 1] for Chebyshev evaluation.
        let t_norm = if radius > 0.0 {
            ((t_sec - mid) / radius).clamp(-1.0, 1.0)
        } else {
            0.0
        };

        // Coefficients for X, Y, Z are interleaved after MID and RADIUS.
        // Each component has ncoeffs coefficients.
        let nc = dir.ncoeffs;
        let x_coeffs = &record[2..2 + nc];
        let y_coeffs = &record[2 + nc..2 + 2 * nc];
        let z_coeffs = &record[2 + 2 * nc..2 + 3 * nc];

        let x = chebyshev_compute(x_coeffs, t_norm);
        let y = chebyshev_compute(y_coeffs, t_norm);
        let z = chebyshev_compute(z_coeffs, t_norm);

        Ok((x, y, z))
    }
}

// ---------------------------------------------------------------------------
// DE440 reader -- NAIF DAF/SPK parser
// ---------------------------------------------------------------------------

/// Best-effort scan of the DAF comment area for a JPL DE/LE ephemeris label
/// such as "DE440". NAIF planetary kernels embed a name like
/// "JPL planetary and lunar ephemeris DE440" (modern comment text) or the legacy
/// "DE-0440LE-0440" token in the comment records that sit between the file
/// record and the first summary record (`FWARD`). Returns the normalized label
/// (e.g. "DE440") when found, else `None`.
///
/// Bounded and panic-free: it never reads outside the comment region. It matches
/// "DE"/"DE-" ONLY at a word boundary (not preceded by an ASCII letter, so
/// "DECEMBER", "NODE440" and "CODE440" do not match the embedded "DE") followed
/// by at least three ASCII digits, with all leading zeros stripped — so both the
/// real "DE-0440" form and a wider zero-pad like "DE-00440" normalize to "DE440"
/// (verified against the real `de440.bsp`/`de440s.bsp` comment bytes).
fn detect_de_label(file_data: &[u8], fward: i32) -> Option<String> {
    if fward < 2 {
        return None;
    }
    // Comment records are records 2..FWARD; record 1 is the file record.
    let start = DAF_RECORD_BYTES;
    let end = (fward as usize)
        .saturating_sub(1)
        .saturating_mul(DAF_RECORD_BYTES)
        .min(file_data.len());
    if start >= end {
        return None;
    }
    let region = &file_data[start..end];
    let n = region.len();
    let mut i = 0;
    while i + 2 < n {
        // Require a LEFT word boundary so the "DE" inside "NODE440"/"DECEMBER"
        // is not matched.
        let at_boundary = i == 0 || !region[i - 1].is_ascii_alphabetic();
        if at_boundary
            && region[i].to_ascii_uppercase() == b'D'
            && region[i + 1].to_ascii_uppercase() == b'E'
        {
            let mut j = i + 2;
            if j < n && region[j] == b'-' {
                j += 1;
            }
            let dig_start = j;
            // Consume the WHOLE digit run (bounded by the comment region), not a
            // fixed width, so a wider zero-pad like "DE-00440" normalizes to
            // "DE440" instead of truncating to "DE44".
            while j < n && region[j].is_ascii_digit() {
                j += 1;
            }
            if j - dig_start >= 3 {
                let digits: String = region[dig_start..j].iter().map(|&b| b as char).collect();
                let num = digits.trim_start_matches('0');
                if !num.is_empty() {
                    return Some(format!("DE{num}"));
                }
            }
        }
        i += 1;
    }
    None
}

/// Reader for JPL DE440 (NAIF DAF/SPK) binary ephemeris data.
///
/// Loads and indexes all SPK Type 2 segments from a `.bsp` file,
/// then evaluates Chebyshev polynomials for any requested body/epoch.
#[derive(Debug, Clone)]
pub struct De440Reader {
    /// Legacy header (populated from segment metadata or defaults).
    header: De440Header,
    /// Loaded segments indexed by (target_naif_id, center_naif_id).
    segments: HashMap<(i32, i32), Vec<LoadedSegment>>,
    /// Legacy records (only used when constructed via `with_records`).
    legacy_records: Vec<De440Record>,
    /// Whether this reader was constructed from a real SPK file.
    is_spk: bool,
    /// JPL ephemeris label (e.g. "DE440") detected from the DAF comment area,
    /// when present. `None` means the kernel is a real SPK but its provenance
    /// could not be confirmed (synthetic test kernel, comment-stripped file, or
    /// a non-DE kernel). Provenance metadata that advertises "DE440" is gated on
    /// this, NOT on [`is_spk`], so we never claim DE440 we have not verified.
    kernel_id: Option<String>,
}

impl De440Reader {
    /// Create a reader with no data (header-only, for testing).
    pub fn empty() -> Self {
        Self {
            header: De440Header::de440_defaults(),
            segments: HashMap::new(),
            legacy_records: Vec::new(),
            is_spk: false,
            kernel_id: None,
        }
    }

    /// Load a NAIF DAF/SPK file (`.bsp`) from disk.
    ///
    /// Parses the DAF file record, walks summary records via the FWARD
    /// chain, loads all SPK Type 2 segments into memory, and indexes
    /// them by (target, center) NAIF body ID pair.
    pub fn from_file(path: &Path) -> Result<Self, EphemerisError> {
        let mut file = std::fs::File::open(path)?;
        let file_len = file.metadata()?.len();

        if file_len < DAF_RECORD_BYTES as u64 {
            return Err(EphemerisError::InvalidFormat(format!(
                "file too small for a DAF record ({file_len} bytes, need >= {DAF_RECORD_BYTES})"
            )));
        }

        // -----------------------------------------------------------------
        // Read the file record (first 1024 bytes)
        // -----------------------------------------------------------------
        let mut file_rec = vec![0u8; DAF_RECORD_BYTES];
        file.seek(SeekFrom::Start(0))?;
        file.read_exact(&mut file_rec)?;

        // LOCIDW: first 8 bytes should be "DAF/SPK " or similar DAF marker.
        let locidw = std::str::from_utf8(&file_rec[0..8])
            .unwrap_or("")
            .trim_end();

        if !locidw.starts_with("DAF/") {
            // Not a DAF file -- try the legacy format as a fallback.
            return Self::from_file_legacy(path);
        }

        // ND and NI at offsets 8 and 12.
        // We try little-endian first, then big-endian.
        let nd_le = i32::from_le_bytes(file_rec[8..12].try_into().unwrap());
        let ni_le = i32::from_le_bytes(file_rec[12..16].try_into().unwrap());

        // Detect endianness from LOCFMT at offset 88.
        let locfmt = std::str::from_utf8(&file_rec[88..96])
            .unwrap_or("")
            .trim_end();

        let endian = if locfmt.starts_with("LTL") {
            Endian::Little
        } else if locfmt.starts_with("BIG") {
            Endian::Big
        } else {
            // Try to infer from ND/NI values (SPK always has ND=2, NI=6).
            if nd_le == 2 && ni_le == 6 {
                Endian::Little
            } else {
                let nd_be = i32::from_be_bytes(file_rec[8..12].try_into().unwrap());
                let ni_be = i32::from_be_bytes(file_rec[12..16].try_into().unwrap());
                if nd_be == 2 && ni_be == 6 {
                    Endian::Big
                } else {
                    return Err(EphemerisError::InvalidFormat(format!(
                        "cannot detect endianness: LOCFMT='{locfmt}', ND_LE={nd_le}, NI_LE={ni_le}"
                    )));
                }
            }
        };

        let nd = endian.read_i32(&file_rec, 8);
        let ni = endian.read_i32(&file_rec, 12);

        if nd != 2 || ni != 6 {
            return Err(EphemerisError::InvalidFormat(format!(
                "unexpected DAF dimensions: ND={nd}, NI={ni} (expected ND=2, NI=6 for SPK)"
            )));
        }

        // FWARD and BWARD at offsets 76 and 80.
        let fward = endian.read_i32(&file_rec, 76);
        let _bward = endian.read_i32(&file_rec, 80);

        if fward < 1 {
            return Err(EphemerisError::InvalidFormat(format!(
                "invalid FWARD pointer: {fward}"
            )));
        }

        // -----------------------------------------------------------------
        // Walk summary records via the FWARD chain
        // -----------------------------------------------------------------
        // NAIF DAF summary layout: each summary is `ND` double-precision words
        // followed by `ceil(NI/2)` words of packed integers (each f64 word holds
        // two i32 values). For an SPK with ND=2, NI=6, that is 2 + ceil(6/2) = 2 + 3
        // = 5 f64 words; the 3 integer words hold all 6 i32 values with no padding.
        let summary_size = nd as usize + (ni as usize).div_ceil(2); // 2 + 3 = 5

        let mut segments: Vec<SpkSegment> = Vec::new();
        let mut current_record = fward as u64;

        // Read all file data into memory for easier access.
        let mut file_data = vec![0u8; file_len as usize];
        file.seek(SeekFrom::Start(0))?;
        file.read_exact(&mut file_data)?;

        // Best-effort JPL ephemeris provenance: scan the DAF comment area for a
        // "DE440"-style label so we only advertise DE440 metadata when we can
        // actually confirm it (see `detect_de_label`).
        let kernel_id = detect_de_label(&file_data, fward);

        // Cycle/runaway guard for the summary-record (FWARD) chain. A malformed
        // file whose NEXT pointer references an already-visited record (or forms
        // a longer loop) would otherwise spin forever. There can be at most one
        // summary record per 1024-byte file record, so bound the walk by the
        // record count and refuse to revisit a record.
        let max_records = (file_data.len() / DAF_RECORD_BYTES) + 1;
        let mut visited: std::collections::HashSet<u64> = std::collections::HashSet::new();

        loop {
            if current_record < 1 {
                break;
            }
            // A DAF summary chain is a linked list of records via NEXT pointers.
            // A NEXT that revisits a record (cycle) or a chain longer than the
            // file can hold is structural corruption, not a normal terminus —
            // reject the whole file rather than silently loading a partial,
            // untrustworthy kernel. `with_de440` turns this Err into a clean
            // VSOP87 fallback, so the caller never gets a half-loaded DE440.
            if !visited.insert(current_record) || visited.len() > max_records {
                return Err(EphemerisError::InvalidFormat(
                    "DAF summary-record chain contains a cycle or exceeds the file's record count"
                        .into(),
                ));
            }
            let rec_offset = (current_record as usize - 1) * DAF_RECORD_BYTES;
            if rec_offset + DAF_RECORD_BYTES > file_data.len() {
                // A NEXT pointer referencing a summary record past end-of-file is
                // a corrupt chain — reject the whole file rather than keep the
                // segments parsed so far (a silent partial load).
                return Err(EphemerisError::InvalidFormat(
                    "DAF summary-record chain points past end-of-file".into(),
                ));
            }

            // Summary record layout:
            // Word 0: NEXT (f64 encoding of next record number, 0.0 if none)
            // Word 1: PREV (f64 encoding of prev record number)
            // Word 2: NSUM (f64 encoding of number of summaries in this record)
            // Words 3+: summaries
            let next_f64 = endian.read_f64(&file_data, rec_offset);
            let _prev_f64 = endian.read_f64(&file_data, rec_offset + 8);
            let nsum_f64 = endian.read_f64(&file_data, rec_offset + 16);

            // NEXT is a record number encoded as an f64. A non-finite or
            // out-of-range NEXT must be rejected BEFORE the `as i64`/`as u64`
            // cast: an infinite NEXT saturates `as i64` to `i64::MAX`, and the
            // subsequent `(current_record as usize - 1) * DAF_RECORD_BYTES` at
            // the top of the loop then overflows (panic in debug, silent wrap in
            // release). A finite NEXT is bounded to a real record index; anything
            // past the file's record count is corruption (the EOF bounds check
            // would otherwise only catch it after the overflow). NaN slips past
            // `<= 0.0` (NaN comparisons are false), so test finiteness explicitly.
            if !next_f64.is_finite() {
                return Err(EphemerisError::InvalidFormat(
                    "DAF summary-record NEXT pointer is not a finite record number".into(),
                ));
            }
            // `max_records` already bounds the chain; a NEXT beyond it cannot name
            // a real record. Cap the candidate so the `as i64`/`as u64` cast and
            // the loop-top multiply can never overflow.
            if next_f64 > max_records as f64 {
                return Err(EphemerisError::InvalidFormat(
                    "DAF summary-record NEXT pointer references a record past end-of-file".into(),
                ));
            }
            let next = next_f64 as i64;
            let nsum = nsum_f64 as usize;

            // NAIF DAF: summaries never cross a physical (1024-byte) record
            // boundary. After the 3 control words (24 bytes), a record holds at
            // most (1024 - 24) / (summary_size * 8) summaries — 25 for an SPK
            // (ND=2, NI=6 → summary_size=5). An NSUM larger than that is corrupt
            // even in a long file (it would otherwise parse the NEXT record's
            // bytes as summaries instead of being rejected).
            let max_sum_per_record = (DAF_RECORD_BYTES - 24) / (summary_size * 8);
            if nsum > max_sum_per_record {
                return Err(EphemerisError::InvalidFormat(format!(
                    "DAF summary record NSUM={nsum} exceeds the per-record maximum \
                     {max_sum_per_record}"
                )));
            }

            for i in 0..nsum {
                let sum_offset = rec_offset + 24 + i * summary_size * 8;
                if sum_offset + summary_size * 8 > file_data.len() {
                    // Defense in depth: also reject if a summary would run past
                    // end-of-file (the per-record cap above already bounds this).
                    return Err(EphemerisError::InvalidFormat(
                        "DAF summary record's NSUM overruns the file".into(),
                    ));
                }

                // Read ND=2 doubles.
                let dc0 = endian.read_f64(&file_data, sum_offset);
                let dc1 = endian.read_f64(&file_data, sum_offset + 8);

                // Read NI=6 integers packed into 3 f64 words.
                // Each f64 word holds 2 i32 values.
                let int_base = sum_offset + nd as usize * 8;
                let mut ints = [0i32; 6];
                for j in 0..3 {
                    let word_offset = int_base + j * 8;
                    ints[j * 2] = endian.read_i32(&file_data, word_offset);
                    ints[j * 2 + 1] = endian.read_i32(&file_data, word_offset + 4);
                }

                // `segments.len()` is the count of segments parsed so far across
                // ALL summary records walked, so it is a monotonically increasing
                // file-order index (later-in-file = higher value).
                let file_order = segments.len();
                segments.push(SpkSegment {
                    start_sec: dc0,
                    end_sec: dc1,
                    target: ints[0],
                    center: ints[1],
                    frame: ints[2],
                    data_type: ints[3],
                    start_word: ints[4],
                    end_word: ints[5],
                    file_order,
                });
            }

            if next <= 0 {
                break;
            }
            current_record = next as u64;
        }

        if segments.is_empty() {
            return Err(EphemerisError::InvalidFormat(
                "no SPK segments found in the DAF file".into(),
            ));
        }

        // -----------------------------------------------------------------
        // Load Type 2 segment data
        // -----------------------------------------------------------------
        let mut loaded: HashMap<(i32, i32), Vec<LoadedSegment>> = HashMap::new();
        let mut jd_start = f64::MAX;
        let mut jd_end = f64::MIN;

        for seg in &segments {
            // Only support Type 2 (Chebyshev position) for now.
            if seg.data_type != 2 {
                continue;
            }

            // Read the Type 2 directory from the END of the segment.
            // The last 4 f64 words of the segment are: INIT, INTLEN, RSIZE, N.
            //
            // SPK word addresses are 1-based positive integers. A malformed file
            // can carry a negative or zero word address; casting a negative i32
            // to usize wraps to a huge value, and the subsequent `* 8` overflows
            // (panic in debug, silent wrap in release). Reject non-positive word
            // addresses, then use checked_mul so an oversized address skips the
            // segment instead of overflowing.
            if seg.end_word < 1 || seg.start_word < 1 {
                return Err(EphemerisError::InvalidFormat(format!(
                    "SPK segment has a non-positive word address (start={}, end={})",
                    seg.start_word, seg.end_word
                )));
            }
            let seg_end_byte =
                match (seg.end_word as usize).checked_mul(8) {
                    Some(b) => b, // 1-based word to byte offset
                    None => return Err(EphemerisError::InvalidFormat(
                        "SPK segment end-word address overflows when converted to a byte offset"
                            .into(),
                    )),
                };
            if seg_end_byte < 32 || seg_end_byte > file_data.len() {
                return Err(EphemerisError::InvalidFormat(format!(
                    "SPK segment end address {seg_end_byte} is out of file bounds ({})",
                    file_data.len()
                )));
            }

            let dir_start = seg_end_byte - 4 * 8;
            let init = endian.read_f64(&file_data, dir_start);
            let intlen = endian.read_f64(&file_data, dir_start + 8);
            let rsize_f64 = endian.read_f64(&file_data, dir_start + 16);
            let n_f64 = endian.read_f64(&file_data, dir_start + 24);

            // RSIZE and N are counts encoded as f64. Casting a non-finite or
            // negative f64 with `as usize` silently saturates (NaN -> 0,
            // +inf -> usize::MAX, negative -> 0), which can pass the cheap
            // `rsize < 5` / `n == 0` checks (usize::MAX is neither) and feed a
            // garbage count into the downstream arithmetic. Validate that each is
            // finite, non-negative and integral BEFORE the cast so only a true
            // whole-number count is ever turned into a `usize`.
            let cast_count = |v: f64, label: &str| -> Result<usize, EphemerisError> {
                if !v.is_finite() || v < 0.0 || v.fract() != 0.0 || v > usize::MAX as f64 {
                    return Err(EphemerisError::InvalidFormat(format!(
                        "SPK Type 2 segment directory {label} is not a finite \
                         non-negative whole number (got {v})"
                    )));
                }
                Ok(v as usize)
            };
            let rsize = cast_count(rsize_f64, "RSIZE")?;
            let n = cast_count(n_f64, "N")?;

            // A NaN/inf `intlen` slips past `intlen <= 0.0` (NaN comparisons are
            // always false), and a non-finite `init` poisons the record-index
            // arithmetic in `position_km`. Reject both explicitly. A Type 2
            // record is [MID, RADIUS, X..., Y..., Z...], so RSIZE must be
            // 2 + 3*ncoeffs — i.e. (rsize - 2) must be a non-negative multiple of
            // 3. Reject any RSIZE that fails this (it would make `ncoeffs` and the
            // per-component coefficient slicing in `position_km` inconsistent).
            if !init.is_finite()
                || !intlen.is_finite()
                || rsize < 5
                || (rsize - 2) % 3 != 0
                || n == 0
                || intlen <= 0.0
            {
                return Err(EphemerisError::InvalidFormat(format!(
                    "SPK Type 2 segment has an invalid directory \
                     (init={init}, rsize={rsize}, n={n}, intlen={intlen})"
                )));
            }

            // Number of coefficients per component:
            // record = [MID, RADIUS, X_coeffs..., Y_coeffs..., Z_coeffs...]
            // rsize = 2 + 3 * ncoeffs  =>  ncoeffs = (rsize - 2) / 3
            let ncoeffs = (rsize - 2) / 3;

            // Validate that the segment's declared word span actually matches the
            // size implied by the directory. An SPK Type 2 segment occupies the
            // 1-based inclusive word range [start_word, end_word], which holds
            // `n` Chebyshev records of `rsize` words each followed by the 4-word
            // directory (INIT, INTLEN, RSIZE, N) read above:
            //     end_word - start_word + 1  ==  n * rsize + 4
            // (verified against the synthetic builder, where each segment is
            // `seg_words = rsize + 4` for n_records = 1). A segment whose summary
            // span disagrees with its own directory is internally inconsistent —
            // reject it rather than trust a partial/overlapping read. start_word
            // and end_word are already known to be >= 1 (checked above), so the
            // `as usize` casts here cannot wrap.
            let declared_words = (seg.end_word as usize)
                .checked_sub(seg.start_word as usize)
                .and_then(|d| d.checked_add(1));
            let expected_words = n.checked_mul(rsize).and_then(|w| w.checked_add(4));
            match (declared_words, expected_words) {
                (Some(decl), Some(exp)) if decl == exp => {}
                _ => {
                    return Err(EphemerisError::InvalidFormat(format!(
                        "SPK Type 2 segment span mismatch: summary declares \
                         {declared:?} words (start={start}, end={end}) but the \
                         directory implies n*rsize+4 = {expected:?} (n={n}, rsize={rsize})",
                        declared = declared_words,
                        start = seg.start_word,
                        end = seg.end_word,
                        expected = expected_words,
                    )));
                }
            }

            let directory = Type2Directory {
                init,
                intlen,
                rsize,
                n,
                ncoeffs,
            };

            // Read all coefficient records. SPK words are 1-based; a malformed
            // file with start_word == 0 must not underflow the usize subtraction,
            // and the word→byte `* 8` must not overflow for an oversized address.
            let seg_start_byte =
                match (seg.start_word as usize)
                    .checked_sub(1)
                    .and_then(|w| w.checked_mul(8))
                {
                    Some(b) => b,
                    None => return Err(EphemerisError::InvalidFormat(
                        "SPK segment start-word address overflows when converted to a byte offset"
                            .into(),
                    )),
                };
            // n and rsize come straight from file-controlled directory values;
            // do every multiplication/addition with checked arithmetic so a
            // malicious RSIZE/N cannot panic (debug) or wrap/over-allocate
            // (release) here.
            let words = match n.checked_mul(rsize) {
                Some(w) => w,
                None => {
                    return Err(EphemerisError::InvalidFormat(
                        "SPK Type 2 segment word count (n * rsize) overflows usize".into(),
                    ));
                }
            };
            let data_bytes = match words.checked_mul(8) {
                Some(b) => b,
                None => {
                    return Err(EphemerisError::InvalidFormat(
                        "SPK Type 2 segment byte size (n * rsize * 8) overflows usize".into(),
                    ));
                }
            };
            match seg_start_byte.checked_add(data_bytes) {
                Some(end) if end <= file_data.len() => {}
                _ => {
                    return Err(EphemerisError::InvalidFormat(format!(
                        "SPK Type 2 segment data ({data_bytes} bytes from {seg_start_byte}) \
                         overruns the file ({})",
                        file_data.len()
                    )));
                }
            }

            let mut data = Vec::with_capacity(words);
            for w in 0..words {
                let offset = seg_start_byte + w * 8;
                data.push(endian.read_f64(&file_data, offset));
            }

            // Track overall epoch range (convert seconds past J2000 to JD).
            let seg_jd_start = J2000_JD + seg.start_sec / SECONDS_PER_DAY;
            let seg_jd_end = J2000_JD + seg.end_sec / SECONDS_PER_DAY;
            if seg_jd_start < jd_start {
                jd_start = seg_jd_start;
            }
            if seg_jd_end > jd_end {
                jd_end = seg_jd_end;
            }

            let key = (seg.target, seg.center);
            loaded.entry(key).or_default().push(LoadedSegment {
                descriptor: seg.clone(),
                directory,
                data,
            });
        }

        if loaded.is_empty() {
            return Err(EphemerisError::InvalidFormat(
                "no loadable Type 2 segments found".into(),
            ));
        }

        // Sort segments within each key by start epoch for binary search.
        for segs in loaded.values_mut() {
            segs.sort_by(|a, b| {
                a.descriptor
                    .start_sec
                    .partial_cmp(&b.descriptor.start_sec)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        // Build a header from the loaded data.
        let header = De440Header {
            ksize: 0,
            ncoeff: 0,
            jd_start,
            jd_end,
            block_span: 32.0,
            ipt: De440Header::de440_defaults().ipt,
            emrat: EMRAT,
            au_km: AU_KM,
        };

        Ok(Self {
            header,
            segments: loaded,
            legacy_records: Vec::new(),
            is_spk: true,
            kernel_id,
        })
    }

    /// Legacy file parser for the old format (pre-DAF).
    /// Called as a fallback when `from_file` detects the file is not DAF.
    fn from_file_legacy(path: &Path) -> Result<Self, EphemerisError> {
        let mut file = std::fs::File::open(path)?;
        let file_len = file.metadata()?.len();

        const HEADER_BUF_LEN: usize = 8192;
        let mut buf = vec![0u8; HEADER_BUF_LEN];
        file.seek(SeekFrom::Start(0))?;
        let bytes_read = file.read(&mut buf)?;
        if bytes_read < 2860 {
            return Err(EphemerisError::InvalidFormat(format!(
                "file too small for a DE header ({bytes_read} bytes read, need >= 2860)"
            )));
        }

        let read_i32_le = |off: usize| -> i32 {
            let b: [u8; 4] = buf[off..off + 4].try_into().unwrap();
            i32::from_le_bytes(b)
        };
        let read_i32_be = |off: usize| -> i32 {
            let b: [u8; 4] = buf[off..off + 4].try_into().unwrap();
            i32::from_be_bytes(b)
        };

        let denum_le = read_i32_le(2840);
        let denum_be = read_i32_be(2840);
        let is_little_endian;
        if (400..=500).contains(&denum_le) {
            is_little_endian = true;
        } else if (400..=500).contains(&denum_be) {
            is_little_endian = false;
        } else {
            return Err(EphemerisError::InvalidFormat(format!(
                "cannot detect endianness: DENUM candidates LE={denum_le}, BE={denum_be}"
            )));
        }

        let read_f64 = |off: usize| -> f64 {
            let b: [u8; 8] = buf[off..off + 8].try_into().unwrap();
            if is_little_endian {
                f64::from_le_bytes(b)
            } else {
                f64::from_be_bytes(b)
            }
        };
        let read_i32 = |off: usize| -> i32 {
            let b: [u8; 4] = buf[off..off + 4].try_into().unwrap();
            if is_little_endian {
                i32::from_le_bytes(b)
            } else {
                i32::from_be_bytes(b)
            }
        };

        let jd_start = read_f64(2652);
        let jd_end = read_f64(2660);
        let block_span = read_f64(2668);

        if jd_start < 1_000_000.0 || jd_end < jd_start || block_span <= 0.0 {
            return Err(EphemerisError::InvalidFormat(format!(
                "invalid SS values: start={jd_start}, end={jd_end}, span={block_span}"
            )));
        }

        let au_km = read_f64(2680);
        let emrat = read_f64(2688);

        // IPT entries are stored as 1-based word indices and counts: they are
        // always >= 0 in a well-formed kernel. A negative i32 (corrupt bytes or
        // a mis-detected endianness) would wrap to an enormous `usize` and then
        // overflow the record-size arithmetic below, so validate the sign here
        // before casting.
        let read_ipt = |off: usize| -> Result<usize, EphemerisError> {
            let v = read_i32(off);
            if v < 0 {
                return Err(EphemerisError::InvalidFormat(format!(
                    "negative IPT value {v} at offset {off} (corrupt header or wrong endianness)"
                )));
            }
            Ok(v as usize)
        };

        let mut ipt = [IptEntry {
            offset: 0,
            num_coeffs: 0,
            num_sub_intervals: 0,
        }; 13];
        for (i, entry) in ipt.iter_mut().take(12).enumerate() {
            let base_off = 2696 + i * 12;
            *entry = IptEntry {
                offset: read_ipt(base_off)?,
                num_coeffs: read_ipt(base_off + 4)?,
                num_sub_intervals: read_ipt(base_off + 8)?,
            };
        }
        ipt[12] = IptEntry {
            offset: read_ipt(2844)?,
            num_coeffs: read_ipt(2848)?,
            num_sub_intervals: read_ipt(2852)?,
        };

        let mut max_word = 0usize;
        for (i, entry) in ipt.iter().enumerate() {
            if entry.num_coeffs == 0 || entry.num_sub_intervals == 0 {
                continue;
            }
            let nc = if i == 11 { 2 } else { 3 };
            // offset is a 1-based word index; guard against a malformed offset == 0.
            let base = match entry.offset.checked_sub(1) {
                Some(b) => b,
                None => continue,
            };
            // Mirror the checked arithmetic used in the SPK path: a corrupt
            // (coeffs × components × sub-intervals) product, or its addition to
            // `base`, must not silently wrap a `usize`.
            let words = entry
                .num_coeffs
                .checked_mul(nc)
                .and_then(|w| w.checked_mul(entry.num_sub_intervals));
            let end_word = match words.and_then(|w| base.checked_add(w)) {
                Some(e) => e,
                None => {
                    return Err(EphemerisError::InvalidFormat(format!(
                        "IPT entry {i} record size overflows usize \
                         (offset={}, coeffs={}, subintervals={})",
                        entry.offset, entry.num_coeffs, entry.num_sub_intervals
                    )));
                }
            };
            if end_word > max_word {
                max_word = end_word;
            }
        }
        let ncoeff = max_word + 2;
        let ksize = 2 * ncoeff;
        let record_bytes = ncoeff * 8;

        if record_bytes == 0 {
            return Err(EphemerisError::InvalidFormat(
                "computed record size is zero".into(),
            ));
        }

        let header = De440Header {
            ksize,
            ncoeff,
            jd_start,
            jd_end,
            block_span,
            ipt,
            emrat,
            au_km,
        };

        let data_offset = 2 * record_bytes as u64;
        if data_offset >= file_len {
            return Ok(Self {
                header,
                segments: HashMap::new(),
                legacy_records: Vec::new(),
                is_spk: false,
                kernel_id: None,
            });
        }

        let num_data_bytes = file_len - data_offset;
        let num_records = (num_data_bytes / record_bytes as u64) as usize;
        let mut records = Vec::with_capacity(num_records);
        file.seek(SeekFrom::Start(data_offset))?;

        let mut rec_buf = vec![0u8; record_bytes];
        for _ in 0..num_records {
            let n = file.read(&mut rec_buf)?;
            if n < record_bytes {
                break;
            }
            let doubles: Vec<f64> = (0..ncoeff)
                .map(|i| {
                    let off = i * 8;
                    let b: [u8; 8] = rec_buf[off..off + 8].try_into().unwrap();
                    if is_little_endian {
                        f64::from_le_bytes(b)
                    } else {
                        f64::from_be_bytes(b)
                    }
                })
                .collect();
            let rec_jd_start = doubles[0];
            let rec_jd_end = doubles[1];
            if rec_jd_end < jd_start || rec_jd_start > jd_end {
                continue;
            }
            records.push(De440Record {
                jd_start: rec_jd_start,
                jd_end: rec_jd_end,
                coefficients: doubles[2..].to_vec(),
            });
        }
        records.sort_by(|a, b| {
            a.jd_start
                .partial_cmp(&b.jd_start)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(Self {
            header,
            segments: HashMap::new(),
            legacy_records: records,
            is_spk: false,
            kernel_id: None,
        })
    }

    /// Create a reader from pre-built header and records (for testing).
    pub fn with_records(header: De440Header, records: Vec<De440Record>) -> Self {
        Self {
            header,
            segments: HashMap::new(),
            legacy_records: records,
            is_spk: false,
            kernel_id: None,
        }
    }

    /// Access the parsed header.
    pub fn header(&self) -> &De440Header {
        &self.header
    }

    /// Check whether any data is loaded (either SPK segments or legacy records).
    pub fn has_data(&self) -> bool {
        !self.segments.is_empty() || !self.legacy_records.is_empty()
    }

    /// Whether this reader was constructed from a real NAIF DAF/SPK binary
    /// (`from_file`). Returns `false` for the synthetic/legacy `with_records`
    /// path. Metadata that asserts genuine DE440 provenance (engine name,
    /// quoted accuracy figure) must be gated on this, NOT on [`has_data`], so
    /// that test fixtures built from `with_records` are not advertised as DE440.
    pub fn is_spk(&self) -> bool {
        self.is_spk
    }

    /// The JPL ephemeris label ("DE440", "DE441", …) detected from the kernel's
    /// DAF comment area, or `None` if it could not be confirmed (synthetic test
    /// kernel, comment-stripped file, or a non-DE SPK). Provenance claims such
    /// as "DE440" MUST be gated on this, not on [`is_spk`](Self::is_spk).
    pub fn kernel_id(&self) -> Option<&str> {
        self.kernel_id.as_deref()
    }

    /// Count the total number of loaded SPK segments.
    pub fn segment_count(&self) -> usize {
        self.segments.values().map(|v| v.len()).sum()
    }

    /// List all (target, center) pairs that have loaded segments.
    pub fn available_bodies(&self) -> Vec<(i32, i32)> {
        self.segments.keys().copied().collect()
    }

    // ----- SPK-based position computation -----

    /// Find the loaded segment covering the given epoch for a (target, center) pair.
    ///
    /// SPK priority is "last-loaded segment wins": when more than one segment for
    /// the same body brackets the requested epoch (e.g. a later patch segment that
    /// supersedes an earlier one over an overlapping interval), the one that
    /// appeared LATER in the file must be used. We therefore select the bracketing
    /// segment with the greatest `file_order` rather than relying on `start_sec`
    /// ordering — a replacement segment can have an earlier start than the segment
    /// it overrides, so a start-sorted lookup would silently pick the stale one.
    fn find_segment(&self, target: i32, center: i32, t_sec: f64) -> Option<&LoadedSegment> {
        let segs = self.segments.get(&(target, center))?;
        segs.iter()
            .filter(|s| t_sec >= s.descriptor.start_sec && t_sec <= s.descriptor.end_sec)
            .max_by_key(|s| s.descriptor.file_order)
    }

    /// Compute position in km for a NAIF body pair at JD TDB.
    ///
    /// `target` and `center` are NAIF integer body IDs.
    /// Returns (x, y, z) in km in the ICRF (J2000 equatorial) frame.
    pub fn position_at(
        &self,
        target: i32,
        center: i32,
        jd_tdb: f64,
    ) -> Result<(f64, f64, f64), EphemerisError> {
        let t_sec = (jd_tdb - J2000_JD) * SECONDS_PER_DAY;
        let seg = self
            .find_segment(target, center, t_sec)
            .ok_or(EphemerisError::EpochOutOfRange(jd_tdb))?;
        seg.position_km(t_sec)
    }

    // ----- Legacy position computation (for backward compatibility) -----

    /// Find the legacy record covering a given JD (TDB).
    fn find_legacy_record(&self, jd_tdb: f64) -> Option<&De440Record> {
        let idx = self
            .legacy_records
            .partition_point(|r| r.jd_start <= jd_tdb);
        if idx == 0 {
            return None;
        }
        let record = &self.legacy_records[idx - 1];
        if jd_tdb >= record.jd_start && jd_tdb <= record.jd_end {
            Some(record)
        } else {
            None
        }
    }

    /// Compute the Cartesian position (km, ICRF) of a DE440 target at JD (TDB).
    pub(crate) fn position_km(
        &self,
        target: De440Target,
        jd_tdb: f64,
    ) -> Result<(f64, f64, f64), EphemerisError> {
        // If we have SPK data, use the NAIF segment path.
        if self.is_spk {
            let (naif_target, naif_center) = target.to_naif_pair();
            return self.position_at(naif_target.0, naif_center.0, jd_tdb);
        }

        // Otherwise use the legacy record path.
        let record = self
            .find_legacy_record(jd_tdb)
            .ok_or(EphemerisError::EpochOutOfRange(jd_tdb))?;

        let ipt = &self.header.ipt[target as usize];
        let num_components = target.num_components();

        if num_components < 3 {
            return Err(EphemerisError::ComputationFailed(format!(
                "{target:?} is not a position target (only {num_components} components)"
            )));
        }

        let block_span = record.jd_end - record.jd_start;
        let sub_span = block_span / ipt.num_sub_intervals as f64;
        let dt = jd_tdb - record.jd_start;
        let sub_idx = ((dt / sub_span) as usize).min(ipt.num_sub_intervals - 1);

        let sub_start = record.jd_start + sub_idx as f64 * sub_span;
        let t_norm = (2.0 * (jd_tdb - sub_start) / sub_span - 1.0).clamp(-1.0, 1.0);

        let x_coeffs = record
            .body_coeffs(ipt, num_components, sub_idx, 0)
            .ok_or_else(|| {
                EphemerisError::ComputationFailed(format!(
                    "no X coefficients for {target:?} in sub-interval {sub_idx}"
                ))
            })?;
        let y_coeffs = record
            .body_coeffs(ipt, num_components, sub_idx, 1)
            .ok_or_else(|| {
                EphemerisError::ComputationFailed(format!(
                    "no Y coefficients for {target:?} in sub-interval {sub_idx}"
                ))
            })?;
        let z_coeffs = record
            .body_coeffs(ipt, num_components, sub_idx, 2)
            .ok_or_else(|| {
                EphemerisError::ComputationFailed(format!(
                    "no Z coefficients for {target:?} in sub-interval {sub_idx}"
                ))
            })?;

        let x = chebyshev_compute(x_coeffs, t_norm);
        let y = chebyshev_compute(y_coeffs, t_norm);
        let z = chebyshev_compute(z_coeffs, t_norm);

        Ok((x, y, z))
    }

    /// Compute the Cartesian position (AU, ICRF) of a DE440 target.
    pub(crate) fn position_au(
        &self,
        target: De440Target,
        jd_tdb: f64,
    ) -> Result<CartesianPosition, EphemerisError> {
        let (x, y, z) = self.position_km(target, jd_tdb)?;
        let au = self.header.au_km;
        Ok(CartesianPosition {
            x: x / au,
            y: y / au,
            z: z / au,
        })
    }

    /// Compute geocentric position of a body by subtracting Earth's position.
    pub fn geocentric_position_au(
        &self,
        body: Body,
        jd_tdb: f64,
    ) -> Result<CartesianPosition, EphemerisError> {
        let _target = De440Target::from_body(body).ok_or(EphemerisError::BodyNotAvailable(body))?;

        match body {
            Body::Moon => {
                // Geocentric Moon from the SPK Moon(301)-relative-to-EMB(3) segment.
                //
                // The Earth–Moon barycentre is EMB = (M_e·Earth + M_m·Moon)/(M_e+M_m)
                // with mass ratio EMRAT = M_e/M_m. The SPK gives the Moon's position
                // relative to the EMB, Moon_emb = Moon − EMB. Substituting the
                // barycentre definition:
                //   Moon − EMB = (M_e/(M_e+M_m))·(Moon − Earth) = (EMRAT/(1+EMRAT))·Moon_geo,
                // where Moon_geo = Moon − Earth is the geocentric vector we want. Hence
                //   Moon_geo = Moon_emb · (1 + EMRAT) / EMRAT.
                if self.is_spk {
                    let (mx, my, mz) = self.position_at(301, 3, jd_tdb)?;
                    let factor = (1.0 + EMRAT) / EMRAT;
                    let au = self.header.au_km;
                    return Ok(CartesianPosition {
                        x: mx * factor / au,
                        y: my * factor / au,
                        z: mz * factor / au,
                    });
                }
                self.position_au(De440Target::MoonGeo, jd_tdb)
            }
            Body::Earth => Ok(CartesianPosition {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }),
            Body::Sun => {
                let sun = self.position_au(De440Target::Sun, jd_tdb)?;
                let earth = self.earth_ssb_au(jd_tdb)?;
                Ok(CartesianPosition {
                    x: sun.x - earth.x,
                    y: sun.y - earth.y,
                    z: sun.z - earth.z,
                })
            }
            _ => {
                let target =
                    De440Target::from_body(body).ok_or(EphemerisError::BodyNotAvailable(body))?;
                let planet = self.position_au(target, jd_tdb)?;
                let earth = self.earth_ssb_au(jd_tdb)?;
                Ok(CartesianPosition {
                    x: planet.x - earth.x,
                    y: planet.y - earth.y,
                    z: planet.z - earth.z,
                })
            }
        }
    }

    /// Geocentric position with the TARGET evaluated at `body_jd` and EARTH at
    /// `earth_jd` (both TDB). This is the building block for the apparent-place
    /// light-time correction: a planet/Sun is seen where it was when its light
    /// left (`earth_jd - τ`), while the observer (Earth) stays at the observation
    /// epoch `earth_jd`.
    ///
    /// The Moon is the exception. The reader returns its (Moon − Earth) vector at
    /// a SINGLE epoch; retarding that to `body_jd` would also retard Earth and
    /// introduce Earth's ~20.5" motion over the ~1.3 s lunar light-time — far
    /// larger than the Moon's own ~0.7" motion. So the Moon is evaluated
    /// GEOMETRICALLY at the OBSERVATION epoch (`earth_jd`), the same un-retarded
    /// treatment the VSOP87 Moon path uses; the light-time loop is a no-op for it.
    pub fn geocentric_position_au_split(
        &self,
        body: Body,
        body_jd: f64,
        earth_jd: f64,
    ) -> Result<CartesianPosition, EphemerisError> {
        match body {
            Body::Moon | Body::Earth => self.geocentric_position_au(body, earth_jd),
            Body::Sun => {
                let sun = self.position_au(De440Target::Sun, body_jd)?;
                let earth = self.earth_ssb_au(earth_jd)?;
                Ok(CartesianPosition {
                    x: sun.x - earth.x,
                    y: sun.y - earth.y,
                    z: sun.z - earth.z,
                })
            }
            _ => {
                let target =
                    De440Target::from_body(body).ok_or(EphemerisError::BodyNotAvailable(body))?;
                let planet = self.position_au(target, body_jd)?;
                let earth = self.earth_ssb_au(earth_jd)?;
                Ok(CartesianPosition {
                    x: planet.x - earth.x,
                    y: planet.y - earth.y,
                    z: planet.z - earth.z,
                })
            }
        }
    }

    /// Compute heliocentric position of a body by subtracting the Sun's
    /// SSB position from the planet's SSB position.
    ///
    /// DE440 stores planet barycenters relative to the Solar System Barycenter
    /// (SSB), NOT relative to the Sun. To get true heliocentric coordinates:
    ///   heliocentric = planet_SSB - Sun_SSB
    pub fn heliocentric_position_au(
        &self,
        body: Body,
        jd_tdb: f64,
    ) -> Result<CartesianPosition, EphemerisError> {
        let _target = De440Target::from_body(body).ok_or(EphemerisError::BodyNotAvailable(body))?;

        match body {
            Body::Sun => {
                // The Sun's heliocentric position is the origin by definition.
                Ok(CartesianPosition {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                })
            }
            Body::Moon => {
                // Moon heliocentric = Moon_geocentric + Earth_heliocentric
                // = Moon_geocentric + (Earth_SSB - Sun_SSB)
                let moon_geo = self.geocentric_position_au(Body::Moon, jd_tdb)?;
                let earth = self.earth_ssb_au(jd_tdb)?;
                let sun = self.position_au(De440Target::Sun, jd_tdb)?;
                Ok(CartesianPosition {
                    x: moon_geo.x + earth.x - sun.x,
                    y: moon_geo.y + earth.y - sun.y,
                    z: moon_geo.z + earth.z - sun.z,
                })
            }
            Body::Earth => {
                // Earth heliocentric = Earth_SSB - Sun_SSB
                let earth = self.earth_ssb_au(jd_tdb)?;
                let sun = self.position_au(De440Target::Sun, jd_tdb)?;
                Ok(CartesianPosition {
                    x: earth.x - sun.x,
                    y: earth.y - sun.y,
                    z: earth.z - sun.z,
                })
            }
            _ => {
                // General planet: planet_SSB - Sun_SSB
                let target =
                    De440Target::from_body(body).ok_or(EphemerisError::BodyNotAvailable(body))?;
                let planet = self.position_au(target, jd_tdb)?;
                let sun = self.position_au(De440Target::Sun, jd_tdb)?;
                Ok(CartesianPosition {
                    x: planet.x - sun.x,
                    y: planet.y - sun.y,
                    z: planet.z - sun.z,
                })
            }
        }
    }

    /// Compute Earth's position relative to the solar system barycenter.
    ///
    /// Earth = EMB - Moon_emb / (1 + EMRAT)
    fn earth_ssb_au(&self, jd_tdb: f64) -> Result<CartesianPosition, EphemerisError> {
        if self.is_spk {
            let (ex, ey, ez) = self.position_at(3, 0, jd_tdb)?;
            let (mx, my, mz) = self.position_at(301, 3, jd_tdb)?;
            let factor = 1.0 / (1.0 + EMRAT);
            let au = self.header.au_km;
            return Ok(CartesianPosition {
                x: (ex - mx * factor) / au,
                y: (ey - my * factor) / au,
                z: (ez - mz * factor) / au,
            });
        }

        let emb = self.position_au(De440Target::EarthMoonBary, jd_tdb)?;
        let moon = self.position_au(De440Target::MoonGeo, jd_tdb)?;
        let factor = 1.0 / (1.0 + self.header.emrat);
        Ok(CartesianPosition {
            x: emb.x - moon.x * factor,
            y: emb.y - moon.y * factor,
            z: emb.z - moon.z * factor,
        })
    }
}

// Alias for backward compatibility: `find_record` used in old tests.
impl De440Reader {
    #[cfg(test)]
    fn find_record(&self, jd_tdb: f64) -> Option<&De440Record> {
        self.find_legacy_record(jd_tdb)
    }
}

// ---------------------------------------------------------------------------
// Accuracy tier — honest capability reporting
// ---------------------------------------------------------------------------

/// Which engine is *actually* serving positions, so callers and documentation
/// can state the correct accuracy figure instead of guessing.
///
/// This makes the with-kernel vs analytical-fallback distinction explicit:
/// a [`De440Provider`] constructed with no (or a failed) kernel load still
/// satisfies the [`EphemerisProvider`] trait, but it is *not* DE440-grade and
/// must not be described as such.
///
/// The arcsecond figure on each tier is the provider's own worst-case
/// apparent-longitude figure ([`EphemerisProvider::accuracy_arcsec`]) — the same
/// number, surfaced as a typed tier rather than a bare float — NOT an
/// independent fabricated claim. The DE440 figure is the conservative bound the
/// provider already reports and is gated on confirmed `DE440` provenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccuracyTier {
    /// A real, DAF-comment-confirmed `DE440` kernel is loaded and serving
    /// positions for the physical bodies it covers. Highest precision.
    Kernel,
    /// A parsed SPK kernel whose provenance could NOT be confirmed as DE440
    /// (synthetic fixture, comment-stripped, or non-DE `.bsp`). Treated as
    /// analytical-grade for accuracy reporting — we never advertise DE440
    /// precision for a kernel we cannot verify.
    UnconfirmedKernel,
    /// No kernel loaded (or load failed): positions come from the analytical
    /// VSOP87 fallback. Wider residual; see the analytical provider's docs.
    AnalyticalFallback,
}

impl AccuracyTier {
    /// `true` only for [`AccuracyTier::Kernel`] — a confirmed DE440 binary is
    /// serving positions. Callers should gate any "JPL-grade" / sub-arcsecond
    /// wording on this.
    pub fn is_kernel_backed(self) -> bool {
        matches!(self, AccuracyTier::Kernel)
    }

    /// Short human-readable label for docs / API responses.
    pub fn label(self) -> &'static str {
        match self {
            AccuracyTier::Kernel => "JPL DE440 kernel",
            AccuracyTier::UnconfirmedKernel => "unconfirmed SPK kernel (analytical-grade)",
            AccuracyTier::AnalyticalFallback => "VSOP87 analytical fallback",
        }
    }
}

// ---------------------------------------------------------------------------
// De440Provider -- implements EphemerisProvider
// ---------------------------------------------------------------------------

/// Ephemeris provider backed by JPL DE440 data.
///
/// Falls back to VSOP87 for any body or epoch not covered by the loaded
/// DE440 data, or when no data file has been loaded.
pub struct De440Provider {
    reader: De440Reader,
    fallback: Vsop87Provider,
}

impl De440Provider {
    /// Create a provider that has no DE440 data loaded (pure fallback mode).
    pub fn fallback_only() -> Self {
        Self {
            reader: De440Reader::empty(),
            fallback: Vsop87Provider::new(),
        }
    }

    /// Create a provider from a pre-loaded reader.
    pub fn with_reader(reader: De440Reader) -> Self {
        Self {
            reader,
            fallback: Vsop87Provider::new(),
        }
    }

    /// Attempt to create a provider by loading a DE440 binary file, degrading
    /// to the analytical fallback if the load fails.
    ///
    /// The failure is NOT silent: a load error is logged to stderr so an
    /// operator who *expected* a kernel can see the provider quietly dropped to
    /// VSOP87 accuracy. Callers that must distinguish "kernel loaded" from
    /// "fell back" programmatically should use [`Self::try_from_file_strict`]
    /// (returns the error) or check [`Self::accuracy_tier`] after construction.
    pub fn try_from_file(path: &Path) -> Self {
        match De440Reader::from_file(path) {
            Ok(reader) => Self::with_reader(reader),
            Err(e) => {
                eprintln!(
                    "[xalen-ephem] DE440 load FAILED for {}: {e} — \
                     degrading to VSOP87 analytical fallback (NOT DE440-grade). \
                     Use try_from_file_strict to make this a hard error.",
                    path.display()
                );
                Self::fallback_only()
            }
        }
    }

    /// Load a DE440 binary file as the provider, returning a LOUD error if the
    /// kernel cannot be read or parsed.
    ///
    /// Use this (rather than [`Self::try_from_file`]) when a kernel is
    /// *expected*: it never silently degrades to VSOP87 while still presenting
    /// as a DE440-capable provider. The error surfaces the exact failure
    /// (missing file, truncated/garbage DAF, unverified provenance) so the
    /// caller decides whether to abort or fall back explicitly.
    pub fn try_from_file_strict(path: &Path) -> Result<Self, EphemerisError> {
        let reader = De440Reader::from_file(path)?;
        Ok(Self::with_reader(reader))
    }

    /// Build a kernel-backed provider using the auto-provisioned DE440 kernel.
    ///
    /// Requires the `kernel-autodownload` feature. On first use this fetches the
    /// public NASA NAIF `de440s.bsp` kernel (~32 MB) into the per-OS cache
    /// directory and verifies it (structural DE440 provenance, plus an optional
    /// SHA-256 when one is configured); subsequent calls reuse the cached copy
    /// with no network access. The returned provider serves the apparent Moon —
    /// and every body the kernel covers — at sub-arcsecond accuracy, with no
    /// manual kernel handling.
    ///
    /// Returns a LOUD error if provisioning or loading fails (e.g. no network on
    /// first run, or a cache directory that cannot be created). Callers that
    /// prefer to degrade silently to the analytical fallback can match the error
    /// and call [`Self::fallback_only`].
    ///
    /// See [`crate::kernel_cache`] for cache-location and integrity controls.
    #[cfg(feature = "kernel-autodownload")]
    pub fn from_auto_cache() -> Result<Self, EphemerisError> {
        let path = crate::kernel_cache::ensure_de440s_kernel()?;
        Self::try_from_file_strict(&path)
    }

    /// Whether DE440 data is actually loaded.
    pub fn has_de440_data(&self) -> bool {
        self.reader.has_data()
    }

    /// The accuracy tier this provider is *actually* operating at right now.
    ///
    /// This is the honest with-kernel vs analytical-fallback accessor: it lets
    /// callers and documentation state the correct accuracy figure without
    /// assuming a kernel loaded. Gated on confirmed `DE440` provenance, mirroring
    /// [`Self::is_de440_loaded`] and [`EphemerisProvider::accuracy_arcsec`].
    pub fn accuracy_tier(&self) -> AccuracyTier {
        if self.reader.kernel_id() == Some("DE440") {
            AccuracyTier::Kernel
        } else if self.reader.is_spk() {
            AccuracyTier::UnconfirmedKernel
        } else {
            AccuracyTier::AnalyticalFallback
        }
    }

    /// Check whether this provider has a real, CONFIRMED DE440 binary loaded,
    /// as opposed to falling back to VSOP87 accuracy.
    ///
    /// Callers can use this to inform users which engine is active. Both the
    /// DE440 and the VSOP87 fallback paths apply the same apparent-place
    /// correction chain (precession + nutation + annual aberration) and are at
    /// the ~1 arcsecond level; DE440's advantage is raw positional accuracy and
    /// extended validity range rather than a different correction model.
    ///
    /// Gated on the DAF comment area confirming `DE440` provenance, NOT on
    /// `is_spk()` alone: a synthetic test kernel or a non-DE `.bsp` parses as an
    /// SPK but is not DE440 and must not report itself as one.
    pub fn is_de440_loaded(&self) -> bool {
        self.reader.kernel_id() == Some("DE440")
    }

    /// Convert a DE440 Cartesian position (J2000/ICRF equatorial) to
    /// ecliptic coordinates suitable for astrological computation.
    fn cartesian_to_ecliptic_of_date(
        &self,
        cart: &CartesianPosition,
        jd_tt: JdTT,
    ) -> EclipticPosition {
        // DE440 vectors are ICRF/GCRS equatorial. The rigorous IAU 2006/P03
        // precession matrix `pmat06` rotates an ICRF/GCRS equatorial vector all
        // the way to the mean equatorial frame of date (the ~23 mas ICRS frame
        // bias is folded in — hence the BIAS-INCLUSIVE matrix here, unlike the
        // dynamical-J2000 VSOP/ELP path which uses the bias-free matrix).
        let t = jd_tt.julian_centuries_from_j2000();
        let prec = xalen_coords::precession_bias_matrix_iau2006(t);
        let eq_date = xalen_coords::rotate3(prec, [cart.x, cart.y, cart.z]);

        // Mean equatorial-of-date → mean ecliptic-of-date (rotate about x by the
        // mean obliquity of date), then IAU 2000B nutation in longitude → true
        // (apparent) ecliptic-of-date. Annual aberration is geocentric-only and is
        // applied by the caller (geocentric_ecliptic), not here, so the
        // heliocentric path is not wrongly aberrated.
        let eps = xalen_coords::mean_obliquity(t);
        let cos_e = eps.cos();
        let sin_e = eps.sin();
        let ecl_cart = CartesianPosition {
            x: eq_date[0],
            y: eq_date[1] * cos_e + eq_date[2] * sin_e,
            z: -eq_date[1] * sin_e + eq_date[2] * cos_e,
        };
        let mut pos = xalen_coords::cartesian_to_ecliptic(&ecl_cart);

        pos.longitude += xalen_coords::nutation_2000b(t).delta_psi;

        pos.normalize()
    }

    /// Apparent geocentric ecliptic Moon (ecliptic of date) from the DE440
    /// kernel, WITHOUT the annual aberration that planets/Sun receive.
    ///
    /// The kernel gives the TRUE geometric geocentric Moon. The Moon shares
    /// Earth's heliocentric velocity, so the full annual aberration term
    /// (κ = 20.49552″) does NOT apply to it — applying it injected ~11–20″ of
    /// spurious longitude. The only displacement is the Moon's own GEOCENTRIC
    /// light-time / planetary aberration (~0.7″): the Moon is seen where it was
    /// τ = ρ/c seconds ago. That is applied here as a longitude/latitude
    /// retardation using the Moon's geocentric rate (central finite difference
    /// of the kernel's geocentric vector — Earth held fixed, so no Earth-motion
    /// contamination, matching the VSOP87 analytical Moon path).
    ///
    /// Expected residual vs Horizons/Swiss apparent place: sub-arcsecond (the
    /// prior full-annual-aberration path was ~11″).
    fn apparent_moon_de440(
        &self,
        tdb_val: f64,
        jd_tt: JdTT,
    ) -> Result<EclipticPosition, EphemerisError> {
        // Geometric geocentric Moon → apparent ecliptic-of-date (precess +
        // nutation, no aberration) at the observation epoch and at ±h to form
        // the geocentric rate.
        const H_DAYS: f64 = 0.01; // ~14.4 min; well inside DE440 smoothness
        // Convert the ±h offsets in TDB to the corresponding jd_tt offsets. The
        // TDB-TT difference is < 2 ms and effectively constant over 0.02 day, so
        // jd_tt simply shifts by the same ±h as tdb_val.
        let mid = self.geocentric_moon_apparent_no_aber(tdb_val, jd_tt)?;
        let before =
            self.geocentric_moon_apparent_no_aber(tdb_val - H_DAYS, JdTT(jd_tt.as_f64() - H_DAYS))?;
        let after =
            self.geocentric_moon_apparent_no_aber(tdb_val + H_DAYS, JdTT(jd_tt.as_f64() + H_DAYS))?;

        let mut dlon = after.longitude - before.longitude;
        if dlon > std::f64::consts::PI {
            dlon -= std::f64::consts::TAU;
        } else if dlon < -std::f64::consts::PI {
            dlon += std::f64::consts::TAU;
        }
        let dlon_dt = dlon / (2.0 * H_DAYS);
        let dlat_dt = (after.latitude - before.latitude) / (2.0 * H_DAYS);

        let tau = mid.distance / LIGHT_SPEED_AU_PER_DAY; // days

        Ok(EclipticPosition {
            longitude: mid.longitude - tau * dlon_dt,
            latitude: mid.latitude - tau * dlat_dt,
            distance: mid.distance,
        })
    }

    /// Geometric geocentric Moon in apparent ecliptic-of-date coordinates
    /// (precession + nutation), WITHOUT any aberration/light-time term. Helper
    /// for `apparent_moon_de440`'s finite-difference rate.
    fn geocentric_moon_apparent_no_aber(
        &self,
        tdb_val: f64,
        jd_tt: JdTT,
    ) -> Result<EclipticPosition, EphemerisError> {
        let cart = self.reader.geocentric_position_au(Body::Moon, tdb_val)?;
        Ok(self.cartesian_to_ecliptic_of_date(&cart, jd_tt))
    }
}

impl EphemerisProvider for De440Provider {
    fn heliocentric_ecliptic(
        &self,
        body: Body,
        jd_tt: JdTT,
    ) -> Result<EclipticPosition, EphemerisError> {
        if self.reader.has_data() {
            let jd_tdb = jd_tt.to_tdb();
            let tdb_val = jd_tdb.as_f64();

            if tdb_val >= self.reader.header().jd_start
                && tdb_val <= self.reader.header().jd_end
                && De440Target::from_body(body).is_some()
            {
                match self.reader.heliocentric_position_au(body, tdb_val) {
                    Ok(cart) => {
                        return Ok(self.cartesian_to_ecliptic_of_date(&cart, jd_tt));
                    }
                    // Unmapped body, or a genuine coverage GAP (epoch outside
                    // THIS body's own segment) → analytical fallback. Any
                    // other failure (corrupt segment, ComputationFailed) is a
                    // real error and must surface, not silently degrade to
                    // VSOP87 while still being reported as the DE440 engine.
                    Err(EphemerisError::BodyNotAvailable(_)) => {}
                    Err(EphemerisError::EpochOutOfRange(_)) => {}
                    Err(e) => return Err(e),
                }
            }
        }

        self.fallback.heliocentric_ecliptic(body, jd_tt)
    }

    fn geocentric_ecliptic(
        &self,
        body: Body,
        jd_tt: JdTT,
    ) -> Result<EclipticPosition, EphemerisError> {
        if self.reader.has_data() {
            let jd_tdb = jd_tt.to_tdb();
            let tdb_val = jd_tdb.as_f64();

            if tdb_val >= self.reader.header().jd_start
                && tdb_val <= self.reader.header().jd_end
                && De440Target::from_body(body).is_some()
            {
                // The Moon is handled separately: it must NOT receive the full
                // annual aberration (κ = 20.49552″) that planets/Sun get, because
                // the geocentric Moon shares Earth's heliocentric velocity. It
                // gets only its small GEOCENTRIC light-time / planetary aberration
                // (~0.7″). See `apparent_moon_de440`. On a genuine coverage gap
                // (e.g. the ±h finite-difference samples straddle the kernel
                // boundary) drop to the analytical fallback — which also applies
                // the correct Moon reduction — rather than to the planet path
                // below (that would wrongly re-apply annual aberration).
                if body == Body::Moon {
                    match self.apparent_moon_de440(tdb_val, jd_tt) {
                        Ok(pos) => return Ok(pos.normalize()),
                        Err(EphemerisError::BodyNotAvailable(_))
                        | Err(EphemerisError::EpochOutOfRange(_)) => {
                            return self.fallback.geocentric_ecliptic(body, jd_tt);
                        }
                        Err(e) => return Err(e),
                    }
                }
                match self.reader.geocentric_position_au(body, tdb_val) {
                    Ok(cart0) => {
                        // Apparent place = light-time retardation + precession +
                        // nutation (ecliptic-of-date) + annual aberration — the
                        // SAME full chain the VSOP87 path uses (so the two engines
                        // agree). The body is seen where it was when its light
                        // left (tdb - τ); Earth (observer) stays at tdb. Two
                        // iterations converge well inside a milliarcsecond.
                        let mut cart = cart0;
                        for _ in 0..2 {
                            let dist = (cart.x * cart.x + cart.y * cart.y + cart.z * cart.z).sqrt();
                            let tau = dist / LIGHT_SPEED_AU_PER_DAY; // days
                            match self.reader.geocentric_position_au_split(
                                body,
                                tdb_val - tau,
                                tdb_val,
                            ) {
                                Ok(c) => cart = c,
                                Err(EphemerisError::BodyNotAvailable(_))
                                | Err(EphemerisError::EpochOutOfRange(_)) => break,
                                Err(e) => return Err(e),
                            }
                        }
                        let pos = self.cartesian_to_ecliptic_of_date(&cart, jd_tt);
                        return Ok(crate::vsop::aberration_correction(pos, jd_tt).normalize());
                    }
                    Err(EphemerisError::BodyNotAvailable(_)) => {}
                    // A genuine coverage GAP for THIS body (the epoch falls
                    // outside its own segment even though the overall header
                    // range brackets it) drops through to the VSOP87
                    // fallback. Any other failure (corrupt segment data,
                    // ComputationFailed) is a real error and must surface
                    // rather than silently degrading to VSOP87 while still
                    // reporting the DE440 engine. Mirrors heliocentric above.
                    Err(EphemerisError::EpochOutOfRange(_)) => {}
                    Err(e) => return Err(e),
                }
            }
        }

        self.fallback.geocentric_ecliptic(body, jd_tt)
    }

    fn coverage(&self) -> (f64, f64) {
        if self.reader.has_data() {
            (self.reader.header().jd_start, self.reader.header().jd_end)
        } else {
            self.fallback.coverage()
        }
    }

    fn accuracy_arcsec(&self) -> f64 {
        // Quote the high-precision figure ONLY for a kernel whose DAF comment
        // area confirms DE440 provenance — not for any parsed SPK. A synthetic or
        // unconfirmed kernel reports the analytical fallback figure so we never
        // advertise precision we cannot stand behind.
        if self.reader.kernel_id() == Some("DE440") {
            DE440_APPARENT_WORST_ARCSEC
        } else {
            self.fallback.accuracy_arcsec()
        }
    }

    fn name(&self) -> &str {
        // Only advertise the "DE440" engine name when the DAF comment area
        // actually confirms DE440 provenance. Any other parsed SPK (a non-DE
        // kernel, a comment-stripped file, or a synthetic test fixture) reports
        // the honest generic "JPL SPK kernel" label; no kernel at all reports
        // the VSOP87 fallback.
        match self.reader.kernel_id() {
            Some("DE440") => "JPL DE440 (apparent place)",
            _ if self.reader.is_spk() => "JPL SPK kernel (apparent place)",
            _ => "JPL DE440 [fallback: VSOP87] (Tier 0)",
        }
    }
}

// ---------------------------------------------------------------------------
// Almanac extension: with_de440
// ---------------------------------------------------------------------------

impl crate::almanac::Almanac {
    /// Load a DE440 SPK file and insert it as the highest-priority provider.
    ///
    /// If the file cannot be loaded, the almanac is returned unchanged
    /// (VSOP87 fallback continues to work) — but the failure is LOGGED to
    /// stderr rather than swallowed silently, so an operator who expected a
    /// kernel can see that the almanac is running at analytical accuracy, not
    /// DE440 accuracy. For a hard error instead of a silent degrade, use
    /// [`Almanac::with_de440_strict`].
    pub fn with_de440(self, path: &Path) -> Self {
        match De440Reader::from_file(path) {
            Ok(reader) => {
                let provider = De440Provider::with_reader(reader);
                self.with_provider(std::sync::Arc::new(provider))
            }
            Err(e) => {
                eprintln!(
                    "[xalen-ephem] Almanac::with_de440 load FAILED for {}: {e} — \
                     almanac stays on VSOP87 analytical fallback (NOT DE440-grade). \
                     Use with_de440_strict to make this a hard error.",
                    path.display()
                );
                self
            }
        }
    }

    /// Load a DE440 SPK file as the highest-priority provider, returning a LOUD
    /// error if the kernel cannot be read or parsed.
    ///
    /// Use this when a kernel is *expected*: it never silently leaves the
    /// almanac at VSOP87 accuracy while appearing DE440-capable. On error the
    /// caller decides whether to abort or fall back explicitly.
    pub fn with_de440_strict(self, path: &Path) -> Result<Self, EphemerisError> {
        let reader = De440Reader::from_file(path)?;
        let provider = De440Provider::with_reader(reader);
        Ok(self.with_provider(std::sync::Arc::new(provider)))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;
    use std::sync::Arc;
    use xalen_time::J2000_JD as TIME_J2000;

    // -----------------------------------------------------------------------
    // Chebyshev computation tests
    // -----------------------------------------------------------------------

    #[test]
    fn chebyshev_constant() {
        assert_eq!(chebyshev_compute(&[42.0], 0.0), 42.0);
        assert_eq!(chebyshev_compute(&[42.0], 1.0), 42.0);
        assert_eq!(chebyshev_compute(&[42.0], -1.0), 42.0);
    }

    #[test]
    fn chebyshev_linear() {
        let coeffs = [3.0, 5.0];
        assert!((chebyshev_compute(&coeffs, 0.0) - 3.0).abs() < 1e-15);
        assert!((chebyshev_compute(&coeffs, 1.0) - 8.0).abs() < 1e-15);
        assert!((chebyshev_compute(&coeffs, -1.0) - (-2.0)).abs() < 1e-15);
        assert!((chebyshev_compute(&coeffs, 0.5) - 5.5).abs() < 1e-15);
    }

    #[test]
    fn chebyshev_quadratic() {
        let coeffs = [0.0, 0.0, 1.0];
        assert!((chebyshev_compute(&coeffs, 0.0) - (-1.0)).abs() < 1e-15);
        assert!((chebyshev_compute(&coeffs, 1.0) - 1.0).abs() < 1e-15);
        assert!((chebyshev_compute(&coeffs, -1.0) - 1.0).abs() < 1e-15);
        assert!((chebyshev_compute(&coeffs, 0.5) - (-0.5)).abs() < 1e-15);
    }

    #[test]
    fn chebyshev_cubic() {
        let coeffs = [0.0, 0.0, 0.0, 1.0];
        assert!((chebyshev_compute(&coeffs, 0.0) - 0.0).abs() < 1e-15);
        assert!((chebyshev_compute(&coeffs, 1.0) - 1.0).abs() < 1e-15);
        assert!((chebyshev_compute(&coeffs, -1.0) - (-1.0)).abs() < 1e-15);
        assert!((chebyshev_compute(&coeffs, 0.5) - (-1.0)).abs() < 1e-14);
    }

    #[test]
    fn chebyshev_mixed_polynomial() {
        let coeffs = [2.0, 3.0, -1.0];
        assert!((chebyshev_compute(&coeffs, 0.0) - 3.0).abs() < 1e-14);
        assert!((chebyshev_compute(&coeffs, 1.0) - 4.0).abs() < 1e-14);
        assert!((chebyshev_compute(&coeffs, -1.0) - (-2.0)).abs() < 1e-14);
        assert!((chebyshev_compute(&coeffs, 0.25) - 3.625).abs() < 1e-14);
    }

    #[test]
    fn chebyshev_empty() {
        assert_eq!(chebyshev_compute(&[], 0.5), 0.0);
    }

    #[test]
    fn chebyshev_many_terms() {
        let coeffs = vec![1.0; 8];
        assert!((chebyshev_compute(&coeffs, 1.0) - 8.0).abs() < 1e-12);
        assert!((chebyshev_compute(&coeffs, -1.0) - 0.0).abs() < 1e-12);
    }

    #[test]
    fn chebyshev_cos_identity() {
        let theta = PI / 3.0;
        let x = theta.cos();
        let coeffs = [0.0, 0.0, 0.0, 0.0, 0.0, 1.0];
        let result = chebyshev_compute(&coeffs, x);
        let expected = (5.0 * theta).cos();
        assert!(
            (result - expected).abs() < 1e-13,
            "T_5(cos(PI/3)) should be cos(5*PI/3) = {expected}, got {result}"
        );
    }

    #[test]
    fn chebyshev_orthogonality_numerical() {
        let n_points = 10000;
        let mut integral = 0.0;
        for i in 0..n_points {
            let theta = PI * (i as f64 + 0.5) / n_points as f64;
            let x = theta.cos();
            let t2 = chebyshev_compute(&[0.0, 0.0, 1.0], x);
            let t3 = chebyshev_compute(&[0.0, 0.0, 0.0, 1.0], x);
            integral += t2 * t3;
        }
        integral *= PI / n_points as f64;
        assert!(
            integral.abs() < 1e-10,
            "T_2 and T_3 should be orthogonal, got integral = {integral}"
        );
    }

    // -----------------------------------------------------------------------
    // Chebyshev derivative tests
    // -----------------------------------------------------------------------

    #[test]
    fn derivative_constant_is_zero() {
        assert_eq!(chebyshev_derivative(&[5.0], 0.0), 0.0);
    }

    #[test]
    fn derivative_linear() {
        let coeffs = [3.0, 7.0];
        let d = chebyshev_derivative(&coeffs, 0.5);
        assert!((d - 7.0).abs() < 1e-13, "d/dx(3 + 7x) = 7, got {d}");
    }

    #[test]
    fn derivative_quadratic() {
        let coeffs = [0.0, 0.0, 1.0];
        for &x in &[-1.0, -0.5, 0.0, 0.5, 1.0] {
            let d = chebyshev_derivative(&coeffs, x);
            let expected = 4.0 * x;
            assert!(
                (d - expected).abs() < 1e-12,
                "d/dx T_2({x}) should be {expected}, got {d}"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Header and record structure tests
    // -----------------------------------------------------------------------

    #[test]
    fn de440_header_defaults_valid() {
        let h = De440Header::de440_defaults();
        assert!((h.block_span - 32.0).abs() < 1e-10);
        assert_eq!(h.ipt.len(), 13);
        assert!(h.ipt[0].num_coeffs > 0);
        assert!(h.ipt[0].num_sub_intervals > 0);
        assert!((h.emrat - 81.3).abs() < 0.1);
    }

    #[test]
    fn ipt_words_per_record() {
        let h = De440Header::de440_defaults();
        let merc = &h.ipt[De440Target::Mercury as usize];
        assert_eq!(merc.words_per_record(3), 14 * 3 * 4);
        let nut = &h.ipt[De440Target::Nutations as usize];
        assert_eq!(nut.words_per_record(2), 10 * 2 * 4);
    }

    #[test]
    fn record_body_coeffs_extraction() {
        let header = De440Header::de440_defaults();
        let merc_ipt = &header.ipt[De440Target::Mercury as usize];

        let total_words = merc_ipt.offset - 3 + merc_ipt.words_per_record(3);
        let mut data = vec![0.0_f64; total_words];

        let base = merc_ipt.offset - 3;
        for i in 0..merc_ipt.num_coeffs {
            data[base + i] = (i + 1) as f64;
        }

        let record = De440Record {
            jd_start: 2_287_184.5,
            jd_end: 2_287_184.5 + 32.0,
            coefficients: data,
        };

        let x_coeffs = record.body_coeffs(merc_ipt, 3, 0, 0).unwrap();
        assert_eq!(x_coeffs.len(), merc_ipt.num_coeffs);
        assert_eq!(x_coeffs[0], 1.0);
        assert_eq!(x_coeffs[13], 14.0);
    }

    // -----------------------------------------------------------------------
    // Reader tests
    // -----------------------------------------------------------------------

    #[test]
    fn empty_reader_has_no_data() {
        let r = De440Reader::empty();
        assert!(!r.has_data());
    }

    #[test]
    fn find_record_binary_search() {
        let header = De440Header::de440_defaults();
        let records = vec![
            De440Record {
                jd_start: 2451545.0,
                jd_end: 2451577.0,
                coefficients: vec![],
            },
            De440Record {
                jd_start: 2451577.0,
                jd_end: 2451609.0,
                coefficients: vec![],
            },
            De440Record {
                jd_start: 2451609.0,
                jd_end: 2451641.0,
                coefficients: vec![],
            },
        ];
        let reader = De440Reader::with_records(header, records);

        assert!(reader.find_record(2451560.0).is_some());
        assert_eq!(reader.find_record(2451560.0).unwrap().jd_start, 2451545.0);

        assert!(reader.find_record(2451577.0).is_some());
        let rec = reader.find_record(2451577.0).unwrap();
        assert!(rec.jd_start <= 2451577.0 && rec.jd_end >= 2451577.0);

        assert!(reader.find_record(2451540.0).is_none());
        assert!(reader.find_record(2451650.0).is_none());
    }

    // -----------------------------------------------------------------------
    // Synthetic end-to-end test with known Chebyshev coefficients (legacy path)
    // -----------------------------------------------------------------------

    #[test]
    fn synthetic_mercury_position() {
        let header = De440Header::de440_defaults();
        let merc_ipt = &header.ipt[De440Target::Mercury as usize];

        let total = merc_ipt.offset - 3 + merc_ipt.words_per_record(3);
        let mut data = vec![0.0_f64; total];

        let base = merc_ipt.offset - 3;
        let nc = merc_ipt.num_coeffs;
        data[base] = 1.0;
        data[base + nc] = 2.0;
        data[base + 2 * nc] = 3.0;

        let jd_start = 2451545.0;
        let jd_end = jd_start + 32.0;
        let record = De440Record {
            jd_start,
            jd_end,
            coefficients: data,
        };

        let reader = De440Reader::with_records(header, vec![record]);
        let (x, y, z) = reader
            .position_km(De440Target::Mercury, jd_start + 0.01)
            .unwrap();
        assert!((x - 1.0).abs() < 1e-10, "X should be 1.0, got {x}");
        assert!((y - 2.0).abs() < 1e-10, "Y should be 2.0, got {y}");
        assert!((z - 3.0).abs() < 1e-10, "Z should be 3.0, got {z}");
    }

    #[test]
    fn synthetic_linear_mercury_position() {
        let header = De440Header::de440_defaults();
        let merc_ipt = &header.ipt[De440Target::Mercury as usize];

        let total = merc_ipt.offset - 3 + merc_ipt.words_per_record(3);
        let mut data = vec![0.0_f64; total];

        let base = merc_ipt.offset - 3;
        let nc = merc_ipt.num_coeffs;
        data[base] = 100.0;
        data[base + 1] = 50.0;
        data[base + nc] = 10.0;

        let sub_span = 32.0 / merc_ipt.num_sub_intervals as f64;
        let jd_start = 2451545.0;
        let jd_end = jd_start + 32.0;
        let record = De440Record {
            jd_start,
            jd_end,
            coefficients: data,
        };
        let reader = De440Reader::with_records(header, vec![record]);

        let mid = jd_start + sub_span / 2.0;
        let (x, y, _z) = reader.position_km(De440Target::Mercury, mid).unwrap();
        assert!(
            (x - 100.0).abs() < 1e-8,
            "X at midpoint should be ~100, got {x}"
        );
        assert!(
            (y - 10.0).abs() < 1e-8,
            "Y at midpoint should be ~10, got {y}"
        );
    }

    // -----------------------------------------------------------------------
    // NAIF DAF/SPK synthetic test fixture
    // -----------------------------------------------------------------------

    /// Build a minimal synthetic NAIF DAF/SPK file in memory.
    /// Contains three Type 2 segments:
    ///   1. EMB (target=3, center=0): constant position X=100000, Y=200000, Z=50000 km
    ///   2. Moon (target=301, center=3): constant position X=300, Y=400, Z=100 km
    ///   3. Sun (target=10, center=0): constant position X=500, Y=-300, Z=100 km
    fn build_synthetic_spk() -> Vec<u8> {
        // We'll build a minimal valid DAF/SPK file.
        // File layout:
        //   Record 1 (1024 bytes): File record
        //   Record 2 (1024 bytes): Comment record (empty)
        //   Record 3 (1024 bytes): Summary record
        //   Record 4+ : Segment data

        let mut file = vec![0u8; 1024 * 8]; // 8 records should be enough for 3 segments.

        // Helper: write LE f64.
        let write_f64 = |buf: &mut Vec<u8>, off: usize, v: f64| {
            buf[off..off + 8].copy_from_slice(&v.to_le_bytes());
        };
        let write_i32 = |buf: &mut Vec<u8>, off: usize, v: i32| {
            buf[off..off + 4].copy_from_slice(&v.to_le_bytes());
        };

        // --- Record 1: File Record ---
        // LOCIDW = "DAF/SPK "
        file[0..8].copy_from_slice(b"DAF/SPK ");
        // ND = 2
        write_i32(&mut file, 8, 2);
        // NI = 6
        write_i32(&mut file, 12, 6);
        // LOCIFN (60 bytes) at offset 16 - leave blank.
        // FWARD at offset 76 = record 3 (summary record).
        write_i32(&mut file, 76, 3);
        // BWARD at offset 80 = record 3 (same, only one summary record).
        write_i32(&mut file, 80, 3);
        // Free at offset 84 = 0.
        // LOCFMT at offset 88 = "LTL-IEEE"
        file[88..96].copy_from_slice(b"LTL-IEEE");

        // --- Record 2: Comment area (empty, just zeros) ---
        // Nothing to write.

        // --- Prepare segment data (records 4, 5, 6) ---
        // Each segment: 1 Chebyshev record with 4 coefficients per component.
        // rsize = 2 + 3*4 = 14 doubles.
        // Directory: INIT, INTLEN, RSIZE, N (4 doubles at end of segment).
        // Total segment words: 14 (record) + 4 (directory) = 18 doubles.
        let ncoeffs = 4;
        let rsize = 2 + 3 * ncoeffs; // 14
        let n_records = 1;

        // Epoch: cover J2000 +/- 1000 days.
        let init_sec = -1000.0 * SECONDS_PER_DAY; // 1000 days before J2000
        let intlen = 2000.0 * SECONDS_PER_DAY; // 2000-day interval
        let mid_sec = 0.0; // midpoint at J2000
        let radius = 1000.0 * SECONDS_PER_DAY; // half-interval

        // Helper to write one segment's Chebyshev record + directory.
        let write_segment = |file: &mut Vec<u8>, data_offset: usize, x: f64, y: f64, z: f64| {
            write_f64(file, data_offset, mid_sec);
            write_f64(file, data_offset + 8, radius);
            write_f64(file, data_offset + 16, x);
            write_f64(file, data_offset + 16 + ncoeffs * 8, y);
            write_f64(file, data_offset + 16 + 2 * ncoeffs * 8, z);
            let dir_offset = data_offset + rsize * 8;
            write_f64(file, dir_offset, init_sec);
            write_f64(file, dir_offset + 8, intlen);
            write_f64(file, dir_offset + 16, rsize as f64);
            write_f64(file, dir_offset + 24, n_records as f64);
        };

        // Segment 1: EMB (target=3, center=0) — X=100000, Y=200000, Z=50000 km
        let seg1_start_word = (3 * DAF_RECORD_BYTES / 8) + 1; // 385
        let seg1_data_offset = 3 * DAF_RECORD_BYTES;
        write_segment(&mut file, seg1_data_offset, 100000.0, 200000.0, 50000.0);
        let seg1_end_word = seg1_start_word + rsize + 4 - 1;

        // Segment 2: Moon (target=301, center=3) — X=300, Y=400, Z=100 km
        let seg2_start_word = seg1_end_word + 1;
        let seg2_data_offset = (seg2_start_word - 1) * 8;
        write_segment(&mut file, seg2_data_offset, 300.0, 400.0, 100.0);
        let seg2_end_word = seg2_start_word + rsize + 4 - 1;

        // Segment 3: Sun (target=10, center=0) — X=500, Y=-300, Z=100 km
        let seg3_start_word = seg2_end_word + 1;
        let seg3_data_offset = (seg3_start_word - 1) * 8;
        write_segment(&mut file, seg3_data_offset, 500.0, -300.0, 100.0);
        let seg3_end_word = seg3_start_word + rsize + 4 - 1;

        // --- Record 3: Summary record ---
        let sum_rec_offset = 2 * DAF_RECORD_BYTES; // record 3 starts at byte 2048

        // NEXT = 0 (no more summary records)
        write_f64(&mut file, sum_rec_offset, 0.0);
        // PREV = 0
        write_f64(&mut file, sum_rec_offset + 8, 0.0);
        // NSUM = 3 (three segments)
        write_f64(&mut file, sum_rec_offset + 16, 3.0);

        // Helper to write a summary entry.
        let write_summary = |file: &mut Vec<u8>,
                             off: usize,
                             target: i32,
                             center: i32,
                             start_w: usize,
                             end_w: usize| {
            write_f64(file, off, init_sec);
            write_f64(file, off + 8, init_sec + intlen);
            write_i32(file, off + 16, target);
            write_i32(file, off + 20, center);
            write_i32(file, off + 24, 1); // frame = J2000
            write_i32(file, off + 28, 2); // data_type = Type 2
            write_i32(file, off + 32, start_w as i32);
            write_i32(file, off + 36, end_w as i32);
        };

        // Summary 1 (EMB): target=3, center=0
        let s1_off = sum_rec_offset + 24;
        write_summary(&mut file, s1_off, 3, 0, seg1_start_word, seg1_end_word);

        // Summary 2 (Moon): target=301, center=3
        let s2_off = s1_off + 5 * 8;
        write_summary(&mut file, s2_off, 301, 3, seg2_start_word, seg2_end_word);

        // Summary 3 (Sun): target=10, center=0
        let s3_off = s2_off + 5 * 8;
        write_summary(&mut file, s3_off, 10, 0, seg3_start_word, seg3_end_word);

        file
    }

    #[test]
    fn parse_synthetic_spk_file() {
        let spk_data = build_synthetic_spk();
        let dir = std::env::temp_dir().join("xalen_de440_spk_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("synthetic.bsp");
        std::fs::write(&path, &spk_data).unwrap();

        let reader = De440Reader::from_file(&path).expect("should parse synthetic SPK file");

        assert!(reader.has_data(), "reader should have data");
        assert!(reader.is_spk, "reader should be in SPK mode");
        assert_eq!(reader.segment_count(), 3, "should have 3 segments");

        // Check that we have the right bodies.
        let bodies = reader.available_bodies();
        assert!(bodies.contains(&(3, 0)), "should have EMB (3, 0)");
        assert!(bodies.contains(&(301, 3)), "should have Moon (301, 3)");
        assert!(bodies.contains(&(10, 0)), "should have Sun (10, 0)");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn spk_position_at_j2000() {
        let spk_data = build_synthetic_spk();
        let dir = std::env::temp_dir().join("xalen_de440_spk_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("position_test.bsp");
        std::fs::write(&path, &spk_data).unwrap();

        let reader = De440Reader::from_file(&path).unwrap();

        // Query EMB position at J2000.
        let (x, y, z) = reader.position_at(3, 0, TIME_J2000).unwrap();
        assert!(
            (x - 100000.0).abs() < 1e-6,
            "EMB X should be 100000 km, got {x}"
        );
        assert!(
            (y - 200000.0).abs() < 1e-6,
            "EMB Y should be 200000 km, got {y}"
        );
        assert!(
            (z - 50000.0).abs() < 1e-6,
            "EMB Z should be 50000 km, got {z}"
        );

        // Query Moon position at J2000.
        let (mx, my, mz) = reader.position_at(301, 3, TIME_J2000).unwrap();
        assert!(
            (mx - 300.0).abs() < 1e-6,
            "Moon X should be 300 km, got {mx}"
        );
        assert!(
            (my - 400.0).abs() < 1e-6,
            "Moon Y should be 400 km, got {my}"
        );
        assert!(
            (mz - 100.0).abs() < 1e-6,
            "Moon Z should be 100 km, got {mz}"
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn spk_geocentric_moon() {
        let spk_data = build_synthetic_spk();
        let dir = std::env::temp_dir().join("xalen_de440_spk_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("geocentric_test.bsp");
        std::fs::write(&path, &spk_data).unwrap();

        let reader = De440Reader::from_file(&path).unwrap();

        // Moon geocentric should use the (1+EMRAT)/EMRAT factor.
        let geo = reader
            .geocentric_position_au(Body::Moon, TIME_J2000)
            .unwrap();
        let factor = (1.0 + EMRAT) / EMRAT;
        let expected_x = 300.0 * factor / AU_KM;
        assert!(
            (geo.x - expected_x).abs() < 1e-12,
            "Moon geo X should be {expected_x}, got {}",
            geo.x
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn spk_epoch_out_of_range() {
        let spk_data = build_synthetic_spk();
        let dir = std::env::temp_dir().join("xalen_de440_spk_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("epoch_range_test.bsp");
        std::fs::write(&path, &spk_data).unwrap();

        let reader = De440Reader::from_file(&path).unwrap();

        // Query at an epoch far outside the segment range.
        let result = reader.position_at(3, 0, TIME_J2000 + 5000.0);
        assert!(result.is_err(), "should fail for out-of-range epoch");

        let _ = std::fs::remove_file(&path);
    }

    // -----------------------------------------------------------------------
    // De440Provider tests
    // -----------------------------------------------------------------------

    #[test]
    fn fallback_provider_uses_vsop87() {
        let p = De440Provider::fallback_only();
        assert!(!p.has_de440_data());
        assert!(p.name().contains("fallback"));

        let sun = p.geocentric_ecliptic(Body::Sun, JdTT::J2000).unwrap();
        let lon_deg = sun.longitude.to_degrees();
        assert!(
            (lon_deg - 280.5).abs() < 1.0,
            "Fallback Sun at J2000 should be ~280.5 deg, got {lon_deg}"
        );
    }

    #[test]
    fn provider_with_records_does_not_claim_de440_metadata() {
        // `with_records` builds a SYNTHETIC/legacy reader, not a parsed `.bsp`.
        // Such a reader has data for routing purposes (`has_de440_data`) but must
        // NOT advertise the DE440 engine name or quote the DE440 accuracy figure.
        // That provenance claim is gated on `is_spk()` (a real parsed kernel).
        let header = De440Header::de440_defaults();
        let record = De440Record {
            jd_start: 2_287_184.5,
            jd_end: 2_287_184.5 + 32.0,
            coefficients: vec![0.0; 1000],
        };
        let reader = De440Reader::with_records(header, vec![record]);
        assert!(!reader.is_spk(), "with_records is not a parsed SPK kernel");

        let p = De440Provider::with_reader(reader);

        // Routing still sees the data...
        assert!(p.has_de440_data());
        // ...but provenance metadata must reflect the analytical fallback, not DE440.
        assert!(
            !p.is_de440_loaded(),
            "synthetic with_records data must not report a DE440 kernel"
        );
        assert!(
            !p.name().contains("DE440 (apparent"),
            "synthetic data must not claim the real DE440 engine name, got {:?}",
            p.name()
        );
        assert_eq!(
            p.accuracy_arcsec(),
            Vsop87Provider::new().accuracy_arcsec(),
            "synthetic data must quote the analytical fallback accuracy figure"
        );
    }

    #[test]
    fn provider_falls_back_for_unsupported_body() {
        let p = De440Provider::fallback_only();
        let result = p.geocentric_ecliptic(Body::MeanNode, JdTT::J2000);
        assert!(result.is_ok(), "MeanNode should work via VSOP87 fallback");
    }

    #[test]
    fn de440_target_mapping() {
        assert_eq!(
            De440Target::from_body(Body::Mercury),
            Some(De440Target::Mercury)
        );
        assert_eq!(
            De440Target::from_body(Body::Moon),
            Some(De440Target::MoonGeo)
        );
        assert_eq!(
            De440Target::from_body(Body::Pluto),
            Some(De440Target::Pluto)
        );
        assert_eq!(De440Target::from_body(Body::MeanNode), None);
        assert_eq!(De440Target::from_body(Body::TrueNode), None);
        assert_eq!(De440Target::from_body(Body::MeanApogee), None);
        assert_eq!(De440Target::from_body(Body::Chiron), None);
    }

    #[test]
    fn provider_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<De440Provider>();
    }

    #[test]
    fn earth_geocentric_is_origin() {
        let reader = De440Reader::empty();
        let result = reader.geocentric_position_au(Body::Earth, 2451545.0);
        assert!(result.is_ok());
        let cart = result.unwrap();
        assert!((cart.x).abs() < 1e-15);
        assert!((cart.y).abs() < 1e-15);
        assert!((cart.z).abs() < 1e-15);
    }

    #[test]
    fn almanac_integration_with_de440_provider() {
        use crate::almanac::Almanac;

        let provider = De440Provider::fallback_only();
        let almanac = Almanac::default_vedic().with_provider(Arc::new(provider));

        let jd = xalen_time::JdUT1(2451545.0);
        for body in Body::VEDIC_GRAHAS {
            let result = almanac.geocentric_longitude_deg(*body, jd);
            assert!(
                result.is_ok(),
                "Failed for {body} with DE440 provider in almanac: {:?}",
                result.err()
            );
        }
    }

    // -----------------------------------------------------------------------
    // Binary file reader tests
    // -----------------------------------------------------------------------

    #[test]
    fn from_file_rejects_nonexistent() {
        let result = De440Reader::from_file(Path::new("/nonexistent/de440.bsp"));
        assert!(result.is_err(), "should fail on missing file");
    }

    #[test]
    fn from_file_rejects_truncated_file() {
        let dir = std::env::temp_dir().join("xalen_de440_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("truncated.bin");
        std::fs::write(&path, &[0u8; 100]).unwrap();
        let result = De440Reader::from_file(&path);
        assert!(result.is_err(), "should reject truncated file");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn from_file_rejects_garbage_denum() {
        let dir = std::env::temp_dir().join("xalen_de440_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("garbage.bin");
        std::fs::write(&path, &[0u8; 8192]).unwrap();
        let result = De440Reader::from_file(&path);
        assert!(result.is_err(), "should reject file with invalid format");
    }

    #[test]
    fn from_file_synthetic_valid_header() {
        // Uses the legacy format builder.
        let dir = std::env::temp_dir().join("xalen_de440_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("synthetic_de440.bin");

        let defaults = De440Header::de440_defaults();

        let mut max_word = 0usize;
        for (i, entry) in defaults.ipt.iter().enumerate() {
            if entry.num_coeffs == 0 || entry.num_sub_intervals == 0 {
                continue;
            }
            let nc = if i == 11 { 2 } else { 3 };
            let base = match entry.offset.checked_sub(1) {
                Some(b) => b,
                None => continue,
            };
            let end_word = base + entry.words_per_record(nc);
            if end_word > max_word {
                max_word = end_word;
            }
        }
        let ncoeff = max_word + 2;
        let record_bytes = ncoeff * 8;

        let total_bytes = 3 * record_bytes;
        let mut file_data = vec![0u8; total_bytes];

        let write_f64 = |buf: &mut Vec<u8>, off: usize, v: f64| {
            buf[off..off + 8].copy_from_slice(&v.to_le_bytes());
        };
        let write_i32 = |buf: &mut Vec<u8>, off: usize, v: i32| {
            buf[off..off + 4].copy_from_slice(&v.to_le_bytes());
        };

        write_f64(&mut file_data, 2652, defaults.jd_start);
        write_f64(&mut file_data, 2660, defaults.jd_end);
        write_f64(&mut file_data, 2668, defaults.block_span);
        write_i32(&mut file_data, 2676, 400);
        write_f64(&mut file_data, 2680, defaults.au_km);
        write_f64(&mut file_data, 2688, defaults.emrat);

        for i in 0..12 {
            let base_off = 2696 + i * 12;
            write_i32(&mut file_data, base_off, defaults.ipt[i].offset as i32);
            write_i32(
                &mut file_data,
                base_off + 4,
                defaults.ipt[i].num_coeffs as i32,
            );
            write_i32(
                &mut file_data,
                base_off + 8,
                defaults.ipt[i].num_sub_intervals as i32,
            );
        }
        write_i32(&mut file_data, 2840, 440);
        write_i32(&mut file_data, 2844, defaults.ipt[12].offset as i32);
        write_i32(&mut file_data, 2848, defaults.ipt[12].num_coeffs as i32);
        write_i32(
            &mut file_data,
            2852,
            defaults.ipt[12].num_sub_intervals as i32,
        );

        let data_off = 2 * record_bytes;
        let rec_start = defaults.jd_start;
        let rec_end = defaults.jd_start + defaults.block_span;
        write_f64(&mut file_data, data_off, rec_start);
        write_f64(&mut file_data, data_off + 8, rec_end);
        let merc_ipt = &defaults.ipt[0];
        let coeff_base = data_off + 16 + (merc_ipt.offset - 3) * 8;
        write_f64(&mut file_data, coeff_base, 42000.0);

        std::fs::write(&path, &file_data).unwrap();

        let reader = De440Reader::from_file(&path).expect("should parse synthetic file");
        assert!(reader.has_data(), "reader should have data records");
        assert!(!reader.is_spk, "should use legacy path (not DAF/SPK)");

        assert!((reader.header.jd_start - defaults.jd_start).abs() < 0.001);
        assert!((reader.header.jd_end - defaults.jd_end).abs() < 0.001);
        assert!((reader.header.emrat - defaults.emrat).abs() < 0.001);

        let (x, _y, _z) = reader
            .position_km(De440Target::Mercury, rec_start + 0.01)
            .unwrap();
        assert!(
            (x - 42000.0).abs() < 1e-6,
            "Mercury X should be 42000.0 km, got {x}"
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn from_file_returns_err_on_bad_epoch_range() {
        let dir = std::env::temp_dir().join("xalen_de440_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("bad_epoch.bin");

        let mut file_data = vec![0u8; 8192];
        let write_f64 = |buf: &mut Vec<u8>, off: usize, v: f64| {
            buf[off..off + 8].copy_from_slice(&v.to_le_bytes());
        };
        let write_i32 = |buf: &mut Vec<u8>, off: usize, v: i32| {
            buf[off..off + 4].copy_from_slice(&v.to_le_bytes());
        };

        write_i32(&mut file_data, 2840, 440);
        write_f64(&mut file_data, 2652, 3_000_000.0);
        write_f64(&mut file_data, 2660, 2_000_000.0);
        write_f64(&mut file_data, 2668, 32.0);

        std::fs::write(&path, &file_data).unwrap();
        let result = De440Reader::from_file(&path);
        assert!(result.is_err(), "should reject bad epoch range");
        let _ = std::fs::remove_file(&path);
    }

    /// Attempt to load a real DE440 BSP file from standard locations.
    #[test]
    fn from_file_real_de440_if_available() {
        let candidates = [
            "/usr/share/ephem/de440.bsp",
            "/usr/local/share/ephem/de440.bsp",
            "de440.bsp",
        ];
        let real_path = candidates.iter().find(|p| Path::new(p).exists());
        if real_path.is_none() {
            eprintln!("Skipping real DE440 test: no file found at standard locations");
            return;
        }
        let path = Path::new(real_path.unwrap());
        let reader = De440Reader::from_file(path).expect("should parse real DE440 file");
        assert!(reader.has_data());
        assert!(reader.is_spk, "real DE440 should be in SPK mode");

        let h = reader.header();
        assert!(h.jd_start < TIME_J2000);
        assert!(h.jd_end > TIME_J2000);

        // If real file loaded, test actual position query.
        let result = reader.position_at(3, 0, TIME_J2000);
        assert!(result.is_ok(), "EMB position at J2000 should succeed");
        let (x, y, z) = result.unwrap();
        // EMB at J2000 should be within a few AU of the Sun (~1 AU).
        let dist_km = (x * x + y * y + z * z).sqrt();
        let dist_au = dist_km / AU_KM;
        assert!(
            dist_au > 0.5 && dist_au < 2.0,
            "EMB distance from SSB should be ~1 AU, got {dist_au} AU"
        );
    }

    // -----------------------------------------------------------------------
    // Almanac::with_de440 test
    // -----------------------------------------------------------------------

    #[test]
    fn almanac_with_de440_nonexistent_file_is_noop() {
        use crate::almanac::Almanac;

        let almanac = Almanac::default_vedic().with_de440(Path::new("/nonexistent/de440.bsp"));

        // Should still work via VSOP87 fallback.
        let jd = xalen_time::JdUT1(2451545.0);
        let result = almanac.geocentric_longitude_deg(Body::Sun, jd);
        assert!(result.is_ok());
    }

    // -----------------------------------------------------------------------
    // NaifId tests
    // -----------------------------------------------------------------------

    #[test]
    fn naif_id_body_mapping() {
        assert_eq!(
            NaifId::body_to_naif(Body::Mercury),
            Some((NaifId(1), NaifId(0)))
        );
        assert_eq!(
            NaifId::body_to_naif(Body::Moon),
            Some((NaifId(301), NaifId(3)))
        );
        assert_eq!(
            NaifId::body_to_naif(Body::Sun),
            Some((NaifId(10), NaifId(0)))
        );
        // Contract lock (P1-14): Earth maps to the Earth-Moon Barycenter (3)
        // relative to SSB (0), NOT to the Earth body center (399). The
        // geocenter is derived from EMB + Moon in `earth_ssb_au`.
        assert_eq!(
            NaifId::body_to_naif(Body::Earth),
            Some((NaifId(3), NaifId(0)))
        );
        assert_eq!(NaifId::body_to_naif(Body::MeanNode), None);
    }

    // -----------------------------------------------------------------------
    // Heliocentric position tests (the P0 fix)
    // -----------------------------------------------------------------------

    #[test]
    fn spk_heliocentric_subtracts_sun_ssb() {
        // In our synthetic SPK:
        //   EMB_SSB = (100000, 200000, 50000) km
        //   Sun_SSB = (500, -300, 100) km
        //   Moon_EMB = (300, 400, 100) km
        //
        // Earth_SSB = EMB_SSB - Moon_EMB / (1 + EMRAT)
        // Heliocentric(Earth) = Earth_SSB - Sun_SSB
        // Heliocentric(Sun) = (0, 0, 0) by definition

        let spk_data = build_synthetic_spk();
        let dir = std::env::temp_dir().join("xalen_de440_spk_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("helio_test.bsp");
        std::fs::write(&path, &spk_data).unwrap();

        let reader = De440Reader::from_file(&path).unwrap();

        // Sun heliocentric = origin
        let sun_helio = reader
            .heliocentric_position_au(Body::Sun, TIME_J2000)
            .unwrap();
        assert!(
            sun_helio.x.abs() < 1e-15,
            "Sun helio X should be 0, got {}",
            sun_helio.x
        );
        assert!(
            sun_helio.y.abs() < 1e-15,
            "Sun helio Y should be 0, got {}",
            sun_helio.y
        );
        assert!(
            sun_helio.z.abs() < 1e-15,
            "Sun helio Z should be 0, got {}",
            sun_helio.z
        );

        // Earth heliocentric = Earth_SSB - Sun_SSB
        let earth_helio = reader
            .heliocentric_position_au(Body::Earth, TIME_J2000)
            .unwrap();
        let factor = 1.0 / (1.0 + EMRAT);
        let earth_ssb_x = 100000.0 - 300.0 * factor;
        let earth_ssb_y = 200000.0 - 400.0 * factor;
        let earth_ssb_z = 50000.0 - 100.0 * factor;
        let expected_x = (earth_ssb_x - 500.0) / AU_KM;
        let expected_y = (earth_ssb_y - (-300.0)) / AU_KM;
        let expected_z = (earth_ssb_z - 100.0) / AU_KM;
        assert!(
            (earth_helio.x - expected_x).abs() < 1e-15,
            "Earth helio X: expected {expected_x}, got {}",
            earth_helio.x
        );
        assert!(
            (earth_helio.y - expected_y).abs() < 1e-15,
            "Earth helio Y: expected {expected_y}, got {}",
            earth_helio.y
        );
        assert!(
            (earth_helio.z - expected_z).abs() < 1e-15,
            "Earth helio Z: expected {expected_z}, got {}",
            earth_helio.z
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn spk_heliocentric_is_not_ssb() {
        // Verify that heliocentric != SSB-centric (the bug we fixed).
        // In our synthetic SPK, Sun_SSB = (500, -300, 100) km,
        // so heliocentric should differ from raw SSB position by that amount.

        let spk_data = build_synthetic_spk();
        let dir = std::env::temp_dir().join("xalen_de440_spk_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("helio_not_ssb_test.bsp");
        std::fs::write(&path, &spk_data).unwrap();

        let reader = De440Reader::from_file(&path).unwrap();

        // EMB SSB position (raw)
        let emb_ssb = reader
            .position_au(De440Target::EarthMoonBary, TIME_J2000)
            .unwrap();
        // Earth heliocentric (corrected)
        let earth_helio = reader
            .heliocentric_position_au(Body::Earth, TIME_J2000)
            .unwrap();

        // They should NOT be equal because Sun_SSB != (0,0,0)
        let diff = (emb_ssb.x - earth_helio.x).abs()
            + (emb_ssb.y - earth_helio.y).abs()
            + (emb_ssb.z - earth_helio.z).abs();
        assert!(
            diff > 1e-10,
            "Heliocentric should differ from SSB-centric (Sun offset is non-zero)"
        );

        let _ = std::fs::remove_file(&path);
    }

    // -----------------------------------------------------------------------
    // Cross-validation: DE440 geocentric vs VSOP87A at J2000
    // -----------------------------------------------------------------------

    #[test]
    fn de440_vs_vsop87_geocentric_at_j2000() {
        // Load DE440 from /tmp/de440s.bsp if available; skip gracefully otherwise.
        let bsp_path = Path::new("/tmp/de440s.bsp");
        if !bsp_path.exists() {
            eprintln!("Skipping DE440-vs-VSOP87 cross-validation: /tmp/de440s.bsp not found");
            return;
        }

        let de440 = De440Provider::try_from_file(bsp_path);
        if !de440.has_de440_data() {
            eprintln!("Skipping DE440-vs-VSOP87 cross-validation: failed to load /tmp/de440s.bsp");
            return;
        }

        let vsop = Vsop87Provider::new();
        let jd = JdTT::J2000;

        // At J2000, VSOP87A is calibrated to match DE; they should agree
        // within ~0.01 degrees for all planets. We use a generous 0.02 deg
        // tolerance to account for minor theory differences and the Pluto
        // analytical series.
        let tolerance_deg = 0.02;

        let bodies = [
            Body::Mercury,
            Body::Venus,
            Body::Mars,
            Body::Jupiter,
            Body::Saturn,
            Body::Uranus,
            Body::Neptune,
        ];

        for &body in &bodies {
            let de440_pos = de440.geocentric_ecliptic(body, jd).expect(&format!(
                "DE440 geocentric for {body} at J2000 should succeed"
            ));
            let vsop_pos = vsop.geocentric_ecliptic(body, jd).expect(&format!(
                "VSOP87 geocentric for {body} at J2000 should succeed"
            ));

            let de440_lon_deg = de440_pos.longitude.to_degrees().rem_euclid(360.0);
            let vsop_lon_deg = vsop_pos.longitude.to_degrees().rem_euclid(360.0);

            // Handle wrap-around at 0/360 boundary.
            let mut diff = (de440_lon_deg - vsop_lon_deg).abs();
            if diff > 180.0 {
                diff = 360.0 - diff;
            }

            assert!(
                diff < tolerance_deg,
                "{body}: DE440 geocentric lon = {de440_lon_deg:.6} deg, \
                 VSOP87 = {vsop_lon_deg:.6} deg, diff = {diff:.6} deg \
                 (tolerance {tolerance_deg} deg)"
            );

            eprintln!(
                "  {body:>8}: DE440 = {de440_lon_deg:>10.6} deg, VSOP87 = {vsop_lon_deg:>10.6} deg, \
                 diff = {diff:.6} deg  OK",
            );
        }

        // Also cross-validate the Sun (geocentric).
        let de440_sun = de440
            .geocentric_ecliptic(Body::Sun, jd)
            .expect("DE440 geocentric Sun at J2000 should succeed");
        let vsop_sun = vsop
            .geocentric_ecliptic(Body::Sun, jd)
            .expect("VSOP87 geocentric Sun at J2000 should succeed");

        let de440_sun_deg = de440_sun.longitude.to_degrees().rem_euclid(360.0);
        let vsop_sun_deg = vsop_sun.longitude.to_degrees().rem_euclid(360.0);
        let mut sun_diff = (de440_sun_deg - vsop_sun_deg).abs();
        if sun_diff > 180.0 {
            sun_diff = 360.0 - sun_diff;
        }

        assert!(
            sun_diff < tolerance_deg,
            "Sun: DE440 geocentric lon = {de440_sun_deg:.6} deg, \
             VSOP87 = {vsop_sun_deg:.6} deg, diff = {sun_diff:.6} deg \
             (tolerance {tolerance_deg} deg)"
        );
        eprintln!(
            "  {:>8}: DE440 = {:>10.6} deg, VSOP87 = {:>10.6} deg, diff = {sun_diff:.6} deg  OK",
            "Sun", de440_sun_deg, vsop_sun_deg,
        );
    }

    // -----------------------------------------------------------------------
    // Pluto coverage: De440Provider maps NAIF 9 and serves Pluto across the
    // kernel's full span, including epochs the analytical Pluto series does
    // not (pre-1885 / post-2099). Kernel-gated: skips if /tmp/de440s.bsp is
    // absent so CI without a kernel still passes.
    // -----------------------------------------------------------------------

    #[test]
    fn de440_serves_pluto_pre1885_and_post2099_if_kernel() {
        let bsp_path = Path::new("/tmp/de440s.bsp");
        if !bsp_path.exists() {
            eprintln!("Skipping Pluto-coverage test: /tmp/de440s.bsp not found");
            return;
        }
        let de440 = De440Provider::try_from_file(bsp_path);
        if !de440.is_de440_loaded() {
            eprintln!("Skipping Pluto-coverage test: /tmp/de440s.bsp did not confirm as DE440");
            return;
        }

        // These epochs are deliberately PRE-1885 and POST-2099 — outside the
        // analytical Pluto series' validated window — yet inside the date span
        // of the trimmed de440s.bsp kernel shipped for this build (~1850–2150
        // CE). NAIF 9 (Pluto System Barycenter) must resolve at both.
        // 1860-01-01 = JD 2_400_410.5 ; 2120-01-01 = JD 2_495_373.5.
        // The inner coverage guard below keeps this honest for any narrower
        // kernel a different environment might ship.
        let pre_1885 = JdTT(2_400_410.5); // 1860 CE — pre-1885
        let post_2099 = JdTT(2_495_373.5); // 2120 CE — post-2099

        for (label, jd) in [("1860-01-01", pre_1885), ("2120-01-01", post_2099)] {
            // Confirm the epoch is actually within this kernel's coverage; if a
            // particular trimmed kernel is narrower, skip rather than fail.
            let (cov_start, cov_end) = de440.coverage();
            let jd_tdb = jd.to_tdb().as_f64();
            if jd_tdb < cov_start || jd_tdb > cov_end {
                eprintln!(
                    "Skipping Pluto @ {label}: JD {jd_tdb:.1} outside kernel coverage \
                     [{cov_start:.1}, {cov_end:.1}]"
                );
                continue;
            }

            let geo = de440
                .geocentric_ecliptic(Body::Pluto, jd)
                .unwrap_or_else(|e| panic!("DE440 geocentric Pluto @ {label} must resolve: {e}"));
            let helio = de440
                .heliocentric_ecliptic(Body::Pluto, jd)
                .unwrap_or_else(|e| panic!("DE440 heliocentric Pluto @ {label} must resolve: {e}"));

            let geo_lon = geo.longitude.to_degrees().rem_euclid(360.0);
            let helio_lon = helio.longitude.to_degrees().rem_euclid(360.0);
            assert!(
                geo_lon.is_finite(),
                "Pluto geo lon @ {label} must be finite"
            );
            assert!(
                helio_lon.is_finite(),
                "Pluto helio lon @ {label} must be finite"
            );
            eprintln!(
                "  Pluto @ {label}: geo = {geo_lon:>10.6} deg, helio = {helio_lon:>10.6} deg  OK"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Honest capability accessor: with-kernel vs analytical-fallback tier.
    // These run without any kernel file (pure unit tests of the accessor).
    // -----------------------------------------------------------------------

    #[test]
    fn accuracy_tier_analytical_fallback_when_no_kernel() {
        let p = De440Provider::fallback_only();
        assert_eq!(p.accuracy_tier(), AccuracyTier::AnalyticalFallback);
        assert!(!p.accuracy_tier().is_kernel_backed());
        assert!(!p.is_de440_loaded());
        // The tier's accuracy story must agree with the trait figure: a
        // non-kernel-backed tier reports the analytical fallback accuracy.
        assert_eq!(p.accuracy_arcsec(), Vsop87Provider::new().accuracy_arcsec());
    }

    #[test]
    fn accuracy_tier_kernel_for_confirmed_de440() {
        // A synthetic kernel whose comment area carries a "DE440" label is
        // detected as DE440 provenance → Kernel tier.
        let day = SECONDS_PER_DAY;
        let segs = [SynthSeg {
            target: 3,
            center: 0,
            start_sec: -1000.0 * day,
            end_sec: 1000.0 * day,
            x: 1.0e8,
            y: 0.0,
            z: 0.0,
        }];
        let bytes = build_spk_labeled(&segs, "JPL planetary ephemeris DE440 / test kernel");
        let path = write_spk_tmp("tier_de440.bsp", &bytes);
        let reader = De440Reader::from_file(&path).expect("synthetic DE440 should parse");
        let p = De440Provider::with_reader(reader);
        assert_eq!(p.accuracy_tier(), AccuracyTier::Kernel);
        assert!(p.accuracy_tier().is_kernel_backed());
        assert!(p.is_de440_loaded());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn accuracy_tier_unconfirmed_for_spk_without_de_label() {
        // A real SPK with NO recognizable DE label parses as an SPK but must
        // NOT claim DE440 provenance → UnconfirmedKernel tier (analytical-grade
        // accuracy figure).
        let day = SECONDS_PER_DAY;
        let segs = [SynthSeg {
            target: 3,
            center: 0,
            start_sec: -1000.0 * day,
            end_sec: 1000.0 * day,
            x: 1.0e8,
            y: 0.0,
            z: 0.0,
        }];
        // build_spk_with zeroes the comment record -> no detectable label.
        let bytes = build_spk_with(&segs, None, None);
        let path = write_spk_tmp("tier_nolabel.bsp", &bytes);
        let reader = De440Reader::from_file(&path).expect("synthetic SPK should parse");
        assert!(reader.is_spk());
        assert!(reader.kernel_id().is_none());
        let p = De440Provider::with_reader(reader);
        assert_eq!(p.accuracy_tier(), AccuracyTier::UnconfirmedKernel);
        assert!(!p.accuracy_tier().is_kernel_backed());
        assert!(!p.is_de440_loaded());
        assert_eq!(p.accuracy_arcsec(), Vsop87Provider::new().accuracy_arcsec());
        let _ = std::fs::remove_file(&path);
    }

    // -----------------------------------------------------------------------
    // LOUD load failures: strict loaders return the error instead of silently
    // degrading to VSOP87 while appearing DE440-capable.
    // -----------------------------------------------------------------------

    #[test]
    fn try_from_file_strict_errors_on_nonexistent() {
        let r = De440Provider::try_from_file_strict(Path::new("/nonexistent/de440.bsp"));
        assert!(
            r.is_err(),
            "strict loader must surface a load error, not silently fall back"
        );
    }

    #[test]
    fn try_from_file_lenient_is_silent_fallback_but_not_de440() {
        // The lenient loader still degrades (with a logged warning), but the
        // resulting provider must honestly report itself as analytical, not
        // DE440-capable.
        let p = De440Provider::try_from_file(Path::new("/nonexistent/de440.bsp"));
        assert!(!p.has_de440_data());
        assert!(!p.is_de440_loaded());
        assert_eq!(p.accuracy_tier(), AccuracyTier::AnalyticalFallback);
    }

    #[test]
    fn almanac_with_de440_strict_errors_on_nonexistent() {
        use crate::almanac::Almanac;
        let r = Almanac::default_vedic().with_de440_strict(Path::new("/nonexistent/de440.bsp"));
        assert!(
            r.is_err(),
            "with_de440_strict must surface a load error, not silently no-op"
        );
    }

    // -----------------------------------------------------------------------
    // P1 regression tests (de440 parser hardening + segment priority)
    // -----------------------------------------------------------------------

    /// A configurable synthetic single-Chebyshev-record segment.
    struct SynthSeg {
        target: i32,
        center: i32,
        /// Segment start epoch, seconds past J2000.
        start_sec: f64,
        /// Segment end epoch, seconds past J2000.
        end_sec: f64,
        /// Constant X position in km the segment evaluates to.
        x: f64,
        y: f64,
        z: f64,
    }

    /// Build a minimal valid DAF/SPK file from a list of segments, in ONE summary
    /// record. `seg_word_override` / `next_override` allow injecting the malformed
    /// values used by the safety regression tests. Returns the file bytes.
    ///
    /// Layout: record 1 = file record, record 2 = comment, record 3 = summary
    /// record, records 4.. = segment data (one Chebyshev record + directory each).
    fn build_spk_with(
        segs: &[SynthSeg],
        next_override: Option<f64>,
        seg_word_override: Option<(usize, i32, i32)>, // (seg_idx, start_word, end_word)
    ) -> Vec<u8> {
        let mut file = vec![0u8; 1024 * (4 + segs.len())];
        let write_f64 = |buf: &mut Vec<u8>, off: usize, v: f64| {
            buf[off..off + 8].copy_from_slice(&v.to_le_bytes());
        };
        let write_i32 = |buf: &mut Vec<u8>, off: usize, v: i32| {
            buf[off..off + 4].copy_from_slice(&v.to_le_bytes());
        };

        // File record.
        file[0..8].copy_from_slice(b"DAF/SPK ");
        write_i32(&mut file, 8, 2); // ND
        write_i32(&mut file, 12, 6); // NI
        write_i32(&mut file, 76, 3); // FWARD = summary record 3
        write_i32(&mut file, 80, 3); // BWARD = 3
        file[88..96].copy_from_slice(b"LTL-IEEE");

        let ncoeffs = 4usize;
        let rsize = 2 + 3 * ncoeffs; // 14
        let n_records = 1usize;
        let seg_words = rsize + 4; // record + directory

        // Summary record (record 3).
        let sum_rec_offset = 2 * DAF_RECORD_BYTES;
        write_f64(&mut file, sum_rec_offset, next_override.unwrap_or(0.0)); // NEXT
        write_f64(&mut file, sum_rec_offset + 8, 0.0); // PREV
        write_f64(&mut file, sum_rec_offset + 16, segs.len() as f64); // NSUM

        for (i, seg) in segs.iter().enumerate() {
            // Segment data lives in record (4 + i).
            let start_word = (3 + i) * DAF_RECORD_BYTES / 8 + 1;
            let end_word = start_word + seg_words - 1;
            let data_offset = (start_word - 1) * 8;

            // Chebyshev record: MID at segment center, RADIUS = half-span,
            // constant term = position (higher coeffs zero -> constant output).
            let mid = (seg.start_sec + seg.end_sec) / 2.0;
            let radius = (seg.end_sec - seg.start_sec) / 2.0;
            write_f64(&mut file, data_offset, mid);
            write_f64(&mut file, data_offset + 8, radius);
            write_f64(&mut file, data_offset + 16, seg.x);
            write_f64(&mut file, data_offset + 16 + ncoeffs * 8, seg.y);
            write_f64(&mut file, data_offset + 16 + 2 * ncoeffs * 8, seg.z);
            // Directory: INIT, INTLEN, RSIZE, N.
            let dir_offset = data_offset + rsize * 8;
            write_f64(&mut file, dir_offset, seg.start_sec);
            write_f64(&mut file, dir_offset + 8, seg.end_sec - seg.start_sec);
            write_f64(&mut file, dir_offset + 16, rsize as f64);
            write_f64(&mut file, dir_offset + 24, n_records as f64);

            // Summary entry.
            let off = sum_rec_offset + 24 + i * 5 * 8;
            write_f64(&mut file, off, seg.start_sec);
            write_f64(&mut file, off + 8, seg.end_sec);
            write_i32(&mut file, off + 16, seg.target);
            write_i32(&mut file, off + 20, seg.center);
            write_i32(&mut file, off + 24, 1); // frame
            write_i32(&mut file, off + 28, 2); // data_type 2
            let (sw, ew) = match seg_word_override {
                Some((idx, sw, ew)) if idx == i => (sw, ew),
                _ => (start_word as i32, end_word as i32),
            };
            write_i32(&mut file, off + 32, sw);
            write_i32(&mut file, off + 36, ew);
        }

        file
    }

    fn write_spk_tmp(name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("xalen_de440_p1_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join(name);
        std::fs::write(&path, bytes).unwrap();
        path
    }

    /// Build a synthetic SPK (via `build_spk_with`) and stamp an ASCII label
    /// into the comment area (record 2, since FWARD=3) so provenance detection
    /// (`detect_de_label`) has something to find.
    fn build_spk_labeled(segs: &[SynthSeg], comment: &str) -> Vec<u8> {
        let mut bytes = build_spk_with(segs, None, None);
        let off = DAF_RECORD_BYTES; // record 2 = comment area
        let c = comment.as_bytes();
        let len = c.len().min(DAF_RECORD_BYTES);
        bytes[off..off + len].copy_from_slice(&c[..len]);
        bytes
    }

    /// P1-9: When two segments for the same (target, center) both bracket the
    /// requested epoch and the LATER-in-file one has an EARLIER start_sec, the
    /// later-file-order segment must win (SPK last-loaded priority). A
    /// start-sorted lookup would wrongly return the earlier (stale) segment.
    #[test]
    fn segment_priority_last_in_file_wins() {
        let day = SECONDS_PER_DAY;
        let segs = [
            // Earlier in file: bracket J2000 widely, value 1000.0.
            SynthSeg {
                target: 3,
                center: 0,
                start_sec: -2000.0 * day,
                end_sec: 2000.0 * day,
                x: 1000.0,
                y: 0.0,
                z: 0.0,
            },
            // Later in file but EARLIER start: still brackets J2000, value 2000.0.
            // This is the "replacement" segment and must take priority.
            SynthSeg {
                target: 3,
                center: 0,
                start_sec: -3000.0 * day,
                end_sec: 1000.0 * day,
                x: 2000.0,
                y: 0.0,
                z: 0.0,
            },
        ];
        let bytes = build_spk_with(&segs, None, None);
        let path = write_spk_tmp("priority.bsp", &bytes);
        let reader = De440Reader::from_file(&path).expect("synthetic SPK should parse");

        let (x, _, _) = reader.position_at(3, 0, TIME_J2000).unwrap();
        assert!(
            (x - 2000.0).abs() < 1e-6,
            "expected the later-in-file segment (x=2000) to win, got x={x}"
        );
        let _ = std::fs::remove_file(&path);
    }

    /// P1-10 (revised): A negative word address in a summary is structural
    /// corruption. It must neither panic (debug overflow-checks catch
    /// `(neg as usize) * 8`) NOR load a silently-partial kernel — the whole file
    /// is rejected as `InvalidFormat`, which `with_de440` turns into a clean
    /// VSOP87 fallback.
    #[test]
    fn negative_word_address_is_rejected() {
        let day = SECONDS_PER_DAY;
        let segs = [
            SynthSeg {
                target: 3,
                center: 0,
                start_sec: -1000.0 * day,
                end_sec: 1000.0 * day,
                x: 111.0,
                y: 0.0,
                z: 0.0,
            },
            SynthSeg {
                target: 10,
                center: 0,
                start_sec: -1000.0 * day,
                end_sec: 1000.0 * day,
                x: 222.0,
                y: 0.0,
                z: 0.0,
            },
        ];
        // Corrupt segment 1 (the Sun) with negative word addresses.
        let bytes = build_spk_with(&segs, None, Some((1, -1, -1)));
        let path = write_spk_tmp("negword.bsp", &bytes);

        // Must not panic, and must not load a partial kernel: a negative word
        // address rejects the whole file as InvalidFormat.
        let result = De440Reader::from_file(&path);
        assert!(
            matches!(result, Err(EphemerisError::InvalidFormat(_))),
            "a negative word address must be rejected as InvalidFormat, got {result:?}"
        );
        let _ = std::fs::remove_file(&path);
    }

    /// P1-11 (revised): A summary-record NEXT pointer that self-references (a
    /// chain cycle) is structural corruption. It must terminate the FWARD walk
    /// rather than spin forever AND reject the whole file as `InvalidFormat`
    /// rather than load a partial kernel.
    #[test]
    fn summary_chain_cycle_is_rejected() {
        let day = SECONDS_PER_DAY;
        let segs = [SynthSeg {
            target: 3,
            center: 0,
            start_sec: -1000.0 * day,
            end_sec: 1000.0 * day,
            x: 100.0,
            y: 0.0,
            z: 0.0,
        }];
        // NEXT = 3.0 -> record 3 points back at itself (FWARD was 3).
        let bytes = build_spk_with(&segs, Some(3.0), None);
        let path = write_spk_tmp("cycle.bsp", &bytes);

        // If the cycle guard is missing this hangs. Reaching the assert proves
        // termination; InvalidFormat proves we reject rather than partial-load.
        let result = De440Reader::from_file(&path);
        assert!(
            matches!(result, Err(EphemerisError::InvalidFormat(_))),
            "a self-referential NEXT must be rejected as InvalidFormat, got {result:?}"
        );
        let _ = std::fs::remove_file(&path);
    }

    /// external SME-review follow-up: a NEXT pointer referencing a summary record past
    /// end-of-file must be rejected as InvalidFormat, not break the walk and keep
    /// the segments parsed so far (a silent partial load).
    #[test]
    fn summary_next_beyond_eof_is_rejected() {
        let day = SECONDS_PER_DAY;
        let segs = [SynthSeg {
            target: 3,
            center: 0,
            start_sec: -1000.0 * day,
            end_sec: 1000.0 * day,
            x: 100.0,
            y: 0.0,
            z: 0.0,
        }];
        // NEXT = 9999 -> record 9999 is far past the ~5-record synthetic file.
        let bytes = build_spk_with(&segs, Some(9999.0), None);
        let path = write_spk_tmp("next_eof.bsp", &bytes);
        let result = De440Reader::from_file(&path);
        assert!(
            matches!(result, Err(EphemerisError::InvalidFormat(_))),
            "a NEXT past EOF must be rejected as InvalidFormat, got {result:?}"
        );
        let _ = std::fs::remove_file(&path);
    }

    /// external SME-review follow-up: an NSUM larger than the record/file can hold must
    /// be rejected as InvalidFormat, not break the inner loop and keep a partial
    /// set of summaries.
    #[test]
    fn summary_nsum_overrun_is_rejected() {
        let day = SECONDS_PER_DAY;
        let segs = [SynthSeg {
            target: 3,
            center: 0,
            start_sec: -1000.0 * day,
            end_sec: 1000.0 * day,
            x: 100.0,
            y: 0.0,
            z: 0.0,
        }];
        let mut bytes = build_spk_with(&segs, None, None);
        // Corrupt NSUM (word 2 of the summary record at record 3 = offset 2*1024)
        // to a value far larger than fits the file.
        let nsum_off = 2 * DAF_RECORD_BYTES + 16;
        bytes[nsum_off..nsum_off + 8].copy_from_slice(&9999.0_f64.to_le_bytes());
        let path = write_spk_tmp("nsum_overrun.bsp", &bytes);
        let result = De440Reader::from_file(&path);
        assert!(
            matches!(result, Err(EphemerisError::InvalidFormat(_))),
            "an overrunning NSUM must be rejected as InvalidFormat, got {result:?}"
        );
        let _ = std::fs::remove_file(&path);
    }

    // Directory layout inside a `build_spk_with` segment: each segment lives in
    // record (4 + i), so its data starts at word (3 + i)*128 + 1 (1-based) and the
    // 4-word directory (INIT, INTLEN, RSIZE, N) follows the single rsize=14-word
    // Chebyshev record. These helpers compute the byte offsets the corruption
    // tests below patch.
    fn synth_seg_dir_offset(seg_idx: usize) -> usize {
        const RSIZE: usize = 14; // build_spk_with uses ncoeffs=4 -> rsize = 2 + 3*4
        let start_word = (3 + seg_idx) * DAF_RECORD_BYTES / 8 + 1;
        let data_offset = (start_word - 1) * 8;
        data_offset + RSIZE * 8
    }

    fn one_synth_seg() -> [SynthSeg; 1] {
        let day = SECONDS_PER_DAY;
        [SynthSeg {
            target: 3,
            center: 0,
            start_sec: -1000.0 * day,
            end_sec: 1000.0 * day,
            x: 100.0,
            y: 0.0,
            z: 0.0,
        }]
    }

    /// C6a (crafted-kernel robustness): an SPK Type 2 segment whose summary word
    /// span disagrees with its own directory (`end - start + 1 != n*rsize + 4`) is
    /// internally inconsistent corruption. It must be rejected as `InvalidFormat`
    /// rather than silently loaded with a span that does not match the records the
    /// directory describes.
    ///
    /// The directory (INIT, INTLEN, RSIZE, N) is read from the last 4 words at
    /// `end_word`, so to exercise the SPAN check specifically (not the directory
    /// check), we keep `end_word` at its real value — leaving the directory
    /// readable with the correct n=1, rsize=14 — and shrink `start_word` by 2.
    /// That makes the declared span 20 words while n*rsize+4 = 18, so only the
    /// span check can reject it.
    #[test]
    fn segment_span_mismatch_is_rejected() {
        let segs = one_synth_seg();
        // build_spk_with places segment 0 in record 4: real start_word = 3*128 + 1,
        // and seg_words = rsize + 4 = 18, so real end_word = start + 17.
        let real_start = (3 * DAF_RECORD_BYTES / 8 + 1) as i32; // 385
        let real_end = real_start + 18 - 1; // 402
        // Shrink start_word by 2: directory still read from `real_end`, but the
        // declared span (real_end - bad_start + 1) = 20 != 18.
        let bad_start = real_start - 2;
        let bytes = build_spk_with(&segs, None, Some((0, bad_start, real_end)));
        let path = write_spk_tmp("span_mismatch.bsp", &bytes);
        let result = De440Reader::from_file(&path);
        assert!(
            matches!(result, Err(EphemerisError::InvalidFormat(_))),
            "a segment whose span disagrees with n*rsize+4 must be rejected, got {result:?}"
        );
        let _ = std::fs::remove_file(&path);
    }

    /// C6c (crafted-kernel robustness): a non-finite RSIZE in the Type 2 directory
    /// must be rejected BEFORE the `as usize` cast (which would saturate +inf to
    /// `usize::MAX`, sailing past the cheap `rsize < 5` check). Patch the RSIZE
    /// word to +inf.
    #[test]
    fn non_finite_rsize_is_rejected() {
        let segs = one_synth_seg();
        let mut bytes = build_spk_with(&segs, None, None);
        let rsize_off = synth_seg_dir_offset(0) + 16; // RSIZE is directory word 2
        bytes[rsize_off..rsize_off + 8].copy_from_slice(&f64::INFINITY.to_le_bytes());
        let path = write_spk_tmp("inf_rsize.bsp", &bytes);
        let result = De440Reader::from_file(&path);
        assert!(
            matches!(result, Err(EphemerisError::InvalidFormat(_))),
            "a non-finite RSIZE must be rejected as InvalidFormat, got {result:?}"
        );
        let _ = std::fs::remove_file(&path);
    }

    /// C6c (crafted-kernel robustness): a non-integral RSIZE (e.g. 14.5) must be
    /// rejected — a record size is a whole word count, and a fractional value
    /// would otherwise truncate silently on the `as usize` cast.
    #[test]
    fn non_integral_rsize_is_rejected() {
        let segs = one_synth_seg();
        let mut bytes = build_spk_with(&segs, None, None);
        let rsize_off = synth_seg_dir_offset(0) + 16;
        bytes[rsize_off..rsize_off + 8].copy_from_slice(&14.5_f64.to_le_bytes());
        let path = write_spk_tmp("frac_rsize.bsp", &bytes);
        let result = De440Reader::from_file(&path);
        assert!(
            matches!(result, Err(EphemerisError::InvalidFormat(_))),
            "a non-integral RSIZE must be rejected as InvalidFormat, got {result:?}"
        );
        let _ = std::fs::remove_file(&path);
    }

    /// C6c (crafted-kernel robustness): an RSIZE that is not of the Type 2 form
    /// `2 + 3*ncoeffs` (so `(rsize - 2) % 3 != 0`) must be rejected — it would make
    /// `ncoeffs` and the per-component coefficient slicing in `position_km`
    /// inconsistent. 15 is whole and >= 5 but (15-2)%3 == 1.
    #[test]
    fn rsize_not_multiple_of_three_form_is_rejected() {
        let segs = one_synth_seg();
        let mut bytes = build_spk_with(&segs, None, None);
        let rsize_off = synth_seg_dir_offset(0) + 16;
        bytes[rsize_off..rsize_off + 8].copy_from_slice(&15.0_f64.to_le_bytes());
        let path = write_spk_tmp("badform_rsize.bsp", &bytes);
        let result = De440Reader::from_file(&path);
        assert!(
            matches!(result, Err(EphemerisError::InvalidFormat(_))),
            "an RSIZE not of the 2+3*ncoeffs form must be rejected, got {result:?}"
        );
        let _ = std::fs::remove_file(&path);
    }

    /// C6c (crafted-kernel robustness): a non-finite summary-record NEXT pointer
    /// must be rejected as `InvalidFormat`, never cast with `as i64` (which
    /// saturates +inf to `i64::MAX`) and fed back into the loop-top
    /// `(current_record - 1) * DAF_RECORD_BYTES` multiply (an overflow panic in
    /// debug). Patch NEXT (word 0 of summary record 3) to +inf.
    #[test]
    fn non_finite_next_pointer_is_rejected() {
        let segs = one_synth_seg();
        let mut bytes = build_spk_with(&segs, None, None);
        let next_off = 2 * DAF_RECORD_BYTES; // NEXT is word 0 of summary record 3
        bytes[next_off..next_off + 8].copy_from_slice(&f64::INFINITY.to_le_bytes());
        let path = write_spk_tmp("inf_next.bsp", &bytes);
        // Must not panic; must reject as InvalidFormat.
        let result = De440Reader::from_file(&path);
        assert!(
            matches!(result, Err(EphemerisError::InvalidFormat(_))),
            "a non-finite NEXT pointer must be rejected as InvalidFormat, got {result:?}"
        );
        let _ = std::fs::remove_file(&path);
    }

    /// C6b (crafted-kernel robustness): when a segment's directory time-coverage is
    /// NARROWER than its summary descriptor's [start_sec, end_sec], an epoch can
    /// pass `find_segment` (which uses the descriptor) yet fall past the records
    /// the directory actually holds. `position_km` must REJECT such an epoch
    /// (`EpochOutOfRange`) rather than silently clamp it to the last record and
    /// return a wrong position. We build a single-record segment whose descriptor
    /// claims a wide span but whose directory (INIT, INTLEN) covers only the first
    /// half, then query the upper half.
    #[test]
    fn position_km_rejects_epoch_past_directory_coverage() {
        let day = SECONDS_PER_DAY;
        // Descriptor claims J2000 ± 1000 days; directory below will cover only the
        // lower 1000 days (INIT = -1000d, INTLEN = 1000d, n=1 -> up to J2000).
        let segs = [SynthSeg {
            target: 3,
            center: 0,
            start_sec: -1000.0 * day,
            end_sec: 1000.0 * day,
            x: 100.0,
            y: 0.0,
            z: 0.0,
        }];
        let mut bytes = build_spk_with(&segs, None, None);
        // Narrow the directory INTLEN so n*intlen only reaches J2000 (offset 1.0
        // at J2000) — querying 500 days past J2000 gives offset = 1.5, which is
        // >= n + 0.5 and must be rejected. INTLEN is directory word 1.
        let intlen_off = synth_seg_dir_offset(0) + 8;
        bytes[intlen_off..intlen_off + 8].copy_from_slice(&(1000.0 * day).to_le_bytes());
        let path = write_spk_tmp("narrow_dir.bsp", &bytes);
        let reader = De440Reader::from_file(&path).expect("segment should still parse");

        // J2000 + 600 days: inside the descriptor span (<= +1000d) so find_segment
        // returns the segment, but past the directory's n*intlen coverage.
        let t_sec = 600.0 * day;
        assert!(
            reader.find_segment(3, 0, t_sec).is_some(),
            "find_segment uses the wide descriptor span and must still match"
        );
        let result = reader.position_at(3, 0, J2000_JD + 600.0);
        assert!(
            matches!(result, Err(EphemerisError::EpochOutOfRange(_))),
            "an epoch past the directory's actual coverage must be rejected, not \
             clamped to the last record, got {result:?}"
        );
        let _ = std::fs::remove_file(&path);
    }

    /// P1-1 (provenance): a kernel whose DAF comment area carries a "DE440"
    /// label is detected and advertised as DE440 across all provenance metadata.
    #[test]
    fn de440_provenance_detected_from_comment() {
        let day = SECONDS_PER_DAY;
        let segs = [SynthSeg {
            target: 3,
            center: 0,
            start_sec: -1000.0 * day,
            end_sec: 1000.0 * day,
            x: 1.0e8,
            y: 0.0,
            z: 0.0,
        }];
        let bytes = build_spk_labeled(&segs, "JPL planetary ephemeris DE440 / test kernel");
        let path = write_spk_tmp("de440_label.bsp", &bytes);
        let reader = De440Reader::from_file(&path).expect("synthetic SPK should parse");
        assert_eq!(
            reader.kernel_id(),
            Some("DE440"),
            "the DE440 label must be detected from the comment area"
        );

        let p = De440Provider::with_reader(reader);
        assert!(
            p.is_de440_loaded(),
            "a confirmed DE440 kernel must report is_de440_loaded"
        );
        assert!(
            p.name().contains("DE440 (apparent"),
            "a confirmed DE440 kernel reports the DE440 engine name, got {:?}",
            p.name()
        );
        // A confirmed DE440 kernel quotes the DE440-apparent worst-body tier
        // (bounds the Moon's ~11" apparent-place residual), NOT a 1" figure the
        // provider's own cross-validation does not actually achieve for the Moon.
        assert_eq!(p.accuracy_arcsec(), DE440_APPARENT_WORST_ARCSEC);
        let _ = std::fs::remove_file(&path);
    }

    /// P1-1 (provenance): a real SPK with NO DE label must NOT be advertised as
    /// DE440 — it reports the generic "JPL SPK kernel" name and the analytical
    /// fallback accuracy, so we never overclaim provenance we cannot verify.
    #[test]
    fn spk_without_de_label_is_not_advertised_as_de440() {
        let day = SECONDS_PER_DAY;
        let segs = [SynthSeg {
            target: 3,
            center: 0,
            start_sec: -1000.0 * day,
            end_sec: 1000.0 * day,
            x: 1.0e8,
            y: 0.0,
            z: 0.0,
        }];
        // build_spk_with zeroes the comment record -> no detectable label.
        let bytes = build_spk_with(&segs, None, None);
        let path = write_spk_tmp("nolabel.bsp", &bytes);
        let reader = De440Reader::from_file(&path).expect("synthetic SPK should parse");
        assert_eq!(
            reader.kernel_id(),
            None,
            "an unlabeled SPK must not be detected as a DE kernel"
        );

        let p = De440Provider::with_reader(reader);
        assert!(
            !p.is_de440_loaded(),
            "an unconfirmed SPK must not claim DE440 provenance"
        );
        assert!(
            p.name().contains("SPK kernel") && !p.name().contains("DE440 (apparent"),
            "an unconfirmed SPK reports the generic label, got {:?}",
            p.name()
        );
        assert_eq!(
            p.accuracy_arcsec(),
            Vsop87Provider::new().accuracy_arcsec(),
            "an unconfirmed SPK quotes the analytical fallback accuracy figure"
        );
        let _ = std::fs::remove_file(&path);
    }

    /// P1-1 (provenance robustness): `detect_de_label` must accept every form a
    /// real JPL kernel embeds (modern "DE440" text + legacy "DE-0440LE-0440"
    /// segment-name token, both present in the real `de440.bsp`/`de440s.bsp`),
    /// normalize zero-padding without truncating, and reject look-alikes that
    /// lack a left word boundary or enough digits.
    #[test]
    fn de440_label_detection_edge_cases() {
        let day = SECONDS_PER_DAY;
        let segs = [SynthSeg {
            target: 3,
            center: 0,
            start_sec: -1000.0 * day,
            end_sec: 1000.0 * day,
            x: 1.0e8,
            y: 0.0,
            z: 0.0,
        }];
        let cases: &[(&str, Option<&str>)] = &[
            // Real-world JPL forms.
            ("JPL planetary and lunar ephemeris DE440", Some("DE440")),
            ("DE-0440LE-0440", Some("DE440")), // legacy DAF segment-name token
            ("DE440", Some("DE440")),
            ("DE441", Some("DE441")),
            ("DE-0440s small kernel", Some("DE440")), // de440s trailing 's'
            ("DE-00440", Some("DE440")),              // wider zero-pad must NOT truncate to DE44
            // Must NOT match.
            ("NODE440 telemetry", None), // no left word boundary
            ("CODE440", None),
            ("DECEMBER 2020 build", None), // no digits right after "DE"
            ("DE2", None),                 // < 3 digits
        ];
        for (comment, want) in cases {
            let bytes = build_spk_labeled(&segs, comment);
            let path = write_spk_tmp("edge_label.bsp", &bytes);
            let reader = De440Reader::from_file(&path).expect("synthetic SPK should parse");
            assert_eq!(
                reader.kernel_id(),
                *want,
                "comment {comment:?} should detect {want:?}, got {:?}",
                reader.kernel_id()
            );
            let _ = std::fs::remove_file(&path);
        }
    }

    /// P1-12: A body whose epoch falls in a coverage GAP — inside the kernel's
    /// overall header range but outside THAT body's own segment — must fall back
    /// to VSOP87 in `geocentric_ecliptic`, not hard-fail. We build a kernel where
    /// the union of EMB+Moon+Sun brackets J2000, but Mars' segment does NOT cover
    /// J2000; requesting Mars at J2000 must return the VSOP87 longitude.
    #[test]
    fn geocentric_gap_falls_back_to_vsop() {
        let day = SECONDS_PER_DAY;
        let wide = (-2000.0 * day, 2000.0 * day);
        // Required bodies for Earth/Sun geometry, all bracketing J2000.
        let mut segs = vec![
            SynthSeg {
                target: 3,
                center: 0,
                start_sec: wide.0,
                end_sec: wide.1,
                x: 1.0e8,
                y: 0.0,
                z: 0.0,
            },
            SynthSeg {
                target: 301,
                center: 3,
                start_sec: wide.0,
                end_sec: wide.1,
                x: 3.0e5,
                y: 0.0,
                z: 0.0,
            },
            SynthSeg {
                target: 10,
                center: 0,
                start_sec: wide.0,
                end_sec: wide.1,
                x: 5.0e2,
                y: 0.0,
                z: 0.0,
            },
            // Mars barycenter present in the file (so header range covers J2000)
            // but its OWN segment only spans a window FAR from J2000.
            SynthSeg {
                target: 4,
                center: 0,
                start_sec: 5000.0 * day,
                end_sec: 6000.0 * day,
                x: 2.0e8,
                y: 0.0,
                z: 0.0,
            },
        ];
        // Keep header jd_start/jd_end derived from the union -> brackets J2000.
        let _ = &mut segs;
        let bytes = build_spk_with(&segs, None, None);
        let path = write_spk_tmp("gap.bsp", &bytes);
        let reader = De440Reader::from_file(&path).expect("synthetic SPK should parse");

        // Sanity: the header range must bracket J2000 (the Mars far-future
        // segment extends jd_end well past J2000).
        assert!(
            reader.header().jd_start <= TIME_J2000 && reader.header().jd_end >= TIME_J2000,
            "header range must bracket J2000 for this test to exercise the gap"
        );
        // Sanity: Mars has no segment covering J2000.
        let t_sec = (TIME_J2000 - J2000_JD) * SECONDS_PER_DAY;
        assert!(
            reader.find_segment(4, 0, t_sec).is_none(),
            "Mars must NOT have a J2000-covering segment for this test"
        );

        let provider = De440Provider::with_reader(reader);
        let vsop = Vsop87Provider::new();
        let jd = JdTT::J2000;
        let got = provider
            .geocentric_ecliptic(Body::Mars, jd)
            .expect("gap body must fall back to VSOP87, not error");
        let want = vsop
            .geocentric_ecliptic(Body::Mars, jd)
            .expect("VSOP87 Mars should compute");
        let mut d = (got.longitude.to_degrees() - want.longitude.to_degrees()).abs();
        if d > 180.0 {
            d = 360.0 - d;
        }
        assert!(
            d < 1e-6,
            "gap fallback longitude {} must equal VSOP87 {} (diff {d} deg)",
            got.longitude.to_degrees(),
            want.longitude.to_degrees()
        );
        let _ = std::fs::remove_file(&path);
    }

    /// The analytical provider's single `accuracy_arcsec` figure must not be
    /// better (smaller) than the worst PHYSICAL body it serves. After the Moon
    /// annual-aberration fix (the full κ=20.49552" term wrongly applied to the
    /// geocentric Moon was removed) the Moon validates at RMS ~2.8" / max ~12"
    /// vs pyswisseph (AD 1600-2100) — so the worst physical body is now the
    /// long-period analytical Pluto (Meeus Ch.37, ~1 arcminute over 1885-2099),
    /// NOT the Moon. The reported figure must bound (a) the live Moon error and
    /// (b) that worst-physical-body (Pluto) residual.
    ///
    /// SCOPE: the derived lunar nodes (mean ~19", true ~111" vs Swiss) are NOT
    /// part of this physical-body figure by design — that deviation reflects
    /// differing node algorithms, not ephemeris error (see vsop.rs / ACCURACY.md).
    ///
    /// Reference for (a) is an INDEPENDENT JPL Horizons DE440 geocentric tropical
    /// longitude (the same value used by `tests/swiss_eph_crossval.rs`):
    /// Moon = 161.9070° at JD 2460311.0 (2024-01-01 12:00 UT).
    #[test]
    fn accuracy_arcsec_not_better_than_worst_body() {
        let provider = Vsop87Provider::new();
        let claimed = provider.accuracy_arcsec();

        // JD for 2024-01-01 12:00 UT.
        let jd_2024 = xalen_time::JdUT1(2_460_311.0_f64);
        // Independent JPL Horizons DE440 reference (tropical, geocentric).
        let moon_ref_deg = 161.9070_f64;

        let almanac = crate::almanac::Almanac::default_vedic();
        let moon_lon = almanac
            .geocentric_longitude_deg(Body::Moon, jd_2024)
            .expect("Moon longitude should compute");

        // Smallest-angle difference, in arcseconds.
        let mut d = (moon_lon - moon_ref_deg).abs();
        if d > 180.0 {
            d = 360.0 - d;
        }
        let moon_err_arcsec = d * 3600.0;

        // Post-fix the annual-aberration bug is gone: the live Moon error is now
        // small (sub-arcsec at this epoch; <=~12" worst over 1600-2100 vs
        // pyswisseph). It must NOT regress back to the old ~17-44" buggy regime.
        assert!(
            moon_err_arcsec < 13.0,
            "post-fix Moon should be accurate (sub-arcsec here, <=~12\" worst), got {moon_err_arcsec}\""
        );
        // (a) the reported figure must bound the live Moon error.
        assert!(
            claimed >= moon_err_arcsec,
            "claimed accuracy {claimed}\" must bound the live Moon error {moon_err_arcsec}\""
        );
        // (b) and must bound the worst PHYSICAL body — now the long-period
        // analytical Pluto (~1 arcminute over its 1885-2099 window), not the Moon.
        const WORST_PHYSICAL_BODY_ARCSEC: f64 = 60.0; // Pluto ~1 arcmin
        assert!(
            claimed >= WORST_PHYSICAL_BODY_ARCSEC,
            "claimed accuracy {claimed}\" must bound the worst physical body \
             (analytical Pluto ~{WORST_PHYSICAL_BODY_ARCSEC}\")"
        );
    }
}
