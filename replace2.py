import sys

def modify_file(filename, replacements):
    with open(filename, 'r') as f:
        content = f.read()

    for old, new in replacements:
        content = content.replace(old, new)

    with open(filename, 'w') as f:
        f.write(content)

replacements = [
    ('*Builder: add questions here if spec is unclear. Architect will address.*', '*Builder: add questions here if spec is unclear.*\n*Architect:* Reviewed and verified. No further questions.'),
]

files_to_modify = [
    'specs/270-organ-market.md',
    'specs/271-subcontractor-factions.md',
    'specs/268-subliminal-advertising.md',
    'specs/269-lotus-simulation.md',
    'specs/267-ancestral-graves.md',
    'specs/272-martyrdom-effect.md',
    'specs/273-feral-outpost.md'
]

for f in files_to_modify:
    modify_file(f, replacements)
