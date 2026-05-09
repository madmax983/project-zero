use super::Layer1SystemSet;
use crate::layer1::inspector::spawn_inspector_system;
use crate::layer1::social::old_guard::{
    apply_founder_benefits_system, apply_mood_modifiers_system,
};
use crate::layer1::*;
use bevy_ecs::prelude::*;

#[allow(clippy::too_many_lines)]
pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(
        (
            update_resource_caps_system,
            crate::layer1::digital_immortality::ghost_power_consumption,
            advance_season_system,
            crate::layer1::core::integration::predecessor_weather_array_bridge_system
                .after(advance_season_system),
            update_taboo_duration_system,
            update_water_system,
            update_weather_system,
            crate::layer1::fertility::update_fertility_system,
            produce_food_system.after(crate::layer1::fertility::update_fertility_system),
            crate::layer1::husbandry::husbandry_production_system.after(produce_food_system),
            (
                hopper_system.after(produce_food_system),
                process_refining_system,
                crate::layer1::tech::rhythm::update_rhythm_system.after(process_refining_system),
                crate::layer1::social::cadet::process_allowance_system,
                crate::layer2::designation::evaluate_designation_bonuses,
                crate::layer2::designation::handle_designation_changes,
                (
                    crate::layer1::gene_bank::process_cloning_system,
                    crate::layer1::genetics::process_gene_splicing_system,
                    crate::layer1::genetics::apply_crop_traits,
                    crate::layer1::genetics::process_mutations,
                ),
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
                crate::layer1::exodus::cannibalize_infrastructure_system,
                crate::layer1::exodus::build_ark_system
                    .after(crate::layer1::exodus::cannibalize_infrastructure_system),
                #[cfg(feature = "nova")]
                crate::layer1::machine_consciousness::consciousness_growth_system,
            ),
        )
            .in_set(Layer1SystemSet::Economy),
    );

    schedule.add_systems(
        (crate::layer1::energy::sky_tether::auroral_harvesting_system,)
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
            restore_rest_in_housing_system,
            restore_leisure_system,
            crate::layer1::economy::apex_diet::apply_apex_mutations,
            crate::layer1::economy::apex_diet::process_apex_meat_consumption,
            crate::layer1::tech_envy::tech_envy_system.after(restore_leisure_system),
            #[cfg(feature = "nova")]
            crate::layer1::loci::apply_loci_effects_system.after(restore_leisure_system),
            apply_mood_modifiers_system.after(restore_leisure_system),
            mascot_buff_system.after(restore_leisure_system),
            crate::layer1::graffiti::graffiti_observation_system.after(apply_mood_modifiers_system),
            crate::layer1::memetics::parasitic_broadcast_risk_system,
            crate::layer1::memetics::process_parasitic_work_reduction,
            apply_catharsis_morale_bonus_system.after(apply_mood_modifiers_system),
            crate::layer1::law::penal::apply_harvesting_horror_system
                .after(apply_mood_modifiers_system),
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
        (crate::layer1::law::penal::process_dead_pops_for_organs_system,)
            .in_set(Layer1SystemSet::Economy),
    );
    schedule.add_systems(
        (
            crate::layer1::economy::remittances::process_remittances_system,
            healing_system,
            crate::layer1::integration::medical_debt_bridge_system.after(healing_system),
            crate::layer1::beauty::update_beauty_grid_system,
            crate::layer1::window::update_window_views_system
                .after(crate::layer1::beauty::update_beauty_grid_system),
            crate::layer1::beauty::apply_beauty_effects_system
                .after(crate::layer1::window::update_window_views_system),
            check_heirloom_status_system,
            crate::layer1::trade::merchant_arrival_system,
            crate::layer1::law::contraband::enforce_prohibition_system,
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
            crate::layer3::market::quantum_famine::process_market_panic_hoarding,
        )
            .in_set(Layer1SystemSet::Economy),
    );

    schedule.add_systems(
        (
            crate::layer1::economy::inflation::trigger_market_crash,
            crate::layer1::economy::inflation::process_barter_trade,
        )
            .in_set(Layer1SystemSet::Economy),
    );

    schedule.add_systems(
        (
            crate::layer1::economy::beacon::process_colony_beacon_system,
            crate::layer1::economy::beacon::toggle_beacon_system,
            crate::layer1::integration::beacon_migrant_arrival_bridge,
            crate::layer1::integration::beacon_trade_ship_bridge,
            crate::layer1::integration::beacon_pirate_raid_bridge,
            crate::layer1::economy::smugglers_cove::spawn_smugglers_cove_system,
            crate::layer1::economy::smugglers_cove::process_smuggler_decay_system,
        )
            .in_set(Layer1SystemSet::Economy),
    );

    schedule.add_systems(
        (
            crate::layer1::integration::grid_overload_fire_bridge
                .after(crate::layer1::energy::power_grid_system),
            art_generation_system,
            crate::layer1::petrification::petrification_exposure_system,
            crate::layer1::petrification::petrification_progression_system,
            crate::layer1::petrification::petrification_transformation_system,
            crate::layer1::factions::update_faction_membership_system,
            crate::layer1::factions::update_faction_satisfaction_system
                .after(crate::layer1::factions::update_faction_membership_system),
            crate::layer1::factions::update_faction_demands_system
                .after(crate::layer1::factions::update_faction_satisfaction_system),
            crate::layer1::factions::update_faction_strikes_system
                .after(crate::layer1::factions::update_faction_demands_system),
            crate::layer1::core::integration::faction_strike_mob_bridge_system
                .after(crate::layer1::factions::update_faction_strikes_system),
            crate::layer1::social::sub_lithic::process_deep_mining_exposure,
            crate::layer1::social::sub_lithic::evaluate_cult_formation,
            crate::layer1::social::sub_lithic::process_cult_sabotage,
            apply_founder_benefits_system,
            // Layer 2 visibility systems are handled in simulation.rs
        )
            .in_set(Layer1SystemSet::Economy),
    );
    schedule.add_systems(
        (
            crate::layer1::energy::phantom_grid::phantom_grid_disconnection_system
                .before(crate::layer1::energy::phantom_grid::phantom_grid_connection_system),
            crate::layer1::energy::phantom_grid::phantom_grid_connection_system
                .after(crate::layer1::energy::power_grid_system),
        )
            .in_set(Layer1SystemSet::Economy),
    );
    schedule.add_systems(
        (
            crate::layer1::environment::ephemeral_moons::apply_moon_modifiers_system
                .after(crate::layer1::solar::update_solar_output_system)
                .before(crate::layer1::energy::power_grid_system),
        )
            .in_set(Layer1SystemSet::Economy),
    );

    schedule.add_systems(
        (
            crate::layer1::economy::photophobic::update_photophobic_light_level_system,
            crate::layer1::economy::photophobic::photophobic_degradation_system,
            crate::layer1::economy::photophobic::mining_in_dark_stress_system,
        )
            .chain()
            .in_set(Layer1SystemSet::Economy),
    );
}
