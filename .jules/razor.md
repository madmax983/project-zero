## [Reduction]
**Bloat:** `InputRouter` using `HashMap` for dynamic dispatch of static handlers.
**Cut:** Replaced with a simple `match` statement.
**Saved:** ~15 lines of code, removed runtime registration complexity, enforced exhaustiveness via compiler.

## [Reduction]
**Bloat:** `Thoughts` system (pure flavor text string generation on eating) redundant with `Memories`.
**Cut:** Deleted `thoughts.rs`, removed component from `Farm` logic and `Inspector` UI.
**Saved:** ~100 lines of code, 1 system execution per tick (or on eating), removed UI clutter.
## [Reduction]
**Bloat:** `utility_ai` module split into `math.rs`, `types.rs`, and `utility_ai.rs` for < 1000 lines of code.
**Cut:** Merged into single `utility_ai.rs`.
**Saved:** 2 files, 1 directory, ~10 import lines, mental overhead of jumping between files.
