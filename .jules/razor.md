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

## [Reduction]
**Bloat:** `InputRouter` struct instantiated as a state-less wrapper for static logic ("Factory Factory").
**Cut:** Replaced with free functions `route_input` and `route_mouse_input`. Removed `InputRouter` struct.
**Saved:** ~20 lines of code, removed `Rc<RefCell<InputRouter>>` complexity in WASM app.

## [Reduction]
**Bloat:** `ColonyMemory` resource defined but never read ("Zombie Code").
**Cut:** Deleted `ColonyMemory` struct and its initialization.
**Saved:** ~10 lines of code, reduced memory footprint (slightly), removed cognitive load.

## [Reduction]
**Bloat:** `SimulationTime` accumulator and `SimSpeed::ticks_per_second` unused "future proofing" (YAGNI).
**Cut:** Removed fields and methods.
**Saved:** ~15 lines of code, cleaner API.
