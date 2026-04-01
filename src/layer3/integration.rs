use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer3::planet::black_market_terraforming::RogueTerraformEvent;
use bevy_ecs::prelude::*;

/// Bridges `RogueTerraformEvent` (Black Market Terraforming) to `AddChronicleEvent` (Chronicle).
pub fn black_market_terraforming_bridge(
    mut events: EventReader<RogueTerraformEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for event in events.read() {
        chronicle_events.send(AddChronicleEvent {
            importance: EventImportance::Major,
            text: format!(
                "Unseasonal terraforming in Sector {} caused extreme local weather disruptions",
                event.target_sector
            ),
        });
    }
}
