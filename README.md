# cxrun

Zero-config single-file C++20 script runner. Write a `.cpp` file, run it:

```sh
cxrun main.cpp
```

No CMake. No build files. No manual linker flags. cxrun parses inline
dependency comments, compiles with the host Clang, and caches the binary by
content hash — unchanged scripts relaunch in milliseconds.

> **Status:** Milestone 1 complete — parse → hash → cache → compile → run works
> end to end (`context/milestones.md` has the board). Header invalidation and
> locking land in Milestone 2.

## Inline manifest

```cpp
// [deps]
// fmt
// -O3
// [end]
#include <fmt/core.h>

int main() { fmt::print("zero-config C++\n"); }
```

Rules, briefly: only the **first** `[deps]` block in the file is read (later
ones are ordinary comments); blank lines inside the block are fine; a leading
BOM is tolerated; a `[`-word inside the block that isn't `[deps]` or `[end]`
is a parse error.

**Run scripts you trust.** Manifest flags reach your compiler as-is. cxrun
never invokes a shell and its own `-o` comes last, so a manifest cannot
redirect the output binary — but arbitrary flags can still do arbitrary
things.

## Scope

- **Is:** scratchpads, algorithm tests, micro-tools, single-file prototypes.
- **Is not:** multi-file applications, Qt/meta-compiler projects, WebAssembly
  builds — use `xmake` or CMake for those.

## Development

```sh
cargo build                                 # build the CLI
cargo test                                  # unit tests (all green)
cargo test --test integration -- --ignored  # end-to-end (requires a host clang++)
```

Cache lives in `~/.cxrun/cache/` (override with `$CXRUN_CACHE_DIR`).
