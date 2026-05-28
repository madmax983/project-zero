### INT-775: The Quantum Famine -> Chronicle
- **Date:** 2026-10-31
- **Systems connected:** `process_market_panic_hoarding` -> `quantum_famine_export_dump_bridge` -> `AddChronicleEvent`
- **Glue added:** `quantum_famine_export_dump_bridge` in `src/layer3/integration.rs` to read `ExportDumpEvent` and emit `AddChronicleEvent` while depositing credits.
- **Tests:** `tests/integration/quantum_famine_bridge.rs`

### INT-473: Bureaucratic Redlining Integration
- **Date:** 2026-06-25
- **Systems connected:** `bureaucratic_redlining_system`, `apply_squatter_visuals_system`, `stateless_squatter_raid_system`, `stateless_expansion_system`
- **Glue added:** Registered systems in `src/layer1/systems/economy.rs`.
- **Tests:** `tests/integration/bureaucratic_redlining.rs`

### INT-1087: Embezzlement Architecture -> Chronicle
- **Date:** 2026-05-07
- **Systems connected:** `process_embezzlement` -> `embezzlement_chronicle_bridge` -> `AddChronicleEvent`
- **Glue added:** `embezzlement_chronicle_bridge` in `src/layer1/core/integration.rs`
- **Schedule:** Added to `Layer1SystemSet::Environment`
- **Tests:** `tests/integration/embezzlement_architecture_bridge.rs`
### INT-1237: Invasive Bureaucracy Nodes -> Empire Stability
- **Date:** 2026-10-31
- **Systems connected:** `expand_bureaucracy_nodes_system` -> `calculate_bureaucracy_stability_system`
- **Glue added:** Initialized `EmpireStability` in `src/simulation.rs` and chained execution in `src/layer1/systems/economy.rs`.
- **Tests:** `tests/integration/invasive_bureaucracy_bridge.rs`

### INT-1079: The Bureaucracy of Vanity -> Economy/Diplomacy
- **Date:** 2026-05-12
- **Systems connected:** `BuildingCompletedEvent` -> `vanity_building_listener_system` -> `ImperialStanding` and `ActiveDemands` -> `vanity_sabotage_system` -> `GlobalEfficiency`
- **Glue added:** Initialized resources `ActiveDemands`, `ImperialStanding`, and `GlobalEfficiency` in `src/simulation.rs`. Registered `vanity_building_listener_system` and `vanity_sabotage_system` in `src/simulation.rs`.
- **Tests:** `tests/integration/bureaucracy_of_vanity_integration.rs`

### INT-477: Ransom Broker -> Chronicle
- **Date:** 2026-05-13
- **Systems connected:** `process_ransom_decisions_system` -> `ransom_broker_chronicle_bridge` -> Chronicle
- **Glue added:** Added `PopRansomedEvent` and `PopLostToPiratesEvent`, bridge in `layer1/core/integration.rs`.
- **Tests:** `tests/integration/ransom_broker_bridge.rs`

### INT-1083: Kinetic Excavation -> Chronicle
- **Date:** 2026-05-10
- **Systems connected:** `KineticStrikeEvent` -> `kinetic_strike_chronicle_bridge` -> `AddChronicleEvent`
- **Glue added:** Added `kinetic_strike_chronicle_bridge` in `src/layer1/core/integration.rs`. Registered in `src/layer1/systems/observation.rs`.
- **Tests:** `tests/integration/kinetic_strike_chronicle.rs`

### INT-642: Propaganda Simulacrum -> Chronicle
- **Date:** 2026-10-31
- **Systems connected:** `Simulacrum` construction -> `simulacrum_chronicle_bridge` -> `AddChronicleEvent`
- **Glue added:** Added `simulacrum_chronicle_bridge` in `src/layer1/core/integration.rs` to detect `Added<Simulacrum>` and emit an `AddChronicleEvent`. Registered in `src/layer1/systems/observation.rs`.
- **Tests:** `tests/integration/propaganda_simulacrum_bridge.rs`

### INT-644: Temporal Echo Chambers -> Power & Chronicle
- **Date:** 2026-07-03
- **Systems connected:** `TemporalChamber` -> `temporal_chamber_power_bridge_system` -> `ColonyResources` & `AddChronicleEvent`
- **Glue added:** Added `temporal_chamber_power_bridge_system` in `src/layer1/core/integration.rs` to deduct fuel based on `energy_cost` and emit an `AddChronicleEvent` temporal shockwave when fuel runs out.
- **Tests:** `tests/integration/temporal_chamber_bridge.rs`

### INT-239: Clutter -> Pathfinding and Beauty
- **Date:** 2026-06-25
- **Systems connected:** `clutter_accumulation_system` (Clutter) -> `find_path_internal` (Pathfinding) and `update_beauty_grid_system` (Beauty)
- **Glue added:**
    - `ClutterGrid` added to `update_beauty_grid_system` in `src/layer1/beauty.rs` to penalize beauty values by -1 per 10 clutter.
    - `ClutterGrid` added to `find_path_internal` in `src/layer1/pathfinding.rs` to add a pathfinding cost penalty.
    - `clutter_accumulation_system` and `clutter_cleaning_system` registered in `src/layer1/systems/environment.rs` and `src/layer1/systems/execution.rs`.
- **Tests:** `tests/integration/clutter_bridge.rs`


### INT-1094: Flora Destruction -> Pioneer Regrowth (Ecological Succession)
- **Date:** 2026-08-01
- **Systems connected:** `process_flora_clearing` (Forestry) -> `ecological_succession_system` (Flora)
- **Glue added:** Updated `process_flora_clearing` to mark `EcologicalState.cleared_recently = true` when clearing flora, triggering `ecological_succession_system` to spawn pioneer species (FireWeed).
- **Tests:** `cargo test process_flora_clearing` and `ecological_succession_tests`

### INT-816: Primitive Retaliation -> Chronicle
- **Date:** 2026-06-25
- **Systems connected:** `primitive_retaliation_system` (Accidental Gods) -> `primitive_retaliation_chronicle_bridge` (Integration) -> `AddChronicleEvent`
- **Glue added:** Added `primitive_retaliation_chronicle_bridge` to emit an `AddChronicleEvent` (`EventImportance::Major`) upon `PrimitiveRetaliationEvent`. Registered in `SimulationSchedule`.
- **Tests:** `tests/integration/primitive_retaliation_bridge.rs`

### INT-948: Orphaned Edicts -> Chronicle
- **Date:** 2026-06-25
- **Systems connected:** `AccessDeniedEvent` and `HackCentralHubEvent` (Edicts) -> `AddChronicleEvent` (Chronicle)
- **Glue added:**
    - `access_denied_chronicle_bridge` and `hack_hub_chronicle_bridge` in `src/layer1/core/integration.rs` to generate chronicle events when orphaned edicts deny access or are hacked.
    - Registered bridges in `src/layer1/systems/observation.rs`.
- **Tests:** `tests/integration/orphaned_edict_bridge.rs` (2 tests verified).

### INT-468: Escape Velocity Economics Integration
- **Date:** 2026-03-27
- **Systems connected:** `PlanetaryTraits` (Layer 1 Quirks) -> `PlanetaryGravity` (Layer 2 Escape Velocity)
- **Glue added:**
    - `escape_velocity_traits_bridge_system` in `src/layer2/integration.rs` to map Layer 1 High/Low Gravity traits to Layer 2 gravity coefficients.
    - Registered bridge in `src/simulation.rs` to run before `process_launch_system`.
- **Tests:** Added `test_escape_velocity_traits_bridge` in `src/layer2/integration.rs` and `test_integration_escape_velocity_traits` in `tests/integration/escape_velocity_traits.rs`.

### INT-452: Movement -> Commuter Tax (Toll)
- **Date:** 2026-03-15
- **Systems connected:** `movement_system` (Execution) -> `transit_toll_system` (Infrastructure/Transit)
- **Glue added:**
    - Updated `transit_toll_system` in `src/layer1/infrastructure/transit.rs` to only process `Pop`s whose `GridPosition` has `Changed<GridPosition>`.
    - Registered `transit_toll_system` in `src/layer1/systems/execution.rs` to run `.after(movement_system)`.
- **Tests:** Added `test_transit_toll_integration` in `src/layer1/infrastructure/transit.rs`.


### INT-446: Waste / Landfills -> Olfactory Map (Scent)
- **Date:** 2026-03-09
- **Systems connected:** `ResourceItem` (Waste) / `Building` (Landfill) -> `waste_scent_bridge` (Integration) -> `ScentEmitter` (Olfactory)
- **Glue added:**
    - Added `waste_scent_bridge` in `src/layer1/integration.rs` which queries for `ResourceItem`s with type `Waste` and `Building`s with type `Landfill` that lack a `ScentEmitter`.
    - It then inserts a `ScentEmitter` with `Foul` scent and strength scaled dynamically or set to a static high value for buildings.
    - Registered `waste_scent_bridge` in `src/layer1/systems/observation.rs` before `scent_diffusion_system`.
- **Tests:** `tests/integration/waste_scent.rs` (2 tests verified)

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

### INT-037: Customs Vetting -> Zone Location
- **Date:** 2026-10-27
- **Systems connected:** `ZoneGrid` (Map) -> `vetting_work_system` (Customs)
- **Glue added:**
    - Updated `src/layer1/customs.rs` to check if entity is physically at `ZoneType::Customs` before processing.
- **Tests:** `tests/integration/customs_vetting.rs` (Integration test verified)

### INT-011: Hygiene -> Recycling
- **Date:** 2026-02-01
- **Systems connected:** `shower_use_system` (Hygiene) -> `haul_system` (Logistics) -> `recycle_processing_system` (Recycling)
- **Glue added:**
    - Updated `src/layer1/hygiene.rs` to spawn `ItemType::Waste` when filth is cleaned.
    - Updated `src/layer1/building.rs` to add `Inventory` to `Recycler`.
    - Updated `src/layer1/hauling.rs` to target `Recycler` for `ItemType::Waste` and deposit into inventory.
- **Tests:** `tests/integration/hygiene_recycling.rs` (3 tests verified)

### INT-012: System Mining -> Colony Resources
- **Date:** 2026-02-01
- **Systems connected:** `fleet_movement_system` (Layer 2) -> `fleet_unload_system` (Integration) -> `ColonyResources` (Layer 1)
- **Glue added:**
    - Added `fleet_unload_system` to `src/layer1/integration.rs`.
    - Registered in `src/simulation.rs` after `fleet_movement_system`.
    - Automatically unloads cargo from fleets orbiting the colony location.
