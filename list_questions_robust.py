import glob

def run():
    files = glob.glob("specs/*.md")
    for f in files:
        with open(f, "r") as file:
            lines = file.readlines()

        in_questions = False
        unanswered_questions = []

        for line in lines:
            if "Questions" in line and line.startswith("#"):
                in_questions = True
                continue

            if in_questions and line.startswith("#") and "Questions" not in line:
                in_questions = False

            if in_questions:
                if "Builder:" in line or "builder:" in line:
                    if "add questions here" not in line.lower() and "add any questions here" not in line.lower():
                        unanswered_questions.append(line.strip())
                elif "Architect:" in line or "architect:" in line:
                    # An architect answer clears previous unanswered questions if we just pair them loosely,
                    # but let's just count if Architect lines are fewer than Builder lines that have actual questions
                    pass

        # More simple: If there is a Builder question but NO Architect answer in the WHOLE questions section?
        # Actually sometimes multiple Builder lines and one Architect line.

        in_q = False
        has_builder_real = False
        has_architect = False
        for line in lines:
            if "Questions" in line and line.startswith("#"):
                in_q = True
                continue
            if in_q and line.startswith("#") and "Questions" not in line:
                in_q = False
            if in_q:
                if "Builder:" in line and "add questions here" not in line.lower() and "add any questions here" not in line.lower():
                    has_builder_real = True
                if "Architect:" in line:
                    has_architect = True

        if has_builder_real and not has_architect:
            print(f"Unanswered in {f}")

run()
