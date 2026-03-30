with open("src/simulation.rs", "r") as f:
    content = f.read()

import re

new_text = "        world.init_resource::<Events<crate::layer1::social::ghost_shift_strike::GhostShiftStartedEvent>>();\n"

content = content.replace("        world.init_resource::<Events<crate::layer2::tourism::disaster_tourism::GriefTouristArrivalEvent>>();\n", "        world.init_resource::<Events<crate::layer2::tourism::disaster_tourism::GriefTouristArrivalEvent>>();\n" + new_text)

with open("src/simulation.rs", "w") as f:
    f.write(content)
