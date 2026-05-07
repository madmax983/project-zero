import re
with open("src/setup.rs", "r") as f:
    content = f.read()
target = "world.init_resource::<Events<PopBorn>>();"
replacement = target + "\n    world.init_resource::<Events<crate::layer1::culture::nostalgia::RumorSpreadEvent>>();"
content = content.replace(target, replacement)
with open("src/setup.rs", "w") as f:
    f.write(content)
