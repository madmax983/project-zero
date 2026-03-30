import os
import sys

def check_file(filepath):
    with open(filepath, 'r') as f:
        lines = f.readlines()

    in_block = False
    lines_in_block = []
    block_start_line = 0

    for i, line in enumerate(lines):
        stripped = line.strip()
        if stripped.startswith('/// ```') or stripped.startswith('//! ```'):
            if 'compile_fail' in stripped or 'text' in stripped:
                in_block = True
                lines_in_block = []
                block_start_line = i + 1
                continue

            if in_block:
                # We reached the end of the block
                # Check if it's empty (only contains /// or //!)
                is_empty = True
                for b_line in lines_in_block:
                    clean_line = b_line.strip()
                    if clean_line != '///' and clean_line != '//!':
                        is_empty = False
                        break

                if is_empty:
                    print(f"EMPTY BLOCK FOUND: {filepath}:{block_start_line}-{i+1}")

                in_block = False
            else:
                in_block = True
                lines_in_block = []
                block_start_line = i + 1
        elif in_block:
            lines_in_block.append(line)

for root, _, files in os.walk('src'):
    for f in files:
        if f.endswith('.rs'):
            filepath = os.path.join(root, f)
            check_file(filepath)
