import re

with open('src/layer3/integration.rs', 'r') as f:
    content = f.read()

new_imports = """use crate::layer3::diplomacy::succession::{CurrentLeader, Faction, Leader, SuccessionCrisis};
"""

new_code = """
/// Bridges `process_succession_system` outcomes (dynastic succession or succession crisis) to `AddChronicleEvent` (Chronicle).
pub fn dynastic_succession_chronicle_bridge(
    faction_query: Query<(&Faction, &CurrentLeader), Changed<CurrentLeader>>,
    crisis_query: Query<&Faction, Added<SuccessionCrisis>>,
    leader_query: Query<&Leader>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    // 1. Leader succession
    for (faction, current_leader) in faction_query.iter() {
        if let Ok(leader) = leader_query.get(current_leader.0) {
            chronicle_events.send(AddChronicleEvent {
                importance: EventImportance::Major,
                text: format!(
                    "{} took the throne of {} following a dynastic succession",
                    leader.name, faction.name
                ),
            });
        }
    }

    // 2. Succession Crisis
    for faction in crisis_query.iter() {
        chronicle_events.send(AddChronicleEvent {
            importance: EventImportance::Major,
            text: format!(
                "Succession crisis declared in {} after the ruler died without an heir",
                faction.name
            ),
        });
    }
}
"""

if "use crate::layer3::diplomacy" not in content:
    content = content.replace("use bevy_ecs::prelude::*;", "use bevy_ecs::prelude::*;\n" + new_imports)

if "pub fn dynastic_succession_chronicle_bridge" not in content:
    content += new_code

with open('src/layer3/integration.rs', 'w') as f:
    f.write(content)

print("Patched layer3/integration.rs")
