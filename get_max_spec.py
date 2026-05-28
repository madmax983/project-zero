import os
import re

specs_dir = "specs"
max_num = 0

for filename in os.listdir(specs_dir):
    match = re.match(r"^(\d+)-", filename)
    if match:
        num = int(match.group(1))
        if num > max_num:
            max_num = num

print(max_num)