- **Tests:** `tests/integration/mining_colony.rs` (Integration test verified)

### INT-013: Volatile Explosion -> Pop Health
- **Date:** 2026-02-26
- **Systems connected:** `handle_explosion_system` (Volatile) -> `Health` (Pop/Fauna)
- **Glue added:**
    - Updated `src/layer1/volatile.rs` to query `Health` components.
    - `handle_explosion_system` now iterates through health entities and applies damage.
- **Tests:** `tests/integration/volatile_health.rs` (Integration test verified)

### INT-014: The Hum -> Public Grievances
- **Date:** 2026-02-26
- **Systems connected:** `HumSource` (Environment) / `Trait::Sensitive` (Pop) -> `post_grievance_system` (Social)
- **Glue added:**
    - Updated `src/layer1/social/grievances.rs` to check for Sensitive trait and high stress.
    - Overrides generic grievance content with Hum-specific messages.
- **Tests:** `tests/integration/hum_grievances.rs` (Integration test verified)

### INT-015: Genetic Sample -> Gene Bank Logistics
- **Date:** 2026-03-27
- **Systems connected:** `collect_sample_action` (Gene Bank) -> `haul_system` (Logistics) -> `GeneBank` (Building)
- **Glue added:**
    - Updated `src/layer1/utility_eval_types.rs` to track `carrying_item_type`.
    - Updated `src/layer1/utility_ai_population.rs` to populate `gene_banks` and `carrying_item_type`.
    - Updated `evaluate_haul` in `src/layer1/actions.rs` to prioritize Gene Bank destinations for genetic samples.
    - Updated `haul_system` in `src/layer1/hauling.rs` to detect Gene Bank targets and use `store_sample`.
- **Tests:** `tests/integration/gene_bank_logistics.rs` (Integration test verified)

### INT-039: Infinite Archive -> Chronicle
- **Date:** 2026-03-03
- **Systems connected:** `purge_tech` -> `AddChronicleEvent`
- **Glue added:**
    - Emits an `AddChronicleEvent` when a tech is purged to note that the secrets have been forgotten.
- **Tests:** `tests/integration/infinite_archive_chronicle.rs`

### INT-040: Sanctuary -> Utility AI
- **Date:** 2026-03-03
- **Systems connected:** `SanctuaryManager` -> `UtilityAI`
- **Glue added:**
    - Populated valid `Sanctuary` locations in AI buffer.
    - `evaluate_visit_sanctuary` triggers `ActionType::VisitSanctuary` when pop stress > 40%.
- **Tests:** `tests/integration/sanctuary_ai.rs`

### INT-038: MegaQuakeEvent -> Chronicle System
- **Date:** 2026-03-03
- **Systems connected:** `MegaQuakeEvent` (Geology) -> `mega_quake_chronicle_bridge` (Integration) -> `AddChronicleEvent` (Chronicle)
- **Glue added:**
    - Added `mega_quake_chronicle_bridge` in `src/layer1/integration.rs` to convert `MegaQuakeEvent` to `AddChronicleEvent`.
    - Registered in `src/layer1/systems/observation.rs`.
    - Initialized `MegaQuakeEvent` resource in `setup.rs`.
- **Tests:** `tests/integration/mega_quake_chronicle.rs` (Integration test verified)

### INT-261: Shadow Markets -> Trade Execution
- **Date:** 2026-03-04
- **Systems connected:** `execute_trade` -> `ShadowTrader`
- **Glue added:**
    - Validated `ShadowTrader` properties.
    - Verified `TradeDeal` exchanges resources successfully.
- **Tests:** `tests/integration/shadow_market_trade.rs`

### INT-249: Hologram Failure -> Chronicle
- **Date:** 2026-03-04
- **Systems connected:** `update_holograms_system` -> `AddChronicleEvent`
- **Glue added:**
    - `hologram_failure_chronicle_bridge` converts `HologramFailureEvent` to `AddChronicleEvent`
- **Tests:** `tests/integration/hologram_failure.rs`

### INT-257: Subspace Pen Pals
- **Date:** 2026-03-05
- **Systems connected:** `update_pen_pals_system` -> `ColonyResources`, `FactionMember`
- **Glue added:**
    - Modified `update_pen_pals_system` to correctly modify `ColonyResources` knowledge pool.
    - Handled Espionage chance to reduce knowledge.
    - Handled Policy checking (`FirewallComms`).
    - Handled Ethics shift mapping to `FactionMember` component.
- **Tests:** `tests/integration/subspace_pen_pals.rs` (Integration test verified)

### INT-224: Scapegoat Denouncement -> Chronicle
- **Date:** 2026-03-07
- **Systems connected:** `denounce_scapegoat_system` (Unrest) -> `AddChronicleEvent` (Chronicle)
- **Glue added:**
    - `scapegoat_chronicle_bridge` converts `DenounceEvent` to `AddChronicleEvent`.
- **Tests:** `tests/integration/scapegoat_chronicle.rs` (Integration test verified)

### INT-260: Industrial Rhythm -> Pop Morale
- **Date:** 2026-03-08
- **Systems connected:** `update_rhythm_system` (Industrial Rhythm) -> `industrial_rhythm_morale_bridge` (Integration) -> `Morale` (Pop)
- **Glue added:**
    - `industrial_rhythm_morale_bridge` converts `MachineRhythm` with sync bonus to a `MoodModifier` applied to nearby `Pop`s' `Morale`.
- **Tests:** `tests/integration/industrial_rhythm.rs`

### INT-347: Great Works -> Chronicle
- **Date:** 2026-03-08
- **Systems connected:** `GreatWorkCompletedEvent` (Construction) -> `great_work_chronicle_bridge` (Integration) -> `AddChronicleEvent` (Chronicle)
- **Glue added:**
    - `great_work_chronicle_bridge` to convert `GreatWorkCompletedEvent` to `AddChronicleEvent` (Legendary)
- **Tests:** `tests/integration/great_work_chronicle.rs`

### INT-348: Needs -> Black Market
- **Date:** 2026-03-08
- **Systems connected:** `Needs` (Pop) -> `update_unmet_luxury_system` (Integration) -> `ColonyStats.unmet_luxury` (Black Market)
- **Glue added:**
    - `update_unmet_luxury_system` to update stats based on pop leisure levels
- **Tests:** `tests/integration/unmet_needs_black_market.rs`

### INT-349: Ancestral Graves -> Sacrilege -> Unrest
- **Date:** 2026-03-08
- **Systems connected:** `SacrilegeEvent` (Building) -> `sacrilege_unrest_bridge` (Integration) -> `Unrest` (Unrest) & `AddChronicleEvent` (Chronicle)
- **Glue added:**
    - `sacrilege_unrest_bridge` to increase unrest and log a chronicle event
- **Tests:** `tests/integration/sacrilege_effects.rs`

### INT-447: Orbital Drop Logistics -> Chronicle
- **Date:** 2026-03-08
- **Systems connected:** `process_orbital_drops` (Logistics) -> `orbital_drop_chronicle_bridge` (Integration) -> `AddChronicleEvent` (Chronicle)
- **Glue added:**
    - Added `orbital_drop_chronicle_bridge` in `src/layer1/integration.rs` to convert `OrbitalDropEvent` to `AddChronicleEvent` with `EventImportance::Major`.
    - Registered in `src/layer1/systems/observation.rs` under `Layer1SystemSet::Observation`.
- **Tests:** `tests/integration/orbital_drop_chronicle.rs` (Integration test verified)

### INT-450: Light Pollution -> Fauna Aggression
- **Date:** 2026-03-09
- **Systems connected:** `NocturnalFauna` (Light Pollution) -> `nocturnal_aggression_bridge_system` (Integration) -> `Fauna` (Fauna Behavior)
- **Glue added:**
    - `nocturnal_aggression_bridge_system` connects the two by calculating an actual detection_range bonus from `aggression`.
- **Tests:** `tests/integration/light_pollution_fauna.rs`

### INT-451: Override Will -> Chronicle
- **Date:** 2026-03-11
- **Systems connected:** `process_override_will_system` (Spiteful Will) -> `override_will_chronicle_bridge` (Integration) -> `AddChronicleEvent` (Chronicle)
- **Glue added:**
    - `override_will_chronicle_bridge` converts `OverrideWillEvent` to `AddChronicleEvent`
- **Tests:** `tests/integration/spiteful_will_chronicle.rs`

### INT-291: NeuralShock -> Mental Breakdown
- **Date:** 2026-03-10
- **Systems connected:** `handle_hub_death_system` -> `apply_neural_shock_system` -> `evaluate_actions_system`
- **Glue added:** `apply_neural_shock_system` in `src/layer1/integration.rs` translates `NeuralShock` into `MentalState::Broken(MentalBreakType::Daze)`.
- **Schedule:** Registered in `Layer1SystemSet::Observation`, specifically `.after(crate::layer1::tech::neural_leech::handle_hub_death_system)`.
- **Tests:** `tests/integration/neural_leech_unrest.rs` (1 test)

### INT-411: Machine Awakening -> Chronicle
- **Date:** 2026-03-13
- **Systems connected:** `process_bot_sentience` (Machine Awakening) -> `bot_awakening_chronicle_bridge` (Integration) -> `AddChronicleEvent` (Chronicle)
- **Glue added:** `bot_awakening_chronicle_bridge` in `src/layer1/integration.rs` translates the addition of the `Awakened` component into a Major `AddChronicleEvent`.
- **Schedule:** Registered in `Layer1SystemSet::Observation`, evaluated `.after(crate::layer1::tech::machine_awakening::process_bot_sentience)`.
- **Tests:** `tests/integration/machine_awakening_chronicle.rs` (1 test)

### INT-269: VR Pod -> Utility AI
- **Date:** 2026-03-15
- **Systems connected:** `InVrPod` (VR Pod) -> `collect_pop_data` (Utility AI)
- **Glue added:**
    - Modified `src/layer1/utility_ai_population.rs` to filter out entities with the `InVrPod` component from Utility AI evaluations.
- **Tests:** `tests/integration/vr_pod_integration.rs`

### INT-293: Clear Flora -> Biosphere Empathy
- **Date:** 2026-03-15
- **Systems connected:** `process_flora_clearing` (Flora) -> `FloraDamagedEvent` -> `handle_flora_damage_empathy_system` (Biosphere Empathy)
- **Glue added:**
    - Updated `src/layer1/flora.rs` to send a `FloraDamagedEvent` whenever `Flora` entities are despawned via the `ClearFlora` designation.
