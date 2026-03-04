import os

with open("src/simulation.rs", "r") as f:
    content = f.read()

content = content.replace(
    """        // Initialize Detection Risk for test
        world.init_resource::<crate::layer3::silence::DetectionRisk>();""",
    """        // Initialize Detection Risk for test
        world.init_resource::<crate::layer3::silence::DetectionRisk>();
        world.init_resource::<Events<crate::layer3::silence::HostileSpawnEvent>>();"""
)

with open("src/simulation.rs", "w") as f:
    f.write(content)
