//! Shared simulation tick logic used by all entry points.
//!
//! Uses a Bevy `Schedule` to run all simulation systems. This enables:
//! - Typed system params (Bevy injects `Query`, `Res`, `ResMut` automatically)
//! - Automatic parallel execution of non-conflicting systems (with `multi_threaded`)
//! - `par_iter_mut` for intra-system parallelism on queries

use bevy_ecs::prelude::*;
use bevy_ecs::schedule::{IntoSystemConfigs, Schedule, ScheduleLabel};

use crate::experimental::biography::biography_monitor_system;
use crate::experimental::dreams::dream_system;
#[cfg(feature = "nova")]
use crate::experimental::ghosts::{
    apply_ghost_beauty_system, ghost_light_damage_system, ghost_movement_system,
};
use crate::gpu::evaluate::gpu_evaluate_actions;
use crate::layer1::{
    AddChronicleEvent, AffinityChange, PopDied, advance_season_system, aging_system,
    apply_faction_mood_system, apply_lighting_penalties_system, apply_noise_effects_system,
    apply_quirk_modifiers_system, arrival_handler_system, art_generation_system,
    art_observation_system, check_milestones_system,
    chronicle_event_handler_system, chronicle_rumor_bridge_system, clean_dead_residents_system,
    clean_dead_workers_system, cleanup_previous_assignment_system, clothing_wear_system,
    combat_execution_system, consume_food_system, death_system, decay_needs_system,
    fire_damage_pops_system, fire_damage_system, fire_spread_system, haul_system, healing_system,
    hypothermia_system, memory_decay_system, modify_affinity_system, movement_system,
    natural_death_system, notification_expiration_system, pop_death_chronicle_bridge,
    process_refining_system, process_research_system, process_scan_system,
    process_start_plan_system, produce_food_system, restore_leisure_system,
    restore_rest_in_housing_system, spoilage_system, starvation_damage_system,
    track_plan_outcomes_system, update_action_timer_system, update_lighting_system,
    update_noise_system, update_resource_caps_system, vermin_growth_system, vermin_morale_system,
    work_execution_system,
};
use crate::shared::time::SimulationTime;

