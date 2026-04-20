## [Reduction]
**Bloat:** [The over-engineered pattern: PlacementError enum for single-site internal validation]
**Cut:** [The simplified solution: Returning `Result<(), &'static str>` directly from `validate_building_placement`]
**Saved:** [Lines of code / Cognitive load: Removed an enum and a match statement indirection.]

## [Reduction]
**Bloat:** `EventType` enum in `src/layer1/geography.rs` used solely to map to Strings.
**Cut:** Removed `EventType` entirely. Replaced it with a direct `event_name: String` inside `HistoricalEvent`.
**Saved:** 7 lines of code and an unnecessary enum indirection.

## [Reduction]
**Bloat:** MapRenderContext using generic S: BuildHasher
**Cut:** Removing the S parameter and assuming the default hasher since we only ever use it with the default hasher.
**Saved:** Removed generic parameters from struct, impls, and ~10 functions.
## [Reduction]
**Bloat:** The `spawn_building_with_material` function in `src/layer1/architecture/building.rs`, which just passed parameters to `spawn_building`.
**Cut:** Made `spawn_building` public, deleted `spawn_building_with_material`, and updated all callers across the codebase.
**Saved:** 11 lines of code and an unnecessary abstraction layer.
