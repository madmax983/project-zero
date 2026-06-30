import sys
with open("examples/test_readme_nova.rs", "r") as f:
    content = f.read()

content = content.replace("use comfy_table::presets::UTF8_FULL;\n", "")
content = content.replace("use comfy_table::{Cell, Color as TableColor, Table};\n", "")
content = content.replace("use crossterm::style::{Color, Stylize};\n", "")

with open("examples/test_readme_nova.rs", "w") as f:
    f.write(content)
