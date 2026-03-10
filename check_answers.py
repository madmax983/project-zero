import os
import re

unanswered = []
files = os.listdir('specs')

for f in files:
    if not f.endswith('.md'): continue
    filepath = os.path.join('specs', f)
    with open(filepath, 'r') as file:
        content = file.read()

    lines = content.split('\n')
    for i, line in enumerate(lines):
        if '*Builder:' in line:
            # check if it's the generic one
            if 'add questions here' in line.lower() or 'unclear' in line.lower():
                continue

            answered = False
            for j in range(i+1, min(i+10, len(lines))):
                if 'Architect:' in lines[j] or '*Architect*' in lines[j]:
                    answered = True
                    break

            if not answered:
                unanswered.append((f, line))

print(f"Total unanswered questions: {len(unanswered)}")
for f, q in unanswered:
    print(f"{f}: {q}")
