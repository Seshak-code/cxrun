//! Host toolchain discovery, baseline flags, header scanning, compilation.
//!
//! Process rules (security): spawn via `std::process::Command` with explicit
//! argument arrays. NEVER build a shell string or invoke through `sh -c` /
//! `cmd.exe` — inline manifest flags are user-controlled input.

use std::path::{Path, PathBuf};
use std::process::Command;

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
    /// A located compiler could not be run, or its version probe printed
    /// nothing usable.
    ProbeFailed {
        /// Compiler binary that was found.
        compiler: PathBuf,
        /// Why the probe failed.
        error: std::io::Error,
    },
    /// A located `g++` is older than the GCC 11 C++20 baseline.
    CompilerTooOld {
        /// Compiler binary that was found.
        compiler: PathBuf,
        /// Raw `-dumpversion` output that failed the gate.
        found: String,
    },
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
            Self::ProbeFailed { compiler, error } => write!(
                f,
                "found compiler at {} but could not read its version: {error}",
                compiler.display()
            ),
            Self::CompilerTooOld { compiler, found } => write!(
                f,
                "g++ at {} is version {found}, but C++20 requires GCC 11 or newer",
                compiler.display()
            ),
            Self::Io(e) => write!(f, "failed to run compiler: {e}"),
        }
    }
}

impl std::error::Error for CompileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::ProbeFailed { error, .. } => Some(error),
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
///
/// # Errors
///
/// `NoCompiler` when no probe finds a binary in PATH, `ProbeFailed` when a
/// located compiler cannot be run or prints no usable version, and
/// `CompilerTooOld` when `g++` predates the GCC 11 C++20 baseline.
pub fn discover_toolchain() -> Result<Toolchain, CompileError> {
    if let Ok(clang) = which::which("clang++") {
        let version = probe_line(&clang, "--version")?;
        return Ok(Toolchain {
            compiler: clang,
            version,
        });
    }

    if cfg!(target_os = "linux") {
        if let Ok(gpp) = which::which("g++") {
            let dump = probe_line(&gpp, "-dumpversion")?;
            let Some(major) = dump_major(&dump) else {
                return Err(CompileError::ProbeFailed {
                    compiler: gpp,
                    error: std::io::Error::other(format!(
                        "`-dumpversion` printed {dump:?}, not a version number"
                    )),
                });
            };
            if major < 11 {
                return Err(CompileError::CompilerTooOld {
                    compiler: gpp,
                    found: dump,
                });
            }
            let version = probe_line(&gpp, "--version")?;
            return Ok(Toolchain {
                compiler: gpp,
                version,
            });
        }
    }

    Err(CompileError::NoCompiler)
}

fn probe_line(compiler: &Path, arg: &str) -> Result<String, CompileError> {
    let out =
        Command::new(compiler)
            .arg(arg)
            .output()
            .map_err(|error| CompileError::ProbeFailed {
                compiler: compiler.to_path_buf(),
                error,
            })?;

    let line = String::from_utf8_lossy(&out.stdout)
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_owned();

    if line.is_empty() {
        return Err(CompileError::ProbeFailed {
            compiler: compiler.to_path_buf(),
            error: std::io::Error::other(format!("`{arg}` printed no usable output")),
        });
    }

    Ok(line)
}

fn dump_major(dump: &str) -> Option<u32> {
    dump.trim()
        .split('.')
        .next()
        .and_then(|major| major.trim().parse().ok())
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
