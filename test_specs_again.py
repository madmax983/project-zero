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
            if re.match(r".*\*Builder:.*", line, flags=re.IGNORECASE):
                if "add questions here" not in line.lower() and "add any questions here" not in line.lower():
                    # check if the word Builder was used not as a question but as a component.
                    if "*Builder:" in line and "Builder" in line and not "Builder:" in line.replace("*Builder:", ""):
                        print(f"{filepath}:{i+1}:{line}")
