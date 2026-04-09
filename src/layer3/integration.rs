use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer3::planet::black_market_terraforming::RogueTerraformEvent;
use bevy_ecs::prelude::*;

/// Bridges `TradeRouteSeveredEvent` (Hyperlane Collapse) to `AddChronicleEvent` (Chronicle).
pub fn hyperlane_collapse_chronicle_bridge(
    mut sever_events: EventReader<crate::layer3::map::TradeRouteSeveredEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for _ in sever_events.read() {
        chronicle_events.send(AddChronicleEvent {
            importance: EventImportance::Major,
            text: "A hyperlane has collapsed. Trade routes are severed, isolating systems."
                .to_string(),
        });
    }
}

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

use crate::layer3::diplomacy::succession::{SuccessionCrisisEvent, SuccessionEvent};

/// Bridges `SuccessionEvent` to `AddChronicleEvent`
pub fn dynastic_succession_chronicle_bridge(
    mut succession_events: EventReader<SuccessionEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for event in succession_events.read() {
        chronicle_events.send(AddChronicleEvent {
            importance: EventImportance::Major,
            text: format!(
                "Succession in {}: {} has died. {} takes the throne.",
                event.faction_name, event.old_leader_name, event.new_leader_name
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

use crate::layer2::navigation::stellar_weather::FleetDamagedEvent;
use crate::layer3::stellar_cartography::JumpRisk;

/// Bridges `JumpRisk` (Stellar Cartography) to `FleetDamagedEvent` and `AddChronicleEvent` (Chronicle).
pub fn jump_risk_bridge_system(
    mut commands: Commands,
    query: Query<Entity, With<JumpRisk>>,
    mut damage_events: EventWriter<FleetDamagedEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for entity in query.iter() {
        // Remove the risk component so we don't repeatedly damage
        commands.entity(entity).remove::<JumpRisk>();

        // Apply a flat minor damage for jumping blind
        damage_events.send(FleetDamagedEvent {
            fleet: entity,
            amount: 20.0,
        });

        // Add to chronicle
        chronicle_events.send(AddChronicleEvent {
            importance: EventImportance::Major,
            text: "A fleet suffered hull damage after jumping blind into an uncharted system."
                .to_string(),
        });
    }
}

use crate::layer2::fleet::InTransit;
use crate::layer3::ghost_ships::{EvaluateLostShipReturnEvent, EvaluateTransitEvent, LostInTransit};

/// Bridges `InTransit` and `LostInTransit` to Ghost Ships evaluation events.
pub fn ghost_ships_bridge_system(
    transit_query: Query<Entity, With<InTransit>>,
    lost_query: Query<Entity, With<LostInTransit>>,
    mut evaluate_transit_events: EventWriter<EvaluateTransitEvent>,
    mut evaluate_return_events: EventWriter<EvaluateLostShipReturnEvent>,
) {
    for entity in transit_query.iter() {
        evaluate_transit_events.send(EvaluateTransitEvent { ship: entity });
    }

    for entity in lost_query.iter() {
        evaluate_return_events.send(EvaluateLostShipReturnEvent { ship: entity });
    }
}
