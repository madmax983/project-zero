use crate::cross_layer::tourism::CasusBelli;
use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
use bevy_ecs::prelude::*;

/// Emits a Chronicle event when Weaponized Tourism leads to a Casus Belli.
pub fn weaponized_tourism_chronicle_bridge(
    query: Query<&CasusBelli, Added<CasusBelli>>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for cb in query.iter() {
        if cb.reason == "Tourist Harmed" {
            chronicle_events.send(AddChronicleEvent {
                importance: EventImportance::Major,
                text:
                    "A diplomatic crisis has erupted: Casus Belli declared due to a harmed tourist."
                        .to_string(),
            });
        }
    }
}
