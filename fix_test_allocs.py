import os
import re

def process_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    new_content = re.sub(
        r'tiles:\s*vec!\[([^;]+);\s*width\.saturating_mul\(height\)\]',
        r'tiles: vec![\1; width * height]',
        content
    )

    new_content = re.sub(
        r'tiles\s*=\s*vec!\[([^;]+);\s*width\.saturating_mul\(height\)\]',
        r'tiles = vec![\1; width * height]',
        new_content
    )

    new_content = re.sub(
        r'tiles:\s*vec!\[([^;]+);\s*size\.saturating_mul\(size\)\]',
        r'tiles: vec![\1; size * size]',
        new_content
    )

    new_content = re.sub(
        r'tiles\s*=\s*vec!\[([^;]+);\s*size\.saturating_mul\(size\)\]',
        r'tiles = vec![\1; size * size]',
        new_content
    )

    if content != new_content:
        with open(filepath, 'w') as f:
            f.write(new_content)
        print(f"Updated {filepath}")

for root, _, files in os.walk('src'):
    for file in files:
        if file.endswith('.rs'):
            process_file(os.path.join(root, file))
