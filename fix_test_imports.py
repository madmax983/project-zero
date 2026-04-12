import re

with open("src/layer1/nature/silent_world_tests.rs", "r") as f:
    content = f.read()

content = content.replace("use bevy_ecs::prelude::*;\n", "")
content = content.replace("use bevy_ecs::system::RunSystemOnce;\n", "")

with open("src/layer1/nature/silent_world_tests.rs", "w") as f:
    f.write(content)
