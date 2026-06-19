use bevy_ecs::prelude::*;
use bevy_app::prelude::*;
use crate::layer1::entities::pop::Pop;

#[derive(Component)]
pub struct Diplomat {
    pub civ_id: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AttireTag {
    Organic,
    HeavyArmor,
    Ceremonial,
}

#[derive(Component)]
pub struct Apparel {
    pub tags: Vec<AttireTag>,
}

#[derive(Component)]
pub struct PreferredAttire {
    pub tags: Vec<AttireTag>,
}

pub struct DiplomaticStanding {
    pub target_id: String,
    pub standing: f32,
    pub sanctioned: bool,
}

#[derive(Component)]
pub struct DiplomaticRelations {
    pub relations: Vec<DiplomaticStanding>,
}

#[derive(Event)]
pub struct DiplomaticMeetingEvent {
    pub ambassador: Entity,
    pub envoy: Entity,
    pub player_civ_id: String,
}

pub fn evaluate_fashion_system(
    mut events: EventReader<DiplomaticMeetingEvent>,
    mut query_ambassador: Query<(&PreferredAttire, &mut DiplomaticRelations)>,
    query_envoy: Query<&Apparel, With<Pop>>,
) {
    for event in events.read() {
        if let Ok((preferred, mut relations)) = query_ambassador.get_mut(event.ambassador) {
            if let Ok(apparel) = query_envoy.get(event.envoy) {
                let mut matched = false;
                for tag in &preferred.tags {
                    if apparel.tags.contains(tag) {
                        matched = true;
                        break;
                    }
                }

                for relation in relations.relations.iter_mut() {
                    if relation.target_id == event.player_civ_id {
                        if matched {
                            relation.standing += 10.0;
                        } else {
                            relation.standing -= 10.0;
                        }
                    }
                }
            }
        }
    }
}

pub struct DiplomaticFashionPlugin;



impl Plugin for DiplomaticFashionPlugin {

