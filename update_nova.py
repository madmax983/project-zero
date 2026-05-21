import re

with open(".jules/nova.md", "r") as f:
    content = f.read()

with open(".jules/nova.md.tmp", "r") as f:
    new_entry = f.read()

# Add the new entry at the top if not already present
if "Radioactive Vermin" not in content:
    with open(".jules/nova.md", "w") as f:
        f.write(new_entry + "\n" + content)
