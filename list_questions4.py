import os
import glob

def run():
    files = glob.glob("specs/*.md")
    for f in files:
        with open(f, "r") as file:
            lines = file.readlines()

        in_questions = False
        for i, line in enumerate(lines):
            if "Questions" in line and ("##" in line or "**" in line):
                in_questions = True
                continue

            if in_questions and line.startswith("## ") and not "Questions" in line:
                in_questions = False

            if in_questions:
                if "Builder:" in line and "add questions here" not in line.lower() and "add any questions here" not in line.lower():
                    # check if answered in next few lines
                    answered = False
                    for j in range(1, 5):
                        if i + j < len(lines) and "Architect:" in lines[i+j]:
                            answered = True
                            break
                    if not answered:
                        print(f"File: {f}")
                        print(line.strip())

run()
