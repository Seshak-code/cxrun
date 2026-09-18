//! Content-addressable build cache rooted at `$CXRUN_CACHE_DIR` or `~/.cxrun/cache`.
use sha2::{Digest, Sha256};
use std::path::PathBuf;

/// Cache-format version. Bumping it invalidates every cached artifact.
pub const KEY_VERSION: u32 = 1;

/// Hex-encoded SHA-256 digest of every build input.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey(String);

impl CacheKey {
    /// Returns the 64-character hex digest as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Directory holding artifacts for this key.
    pub fn dir(&self) -> Result<PathBuf, CacheError> {
        todo!("Ok(cache_root()?.join(self.as_str()))")
    }

    /// Executable path inside `dir()` (`app.exe` on Windows, `app.bin` elsewhere).
    pub fn binary_path(&self) -> Result<PathBuf, CacheError> {
        todo!()
    }
}

/// All inputs hashed into the cache key.
#[derive(Debug)]
pub struct KeyInputs {
    /// Raw bytes of the `.cpp` source file.
    pub source: Vec<u8>,
    /// Transitive local headers sorted by path: `(path, contents)`.
    pub local_headers: Vec<(PathBuf, Vec<u8>)>,
    /// Packages from the inline manifest.
    pub packages: Vec<String>,
    /// User flags from the inline manifest.
    pub compiler_flags: Vec<String>,
    /// Baseline flags passed to the compiler.
    pub baseline_flags: Vec<String>,
    /// First line of `<compiler> --version`.
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

/// Returns `$CXRUN_CACHE_DIR`, `$HOME/.cxrun/cache`, or `%USERPROFILE%\.cxrun\cache`.
pub fn cache_root() -> Result<PathBuf, CacheError> {
    todo!()
}

/// Computes SHA-256 over the key-format version, then all `KeyInputs`.
#[must_use]
pub fn compute_key(inputs: &KeyInputs) -> CacheKey {
    let mut hasher = Sha256::new();

    hasher.update(KEY_VERSION.to_be_bytes()); // no length prefix. remove if layout changes

    feed(&mut hasher, &inputs.source); // hashing is appended onto list at the end


    for (path, contents) in &inputs.local_headers {
        feed(&mut hasher, path.as_os_str().as_encoded_bytes());
        feed(&mut hasher, contents);
    }

    for package in &inputs.packages {
        feed(&mut hasher, package.as_bytes());
    }

    for flag in &inputs.compiler_flags {
        feed(&mut hasher, flag.as_bytes());
    }

    for flag in &inputs.baseline_flags {
        feed(&mut hasher, flag.as_bytes());
    }

    feed(&mut hasher, inputs.compiler_version.as_bytes());

    CacheKey(hex::encode(hasher.finalize())) // makes sure the hash only contains filesystem safe chars
}

// length prefix to make sure identical hash collision dont occur
fn feed(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update((bytes.len() as u64).to_be_bytes());
    hasher.update(bytes);
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

/// Acquires advisory lock `<cache_root>/<key>.lock` during compilation.
pub fn acquire_lock(_key: &CacheKey) -> Result<LockGuard, CacheError> {
    todo!()
}
