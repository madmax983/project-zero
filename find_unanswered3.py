import os
import re

for filename in os.listdir("specs"):
    filepath = os.path.join("specs", filename)
    if not os.path.isfile(filepath):
        continue
    with open(filepath, "r") as f:
        lines = f.readlines()
        has_q = False
        for line in lines:
            if re.match(r".*\*Builder:.*", line, flags=re.IGNORECASE):
                if "add questions here" not in line.lower() and "add any questions here" not in line.lower():
                    has_q = True

        has_a = False
        for line in lines:
            if re.match(r".*\*Architect:.*", line, flags=re.IGNORECASE):
                has_a = True

        if has_q and not has_a:
            print(filepath)
