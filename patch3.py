import re

with open("design/IN_PROGRESS.md", "w") as f:
    f.write("- [ ] `1043` Black Market Infrastructure — `specs/1043-black-market-infrastructure.md` — claimed 2024-06-13\n")

with open("design/BACKLOG.md", "r") as f:
    content = f.read()

if "- [ ] `1043` Black Market Infrastructure — `specs/1043-black-market-infrastructure.md`" in content:
    content = content.replace("- [ ] `1043` Black Market Infrastructure — `specs/1043-black-market-infrastructure.md`", "")
    with open("design/BACKLOG.md", "w") as f:
        f.write(content)
