use bevy::prelude::*;

use crate::layer1::combat::CombatStats;
use crate::layer1::social_stratification::Prestige;
use crate::layer3::diplomacy_reflection::{Civilization, DiplomaticRelations, DiplomaticTraits};
use bevy::utils::HashSet;

#[derive(Component)]
pub struct Discovery {
    pub data_type: String,
    pub value: u32,
}

#[derive(Component)]
pub struct Published;

#[derive(Component, Default)]
pub struct BypassPathfinding;

#[derive(Event)]
pub struct PublishDiscoveryEvent {
    pub discovery: Entity,
}

pub fn process_publication_system(
    mut commands: Commands,
    mut events: EventReader<PublishDiscoveryEvent>,
    query: Query<&Discovery, Without<Published>>,
    mut prestige_query: Query<&mut Prestige>,
    mut diplomatic_relations: Query<&mut DiplomaticRelations>,
    civs: Query<(&Civilization, &DiplomaticTraits)>,
) {
    let mut processed_this_tick = HashSet::new();

    for event in events.read() {
        if !processed_this_tick.insert(event.discovery) {
            continue; // Already processed this event for this entity this tick
        }

        if let Ok(discovery) = query.get(event.discovery) {
            // Add value to prestige
            for mut prestige in prestige_query.iter_mut() {
                prestige.value = prestige.value.saturating_add(discovery.value as u8);
            }

            // Mark as published
            commands.entity(event.discovery).insert(Published);

            // Update relations
            for mut relations in diplomatic_relations.iter_mut() {
                for standing in relations.relations.iter_mut() {
                    // Try to find the matching civ by ID
                    if let Some((_, traits)) = civs.iter().find(|(civ, _)| civ.id == standing.target_id) {
                        if traits.is_pacifist || traits.is_ecological {
                            standing.standing += 10.0;
                        } else if traits.is_warlike || traits.is_barbarian {
                            standing.standing -= 5.0;
                        }
                    }
                }
            }
        }
    }
}

