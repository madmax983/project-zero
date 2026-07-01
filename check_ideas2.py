import re

with open("design/IDEAS.md") as f:
    ideas = f.read()

def check(keyword):
    if re.search(keyword, ideas, re.IGNORECASE):
        print(f"FOUND: {keyword}")
    else:
        print(f"NOT FOUND: {keyword}")

check("Firstborn")
check("Abyssal Augury")
check("Feral Terraforming")
check("Oracle of")
