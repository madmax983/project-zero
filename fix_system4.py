with open("src/layer1/systems/observation.rs", "r") as f:
    text = f.read()

# We will add our systems to the register function in observation.rs
add_systems_text = """
    schedule.add_systems(
        (
            crate::layer1::social::xenoflora_pet::pet_resource_consumption_system,
            crate::layer1::social::xenoflora_pet::pet_viral_spread_system,
            crate::layer1::social::xenoflora_pet::apply_pet_mood_boost,
        )
            .chain()
            .in_set(Layer1SystemSet::Observation),
    );
"""

text = text.replace("pub fn register(schedule: &mut Schedule) {", "pub fn register(schedule: &mut Schedule) {" + add_systems_text)

with open("src/layer1/systems/observation.rs", "w") as f:
    f.write(text)
