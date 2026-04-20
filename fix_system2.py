with open("src/layer1/systems/execution.rs", "r") as f:
    text = f.read()

import re
new_text = re.sub(
    r'\.chain\(\)\.in_set\(Layer1SystemSet::Social\),',
    '.chain().in_set(Layer1SystemSet::Observation),',
    text
)

with open("src/layer1/systems/execution.rs", "w") as f:
    f.write(new_text)
