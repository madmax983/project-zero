import re

with open('design/IDEAS.md', 'r') as f:
    content = f.read()

def repl(match):
    return match.group(1) + " [SPECCED]"

# 692 Bioluminescent Flora
content = re.sub(r'(## Bioluminescent Flora)(?! \[SPECCED\])', repl, content)
# 693 Procedural Dialects
content = re.sub(r'(## Procedural Dialects)(?! \[SPECCED\])', repl, content)
# 694 Corporate Sponsorship
content = re.sub(r'(## Corporate Sponsorship)(?! \[SPECCED\])', repl, content)
# 695 Desire Paths
content = re.sub(r'(## Desire Paths)(?! \[SPECCED\])', repl, content)

with open('design/IDEAS.md', 'w') as f:
    f.write(content)
