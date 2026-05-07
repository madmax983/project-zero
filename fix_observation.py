import re
with open("src/layer1/systems/observation.rs", "r") as f:
    content = f.read()
target = "crate::layer1::core::integration::society_suspicion_bridge_system,"
replacement = target + """
            crate::layer1::culture::nostalgia::nostalgia_trigger_system,
            crate::layer1::core::integration::nostalgia_tavern_bridge_system,
            crate::layer1::culture::nostalgia::nostalgia_spread_system,
"""
content = content.replace(target, replacement)
with open("src/layer1/systems/observation.rs", "w") as f:
    f.write(content)
