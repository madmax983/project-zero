//! Shared simulation tick logic used by all entry points.
//!
//! Uses a Bevy `Schedule` to run all simulation systems. This enables:
//! - Typed system params (Bevy injects `Query`, `Res`, `ResMut` automatically)
//! - Automatic parallel execution of non-conflicting systems (with `multi_threaded`)
//! - `par_iter_mut` for intra-system parallelism on queries

use bevy_ecs::prelude::*;
use bevy_ecs::schedule::{IntoSystemConfigs, Schedule, ScheduleLabel};

use crate::gpu::evaluate::gpu_evaluate_actions;
use crate::layer1::building::{update_building_map_system, BuildingMap};
use crate::layer1::systems::{register_layer1_systems, update_event_buffer, Layer1SystemSet};
use crate::layer1::update_action_timer_system;
use crate::layer2::events::{DetectionEvent, LaunchEvent, ShipDestroyedEvent};
use crate::layer3::silence::{
    check_hostile_spawn_system, update_detection_risk_system, DetectionRisk, HostileSpawnEvent,
};
use crate::shared::time::SimulationTime;

/// Schedule label for the main simulation tick.
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimulationSchedule;

/// Build the simulation schedule with all systems and ordering constraints.
///
/// Systems are organized into ordered groups matching the original sequential execution:
///
/// ```text
/// 1. AI Decision:     gpu_evaluate_actions → update_action_timer
/// 2. Execution:       cleanup_previous → process_start_plan → movement → arrival → work/haul
/// 3. Economy:         update_resource_caps, advance_season, produce_food, process_refining,
///                     process_research, restore_rest, restore_leisure (can run in parallel)
/// 4. Consumption:     consume_food → decay_needs → kill_starving → clean_dead_*
/// 5. Observation:     track_plan_outcomes, biography, dreams, milestones (can run in parallel)
/// 6. Tick increment:  (handled outside schedule)
/// ```
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn build_simulation_schedule() -> Schedule {
    let mut schedule = Schedule::new(SimulationSchedule);

    // --- Register Core Layer 1 Systems ---
    register_layer1_systems(&mut schedule);

    // --- AI Decision Chain (GPU compute) ---
    schedule.add_systems((
        update_building_map_system,
        gpu_evaluate_actions.after(update_building_map_system),
        crate::layer1::visitor::visitor_behavior_system,
        crate::layer1::drone::evaluate_drone_actions_system.after(update_building_map_system),
        update_action_timer_system
            .after(gpu_evaluate_actions)
            .before(Layer1SystemSet::Execution),
    ));

    // --- Layer 3 Integration ---
    schedule.add_systems((
        update_detection_risk_system.after(Layer1SystemSet::Economy),
        check_hostile_spawn_system.after(update_detection_risk_system),
    ));

    // --- Layer 2 Integration ---
    schedule.add_systems((
        // Cleanup Layer 2 events
        update_event_buffer::<LaunchEvent>,
        update_event_buffer::<ShipDestroyedEvent>,
        update_event_buffer::<DetectionEvent>,
        crate::layer2::fleet::fleet_order_system,
        crate::layer2::station::build_station_system
            .after(crate::layer2::fleet::fleet_order_system),
        crate::layer2::fleet::fleet_movement_system.after(crate::layer2::fleet::fleet_order_system),
        crate::layer1::integration::fleet_unload_system
            .after(crate::layer2::fleet::fleet_movement_system),
        crate::layer2::fleet::ensure_fleet_health_system,
        crate::layer2::combat::fleet_combat_system
            .after(crate::layer2::fleet::fleet_movement_system),
        crate::layer2::barnacles::ensure_barnacles_component_system,
        crate::layer2::barnacles::barnacle_accumulation_system,
        // Debris Systems
        crate::layer2::debris::debris_accumulation_system
            .after(crate::layer2::combat::fleet_combat_system),
        crate::layer2::debris::debris_attrition_system
            .after(crate::layer2::debris::debris_accumulation_system),
        crate::layer2::debris::debris_decay_system
            .after(crate::layer2::debris::debris_attrition_system),
        // Thermal Bloom Systems
        crate::layer2::thermal::update_thermal_bloom_system.after(Layer1SystemSet::Economy),
        crate::layer2::thermal::detection_risk_system
            .after(crate::layer2::thermal::update_thermal_bloom_system),
        crate::layer2::integration::thermal_detection_handler_system
            .after(crate::layer2::thermal::detection_risk_system),
        crate::layer2::visibility::update_visibility_system.after(Layer1SystemSet::Economy),
        crate::layer2::visibility::enforce_view_mode_system
            .after(crate::layer2::visibility::update_visibility_system),
    ));

    schedule.add_systems((
        crate::layer2::phantom::check_scrapcode_threshold_system
            .after(crate::layer1::scrapcode::scrapcode_decay_system),
        crate::layer2::phantom::spawn_ghost_fleet_system
            .after(crate::layer2::phantom::check_scrapcode_threshold_system),
    ));

    schedule
}

