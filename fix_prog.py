file_in_prog = 'design/IN_PROGRESS.md'
with open(file_in_prog, 'r') as f:
    content = f.read()

content = content.replace('- [ ] `257` Subspace Pen Pals — `specs/257-subspace-pen-pals.md` — claimed 2026-03-04\n', '')

with open(file_in_prog, 'w') as f:
    f.write(content)

file_comp = 'design/COMPLETED.md'
with open(file_comp, 'r') as f:
    content = f.read()

content = content + '- [x] `257` Subspace Pen Pals — `specs/257-subspace-pen-pals.md` — completed 2026-03-04\n'

with open(file_comp, 'w') as f:
    f.write(content)
