import glob

for filepath in glob.glob('specs/*.md'):
    with open(filepath, 'r', encoding='utf-8') as f:
        lines = f.readlines()

    for i, line in enumerate(lines):
        # We look for lines containing "Builder:" but ignore boilerplate ones
        if 'Builder:' in line and 'add questions here' not in line.lower() and 'ScheduleBuilder' not in line:
            # Check the next 5 lines for "Architect:"
            answered = False
            for j in range(1, 6):
                if i + j < len(lines) and 'Architect:' in lines[i+j]:
                    answered = True
                    break
            if not answered:
                print(f"{filepath}:{i+1}:{line.strip()}")
