import re

with open("tests/integration/quirks_atmosphere.rs", "r") as f:
    content = f.read()

# We need to run apply_quirk_modifiers_system first which actually modifies `AtmosphereGrid.diffusion_rate`
# Then we don't need to `simulate_diffusion_system` because `update_atmosphere_system` runs the diffusion!
# BUT wait! AtmosphereGrid.diffuse() is NOT called in `update_atmosphere_system` !
# It is called in `simulate_diffusion_system` OR we can call `diffuse()` directly! Let's check `simulate_diffusion_system`

content = content.replace("world.run_system_once(update_atmosphere_system).unwrap();", "world.resource_mut::<scale::layer1::atmosphere::AtmosphereGrid>().diffuse();")
content = content.replace("world_normal.run_system_once(update_atmosphere_system).unwrap();", "world_normal.resource_mut::<scale::layer1::atmosphere::AtmosphereGrid>().diffuse();")

with open("tests/integration/quirks_atmosphere.rs", "w") as f:
    f.write(content)
