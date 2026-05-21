with open("src/simulation.rs", "r") as f:
    lines = f.readlines()

new_lines = []
for line in lines:
    if line.startswith("use crate::layer2::piracy::"):
        continue
    new_lines.append(line)

content = "".join(new_lines)
import_stmt = "use crate::layer2::piracy::{haven_to_republic_system, haven_upgrade_system, process_raid_success_system, RaidSuccessEvent};\n"
content = content.replace("use crate::layer2::events::RoguePlanetEvent;", "use crate::layer2::events::RoguePlanetEvent;\n" + import_stmt)

with open("src/simulation.rs", "w") as f:
    f.write(content)
