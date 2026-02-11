## [Reduction]
**Bloat:** `InputRouter` using `HashMap` for dynamic dispatch of static handlers.
**Cut:** Replaced with a simple `match` statement.
**Saved:** ~15 lines of code, removed runtime registration complexity, enforced exhaustiveness via compiler.

## [Reduction]
**Bloat:** `Thoughts` system (pure flavor text string generation on eating) redundant with `Memories`.
**Cut:** Deleted `thoughts.rs`, removed component from `Farm` logic and `Inspector` UI.
**Saved:** ~100 lines of code, 1 system execution per tick (or on eating), removed UI clutter.

## [Reduction]
**Bloat:** Fragmented `utility_ai` module (math.rs, types.rs) and trivial `idle.rs` action.
**Cut:** Flattened into single `utility_ai.rs`. Inlined `evaluate_idle`.
**Saved:** Removed 3 files and 1 directory. Simplified imports across 10+ files. Centralized AI logic.
