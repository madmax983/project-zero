import os
import glob

def run():
    files = glob.glob("specs/*.md")
    for f in files:
        with open(f, "r") as file:
            lines = file.readlines()

        has_unanswered = False
        for i, line in enumerate(lines):
            if "Builder:" in line and "add questions here" not in line.lower() and "add any questions here" not in line.lower():
                # check if next line or two has Architect:
                answered = False
                for j in range(1, 4):
                    if i + j < len(lines) and "Architect:" in lines[i+j]:
                        answered = True
                        break
                if not answered:
                    print(f"File: {f}, Line: {i+1}")
                    print(line.strip())

run()
