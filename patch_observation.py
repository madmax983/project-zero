with open('src/layer1/systems/observation.rs', 'r') as f:
    lines = f.readlines()

new_lines = []
for i, line in enumerate(lines):
    new_lines.append(line)
    # Insert right after paranoia_network_system in the nova block
    if "crate::experimental::paranoia_network::paranoia_network_system," in line:
        new_lines.append("            crate::experimental::architectural_palimpsest::detect_building_demolitions,\n")
        new_lines.append("            crate::experimental::architectural_palimpsest::apply_palimpsest_aura\n")
        new_lines.append("                .after(crate::experimental::architectural_palimpsest::detect_building_demolitions),\n")

with open('src/layer1/systems/observation.rs', 'w') as f:
    f.writelines(new_lines)
