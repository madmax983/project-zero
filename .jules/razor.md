## [Reduction]
**Bloat:** `InputRouter` using `HashMap` for dynamic dispatch of static handlers.
**Cut:** Replaced with a simple `match` statement.
**Saved:** ~15 lines of code, removed runtime registration complexity, enforced exhaustiveness via compiler.

## [Reduction]
**Bloat:** Unused `accumulator` field and `ticks_per_second` method in `SimulationTime` / `SimSpeed`.
**Cut:** Deleted them.
**Saved:** ~15 lines of speculative code.

## [Reduction]
**Bloat:** Empty architectural layers `src/layer2` and `src/layer3` (Future Proofing).
**Cut:** Deleted the directories and modules.
**Saved:** 2 files, 2 directories, cognitive load of navigating empty shells.
