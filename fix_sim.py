with open("src/simulation.rs", "r") as f:
    text = f.read()

lines = text.split("\n")
out = []
for line in lines:
    if "world.init_resource::<Events<crate::layer3::fleets::ColonyFoundedEvent>>();" in line:
        out.append(line)
        out.append("        world.init_resource::<crate::layer1::diplomacy::factions::rivals::TerritoryGrid>();")
    else:
        out.append(line)

with open("src/simulation.rs", "w") as f:
    f.write("\n".join(out))
