with open("src/layer1/systems/execution.rs", "r") as f:
    content = f.read()

import re

# Find conveyor_system
match = re.search(r'conveyor_system\.after\(haul_system\),', content)
if match:
    replacement = "            conveyor_system.after(haul_system),\n            crate::layer1::logistics::conveyor::inserter_system.after(conveyor_system),\n            crate::layer1::logistics::conveyor::hopper_system.after(crate::layer1::logistics::conveyor::inserter_system),"
    content = content[:match.start()] + replacement + content[match.end():]

with open("src/layer1/systems/execution.rs", "w") as f:
    f.write(content)
