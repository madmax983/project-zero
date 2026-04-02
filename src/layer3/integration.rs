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

use crate::layer3::diplomacy::succession::{SuccessionEvent, SuccessionCrisisEvent};

/// Bridges `SuccessionEvent` to `AddChronicleEvent`
pub fn dynastic_succession_chronicle_bridge(
    mut succession_events: EventReader<SuccessionEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for event in succession_events.read() {
        let trait_str = match &event.leader_trait {
            Some(t) => format!(" {:?}", t),
            None => "".to_string(),
        };
        chronicle_events.send(AddChronicleEvent {
            importance: EventImportance::Major,
            text: format!(
                "Succession in {}: {} has died. {} takes the throne.{}",
                event.faction_name, event.old_leader_name, event.new_leader_name, trait_str
            ),
        });
    }
}

/// Bridges `SuccessionCrisisEvent` to `AddChronicleEvent`
pub fn dynastic_crisis_chronicle_bridge(
    mut crisis_events: EventReader<SuccessionCrisisEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for event in crisis_events.read() {
        chronicle_events.send(AddChronicleEvent {
            importance: EventImportance::Legendary,
            text: format!(
                "Succession crisis in {}! {} has died without an heir. The realm bleeds.",
                event.faction_name, event.old_leader_name
            ),
        });
    }
}
