import re

with open("src/layer1/execution/mining.rs", "r") as f:
    content = f.read()

content = content.replace("use crate::layer1::economy::inventory::{Inventory, InventoryItem};\n    use crate::layer1::items::ItemType;\n", "use crate::layer1::economy::inventory::Inventory;\n")
content = content.replace("let mut map = PurityMap::new(10, 10);\n        map.set(5, 5, 0.1); // Low purity", "let mut map = PurityMap::new(1);\n        map.set_override(5, 5, 0.1); // Low purity")

with open("src/layer1/execution/mining.rs", "w") as f:
    f.write(content)
