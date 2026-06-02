//! Optional automatic provisioning of the public NASA NAIF `de440s.bsp` kernel.
//!
//! This module is compiled ONLY when the `kernel-autodownload` cargo feature is
//! enabled. With the feature off the base crate stays offline and pulls in no
//! network dependency; the manual [`De440Provider::try_from_file`] path is the
//! only way to load a kernel.
//!
//! When enabled, [`ensure_de440s_kernel`] fetches the small (~32 MB) trimmed
//! DE440 short-span kernel `de440s.bsp` from the public NAIF archive on first
//! use, stores it in the per-OS cache directory, and reuses the on-disk copy on
//! every subsequent call. [`De440Provider::from_auto_cache`] wraps that into a
//! one-call constructor that yields a sub-arcsecond, kernel-backed provider with
//! no manual file handling.
//!
//! # Integrity
//!
//! Two independent checks guard the cached kernel:
//!
//! 1. **Structural provenance.** The downloaded bytes are parsed with
//!    [`De440Reader::from_file`], which confirms a valid NAIF DAF/SPK layout and
//!    reads the embedded JPL label from the DAF comment area. A file is only
//!    accepted into the cache if it confirms as `DE440`. A truncated download, a
//!    proxy error page, or a non-DE `.bsp` is rejected — it never silently
//!    becomes the cached kernel.
//! 2. **Optional SHA-256.** If the caller supplies an expected digest (via
//!    [`KernelFetch::expected_sha256`] or the `XALEN_DE440S_SHA256` environment
//!    variable) the freshly downloaded bytes are checked against it before being
//!    accepted. This is opt-in because NAIF re-releases generic kernels over
//!    time and does not publish a permanent per-file digest in the archive; the
//!    structural provenance check above is always applied regardless.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::de440::De440Reader;
use crate::provider::EphemerisError;

/// Public NAIF archive URL for the trimmed short-span DE440 kernel.
pub const DE440S_URL: &str =
    "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440s.bsp";

/// File name used for the cached kernel inside the cache directory.
pub const DE440S_FILENAME: &str = "de440s.bsp";

/// Lower sanity bound on the de440s.bsp size (bytes). The real kernel is ~32 MB;
/// anything materially smaller is a truncated download or an error page, not a
/// kernel. Used only as a cheap pre-parse guard — the authoritative acceptance
/// gate is the structural DE440 provenance check.
const MIN_PLAUSIBLE_KERNEL_BYTES: u64 = 8 * 1024 * 1024;

/// Environment variable that overrides the cache directory the kernel is stored
/// in. When unset, the per-OS cache directory (`dirs::cache_dir()`) is used.
pub const CACHE_DIR_ENV: &str = "XALEN_KERNEL_CACHE_DIR";

/// Environment variable holding an expected lowercase hex SHA-256 of the kernel.
/// When set, a freshly downloaded kernel is verified against it before caching.
pub const SHA256_ENV: &str = "XALEN_DE440S_SHA256";

/// Configurable kernel-provisioning request.
///
/// The defaults match [`ensure_de440s_kernel`]: NAIF `de440s.bsp`, the per-OS
/// cache directory, no required SHA-256 (structural provenance only).
#[derive(Debug, Clone)]
pub struct KernelFetch {
    url: String,
    file_name: String,
    cache_dir: Option<PathBuf>,
    expected_sha256: Option<String>,
}

impl Default for KernelFetch {
    fn default() -> Self {
        Self {
            url: DE440S_URL.to_string(),
            file_name: DE440S_FILENAME.to_string(),
            cache_dir: None,
            expected_sha256: None,
        }
    }
}

impl KernelFetch {
    /// A fetch request for the default NAIF `de440s.bsp` kernel.
    pub fn de440s() -> Self {
        Self::default()
    }

