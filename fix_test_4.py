import re

with open("tests/integration/quirks_atmosphere.rs", "r") as f:
    content = f.read()

# We need to run apply_quirk_modifiers_system first which actually modifies `AtmosphereGrid.diffusion_rate`
# Then we don't need to `simulate_diffusion_system` because `update_atmosphere_system` runs the diffusion!
# Wait, `update_atmosphere_system` uses `diffusion_rate` but let's check its logic.
content = content.replace("world.run_system_once(scale::layer1::atmosphere::simulate_diffusion_system).unwrap();", "world.run_system_once(update_atmosphere_system).unwrap();")
content = content.replace("world_normal.run_system_once(scale::layer1::atmosphere::simulate_diffusion_system).unwrap();", "world_normal.run_system_once(update_atmosphere_system).unwrap();")

with open("tests/integration/quirks_atmosphere.rs", "w") as f:
    f.write(content)
