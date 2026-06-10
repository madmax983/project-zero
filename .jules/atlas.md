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

**[Title] Break Circular Dependency Between Setup and UI Menu State**
**Tangle:** A circular dependency existed where `src/setup.rs` imported `crate::ui::menu_state::MenuState` to inject it into the world, while `src/ui/menu_state.rs` and `src/ui/input.rs` imported `crate::setup::StartScenarioId` to manage the currently selected scenario. This tangled the startup plumbing with the UI logic.
**Blueprint:** Extracted the core start scenario definitions (`StartScenarioId`, `SetupConfig`, `StartScenarioDefinition`, etc.) into a new dedicated module at `src/shared/scenario.rs`. `src/setup.rs` now re-exports this using `pub use crate::shared::scenario::*;` to act as a Facade for internal plumbing, while explicit UI imports were updated to correctly pull from the newly defined structural boundary in `crate::shared::scenario`, cleanly breaking the cyclic reference.

**[Title] Break Testing Dependency Between Fleet and Ship Modules**
**Tangle:** Tests for `FleetComposition` (defined in `fleet.rs`) were located in `ship.rs`, causing an unnecessary cross-module import (`use crate::layer2::fleet::FleetComposition;`) just for testing.
**Blueprint:** Moved `FleetComposition` tests from `src/layer2/ship.rs` to `src/layer2/fleet.rs` to enforce domain cohesion, ensuring tests live alongside the logic they evaluate.

**[Title] Break Circular Dependency in Execution Module**
**Tangle:** A circular dependency existed between `src/layer1/execution/general_work.rs` and `src/layer1/execution/mining.rs`. `mining.rs` imported `WORK_CRIT_CHANCE` and `WORK_CRIT_MULTIPLIER` from `general_work.rs`, while `general_work.rs` imported `handle_mining_work` and `handle_chopping_work` from `mining.rs`.
**Blueprint:** Extracted `WORK_CRIT_CHANCE` and `WORK_CRIT_MULTIPLIER` into a new dedicated constants module at `src/layer1/execution/constants.rs`. Both `general_work.rs` and `mining.rs` now import the constants from this new module, breaking the cyclic reference.
