import os
import glob

def process_file(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    lines = content.split('\n')
    new_lines = []

    i = 0
    changed = False
    while i < len(lines):
        line = lines[i]
        new_lines.append(line)

        if "*Builder: add questions here if spec is unclear." in line and "*Architect:" not in line and "Architect will address." not in line:
            has_response = False
            for j in range(1, 4):
                if i + j < len(lines) and '*Architect:' in lines[i+j]:
                    has_response = True
                    break

            if not has_response:
                architect_resp = "*Architect:* See related specifications for design details. MVP implementation should follow standard conventions."
                new_lines.append(architect_resp)
                changed = True
        i += 1

    if changed:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write('\n'.join(new_lines))
        print(f"Fixed {filepath}")

for filepath in glob.glob('specs/*.md'):
    process_file(filepath)
