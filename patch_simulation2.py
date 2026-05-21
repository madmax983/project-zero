with open("src/simulation.rs", "r") as f:
    content = f.read()

import_statement = "use crate::layer2::piracy::{haven_to_republic_system, haven_upgrade_system, process_raid_success_system, RaidSuccessEvent};\n"
if "RaidSuccessEvent" not in content:
    content = import_statement + content

if "world.init_resource::<Events<RaidSuccessEvent>>();" not in content:
    content = content.replace("world.init_resource::<Events<crate::layer2::events::RoguePlanetEvent>>();", "world.init_resource::<Events<crate::layer2::events::RoguePlanetEvent>>();\n    world.init_resource::<Events<RaidSuccessEvent>>();")

if "process_raid_success_system" not in content:
    content = content.replace("crate::layer2::diaspora::handle_diaspora_arrival_system,", "crate::layer2::diaspora::handle_diaspora_arrival_system,\n        process_raid_success_system,\n        haven_upgrade_system,\n        haven_to_republic_system,")

with open("src/simulation.rs", "w") as f:
    f.write(content)
