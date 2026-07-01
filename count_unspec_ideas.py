import re
import os

with open("design/IDEAS.md", "r") as f:
    content = f.read()

count = 0
for match in re.finditer(r"## (.*)", content):
    idea = match.group(1)
    if "[SPECCED]" not in idea and "[REJECTED]" not in idea:
        print(idea)
        count += 1

print(f"Total un-specced ideas: {count}")
