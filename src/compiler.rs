//! Host toolchain discovery, baseline flags, header scanning, compilation.
//!
//! Process rules (security): spawn via `std::process::Command` with explicit
//! argument arrays. NEVER build a shell string or invoke through `sh -c` /
//! `cmd.exe` — inline manifest flags are user-controlled input.

use std::path::{Path, PathBuf};

use crate::parser::ScriptMetadata;

/// A located C++ compiler plus its version fingerprint.
#[derive(Debug)]
pub struct Toolchain {
    /// Absolute path to `clang++` (or the `g++` fallback).
    pub compiler: PathBuf,
    /// First line of `<compiler> --version`; mixed into the cache key.
    pub version: String,
}

/// Toolchain and compilation failures.
#[derive(Debug)]
pub enum CompileError {
    /// No usable compiler found in PATH.
    NoCompiler,
    /// Compiler exited non-zero; payload is its captured stderr.
    CompileFailed(String),
    /// Process spawn or pipe failure.
    Io(std::io::Error),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoCompiler => {
                let hint = if cfg!(target_os = "macos") {
                    "xcode-select --install"
                } else if cfg!(target_os = "windows") {
                    "winget install LLVM.LLVM"
                } else {
                    "sudo apt install clang   # or your distro's equivalent"
                };
                write!(
                    f,
                    "no C++ compiler found in PATH. Install one with:\n  {hint}"
                )
            }
            Self::CompileFailed(stderr) => write!(f, "compilation failed:\n{stderr}"),
            Self::Io(e) => write!(f, "failed to run compiler: {e}"),
        }
    }
}

impl std::error::Error for CompileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for CompileError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// Probe order: `clang++`, then `g++` on Linux only (require `-dumpversion`
/// major >= 11 for full C++20). Fingerprint: first line of `<bin> --version`.
/// Use the `which` crate for PATH lookup.
pub fn discover_toolchain() -> Result<Toolchain, CompileError> {
    todo!()
}

/// Baseline flags from the 80/20 compatibility matrix (see project docs).
///
/// - all platforms:  `-std=c++20 -O2`
/// - macOS:          `-mmacosx-version-min=12.0` (Apple clang already targets
///   the host ARM baseline; no `-march` needed here)
/// - Linux x86_64:   `-march=x86-64-v2` (SSE4.2/POPCNT floor since ~2008;
///   NEVER `-march=native` — cached binaries must not `SIGILL` on other CPUs)
/// - Linux arm64:    `-march=armv8-a+simd`
/// - Windows:        static CRT arrives with vcpkg triplets in Milestone 3
pub fn baseline_flags() -> &'static [&'static str] {
    if cfg!(target_os = "macos") {
        &["-std=c++20", "-O2", "-mmacosx-version-min=12.0"]
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        &["-std=c++20", "-O2", "-march=x86-64-v2"]
    } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        &["-std=c++20", "-O2", "-march=armv8-a+simd"]
    } else {
        &["-std=c++20", "-O2"]
    }
}

/// Transitive LOCAL headers of `source`, via `<compiler> -MM <source>`.
///
/// `-MM` emits a Makefile rule and omits system headers:
/// `main.o: main.cpp foo.h sub/bar.h \`<newline>`  baz.h`
/// Parse it: skip the target token (before `:`) and the source itself, handle
/// `\` line continuations, resolve relative paths against the source's parent
/// directory, sort for determinism.
///
/// NOTE: `-MM` needs `-I` flags once packages exist (Milestone 3); for
/// dependency-free M2 scripts it works as-is.
pub fn discover_local_headers(
    _source: &Path,
    _toolchain: &Toolchain,
) -> Result<Vec<PathBuf>, CompileError> {
    todo!()
}

/// Compile `source` to `output`.
///
/// Argument order: `baseline_flags()`, then `meta.compiler_flags`, then
/// (Milestone 3) `-I`/`-L`/`-l`, then `source -o output`. Capture stderr; on
/// failure return `CompileFailed(stderr)` and leave NO file at `output`.
/// Compile to a temp file in the same directory and rename into place so
/// concurrent readers never observe a partial binary.
pub fn compile(
    _source: &Path,
    _output: &Path,
    _meta: &ScriptMetadata,
    _toolchain: &Toolchain,
) -> Result<(), CompileError> {
    todo!()
}
