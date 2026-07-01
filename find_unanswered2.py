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
            if re.match(r".*\*Builder:.*", line):
                if "add questions here" not in line and "add any questions here" not in line:
                    has_q = True

        has_a = False
        for line in lines:
            if re.match(r".*\*Architect:.*", line):
                has_a = True

        if has_q and not has_a:
            print(filepath)
