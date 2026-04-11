with open('design/IN_PROGRESS.md', 'r') as f:
    content = f.read()

content = content.replace("- [ ] `569` The Syzygy — `specs/569-the-syzygy.md` — claimed 2026-05-01\n", "")

with open('design/IN_PROGRESS.md', 'w') as f:
    f.write(content)

with open('design/COMPLETED.md', 'r') as f:
    content = f.read()

content += "- [x] `569` The Syzygy — `specs/569-the-syzygy.md` — completed 2026-05-01\n"

with open('design/COMPLETED.md', 'w') as f:
    f.write(content)

with open('design/BACKLOG.md', 'r') as f:
    content = f.read()

content = content.replace("- [ ] `569` The Syzygy — `specs/569-the-syzygy.md`\n", "")

with open('design/BACKLOG.md', 'w') as f:
    f.write(content)
