import re

with open("src/layer1/biology/rust_lung.rs", "r") as f:
    content = f.read()

# Fix app.world syntax and unused imports
content = content.replace("app.world.spawn", "app.world_mut().spawn")
content = content.replace("&mut app.world,", "&mut app.world_mut(),")
content = content.replace("app.world.get", "app.world().get")
content = content.replace("use crate::layer1::economy::inventory::InventoryItem;\n", "")

with open("src/layer1/biology/rust_lung.rs", "w") as f:
    f.write(content)

