import sys
with open("examples/minimal_nova_demo.rs", "r") as f:
    content = f.read()

# Fix unused imports
content = content.replace("use comfy_table::presets::UTF8_FULL;\n", "")
content = content.replace("use comfy_table::{Cell, Color as TableColor, Table};\n", "")

with open("examples/minimal_nova_demo.rs", "w") as f:
    f.write(content)
