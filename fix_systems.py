import re

with open('src/layer1/systems/execution.rs', 'r') as f:
    content = f.read()

# Bevy system tuples have a max size. The current tuple is too large.
# Let's find the tuple and break it into two schedule.add_systems calls.

# The failing block seems to be the third add_systems block. Let's just create a new add_systems block for shipbreaking.
# First, remove the injected shipbreaking lines.
content = re.sub(r'\s*crate::layer1::shipbreaking::spawn_crashed_ship_system.after\(process_start_plan_system\),', '', content)
content = re.sub(r'\s*crate::layer1::shipbreaking::mine_system.after\(arrival_handler_system\),', '', content)
content = re.sub(r'\s*crate::layer1::shipbreaking::hull_destroyed_system.after\(crate::layer1::shipbreaking::mine_system\),', '', content)

# Then add them in a new block at the end of the file.
new_block = """
    schedule.add_systems(
        (
            crate::layer1::shipbreaking::spawn_crashed_ship_system.after(process_start_plan_system),
            crate::layer1::shipbreaking::mine_system.after(arrival_handler_system),
            crate::layer1::shipbreaking::hull_destroyed_system.after(crate::layer1::shipbreaking::mine_system),
        )
            .in_set(Layer1SystemSet::Execution),
    );
"""

# inject before the last closing brace
last_brace_idx = content.rfind('}')
content = content[:last_brace_idx] + new_block + content[last_brace_idx:]

with open('src/layer1/systems/execution.rs', 'w') as f:
    f.write(content)
