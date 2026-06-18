with open("src/layer1/fauna/mod.rs", "r") as f:
    content = f.read()

content = content.replace("use crate::layer1::particles::spawn_particle;\n", "")

with open("src/layer1/fauna/mod.rs", "w") as f:
    f.write(content)
