//! Inline manifest parser (PEP 723-style) — pure string processing, zero I/O.
//!
//! Grammar (line-oriented, whitespace-tolerant):
//! ```text
//! // [deps]            opens the block (leading/trailing whitespace ok)
//! // fmt               package entry         -> ScriptMetadata::packages
//! // -O3               flag entry (dash-led) -> ScriptMetadata::compiler_flags
//! // [end]             closes the block
//! ```
//! - The block also closes implicitly at the first line that is not a `//`
//!   comment.
//! - Only the FIRST `[deps]` block is parsed; later ones are ignored.
//! - Entry text is everything after `//`, trimmed; empty entries are skipped.
//!
//! Implementation constraints: std only, no `unwrap`/`expect`, no regex.
//! Minimize allocations: one `String` per collected entry, built by trimming
//! string slices — no intermediate `Vec<&str>`.

/// Metadata extracted from a source file's `// [deps]` block.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ScriptMetadata {
    pub packages: Vec<String>,
    pub compiler_flags: Vec<String>,
}

/// Failures while parsing the inline manifest.
#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    /// Source contained no bytes at all.
    EmptySource,
    /// `// [deps]` opened at this 1-based line was never closed by `// [end]`
    /// or a code line before EOF.
    UnterminatedBlock(usize),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptySource => write!(f, "source file is empty"),
            Self::UnterminatedBlock(line) => {
                write!(f, "`// [deps]` block opened at line {line} is never closed")
            }
        }
    }
}

impl std::error::Error for ParseError {}

/// Parse the inline manifest from `content`.
///
/// Returns `Ok(ScriptMetadata::default())` when the source has no `[deps]`
/// block. Entry order within the block is preserved.
pub fn parse_manifest(_content: &str) -> Result<ScriptMetadata, ParseError> {
    todo!("scan lines; on `[deps]` collect entries until `[end]`, a code line, or EOF (error)")
}
