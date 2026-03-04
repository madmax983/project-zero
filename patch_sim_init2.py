import os

with open("src/simulation.rs", "r") as f:
    content = f.read()

content = content.replace(
    """    // Add our schedule if not yet added""",
    """    if !world.contains_resource::<Events<HostileSpawnEvent>>() {
        world.init_resource::<Events<HostileSpawnEvent>>();
    }

    // Add our schedule if not yet added"""
)

with open("src/simulation.rs", "w") as f:
    f.write(content)
