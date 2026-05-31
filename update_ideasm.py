import re

ideas_file = "design/IDEAS.md"
with open(ideas_file, "r") as f:
    ideas_content = f.read()

ideas = re.split(r'\n## ', '\n' + ideas_content)

new_ideas_content = ideas[0]

# Add [SPECCED] to the first 5 ideas that didn't have it
specced_count = 0
for idea in ideas[1:]:
    lines = idea.split('\n')
    title = lines[0]

    if '[SPECCED]' not in title and specced_count < 5:
        lines[0] = f"{title} [SPECCED]"
        specced_count += 1

    new_ideas_content += '\n## ' + '\n'.join(lines)

with open(ideas_file, "w") as f:
    f.write(new_ideas_content[1:])