- **Tests:** `tests/integration/flora_empathy.rs` (1 test verified)

### INT-296: Martyr's Engine Integration
- **Date:** 2026-06-25
- **Systems connected:** `attune_engine_system` (MartyrsEngine) -> `AddChronicleEvent` (Chronicle) & `LightSource` (VFX) & `StressTracker` (with `Traits` influence)
- **Glue added:**
    - Updated `attune_engine_system` to emit an `EventImportance::Legendary` `AddChronicleEvent` and mitigate the stress penalty based on traits like `Cannibal`, `Outsider`, `EmpathicLink`, or `Compassionate`.
    - Updated `process_martyrs_engine` to add an eerie red `LightSource` to the active engine and remove it when it decays.
- **Tests:** `tests/integration/martyrs_engine.rs` (Integration test verified)

### INT-211: Gut Biome
- **Date:** 2026-03-16
- **Systems connected:** `consume_food_system` (Farm) -> `GutBiome`
- **Glue added:**
    - Updated `consume_food_system` in `src/layer1/farm.rs` to fetch `GutBiome` and apply effects (`Gut Comfort` or `Indigestion`).
    - Added `GutBiome` initialized within `PopBundle`.
- **Tests:** `src/layer1/gut_biome.rs` tests (5 tests verified)

### INT-515: Deep Crust Geomes -> Environmental Damage
- **Date:** 2026-03-27
- **Systems connected:** `GeomeHazard` (Geomes) -> `environmental_damage_system` (Integration) -> `Health` (Health)
- **Glue added:**
    - Updated `environmental_damage_system` in `src/layer1/geomes.rs` to correctly accumulate damage for multiple hazards on the same tile using a HashMap and apply it using `Health::take_damage`.
    - Integrated with `diffuse_geome_hazards_system` that spawns hazards when Geome boundaries are breached.
- **Tests:** `tests/integration/geome_hazards.rs` (1 integration test verified)

### INT-540: Exodus -> ColonyResources
- **Date:** 2026-03-19
- **Systems connected:** `build_ark_system` and `cannibalize_infrastructure_system` integrated into `Layer1SystemSet::Economy`
- **Glue added:** Directly registered in `src/layer1/systems/economy.rs`
- **Schedule:** Chained in Economy loop
- **Tests:** `tests/integration/exodus_ark.rs` (1 test)

### INT-467: Psychic Background Radiation -> Pop Psychology
- **Date:** 2026-03-18
- **Systems connected:** `PsychicBackground` (Environment) -> `apply_psychic_radiation_system` (Psychology)
- **Glue added:**
    - `apply_psychic_radiation_system` in `src/layer1/psychic.rs` that reads `PsychicBackground` and modifies `Needs.rest` and `StressTracker.accumulated_stress`.
    - Registered in `src/layer1/systems/observation.rs`.
- **Tests:** `tests/integration/psychic_psychology.rs` (Integration test verified)

### INT-538: Trade Routes -> ColonyResources
- **Date:** 2026-03-20
- **Systems connected:** `Colony` component (Layer 2 Trade Route) -> `ColonyResources` (Layer 1)
- **Glue added:**
    - Two bridge systems: `pre_trade_route_sync_system` and `post_trade_route_sync_system`.
    - Added `HomeColony` marker to identify the player's colony.
- **Tests:** `tests/integration/trade_routes_bridge.rs` (2 tests verified)

### INT-539: Penal Contracts -> ColonyResources & Chronicle
- **Date:** 2026-03-20
- **Systems connected:** `ColonyFunds` (Penal Contract) -> `ColonyResources.credits` (Economy), `check_prisoner_status_system` (Penal Contract) -> `AddChronicleEvent` (Chronicle)
- **Glue added:**
    - Added `PrisonerDiedEvent` in `penal_contracts.rs`.
    - `penal_funds_to_resources_system` moves `ColonyFunds` to `ColonyResources.credits`.
    - `prisoner_death_chronicle_bridge_system` transforms `PrisonerDiedEvent` to `AddChronicleEvent`.
- **Tests:** `tests/integration/penal_contracts_bridge.rs` (2 tests verified)

### INT-533: SensorGlitchEvent -> Chronicle
- **Date:** 2026-03-20
- **Systems connected:** `process_mutiny_effects_system` -> `sensor_glitch_chronicle_bridge_system` -> `chronicle_event_handler_system`
- **Glue added:** `sensor_glitch_chronicle_bridge_system` in `src/layer2/integration.rs`
- **Schedule:** Chained in Update after `process_mutiny_effects_system`
- **Tests:** `tests/integration/silent_mutiny_chronicle.rs`

### INT-544: RebellionEvent -> AddChronicleEvent
- **Date:** 2026-03-21
- **Systems connected:** `check_governor_rebellion_system` (Layer 2 Governance) -> `rebellion_chronicle_bridge_system` (Integration) -> `AddChronicleEvent` (Chronicle)
- **Glue added:**
    - `rebellion_chronicle_bridge_system` in `src/layer2/integration.rs` to convert `RebellionEvent` to `AddChronicleEvent` with `EventImportance::Major`.
    - Chained in `src/simulation.rs` in `Layer2SystemSet`.
- **Tests:** `tests/integration/governance_rebellion.rs` (Integration test verified)

### INT-469: Galactic Council -> Trade Sanctions
- **Date:** 2026-03-22
- **Systems connected:** `GalacticCouncil` (Council) -> `enforce_resolutions_system` (Council)
- **Glue added:** Added `GalacticCouncil` resource and `enforce_resolutions_system` system to `SimulationSchedule`.
- **Tests:** `src/layer3/council.rs` (Integration tests verified)

### INT-548: Biomass Tariff -> Economy
- **Date:** 2026-03-22
- **Systems connected:** `TradeDeal` (Biomass Tariff) -> `process_biomass_tariff_system` (Biomass Tariff)
- **Glue added:** Added `TradeDeal` event and `process_biomass_tariff_system` system to `SimulationSchedule` (chained after `post_trade_route_sync_system`).
- **Tests:** `src/layer2/trade/biomass_tariff.rs` (Integration tests verified)

### INT-545: Disaster Tourism -> ColonyResources & Chronicle
- **Date:** 2026-03-24
- **Systems connected:** `GriefTouristArrivalEvent` (Tourism) -> `process_grief_tourist_arrival_system` (Integration) -> `ColonyResources` (Economy) & `AddChronicleEvent` (Chronicle)
- **Glue added:**
    - `process_grief_tourist_arrival_system` in `src/layer2/integration.rs` translates `GriefTouristArrivalEvent` into credits and narrative chronicle events.
- **Schedule:** Chained in `Layer2SystemSet` after `process_disaster_tourism_system`.
- **Tests:** `tests/integration/disaster_tourism_bridge.rs` (Integration test verified)

### INT-453: Nanite Fabrication -> Chronicle
- **Date:** 2026-03-24
- **Systems connected:** `nanite_fabrication_system` -> `nanite_breach_chronicle_bridge`
- **Glue added:** `nanite_breach_chronicle_bridge` in `src/layer1/integration.rs`
- **Tests:** `tests/integration/nanite_fabrication_chronicle.rs`
### INT-565: Gene Splicing -> Chronicle
- **Date:** 2026-03-27
- **Systems connected:** `process_gene_splicing_system` -> `gene_splicing_chronicle_bridge`
- **Glue added:** Added `GeneSplicingResultEvent` and `gene_splicing_chronicle_bridge` in `src/layer1/integration.rs`. Registered in `Layer1SystemSet::Economy` and `Observation`.
- **Tests:** `tests/integration/gene_splicing_chronicle.rs`
### INT-409: Shipbreaking -> Mining Logistics
- **Date:** 2026-03-27
- **Systems connected:** `SpawnCrashedShipEvent` & `MineEvent` registered globally. `spawn_crashed_ship_system`, `mine_system`, and `hull_destroyed_system` registered in `Layer1SystemSet::Execution`.
- **Glue added:** Event buffer registrations in `cleanup.rs`, schedule in `execution.rs`.
- **Tests:** `tests/integration/shipbreaking_bridge.rs` (1 test verified)
### INT-409: Shipbreaking -> Mining Logistics
- **Date:** 2026-03-27
- **Systems connected:** `SpawnCrashedShipEvent` & `MineEvent` registered globally. `spawn_crashed_ship_system`, `mine_system`, and `hull_destroyed_system` registered in `Layer1SystemSet::Execution`.
- **Glue added:** Event buffer registrations in `cleanup.rs`, schedule in `execution.rs`.
- **Tests:** `tests/integration/shipbreaking_bridge.rs` (1 test verified)
### INT-621: Mass Driver -> Chronicle
- **Date:** 2026-03-27
- **Systems connected:** `package_arrival_system` (Mass Driver) -> `mass_driver_chronicle_bridge` (Integration) -> `AddChronicleEvent` (Chronicle)
- **Glue added:**
    - `mass_driver_chronicle_bridge` in `src/layer1/integration.rs` converts `BombardmentEvent` to `AddChronicleEvent`.
- **Schedule:** Registered in `Layer1SystemSet::Observation`, chained after `package_arrival_system`.
- **Tests:** `tests/integration/mass_driver_chronicle.rs`

### INT-647: Cascade Failure -> Chronicle
- **Date:** 2026-03-27
- **Systems connected:** `evaluate_system_logistics` & `update_sector_defenses` -> `AddChronicleEvent` (Chronicle)
- **Glue added:** `logistics_strained_chronicle_bridge` and `defense_weakened_chronicle_bridge` in `src/layer2/integration.rs`.
- **Schedule:** Registered in `src/simulation.rs` after the primary cascade failure systems.
- **Tests:** `tests/integration/cascade_failure_chronicle.rs`
### INT-183: Geodetic Sentience -> Chronicle
- **Date:** 2026-03-28
- **Systems connected:** `form_golem_system` (Geodetic Sentience) -> `golem_formed_chronicle_bridge_system` (Integration) -> `AddChronicleEvent` (Chronicle)
- **Glue added:**
    - `GolemFormedEvent` in `src/layer1/geodetic.rs`.
    - `golem_formed_chronicle_bridge_system` in `src/layer1/integration.rs` translates `GolemFormedEvent` to `AddChronicleEvent`.
