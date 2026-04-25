//! Cultural Ransom
//!
//! Systems for stealing cultural artifacts via raids, applying morale penalties when artifacts
//! are held hostage, and trading them back during diplomatic negotiations.
use crate::layer1::social::morale::Morale;
use crate::layer3::diplomacy::proxy_wars::Credits;
use crate::layer3::diplomacy::succession::Faction;
use bevy::prelude::*;

#[derive(Event)]
pub struct RaidEvent {
    pub target_faction: Entity,
    pub raider_faction: Entity,
    pub successful: bool,
    pub is_deep_strike: bool,
}

#[derive(Event)]
pub struct DiplomaticNegotiationEvent {
    pub proposer: Entity,
    pub target: Entity,
    pub offer_artifact_return: bool,
    pub demand_credits: i32,
}

// Feature components
#[derive(Component)]
pub struct CulturalArtifact {
    pub name: String,
    pub original_owner: Entity,
    pub held_by: Entity,
}

// Systems
pub fn process_artifact_raid_system(
    mut events: EventReader<RaidEvent>,
    mut artifacts: Query<&mut CulturalArtifact>,
) {
    for event in events.read() {
        if event.successful && event.is_deep_strike {
            for mut artifact in artifacts.iter_mut() {
                if artifact.original_owner == event.target_faction
                    && artifact.held_by == event.target_faction
                {
                    artifact.held_by = event.raider_faction; // Stolen!
                }
            }
        }
    }
}

pub fn apply_hostage_penalties_system(
    artifacts: Query<&CulturalArtifact>,
    mut factions_morale: Query<(Entity, &mut Morale), With<Faction>>,
) {
    for artifact in artifacts.iter() {
        if artifact.held_by != artifact.original_owner {
            if let Ok((_, mut morale)) = factions_morale.get_mut(artifact.original_owner) {
                morale.value -= 5.0; // Penalty for lost artifact
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn handle_ransom_negotiation_system(
    mut events: EventReader<DiplomaticNegotiationEvent>,
    mut factions: Query<(Entity, &mut Credits)>,
    mut artifacts: Query<&mut CulturalArtifact>,
) {
    for event in events.read() {
        if event.offer_artifact_return {
            if let Ok([(_, mut target_credits), (_, mut proposer_credits)]) =
                factions.get_many_mut([event.target, event.proposer])
            {
                if target_credits.0 >= event.demand_credits {
                    // Check if proposer holds any artifact of target
                    let mut returned = false;
                    for mut artifact in artifacts.iter_mut() {
                        if artifact.held_by == event.proposer
                            && artifact.original_owner == event.target
                        {
                            artifact.held_by = event.target;
                            returned = true;
                        }
                    }

                    if returned {
                        target_credits.0 -= event.demand_credits;
                        proposer_credits.0 += event.demand_credits;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_steal_cultural_artifact() {
        // Arrange
        let mut app = App::new();
        app.add_event::<RaidEvent>();
        app.add_systems(Update, process_artifact_raid_system);

        let victim_faction = app
            .world_mut()
            .spawn(Faction {
                name: "Victim".to_string(),
            })
            .id();
        app.world_mut()
            .entity_mut(victim_faction)
            .insert(CulturalArtifact {
                name: "Original Charter".to_string(),
                original_owner: victim_faction,
                held_by: victim_faction,
            });

        let raider_faction = app
            .world_mut()
            .spawn(Faction {
                name: "Raider".to_string(),
            })
            .id();

        // Act
        app.world_mut().send_event(RaidEvent {
            target_faction: victim_faction,
            raider_faction,
            successful: true,
            is_deep_strike: true,
        });
        app.update();

        // Assert
        // The artifact's held_by value should now be the raider's faction ID
        let artifact = app.world().get::<CulturalArtifact>(victim_faction).unwrap();
        assert_eq!(
            artifact.held_by, raider_faction,
            "Artifact should be stolen by raider."
        );
    }

    #[test]
    fn test_artifact_hostage_penalties() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, apply_hostage_penalties_system);

        let raider_faction = app
            .world_mut()
            .spawn(Faction {
                name: "Raider".to_string(),
            })
            .id();

        let victim_faction = app
            .world_mut()
            .spawn((
                Faction {
                    name: "Victim".to_string(),
                },
                Morale {
                    value: 100.0,
                    modifiers: vec![],
                },
                CulturalArtifact {
                    name: "Original Charter".to_string(),
                    original_owner: raider_faction,
                    held_by: raider_faction,
                },
            ))
            .id();

        let mut artifact = app
            .world_mut()
            .get_mut::<CulturalArtifact>(victim_faction)
            .unwrap();
        artifact.original_owner = victim_faction;
        // held_by is raider_faction (hostage)

        // Act
        app.update();

        // Assert
        // Victim's morale should be heavily penalized
        let morale = app.world().get::<Morale>(victim_faction).unwrap();
        assert!(
            morale.value < 100.0,
            "Morale should decrease when an artifact is held hostage."
        );
    }

    #[test]
    fn test_trade_artifact_for_concessions() {
        // Arrange
        let mut app = App::new();
        app.add_event::<DiplomaticNegotiationEvent>();
        app.add_systems(Update, handle_ransom_negotiation_system);

        let raider_faction = app
            .world_mut()
            .spawn((
                Faction {
                    name: "Raider".to_string(),
                },
                Credits(0),
            ))
            .id();

        let victim_faction = app
            .world_mut()
            .spawn((
                Faction {
                    name: "Victim".to_string(),
                },
                Credits(1000),
                CulturalArtifact {
                    name: "Original Charter".to_string(),
                    original_owner: raider_faction,
                    held_by: raider_faction,
                },
            ))
            .id();
        let mut artifact = app
            .world_mut()
            .get_mut::<CulturalArtifact>(victim_faction)
            .unwrap();
        artifact.original_owner = victim_faction;

        // Act
        app.world_mut().send_event(DiplomaticNegotiationEvent {
            proposer: raider_faction,
            target: victim_faction,
            offer_artifact_return: true,
            demand_credits: 500,
        });
        app.update();

        // Assert
        let victim_pool = app.world().get::<Credits>(victim_faction).unwrap();
        let raider_pool = app.world().get::<Credits>(raider_faction).unwrap();
        let artifact = app.world().get::<CulturalArtifact>(victim_faction).unwrap();

        assert_eq!(victim_pool.0, 500, "Victim should have paid the ransom.");
        assert_eq!(
            raider_pool.0, 500,
            "Raider should have received the ransom."
        );
        assert_eq!(
            artifact.held_by, victim_faction,
            "Artifact should be returned to victim."
        );
    }
}
