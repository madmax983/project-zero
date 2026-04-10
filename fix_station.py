import sys
with open('src/layer2/station.rs', 'r') as f:
    c = f.read()
c = c.replace("/// Component marking an entity as a space station.\n#[derive(Component, Debug, Clone)]\n\n/// Component for a Zero-G Brewery station.\n#[derive(Component, Debug, Clone)]\n\n#[derive(Component, Debug, Clone)]", "/// Component for a Zero-G Brewery station.\n#[derive(Component, Debug, Clone)]")
with open('src/layer2/station.rs', 'w') as f:
    f.write(c)
