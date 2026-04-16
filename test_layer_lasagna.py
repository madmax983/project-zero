import re
import os

for root, dirs, files in os.walk('src'):
    for file in files:
        if file.endswith('.rs'):
            path = os.path.join(root, file)
            with open(path, 'r', encoding='utf-8') as f:
                content = f.read()
                # find functions that just call another function
                # Example: pub fn foo(x: i32) { bar(x) }
                matches = re.finditer(r'pub fn (\w+)\(([^)]*)\)(?:\s*->\s*[^\{]+)?\s*\{\s*(\w+)\(([^)]*)\)\s*\}', content)
                for match in matches:
                    print(f"Lasagna in {path}: {match.group(0)}")
