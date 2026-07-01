import os
import re

for filename in os.listdir("specs"):
    filepath = os.path.join("specs", filename)
    if not os.path.isfile(filepath):
        continue
    with open(filepath, "r") as f:
        content = f.read()
        lines = content.split('\n')
        for i, line in enumerate(lines):
            if "Builder" in line and "*" in line and "add questions here" not in line.lower() and "add any questions here" not in line.lower():
                pass