pub fn process_enemy_exploits_system(
    mut commands: Commands,
    mut events: EventReader<PublishDiscoveryEvent>,
    discovery_query: Query<&Discovery, Without<Published>>,
    mut hostile_query: Query<(Entity, &DiplomaticTraits, Option<&mut CombatStats>)>,
) {
    let mut processed_this_tick = HashSet::new();

    for event in events.read() {
        if !processed_this_tick.insert(event.discovery) {
            continue;
        }

        if let Ok(discovery) = discovery_query.get(event.discovery) {
            if discovery.data_type == "ShieldFrequency" {
                for (_, traits, mut stats) in hostile_query.iter_mut() {
                    if traits.is_warlike || traits.is_barbarian {
                        if let Some(ref mut stats) = stats {
                            stats.melee_damage += 10.0;
                        }
                    }
                }
            } else if discovery.data_type == "MapData" {
                for (entity, traits, _) in hostile_query.iter_mut() {
                    if traits.is_warlike || traits.is_barbarian {
                        commands.entity(entity).insert(BypassPathfinding);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer3::diplomacy_reflection::DiplomaticStanding;

    #[test]
    fn test_publishing_discovery_increases_prestige() {
        let mut app = App::new();
        app.world_mut().spawn(Prestige { value: 100 });
        app.add_event::<PublishDiscoveryEvent>();
        app.add_systems(Update, process_publication_system);

        let discovery = app
            .world_mut()
            .spawn(Discovery {
                data_type: "ShieldFrequency".to_string(),
                value: 50,
            })
            .id();

        app.world_mut()
            .resource_mut::<Events<PublishDiscoveryEvent>>()
            .send(PublishDiscoveryEvent { discovery });

        app.update();

        let mut prestige_query = app.world_mut().query::<&Prestige>();
        let prestige = prestige_query.single(app.world());
        assert_eq!(
            prestige.value, 150,
            "Publishing a discovery should increase the colony's global prestige."
        );
        assert!(
            app.world().get::<Published>(discovery).is_some(),
            "Discovery should be marked as published."
        );
    }

    #[test]
    fn test_publishing_discovery_prevents_duplicate_processing() {
        let mut app = App::new();
        app.world_mut().spawn(Prestige { value: 100 });
        app.add_event::<PublishDiscoveryEvent>();
        app.add_systems(Update, process_publication_system);

        let discovery = app
            .world_mut()
            .spawn(Discovery {
                data_type: "ShieldFrequency".to_string(),
                value: 50,
            })
            .id();

        // Send two events for the same discovery
        app.world_mut()
            .resource_mut::<Events<PublishDiscoveryEvent>>()
            .send(PublishDiscoveryEvent { discovery });
        app.world_mut()
            .resource_mut::<Events<PublishDiscoveryEvent>>()
            .send(PublishDiscoveryEvent { discovery });

        app.update();

        let mut prestige_query = app.world_mut().query::<&Prestige>();
        let prestige = prestige_query.single(app.world());
        // Value should only increase by 50 once
        assert_eq!(
            prestige.value, 150,
            "Publishing a discovery twice in the same frame should only process it once."
        );
    }

    #[test]
    fn test_published_discovery_gives_enemies_combat_bonus() {
        let mut app = App::new();
        app.add_event::<PublishDiscoveryEvent>();
        app.add_systems(Update, process_enemy_exploits_system);

        let discovery = app
            .world_mut()
            .spawn(Discovery {
                data_type: "ShieldFrequency".to_string(),
                value: 50,
            })
            .id();

        // Hostile Faction
        let pirate = app
            .world_mut()
            .spawn((
                DiplomaticTraits {
                    is_warlike: true,
                    is_barbarian: false,
                    is_ecological: false,
                    is_pacifist: false,
                },
                CombatStats {
                    melee_damage: 5.0,
                    damage_types: vec![],
                },
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<PublishDiscoveryEvent>>()
            .send(PublishDiscoveryEvent { discovery });

        app.update();

        let stats = app.world().get::<CombatStats>(pirate).unwrap();
        assert_eq!(
            stats.melee_damage, 15.0,
            "Hostile factions should receive an attack bonus when tactical data is published."
        );
    }

    #[test]
    fn test_published_map_data_gives_bypass_pathfinding() {
        let mut app = App::new();
        app.add_event::<PublishDiscoveryEvent>();
        app.add_systems(Update, process_enemy_exploits_system);

        let discovery = app
            .world_mut()
            .spawn(Discovery {
                data_type: "MapData".to_string(),
                value: 50,
            })
            .id();

        let pirate = app
            .world_mut()
            .spawn((DiplomaticTraits {
                is_warlike: true,
                is_barbarian: false,
                is_ecological: false,
                is_pacifist: false,
            },))
            .id();

        app.world_mut()
            .resource_mut::<Events<PublishDiscoveryEvent>>()
            .send(PublishDiscoveryEvent { discovery });

        app.update();

        assert!(app.world().get::<BypassPathfinding>(pirate).is_some());
    }

    #[test]
    fn test_relations_change_on_publish() {
        let mut app = App::new();
        app.world_mut().spawn(Prestige { value: 100 });
        app.add_event::<PublishDiscoveryEvent>();
        app.add_systems(Update, process_publication_system);

        let discovery = app
            .world_mut()
            .spawn(Discovery {
                data_type: "Biology".to_string(),
                value: 50,
            })
            .id();

        app.world_mut().spawn((
            DiplomaticTraits {
                is_warlike: false,
                is_barbarian: false,
                is_ecological: true,
                is_pacifist: false,
            },
            Civilization { id: "Scholars".to_string() }
        ));

        app.world_mut().spawn((
            DiplomaticTraits {
                is_warlike: true,
                is_barbarian: false,
                is_ecological: false,
                is_pacifist: false,
            },
            Civilization { id: "Pirates".to_string() }
        ));

        let relations_entity = app
            .world_mut()
            .spawn(DiplomaticRelations {
                relations: vec![
                    DiplomaticStanding {
                        target_id: "Scholars".to_string(),
                        standing: 0.0,
                        sanctioned: false,
                    },
                    DiplomaticStanding {
                        target_id: "Pirates".to_string(),
                        standing: 0.0,
                        sanctioned: false,
                    },
                ],
            })
            .id();

        app.world_mut()
            .resource_mut::<Events<PublishDiscoveryEvent>>()
            .send(PublishDiscoveryEvent { discovery });

        app.update();

        let relations = app
            .world()
            .get::<DiplomaticRelations>(relations_entity)
            .unwrap();
        assert_eq!(relations.relations[0].standing, 10.0);
        assert_eq!(relations.relations[1].standing, -5.0);
    }
}
