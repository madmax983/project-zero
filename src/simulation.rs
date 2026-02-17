//! Shared simulation tick logic used by all entry points.
//!
//! Uses a Bevy `Schedule` to run all simulation systems. This enables:
//! - Typed system params (Bevy injects `Query`, `Res`, `ResMut` automatically)
//! - Automatic parallel execution of non-conflicting systems (with `multi_threaded`)
//! - `par_iter_mut` for intra-system parallelism on queries

use bevy_ecs::prelude::*;
use bevy_ecs::schedule::{IntoSystemConfigs, Schedule, ScheduleLabel};

use crate::gpu::evaluate::gpu_evaluate_actions;
use crate::layer1::biography::biography_monitor_system;
use crate::layer1::blob::{blob_consumption_system, blob_spread_system};
use crate::layer1::dreams::{cleanup_dream_marker_system, dream_system};
use crate::layer1::heirloom::RetrogradeEngineeringEvent;
use crate::layer1::{
    AddChronicleEvent, AffinityChange, DeathEvent, PopDied, advance_season_system, aging_system,
    ancient_structure_decay_system, apply_cabin_fever_morale_system,
    apply_catharsis_morale_bonus_system, apply_lighting_penalties_system,
    apply_noise_effects_system, apply_palette_fatigue_system, apply_quirk_modifiers_system,
    apply_taboo_stress_system, apply_weather_effects_system, arrival_handler_system,
    art_generation_system, art_observation_system, assign_sleepwalk_target_system,
    biocompatibility_system, check_death_event_system, check_heirloom_status_system,
    check_milestones_system, check_sleepwalking_start_system, check_stress_breakdown_system,
    chronicle_event_handler_system, chronicle_rumor_bridge_system, clean_dead_residents_system,
    clean_dead_workers_system, cleanup_previous_assignment_system, clothing_wear_system,
    combat_execution_system, consume_food_system, death_system, decay_needs_system,
    discovery_system, entropy_system, faction_satisfaction_morale_bridge, fire_damage_pops_system,
    fire_damage_system, fire_pressure_check_system, fire_spread_system, flora_attack_system,
    flora_spread_system, haul_system, healing_system, infiltration_system,
    inspector::{inspector_report_system, observe_inspector_system, spawn_inspector_system},
    inspector_outcome_bridge_system,
    logistics::{conveyor_system, hopper_system},
    malfunction_system, mascot_behavior_system, mascot_buff_system, mascot_death_grief_system,
    mastery_accumulation_system, memory_decay_system, modify_affinity_system, morale_decay_system,
    movement_system, natural_death_system, notification_expiration_system,
    pop_death_chronicle_bridge, pressure_damage_system, process_fuel_consumption_system,
    process_observe_system, process_refining_system, process_research_system, process_scan_system,
    process_start_plan_system, produce_food_system, quirk_generation_system, regrowth_system,
    restore_leisure_system, restore_rest_in_housing_system, retrograde_chronicle_bridge,
    sleepwalk_end_system,
    social::old_guard::{
        apply_founder_benefits_system, apply_mood_modifiers_system,
        check_generational_friction_system, mood_lifecycle_system,
    },
    social_mimicry::{
        clear_just_consumed_system, trend_satisfaction_system, trend_setting_system,
        trend_spread_system,
    },
    social_stratification::{class_friction_system, update_social_class_system},
    spirit_decay_system, spoilage_system, starvation_damage_system, taboo_event_system,
    theft_system, track_plan_outcomes_system, update_action_timer_system,
    update_bioluminescence_system, update_breakdown_system, update_cabin_fever_system,
    update_catharsis_duration_system, update_erosion_system, update_lighting_system,
    update_morale_cache_system, update_noise_system, update_pressure_system,
    update_resource_caps_system, update_screen_shake_system, update_taboo_duration_system,
    update_water_system, update_weather_system, vermin_growth_system, vermin_morale_system,
    waste_pollution_bridge, wild_child_system, work_execution_system,
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
        update_event_buffer::<DeathEvent>,
        update_event_buffer::<PopDied>,
        update_event_buffer::<crate::layer1::structural_integrity::StructureCollapsed>,
        update_event_buffer::<RetrogradeEngineeringEvent>,
        update_event_buffer::<crate::layer1::energy::GridOverloadEvent>,
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
        assign_sleepwalk_target_system
            .after(crate::layer1::room_quality::apply_waking_thoughts_system),
        cleanup_previous_assignment_system.after(assign_sleepwalk_target_system),
        process_start_plan_system.after(cleanup_previous_assignment_system),
        crate::layer1::fauna::fauna_behavior_system.after(process_start_plan_system),
        mascot_behavior_system.after(process_start_plan_system),
        crate::layer1::day_night::update_day_night_cycle_system.after(process_start_plan_system),
        crate::layer1::day_night::update_ambient_light_from_cycle_system
            .after(crate::layer1::day_night::update_day_night_cycle_system),
        update_bioluminescence_system
            .after(crate::layer1::day_night::update_day_night_cycle_system),
        update_lighting_system
            .after(process_start_plan_system)
            .after(crate::layer1::day_night::update_ambient_light_from_cycle_system)
            .after(update_bioluminescence_system),
        apply_lighting_penalties_system.after(update_lighting_system),
        apply_weather_effects_system.after(apply_lighting_penalties_system),
        apply_quirk_modifiers_system
            .after(apply_lighting_penalties_system)
            .after(apply_weather_effects_system),
        crate::layer1::combat::hit_stop_system.after(process_start_plan_system),
    ));

    schedule.add_systems((
        movement_system
            .after(apply_quirk_modifiers_system)
            .after(crate::layer1::fauna::fauna_behavior_system)
            .after(crate::layer1::combat::hit_stop_system),
        arrival_handler_system.after(movement_system),
        work_execution_system.after(arrival_handler_system),
        combat_execution_system.after(arrival_handler_system),
        crate::layer1::turret::turret_fire_system.after(combat_execution_system),
        crate::layer1::justice::warden_execution_system.after(combat_execution_system),
        crate::layer1::execution::vandalize_execution_system.after(arrival_handler_system),
        update_social_class_system.after(arrival_handler_system),
        class_friction_system.after(update_social_class_system),
        haul_system.after(arrival_handler_system),
        conveyor_system.after(haul_system),
        process_scan_system.after(arrival_handler_system),
        update_cabin_fever_system.after(movement_system),
        wild_child_system.after(movement_system),
        update_erosion_system.after(movement_system),
        update_screen_shake_system.after(movement_system),
        crate::layer1::particles::particle_system.after(movement_system),
        infiltration_system.after(movement_system),
        discovery_system.after(process_scan_system),
    ));

    // --- Economy (after execution, before consumption) ---
    // These systems can run in parallel with each other.
    schedule.add_systems((
        update_resource_caps_system.after(work_execution_system),
        advance_season_system.after(work_execution_system),
        update_taboo_duration_system.after(work_execution_system),
        update_water_system.after(work_execution_system),
        update_weather_system.after(work_execution_system),
        produce_food_system.after(work_execution_system),
        hopper_system.after(produce_food_system),
        process_refining_system.after(work_execution_system),
        crate::layer1::tech::update_tech_capacity_system.after(work_execution_system),
        process_research_system.after(work_execution_system),
        process_observe_system.after(work_execution_system),
        regrowth_system.after(work_execution_system),
        flora_spread_system.after(work_execution_system),
        mastery_accumulation_system.after(work_execution_system),
    ));

    schedule.add_systems((
        restore_rest_in_housing_system
            .after(work_execution_system)
            .after(update_noise_system),
        restore_leisure_system.after(work_execution_system),
        apply_mood_modifiers_system.after(restore_leisure_system),
        mascot_buff_system.after(restore_leisure_system),
        apply_catharsis_morale_bonus_system.after(apply_mood_modifiers_system),
        update_morale_cache_system
            .after(apply_catharsis_morale_bonus_system)
            .after(mascot_buff_system),
        morale_decay_system.after(update_morale_cache_system),
    ));

    schedule.add_systems((
        healing_system.after(work_execution_system),
        crate::layer1::beauty::update_beauty_grid_system.after(work_execution_system),
        crate::layer1::beauty::apply_beauty_effects_system
            .after(crate::layer1::beauty::update_beauty_grid_system),
        check_heirloom_status_system.after(work_execution_system),
        crate::layer1::trade::merchant_arrival_system.after(work_execution_system),
        crate::layer1::visitor::spawn_visitor_system.after(work_execution_system),
        spawn_inspector_system.after(crate::layer1::visitor::spawn_visitor_system),
        process_fuel_consumption_system.after(work_execution_system),
        crate::layer1::energy::power_grid_system
            .after(work_execution_system)
            .after(process_fuel_consumption_system),
        crate::layer1::integration::grid_overload_fire_bridge
            .after(crate::layer1::energy::power_grid_system),
        art_generation_system.after(work_execution_system),
        crate::layer1::factions::update_faction_membership_system.after(work_execution_system),
        crate::layer1::factions::update_faction_satisfaction_system
            .after(crate::layer1::factions::update_faction_membership_system),
        crate::layer1::factions::update_faction_demands_system
            .after(crate::layer1::factions::update_faction_satisfaction_system),
        crate::layer1::factions::update_faction_strikes_system
            .after(crate::layer1::factions::update_faction_demands_system),
        apply_founder_benefits_system.after(work_execution_system),
        crate::layer2::visibility::update_visibility_system
            .after(crate::layer1::energy::power_grid_system),
        crate::layer2::visibility::enforce_view_mode_system
            .after(crate::layer2::visibility::update_visibility_system),
    ));

    // --- Environment (Fire, Acoustic) ---
    schedule.add_systems((
        fire_pressure_check_system.after(work_execution_system),
        fire_spread_system.after(fire_pressure_check_system),
        fire_damage_pops_system.after(fire_spread_system),
        crate::layer1::structure::fire_damage_structure_system.after(fire_spread_system),
        fire_damage_system
            .after(fire_spread_system)
            .after(fire_damage_pops_system)
            .after(crate::layer1::structure::fire_damage_structure_system),
        blob_spread_system.after(work_execution_system),
        blob_consumption_system.after(blob_spread_system),
        flora_attack_system.after(work_execution_system),
        ancient_structure_decay_system.after(work_execution_system),
        spirit_decay_system.after(work_execution_system),
        quirk_generation_system.after(spirit_decay_system),
        entropy_system.after(work_execution_system),
    ));

    schedule.add_systems((
        malfunction_system.after(entropy_system),
        update_noise_system.after(work_execution_system),
        apply_noise_effects_system.after(update_noise_system),
        waste_pollution_bridge.after(work_execution_system),
        crate::layer1::atmosphere::update_atmosphere_system
            .after(work_execution_system)
            .after(waste_pollution_bridge),
        update_pressure_system.after(work_execution_system),
        crate::layer1::temperature::update_temperature_system.after(update_pressure_system),
        crate::layer1::suction::suction_system.after(update_pressure_system),
        crate::layer1::integration::vacuum_clears_pollution_system
            .after(crate::layer1::atmosphere::update_atmosphere_system)
            .after(update_pressure_system),
        biocompatibility_system.after(crate::layer1::atmosphere::update_atmosphere_system),
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
        theft_system
            .after(consume_food_system)
            .before(decay_needs_system),
        decay_needs_system.after(consume_food_system),
        apply_palette_fatigue_system.after(consume_food_system),
        apply_cabin_fever_morale_system
            .after(decay_needs_system)
            .before(mood_lifecycle_system),
        faction_satisfaction_morale_bridge
            .after(apply_cabin_fever_morale_system)
            .before(mood_lifecycle_system),
        apply_taboo_stress_system
            .after(decay_needs_system)
            .before(mood_lifecycle_system),
        mood_lifecycle_system.after(decay_needs_system),
        trend_setting_system.after(consume_food_system),
        trend_spread_system.after(trend_setting_system),
        trend_satisfaction_system.after(trend_spread_system),
        clear_just_consumed_system.after(trend_satisfaction_system),
    ));

    schedule.add_systems((
        crate::layer1::justice::update_inmates_system.after(decay_needs_system),
        crate::layer1::day_night::circadian_rhythm_system.after(consume_food_system),
        aging_system.after(consume_food_system),
        natural_death_system.after(aging_system),
        memory_decay_system.after(decay_needs_system),
        notification_expiration_system.after(decay_needs_system),
        crate::layer1::temperature::thermal_damage_system.after(decay_needs_system),
        pressure_damage_system.after(decay_needs_system),
        starvation_damage_system.after(decay_needs_system),
        check_death_event_system
            .after(starvation_damage_system)
            .after(crate::layer1::temperature::thermal_damage_system)
            .after(pressure_damage_system)
            .after(natural_death_system),
        mascot_death_grief_system.after(check_death_event_system),
        death_system.after(mascot_death_grief_system),
        clean_dead_residents_system.after(death_system),
        clean_dead_workers_system.after(death_system),
    ));

    // --- Observation (after consumption, can run in parallel) ---
    schedule.add_systems((
        track_plan_outcomes_system.after(death_system),
        biography_monitor_system.after(death_system),
        dream_system.after(death_system),
        cleanup_dream_marker_system.after(dream_system),
        check_milestones_system.after(death_system),
        crate::layer1::rumor::generate_rumor_system.after(death_system),
        crate::layer1::rumor::exchange_rumors_system.after(death_system),
        crate::layer1::funeral::grief_system.after(death_system),
        crate::layer1::unrest::check_mental_break_system.after(decay_needs_system),
        check_stress_breakdown_system.after(decay_needs_system),
        update_breakdown_system.after(check_stress_breakdown_system),
        update_catharsis_duration_system.after(decay_needs_system),
        check_sleepwalking_start_system.after(decay_needs_system),
        sleepwalk_end_system.after(decay_needs_system),
        crate::layer1::justice::check_crime_system
            .after(crate::layer1::unrest::check_mental_break_system),
        crate::layer1::unrest::recover_mental_break_system.after(decay_needs_system),
        check_generational_friction_system.after(decay_needs_system),
    ));

    schedule.add_systems((
        // Process new rumors and affinity changes
        modify_affinity_system.after(crate::layer1::rumor::exchange_rumors_system),
        crate::layer1::social::proximity_social_system.after(modify_affinity_system),
        // Process chronicle events
        chronicle_event_handler_system.after(check_milestones_system),
        chronicle_rumor_bridge_system.after(check_milestones_system),
        pop_death_chronicle_bridge.after(death_system),
        retrograde_chronicle_bridge.after(work_execution_system),
        art_observation_system.after(death_system),
        observe_inspector_system.after(art_observation_system),
    ));

    schedule.add_systems((
        inspector_report_system.after(chronicle_event_handler_system),
        inspector_outcome_bridge_system.after(inspector_report_system),
        taboo_event_system.after(death_system),
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
