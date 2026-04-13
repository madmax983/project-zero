import re

with open("src/layer1/flora.rs", "r") as f:
    content = f.read()

content = re.sub(
    r'pub fn emit_flora_pheromones_system\(.*?\}\s*\}\s*\}\n+',
    '',
    content,
    flags=re.DOTALL
)

content = re.sub(
    r'pub fn apply_pheromone_mood_system\(.*?\}\s*\}\s*\}\s*\}\n+',
    '',
    content,
    flags=re.DOTALL
)

content = content.replace("apply_pheromone_mood_system, emit_flora_pheromones_system, PheromoneEmission,", "PheromoneEmission,")

content = re.sub(
    r'#\[test\]\s*fn test_flora_emits_calming_pheromones\(\)\s*\{.*?get_scent\(GridPosition \{ x: 5, y: 5 \}\)\.pleasant > 0\.0\);\s*\}\n*',
    '',
    content,
    flags=re.DOTALL
)

content = re.sub(
    r'#\[test\]\s*fn test_calming_pheromones_boost_morale\(\)\s*\{.*?label == "Calming Scent"\)\);\s*\}\n*',
    '',
    content,
    flags=re.DOTALL
)

content = re.sub(
    r'use crate::layer1::olfactory::ScentMap;\n',
    '',
    content
)

content = re.sub(
    r'use crate::layer1::flora::\{\n\s*PheromoneEmission,\n\s*PheromoneFlora,\n\s*\};\n\s*use crate::layer1::morale::Morale;\n\s*use bevy_app::App;\n',
    '',
    content,
    flags=re.DOTALL
)

with open("src/layer1/flora.rs", "w") as f:
    f.write(content)
