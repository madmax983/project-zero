import re

with open("src/layer1/systems/observation.rs", "r") as f:
    content = f.read()

content = content.replace("crate::layer1::integration::tether_snap_chronicle_bridge,", "crate::layer1::integration::tether_snap_chronicle_bridge,\n            crate::layer1::integration::famine_chronicle_bridge,\n            crate::layer1::integration::silent_flora_chronicle_bridge,\n            crate::layer1::integration::aesthetic_edict_chronicle_bridge,")

with open("src/layer1/systems/observation.rs", "w") as f:
    f.write(content)
