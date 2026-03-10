import os

for f in os.listdir('specs'):
    if not f.endswith('.md'): continue
    with open(os.path.join('specs', f), 'r') as file:
        content = file.read()

    lines = content.split('\n')
    questions = []
    responses = []

    for i, line in enumerate(lines):
        if '*Builder:' in line and 'add questions here if spec is unclear' not in line.lower() and 'add questions here.' not in line.lower():
            questions.append((i, line))
        elif '*Architect:*' in line or '*Architect:' in line:
            responses.append((i, line))

    # Simple check: does every question have a response closely following it?
    for q_idx, q_text in questions:
        answered = False
        for a_idx, a_text in responses:
            if q_idx < a_idx <= q_idx + 5:  # Response within 5 lines
                answered = True
                break
        if not answered:
            print(f"UNANSWERED in {f}: {q_text}")
