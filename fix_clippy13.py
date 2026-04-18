import sys

filepath = "src/layer1/systems/cleanup.rs"
with open(filepath, "r") as f:
    content = f.read()

content = content.replace("update_event_buffer::<crate::layer1::economy::remittances::MigrantArrivalEvent>,\n            handle_direct_input_system,", "update_event_buffer::<crate::layer1::economy::remittances::MigrantArrivalEvent>,\n        )\n            .in_set(Layer1SystemSet::EventCleanup),\n    );\n    schedule.add_systems(\n        (\n            handle_direct_input_system,")

with open(filepath, "w") as f:
    f.write(content)

filepath = "src/layer1/physics/kinetic_strike.rs"
with open(filepath, "r") as f:
    content = f.read()

content = content.replace("use bevy::prelude::*;\n", "")

with open(filepath, "w") as f:
    f.write(content)
