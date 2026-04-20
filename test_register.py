with open("src/layer1/systems/mod.rs", "r") as f:
    text = f.read()

import re
new_text = re.sub(r'(pub use execution::register_layer1_systems;)', r'pub use execution::xenoflora_systems;\n\1', text)

with open("src/layer1/systems/mod.rs", "w") as f:
    f.write(new_text)

with open("src/simulation.rs", "r") as f:
    text2 = f.read()

new_text2 = re.sub(r'(register_layer1_systems\(app\);)', r'\1\n    crate::layer1::systems::xenoflora_systems(app);', text2)

with open("src/simulation.rs", "w") as f:
    f.write(new_text2)
