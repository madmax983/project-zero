import os
import re

for filename in os.listdir("specs"):
    filepath = os.path.join("specs", filename)
    if not os.path.isfile(filepath):
        continue
    with open(filepath, "r") as f:
        content = f.read()
        if re.search(r"\*Builder: (?!add questions here)(?!add any questions here)", content, flags=re.IGNORECASE):
            if not re.search(r"\*Architect:", content, flags=re.IGNORECASE):
                print(filepath)
