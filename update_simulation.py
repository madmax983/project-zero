import re

with open('src/simulation.rs', 'r') as f:
    content = f.read()

systems_to_add = """
        crate::layer1::psychology::doomsday_clock::apply_doomsday_panic_effects,
        crate::layer1::psychology::doomsday_clock::resolve_doomsday_event,
        crate::layer1::psychology::doomsday_clock::cleanup_nihilism_debuffs,"""

# Find the place where psychology systems are added, for example `crate::layer1::psychology::stress::stress_decay_system,`
pattern = r'(crate::layer1::psychology::stress::stress_decay_system,)'
replacement = r'\1' + systems_to_add

new_content = re.sub(pattern, replacement, content)

with open('src/simulation.rs', 'w') as f:
    f.write(new_content)

print("Updated simulation.rs")
