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

**[Title] Break Circular Dependency Between Pathfinding and Access Control**
**Tangle:** `src/layer1/access_control.rs` imported `find_path_for_pop` from `src/layer1/pathfinding.rs` in its integration tests, while `pathfinding.rs` relied on `AccessControl` and `AccessMode` from `access_control.rs` for main functionality.
**Blueprint:** Moved `test_pathfinding_integration` from `access_control.rs` into `pathfinding.rs`. Since `pathfinding.rs` already depends on access control to build the paths, testing the integration there resolves the cyclic dependency cleanly while preserving the tests.

**[Title] Break Circular Dependency Between Combat and Fauna**
**Tangle:** `src/layer1/fauna/mod.rs` imported `HitStop` from `src/layer1/combat.rs`, while `combat.rs` imported `Fauna` and `FaunaType` in its integration tests for testing drafting and utility behavior.
**Blueprint:** Moved the integration tests that strictly required `Fauna` (`test_draft_toggle_overrides_behavior` and `test_undrafted_pop_flees_or_ignores`) into `fauna/mod.rs`. Modified the remaining combat tests to use `Health::default()` instead of `Fauna::default()` since `execute_attack` is generalized and only needs an entity with `Health`. This fully untangled the test dependencies from the core combat logic.

**[Title] Break Circular Dependency Between Memetics and Tech**
**Tangle:** `src/layer1/tech/mod.rs` imported `MemeticCarrier` from `src/layer1/memetics/mod.rs` to infect researchers when unlocking hazardous tech. However, `memetics/mod.rs` imported `unlock_tech` back from `tech/mod.rs` to test this specific interaction.
**Blueprint:** Moved the `test_unlocking_hazardous_tech_infects_researcher` test out of `memetics/mod.rs` and into the test suite of `tech/mod.rs` where the actual `unlock_tech` function lives.
**Dependency Cycle `combat` <--> `fauna` and `access_control` <--> `pathfinding`**
**Tangle:** `src/layer1/combat.rs` defined `HitStop` component and `hit_stop_system` which was imported by `src/layer1/fauna/mod.rs`, but `combat.rs` used `FaunaType` in its tests. Also, `src/layer1/access_control.rs` had an integration test that imported `find_path_for_pop` from `pathfinding.rs`, while `pathfinding.rs` imported `AccessControl` from `access_control.rs`.
**Blueprint:** Extracted `HitStop` and `hit_stop_system` into `src/layer1/physics/hit_stop.rs` to break the cyclic dependency. Also moved the integration test from `access_control.rs` into `pathfinding.rs`.

**[Title] Break Circular Dependency and Fix Blob Anti-pattern with Gravity Engineering Chronicle Bridge**
**Tangle:** The `gravity_engineering_chronicle_bridge` was located in `src/layer1/core/integration.rs`, exacerbating the "Blob" anti-pattern in `integration.rs` and distancing the bridging logic from the `gravity_engineering` domain.
**Blueprint:** Moved `gravity_engineering_chronicle_bridge` from `src/layer1/core/integration.rs` to `src/layer1/architecture/gravity_engineering.rs` to enforce domain cohesion. Updated references in `src/layer1/systems/observation.rs` and the integration tests.

**[Title] Break Circular Dependency and Fix Blob Anti-pattern with Hack Central Hub Chronicle Bridge**
**Tangle:** The `hack_hub_chronicle_bridge` was located in `src/layer1/core/integration.rs`, exacerbating the "Blob" anti-pattern in `integration.rs` and distancing the bridging logic from the `administration` domain where `HackCentralHubEvent` is defined. This led to bloated files and poor cohesion.
**Blueprint:** Moved `hack_hub_chronicle_bridge` from `src/layer1/core/integration.rs` to `src/layer1/administration/edicts.rs` to enforce domain cohesion. Updated references in `src/layer1/systems/observation.rs` and the integration tests (`tests/integration/orphaned_edict_bridge.rs`).

**[Title] Break Up Oversized Bevy System Registration Tuple**
**Tangle:** In `src/layer1/systems/environment.rs`, a single `schedule.add_systems` call contained a tuple of systems that exceeded Bevy's macro limit for `IntoSystemConfigs` (typically 21 elements). This caused a cryptic `E0599` compiler error where `in_set` could not be resolved because the trait bounds were not satisfied for a tuple of that size.
**Blueprint:** Split the oversized tuple into two separate `schedule.add_systems` blocks, each chaining into `.in_set(Layer1SystemSet::Environment)`. This resolved the compiler error without changing system execution order or architectural boundaries.

**[Title] Initialize Missing Test Resources to Prevent Execution Panics**
**Tangle:** Several integration tests (`binge_resources.rs`, `drone_network.rs`, `hauling_execution.rs`, `public_grievances_bridge.rs`) were failing with a panic because `arrival_handler_system` expected `UnequipFailedEvent` to be registered as an event resource (`Events<UnequipFailedEvent>`), but the test dummy worlds lacked this initialization.
**Blueprint:** Added explicit event resource initialization (`world.init_resource::<Events<scale::layer1::economy::bio_loom::UnequipFailedEvent>>();` or `app.add_event::<...>()`) to the setup phase of the failing integration tests to ensure all expected system parameters were available during execution.

