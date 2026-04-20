import re

# Update IN_PROGRESS.md
with open("design/IN_PROGRESS.md", "r") as f:
    in_prog = f.read()

in_prog = re.sub(r"- \[ \] `965` The Vertical Schism — `specs/965-the-vertical-schism.md` — claimed 2026-02-01\n", "", in_prog)

with open("design/IN_PROGRESS.md", "w") as f:
    f.write(in_prog)

# Update COMPLETED.md
with open("design/COMPLETED.md", "r") as f:
    comp = f.read()

comp += "\n- [x] `965` The Vertical Schism — `specs/965-the-vertical-schism.md` — completed 2026-02-01\n"

with open("design/COMPLETED.md", "w") as f:
    f.write(comp)
