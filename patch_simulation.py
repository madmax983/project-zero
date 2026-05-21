with open("src/simulation.rs", "r") as f:
    content = f.read()

# We need to add PiracyPlugin to a place where other plugins are added, or just directly in setup
if "use crate::layer2::piracy::PiracyPlugin;" not in content:
    content = content.replace("use crate::layer2::trade::blockade::debt_blockade_system;", "use crate::layer2::trade::blockade::debt_blockade_system;\nuse crate::layer2::piracy::PiracyPlugin;")

if "app.add_plugins(PiracyPlugin);" not in content:
    # Find a good place in build_schedule or where layer 2 plugins might be added
    # There doesn't seem to be a Layer2Plugin, so we'll add it in the general app setup
    pass
