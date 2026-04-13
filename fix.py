with open("src/layer1/flora.rs", "r") as f:
    content = f.read()

content = content.replace("use crate::layer1::map::GridPosition;\n    use crate::layer1::morale::Morale;", "use crate::layer1::morale::Morale;")
with open("src/layer1/flora.rs", "w") as f:
    f.write(content)
