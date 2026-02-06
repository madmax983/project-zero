## [Reduction]
**Bloat:** `InputRouter` using `HashMap` for dynamic dispatch of static handlers.
**Cut:** Replaced with a simple `match` statement.
**Saved:** ~15 lines of code, removed runtime registration complexity, enforced exhaustiveness via compiler.

## [Reduction]
**Bloat:** `utility_ai` split into `math.rs` and `types.rs` despite low complexity.
**Cut:** Flattened into single `utility_ai.rs` module.
**Saved:** Removed 2 files and a directory, simplified imports, reduced module hopping.

## [Reduction]
**Bloat:** Unused `Viewport` variable and import in `locations.rs` test.
**Cut:** Deleted it.
**Saved:** Cleaned up zombie code and clippy warnings.
