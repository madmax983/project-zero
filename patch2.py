import re

with open("src/simulation.rs", "r") as f:
    content = f.read()

target = "world.init_resource::<Events<crate::layer3::planet::black_market_terraforming::RogueTerraformEvent>>();"
replacement = """world.init_resource::<Events<crate::layer3::planet::black_market_terraforming::RogueTerraformEvent>>();
        world.init_resource::<Events<crate::layer1::economy::black_market::SmugglerArrivalEvent>>();
        world.init_resource::<Events<crate::layer1::economy::black_market::ShutdownDropNodeEvent>>();"""

if target in content:
    content = content.replace(target, replacement)
    with open("src/simulation.rs", "w") as f:
        f.write(content)
    print("Patched successfully")
else:
    print("Target not found")
