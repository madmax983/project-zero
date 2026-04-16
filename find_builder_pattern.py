import os
import re

for root, dirs, files in os.walk('src'):
    for file in files:
        if file.endswith('.rs'):
            path = os.path.join(root, file)
            with open(path, 'r', encoding='utf-8') as f:
                content = f.read()

                # Check for Builders
                if 'struct' in content and 'Builder' in content:
                    # Let's see if the file defines a builder struct
                    matches = re.finditer(r'struct\s+(\w+Builder)', content)
                    for match in matches:
                        print(f"[{match.group(1)}] found in {path}")
