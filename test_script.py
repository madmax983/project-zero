import re
import os

with open("design/BACKLOG.md", "r") as f:
    backlog_lines = f.readlines()

def get_spec_number_from_backlog(line):
    match = re.search(r"- \[ \] `(\d+)`", line)
    if match:
        return int(match.group(1))
    return None

max_spec_num = 0
for filename in os.listdir("specs"):
    match = re.match(r"^(\d+)-.*\.md$", filename)
    if match:
        max_spec_num = max(max_spec_num, int(match.group(1)))

for line in backlog_lines:
    num = get_spec_number_from_backlog(line)
    if num:
        max_spec_num = max(max_spec_num, num)

print(f"Max spec num: {max_spec_num}")
