import re

with open('tests/integration.rs', 'r') as f:
    content = f.read()

new_mod = '#[path = "integration/dynastic_succession_chronicle.rs"]\nmod dynastic_succession_chronicle;\n'

if "dynastic_succession_chronicle" not in content:
    content += "\n" + new_mod

with open('tests/integration.rs', 'w') as f:
    f.write(content)

print("Patched tests/integration.rs")