/// Helper system to update event buffers (clear old events).
pub fn update_event_buffer<T: Event>(mut events: ResMut<Events<T>>) {
    events.update();
}

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

    // --- Event Cleanup (Start of Frame) ---
    schedule.add_systems((
        update_event_buffer::<AddChronicleEvent>,
        update_event_buffer::<AffinityChange>,
        update_event_buffer::<PopDied>,
    ));

    // --- AI Decision Chain (GPU compute) ---
    schedule.add_systems((
        gpu_evaluate_actions,
        crate::layer1::visitor::visitor_behavior_system,
        update_action_timer_system.after(gpu_evaluate_actions),
    ));

    // --- Execution Chain (must be sequential) ---
    schedule.add_systems((
        crate::layer1::zone::apply_zone_designation_system.after(update_action_timer_system),
        crate::layer1::room_quality::apply_waking_thoughts_system
            .after(crate::layer1::zone::apply_zone_designation_system),
        cleanup_previous_assignment_system
            .after(crate::layer1::room_quality::apply_waking_thoughts_system),
        process_start_plan_system.after(cleanup_previous_assignment_system),
        crate::layer1::fauna::fauna_behavior_system.after(process_start_plan_system),
        crate::layer1::day_night::update_day_night_cycle_system.after(process_start_plan_system),
        crate::layer1::day_night::update_ambient_light_from_cycle_system
            .after(crate::layer1::day_night::update_day_night_cycle_system),
        update_lighting_system
            .after(process_start_plan_system)
            .after(crate::layer1::day_night::update_ambient_light_from_cycle_system),
        apply_lighting_penalties_system.after(update_lighting_system),
        apply_quirk_modifiers_system.after(apply_lighting_penalties_system),
        movement_system
            .after(apply_quirk_modifiers_system)
            .after(crate::layer1::fauna::fauna_behavior_system),
        arrival_handler_system.after(movement_system),
        work_execution_system.after(arrival_handler_system),
        combat_execution_system.after(arrival_handler_system),
        crate::layer1::justice::warden_execution_system.after(combat_execution_system),
        crate::layer1::execution::vandalize_execution_system.after(arrival_handler_system),
        haul_system.after(arrival_handler_system),
        process_scan_system.after(arrival_handler_system),
    ));

    #[cfg(feature = "nova")]
    schedule.add_systems(ghost_movement_system.after(movement_system));

    // --- Economy (after execution, before consumption) ---
    // These systems can run in parallel with each other.
    schedule.add_systems((
        update_resource_caps_system.after(work_execution_system),
        advance_season_system.after(work_execution_system),
        produce_food_system.after(work_execution_system),
        process_refining_system.after(work_execution_system),
        process_research_system.after(work_execution_system),
        restore_rest_in_housing_system
            .after(work_execution_system)
            .after(update_noise_system),
        restore_leisure_system.after(work_execution_system),
        healing_system.after(work_execution_system),
        crate::layer1::beauty::update_beauty_grid_system.after(work_execution_system),
        crate::layer1::beauty::apply_beauty_effects_system
            .after(crate::layer1::beauty::update_beauty_grid_system),
        crate::layer1::trade::merchant_arrival_system.after(work_execution_system),
        crate::layer1::visitor::spawn_visitor_system.after(work_execution_system),
        crate::layer1::energy::power_grid_system.after(work_execution_system),
        art_generation_system.after(work_execution_system),
        crate::layer1::factions::update_faction_membership_system.after(work_execution_system),
        crate::layer1::factions::update_faction_satisfaction_system
            .after(crate::layer1::factions::update_faction_membership_system),
    ));

    #[cfg(feature = "nova")]
    schedule.add_systems((
        apply_ghost_beauty_system
            .after(crate::layer1::beauty::update_beauty_grid_system)
            .before(crate::layer1::beauty::apply_beauty_effects_system),
        ghost_light_damage_system.after(update_lighting_system),
    ));

    // --- Environment (Fire, Acoustic) ---
    schedule.add_systems((
        fire_spread_system.after(work_execution_system),
        fire_damage_pops_system.after(fire_spread_system),
        crate::layer1::structure::fire_damage_structure_system.after(fire_spread_system),
        fire_damage_system
            .after(fire_spread_system)
            .after(fire_damage_pops_system)
            .after(crate::layer1::structure::fire_damage_structure_system),
        update_noise_system.after(work_execution_system),
        apply_noise_effects_system.after(update_noise_system),
        crate::layer1::atmosphere::update_atmosphere_system.after(work_execution_system),
        crate::layer1::atmosphere::pollution_effects_system
            .after(crate::layer1::atmosphere::update_atmosphere_system),
    ));

    // --- Consumption Chain (sequential, depends on economy) ---
    schedule.add_systems((
        consume_food_system
            .after(produce_food_system)
            .after(update_resource_caps_system),
        clothing_wear_system.after(consume_food_system),
        vermin_growth_system.after(consume_food_system),
        vermin_morale_system.after(vermin_growth_system),
        spoilage_system
            .after(consume_food_system)
            .after(vermin_growth_system),
        crate::layer1::visitor::visitor_lifecycle_system.after(consume_food_system),
        decay_needs_system.after(consume_food_system),
        apply_faction_mood_system.after(decay_needs_system),
        crate::layer1::justice::update_inmates_system.after(decay_needs_system),
        crate::layer1::day_night::circadian_rhythm_system.after(consume_food_system),
        aging_system.after(consume_food_system),
        natural_death_system.after(aging_system),
        memory_decay_system.after(decay_needs_system),
        notification_expiration_system.after(decay_needs_system),
        hypothermia_system.after(decay_needs_system),
        starvation_damage_system.after(decay_needs_system),
        death_system
            .after(starvation_damage_system)
            .after(hypothermia_system)
            .after(natural_death_system),
        clean_dead_residents_system.after(death_system),
        clean_dead_workers_system.after(death_system),
    ));

    // --- Observation (after consumption, can run in parallel) ---
    schedule.add_systems((
        track_plan_outcomes_system.after(death_system),
        biography_monitor_system.after(death_system),
        dream_system.after(death_system),
        check_milestones_system.after(death_system),
        crate::layer1::rumor::generate_rumor_system.after(death_system),
        crate::layer1::rumor::exchange_rumors_system.after(death_system),
        crate::layer1::funeral::grief_system.after(death_system),
        crate::layer1::unrest::check_mental_break_system.after(decay_needs_system),
        crate::layer1::justice::check_crime_system
            .after(crate::layer1::unrest::check_mental_break_system),
        crate::layer1::unrest::recover_mental_break_system.after(decay_needs_system),
        // Process new rumors and affinity changes
        modify_affinity_system.after(crate::layer1::rumor::exchange_rumors_system),
        // Process chronicle events
        chronicle_event_handler_system.after(check_milestones_system),
        chronicle_rumor_bridge_system.after(check_milestones_system),
        pop_death_chronicle_bridge.after(death_system),
        art_observation_system.after(death_system),
    ));

    schedule
}

/// Run one simulation tick: all game systems via schedule, then increment tick counter.
pub fn run_simulation_tick(world: &mut World) {
    // Initialize schedule on first call (stored in World's Schedules resource)
    if !world.contains_resource::<Schedules>() {
        world.insert_resource(Schedules::default());
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

        let schedule = build_simulation_schedule();
        world.add_schedule(schedule);
        world.run_schedule(SimulationSchedule);

        // Should not panic — all systems run correctly on a fresh world
    }
}
