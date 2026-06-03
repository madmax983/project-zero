with open("src/layer1/graffiti.rs", "r") as f:
    lines = f.readlines()

new_lines = []
for line in lines:
    if "pops: Query<(&GridPosition, &Morale), With<crate::layer1::pop::Pop>>," in line:
        new_lines.append("    pops: Query<(&GridPosition, &Morale, Option<&crate::layer1::psychology::stress::StressTracker>, Option<&crate::layer1::traits::Traits>), With<crate::layer1::pop::Pop>>,\n")
    elif "for (pos, morale) in &pops {" in line:
        new_lines.append("    for (pos, morale, stress, traits) in &pops {\n")
    elif "let graffiti_type = if morale.value < 0.2 {" in line:
        new_lines.append("""        let graffiti_type = if stress.map_or(0.0, |s| s.accumulated_stress) > 80.0 && traits.map_or(false, |t| t.has(crate::layer1::traits::Trait::Creative)) {
            GraffitiType::Propaganda
        } else if morale.value < 0.2 {
""")
    elif "GraffitiType::Inspiration => (1000.0, 0.05)," in line:
        new_lines.append("                    GraffitiType::Inspiration => (1000.0, 0.05),\n")
        new_lines.append("                    GraffitiType::Propaganda => (1000.0, 0.05),\n")
    else:
        new_lines.append(line)

with open("src/layer1/graffiti.rs", "w") as f:
    f.writelines(new_lines)
