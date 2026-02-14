### INT-021: Faction Strike -> Work Execution
- **Date:** 2026-03-27
- **Systems connected:** `is_pop_striking` -> `utility_ai::evaluate_single_pop` / `produce_food_system` / `process_refining_system` / `process_research_system` / `haul_system`
- **Glue added:**
    - Updated `utility_ai.rs` to block all work assignments.
    - Updated `farm.rs`, `refining.rs`, `tech.rs`, `hauling.rs` to check for strike status.
- **Tests:** `tests/integration/strikes_work.rs` (Integration test verified)

### INT-021: Faction Strike -> Work Execution
- **Date:** 2026-03-27
- **Systems connected:** `is_pop_striking` -> `utility_ai::evaluate_single_pop` / `produce_food_system` / `process_refining_system` / `process_research_system` / `haul_system`
- **Glue added:**
    - Updated `utility_ai.rs` to block all work assignments.
    - Updated `farm.rs`, `refining.rs`, `tech.rs`, `hauling.rs` to check for strike status.
- **Tests:** `tests/integration/strikes_work.rs` (Integration test verified)
