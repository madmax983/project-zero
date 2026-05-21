with open("src/simulation.rs", "r") as f:
    content = f.read()

import_stmt = "use crate::layer2::piracy::PiracyPlugin;\n"
if "use crate::layer2::piracy::PiracyPlugin;" not in content:
    content = content.replace("use crate::layer2::events::RoguePlanetEvent;", import_stmt + "use crate::layer2::events::RoguePlanetEvent;")

if "app.add_plugins(PiracyPlugin);" not in content:
    if "app.add_plugins(MinimalPlugins);" in content:
        content = content.replace("app.add_plugins(MinimalPlugins);", "app.add_plugins(MinimalPlugins);\n        app.add_plugins(PiracyPlugin);", 1)

with open("src/simulation.rs", "w") as f:
    f.write(content)
