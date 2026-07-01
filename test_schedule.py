import re

with open('src/simulation.rs', 'r') as f:
    content = f.read()

# Let's find ANY call adding psychology systems
matches = re.finditer(r'schedule\.add_systems\([^;]*psychology[^;]*\);', content, re.DOTALL)
for match in matches:
    print(match.group(0))
