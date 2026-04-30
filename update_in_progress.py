import re

with open("design/IN_PROGRESS.md", "r") as f:
    lines = f.readlines()

with open("design/COMPLETED.md", "a") as f:
    for line in lines:
        if "INT-1081" in line:
            f.write(line.replace("- [ ]", "- [x]").replace("claimed", "completed"))
        if "INT-1098" in line:
            f.write(line.replace("- [ ]", "- [x]").replace("claimed", "completed"))

with open("design/IN_PROGRESS.md", "w") as f:
    for line in lines:
        if "INT-1081" not in line and "INT-1098" not in line:
            f.write(line)
