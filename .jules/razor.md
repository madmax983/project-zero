## [Reduction]
**Bloat:** `DataSize`
**Cut:** Removed the unused `DataSize` struct entirely.
**Saved:** 2 lines of dead code.

## [Reduction]
**Bloat:** `ColonyResources` `with_*` builder methods
**Cut:** Removed unused builder methods.
**Saved:** ~200 lines of dead code/boilerplate.
## [Reduction]
**Bloat:** `ColonyResources` `with_*` builder methods
**Cut:** Removed the unused `with_*` builder methods entirely and refactored usages to use standard struct update syntax. Also resolved a massive amount of `Events::get_reader` deprecation warnings in test code to keep clippy strict.
**Saved:** Dozens of lines of builder boilerplate and 90+ instances of deprecated test calls.
