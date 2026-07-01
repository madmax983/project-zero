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
            if "*Builder:" in line and "add questions here" not in line.lower() and "add any questions here" not in line.lower():
                if "Architect:" not in line and "Builder:" in line.replace("*Builder:", "") == False and not "Builder: Implement" in line and not "ScheduleBuilder" in line and not "Builder::new" in line:
                    has_arch = False
                    for j in range(i+1, min(i+5, len(lines))):
                        if "Architect:" in lines[j]:
                            has_arch = True
                    if not has_arch:
                        print(f"{filepath}:{i+1}:{line}")
