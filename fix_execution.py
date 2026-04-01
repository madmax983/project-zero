with open("src/layer1/systems/execution.rs", "r") as f:
    content = f.read()

# Find line 75 and split the tuple
old_chunk = """
            crate::layer1::nature::mutagenic_rain::clear_mutation_immunity_system
                .after(crate::layer1::nature::mutagenic_rain::apply_mutagenic_rain_system),
            apply_quirk_modifiers_system
                .after(apply_lighting_penalties_system)
                .after(apply_weather_effects_system),
            crate::layer1::chemical::apply_chemical_speed_modifiers_system,
            crate::layer1::cryo_shock::apply_cryo_debuff_system
                .after(crate::layer1::pop::reset_speed_system)
                .before(crate::layer1::chemical::apply_chemical_speed_modifiers_system)
                .after(apply_quirk_modifiers_system),
            #[cfg(feature = "nova")]
            crate::layer1::observer::observer_reaction_system
                .after(apply_lighting_penalties_system),
            #[cfg(feature = "nova")]
            crate::layer1::machine_consciousness::consciousness_effect_system
                .after(crate::layer1::pop::reset_speed_system),
            crate::layer1::combat::hit_stop_system.after(process_start_plan_system),
            crate::layer1::combat::combat_cooldown_system.after(process_start_plan_system),
        )
            .in_set(Layer1SystemSet::Execution),
    );
"""

new_chunk = """
            crate::layer1::nature::mutagenic_rain::clear_mutation_immunity_system
                .after(crate::layer1::nature::mutagenic_rain::apply_mutagenic_rain_system),
        )
            .in_set(Layer1SystemSet::Execution),
    );

    schedule.add_systems(
        (
            apply_quirk_modifiers_system
                .after(apply_lighting_penalties_system)
                .after(apply_weather_effects_system),
            crate::layer1::chemical::apply_chemical_speed_modifiers_system,
            crate::layer1::cryo_shock::apply_cryo_debuff_system
                .after(crate::layer1::pop::reset_speed_system)
                .before(crate::layer1::chemical::apply_chemical_speed_modifiers_system)
                .after(apply_quirk_modifiers_system),
            #[cfg(feature = "nova")]
            crate::layer1::observer::observer_reaction_system
                .after(apply_lighting_penalties_system),
            #[cfg(feature = "nova")]
            crate::layer1::machine_consciousness::consciousness_effect_system
                .after(crate::layer1::pop::reset_speed_system),
            crate::layer1::combat::hit_stop_system.after(process_start_plan_system),
            crate::layer1::combat::combat_cooldown_system.after(process_start_plan_system),
        )
            .in_set(Layer1SystemSet::Execution),
    );
"""

content = content.replace(old_chunk, new_chunk)
with open("src/layer1/systems/execution.rs", "w") as f:
    f.write(content)
