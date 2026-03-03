import re

with open("src/layer1/systems/consumption.rs", "r") as f:
    consumption = f.read()

consumption = consumption.replace(
    "crate::layer1::chemical::addiction_system.after(decay_needs_system),",
    "crate::layer1::chemical::addiction_system.after(decay_needs_system),\n            crate::layer1::tech::hypno_learning::hypno_sleep_system.after(decay_needs_system),"
)

with open("src/layer1/systems/consumption.rs", "w") as f:
    f.write(consumption)

with open("src/layer1/systems/execution.rs", "r") as f:
    execution = f.read()

execution = execution.replace(
    "crate::layer1::chemical::apply_chemical_speed_modifiers_system\n                .after(apply_quirk_modifiers_system),",
    "crate::layer1::chemical::apply_chemical_speed_modifiers_system\n                .after(apply_quirk_modifiers_system),\n            crate::layer1::tech::hypno_learning::apply_mental_fog_penalties_system.after(crate::layer1::pop::reset_speed_system),"
)

execution = execution.replace(
    "crate::layer1::room_quality::apply_waking_thoughts_system\n                .after(crate::layer1::zone::apply_zone_designation_system),",
    "crate::layer1::room_quality::apply_waking_thoughts_system\n                .after(crate::layer1::zone::apply_zone_designation_system),\n            crate::layer1::tech::hypno_learning::wake_up_hypno_system.after(crate::layer1::zone::apply_zone_designation_system),\n            crate::layer1::tech::hypno_learning::update_mental_fog_system,"
)

with open("src/layer1/systems/execution.rs", "w") as f:
    f.write(execution)
