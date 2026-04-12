import re

with open("src/layer1/systems/observation.rs", "r") as f:
    content = f.read()

replacement = """
    schedule.add_systems(
        (
            crate::layer1::integration::famine_chronicle_bridge,
            crate::layer1::integration::silent_flora_chronicle_bridge,
            crate::layer1::integration::aesthetic_edict_chronicle_bridge,
            crate::layer1::integration::smuggler_arrival_event_bridge
                .before(crate::layer1::void_weed::process_void_weed_trade_system),
        )
            .in_set(Layer1SystemSet::Observation),
    );
"""

# Replace the overly long tuple with two split ones.
content = content.replace("            crate::layer1::integration::tether_snap_chronicle_bridge,\n            crate::layer1::integration::famine_chronicle_bridge,\n            crate::layer1::integration::silent_flora_chronicle_bridge,\n            crate::layer1::integration::aesthetic_edict_chronicle_bridge,\n            crate::layer1::integration::smuggler_arrival_event_bridge\n                .before(crate::layer1::void_weed::process_void_weed_trade_system),\n        )\n            .in_set(Layer1SystemSet::Observation),\n    );", "            crate::layer1::integration::tether_snap_chronicle_bridge,\n        )\n            .in_set(Layer1SystemSet::Observation),\n    );" + replacement)

with open("src/layer1/systems/observation.rs", "w") as f:
    f.write(content)
