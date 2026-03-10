import os
import re

files = os.listdir('specs')
missing_architects = []

for f in files:
    if not f.endswith('.md'): continue
    filepath = os.path.join('specs', f)
    with open(filepath, 'r') as file:
        content = file.read()

    builder_questions = re.findall(r'\*Builder:.*?\*', content)
    architect_responses = re.findall(r'\*Architect:.*?\*', content, re.IGNORECASE)

    # Filter out the standard "add questions here if spec is unclear."
    real_questions = [q for q in builder_questions if 'add questions here if spec is unclear' not in q.lower() and 'add questions here' not in q.lower()]

    if len(real_questions) > len(architect_responses):
        missing_architects.append((f, real_questions, architect_responses))

for f, q, a in missing_architects:
    print(f"File: {f}")
    print(f"  Questions: {q}")
    print(f"  Responses: {a}")
