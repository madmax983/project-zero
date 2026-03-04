import os

with open("src/simulation.rs", "r") as f:
    content = f.read()

content = content.replace(
    """    // Initialize Thermal Bloom Resource
    if !world.contains_resource::<crate::layer2::thermal::ThermalSignature>() {
        world.init_resource::<crate::layer2::thermal::ThermalSignature>();
    }""",
    """    // Initialize Thermal Bloom Resource
    if !world.contains_resource::<crate::layer2::thermal::ThermalSignature>() {
        world.init_resource::<crate::layer2::thermal::ThermalSignature>();
    }

    // Initialize Detection Risk
    if !world.contains_resource::<DetectionRisk>() {
        world.init_resource::<DetectionRisk>();
    }"""
)

with open("src/simulation.rs", "w") as f:
    f.write(content)
