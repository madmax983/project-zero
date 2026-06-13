with open("design/BACKLOG.md", "r") as f:
    content = f.read()

content = content.replace("- [ ] `285` Echoes of the Past — `specs/285-echoes-of-the-past.md`\n", "")

with open("design/BACKLOG.md", "w") as f:
    f.write(content)
