use super::Layer1SystemSet;
use crate::layer1::direct_link::clear_input_system;
use crate::layer1::inspector::{inspector_report_system, observe_inspector_system};
use crate::layer1::social::old_guard::check_generational_friction_system;
use crate::layer1::*;
use bevy_ecs::prelude::*;

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(
        (
            crate::layer1::stress::assign_generational_traits_system,
            crate::layer1::stress::silent_needs_suppression_system,
            crate::layer1::law::aesthetic_edict::evaluate_aesthetic_edict_system,
            crate::layer1::integration::diplomatic_reflection_kill_bridge,
            crate::layer1::integration::diplomatic_reflection_plant_bridge,
            crate::layer1::integration::waste_scent_bridge,
            crate::layer1::core::integration::flora_scent_bridge_system,
            crate::layer1::olfactory::scent_diffusion_system
                .after(crate::layer1::integration::waste_scent_bridge)
                .after(crate::layer1::core::integration::flora_scent_bridge_system),
            crate::layer1::olfactory::scent_mood_system
                .after(crate::layer1::olfactory::scent_diffusion_system),
        )
            .in_set(Layer1SystemSet::Observation),
    );

    #[cfg(feature = "nova")]
    schedule.add_systems(
        (
            crate::experimental::chrono_stutter::spawn_chrono_anomaly_system
                .after(decay_needs_system),
            crate::experimental::chrono_stutter::apply_chrono_stutter_system
                .after(crate::experimental::chrono_stutter::spawn_chrono_anomaly_system),
            crate::experimental::cartography_export::map_export_system,
            crate::experimental::paranoia_network::paranoia_network_system,
            crate::experimental::epigenetic_stress::epigenetic_mutation_system,
            crate::experimental::cursed_artifacts::spawn_cursed_anomalies_system,
            crate::experimental::cursed_artifacts::apply_curse_system,
            crate::experimental::empathy_cascade::init_empathic_health_tracker_system,
            crate::experimental::empathy_cascade::empathy_cascade_system
                .after(crate::experimental::empathy_cascade::init_empathic_health_tracker_system),
            crate::experimental::symbiotic_parasite::symbiotic_parasite_system,
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
            crate::experimental::dream_economy::harvest_dreams_system
                .after(crate::layer1::dreams::dream_system),
            #[cfg(feature = "nova")]
            crate::experimental::dream_economy::nightmare_paranoia_system,
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
            crate::layer1::social::society::form_societies_system
                .after(crate::layer1::health::despawn_dead_entities_system),
            crate::layer1::social::society::society_meeting_system
                .after(crate::layer1::health::despawn_dead_entities_system),
            crate::layer1::social::society::investigation_handler_system
                .after(crate::layer1::health::despawn_dead_entities_system),
            crate::layer1::social::society::suppression_handler_system
                .after(crate::layer1::health::despawn_dead_entities_system),
            crate::layer1::funeral::grief_system
                .after(crate::layer1::health::despawn_dead_entities_system),
            crate::layer1::ancestral_graves::grave_visit_system
                .after(crate::layer1::funeral::grief_system),
            crate::layer1::unrest::calculate_unrest_system.after(decay_needs_system),
            crate::layer1::environment::bio_acoustic::bio_acoustic_chorus_system
                .after(crate::layer1::unrest::calculate_unrest_system),
            crate::layer1::environment::bio_acoustic_miasma::record_miasma_secret
                .after(crate::layer1::unrest::calculate_unrest_system),
            crate::layer1::environment::bio_acoustic_miasma::broadcast_miasma_secrets
                .after(crate::layer1::environment::bio_acoustic_miasma::record_miasma_secret),
            crate::layer1::integration::paranoia_stress_bridge_system
                .after(crate::layer1::environment::bio_acoustic_miasma::broadcast_miasma_secrets)
                .before(crate::layer1::stress::check_stress_breakdown_system),
        )
            .in_set(Layer1SystemSet::Observation),
    );

    schedule.add_systems(
        (
            crate::layer1::unrest::identify_scapegoat_system
                .after(crate::layer1::unrest::calculate_unrest_system),
            crate::layer1::unrest::handle_denounce_event_system
                .after(crate::layer1::unrest::calculate_unrest_system),
            crate::layer1::social::ghost_shift_strike::evaluate_ghost_shifts
                .after(crate::layer1::unrest::calculate_unrest_system),
            crate::layer1::unrest::check_mental_break_system.after(decay_needs_system),
            check_stress_breakdown_system.after(decay_needs_system),
            crate::layer1::totems::check_spontaneous_totem_creation
                .after(check_stress_breakdown_system),
            crate::layer1::totems::unequip_totem_system.after(decay_needs_system),
            crate::layer1::administration::edicts::update_policy_tradition_system,
            crate::layer1::administration::edicts::handle_revoke_policy_system,
            update_breakdown_system.after(check_stress_breakdown_system),
            crate::layer1::mind::process_fugue_onset.after(check_stress_breakdown_system),
            crate::layer1::mind::enforce_fugue_job.after(crate::layer1::mind::process_fugue_onset),
            crate::layer1::mind::process_fugue_spread.after(crate::layer1::mind::enforce_fugue_job),
            update_catharsis_duration_system.after(decay_needs_system),
            #[cfg(feature = "nova")]
            crate::layer1::sleep_deprived_savant::fever_dream_system
                .after(crate::layer1::needs::decay_needs_system),
            check_sleepwalking_start_system.after(decay_needs_system),
            crate::layer1::cryo_shock::decay_cryo_shock_system.after(decay_needs_system),
            sleepwalk_end_system.after(decay_needs_system),
        )
            .in_set(Layer1SystemSet::Observation),
    );

    schedule.add_systems(
        (
            crate::layer1::law::justice::check_crime_system
                .after(crate::layer1::unrest::check_mental_break_system),
            crate::layer1::law::contraband::detect_contraband_system
                .after(crate::layer1::law::justice::check_crime_system),
            crate::layer1::unrest::recover_mental_break_system.after(decay_needs_system),
            crate::layer1::civic_ideology::apply_ideological_modifiers_system
                .after(crate::layer1::unrest::recover_mental_break_system),
            crate::layer1::civic_ideology::decay_recent_action_system
                .after(crate::layer1::civic_ideology::apply_ideological_modifiers_system),
            check_generational_friction_system.after(decay_needs_system),
            crate::layer1::social::process_generational_dissonance_system.after(decay_needs_system),
            crate::layer1::social::evaluate_safety_edicts_system.after(decay_needs_system),
            crate::layer1::hobby::assign_hobby_system.after(decay_needs_system),
            crate::layer1::law::predictive_policing::check_prediction_system
                .after(decay_needs_system),
            crate::layer1::social::grievances::post_grievance_system.after(decay_needs_system),
            crate::layer1::social::grievances::read_board_system.after(decay_needs_system),
            crate::layer1::social::cultural_vandalism::vandalism_system.after(decay_needs_system),
            crate::layer1::social::cultural_vandalism::update_structure_buffs
                .after(crate::layer1::social::cultural_vandalism::vandalism_system),
            crate::layer1::social::indoctrination::process_indoctrination_system
                .after(decay_needs_system),
        )
            .in_set(Layer1SystemSet::Observation),
    );

    #[cfg(feature = "nova")]
    schedule.add_systems(
        (
            crate::experimental::dreams_of_genesis::dreams_of_genesis_system
                .after(decay_needs_system),
        )
            .in_set(Layer1SystemSet::Observation),
    );
    #[cfg(feature = "nova")]
    schedule.add_systems(
        (
            crate::experimental::the_feral_choir::detect_feral_choir_system
                .after(decay_needs_system),
            crate::experimental::the_feral_choir::dissolve_feral_choir_system
                .after(crate::experimental::the_feral_choir::detect_feral_choir_system),
        )
            .in_set(Layer1SystemSet::Observation),
    );

    schedule.add_systems(
        (
            #[cfg(feature = "nova")]
            crate::experimental::the_humming_monolith::detect_and_spawn_monolith_system
                .after(decay_needs_system),
            #[cfg(feature = "nova")]
            crate::experimental::the_humming_monolith::monolith_influence_system
                .after(crate::experimental::the_humming_monolith::detect_and_spawn_monolith_system),
            crate::layer1::integration::update_unmet_luxury_system.after(decay_needs_system),
            crate::layer1::integration::sacrilege_unrest_bridge.after(decay_needs_system),
            crate::layer1::integration::issue_placebo_from_edict_system.after(decay_needs_system),
            crate::layer1::social::placebo::placebo_tick_system.after(decay_needs_system),
            crate::layer1::social::placebo::reveal_betrayal_system.after(decay_needs_system),
            crate::layer1::social::exile::process_banishments.after(decay_needs_system),
            crate::layer1::social::exile::evaluate_exile_returns.after(decay_needs_system),
            crate::layer1::contagion::emotional_contagion_system.after(decay_needs_system),
            crate::layer1::social::sentient_standard::apply_sentient_standard_stress_system
                .after(decay_needs_system),
            #[cfg(feature = "nova")]
            crate::experimental::necro_industry::necro_industry_system
                .after(crate::layer1::needs::decay_needs_system),
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
            crate::experimental::cargo_cult_fleet::spawn_cargo_cult_fleet_system,
            #[cfg(feature = "nova")]
            crate::experimental::cargo_cult_fleet::feed_cargo_cult_tether_system,
        )
            .in_set(Layer1SystemSet::Observation),
    );

    #[cfg(feature = "nova")]
    schedule.add_systems(
        (
            crate::experimental::phantom_workforce::spawn_phantom_workers_system,
            crate::experimental::phantom_workforce::phantom_production_system
                .after(crate::experimental::phantom_workforce::spawn_phantom_workers_system),
            crate::experimental::phantom_workforce::phantom_terror_system
                .after(crate::experimental::phantom_workforce::spawn_phantom_workers_system),
        )
            .in_set(Layer1SystemSet::Observation),
    );

    schedule.add_systems(
        (
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
            #[cfg(feature = "nova")]
            crate::experimental::sleep_deprived_savant::sleep_deprived_savant_system,
            #[cfg(feature = "nova")]
            crate::experimental::fever_pitch::fever_pitch_system
                .after(crate::layer1::needs::decay_needs_system),
            #[cfg(feature = "nova")]
            crate::experimental::genetic_memory::absorb_genetic_memory_system
                .after(crate::layer1::health::despawn_dead_entities_system),
            #[cfg(feature = "nova")]
            crate::experimental::genetic_memory::inherit_genetic_memory_system
                .after(crate::experimental::genetic_memory::absorb_genetic_memory_system),
            crate::layer1::psychic::apply_psychic_radiation_system
                .after(crate::layer1::needs::decay_needs_system),
            #[cfg(feature = "nova")]
            crate::experimental::meme_plague::process_meme_contagion
                .after(crate::layer1::needs::decay_needs_system),
            #[cfg(feature = "nova")]
            crate::experimental::meme_plague::apply_meme_effects
                .after(crate::experimental::meme_plague::process_meme_contagion),
        )
            .in_set(Layer1SystemSet::Observation),
    );

    schedule.add_systems(
        (
            // Mentorship
            crate::layer1::mentorship::check_mentorship_system,
            crate::layer1::mentorship::apply_mentorship_xp_system,
            crate::layer1::mentorship::mentorship_mood_system
                .after(crate::layer1::mentorship::check_mentorship_system),
            crate::layer1::mentorship::mentorship_mood_system
                .after(crate::layer1::mentorship::check_mentorship_system),
            // Process new rumors and affinity changes
            modify_affinity_system.after(crate::layer1::rumor::exchange_rumors_system),
            crate::layer1::social::gossip_economy::process_gossip
                .after(crate::layer1::rumor::exchange_rumors_system),
            crate::layer1::social::gossip_economy::spend_intel
                .after(crate::layer1::social::gossip_economy::process_gossip),
            crate::layer1::social::gossip_economy::decrement_gossiping_system
                .after(crate::layer1::social::gossip_economy::process_gossip),
            crate::layer1::social::proximity_social_system.after(modify_affinity_system),
            crate::layer1::social::pen_pals::update_pen_pals_system.after(modify_affinity_system),
            // Process chronicle events
            crate::layer1::geography::process_historical_events,
            crate::layer1::flora::detect_hazards_system,
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
            crate::layer1::integration::trauma_death_bridge_system,
            crate::layer1::integration::famine_tracking_system,
            crate::layer1::integration::trauma_decay_system,
            pop_death_chronicle_bridge.after(crate::layer1::health::despawn_dead_entities_system),
            mega_quake_chronicle_bridge.after(crate::layer1::geology::tectonic::check_quake_system),
            crate::layer1::integration::orbital_drop_chronicle_bridge
                .after(crate::layer1::logistics::orbital_drop::process_orbital_drops),
            crate::layer1::integration::mass_driver_chronicle_bridge
                .after(crate::layer1::logistics::mass_driver::package_arrival_system),
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
            crate::layer1::radio_nostalgia::handle_broadcasts_system,
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
            crate::layer1::integration::gene_splicing_chronicle_bridge,
            crate::layer1::integration::crop_mutation_chronicle_bridge,
            crate::layer1::integration::grafting_chronicle_bridge,
            crate::layer1::integration::temporal_stutter_chronicle_bridge,
            crate::layer1::integration::parasitic_architecture_chronicle_bridge,
            crate::layer1::integration::phantom_shift_chronicle_bridge,
            crate::layer1::integration::tether_snap_chronicle_bridge,
        )
            .in_set(Layer1SystemSet::Observation),
    );
    schedule.add_systems(
        (
            crate::layer1::integration::famine_chronicle_bridge,
            crate::layer1::integration::silent_flora_chronicle_bridge,
            crate::layer1::integration::aesthetic_edict_chronicle_bridge,
            crate::layer1::integration::access_denied_chronicle_bridge,
            crate::layer1::integration::hack_hub_chronicle_bridge,
            crate::layer1::integration::smuggler_arrival_event_bridge
                .before(crate::layer1::void_weed::process_void_weed_trade_system),
        )
            .in_set(Layer1SystemSet::Observation),
    );

    schedule.add_systems(
        (
            crate::layer1::void_weed::process_void_weed_trade_system,
            crate::layer1::void_weed::evaluate_smuggling_heat_system,
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
            crate::layer1::memory_core::implant_memory_core_system,
            crate::layer1::memory_core::harvest_memory_core_system,
            crate::layer1::integration::bot_awakening_chronicle_bridge
                .after(crate::layer1::tech::machine_awakening::process_bot_sentience),
            crate::layer1::fauna::shadow::spawn_shadow_fauna_system,
            crate::layer1::fauna::shadow::shadow_feed_system,
            crate::layer1::fauna::shadow::shadow_visibility_system,
            crate::layer1::spore_diplomat::spore_stat_boost_system,
            crate::layer1::spore_diplomat::spore_stat_remove_system,
        )
            .in_set(Layer1SystemSet::Observation),
    );

    schedule.add_systems(
        (
            crate::layer1::diplomacy::wards::process_diplomatic_wards_system,
            crate::layer1::diplomacy::wards::process_ward_deaths_system,
            crate::layer1::unseen_bureaucracy::phantom_shift_system,
        )
            .in_set(Layer1SystemSet::Observation),
    );

    schedule.add_systems(clear_input_system.after(Layer1SystemSet::Observation));
}
