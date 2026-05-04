with open("src/layer1/systems/observation.rs", "r") as f:
    lines = f.readlines()

for i, line in enumerate(lines):
    if "crate::layer1::core::integration::digital_immortality_chronicle_bridge," in line:
        lines.insert(i, "            crate::layer1::core::integration::sub_lithic_sabotage_bridge,\n")
        break

with open("src/layer1/systems/observation.rs", "w") as f:
    f.writelines(lines)
