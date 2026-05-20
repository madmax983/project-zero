## 2025-04-25 - Haunted Assembly Lines Test Coverage
**Learning:** Writing tests for standalone ECS systems requires registering resources manually (like `Events<PopDiedInAccidentEvent>`) to avoid panics. When dealing with Bevy bundles or specific generic parameters in tests, it is critical to construct the component explicitly rather than just importing the generic definition (e.g. `Building { building_type: BuildingType::Farm }` instead of just `Building`).
**Action:** When writing tests that spawn entities and assign them, always look up the structure of required components (`AssignedTo`, `Building`) in their respective source files using `grep` or `cat` before writing the assertions, instead of assuming empty struct derivations exist.

## 2024-05-25 - TechState Corruption and Fallback coverage
**Learning:** `unwrap_or` calls when resolving active enum states (`TechStatus::Active`) and handling sorting logic (`partial_cmp` on floating points) within core manager structures like `TechState` can lack explicit tests.
**Action:** When finding `unwrap_or` on `partial_cmp` or fallback map fetches in manager structs, write targeted unit tests that construct edge-case states (like identically costed structs forcing a stable sort resolution, or fetching unknown keys) to explicitly prove the fallback works safely without panic.

## 2024-05-25 - PopAction/Sabotage Testing
**Learning:** Testing logic tied directly to specific `ActionType` matching (like `ActionType::Sabotage`) in complex queries requires carefully setting up related system components (like `MovementTarget`) and initializing properties precisely instead of relying solely on `Default::default()`, as incorrect setups can silently bypass the system conditions.
**Action:** Always verify exactly which component fields a system iterates over and queries before constructing mock entities to ensure your assertions are correctly hitting the logic branches.

## 2024-05-25 - Triage NaN Sorting Stability
**Learning:** `partial_cmp` on `f32` in Rust returns `Option<Ordering>` due to `NaN`. Utilizing `.unwrap_or(Ordering::Equal)` prevents sort panics. But to be a good Sentry, this fallback itself must be tested by actively inserting a `NaN` into the data.
**Action:** When auditing `partial_cmp` calls, write a dedicated test that deliberately feeds `std::f32::NAN` into the system to verify the `unwrap_or` fallback branch handles the uncomparable state safely without panicking.
**[Testing  Coverage]
**Learning:** Adding test coverage to  ensures we catch bugs relating to distance penalty logic and empty mob candidates. Code modifications to test modules should ensure imports are carefully reviewed, especially when relying on structs across  versus .
**Action:** Always test -returning functions with both  outcomes and selected best-choice  outcomes, and be extremely careful about avoiding duplicate imports or referencing non-existent properties (e.g. ) on types when writing tests.
**[Testing `evaluate_protest` Coverage]**
**Learning:** Adding test coverage to `evaluate_protest` ensures we catch bugs relating to distance penalty logic and empty mob candidates. Code modifications to test modules should ensure imports are carefully reviewed, especially when relying on structs across `mind::utility_eval_types` versus `map::GridPosition`.
**Action:** Always test `Option`-returning functions with both `None` outcomes and selected best-choice `Some` outcomes, and be extremely careful about avoiding duplicate imports or referencing non-existent properties (e.g. `base_score`) on types when writing tests.
**Bombardment OOB Handling**
**Learning:** `execute_bombardment_system` manually bounds-checks `grid.set` but the current tests only verify direct hits in-bounds. Added a new out-of-bounds check (`test_bombardment_out_of_bounds`) to `src/layer2/bombardment.rs` but it can stay as an internal test rather than PR unless requested.
**Action:** Always check edge cases with array indexing.

**Orphan Fleet Faction Missing**
**Learning:** `process_orphan_defection_system` only updates faction if `FleetFaction` exists. This is safe but unchecked.
**Action:** When working with multiple optional or linked components, write tests where components are missing to ensure we don't accidentally `.unwrap()` later.

