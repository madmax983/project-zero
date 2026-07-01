import re

with open("src/layer1/systems/observation.rs", "r") as f:
    content = f.read()

# Let's count the number of elements in the tuple inside schedule.add_systems in lines 413-441
lines = content.split('\n')
for i, line in enumerate(lines[412:445]):
    print(f"{i+413}: {line}")
