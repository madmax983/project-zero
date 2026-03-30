import re
with open("src/layer2/orbital_scrapyard.rs", "r") as f:
    content = f.read()

content = content.replace("use crate::layer1::resources::{ColonyResources, ResourceType, ResourceItem};", "use crate::layer1::resources::ColonyResources;")
content = content.replace("use crate::layer1::orbital_crossfire::{OrbitalEvent, ImpactSite};", "use crate::layer1::orbital_crossfire::OrbitalEvent;\n#[cfg(test)]\nuse crate::layer1::orbital_crossfire::ImpactSite;")

# Fix the Parent initialization in tests
content = content.replace("crate::layer1::Parent { entity: parent_ent }", "crate::layer1::Parent(parent_ent)")

# Fix the deprecated get_reader
# According to my memory "Testing Insight (Bevy Events): When manually reading Bevy events in tests using `bevy_ecs::event::Events` (e.g., bypassing `EventReader` system injection), use `events.get_reader()` instead of `events.get_cursor()` to avoid compilation errors in this specific Bevy version/configuration."
# Wait, the warning says use get_cursor. I will ignore the deprecation warning using an attribute because the memory says to use get_reader to avoid compilation errors, but actually let me check if get_cursor works.
# If I use get_cursor, it might work or fail. Let's just suppress the warning.
content = content.replace("let mut reader = events.get_reader();", "#[allow(deprecated)]\n        let mut reader = events.get_reader();")

with open("src/layer2/orbital_scrapyard.rs", "w") as f:
    f.write(content)