- **Schedule:** Chained in `Layer1SystemSet::Environment` after `form_golem_system`.
- **Tests:** `tests/integration/geodetic_integration.rs` (1 test verified)

### INT-672: Sensor Ambiguity
- **Date:** 2026-03-30
- **Systems connected:** `FleetFaction` -> `Sensors` & `SensorContact`
- **Glue added:**
    - `assign_sensors_to_player_fleets_system` to automatically give player fleets sensors.
    - `ensure_player_fleets_identified_system` to prevent player fleets from becoming `UnidentifiedContact` to the player.
    - Added UI obscuration logic for `UnidentifiedContact` in `render_system_view`.
- **Schedule:** Registered after `resolve_sensors_system`.
- **Tests:** `tests/integration/sensor_ambiguity_bridge.rs`

### INT-688: Moon Hermits -> Chronicle
- **Date:** 2026-03-30
- **Systems connected:** `process_hermit_desertions` (Moon Hermits) -> `moon_hermits_chronicle_bridge_system` (Integration) -> `AddChronicleEvent` (Chronicle)
- **Glue added:**
    - `moon_hermits_chronicle_bridge_system` in `src/layer2/integration.rs` converts `PopDesertedEvent` to `AddChronicleEvent`.
- **Schedule:** Registered in `src/simulation.rs`, chained after `process_hermit_desertions`.
- **Tests:** `tests/integration/moon_hermits_chronicle.rs`

### INT-727: Empathic Plague -> Needs/Morale
- **Date:** 2026-04-01
- **Systems connected:** `process_empathic_resonance` (Empathic Plague) -> `Morale` & `Needs`
- **Glue added:** Registered `process_empathic_resonance` in `src/simulation.rs`.
- **Schedule:** Chained in Layer 2 after `Layer1SystemSet::Observation`.
- **Tests:** `tests/integration/empathic_plague_bridge.rs`

### INT-735: Genetic Crop Modification -> Chronicle
- **Date:** 2026-04-01
- **Systems connected:** `process_mutations` (Genetic Crop Modification) -> `crop_mutation_chronicle_bridge` (Integration) -> `AddChronicleEvent` (Chronicle)
- **Glue added:**
    - `crop_mutation_chronicle_bridge` in `src/layer1/integration.rs` converts `CropMutationEvent` to `AddChronicleEvent`.
- **Schedule:** Registered in `Layer1SystemSet::Observation`, chained with the other integrations.
- **Tests:** `tests/integration/crop_mutation_chronicle.rs`

### INT-546: Reverse Quarantine -> Chronicle
- **Date:** 2026-04-01
- **Systems connected:** `process_refugee_decisions_system` (Reverse Quarantine) -> `reverse_quarantine_chronicle_bridge` (Integration) -> `AddChronicleEvent` (Chronicle)
- **Glue added:**
    - `reverse_quarantine_chronicle_bridge` in `src/layer2/integration.rs` converts `RefugeeFleetEvent` (when rejected) to `AddChronicleEvent` with `EventImportance::Major`.
- **Schedule:** Chained in `Layer2SystemSet` after `process_refugee_decisions_system`.
- **Tests:** `tests/integration/reverse_quarantine_bridge.rs`

### INT-772: Biomass Commute -> PopDied
- **Date:** 2026-04-10
- **Systems connected:** `digest_transit_contents` (Biomass Network) -> `PopDied` (Pop lifecycle)
- **Glue added:** Modifies `digest_transit_contents` to emit `PopDied` when an entity has a `PopName`.
- **Tests:** `tests/integration/biomass_commute_bridge.rs`

### INT-773: Stellar Weather Navigation -> FleetHealth/Chronicle
- **Date:** 2026-04-10
- **Systems connected:** `apply_stellar_weather_effects` (Stellar Weather) -> `stellar_weather_damage_bridge_system` (Integration) -> `FleetHealth` & `AddChronicleEvent`
- **Glue added:** Added `stellar_weather_damage_bridge_system` to apply `FleetDamagedEvent` damage to ships, despawn if health drops below 0, and record it in Chronicle. Registered in `Layer2SystemSet`.
- **Tests:** `tests/integration/stellar_weather_bridge.rs`

### INT-774: Architectural Grafting -> Chronicle
- **Date:** 2026-04-10
- **Systems connected:** `process_grafting` (Grafting) -> `grafting_chronicle_bridge` (Integration) -> `AddChronicleEvent`
- **Glue added:** Added `grafting_chronicle_bridge` to emit an `AddChronicleEvent` (`EventImportance::Minor`) upon `GraftBuildingEvent`. Registered in `Layer1SystemSet::Observation`.
- **Tests:** `tests/integration/architectural_grafting_bridge.rs`

### INT-776: Black Market Terraforming -> Chronicle
- **Date:** 2026-04-10
- **Systems connected:** `trigger_rogue_terraforming` (Black Market Terraforming) -> `black_market_terraforming_bridge` (Integration) -> `AddChronicleEvent`
- **Glue added:** Added `black_market_terraforming_bridge` to emit an `AddChronicleEvent` (`EventImportance::Major`) upon `RogueTerraformEvent`. Registered in `simulation.rs`.
- **Tests:** `tests/integration/black_market_terraforming_bridge.rs`

### INT-570: Bio-Acoustic Miasma Paranoia -> Stress Breakdown
- **Date:** 2026-04-10
- **Systems connected:** `broadcast_miasma_secrets` (Bio-Acoustic Miasma) -> `paranoia_stress_bridge_system` (Integration) -> `check_stress_breakdown_system` (Stress)
- **Glue added:** Added `paranoia_stress_bridge_system` in `src/layer1/integration.rs` to convert `ParanoiaTracker.level` into `StressTracker.accumulated_stress`.
- **Schedule:** Registered in `Layer1SystemSet::Observation`, chained correctly between broadcast and stress check.
- **Tests:** `tests/integration/bio_acoustic_miasma_bridge.rs` (2 tests verified)

### INT-768: Dynastic Succession -> Chronicle
- **Date:** 2026-04-10
- **Systems connected:** `process_succession_system` (Dynastic Succession) -> `dynastic_succession_chronicle_bridge` & `dynastic_crisis_chronicle_bridge` (Integration) -> `AddChronicleEvent`
- **Glue added:** Added `SuccessionEvent` and `SuccessionCrisisEvent`. Created bridge systems in `src/layer3/integration.rs`.
- **Tests:** `tests/integration/dynastic_succession_bridge.rs`

### INT-777: Void Whispers -> Rumor Web & Chronicle
- **Date:** 2026-04-10
- **Systems connected:** `spread_whispers_to_colony` (Void Whispers) -> `MemeticCarrier` (Memetics) & `void_whispers_chronicle_bridge` (Integration) -> `AddChronicleEvent`
- **Glue added:** Refactored `spread_whispers_to_colony` to apply the actual `MemeticCarrier` component. Added `void_whispers_chronicle_bridge` to emit a Chronicle event upon fleet return with whispers.
- **Tests:** `tests/integration/void_whispers_chronicle.rs`

### INT-767: Orbital Commute System Scheduling
- **Date:** 2026-04-10
- **Systems connected:** `process_orbital_commutes` (Orbital Commute) -> `Layer1SystemSet::Execution` (Execution Schedule)
- **Glue added:** Registered `process_orbital_commutes` in `src/layer1/systems/execution.rs` correctly after start-of-tick actions.
- **Tests:** `tests/integration/orbital_commute_bridge.rs`

### INT-770: Temporal Ghost Towns -> Chronicle
- **Date:** 2026-05-18
- **Systems connected:** `process_temporal_stutters` (Temporal Ghost Towns) -> `temporal_stutter_chronicle_bridge` (Integration) -> `AddChronicleEvent`
- **Glue added:** Added `temporal_stutter_chronicle_bridge` to emit an `AddChronicleEvent` (`EventImportance::Minor`) upon `TemporalStutterEvent`. Registered in `Layer1SystemSet::Observation`.
- **Tests:** `tests/integration/temporal_ghost_towns_bridge.rs`

### INT-771: Parasitic Architecture -> Chronicle
- **Date:** 2026-05-18
- **Systems connected:** `process_megastructure_consumption` (Parasitic Architecture) -> `parasitic_architecture_chronicle_bridge` (Integration) -> `AddChronicleEvent`
- **Glue added:** Added `parasitic_architecture_chronicle_bridge` to emit an `AddChronicleEvent` (`EventImportance::Minor`) upon `BuildingConsumedEvent`. Registered in `Layer1SystemSet::Observation`.
- **Tests:** `tests/integration/parasitic_architecture_bridge.rs`

### INT-778: Hyperlane Collapse -> Chronicle
- **Date:** 2026-05-18
- **Systems connected:** `process_hyperlane_collapse_system` (Hyperlane Collapse) -> `hyperlane_collapse_chronicle_bridge` (Integration) -> `AddChronicleEvent`
- **Glue added:** Added `hyperlane_collapse_chronicle_bridge` to emit an `AddChronicleEvent` (`EventImportance::Major`) upon `TradeRouteSeveredEvent`. Registered in `SimulationSchedule`.
- **Tests:** `tests/integration/hyperlane_collapse_bridge.rs`

### INT-837: Skyhook Launch System Scheduling
- **Date:** 2026-04-07
- **Systems connected:** `process_skyhook_launch` (Skyhooks) -> `SimulationSchedule` (Execution Schedule)
- **Glue added:** Registered `process_skyhook_launch` in `src/simulation.rs` correctly after syzygy effects and initialized `LaunchIntent` resource.
- **Tests:** `tests/integration/skyhook_launch_bridge.rs`

### INT-861: Stellar Cartography JumpRisk -> Damage/Chronicle
- **Date:** 2026-04-10
- **Systems connected:** `handle_jump_risk_system` (Stellar Cartography) -> `jump_risk_bridge_system` (Integration) -> `FleetDamagedEvent` & `AddChronicleEvent`
- **Glue added:** Added `jump_risk_bridge_system` to consume `JumpRisk` from fleets and apply hull damage, as well as emit an `AddChronicleEvent` (`EventImportance::Major`). Registered in `SimulationSchedule`.
- **Tests:** `tests/integration/jump_risk_bridge.rs`

### INT-874: The Blob -> Building Destruction
- **Date:** 2026-04-10
- **Systems connected:** `blob_expansion_system` (The Blob) -> `blob_building_destruction_system` (Integration) -> `BuildingRemovedEvent`
- **Glue added:** Added `blob_building_destruction_system` to emit an `BuildingRemovedEvent` and despawn buildings when the Blob expands onto their location.
- **Tests:** `tests/integration/blob_building_destruction.rs`

