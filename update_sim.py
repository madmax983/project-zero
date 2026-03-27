import re

with open('src/layer1/systems/observation.rs', 'r') as f:
    content = f.read()

replacement = """            crate::layer1::integration::diplomatic_reflection_kill_bridge,
            crate::layer1::integration::diplomatic_reflection_plant_bridge,
            crate::layer1::integration::waste_scent_bridge,"""

content = content.replace("            crate::layer1::integration::waste_scent_bridge,", replacement)

with open('src/layer1/systems/observation.rs', 'w') as f:
    f.write(content)
