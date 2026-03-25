import re

with open('src/layer1/systems/cleanup.rs', 'r') as f:
    content = f.read()

# The first tuple in cleanup.rs is too large after adding the new systems. Let's split it.
# Find the first schedule.add_systems block in cleanup.rs and split it.

# Let's remove the two added lines first.
content = re.sub(r'\s*update_event_buffer::<crate::layer1::shipbreaking::SpawnCrashedShipEvent>,', '', content)
content = re.sub(r'\s*update_event_buffer::<crate::layer1::shipbreaking::MineEvent>,', '', content)

# Now add a new block at the end.
new_block = """
    schedule.add_systems(
        (
            update_event_buffer::<crate::layer1::shipbreaking::SpawnCrashedShipEvent>,
            update_event_buffer::<crate::layer1::shipbreaking::MineEvent>,
        )
            .in_set(Layer1SystemSet::EventCleanup),
    );
"""

last_brace_idx = content.rfind('}')
content = content[:last_brace_idx] + new_block + content[last_brace_idx:]

with open('src/layer1/systems/cleanup.rs', 'w') as f:
    f.write(content)
