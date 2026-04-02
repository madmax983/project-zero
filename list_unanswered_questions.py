import glob

def find_unanswered_questions():
    files = glob.glob("specs/*.md")
    unanswered_files = []

    for f in files:
        with open(f, "r", encoding="utf-8") as file:
            content = file.read()

        if "Builder:" in content:
            lines = content.splitlines()
            in_questions = False
            has_unanswered = False

            for i, line in enumerate(lines):
                if line.startswith("## Questions") or line.startswith("## 8. Questions"):
                    in_questions = True

                if in_questions and "Builder:" in line:
                    lower_line = line.lower()
                    if "add questions here" in lower_line or "add any questions here" in lower_line or "if spec is unclear" in lower_line:
                        continue

                    # Check next few lines for "Architect:"
                    answered = False
                    for j in range(i+1, min(i+4, len(lines))):
                        if "Architect:" in lines[j]:
                            answered = True
                            break

                    if not answered:
                        has_unanswered = True
                        print(f"{f}: {line}")

            if has_unanswered:
                unanswered_files.append(f)

    print(f"Total files with unanswered questions: {len(unanswered_files)}")

find_unanswered_questions()
