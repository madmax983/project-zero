use super::Layer1SystemSet;
use crate::layer1::blob::{blob_consumption_system, blob_spread_system};
use crate::layer1::*;
use bevy_ecs::prelude::*;

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(
        (
            crate::layer1::shadow_market::despawn_in_light_system,
            crate::layer1::shadow_market::spawn_shadow_trader_system,
        )
            .in_set(Layer1SystemSet::Environment),
    );
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
            crate::layer1::orbital_crossfire::impact_system,
            crate::layer1::volatile::volatile_decay_system,
            crate::layer1::volatile::handle_explosion_system
                .after(crate::layer1::volatile::volatile_decay_system),
            crate::layer1::logistics::pneumatic::tube_clog_system,
            crate::layer1::ecology::biome_collapse_system,
            crate::layer1::social::grievances::decay_notes_system,
            crate::layer1::hum::update_hum_system,
            crate::layer1::photophobic::photophobic_decay_system,
            crate::layer1::geodetic::update_living_stone_system,
            crate::layer1::geodetic::form_golem_system
                .after(crate::layer1::geodetic::update_living_stone_system),
        )
            .in_set(Layer1SystemSet::Environment),
    );

    schedule.add_systems(
        (crate::layer1::clutter::clutter_accumulation_system,).in_set(Layer1SystemSet::Environment),
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
            crate::layer1::geology::tectonic::update_stress_system,
            crate::layer1::geology::tectonic::check_quake_system
                .after(crate::layer1::geology::tectonic::update_stress_system),
            spirit_decay_system,
            quirk_generation_system.after(spirit_decay_system),
            entropy_system,
            crate::layer1::structure::fragile_decay_system.after(entropy_system),
            crate::layer1::crowding::crowding_decay_system,
            #[cfg(feature = "nova")]
            crate::layer1::loci::update_loci_system,
            crate::layer1::social::empty_room::update_sanctuary_system,
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
            crate::layer1::atmosphere::corrosion_damage_system
                .after(crate::layer1::atmosphere::update_atmosphere_system),
            crate::layer1::terraforming::update_planetary_atmosphere_system
                .after(crate::layer1::atmosphere::update_atmosphere_system),
            crate::layer1::atmosphere::update_weather_diffusion_system
                .after(crate::layer1::terraforming::update_planetary_atmosphere_system),
            crate::layer1::terraforming::apply_planetary_effects_system
                .after(crate::layer1::atmosphere::update_weather_diffusion_system),
            crate::layer1::atmosphere::simulate_diffusion_system
                .after(crate::layer1::terraforming::apply_planetary_effects_system),
            crate::layer1::logistics::orbital_drop::process_orbital_drops,
        )
            .in_set(Layer1SystemSet::Environment),
    );

    schedule.add_systems(
        (
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
            crate::layer1::light_pollution::calculate_sky_glow_system,
            crate::layer1::light_pollution::apply_light_pollution_system
                .after(crate::layer1::light_pollution::calculate_sky_glow_system),
        )
            .in_set(Layer1SystemSet::Environment),
    );
}