    /// Override the source URL (e.g. an internal mirror of the NAIF archive).
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = url.into();
        self
    }

    /// Override the cache directory the kernel is stored in. Takes precedence
    /// over the [`CACHE_DIR_ENV`] environment variable.
    pub fn cache_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.cache_dir = Some(dir.into());
        self
    }

    /// Require the downloaded bytes to match this lowercase hex SHA-256 digest
    /// before they are accepted into the cache. Takes precedence over the
    /// [`SHA256_ENV`] environment variable.
    pub fn expected_sha256(mut self, hex: impl Into<String>) -> Self {
        self.expected_sha256 = Some(hex.into());
        self
    }

    /// Resolve the directory the kernel is (or will be) cached in.
    fn resolve_cache_dir(&self) -> Result<PathBuf, EphemerisError> {
        if let Some(dir) = &self.cache_dir {
            return Ok(dir.clone());
        }
        if let Ok(dir) = std::env::var(CACHE_DIR_ENV) {
            if !dir.is_empty() {
                return Ok(PathBuf::from(dir));
            }
        }
        let base = dirs::cache_dir().ok_or_else(|| {
            EphemerisError::ComputationFailed(
                "could not determine an OS cache directory; set the \
                 XALEN_KERNEL_CACHE_DIR environment variable to choose one"
                    .to_string(),
            )
        })?;
        Ok(base.join("xalen-ephem"))
    }

    /// Resolve the on-disk path of the cached kernel.
    fn cached_path(&self) -> Result<PathBuf, EphemerisError> {
        Ok(self.resolve_cache_dir()?.join(&self.file_name))
    }

    /// The expected SHA-256 to verify against, from the builder or the
    /// environment, whichever is set (builder wins).
    fn resolved_sha256(&self) -> Option<String> {
        if let Some(h) = &self.expected_sha256 {
            return Some(h.to_ascii_lowercase());
        }
        std::env::var(SHA256_ENV)
            .ok()
            .filter(|h| !h.is_empty())
            .map(|h| h.to_ascii_lowercase())
    }

    /// Ensure the kernel exists in the cache, fetching it once if necessary, and
    /// return its on-disk path.
    ///
    /// Fast path: if a confirmed-`DE440` kernel is already cached, no network
    /// access happens. Otherwise the kernel is downloaded, verified (size,
    /// optional SHA-256, then structural DE440 provenance), atomically renamed
    /// into place, and the path returned.
    pub fn ensure(&self) -> Result<PathBuf, EphemerisError> {
        let path = self.cached_path()?;

        // Fast path: a valid cached kernel already present.
        if path.exists() && is_confirmed_de440(&path) {
            return Ok(path);
        }

        let dir = self.resolve_cache_dir()?;
        fs::create_dir_all(&dir)?;

        // Download into a sibling temp file first so a partial/failed download
        // never clobbers a good cached kernel.
        let tmp = dir.join(format!("{}.partial", self.file_name));
        let bytes = download(&self.url)?;

        if (bytes.len() as u64) < MIN_PLAUSIBLE_KERNEL_BYTES {
            return Err(EphemerisError::InvalidFormat(format!(
                "downloaded {} from {} is only {} bytes (< {} MB); \
                 likely a truncated download or an error page, not a kernel",
                self.file_name,
                self.url,
                bytes.len(),
                MIN_PLAUSIBLE_KERNEL_BYTES / (1024 * 1024),
            )));
        }

        if let Some(expected) = self.resolved_sha256() {
            let actual = sha256_hex(&bytes);
            if actual != expected {
                return Err(EphemerisError::InvalidFormat(format!(
                    "SHA-256 mismatch for {}: expected {}, got {}",
                    self.file_name, expected, actual
                )));
            }
        }

        fs::write(&tmp, &bytes)?;

        // Structural provenance gate: only a file that parses as a real DAF/SPK
        // and confirms as DE440 is accepted into the cache.
        if !is_confirmed_de440(&tmp) {
            let _ = fs::remove_file(&tmp);
            return Err(EphemerisError::InvalidFormat(format!(
                "downloaded {} did not confirm as a DE440 NAIF kernel \
                 (structural/provenance check failed); not cached",
                self.file_name
            )));
        }

        // Atomic publish into the final cache path.
        fs::rename(&tmp, &path)?;
        Ok(path)
    }
}

/// Parse a candidate file and report whether it confirms as a real `DE440`
/// NAIF kernel. Any read/parse failure is treated as "not confirmed".
fn is_confirmed_de440(path: &Path) -> bool {
    match De440Reader::from_file(path) {
        Ok(reader) => reader.kernel_id() == Some("DE440"),
        Err(_) => false,
    }
}

