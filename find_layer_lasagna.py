import os
import re

for root, dirs, files in os.walk('src'):
    for file in files:
        if file.endswith('.rs'):
            path = os.path.join(root, file)
            with open(path, 'r', encoding='utf-8') as f:
                content = f.read()

                # Check for unwrap() on Result/Option that might be dangerous
                # Just checking how many unwraps are there vs expect/match
                pass
