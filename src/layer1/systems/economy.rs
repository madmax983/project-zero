use super::Layer1SystemSet;
use crate::layer1::inspector::spawn_inspector_system;
use crate::layer1::social::old_guard::{
    apply_founder_benefits_system, apply_mood_modifiers_system,
};
use crate::layer1::*;
use bevy_ecs::prelude::*;

pub fn register(schedule: &mut Schedule) {
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
            crate::layer1::husbandry::simulate_lithovore_metabolism
                .after(crate::layer1::husbandry::husbandry_production_system),
            crate::layer1::husbandry::lithovore_eating_system
                .after(crate::layer1::husbandry::simulate_lithovore_metabolism),
            (
                hopper_system.after(produce_food_system),
                process_refining_system,
                crate::layer1::tech::rhythm::update_rhythm_system.after(process_refining_system),
                crate::layer1::social::cadet::process_allowance_system,
                crate::layer1::gene_bank::process_cloning_system,
                crate::layer1::clone_vat::process_clone_vats_system,
                crate::layer1::permit::permit_activation_system,
                crate::layer1::permit::enforce_permit_restrictions_system
                    .before(crate::layer1::energy::power_grid_system),
                crate::layer1::tech::update_tech_capacity_system,
                crate::layer1::tech::infinite_archive::update_efficiency_system
                    .after(crate::layer1::tech::update_tech_capacity_system),
                crate::layer1::admin::calculate_admin_stats,
                crate::layer1::tech::martyrs_engine::process_martyrs_engine,
                crate::layer1::eureka::handle_eureka_events,
                recycle_processing_system,
                #[cfg(feature = "nova")]
                crate::layer1::machine_consciousness::consciousness_growth_system,
            ),
        )
            .in_set(Layer1SystemSet::Economy),
    );

    schedule.add_systems(
        (
            process_research_system,
            process_observe_system,
            #[cfg(feature = "nova")]
            crate::layer1::constellations::observe_constellations_system,
            #[cfg(feature = "nova")]
            crate::layer1::void_signals::scan_for_signals_system,
            #[cfg(feature = "nova")]
            crate::layer1::void_signals::auto_tune_system,
            #[cfg(feature = "nova")]
            crate::layer1::void_signals::decrypt_signals_system,
            regrowth_system,
            crate::layer1::ecology::process_ecological_succession,
            flora_spread_system,
            mastery_accumulation_system,
            crate::layer1::cybernetics::surgery_system,
            crate::layer1::institutional_memory::produce_manual_system,
            crate::layer1::institutional_memory::manual_aura_system,
            crate::layer1::tech::ghost_code::residue_system,
            crate::layer1::tech::ghost_code::ghost_infection_system,
            crate::layer1::tech::ghost_code::apply_ghost_traits_system,
            crate::layer1::hologram::update_holograms_system,
            crate::layer1::hologram::apply_disillusionment_system,
        )
            .in_set(Layer1SystemSet::Economy),
    );

    schedule.add_systems(
        (
            restore_rest_in_housing_system.after(update_noise_system),
            restore_leisure_system,
            crate::layer1::tech_envy::tech_envy_system.after(restore_leisure_system),
            #[cfg(feature = "nova")]
            crate::layer1::loci::apply_loci_effects_system.after(restore_leisure_system),
            apply_mood_modifiers_system.after(restore_leisure_system),
            mascot_buff_system.after(restore_leisure_system),
            crate::layer1::graffiti::graffiti_observation_system.after(apply_mood_modifiers_system),
            apply_catharsis_morale_bonus_system.after(apply_mood_modifiers_system),
            update_morale_cache_system
                .after(apply_catharsis_morale_bonus_system)
                .after(crate::layer1::graffiti::graffiti_observation_system)
                .after(mascot_buff_system),
            morale_decay_system.after(update_morale_cache_system),
            crate::layer1::private_stash::stash_creation_system,
        )
            .in_set(Layer1SystemSet::Economy),
    );

    schedule.add_systems(
        (
            healing_system,
            crate::layer1::integration::medical_debt_bridge_system.after(healing_system),
            crate::layer1::beauty::update_beauty_grid_system,
            crate::layer1::window::update_window_views_system
                .after(crate::layer1::beauty::update_beauty_grid_system),
            crate::layer1::beauty::apply_beauty_effects_system
                .after(crate::layer1::window::update_window_views_system),
            check_heirloom_status_system,
            crate::layer1::trade::merchant_arrival_system,
            crate::layer1::contraband::enforce_prohibition_system,
            crate::layer1::visitor::spawn_visitor_system,
            spawn_inspector_system.after(crate::layer1::visitor::spawn_visitor_system),
            process_fuel_consumption_system,
            crate::layer1::energy::update_auroral_output_system,
            crate::layer1::solar::update_solar_cycle_system,
            crate::layer1::solar::update_solar_output_system
                .after(crate::layer1::solar::update_solar_cycle_system),
            crate::layer1::energy::power_grid_system
                .after(process_fuel_consumption_system)
                .after(crate::layer1::energy::update_auroral_output_system)
                .after(crate::layer1::solar::update_solar_output_system),
            crate::layer1::energy::load_limits::evaluate_grid_load_system
                .after(crate::layer1::energy::power_grid_system),
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
}
