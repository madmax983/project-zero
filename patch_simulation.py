import re

with open('src/simulation.rs', 'r') as f:
    content = f.read()

# Register the system
target = "schedule.add_systems((crate::layer3::diplomacy::succession::process_succession_system,));"
replacement = """schedule.add_systems((
        crate::layer3::diplomacy::succession::process_succession_system,
        crate::layer3::integration::dynastic_succession_chronicle_bridge
            .after(crate::layer3::diplomacy::succession::process_succession_system),
    ));"""

content = content.replace(target, replacement)

with open('src/simulation.rs', 'w') as f:
    f.write(content)

print("Patched src/simulation.rs")
