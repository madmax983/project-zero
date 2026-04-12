import re

with open("tests/integration/quirks_atmosphere.rs", "r") as f:
    content = f.read()

# The system that diffuses is `simulate_diffusion_system` not `update_atmosphere_system`!
content = content.replace("world.run_system_once(update_atmosphere_system).unwrap();", "world.run_system_once(scale::layer1::atmosphere::simulate_diffusion_system).unwrap();")

with open("tests/integration/quirks_atmosphere.rs", "w") as f:
    f.write(content)
