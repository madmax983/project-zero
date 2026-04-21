import os
import re

mod_path = "tests/integration/mod.rs"

with open(mod_path, 'r') as f:
    text = f.read()

if "mod ecological_succession_bridge;" not in text:
    text += "\nmod ecological_succession_bridge;"
    with open(mod_path, 'w') as f:
        f.write(text)
