import re

with open("tests/integration/quirks_atmosphere.rs", "r") as f:
    content = f.read()

insertion = """
    world.insert_resource(scale::layer1::atmosphere::DiffusionConfig::default());
"""

content = content.replace("world.insert_resource(AtmosphereGrid::new(10, 10));", "world.insert_resource(AtmosphereGrid::new(10, 10));" + insertion)

with open("tests/integration/quirks_atmosphere.rs", "w") as f:
    f.write(content)
