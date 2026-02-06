## Seam Map

Map of connected and disconnected systems.

### Connected Seams

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

### Pending Seams

- [ ] Utility AI -> Housing Assignment (Checked: Connected via `ActionType::SatisfyRest`)
- [ ] Food Production -> Needs Satisfaction (Checked: Connected via `consume_food_system`)
- [ ] Building Costs -> Resource Deduction (Checked: Connected via `try_place_building`)
- [ ] Pop Death -> UI Counter (Unknown status)
