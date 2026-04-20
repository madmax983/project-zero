with open("src/layer1/systems/execution.rs", "r") as f:
    text = f.read()

# Replace run_if and in_set lines for xenoflora to match typical Layer 1 registration
# In Bevy 0.15, States might be used differently, or maybe Layer 1 systems just run in Update
import re
new_text = re.sub(
    r'\.chain\(\)\s*\.in_set\(Layer1Systems::Social\)\s*\.run_if\(bevy_ecs::schedule::in_state\(crate::GameState::Playing\)\)\s*\.run_if\(bevy_time::common_conditions::on_timer\(std::time::Duration::from_secs_f32\(0\.5\)\)\),',
    '.chain().in_set(Layer1SystemSet::Social),',
    text
)

with open("src/layer1/systems/execution.rs", "w") as f:
    f.write(new_text)
