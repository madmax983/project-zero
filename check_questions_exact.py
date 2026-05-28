import os

specs_dir = "specs"
unanswered = []

for filename in os.listdir(specs_dir):
    if not filename.endswith(".md"): continue
    path = os.path.join(specs_dir, filename)
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()

    lines = content.split('\n')
    for i, line in enumerate(lines):
        if "*builder:" in line.lower() and "add questions here" not in line.lower() and "add any questions here" not in line.lower() and "add questions regarding" not in line.lower():
            # Check the next few lines for an Architect response
            found_answer = False
            for j in range(1, min(10, len(lines) - i)):
                if "*architect:" in lines[i+j].lower() or "*architect :" in lines[i+j].lower() or "architect:" in lines[i+j].lower():
                    found_answer = True
                    break
            if not found_answer:
                unanswered.append((filename, i+1, line))

for file, line_num, line in unanswered:
    print(f"{file}:{line_num}: {line}")
