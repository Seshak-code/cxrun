//! Pure in-memory tests: parser grammar + hash invariants. NO disk I/O.
//!
//! These are the executable spec for the `todo!()` stubs — they fail until
//! the corresponding implementations land. That is the scaffold contract.

use std::path::PathBuf;

use cxrun::cache::{compute_key, KeyInputs};
use cxrun::parser::{parse_manifest, ParseError};

// ---------- parser ----------

#[test]
fn parses_valid_deps_and_flags() {
    let input = "// [deps]\n// fmt\n// -O3\n// [end]\nint main() {}";
    let meta = parse_manifest(input).expect("Should parse valid block");
    assert_eq!(meta.packages, vec!["fmt"]);
    assert_eq!(meta.compiler_flags, vec!["-O3"]);
}

#[test]
fn handles_source_without_deps() {
    let input = "int main() { return 0; }";
    let meta = parse_manifest(input).expect("Source without deps must succeed");
    assert!(meta.packages.is_empty());
    assert!(meta.compiler_flags.is_empty());
}

#[test]
fn rejects_unterminated_block() {
    let input = "// [deps]\n// fmt\n// still inside the block at EOF";
    assert_eq!(parse_manifest(input), Err(ParseError::UnterminatedBlock(1)));
}

#[test]
fn block_closes_at_first_code_line() {
    let input = "// [deps]\n// fmt\nint main() {}\n// -O3 is just a comment now";
    let meta = parse_manifest(input).expect("Block closes at code line");
    assert_eq!(meta.packages, vec!["fmt"]);
    assert!(meta.compiler_flags.is_empty());
}

#[test]
fn empty_source_is_an_error() {
    assert_eq!(parse_manifest(""), Err(ParseError::EmptySource));
}

#[test]
fn blank_line_inside_block_is_transparent() {
    let meta = parse_manifest("// [deps]\n// fmt\n\n// -O3\n// [end]").expect("blank line ignored");
    assert_eq!(meta.packages, vec!["fmt"]);
    assert_eq!(meta.compiler_flags, vec!["-O3"]);
}

#[test]
fn a_second_deps_block_is_ignored() {
    let meta = parse_manifest("// [deps]\n// fmt\n// [end]\n// [deps]\n// -O3\n// [end]")
        .expect("second block ignored");
    assert_eq!(meta.packages, vec!["fmt"]);
    assert!(meta.compiler_flags.is_empty());
}

#[test]
fn a_nested_opener_is_ignored_not_an_error() {
    let meta = parse_manifest("// [deps]\n// [deps]\n// fmt\n// [end]").expect("nested ignored");
    assert_eq!(meta.packages, vec!["fmt"]);
}

#[test]
fn crlf_line_endings_are_handled() {
    let meta = parse_manifest("// [deps]\r\n// fmt\r\n// -O3\r\n// [end]").expect("CRLF");
    assert_eq!(meta.packages, vec!["fmt"]);
    assert_eq!(meta.compiler_flags, vec!["-O3"]);
}

#[test]
fn a_flag_containing_a_space_is_kept_whole() {
    let meta = parse_manifest("// [deps]\n// -O3 extra\n// [end]").expect("space flag");
    assert_eq!(meta.compiler_flags, vec!["-O3 extra"]);
}

#[test]
fn end_with_extra_payload_is_an_error() {
    assert_eq!(
        parse_manifest("// [deps]\n// [end] extra\n// [end]"),
        Err(ParseError::UnknownDirective { line: 2 })
    );
}

#[test]
fn strips_a_utf8_bom() {
    let meta = parse_manifest("\u{FEFF}// [deps]\n// fmt\n// [end]").expect("BOM is stripped");
    assert_eq!(meta.packages, vec!["fmt"]);
}

#[test]
fn unknown_directive_inside_a_block_is_an_error() {
    assert_eq!(
        parse_manifest("// [deps]\n// [DEP]\n// [end]"),
        Err(ParseError::UnknownDirective { line: 2 })
    );
}

// ---------- cache key ----------

fn base_inputs() -> KeyInputs {
    KeyInputs {
        source: b"int main() {}".to_vec(),
        local_headers: vec![(PathBuf::from("util.hpp"), b"#pragma once".to_vec())],
        packages: vec!["fmt".to_string()],
        compiler_flags: vec!["-O3".to_string()],
        baseline_flags: vec!["-std=c++20".to_string()],
        compiler_version: "Apple clang 21.0.0".to_string(),
    }
}

#[test]
fn hash_is_deterministic() {
    assert_eq!(compute_key(&base_inputs()), compute_key(&base_inputs()));
}

#[test]
fn hash_changes_with_source() {
    let mut changed = base_inputs();
    changed.source = b"int main() { return 1; }".to_vec();
    assert_ne!(compute_key(&base_inputs()), compute_key(&changed));
}

#[test]
fn hash_changes_with_compiler_version_and_flags() {
    let mut other_compiler = base_inputs();
    other_compiler.compiler_version = "Apple clang 22.0.0".to_string();
    assert_ne!(compute_key(&base_inputs()), compute_key(&other_compiler));

    let mut other_flags = base_inputs();
    other_flags.compiler_flags = vec!["-O0".to_string()];
    assert_ne!(compute_key(&base_inputs()), compute_key(&other_flags));
}