### INT-573: The Silent Generation -> Trauma Tracker
- **Date:** 2026-06-15
- **Systems connected:** `PopDied` (Pop lifecycle) & `ColonyResources` (Economy) -> `trauma_death_bridge_system`, `famine_tracking_system`, and `trauma_decay_system` (Integration) -> `TraumaTracker` (Stress)
- **Glue added:** Added bridge systems in `src/layer1/integration.rs` to track deaths and famine and apply trauma decay. Registered in `Layer1SystemSet::Observation`.
- **Tests:** `tests/integration/the_silent_generation_bridge.rs`

### INT-947: Aesthetic Edict -> Chronicle
- **Date:** 2026-06-20
- **Systems connected:** `aesthetic_edict_chronicle_bridge` (Integration) -> `AddChronicleEvent` (Chronicle)
- **Glue added:** Added `aesthetic_edict_chronicle_bridge` to emit an `AddChronicleEvent` when `Policy::Aesthetic` is added or removed from `ColonyPolicies`. Registered in `Layer1SystemSet::Observation`.
- **Tests:** `tests/integration/aesthetic_edict_chronicle_bridge.rs`

### INT-975: Geothermal Heartbeat -> Explosion System
- **Date:** 2026-07-02
- **Systems connected:** `geothermal_decay_system` (Layer 1 Geothermal Pulse) -> `ExplosionEvent` (Volatile Systems)
- **Glue added:**
    - Replaced custom neighbor damage logic in `geothermal_decay_system` with emitting an `ExplosionEvent` with `damage: 50.0` and `radius: 1`.
- **Tests:** `tests/integration/geothermal_explosion.rs` (1 test verified).

### INT-805: Digital Immortality -> Chronicle
- **Date:** 2026-06-01
- **Systems connected:** `handle_mind_upload` (Digital Immortality) -> `digital_immortality_chronicle_bridge` (Integration) -> `AddChronicleEvent`
- **Glue added:** Added `digital_immortality_chronicle_bridge` to emit an `AddChronicleEvent` (`EventImportance::Major`) upon `MindUploadEvent`. Registered in `Layer1SystemSet::Observation`.
- **Tests:** `tests/integration/digital_immortality_chronicle.rs`

### INT-060: Vacuum Pressure -> Acoustic Shadows
- **Date:** 2026-06-25
- **Systems connected:** `update_pressure_system` -> `update_noise_system`
- **Glue added:** Moved `update_noise_system` and `apply_noise_effects_system` from `Layer1SystemSet::Execution` to `Layer1SystemSet::Environment` so they execute after `update_pressure_system`
- **Tests:** `tests/integration/vacuum_noise_bridge.rs`

### INT-862: Ventilation Networks -> Atmosphere Diffusion
- **Date:** 2026-06-02
- **Systems connected:** `VentConnection` (Ventilation) -> `simulate_diffusion_system` (Atmosphere)
- **Glue added:**
    - `simulate_diffusion_system` in `src/layer1/nature/atmosphere.rs` now queries `VentConnection`s and uses `calculate_vent_airflow` to determine diffusion transmissivity through walls.
- **Tests:** `tests/integration/ventilation_atmosphere_bridge.rs` (2 tests verified)

### INT-895: Ephemeral Moons -> Energy System
- **Date:** 2026-06-25
- **Systems connected:** `apply_moon_modifiers_system` (Ephemeral Moons) -> `PowerSource` (Energy)
- **Glue added:** Scheduled `apply_moon_modifiers_system` in `Layer1SystemSet::Economy` between `update_solar_output_system` and `power_grid_system` to ensure solar yield boosts apply before distribution.
- **Tests:** `tests/integration/ephemeral_moons_bridge.rs` (1 test)

### INT-784: The Ephemeral Market -> Simulation Schedule
- **Date:** 2026-06-25
- **Systems connected:** `spawn_ephemeral_market_system`, `process_market_despawn_system`, `fulfill_market_trade_system` (Ephemeral Market) -> `SimulationSchedule`
- **Glue added:** Registered the ephemeral market systems in `src/simulation.rs` under Layer 3 Integration, and initialized `MarketSpawnEvent`, `MarketTradeEvent`, and `MarketTradeFailedEvent` in `src/setup.rs` and `src/layer1/systems/cleanup.rs`.
- **Tests:** `tests/integration/ephemeral_market_bridge.rs`

### INT-900: Latent Psionics -> Fire Spawn
- **Date:** 2026-04-17
- **Systems connected:** `pyrokinesis_power_activation_system` (Psionics) -> `psionic_fire_bridge_system` (Integration) -> `Fire` component
- **Glue added:** Added `psionic_fire_bridge_system` to spawn `Fire` entity upon receiving `FireEvent` from `pyrokinesis_power_activation_system`. Registered in `Layer1SystemSet::Observation`.
- **Tests:** `tests/integration/psionics_fire_bridge.rs`

### INT-762: The Gold Rush Beacon -> Colony Population & Trade
- **Date:** 2026-04-18
- **Systems connected:** `process_colony_beacon_system` -> `beacon_migrant_arrival_bridge`, `beacon_trade_ship_bridge`, `beacon_pirate_raid_bridge`
- **Glue added:** Added bridge systems to consume `MigrantArrivalEvent`, `TradeShipArrivalEvent`, and `PirateRaidEvent` and generate corresponding state changes (`Pop` spawns, `Merchant` spawns, resource loss and morale drop). Scheduled in `Layer1SystemSet::Economy`.
- **Tests:** `tests/integration/beacon_bridge.rs` (3 tests)

### INT-1132: Deep Crust Resonance -> Chronicle
- **Date:** 2026-05-01
- **Systems connected:** `resonant_ore_exposure_system` (Deep Crust Resonance) -> `deep_crust_resonance_chronicle_bridge` (Integration) -> `AddChronicleEvent`
- **Glue added:** Added `deep_crust_resonance_chronicle_bridge` in `src/layer1/core/integration.rs` to emit an `AddChronicleEvent` (`EventImportance::Major`) upon `ExcavationEvent` with discovery type "ResonantOre". Registered in `SimulationSchedule` (Layer 1 Economy).
- **Tests:** `tests/integration/deep_crust_resonance_bridge.rs`

### INT-915: Orbital Debris Cult -> Morale
- **Date:** 2026-07-26
- **Systems connected:** `OrbitalDebris` (Orbit) & `Station` -> `evaluate_debris_cult_formation_system` & `apply_debris_cult_morale_system` (Integration) -> `Morale` (Social)
- **Glue added:** Added `evaluate_debris_cult_formation_system` to assign `DebrisCultist` and `apply_debris_cult_morale_system` to add Morale based on debris density. Registered in `SimulationSchedule` (Layer 2 Execution).
- **Tests:** `tests/integration/orbital_debris_cult_bridge.rs`

### INT-892: Ghost Ships -> Chronicle
- **Date:** 2026-08-01
- **Systems connected:** `evaluate_transit_system` and `evaluate_lost_ship_return_system` (Ghost Ships) -> `AddChronicleEvent` (Chronicle)
- **Glue added:** Registered `evaluate_transit_system` and `evaluate_lost_ship_return_system` in `Layer3SystemSet`.
- **Tests:** `tests/integration/ghost_ships_bridge.rs`

### INT-1104: Swarm Intelligence -> Drone Behavior
- **Date:** 2026-06-26
- **Systems connected:** `update_drone_clusters` (Swarm Intelligence) -> `update_drone_behavior` (Integration) -> `PopAction` (Utility AI)
- **Glue added:** Modified `update_drone_behavior` in `src/layer1/entities/swarm_intelligence.rs` to assign `ActionType::Repair` for High Intelligence drones and `ActionType::Explore` for Low Intelligence drones (while skipping drones that are already charging).
- **Tests:** `tests/integration/swarm_intelligence_bridge.rs`

### INT-983: Cargo Cult Logistics -> Morale/Efficiency
- **Date:** 2026-06-27
- **Systems connected:** `process_orbital_drops` (Orbital Drop Event) -> `apply_cargo_cult_belief_system` -> `process_ritual_actions_system` -> `Morale` / `EfficiencyDebuff`
- **Glue added:** Registered `apply_cargo_cult_belief_system` and `process_ritual_actions_system` in `Layer1SystemSet::Observation` immediately after `process_orbital_drops` in `src/layer1/systems/observation.rs`.
- **Tests:** `tests/cargo_cult_bridge.rs` and `tests/integration.rs` entry.

### INT-1117: Sonic Suppression -> Acoustic Noise
- **Date:** 2026-12-10
- **Systems connected:** `sonic_suppression_system` (Sonic Suppression) -> `NoiseMap` (Acoustic)
- **Glue added:** Added `sonic_turret_noise_bridge_system` to `src/layer1/core/integration.rs` to insert `NoiseSource` into active `SonicTurret`s, and registered it in `src/layer1/systems/execution.rs`.
- **Tests:** `tests/integration/sonic_suppression_bridge.rs`

### INT-1106: Photophobic Resources -> LightMap
- **Date:** 2026-07-01
- **Systems connected:** `LightMap` -> `update_photophobic_light_level_system` -> `photophobic_degradation_system`
- **Glue added:** `update_photophobic_light_level_system` in `src/layer1/economy/photophobic.rs`
- **Schedule:** Chained in Economy
- **Tests:** `tests/integration/photophobic_light_bridge.rs`

### INT-1145: Inflationary Spiral -> Economy
- **Date:** 2026-04-24
- **Systems connected:** `MarketCrashEvent` -> `trigger_market_crash`, and `BarterRequest` -> `process_barter_trade` (Inflationary Spiral) -> `Layer1SystemSet::Economy`
- **Glue added:** Registered `trigger_market_crash` and `process_barter_trade` in `src/layer1/systems/economy.rs`.
- **Tests:** `tests/integration/inflation_spiral_bridge.rs`

### INT-1107: Astrological Beliefs -> Syzygy Cycle
- **Date:** 2026-07-02
- **Systems connected:** `SyzygyCycle` -> `astrological_beliefs_bridge_system` -> `AstrologicalBelief`
- **Glue added:** `astrological_beliefs_bridge_system` in `src/layer2/integration.rs`
- **Schedule:** Runs after `update_syzygy_cycle_system`
- **Tests:** `tests/integration/astrological_beliefs_bridge.rs`

