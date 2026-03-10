import sys

def modify_file(filename, replacements):
    with open(filename, 'r') as f:
        content = f.read()

    for old, new in replacements:
        content = content.replace(old, new)

    with open(filename, 'w') as f:
        f.write(content)

modify_file('specs/221-organic-recycling.md', [
    ('*   *Builder*: Should `Recycler` require power? (Yes, assumed standard building).', '*   *Builder*: Should `Recycler` require power?\n    *Architect:* Yes, assumed standard building.'),
    ('*   *Builder*: Does `Waste` come from `ColonyResources.waste` or items on the ground? (Both. Haulers pick up items, Industry outputs to `ColonyResources.waste`. `Recycler` should probably pull from `ColonyResources.waste` via a worker job, or have haulers bring items).', '*   *Builder*: Does `Waste` come from `ColonyResources.waste` or items on the ground?\n    *Architect:* Both. Haulers pick up items, Industry outputs to `ColonyResources.waste`. `Recycler` should probably pull from `ColonyResources.waste` via a worker job, or have haulers bring items.')
])

modify_file('specs/222-paperwork-physicality.md', [
    ('*   *Builder*: Should permits stack? (Yes, they are items).', '*   *Builder*: Should permits stack?\n    *Architect:* Yes, they are items.'),
    ('*   *Builder*: What if I deconstruct the building? Do I get the permit back? (No, bureaucracy is a sunk cost).', '*   *Builder*: What if I deconstruct the building? Do I get the permit back?\n    *Architect:* No, bureaucracy is a sunk cost.')
])
