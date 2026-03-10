file_backlog = 'design/BACKLOG.md'
with open(file_backlog, 'r') as f:
    content = f.read()

content = content.replace('- [ ] `257` Subspace Pen Pals — `specs/257-subspace-pen-pals.md`\n', '')

with open(file_backlog, 'w') as f:
    f.write(content)

file_in_prog = 'design/IN_PROGRESS.md'
with open(file_in_prog, 'r') as f:
    content = f.read()

if '257' not in content:
    content = content.replace('---\n', '---\n- [ ] `257` Subspace Pen Pals — `specs/257-subspace-pen-pals.md` — claimed 2026-03-04\n')

with open(file_in_prog, 'w') as f:
    f.write(content)
