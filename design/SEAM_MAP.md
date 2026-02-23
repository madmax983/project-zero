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

### INT-030: Drone Networks -> Sanctuary Districts
- **Date:** 2026-06-03
- **Systems connected:** `haul_system` (Logistics) -> `ZoneType::Sanctuary` (Zone)
- **Glue added:**
    - Updated `src/layer1/hauling.rs` to filter targets based on Zone.
    - Drones ignore items and stockpiles in Sanctuary zones.
- **Tests:** `tests/integration/drone_sanctuary.rs` (Integration test verified)

### INT-031: Institutional Memory (Items) -> Hauling System
- **Date:** 2026-06-04
- **Systems connected:** `produce_manual_system` (Output) -> `haul_system` (Logistics)
- **Glue added:**
    - Updated `haul_system` in `src/layer1/hauling.rs` to support `CarryingItem`.
    - Updated `utility_ai.rs` to evaluate hauling for generic `Item` entities.
    - Updated `src/layer1/items.rs` with `CarryingItem` component.
- **Tests:** `test_haul_manual_system_lifecycle` in `src/layer1/hauling.rs` (Integration test verified)

### INT-002: Power Grid -> Utility AI (Refining, Farming, Medical)
- **Date:** 2026-02-xx
- **Systems connected:** `PowerConsumer` (Energy) -> `utility_ai` (Decision Making)
- **Glue added:**
    - Updated `src/layer1/utility_ai.rs` (`populate_refining`, `populate_farms`, `populate_hospitals`) to query `PowerConsumer`.
    - Added filter to ignore unpowered buildings (active=false) during candidate population.
- **Tests:** `tests/integration/power_utility.rs` (3 tests verified)

### INT-032: Wind -> Atmosphere Advection
- **Date:** 2026-05-22
- **Systems connected:** `WindGrid` -> `AtmosphereGrid` (Pollution Transport)
- **Glue added:**
    - Updated `src/layer1/atmosphere.rs` to implement semi-Lagrangian advection.
    - Updated `update_atmosphere_system` to consume `WindGrid`.
    - Registered `update_wind_system` in `Layer1SystemSet::Environment` before atmosphere.
    - Initialized `WindGrid` and `GlobalWind` in `src/setup.rs`.
- **Tests:** `tests/integration/wind_atmosphere.rs` (Integration test verified), Fixed regressions in `drone_network` and `hauling_execution`.

### INT-033: Machine Vibration -> Seismic Activity
- **Date:** 2026-02-21
- **Systems connected:** `SeismicSource` (Buildings) -> `VibrationGrid` (Seismic) -> `Flora` (Environment) / `GeologicalEvent` (Geology)
- **Glue added:**
    - Initialized `VibrationGrid` in `src/setup.rs`.
    - Added `SeismicSource` component to industrial buildings in `src/layer1/building.rs`.
    - Registered `update_seismic_system`, `seismic_flora_reaction_system`, `seismic_instability_system` in `src/layer1/systems.rs`.
- **Tests:** `tests/integration/seismic_vibration.rs` (Integration test verified)

### INT-034: Beauty Radius Propagation
- **Date:** 2026-02-22
- **Systems connected:** `BeautySource` (Buildings) -> `BeautyGrid` (Environment) -> `apply_beauty_effects_system` (Pop)
- **Glue added:**
    - Updated `src/layer1/building.rs` to add `beauty_radius()` to `BuildingType`.
    - Updated `src/layer1/beauty.rs` to propagate beauty value over the radius with linear falloff.
- **Tests:** `tests/integration/beauty_radius.rs` (Integration test verified)

### INT-035: Vermin Severity -> Perishable Item Decay
- **Date:** 2026-03-31
- **Systems connected:** `VerminState` (Environment) -> `Perishable` (Items)
- **Glue added:**
    - Added `vermin_item_rot_system` to `src/layer1/integration.rs`.
    - Registered in `src/layer1/systems.rs` (Consumption phase).
- **Tests:** `tests/integration/vermin_spoilage_rot.rs` (Integration test verified)

### INT-003: Medical Notifications
- **Date:** 2026-11-06
- **Systems connected:**
    - `healing_system` -> `medical_treatment_notification_system` (Treatment -> Notification)
    - `evaluate_actions_system` -> `hospitalization_notification_system` (Action Change -> Notification)
    - `death_system` -> `pop_death_notification_system` (Death -> Notification)
- **Glue added:** Systems in `src/layer1/integration.rs`
- **Schedule:** `Observation` set
- **Tests:** `tests/integration/medical_notifications.rs`

### INT-004: Medical System -> Condition Treatment
- **Date:** 2026-02-22
- **Systems connected:** `healing_system` (Medical) -> `CryoTrauma` (Dreams) / `RadiationSickness` (Radioactive)
- **Glue added:**
    - Updated `src/layer1/medical.rs` to treat `CryoTrauma` and `RadiationSickness` in hospitals.
    - Consumes hospital capacity to remove/reduce these conditions.
- **Tests:** `tests/integration/medical_conditions.rs` (2 tests verified)

### INT-005: Pheromones -> Morale
- **Date:** 2026-03-03
- **Systems connected:** `StrangeFlora` (Anomaly) / `XenoMoss` (Flora) -> `PheromoneEmitter` (Environment) -> `pheromone_emission_system` -> `Morale` (Pop)
- **Glue added:**
    - Updated `spawn_initial_anomalies` in `src/layer1/science.rs` to attach `PheromoneEmitter` to `StrangeFlora`.
    - Updated `flora_spread_system` in `src/layer1/flora.rs` to attach `PheromoneEmitter` to `XenoMoss`.
- **Tests:** `tests/integration/pheromone_integration.rs` (3 tests verified)

### INT-036: Magnetic Storm Weather -> Auroral Collector
- **Date:** 2026-03-03
- **Systems connected:** `WeatherState` (Weather) -> `PowerSource` (Energy)
- **Glue added:**
    - Updated `src/layer1/weather.rs` to include `MagneticStorm` in weather generation.
    - Updated `src/layer1/energy/mod.rs` to increase grid demand by 50% during `MagneticStorm`.
    - `AuroralCollector` in `src/layer1/energy/auroral.rs` already listens to this state.
- **Tests:** `tests/integration/weather_energy.rs` (3 tests verified)

### INT-005: Building Operation -> Acoustic Noise
- **Date:** 2026-03-31
- **Systems connected:** `Building` (Production/Power/Tech) -> `NoiseSource` (Acoustic) -> `update_noise_system` (Simulation)
- **Glue added:**
    - Updated `src/layer1/building.rs` to attach `NoiseSource` to industrial buildings.
- **Tests:** `tests/integration/acoustic_buildings.rs` (Integration test verified)