/// Compute the lowercase hex SHA-256 of `bytes`.
fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    use std::fmt::Write as _;
    let digest = Sha256::digest(bytes);
    let mut s = String::with_capacity(64);
    for b in digest {
        // Infallible: writing to a String never errors.
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Blocking download of `url` into a byte buffer.
fn download(url: &str) -> Result<Vec<u8>, EphemerisError> {
    let resp = ureq::get(url).call().map_err(|e| {
        EphemerisError::ComputationFailed(format!("kernel download from {url} failed: {e}"))
    })?;

    let mut buf = Vec::new();
    resp.into_reader()
        .read_to_end(&mut buf)
        .map_err(EphemerisError::IoError)?;
    Ok(buf)
}

/// Ensure the public NAIF `de440s.bsp` kernel is available locally, fetching it
/// once into the per-OS cache directory if needed, and return its path.
///
/// Requires the `kernel-autodownload` feature. Subsequent calls reuse the
/// cached file with no network access. The cache directory can be overridden
/// with the [`CACHE_DIR_ENV`] environment variable, and an expected SHA-256 can
/// be enforced with [`SHA256_ENV`].
///
/// For full control (custom mirror URL, required digest, explicit cache
/// directory) use [`KernelFetch`].
pub fn ensure_de440s_kernel() -> Result<PathBuf, EphemerisError> {
    KernelFetch::de440s().ensure()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_hex_is_lowercase_64_chars() {
        // SHA-256 of the empty input is a well-known constant.
        let h = sha256_hex(b"");
        assert_eq!(
            h,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(h.len(), 64);
        assert!(
            h.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        );
    }

    #[test]
    fn sha256_hex_known_vector() {
        // "abc" SHA-256 — FIPS 180-4 published test vector.
        let h = sha256_hex(b"abc");
        assert_eq!(
            h,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn builder_overrides_take_precedence() {
        let f = KernelFetch::de440s()
            .url("https://example.invalid/mirror/de440s.bsp")
            .cache_dir("/tmp/xalen-test-cache")
            .expected_sha256("ABCDEF");
        assert_eq!(f.url, "https://example.invalid/mirror/de440s.bsp");
        assert_eq!(
            f.cache_dir.as_deref(),
            Some(Path::new("/tmp/xalen-test-cache"))
        );
        // expected_sha256 is normalised to lowercase on resolution.
        assert_eq!(f.resolved_sha256().as_deref(), Some("abcdef"));
    }

    #[test]
    fn explicit_cache_dir_resolves_path() {
        let f = KernelFetch::de440s().cache_dir("/tmp/xalen-kc-test");
        let p = f.cached_path().unwrap();
        assert_eq!(p, Path::new("/tmp/xalen-kc-test/de440s.bsp"));
    }

    #[test]
    fn missing_file_is_not_confirmed_de440() {
        assert!(!is_confirmed_de440(Path::new("/nonexistent/de440s.bsp")));
    }

    #[test]
    fn tiny_file_is_not_confirmed_de440() {
        // A small non-kernel file must never confirm as DE440.
        let p = std::env::temp_dir().join("xalen-kc-not-a-kernel.bin");
        fs::write(&p, b"this is not a DAF/SPK kernel").unwrap();
        assert!(!is_confirmed_de440(&p));
        let _ = fs::remove_file(&p);
    }

    /// End-to-end provisioning: only runs when a pre-fetched kernel exists at
    /// `/tmp/de440s.bsp` (the repo's kernel-gated test convention). Never forces
    /// a network fetch in CI.
    #[test]
    fn ensure_uses_existing_cached_kernel_without_network() {
        let prefetched = Path::new("/tmp/de440s.bsp");
        if !prefetched.exists() || !is_confirmed_de440(prefetched) {
            eprintln!(
                "Skipping kernel-cache end-to-end test: no confirmed DE440 at /tmp/de440s.bsp"
            );
            return;
        }
        // Seed an isolated cache dir with the prefetched kernel, then prove
        // `ensure` returns it WITHOUT touching the network (an unreachable URL).
        let dir = std::env::temp_dir().join("xalen-kc-e2e");
        let _ = fs::create_dir_all(&dir);
        let cached = dir.join(DE440S_FILENAME);
        fs::copy(prefetched, &cached).unwrap();

        let f = KernelFetch::de440s()
            .cache_dir(&dir)
            .url("https://127.0.0.1:1/should-not-be-reached.bsp");
        let got = f.ensure().expect("cached kernel should be returned");
        assert_eq!(got, cached);

        let _ = fs::remove_dir_all(&dir);
    }

    /// `De440Provider::from_auto_cache` honors `XALEN_KERNEL_CACHE_DIR` and loads
    /// a kernel-backed provider from a pre-seeded cache (no network). Kernel-
    /// gated on a pre-fetched `/tmp/de440s.bsp`.
    #[test]
    fn from_auto_cache_loads_seeded_kernel() {
        let prefetched = Path::new("/tmp/de440s.bsp");
        if !prefetched.exists() || !is_confirmed_de440(prefetched) {
            eprintln!("Skipping from_auto_cache test: no confirmed DE440 at /tmp/de440s.bsp");
            return;
        }
        let dir = std::env::temp_dir().join("xalen-kc-from-auto");
        let _ = fs::create_dir_all(&dir);
        fs::copy(prefetched, dir.join(DE440S_FILENAME)).unwrap();

        // SAFETY: single-threaded test; restore the prior value afterward.
        let prev = std::env::var(CACHE_DIR_ENV).ok();
        unsafe {
            std::env::set_var(CACHE_DIR_ENV, &dir);
        }

        let provider = crate::de440::De440Provider::from_auto_cache()
            .expect("seeded cache should yield a kernel-backed provider");
        assert!(
            provider.is_de440_loaded(),
            "provider should report DE440 loaded"
        );

        unsafe {
            match prev {
                Some(v) => std::env::set_var(CACHE_DIR_ENV, v),
                None => std::env::remove_var(CACHE_DIR_ENV),
            }
        }
        let _ = fs::remove_dir_all(&dir);
    }
}
