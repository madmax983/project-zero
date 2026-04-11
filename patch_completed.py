import re

with open('design/COMPLETED.md', 'r') as f:
    content = f.read()

new_line = "- [x] `INT-768` Integration: Dynastic Succession -> Chronicle — completed 2026-04-10\n"

if "`INT-768`" not in content:
    content = content.replace("- [x] `768` Dynastic Succession — `specs/768-dynastic-succession.md` — completed 2026-04-10\n", "- [x] `768` Dynastic Succession — `specs/768-dynastic-succession.md` — completed 2026-04-10\n" + new_line)

with open('design/COMPLETED.md', 'w') as f:
    f.write(content)

with open('design/IN_PROGRESS.md', 'r') as f:
    in_prog = f.read()

in_prog = in_prog.replace("- [ ] `INT-768` Integration: Dynastic Succession -> Chronicle — claimed 2026-04-10\n", "")

with open('design/IN_PROGRESS.md', 'w') as f:
    f.write(in_prog)

print("Patched COMPLETED.md and IN_PROGRESS.md")
