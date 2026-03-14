
**[Deriving Copy for Light Bevy Resources]**
**Learning:** Found multiple small, plain-old-data structs (`ColonyResources`, `DayNightCycle`, `UtilityConfig`) being explicitly `.clone()`'d in the hot path of the `evaluate_actions_system` during parallel context construction. Explicit cloning on non-Copy types, even small ones, carries conceptual bloat and potential optimization hurdles compared to standard bitwise stack-copies for `Copy` types. By deriving `Copy`, we communicate to the compiler and the developer that this data is just simple bytes.
**Action:** When creating new lightweight `Resource` structs containing only numbers or small enums, always add `Copy` to the derive macros alongside `Clone` to allow safe, fast dereferencing (`*world.resource::<T>()`) instead of `.clone()`.

**[Removed Intermediate Vec Allocation in general_work.rs]**
**Learning:** Collecting iterators into intermediate Vecs before populating a HashMap is a common pattern that wastes memory and CPU due to unnecessary allocations, particularly on hot paths like evaluating populations of agents.
**Action:** When filtering and mapping query results to populate a target collection (like `HashMap`), iterate and push directly into the target collection to avoid `.collect::<Vec<_>>()` allocations.

**[Avoid Bevy HashMap Cloning in Tick Systems]**
**Learning:** Calling `world.get_resource::<T>()` and cloning a large `HashMap` to appease the borrow checker before executing a `world.query()` causes unnecessary heap allocations every single tick. This happens because `world.get_resource()` borrows `&World` immutably, and `world.query().iter(world)` also borrows `&World` (often mutably if using `query_mut`, though even `iter` requires care).
**Action:** Tightly scope the Bevy `query` execution. Create the query state (`let mut query = world.query::<T>();`) *first*, then fetch the resource reference `let data = world.get_resource::<T>();`, and pass the resource reference down into the loop `query.iter(world)` rather than cloning the data beforehand.

**Deriving `Copy` on Hot Enums by Replacing `String`**
**Learning:** `ItemType` contained a single dynamically allocated variant `Curio(String)` which prevented the entire 32-byte enum from implementing `Copy`. This forced the Utility AI (which processes thousands of items per tick) to call `.clone()` continuously, resulting in massive heap allocation overhead.
**Action:** Replace `String` with `&'static str` for hardcoded strings in enums whenever possible to allow `#[derive(Copy)]`. This turns O(N) heap allocations into zero-cost stack copies. Add a `assert_is_copy::<T>()` test to lock in the performance gain and prevent future regressions.

**[HashMap Clone in Hot Path Avoided]**
**Learning:** Cloning a struct containing a `HashMap` (e.g. `TabooState`) per-tick just to use it immutably within a parallel ComputeTaskPool results in continuous, expensive heap allocations on the hot path.
**Action:** Extract the resource temporarily using `world.remove_resource::<T>()` to gain ownership without cloning, run the parallel context, and `world.insert_resource()` it back afterwards.
