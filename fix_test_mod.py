import sys
import re

with open("src/layer2/ftl/wakes.rs", "r") as f:
    lines = f.readlines()

new_lines = []
for line in lines:
    if "fn test_high_severity_wake_teleports_entities()" in line:
        pass
    new_lines.append(line)

# Wait we already replaced wakes.rs with test cases, but it seems there was a typo that test cases are missed from coverage run.
