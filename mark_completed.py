content = open("design/IN_PROGRESS.md").read()
content = content.replace("- [ ] `633` The Golden Age — `specs/633-the-golden-age.md` — claimed 2024-05-24", "")
open("design/IN_PROGRESS.md", "w").write(content)

content = open("design/COMPLETED.md").read()
content = content.replace("---", "---\n\n- [x] `633` The Golden Age — `specs/633-the-golden-age.md` — completed 2024-05-24")
open("design/COMPLETED.md", "w").write(content)
