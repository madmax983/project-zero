import os
import re

backlog_lines = []
with open("design/BACKLOG.md", "r") as f:
    backlog_lines = f.readlines()

specs = set()
for filename in os.listdir("specs"):
    match = re.match(r"^(\d+)-.*\.md$", filename)
    if match:
        specs.add(match.group(1))

for line in backlog_lines:
    match = re.search(r"- \[ \] `(\d+)`.*— `specs/(\d+)-.*\.md`", line)
    if match:
        num = match.group(1)
        if num not in specs:
            print(f"Missing spec file for {num}: {line.strip()}")
