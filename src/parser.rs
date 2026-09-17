//! Inline `// [deps]` manifest parser — pure string processing, zero I/O.
/// Manifest metadata parsed from a source file's `// [deps]` block.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScriptMetadata {
    /// Third-party package names (non-dash entries, e.g. `fmt`).
    pub packages: Vec<String>,
    /// Compiler flags (dash-led entries, e.g. `-O3`).
    pub compiler_flags: Vec<String>,
}
// implementation of Lexer; answers what the type of line is
// State machine in `parse_manifest` function answers what the line means
// borrows slice of input string instead of copying with lifetime 'a so theres no heap allocation/ memory cost
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Line<'a> {
    OpenBlock,
    CloseBlock,
    EmptyComment,
    Flag(&'a str),
    Package(&'a str),
    Blank, // if line only has whitespace, ignores when inside block so it doesn't close early.
    NonComment, // found start of code, closes an open block.
}

fn classify(line: &str) -> Line<'_> {
    // trim strips whitespace around packages and comments (payload)
    // gives state machine exact state no guessing

    let trimmed = line.trim();

    //checks if line is whitespace only
    if trimmed.is_empty() {
        return Line::Blank;
    }

    let Some(payload) = trimmed.strip_prefix("//") else {
        return Line::NonComment;
    };

    let payload = payload.trim();

    match payload {
        "" => Line::EmptyComment,
        "[deps]" => Line::OpenBlock,
        "[end]" => Line::CloseBlock,
        flag if flag.starts_with('-') => Line::Flag(flag),
        other => Line::Package(other),
    }
}

// this is the state of scanner relative to the '[deps]` block.
#[derive(Debug, Clone, Copy)]
enum BlockState {
    Before,
    Inside { open_line: usize }, // holds the line `[deps]` block is on, helps error function if never closed
    After,
}

/// Failures while parsing the inline manifest.
#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    /// Source contained no bytes at all.
    EmptySource,
    /// Block opened at this 1-based line was never closed.
    UnterminatedBlock(usize),
    /// inside block  that isn't `[deps]` or `[end]`.
    UnknownDirective {
        /// line of the offending comment.
        line: usize,
    },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptySource => write!(f, "source file is empty"),
            Self::UnterminatedBlock(line) => {
                write!(f, "`// [deps]` block opened at line {line} is never closed")
            }
            Self::UnknownDirective { line } => {
                write!(
                    f,
                    "unknown directive on line {line}; only `[deps]` and `[end]` are known"
                )
            }
        }
    }
}

impl std::error::Error for ParseError {}

/// Reads `// [deps]` from C++ source text to find packages and compiler flags.
///
/// Returns empty metadata if the file does not have a `// [deps]` block.
///
/// # Errors
///
/// Returns an error if:
/// - The file is completely empty (`EmptySource`).
/// - A `// [deps]` block is opened but never closed before the end of the file (`UnterminatedBlock`).
/// - An unknown directive like `// [foo]` is used inside the block (`UnknownDirective`).
pub fn parse_manifest(content: &str) -> Result<ScriptMetadata, ParseError> {
    if content.is_empty() {
        return Err(ParseError::EmptySource);
    }

    // strips Windows UTF-8 BOM so the first line's `//` isn't hidden
    let content = content.strip_prefix('\u{FEFF}').unwrap_or(content);

    let mut metadata = ScriptMetadata::default();
    let mut state = BlockState::Before;

    // normalizes `\r\n` across OSs and tracks starting line numbers for errors
    for (idx, line_raw) in content.lines().enumerate() {
        let line_number = idx + 1;
        let line = classify(line_raw);

        // state switching based on scanner's position in file
        match state {
            BlockState::After => break, // stops scanning once the first block ends; rest of file is ignored
            BlockState::Before => match line {
                // logic state before desicion tree of states to switch too
                Line::OpenBlock => {
                    state = BlockState::Inside {
                        open_line: line_number,
                    }
                }
                //without `_` so future Line variants will trigger a compile error
                Line::CloseBlock
                | Line::EmptyComment
                | Line::Blank
                | Line::Flag(_)
                | Line::Package(_)
                | Line::NonComment => {}
            },
            BlockState::Inside { .. } => match line {
                // blank lines, empty commentsm and repeated dependencies don't close block
                Line::Blank | Line::EmptyComment | Line::OpenBlock => {}
                // code starts or explicit "[end]" listed to end block
                Line::CloseBlock | Line::NonComment => state = BlockState::After,
                // caches typos and returns error
                Line::Package(name) if name.starts_with('[') => {
                    return Err(ParseError::UnknownDirective { line: line_number });
                }
                Line::Flag(flag) => metadata.compiler_flags.push(flag.to_string()),
                Line::Package(name) => metadata.packages.push(name.to_string()),
            },
        }
    }
    // if end of file is reached without close returns error and points to start
    if let BlockState::Inside { open_line } = state {
        return Err(ParseError::UnterminatedBlock(open_line));
    }

    Ok(metadata) // return value for the function
}

#[cfg(test)]
mod tests {

    use super::{classify, Line};

    #[test]
    fn recognizes_every_variant() {
        assert_eq!(classify("// [deps]"), Line::OpenBlock);
        assert_eq!(classify("// [end]"), Line::CloseBlock);
        assert_eq!(classify("// fmt"), Line::Package("fmt"));
        assert_eq!(classify("// -O3"), Line::Flag("-O3"));
        assert_eq!(classify("//"), Line::EmptyComment);
        assert_eq!(classify("int main() {}"), Line::NonComment);
        assert_eq!(classify(""), Line::Blank);
        assert_eq!(classify("   "), Line::Blank);
        assert_eq!(classify("\t"), Line::Blank);
        assert_eq!(classify("   //   "), Line::EmptyComment);
    }

    #[test]
    fn checks_for_correct_payload_around_whitespace() {
        assert_eq!(classify("   //   -Wall  "), Line::Flag("-Wall"));
        assert_eq!(classify("// [deps] x"), Line::Package("[deps] x"));
        assert_eq!(classify("/// doc"), Line::Package("/ doc"));
    }

    #[test]
    fn borrows_from_input_instead_of_copying() {
        let source = String::from("// fmt");

        let Line::Package(name) = classify(&source) else {
            panic!("`// fmt` should classify as a package");
        };

        let offset = name.as_ptr() as usize - source.as_ptr() as usize;

        assert_eq!(offset, 3, "payload should start after `// `");
        assert_eq!(&source[offset..], "fmt");
    }
}
