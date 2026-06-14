import re

with open('src/layer1/core/integration.rs', 'r') as f:
    content = f.read()

new_bridge_code = """
/// Bridges `Consumed` addition to `AddChronicleEvent` for Edible Architecture.
pub fn edible_architecture_chronicle_bridge(
    mut chronicle_events: EventWriter<crate::layer1::core::chronicle::AddChronicleEvent>,
    q_edible: Query<(), Added<crate::layer1::architecture::edible::Consumed>>,
) {
    for _ in q_edible.iter() {
        chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
            text: "The walls are Edible Architecture now. We consumed the mycelial scaffolding to survive.".to_string(),
            importance: crate::layer1::core::chronicle::EventImportance::Major,
        });
    }
}
"""

content = re.sub(
    r"/// Bridges `BuildingRemovedEvent` to `AddChronicleEvent` for Edible Architecture\.\npub fn edible_architecture_chronicle_bridge\(.*?\}\n",
    new_bridge_code,
    content,
    flags=re.DOTALL
)

with open('src/layer1/core/integration.rs', 'w') as f:
    f.write(content)