### INT-1234: Faction Strikes -> Protest Mobs
- **Date:** 2026-06-25
- **Systems connected:** `update_faction_strikes_system` (Social/Factions) -> `faction_strike_mob_bridge_system` (Integration) -> `form_mob_system` (Protest Crowds)
- **Glue added:** Added `faction_strike_mob_bridge_system` in `src/layer1/core/integration.rs` to emit `FormMobEvent` and `DisperseMobEvent` when a faction transitions in and out of the `Striking` state. Modified `Mob` to store `FactionId`. Scheduled immediately after `update_faction_strikes_system` in `Layer1SystemSet::Economy`.
- **Tests:** `tests/integration/protest_crowds_integration.rs`

### INT-1233: Public Grievances -> Social/Work
- **Date:** 2026-04-29
- **Systems connected:** `Public Grievances` (Ostracized marker) -> `Social/Work Systems` (arrival, work_execution, proximity)
- **Glue added:**
    - Updated `collect_workers_by_target` in `src/layer1/execution/general_work.rs` to query for `Ostracized` component and pass it into `WorkerData`, applying a 0.2 work speed multiplier if ostracized.
    - Updated `arrival_handler_system` in `src/layer1/execution/arrival.rs` to query for `Ostracized` component and pass it to `process_arrival`, which passes it to `handle_socialize`.
    - Updated `handle_socialize` in `src/layer1/social/mod.rs` to early-return if the pop is ostracized, preventing them from joining a tavern.
    - Updated `proximity_social_system` in `src/layer1/social/mod.rs` to ignore ostracized pops (they don't gain proximity buffs and others don't gain proximity buffs from them).
- **Tests:** Added `tests/integration/public_grievances_bridge.rs` testing socialize, proximity, and work penalties.

### INT-648: Founder Effect -> Pop Generation
- **Date:** 2026-06-25
- **Systems connected:** `ColonyCulture` (Founder Effect) -> `founder_effect_bridge_system` -> `Traits` (Pop Generation)
- **Glue added:** `founder_effect_bridge_system` in `src/layer2/integration.rs` reads `PopBorn` events, retrieves the `dominant_trait` from `ColonyCulture`, and adds it to the new `Pop`'s `Traits`.
- **Tests:** `tests/integration/founder_effect_bridge.rs`

### INT-649: Pop Relationships -> Day Night Cycle
- **Date:** 2026-10-30
- **Systems connected:** `DayNightCycle` -> `trigger_shift_end_system` -> `update_workplace_relationships_system`
- **Glue added:** Added `trigger_shift_end_system` in `src/layer1/core/integration.rs` to emit `ShiftEndEvent` when TimeOfDay transitions from Day to Dusk. Registered in `Layer1SystemSet::Execution`.
- **Tests:** `tests/integration/pop_relationships_bridge.rs`

### INT-1236: The Kessler Gambit -> Simulation Schedule
- **Date:** 2026-06-25
- **Systems connected:** `trigger_kessler_gambit_system` -> `OrbitalDebris`
- **Glue added:** Scheduled `trigger_kessler_gambit_system` in Layer 2 systems block. Added `TriggerKesslerGambitEvent` to App.
- **Tests:** `tests/integration/kessler_gambit_integration.rs`

### INT-1235: The Gastronomers -> Chronicle
- **Date:** 2026-06-25
- **Systems connected:** `CulinarySingularityEvent` -> `gastronomer_chronicle_bridge` -> `AddChronicleEvent`
- **Glue added:** Added `gastronomer_chronicle_bridge` in `src/layer1/core/integration.rs`
- **Tests:** `tests/integration/gastronomer_chronicle.rs`

### INT-1247: The Visitor -> Chronicle
- **Date:** 2026-10-31
- **Systems connected:** `TheVisitor` -> `visitor_chronicle_bridge` -> `AddChronicleEvent`
- **Glue added:** Added `visitor_chronicle_bridge` in `src/layer1/core/integration.rs`.
- **Tests:** `tests/integration/visitor_chronicle_bridge.rs`

### INT-1248: Red Tape Defense -> Chronicle
- **Date:** 2026-06-25
- **Systems connected:** `invoke_red_tape` (Red Tape Defense) -> `red_tape_chronicle_bridge` (Integration) -> `AddChronicleEvent`
- **Glue added:** Added `red_tape_chronicle_bridge` in `src/layer3/integration.rs`.
- **Tests:** `tests/integration/red_tape_bridge.rs`

### INT-643: Living Architecture -> Chronicle
- **Date:** 2026-10-31
- **Systems connected:** `PopConsumedEvent` -> `living_architecture_chronicle_bridge` -> `AddChronicleEvent`
- **Glue added:** Added `living_architecture_chronicle_bridge` in `src/layer1/core/integration.rs`.
- **Tests:** `tests/integration/living_architecture_chronicle.rs`

### INT-661: Secret Societies -> Predictive Policing -> Chronicle
- **Date:** 2026-06-25
- **Systems connected:** `SecretSocietyMember` -> `society_suspicion_bridge_system` -> `Suspect` -> `execute_pre_crime_arrest` -> `Inmate` -> `secret_society_discovery_bridge_system` -> `AddChronicleEvent`
- **Glue added:**
    - `society_suspicion_bridge_system` marks `SecretSocietyMember` Pops as `Suspect`s (Conspiracy) when a `PredictiveModel` is active.
    - `secret_society_discovery_bridge_system` disbands the `SecretSociety` and emits an `AddChronicleEvent` when a member is arrested (`Inmate`).
    - Registered in `src/layer1/systems/observation.rs`.
- **Tests:** `test_society_discovery` and `test_society_suspicion` in `tests/integration/secret_societies.rs`.

### INT-637: Echoes of the Predecessors -> Fleets & Seasons
- **Date:** 2026-05-04
- **Systems connected:** `PredecessorOrbitalShield` -> `FleetOrder` blocking; `PredecessorWeatherArray` -> `SeasonState` lock
- **Glue added:** Added `predecessor_orbital_shield_bridge_system` to `src/layer2/integration.rs` to block Layer 2 fleet movement to the colony. Added `predecessor_weather_array_bridge_system` to `src/layer1/core/integration.rs` to lock the season to Spring.
- **Tests:** `tests/integration/predecessors_echoes.rs`

