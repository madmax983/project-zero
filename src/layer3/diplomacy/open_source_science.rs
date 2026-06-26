use bevy_ecs::prelude::*;
use crate::layer1::research::Discovery;
use crate::layer3::diplomacy::succession::Faction;
use crate::layer1::rearguard::CombatStats;

#[derive(Event)]
pub struct PublishDiscoveryEvent {
    pub discovery: Entity,
}

#[derive(Resource, Default)]
pub struct GlobalPrestige {
    pub value: u32,
}

pub fn process_publication_system(
    mut events: EventReader<PublishDiscoveryEvent>,
    query: Query<&Discovery>,
    mut prestige: ResMut<GlobalPrestige>,
) {
    for event in events.read() {
        if let Ok(discovery) = query.get(event.discovery) {
            prestige.value += discovery.value;
        }
    }
}

pub fn process_enemy_exploits_system(
    mut events: EventReader<PublishDiscoveryEvent>,
    discovery_query: Query<&Discovery>,
    mut hostile_query: Query<(&Faction, &mut CombatStats)>,
) {
    for event in events.read() {
        if let Ok(discovery) = discovery_query.get(event.discovery) {
            if discovery.data_type == "ShieldFrequency" || discovery.data_type == "MapData" {
                for (_faction, mut stats) in hostile_query.iter_mut() {
                    // For MVP we just buff all hostile factions if they have CombatStats
                    stats.attack_bonus += 10.0;
                }
            }
        }
    }
}
