# cxrun

Zero-config single-file C++20 script runner. Write a `.cpp` file, run it:

```sh
cxrun main.cpp
```

No CMake. No build files. No manual linker flags. cxrun parses inline
dependency comments, compiles with the host Clang, and caches the binary by
content hash — unchanged scripts relaunch in milliseconds.

> **Status:** scaffold. Project structure and type contracts through
> Milestone 2 are in place; core function bodies are `todo!()` stubs with
> documented behavior contracts in each module's doc comments.

## Inline manifest (planned syntax)

```cpp
// [deps]
// fmt
// -O3
// [end]
#include <fmt/core.h>

int main() { fmt::print("zero-config C++\n"); }
```

## Scope

- **Is:** scratchpads, algorithm tests, micro-tools, single-file prototypes.
- **Is not:** multi-file applications, Qt/meta-compiler projects, WebAssembly
  builds — use `xmake` or CMake for those.

## Development

```sh
cargo build                                 # build the CLI
cargo test                                  # unit tests (fail until stubs are implemented)
cargo test --test integration -- --ignored  # end-to-end (requires a host clang++)
```

Cache lives in `~/.cxrun/cache/` (override with `$CXRUN_CACHE_DIR`).
