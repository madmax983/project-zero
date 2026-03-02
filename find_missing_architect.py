import re

with open('real_questions.txt', 'r') as f:
    lines = f.readlines()

for line in lines:
    parts = line.split(':', 2)
    if len(parts) >= 3:
        filepath = parts[0]
        line_num = int(parts[1])
        question = parts[2].strip()

        if 'ScheduleBuilder' in question:
            continue

        with open(filepath, 'r') as f_orig:
            orig_lines = f_orig.readlines()

        answered = False
        for i in range(line_num, min(line_num + 3, len(orig_lines))):
            if 'Architect:' in orig_lines[i]:
                answered = True
                break

        if not answered:
            print(f"{filepath}:{line_num}:{question}")

