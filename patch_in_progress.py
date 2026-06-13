with open("design/IN_PROGRESS.md", "r") as f:
    content = f.read()

content += "- [ ] `285` Echoes of the Past — `specs/285-echoes-of-the-past.md` — claimed 2026-02-01\n"

with open("design/IN_PROGRESS.md", "w") as f:
    f.write(content)
