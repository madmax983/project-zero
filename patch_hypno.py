import re

with open("src/layer1/tech/hypno_learning.rs", "r") as f:
    code = f.read()

code = code.replace(
    "use crate::layer1::utility_types::AssignedTo;",
    "use crate::layer1::actions::AssignedTo;"
)

code = code.replace(
    "skills.add_xp(skill_type, xp_rate);",
    "Skills::add_xp(&mut skills, skill_type, xp_rate);"
)

code = code.replace(
    "#[derive(Event, Default)]",
    "#[derive(Event)]"
)

code = code.replace(
    "PowerConsumer { active: true, amount: 10.0 }",
    "PowerConsumer { active: true, demand: 10.0 }"
)

code = code.replace(
    "PowerConsumer { active: false, amount: 10.0 }",
    "PowerConsumer { active: false, demand: 10.0 }"
)

with open("src/layer1/tech/hypno_learning.rs", "w") as f:
    f.write(code)
