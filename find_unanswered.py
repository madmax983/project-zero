import os
import glob

# Find questions that are literally just "Builder: add questions here if spec is unclear"
# BUT where the builder *did* add a question below it, or modified it.
# Actually, let's look for "Builder:" anywhere in specs.
# We want to find questions that do NOT have a following "Architect:" or "Answer:" line.

unanswered = []

for filepath in glob.glob("specs/*.md"):
    with open(filepath, 'r') as f:
        lines = [l.strip() for l in f.readlines()]

    for i, line in enumerate(lines):
        if "Builder:" in line and "add questions here if spec is unclear" not in line and "ScheduleBuilder" not in line:
            # Found a question from the builder.
            # Does it have an Architect response?
            has_response = False
            for j in range(i+1, min(i+4, len(lines))): # Look next 3 lines
                if "Architect:" in lines[j] or "Answer:" in lines[j] or "Yes" in lines[j] or "No" in lines[j] and line.startswith('-'):
                    # A bit fuzzy. Let's just check for Architect/Answer.
                    if "Architect:" in lines[j] or "Answer:" in lines[j]:
                        has_response = True
                        break

            if not has_response:
                # Let's check if the builder's question *already* has an answer appended on the same line or next line but without "Architect:".
                # Like in 151-cybernetic-augmentation.md
                if "Yes" in line or "No" in line and "?" in line:
                    pass
                else:
                    unanswered.append((filepath, i, line))

for filepath, i, line in unanswered:
    print(f"{filepath}:{i+1}:{line}")
