## 2024-05-24 - Resource Deduction Readability Smell
**Learning:** Checking massive `ColonyResources` capacity in-place by instantiating a nearly-empty struct creates immense visual clutter (e.g., in `shadow_market.rs`). Similarly, manual bounds checking `if resources.ration >= cost { resources.consume(...) }` is repetitive and prone to error.
**Action:** Implemented `get_amount` and `try_consume` helper methods directly on `ColonyResources`. Replaced inline dummy-struct instantiation with clean, single-line method calls that check capacity and consume atomically, eliminating ~80 lines of boilerplate logic.

## 2024-05-25 - Building Cost Initialization Clutter Smell
**Learning:** Massive struct instantiation using `ColonyResources { metal: 50.0, ..ColonyResources::zeroed() }` within `match` arms inside `src/layer1/architecture/building.rs` creates extreme pyramid-of-doom style visual clutter when the struct has dozens of fields.
**Action:** Implemented the builder pattern on `ColonyResources` (`with_metal()`, `with_wood()`, etc). Replaced all block initializations with clean `ColonyResources::zeroed().with_X(...).with_Y(...)` method chains, compressing logic down to a single line per match arm without changing behavior.

## 2024-05-27 - Redundant Init Checks and Destructive Insert Smells
**Learning:** Checking `if !world.contains_resource::<T>()` before calling `world.init_resource::<T>()` creates immense visual clutter (e.g., in `simulation.rs` God Functions) because `init_resource` already internally handles existence checks and acts as a safe no-op. However, using `world.insert_resource(...)` unconditionally overwrites state. Removing protective `if` checks around `insert_resource` will wipe the game state (like `Schedules` or `DiplomaticStanding`) on every simulation tick.
**Action:** Remove redundant `contains_resource` checks only for `init_resource` calls. Strictly preserve `if !world.contains_resource::<T>()` protective blocks around `insert_resource` calls to prevent state wiping regressions.

## 2024-06-25 - Bevy System Tuple Pyramid of Doom Smell
**Learning:** Adding too many systems (more than ~15-20 depending on Bevy version) into a single tuple like `schedule.add_systems((sys1, sys2, ...))` causes the Rust compiler to exceed its recursion limits or fail with a vague `error[E0599]: the method in_set exists for tuple but its trait bounds were not satisfied`. This happens because Bevy defines `IntoSystemConfigs` and `IntoSystemSetConfigs` up to a certain tuple arity.
**Action:** When encountering massive system registration tuples, split them into multiple smaller `schedule.add_systems((...))` calls to keep the arity well below Bevy's internal limits, keeping the code clean and strictly avoiding compiler recursion bound failures.

## 2024-05-28 - Flattened UI Status Logic Smell
**Learning:** `get_status_line` in `ui/status.rs` was a single massive God Function building an immense vector of spans manually, creating high visual complexity.
**Action:** Extracted the visual sections (play/pause, time, colony stats, resources, modes) into small independent helper functions, dramatically reducing the size of the main `get_status_line` method and making it purely compositional without altering any behavior.

## 2024-05-28 - Extracted Inspector Logic Smell
**Learning:** `render_entity_inspector` in `ui/inspector.rs` had a 40+ line block of `else if let` checks for different component types, violating "Flattening the Pyramid" principles.
**Action:** Moved the long `else if let` sequence into a `render_specific_details` helper function containing guard clauses (`if let Some(...) = ... { render_...(); return; }`), greatly simplifying the parent function's logic flow.

## 2024-05-28 - Extracted Hit Stop Juice Logic Smell
**Learning:** `execute_attack` in `layer1/combat.rs` was bloated with heavy "Hit Stop" visual effect logic and particle spawning mixed directly with core combat damage calculations.
**Action:** Flattened the logic using early returns (`let Some(...) = ... else { return; }`) and extracted the entire visual hit stop/particle "juice" logic into a clean `apply_hit_stop_and_juice` helper function.
