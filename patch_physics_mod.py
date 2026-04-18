import sys

filepath = "src/layer1/physics/mod.rs"
with open(filepath, "r") as f:
    content = f.read()

if "pub mod kinetic_strike;" not in content:
    content = content.replace("pub mod vent;", "pub mod kinetic_strike;\npub mod vent;")
    content = content.replace("pub use vent::*;", "pub use kinetic_strike::*;\npub use vent::*;")

with open(filepath, "w") as f:
    f.write(content)
