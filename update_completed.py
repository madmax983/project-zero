import re
import datetime

# Update IN_PROGRESS.md
with open("design/IN_PROGRESS.md", "r") as f:
    in_progress = f.read()

in_progress_pattern = r"- \[ \] `547` The Parasitic Broadcast — `specs/547-parasitic-broadcast\.md` — claimed \d{4}-\d{2}-\d{2}\n"
in_progress = re.sub(in_progress_pattern, '', in_progress)

with open("design/IN_PROGRESS.md", "w") as f:
    f.write(in_progress)

# Update COMPLETED.md
with open("design/COMPLETED.md", "r") as f:
    completed = f.read()

today = datetime.date.today().strftime("%Y-%m-%d")
new_line = f"- [x] `547` The Parasitic Broadcast — `specs/547-parasitic-broadcast.md` — completed {today}\n"

if new_line not in completed:
    completed = completed.rstrip() + "\n" + new_line
    with open("design/COMPLETED.md", "w") as f:
        f.write(completed)

print("Updated IN_PROGRESS.md and COMPLETED.md")
