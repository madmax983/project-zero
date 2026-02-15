### INT-021: Faction Strike -> Work Execution
- **Date:** 2026-03-27
- **Systems connected:** `is_pop_striking` -> `utility_ai::evaluate_single_pop` / `produce_food_system` / `process_refining_system` / `process_research_system` / `haul_system`
- **Glue added:**
    - Updated `utility_ai.rs` to block all work assignments.
    - Updated `farm.rs`, `refining.rs`, `tech.rs`, `hauling.rs` to check for strike status.
- **Tests:** `tests/integration/strikes_work.rs` (Integration test verified)

### INT-006: Strikes -> Work/Science Execution
- **Date:** 2026-10-26
- **Systems connected:** `FactionState::Striking` -> `work_execution_system` (Mining/Forestry) / `process_scan_system` (Exploration)
- **Glue added:**
    - Updated `src/layer1/execution.rs` to query Factions and block work.
    - Updated `src/layer1/science.rs` to query Factions and block scanning.
- **Tests:** `tests/integration/strikes_mining_science.rs` (2 tests verified)
