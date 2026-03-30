with open("src/simulation.rs", "r") as f:
    content = f.read()

import re

# Add the event initialization to setup_world logic which is what `test_schedule_runs_on_fresh_world` uses.
new_text = "        world.init_resource::<Events<crate::layer1::social::ghost_shift_strike::GhostShiftStartedEvent>>();\n"

content = content.replace("        world.init_resource::<Events<crate::layer1::disasters::DisasterEvent>>();\n", "        world.init_resource::<Events<crate::layer1::disasters::DisasterEvent>>();\n" + new_text)

with open("src/simulation.rs", "w") as f:
    f.write(content)
