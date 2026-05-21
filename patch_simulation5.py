with open("src/simulation.rs", "r") as f:
    content = f.read()

content = content.replace("\nuse crate::layer2::piracy::{haven_to_republic_system, haven_upgrade_system, process_raid_success_system, RaidSuccessEvent};\n", "")

import_stmt = "use crate::layer2::piracy::{haven_to_republic_system, haven_upgrade_system, process_raid_success_system, RaidSuccessEvent};\n"
if "use crate::layer2::events::RoguePlanetEvent;" in content:
    content = content.replace("use crate::layer2::events::RoguePlanetEvent;", import_stmt + "use crate::layer2::events::RoguePlanetEvent;")

with open("src/simulation.rs", "w") as f:
    f.write(content)
