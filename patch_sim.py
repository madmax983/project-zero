import os

with open("src/simulation.rs", "r") as f:
    content = f.read()

# Bevy tuple limit fix: split the add_systems call
sys1 = "    // --- Layer 2 Integration ---"
sys2 = """    // --- Layer 3 Integration ---
    schedule.add_systems((
        update_detection_risk_system.after(Layer1SystemSet::Economy),
        check_hostile_spawn_system.after(update_detection_risk_system),
    ));"""
content = content.replace(sys1, sys2 + "\n\n" + sys1)

with open("src/simulation.rs", "w") as f:
    f.write(content)
