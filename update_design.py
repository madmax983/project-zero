import re

with open("design/BACKLOG.md", "r") as f:
    backlog = f.read()

backlog = re.sub(r"- \[ \] `1007` Generational Spite — `specs/1007-generational-spite\.md`\n", "", backlog)

with open("design/BACKLOG.md", "w") as f:
    f.write(backlog)

with open("design/IN_PROGRESS.md", "r") as f:
    in_progress = f.read()

in_progress += "- [ ] `1007` Generational Spite — `specs/1007-generational-spite.md` — claimed 2026-06-11\n"

with open("design/IN_PROGRESS.md", "w") as f:
    f.write(in_progress)
