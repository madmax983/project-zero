use super::Layer1SystemSet;
use crate::layer1::direct_link::clear_input_system;
use crate::layer1::inspector::{inspector_report_system, observe_inspector_system};
use crate::layer1::social::old_guard::check_generational_friction_system;
use crate::layer1::*;
use bevy_ecs::prelude::*;

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(
        (
            crate::layer1::integration::waste_scent_bridge,
            crate::layer1::olfactory::scent_diffusion_system
                .after(crate::layer1::integration::waste_scent_bridge),
            crate::layer1::olfactory::scent_mood_system
                .after(crate::layer1::olfactory::scent_diffusion_system),
        )
            .in_set(Layer1SystemSet::Observation),
    );
    schedule.add_systems(
        (
            biography_monitor_system.after(crate::layer1::health::despawn_dead_entities_system),
            crate::layer1::social::cadet::death_consequence_system,
            crate::layer1::graffiti::graffiti_placement_system
                .after(crate::layer1::health::despawn_dead_entities_system),
            dream_system.after(crate::layer1::health::despawn_dead_entities_system),
            crate::layer1::cryo_dreams::cryo_dream_system
                .after(crate::layer1::health::despawn_dead_entities_system),
            cleanup_dream_marker_system.after(dream_system),
            #[cfg(feature = "nova")]
            crate::layer1::observer::observer_awareness_system
                .after(crate::layer1::health::despawn_dead_entities_system),
            #[cfg(feature = "nova")]
            crate::layer1::machine_consciousness::machine_personality_system
                .after(crate::layer1::health::despawn_dead_entities_system),
            crate::layer1::void_stare::update_void_exposure_system
                .after(crate::layer1::health::despawn_dead_entities_system),
            crate::layer1::void_stare::void_manifestation_system
                .after(crate::layer1::void_stare::update_void_exposure_system),
            crate::layer1::integration::scapegoat_chronicle_bridge,
        )
            .in_set(Layer1SystemSet::Observation),
    );

    schedule.add_systems(
        (
            check_milestones_system.after(crate::layer1::health::despawn_dead_entities_system),
            crate::layer1::rumor::generate_rumor_system
                .after(crate::layer1::health::despawn_dead_entities_system),
            crate::layer1::rumor::exchange_rumors_system
                .after(crate::layer1::health::despawn_dead_entities_system),
            crate::layer1::society::form_societies_system
                .after(crate::layer1::health::despawn_dead_entities_system),
            crate::layer1::society::society_meeting_system
                .after(crate::layer1::health::despawn_dead_entities_system),
            crate::layer1::society::investigation_handler_system
                .after(crate::layer1::health::despawn_dead_entities_system),
            crate::layer1::society::suppression_handler_system
                .after(crate::layer1::health::despawn_dead_entities_system),
            crate::layer1::funeral::grief_system
                .after(crate::layer1::health::despawn_dead_entities_system),
            crate::layer1::funeral::grave_visit_system.after(crate::layer1::funeral::grief_system),
            crate::layer1::unrest::calculate_unrest_system.after(decay_needs_system),
            crate::layer1::bio_acoustic::bio_acoustic_chorus_system
                .after(crate::layer1::unrest::calculate_unrest_system),
        )
            .in_set(Layer1SystemSet::Observation),
    );

    schedule.add_systems(
        (
            crate::layer1::unrest::identify_scapegoat_system
                .after(crate::layer1::unrest::calculate_unrest_system),
            crate::layer1::unrest::handle_denounce_event_system
                .after(crate::layer1::unrest::calculate_unrest_system),
            crate::layer1::unrest::check_mental_break_system.after(decay_needs_system),
            check_stress_breakdown_system.after(decay_needs_system),
            crate::layer1::totems::check_spontaneous_totem_creation
                .after(check_stress_breakdown_system),
            crate::layer1::totems::unequip_totem_system.after(decay_needs_system),
            update_breakdown_system.after(check_stress_breakdown_system),
            update_catharsis_duration_system.after(decay_needs_system),
            #[cfg(feature = "nova")]
            crate::layer1::sleep_deprived_savant::fever_dream_system
                .after(crate::layer1::needs::decay_needs_system),
            check_sleepwalking_start_system.after(decay_needs_system),
            sleepwalk_end_system.after(decay_needs_system),
        )
            .in_set(Layer1SystemSet::Observation),
    );

    schedule.add_systems(
        (
            crate::layer1::justice::check_crime_system
                .after(crate::layer1::unrest::check_mental_break_system),
            crate::layer1::contraband::detect_contraband_system
                .after(crate::layer1::justice::check_crime_system),
            crate::layer1::unrest::recover_mental_break_system.after(decay_needs_system),
            check_generational_friction_system.after(decay_needs_system),
            crate::layer1::hobby::assign_hobby_system.after(decay_needs_system),
            crate::layer1::predictive_policing::check_prediction_system.after(decay_needs_system),
            crate::layer1::social::grievances::post_grievance_system.after(decay_needs_system),
            crate::layer1::social::grievances::read_board_system.after(decay_needs_system),
            crate::layer1::social::cultural_vandalism::vandalism_system.after(decay_needs_system),
            crate::layer1::social::cultural_vandalism::update_structure_buffs
                .after(crate::layer1::social::cultural_vandalism::vandalism_system),
        )
            .in_set(Layer1SystemSet::Observation),
    );

    schedule.add_systems(
        (
            crate::layer1::integration::update_unmet_luxury_system.after(decay_needs_system),
            crate::layer1::integration::sacrilege_unrest_bridge.after(decay_needs_system),
            crate::layer1::integration::issue_placebo_from_edict_system.after(decay_needs_system),
            crate::layer1::social::placebo::placebo_tick_system.after(decay_needs_system),
            crate::layer1::social::placebo::reveal_betrayal_system.after(decay_needs_system),
            crate::layer1::contagion::emotional_contagion_system.after(decay_needs_system),
            #[cfg(feature = "nova")]
            crate::experimental::echo_chamber::detect_echo_chamber_system
                .after(crate::layer1::needs::decay_needs_system),
            #[cfg(feature = "nova")]
            crate::experimental::echo_chamber::apply_echo_chamber_system
                .after(crate::experimental::echo_chamber::detect_echo_chamber_system),
            #[cfg(feature = "nova")]
            crate::experimental::emotional_weather::emotional_weather_system
                .after(crate::layer1::needs::decay_needs_system),
            #[cfg(feature = "nova")]
            crate::experimental::synesthesia::synesthesia_system
                .after(crate::layer1::needs::decay_needs_system),
            #[cfg(feature = "nova")]
            crate::experimental::fungal_death::fungal_death_system
                .after(crate::layer1::health::despawn_dead_entities_system),
            #[cfg(feature = "nova")]
            crate::experimental::bioluminescent_trails::spawn_bioluminescent_trails_system
                .after(crate::layer1::needs::decay_needs_system),
            #[cfg(feature = "nova")]
            crate::experimental::bioluminescent_trails::fade_bioluminescent_trails_system.after(
                crate::experimental::bioluminescent_trails::spawn_bioluminescent_trails_system,
            ),
            #[cfg(feature = "nova")]
            crate::experimental::sympathetic_architecture::sympathetic_architecture_system
                .after(crate::layer1::health::despawn_dead_entities_system),
            #[cfg(feature = "nova")]
            crate::experimental::psychic_resonance::psychic_resonance_system
                .after(crate::layer1::needs::decay_needs_system),
        )
            .in_set(Layer1SystemSet::Observation),
    );

    schedule.add_systems(
        (
            // Process new rumors and affinity changes
            modify_affinity_system.after(crate::layer1::rumor::exchange_rumors_system),
            crate::layer1::social::proximity_social_system.after(modify_affinity_system),
            crate::layer1::social::pen_pals::update_pen_pals_system.after(modify_affinity_system),
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
        )
            .in_set(Layer1SystemSet::Observation),
    );

    schedule.add_systems(
        (
            pop_death_chronicle_bridge.after(crate::layer1::health::despawn_dead_entities_system),
            mega_quake_chronicle_bridge.after(crate::layer1::geology::tectonic::check_quake_system),
            crate::layer1::integration::orbital_drop_chronicle_bridge
                .after(crate::layer1::logistics::orbital_drop::process_orbital_drops),
            crate::layer1::integration::hologram_failure_chronicle_bridge
                .after(crate::layer1::hologram::update_holograms_system),
            retrograde_chronicle_bridge.after(work_execution_system), // work_execution_system is in Execution set
            amputation_handler_system.after(work_execution_system),
            art_observation_system.after(crate::layer1::health::despawn_dead_entities_system),
            observe_inspector_system.after(art_observation_system),
            crate::layer1::integration::medical_treatment_notification_system.after(healing_system),
            crate::layer1::integration::hospitalization_notification_system
                .after(work_execution_system),
            crate::layer1::integration::pop_death_notification_system.after(natural_death_system),
            crate::layer1::integration::pop_born_notification_system
                .after(Layer1SystemSet::Economy),
        )
            .in_set(Layer1SystemSet::Observation),
    );

    schedule.add_systems(
        (
            inspector_report_system.after(chronicle_event_handler_system),
            inspector_outcome_bridge_system.after(inspector_report_system),
            taboo_event_system.after(crate::layer1::health::despawn_dead_entities_system),
            crate::layer1::quantum_twins::update_twin_sync_system.after(Layer1SystemSet::Economy),
            crate::layer1::quantum_twins::update_twin_mood_system
                .after(Layer1SystemSet::Consumption),
            // Fix: handle_pop_death_system is re-exported in layer1/mod.rs or located in layer1/pop.rs
            // The previous error was referencing crate::layer1::health::handle_pop_death_system
            crate::layer1::quantum_twins::handle_severance_system
                .after(crate::layer1::pop::handle_pop_death_system),
            crate::layer1::ad_screen::update_ad_screens_system,
            crate::layer1::integration::industrial_rhythm_morale_bridge,
            crate::layer1::integration::great_work_chronicle_bridge,
            crate::layer1::integration::orbital_crossfire_bridge,
        )
            .in_set(Layer1SystemSet::Observation),
    );

    schedule.add_systems(
        (
            crate::layer1::overview_effect::overview_effect_system,
            crate::layer1::spiteful_will::process_spiteful_will_system,
            crate::layer1::spiteful_will::process_override_will_system,
            crate::layer1::integration::override_will_chronicle_bridge
                .after(crate::layer1::spiteful_will::process_override_will_system),
            crate::layer1::tech::neural_leech::apply_neural_link_buffs_system,
            crate::layer1::tech::neural_leech::process_neural_hub_decay_system,
            crate::layer1::tech::neural_leech::handle_hub_death_system,
            crate::layer1::integration::apply_neural_shock_system
                .after(crate::layer1::tech::neural_leech::handle_hub_death_system),
            crate::layer1::tech::neural_leech::neural_hub_death_bridge_system,
        )
            .in_set(Layer1SystemSet::Observation),
    );

    schedule.add_systems(clear_input_system.after(Layer1SystemSet::Observation));
}
