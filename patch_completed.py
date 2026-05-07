with open('design/IN_PROGRESS.md', 'r') as f:
    content = f.read()

content = content.replace('- [ ] `623` The Nostalgia Plague — `specs/623-nostalgia-plague.md` — claimed 2024-05-24\n', '')

with open('design/IN_PROGRESS.md', 'w') as f:
    f.write(content)

with open('design/COMPLETED.md', 'r') as f:
    content = f.read()

if '623' not in content:
    content += "\n- [x] `623` The Nostalgia Plague — `specs/623-nostalgia-plague.md` — completed 2024-05-24\n"
    with open('design/COMPLETED.md', 'w') as f:
        f.write(content)
