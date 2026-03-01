import glob

for filepath in glob.glob('specs/*.md'):
    with open(filepath, 'r', encoding='utf-8') as f:
        lines = f.readlines()

    for i, line in enumerate(lines):
        if 'Builder:' in line and 'add questions here' not in line.lower() and 'ScheduleBuilder' not in line:
            answered = False
            for j in range(1, 4):
                if i + j < len(lines) and 'Architect:' in lines[i+j]:
                    answered = True
                    break
            if not answered:
                print(f"{filepath}:{i+1}:{line.strip()}")
