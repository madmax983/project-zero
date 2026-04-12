import re

with open("src/simulation.rs", "r") as f:
    content = f.read()

# Add the new system scheduling in its own `add_systems` call right after the `in_set(Layer2SystemSet),` block around line 144
insertion = """
    schedule.add_systems(
        crate::layer2::integration::derelict_station_chronicle_bridge
            .after(crate::layer2::derelict_stations::claim_station_system)
            .in_set(Layer2SystemSet),
    );
"""

# Find a good place to insert this
content = content.replace("        )\n            .in_set(Layer2SystemSet),\n    );", "        )\n            .in_set(Layer2SystemSet),\n    );" + insertion)

with open("src/simulation.rs", "w") as f:
    f.write(content)
