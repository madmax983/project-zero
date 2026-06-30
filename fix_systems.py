import re

with open("src/layer1/systems/environment.rs", "r") as f:
    content = f.read()

pattern = r"(            crate::layer1::geology::tectonic::apply_mega_quake_damage_system\n                 \.after\(crate::layer1::geology::tectonic::check_quake_system\),\n)(            spirit_decay_system,)"
replacement = r"\1        )\n            .in_set(Layer1SystemSet::Environment),\n    );\n\n    schedule.add_systems(\n        (\n\2"

new_content = re.sub(pattern, replacement, content)

with open("src/layer1/systems/environment.rs", "w") as f:
    f.write(new_content)
