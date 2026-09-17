//! cxrun — zero-config single-file C++20 script runner.
//!
//! Pipeline: parse inline `// [deps]` metadata → hash all build inputs →
//! check the content-addressable cache → compile via host clang++ on a miss
//! → exec the cached binary, forwarding stdio and the exit code.
//!
//! Crate-wide rules (see project docs):
//! - No `unsafe` (enforced by the attribute below).
//! - No `Box<dyn Error>`: every module exposes a concrete error enum.
//! - Borrow, don't clone: pass `&str` / `&Path`; allocate only at boundaries.
//! - No factories / dynamic dispatch for simple structs; plain `impl` blocks.
//! - Expression-based returns; use `?` instead of verbose `match` on `Result`.
#![forbid(unsafe_code)]

pub mod cache;
pub mod compiler;
pub mod parser;

/// Top-level error aggregating every subsystem failure.
///
/// Each variant wraps the concrete error of the module that produced it, so the
/// originating subsystem is recoverable by `match` rather than by string
/// inspection. `Display` forwards the inner message verbatim; `Error::source`
/// chains the wrapped cause so diagnostic tooling can walk the failure.
#[derive(Debug)]
pub enum CxrunError {
    /// The inline `// [deps]` manifest could not be parsed.
    Parse(parser::ParseError),
    /// The cache root, advisory lock, or key path could not be resolved.
    Cache(cache::CacheError),
    /// Toolchain discovery or the compilation itself failed.
    Compile(compiler::CompileError),
    /// I/O failed outside a subsystem-specific case (e.g. reading the source file).
    Io(std::io::Error),
}

impl std::fmt::Display for CxrunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(e) => write!(f, "{e}"),
            Self::Cache(e) => write!(f, "{e}"),
            Self::Compile(e) => write!(f, "{e}"),
            Self::Io(e) => write!(f, "I/O error: {e}"),
        }
    }
}

impl std::error::Error for CxrunError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(e) => Some(e),
            Self::Cache(e) => Some(e),
            Self::Compile(e) => Some(e),
            Self::Io(e) => Some(e),
        }
    }
}

impl From<parser::ParseError> for CxrunError {
    fn from(e: parser::ParseError) -> Self {
        Self::Parse(e)
    }
}

impl From<cache::CacheError> for CxrunError {
    fn from(e: cache::CacheError) -> Self {
        Self::Cache(e)
    }
}

impl From<compiler::CompileError> for CxrunError {
    fn from(e: compiler::CompileError) -> Self {
        Self::Compile(e)
    }
}

impl From<std::io::Error> for CxrunError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
