## [Reduction]
**Bloat:** `InputRouter` using `HashMap` for dynamic dispatch of static handlers.
**Cut:** Replaced with a simple `match` statement.
**Saved:** ~15 lines of code, removed runtime registration complexity, enforced exhaustiveness via compiler.

## [Reduction]
**Bloat:** `ColonyMemory` resource defined but never used (Zombie Code).
**Cut:** Deleted the struct and initialization logic.
**Saved:** Removed 1 unused struct, 1 unused test, and 2 lines of initialization. Reduced mental overhead.
