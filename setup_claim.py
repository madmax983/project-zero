import re
from datetime import datetime

date_str = datetime.now().strftime("%Y-%m-%d")

with open("design/BACKLOG.md", "r") as f:
    backlog = f.read()
new_backlog = backlog.replace("- [ ] `874` The Blob — `specs/874-the-blob.md`\n", "")
with open("design/BACKLOG.md", "w") as f:
    f.write(new_backlog)

with open("design/IN_PROGRESS.md", "r") as f:
    in_progress = f.read()
in_progress += f"- [ ] `874` The Blob — `specs/874-the-blob.md` — claimed {date_str}\n"
with open("design/IN_PROGRESS.md", "w") as f:
    f.write(in_progress)

print("Task claimed.")
