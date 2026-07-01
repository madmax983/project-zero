import re

with open("src/layer1/systems/observation.rs", "r") as f:
    content = f.read()

# The tuple in lines 413-448 contains exactly 22 items, which is over the Bevy limit of 21.
# We will split it into two `schedule.add_systems` calls.

old_block = """    schedule.add_systems(
        (
            crate::layer1::anomalies::cryptid::cryptid_chronicle_bridge_system,
            crate::layer1::integration::famine_tracking_system,
            crate::layer1::integration::trauma_decay_system,
            pop_death_chronicle_bridge.after(crate::layer1::health::despawn_dead_entities_system),
            mega_quake_chronicle_bridge.after(crate::layer1::geology::tectonic::check_quake_system),
            crate::layer1::core::integration::reformat_chronicle_bridge,
            crate::layer1::core::integration::tectonic_fracking_chronicle_bridge
                .after(crate::layer1::geology::fracking::tectonic_fracking_system),
            crate::layer1::integration::orbital_drop_chronicle_bridge
                .after(crate::layer1::logistics::orbital_drop::process_orbital_drops),
            crate::layer1::social::cargo_cult::apply_cargo_cult_belief_system
                .after(crate::layer1::logistics::orbital_drop::process_orbital_drops),
            crate::layer1::social::cargo_cult::process_ritual_actions_system
                .after(crate::layer1::social::cargo_cult::apply_cargo_cult_belief_system),
            crate::layer1::integration::mass_driver_chronicle_bridge
                .after(crate::layer1::logistics::mass_driver::package_arrival_system),
            crate::layer1::integration::predatory_weather_emission_bridge_system,
            crate::layer1::integration::predatory_weather_impact_bridge_system,
            crate::layer1::integration::hologram_failure_chronicle_bridge
                .after(crate::layer1::hologram::update_holograms_system),
            retrograde_chronicle_bridge.after(work_execution_system), // work_execution_system is in Execution set
            amputation_handler_system.after(work_execution_system),
            art_observation_system.after(crate::layer1::health::despawn_dead_entities_system),
            observe_inspector_system.after(art_observation_system),
            crate::layer1::integration::medical_treatment_notification_system.after(healing_system),
            crate::layer1::integration::hospitalization_notification_system
                .after(work_execution_system),
            (
                crate::layer1::core::integration::pop_died_count_system
                    .after(crate::layer1::pop::handle_pop_death_system),
                crate::layer1::core::integration::pop_born_count_system,
                crate::layer1::integration::pop_death_notification_system
                    .after(natural_death_system),
            ),
        )
            .in_set(Layer1SystemSet::Observation),
    );"""

new_block = """    schedule.add_systems(
        (
            crate::layer1::anomalies::cryptid::cryptid_chronicle_bridge_system,
            crate::layer1::integration::famine_tracking_system,
            crate::layer1::integration::trauma_decay_system,
            pop_death_chronicle_bridge.after(crate::layer1::health::despawn_dead_entities_system),
            mega_quake_chronicle_bridge.after(crate::layer1::geology::tectonic::check_quake_system),
            crate::layer1::core::integration::reformat_chronicle_bridge,
            crate::layer1::core::integration::tectonic_fracking_chronicle_bridge
                .after(crate::layer1::geology::fracking::tectonic_fracking_system),
            crate::layer1::integration::orbital_drop_chronicle_bridge
                .after(crate::layer1::logistics::orbital_drop::process_orbital_drops),
            crate::layer1::social::cargo_cult::apply_cargo_cult_belief_system
                .after(crate::layer1::logistics::orbital_drop::process_orbital_drops),
            crate::layer1::social::cargo_cult::process_ritual_actions_system
                .after(crate::layer1::social::cargo_cult::apply_cargo_cult_belief_system),
            crate::layer1::integration::mass_driver_chronicle_bridge
                .after(crate::layer1::logistics::mass_driver::package_arrival_system),
            crate::layer1::integration::predatory_weather_emission_bridge_system,
        )
            .in_set(Layer1SystemSet::Observation),
    );

    schedule.add_systems(
        (
            crate::layer1::integration::predatory_weather_impact_bridge_system,
            crate::layer1::integration::hologram_failure_chronicle_bridge
                .after(crate::layer1::hologram::update_holograms_system),
            retrograde_chronicle_bridge.after(work_execution_system), // work_execution_system is in Execution set
            amputation_handler_system.after(work_execution_system),
            art_observation_system.after(crate::layer1::health::despawn_dead_entities_system),
            observe_inspector_system.after(art_observation_system),
            crate::layer1::integration::medical_treatment_notification_system.after(healing_system),
            crate::layer1::integration::hospitalization_notification_system
                .after(work_execution_system),
            (
                crate::layer1::core::integration::pop_died_count_system
                    .after(crate::layer1::pop::handle_pop_death_system),
                crate::layer1::core::integration::pop_born_count_system,
                crate::layer1::integration::pop_death_notification_system
                    .after(natural_death_system),
            ),
        )
            .in_set(Layer1SystemSet::Observation),
    );"""

content = content.replace(old_block, new_block)
with open("src/layer1/systems/observation.rs", "w") as f:
    f.write(content)
