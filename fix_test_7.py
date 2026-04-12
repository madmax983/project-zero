import re

with open("tests/integration/quirks_atmosphere.rs", "r") as f:
    content = f.read()

content = content.replace("world_normal.resource_mut::<scale::layer1::atmosphere::AtmosphereGrid>().diffuse(&bevy_utils::HashMap::new(), 0.1);", "let grid = world_normal.resource_mut::<scale::layer1::atmosphere::AtmosphereGrid>(); grid.clone().diffuse(&bevy_utils::HashMap::new(), 0.1);")

with open("tests/integration/quirks_atmosphere.rs", "w") as f:
    f.write(content)
