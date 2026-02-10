## Seam Map

Map of connected and disconnected systems.

### Connected Seams

### INT-002: Stats -> Status Bar UI
- **Date:** 2026-02-18
- **Systems connected:** `Pop` / `ColonyResources` -> `ui::status::render_status_bar`
- **Glue added:**
    - Updated `render_status_bar` in `src/ui/status.rs` to count pops and read resources.
    - Updated `get_status_string` to format "Souls" and "Yield".
- **Tests:** `tests/integration/ui_stats.rs` (Integration test verified)

### INT-001: Socialize Action -> Tavern Visitor
- **Date:** 2026-02-17
- **Systems connected:** `utility_ai::evaluate_actions_system` -> `execution::arrival_handler_system` -> `social::restore_leisure_system`
- **Glue added:**
    - `AssignmentType::TavernVisitor` in `src/layer1/execution.rs`
    - `assign_to_tavern` helper in `src/layer1/execution.rs`
    - Updated `cleanup_previous_assignment_system` in `src/layer1/execution.rs`
    - Updated `biography_monitor_system` in `src/experimental/biography.rs`
- **Schedule:** `run_simulation_tick` in `src/simulation.rs` handles the order.
- **Tests:** `tests/social_tavern.rs` (Integration test verified)

### INT-003: Fire -> Pop Health
- **Date:** 2026-02-18
- **Systems connected:** `fire_spread_system` -> `fire_damage_pops_system` -> `death_system`
- **Glue added:** `fire_damage_pops_system` in `src/layer1/integration.rs`
- **Schedule:** Chained in Simulation, spread -> pop damage -> building damage
- **Tests:** `tests/integration/fire_health.rs`

### INT-005: Pop Health -> Memories
- **Date:** 2026-02-07
- **Systems connected:** `death_system` -> `Memories` (WitnessedDeath), `starvation_damage_system` -> `Memories` (StarvationTrauma)
- **Glue added:**
    - Modified `death_system` in `src/layer1/health.rs` to add `WitnessedDeath` memory to survivors.
    - Modified `starvation_damage_system` in `src/layer1/health.rs` to add `StarvationTrauma` memory.
- **Schedule:** Part of standard simulation update.
- **Tests:** `tests/integration/health_memory.rs`

### INT-008: Buildings -> Lighting System
- **Date:** 2026-10-27
- **Systems connected:** `try_place_building` -> `LightSource` -> `update_lighting_system` -> `apply_lighting_penalties_system`
- **Glue added:**
    - Modified `src/layer1/building.rs` to attach `LightSource` components to `Tavern`, `Smelter`, `Smithy`, `Housing`, `Library`, `Hospital`, `LumberMill`.
- **Tests:** `tests/integration/lighting_buildings.rs`

### INT-009: Chronicle -> Rumor Web
- **Date:** 2026-03-10
- **Systems connected:** `advance_season_system` / `check_milestones_system` -> `AddChronicleEvent` -> `chronicle_rumor_bridge_system` -> `Knowledge`
- **Glue added:**
    - `AddChronicleEvent` in `src/layer1/chronicle.rs`
    - `chronicle_rumor_bridge_system` in `src/layer1/integration.rs`
    - Refactored `Chronicle::add_event` call sites to emit `AddChronicleEvent`.
- **Tests:** `tests/integration/chronicle_rumor.rs`

### INT-010: Affinity Change Events -> Relationships
- **Date:** 2026-03-10
- **Systems connected:** `exchange_rumors_system` -> `AffinityChange` -> `modify_affinity_system` -> `Relationships`
- **Glue added:**
    - Added `modify_affinity_system` to `SimulationSchedule` (was missing).
    - Added `Events::update_system` for `AffinityChange` and `AddChronicleEvent`.
- **Tests:** `tests/integration/chronicle_rumor.rs` (implicitly tests event flow), `tests/social_tavern.rs` (updated to support events)

### INT-006: Energy System -> Refining System
- **Date:** 2026-03-10
- **Systems connected:** `PowerConsumer` -> `process_refining_system` -> `RefiningProgress`
- **Glue added:**
    - Modified `process_refining_system` in `src/layer1/refining.rs` to query `Option<&PowerConsumer>`.
    - Added logic to skip refining if a consumer is present but inactive.
- **Tests:** `tests/integration/power_refining_seam.rs`

### Pending Seams

- [ ] Utility AI -> Housing Assignment (Checked: Connected via `ActionType::SatisfyRest`)
- [ ] Food Production -> Needs Satisfaction (Checked: Connected via `consume_food_system`)
- [ ] Building Costs -> Resource Deduction (Checked: Connected via `try_place_building`)
- [ ] Pop Death -> UI Counter (Resolved by INT-002)

### INT-007: Acoustics -> Rest Recovery
- **Date:** 2026-02-09
- **Systems connected:** `update_noise_system` -> `restore_rest_in_housing_system` -> `Needs.rest`
- **Glue added:**
    - Modified `restore_rest_in_housing_system` in `src/layer1/housing.rs` to query `NoiseMap`.
    - Updated `SimulationSchedule` in `src/simulation.rs` to order rest recovery after noise update.
- **Tests:** `tests/integration/acoustics_rest.rs`

### INT-011: Pop Death -> Chronicle
- **Date:** 2026-03-27
- **Systems connected:** `death_system` -> `PopDied` -> `pop_death_chronicle_bridge` -> `AddChronicleEvent`
- **Glue added:**
    - `PopDied` event in `src/layer1/pop.rs`.
    - `pop_death_chronicle_bridge` in `src/layer1/integration.rs`.
    - Updated `death_system` in `src/layer1/health.rs` to emit `PopDied`.
- **Tests:** `tests/integration/death_chronicle.rs`

### INT-012: Vermin -> Pop Morale
- **Date:** 2026-10-17
- **Systems connected:** `vermin_growth_system` -> `VerminState` -> `vermin_morale_system` -> `Memories`
- **Glue added:**
    - `vermin_morale_system` in `src/layer1/integration.rs`.
    - `MemoryType::DisgustedByVermin` in `src/layer1/memory.rs`.
    - Updated `SimulationSchedule` in `src/simulation.rs` to run `vermin_morale_system`.
- **Tests:** `tests/integration/vermin_morale.rs` (Integration test verified)
