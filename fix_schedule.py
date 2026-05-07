import re

with open("src/layer1/systems/observation.rs", "r") as f:
    content = f.read()

# Fix the schedule bug, the tuple is too big in `register` of observation.rs.
# Bevy system tuple limit is 21. We have added too many.
# We need to split the `.in_set(Layer1SystemSet::Observation)` blocks.

# Replace the block that has society_suspicion_bridge_system and nostalgia systems
# by breaking the tuple.
content = content.replace("crate::layer1::culture::nostalgia::nostalgia_trigger_system,",
""")
            .in_set(Layer1SystemSet::Observation),
        (
            crate::layer1::culture::nostalgia::nostalgia_trigger_system,""")

with open("src/layer1/systems/observation.rs", "w") as f:
    f.write(content)
