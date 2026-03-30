import sys
for line in open('empty_code_block_lines.txt'):
    path, line_num = line.strip().split(':')
    line_num = int(line_num)
    with open(path) as f:
        lines = f.readlines()
    if '```' in lines[line_num-1] and '```' in lines[line_num]:
        print(f"FOUND EMPTY at {path}:{line_num}")
