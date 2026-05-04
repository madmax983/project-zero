content = open("design/BACKLOG.md").read()
content = content.replace("- [ ] `633` The Golden Age — `specs/633-the-golden-age.md`", "")
open("design/BACKLOG.md", "w").write(content)

content = open("design/IN_PROGRESS.md").read()
content = content.replace("---", "---\n\n- [ ] `633` The Golden Age — `specs/633-the-golden-age.md` — claimed 2024-05-24")
open("design/IN_PROGRESS.md", "w").write(content)
