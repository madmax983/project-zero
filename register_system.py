import sys

filepath = "src/layer1/systems/environment.rs"
with open(filepath, "r") as f:
    env_content = f.read()

if "kinetic_strike_system" not in env_content:
    env_content = env_content.replace(
        "crate::layer1::logistics::orbital_drop::process_orbital_drops,",
        "crate::layer1::logistics::orbital_drop::process_orbital_drops,\n            crate::layer1::physics::kinetic_strike::kinetic_strike_system.after(crate::layer1::logistics::orbital_drop::process_orbital_drops),"
    )
    with open(filepath, "w") as f:
        f.write(env_content)