**Trade Route Missing Destination**
**Learning:** `execute_trade_routes_system` gracefully skips if `source` or `destination` is missing via `get_many_mut`. The test `test_trade_route_missing_destination` only checks missing destination. I added a test for missing source `test_trade_route_source_missing`.
**Action:** Ensure `get_many_mut` results are fully handled.

**General Unwrap Audit**
**Learning:** Most `unwrap()` usages in `src/` are isolated to `#[cfg(test)]` modules where they are acceptable for asserting preconditions. I did not find any egregious `unwrap()` usages in production logic that pose immediate crash risks in these modules.
**Action:** In `layer2`, tests are generally very solid and robust. The `unwrap()`s inside tests are just asserting that the expected entities and components were successfully created or modified, which is standard practice in Rust/Bevy test suites.

**Conclusion:** The codebase is extremely well-tested and robust. The `unwrap()`s I found are all constrained within `#[cfg(test)]` blocks (e.g. `world.get::<...>().unwrap()`), where they are the idiomatic way to assert component existence in Bevy ECS tests. Edge cases are largely covered. I will not submit a PR as the code is already solid.
## [Testing Despawns in Bevy 0.15+]
**Learning:** `world.get_entity(id)` returns a `Result`, not an `Option`. When checking if an entity has been properly despawned in a unit test, you cannot use `.is_none()`. You must use `.is_err()`. Similarly, to assert it still exists, use `.is_ok()`.
**Action:** When writing tests that verify entity cleanup (e.g. `execute_cannibalize` despawning the building and designation), assert with `assert!(world.get_entity(id).is_err());`.
## [UI/Rendering Integration Testing]
**Learning:** Testing top-level Bevy UI rendering functions (like `render` or `render_with_shell` in `src/ui/mod.rs`) that accept a `&World` reference is highly complex. The `render` loop iterates over numerous child components (Map, Status Bar, Info Panels, Chronicle, Notifications), each of which pulls arbitrary resources from the ECS `World` (e.g., `ColonyResources`, `TerrainGrid`, `WaterGrid`, `Selection`, `RenderCache`, `WallTime`, `BuildMode`, `DesignationMode`). Failing to initialize *any* of these resources in the test's mock `World` causes deep panics (e.g. `world.resource::<T>()`) during rendering.
**Action:** When writing tests for high-level UI render loops, systematically run the tests to identify missing resources via panics, and iteratively inject default implementations of those resources (e.g. `world.insert_resource(T::default())`) until the render pass succeeds without panicking.
## [Unused Result in RunSystemOnce]
**Learning:** `world.run_system_once(system)` in `bevy_ecs` returns a `Result`. Calling it without handling the result will cause `unused_must_use` clippy warnings and could hide silent failures.
**Action:** Always append `.unwrap()` (or explicitly handle the error) when calling `world.run_system_once(system)` in tests to satisfy clippy and ensure robust assertions.

## [Clippy field-reassign-with-default]
**Learning:** Constructing a struct with `Default::default()` and then re-assigning individual fields sequentially on the mutable variable triggers the `clippy::field-reassign-with-default` lint.
**Action:** Construct the struct with the target fields assigned inline and spread the default (e.g., `let s = Struct { field: val, ..Default::default() };`).

## 2024-05-25 - Hedonic Treadmill Consumption Bridge test setup
**Learning:** `Events<T>::get_reader()` is deprecated in newer Bevy versions. Use `events.get_cursor()` instead to satisfy clippy and maintain up-to-date syntax when writing tests that consume and assert on emitted events.
**Action:** When manually evaluating emitted events within test fixtures, always initialize a cursor via `let mut cursor = events.get_cursor();` rather than `get_reader()`.

## 2024-05-25 - Test Module Inception
**Learning:** Adding a `#[cfg(test)] mod tests` block inside a file named `tests.rs` triggers the `clippy::module-inception` lint.
**Action:** Always add `#[allow(clippy::module_inception)]` above `mod tests {` when working in files named `tests.rs` to keep the build perfectly green.
