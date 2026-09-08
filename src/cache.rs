//! Content-addressable build cache.
//!
//! Layout (root is `$CXRUN_CACHE_DIR` when set — used by integration tests —
//! otherwise `~/.cxrun/cache`):
//! ```text
//! <root>/<sha256>/app.bin    compiled binary (app.exe on Windows)
//! <root>/<sha256>.lock       advisory lock held during compilation
//! ```
//!
//! Pure/impure split: `compute_key` is a pure function (unit-test it in
//! memory); everything that touches the filesystem returns `CacheError`.

use std::path::PathBuf;

/// Hex-encoded SHA-256 of every build input. Newtype so a hash can never be
/// passed where a path or flag belongs — zero runtime cost.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey(String);

impl CacheKey {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Directory that holds every artifact for this key.
    pub fn dir(&self) -> Result<PathBuf, CacheError> {
        todo!("Ok(cache_root()?.join(self.as_str()))")
    }

    /// Final executable path inside `dir()` (`app.exe` on Windows,
    /// `app.bin` elsewhere).
    pub fn binary_path(&self) -> Result<PathBuf, CacheError> {
        todo!()
    }
}

/// Everything that must change the cache key. Callers read bytes from disk
/// first so `compute_key` stays pure.
#[derive(Debug)]
pub struct KeyInputs {
    /// Raw bytes of the `.cpp` source file.
    pub source: Vec<u8>,
    /// `(path, contents)` of every transitive LOCAL header, sorted by path
    /// for determinism (see `compiler::discover_local_headers`).
    pub local_headers: Vec<(PathBuf, Vec<u8>)>,
    /// Packages from the inline manifest.
    pub packages: Vec<String>,
    /// User flags from the inline manifest.
    pub compiler_flags: Vec<String>,
    /// Baseline flags actually passed to the compiler.
    pub baseline_flags: Vec<String>,
    /// First line of `<compiler> --version` — invalidates the cache when the
    /// toolchain is upgraded.
    pub compiler_version: String,
}

/// Cache subsystem failures.
#[derive(Debug)]
pub enum CacheError {
    /// Neither `$CXRUN_CACHE_DIR`, `$HOME`, nor `%USERPROFILE%` is set.
    NoHomeDir,
    /// Filesystem failure.
    Io(std::io::Error),
    /// Lock file still held after all retries (payload: lock path).
    LockContended(PathBuf),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoHomeDir => {
                write!(f, "could not determine cache directory ($CXRUN_CACHE_DIR, $HOME and %USERPROFILE% are all unset)")
            }
            Self::Io(e) => write!(f, "cache I/O error: {e}"),
            Self::LockContended(path) => {
                write!(f, "cache lock held by another process: {}", path.display())
            }
        }
    }
}

impl std::error::Error for CacheError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for CacheError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// Cache root directory, creating it if missing.
///
/// Resolution order: `$CXRUN_CACHE_DIR` (used directly as the cache dir) →
/// `$HOME/.cxrun/cache` (Unix) → `%USERPROFILE%\.cxrun\cache` (Windows).
pub fn cache_root() -> Result<PathBuf, CacheError> {
    todo!()
}

/// Pure SHA-256 over all `KeyInputs` fields in a fixed order.
///
/// Feed each byte run with a length prefix (e.g. `(len as u64).to_be_bytes()`)
/// so concatenations cannot collide: `("ab", "c")` must differ from
/// `("a", "bc")`. Return `CacheKey(hex::encode(hasher.finalize()))`.
pub fn compute_key(_inputs: &KeyInputs) -> CacheKey {
    todo!()
}

/// RAII guard: removes the lock file on drop.
pub struct LockGuard {
    path: PathBuf,
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Advisory lock on `<cache_root>/<key>.lock`, held across compilation so two
/// concurrent cxrun processes never write the same binary.
///
/// Contract: create the key's cache dir first, then retry
/// `OpenOptions::create_new(true)` every ~50ms for up to ~30s. Treat locks
/// older than ~10 minutes as stale (crashed holder) and break them.
/// Std-only is fine for now; if you want true `flock`/`LockFileEx` semantics,
/// add the `fs4` crate and update the dependency budget note in Cargo.toml.
pub fn acquire_lock(_key: &CacheKey) -> Result<LockGuard, CacheError> {
    todo!()
}
