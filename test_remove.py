with open('design/IN_PROGRESS.md', 'r') as f:
    lines = f.readlines()
with open('design/IN_PROGRESS.md', 'w') as f:
    for line in lines:
        if 'INT-695' not in line:
            f.write(line)
