import sys

filepath = "src/layer1/physics/kinetic_strike.rs"
with open(filepath, "r") as f:
    content = f.read()

content = content.replace("mod tests {\n    use super::*;", "mod tests {\n    use super::*;\n    use bevy::prelude::{App, Update};")

with open(filepath, "w") as f:
    f.write(content)
