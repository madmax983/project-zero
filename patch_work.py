with open("src/layer1/execution/general_work.rs", "r") as f:
    lines = f.readlines()

new_lines = []
for line in lines:
    if "let gravity_nightmare_modifier = if world" in line:
        new_lines.append("""    let propaganda_graffiti_modifier = {
        let pop_pos = world.get::<GridPosition>(pop_entity);
        let mut modifier = 1.0;
        if let Some(pos) = pop_pos {
            if let Some(map) = world.get_resource::<crate::layer1::graffiti::GraffitiMap>() {
                let neighbors = [
                    (pos.x, pos.y), // Check same tile
                    (pos.x, pos.y - 1),
                    (pos.x + 1, pos.y),
                    (pos.x, pos.y + 1),
                    (pos.x - 1, pos.y),
                ];
                for target in neighbors {
                    if let Some(graffiti) = map.markings.get(&target) {
                        if graffiti.graffiti_type == crate::layer1::graffiti::GraffitiType::Propaganda {
                            modifier = 0.9;
                            break;
                        }
                    }
                }
            }
        }
        modifier
    };

""")
        new_lines.append(line)
    elif "* gravity_nightmare_modifier;" in line:
        new_lines.append("        * gravity_nightmare_modifier\n")
        new_lines.append("        * propaganda_graffiti_modifier;\n")
    else:
        new_lines.append(line)

with open("src/layer1/execution/general_work.rs", "w") as f:
    f.writelines(new_lines)
