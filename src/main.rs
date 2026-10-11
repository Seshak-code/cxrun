//! cxrun CLI: `cxrun <script.cpp> [args...]`
//!
//! Exit codes: 2 = usage error, 1 = cxrun failure (parse/cache/compile);
//! otherwise the child program's own exit code is forwarded verbatim.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

use cxrun::cache::{compute_key, KeyInputs};
use cxrun::compiler::{baseline_flags, compile, discover_toolchain};
use cxrun::parser::parse_manifest;
use cxrun::CxrunError;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(source) = args.next() else {
        eprintln!("Usage: cxrun <source_file.cpp> [args...]");
        return ExitCode::from(2);
    };
    let script_args: Vec<String> = args.collect();

    match run(&PathBuf::from(source), &script_args) {
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            eprintln!("[cxrun] {e}");
            ExitCode::from(1)
        }
    }
}

/// The full pipeline:
///   1. `compiler::discover_toolchain`
///   2. read source bytes; `parser::parse_manifest`
///   3. `compiler::discover_local_headers` (M2: transitive header hashing)
///   4. `cache::compute_key` over all inputs (source, headers, packages,
///      flags, baseline flags, compiler version)
///   5. `cache::acquire_lock`
///   6. `key.binary_path()` exists → exec (cache HIT, target < 10 ms)
///   7. otherwise `compiler::compile` → exec
///
/// Exec contract: `std::process::Command` with inherited stdio and NO shell;
/// return the child's exit code as `u8` (map signal termination to 1).
fn run(source: &Path, script_args: &[String]) -> Result<u8, CxrunError> {
    let toolchain = discover_toolchain()?;

    let source_bytes = fs::read(source)?;
    // Cow: borrows when the file is valid UTF-8, allocates only to replace bad bytes.
    let source_text = String::from_utf8_lossy(&source_bytes);
    let meta = parse_manifest(&source_text)?;

    let key = compute_key(&KeyInputs {
        source: source_bytes,

        local_headers: Vec::new(),
        packages: meta.packages.clone(),
        compiler_flags: meta.compiler_flags.clone(),
        baseline_flags: baseline_flags().iter().copied().map(String::from).collect(),
        compiler_version: toolchain.version.clone(),
    });



    let binary_path = key.binary_path()?;
    if !binary_path.exists() {

        fs::create_dir_all(key.dir()?)?;
        compile(source, &binary_path, &meta, &toolchain)?;
    }

    let status = Command::new(&binary_path)
        .args(script_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    // Signal death (code() is None) and codes outside 0-255 both map to 1.
    Ok(u8::try_from(status.code().unwrap_or(1)).unwrap_or(1))
}
