with open("src/layer1/systems/execution.rs", "r") as f:
    content = f.read()

import re

# Split the large add_systems call into multiple to avoid Bevy tuple limits
pattern = r'(crate::layer1::tinkering::obsessive_optimization_system.*?)(crate::layer1::social::empty_room::visit_sanctuary_system\.after\(movement_system\),)'
match = re.search(pattern, content, re.DOTALL)
if match:
    block = match.group(0)
    lines = block.split('\n')

    first_half = "\n".join(lines[:10])
    second_half = "\n".join(lines[10:])

    replacement = first_half + "\n        )\n            .in_set(Layer1SystemSet::Execution),\n    );\n\n    schedule.add_systems(\n        (\n" + second_half

    content = content[:match.start()] + replacement + content[match.end():]

with open("src/layer1/systems/execution.rs", "w") as f:
    f.write(content)
