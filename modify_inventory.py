import re

with open("src/layer1/economy/inventory.rs", "r") as f:
    content = f.read()

has_item_func = """
    /// Checks if the inventory contains an item of the specified type.
    pub fn has_item(&self, item_type: ItemType) -> bool {
        self.items.iter().any(|item| item.item_type == item_type)
    }
"""

content = content.replace("        true\n    }\n}", "        true\n    }\n" + has_item_func + "}")

with open("src/layer1/economy/inventory.rs", "w") as f:
    f.write(content)