/// Run one simulation tick: all game systems via schedule, then increment tick counter.
pub fn run_simulation_tick(world: &mut World) {
    // Initialize schedule on first call (stored in World's Schedules resource)
    if !world.contains_resource::<Schedules>() {
        world.insert_resource(Schedules::default());
    }

    if !world.contains_resource::<crate::layer1::social::old_guard::Demographics>() {
        world.init_resource::<crate::layer1::social::old_guard::Demographics>();
    }

    if !world.contains_resource::<BuildingMap>() {
        world.init_resource::<BuildingMap>();
    }

    if !world.contains_resource::<crate::layer1::tech_envy::TechEnvyConfig>() {
        world.init_resource::<crate::layer1::tech_envy::TechEnvyConfig>();
    }

    if !world.contains_resource::<crate::layer1::shadow_market::ShadowMarketManager>() {
        world.init_resource::<crate::layer1::shadow_market::ShadowMarketManager>();
    }

    // Initialize Layer 2 Events
    if !world.contains_resource::<Events<LaunchEvent>>() {
        world.init_resource::<Events<LaunchEvent>>();
    }
    if !world.contains_resource::<Events<ShipDestroyedEvent>>() {
        world.init_resource::<Events<ShipDestroyedEvent>>();
    }
    if !world.contains_resource::<Events<DetectionEvent>>() {
        world.init_resource::<Events<DetectionEvent>>();
    }
    if !world.contains_resource::<Events<HostileSpawnEvent>>() {
        world.init_resource::<Events<HostileSpawnEvent>>();
    }
    if !world.contains_resource::<DetectionRisk>() {
        world.init_resource::<DetectionRisk>();
    }
    if !world.contains_resource::<Events<crate::layer1::unrest::DenounceEvent>>() {
        world.init_resource::<Events<crate::layer1::unrest::DenounceEvent>>();
    }
    if !world.contains_resource::<Events<crate::layer1::volatile::ExplosionEvent>>() {
        world.init_resource::<Events<crate::layer1::volatile::ExplosionEvent>>();
    }
    if !world.contains_resource::<Events<crate::layer1::geology::tectonic::MegaQuakeEvent>>() {
        world.init_resource::<Events<crate::layer1::geology::tectonic::MegaQuakeEvent>>();
    }
    if !world.contains_resource::<Events<crate::layer1::resources::MiningEvent>>() {
        world.init_resource::<Events<crate::layer1::resources::MiningEvent>>();
    }
    if !world.contains_resource::<Events<crate::layer1::spiteful_will::InheritanceEvent>>() {
        world.init_resource::<Events<crate::layer1::spiteful_will::InheritanceEvent>>();
    }
    if !world
        .contains_resource::<Events<crate::layer1::nature::biosphere_empathy::FloraDamagedEvent>>()
    {
        world
            .init_resource::<Events<crate::layer1::nature::biosphere_empathy::FloraDamagedEvent>>();
    }
    if !world.contains_resource::<crate::layer1::nature::biosphere_empathy::GlobalFloraHealth>() {
        world.init_resource::<crate::layer1::nature::biosphere_empathy::GlobalFloraHealth>();
    }
    if !world.contains_resource::<Events<crate::layer1::spiteful_will::OverrideWillEvent>>() {
        world.init_resource::<Events<crate::layer1::spiteful_will::OverrideWillEvent>>();
    }
    if !world.contains_resource::<crate::layer1::geology::tectonic::TectonicStress>() {
        world.init_resource::<crate::layer1::geology::tectonic::TectonicStress>();
    }

    if !world.contains_resource::<crate::layer1::unrest::Unrest>() {
        world.init_resource::<crate::layer1::unrest::Unrest>();
    }
    if !world.contains_resource::<crate::layer1::atmosphere::CorrosiveAtmosphere>() {
        world.init_resource::<crate::layer1::atmosphere::CorrosiveAtmosphere>();
    }

    // Initialize Thermal Bloom Resource
    if !world.contains_resource::<crate::layer2::thermal::ThermalSignature>() {
        world.init_resource::<crate::layer2::thermal::ThermalSignature>();
    }

    // Initialize Detection Risk
    if !world.contains_resource::<DetectionRisk>() {
        world.init_resource::<DetectionRisk>();
    }

    if !world.contains_resource::<crate::layer2::phantom::EmpireAutomationState>() {
        world.init_resource::<crate::layer2::phantom::EmpireAutomationState>();
    }
    if !world.contains_resource::<Events<crate::layer2::phantom::SpawnGhostFleetEvent>>() {
        world.init_resource::<Events<crate::layer2::phantom::SpawnGhostFleetEvent>>();
    }

    // Initialize Infinite Archive Resource (Spec 248)
    if !world.contains_resource::<crate::layer1::tech::infinite_archive::Archive>() {
        world.init_resource::<crate::layer1::tech::infinite_archive::Archive>();
    }

    if !world.contains_resource::<Events<HostileSpawnEvent>>() {
        world.init_resource::<Events<HostileSpawnEvent>>();
    }

    // Add our schedule if not yet added
    {
        let schedules = world.resource::<Schedules>();
        if schedules.get(SimulationSchedule).is_none() {
            let schedule = build_simulation_schedule();
            world.add_schedule(schedule);
        }
    }

    world.run_schedule(SimulationSchedule);
    world.resource_mut::<SimulationTime>().tick += 1;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup::setup_world;
    use crate::shared::state::GameState;

    #[test]
    fn test_run_simulation_tick_increments() {
        let mut world = setup_world();
        *world.resource_mut::<GameState>() = GameState::Running;

        let tick_before = world.resource::<SimulationTime>().tick;
        run_simulation_tick(&mut world);
        let tick_after = world.resource::<SimulationTime>().tick;

        assert_eq!(tick_after, tick_before + 1);
    }

    #[test]
    fn test_run_multiple_ticks() {
        let mut world = setup_world();
        *world.resource_mut::<GameState>() = GameState::Running;

        for _ in 0..10 {
            run_simulation_tick(&mut world);
        }

        assert_eq!(world.resource::<SimulationTime>().tick, 10);
    }

    #[test]
    fn test_schedule_builds_without_panic() {
        let _schedule = build_simulation_schedule();
    }

    #[test]
    fn test_schedule_runs_on_fresh_world() {
        let mut world = setup_world();
        *world.resource_mut::<GameState>() = GameState::Running;

        // Initialize Detection Risk for test
        world.init_resource::<crate::layer3::silence::DetectionRisk>();
        world.init_resource::<Events<crate::layer3::silence::HostileSpawnEvent>>();

        world.init_resource::<Events<crate::layer1::spiteful_will::InheritanceEvent>>();
        world.init_resource::<Events<crate::layer1::spiteful_will::OverrideWillEvent>>();
        world
            .init_resource::<Events<crate::layer1::nature::biosphere_empathy::FloraDamagedEvent>>();
        world.init_resource::<crate::layer2::phantom::EmpireAutomationState>();
        world.init_resource::<Events<crate::layer2::phantom::SpawnGhostFleetEvent>>();

        let schedule = build_simulation_schedule();
        world.add_schedule(schedule);
        world.run_schedule(SimulationSchedule);

        // Should not panic — all systems run correctly on a fresh world
    }
}
