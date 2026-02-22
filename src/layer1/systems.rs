#[allow(clippy::wildcard_imports)]
use super::*;
use bevy_ecs::prelude::*;

use crate::layer1::blob::{blob_consumption_system, blob_spread_system};
use crate::layer1::inspector::{
    inspector_report_system, observe_inspector_system, spawn_inspector_system,
};
use crate::layer1::social::old_guard::{
    apply_founder_benefits_system, apply_mood_modifiers_system, check_generational_friction_system,
    mood_lifecycle_system,
};

/// Helper system to update event buffers (clear old events).
pub fn update_event_buffer<T: Event>(mut events: ResMut<Events<T>>) {
    events.update();
}

/// System sets for Layer 1 simulation phases.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Layer1SystemSet {
    /// Cleanup of previous frame's events.
    EventCleanup,
    /// Execution of plans (movement, work, etc.).
    Execution,
    /// Economic production and resource management.
    Economy,
    /// Environmental effects (fire, weather).
    Environment,
    /// Consumption and decay (needs, spoilage).
    Consumption,
    /// Observation and social systems (history, dreams).
    Observation,
}

/// Registers all Layer 1 systems into the provided schedule.
///
/// This function groups systems into ordered `SystemSet`s to enforce
/// execution order and logical grouping.
#[allow(clippy::too_many_lines)]
pub fn register_layer1_systems(schedule: &mut Schedule) {
    // Configure Sets
    schedule.configure_sets((
        Layer1SystemSet::EventCleanup,
        Layer1SystemSet::Execution.after(Layer1SystemSet::EventCleanup),
        Layer1SystemSet::Economy.after(Layer1SystemSet::Execution),
        Layer1SystemSet::Environment.after(Layer1SystemSet::Economy),
        Layer1SystemSet::Consumption.after(Layer1SystemSet::Economy), // Parallel with Environment
        Layer1SystemSet::Observation.after(Layer1SystemSet::Consumption),
    ));

    // --- Event Cleanup ---
    schedule.add_systems(
        (
            update_event_buffer::<AddChronicleEvent>,
            update_event_buffer::<AffinityChange>,
            update_event_buffer::<PopDied>,
            update_event_buffer::<crate::layer1::structural_integrity::StructureCollapsed>,
            update_event_buffer::<crate::layer1::heirloom::RetrogradeEngineeringEvent>,
            update_event_buffer::<crate::layer1::energy::GridOverloadEvent>,
            update_event_buffer::<crate::layer1::hazards::AmputationEvent>,
            update_event_buffer::<crate::layer1::geology::GeologicalEvent>,
            update_event_buffer::<crate::layer1::society::InvestigationEvent>,
            update_event_buffer::<crate::layer1::society::SuppressSocietyEvent>,
            update_event_buffer::<crate::layer1::medical::PatientTreated>,
            update_event_buffer::<crate::layer1::eureka::EurekaEvent>,
        )
            .in_set(Layer1SystemSet::EventCleanup),
    );

    // --- Execution Chain ---
    // Note: AI Decision happens before this (handled in simulation.rs for now)
    schedule.add_systems(
        (
            crate::layer1::zone::apply_zone_designation_system,
            crate::layer1::room_quality::apply_waking_thoughts_system
                .after(crate::layer1::zone::apply_zone_designation_system),
            assign_sleepwalk_target_system
                .after(crate::layer1::room_quality::apply_waking_thoughts_system),
            cleanup_previous_assignment_system.after(assign_sleepwalk_target_system),
            process_start_plan_system.after(cleanup_previous_assignment_system),
            crate::layer1::integration::drone_spawner_bridge_system
                .after(process_start_plan_system),
            crate::layer1::integration::drone_work_bridge_system.after(process_start_plan_system),
            crate::layer1::husbandry::pasture_confinement_system.after(process_start_plan_system),
            crate::layer1::fauna::fauna_behavior_system.after(process_start_plan_system),
            mascot_behavior_system.after(process_start_plan_system),
        )
            .in_set(Layer1SystemSet::Execution),
    );

    schedule.add_systems(
        (
            crate::layer1::day_night::update_day_night_cycle_system
                .after(process_start_plan_system),
            crate::layer1::day_night::update_ambient_light_from_cycle_system
                .after(crate::layer1::day_night::update_day_night_cycle_system),
            update_bioluminescence_system
                .after(crate::layer1::day_night::update_day_night_cycle_system),
            crate::layer1::pop::reset_speed_system.before(apply_lighting_penalties_system),
            update_lighting_system
                .after(process_start_plan_system)
                .after(crate::layer1::day_night::update_ambient_light_from_cycle_system)
                .after(update_bioluminescence_system),
            apply_lighting_penalties_system.after(update_lighting_system),
            apply_weather_effects_system.after(apply_lighting_penalties_system),
            apply_quirk_modifiers_system
                .after(apply_lighting_penalties_system)
                .after(apply_weather_effects_system),
            crate::layer1::chemical::apply_chemical_speed_modifiers_system
                .after(apply_quirk_modifiers_system),
            #[cfg(feature = "nova")]
            crate::layer1::observer::observer_reaction_system
                .after(apply_lighting_penalties_system),
            crate::layer1::combat::hit_stop_system.after(process_start_plan_system),
        )
            .in_set(Layer1SystemSet::Execution),
    );

    schedule.add_systems(
        (
            movement_system
                .after(apply_quirk_modifiers_system)
                .after(crate::layer1::fauna::fauna_behavior_system)
                .after(crate::layer1::combat::hit_stop_system),
            crate::layer1::crowding::crowding_accumulation_system.after(movement_system),
            crate::layer1::artifacts::aura_system.after(movement_system),
            arrival_handler_system.after(movement_system),
            work_execution_system.after(arrival_handler_system),
            crate::layer1::hobby::execute_hobby_system.after(arrival_handler_system),
            crate::layer1::husbandry::tame_execution_system.after(arrival_handler_system),
            combat_execution_system.after(arrival_handler_system),
            crate::layer1::turret::turret_fire_system.after(combat_execution_system),
            crate::layer1::justice::warden_execution_system.after(combat_execution_system),
            crate::layer1::predictive_policing::pre_crime_execution_system
                .after(combat_execution_system),
            crate::layer1::execution::vandalize_execution_system.after(arrival_handler_system),
            crate::layer1::drone::process_charge_system.after(arrival_handler_system),
            update_social_class_system.after(arrival_handler_system),
            class_friction_system.after(update_social_class_system),
            haul_system.after(arrival_handler_system),
            conveyor_system.after(haul_system),
            process_scan_system.after(arrival_handler_system),
            update_cabin_fever_system.after(movement_system),
            update_noise_system.after(work_execution_system),
        )
            .in_set(Layer1SystemSet::Execution),
    );

    schedule.add_systems(
        (
            wild_child_system.after(movement_system),
            update_erosion_system.after(movement_system),
            update_screen_shake_system.after(movement_system),
            crate::layer1::particles::particle_physics_system.after(movement_system),
            crate::layer1::particles::particle_system
                .after(crate::layer1::particles::particle_physics_system),
            infiltration_system.after(movement_system),
            discovery_system.after(process_scan_system),
        )
            .in_set(Layer1SystemSet::Execution),
    );

    // --- Economy ---
    schedule.add_systems(
        (
            update_resource_caps_system,
            advance_season_system,
            update_taboo_duration_system,
            update_water_system,
            update_weather_system,
            crate::layer1::fertility::update_fertility_system,
            produce_food_system.after(crate::layer1::fertility::update_fertility_system),
            crate::layer1::husbandry::husbandry_production_system.after(produce_food_system),
            hopper_system.after(produce_food_system),
            process_refining_system,
            crate::layer1::tech::update_tech_capacity_system,
            crate::layer1::admin::calculate_admin_stats,
            crate::layer1::eureka::handle_eureka_events,
        )
            .in_set(Layer1SystemSet::Economy),
    );

    schedule.add_systems(
        (
            process_research_system,
            process_observe_system,
            #[cfg(feature = "nova")]
            crate::layer1::constellations::observe_constellations_system,
            regrowth_system,
            crate::layer1::ecology::process_ecological_succession,
            flora_spread_system,
            mastery_accumulation_system,
            crate::layer1::cybernetics::surgery_system,
            crate::layer1::institutional_memory::produce_manual_system,
            crate::layer1::institutional_memory::manual_aura_system,
        )
            .in_set(Layer1SystemSet::Economy),
    );

    schedule.add_systems(
        (
            restore_rest_in_housing_system.after(update_noise_system),
            restore_leisure_system,
            crate::layer1::tech_envy::tech_envy_system.after(restore_leisure_system),
            apply_mood_modifiers_system.after(restore_leisure_system),
            mascot_buff_system.after(restore_leisure_system),
            crate::layer1::graffiti::graffiti_observation_system.after(apply_mood_modifiers_system),
            apply_catharsis_morale_bonus_system.after(apply_mood_modifiers_system),
            update_morale_cache_system
                .after(apply_catharsis_morale_bonus_system)
                .after(crate::layer1::graffiti::graffiti_observation_system)
                .after(mascot_buff_system),
            morale_decay_system.after(update_morale_cache_system),
        )
            .in_set(Layer1SystemSet::Economy),
    );

    schedule.add_systems(
        (
            healing_system,
            crate::layer1::integration::medical_debt_bridge_system.after(healing_system),
            crate::layer1::beauty::update_beauty_grid_system,
            crate::layer1::beauty::apply_beauty_effects_system
                .after(crate::layer1::beauty::update_beauty_grid_system),
            check_heirloom_status_system,
            crate::layer1::trade::merchant_arrival_system,
            crate::layer1::visitor::spawn_visitor_system,
            spawn_inspector_system.after(crate::layer1::visitor::spawn_visitor_system),
            process_fuel_consumption_system,
            crate::layer1::energy::power_grid_system.after(process_fuel_consumption_system),
            ai_automation_system.after(crate::layer1::energy::power_grid_system),
            ai_rogue_system,
        )
            .in_set(Layer1SystemSet::Economy),
    );

    schedule.add_systems(
        (
            crate::layer1::integration::grid_overload_fire_bridge
                .after(crate::layer1::energy::power_grid_system),
            art_generation_system,
            crate::layer1::factions::update_faction_membership_system,
            crate::layer1::factions::update_faction_satisfaction_system
                .after(crate::layer1::factions::update_faction_membership_system),
            crate::layer1::factions::update_faction_demands_system
                .after(crate::layer1::factions::update_faction_satisfaction_system),
            crate::layer1::factions::update_faction_strikes_system
                .after(crate::layer1::factions::update_faction_demands_system),
            apply_founder_benefits_system,
            // Layer 2 visibility systems are handled in simulation.rs
        )
            .in_set(Layer1SystemSet::Economy),
    );

    // --- Environment ---
    schedule.add_systems(
        (
            fire_pressure_check_system,
            fire_spread_system.after(fire_pressure_check_system),
            fire_damage_pops_system.after(fire_spread_system),
            crate::layer1::structure::fire_damage_structure_system.after(fire_spread_system),
            fire_damage_system
                .after(fire_spread_system)
                .after(fire_damage_pops_system)
                .after(crate::layer1::structure::fire_damage_structure_system),
            blob_spread_system,
            blob_consumption_system.after(blob_spread_system),
            flora_attack_system,
            ancient_structure_decay_system,
            crate::layer1::graffiti::graffiti_decay_system,
        )
            .in_set(Layer1SystemSet::Environment),
    );

    schedule.add_systems(
        (
            crate::layer1::seismic::update_seismic_system,
            crate::layer1::seismic::seismic_flora_reaction_system
                .after(crate::layer1::seismic::update_seismic_system),
            crate::layer1::seismic::seismic_instability_system
                .after(crate::layer1::seismic::update_seismic_system),
            crate::layer1::geology::seismic_decay_system,
            crate::layer1::geology::check_seismic_events
                .after(crate::layer1::geology::seismic_decay_system),
            crate::layer1::geology::apply_geological_event_system
                .after(crate::layer1::geology::check_seismic_events)
                .after(crate::layer1::seismic::seismic_instability_system),
            spirit_decay_system,
            quirk_generation_system.after(spirit_decay_system),
            entropy_system,
            crate::layer1::structure::fragile_decay_system.after(entropy_system),
            crate::layer1::crowding::crowding_decay_system,
        )
            .in_set(Layer1SystemSet::Environment),
    );

    schedule.add_systems(
        (
            crate::layer1::atmosphere::update_atmospheric_tide_system,
            crate::layer1::atmosphere::sync_global_wind_system
                .after(crate::layer1::atmosphere::update_atmospheric_tide_system),
            crate::layer1::wind::update_wind_system
                .after(crate::layer1::atmosphere::sync_global_wind_system),
            #[cfg(feature = "nova")]
            crate::layer1::constellations::update_sky_system,
            malfunction_system.after(entropy_system),
            apply_noise_effects_system.after(update_noise_system),
            waste_pollution_bridge,
            crate::layer1::atmosphere::update_atmosphere_system
                .after(waste_pollution_bridge)
                .after(crate::layer1::wind::update_wind_system),
            update_pressure_system,
            crate::layer1::temperature::update_temperature_system.after(update_pressure_system),
            crate::layer1::radioactive::radiation_system
                .after(crate::layer1::temperature::update_temperature_system),
            crate::layer1::suction::suction_system.after(update_pressure_system),
            crate::layer1::integration::vacuum_clears_pollution_system
                .after(crate::layer1::atmosphere::update_atmosphere_system)
                .after(update_pressure_system),
            biocompatibility_system.after(crate::layer1::atmosphere::update_atmosphere_system),
            crate::layer1::pheromone::reactive_emitter_system
                .after(crate::layer1::atmosphere::update_atmosphere_system),
            crate::layer1::pheromone::pheromone_emission_system
                .after(crate::layer1::pheromone::reactive_emitter_system),
        )
            .in_set(Layer1SystemSet::Environment),
    );

    // --- Consumption ---
    schedule.add_systems(
        (
            consume_food_system
                .after(produce_food_system)
                .after(update_resource_caps_system),
            crate::layer1::drone::drone_battery_system.after(consume_food_system),
            clothing_wear_system.after(consume_food_system),
            vermin_growth_system.after(consume_food_system),
            vermin_effect_system.after(vermin_growth_system),
            vermin_morale_system.after(vermin_growth_system),
            crate::layer1::integration::vermin_item_rot_system.after(vermin_growth_system),
            spoilage_system
                .after(consume_food_system)
                .after(vermin_growth_system),
            crate::layer1::visitor::visitor_lifecycle_system.after(consume_food_system),
            theft_system
                .after(consume_food_system)
                .before(decay_needs_system),
            decay_needs_system.after(consume_food_system),
            crate::layer1::chemical::addiction_system.after(decay_needs_system),
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
        )
            .in_set(Layer1SystemSet::Consumption),
    );

    schedule.add_systems(
        (
            mood_lifecycle_system.after(decay_needs_system),
            trend_setting_system.after(consume_food_system),
            trend_spread_system.after(trend_setting_system),
            trend_satisfaction_system.after(trend_spread_system),
            clear_just_consumed_system.after(trend_satisfaction_system),
            crate::layer1::gastronomy::handle_work_speed_buff_decay.after(decay_needs_system),
            crate::layer1::gastronomy::handle_hallucination_decay.after(decay_needs_system),
        )
            .in_set(Layer1SystemSet::Consumption),
    );

    schedule.add_systems(
        (
            crate::layer1::justice::update_inmates_system.after(decay_needs_system),
            crate::layer1::day_night::circadian_rhythm_system.after(consume_food_system),
            aging_system.after(consume_food_system),
            natural_death_system.after(aging_system),
            memory_decay_system.after(decay_needs_system),
            notification_expiration_system.after(decay_needs_system),
            crate::layer1::temperature::thermal_damage_system.after(decay_needs_system),
            crate::layer1::radioactive::sickness_damage_system.after(decay_needs_system),
            pressure_damage_system.after(decay_needs_system),
            crate::layer1::needs::starvation_damage_system.after(decay_needs_system),
            crate::layer1::health::check_health_status_system
                .after(crate::layer1::needs::starvation_damage_system)
                .after(crate::layer1::temperature::thermal_damage_system)
                .after(crate::layer1::radioactive::sickness_damage_system)
                .after(pressure_damage_system)
                .after(natural_death_system),
            crate::layer1::pop::handle_pop_death_system
                .after(crate::layer1::health::check_health_status_system),
            crate::layer1::fauna::handle_fauna_death_system
                .after(crate::layer1::health::check_health_status_system),
            crate::layer1::pop::handle_witness_death_system
                .after(crate::layer1::pop::handle_pop_death_system),
            mascot_death_grief_system.after(crate::layer1::health::check_health_status_system),
            crate::layer1::health::despawn_dead_entities_system
                .after(crate::layer1::pop::handle_pop_death_system)
                .after(crate::layer1::fauna::handle_fauna_death_system)
                .after(mascot_death_grief_system),
            clean_dead_residents_system.after(crate::layer1::health::despawn_dead_entities_system),
            clean_dead_workers_system.after(crate::layer1::health::despawn_dead_entities_system),
        )
            .in_set(Layer1SystemSet::Consumption),
    );

    // --- Observation ---
    schedule.add_systems(
        (
            biography_monitor_system.after(crate::layer1::health::despawn_dead_entities_system),
            crate::layer1::graffiti::graffiti_placement_system.after(crate::layer1::health::despawn_dead_entities_system),
            dream_system.after(crate::layer1::health::despawn_dead_entities_system),
            crate::layer1::cryo_dreams::cryo_dream_system.after(crate::layer1::health::despawn_dead_entities_system),
            cleanup_dream_marker_system.after(dream_system),
            #[cfg(feature = "nova")]
            crate::layer1::observer::observer_awareness_system.after(crate::layer1::health::despawn_dead_entities_system),
        )
            .in_set(Layer1SystemSet::Observation),
    );

    schedule.add_systems(
        (
            check_milestones_system.after(crate::layer1::health::despawn_dead_entities_system),
            crate::layer1::rumor::generate_rumor_system.after(crate::layer1::health::despawn_dead_entities_system),
            crate::layer1::rumor::exchange_rumors_system.after(crate::layer1::health::despawn_dead_entities_system),
            crate::layer1::society::form_societies_system.after(crate::layer1::health::despawn_dead_entities_system),
            crate::layer1::society::society_meeting_system.after(crate::layer1::health::despawn_dead_entities_system),
            crate::layer1::society::investigation_handler_system.after(crate::layer1::health::despawn_dead_entities_system),
            crate::layer1::society::suppression_handler_system.after(crate::layer1::health::despawn_dead_entities_system),
            crate::layer1::funeral::grief_system.after(crate::layer1::health::despawn_dead_entities_system),
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
            crate::layer1::hobby::assign_hobby_system.after(decay_needs_system),
            crate::layer1::predictive_policing::check_prediction_system.after(decay_needs_system),
        )
            .in_set(Layer1SystemSet::Observation),
    );

    schedule.add_systems(
        (
            // Process new rumors and affinity changes
            modify_affinity_system.after(crate::layer1::rumor::exchange_rumors_system),
            crate::layer1::social::proximity_social_system.after(modify_affinity_system),
            // Process chronicle events
            chronicle_event_handler_system.after(check_milestones_system),
            crate::layer1::festivals::check_for_festivals_system
                .after(chronicle_event_handler_system),
            crate::layer1::festivals::festival_lifecycle_system
                .after(chronicle_event_handler_system),
            chronicle_rumor_bridge_system.after(check_milestones_system),
            #[cfg(feature = "nova")]
            crate::layer1::oral_tradition::collect_chronicles_system
                .after(chronicle_event_handler_system),
            #[cfg(feature = "nova")]
            crate::layer1::oral_tradition::storytelling_system
                .after(crate::layer1::rumor::exchange_rumors_system),
            pop_death_chronicle_bridge.after(crate::layer1::health::despawn_dead_entities_system),
            retrograde_chronicle_bridge.after(work_execution_system), // work_execution_system is in Execution set
            amputation_handler_system.after(work_execution_system),
            art_observation_system.after(crate::layer1::health::despawn_dead_entities_system),
            observe_inspector_system.after(art_observation_system),
            crate::layer1::integration::medical_treatment_notification_system.after(healing_system),
            crate::layer1::integration::hospitalization_notification_system.after(work_execution_system),
            crate::layer1::integration::pop_death_notification_system.after(natural_death_system),
        )
            .in_set(Layer1SystemSet::Observation),
    );

    schedule.add_systems(
        (
            inspector_report_system.after(chronicle_event_handler_system),
            inspector_outcome_bridge_system.after(inspector_report_system),
            taboo_event_system.after(crate::layer1::health::despawn_dead_entities_system),
        )
            .in_set(Layer1SystemSet::Observation),
    );
}
