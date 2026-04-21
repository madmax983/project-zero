import re

with open("src/layer1/biology/rust_lung.rs", "r") as f:
    content = f.read()

# Fix the extra brace
content = content.replace("}\n}", "}")

# Fix test missing Pop
content = content.replace("use crate::layer1::economy::inventory::InventoryItem;\n", "use crate::layer1::economy::inventory::InventoryItem;\n    use crate::layer1::pop::Pop;\n")

with open("src/layer1/biology/rust_lung.rs", "w") as f:
    f.write(content)
