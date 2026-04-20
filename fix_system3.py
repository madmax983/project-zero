with open("src/layer1/systems/execution.rs", "r") as f:
    text = f.read()

import re
# Remove the custom xenoflora_systems function
text = re.sub(r'pub fn xenoflora_systems.*?\n}\n', '', text, flags=re.DOTALL)

with open("src/layer1/systems/execution.rs", "w") as f:
    f.write(text)

with open("src/simulation.rs", "r") as f:
    text = f.read()

text = re.sub(r'    crate::layer1::systems::xenoflora_systems\(app\);\n', '', text)

with open("src/simulation.rs", "w") as f:
    f.write(text)

with open("src/layer1/systems/mod.rs", "r") as f:
    text = f.read()

text = re.sub(r'pub use execution::xenoflora_systems;\n', '', text)

with open("src/layer1/systems/mod.rs", "w") as f:
    f.write(text)
