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

use crate::layer3::diplomacy::red_tape_defense::BureaucraticHold;

/// Bridges `BureaucraticHold` to `AddChronicleEvent` (Chronicle).
pub fn red_tape_chronicle_bridge(
    query: Query<Entity, Added<BureaucraticHold>>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for _ in query.iter() {
        chronicle_events.send(AddChronicleEvent {
            importance: EventImportance::Major,
            text: "Hostile fleet stalled by bureaucratic red tape.".to_string(),
        });
    }
}

use crate::layer1::law::penal::OrganHarvestedEvent;
use crate::layer3::diplomacy_reflection::{Civilization, DiplomaticTraits, TraitChangedEvent};

use crate::layer2::trade::routes::{Colony, TradeRouteExecutedEvent};
use crate::layer3::linguistic_drift::LinguisticNetwork;

/// Bridges `TradeRouteExecutedEvent` and `LinguisticNetwork` to apply a Translation Tax
pub fn language_drift_trade_bridge(
    mut events: EventReader<TradeRouteExecutedEvent>,
    mut colonies: Query<&mut Colony>,
    network: Res<LinguisticNetwork>,
) {
    for event in events.read() {
        let drift = network.get_drift(event.source, event.destination);

        // Applying Translation Tax: High drift imposes a tax on trade efficiency.
        // We'll calculate tax as 0.1% per point of drift, capped at 90%
        let tax_rate = (drift / 1000.0).clamp(0.0, 0.9);
        let tax_amount = (event.amount as f32 * tax_rate) as u32;

        if tax_amount > 0 {
            if let Ok(mut dest_colony) = colonies.get_mut(event.destination) {
                dest_colony.remove_resource(&event.item_type, tax_amount);
            }
        }
    }
}

/// Bridges `OrganHarvestedEvent` to `DiplomaticTraits` for Layer 3 Diplomacy
pub fn organ_trade_diplomacy_bridge(
    mut harvest_events: EventReader<OrganHarvestedEvent>,
    mut civ_query: Query<(Entity, &mut DiplomaticTraits), With<Civilization>>,
    mut trait_events: EventWriter<TraitChangedEvent>,
) {
    if harvest_events.read().next().is_some() {
        for (entity, mut traits) in civ_query.iter_mut() {
            if !traits.is_barbarian {
                traits.is_barbarian = true;
                trait_events.send(TraitChangedEvent { civ_entity: entity });
            }
        }
    }
}

use crate::layer3::intellectual_property_wars::{CassusBelli, CassusBelliReason};

/// Bridges IP Piracy (CassusBelli) to DiplomaticTraits (is_barbarian).
///
/// When a civilization illegally uses a patented tech, a CassusBelli is generated.
/// This system catches the new CassusBelli and updates the pirating civilization's
/// traits to mark them as a barbarian (violator of galactic law), which then
/// triggers diplomatic sanctions from pacifist neighbors.
pub fn ip_piracy_diplomacy_bridge(
    cb_query: Query<&CassusBelli, Added<CassusBelli>>,
    mut civ_query: Query<(Entity, &mut DiplomaticTraits), With<Civilization>>,
    mut trait_events: EventWriter<TraitChangedEvent>,
) {
    for cb in cb_query.iter() {
        if cb.reason == CassusBelliReason {
            // Target is the one pirating the tech
            if let Ok((entity, mut traits)) = civ_query.get_mut(cb.target) {
                if !traits.is_barbarian {
                    traits.is_barbarian = true;
                    trait_events.send(TraitChangedEvent { civ_entity: entity });
                }
            }
        }
    }
}

use crate::layer1::resources::ColonyResources;
use crate::layer1::social::unrest::Unrest;
use crate::layer3::bureaucracy_of_truth::ColonyState;

/// Bridges Layer 1 `ColonyResources` and `Unrest` to Layer 3 `ColonyState`.
pub fn bureaucracy_of_truth_integration_system(
    resources: Option<Res<ColonyResources>>,
    unrest: Option<Res<Unrest>>,
    mut colonies: Query<&mut ColonyState>,
) {
    if let (Some(res), Some(unr)) = (resources, unrest) {
        // Find the main colony entity (assuming there's one for now)
        for mut state in colonies.iter_mut() {
            state.food_reserves = res.food as u32;
            state.unrest = unr.level * 100.0; // Assuming unrest level is 0.0-1.0
        }
    }
}

/// INT-585: Bridges AnomalyDiscoveredEvent (Cartographic Delusion) to AddChronicleEvent (Chronicle).
pub fn anomaly_discovered_chronicle_bridge(
    mut events: bevy_ecs::prelude::EventReader<crate::layer3::map::AnomalyDiscoveredEvent>,
    mut chronicle_events: bevy_ecs::prelude::EventWriter<crate::layer1::core::chronicle::AddChronicleEvent>,
) {
    for event in events.read() {
        chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
            importance: crate::layer1::core::chronicle::EventImportance::Minor,
            text: format!("Anomaly Discovered in Sector {}", event.sector.0),
        });
    }
}
