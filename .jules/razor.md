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

## [Reduction]
**Bloat:** Unused `Mainframe` marker component, dead `Offline` variant from `SystemStatus` enum, and unused helper functions `start_reformat`, `finish_reformat`, and `reformat_system` in `legacy_code.rs`.
**Cut:** Removed the marker component to avoid collisions with `digital_immortality`, excised the dead enum variant to make pattern matching exhaustive, and deleted the deprecated helper functions.
**Saved:** ~30 lines of dead code and indirection, preventing component collisions and unhandled enum variants.

## [Reduction]
**Bloat:** Redundant custom `ResourceType` enum in `src/layer1/diplomacy/factions/rivals.rs` containing only a single variant `Minerals`.
**Cut:** Deleted the custom enum and replaced it with the existing global `crate::layer1::economy::resources::ResourceType`, migrating the `Minerals` concept to the global `Ore` type.
**Saved:** 7 lines of redundant enum boilerplate and eliminated cognitive overhead of distinguishing between local and global resource types.

## [Reduction]
**Bloat:** The `HealthCondition` enum in `src/layer1/biology/health.rs` with exactly one variant (`RustLung`) stored inside a dynamically-allocated `Vec<HealthCondition>`.
**Cut:** Removed the entire enum and the vector, replacing it with a simple `has_rust_lung: bool` primitive field on the `Health` struct.
**Saved:** A dedicated struct, dozens of wrapper functions across multiple modules (`add_condition`, `has_condition`), and removed per-entity heap allocations from `Vec::new()`, vastly simplifying the `Health` interface.
