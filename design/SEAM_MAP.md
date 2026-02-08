## Seam Map

Map of connected and disconnected systems.

### Connected Seams

### INT-006: Relationships -> Socialize Utility
- **Date:** 2026-03-08
- **Systems connected:** `evaluate_socialize` -> `Relationships`
- **Glue added:**
    - Updated `evaluate_socialize` in `src/layer1/social.rs` to accept `relationships` and boost utility if friends are at the tavern.
    - Updated `evaluate_actions_system` in `src/layer1/utility_ai.rs` to fetch and pass `Relationships`.
- **Tests:** `tests/integration/social_utility.rs`

### INT-007: Morale -> Work Utility
- **Date:** 2026-03-08
- **Systems connected:** `evaluate_work` -> `calculate_effective_morale`
- **Glue added:**
    - Updated `evaluate_work` in `src/layer1/utility_ai.rs` to accept `morale` and scale base utility.
    - Updated `evaluate_actions_system` in `src/layer1/utility_ai.rs` to calculate `effective_morale` using `Needs`, `Memories`, and `SocialBuff`.
- **Tests:** `tests/integration/morale_work.rs`

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

### Pending Seams

- [ ] Utility AI -> Housing Assignment (Checked: Connected via `ActionType::SatisfyRest`)
- [ ] Food Production -> Needs Satisfaction (Checked: Connected via `consume_food_system`)
- [ ] Building Costs -> Resource Deduction (Checked: Connected via `try_place_building`)
- [ ] Pop Death -> UI Counter (Resolved by INT-002)

### INT-005: Pop Health -> Memories
- **Date:** 2026-02-07
- **Systems connected:** `death_system` -> `Memories` (WitnessedDeath), `starvation_damage_system` -> `Memories` (StarvationTrauma)
- **Glue added:**
    - Modified `death_system` in `src/layer1/health.rs` to add `WitnessedDeath` memory to survivors.
    - Modified `starvation_damage_system` in `src/layer1/health.rs` to add `StarvationTrauma` memory.
- **Schedule:** Part of standard simulation update.
- **Tests:** `tests/integration/health_memory.rs`
