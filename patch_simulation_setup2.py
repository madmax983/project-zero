with open("src/simulation.rs", "r") as f:
    content = f.read()

import re

# Remove the incorrectly added line nested inside the if check
content = content.replace("        world.init_resource::<Events<crate::layer2::tourism::disaster_tourism::GriefTouristArrivalEvent>>();\n        world.init_resource::<Events<crate::layer1::social::ghost_shift_strike::GhostShiftStartedEvent>>();\n", "        world.init_resource::<Events<crate::layer2::tourism::disaster_tourism::GriefTouristArrivalEvent>>();\n")

with open("src/simulation.rs", "w") as f:
    f.write(content)
