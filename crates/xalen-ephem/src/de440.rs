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

/// Earth-Moon mass ratio (DE440 value).
const EMRAT: f64 = 81.300568;

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
    /// DE440 stores positions relative to different centers:
    /// - Planets 1-9: relative to Solar System Barycenter (0)
    /// - Sun (10): relative to SSB (0)
    /// - Moon (301): relative to Earth (3 = Earth-Moon Barycenter)
    /// - Earth (399): relative to Earth-Moon Barycenter (3)
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
        let base = ipt.offset.checked_sub(3)?;
        let sub_block_size = ipt.num_coeffs * num_components;
        let start = base + sub_interval * sub_block_size + component * ipt.num_coeffs;
        let end = start + ipt.num_coeffs;
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

        // Find which record covers this epoch.
        let record_idx = ((t_sec - dir.init) / dir.intlen).floor() as usize;
        let record_idx = record_idx.min(dir.n.saturating_sub(1));

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
}

impl De440Reader {
    /// Create a reader with no data (header-only, for testing).
    pub fn empty() -> Self {
        Self {
            header: De440Header::de440_defaults(),
            segments: HashMap::new(),
            legacy_records: Vec::new(),
            is_spk: false,
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
        // Each summary has ND + ceil((NI+1)/2) = 2 + ceil(7/2) = 2 + 4 = 6
        // Wait -- let me be precise:
        // summary_size = ND + (NI + 1) / 2  (integer division, rounded up)
        // For ND=2, NI=6: (6+1)/2 = 3 (integer div), so summary_size = 2 + 3 = 5?
        // No. The NAIF spec says: SS = ND + (NI+1)/2 where / is integer division.
        // (6+1)/2 = 7/2 = 3 (integer division). So SS = 2 + 3 = 5 f64 words.
        //
        // But the integer unpacking gives NI=6 integers from ceil(NI/2)=3 f64 words.
        // The summary_size is ND + ceil((NI+1)/2). For NI=6: ceil(7/2)=4. So SS = 2+4 = 6?
        //
        // Let me think again from the NAIF DAF spec:
        // The number of double-precision numbers in each summary is: ND + (NI+1)/2
        // where the division is integer division (floor). NI=6 -> (6+1)/2 = 3.
        // Summary size = 2 + 3 = 5 f64 words.
        //
        // The unpacking: the first ND f64s are the doubles. The remaining
        // (NI+1)/2 f64s are packed integers: each f64 holds 2 i32 values
        // (the +1 accounts for odd NI with a padding int).
        // For NI=6: (6+1)/2 = 3 f64s hold 6 integers (3 pairs of i32s,
        // plus the 7th slot is padding/unused).
        //
        // Wait, (NI+1)/2 with integer div: (6+1)/2 = 3. 3 f64 words hold
        // 6 i32 values. There's exactly NI=6 integers, no padding needed.
        // If NI were 5: (5+1)/2=3, and 3 f64s hold 6 i32 slots, 5 used + 1 pad.
        let summary_size = nd as usize + (ni as usize).div_ceil(2); // 2 + 3 = 5

        let mut segments: Vec<SpkSegment> = Vec::new();
        let mut current_record = fward as u64;

        // Read all file data into memory for easier access.
        let mut file_data = vec![0u8; file_len as usize];
        file.seek(SeekFrom::Start(0))?;
        file.read_exact(&mut file_data)?;

        loop {
            if current_record < 1 {
                break;
            }
            let rec_offset = (current_record as usize - 1) * DAF_RECORD_BYTES;
            if rec_offset + DAF_RECORD_BYTES > file_data.len() {
                break;
            }

            // Summary record layout:
            // Word 0: NEXT (f64 encoding of next record number, 0.0 if none)
            // Word 1: PREV (f64 encoding of prev record number)
            // Word 2: NSUM (f64 encoding of number of summaries in this record)
            // Words 3+: summaries
            let next_f64 = endian.read_f64(&file_data, rec_offset);
            let _prev_f64 = endian.read_f64(&file_data, rec_offset + 8);
            let nsum_f64 = endian.read_f64(&file_data, rec_offset + 16);

            let next = next_f64 as i64;
            let nsum = nsum_f64 as usize;

            for i in 0..nsum {
                let sum_offset = rec_offset + 24 + i * summary_size * 8;
                if sum_offset + summary_size * 8 > file_data.len() {
                    break;
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

                segments.push(SpkSegment {
                    start_sec: dc0,
                    end_sec: dc1,
                    target: ints[0],
                    center: ints[1],
                    frame: ints[2],
                    data_type: ints[3],
                    start_word: ints[4],
                    end_word: ints[5],
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
            let seg_end_byte = (seg.end_word as usize) * 8; // 1-based word to byte offset
            if seg_end_byte < 32 || seg_end_byte > file_data.len() {
                continue;
            }

            let dir_start = seg_end_byte - 4 * 8;
            let init = endian.read_f64(&file_data, dir_start);
            let intlen = endian.read_f64(&file_data, dir_start + 8);
            let rsize = endian.read_f64(&file_data, dir_start + 16) as usize;
            let n = endian.read_f64(&file_data, dir_start + 24) as usize;

            if rsize < 5 || n == 0 || intlen <= 0.0 {
                continue;
            }

            // Number of coefficients per component:
            // record = [MID, RADIUS, X_coeffs..., Y_coeffs..., Z_coeffs...]
            // rsize = 2 + 3 * ncoeffs  =>  ncoeffs = (rsize - 2) / 3
            let ncoeffs = (rsize - 2) / 3;

            let directory = Type2Directory {
                init,
                intlen,
                rsize,
                n,
                ncoeffs,
            };

            // Read all coefficient records. SPK words are 1-based; a malformed
            // file with start_word == 0 must not underflow the usize subtraction.
            let seg_start_byte = match (seg.start_word as usize).checked_sub(1) {
                Some(w) => w * 8,
                None => continue,
            };
            let data_bytes = n * rsize * 8;
            if seg_start_byte + data_bytes > file_data.len() {
                continue;
            }

            let mut data = Vec::with_capacity(n * rsize);
            for w in 0..(n * rsize) {
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

        let mut ipt = [IptEntry {
            offset: 0,
            num_coeffs: 0,
            num_sub_intervals: 0,
        }; 13];
        for i in 0..12 {
            let base_off = 2696 + i * 12;
            ipt[i] = IptEntry {
                offset: read_i32(base_off) as usize,
                num_coeffs: read_i32(base_off + 4) as usize,
                num_sub_intervals: read_i32(base_off + 8) as usize,
            };
        }
        ipt[12] = IptEntry {
            offset: read_i32(2844) as usize,
            num_coeffs: read_i32(2848) as usize,
            num_sub_intervals: read_i32(2852) as usize,
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
            let end_word = base + entry.words_per_record(nc);
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
        })
    }

    /// Create a reader from pre-built header and records (for testing).
    pub fn with_records(header: De440Header, records: Vec<De440Record>) -> Self {
        Self {
            header,
            segments: HashMap::new(),
            legacy_records: records,
            is_spk: false,
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
    fn find_segment(&self, target: i32, center: i32, t_sec: f64) -> Option<&LoadedSegment> {
        let segs = self.segments.get(&(target, center))?;
        // Segments are sorted by start_sec; binary search.
        let idx = segs.partition_point(|s| s.descriptor.start_sec <= t_sec);
        if idx == 0 {
            // Check if the first segment covers this epoch anyway.
            if !segs.is_empty()
                && t_sec >= segs[0].descriptor.start_sec
                && t_sec <= segs[0].descriptor.end_sec
            {
                return Some(&segs[0]);
            }
            return None;
        }
        let seg = &segs[idx - 1];
        if t_sec >= seg.descriptor.start_sec && t_sec <= seg.descriptor.end_sec {
            Some(seg)
        } else {
            None
        }
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
                // Moon: need geocentric position.
                // In SPK, Moon (301) is relative to EMB (3), and Earth (399) is
                // relative to EMB (3). So Moon_geocentric = Moon_emb - Earth_emb.
                // But in DE440 SPK, there's typically no Earth (399) segment.
                // Moon (301) w.r.t. EMB (3) + Earth = EMB - Moon/(1+EMRAT).
                // Moon_geocentric = Moon_emb + Earth_emb_offset
                //   where Earth_emb_offset = Moon_emb / (1 + EMRAT) (pointing toward EMB from Earth)
                // Actually: Moon_geo = Moon_emb * (1 + 1/(1+EMRAT)) = Moon_emb * (2+EMRAT)/(1+EMRAT)
                // No. Let me think more carefully:
                //   EMB = Earth + Moon * mu_moon / (mu_earth + mu_moon)
                //   where mu ratios: EMRAT = M_earth/M_moon
                //   So: EMB = Earth + Moon / (1 + EMRAT)   [geocentric vectors]
                //   => Moon_geo = (EMB - Earth)_geo ... no this is circular.
                //
                // In the SPK: Moon_emb = Moon_pos - EMB_pos (position of Moon w.r.t. EMB).
                // Earth_emb = Earth_pos - EMB_pos.
                // We need Moon_geo = Moon_pos - Earth_pos = Moon_emb - Earth_emb.
                //
                // From barycenter definition:
                //   EMB = (M_earth * Earth + M_moon * Moon) / (M_earth + M_moon)
                //   Let R = Moon_emb = Moon - EMB.
                //   Earth_emb = Earth - EMB = -Moon * M_moon / (M_earth + M_moon) + Earth * ...
                //   Actually: Earth - EMB = -(M_moon/(M_earth+M_moon)) * (Moon - Earth)
                //     = -(1/(1+EMRAT)) * (Moon - Earth)
                //     = (1/(1+EMRAT)) * (Earth - Moon)
                //   And Moon - EMB = (M_earth/(M_earth+M_moon)) * (Moon - Earth)
                //     = (EMRAT/(1+EMRAT)) * (Moon - Earth)
                //
                // So Moon_emb = (EMRAT/(1+EMRAT)) * Moon_geo
                // => Moon_geo = Moon_emb * (1+EMRAT)/EMRAT
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

    /// Attempt to create a provider by loading a DE440 binary file.
    pub fn try_from_file(path: &Path) -> Self {
        match De440Reader::from_file(path) {
            Ok(reader) => Self::with_reader(reader),
            Err(_) => Self::fallback_only(),
        }
    }

    /// Whether DE440 data is actually loaded.
    pub fn has_de440_data(&self) -> bool {
        self.reader.has_data()
    }

    /// Check whether this provider has a real DE440 binary loaded,
    /// as opposed to falling back to VSOP87 accuracy.
    ///
    /// Callers can use this to inform users which engine is active. Both the
    /// DE440 and the VSOP87 fallback paths apply the same apparent-place
    /// correction chain (precession + nutation + annual aberration) and are at
    /// the ~1 arcsecond level; DE440's advantage is raw positional accuracy and
    /// extended validity range rather than a different correction model.
    pub fn is_de440_loaded(&self) -> bool {
        self.reader.has_data()
    }

    /// Convert a DE440 Cartesian position (J2000/ICRF equatorial) to
    /// ecliptic coordinates suitable for astrological computation.
    fn cartesian_to_ecliptic_of_date(
        &self,
        cart: &CartesianPosition,
        jd_tt: JdTT,
    ) -> EclipticPosition {
        let obliquity_j2000: f64 = 23.439_291_1_f64.to_radians();
        let cos_e = obliquity_j2000.cos();
        let sin_e = obliquity_j2000.sin();

        let ecl_x = cart.x;
        let ecl_y = cart.y * cos_e + cart.z * sin_e;
        let ecl_z = -cart.y * sin_e + cart.z * cos_e;

        let ecl_cart = CartesianPosition {
            x: ecl_x,
            y: ecl_y,
            z: ecl_z,
        };
        let mut pos = xalen_coords::cartesian_to_ecliptic(&ecl_cart);

        // J2000 ecliptic -> ecliptic-of-date: general precession in longitude,
        // then IAU 2000B nutation in longitude (true equinox of date). This
        // matches the analytical (VSOP87) path so the two engines share one
        // reference frame. Annual aberration is geocentric-only and is applied
        // by the caller (geocentric_ecliptic), not here, so the heliocentric
        // path is not wrongly aberrated.
        let t = jd_tt.julian_centuries_from_j2000();
        pos.longitude += xalen_coords::general_precession_longitude(t);
        pos.longitude += xalen_coords::nutation_2000b(t).delta_psi;

        pos.normalize()
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

            if tdb_val >= self.reader.header().jd_start && tdb_val <= self.reader.header().jd_end
                && De440Target::from_body(body).is_some() {
                    match self.reader.heliocentric_position_au(body, tdb_val) {
                        Ok(cart) => {
                            return Ok(self.cartesian_to_ecliptic_of_date(&cart, jd_tt));
                        }
                        Err(EphemerisError::BodyNotAvailable(_)) => {}
                        Err(_) => { /* fall through to VSOP87 */ }
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

            if tdb_val >= self.reader.header().jd_start && tdb_val <= self.reader.header().jd_end
                && De440Target::from_body(body).is_some() {
                    match self.reader.geocentric_position_au(body, tdb_val) {
                        Ok(cart) => {
                            // Precession + nutation (ecliptic-of-date), then annual
                            // aberration — the same apparent-place chain as the
                            // analytical path. NOTE: the body's own light-time
                            // motion is not separately iterated here (see
                            // accuracy_arcsec docs), so fast/outer bodies carry a
                            // residual up to a few arcseconds.
                            let pos = self.cartesian_to_ecliptic_of_date(&cart, jd_tt);
                            return Ok(crate::vsop::aberration_correction(pos, jd_tt).normalize());
                        }
                        Err(EphemerisError::BodyNotAvailable(_)) => {}
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
        if self.reader.has_data() {
            // DE440 raw positions are sub-milliarcsecond, but this provider's
            // APPARENT longitude applies precession + IAU 2000B nutation + annual
            // aberration WITHOUT iterating the body's own light-time motion. That
            // leaves a residual up to a few arcseconds for fast/outer bodies, so
            // we report ~1" rather than the raw-ephemeris sub-mas figure. Honest
            // over flattering. (Validity range and raw geometry remain the reason
            // to load DE440.)
            1.0
        } else {
            self.fallback.accuracy_arcsec()
        }
    }

    fn name(&self) -> &str {
        if self.reader.has_data() {
            "JPL DE440 (apparent place, no body light-time)"
        } else {
            "JPL DE440 [fallback: VSOP87] (Tier 0)"
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
    /// (VSOP87 fallback continues to work).
    pub fn with_de440(self, path: &Path) -> Self {
        match De440Reader::from_file(path) {
            Ok(reader) => {
                let provider = De440Provider::with_reader(reader);
                self.with_provider(std::sync::Arc::new(provider))
            }
            Err(_) => self,
        }
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
    fn provider_with_data_reports_de440_accuracy() {
        let header = De440Header::de440_defaults();
        let record = De440Record {
            jd_start: 2_287_184.5,
            jd_end: 2_287_184.5 + 32.0,
            coefficients: vec![0.0; 1000],
        };
        let reader = De440Reader::with_records(header, vec![record]);
        let p = De440Provider::with_reader(reader);

        assert!(p.has_de440_data());
        // Apparent-place accuracy with DE440 loaded: ~1" (body light-time not
        // iterated), better than or equal to the analytical fallback.
        assert!(p.accuracy_arcsec() <= 1.0);
        assert!(p.name().contains("DE440"));
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
}
