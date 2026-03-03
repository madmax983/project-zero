import glob

for filepath in glob.glob('specs/*.md'):
    with open(filepath, 'r', encoding='utf-8') as f:
        lines = f.readlines()

    for i, line in enumerate(lines):
        if 'Builder:' in line and 'add questions here if spec is unclear' not in line.lower() and 'ScheduleBuilder' not in line:
            # Check the next 3 lines for an Architect response
            answered = False
            for j in range(1, min(4, len(lines) - i)):
                if 'Architect:' in lines[i+j]:
                    answered = True
                    break
            if not answered:
                print(f"{filepath}:{i+1}:{line.strip()}")
