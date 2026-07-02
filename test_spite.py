import re

with open("src/layer1/systems/observation.rs", "r") as f:
    print(f.read().find("fn register("))
