import sys

cleanup_path = "src/layer1/systems/cleanup.rs"
with open(cleanup_path, "r") as f:
    content = f.read()

if "KineticStrikeEvent" not in content:
    content = content.replace(
        "update_event_buffer::<crate::layer1::logistics::orbital_drop::OrbitalDropEvent>,",
        "update_event_buffer::<crate::layer1::logistics::orbital_drop::OrbitalDropEvent>,\n            update_event_buffer::<crate::layer1::physics::kinetic_strike::KineticStrikeEvent>,"
    )
    with open(cleanup_path, "w") as f:
        f.write(content)

setup_path = "src/setup.rs"
with open(setup_path, "r") as f:
    content = f.read()

if "KineticStrikeEvent" not in content:
    content = content.replace(
        "world.init_resource::<Events<crate::layer1::logistics::orbital_drop::OrbitalDropEvent>>();",
        "world.init_resource::<Events<crate::layer1::logistics::orbital_drop::OrbitalDropEvent>>();\n    world.init_resource::<Events<crate::layer1::physics::kinetic_strike::KineticStrikeEvent>>();"
    )
    with open(setup_path, "w") as f:
        f.write(content)
