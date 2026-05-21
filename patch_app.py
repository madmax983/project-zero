with open("src/layer2/piracy.rs", "r") as f:
    content = f.read()

import_lines = "use bevy::prelude::*;\n"
if "use bevy::prelude::*;" not in content:
    content = import_lines + content

plugin_code = """
pub struct PiracyPlugin;

impl Plugin for PiracyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RaidSuccessEvent>();
        app.add_systems(Update, (
            process_raid_success_system,
            haven_upgrade_system,
            haven_to_republic_system,
        ));
    }
}
"""

if "PiracyPlugin" not in content:
    content = content.replace("#[cfg(test)]", plugin_code + "\n#[cfg(test)]")

with open("src/layer2/piracy.rs", "w") as f:
    f.write(content)
