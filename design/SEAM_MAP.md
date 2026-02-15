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

### INT-022: Social Class -> Room Quality Expectations
- **Date:** 2026-11-23
- **Systems connected:** `SocialClass` -> `apply_room_quality_thoughts`
- **Glue added:**
    - Updated `src/layer1/room_quality.rs` to fetch `SocialClass`.
    - Modified quality thresholds based on class (Elites demand higher quality).
- **Tests:** `tests/integration/social_room_quality.rs` (1 test verified)

### INT-023: Vacuum Clears Pollution
- **Date:** 2026-12-06
- **Systems connected:** `PressureGrid` -> `AtmosphereGrid`
- **Glue added:**
    - Added `vacuum_clears_pollution_system` to `src/layer1/integration.rs`.
    - Registered in `src/simulation.rs` after `update_pressure_system`.
- **Tests:** `tests/integration/atmosphere_vacuum.rs` (Integration test verified)
