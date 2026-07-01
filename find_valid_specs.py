import re
import os

with open("design/BACKLOG.md", "r") as f:
    backlog_lines = f.readlines()

for line in backlog_lines:
    match = re.search(r"- \[ \] `(\d+)`", line)
    if match:
        num = match.group(1)
        for filename in os.listdir("specs"):
            if filename.startswith(num + "-"):
                filepath = os.path.join("specs", filename)
                with open(filepath, "r") as spec_file:
                    content = spec_file.read()
                    if "*Builder: add questions here if spec is unclear.*" in content and "*Architect:" not in content and "- **Architectural Contradictions:**" not in content:
                        print(f"Found valid spec: {filepath}")
