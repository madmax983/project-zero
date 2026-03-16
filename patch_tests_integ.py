import re

with open("tests/integration/mod.rs", "r") as f:
    content = f.read()

if "pub mod martyrs_engine;" not in content:
    content += "\n#[cfg(test)]\npub mod martyrs_engine;\n"

with open("tests/integration/mod.rs", "w") as f:
    f.write(content)
