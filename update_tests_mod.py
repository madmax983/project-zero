import re

content = ""
with open("tests/integration.rs", "r") as f:
    content = f.read()

to_add = """#[path = "integration/interplanetary_pollination.rs"]
mod interplanetary_pollination;
"""

content = content + "\n" + to_add

with open("tests/integration.rs", "w") as f:
    f.write(content)
