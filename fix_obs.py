import re

with open("src/layer1/systems/observation.rs", "r") as f:
    content = f.read()

content = content.replace("            crate::layer1::integration::waste_scent_bridge,\n            crate::layer1::olfactory::scent_diffusion_system\n                .after(crate::layer1::integration::waste_scent_bridge),\n", "            crate::layer1::integration::waste_scent_bridge,\n            crate::layer1::core::integration::flora_scent_bridge_system,\n            crate::layer1::olfactory::scent_diffusion_system\n                .after(crate::layer1::integration::waste_scent_bridge)\n                .after(crate::layer1::core::integration::flora_scent_bridge_system),\n")

content = re.sub(
    r'\s*crate::layer1::flora::emit_flora_pheromones_system,\s*crate::layer1::flora::apply_pheromone_mood_system\s*\.after\(crate::layer1::flora::emit_flora_pheromones_system\),',
    '',
    content,
    flags=re.DOTALL
)

with open("src/layer1/systems/observation.rs", "w") as f:
    f.write(content)
