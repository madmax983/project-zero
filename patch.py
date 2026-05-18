with open("src/layer1/systems/environment.rs", "r") as f:
    data = f.read()

system_to_add = """
    schedule.add_systems(
        (
            crate::layer1::tech::symbiotic_habitation::grow_hab_seed_system,
            crate::layer1::tech::symbiotic_habitation::process_symbiotic_pain_system,
        )
            .in_set(Layer1SystemSet::Environment),
    );
"""

# Let's add it near the end of register()
last_bracket_idx = data.rfind("}")
data = data[:last_bracket_idx] + system_to_add + data[last_bracket_idx:]

with open("src/layer1/systems/environment.rs", "w") as f:
    f.write(data)
