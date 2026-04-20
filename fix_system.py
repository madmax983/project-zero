import re

with open("src/layer1/systems/execution.rs", "r") as f:
    text = f.read()

# Replace Layer1Systems with Layer1SystemSet
text = text.replace("Layer1Systems::Social", "Layer1SystemSet::Social")
# Replace bevy_ecs::schedule::in_state(crate::GameState::Playing)
# The state is usually crate::shared::state::GameState
# Let's check shared state
