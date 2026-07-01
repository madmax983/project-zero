import re

with open('src/simulation.rs', 'r') as f:
    content = f.read()

# We look for the general area where layer1::psychology systems are added to the schedule.
# Here we find crate::layer1::psychology::stress::stress_decay_system
match = re.search(r'crate::layer1::psychology::stress::stress_decay_system,\n', content)
if match:
    print("Found psychology::stress::stress_decay_system")
else:
    print("NOT FOUND")
