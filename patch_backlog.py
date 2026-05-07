import re

with open('design/BACKLOG.md', 'r') as f:
    content = f.read()

# Make sure we don't duplicate
if '- [ ] `623` The Nostalgia Plague' in content:
    content = content.replace('- [ ] `623` The Nostalgia Plague — `specs/623-nostalgia-plague.md`\n', '')
    with open('design/BACKLOG.md', 'w') as f:
        f.write(content)

with open('design/IN_PROGRESS.md', 'r') as f:
    content = f.read()

if '623' not in content:
    content += "\n- [ ] `623` The Nostalgia Plague — `specs/623-nostalgia-plague.md` — claimed 2024-05-24\n"
    with open('design/IN_PROGRESS.md', 'w') as f:
        f.write(content)
