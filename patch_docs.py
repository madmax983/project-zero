with open('design/IN_PROGRESS.md', 'r') as f:
    content = f.read()

content = content.replace("- [ ] `453` Nanite Fabrication — `specs/453-nanite-fabrication.md` — claimed 2026-03-24\n", "")

with open('design/IN_PROGRESS.md', 'w') as f:
    f.write(content)

with open('design/COMPLETED.md', 'r') as f:
    content = f.read()

if "- [x] `453` Nanite Fabrication — `specs/453-nanite-fabrication.md`" not in content:
    content += "- [x] `453` Nanite Fabrication — `specs/453-nanite-fabrication.md` — completed 2026-03-24\n"

    with open('design/COMPLETED.md', 'w') as f:
        f.write(content)
