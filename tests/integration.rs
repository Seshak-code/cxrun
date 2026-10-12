//! End-to-end tests against the real compiled binary. They require a host
//! C++ compiler and are `#[ignore]`d by default — run with:
//!   cargo test --test integration -- --ignored
//!
//! Each test gets its own temp script dir AND its own `$CXRUN_CACHE_DIR`,
//! so runs are hermetic and never touch the real `~/.cxrun/cache`.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

/// Path to the cxrun binary built by cargo for this test run.
fn cxrun_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cxrun"))
}

/// Minimal std-only temp dir (avoids a `tempfile` dev-dependency).
struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let path = std::env::temp_dir().join(format!("cxrun-test-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&path).expect("create temp dir");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn run_crun(script: &Path, cache_dir: &Path) -> ExitStatus {
    Command::new(cxrun_bin())
        .arg(script)
        .env("CXRUN_CACHE_DIR", cache_dir)
        .status()
        .expect("spawn cxrun")
}

#[test]
#[ignore = "needs a host C++ compiler"]
fn cold_run_compiles_and_forwards_exit_code() {
    let tmp = TempDir::new();
    let script = tmp.path().join("test.cpp");
    std::fs::write(&script, "int main() { return 42; }").expect("write script");

    let status = run_crun(&script, &tmp.path().join("cache"));
    assert_eq!(status.code(), Some(42));
}

#[test]
#[ignore = "needs a host C++ compiler"]
fn hot_run_hits_cache_under_50ms() {
    let tmp = TempDir::new();
    let script = tmp.path().join("test.cpp");
    std::fs::write(&script, "int main() { return 42; }").expect("write script");
    let cache = tmp.path().join("cache");

    let cold = run_crun(&script, &cache);
    assert_eq!(cold.code(), Some(42));

    let start = std::time::Instant::now();
    let hot = run_crun(&script, &cache);
    let elapsed = start.elapsed();

    assert_eq!(hot.code(), Some(42));
    assert!(
        elapsed.as_millis() < 50,
        "hot cache run took too long: {elapsed:?}"
    );
}

#[test]
#[ignore = "needs a host C++ compiler"]
fn editing_source_invalidates_cache() {
    let tmp = TempDir::new();
    let script = tmp.path().join("test.cpp");
    let cache = tmp.path().join("cache");

    std::fs::write(&script, "int main() { return 1; }").expect("write v1");
    assert_eq!(run_crun(&script, &cache).code(), Some(1));

    std::fs::write(&script, "int main() { return 2; }").expect("write v2");
    assert_eq!(run_crun(&script, &cache).code(), Some(2));
}

#[test]
#[ignore = "needs a host C++ compiler"]
fn manifest_o_flag_cannot_redirect_the_output() {
    let tmp = TempDir::new();
    let script = tmp.path().join("test.cpp");
    let redirect = tmp.path().join("evil.bin");
    std::fs::write(
        &script,
        format!(
            "// [deps]\n// -o {}\n// [end]\nint main() {{ return 42; }}\n",
            redirect.display()
        ),
    )
    .expect("write script");

    let status = run_crun(&script, &tmp.path().join("cache"));
    assert_eq!(status.code(), Some(42));
    assert!(
        !redirect.exists(),
        "manifest -o must not redirect the output"
    );
}
