import os
import re
import subprocess

def fix_all():
    while True:
        result = subprocess.run(['cargo', 'check', '--tests'], capture_output=True, text=True)
        out = result.stderr

        matches = re.findall(r"error\[E0063\]: missing field `meaning` in initializer of `[a-zA-Z0-9_:]*Needs`\s+-->\s+([^:]+):(\d+)", out)
        if not matches:
            print("No more errors.")
            break

        for file, line in set(matches):
            line_idx = int(line) - 1
            with open(file, 'r') as f:
                lines = f.readlines()

            # Find the closing brace of the Needs struct starting from line_idx
            brace_count = 0
            started = False
            for i in range(line_idx, len(lines)):
                brace_count += lines[i].count('{')
                brace_count -= lines[i].count('}')
                if '{' in lines[i]:
                    started = True
                if started and brace_count == 0:
                    lines.insert(i, "meaning: 0.8,\n")
                    break
            with open(file, 'w') as f:
                f.writelines(lines)
        print(f"Fixed {len(set(matches))} locations")
