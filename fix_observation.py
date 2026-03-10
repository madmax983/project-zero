with open('src/layer1/systems/observation.rs', 'r') as f:
    lines = f.readlines()

new_lines = []
for line in lines:
    if "crate::layer1::quantum_twins::handle_severance_system" in line:
        pass
    if ".after(crate::layer1::pop::handle_pop_death_system)," in line and "quantum_twins" in new_lines[-1]:
        new_lines.append(line)
        new_lines.append("            crate::layer1::ad_screen::update_ad_screens_system,\n")
        new_lines.append("            crate::layer1::integration::industrial_rhythm_morale_bridge,\n")
        new_lines.append("            crate::layer1::tech::posthumous_work_shift::posthumous_work_shift_system.before(crate::layer1::health::despawn_dead_entities_system),\n")
        new_lines.append("            crate::layer1::tech::posthumous_work_shift::digital_ghost_decay_system,\n")
        new_lines.append("            crate::layer1::tech::posthumous_work_shift::haunted_workplace_system,\n")
        new_lines.append("            crate::layer1::integration::great_work_chronicle_bridge,\n")
        continue
    if "crate::layer1::ad_screen::update_ad_screens_system" in line or "crate::layer1::integration::industrial_rhythm_morale_bridge" in line or "crate::layer1::integration::great_work_chronicle_bridge" in line:
        continue
    new_lines.append(line)

with open('src/layer1/systems/observation.rs', 'w') as f:
    f.writelines(new_lines)
