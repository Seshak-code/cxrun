//! cxrun CLI: `cxrun <script.cpp> [args...]`
//!
//! Exit codes: 2 = usage error, 1 = cxrun failure (parse/cache/compile);
//! otherwise the child program's own exit code is forwarded verbatim.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

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
fn run(_source: &Path, _script_args: &[String]) -> Result<u8, CxrunError> {
    todo!("orchestrate the 7 steps above")
}
