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

### INT-004: Hauling Execution Flow
- **Date:** 2026-12-10
- **Systems connected:** `evaluate_haul` (Utility AI) -> `haul_system` (Execution)
- **Glue added:**
    - Updated `PopEvalData` to include `Carrying`.
    - Updated `evaluate_haul` to target stockpiles if already carrying.
- **Tests:** `tests/integration/hauling_flow.rs` (Integration test verified)

### INT-008: Grid Overload -> Fire Ignition
- **Date:** 2026-02-16
- **Systems connected:** `power_grid_system` (Energy) -> `Fire` (Environment)
- **Glue added:**
    - Updated `src/layer1/energy/mod.rs` to spawn `Fire` entity on severe overload.
- **Tests:** `tests/energy_fire.rs` (Integration test verified)

### INT-009: Medical Power Dependency
- **Date:** 2026-10-28
- **Systems connected:** `PowerConsumer` (Energy) -> `healing_system` (Medical)
- **Glue added:**
    - Updated `src/layer1/building.rs` to add `PowerConsumer` to `Hospital`.
    - Updated `src/layer1/medical.rs` to query `PowerConsumer` and block healing if inactive.
- **Tests:** `tests/integration/medical_power.rs` (4 tests verified)

### INT-010: Tech Corruption -> Building Functionality
- **Date:** 2026-02-28
- **Systems connected:** `TechState` (Research) -> `process_refining_system` (Refining) / `produce_food_system` (Farming) / `turret_fire_system` (Defense)
- **Glue added:**
    - Updated `src/layer1/refining.rs` to skip production if required tech is corrupted.
    - Updated `src/layer1/farm.rs` to skip food production if required tech is corrupted.
    - Updated `src/layer1/turret.rs` to skip firing if required tech is corrupted.
- **Tests:** `tests/integration/tech_corruption.rs` (3 tests verified)

### INT-024: Items on Ground -> Vermin Growth
- **Date:** 2026-03-27
- **Systems connected:** `ResourceItem` (World) -> `VerminState` (Environment)
- **Glue added:**
    - Updated `src/layer1/vermin.rs` to query `ResourceItem` and add to growth calculation.
- **Tests:** `tests/integration/vermin_items.rs` (3 tests verified)

### INT-026: Amputation -> Cybernetics/Memory
- **Date:** 2026-02-18
- **Systems connected:** `AmputationEvent` (Hazards) -> `MissingLimb` (Cybernetics) / `Memories` (Pop)
- **Glue added:**
    - Added `amputation_handler_system` in `src/layer1/integration.rs`.
    - Modified `get_efficiency_bonus` to account for `MissingLimb`.
    - Modified `surgery_system` to cure `MissingLimb`.
- **Tests:** `tests/integration/amputation_prosthetic.rs` (3 tests verified)

### INT-002: Medical Treatment -> Social Debt
- **Date:** 2026-02-xx
- **Systems connected:** `healing_system` (Medical) -> `medical_debt_bridge_system` (Integration) -> `SocialDebt` (Social)
- **Glue added:**
    - Added `PatientTreated` event in `src/layer1/medical.rs`.
    - Modified `healing_system` to emit `PatientTreated`.
    - Added `medical_debt_bridge_system` in `src/layer1/integration.rs` to create debt.
- **Tests:** `tests/integration/medical_debt.rs` (Integration test verified)
