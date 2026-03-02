import glob

for filepath in glob.glob('specs/*.md'):
    with open(filepath, 'r', encoding='utf-8') as f:
        lines = f.readlines()

    for i, line in enumerate(lines):
        if '*Builder:' in line and 'add questions here' not in line:
            # Check next line
            if i + 1 < len(lines) and '*Architect:*' in lines[i+1]:
                continue
            if i + 1 < len(lines) and '- *Architect:*' in lines[i+1]:
                continue
            if i + 1 < len(lines) and '*Architect:' in lines[i+1]:
                continue
            print(f"{filepath}:{i+1}:{line.strip()}")
