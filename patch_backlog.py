with open("design/IN_PROGRESS.md", "r") as f:
    lines = f.readlines()
with open("design/IN_PROGRESS.md", "w") as f:
    for line in lines:
        if "`701` Primitive Civilizations" not in line:
            f.write(line)

with open("design/COMPLETED.md", "r") as f:
    lines = f.readlines()
with open("design/COMPLETED.md", "w") as f:
    for line in lines:
        f.write(line)
        if "---" in line:
            f.write("- [x] `701` Primitive Civilizations - `specs/701-primitive-civilizations.md` - completed 2026-06-01\n")
