import os
import re

for root, dirs, files in os.walk('src'):
    for file in files:
        if file.endswith('.rs'):
            path = os.path.join(root, file)
            with open(path, 'r', encoding='utf-8') as f:
                content = f.read()

                matches = re.finditer(r'(pub\s+)?enum\s+(\w+)(?:\s*<[^>]*>)?\s*\{([^}]*)\}', content)
                for match in matches:
                    is_pub = match.group(1) is not None
                    enum_name = match.group(2)
                    body = match.group(3)

                    # Remove attributes like #[...] and comments
                    body_clean = re.sub(r'#\[[^\]]*\]', '', body)
                    body_clean = re.sub(r'//.*', '', body_clean)
                    body_clean = re.sub(r'/\*.*?\*/', '', body_clean, flags=re.DOTALL)

                    # Split by comma to get variants
                    variants = [v.strip() for v in body_clean.split(',') if v.strip()]

                    if len(variants) == 1:
                        print(f"[{enum_name}] in {path} has exactly 1 variant: {variants[0]}")
