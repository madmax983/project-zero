import re

with open('src/simulation.rs', 'r') as f:
    content = f.read()

# Register event AwakenTitanEvent in setup
if "crate::layer1::nature::megafauna_terrain::AwakenTitanEvent" not in content:
    content = content.replace("app.add_event::<GhostShiftStartedEvent>();", "app.add_event::<GhostShiftStartedEvent>();\n    app.add_event::<crate::layer1::nature::megafauna_terrain::AwakenTitanEvent>();")

with open('src/simulation.rs', 'w') as f:
    f.write(content)

with open('src/layer1/systems/cleanup.rs', 'r') as f:
    cleanup = f.read()

if "crate::layer1::nature::megafauna_terrain::AwakenTitanEvent" not in cleanup:
    cleanup = cleanup.replace("            update_event_buffer::<crate::layer1::shipbreaking::MineEvent>,", "            update_event_buffer::<crate::layer1::shipbreaking::MineEvent>,\n            update_event_buffer::<crate::layer1::nature::megafauna_terrain::AwakenTitanEvent>,")

with open('src/layer1/systems/cleanup.rs', 'w') as f:
    f.write(cleanup)
