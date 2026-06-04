**Atlas's Journal**

**[Title] Fix Circular Dependencies in SCALE Modules**
**Tangle:** The codebase had multiple cyclic dependencies including:
1. `layer1::core` <-> `layer1::anomalies`
2. `shared::keyboard` <-> `ui::shell`
3. `ui::input` <-> `ui::menu_state`

**Blueprint:**
1. Moved `cryptid_chronicle_bridge_system` from `src/layer1/core/integration.rs` to `src/layer1/anomalies/cryptid.rs` to ensure domain logic belongs with its specific feature definition.
2. Extracted tests dealing with input context (such as route_input behaviour) from `src/ui/menu_state.rs` into `src/ui/input.rs`, maintaining testing cohesion.
3. Moved the test `test_shell_config_round_trips_default_workspace` from `src/shared/keyboard.rs` into `src/ui/shell/config.rs` where the `ShellConfig` type is actually defined.

**[Title] Resolve Leviathan Naming Collision Across Layers**
**Tangle:** A naming collision and conceptual overload occurred where both `src/layer1/local_tributes.rs` and `src/layer2/leviathans.rs` defined a `Leviathan` component. This violated DRY, created a confusing API boundary between planetary lore beasts and spaceborne entities, and led to potential compiler/import leaks ("The Leak").
**Blueprint:** Renamed the layer 2 entity from `Leviathan` to `VoidLeviathan` (a "New Type" abstraction) in `src/layer2/leviathans.rs` to clearly differentiate its domain boundary (fleet/space mechanics consuming planetary resources) from the planetary tribute system (`layer1::local_tributes::Leviathan`).

**[Title] Resolve Ambiguous Glob Re-exports in Culture Module**
**Tangle:** The codebase contained two identical module names being exported (`pub mod culture;` in both `src/layer1/mod.rs` and `src/layer1/social/mod.rs`), which caused a compiler warning (`ambiguous_glob_reexports`) when a wildcard export (`pub use`) was used. This forced the use of `#[allow(ambiguous_glob_reexports)]` which masked potential architectural leaks.
**Blueprint:** Renamed the nested `social/culture.rs` to `culture/cultural_influence.rs` to keep domain logic organized while eliminating the naming collision. Removed the `#[allow]` directive to reinstate strict compiler boundaries and updated dependent systems (`layer3::diplomacy::cultural_pressure` and `layer3::fleets::generation_ship`) to use the newly disambiguated path.

**[Title] Extract Spatial Data Structures from Building Module**
**Tangle:** The `BuildingMap` and `OccupiedTiles` structures were defined inside `src/layer1/architecture/building.rs`, intertwining the concept of spatial occupancy with the architectural definitions of buildings. This caused a high degree of coupling and scattered imports across 50+ files in `layer1` core, pathfinding, geology, and access control.
**Blueprint:** Extracted `OccupiedTiles`, `BuildingMap`, and `update_building_map_system` into a new dedicated module at `src/layer1/core/spatial.rs`, exposing it globally via `crate::layer1::core::spatial`. Left the exports mapped over in `building.rs` as re-exports to not touch 500 lines of `use` block across the codebase while still fully separating the domains in the AST and modules.

**[Title] Break Circular Dependency Between Core Spatial and Architecture Building**
**Tangle:** `src/layer1/core/spatial.rs` imported the `Building` component from `src/layer1/architecture/building.rs` to run the `update_building_map_system`. Meanwhile, `src/layer1/architecture/building.rs` imported the `BuildingMap` resource back from `src/layer1/core/spatial.rs`, creating an architectural cycle.
**Blueprint:** Moved `update_building_map_system` out of `src/layer1/core/spatial.rs` and into `src/layer1/architecture/building.rs`. The core spatial module now strictly provides the raw data structures (`BuildingMap`, `OccupiedTiles`) without depending on higher-level architectural constructs. The architectural module handles the domain logic of mapping its components (`Building`) to the core data structures. Also cleaned up an unused cyclic import in `src/layer1/core/integration_stub.rs`.
