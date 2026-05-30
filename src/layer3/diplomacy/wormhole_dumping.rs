use bevy::prelude::*;
use crate::layer1::economy::resources::ColonyResources;
use crate::layer3::diplomacy::proxy_wars::ThreatMap;
use crate::layer3::diplomacy::succession::Faction;

#[derive(Component)]
pub struct DisposalGate;

#[derive(Event)]
pub struct DumpWasteEvent {
    pub gate: Entity,
    pub amount: f32,
}

pub fn process_dump_waste(
    mut events: EventReader<DumpWasteEvent>,
    resources: Option<ResMut<ColonyResources>>,
    threat_map: Option<ResMut<ThreatMap>>,
    gates: Query<&DisposalGate>,
    factions: Query<Entity, With<Faction>>,
) {
    if let (Some(mut res), Some(mut threats)) = (resources, threat_map) {
        for event in events.read() {
            if gates.get(event.gate).is_ok() {
                res.waste = (res.waste - event.amount).max(0.0);

                // Add threat to all factions (could be randomized in REFACTOR)
                for faction_entity in factions.iter() {
                    let current_threat = threats.get_threat(faction_entity);
                    // Add threat based on amount dumped
                    let threat_increase = (event.amount * 0.1) as i32;
                    threats.threats.insert(faction_entity, current_threat + threat_increase);
                }
            }
        }
    }
}
