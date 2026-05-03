import re

with open('design/IN_PROGRESS.md', 'r') as f:
    content = f.read()

content = re.sub(
    r'- \[ \] `661` Secret Societies — `specs/661-secret-societies.md` — claimed (\d{4}-\d{2}-\d{2})',
    r'',
    content
)

with open('design/IN_PROGRESS.md', 'w') as f:
    f.write(content)

with open('design/COMPLETED.md', 'r') as f:
    completed = f.read()

if 'INT-661' not in completed:
    with open('design/COMPLETED.md', 'a') as f:
        f.write('- [x] `INT-661` Integration: Secret Societies -> Predictive Policing -> Chronicle — completed 2026-06-25\n')
