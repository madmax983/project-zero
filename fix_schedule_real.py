import re

with open("src/layer1/systems/observation.rs", "r") as f:
    content = f.read()

# Fix the method signature:
# Instead of `schedule.add_systems((sys1, sys2).in_set(), (sys3, sys4).in_set())`
# we need `schedule.add_systems((sys1, sys2).in_set()); schedule.add_systems((sys3, sys4).in_set());`

content = content.replace(""")
            .in_set(Layer1SystemSet::Observation),
        (
            crate::layer1::culture::nostalgia::nostalgia_trigger_system,""",
""")
            .in_set(Layer1SystemSet::Observation));

        schedule.add_systems(
        (
            crate::layer1::culture::nostalgia::nostalgia_trigger_system,""")

with open("src/layer1/systems/observation.rs", "w") as f:
    f.write(content)
