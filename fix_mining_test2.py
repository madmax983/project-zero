import re

with open("src/layer1/execution/mining.rs", "r") as f:
    content = f.read()

content = content.replace("let map = PurityMap::new(1); // Assuming 1 seed gives low purity or we can just mock it if needed", "let mut map = PurityMap::new(1);\n        map.set_override(5, 5, 0.1); // Low purity")

with open("src/layer1/execution/mining.rs", "w") as f:
    f.write(content)