**[Title] Break Circular Dependency Between Core Integration and Architecture Building Events**
**Tangle:** `src/layer1/core/events.rs` was not used across the entire codebase where events were being triggered. `src/layer1/architecture/building.rs` generated its own module export `crate::layer1::events` which mapped back into `crate::layer1::core::events` creating a "Tangle" of imports that spanned across domains, specifically in `integration.rs` where we were accessing `building.rs`'s `events` rather than using the explicit path.
**Blueprint:** Re-routed all imports from `crate::layer1::events::*` to directly use `crate::layer1::core::events::*`. This enforces the domain boundary: the `events` are part of `core`, and `building.rs` should not be providing a facade for `events` that the rest of the app uses. Removed the unused facade from `mod.rs` (implicitly, by ensuring it was not being used anymore).
**[Title] Continue Dismantling Core Integration Blob (Logistics)**
**Tangle:** The file `src/layer1/core/integration.rs` was massive containing logistics-related bridges like `mass_driver_chronicle_bridge` and `orbital_drop_chronicle_bridge`, exacerbating the "Blob" anti-pattern and distancing the bridging logic from the logistics domain.
**Blueprint:** Moved `mass_driver_chronicle_bridge` to `src/layer1/logistics/mass_driver.rs` and `orbital_drop_chronicle_bridge` to `src/layer1/logistics/orbital_drop.rs`. Updated the references in `src/layer1/systems/observation.rs`.

**[Title] Break Circular Dependency and Fix Blob Anti-pattern with Escape Pods Chronicle Bridge**
**Tangle:** The `escape_pods_chronicle_bridge` was located in `src/layer1/core/integration.rs`, exacerbating the "Blob" anti-pattern in `integration.rs` and distancing the bridging logic from the `escape` domain where `DistressSignal` and `Lifeboat` are defined. This led to bloated files and poor cohesion.
**Blueprint:** Moved `escape_pods_chronicle_bridge` from `src/layer1/core/integration.rs` to `src/layer1/actions/escape.rs` to enforce domain cohesion. Updated references in `src/layer1/systems/observation.rs` and the integration tests (`tests/integration/escape_pods_chronicle.rs`).

**[Title] Break Circular Dependency and Fix Blob Anti-pattern with Drone and Waste Chronicle Bridges**
**Tangle:** The `drone_spawner_bridge_system`, `drone_work_bridge_system`, `waste_pollution_bridge`, and `waste_scent_bridge` were located in `src/layer1/core/integration.rs`, exacerbating the "Blob" anti-pattern and distancing the bridging logic from their domains (`drone` and `resources`).
**Blueprint:** Moved drone bridges from `src/layer1/core/integration.rs` to `src/layer1/entities/drone.rs` and waste bridges to `src/layer1/economy/resources.rs` to enforce domain cohesion. Updated references in `src/layer1/systems/execution.rs`, `environment.rs`, and `observation.rs`.

**[Title] Break Circular Dependency and Fix Blob Anti-pattern with Cassandra Syndrome, Memory Smugglers, and Generation Ship Mutiny Chronicle Bridges**
**Tangle:** The `cassandra_syndrome_chronicle_bridge`, `cassandra_cult_chronicle_bridge`, `cassandra_syndrome_disaster_bridge`, `memory_smugglers_chronicle_bridge`, and `generation_ship_mutiny_chronicle_bridge` were all located in `src/layer1/core/integration.rs`. This exacerbated the "Blob" anti-pattern in the integration file and distanced the bridging logic from their respective domains (`cassandra_syndrome`, `memetics`, and `generation_ship_mutiny`).
**Blueprint:** Moved `cassandra_syndrome` bridges to `src/layer1/cassandra_syndrome/mod.rs`, the `memory_smugglers_chronicle_bridge` and `ReportedMemeticDisassociation` to `src/layer1/memetics/memory_smugglers.rs`, and the `generation_ship_mutiny_chronicle_bridge` to `src/cross_layer/generation_ship_mutiny.rs` to enforce domain cohesion. Updated references in `src/layer1/systems/observation.rs`.

**[Title] Break Circular Dependency and Fix Blob Anti-pattern with Apex Meat Harvest and Distribution Bridges**
**Tangle:** The `apex_meat_harvest_bridge_system` and `apex_meat_distribution_system` were located in `src/layer1/core/integration.rs`, exacerbating the "Blob" anti-pattern in `integration.rs` and distancing the bridging logic from the `economy/apex_diet` domain. This led to bloated files and poor cohesion.
**Blueprint:** Moved `apex_meat_harvest_bridge_system` and `apex_meat_distribution_system` from `src/layer1/core/integration.rs` to `src/layer1/economy/apex_diet.rs` to enforce domain cohesion. Updated references in `src/simulation.rs` and the integration tests (`tests/integration/apex_diet_integration.rs`).
**[Title] Fix Clippy Error: items_after_test_module**
**Tangle:** The `src/layer3/integration.rs` file had a function `sovereign_armada_chronicle_bridge` defined after the `mod tests` block, causing a clippy error `items_after_test_module`.
**Blueprint:** Moved the function to be placed before the `mod tests` block to satisfy clippy and ensure proper file structure.

**[Title] Fix Missing/Implicit Pop Dependency in layer1 mod**
**Tangle:** In `src/layer1/systems/consumption.rs` and `src/layer1/systems/observation.rs`, the code incorrectly referenced `crate::layer1::pop::handle_pop_death_system`. The actual source was `crate::layer1::entities::pop::handle_pop_death_system`, and while `entities::*` was re-exported using a wildcard in `layer1/mod.rs`, referencing it via `crate::layer1::pop::` instead of the full path confused tooling and broke strict paths.
**Blueprint:** Explicitly re-routed `crate::layer1::pop::handle_pop_death_system` to `crate::layer1::entities::pop::handle_pop_death_system` in both system registration files.
