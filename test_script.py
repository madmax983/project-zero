import re

def parse_ideas():
    with open("design/IDEAS.md", "r") as f:
        content = f.read()

    ideas = []
    # Split by ---
    sections = content.split("---")
    for section in sections:
        if "##" in section and "[SPECCED]" not in section:
            ideas.append(section)

    print(len(ideas))
    if len(ideas) > 0:
        print(ideas[1].strip())

parse_ideas()
