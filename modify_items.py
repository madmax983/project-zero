import re

with open("src/layer1/economy/items.rs", "r") as f:
    content = f.read()

# Add Rebreather to ItemType enum
content = content.replace("    Waste,\n", "    Waste,\n    /// A mask that protects against dust and gas.\n    Rebreather,\n")

with open("src/layer1/economy/items.rs", "w") as f:
    f.write(content)
