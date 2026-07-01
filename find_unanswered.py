import os
import re

for filename in os.listdir("specs"):
    filepath = os.path.join("specs", filename)
    if not os.path.isfile(filepath):
        continue
    with open(filepath, "r") as f:
        content = f.read()
        if "*Builder:" in content and "*Architect:" not in content:
            print(filepath)
