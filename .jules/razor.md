## [Reduction]
**Bloat:** `InputRouter` using `HashMap` for dynamic dispatch of static handlers.
**Cut:** Replaced with a simple `match` statement.
**Saved:** ~15 lines of code, removed runtime registration complexity, enforced exhaustiveness via compiler.

## [Reduction]
**Bloat:** `Thoughts` system (pure flavor text string generation on eating) redundant with `Memories`.
**Cut:** Deleted `thoughts.rs`, removed component from `Farm` logic and `Inspector` UI.
**Saved:** ~100 lines of code, 1 system execution per tick (or on eating), removed UI clutter.

## [Reduction]
**Bloat:** `utility_ai` split into `math.rs` (4 functions) and `types.rs` (structs), requiring users to juggle 3 files and fix circular/complex imports.
**Cut:** Flattened `math.rs` and `types.rs` into `src/layer1/utility_ai.rs` and deleted the directory.
**Saved:** 2 files, removed `mod` boilerplate, improved locality of reference (types near logic), simplified imports across 11 files.