### INT-626: Cartographer's Curse -> Chronicle
- **Date:** 2026-06-25
- **Systems connected:** `process_telemetry_sale` (Cartographer's Curse) -> `cartographers_curse_chronicle_bridge` -> `AddChronicleEvent`
- **Glue added:** Added `cartographers_curse_chronicle_bridge` in `src/layer2/integration.rs` to emit an `AddChronicleEvent` when `SellTelemetryEvent` occurs.
- **Tests:** `tests/integration/cartographers_curse_bridge.rs`

### INT-1084: Ideological Contraband
- **Systems connected:** `execute_trade_routes_system` -> `ideological_contraband_route_bridge` -> `apply_cultural_contraband_system`
- **Glue added:** `ideological_contraband_route_bridge` in `src/layer2/integration.rs`

### INT-659: Orbital Bombardment -> Chronicle
- **Date:** 2026-06-25
- **Systems connected:** `BombardmentEvent` -> `orbital_bombardment_chronicle_bridge` -> `AddChronicleEvent`
- **Glue added:** `orbital_bombardment_chronicle_bridge` in `src/layer2/integration.rs`
- **Tests:** `tests/integration/orbital_bombardment_bridge.rs`

### INT-660: Orbital Mirrors -> Chronicle
- **Date:** 2026-06-25
- **Systems connected:** `Added<OrbitalMirror>` -> `orbital_mirror_chronicle_bridge` -> `AddChronicleEvent`
- **Glue added:** `orbital_mirror_chronicle_bridge` in `src/layer2/integration.rs`
- **Tests:** `tests/integration/orbital_mirror_bridge.rs`

### INT-244: Biometric Drift -> Access Control
- **Date:** 2026-05-05
- **Systems connected:** `BiometricProfile` (Biometric Drift) -> `check_access` / `is_walkable` (Access Control / Pathfinding)
- **Glue added:** Added `check_security_clearance` checks inside `check_access` and `is_walkable`.
- **Tests:** `tests/integration/biometric_access_bridge.rs`

### INT-687: Ghost Shift -> Chronicle
- **Date:** 2026-06-25
- **Systems connected:** `GhostShiftStartedEvent` -> `ghost_shift_chronicle_bridge` -> `AddChronicleEvent`
- **Glue added:** `ghost_shift_chronicle_bridge` in `src/layer1/core/integration.rs`
- **Tests:** `tests/integration/ghost_shift_chronicle.rs`

### INT-668: Impact Warning -> Chronicle
- **Date:** 2026-06-25
- **Systems connected:** `ImpactWarningEvent` -> `impact_warning_chronicle_bridge` -> `AddChronicleEvent`
- **Glue added:** `impact_warning_chronicle_bridge` in `src/layer1/core/integration.rs`
- **Tests:** `tests/integration/impact_warning_chronicle.rs`

### INT-764: Diplomatic Incident -> Chronicle
- **Date:** 2026-06-25
- **Systems connected:** `DiplomaticIncidentEvent` -> `diplomatic_incident_chronicle_bridge` -> `AddChronicleEvent`
- **Glue added:** `diplomatic_incident_chronicle_bridge` in `src/layer1/core/integration.rs`
- **Tests:** `tests/integration/diplomatic_incident_chronicle.rs`

### INT-631: Conveyor Logistics -> Pathfinding
- **Date:** 2026-10-31
- **Systems connected:** `ConveyorBelt` -> `is_walkable` (Pathfinding)
- **Glue added:** Updated `is_walkable` logic in `src/layer1/pathfinding.rs` to query `crate::layer1::logistics::conveyor::ConveyorBelt` component and ensure standard conveyors block movement while underground variants don't.
- **Tests:** `tests/integration/conveyor_pathfinding_bridge.rs` (2 tests)

### INT-634: Petrification Sickness -> Chronicle
- **Date:** 2026-06-25
- **Systems connected:** `PopPetrifiedEvent` -> `petrification_chronicle_bridge` -> `AddChronicleEvent`
- **Glue added:** Added `petrification_chronicle_bridge` in `src/layer1/core/integration.rs` to emit an `AddChronicleEvent` when `PopPetrifiedEvent` occurs.
- **Tests:** `tests/integration/petrification_chronicle_bridge.rs`

### INT-471: The Organ Trade -> Market/Diplomacy
- **Date:** 2026-05-08
- **Systems connected:** `OrganHarvestedEvent` (Penal Law) -> `organ_trade_diplomacy_bridge` (Integration) -> `TraitChangedEvent` (Diplomacy Reflection)
- **Glue added:** Added `organ_trade_diplomacy_bridge` in `src/layer3/integration.rs` to mark a civilization as `is_barbarian` when they harvest organs. Registered in `src/simulation.rs`.
- **Tests:** `tests/integration/organ_trade_bridge.rs`

### INT-623: Nostalgia Plague -> Rumor Network
- **Date:** 2026-02-01
- **Systems connected:** `Nostalgia` -> `nostalgia_rumor_generation_bridge` -> `process_rumor_reaction` -> `RumorSpreadEvent`
- **Glue added:** Added `nostalgia_rumor_generation_bridge` in `src/layer1/core/integration.rs` to generate "Past Glory" rumor. Emitted `RumorSpreadEvent` in `process_rumor_reaction` when "Past Glory" is heard. Registered systems in `src/layer1/systems/observation.rs`.
- **Tests:** `tests/integration/nostalgia_plague_bridge.rs`

### INT-1085: Resource Curse -> Pirate Raids
- **Date:** 2026-06-25
- **Systems connected:** `PirateThreatLevel` (Layer 3) -> `resource_curse_raid_bridge` -> `PirateRaidEvent` (Layer 1)
- **Glue added:** Added `resource_curse_raid_bridge` in `src/layer3/pirates.rs` to conditionally emit `PirateRaidEvent` and lower threat level when `PirateThreatLevel` gets too high (>= 10.0). Registered in `src/simulation.rs`.
- **Tests:** `tests/integration/resource_curse_bridge.rs`

### INT-1255: Sentient Trade Routes -> Chronicle
- **Date:** 2026-10-31
- **Systems connected:** `SentientTollDemandEvent` (Layer 2) -> `sentient_route_chronicle_bridge` -> `AddChronicleEvent` (Layer 1)
- **Glue added:** Added `sentient_route_chronicle_bridge` in `src/layer2/integration.rs` to conditionally emit `AddChronicleEvent` and reset `RouteComplexity` when a toll is demanded. Registered in `src/simulation.rs`.
- **Tests:** `tests/integration/sentient_route_bridge.rs`

### INT-453-2: Nanite Fabrication Breach -> Grey Goo Spawning
- **Date:** 2026-10-31
- **Systems connected:** `nanite_fabrication_system` -> `nanite_breach_goo_bridge` -> `grey_goo_replication_system`
- **Glue added:** `nanite_breach_goo_bridge` in `src/layer1/core/integration.rs`
- **Schedule:** Chained in Update, fabrication before bridge before replication
- **Tests:** `tests/integration/nanite_breach_goo.rs`

### INT-1089: Nanite Fabrication Breach -> Nanite Storms
- **Date:** 2026-05-10
- **Systems connected:** `nanite_fabrication_system` -> `nanite_breach_storm_bridge` -> `apply_nanite_storm_effects`
- **Glue added:** `nanite_breach_storm_bridge` in `src/layer1/core/integration.rs`
- **Schedule:** Chained in Update, fabrication before storm generation before effects
- **Tests:** `tests/integration/nanite_breach_storm.rs`

### INT-939: Language Drift -> Trade Routes (Translation Tax)
- **Date:** 2026-06-28
- **Systems connected:** `execute_trade_routes_system` -> `language_drift_trade_bridge` -> `LinguisticNetwork`
- **Glue added:** Added `TradeRouteExecutedEvent` in `src/layer2/trade/routes.rs`. Added `language_drift_trade_bridge` in `src/layer3/integration.rs` to read the event and deduct the "Translation Tax" from the destination colony based on the linguistic drift. Registered in `src/simulation.rs`.
- **Tests:** `tests/integration/language_drift_trade_bridge.rs`

### INT-1088: Megafauna Death -> Apex Meat Diet
- **Date:** 2026-05-10
- **Systems connected:** `handle_fauna_death_system` -> `apex_meat_harvest_bridge_system` -> `apex_meat_distribution_system` -> `process_apex_meat_consumption`
- **Glue added:** `apex_meat_harvest_bridge_system` and `apex_meat_distribution_system` in `src/layer1/core/integration.rs`, `ApexMeatStores` resource
- **Schedule:** Harvest on Fauna death. Distribution in Update before `consume_food_system`
- **Tests:** `tests/integration/apex_diet_integration.rs`

### INT-893: Psychic Stains -> Pathfinding
- **Date:** 2026-10-31
- **Systems connected:** `PsychicStain` component -> `find_path_internal`
- **Glue added:** Pathfinding checks the `world` for `PsychicStain` entities and applies a proportional penalty to the path cost, making pops avoid highly traumatized tiles.
- **Tests:** `tests/integration/psychic_stains_pathfinding.rs`

### INT-1077: Splicer's Dilemma -> Social/Traits
- **Date:** 2026-06-25
- **Systems connected:** `apply_mutation_system`, `evaluate_social_friction_system`
- **Glue added:** Registered in `src/layer1/systems/economy.rs` and `src/layer1/systems/observation.rs`.
- **Tests:** `tests/integration/splicers_dilemma_integration.rs`

### INT-1257: Generational Skill Atrophy -> Economy
- **Date:** 2026-06-25
- **Systems connected:** `apply_skill_atrophy_system`, `AutomationLevel`
- **Glue added:** Initialized `AutomationLevel` resource in `src/simulation.rs` and registered `apply_skill_atrophy_system` in `src/layer1/systems/economy.rs`.
- **Tests:** `tests/integration/generational_atrophy_integration.rs`

### INT-1074: System Quarantine -> Trade/Warlords
- **Date:** 2026-06-25
- **Systems connected:** `apply_quarantine_effects`, `handle_quarantine_decay`
- **Glue added:** Registered Layer 2 events in `src/simulation.rs`.
- **Tests:** `tests/integration/system_quarantine_integration.rs`

### INT-1076: Fading Homeworld -> Economy & Diplomacy
- **Date:** 2026-07-04
- **Systems connected:** `generate_core_world_demand_system` -> `handle_core_world_demands_system` -> `ColonyResources` and `DiplomaticRelations`
- **Glue added:** Added `generate_core_world_demand_system` to emit `CoreWorldDemandEvent`s. Added `PlayerDemandResponse` event and modified `handle_core_world_demands_system` to fulfill or refuse demands based on player responses. Initialized `CoreWorldDemandEvent` and `PlayerDemandResponse` in `simulation.rs`. Registered all 3 fading homeworld systems in `simulation.rs` and added `CoreWorldDemandEvent` and `PlayerDemandResponse` to `cleanup.rs`.
- **Tests:** `tests/integration/fading_homeworld.rs`

### INT-887: Sunk-Cost Monument -> Economy
- **Date:** 2026-06-25
- **Systems connected:** `SunkCostUpkeep` -> `ColonyResources` (Metal) -> `CancelConstructionEvent`
- **Glue added:** `sunk_cost_resource_drain_system` in `src/layer1/core/integration.rs`
- **Schedule:** Chained in Update in `src/layer1/systems/economy.rs`
- **Tests:** `tests/integration/sunk_cost_integration.rs`

### INT-249: Holographic Facades -> Morale
- **Date:** 2026-06-25
- **Systems connected:** `update_holograms_system` -> `HologramFailureEvent` -> `apply_disillusionment_system` -> `Morale`
- **Glue added:** Added `HologramFailureEvent` to `update_event_buffer` in `src/layer1/systems/cleanup.rs`.
- **Tests:** `tests/integration/holographic_facades.rs`

### INT-250: Surgical Addiction -> Chronicle
- **Date:** 2026-05-18
- **Systems connected:** `check_self_surgery_system` (Surgical Addiction) -> `AddChronicleEvent`
- **Glue added:** Modified `check_self_surgery_system` in `src/layer1/biology/addiction.rs` to emit an `AddChronicleEvent` when a pop performs self-surgery.
- **Tests:** `tests/integration/surgical_addiction_bridge.rs`

### INT-1095: Intellectual Property Wars -> Diplomacy
- **Systems connected:** `detect_ip_piracy_system` -> `ip_piracy_diplomacy_bridge` -> `apply_diplomatic_reactions`
- **Glue added:** `ip_piracy_diplomacy_bridge` in `src/layer3/integration.rs`
- **Tests:** `tests/integration/intellectual_property_wars_bridge.rs`

### INT-694: Hedonic Treadmill -> Pop Consumption
- **Date:** 2026-06-25
- **Systems connected:** `consume_food_system` -> `hedonic_treadmill_consumption_bridge` -> `process_consumption_quality`
- **Glue added:** Added `hedonic_treadmill_consumption_bridge` in `src/layer1/social/hedonic_treadmill_integration.rs` to read `JustConsumed` component added by `consume_food_system` and emit `ConsumeItemEvent` with calculated item quality for `process_consumption_quality`. Registered the new system and events in `src/layer1/systems/economy.rs` and `src/simulation.rs`.
- **Tests:** `tests/integration/hedonic_treadmill_bridge.rs`

### INT-1258: Cassandra Protocol -> Chronicle
- **Date:** 2026-08-01
- **Systems connected:** `activate_cassandra_protocol` -> `cassandra_protocol_chronicle_bridge`
- **Glue added:** `cassandra_protocol_chronicle_bridge` in `src/layer1/core/integration.rs`
- **Tests:** `tests/integration/cassandra_protocol_bridge.rs`

### INT-266: Emotional Contagion -> Pop Morale Status
- **Date:** 2026-05-16
- **Systems connected:** `trigger_emotional_contagion_system`
- **Glue added:** Added `trigger_emotional_contagion_system` in `src/layer1/social/emotional_contagion.rs` to add/remove the `EmotionalContagion` component based on extreme `Morale`.
- **Tests:** `tests/integration/emotional_contagion_trigger.rs`

### INT-1071: Bureaucracy of Truth -> Colony State
- **Date:** 2026-10-31
- **Systems connected:** `ColonyResources`, `Unrest` (Layer 1) -> `ColonyState` (Layer 3)
- **Glue added:** `bureaucracy_of_truth_integration_system` in `src/layer3/integration.rs` synchronizes Layer 1 state into Layer 3 state for the `generate_colony_reports_system`.
- **Schedule:** Chained in Update, syncs before reporting
- **Tests:** `tests/integration/bureaucracy_of_truth_integration.rs`

### INT-888: The Symbiotic Parasite -> Needs/Health
- **Date:** 2026-03-24
- **Systems connected:** `apply_parasite_buffs_system` -> `Needs` / `apply_parasite_health_drain_system` -> `Health`
- **Glue added:** Registered systems from `src/layer1/biology/symbiotic_parasite.rs` to `src/layer1/systems/execution.rs` schedule.
- **Tests:** `tests/integration/symbiotic_parasite_bridge.rs`

### INT-1068: Crustal Tides -> Chronicle
- **Date:** 2026-10-31
- **Systems connected:** `process_crustal_tides` -> `crustal_tide_chronicle_bridge` -> `AddChronicleEvent`
- **Glue added:** Added `crustal_tide_chronicle_bridge` in `src/layer1/core/integration.rs` to monitor `TidalForce` and emit `AddChronicleEvent` upon transitions to high or low tide thresholds.
- **Tests:** `tests/integration/crustal_tides_bridge.rs`

### INT-549: Sleep Debt -> Repo Man Spawning
- **Date:** 2026-05-18
- **Systems connected:** `check_critical_sleep_debt_system` -> `repo_man_arrival_bridge` -> `process_repo_men_action_system`
- **Glue added:** Added `repo_man_arrival_bridge` in `src/layer1/core/integration.rs` to read `RepoManArrivalEvent`, spawn a `RepoMan` entity, and log it to the Chronicle.
- **Tests:** `tests/integration/sleep_debt_repo_bridge.rs`

### INT-585: Cartographic Delusion -> Chronicle
- **Date:** 2026-10-31
- **Systems connected:** `AnomalyDiscoveredEvent` -> `anomaly_discovered_chronicle_bridge` -> `AddChronicleEvent`
- **Glue added:** Added `anomaly_discovered_chronicle_bridge` in `src/layer3/integration.rs` to monitor `AnomalyDiscoveredEvent` and emit `AddChronicleEvent`. Registered in `src/simulation.rs`.
- **Tests:** `tests/integration/cartographic_delusion_bridge.rs`

### INT-1061: Pirate Republics -> Diplomacy
- **Date:** 2026-06-25
- **Systems connected:** `PirateHaven` upgrading to `PirateRepublic` -> `Civilization`, `DiplomaticTraits`, `DiplomaticRelations`
- **Glue added:** `pirate_republic_diplomacy_bridge` in `src/layer3/integration.rs`
- **Tests:** `tests/integration/pirate_republic_diplomacy_bridge.rs`

### INT-1064: Digital Detritus -> Chronicle
- **Date:** 2026-05-31
- **Systems connected:** `process_data_mining_system` -> `record_virus_event_chronicle_system`
- **Glue added:** `record_virus_event_chronicle_system` in `src/layer3/digital_detritus.rs`
- **Tests:** `tests/integration/digital_detritus_chronicle_bridge.rs`

### INT-1059: System Sovereignty -> Simulation
- **Date:** 2026-10-31
- **Systems connected:** `process_sovereignty_declaration` -> `SimulationSchedule`
- **Glue added:** Registered `process_sovereignty_declaration`, `DeclarationOfIndependenceEvent`, and `WarDeclarationEvent` in `src/simulation.rs`.

### INT-1066: Zero-G Flora -> Simulation
- **Date:** 2026-10-31
- **Systems connected:** `handle_depressurization` -> `SimulationSchedule`
- **Glue added:** Registered `handle_depressurization` and `DepressurizationEvent` in `src/simulation.rs`.

### INT-1057: The Living Score -> Simulation
- **Date:** 2026-05-22
- **Systems connected:** `Morale` -> `update_renown_from_morale_system` -> `ColonyRenown`
- **Glue added:** Added `update_renown_from_morale_system` in `src/layer1/core/integration.rs` to compute average morale across all pops and translate it into a 0.0 - 100.0 scale for `ColonyRenown`. Registered in `src/layer1/systems/economy.rs` before `update_living_score_aesthetics`.
- **Tests:** `tests/integration/living_score_renown.rs`

### INT-1069: Existential Audit -> Chronicle
- **Date:** 2026-05-23
- **Systems connected:** `existential_audit_system` -> `existential_audit_chronicle_bridge`
- **Glue added:** `ExistentialAuditCompletedEvent` in `src/layer1/economy/existential_audit.rs` and `existential_audit_chronicle_bridge` in `src/layer1/core/integration.rs`
- **Schedule:** Chained in Update within `Layer1SystemSet::Economy`
- **Tests:** `tests/integration/existential_audit_bridge.rs`

### INT-480: Memory Forgery -> Chronicle
- **Date:** 2026-05-25
- **Systems connected:** `truth_outbreak_system` -> `truth_outbreak_chronicle_bridge` -> `AddChronicleEvent`
- **Glue added:** `truth_outbreak_chronicle_bridge` in `src/layer1/core/integration.rs`
- **Tests:** `tests/integration/memory_forgery.rs`

### INT-1067: Void Sirens -> Chronicle
- **Date:** 2026-05-25
- **Systems connected:** `apply_siren_obsession` -> `siren_signal_chronicle_bridge` -> `AddChronicleEvent`
- **Glue added:** `siren_signal_chronicle_bridge` in `src/layer1/core/integration.rs`
- **Tests:** `tests/integration/void_sirens_chronicle.rs`

### INT-1102: The Cult of the First Ship -> Chronicle
- **Date:** 2026-05-25
- **Systems connected:** `FirstShip` (removal) -> `first_ship_destruction_chronicle_bridge` -> `AddChronicleEvent`
- **Glue added:** `first_ship_destruction_chronicle_bridge` in `src/layer1/core/integration.rs`
- **Schedule:** Chained in Observation schedule after `first_ship_destruction_system`
- **Tests:** `tests/integration/cult_of_first_ship_bridge.rs`

### INT-481: Subterranean Mycelial Network -> Disease & Chronicle
- **Date:** 2026-10-31
- **Systems connected:** `process_mutations` (Crop) -> `ContaminationEvent` -> `AddChronicleEvent`
- **Glue added:** `crop_mutation_mycelial_bridge` and `mycelial_chronicle_bridge` in `src/layer1/core/integration.rs`
- **Schedule:** Registered in Observation schedule
- **Tests:** `tests/integration/mycelial_network_bridge.rs`

### INT-495: Temporal Echoes -> Maintenance Debt
- **Date:** 2026-10-31
- **Systems connected:** `BuildingAge` -> `temporal_echo_maintenance_bridge_system` -> `Structure`
- **Glue added:** Added `temporal_echo_maintenance_bridge_system` in `src/layer1/core/integration.rs` to convert accumulated `BuildingAge.ticks` into `Structure` damage according to base entropy decay. Registered in `Layer1SystemSet::Observation`.
- **Tests:** `tests/integration/temporal_echoes_maintenance.rs`

### INT-680: The Memory Blackout -> Pop Memories, Relationships, and Skills
- **Date:** 2026-05-25
- **Systems connected:** `MemoryBlackoutEvent` -> `process_memory_blackout` -> `Memories`, `Relationships`, `Skills`, `AddChronicleEvent`
- **Glue added:** `process_memory_blackout` in `src/layer1/psychology/memory_blackout.rs` and `memory_blackout_chronicle_bridge` in `src/layer1/core/integration.rs` to record the event in the Chronicle.
- **Schedule:** Chained in Update within `Layer1SystemSet::Economy`
- **Tests:** `tests/integration/memory_blackout_bridge.rs` (4 tests)

### INT-1126: Orbital Drydocks -> Fleet Spawning
- **Date:** 2026-10-31
- **Systems connected:** `process_drydock_construction_system` -> `orbital_drydock_fleet_bridge_system` -> `Fleet`, `InOrbit`
- **Glue added:** `ShipConstructionCompletedEvent` emitted by `process_drydock_construction_system` and consumed by `orbital_drydock_fleet_bridge_system` to spawn the `Fleet` entity and an `AddChronicleEvent`.
- **Tests:** `tests/integration/orbital_drydocks_bridge.rs`

### INT-1135: Gravitational Doldrums -> Fleet Movement
- **Date:** 2026-10-31
- **Systems connected:** `doldrums_effects_system` -> `fleet_movement_system`
- **Glue added:** Modified `fleet_movement_system` in `src/layer2/fleet.rs` to query `MovementSpeed` and apply the current speed multiplier.
- **Schedule:** Added `doldrums_effects_system` to run before `fleet_movement_system` in `src/simulation.rs`.
- **Tests:** `tests/integration/gravitational_doldrums_bridge.rs` (1 test)

### INT-279: Signal Latency -> Fleet Movement
- **Date:** 2026-05-28
- **Systems connected:** `execute_delayed_orders_system` -> `signal_latency_fleet_bridge` -> `fleet_order_system`
- **Glue added:** Added `signal_latency_fleet_bridge` in `src/layer2/integration.rs` to convert `ExecuteOrderEvent` into `FleetOrder` components on fleets.
- **Tests:** `tests/signal_latency_bridge.rs`

### INT-310: Celestial Library -> ColonyResources & Chronicle
- **Date:** 2026-05-27
- **Systems connected:** `LibraryDonationEvent` -> `celestial_library_chronicle_bridge` -> `ColonyResources`, `AddChronicleEvent`
- **Glue added:** Added `celestial_library_chronicle_bridge` in `src/layer2/integration.rs` to process library donations, deduct knowledge from `ColonyResources`, and emit `AddChronicleEvent`s.
- **Tests:** `tests/celestial_library_bridge.rs`

### INT-679: The Flesh Tax -> Simulation
- **Date:** 2026-10-31
- **Systems connected:** `process_flesh_tax_payment`, `process_flesh_tax_failure` -> `SimulationSchedule`
- **Glue added:** Registered `process_flesh_tax_payment` and `process_flesh_tax_failure`, along with `FleshTaxPaymentEvent` and `FleshTaxFailedEvent` in `src/simulation.rs`.
- **Tests:** `tests/flesh_tax_bridge.rs`

### INT-1113: The Fossilized Fleet -> Combat Defense Bonus
- **Date:** 2026-10-31
- **Systems connected:** `execute_attack` -> `FossilizedShip`
- **Glue added:** Modified `execute_attack` in `src/layer1/combat.rs` to apply `defense_bonus` from `FossilizedShip`.
- **Tests:** `tests/integration/fossilized_fleet_defense_bridge.rs`