    fn build(&self, app: &mut App) {

        app.add_event::<DiplomaticMeetingEvent>();

        app.add_systems(Update, evaluate_fashion_system);

    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diplomatic_fashion_match_provides_bonus() {
        // Arrange: Setup test data
        let mut app = App::new();

        let ambassador_entity = app
            .world_mut()
            .spawn((
                Diplomat {
                    civ_id: "alien_empire".to_string(),
                },
                PreferredAttire {
                    tags: vec![AttireTag::Organic],
                },
                DiplomaticRelations {
                    relations: vec![DiplomaticStanding {
                        target_id: "player".to_string(),
                        standing: 50.0,
                        sanctioned: false,
                    }],
                },
            ))
            .id();

        let envoy_entity = app
            .world_mut()
            .spawn((
                Pop,
                Apparel {
                    tags: vec![AttireTag::Organic, AttireTag::Ceremonial],
                },
            ))
            .id();

        app.world_mut()
            .init_resource::<Events<DiplomaticMeetingEvent>>();
        app.world_mut()
            .resource_mut::<Events<DiplomaticMeetingEvent>>()
            .send(DiplomaticMeetingEvent {
                ambassador: ambassador_entity,
                envoy: envoy_entity,
                player_civ_id: "player".to_string(),
            });

        app.add_systems(Update, evaluate_fashion_system);

        // Act: Call the feature
        app.update();

        // Assert: Verify expected behavior (Standing should increase because attire matches)
        let relations = app
            .world()
            .get::<DiplomaticRelations>(ambassador_entity)
            .unwrap();
        assert!(
            relations.relations[0].standing > 50.0,
            "Standing should increase due to matching attire."
        );
    }

    #[test]
    fn test_diplomatic_fashion_mismatch_causes_penalty() {
        // Arrange
        let mut app = App::new();

        let ambassador_entity = app
            .world_mut()
            .spawn((
                Diplomat {
                    civ_id: "alien_empire".to_string(),
                },
                PreferredAttire {
                    tags: vec![AttireTag::HeavyArmor],
                },
                DiplomaticRelations {
                    relations: vec![DiplomaticStanding {
                        target_id: "player".to_string(),
                        standing: 50.0,
                        sanctioned: false,
                    }],
                },
            ))
            .id();

        let envoy_entity = app
            .world_mut()
            .spawn((
                Pop,
                Apparel {
                    tags: vec![AttireTag::Organic],
                },
            ))
            .id();

        app.world_mut()
            .init_resource::<Events<DiplomaticMeetingEvent>>();
        app.world_mut()
            .resource_mut::<Events<DiplomaticMeetingEvent>>()
            .send(DiplomaticMeetingEvent {
                ambassador: ambassador_entity,
                envoy: envoy_entity,
                player_civ_id: "player".to_string(),
            });

        app.add_systems(Update, evaluate_fashion_system);

        // Act
        app.update();

        // Assert: Verify expected behavior (Standing should decrease because attire does not match)
        let relations = app
            .world()
            .get::<DiplomaticRelations>(ambassador_entity)
            .unwrap();
        assert!(
            relations.relations[0].standing < 50.0,
            "Standing should decrease due to attire mismatch."
        );
    }

    #[test]
    fn test_diplomatic_fashion_ignores_other_factions() {
        // Arrange
        let mut app = App::new();

        let ambassador_entity = app
            .world_mut()
            .spawn((
                Diplomat {
                    civ_id: "alien_empire".to_string(),
                },
                PreferredAttire {
                    tags: vec![AttireTag::HeavyArmor],
                },
                DiplomaticRelations {
                    relations: vec![DiplomaticStanding {
                        target_id: "other_empire".to_string(), // different target_id
                        standing: 50.0,
                        sanctioned: false,
                    }],
                },
            ))
            .id();

        let envoy_entity = app
            .world_mut()
            .spawn((
                Pop,
                Apparel {
                    tags: vec![AttireTag::Organic],
                },
            ))
            .id();

        app.world_mut()
            .init_resource::<Events<DiplomaticMeetingEvent>>();
        app.world_mut()
            .resource_mut::<Events<DiplomaticMeetingEvent>>()
            .send(DiplomaticMeetingEvent {
                ambassador: ambassador_entity,
                envoy: envoy_entity,
                player_civ_id: "player".to_string(), // Meeting is about "player"
            });

        app.add_systems(Update, evaluate_fashion_system);

        // Act
        app.update();

        // Assert: Verify expected behavior (Standing should not change)
        let relations = app
            .world()
            .get::<DiplomaticRelations>(ambassador_entity)
            .unwrap();
        assert_eq!(
            relations.relations[0].standing, 50.0,
            "Standing should not change for other factions."
        );
    }

    #[test]
    fn test_diplomatic_fashion_ignores_missing_ambassador_or_envoy() {
        // Arrange
        let mut app = App::new();

        let ambassador_entity = app.world_mut().spawn(()).id(); // not a diplomat
        let envoy_entity = app.world_mut().spawn(()).id(); // no apparel

        app.world_mut()
            .init_resource::<Events<DiplomaticMeetingEvent>>();
        app.world_mut()
            .resource_mut::<Events<DiplomaticMeetingEvent>>()
            .send(DiplomaticMeetingEvent {
                ambassador: ambassador_entity,
                envoy: envoy_entity,
                player_civ_id: "player".to_string(),
            });

        app.add_systems(Update, evaluate_fashion_system);

        // Act
        app.update();

        // Assert: Verify expected behavior (Nothing panics, coverage increases for skipped logic)
        assert!(app
            .world()
            .get::<DiplomaticRelations>(ambassador_entity)
            .is_none());
    }

    #[test]
    fn test_diplomatic_fashion_missing_envoy_apparel() {
        // Arrange
        let mut app = App::new();

        let ambassador_entity = app
            .world_mut()
            .spawn((
                Diplomat {
                    civ_id: "alien_empire".to_string(),
                },
                PreferredAttire {
                    tags: vec![AttireTag::HeavyArmor],
                },
                DiplomaticRelations {
                    relations: vec![DiplomaticStanding {
                        target_id: "player".to_string(),
                        standing: 50.0,
                        sanctioned: false,
                    }],
                },
            ))
            .id();

        let envoy_entity = app.world_mut().spawn(Pop).id(); // missing apparel

        app.world_mut()
            .init_resource::<Events<DiplomaticMeetingEvent>>();
        app.world_mut()
            .resource_mut::<Events<DiplomaticMeetingEvent>>()
            .send(DiplomaticMeetingEvent {
                ambassador: ambassador_entity,
                envoy: envoy_entity,
                player_civ_id: "player".to_string(),
            });

        app.add_systems(Update, evaluate_fashion_system);

        // Act
        app.update();

        // Assert: Verify expected behavior (Nothing changes because envoy is missing apparel)
        let relations = app
            .world()
            .get::<DiplomaticRelations>(ambassador_entity)
            .unwrap();
        assert_eq!(relations.relations[0].standing, 50.0);
    }
}

#[cfg(test)]
mod plugin_tests {
    use super::*;
    #[test]
    fn test_plugin_build() {
        let mut app = App::new();
        app.add_plugins(DiplomaticFashionPlugin);
    }
}
